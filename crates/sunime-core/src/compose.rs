use sunime_dict::Candidate;
use sunime_dict::reader::Dict;

use crate::pinyin::{SyllableDAG, build_syllable_dag};

const MAX_WORD_SYLLABLES: usize = 8;
const BEAM_SIZE: usize = 5;
const WORD_COUNT_PENALTY: f64 = 100.0;
const MAX_ALTS_PER_WORD: usize = 3;

struct WordEdge {
    end: usize,
    candidates: Vec<Candidate>,
}

#[derive(Clone)]
struct BeamEntry {
    score: f64,
    prev_pos: usize,
    prev_beam_idx: usize,
    word_edge_idx: usize,
}

pub fn compose(input: &str, dict: &Dict, max_results: usize) -> Vec<Candidate> {
    let input_lower = input.to_lowercase();
    let dag = build_syllable_dag(&input_lower);
    if dag.is_empty() || dag[0].is_empty() {
        return Vec::new();
    }

    let target = input_lower.len();
    let lattice = build_word_lattice(&dag, dict, target);
    let paths = find_best_paths(&lattice, target);
    if paths.is_empty() {
        return Vec::new();
    }

    assemble_candidates(&paths, &lattice, max_results)
}

fn build_word_lattice(dag: &SyllableDAG, dict: &Dict, input_len: usize) -> Vec<Vec<WordEdge>> {
    let mut reachable = vec![false; input_len + 1];
    reachable[0] = true;
    for pos in 0..input_len {
        if !reachable[pos] {
            continue;
        }
        if pos < dag.len() {
            for edge in &dag[pos] {
                reachable[edge.end] = true;
            }
        }
    }

    let mut lattice: Vec<Vec<WordEdge>> = (0..=input_len).map(|_| Vec::new()).collect();

    for start in 0..input_len {
        if !reachable[start] || start >= dag.len() || dag[start].is_empty() {
            continue;
        }

        let mut stack: Vec<(usize, Vec<&str>)> = Vec::new();
        for edge in &dag[start] {
            stack.push((edge.end, vec![edge.syllable.as_str()]));
        }

        while let Some((pos, syllables)) = stack.pop() {
            if syllables.len() > MAX_WORD_SYLLABLES {
                continue;
            }

            let key = syllables.join(" ");
            let candidates = dict.lookup(&key);
            if !candidates.is_empty() {
                lattice[start].push(WordEdge {
                    end: pos,
                    candidates,
                });
            }

            if pos < input_len && syllables.len() < MAX_WORD_SYLLABLES && pos < dag.len() {
                for next_edge in &dag[pos] {
                    let mut extended = syllables.clone();
                    extended.push(next_edge.syllable.as_str());
                    stack.push((next_edge.end, extended));
                }
            }
        }
    }

    lattice
}

fn word_score(candidate: &Candidate) -> f64 {
    if candidate.freq > 0 {
        (candidate.freq as f64).ln()
    } else {
        -10.0
    }
}

fn find_best_paths(lattice: &[Vec<WordEdge>], target: usize) -> Vec<Vec<(usize, usize)>> {
    let mut beam: Vec<Vec<BeamEntry>> = vec![Vec::new(); target + 1];
    beam[0].push(BeamEntry {
        score: 0.0,
        prev_pos: usize::MAX,
        prev_beam_idx: usize::MAX,
        word_edge_idx: usize::MAX,
    });

    for pos in 0..=target {
        if beam[pos].is_empty() || pos >= lattice.len() {
            continue;
        }

        let entries_at_pos = beam[pos].clone();

        for (edge_idx, edge) in lattice[pos].iter().enumerate() {
            if edge.candidates.is_empty() {
                continue;
            }
            let edge_score = word_score(&edge.candidates[0]) - WORD_COUNT_PENALTY;

            for (beam_idx, entry) in entries_at_pos.iter().enumerate() {
                let new_score = entry.score + edge_score;
                let new_entry = BeamEntry {
                    score: new_score,
                    prev_pos: pos,
                    prev_beam_idx: beam_idx,
                    word_edge_idx: edge_idx,
                };

                insert_beam(&mut beam[edge.end], new_entry);
            }
        }
    }

    let mut paths = Vec::new();
    for entry in &beam[target] {
        let path = backtrack(&beam, entry, target);
        paths.push(path);
    }
    paths
}

