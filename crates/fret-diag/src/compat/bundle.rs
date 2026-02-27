use serde_json::Value;
use std::path::Path;

pub(crate) fn bundle_schema_version_from_value(value: &Value) -> u32 {
    value
        .get("schema_version")
        .or_else(|| value.get("schemaVersion"))
        .and_then(|v| v.as_u64())
        .unwrap_or(0)
        .min(u32::MAX as u64) as u32
}

fn find_subslice(haystack: &[u8], needle: &[u8]) -> Option<usize> {
    if needle.is_empty() || haystack.len() < needle.len() {
        return None;
    }
    haystack
        .windows(needle.len())
        .position(|window| window == needle)
}

fn parse_number_after_key(bytes: &[u8], start: usize) -> Option<u64> {
    let mut i = start;
    while i < bytes.len()
        && (bytes[i] == b' ' || bytes[i] == b'\n' || bytes[i] == b'\r' || bytes[i] == b'\t')
    {
        i += 1;
    }
    if i >= bytes.len() || bytes[i] != b':' {
        return None;
    }
    i += 1;
    while i < bytes.len()
        && (bytes[i] == b' ' || bytes[i] == b'\n' || bytes[i] == b'\r' || bytes[i] == b'\t')
    {
        i += 1;
    }
    let start_num = i;
    while i < bytes.len() && bytes[i].is_ascii_digit() {
        i += 1;
    }
    if start_num == i {
        return None;
    }
    std::str::from_utf8(&bytes[start_num..i])
        .ok()?
        .parse::<u64>()
        .ok()
}

pub(crate) fn sniff_schema_version_from_json_prefix(bytes: &[u8]) -> Option<u64> {
    for key in [&br#""schema_version""#[..], &br#""schemaVersion""#[..]] {
        let Some(off) = find_subslice(bytes, key) else {
            continue;
        };
        let after = off.saturating_add(key.len());
        if let Some(v) = parse_number_after_key(bytes, after) {
            return Some(v);
        }
    }
    None
}

pub(crate) fn sniff_bundle_schema_version(bundle_json_path: &Path) -> Result<Option<u64>, String> {
    const MAX_PREFIX_BYTES: usize = 64 * 1024;
    let mut bytes = std::fs::read(bundle_json_path).map_err(|e| e.to_string())?;
    if bytes.len() > MAX_PREFIX_BYTES {
        bytes.truncate(MAX_PREFIX_BYTES);
    }
    Ok(sniff_schema_version_from_json_prefix(&bytes))
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn bundle_schema_version_reads_schema_version_key() {
        let v = serde_json::json!({ "schema_version": 2, "windows": [] });
        assert_eq!(bundle_schema_version_from_value(&v), 2);
    }

    #[test]
    fn bundle_schema_version_reads_schema_version_camel_case() {
        let v = serde_json::json!({ "schemaVersion": 1, "windows": [] });
        assert_eq!(bundle_schema_version_from_value(&v), 1);
    }

    #[test]
    fn sniff_schema_version_parses_number_after_key() {
        let bytes = br#"{ "schema_version" : 2, "windows": [] }"#;
        assert_eq!(sniff_schema_version_from_json_prefix(bytes), Some(2));
    }

    #[test]
    fn sniff_schema_version_parses_camel_case_key() {
        let bytes = br#"{ "schemaVersion": 1, "windows": [] }"#;
        assert_eq!(sniff_schema_version_from_json_prefix(bytes), Some(1));
    }
}
