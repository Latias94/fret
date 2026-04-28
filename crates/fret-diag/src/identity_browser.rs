use std::collections::{BTreeMap, HashMap};

use serde_json::Value;

#[derive(Debug, Clone, PartialEq, Eq, Hash)]
pub(crate) enum IdentityWarningKind {
    DuplicateKeyedListItemKeyHash,
    UnkeyedListOrderChanged,
    Other(String),
}

impl IdentityWarningKind {
    fn from_raw(raw: Option<&str>) -> Self {
        match raw {
            Some("duplicate_keyed_list_item_key_hash") => Self::DuplicateKeyedListItemKeyHash,
            Some("unkeyed_list_order_changed") => Self::UnkeyedListOrderChanged,
            Some(raw) => Self::Other(raw.to_string()),
            None => Self::Other("unknown".to_string()),
        }
    }

    pub(crate) fn as_str(&self) -> &str {
        match self {
            Self::DuplicateKeyedListItemKeyHash => "duplicate_keyed_list_item_key_hash",
            Self::UnkeyedListOrderChanged => "unkeyed_list_order_changed",
            Self::Other(raw) => raw.as_str(),
        }
    }
}

pub(crate) fn parse_identity_warning_kind(s: &str) -> Option<&'static str> {
    match s.trim().to_ascii_lowercase().as_str() {
        "duplicate_keyed_list_item_key_hash" | "duplicate-keyed-list-item-key-hash" => {
            Some("duplicate_keyed_list_item_key_hash")
        }
        "unkeyed_list_order_changed" | "unkeyed-list-order-changed" => {
            Some("unkeyed_list_order_changed")
        }
        _ => None,
    }
}

