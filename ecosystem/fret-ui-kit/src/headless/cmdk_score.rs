//! cmdk-style fuzzy scoring (ported from `repo-ref/cmdk`).
//!
//! This module provides a deterministic, dependency-free scoring function intended for command
//! palettes and searchable lists.
//!
//! - It ports behavior outcomes from `repo-ref/cmdk/cmdk/src/command-score.ts`.
//! - It is ASCII-oriented on purpose (stable indexing + predictable performance).
//! - It returns a score in `[0, 1]` (0 means "no match").

use std::ops::Range;

const SCORE_CONTINUE_MATCH: f32 = 1.0;
const SCORE_SPACE_WORD_JUMP: f32 = 0.9;
const SCORE_NON_SPACE_WORD_JUMP: f32 = 0.8;
const SCORE_CHARACTER_JUMP: f32 = 0.17;
const SCORE_TRANSPOSITION: f32 = 0.1;

const PENALTY_SKIPPED: f32 = 0.999;
const PENALTY_CASE_MISMATCH: f32 = 0.9999;
const PENALTY_NOT_COMPLETE: f32 = 0.99;

#[inline]
fn is_gap(ch: char) -> bool {
    matches!(
        ch,
        '\\' | '/' | '_' | '+' | '.' | '#' | '"' | '@' | '[' | '(' | '{' | '&'
    )
}

#[inline]
fn is_space(ch: char) -> bool {
    ch.is_whitespace() || ch == '-'
}

fn format_input(s: &str) -> Vec<char> {
    s.chars()
        .map(|ch| {
            if is_space(ch) {
                ' '
            } else {
                ch.to_ascii_lowercase()
            }
        })
        .collect()
}

#[inline]
fn pow_penalty(base: f32, exp: usize) -> f32 {
    if exp == 0 {
        return 1.0;
    }
    base.powi(exp as i32)
}

fn find_from(haystack: &[char], needle: char, start: usize) -> Option<usize> {
    haystack
        .iter()
        .copied()
        .enumerate()
        .skip(start)
        .find_map(|(idx, ch)| (ch == needle).then_some(idx))
}

fn count_gaps(chars: &[char]) -> usize {
    chars.iter().copied().filter(|ch| is_gap(*ch)).count()
}

fn count_spaces(chars: &[char]) -> usize {
    chars.iter().copied().filter(|ch| is_space(*ch)).count()
}

fn command_score_inner(
    s: &[char],
    abbr: &[char],
    lower_s: &[char],
    lower_abbr: &[char],
    s_idx: usize,
    abbr_idx: usize,
    memo: &mut [Vec<f32>],
) -> f32 {
    if abbr_idx == abbr.len() {
        if s_idx == s.len() {
            return SCORE_CONTINUE_MATCH;
        }
        return PENALTY_NOT_COMPLETE;
    }

    let cached = memo
        .get(s_idx)
        .and_then(|row| row.get(abbr_idx))
        .copied()
        .unwrap_or(f32::NAN);
    if !cached.is_nan() {
        return cached;
    }

    let Some(&abbr_char) = lower_abbr.get(abbr_idx) else {
        return 0.0;
    };

    let mut high_score = 0.0;
    let mut index = find_from(lower_s, abbr_char, s_idx);
    while let Some(i) = index {
        let mut score =
            command_score_inner(s, abbr, lower_s, lower_abbr, i + 1, abbr_idx + 1, memo);

        if score > high_score {
            if i == s_idx {
                score *= SCORE_CONTINUE_MATCH;
            } else if i > 0 && is_gap(s[i - 1]) {
                score *= SCORE_NON_SPACE_WORD_JUMP;
                if s_idx > 0 {
                    let end = (i.saturating_sub(1)).min(s.len());
                    let start = s_idx.min(end);
                    let breaks = count_gaps(&s[start..end]);
                    score *= pow_penalty(PENALTY_SKIPPED, breaks);
                }
            } else if i > 0 && is_space(s[i - 1]) {
                score *= SCORE_SPACE_WORD_JUMP;
                if s_idx > 0 {
                    let end = (i.saturating_sub(1)).min(s.len());
                    let start = s_idx.min(end);
                    let breaks = count_spaces(&s[start..end]);
                    score *= pow_penalty(PENALTY_SKIPPED, breaks);
                }
            } else {
                score *= SCORE_CHARACTER_JUMP;
                if s_idx > 0 {
                    score *= pow_penalty(PENALTY_SKIPPED, i.saturating_sub(s_idx));
                }
            }

            if s.get(i) != abbr.get(abbr_idx) {
                score *= PENALTY_CASE_MISMATCH;
            }

            // If the user transposed two letters, penalize it strongly (cmdk behavior).
            let next_abbr = lower_abbr.get(abbr_idx + 1).copied();
            let prev_s = i.checked_sub(1).and_then(|j| lower_s.get(j).copied());
            let cur_abbr = lower_abbr.get(abbr_idx).copied();

            let transposition_candidate = (score < SCORE_TRANSPOSITION
                && prev_s.is_some_and(|prev| next_abbr.is_some_and(|next| prev == next)))
                || (next_abbr.is_some_and(|next| cur_abbr.is_some_and(|cur| next == cur))
                    && prev_s.is_some_and(|prev| cur_abbr.is_some_and(|cur| prev != cur)));

            if transposition_candidate {
                let transposed_score =
                    command_score_inner(s, abbr, lower_s, lower_abbr, i + 1, abbr_idx + 2, memo);
                let candidate = transposed_score * SCORE_TRANSPOSITION;
                if candidate > score {
                    score = candidate;
                }
            }

            if score > high_score {
                high_score = score;
            }
        }

        index = find_from(lower_s, abbr_char, i + 1);
    }

    if let Some(row) = memo.get_mut(s_idx)
        && let Some(slot) = row.get_mut(abbr_idx)
    {
        *slot = high_score;
    }
    high_score
}

