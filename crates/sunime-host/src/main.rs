mod engine;
mod server;

use std::path::PathBuf;

fn main() {
    let args: Vec<String> = std::env::args().collect();
    let dict_dir = args.get(1).map(PathBuf::from).unwrap_or_else(|| {
        let exe_dir = std::env::current_exe()
            .ok()
            .and_then(|p| p.parent().map(|p| p.to_path_buf()))
            .unwrap_or_else(|| PathBuf::from("."));
        exe_dir.join("dict")
    });

    println!("SunIME Host");
    println!("  Dict: {}", dict_dir.display());
    println!("  Addr: {}", sunime_ipc::pipe::addr());

    let dict = sunime_dict::reader::Dict::open(&dict_dir).unwrap_or_else(|e| {
        eprintln!("Failed to open dictionary: {e}");
        std::process::exit(1);
    });
    println!("  Dictionary loaded");

    let ime_engine = sunime_core::Engine::new(dict);
    server::run(ime_engine);
}
