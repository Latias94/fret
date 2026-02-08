#[derive(Debug, Clone, PartialEq, Eq)]
pub struct QueryPair {
    pub key: String,
    pub value: Option<String>,
}

/// Parse query pairs from a raw query-like string.
///
/// Accepted inputs:
/// - `?a=1&b=2`
/// - `#a=1&b=2`
/// - `#/path?a=1&b=2`
/// - `a=1&b=2`
pub fn parse_query_pairs(raw: &str) -> Vec<QueryPair> {
    let body = query_body(raw);
    if body.is_empty() {
        return Vec::new();
    }

    body.split('&')
        .filter(|part| !part.is_empty())
        .filter_map(|part| {
            let (raw_key, raw_value) = part
                .split_once('=')
                .map_or((part, None), |(k, v)| (k, Some(v)));
            let key = decode_component(raw_key);
            if key.is_empty() {
                return None;
            }

            let value = raw_value
                .map(decode_component)
                .and_then(|value| if value.is_empty() { None } else { Some(value) });

            Some(QueryPair { key, value })
        })
        .collect()
}

/// Return the first decoded value for `key` in a raw query-like string.
pub fn first_query_value(raw: &str, key: &str) -> Option<String> {
    parse_query_pairs(raw)
        .into_iter()
        .find(|pair| pair.key == key)
        .and_then(|pair| pair.value)
}

/// Return all decoded values for `key` in a raw query-like string.
///
/// Empty values are represented as `None`.
pub fn query_values(raw: &str, key: &str) -> Vec<Option<String>> {
    parse_query_pairs(raw)
        .into_iter()
        .filter(|pair| pair.key == key)
        .map(|pair| pair.value)
        .collect()
}

/// Return the first query value for `key`, preferring search over hash.
pub fn first_query_value_from_search_or_hash(
    search: &str,
    hash: &str,
    key: &str,
) -> Option<String> {
    first_query_value(search, key).or_else(|| first_query_value(hash, key))
}

/// Format query pairs into a canonical query string.
///
/// Returned string includes the `?` prefix when non-empty.
pub fn format_query_pairs(pairs: &[QueryPair]) -> String {
    if pairs.is_empty() {
        return String::new();
    }

    let mut out = String::new();
    out.push('?');

    for (index, pair) in pairs.iter().enumerate() {
        if index > 0 {
            out.push('&');
        }

        out.push_str(encode_component(pair.key.as_str()).as_str());

        if let Some(value) = pair.value.as_deref() {
            out.push('=');
            out.push_str(encode_component(value).as_str());
        }
    }

    out
}

fn query_body(raw: &str) -> &str {
    let raw = raw.trim();
    if raw.is_empty() {
        return "";
    }

    if let Some(stripped) = raw.strip_prefix('?') {
        return strip_fragment(stripped);
    }

    if let Some(stripped) = raw.strip_prefix('#') {
        return query_body_from_hash(stripped);
    }

    if let Some((_, query_and_fragment)) = raw.split_once('?') {
        return strip_fragment(query_and_fragment);
    }

    if raw.contains('/') && !raw.contains('=') && !raw.contains('&') {
        ""
    } else {
        strip_fragment(raw)
    }
}

fn query_body_from_hash(hash: &str) -> &str {
    let hash = hash.trim();
    if hash.is_empty() {
        return "";
    }

    if let Some((_, query_and_fragment)) = hash.split_once('?') {
        return strip_fragment(query_and_fragment);
    }

    if hash.contains('=') || hash.contains('&') {
        strip_fragment(hash)
    } else {
        ""
    }
}

fn strip_fragment(raw: &str) -> &str {
    raw.split('#').next().unwrap_or_default()
}

/// Decode a query component (application/x-www-form-urlencoded compatible).
///
/// Notes:
/// - `+` decodes to space.
/// - Percent decoding is applied when possible; invalid sequences are preserved.
pub(crate) fn decode_component(input: &str) -> String {
    decode_percent_component(input, true)
}

/// Decode a path/hash/fragment component (RFC 3986 style).
///
/// Notes:
/// - `+` is treated as a literal plus (not a space).
/// - Percent decoding is applied when possible; invalid sequences are preserved.
pub(crate) fn decode_path_component(input: &str) -> String {
    decode_percent_component(input, false)
}

