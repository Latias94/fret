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

    if let Some(pos) = pointer_pos {
        let hit = ui.debug_hit_test_routing(pos);
        let hit_node = hit.hit.map(|id| id.data().as_ffi());
        let hit_barrier = hit.barrier_root.map(|id| id.data().as_ffi());
        lines.push(format!(
            "pointer: {pos:?} hit_node={:?} barrier_root={:?}",
            hit_node, hit_barrier
        ));

        if !hit.active_layer_roots.is_empty() {
            let roots = hit
                .active_layer_roots
                .iter()
                .map(|id| id.data().as_ffi().to_string())
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
    } else {
        "unblocked_or_hit_test_specific"
    };
    lines.push(format!("likely_reason: {likely_reason}"));

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
