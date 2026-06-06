use fret_app::App;
use fret_core::Px;
use fret_diag_protocol::{
    UiInspectFocusV1, UiInspectHoverV1, UiInspectNodeSummaryV1, UiInspectOverlayHookV1,
    UiOverlaySummaryV1, UiRectV1,
};
use fret_ui::ElementContext;
use fret_ui::element::AnyElement;
use fret_ui_kit::ui;

use super::{diag_section, text_blob_sized};

pub(super) fn inspect_panel(
    cx: &mut ElementContext<'_, App>,
    hover_json: &str,
    focus_json: &str,
    overlay_json: &str,
) -> AnyElement {
    let hover_bounds = text_blob_sized(
        cx,
        inspect_hover_bounds_lines(hover_json).join("\n"),
        Px(112.0),
    )
    .test_id("devtools.inspect.hover_bounds");
    let overlay_hooks = text_blob_sized(
        cx,
        inspect_overlay_hook_lines(hover_json, focus_json, overlay_json).join("\n"),
        Px(144.0),
    )
    .test_id("devtools.inspect.overlay_hooks");
    let raw_payloads = text_blob_sized(
        cx,
        inspect_raw_payload_text(hover_json, focus_json, overlay_json),
        Px(180.0),
    )
    .test_id("devtools.inspect.raw_payloads");

    let hover_section = diag_section(
        cx,
        "Live Inspect Hover Bounds",
        "Structured hovered-node bounds projected from inspect.hover.",
        vec![hover_bounds],
    );
    let overlay_section = diag_section(
        cx,
        "Live Inspect Overlay Hooks",
        "Viewport overlay hooks and overlay.summary root hints for live inspect overlays.",
        vec![overlay_hooks],
    );
    let raw_section = diag_section(
        cx,
        "Raw Inspect Payloads",
        "Raw inspect.hover, inspect.focus, and overlay.summary payloads remain available for protocol triage.",
        vec![raw_payloads],
    );

    ui::v_stack(|_cx| [hover_section, overlay_section, raw_section])
        .gap(fret_ui_kit::Space::N2)
        .layout(fret_ui_kit::LayoutRefinement::default().w_full())
        .into_element(cx)
}

pub(super) fn inspect_hover_bounds_lines(hover_json: &str) -> Vec<String> {
    let Some(payload) = parse_inspect_json::<UiInspectHoverV1>(hover_json) else {
        return vec![
            "hover: <none>".to_string(),
            "hover bounds: <none>".to_string(),
        ];
    };

    let mut lines = vec![format!("hover window={}", payload.window)];
    lines.push(inspect_rect_line("viewport", &payload.viewport_bounds));

    let Some(node) = payload.hovered else {
        lines.push("hovered node: <none>".to_string());
        lines.push("hover bounds: <none>".to_string());
        return lines;
    };

    lines.push(inspect_node_line("hovered node", &node));
    lines.push(inspect_rect_line("hover bounds", &node.bounds));
    lines.push(format!("selector_json: {}", node.selector_json));
    lines
}

pub(super) fn inspect_overlay_hook_lines(
    hover_json: &str,
    focus_json: &str,
    overlay_json: &str,
) -> Vec<String> {
    let hover = parse_inspect_json::<UiInspectHoverV1>(hover_json);
    let focus = parse_inspect_json::<UiInspectFocusV1>(focus_json);
    let overlay = parse_inspect_json::<UiOverlaySummaryV1>(overlay_json);

    let mut lines = Vec::new();
    if let Some(hover) = hover.as_ref() {
        lines.push(inspect_overlay_hook_line("hover overlay hook", &hover.overlay_hook));
    } else {
        lines.push("hover overlay hook: <none>".to_string());
    }
    if let Some(focus) = focus.as_ref() {
        lines.push(inspect_overlay_hook_line("focus overlay hook", &focus.overlay_hook));
        if let Some(summary) = focus.summary.as_deref() {
            lines.push(format!("focus summary: {summary}"));
        }
        if let Some(path) = focus.path.as_deref() {
            lines.push(format!("focus path: {path}"));
        }
    } else {
        lines.push("focus overlay hook: <none>".to_string());
    }

    if let Some(overlay) = overlay {
        lines.push(format!(
            "overlay barrier root: {}",
            inspect_opt_u64(overlay.barrier_root)
        ));
        lines.push(format!(
            "overlay focus barrier root: {}",
            inspect_opt_u64(overlay.focus_barrier_root)
        ));
        lines.push(format!(
            "overlay blocking roots: {}",
            overlay.blocking_roots.len()
        ));
        for root in overlay.blocking_roots.iter().take(4) {
            lines.push(format!(
                "blocking root={} z={} visible={} hit_testable={}",
                root.root, root.z_index, root.visible, root.hit_testable
            ));
        }
        if let Some(root) = overlay.topmost_interactive_root {
            lines.push(format!(
                "topmost interactive root={} z={} blocks_underlay_input={}",
                root.root, root.z_index, root.blocks_underlay_input
            ));
        } else {
            lines.push("topmost interactive root: <none>".to_string());
        }
    } else {
        lines.push("overlay summary: <none>".to_string());
    }

    lines
}

fn inspect_raw_payload_text(hover_json: &str, focus_json: &str, overlay_json: &str) -> String {
    if hover_json.trim().is_empty() && focus_json.trim().is_empty() && overlay_json.trim().is_empty()
    {
        return String::new();
    }
    format!("hover:\n{hover_json}\n\nfocus:\n{focus_json}\n\noverlay.summary:\n{overlay_json}")
}

fn parse_inspect_json<T: serde::de::DeserializeOwned>(text: &str) -> Option<T> {
    if text.trim().is_empty() {
        return None;
    }
    serde_json::from_str(text).ok()
}

fn inspect_node_line(label: &str, node: &UiInspectNodeSummaryV1) -> String {
    let test_id = node.test_id.as_deref().unwrap_or("<none>");
    let root = node
        .root
        .map(|root| root.to_string())
        .unwrap_or_else(|| "<none>".to_string());
    let root_z = node
        .root_z_index
        .map(|z| z.to_string())
        .unwrap_or_else(|| "<none>".to_string());
    format!(
        "{label}: node={} role={} test_id={} root={} root_z_index={}",
        node.node_id, node.role, test_id, root, root_z
    )
}

fn inspect_rect_line(label: &str, rect: &UiRectV1) -> String {
    format!(
        "{label}: x={:.1} y={:.1} w={:.1} h={:.1}",
        rect.x_px, rect.y_px, rect.w_px, rect.h_px
    )
}

fn inspect_overlay_hook_line(label: &str, hook: &UiInspectOverlayHookV1) -> String {
    let target = hook
        .target_node_id
        .map(|node| node.to_string())
        .unwrap_or_else(|| "<none>".to_string());
    let bounds = hook
        .target_bounds
        .as_ref()
        .map(|rect| inspect_rect_line("target bounds", rect))
        .unwrap_or_else(|| "target bounds: <none>".to_string());
    format!(
        "{label}: kind={} space={} target_node={} {} {}",
        hook.kind,
        hook.coordinate_space,
        target,
        bounds,
        inspect_rect_line("viewport", &hook.viewport_bounds)
    )
}

fn inspect_opt_u64(value: Option<u64>) -> String {
    value
        .map(|value| value.to_string())
        .unwrap_or_else(|| "<none>".to_string())
}
