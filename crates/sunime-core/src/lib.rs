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
        let syllables = segment(input);
        if syllables.is_empty() {
            return Vec::new();
        }

        let code = syllables.join(" ");
        let mut results = self.dict.lookup(&code);

        if results.is_empty() && syllables.len() == 1 {
            return results;
        }

        if results.is_empty() {
            for s in &syllables {
                let mut chars = self.dict.lookup(s);
                if !chars.is_empty() {
                    chars.truncate(1);
                    results.extend(chars);
                }
            }
        }

        results
    }
}
