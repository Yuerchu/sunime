use std::io::{Write, stdout};
use std::path::PathBuf;
use std::time::Instant;

use crossterm::{
    cursor, execute,
    event::{self, Event, KeyCode, KeyEvent, KeyEventKind, KeyModifiers},
    style::Stylize,
    terminal::{self, ClearType},
};

fn main() {
    let args: Vec<String> = std::env::args().collect();

    match args.get(1).map(|s| s.as_str()) {
        Some("build") => cmd_build(&args[2..]),
        Some("repl") => cmd_repl(&args[2..]),
        Some("ime") => cmd_ime(&args[2..]),
        _ => {
            eprintln!("Usage:");
            eprintln!("  sunime build <input.tsv> <output_dir>");
            eprintln!("  sunime repl <dict_dir>     (line-based, legacy)");
            eprintln!("  sunime ime <dict_dir>      (interactive, keystroke-by-keystroke)");
            std::process::exit(1);
        }
    }
}

fn cmd_build(args: &[String]) {
    if args.len() < 2 {
        eprintln!("Usage: sunime build <input.tsv> <output_dir>");
        std::process::exit(1);
    }
    let tsv_path = PathBuf::from(&args[0]);
    let output_dir = PathBuf::from(&args[1]);

    let start = Instant::now();
    match sunime_dict::migrate::build_from_tsv(&tsv_path, &output_dir) {
        Ok(stats) => {
            let elapsed = start.elapsed();
            println!("Built dictionary in {:.2?}", elapsed);
            println!("  codes:   {}", stats.codes);
            println!("  entries: {}", stats.entries);
            println!("  FST:     {:.1} KB", stats.fst_size as f64 / 1024.0);
            println!("  entries: {:.1} KB", stats.entries_size as f64 / 1024.0);
            println!("  strings: {:.1} KB", stats.strings_size as f64 / 1024.0);
        }
        Err(e) => {
            eprintln!("Error: {e}");
            std::process::exit(1);
        }
    }
}

fn cmd_repl(args: &[String]) {
    if args.is_empty() {
        eprintln!("Usage: sunime repl <dict_dir>");
        std::process::exit(1);
    }
    let dict_dir = PathBuf::from(&args[0]);

    let start = Instant::now();
    let dict = sunime_dict::reader::Dict::open(&dict_dir).unwrap_or_else(|e| {
        eprintln!("Failed to open dictionary: {e}");
        std::process::exit(1);
    });
    println!("Dictionary loaded in {:.2?}", start.elapsed());

    let engine = sunime_core::Engine::new(dict);

    let mut rl = rustyline::DefaultEditor::new().unwrap();
    println!("SunIME REPL. Type pinyin to lookup. Ctrl+C to exit.");

    loop {
        let Ok(line) = rl.readline("> ") else { break };
        let input = line.trim();
        if input.is_empty() {
            continue;
        }

        let start = Instant::now();
        let candidates = engine.lookup(input);
        let elapsed = start.elapsed();

        if candidates.is_empty() {
            println!("  (no candidates)  [{:.3?}]", elapsed);
        } else {
            for (i, c) in candidates.iter().take(9).enumerate() {
                println!("  {} {}  (freq: {})", i + 1, c.text, c.freq);
            }
            println!("  [{:.3?}, {} candidates]", elapsed, candidates.len());
        }

        let _ = rl.add_history_entry(input);
    }
}

