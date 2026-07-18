pub const ENTRY_SIZE: usize = 10;

#[repr(C, packed)]
#[derive(Clone, Copy)]
pub struct RawEntry {
    pub string_offset: u32,
    pub string_len: u16,
    pub freq: u32,
}

impl RawEntry {
    pub fn to_bytes(&self) -> [u8; ENTRY_SIZE] {
        let mut buf = [0u8; ENTRY_SIZE];
        buf[0..4].copy_from_slice(&self.string_offset.to_le_bytes());
        buf[4..6].copy_from_slice(&self.string_len.to_le_bytes());
        buf[6..10].copy_from_slice(&self.freq.to_le_bytes());
        buf
    }

    pub fn from_bytes(buf: &[u8; ENTRY_SIZE]) -> Self {
        Self {
            string_offset: u32::from_le_bytes([buf[0], buf[1], buf[2], buf[3]]),
            string_len: u16::from_le_bytes([buf[4], buf[5]]),
            freq: u32::from_le_bytes([buf[6], buf[7], buf[8], buf[9]]),
        }
    }
}

#[derive(Debug, Clone)]
pub struct Candidate {
    pub text: String,
    pub freq: u32,
}
