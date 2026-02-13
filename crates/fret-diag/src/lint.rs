use serde_json::Value;
use std::collections::HashMap;
use std::path::Path;

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
    Some(RectF64 {
        x: v.get("x")?.as_f64()?,
        y: v.get("y")?.as_f64()?,
        w: v.get("w")?.as_f64()?,
        h: v.get("h")?.as_f64()?,
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

fn pick_last_snapshot_after_warmup<'a>(
    snaps: &'a [Value],
    warmup_frames: u64,
) -> Option<&'a Value> {
    snaps
        .iter()
        .rev()
        .find(|s| s.get("frame_id").and_then(|v| v.as_u64()).unwrap_or(0) >= warmup_frames)
        .or_else(|| snaps.last())
}

pub(super) fn lint_bundle_from_path(
    bundle_path: &Path,
    warmup_frames: u64,
    opts: LintOptions,
) -> Result<LintReport, String> {
    let bytes = std::fs::read(bundle_path).map_err(|e| e.to_string())?;
    let bundle: Value = serde_json::from_slice(&bytes).map_err(|e| e.to_string())?;
    lint_bundle_from_json(&bundle, bundle_path, warmup_frames, opts)
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
        .ok_or_else(|| "invalid bundle.json: missing windows".to_string())?;

    let mut findings: Vec<Value> = Vec::new();

    for w in windows {
        let window_id = w.get("window").and_then(|v| v.as_u64()).unwrap_or(0);
        let snaps = w
            .get("snapshots")
            .and_then(|v| v.as_array())
            .map_or(&[][..], |v| v);
        let Some(snapshot) = pick_last_snapshot_after_warmup(snaps, warmup_frames) else {
            continue;
        };

        let frame_id = snapshot
            .get("frame_id")
            .and_then(|v| v.as_u64())
            .unwrap_or(0);
        let window_bounds = snapshot
            .get("window_bounds")
            .and_then(rect_from_bounds)
            .unwrap_or(RectF64 {
                x: 0.0,
                y: 0.0,
                w: 0.0,
                h: 0.0,
            });

        let semantics = snapshot
            .get("debug")
            .and_then(|v| v.get("semantics"))
            .and_then(|v| v.as_object());
        let Some(semantics) = semantics else {
            continue;
        };

        let focus = semantics.get("focus").and_then(|v| v.as_u64());

        let nodes = semantics
            .get("nodes")
            .and_then(|v| v.as_array())
            .map_or(&[][..], |v| v);
        if nodes.is_empty() {
            continue;
        }

        let mut by_id: HashMap<u64, &Value> = HashMap::new();
        let mut test_id_to_nodes: HashMap<&str, Vec<u64>> = HashMap::new();

        for n in nodes {
            let Some(id) = n.get("id").and_then(|v| v.as_u64()) else {
                continue;
            };
            by_id.insert(id, n);
            if let Some(test_id) = n.get("test_id").and_then(|v| v.as_str()) {
                if !test_id.trim().is_empty() {
                    test_id_to_nodes.entry(test_id).or_default().push(id);
                }
            }
        }

        for (test_id, ids) in test_id_to_nodes.iter() {
            if ids.len() <= 1 {
                continue;
            }
            let mut ids_sorted = ids.clone();
            ids_sorted.sort_unstable();
            push_finding(
                &mut findings,
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
            if let Some(active_descendant) = active_descendant {
                if !by_id.contains_key(&active_descendant) {
                    push_finding(
                        &mut findings,
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
            }

            let label = n.get("label").and_then(|v| v.as_str());
            let value = n.get("value").and_then(|v| v.as_str());
            let role_str = role.as_deref().unwrap_or("");
            if role_requires_label(role_str) && label.is_none() && value.is_none() {
                push_finding(
                    &mut findings,
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

            let is_focused = focus == Some(id);

            if !rect_is_non_empty(bounds, eps) {
                let level = if is_focused {
                    LintLevel::Error
                } else {
                    LintLevel::Warning
                };
                if test_id.is_some() || is_focused {
                    push_finding(
                        &mut findings,
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

            if is_focused && !rects_intersect(bounds, window_bounds, eps) {
                push_finding(
                    &mut findings,
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
                        "window_bounds": snapshot.get("window_bounds").cloned().unwrap_or(Value::Null),
                    }),
                );
            }

            if opts.all_test_ids_bounds
                && test_id.is_some()
                && !rects_intersect(bounds, window_bounds, eps)
            {
                push_finding(
                    &mut findings,
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
                        "window_bounds": snapshot.get("window_bounds").cloned().unwrap_or(Value::Null),
                    }),
                );
            }

            if is_focused {
                if let Some(active) = active_descendant {
                    if let Some(active_node) = by_id.get(&active) {
                        if let Some(active_bounds) =
                            active_node.get("bounds").and_then(rect_from_bounds)
                        {
                            if !rects_intersect(active_bounds, window_bounds, eps) {
                                push_finding(
                                    &mut findings,
                                    LintLevel::Error,
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
                                        "window_bounds": snapshot.get("window_bounds").cloned().unwrap_or(Value::Null),
                                    }),
                                );
                            }
                        }
                    }
                }
            }
        }
    }

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

    let payload = serde_json::json!({
        "schema_version": 1,
        "kind": "lint",
        "bundle_json": bundle_path.display().to_string(),
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

    Ok(LintReport {
        error_issues,
        payload,
    })
}

#[cfg(test)]
mod tests {
    use super::*;

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
}
