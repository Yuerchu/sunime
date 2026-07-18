mod compose;
mod pinyin;

use sunime_dict::Candidate;
use sunime_dict::reader::Dict;

pub use pinyin::segment;

pub struct Engine {
    dict: Dict,
}

impl Engine {
    pub fn new(dict: Dict) -> Self {
        Self { dict }
    }

    pub fn lookup(&self, input: &str) -> Vec<Candidate> {
        compose::compose(input, &self.dict, 9)
    }
}
