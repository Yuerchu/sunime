fn main() {
    let args: Vec<String> = std::env::args().collect();
    match args.get(1).map(|s| s.as_str()) {
        Some("--register") | Some("-r") => {
            println!("TODO: Register SunIME TSF");
        }
        Some("--unregister") | Some("-u") => {
            println!("TODO: Unregister SunIME TSF");
        }
        _ => {
            eprintln!("Usage:");
            eprintln!("  sunime-reg --register     Register SunIME as system IME");
            eprintln!("  sunime-reg --unregister   Remove SunIME from system");
        }
    }
}
