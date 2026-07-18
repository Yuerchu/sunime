use std::io::{self, BufRead, BufReader};
use std::path::Path;

use crate::build::DictBuilder;

pub fn build_from_tsv(
    tsv_path: &Path,
    output_dir: &Path,
) -> io::Result<crate::build::Stats> {
    let file = std::fs::File::open(tsv_path)?;
    let reader = BufReader::new(file);
    let mut builder = DictBuilder::new();

    for line in reader.lines() {
        let line = line?;
        let line = line.trim();
        if line.is_empty() || line.starts_with('#') {
            continue;
        }

        let parts: Vec<&str> = line.splitn(3, '\t').collect();
        if parts.len() < 2 {
            continue;
        }

        let code = parts[0];
        let text = parts[1];
        let freq: u32 = parts.get(2).and_then(|s| s.parse().ok()).unwrap_or(0);

        builder.add(code, text, freq);
    }

    builder.build(output_dir)
}
