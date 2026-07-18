use std::io::{self, Read, Write, BufReader, BufRead};
use std::net::{TcpListener, TcpStream};

use crate::messages::{NUL, Request, Response, encode_message, decode_message};

pub const DEFAULT_PORT: u16 = 23981;

pub fn addr() -> String {
    format!("127.0.0.1:{DEFAULT_PORT}")
}

pub struct IpcServer {
    listener: TcpListener,
}

impl IpcServer {
    pub fn bind() -> io::Result<Self> {
        let listener = TcpListener::bind(addr())?;
        Ok(Self { listener })
    }

    pub fn accept(&self) -> io::Result<IpcConnection> {
        let (stream, _) = self.listener.accept()?;
        stream.set_nodelay(true)?;
        Ok(IpcConnection { stream })
    }

    pub fn addr(&self) -> String {
        addr()
    }
}

pub struct IpcConnection {
    stream: TcpStream,
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

pub struct IpcClient {
    addr: String,
}

impl IpcClient {
    pub fn new() -> Self {
        Self { addr: addr() }
    }

    pub fn send(&self, request: &Request) -> Option<Response> {
        let mut stream = TcpStream::connect(&self.addr).ok()?;
        stream.set_nodelay(true).ok()?;

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
