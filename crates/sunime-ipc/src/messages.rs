use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(tag = "method", content = "params")]
pub enum Request {
    Activate,
    Deactivate,
    OnTestKeyDown(KeyEventParams),
    OnKeyDown(KeyEventParams),
    OnTestKeyUp(KeyEventParams),
    OnKeyUp(KeyEventParams),
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct KeyEventParams {
    pub vk: u32,
    pub scancode: u32,
    pub shift: bool,
    pub ctrl: bool,
    pub alt: bool,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(tag = "type")]
pub enum Response {
    Ok,
    TestKeyReply {
        handled: bool,
    },
    KeyReply {
        handled: bool,
        #[serde(skip_serializing_if = "Option::is_none")]
        commit: Option<String>,
        #[serde(skip_serializing_if = "Option::is_none")]
        preedit: Option<String>,
        #[serde(skip_serializing_if = "Option::is_none")]
        candidates: Option<Vec<CandidateItem>>,
    },
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct CandidateItem {
    pub text: String,
    pub confidence: u32,
}

pub const NUL: u8 = 0x00;

pub fn encode_message<T: Serialize>(msg: &T) -> Vec<u8> {
    let mut buf = serde_json::to_vec(msg).expect("serialize failed");
    buf.push(NUL);
    buf
}

pub fn decode_message<'a, T: Deserialize<'a>>(buf: &'a [u8]) -> Option<T> {
    let end = buf.iter().position(|&b| b == NUL).unwrap_or(buf.len());
    serde_json::from_slice(&buf[..end]).ok()
}
