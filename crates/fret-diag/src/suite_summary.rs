use std::collections::BTreeMap;
use std::path::Path;

use fret_diag_protocol::{UiOverlayPlacementTraceEntryV1, UiOverlaySideV1, UiScriptResultV1};

#[derive(Debug, Default, Clone)]
pub(super) struct SuiteEvidenceAggregate {
    pub(super) scripts_with_evidence: u64,
    pub(super) scripts_with_focus_mismatch: u64,
    pub(super) focus_mismatch_total: u64,
    pub(super) blocking_reason_counts: BTreeMap<String, u64>,
    pub(super) overlay_chosen_side_counts: BTreeMap<String, u64>,
    pub(super) ime_event_kind_counts: BTreeMap<String, u64>,
}

impl SuiteEvidenceAggregate {
    pub(super) fn as_json(&self) -> serde_json::Value {
        serde_json::json!({
            "scripts_with_evidence": self.scripts_with_evidence,
            "scripts_with_focus_mismatch": self.scripts_with_focus_mismatch,
            "focus_mismatch_total": self.focus_mismatch_total,
            "blocking_reason_counts": self.blocking_reason_counts,
            "overlay_chosen_side_counts": self.overlay_chosen_side_counts,
            "ime_event_kind_counts": self.ime_event_kind_counts,
        })
    }
}

fn push_count(map: &mut BTreeMap<String, u64>, key: &str) {
    if key.trim().is_empty() {
        return;
    }
    *map.entry(key.to_string()).or_default() += 1;
}

fn overlay_side_as_str(side: UiOverlaySideV1) -> &'static str {
    match side {
        UiOverlaySideV1::Top => "top",
        UiOverlaySideV1::Bottom => "bottom",
        UiOverlaySideV1::Left => "left",
        UiOverlaySideV1::Right => "right",
    }
}

fn read_script_result_typed(path: &Path) -> Option<UiScriptResultV1> {
    let bytes = std::fs::read(path).ok()?;
    serde_json::from_slice::<UiScriptResultV1>(&bytes).ok()
}

pub(super) fn evidence_highlights_from_script_result_path(
    script_result_path: &Path,
    agg: &mut SuiteEvidenceAggregate,
) -> Option<serde_json::Value> {
    let result = read_script_result_typed(script_result_path)?;
    let e = result.evidence.as_ref()?;

    let mut trace_counts = BTreeMap::<&'static str, u64>::new();
    trace_counts.insert(
        "selector_resolution_trace",
        e.selector_resolution_trace.len() as u64,
    );
    trace_counts.insert("hit_test_trace", e.hit_test_trace.len() as u64);
    trace_counts.insert("click_stable_trace", e.click_stable_trace.len() as u64);
    trace_counts.insert("bounds_stable_trace", e.bounds_stable_trace.len() as u64);
    trace_counts.insert("focus_trace", e.focus_trace.len() as u64);
    trace_counts.insert(
        "shortcut_routing_trace",
        e.shortcut_routing_trace.len() as u64,
    );
    trace_counts.insert(
        "overlay_placement_trace",
        e.overlay_placement_trace.len() as u64,
    );
    trace_counts.insert("web_ime_trace", e.web_ime_trace.len() as u64);
    trace_counts.insert("ime_event_trace", e.ime_event_trace.len() as u64);

    let evidence_present = trace_counts.values().any(|&n| n > 0);
    if evidence_present {
        agg.scripts_with_evidence += 1;
    }

    let mut hit_test_blocking = BTreeMap::<String, u64>::new();
    for entry in &e.hit_test_trace {
        if let Some(reason) = entry.blocking_reason.as_deref() {
            push_count(&mut hit_test_blocking, reason);
            push_count(&mut agg.blocking_reason_counts, reason);
        }
    }

    let mut mismatch_count: u64 = 0;
    let mut text_input_snapshots: u64 = 0;
    let mut composing_true: u64 = 0;
    for entry in &e.focus_trace {
        if entry.matches_expected == Some(false) {
            mismatch_count += 1;
        }
        if let Some(snap) = entry.text_input_snapshot.as_ref() {
            text_input_snapshots += 1;
            if snap.is_composing {
                composing_true += 1;
            }
        }
    }
    if mismatch_count > 0 {
        agg.scripts_with_focus_mismatch += 1;
        agg.focus_mismatch_total += mismatch_count;
    }

    let mut overlay_chosen_sides = BTreeMap::<String, u64>::new();
    let mut anchored_panel_count: u64 = 0;
    for entry in &e.overlay_placement_trace {
        if let UiOverlayPlacementTraceEntryV1::AnchoredPanel { chosen_side, .. } = entry {
            anchored_panel_count += 1;
            let side = overlay_side_as_str(*chosen_side);
            push_count(&mut overlay_chosen_sides, side);
            push_count(&mut agg.overlay_chosen_side_counts, side);
        }
    }

    let mut ime_event_kinds = BTreeMap::<String, u64>::new();
    for entry in &e.ime_event_trace {
        push_count(&mut ime_event_kinds, &entry.kind);
        push_count(&mut agg.ime_event_kind_counts, &entry.kind);
    }

    Some(serde_json::json!({
        "evidence_present": evidence_present,
        "trace_counts": trace_counts,
        "hit_test": {
            "blocking_reason_counts": hit_test_blocking,
        },
        "focus": {
            "mismatch_count": mismatch_count,
            "text_input_snapshots": text_input_snapshots,
            "composing_true": composing_true,
        },
        "overlay": {
            "anchored_panel_count": anchored_panel_count,
            "chosen_side_counts": overlay_chosen_sides,
        },
        "ime": {
            "event_kind_counts": ime_event_kinds,
        }
    }))
}
