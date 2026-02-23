use std::path::Path;

use super::semantics::{semantics_node_id_for_test_id, semantics_parent_map};

pub(super) fn check_bundle_for_drag_cache_root_paint_only(
    bundle_path: &Path,
    test_id: &str,
    warmup_frames: u64,
) -> Result<(), String> {
    let bytes = std::fs::read(bundle_path).map_err(|e| e.to_string())?;
    let bundle: serde_json::Value = serde_json::from_slice(&bytes).map_err(|e| e.to_string())?;
    let semantics = crate::json_bundle::SemanticsResolver::new(&bundle);

    let windows = bundle
        .get("windows")
        .and_then(|v| v.as_array())
        .ok_or_else(|| "invalid bundle.json: missing windows".to_string())?;
    if windows.is_empty() {
        return Ok(());
    }

    let mut examined_snapshots: u64 = 0;
    let mut good_frames: u64 = 0;
    let mut bad_frames: Vec<String> = Vec::new();
    let mut missing_target_count: u64 = 0;
    let mut any_view_cache_active = false;
    let mut seen_good = false;

    for w in windows {
        let window_id = w.get("window").and_then(|v| v.as_u64()).unwrap_or(0);
        let snaps = w
            .get("snapshots")
            .and_then(|v| v.as_array())
            .map_or(&[][..], |v| v);

        for s in snaps {
            let frame_id = s.get("frame_id").and_then(|v| v.as_u64()).unwrap_or(0);
            if frame_id < warmup_frames {
                continue;
            }
            examined_snapshots = examined_snapshots.saturating_add(1);

            let view_cache_active = s
                .get("debug")
                .and_then(|v| v.get("stats"))
                .and_then(|v| v.get("view_cache_active"))
                .and_then(|v| v.as_bool())
                .unwrap_or(false);
            any_view_cache_active |= view_cache_active;
            if !view_cache_active {
                continue;
            }

            let Some(target_node_id) = semantics_node_id_for_test_id(&semantics, s, test_id) else {
                missing_target_count = missing_target_count.saturating_add(1);
                continue;
            };

            let _nodes = semantics
                .nodes(s)
                .ok_or_else(|| "invalid bundle.json: missing semantics nodes".to_string())?;
            let parents = semantics_parent_map(&semantics, s);

            let roots = s
                .get("debug")
                .and_then(|v| v.get("cache_roots"))
                .and_then(|v| v.as_array())
                .ok_or_else(|| "invalid bundle.json: missing debug.cache_roots".to_string())?;
            let mut cache_roots: std::collections::HashMap<u64, &serde_json::Value> =
                std::collections::HashMap::new();
            for r in roots {
                if let Some(root) = r.get("root").and_then(|v| v.as_u64()) {
                    cache_roots.insert(root, r);
                }
            }

            let mut current = target_node_id;
            let mut cache_root_node: Option<u64> = None;
            loop {
                if cache_roots.contains_key(&current) {
                    cache_root_node = Some(current);
                    break;
                }
                let Some(parent) = parents.get(&current).copied() else {
                    break;
                };
                current = parent;
            }
            let Some(cache_root_node) = cache_root_node else {
                return Err(format!(
                    "could not resolve a cache root ancestor for test_id={test_id} (node_id={target_node_id}) in bundle: {}",
                    bundle_path.display()
                ));
            };

            let root = cache_roots
                .get(&cache_root_node)
                .ok_or_else(|| "internal error: cache root missing".to_string())?;

            let reused = root
                .get("reused")
                .and_then(|v| v.as_bool())
                .unwrap_or(false);
            let contained_relayout_in_frame = root
                .get("contained_relayout_in_frame")
                .and_then(|v| v.as_bool())
                .unwrap_or(false);

            let dirty = s
                .get("debug")
                .and_then(|v| v.get("dirty_views"))
                .and_then(|v| v.as_array())
                .is_some_and(|dirty| {
                    dirty.iter().any(|d| {
                        d.get("root_node")
                            .and_then(|v| v.as_u64())
                            .is_some_and(|n| n == cache_root_node)
                    })
                });

            let ok = reused && !contained_relayout_in_frame && !dirty;
            if ok {
                good_frames = good_frames.saturating_add(1);
                seen_good = true;
                continue;
            }

            if seen_good {
                bad_frames.push(format!(
                    "window={window_id} frame_id={frame_id} cache_root={cache_root_node} reused={reused} contained_relayout_in_frame={contained_relayout_in_frame} dirty={dirty}"
                ));
            }
        }
    }

    if !bad_frames.is_empty() {
        let mut msg = String::new();
        msg.push_str("expected paint-only drag indicator updates (cache-root reuse, no contained relayout, no dirty view), but found violations after reuse began\n");
        msg.push_str(&format!("bundle: {}\n", bundle_path.display()));
        msg.push_str(&format!("test_id: {test_id}\n"));
        for line in bad_frames.into_iter().take(10) {
            msg.push_str("  ");
            msg.push_str(&line);
            msg.push('\n');
        }
        return Err(msg);
    }

    if good_frames == 0 {
        return Err(format!(
            "did not observe any cache-root-reuse paint-only frames for test_id={test_id} \
(any_view_cache_active={any_view_cache_active}, warmup_frames={warmup_frames}, examined_snapshots={examined_snapshots}, missing_target_count={missing_target_count}) \
in bundle: {}",
            bundle_path.display()
        ));
    }

    Ok(())
}
