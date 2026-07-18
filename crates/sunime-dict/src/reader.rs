use std::path::Path;

use fst::{Automaton, IntoStreamer, Map};
use memmap2::Mmap;

use crate::entry::{Candidate, RawEntry, ENTRY_SIZE};

pub struct Dict {
    fst: Map<Mmap>,
    entries: Mmap,
    strings: Mmap,
}

impl Dict {
    pub fn open(dir: &Path) -> std::io::Result<Self> {
        let fst_file = std::fs::File::open(dir.join("index.fst"))?;
        let fst_mmap = unsafe { Mmap::map(&fst_file)? };
        let fst = Map::new(fst_mmap)
            .map_err(|e| std::io::Error::new(std::io::ErrorKind::InvalidData, e.to_string()))?;

        let entries_file = std::fs::File::open(dir.join("entries.bin"))?;
        let entries = unsafe { Mmap::map(&entries_file)? };

        let strings_file = std::fs::File::open(dir.join("strings.bin"))?;
        let strings = unsafe { Mmap::map(&strings_file)? };

        Ok(Self {
            fst,
            entries,
            strings,
        })
    }

    pub fn lookup(&self, code: &str) -> Vec<Candidate> {
        let Some(packed) = self.fst.get(code.as_bytes()) else {
            return Vec::new();
        };

        let offset = (packed >> 16) as usize;
        let count = (packed & 0xFFFF) as usize;

        self.read_entries(offset, count)
    }

    pub fn prefix_search(&self, prefix: &str, limit: usize) -> Vec<(String, Vec<Candidate>)> {
        use fst::automaton::Str;
        use fst::Streamer;

        let automaton = Str::new(prefix).starts_with();
        let mut stream = self.fst.search(automaton).into_stream();
        let mut results = Vec::new();

        while let Some((key, packed)) = stream.next() {
            let code = String::from_utf8_lossy(key).into_owned();
            let offset = (packed >> 16) as usize;
            let count = (packed & 0xFFFF) as usize;
            let candidates = self.read_entries(offset, count);
            results.push((code, candidates));
            if results.len() >= limit {
                break;
            }
        }

        results
    }

    fn read_entries(&self, offset: usize, count: usize) -> Vec<Candidate> {
        let mut candidates = Vec::with_capacity(count);
        for i in 0..count {
            let pos = (offset + i) * ENTRY_SIZE;
            if pos + ENTRY_SIZE > self.entries.len() {
                break;
            }
            let buf: [u8; ENTRY_SIZE] =
                self.entries[pos..pos + ENTRY_SIZE].try_into().unwrap();
            let entry = RawEntry::from_bytes(&buf);

            let str_start = entry.string_offset as usize;
            let str_end = str_start + entry.string_len as usize;
            if str_end > self.strings.len() {
                break;
            }
            let text = String::from_utf8_lossy(&self.strings[str_start..str_end]).into_owned();

            candidates.push(Candidate {
                text,
                freq: entry.freq,
            });
        }
        candidates
    }
}
