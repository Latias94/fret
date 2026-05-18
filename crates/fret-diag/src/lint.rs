use serde_json::Value;
use std::collections::HashMap;
use std::path::Path;

use crate::json_bundle::{
    SemanticsResolver, pick_last_snapshot_with_resolved_semantics_after_warmup,
};

mod streaming;

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
enum LintLevel {
    Error,
    Warning,
}

impl LintLevel {
    fn as_str(self) -> &'static str {
        match self {
            Self::Error => "error",
            Self::Warning => "warning",
        }
    }
}

#[derive(Debug, Clone, Copy, Default)]
pub(super) struct LintOptions {
    pub(super) all_test_ids_bounds: bool,
    pub(super) eps_px: f32,
}

#[derive(Debug)]
pub(super) struct LintReport {
    pub(super) error_issues: u64,
    pub(super) payload: Value,
}

#[derive(Debug, Clone, Copy)]
struct RectF64 {
    x: f64,
    y: f64,
    w: f64,
    h: f64,
}

impl RectF64 {
    fn right(self) -> f64 {
        self.x + self.w
    }
    fn bottom(self) -> f64 {
        self.y + self.h
    }
}

fn rect_from_bounds(v: &Value) -> Option<RectF64> {
    fn component(v: &Value, logical_key: &str, physical_key: &str) -> Option<f64> {
        v.get(logical_key)
            .and_then(|v| v.as_f64())
            .or_else(|| v.get(physical_key).and_then(|v| v.as_f64()))
    }

    Some(RectF64 {
        x: component(v, "x", "x_px")?,
        y: component(v, "y", "y_px")?,
        w: component(v, "w", "w_px").or_else(|| v.get("width").and_then(|v| v.as_f64()))?,
        h: component(v, "h", "h_px").or_else(|| v.get("height").and_then(|v| v.as_f64()))?,
    })
}

fn rects_intersect(a: RectF64, b: RectF64, eps: f64) -> bool {
    let ax1 = a.right();
    let ay1 = a.bottom();
    let bx1 = b.right();
    let by1 = b.bottom();
    a.x <= bx1 + eps && ax1 + eps >= b.x && a.y <= by1 + eps && ay1 + eps >= b.y
}

fn rect_is_non_empty(r: RectF64, eps: f64) -> bool {
    r.w > eps && r.h > eps
}

fn node_has_scrollable_ancestor(node: &Value, by_id: &HashMap<u64, &Value>) -> bool {
    let mut parent = node.get("parent").and_then(|v| v.as_u64());
    for _ in 0..256 {
        let Some(parent_id) = parent else {
            return false;
        };
        let Some(parent_node) = by_id.get(&parent_id).copied() else {
            return false;
        };
        if node_is_scrollable(parent_node) {
            return true;
        }
        parent = parent_node.get("parent").and_then(|v| v.as_u64());
    }
    false
}

fn node_is_scrollable(node: &Value) -> bool {
    let Some(scroll) = node.get("scroll") else {
        return false;
    };
    let x_min = scroll.get("x_min").and_then(|v| v.as_f64()).unwrap_or(0.0);
    let x_max = scroll.get("x_max").and_then(|v| v.as_f64()).unwrap_or(0.0);
    let y_min = scroll.get("y_min").and_then(|v| v.as_f64()).unwrap_or(0.0);
    let y_max = scroll.get("y_max").and_then(|v| v.as_f64()).unwrap_or(0.0);
    x_max > x_min + 0.5 || y_max > y_min + 0.5
}

fn node_is_hidden(node: &Value) -> bool {
    node.get("flags")
        .and_then(|v| v.get("hidden"))
        .and_then(|v| v.as_bool())
        .unwrap_or(false)
        || node
            .get("hidden")
            .and_then(|v| v.as_bool())
            .unwrap_or(false)
}

fn role_requires_label(role: &str) -> bool {
    matches!(
        role,
        "button"
            | "link"
            | "checkbox"
            | "switch"
            | "slider"
            | "combo_box"
            | "radio_button"
            | "tab"
            | "menu_item"
            | "menu_item_checkbox"
            | "menu_item_radio"
            | "list_box_option"
            | "tree_item"
            | "text_field"
    )
}

fn has_accessible_name_source(node: &Value) -> bool {
    node.get("label")
        .and_then(|v| v.as_str())
        .is_some_and(|s| !s.is_empty())
        || node
            .get("value")
            .and_then(|v| v.as_str())
            .is_some_and(|s| !s.is_empty())
        || node
            .get("labelled_by")
            .and_then(|v| v.as_array())
            .is_some_and(|ids| !ids.is_empty())
}

#[allow(clippy::too_many_arguments)]
fn push_finding(
    findings: &mut Vec<Value>,
    level: LintLevel,
    code: &str,
    window: u64,
    frame_id: u64,
    node_id: Option<u64>,
    test_id: Option<String>,
    role: Option<String>,
    message: impl Into<String>,
    evidence: Value,
) {
    findings.push(serde_json::json!({
        "level": level.as_str(),
        "code": code,
        "window": window,
        "frame_id": frame_id,
        "node_id": node_id,
        "test_id": test_id,
        "role": role,
        "message": message.into(),
        "evidence": evidence,
    }));
}

