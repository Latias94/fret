use fret_app::App;
use fret_app::Effect;
use fret_core::AppWindowId;

use super::UiDiagnosticsService;
use super::selector::SemanticsIndex;

#[cfg(feature = "diagnostics")]
pub(crate) fn render_diag_inspect_overlay(
    ui: &mut fret_ui::UiTree<App>,
    app: &mut App,
    services: &mut dyn fret_core::UiServices,
    window: AppWindowId,
    bounds: fret_core::Rect,
    inspection_active: bool,
) {
    use slotmap::Key as _;

    const ROOT_NAME: &str = "__diag_inspect";

    let pending_copy_payload = app
        .with_global_mut_untracked(UiDiagnosticsService::default, |svc, _app| {
            svc.inspect_pending_copy_details_payload.remove(&window)
        });
    if let Some(text) = pending_copy_payload {
        app.push_effect(Effect::ClipboardSetText { text });
    }

    if !inspection_active {
        let root_node = fret_ui::declarative::render_root(
            ui,
            app,
            services,
            window,
            bounds,
            ROOT_NAME,
            |_cx| Vec::new(),
        );
        let layer = ui
            .node_layer(root_node)
            .unwrap_or_else(|| ui.push_overlay_root_ex(root_node, false, false));
        ui.set_layer_visible(layer, false);
        ui.set_layer_hit_testable(layer, false);
        ui.set_layer_wants_pointer_down_outside_events(layer, false);
        ui.set_layer_wants_pointer_move_events(layer, false);
        ui.set_layer_wants_timer_events(layer, false);
        return;
    }

    let (
        pointer_pos,
        picked_node_id,
        focus_node_id,
        redact_text,
        pick_armed,
        pick_pending,
        inspect_enabled,
        help_open,
        consume_clicks,
        locked,
    ) = app.with_global_mut_untracked(UiDiagnosticsService::default, |svc, _app| {
        (
            svc.last_pointer_position(window),
            svc.last_picked_node_id(window),
            svc.inspect_focus_node_id(window),
            svc.redact_text(),
            svc.pick_is_armed(),
            svc.pick_is_pending(window),
            svc.inspect_is_enabled(),
            svc.inspect_help_is_open(window),
            svc.inspect_consume_clicks(),
            svc.inspect_is_locked(window),
        )
    });

    let explainability_lines: Vec<String> = if help_open {
        let mut lines: Vec<String> = Vec::new();
        lines.push("explain:".to_string());

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
    } else {
        Vec::new()
    };

    struct InspectNodeInfo {
        bounds: fret_core::Rect,
        role: fret_core::SemanticsRole,
        root_z_index: u32,
        node_id: u64,
        test_id: Option<String>,
        label: Option<String>,
    }

    let snapshot = ui.semantics_snapshot();
    let (hovered, picked, focus) = if let Some(snapshot) = snapshot {
        let index = SemanticsIndex::new(snapshot);

        let hovered = pointer_pos.and_then(|pos| {
            let node = super::pick::pick_semantics_node_at(snapshot, ui, pos)?;
            let id = node.id.data().as_ffi();
            Some(InspectNodeInfo {
                bounds: node.bounds,
                role: node.role,
                root_z_index: index.root_z_for(id),
                node_id: id,
                test_id: node.test_id.clone(),
                label: node.label.clone(),
            })
        });

        let picked = picked_node_id
            .and_then(|id| snapshot.nodes.iter().find(|n| n.id.data().as_ffi() == id))
            .map(|node| InspectNodeInfo {
                bounds: node.bounds,
                role: node.role,
                root_z_index: index.root_z_for(node.id.data().as_ffi()),
                node_id: node.id.data().as_ffi(),
                test_id: node.test_id.clone(),
                label: node.label.clone(),
            });

        let focus = focus_node_id
            .and_then(|id| snapshot.nodes.iter().find(|n| n.id.data().as_ffi() == id))
            .map(|node| InspectNodeInfo {
                bounds: node.bounds,
                role: node.role,
                root_z_index: index.root_z_for(node.id.data().as_ffi()),
                node_id: node.id.data().as_ffi(),
                test_id: node.test_id.clone(),
                label: node.label.clone(),
            });

        (hovered, picked, focus)
    } else {
        (None, None, None)
    };

    let hovered = if locked || !(pick_armed || inspect_enabled) {
        None
    } else {
        hovered
    };

    let (toast, best_selector) =
        app.with_global_mut_untracked(UiDiagnosticsService::default, |svc, _app| {
            (
                svc.inspect_toast_message(window).map(|s| s.to_string()),
                svc.inspect_best_selector_json(window)
                    .map(|s| s.to_string()),
            )
        });

    let (focus_summary, focus_path) =
        app.with_global_mut_untracked(UiDiagnosticsService::default, |svc, _app| {
            (
                svc.inspect_focus_summary_line(window)
                    .map(|s| s.to_string()),
                svc.inspect_focus_path_line(window).map(|s| s.to_string()),
            )
        });

    let root_node = fret_ui::declarative::render_root(
        ui,
        app,
        services,
        window,
        bounds,
        ROOT_NAME,
        move |cx| {
            use fret_core::{Color, Corners, Edges, Px};
            use fret_ui::element::{
                ContainerProps, InsetStyle, LayoutStyle, Length, PositionStyle, SizeStyle,
            };

            let mut children = Vec::new();

            if pick_armed
                || pick_pending
                || inspect_enabled
                || help_open
                || toast.is_some()
                || best_selector.is_some()
                || focus_summary.is_some()
                || focus_path.is_some()
            {
                let mut layout = LayoutStyle::default();
                layout.position = PositionStyle::Absolute;
                layout.inset = InsetStyle {
                    top: Some(Px(8.0)).into(),
                    left: Some(Px(8.0)).into(),
                    ..Default::default()
                };

                let mut props = ContainerProps::default();
                props.layout = layout;
                props.padding = Edges::all(Px(6.0)).into();
                props.background = Some(Color {
                    r: 0.0,
                    g: 0.0,
                    b: 0.0,
                    a: 0.65,
                });
                props.corner_radii = Corners::all(Px(6.0));
                props.border = Edges::all(Px(1.0));
                props.border_color = Some(Color {
                    r: 0.2,
                    g: 0.8,
                    b: 1.0,
                    a: 1.0,
                });

                let mut lines: Vec<String> = Vec::new();
                if pick_armed || pick_pending {
                    lines.push(
                        "INSPECT: click to pick a target (Esc to cancel, Ctrl+Alt+H help)"
                            .to_string(),
                    );
                } else {
                    if help_open {
                        lines.push(format!(
                            "INSPECT (enabled={inspect_enabled}, consume_clicks={consume_clicks}, locked={locked})"
                        ));
                        lines.push("Ctrl/Cmd+Alt+I: toggle inspect".to_string());
                        lines.push("Ctrl/Cmd+Alt+H: toggle help".to_string());
                        lines.push("Esc: exit inspect / disarm pick".to_string());
                        lines.push("Ctrl/Cmd+C: copy best selector".to_string());
                        lines.push("Ctrl/Cmd+Shift+C: copy inspect details".to_string());
                        lines.push("F: select focused node".to_string());
                        lines.push("L: lock/unlock selection".to_string());
                        lines.push("Alt+Up/Down: navigate parent chain (locked)".to_string());

                        if !explainability_lines.is_empty() {
                            lines.push(String::new());
                            lines.extend(explainability_lines.iter().cloned());
                        }
                    } else {
                        lines.push(format!(
                            "INSPECT: Ctrl+Alt+I toggle | Ctrl+Alt+H help | Esc exit | Ctrl+C copy selector | F focus | L lock | Alt+Up/Down nav (consume_clicks={consume_clicks}, locked={locked})"
                        ));
                    }
                }
                if let Some(t) = toast.as_deref() {
                    lines.push(format!("status: {t}"));
                }
                if let Some(summary) = focus_summary.as_deref() {
                    lines.push(summary.to_string());
                }
                if let Some(path) = focus_path.as_deref() {
                    lines.push(path.to_string());
                }
                if let Some(sel) = best_selector.as_deref() {
                    let trimmed = if sel.len() > 180 {
                        format!("{}…", &sel[..180])
                    } else {
                        sel.to_string()
                    };
                    lines.push(format!("selector: {trimmed}"));
                }

                children.push(cx.container(props, |cx| {
                    lines.into_iter().map(|t| cx.text(t)).collect::<Vec<_>>()
                }));
            }

            let show_focus = focus.as_ref().is_some_and(|f| {
                picked.as_ref().is_none_or(|p| p.node_id != f.node_id)
                    && hovered.as_ref().is_none_or(|h| h.node_id != f.node_id)
            });
            let focus_outline = if show_focus { focus } else { None };

            let outlines = [
                (
                    focus_outline,
                    Color {
                        r: 0.2,
                        g: 0.8,
                        b: 1.0,
                        a: 1.0,
                    },
                    "focus",
                ),
                (
                    picked,
                    Color {
                        r: 1.0,
                        g: 0.2,
                        b: 1.0,
                        a: 1.0,
                    },
                    "picked",
                ),
                (
                    hovered,
                    Color {
                        r: 0.2,
                        g: 1.0,
                        b: 0.4,
                        a: 1.0,
                    },
                    "hovered",
                ),
            ];

            for (node, color, tag) in outlines {
                let Some(node) = node else {
                    continue;
                };

                let rect = node.bounds;
                let mut layout = LayoutStyle::default();
                layout.position = PositionStyle::Absolute;
                layout.inset = InsetStyle {
                    top: Some(rect.origin.y).into(),
                    left: Some(rect.origin.x).into(),
                    ..Default::default()
                };
                layout.size = SizeStyle {
                    width: Length::Px(rect.size.width),
                    height: Length::Px(rect.size.height),
                    ..Default::default()
                };

                let mut props = ContainerProps::default();
                props.layout = layout;
                props.border = Edges::all(Px(1.0));
                props.border_color = Some(color);
                props.corner_radii = Corners::all(Px(2.0));

                children.push(cx.container(props, |_cx| Vec::new()));

                let mut label_layout = LayoutStyle::default();
                label_layout.position = PositionStyle::Absolute;
                label_layout.inset = InsetStyle {
                    top: Some(rect.origin.y).into(),
                    left: Some(rect.origin.x).into(),
                    ..Default::default()
                };

                let mut label_props = ContainerProps::default();
                label_props.layout = label_layout;
                label_props.padding = Edges::all(Px(4.0)).into();
                label_props.background = Some(Color {
                    r: 0.0,
                    g: 0.0,
                    b: 0.0,
                    a: 0.75,
                });
                label_props.corner_radii = Corners::all(Px(4.0));

                let role = crate::ui_diagnostics::semantics_role_label(node.role);
                let mut label = format!(
                    "[{tag}] {role} z={} node={}",
                    node.root_z_index, node.node_id
                );
                if let Some(test_id) = node.test_id.as_deref() {
                    label.push_str(&format!(" test_id={test_id}"));
                }
                if !redact_text && let Some(name) = node.label.as_deref() {
                    label.push_str(&format!(" label={name}"));
                }

                children.push(cx.container(label_props, |cx| vec![cx.text(label)]));
            }

            let mut root_layout = LayoutStyle::default();
            root_layout.position = PositionStyle::Relative;
            root_layout.size = SizeStyle {
                width: Length::Fill,
                height: Length::Fill,
                ..Default::default()
            };

            let mut root_props = ContainerProps::default();
            root_props.layout = root_layout;
            root_props.background = None;
            root_props.border = Edges::all(Px(0.0));
            root_props.border_color = None;

            vec![cx.container(root_props, |_cx| children)]
        },
    );

    let layer = ui
        .node_layer(root_node)
        .unwrap_or_else(|| ui.push_overlay_root_ex(root_node, false, false));
    ui.set_layer_visible(layer, true);
    ui.set_layer_hit_testable(layer, false);
    ui.set_layer_wants_pointer_down_outside_events(layer, false);
    ui.set_layer_wants_pointer_move_events(layer, false);
    ui.set_layer_wants_timer_events(layer, false);
}
