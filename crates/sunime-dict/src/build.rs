use std::collections::BTreeMap;
use std::fs::File;
use std::io::{BufWriter, Write};
use std::path::Path;

use fst::MapBuilder;

use crate::entry::RawEntry;

pub struct DictBuilder {
    codes: BTreeMap<String, Vec<(String, u32)>>,
}

impl DictBuilder {
    pub fn new() -> Self {
        Self {
            codes: BTreeMap::new(),
        }
    }

    pub fn add(&mut self, code: &str, text: &str, freq: u32) {
        self.codes
            .entry(code.to_string())
            .or_default()
            .push((text.to_owned(), freq));
    }

    pub fn build(&self, dir: &Path) -> std::io::Result<Stats> {
        std::fs::create_dir_all(dir)?;

        let mut strings_buf: Vec<u8> = Vec::new();
        let mut entries_buf: Vec<u8> = Vec::new();
        let mut fst_builder = MapBuilder::memory();

        let mut entry_offset: u64 = 0;
        let mut total_entries: u64 = 0;

        for (code, candidates) in &self.codes {
            let count = candidates.len() as u64;
            let packed = (entry_offset << 16) | count;
            fst_builder.insert(code.as_bytes(), packed).map_err(|e| {
                std::io::Error::new(std::io::ErrorKind::InvalidData, e.to_string())
            })?;

            let mut sorted = candidates.clone();
            sorted.sort_by(|a, b| b.1.cmp(&a.1));

            for (text, freq) in &sorted {
                let text_bytes = text.as_bytes();
                let string_offset = strings_buf.len() as u32;
                let string_len = text_bytes.len() as u16;
                strings_buf.extend_from_slice(text_bytes);

                let entry = RawEntry {
                    string_offset,
                    string_len,
                    freq: *freq,
                };
                entries_buf.extend_from_slice(&entry.to_bytes());
                total_entries += 1;
            }

            entry_offset += count;
        }

        let fst_bytes = fst_builder.into_inner().map_err(|e| {
            std::io::Error::new(std::io::ErrorKind::Other, e.to_string())
        })?;

        let mut f = BufWriter::new(File::create(dir.join("index.fst"))?);
        f.write_all(&fst_bytes)?;
        f.flush()?;

        let mut f = BufWriter::new(File::create(dir.join("entries.bin"))?);
        f.write_all(&entries_buf)?;
        f.flush()?;

        let mut f = BufWriter::new(File::create(dir.join("strings.bin"))?);
        f.write_all(&strings_buf)?;
        f.flush()?;

        Ok(Stats {
            codes: self.codes.len() as u64,
            entries: total_entries,
            fst_size: fst_bytes.len() as u64,
            entries_size: entries_buf.len() as u64,
            strings_size: strings_buf.len() as u64,
        })
    }
}

pub struct Stats {
    pub codes: u64,
    pub entries: u64,
    pub fst_size: u64,
    pub entries_size: u64,
    pub strings_size: u64,
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::reader::Dict;

    #[test]
    fn build_and_read() {
        let dir = std::env::temp_dir().join("sunime_test_dict");
        let _ = std::fs::remove_dir_all(&dir);

        let mut builder = DictBuilder::new();
        builder.add("ni", "你", 100);
        builder.add("ni", "泥", 50);
        builder.add("ni", "逆", 30);
        builder.add("nihao", "你好", 200);
        builder.add("zhongguo", "中国", 500);

        let stats = builder.build(&dir).unwrap();
        assert_eq!(stats.codes, 3);
        assert_eq!(stats.entries, 5);

        let dict = Dict::open(&dir).unwrap();

        let results = dict.lookup("ni");
        assert_eq!(results.len(), 3);
        assert_eq!(results[0].text, "你");
        assert_eq!(results[0].freq, 100);

        let results = dict.lookup("nihao");
        assert_eq!(results.len(), 1);
        assert_eq!(results[0].text, "你好");

        let results = dict.lookup("nonexistent");
        assert!(results.is_empty());

        let _ = std::fs::remove_dir_all(&dir);
    }
}