pub(super) fn lint_bundle_from_path(
    bundle_path: &Path,
    warmup_frames: u64,
    opts: LintOptions,
) -> Result<LintReport, String> {
    const STREAMING_THRESHOLD_BYTES: u64 = 16 * 1024 * 1024;
    let file_len = std::fs::metadata(bundle_path)
        .map(|m| m.len())
        .unwrap_or(STREAMING_THRESHOLD_BYTES + 1);
    if file_len > STREAMING_THRESHOLD_BYTES {
        return streaming::lint_bundle_from_path_streaming(bundle_path, warmup_frames, opts);
    }

    let bytes = std::fs::read(bundle_path).map_err(|e| e.to_string())?;
    let bundle: Value = serde_json::from_slice(&bytes).map_err(|e| e.to_string())?;
    lint_bundle_from_json(&bundle, bundle_path, warmup_frames, opts)
}

fn focused_node_id_from_nodes(nodes: &[Value]) -> Option<u64> {
    nodes.iter().find_map(|n| {
        let id = n.get("id").and_then(|v| v.as_u64())?;
        let flags = n.get("flags");
        let is_focused = flags
            .and_then(|v| v.get("focused"))
            .and_then(|v| v.as_bool())
            .unwrap_or(false)
            || n.get("focused").and_then(|v| v.as_bool()).unwrap_or(false);
        if is_focused { Some(id) } else { None }
    })
}

fn lint_nodes_for_window(
    findings: &mut Vec<Value>,
    window_id: u64,
    frame_id: u64,
    window_bounds_value: &Value,
    nodes: &[Value],
    mut focus: Option<u64>,
    opts: LintOptions,
) {
    let window_bounds = rect_from_bounds(window_bounds_value).unwrap_or(RectF64 {
        x: 0.0,
        y: 0.0,
        w: 0.0,
        h: 0.0,
    });

    if focus.is_none() {
        focus = focused_node_id_from_nodes(nodes);
    }

    let mut by_id: HashMap<u64, &Value> = HashMap::new();
    let mut test_id_to_nodes: HashMap<&str, Vec<u64>> = HashMap::new();

    for n in nodes {
        let Some(id) = n.get("id").and_then(|v| v.as_u64()) else {
            continue;
        };
        by_id.insert(id, n);
        if let Some(test_id) = n.get("test_id").and_then(|v| v.as_str())
            && !test_id.trim().is_empty()
        {
            test_id_to_nodes.entry(test_id).or_default().push(id);
        }
    }

    for (test_id, ids) in test_id_to_nodes.iter() {
        if ids.len() <= 1 {
            continue;
        }
        let mut ids_sorted = ids.clone();
        ids_sorted.sort_unstable();
        push_finding(
            findings,
            LintLevel::Error,
            "semantics.duplicate_test_id",
            window_id,
            frame_id,
            None,
            Some(test_id.to_string()),
            None,
            format!("duplicate test_id: {test_id}"),
            serde_json::json!({ "node_ids": ids_sorted }),
        );
    }

    for n in nodes {
        let Some(id) = n.get("id").and_then(|v| v.as_u64()) else {
            continue;
        };
        let role = n
            .get("role")
            .and_then(|v| v.as_str())
            .map(|s| s.to_string());
        let test_id = n
            .get("test_id")
            .and_then(|v| v.as_str())
            .map(|s| s.to_string());

        let active_descendant = n.get("active_descendant").and_then(|v| v.as_u64());
        if let Some(active_descendant) = active_descendant
            && !by_id.contains_key(&active_descendant)
        {
            push_finding(
                findings,
                LintLevel::Error,
                "semantics.active_descendant_missing",
                window_id,
                frame_id,
                Some(id),
                test_id.clone(),
                role.clone(),
                "active_descendant points to a missing node",
                serde_json::json!({ "active_descendant": active_descendant }),
            );
        }

        let hidden = node_is_hidden(n);
        let is_focused = focus == Some(id);
        let skip_visible_semantics_lints = hidden && !is_focused;

        let role_str = role.as_deref().unwrap_or("");
        if !skip_visible_semantics_lints
            && role_requires_label(role_str)
            && !has_accessible_name_source(n)
        {
            push_finding(
                findings,
                LintLevel::Warning,
                "semantics.missing_label",
                window_id,
                frame_id,
                Some(id),
                test_id.clone(),
                role.clone(),
                "interactive semantics node is missing label/value",
                Value::Null,
            );
        }

        let Some(bounds) = n.get("bounds").and_then(rect_from_bounds) else {
            continue;
        };
        let eps = opts.eps_px.max(0.0) as f64;

        if !skip_visible_semantics_lints && !rect_is_non_empty(bounds, eps) {
            let level = if is_focused {
                LintLevel::Error
            } else {
                LintLevel::Warning
            };
            if test_id.is_some() || is_focused {
                push_finding(
                    findings,
                    level,
                    "layout.zero_size",
                    window_id,
                    frame_id,
                    Some(id),
                    test_id.clone(),
                    role.clone(),
                    "semantics bounds are empty (w/h too small)",
                    serde_json::json!({ "bounds": n.get("bounds").cloned().unwrap_or(Value::Null) }),
                );
            }
        }

        if !skip_visible_semantics_lints
            && is_focused
            && !rects_intersect(bounds, window_bounds, eps)
        {
            push_finding(
                findings,
                LintLevel::Error,
                "layout.focused_out_of_window",
                window_id,
                frame_id,
                Some(id),
                test_id.clone(),
                role.clone(),
                "focused semantics node is outside the window bounds",
                serde_json::json!({
                    "bounds": n.get("bounds").cloned().unwrap_or(Value::Null),
                    "window_bounds": window_bounds_value.clone(),
                }),
            );
        }

        if !skip_visible_semantics_lints
            && opts.all_test_ids_bounds
            && test_id.is_some()
            && !rects_intersect(bounds, window_bounds, eps)
        {
            push_finding(
                findings,
                LintLevel::Warning,
                "layout.test_id_out_of_window",
                window_id,
                frame_id,
                Some(id),
                test_id.clone(),
                role.clone(),
                "test_id node is outside the window bounds",
                serde_json::json!({
                    "bounds": n.get("bounds").cloned().unwrap_or(Value::Null),
                    "window_bounds": window_bounds_value.clone(),
                }),
            );
        }

        if is_focused
            && let Some(active) = active_descendant
            && let Some(active_node) = by_id.get(&active)
            && let Some(active_bounds) = active_node.get("bounds").and_then(rect_from_bounds)
            && !rects_intersect(active_bounds, window_bounds, eps)
        {
            let scrollable_ancestor = node_has_scrollable_ancestor(active_node, &by_id);
            push_finding(
                findings,
                if scrollable_ancestor {
                    LintLevel::Warning
                } else {
                    LintLevel::Error
                },
                "layout.active_item_out_of_window",
                window_id,
                frame_id,
                Some(active),
                active_node
                    .get("test_id")
                    .and_then(|v| v.as_str())
                    .map(|s| s.to_string()),
                active_node
                    .get("role")
                    .and_then(|v| v.as_str())
                    .map(|s| s.to_string()),
                "active item is outside the window bounds",
                serde_json::json!({
                    "bounds": active_node.get("bounds").cloned().unwrap_or(Value::Null),
                    "window_bounds": window_bounds_value.clone(),
                    "scrollable_ancestor": scrollable_ancestor,
                }),
            );
        }
    }
}

