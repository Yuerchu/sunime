use std::path::PathBuf;
use std::time::Instant;

fn main() {
    let args: Vec<String> = std::env::args().collect();

    match args.get(1).map(|s| s.as_str()) {
        Some("build") => cmd_build(&args[2..]),
        Some("repl") => cmd_repl(&args[2..]),
        _ => {
            eprintln!("Usage:");
            eprintln!("  sunime build <input.tsv> <output_dir>");
            eprintln!("  sunime repl <dict_dir>");
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