/// Returns a fuzzy score for `search` within `value`, optionally including `aliases`.
///
/// - Score is in `[0, 1]`.
/// - `0.0` means "no match".
/// - Empty `search` returns `1.0` and is intended to keep original ordering at the caller.
pub fn command_score(value: &str, search: &str, aliases: &[&str]) -> f32 {
    let search = search.trim();
    if search.is_empty() {
        return 1.0;
    }

    let value = if aliases.is_empty() {
        value.to_owned()
    } else {
        let mut s = String::with_capacity(
            value.len() + 1 + aliases.iter().map(|a| a.len() + 1).sum::<usize>(),
        );
        s.push_str(value);
        s.push(' ');
        for (idx, alias) in aliases.iter().enumerate() {
            if idx > 0 {
                s.push(' ');
            }
            s.push_str(alias);
        }
        s
    };

    let s: Vec<char> = value.chars().collect();
    let abbr: Vec<char> = search.chars().collect();
    if s.is_empty() || abbr.is_empty() {
        return 0.0;
    }

    let lower_s = format_input(&value);
    let lower_abbr = format_input(search);

    // Keep memo small and indexing stable: [string_index][abbr_index].
    let mut memo = vec![vec![f32::NAN; abbr.len() + 1]; s.len() + 1];
    command_score_inner(&s, &abbr, &lower_s, &lower_abbr, 0, 0, &mut memo).clamp(0.0, 1.0)
}

/// Returns a best-effort set of match ranges for highlighting `search` within `value`.
///
/// This is intentionally a lightweight, deterministic helper for UI rendering:
/// - It performs a greedy subsequence match over the same normalized (ASCII-oriented) inputs used by
///   [`command_score`].
/// - Returned ranges are **character index ranges** (not byte ranges) into the original `value`.
/// - If no full subsequence match exists, returns an empty list.
///
/// Note: cmdk upstream does not provide "true" best-path match indices for its score; this helper is
/// meant to be "good enough" for match highlighting while preserving stable behavior.
pub fn command_match_ranges(value: &str, search: &str) -> Vec<Range<usize>> {
    let search = search.trim();
    if search.is_empty() {
        return Vec::new();
    }

    let lower_s = format_input(value);
    let lower_abbr = format_input(search);
    if lower_s.is_empty() || lower_abbr.is_empty() {
        return Vec::new();
    }

    let mut matched_indices: Vec<usize> = Vec::with_capacity(lower_abbr.len());
    let mut s_idx: usize = 0;
    for ch in lower_abbr {
        let Some(i) = find_from(&lower_s, ch, s_idx) else {
            return Vec::new();
        };
        matched_indices.push(i);
        s_idx = i.saturating_add(1);
        if s_idx > lower_s.len() {
            s_idx = lower_s.len();
        }
    }

    if matched_indices.is_empty() {
        return Vec::new();
    }

    let mut ranges: Vec<Range<usize>> = Vec::new();
    let mut start = matched_indices[0];
    let mut prev = matched_indices[0];
    for &idx in matched_indices.iter().skip(1) {
        if idx == prev.saturating_add(1) {
            prev = idx;
            continue;
        }
        ranges.push(start..prev.saturating_add(1));
        start = idx;
        prev = idx;
    }
    ranges.push(start..prev.saturating_add(1));
    ranges
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn empty_search_returns_one() {
        assert_eq!(command_score("Open File", "", &[]), 1.0);
        assert_eq!(command_score("Open File", "   ", &[]), 1.0);
    }

    #[test]
    fn exact_contiguous_match_scores_high() {
        let a = command_score("open", "op", &[]);
        let b = command_score("xopen", "op", &[]);
        assert!(a > b);
        assert!(a > 0.0);
    }

    #[test]
    fn word_jumps_score_higher_than_character_jumps() {
        let a = command_score("open-file", "of", &[]);
        let b = command_score("openfile", "of", &[]);
        assert!(a > b);
    }

    #[test]
    fn aliases_participate_in_matching() {
        let no_alias = command_score("Open", "settings", &[]);
        let with_alias = command_score("Open", "settings", &["Settings"]);
        assert_eq!(no_alias, 0.0);
        assert!(with_alias > 0.0);
    }

    #[test]
    fn case_mismatch_is_penalized() {
        let exact = command_score("HTML", "HM", &[]);
        let mismatch = command_score("haml", "HM", &[]);
        assert!(exact > mismatch);
    }

    #[test]
    fn match_ranges_empty_when_search_empty() {
        assert_eq!(
            command_match_ranges("Open File", ""),
            Vec::<Range<usize>>::new()
        );
        assert_eq!(
            command_match_ranges("Open File", "   "),
            Vec::<Range<usize>>::new()
        );
    }

    #[test]
    fn match_ranges_returns_subsequence_character_ranges() {
        // "of" matches non-contiguously in "open file".
        let ranges = command_match_ranges("open file", "of");
        assert_eq!(ranges, vec![0..1, 5..6]);

        // contiguous match is returned as a single range.
        let ranges = command_match_ranges("open file", "open");
        assert_eq!(ranges, vec![0..4]);
    }

    #[test]
    fn match_ranges_normalizes_hyphen_as_space() {
        // cmdk normalization treats '-' as space, so "of" matches "open-file" at the same indices.
        let ranges = command_match_ranges("open-file", "of");
        assert_eq!(ranges, vec![0..1, 5..6]);
    }
}
