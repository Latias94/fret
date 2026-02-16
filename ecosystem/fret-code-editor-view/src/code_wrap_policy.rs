use unicode_ident::is_xid_continue;

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum CodeWrapPreset {
    Conservative,
    Balanced,
    Aggressive,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct CodeWrapKnobs {
    pub break_after_path_separators: bool,
    pub break_after_url_separators: bool,
    pub break_after_punctuation: bool,
    pub break_at_identifier_boundaries: bool,
    pub break_around_operators: bool,
}

impl CodeWrapKnobs {
    pub fn for_preset(preset: CodeWrapPreset) -> Self {
        match preset {
            CodeWrapPreset::Conservative => Self {
                break_after_path_separators: true,
                break_after_url_separators: true,
                break_after_punctuation: false,
                break_at_identifier_boundaries: false,
                break_around_operators: false,
            },
            CodeWrapPreset::Balanced => Self {
                break_after_path_separators: true,
                break_after_url_separators: true,
                break_after_punctuation: true,
                break_at_identifier_boundaries: true,
                break_around_operators: true,
            },
            CodeWrapPreset::Aggressive => Self {
                break_after_path_separators: true,
                break_after_url_separators: true,
                break_after_punctuation: true,
                break_at_identifier_boundaries: true,
                break_around_operators: true,
            },
        }
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct CodeWrapPolicy {
    pub preset: CodeWrapPreset,
    pub knobs: CodeWrapKnobs,
}

impl CodeWrapPolicy {
    pub fn preset(preset: CodeWrapPreset) -> Self {
        Self {
            preset,
            knobs: CodeWrapKnobs::for_preset(preset),
        }
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord)]
enum BreakStrength {
    Identifier,
    Punctuation,
    Operator,
    Separator,
    Whitespace,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
struct BreakCandidate {
    byte: usize,
    cols: usize,
    strength: BreakStrength,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct CodeWrapRowStart {
    pub byte: usize,
    pub col: usize,
}

pub fn row_starts_for_code_wrap(
    text: &str,
    wrap_cols: usize,
    policy: CodeWrapPolicy,
) -> Vec<CodeWrapRowStart> {
    let wrap_cols = wrap_cols.max(1);
    if text.is_empty() {
        return vec![CodeWrapRowStart { byte: 0, col: 0 }];
    }

    let mut starts: Vec<CodeWrapRowStart> = vec![CodeWrapRowStart { byte: 0, col: 0 }];
    let mut row_start_byte = 0usize;
    let mut row_start_col = 0usize;

    while row_start_byte < text.len() {
        let mut col = 0usize;
        let mut i = row_start_byte;
        let mut prev_ch: Option<char> = None;

        let mut last_grapheme_boundary: Option<(usize, usize)> = Some((row_start_byte, 0));
        let mut last_candidate: Option<BreakCandidate> = None;

        while i < text.len() && col < wrap_cols {
            let ch = text[i..].chars().next().unwrap_or('\0');
            let next_i = i.saturating_add(ch.len_utf8()).min(text.len());
            col = col.saturating_add(1);

            let is_boundary = fret_text_nav::is_grapheme_boundary(text, next_i);
            if is_boundary {
                last_grapheme_boundary = Some((next_i, col));
            }

            let next_ch = text.get(next_i..).and_then(|s| s.chars().next());
            let next_next_ch = next_ch.and_then(|next_ch| {
                let after_next = next_i.saturating_add(next_ch.len_utf8()).min(text.len());
                text.get(after_next..).and_then(|s| s.chars().next())
            });
            if let Some(strength) =
                preferred_break_after(prev_ch, ch, next_ch, next_next_ch, policy.knobs)
                && is_boundary
            {
                let cand = BreakCandidate {
                    byte: next_i,
                    cols: col,
                    strength,
                };
                if last_candidate.is_none_or(|prev| {
                    cand.strength > prev.strength
                        || (cand.strength == prev.strength && cand.byte > prev.byte)
                }) {
                    last_candidate = Some(cand);
                }
            }

            prev_ch = Some(ch);
            i = next_i;
        }

        if i >= text.len() {
            break;
        }

        let mut break_at = last_candidate
            .map(|c| (c.byte, c.cols))
            .or(last_grapheme_boundary)
            .map(|v| v.0)
            .unwrap_or(i);
        let mut break_col = last_candidate
            .map(|c| c.cols)
            .or(last_grapheme_boundary.map(|v| v.1))
            .unwrap_or(col);

        if break_at == row_start_byte {
            let next = fret_text_nav::next_grapheme_boundary(text, row_start_byte);
            if next <= row_start_byte {
                break;
            }
            break_at = next;
            break_col = text
                .get(row_start_byte..break_at)
                .unwrap_or("")
                .chars()
                .count()
                .max(1);
        }

        if break_at >= text.len() {
            break;
        }

        starts.push(CodeWrapRowStart {
            byte: break_at,
            col: row_start_col.saturating_add(break_col),
        });
        row_start_byte = break_at;
        row_start_col = row_start_col.saturating_add(break_col);
    }

    if starts.is_empty() {
        starts.push(CodeWrapRowStart { byte: 0, col: 0 });
    }

    starts
}

fn preferred_break_after(
    prev: Option<char>,
    ch: char,
    next: Option<char>,
    next_next: Option<char>,
    knobs: CodeWrapKnobs,
) -> Option<BreakStrength> {
    let next = next?;

    if ch.is_whitespace() && !next.is_whitespace() {
        return Some(BreakStrength::Whitespace);
    }

    if knobs.break_around_operators {
        if next == '-' && next_next == Some('>') {
            return Some(BreakStrength::Operator);
        }
        if next == ':' && next_next == Some(':') {
            return Some(BreakStrength::Operator);
        }
        if prev == Some(':') && ch == ':' && next != ':' {
            return Some(BreakStrength::Operator);
        }
        if prev == Some('-') && ch == '>' {
            return Some(BreakStrength::Operator);
        }
    }

    if knobs.break_after_path_separators && matches!(ch, '/' | '\\') {
        return Some(BreakStrength::Separator);
    }

    if knobs.break_after_url_separators && matches!(ch, '?' | '&' | '#' | '=') {
        return Some(BreakStrength::Separator);
    }

    if knobs.break_after_punctuation && matches!(ch, '.' | ',' | ':' | ';') {
        if knobs.break_around_operators && next == ':' && ch == ':' {
            return None;
        }
        return Some(BreakStrength::Punctuation);
    }

    if knobs.break_at_identifier_boundaries && is_identifier_char(ch) && is_identifier_char(next) {
        if ch == '_' {
            return Some(BreakStrength::Identifier);
        }

        let is_alpha = |c: char| c.is_alphabetic();
        let is_digit = |c: char| c.is_ascii_digit() || c.is_numeric();
        let is_lower = |c: char| c.is_lowercase();
        let is_upper = |c: char| c.is_uppercase();

        if (is_alpha(ch) && is_digit(next)) || (is_digit(ch) && is_alpha(next)) {
            return Some(BreakStrength::Identifier);
        }
        if is_lower(ch) && is_upper(next) {
            return Some(BreakStrength::Identifier);
        }
    }

    None
}

fn is_identifier_char(ch: char) -> bool {
    ch == '_' || is_xid_continue(ch)
}

#[cfg(test)]
mod tests {
    use super::*;

    fn rows(text: &str, wrap_cols: usize, policy: CodeWrapPolicy) -> Vec<&str> {
        let starts = row_starts_for_code_wrap(text, wrap_cols, policy);
        let mut out: Vec<&str> = Vec::new();
        for (idx, start) in starts.iter().enumerate() {
            let end = starts
                .get(idx + 1)
                .map(|v| v.byte)
                .unwrap_or_else(|| text.len());
            out.push(text.get(start.byte..end).unwrap_or(""));
        }
        out
    }

    #[test]
    fn code_wrap_prefers_break_after_whitespace_run() {
        let text = "foo   bar";
        let got = rows(text, 6, CodeWrapPolicy::preset(CodeWrapPreset::Balanced));
        assert_eq!(got, vec!["foo   ", "bar"]);
    }

    #[test]
    fn code_wrap_breaks_after_path_separators() {
        let text = "C:\\foo\\bar\\baz";
        let got = rows(text, 6, CodeWrapPolicy::preset(CodeWrapPreset::Balanced));
        assert!(got.len() >= 2);
        assert!(got.iter().any(|r| r.ends_with('\\')));
    }

    #[test]
    fn code_wrap_breaks_at_snake_and_camel_boundaries() {
        let text = "veryLongIdentifier_name42More";
        let got = rows(text, 10, CodeWrapPolicy::preset(CodeWrapPreset::Balanced));
        assert!(got.len() >= 2);
    }

    #[test]
    fn code_wrap_does_not_split_zwj_emoji_cluster() {
        let text = "aa👨\u{200D}👩\u{200D}👧\u{200D}👦bb";
        let got = rows(text, 3, CodeWrapPolicy::preset(CodeWrapPreset::Aggressive));
        assert!(
            got.iter().any(|r| r.contains('👨')
                || r.contains('👩')
                || r.contains('👧')
                || r.contains('👦'))
        );
        assert!(got.iter().all(|r| !r.ends_with('\u{200D}')));
    }

    #[test]
    fn code_wrap_emergency_breaks_at_grapheme_boundaries() {
        let text = "👨\u{200D}👩\u{200D}👧\u{200D}👦";
        let got = rows(text, 1, CodeWrapPolicy::preset(CodeWrapPreset::Balanced));
        assert_eq!(got, vec![text]);
    }
}