fn finish_lint_report(
    mut findings: Vec<Value>,
    bundle_path: &Path,
    warmup_frames: u64,
    opts: LintOptions,
) -> LintReport {
    findings.sort_by(|a, b| {
        let la = a.get("level").and_then(|v| v.as_str()).unwrap_or("");
        let lb = b.get("level").and_then(|v| v.as_str()).unwrap_or("");
        let level_ord = match (la, lb) {
            ("error", "warning") => std::cmp::Ordering::Less,
            ("warning", "error") => std::cmp::Ordering::Greater,
            _ => std::cmp::Ordering::Equal,
        };
        let ca = a.get("code").and_then(|v| v.as_str()).unwrap_or("");
        let cb = b.get("code").and_then(|v| v.as_str()).unwrap_or("");
        let wa = a.get("window").and_then(|v| v.as_u64()).unwrap_or(0);
        let wb = b.get("window").and_then(|v| v.as_u64()).unwrap_or(0);
        let fa = a.get("frame_id").and_then(|v| v.as_u64()).unwrap_or(0);
        let fb = b.get("frame_id").and_then(|v| v.as_u64()).unwrap_or(0);
        let ta = a.get("test_id").and_then(|v| v.as_str()).unwrap_or("");
        let tb = b.get("test_id").and_then(|v| v.as_str()).unwrap_or("");
        let na = a.get("node_id").and_then(|v| v.as_u64()).unwrap_or(0);
        let nb = b.get("node_id").and_then(|v| v.as_u64()).unwrap_or(0);

        level_ord
            .then_with(|| ca.cmp(cb))
            .then_with(|| wa.cmp(&wb))
            .then_with(|| fa.cmp(&fb))
            .then_with(|| ta.cmp(tb))
            .then_with(|| na.cmp(&nb))
    });

    let mut error_issues = 0u64;
    let mut warning_issues = 0u64;
    let mut counts: HashMap<&str, (u64, u64)> = HashMap::new(); // (error, warning)
    for f in &findings {
        let level = f.get("level").and_then(|v| v.as_str()).unwrap_or("");
        match level {
            "error" => error_issues += 1,
            "warning" => warning_issues += 1,
            _ => {}
        }
        let code = f.get("code").and_then(|v| v.as_str()).unwrap_or("unknown");
        let entry = counts.entry(code).or_insert((0, 0));
        if level == "error" {
            entry.0 += 1;
        } else {
            entry.1 += 1;
        }
    }

    let mut counts_vec: Vec<Value> = counts
        .into_iter()
        .map(|(code, (errors, warnings))| {
            serde_json::json!({
                "code": code,
                "errors": errors,
                "warnings": warnings,
            })
        })
        .collect();
    counts_vec.sort_by(|a, b| {
        a.get("code")
            .and_then(|v| v.as_str())
            .cmp(&b.get("code").and_then(|v| v.as_str()))
    });
    let (bundle_artifact, bundle_json) =
        crate::artifact_alias::bundle_artifact_alias_pair(bundle_path);

    let payload = serde_json::json!({
        "schema_version": 1,
        "kind": "lint",
        "bundle_artifact": bundle_artifact,
        "bundle_json": bundle_json,
        "warmup_frames": warmup_frames,
        "options": {
            "all_test_ids_bounds": opts.all_test_ids_bounds,
            "eps_px": opts.eps_px,
        },
        "error_issues": error_issues,
        "warning_issues": warning_issues,
        "counts_by_code": counts_vec,
        "findings": findings,
    });

    LintReport {
        error_issues,
        payload,
    }
}