pub(crate) fn encode_component(input: &str) -> String {
    if input.is_empty() {
        return String::new();
    }

    let mut out = String::with_capacity(input.len());
    for byte in input.bytes() {
        if is_unreserved(byte) {
            out.push(char::from(byte));
        } else {
            out.push('%');
            out.push(
                char::from_digit((byte >> 4) as u32, 16)
                    .unwrap()
                    .to_ascii_uppercase(),
            );
            out.push(
                char::from_digit((byte & 0x0F) as u32, 16)
                    .unwrap()
                    .to_ascii_uppercase(),
            );
        }
    }
    out
}

fn is_unreserved(byte: u8) -> bool {
    byte.is_ascii_alphanumeric() || matches!(byte, b'-' | b'.' | b'_' | b'~')
}

fn decode_percent_component(input: &str, plus_to_space: bool) -> String {
    let bytes = input.as_bytes();
    let mut out = Vec::with_capacity(bytes.len());
    let mut index = 0;

    while index < bytes.len() {
        match bytes[index] {
            b'+' if plus_to_space => {
                out.push(b' ');
                index += 1;
            }
            b'%' if index + 2 < bytes.len() => {
                if let (Some(high), Some(low)) =
                    (hex_value(bytes[index + 1]), hex_value(bytes[index + 2]))
                {
                    out.push((high << 4) | low);
                    index += 3;
                } else {
                    out.push(bytes[index]);
                    index += 1;
                }
            }
            byte => {
                out.push(byte);
                index += 1;
            }
        }
    }

    String::from_utf8_lossy(&out).into_owned()
}

fn hex_value(byte: u8) -> Option<u8> {
    match byte {
        b'0'..=b'9' => Some(byte - b'0'),
        b'a'..=b'f' => Some(byte - b'a' + 10),
        b'A'..=b'F' => Some(byte - b'A' + 10),
        _ => None,
    }
}

#[cfg(test)]
mod tests {
    use super::{
        QueryPair, first_query_value, first_query_value_from_search_or_hash, format_query_pairs,
        parse_query_pairs, query_values,
    };

    #[test]
    fn parse_query_pairs_decodes_components() {
        let pairs = parse_query_pairs("?a=1&name=hello+world&x=%E4%BD%A0%E5%A5%BD");
        assert_eq!(pairs.len(), 3);
        assert_eq!(pairs[0].key, "a");
        assert_eq!(pairs[0].value.as_deref(), Some("1"));
        assert_eq!(pairs[1].key, "name");
        assert_eq!(pairs[1].value.as_deref(), Some("hello world"));
        assert_eq!(pairs[2].key, "x");
        assert_eq!(pairs[2].value.as_deref(), Some("\u{4f60}\u{597d}"));
    }

    #[test]
    fn first_query_value_prefers_search_then_hash() {
        let value =
            first_query_value_from_search_or_hash("?demo=ui_gallery", "#demo=chart_demo", "demo");
        assert_eq!(value.as_deref(), Some("ui_gallery"));

        let value = first_query_value_from_search_or_hash("", "#demo=chart_demo", "demo");
        assert_eq!(value.as_deref(), Some("chart_demo"));
    }

    #[test]
    fn first_query_value_returns_none_for_missing_key() {
        assert_eq!(first_query_value("?a=1", "b"), None);
    }

    #[test]
    fn format_query_pairs_encodes_components() {
        let query = format_query_pairs(&[
            QueryPair {
                key: "a b".to_string(),
                value: Some("x/y".to_string()),
            },
            QueryPair {
                key: "empty".to_string(),
                value: None,
            },
        ]);

        assert_eq!(query, "?a%20b=x%2Fy&empty");
    }

    #[test]
    fn query_values_preserves_duplicate_keys() {
        let values = query_values("?tag=rust&tag=wasm&tag", "tag");
        assert_eq!(
            values,
            vec![Some("rust".to_string()), Some("wasm".to_string()), None]
        );
    }

    #[test]
    fn parse_query_pairs_handles_hash_route_query() {
        let value = first_query_value("#/docs/getting-started?page=intro", "page");
        assert_eq!(value.as_deref(), Some("intro"));
    }

    #[test]
    fn parse_query_pairs_preserves_invalid_percent_sequences() {
        let pairs = parse_query_pairs("?q=%ZZ&ok=1");
        assert_eq!(pairs.len(), 2);
        assert_eq!(pairs[0].key, "q");
        assert_eq!(pairs[0].value.as_deref(), Some("%ZZ"));
        assert_eq!(pairs[1].key, "ok");
        assert_eq!(pairs[1].value.as_deref(), Some("1"));
    }

    #[test]
    fn parse_query_pairs_ignores_path_without_query() {
        let pairs = parse_query_pairs("#/docs/getting-started");
        assert!(pairs.is_empty());
    }
}
