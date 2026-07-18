use sunime_core::Engine;
use sunime_ipc::{CandidateItem, KeyEventParams, Response};

pub struct ImeState {
    engine: Engine,
    input_buf: String,
    candidates: Vec<sunime_dict::Candidate>,
}

impl ImeState {
    pub fn new(engine: Engine) -> Self {
        Self {
            engine,
            input_buf: String::new(),
            candidates: Vec::new(),
        }
    }

    pub fn on_test_key_down(&self, params: &KeyEventParams) -> bool {
        let vk = params.vk;
        match vk {
            0x41..=0x5A => true,
            0x30..=0x39 if !self.input_buf.is_empty() => true,
            0x08 => !self.input_buf.is_empty(),
            0x0D | 0x20 => !self.input_buf.is_empty(),
            0x1B => !self.input_buf.is_empty(),
            _ => false,
        }
    }

    pub fn on_key_down(&mut self, params: &KeyEventParams) -> Response {
        let vk = params.vk;

        match vk {
            0x41..=0x5A => {
                let ch = if params.shift {
                    (vk as u8) as char
                } else {
                    (vk as u8 + 32) as char
                };
                self.input_buf.push(ch);
                self.update_candidates();
                self.make_reply(false, None)
            }

            0x30..=0x39 if !self.input_buf.is_empty() => {
                let idx = (vk - 0x30) as usize;
                let idx = if idx == 0 { 9 } else { idx - 1 };
                if idx < self.candidates.len() {
                    let text = self.candidates[idx].text.clone();
                    self.input_buf.clear();
                    self.candidates.clear();
                    Response::KeyReply {
                        handled: true,
                        commit: Some(text),
                        preedit: None,
                        candidates: None,
                    }
                } else {
                    self.make_reply(true, None)
                }
            }

            0x08 => {
                self.input_buf.pop();
                if self.input_buf.is_empty() {
                    self.candidates.clear();
                    Response::KeyReply {
                        handled: true,
                        commit: None,
                        preedit: None,
                        candidates: None,
                    }
                } else {
                    self.update_candidates();
                    self.make_reply(false, None)
                }
            }

            0x0D | 0x20 => {
                if !self.input_buf.is_empty() {
                    let commit = self.candidates.first().map(|c| c.text.clone());
                    self.input_buf.clear();
                    self.candidates.clear();
                    Response::KeyReply {
                        handled: true,
                        commit,
                        preedit: None,
                        candidates: None,
                    }
                } else {
                    Response::KeyReply {
                        handled: false,
                        commit: None,
                        preedit: None,
                        candidates: None,
                    }
                }
            }

            0x1B => {
                self.input_buf.clear();
                self.candidates.clear();
                Response::KeyReply {
                    handled: true,
                    commit: None,
                    preedit: None,
                    candidates: None,
                }
            }

            _ => Response::KeyReply {
                handled: false,
                commit: None,
                preedit: None,
                candidates: None,
            },
        }
    }

    fn update_candidates(&mut self) {
        self.candidates = self.engine.lookup(&self.input_buf);
    }

    fn make_reply(&self, handled: bool, commit: Option<String>) -> Response {
        let candidates = if self.candidates.is_empty() {
            None
        } else {
            Some(
                self.candidates
                    .iter()
                    .take(9)
                    .map(|c| CandidateItem {
                        text: c.text.clone(),
                        confidence: c.freq,
                    })
                    .collect(),
            )
        };

        let preedit = if self.input_buf.is_empty() {
            None
        } else {
            Some(self.input_buf.clone())
        };

        Response::KeyReply {
            handled: true,
            commit,
            preedit,
            candidates,
        }
    }
}