fn lint_bundle_from_json(
    bundle: &Value,
    bundle_path: &Path,
    warmup_frames: u64,
    opts: LintOptions,
) -> Result<LintReport, String> {
    let windows = bundle
        .get("windows")
        .and_then(|v| v.as_array())
        .ok_or_else(|| "invalid bundle artifact: missing windows".to_string())?;

    let semantics = SemanticsResolver::new(bundle);

    let mut findings: Vec<Value> = Vec::new();

    for w in windows {
        let window_id = w.get("window").and_then(|v| v.as_u64()).unwrap_or(0);
        let snaps = w
            .get("snapshots")
            .and_then(|v| v.as_array())
            .map_or(&[][..], |v| v);
        let Some(snapshot) = pick_last_snapshot_with_resolved_semantics_after_warmup(
            snaps,
            warmup_frames,
            &semantics,
        ) else {
            continue;
        };

        let frame_id = crate::json_bundle::snapshot_frame_id(snapshot);
        let window_bounds_value = snapshot
            .get("window_bounds")
            .cloned()
            .unwrap_or(Value::Null);

        let focus = semantics
            .semantics_snapshot(snapshot)
            .and_then(|v| v.get("focus"))
            .and_then(|v| v.as_u64());

        let nodes = semantics.nodes(snapshot).unwrap_or(&[]);
        if nodes.is_empty() {
            continue;
        }

        lint_nodes_for_window(
            &mut findings,
            window_id,
            frame_id,
            &window_bounds_value,
            nodes,
            focus,
            opts,
        );
    }

    Ok(finish_lint_report(
        findings,
        bundle_path,
        warmup_frames,
        opts,
    ))
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::time::{SystemTime, UNIX_EPOCH};

    #[test]
    fn lint_rect_from_bounds_accepts_physical_keys() {
        let rect = rect_from_bounds(&serde_json::json!({
            "x_px": 4.0,
            "y_px": 5.0,
            "w_px": 640.0,
            "h_px": 480.0
        }))
        .expect("rect");

        assert_eq!(rect.x, 4.0);
        assert_eq!(rect.y, 5.0);
        assert_eq!(rect.w, 640.0);
        assert_eq!(rect.h, 480.0);
    }

    #[test]
    fn lint_accepts_physical_bounds_keys_in_bundle_payloads() {
        let bundle = serde_json::json!({
            "schema_version": 1,
            "windows": [
                {
                    "window": 1,
                    "snapshots": [
                        {
                            "frame_id": 10,
                            "window_bounds": { "x_px": 0.0, "y_px": 0.0, "w_px": 100.0, "h_px": 100.0 },
                            "debug": {
                                "semantics": {
                                    "window": 1,
                                    "focus": 1,
                                    "captured": null,
                                    "nodes": [
                                        {
                                            "id": 1,
                                            "parent": null,
                                            "role": "button",
                                            "bounds": { "x_px": 0.0, "y_px": 0.0, "w_px": 20.0, "h_px": 20.0 },
                                            "flags": { "focused": true },
                                            "test_id": "dup",
                                            "label": "One"
                                        },
                                        {
                                            "id": 2,
                                            "parent": null,
                                            "role": "button",
                                            "bounds": { "x_px": 24.0, "y_px": 0.0, "w_px": 20.0, "h_px": 20.0 },
                                            "flags": { "focused": false },
                                            "test_id": "dup",
                                            "label": "Two"
                                        }
                                    ]
                                }
                            }
                        }
                    ]
                }
            ]
        });

        let report = lint_bundle_from_json(
            &bundle,
            Path::new("bundle.json"),
            0,
            LintOptions {
                all_test_ids_bounds: false,
                eps_px: 0.5,
            },
        )
        .expect("lint should succeed");

        let findings = report
            .payload
            .get("findings")
            .and_then(|v| v.as_array())
            .expect("expected findings");
        assert!(
            findings
                .iter()
                .any(|f| f.get("code").and_then(|v| v.as_str())
                    == Some("semantics.duplicate_test_id")),
            "expected duplicate test_id finding for physical-key bounds payload",
        );
    }

    #[test]
    fn lint_detects_duplicate_test_id_and_missing_active_descendant() {
        let bundle = serde_json::json!({
            "schema_version": 1,
            "windows": [
                {
                    "window": 1,
                    "snapshots": [
                        {
                            "frame_id": 10,
                            "window_bounds": { "x": 0.0, "y": 0.0, "w": 100.0, "h": 100.0 },
                            "debug": {
                                "semantics": {
                                    "window": 1,
                                    "focus": 1,
                                    "captured": null,
                                    "nodes": [
                                        {
                                            "id": 1,
                                            "parent": null,
                                            "role": "list_box",
                                            "bounds": { "x": 0.0, "y": 0.0, "w": 100.0, "h": 100.0 },
                                            "flags": { "focused": true, "captured": false, "disabled": false, "selected": false, "expanded": false, "checked": null },
                                            "test_id": "dup",
                                            "active_descendant": 999,
                                            "pos_in_set": null,
                                            "set_size": null,
                                            "label": null,
                                            "value": null,
                                            "text_selection": null,
                                            "text_composition": null,
                                            "actions": { "focus": true, "invoke": false, "set_value": false, "set_text_selection": false },
                                            "labelled_by": [],
                                            "described_by": [],
                                            "controls": []
                                        },
                                        {
                                            "id": 2,
                                            "parent": 1,
                                            "role": "list_box_option",
                                            "bounds": { "x": 0.0, "y": 0.0, "w": 100.0, "h": 20.0 },
                                            "flags": { "focused": false, "captured": false, "disabled": false, "selected": false, "expanded": false, "checked": null },
                                            "test_id": "dup",
                                            "active_descendant": null,
                                            "pos_in_set": null,
                                            "set_size": null,
                                            "label": "A",
                                            "value": null,
                                            "text_selection": null,
                                            "text_composition": null,
                                            "actions": { "focus": true, "invoke": true, "set_value": false, "set_text_selection": false },
                                            "labelled_by": [],
                                            "described_by": [],
                                            "controls": []
                                        }
                                    ]
                                }
                            }
                        }
                    ]
                }
            ]
        });

        let report = lint_bundle_from_json(
            &bundle,
            Path::new("bundle.json"),
            0,
            LintOptions {
                all_test_ids_bounds: false,
                eps_px: 0.5,
            },
        )
        .expect("lint should succeed");

        let findings = report
            .payload
            .get("findings")
            .and_then(|v| v.as_array())
            .expect("expected findings");

        assert!(
            findings
                .iter()
                .any(|f| f.get("code").and_then(|v| v.as_str())
                    == Some("semantics.duplicate_test_id")),
            "expected duplicate test_id finding"
        );
        assert!(
            findings.iter().any(|f| {
                f.get("code").and_then(|v| v.as_str())
                    == Some("semantics.active_descendant_missing")
            }),
            "expected active_descendant missing finding"
        );
    }

    #[test]
    fn lint_downgrades_scrollable_active_descendant_out_of_window_to_warning() {
        let bundle = serde_json::json!({
            "schema_version": 1,
            "windows": [
                {
                    "window": 1,
                    "snapshots": [
                        {
                            "frame_id": 10,
                            "window_bounds": { "x": 0.0, "y": 0.0, "w": 100.0, "h": 100.0 },
                            "debug": {
                                "semantics": {
                                    "window": 1,
                                    "focus": 1,
                                    "captured": null,
                                    "nodes": [
                                        {
                                            "id": 1,
                                            "parent": null,
                                            "role": "viewport",
                                            "bounds": { "x": 0.0, "y": 0.0, "w": 100.0, "h": 40.0 },
                                            "flags": { "focused": true, "captured": false, "disabled": false, "selected": false, "expanded": false, "checked": null },
                                            "test_id": "list-viewport",
                                            "active_descendant": 2,
                                            "scroll": { "y": 0.0, "y_min": 0.0, "y_max": 200.0 },
                                            "label": "List",
                                            "value": null,
                                            "actions": { "focus": true, "invoke": false, "set_value": false, "set_text_selection": false },
                                            "labelled_by": [],
                                            "described_by": [],
                                            "controls": []
                                        },
                                        {
                                            "id": 2,
                                            "parent": 1,
                                            "role": "list_box_option",
                                            "bounds": { "x": 0.0, "y": 180.0, "w": 100.0, "h": 20.0 },
                                            "flags": { "focused": false, "captured": false, "disabled": false, "selected": false, "expanded": false, "checked": null },
                                            "test_id": "item-10",
                                            "active_descendant": null,
                                            "pos_in_set": 10,
                                            "set_size": 10,
                                            "label": "Item 10",
                                            "value": null,
                                            "actions": { "focus": false, "invoke": true, "set_value": false, "set_text_selection": false },
                                            "labelled_by": [],
                                            "described_by": [],
                                            "controls": []
                                        }
                                    ]
                                }
                            }
                        }
                    ]
                }
            ]
        });

        let report = lint_bundle_from_json(
            &bundle,
            Path::new("bundle.json"),
            0,
            LintOptions {
                all_test_ids_bounds: false,
                eps_px: 0.5,
            },
        )
        .expect("lint should succeed");

        assert_eq!(report.error_issues, 0);
        let findings = report
            .payload
            .get("findings")
            .and_then(|v| v.as_array())
            .expect("expected findings");
        let active = findings
            .iter()
            .find(|f| {
                f.get("code").and_then(|v| v.as_str()) == Some("layout.active_item_out_of_window")
            })
            .expect("expected active item finding");
        assert_eq!(
            active.get("level").and_then(|v| v.as_str()),
            Some("warning")
        );
        assert_eq!(
            active
                .get("evidence")
                .and_then(|v| v.get("scrollable_ancestor"))
                .and_then(|v| v.as_bool()),
            Some(true)
        );
    }

    #[test]
    fn lint_treats_labelled_by_relation_as_accessible_name_source() {
        let bundle = serde_json::json!({
            "schema_version": 1,
            "windows": [
                {
                    "window": 1,
                    "snapshots": [
                        {
                            "frame_id": 10,
                            "window_bounds": { "x": 0.0, "y": 0.0, "w": 100.0, "h": 100.0 },
                            "debug": {
                                "semantics": {
                                    "window": 1,
                                    "focus": null,
                                    "captured": null,
                                    "nodes": [
                                        {
                                            "id": 1,
                                            "parent": null,
                                            "role": "text",
                                            "bounds": { "x": 0.0, "y": 0.0, "w": 80.0, "h": 20.0 },
                                            "flags": { "focused": false, "captured": false, "disabled": false, "selected": false, "expanded": false, "checked": null },
                                            "test_id": "country-label",
                                            "label": "Country",
                                            "value": null,
                                            "actions": { "focus": false, "invoke": false, "set_value": false, "set_text_selection": false },
                                            "labelled_by": [],
                                            "described_by": [],
                                            "controls": [2]
                                        },
                                        {
                                            "id": 2,
                                            "parent": null,
                                            "role": "combo_box",
                                            "bounds": { "x": 0.0, "y": 24.0, "w": 120.0, "h": 32.0 },
                                            "flags": { "focused": false, "captured": false, "disabled": false, "selected": false, "expanded": false, "checked": null },
                                            "test_id": "country-select",
                                            "label": null,
                                            "value": null,
                                            "actions": { "focus": true, "invoke": true, "set_value": false, "set_text_selection": false },
                                            "labelled_by": [1],
                                            "described_by": [],
                                            "controls": []
                                        }
                                    ]
                                }
                            }
                        }
                    ]
                }
            ]
        });

        let report = lint_bundle_from_json(
            &bundle,
            Path::new("bundle.json"),
            0,
            LintOptions {
                all_test_ids_bounds: false,
                eps_px: 0.5,
            },
        )
        .expect("lint should succeed");

        let findings = report
            .payload
            .get("findings")
            .and_then(|v| v.as_array())
            .expect("expected findings");
        assert!(
            findings.iter().all(|f| {
                f.get("code").and_then(|v| v.as_str()) != Some("semantics.missing_label")
            }),
            "labelled_by should satisfy the accessible-name source check"
        );
    }

    #[test]
    fn lint_ignores_hidden_state_anchors_for_visible_bounds_warnings() {
        let bundle = serde_json::json!({
            "schema_version": 1,
            "windows": [
                {
                    "window": 1,
                    "snapshots": [
                        {
                            "frame_id": 10,
                            "window_bounds": { "x": 0.0, "y": 0.0, "w": 100.0, "h": 100.0 },
                            "debug": {
                                "semantics": {
                                    "window": 1,
                                    "focus": null,
                                    "captured": null,
                                    "nodes": [
                                        {
                                            "id": 1,
                                            "parent": null,
                                            "role": "group",
                                            "bounds": { "x": 12.0, "y": 8.0, "w": 0.0, "h": 0.0 },
                                            "flags": { "focused": false, "captured": false, "hidden": true },
                                            "test_id": "state-anchor"
                                        },
                                        {
                                            "id": 2,
                                            "parent": null,
                                            "role": "button",
                                            "bounds": { "x": 12.0, "y": 16.0, "w": 0.0, "h": 0.0 },
                                            "flags": { "focused": false, "captured": false, "hidden": false },
                                            "test_id": "visible-bad-button",
                                            "label": "Visible bad button"
                                        }
                                    ]
                                }
                            }
                        }
                    ]
                }
            ]
        });

        let report = lint_bundle_from_json(
            &bundle,
            Path::new("bundle.json"),
            0,
            LintOptions {
                all_test_ids_bounds: false,
                eps_px: 0.5,
            },
        )
        .expect("lint should succeed");

        let findings = report
            .payload
            .get("findings")
            .and_then(|v| v.as_array())
            .expect("expected findings");
        assert!(
            findings
                .iter()
                .all(|f| { f.get("test_id").and_then(|v| v.as_str()) != Some("state-anchor") }),
            "hidden state anchors should stay raw-observable without visible layout warnings"
        );
        assert!(
            findings.iter().any(|f| {
                f.get("test_id").and_then(|v| v.as_str()) == Some("visible-bad-button")
                    && f.get("code").and_then(|v| v.as_str()) == Some("layout.zero_size")
            }),
            "visible zero-size test_id nodes should still be linted"
        );
    }

    #[test]
    fn lint_streaming_supports_inline_semantics() {
        let bundle = serde_json::json!({
            "schema_version": 1,
            "windows": [
                {
                    "window": 1,
                    "snapshots": [
                        {
                            "frame_id": 10,
                            "window_bounds": { "x": 0.0, "y": 0.0, "w": 100.0, "h": 100.0 },
                            "debug": {
                                "semantics": {
                                    "nodes": [
                                        {
                                            "id": 1,
                                            "parent": null,
                                            "role": "list_box",
                                            "bounds": { "x": 0.0, "y": 0.0, "w": 100.0, "h": 100.0 },
                                            "flags": { "focused": true },
                                            "test_id": "dup",
                                            "active_descendant": 999
                                        },
                                        {
                                            "id": 2,
                                            "parent": 1,
                                            "role": "list_box_option",
                                            "bounds": { "x": 0.0, "y": 0.0, "w": 100.0, "h": 20.0 },
                                            "flags": { "focused": false },
                                            "test_id": "dup",
                                            "active_descendant": null,
                                            "label": "A"
                                        }
                                    ]
                                }
                            }
                        }
                    ]
                }
            ]
        });

        let tmp_name = SystemTime::now()
            .duration_since(UNIX_EPOCH)
            .expect("time")
            .as_nanos();
        let tmp_path =
            std::env::temp_dir().join(format!("fret-diag-lint-streaming-{tmp_name}.json"));
        std::fs::write(&tmp_path, serde_json::to_vec(&bundle).expect("json")).expect("write");

        let report = streaming::lint_bundle_from_path_streaming(
            &tmp_path,
            0,
            LintOptions {
                all_test_ids_bounds: false,
                eps_px: 0.5,
            },
        )
        .expect("streaming lint should succeed");

        let _ = std::fs::remove_file(&tmp_path);

        let findings = report
            .payload
            .get("findings")
            .and_then(|v| v.as_array())
            .expect("expected findings");
        assert!(
            findings.iter().any(|f| {
                f.get("code").and_then(|v| v.as_str()) == Some("semantics.duplicate_test_id")
            }),
            "expected duplicate test_id finding"
        );
    }

    #[test]
    fn lint_streaming_supports_schema_v2_table_semantics() {
        // Keep key ordering stable for streaming readers that avoid materializing the full table entry.
        let bundle = r#"
{
  "schema_version": 2,
  "windows": [
    {
      "window": 1,
      "snapshots": [
        {
          "frame_id": 10,
          "window": 1,
          "semantics_fingerprint": 42,
          "window_bounds": { "x": 0.0, "y": 0.0, "w": 100.0, "h": 100.0 },
          "debug": {}
        }
      ]
    }
  ],
  "tables": {
    "semantics": {
      "schema_version": 1,
      "entries": [
        {
          "window": 1,
          "semantics_fingerprint": 42,
          "semantics": {
            "window": 1,
            "focus": 1,
            "nodes": [
              {
                "id": 1,
                "parent": null,
                "role": "list_box",
                "bounds": { "x": 0.0, "y": 0.0, "w": 100.0, "h": 100.0 },
                "flags": { "focused": true },
                "test_id": "dup",
                "active_descendant": 999
              },
              {
                "id": 2,
                "parent": 1,
                "role": "list_box_option",
                "bounds": { "x": 0.0, "y": 0.0, "w": 100.0, "h": 20.0 },
                "flags": { "focused": false },
                "test_id": "dup",
                "active_descendant": null,
                "label": "A"
              }
            ]
          }
        }
      ]
    }
  }
}
"#;

        let tmp_name = SystemTime::now()
            .duration_since(UNIX_EPOCH)
            .expect("time")
            .as_nanos();
        let tmp_path =
            std::env::temp_dir().join(format!("fret-diag-lint-streaming-v2-{tmp_name}.json"));
        std::fs::write(&tmp_path, bundle.as_bytes()).expect("write");

        let report = streaming::lint_bundle_from_path_streaming(
            &tmp_path,
            0,
            LintOptions {
                all_test_ids_bounds: false,
                eps_px: 0.5,
            },
        )
        .expect("streaming lint should succeed");

        let _ = std::fs::remove_file(&tmp_path);

        let findings = report
            .payload
            .get("findings")
            .and_then(|v| v.as_array())
            .expect("expected findings");
        assert!(
            findings.iter().any(|f| {
                f.get("code").and_then(|v| v.as_str()) == Some("semantics.duplicate_test_id")
            }),
            "expected duplicate test_id finding"
        );
    }

    #[test]
    fn lint_supports_schema_v2_table_semantics() {
        let bundle = serde_json::json!({
            "schema_version": 2,
            "windows": [
                {
                    "window": 1,
                    "snapshots": [
                        {
                            "frame_id": 10,
                            "window": 1,
                            "semantics_fingerprint": 42,
                            "window_bounds": { "x": 0.0, "y": 0.0, "w": 100.0, "h": 100.0 },
                            "debug": {}
                        }
                    ]
                }
            ],
            "tables": {
                "semantics": {
                    "schema_version": 1,
                    "entries": [
                        {
                            "window": 1,
                            "semantics_fingerprint": 42,
                            "semantics": {
                                "window": 1,
                                "focus": 1,
                                "captured": null,
                                "nodes": [
                                    {
                                        "id": 1,
                                        "parent": null,
                                        "role": "list_box",
                                        "bounds": { "x": 0.0, "y": 0.0, "w": 100.0, "h": 100.0 },
                                        "flags": { "focused": true, "captured": false, "disabled": false, "selected": false, "expanded": false, "checked": null },
                                        "test_id": "dup",
                                        "active_descendant": 999,
                                        "pos_in_set": null,
                                        "set_size": null,
                                        "label": null,
                                        "value": null,
                                        "text_selection": null,
                                        "text_composition": null,
                                        "actions": { "focus": true, "invoke": false, "set_value": false, "set_text_selection": false },
                                        "labelled_by": [],
                                        "described_by": [],
                                        "controls": []
                                    },
                                    {
                                        "id": 2,
                                        "parent": 1,
                                        "role": "list_box_option",
                                        "bounds": { "x": 0.0, "y": 0.0, "w": 100.0, "h": 20.0 },
                                        "flags": { "focused": false, "captured": false, "disabled": false, "selected": false, "expanded": false, "checked": null },
                                        "test_id": "dup",
                                        "active_descendant": null,
                                        "pos_in_set": null,
                                        "set_size": null,
                                        "label": "A",
                                        "value": null,
                                        "text_selection": null,
                                        "text_composition": null,
                                        "actions": { "focus": true, "invoke": true, "set_value": false, "set_text_selection": false },
                                        "labelled_by": [],
                                        "described_by": [],
                                        "controls": []
                                    }
                                ]
                            }
                        }
                    ]
                }
            }
        });

        let report = lint_bundle_from_json(
            &bundle,
            Path::new("bundle.json"),
            0,
            LintOptions {
                all_test_ids_bounds: false,
                eps_px: 0.5,
            },
        )
        .expect("lint should succeed");

        let findings = report
            .payload
            .get("findings")
            .and_then(|v| v.as_array())
            .expect("expected findings");

        assert!(
            findings
                .iter()
                .any(|f| f.get("code").and_then(|v| v.as_str())
                    == Some("semantics.duplicate_test_id")),
            "expected duplicate test_id finding"
        );
        assert!(
            findings.iter().any(|f| {
                f.get("code").and_then(|v| v.as_str())
                    == Some("semantics.active_descendant_missing")
            }),
            "expected active_descendant missing finding"
        );
    }

    #[test]
    fn lint_payload_dual_writes_bundle_artifact_alias_pair() {
        let bundle = serde_json::json!({
            "schema_version": 1,
            "windows": [
                {
                    "window": 1,
                    "snapshots": [
                        {
                            "frame_id": 10,
                            "window_bounds": { "x": 0.0, "y": 0.0, "w": 100.0, "h": 100.0 },
                            "debug": {
                                "semantics": {
                                    "window": 1,
                                    "focus": 1,
                                    "nodes": [
                                        {
                                            "id": 1,
                                            "parent": null,
                                            "role": "button",
                                            "bounds": { "x": 0.0, "y": 0.0, "w": 20.0, "h": 20.0 },
                                            "flags": { "focused": true },
                                            "test_id": "ok",
                                            "label": "Ok"
                                        }
                                    ]
                                }
                            }
                        }
                    ]
                }
            ]
        });

        let report = lint_bundle_from_json(
            &bundle,
            Path::new("target/fret-diag/bundle.schema2.json"),
            0,
            LintOptions::default(),
        )
        .expect("lint should succeed");

        assert_eq!(
            report
                .payload
                .get("bundle_artifact")
                .and_then(|v| v.as_str()),
            Some("target/fret-diag/bundle.schema2.json")
        );
        assert_eq!(
            report.payload.get("bundle_json").and_then(|v| v.as_str()),
            Some("target/fret-diag/bundle.schema2.json")
        );
    }
}