fn insert_beam(beam: &mut Vec<BeamEntry>, entry: BeamEntry) {
    let pos = beam
        .iter()
        .position(|e| e.score < entry.score)
        .unwrap_or(beam.len());
    beam.insert(pos, entry);
    if beam.len() > BEAM_SIZE {
        beam.pop();
    }
}

fn backtrack(
    beam: &[Vec<BeamEntry>],
    entry: &BeamEntry,
    end_pos: usize,
) -> Vec<(usize, usize)> {
    let mut path = Vec::new();
    let mut current = entry.clone();
    let mut _current_pos = end_pos;

    loop {
        if current.prev_pos == usize::MAX {
            break;
        }
        path.push((current.prev_pos, current.word_edge_idx));
        _current_pos = current.prev_pos;
        current = beam[current.prev_pos][current.prev_beam_idx].clone();
    }

    path.reverse();
    path
}

fn assemble_candidates(
    paths: &[Vec<(usize, usize)>],
    lattice: &[Vec<WordEdge>],
    max_results: usize,
) -> Vec<Candidate> {
    // (text, rank_score, display_freq)
    let mut results: Vec<(String, f64, u32)> = Vec::new();
    let mut seen = std::collections::HashSet::new();

    for path in paths {
        let base: Vec<&Candidate> = path
            .iter()
            .map(|&(pos, edge_idx)| &lattice[pos][edge_idx].candidates[0])
            .collect();

        let base_text: String = base.iter().map(|c| c.text.as_str()).collect();
        let log_freq_sum: f64 = base.iter().map(|c| word_score(c)).sum();
        let rank_score = log_freq_sum - path.len() as f64 * WORD_COUNT_PENALTY;
        let geo_mean = (log_freq_sum / path.len() as f64).exp();
        let display_freq = geo_mean.min(u32::MAX as f64) as u32;

        if seen.insert(base_text.clone()) {
            results.push((base_text, rank_score, display_freq));
        }

        for (word_idx, &(pos, edge_idx)) in path.iter().enumerate() {
            let edge = &lattice[pos][edge_idx];
            for alt_idx in 1..edge.candidates.len().min(MAX_ALTS_PER_WORD) {
                let mut text = String::new();
                let mut alt_log_sum = 0.0f64;
                for (i, &(p, ei)) in path.iter().enumerate() {
                    if i == word_idx {
                        text.push_str(&edge.candidates[alt_idx].text);
                        alt_log_sum += word_score(&edge.candidates[alt_idx]);
                    } else {
                        text.push_str(&lattice[p][ei].candidates[0].text);
                        alt_log_sum += word_score(&lattice[p][ei].candidates[0]);
                    }
                }
                let alt_rank = alt_log_sum - path.len() as f64 * WORD_COUNT_PENALTY;
                let alt_geo = (alt_log_sum / path.len() as f64).exp();
                let alt_display = alt_geo.min(u32::MAX as f64) as u32;

                if seen.insert(text.clone()) {
                    results.push((text, alt_rank, alt_display));
                }
            }
            if results.len() >= max_results * 2 {
                break;
            }
        }
    }

    results.sort_by(|a, b| b.1.partial_cmp(&a.1).unwrap_or(std::cmp::Ordering::Equal));
    results.truncate(max_results);

    if results.is_empty() {
        return Vec::new();
    }

    let top_score = results[0].1;
    results
        .into_iter()
        .map(|(text, rank_score, _)| {
            let diff = top_score - rank_score;
            let confidence = (100.0 * (-diff * 0.1).exp()).round() as u32;
            Candidate {
                text,
                freq: confidence.max(1),
            }
        })
        .collect()
}
