use interprocess::os::windows::named_pipe::{pipe_mode::Bytes, DuplexPipeStream};

#[test]
fn test_pipe_connection() {
    let path = r"\\.\pipe\sunime";
    println!("Connecting to: {path}");
    match DuplexPipeStream::<Bytes>::connect_by_path(path) {
        Ok(stream) => {
            println!("Connected!");
            drop(stream);
        }
        Err(e) => {
            println!("Connection error: {e}");
            println!("  Kind: {:?}", e.kind());
            if let Some(code) = e.raw_os_error() {
                println!("  OS error: {code}");
            }
            panic!("Failed to connect");
        }
    }
}
