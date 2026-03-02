use super::*;

pub(super) const MAX_SELECTOR_TRACE_ENTRIES: usize = 64;
pub(super) const MAX_SELECTOR_TRACE_CANDIDATES: usize = 6;
pub(super) const MAX_HIT_TEST_TRACE_ENTRIES: usize = 64;
pub(super) const MAX_FOCUS_TRACE_ENTRIES: usize = 64;
pub(super) const MAX_SHORTCUT_ROUTING_TRACE_ENTRIES: usize = 128;
pub(super) const MAX_OVERLAY_PLACEMENT_TRACE_ENTRIES: usize = 128;
pub(super) const MAX_WEB_IME_TRACE_ENTRIES: usize = 64;
pub(super) const MAX_IME_EVENT_TRACE_ENTRIES: usize = 64;
pub(super) const MAX_BOUNDS_STABLE_TRACE_ENTRIES: usize = 32;
pub(super) const MAX_CLICK_STABLE_TRACE_ENTRIES: usize = 32;

fn selector_trace_eq(a: &UiSelectorV1, b: &UiSelectorV1) -> bool {
    match (a, b) {
        (
            UiSelectorV1::RoleAndName {
                role: a_role,
                name: a_name,
                root_z_index: a_root_z,
            },
            UiSelectorV1::RoleAndName {
                role: b_role,
                name: b_name,
                root_z_index: b_root_z,
            },
        ) => a_role == b_role && a_name == b_name && a_root_z == b_root_z,
        (
            UiSelectorV1::RoleAndPath {
                role: a_role,
                name: a_name,
                ancestors: a_ancestors,
                root_z_index: a_root_z,
            },
            UiSelectorV1::RoleAndPath {
                role: b_role,
                name: b_name,
                ancestors: b_ancestors,
                root_z_index: b_root_z,
            },
        ) => {
            a_role == b_role
                && a_name == b_name
                && a_root_z == b_root_z
                && a_ancestors.len() == b_ancestors.len()
                && a_ancestors
                    .iter()
                    .zip(b_ancestors.iter())
                    .all(|(a, b)| a.role == b.role && a.name == b.name)
        }
        (
            UiSelectorV1::TestId {
                id: a_id,
                root_z_index: a_root_z,
            },
            UiSelectorV1::TestId {
                id: b_id,
                root_z_index: b_root_z,
            },
        ) => a_id == b_id && a_root_z == b_root_z,
        (
            UiSelectorV1::GlobalElementId {
                element: a_el,
                root_z_index: a_root_z,
            },
            UiSelectorV1::GlobalElementId {
                element: b_el,
                root_z_index: b_root_z,
            },
        ) => a_el == b_el && a_root_z == b_root_z,
        (
            UiSelectorV1::NodeId {
                node: a_node,
                root_z_index: a_root_z,
            },
            UiSelectorV1::NodeId {
                node: b_node,
                root_z_index: b_root_z,
            },
        ) => a_node == b_node && a_root_z == b_root_z,
        _ => false,
    }
}

fn hit_test_trace_entry_eq(a: &UiHitTestTraceEntryV1, b: &UiHitTestTraceEntryV1) -> bool {
    a.step_index == b.step_index
        && selector_trace_eq(&a.selector, &b.selector)
        && a.note == b.note
        && a.position.x_px == b.position.x_px
        && a.position.y_px == b.position.y_px
}

pub(super) fn push_selector_resolution_trace(
    trace: &mut Vec<UiSelectorResolutionTraceEntryV1>,
    entry: UiSelectorResolutionTraceEntryV1,
) {
    if let Some(existing) = trace.iter_mut().rev().find(|e| {
        e.step_index == entry.step_index && selector_trace_eq(&e.selector, &entry.selector)
    }) {
        *existing = entry;
        return;
    }

    trace.push(entry);
    if trace.len() > MAX_SELECTOR_TRACE_ENTRIES {
        let extra = trace.len().saturating_sub(MAX_SELECTOR_TRACE_ENTRIES);
        trace.drain(0..extra);
    }
}

pub(super) fn push_hit_test_trace(
    trace: &mut Vec<UiHitTestTraceEntryV1>,
    entry: UiHitTestTraceEntryV1,
) {
    if let Some(existing) = trace
        .iter_mut()
        .rev()
        .find(|e| hit_test_trace_entry_eq(e, &entry))
    {
        *existing = entry;
        return;
    }
    trace.push(entry);
    if trace.len() > MAX_HIT_TEST_TRACE_ENTRIES {
        let extra = trace.len().saturating_sub(MAX_HIT_TEST_TRACE_ENTRIES);
        trace.drain(0..extra);
    }
}

fn bounds_stable_trace_entry_eq(
    a: &UiBoundsStableTraceEntryV1,
    b: &UiBoundsStableTraceEntryV1,
) -> bool {
    a.step_index == b.step_index && selector_trace_eq(&a.selector, &b.selector)
}

pub(super) fn push_bounds_stable_trace(
    trace: &mut Vec<UiBoundsStableTraceEntryV1>,
    entry: UiBoundsStableTraceEntryV1,
) {
    if let Some(existing) = trace
        .iter_mut()
        .rev()
        .find(|e| bounds_stable_trace_entry_eq(e, &entry))
    {
        *existing = entry;
        return;
    }
    trace.push(entry);
    if trace.len() > MAX_BOUNDS_STABLE_TRACE_ENTRIES {
        let extra = trace.len().saturating_sub(MAX_BOUNDS_STABLE_TRACE_ENTRIES);
        trace.drain(0..extra);
    }
}
