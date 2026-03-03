use super::*;

use serde_json::{Value, json};
use std::collections::BTreeMap;

pub(super) type DebugExtensionsMapV1 = BTreeMap<String, Value>;

pub(super) type DebugExtensionWriterV1 = Arc<dyn Fn(&App, AppWindowId) -> Option<Value> + 'static>;

const MAX_EXTENSION_BYTES_V1: usize = 64 * 1024;
const MAX_EXTENSIONS_TOTAL_BYTES_V1: usize = 256 * 1024;

#[derive(Default)]
pub(super) struct DebugExtensionsRegistryV1 {
    writers: Vec<(String, DebugExtensionWriterV1)>,
}

impl DebugExtensionsRegistryV1 {
    pub(super) fn register_best_effort(&mut self, key: String, writer: DebugExtensionWriterV1) {
        if !is_valid_extension_key_v1(&key) {
            tracing::warn!(
                target: "fret",
                key = %key,
                "ui diagnostics: ignoring debug extension with invalid key (expected: lowercase, dot-separated, ends with .vN)"
            );
            return;
        }
        if self.writers.iter().any(|(k, _)| k == &key) {
            tracing::warn!(
                target: "fret",
                key = %key,
                "ui diagnostics: ignoring debug extension with duplicate key"
            );
            return;
        }
        self.writers.push((key, writer));
    }

    pub(super) fn capture(&self, app: &App, window: AppWindowId) -> DebugExtensionsMapV1 {
        let mut out: DebugExtensionsMapV1 = BTreeMap::new();
        let mut used_bytes: usize = 0;

        for (key, writer) in &self.writers {
            if used_bytes >= MAX_EXTENSIONS_TOTAL_BYTES_V1 {
                break;
            }
            let Some(value) = writer(app, window) else {
                continue;
            };
            if value.is_null() {
                continue;
            }

            let bytes = serde_json::to_vec(&value).map(|b| b.len()).unwrap_or(0);
            let (value, accounted_bytes) = if bytes > MAX_EXTENSION_BYTES_V1 {
                (
                    json!({
                        "_clipped": true,
                        "_reason": "max_extension_bytes_exceeded",
                        "_bytes": bytes,
                        "_max_bytes": MAX_EXTENSION_BYTES_V1,
                    }),
                    // Count the original size toward the total budget; clipping is an escape hatch,
                    // not a budget bypass.
                    bytes,
                )
            } else {
                (value, bytes)
            };

            if used_bytes.saturating_add(accounted_bytes) > MAX_EXTENSIONS_TOTAL_BYTES_V1 {
                break;
            }
            used_bytes = used_bytes.saturating_add(accounted_bytes);
            out.insert(key.clone(), value);
        }

        out
    }
}

pub(super) fn default_debug_extensions_registry_v1() -> DebugExtensionsRegistryV1 {
    let mut reg = DebugExtensionsRegistryV1::default();

    // Always-present, small, and useful for debugging diagnostics plumbing across transports.
    reg.register_best_effort(
        "diag.runtime.v1".to_string(),
        Arc::new(|app, window| {
            let fixed_delta_ms = app
                .global::<fret_core::WindowFrameClockService>()
                .and_then(|svc| svc.effective_fixed_delta(window))
                .map(|d| d.as_millis().min(u64::MAX as u128) as u64);
            Some(json!({
                "schema_version": "v1",
                "frame_id": app.frame_id().0,
                "fixed_delta_ms": fixed_delta_ms,
            }))
        }),
    );

    // Optional “real” ecosystem-facing example: docking/window interaction summary.
    //
    // This is intentionally small; the typed docking snapshot is already exported elsewhere.
    reg.register_best_effort(
        "dock.graph.v1".to_string(),
        Arc::new(|app, window| {
            let store = app.global::<fret_runtime::WindowInteractionDiagnosticsStore>()?;
            let snap = store
                .docking_for_window(window, app.frame_id())
                .or_else(|| store.docking_latest_for_window(window))?;

            let stats = snap.dock_graph_stats?;
            let signature = snap.dock_graph_signature.as_ref();

            Some(json!({
                "schema_version": "v1",
                "stats": {
                    "node_count": stats.node_count,
                    "tabs_count": stats.tabs_count,
                    "split_count": stats.split_count,
                    "floating_count": stats.floating_count,
                    "max_depth": stats.max_depth,
                    "max_split_depth": stats.max_split_depth,
                    "canonical_ok": stats.canonical_ok,
                    "has_nested_same_axis_splits": stats.has_nested_same_axis_splits,
                },
                "signature": signature.map(|s| json!({
                    "fingerprint64": s.fingerprint64,
                    "signature": s.signature,
                })),
            }))
        }),
    );

    reg
}

fn is_valid_extension_key_v1(key: &str) -> bool {
    let k = key.trim();
    if k.is_empty() || k.len() > 128 {
        return false;
    }
    if !k.contains('.') {
        return false;
    }
    if !k.starts_with(|c: char| c.is_ascii_lowercase() || c.is_ascii_digit()) {
        return false;
    }
    if !k.ends_with(".v1")
        && !k.ends_with(".v2")
        && !k.ends_with(".v3")
        && !k.ends_with(".v4")
        && !k.ends_with(".v5")
        && !k.ends_with(".v6")
        && !k.ends_with(".v7")
        && !k.ends_with(".v8")
        && !k.ends_with(".v9")
    {
        return false;
    }
    k.chars()
        .all(|c| c.is_ascii_lowercase() || c.is_ascii_digit() || c == '.' || c == '_' || c == '-')
}

#[cfg(test)]
mod tests {
    use super::*;
    use slotmap::KeyData;

    #[test]
    fn extension_key_rules_are_reasonable() {
        assert!(is_valid_extension_key_v1("diag.runtime.v1"));
        assert!(is_valid_extension_key_v1("dock.graph.v1"));
        assert!(is_valid_extension_key_v1("a.b_c-d.v9"));
        assert!(!is_valid_extension_key_v1(""));
        assert!(!is_valid_extension_key_v1("NoCaps.v1"));
        assert!(!is_valid_extension_key_v1("no_dots_v1"));
        assert!(!is_valid_extension_key_v1("bad space.v1"));
        assert!(!is_valid_extension_key_v1("missing_version_suffix"));
    }

    #[test]
    fn registry_clips_large_payloads() {
        let mut reg = DebugExtensionsRegistryV1::default();
        reg.register_best_effort(
            "test.big.v1".to_string(),
            Arc::new(|_app, _window| Some(Value::String("x".repeat(MAX_EXTENSION_BYTES_V1 + 16)))),
        );
        let app = App::new();
        let window = AppWindowId::from(KeyData::from_ffi(1));

        let out = reg.capture(&app, window);
        let v = out.get("test.big.v1").expect("extension exists");
        assert_eq!(v.get("_clipped").and_then(|v| v.as_bool()), Some(true));
        assert_eq!(
            v.get("_max_bytes").and_then(|v| v.as_u64()),
            Some(MAX_EXTENSION_BYTES_V1 as u64)
        );
    }
}