pub(crate) fn parse_u64_maybe_hex(s: &str, flag: &str) -> Result<u64, String> {
    let s = s.trim();
    if let Some(hex) = s.strip_prefix("0x").or_else(|| s.strip_prefix("0X")) {
        return u64::from_str_radix(hex, 16).map_err(|_| {
            format!("invalid value for {flag} (expected decimal u64 or 0x-prefixed hex)")
        });
    }
    s.parse::<u64>()
        .map_err(|_| format!("invalid value for {flag} (expected decimal u64 or 0x-prefixed hex)"))
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub(crate) struct IdentityWarningLocation {
    pub(crate) file: Option<String>,
    pub(crate) line: Option<u64>,
    pub(crate) column: Option<u64>,
    raw: Value,
}

impl IdentityWarningLocation {
    fn from_warning(warning: &Value) -> Self {
        let raw = warning.get("location").cloned().unwrap_or(Value::Null);
        let file = raw
            .get("file")
            .and_then(|v| v.as_str())
            .map(|s| s.to_string());
        let line = raw.get("line").and_then(|v| v.as_u64());
        let column = raw.get("column").and_then(|v| v.as_u64());
        Self {
            file,
            line,
            column,
            raw,
        }
    }

    fn to_query_json(&self) -> Value {
        self.raw.clone()
    }
}

#[derive(Debug, Clone, PartialEq, Eq, PartialOrd, Ord)]
pub(crate) struct IdentityWarningBrowserGroupKey {
    pub(crate) kind: String,
    pub(crate) window: u64,
    pub(crate) frame_id: Option<u64>,
    pub(crate) source_file: Option<String>,
    pub(crate) list_id: Option<u64>,
    pub(crate) key_hash: Option<u64>,
    pub(crate) element_path: Option<String>,
}

impl IdentityWarningBrowserGroupKey {
    fn from_row(row: &IdentityWarningBrowserRow) -> Self {
        Self {
            kind: row.kind.as_str().to_string(),
            window: row.window,
            frame_id: row.frame_id,
            source_file: row.location.file.clone(),
            list_id: row.list_id,
            key_hash: row.key_hash,
            element_path: row.element_path.clone(),
        }
    }

    fn to_json(&self) -> Value {
        serde_json::json!({
            "kind": self.kind,
            "window": self.window,
            "frame_id": self.frame_id,
            "source_file": self.source_file,
            "list_id": self.list_id,
            "key_hash": self.key_hash,
            "element_path": self.element_path,
        })
    }
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub(crate) struct IdentityWarningBrowserGroup {
    pub(crate) key: IdentityWarningBrowserGroupKey,
    pub(crate) rows: usize,
}

impl IdentityWarningBrowserGroup {
    fn to_json(&self) -> Value {
        serde_json::json!({
            "key": self.key.to_json(),
            "rows": self.rows,
        })
    }
}

#[derive(Debug, Clone)]
pub(crate) struct IdentityWarningBrowserRow {
    pub(crate) window: u64,
    pub(crate) tick_id: Option<u64>,
    pub(crate) snapshot_frame_id: u64,
    pub(crate) window_snapshot_seq: Option<u64>,
    pub(crate) timestamp_unix_ms: Option<u64>,
    pub(crate) kind: IdentityWarningKind,
    kind_raw: Option<String>,
    pub(crate) frame_id: Option<u64>,
    pub(crate) element: Option<u64>,
    pub(crate) element_path: Option<String>,
    pub(crate) list_id: Option<u64>,
    pub(crate) key_hash: Option<u64>,
    pub(crate) first_index: Option<u64>,
    pub(crate) second_index: Option<u64>,
    pub(crate) previous_len: Option<u64>,
    pub(crate) next_len: Option<u64>,
    pub(crate) location: IdentityWarningLocation,
    pub(crate) group_key: IdentityWarningBrowserGroupKey,
}

impl IdentityWarningBrowserRow {
    fn from_warning(window: u64, snapshot: &Value, warning: &Value) -> Self {
        let kind_raw = warning
            .get("kind")
            .and_then(|v| v.as_str())
            .map(|s| s.to_string());
        let kind = IdentityWarningKind::from_raw(kind_raw.as_deref());
        let location = IdentityWarningLocation::from_warning(warning);
        let mut row = Self {
            window,
            tick_id: snapshot.get("tick_id").and_then(|v| v.as_u64()),
            snapshot_frame_id: crate::json_bundle::snapshot_frame_id(snapshot),
            window_snapshot_seq: crate::json_bundle::snapshot_window_snapshot_seq(snapshot),
            timestamp_unix_ms: snapshot.get("timestamp_unix_ms").and_then(|v| v.as_u64()),
            kind,
            kind_raw,
            frame_id: warning.get("frame_id").and_then(|v| v.as_u64()),
            element: warning.get("element").and_then(|v| v.as_u64()),
            element_path: warning
                .get("element_path")
                .and_then(|v| v.as_str())
                .map(|s| s.to_string()),
            list_id: warning.get("list_id").and_then(|v| v.as_u64()),
            key_hash: warning.get("key_hash").and_then(|v| v.as_u64()),
            first_index: warning.get("first_index").and_then(|v| v.as_u64()),
            second_index: warning.get("second_index").and_then(|v| v.as_u64()),
            previous_len: warning.get("previous_len").and_then(|v| v.as_u64()),
            next_len: warning.get("next_len").and_then(|v| v.as_u64()),
            location,
            group_key: IdentityWarningBrowserGroupKey {
                kind: String::new(),
                window,
                frame_id: None,
                source_file: None,
                list_id: None,
                key_hash: None,
                element_path: None,
            },
        };
        row.group_key = IdentityWarningBrowserGroupKey::from_row(&row);
        row
    }

    fn matches_filters(&self, filters: &IdentityWarningBrowserFilters) -> bool {
        if let Some(want) = filters.kind.as_deref()
            && self.kind.as_str() != want
        {
            return false;
        }
        if let Some(want) = filters.element
            && self.element != Some(want)
        {
            return false;
        }
        if let Some(want) = filters.list_id
            && self.list_id != Some(want)
        {
            return false;
        }
        if let Some(want) = filters.element_path_contains.as_deref()
            && !self
                .element_path
                .as_deref()
                .is_some_and(|path| path.contains(want))
        {
            return false;
        }
        if let Some(want) = filters.file_contains.as_deref()
            && !self
                .location
                .file
                .as_deref()
                .is_some_and(|file| file.contains(want))
        {
            return false;
        }
        true
    }

    pub(crate) fn to_query_json(&self) -> Value {
        serde_json::json!({
            "window": self.window,
            "tick_id": self.tick_id,
            "snapshot_frame_id": self.snapshot_frame_id,
            "window_snapshot_seq": self.window_snapshot_seq,
            "timestamp_unix_ms": self.timestamp_unix_ms,
            "kind": self.kind_raw.as_deref(),
            "frame_id": self.frame_id,
            "element": self.element,
            "element_path": self.element_path.as_deref(),
            "list_id": self.list_id,
            "key_hash": self.key_hash,
            "first_index": self.first_index,
            "second_index": self.second_index,
            "previous_len": self.previous_len,
            "next_len": self.next_len,
            "location": self.location.to_query_json(),
        })
    }
}

#[derive(Debug, Clone)]
pub(crate) struct IdentityWarningBrowserFilters {
    pub(crate) window: Option<u64>,
    pub(crate) kind: Option<String>,
    pub(crate) element: Option<u64>,
    pub(crate) list_id: Option<u64>,
    pub(crate) element_path_contains: Option<String>,
    pub(crate) file_contains: Option<String>,
    pub(crate) warmup_frames: u64,
    pub(crate) timeline: bool,
    pub(crate) top: usize,
}

impl Default for IdentityWarningBrowserFilters {
    fn default() -> Self {
        Self {
            window: None,
            kind: None,
            element: None,
            list_id: None,
            element_path_contains: None,
            file_contains: None,
            warmup_frames: 0,
            timeline: false,
            top: 50,
        }
    }
}

#[derive(Debug, Clone)]
pub(crate) struct IdentityWarningBrowserReport {
    pub(crate) rows: Vec<IdentityWarningBrowserRow>,
    pub(crate) groups: Vec<IdentityWarningBrowserGroup>,
    pub(crate) total_observations: usize,
    pub(crate) matching_observations: usize,
    pub(crate) deduped_observations: usize,
}

impl IdentityWarningBrowserReport {
    pub(crate) fn summary_json(&self) -> Value {
        serde_json::json!({
            "total_observations": self.total_observations,
            "matching_observations": self.matching_observations,
            "deduped_observations": self.deduped_observations,
            "groups": self.groups.iter().map(|group| group.to_json()).collect::<Vec<_>>(),
        })
    }
}

#[derive(Debug, Clone, Hash, PartialEq, Eq)]
struct IdentityWarningDedupKey {
    window: u64,
    kind: String,
    frame_id: Option<u64>,
    element: Option<u64>,
    list_id: Option<u64>,
    key_hash: Option<u64>,
    first_index: Option<u64>,
    second_index: Option<u64>,
    previous_len: Option<u64>,
    next_len: Option<u64>,
}

impl IdentityWarningDedupKey {
    fn from_row(row: &IdentityWarningBrowserRow) -> Self {
        Self {
            window: row.window,
            kind: row.kind.as_str().to_string(),
            frame_id: row.frame_id,
            element: row.element,
            list_id: row.list_id,
            key_hash: row.key_hash,
            first_index: row.first_index,
            second_index: row.second_index,
            previous_len: row.previous_len,
            next_len: row.next_len,
        }
    }
}

pub(crate) fn collect_identity_warning_browser_report(
    bundle: &Value,
    filters: &IdentityWarningBrowserFilters,
) -> IdentityWarningBrowserReport {
    let mut encounter_seq = 0u64;
    let mut total_observations = 0usize;
    let mut matching_observations = 0usize;
    let mut timeline_rows: Vec<IdentityWarningBrowserRow> = Vec::new();
    let mut latest_by_warning: HashMap<IdentityWarningDedupKey, (u64, IdentityWarningBrowserRow)> =
        HashMap::new();

    if let Some(windows) = bundle.get("windows").and_then(|v| v.as_array()) {
        for window_value in windows {
            let window = window_value
                .get("window")
                .and_then(|v| v.as_u64())
                .unwrap_or(0);
            if let Some(want) = filters.window
                && want != window
            {
                continue;
            }

            let Some(snapshots) = window_value.get("snapshots").and_then(|v| v.as_array()) else {
                continue;
            };
            for snapshot in snapshots {
                let snapshot_frame_id = crate::json_bundle::snapshot_frame_id(snapshot);
                if snapshot_frame_id < filters.warmup_frames {
                    continue;
                }

                let Some(warnings) = snapshot
                    .get("debug")
                    .and_then(|v| v.get("element_runtime"))
                    .and_then(|v| v.get("identity_warnings"))
                    .and_then(|v| v.as_array())
                else {
                    continue;
                };

                for warning in warnings {
                    total_observations = total_observations.saturating_add(1);
                    let row = IdentityWarningBrowserRow::from_warning(window, snapshot, warning);
                    if !row.matches_filters(filters) {
                        continue;
                    }

                    matching_observations = matching_observations.saturating_add(1);
                    encounter_seq = encounter_seq.saturating_add(1);
                    if filters.timeline {
                        timeline_rows.push(row);
                    } else {
                        latest_by_warning.insert(
                            IdentityWarningDedupKey::from_row(&row),
                            (encounter_seq, row),
                        );
                    }
                }
            }
        }
    }

    let mut rows = if filters.timeline {
        timeline_rows
    } else {
        let mut rows: Vec<(u64, IdentityWarningBrowserRow)> =
            latest_by_warning.into_values().collect();
        rows.sort_by(|a, b| b.0.cmp(&a.0));
        rows.into_iter().map(|(_, row)| row).collect()
    };
    let deduped_observations = rows.len();
    if filters.top > 0 && rows.len() > filters.top {
        rows.truncate(filters.top);
    }
    let groups = build_groups(&rows);

    IdentityWarningBrowserReport {
        rows,
        groups,
        total_observations,
        matching_observations,
        deduped_observations,
    }
}

fn build_groups(rows: &[IdentityWarningBrowserRow]) -> Vec<IdentityWarningBrowserGroup> {
    let mut groups: BTreeMap<IdentityWarningBrowserGroupKey, usize> = BTreeMap::new();
    for row in rows {
        *groups.entry(row.group_key.clone()).or_insert(0) += 1;
    }
    groups
        .into_iter()
        .map(|(key, rows)| IdentityWarningBrowserGroup { key, rows })
        .collect()
}

#[cfg(test)]
mod tests {
    use super::*;

    fn bundle_with_identity_warnings() -> Value {
        let warning_a = serde_json::json!({
            "kind": "unkeyed_list_order_changed",
            "frame_id": 20u64,
            "element": 123u64,
            "element_path": "root.panel.file.rs:1:1[key=0x1] (0x7b)",
            "list_id": 7u64,
            "previous_len": 3u64,
            "next_len": 4u64,
            "location": {
                "file": "src/view.rs",
                "line": 11u64,
                "column": 9u64
            }
        });
        let warning_b = serde_json::json!({
            "kind": "duplicate_keyed_list_item_key_hash",
            "frame_id": 21u64,
            "element": 456u64,
            "element_path": "root.panel.other.rs:2:1[key=0x2] (0x1c8)",
            "list_id": 42u64,
            "key_hash": 9001u64,
            "first_index": 1u64,
            "second_index": 2u64,
            "location": {
                "file": "src/list.rs",
                "line": 31u64,
                "column": 13u64
            }
        });

        serde_json::json!({
            "schema_version": 2,
            "windows": [{
                "window": 1u64,
                "snapshots": [
                    {
                        "tick_id": 10u64,
                        "frame_id": 20u64,
                        "window_snapshot_seq": 30u64,
                        "timestamp_unix_ms": 40u64,
                        "debug": {
                            "element_runtime": {
                                "identity_warnings": [warning_a.clone(), warning_b]
                            }
                        }
                    },
                    {
                        "tick_id": 11u64,
                        "frame_id": 22u64,
                        "window_snapshot_seq": 31u64,
                        "timestamp_unix_ms": 41u64,
                        "debug": {
                            "element_runtime": {
                                "identity_warnings": [warning_a]
                            }
                        }
                    }
                ]
            }]
        })
    }

    #[test]
    fn identity_browser_maps_duplicate_key_warning_fields() {
        let bundle = bundle_with_identity_warnings();
        let filters = IdentityWarningBrowserFilters {
            list_id: Some(42),
            ..Default::default()
        };

        let report = collect_identity_warning_browser_report(&bundle, &filters);

        assert_eq!(report.total_observations, 3);
        assert_eq!(report.matching_observations, 1);
        assert_eq!(report.deduped_observations, 1);
        assert_eq!(report.rows.len(), 1);
        assert_eq!(report.groups.len(), 1);

        let row = &report.rows[0];
        assert_eq!(row.kind, IdentityWarningKind::DuplicateKeyedListItemKeyHash);
        assert_eq!(row.window, 1);
        assert_eq!(row.snapshot_frame_id, 20);
        assert_eq!(row.frame_id, Some(21));
        assert_eq!(row.element, Some(456));
        assert_eq!(row.list_id, Some(42));
        assert_eq!(row.key_hash, Some(9001));
        assert_eq!(row.first_index, Some(1));
        assert_eq!(row.second_index, Some(2));
        assert_eq!(row.location.file.as_deref(), Some("src/list.rs"));
        assert_eq!(row.location.line, Some(31));
        assert_eq!(row.location.column, Some(13));
        assert_eq!(row.group_key.kind, "duplicate_keyed_list_item_key_hash");
        assert_eq!(row.group_key.source_file.as_deref(), Some("src/list.rs"));

        let query_json = row.to_query_json();
        assert_eq!(
            query_json.get("kind").and_then(|v| v.as_str()),
            Some("duplicate_keyed_list_item_key_hash")
        );
        assert_eq!(
            query_json
                .get("location")
                .and_then(|v| v.get("line"))
                .and_then(|v| v.as_u64()),
            Some(31)
        );
    }

    #[test]
    fn identity_browser_dedups_unkeyed_warning_to_latest_snapshot() {
        let bundle = bundle_with_identity_warnings();
        let filters = IdentityWarningBrowserFilters {
            kind: parse_identity_warning_kind("unkeyed-list-order-changed").map(|s| s.to_string()),
            ..Default::default()
        };

        let report = collect_identity_warning_browser_report(&bundle, &filters);

        assert_eq!(report.matching_observations, 2);
        assert_eq!(report.deduped_observations, 1);
        assert_eq!(report.rows.len(), 1);

        let row = &report.rows[0];
        assert_eq!(row.kind, IdentityWarningKind::UnkeyedListOrderChanged);
        assert_eq!(row.snapshot_frame_id, 22);
        assert_eq!(row.window_snapshot_seq, Some(31));
        assert_eq!(row.previous_len, Some(3));
        assert_eq!(row.next_len, Some(4));
        assert_eq!(row.location.file.as_deref(), Some("src/view.rs"));
    }

    #[test]
    fn identity_browser_timeline_keeps_repeated_snapshot_observations() {
        let bundle = bundle_with_identity_warnings();
        let filters = IdentityWarningBrowserFilters {
            kind: parse_identity_warning_kind("unkeyed_list_order_changed").map(|s| s.to_string()),
            timeline: true,
            ..Default::default()
        };

        let report = collect_identity_warning_browser_report(&bundle, &filters);

        assert_eq!(report.matching_observations, 2);
        assert_eq!(report.deduped_observations, 2);
        assert_eq!(report.rows.len(), 2);
        assert_eq!(report.rows[0].snapshot_frame_id, 20);
        assert_eq!(report.rows[1].snapshot_frame_id, 22);
    }
}
