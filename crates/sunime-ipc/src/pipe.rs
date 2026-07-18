use std::io::{self, Read, Write};

use interprocess::os::windows::named_pipe::{
    pipe_mode::Bytes, DuplexPipeStream, PipeListenerOptions, PipeMode,
};

use crate::messages::{NUL, Request, Response, decode_message, encode_message};

pub fn pipe_path() -> String {
    r"\\.\pipe\sunime".to_string()
}

pub struct IpcServer {
    listener: interprocess::os::windows::named_pipe::PipeListener<Bytes, Bytes>,
}

impl IpcServer {
    pub fn bind() -> io::Result<Self> {
        let path = pipe_path();
        let listener = PipeListenerOptions::new()
            .path(path.as_str())
            .mode(PipeMode::Bytes)
            .create_duplex::<Bytes>()?;
        Ok(Self { listener })
    }

    pub fn accept(&self) -> io::Result<IpcConnection> {
        let stream = self.listener.accept()?;
        Ok(IpcConnection { stream })
    }
}

pub struct IpcConnection {
    stream: DuplexPipeStream<Bytes>,
}

impl IpcConnection {
    pub fn read_request(&mut self) -> io::Result<Request> {
        let buf = read_until_nul(&mut self.stream)?;
        decode_message::<Request>(&buf)
            .ok_or_else(|| io::Error::new(io::ErrorKind::InvalidData, "bad request"))
    }

    pub fn write_response(&mut self, response: &Response) -> io::Result<()> {
        let data = encode_message(response);
        self.stream.write_all(&data)?;
        self.stream.flush()
    }
}

pub struct IpcClient;

impl IpcClient {
    pub fn new() -> Self {
        Self
    }

    pub fn send(&self, request: &Request) -> Option<Response> {
        let mut stream = DuplexPipeStream::<Bytes>::connect_by_path(pipe_path()).ok()?;
        let data = encode_message(request);
        stream.write_all(&data).ok()?;
        stream.flush().ok()?;
        let buf = read_until_nul(&mut stream).ok()?;
        decode_message::<Response>(&buf)
    }
}

fn read_until_nul(reader: &mut impl Read) -> io::Result<Vec<u8>> {
    let mut buf = Vec::new();
    let mut byte = [0u8; 1];
    loop {
        match reader.read(&mut byte) {
            Ok(0) => return Err(io::Error::new(io::ErrorKind::UnexpectedEof, "closed")),
            Ok(_) => {
                if byte[0] == NUL {
                    return Ok(buf);
                }
                buf.push(byte[0]);
                if buf.len() >= 65536 {
                    return Err(io::Error::new(io::ErrorKind::InvalidData, "too large"));
                }
            }
            Err(e) => return Err(e),
        }
    }
}