fn cmd_ime(args: &[String]) {
    if args.is_empty() {
        eprintln!("Usage: sunime ime <dict_dir>");
        std::process::exit(1);
    }
    let dict_dir = PathBuf::from(&args[0]);

    let dict = sunime_dict::reader::Dict::open(&dict_dir).unwrap_or_else(|e| {
        eprintln!("Failed to open dictionary: {e}");
        std::process::exit(1);
    });
    let engine = sunime_core::Engine::new(dict);

    terminal::enable_raw_mode().unwrap();
    let mut out = stdout();
    execute!(out, terminal::Clear(ClearType::All), cursor::MoveTo(0, 0)).unwrap();

    let mut committed = String::new();
    let mut input_buf = String::new();
    let mut candidates: Vec<sunime_dict::Candidate> = Vec::new();
    let mut last_elapsed = std::time::Duration::ZERO;

    draw(&mut out, &committed, &input_buf, &candidates, last_elapsed);

    loop {
        if !event::poll(std::time::Duration::from_millis(500)).unwrap() {
            continue;
        }

        let Event::Key(KeyEvent { code, modifiers, kind, .. }) = event::read().unwrap() else {
            continue;
        };
        if kind != KeyEventKind::Press {
            continue;
        }

        if modifiers.contains(KeyModifiers::CONTROL) && code == KeyCode::Char('c') {
            break;
        }

        match code {
            KeyCode::Char(c) if c.is_ascii_alphabetic() => {
                input_buf.push(c.to_ascii_lowercase());
                let start = Instant::now();
                candidates = engine.lookup(&input_buf);
                last_elapsed = start.elapsed();
            }

            KeyCode::Char(c) if c.is_ascii_digit() && !input_buf.is_empty() => {
                let idx = c.to_digit(10).unwrap_or(0) as usize;
                let idx = if idx == 0 { 9 } else { idx - 1 };
                if idx < candidates.len() {
                    committed.push_str(&candidates[idx].text);
                    input_buf.clear();
                    candidates.clear();
                    last_elapsed = std::time::Duration::ZERO;
                }
            }

            KeyCode::Backspace => {
                if input_buf.pop().is_some() {
                    if input_buf.is_empty() {
                        candidates.clear();
                        last_elapsed = std::time::Duration::ZERO;
                    } else {
                        let start = Instant::now();
                        candidates = engine.lookup(&input_buf);
                        last_elapsed = start.elapsed();
                    }
                } else {
                    committed.pop();
                }
            }

            KeyCode::Esc => {
                if !input_buf.is_empty() {
                    input_buf.clear();
                    candidates.clear();
                    last_elapsed = std::time::Duration::ZERO;
                } else {
                    committed.clear();
                }
            }

            KeyCode::Enter => {
                if !input_buf.is_empty() {
                    if let Some(c) = candidates.first() {
                        committed.push_str(&c.text);
                    }
                    input_buf.clear();
                    candidates.clear();
                    last_elapsed = std::time::Duration::ZERO;
                } else {
                    committed.push('\n');
                }
            }

            KeyCode::Char(' ') => {
                if !input_buf.is_empty() {
                    if let Some(c) = candidates.first() {
                        committed.push_str(&c.text);
                    }
                    input_buf.clear();
                    candidates.clear();
                    last_elapsed = std::time::Duration::ZERO;
                } else {
                    committed.push(' ');
                }
            }

            _ => {}
        }

        draw(&mut out, &committed, &input_buf, &candidates, last_elapsed);
    }

    terminal::disable_raw_mode().unwrap();
    execute!(out, cursor::Show, terminal::Clear(ClearType::All), cursor::MoveTo(0, 0)).unwrap();
    if !committed.is_empty() {
        println!("{committed}");
    }
}

fn draw(
    out: &mut impl Write,
    committed: &str,
    input_buf: &str,
    candidates: &[sunime_dict::Candidate],
    elapsed: std::time::Duration,
) {
    execute!(out, terminal::Clear(ClearType::All), cursor::MoveTo(0, 0)).unwrap();

    write!(out, "{}", "SunIME".dark_cyan()).unwrap();
    if !elapsed.is_zero() {
        write!(out, "  {}", format!("[{:.0?}]", elapsed).dark_grey()).unwrap();
    }
    writeln!(out, "\r").unwrap();

    let display_committed = if committed.is_empty() {
        "(empty)".dark_grey().to_string()
    } else {
        committed.to_string()
    };
    writeln!(out, "  {display_committed}\r").unwrap();

    if !input_buf.is_empty() {
        writeln!(out, "  > {}\r", input_buf.yellow()).unwrap();
    }

    if !candidates.is_empty() {
        writeln!(out, "\r").unwrap();
        for (i, c) in candidates.iter().take(9).enumerate() {
            let num = if i + 1 == 10 { 0 } else { i + 1 };
            writeln!(
                out,
                "  {} {}  {}\r",
                format!("{num}").dark_cyan(),
                c.text,
                format!("{}", c.freq).dark_grey(),
            )
            .unwrap();
        }
    }

    writeln!(out, "\r").unwrap();
    write!(
        out,
        "{}",
        "Esc: clear | Space/Enter: commit first | 1-9: select | Ctrl+C: quit"
            .dark_grey()
    )
    .unwrap();
    out.flush().unwrap();
}
