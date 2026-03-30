use std::sync::Arc;

use crate::{ListInfo, RawBlockKind};

pub(crate) fn parse_fenced_code_language(info: &str) -> Option<Arc<str>> {
    let info = info.trim();
    if info.is_empty() {
        return None;
    }

    let token = info.split_whitespace().next().unwrap_or("");
    if token.is_empty() {
        return None;
    }

    // Common patterns seen in the wild:
    // - ```rust
    // - ```rust,ignore
    // - ```language-rust
    // - ```{.rust}
    // - ```{.rust .numberLines}
    // - ```{#id .rust}
    let token = token.trim_matches(|c| c == '{' || c == '}');
    let token = token.strip_prefix("language-").unwrap_or(token);
    let token = token.strip_prefix("lang-").unwrap_or(token);

    let token = if token.contains('.') {
        token.split('.').find(|s| !s.is_empty()).unwrap_or(token)
    } else {
        token
    };

    let token = token.split(',').next().unwrap_or(token).trim();
    if token.is_empty() {
        return None;
    }

    Some(Arc::<str>::from(token.to_string()))
}

pub(crate) fn heading_level_to_u8(level: pulldown_cmark::HeadingLevel) -> u8 {
    use pulldown_cmark::HeadingLevel;
    match level {
        HeadingLevel::H1 => 1,
        HeadingLevel::H2 => 2,
        HeadingLevel::H3 => 3,
        HeadingLevel::H4 => 4,
        HeadingLevel::H5 => 5,
        HeadingLevel::H6 => 6,
    }
}

pub(crate) fn strip_blockquote_prefix(text: &str) -> Arc<str> {
    let mut out = String::new();
    for (i, line) in text.lines().enumerate() {
        if i > 0 {
            out.push('\n');
        }
        let mut s = line;
        let mut spaces = 0usize;
        while spaces < 3 && s.starts_with(' ') {
            s = &s[1..];
            spaces += 1;
        }
        if let Some(rest) = s.strip_prefix('>') {
            out.push_str(rest.strip_prefix(' ').unwrap_or(rest));
        } else {
            out.push_str(s);
        }
    }
    Arc::<str>::from(out.trim_end().to_string())
}

pub(crate) fn parse_list_info(text: &str) -> ListInfo {
    let mut ordered = None::<bool>;
    let mut start = 1u32;
    let mut items: Vec<String> = Vec::new();
    let mut cur: Option<String> = None;

    for line in text.lines() {
        if let Some((o, num, content)) = parse_list_item_start(line) {
            if let Some(prev) = cur.take()
                && !prev.trim().is_empty()
            {
                items.push(prev.trim_end().to_string());
            }
            if ordered.is_none() {
                ordered = Some(o);
                if o {
                    start = num.max(1);
                }
            }
            cur = Some(content.to_string());
            continue;
        }

        if let Some(buf) = cur.as_mut() {
            let trimmed = line.trim_end();
            if trimmed.is_empty() {
                continue;
            }
            if !buf.is_empty() {
                buf.push('\n');
            }
            buf.push_str(trimmed.trim_start());
        }
    }

    if let Some(prev) = cur.take()
        && !prev.trim().is_empty()
    {
        items.push(prev.trim_end().to_string());
    }

    ListInfo {
        ordered: ordered.unwrap_or(false),
        start,
        items: items.into_iter().map(Arc::<str>::from).collect(),
    }
}

fn parse_list_item_start(line: &str) -> Option<(bool, u32, &str)> {
    let mut s = line;
    let mut spaces = 0usize;
    while spaces < 3 && s.starts_with(' ') {
        s = &s[1..];
        spaces += 1;
    }

    let bytes = s.as_bytes();
    if bytes.len() >= 2 {
        match bytes[0] {
            b'-' | b'+' | b'*' if bytes[1] == b' ' || bytes[1] == b'\t' => {
                return Some((false, 1, s[2..].trim_start()));
            }
            _ => {}
        }
    }

    let mut i = 0usize;
    while i < bytes.len() && bytes[i].is_ascii_digit() {
        i += 1;
    }
    if i == 0 || i + 1 >= bytes.len() {
        return None;
    }
    let delim = bytes[i];
    if delim != b'.' && delim != b')' {
        return None;
    }
    let ws = bytes[i + 1];
    if ws != b' ' && ws != b'\t' {
        return None;
    }
    let num: u32 = s[..i].parse().ok()?;
    Some((true, num, s[i + 2..].trim_start()))
}

pub(crate) fn split_trailing_heading_id(text: &str) -> (Arc<str>, Option<Arc<str>>) {
    let trimmed = text.trim();
    if !trimmed.ends_with('}') {
        return (Arc::<str>::from(trimmed.to_string()), None);
    }

    let Some(brace_start) = trimmed.rfind('{') else {
        return (Arc::<str>::from(trimmed.to_string()), None);
    };
    if brace_start + 2 >= trimmed.len() {
        return (Arc::<str>::from(trimmed.to_string()), None);
    }

    // Require a whitespace separator so we don't treat literal `{#...}` in the title as an id.
    if brace_start > 0 {
        let sep = trimmed.as_bytes()[brace_start - 1];
        if !sep.is_ascii_whitespace() {
            return (Arc::<str>::from(trimmed.to_string()), None);
        }
    }

    let body = &trimmed[brace_start + 1..trimmed.len() - 1];
    let body = body.trim();
    let Some(rest) = body.strip_prefix('#') else {
        return (Arc::<str>::from(trimmed.to_string()), None);
    };

    let id = rest.split_whitespace().next().unwrap_or("").trim();
    if id.is_empty() {
        return (Arc::<str>::from(trimmed.to_string()), None);
    }

    let title = trimmed[..brace_start].trim_end();
    (
        Arc::<str>::from(title.to_string()),
        Some(Arc::<str>::from(id.to_string())),
    )
}

