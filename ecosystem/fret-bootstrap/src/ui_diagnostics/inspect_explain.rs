use fret_app::App;
use fret_core::{AppWindowId, Point};
use fret_ui::UiTree;
use fret_ui::tree::PointerOcclusion;
use slotmap::Key as _;

pub(super) fn build_inspect_explainability_lines(
    ui: &mut UiTree<App>,
    window: AppWindowId,
    pointer_pos: Option<Point>,
) -> Vec<String> {
    let _ = window;

    let mut lines: Vec<String> = Vec::new();
    lines.push("explain:".to_string());

    let arbitration = ui.input_arbitration_snapshot();
    let layer_roots = ui
        .debug_layers_in_paint_order()
        .into_iter()
        .map(|layer| (layer.id, layer.root))
        .collect::<std::collections::HashMap<_, _>>();

    let mut hit_node: Option<u64> = None;

    if let Some(pos) = pointer_pos {
        let hit = ui.debug_hit_test_routing(pos);
        hit_node = hit.hit.map(|id| id.data().as_ffi());
        let hit_barrier_root = hit.barrier_root.map(|id| id.data().as_ffi());
        let active_layer_roots = hit
            .active_layer_roots
            .iter()
            .map(|id| id.data().as_ffi())
            .collect::<Vec<_>>();
        lines.push(format!(
            "pointer: {pos:?} hit_node={:?} barrier_root={:?}",
            hit_node, hit_barrier_root
        ));

        if !active_layer_roots.is_empty() {
            let roots = active_layer_roots
                .iter()
                .map(|id| id.to_string())
                .collect::<Vec<_>>()
                .join(", ");
            lines.push(format!("active_layer_roots: [{roots}]"));
        }
    } else {
        lines.push("pointer: <unknown>".to_string());
    }

    let pointer_occlusion_root = arbitration
        .pointer_occlusion_layer
        .and_then(|layer| layer_roots.get(&layer).copied())
        .map(|id| id.data().as_ffi());
    let pointer_capture_root = arbitration
        .pointer_capture_layer
        .and_then(|layer| layer_roots.get(&layer).copied())
        .map(|id| id.data().as_ffi());
    lines.push(format!(
        "arbitration: modal_barrier_root={:?} focus_barrier_root={:?} pointer_occlusion={:?} pointer_occlusion_root={:?} pointer_capture_active={} pointer_capture_root={:?} pointer_capture_multiple_layers={}",
        arbitration.modal_barrier_root.map(|id| id.data().as_ffi()),
        arbitration.focus_barrier_root.map(|id| id.data().as_ffi()),
        arbitration.pointer_occlusion,
        pointer_occlusion_root,
        arbitration.pointer_capture_active,
        pointer_capture_root,
        arbitration.pointer_capture_multiple_layers
    ));

    let likely_reason = if arbitration.pointer_capture_active {
        "pointer_capture"
    } else if !matches!(arbitration.pointer_occlusion, PointerOcclusion::None) {
        "pointer_occlusion"
    } else if arbitration.modal_barrier_root.is_some() {
        "modal_barrier_active"
    } else if pointer_pos.is_none() {
        "pointer_unknown"
    } else if hit_node.is_none() {
        "no_hit_target"
    } else {
        "unblocked_or_hit_test_specific"
    };
    lines.push(format!("likely_reason: {likely_reason}"));

    let mut suggestions: Vec<String> = Vec::new();
    if arbitration.pointer_capture_active {
        suggestions.push(
            "suggest: a pointer is captured; inspect pointer_capture_root and ensure the app receives a matching pointer-up to release capture".to_string(),
        );
    } else if !matches!(arbitration.pointer_occlusion, PointerOcclusion::None) {
        suggestions.push(format!(
            "suggest: underlay is occluded; inspect layer root {:?} and its pointer_occlusion/blocks_underlay_input flags",
            pointer_occlusion_root
        ));
    } else if arbitration.modal_barrier_root.is_some() {
        suggestions.push(
            "suggest: a modal barrier is active; close the modal/overlay or inspect modal_barrier_root reachability"
                .to_string(),
        );
    } else if hit_node.is_none() && pointer_pos.is_some() {
        suggestions.push(
            "suggest: no hit-test target under pointer; check layer visibility/hit_testable flags and whether the pointer is inside any bounds"
                .to_string(),
        );
    }

    if suggestions.is_empty() && arbitration.focus_barrier_root.is_some() {
        suggestions.push(
            "suggest: focus barrier active; keyboard focus may not reach underlay until the barrier is cleared"
                .to_string(),
        );
    }

    if !suggestions.is_empty() {
        lines.push(String::new());
        lines.push("suggestions:".to_string());
        lines.extend(suggestions.into_iter().take(3));
    }

    if let Some(snapshot) = ui.semantics_snapshot() {
        let barrier_root = snapshot.barrier_root.map(|id| id.data().as_ffi());
        let focus_barrier_root = snapshot.focus_barrier_root.map(|id| id.data().as_ffi());
        lines.push(format!(
            "semantics: barrier_root={:?} focus_barrier_root={:?}",
            barrier_root, focus_barrier_root
        ));

        let mut roots = snapshot
            .roots
            .iter()
            .filter(|r| r.visible)
            .map(|r| (r.z_index, r))
            .collect::<Vec<_>>();
        roots.sort_by(|(a, _), (b, _)| b.cmp(a));

        if !roots.is_empty() {
            lines.push("visible_roots (topmost first):".to_string());
            for (_, root) in roots.into_iter().take(8) {
                lines.push(format!(
                    "- root={} z={} blocks_underlay_input={} hit_testable={}",
                    root.root.data().as_ffi(),
                    root.z_index,
                    root.blocks_underlay_input,
                    root.hit_testable
                ));
            }
        }
    } else {
        lines.push("semantics: <none>".to_string());
    }

    let layers = ui.debug_layers_in_paint_order();
    if !layers.is_empty() {
        lines.push("layers (topmost first):".to_string());
        for layer in layers.into_iter().rev().take(8) {
            lines.push(format!(
                "- layer={:?} root={} visible={} blocks_underlay_input={} hit_testable={} pointer_occlusion={:?}",
                layer.id,
                layer.root.data().as_ffi(),
                layer.visible,
                layer.blocks_underlay_input,
                layer.hit_testable,
                layer.pointer_occlusion
            ));
        }
    }

    lines
}
