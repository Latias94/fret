use std::sync::Arc;

pub(crate) fn slugify(text: &str) -> Arc<str> {
    let mut out = String::new();
    let mut prev_dash = false;

    for ch in text.trim().chars() {
        let ch = ch.to_ascii_lowercase();
        if ch.is_alphanumeric() {
            out.push(ch);
            prev_dash = false;
            continue;
        }

        if ch.is_whitespace() || ch == '-' || ch == '_' {
            if !out.is_empty() && !prev_dash {
                out.push('-');
                prev_dash = true;
            }
            continue;
        }
    }

    while out.ends_with('-') {
        out.pop();
    }

    if out.is_empty() {
        return Arc::<str>::from("section");
    }

    Arc::<str>::from(out)
}

fn decode_percent_fragment(fragment: &str) -> String {
    fn from_hex(b: u8) -> Option<u8> {
        match b {
            b'0'..=b'9' => Some(b - b'0'),
            b'a'..=b'f' => Some(b - b'a' + 10),
            b'A'..=b'F' => Some(b - b'A' + 10),
            _ => None,
        }
    }

    // Best-effort percent-decoding for in-app `#fragment` navigation.
    // We keep invalid sequences as-is and avoid pulling in a URL dependency.
    let bytes = fragment.as_bytes();
    let mut out: Vec<u8> = Vec::with_capacity(bytes.len());
    let mut i = 0usize;
    while i < bytes.len() {
        if bytes[i] == b'%' && i + 2 < bytes.len()
            && let (Some(hi), Some(lo)) = (from_hex(bytes[i + 1]), from_hex(bytes[i + 2])) {
                out.push(hi << 4 | lo);
                i += 3;
                continue;
            }
        out.push(bytes[i]);
        i += 1;
    }
    String::from_utf8(out).unwrap_or_else(|e| String::from_utf8_lossy(e.as_bytes()).into_owned())
}

pub(crate) fn heading_anchor_test_id(text: &str) -> Arc<str> {
    heading_anchor_test_id_with_id(text, None)
}

pub(crate) fn heading_anchor_test_id_with_id(text: &str, explicit_id: Option<&str>) -> Arc<str> {
    let source = explicit_id.unwrap_or(text);
    let slug = slugify(source);
    Arc::<str>::from(format!("fret-markdown.anchor.{slug}"))
}

pub(crate) fn footnote_anchor_test_id(label: &str) -> Arc<str> {
    let label = label.trim();
    if label.is_empty() {
        return Arc::<str>::from("fret-markdown.anchor.fn");
    }
    // Keep it readable and test-id safe.
    let slug = slugify(label);
    Arc::<str>::from(format!("fret-markdown.anchor.fn-{slug}"))
}

/// Maps an in-document `#fragment` (e.g. `math`, `fn-note`) to the corresponding
/// `test_id` used by `fret-markdown` anchors.
///
/// - Headings: `fret-markdown.anchor.<slug>`
/// - Footnotes: `fret-markdown.anchor.fn-<slug>`
///
/// The input may include a leading `#`.
pub fn anchor_test_id_from_fragment(fragment: &str) -> Arc<str> {
    let fragment = fragment.trim().trim_start_matches('#').trim();
    let decoded = decode_percent_fragment(fragment);
    let decoded = decoded.trim();

    if let Some(rest) = decoded.strip_prefix("fn-") {
        return footnote_anchor_test_id(rest);
    }

    heading_anchor_test_id(decoded)
}
