use std::collections::HashSet;
use std::sync::LazyLock;

static SYLLABLES: LazyLock<HashSet<&'static str>> = LazyLock::new(|| {
    include_str!("pinyin_table.txt")
        .lines()
        .filter(|l| !l.is_empty())
        .collect()
});

pub fn segment(input: &str) -> Vec<String> {
    let input = input.to_lowercase();
    let mut results = Vec::new();
    if do_segment(&input, 0, &mut results) {
        results
    } else {
        Vec::new()
    }
}

fn do_segment(input: &str, start: usize, result: &mut Vec<String>) -> bool {
    if start >= input.len() {
        return true;
    }

    let remaining = &input[start..];
    let max_len = remaining.len().min(6);

    for len in (1..=max_len).rev() {
        let candidate = &remaining[..len];
        if SYLLABLES.contains(candidate) {
            result.push(candidate.to_string());
            if do_segment(input, start + len, result) {
                return true;
            }
            result.pop();
        }
    }

    false
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn basic_segmentation() {
        assert_eq!(segment("nihao"), vec!["ni", "hao"]);
        assert_eq!(segment("zhongguo"), vec!["zhong", "guo"]);
        assert_eq!(segment("zhongguoren"), vec!["zhong", "guo", "ren"]);
    }

    #[test]
    fn single_syllable() {
        assert_eq!(segment("wo"), vec!["wo"]);
        assert_eq!(segment("zhuang"), vec!["zhuang"]);
    }

    #[test]
    fn invalid_input() {
        assert!(segment("xyz").is_empty());
    }
}
