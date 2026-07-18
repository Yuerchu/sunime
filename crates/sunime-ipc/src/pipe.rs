use std::io::{self, Read, Write};

use crate::messages::NUL;

pub const PIPE_NAME_PREFIX: &str = r"\\.\pipe\sunime.";
pub const PIPE_SUFFIX: &str = ".pipe";
pub const BUF_SIZE: usize = 65536;

pub fn pipe_name() -> String {
    let username = std::env::var("USERNAME").unwrap_or_else(|_| "default".into());
    let hash = fnv1a_32(username.as_bytes());
    format!("{PIPE_NAME_PREFIX}{hash:08x}{PIPE_SUFFIX}")
}

fn fnv1a_32(data: &[u8]) -> u32 {
    let mut hash: u32 = 0x811c9dc5;
    for &byte in data {
        hash ^= byte as u32;
        hash = hash.wrapping_mul(0x01000193);
    }
    hash
}

pub fn write_message(writer: &mut impl Write, data: &[u8]) -> io::Result<()> {
    writer.write_all(data)?;
    writer.flush()
}

pub fn read_message(reader: &mut impl Read, buf: &mut Vec<u8>) -> io::Result<usize> {
    buf.clear();
    let mut byte = [0u8; 1];
    loop {
        match reader.read(&mut byte) {
            Ok(0) => return Err(io::Error::new(io::ErrorKind::UnexpectedEof, "pipe closed")),
            Ok(_) => {
                if byte[0] == NUL {
                    return Ok(buf.len());
                }
                buf.push(byte[0]);
                if buf.len() >= BUF_SIZE {
                    return Err(io::Error::new(io::ErrorKind::InvalidData, "message too large"));
                }
            }
            Err(e) => return Err(e),
        }
    }
}