pub(crate) fn parse_heading_text(raw: &str) -> Option<(u8, Arc<str>, Option<Arc<str>>)> {
    let mut lines = raw.lines();
    let first = lines.next()?.trim_end();
    let second = lines.next().map(str::trim_end);

    // ATX: ### Title
    let atx = first.trim_start_matches(' ');
    if let Some(rest) = atx.strip_prefix('#') {
        let mut level = 1u8;
        let mut tail = rest;
        while level < 6 && tail.starts_with('#') {
            level += 1;
            tail = &tail[1..];
        }
        if !tail.starts_with([' ', '\t']) {
            return None;
        }
        let text = tail.trim();
        if text.is_empty() {
            return None;
        }
        let (title, id) = split_trailing_heading_id(text);
        return Some((level, title, id));
    }

    // Setext:
    // Title
    // -----
    if let Some(underline) = second {
        let underline_trimmed = underline.trim_start_matches(' ').trim_end();
        if underline_trimmed.chars().all(|c| c == '=') && underline_trimmed.len() >= 2 {
            let text = first.trim();
            if text.is_empty() {
                return None;
            }
            let (title, id) = split_trailing_heading_id(text);
            return Some((1, title, id));
        }
        if underline_trimmed.chars().all(|c| c == '-') && underline_trimmed.len() >= 2 {
            let text = first.trim();
            if text.is_empty() {
                return None;
            }
            let (title, id) = split_trailing_heading_id(text);
            return Some((2, title, id));
        }
    }

    None
}

pub(crate) fn parse_code_fence_body(raw: &str) -> (Option<Arc<str>>, Arc<str>) {
    let header = mdstream::syntax::parse_code_fence_header_from_block(raw);
    let language = header
        .and_then(|h| h.language)
        .and_then(parse_fenced_code_language)
        .or_else(|| {
            header
                .and_then(|h| h.language)
                .map(|s| Arc::<str>::from(s.to_string()))
        });

    let mut lines = raw.lines();
    let _first = lines.next().unwrap_or("");
    let mut body_lines: Vec<&str> = lines.collect();

    if let Some(h) = header
        && let Some(last) = body_lines.last().copied()
        && mdstream::syntax::is_code_fence_closing_line(last, h.fence_char, h.fence_len)
    {
        body_lines.pop();
    }

    let body = body_lines.join("\n");
    (language, Arc::<str>::from(body))
}

pub(crate) fn raw_block_kind_from_mdstream(kind: mdstream::BlockKind) -> RawBlockKind {
    match kind {
        mdstream::BlockKind::HtmlBlock => RawBlockKind::HtmlBlock,
        mdstream::BlockKind::MathBlock => RawBlockKind::MathBlock,
        mdstream::BlockKind::FootnoteDefinition => RawBlockKind::FootnoteDefinition,
        mdstream::BlockKind::Unknown => RawBlockKind::Unknown,
        _ => RawBlockKind::Unknown,
    }
}

pub(crate) fn is_display_math_block_text(text: &str) -> bool {
    let s = text.trim();
    if s.is_empty() {
        return false;
    }

    (s.starts_with("$$") && s.ends_with("$$")) || (s.starts_with("\\[") && s.ends_with("\\]"))
}

pub(crate) fn parse_math_block_body(text: &str) -> Arc<str> {
    let s = text.trim();
    if s.is_empty() {
        return Arc::<str>::from("");
    }

    // Support common delimiters. mdstream may already have stripped them, so we fall back to `s`.
    if let Some(rest) = s.strip_prefix("$$") {
        let rest = rest.strip_suffix("$$").unwrap_or(rest);
        return Arc::<str>::from(rest.trim().to_string());
    }
    if let Some(rest) = s.strip_prefix("\\[") {
        let rest = rest.strip_suffix("\\]").unwrap_or(rest);
        return Arc::<str>::from(rest.trim().to_string());
    }

    Arc::<str>::from(s.to_string())
}

pub(crate) fn latex_from_pulldown_math_events(
    events: &[pulldown_cmark::Event<'static>],
) -> Option<Arc<str>> {
    use pulldown_cmark::Event;

    for e in events {
        if let Event::DisplayMath(latex) = e {
            return Some(Arc::<str>::from(latex.to_string()));
        }
    }

    let mut buf = String::new();
    for e in events {
        match e {
            Event::Text(t) | Event::Code(t) | Event::InlineMath(t) | Event::DisplayMath(t) => {
                buf.push_str(t.as_ref())
            }
            Event::SoftBreak | Event::HardBreak => buf.push('\n'),
            _ => {}
        }
    }

    let trimmed = buf.trim();
    if trimmed.is_empty() {
        return None;
    }

    Some(Arc::<str>::from(trimmed.to_string()))
}

pub(crate) fn display_math_only_events(
    events: &[pulldown_cmark::Event<'static>],
) -> Option<Arc<str>> {
    use pulldown_cmark::Event;

    let mut display_latex: Option<Arc<str>> = None;
    let mut has_other = false;

    for e in events {
        match e {
            Event::DisplayMath(latex) => {
                if display_latex.is_some() {
                    return None;
                }
                display_latex = Some(Arc::<str>::from(latex.to_string()));
            }
            Event::Text(t) | Event::Code(t) | Event::InlineMath(t) => {
                if !t.trim().is_empty() {
                    has_other = true;
                }
            }
            _ => {}
        }
    }

    if has_other {
        return None;
    }

    display_latex
}
