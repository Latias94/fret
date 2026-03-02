use fret_app::App;
use fret_app::Effect;
use fret_core::AppWindowId;

use super::UiDiagnosticsService;
use super::inspect_explain::build_inspect_explainability_lines;
use super::inspect_neighborhood::build_inspect_neighborhood_model;
use super::inspect_tree::build_inspect_tree_model;
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

    let clipboard = app.with_global_mut_untracked(UiDiagnosticsService::default, |svc, _app| {
        svc.take_inspect_overlay_clipboard_payloads(window)
    });
    if let Some(text) = clipboard.copy_details {
        app.push_effect(Effect::ClipboardSetText { text });
    }

    if let Some(text) = clipboard.copy_selector {
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

    let model = app.with_global_mut_untracked(UiDiagnosticsService::default, |svc, _app| {
        svc.inspect_overlay_model(window)
    });

    let explainability_lines: Vec<String> = model
        .help_open
        .then(|| build_inspect_explainability_lines(ui, window, model.pointer_pos))
        .unwrap_or_default();

    struct InspectNodeInfo {
        bounds: fret_core::Rect,
        role: fret_core::SemanticsRole,
        root_z_index: u32,
        node_id: u64,
        test_id: Option<String>,
        label: Option<String>,
    }

    let snapshot = ui.semantics_snapshot();
    let (hovered, picked, focus, neighborhood_model, tree_model) = if let Some(snapshot) = snapshot
    {
        let index = SemanticsIndex::new(snapshot);

        let hovered = model.pointer_pos.and_then(|pos| {
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

        let picked = model
            .picked_node_id
            .and_then(|id| snapshot.nodes.iter().find(|n| n.id.data().as_ffi() == id))
            .map(|node| InspectNodeInfo {
                bounds: node.bounds,
                role: node.role,
                root_z_index: index.root_z_for(node.id.data().as_ffi()),
                node_id: node.id.data().as_ffi(),
                test_id: node.test_id.clone(),
                label: node.label.clone(),
            });

        let focus = model
            .focus_node_id
            .and_then(|id| snapshot.nodes.iter().find(|n| n.id.data().as_ffi() == id))
            .map(|node| InspectNodeInfo {
                bounds: node.bounds,
                role: node.role,
                root_z_index: index.root_z_for(node.id.data().as_ffi()),
                node_id: node.id.data().as_ffi(),
                test_id: node.test_id.clone(),
                label: node.label.clone(),
            });

        let neighborhood_model = model.help_open.then(|| {
            build_inspect_neighborhood_model(
                snapshot,
                &index,
                model.focus_node_id.or(model.picked_node_id),
                model.help_search_query.as_deref(),
                model.redact_text,
                model.help_selected_match_index,
            )
        });

        let tree_model = (model.help_open && model.tree_open).then(|| {
            app.with_global_mut_untracked(UiDiagnosticsService::default, |svc, _app| {
                svc.ensure_inspect_tree_state_initialized(
                    window,
                    snapshot,
                    &index,
                    model.focus_node_id,
                    model.picked_node_id,
                );
            });

            let (expanded, selected_node_id) = app
                .with_global_mut_untracked(UiDiagnosticsService::default, |svc, _app| {
                    svc.inspect_tree_state_snapshot(window)
                });

            let model = build_inspect_tree_model(
                snapshot,
                &index,
                &expanded,
                selected_node_id,
                model.redact_text,
            );
            app.with_global_mut_untracked(UiDiagnosticsService::default, |svc, _app| {
                svc.set_inspect_tree_items(window, model.flat_node_ids.clone());
            });
            model
        });

        (
            hovered,
            picked,
            focus,
            neighborhood_model.unwrap_or_default(),
            tree_model.unwrap_or_default(),
        )
    } else {
        app.with_global_mut_untracked(UiDiagnosticsService::default, |svc, _app| {
            svc.set_inspect_tree_items(window, Vec::new());
        });
        (None, None, None, Default::default(), Default::default())
    };

    app.with_global_mut_untracked(UiDiagnosticsService::default, |svc, _app| {
        svc.set_inspect_help_matches(window, neighborhood_model.match_node_ids.clone());
    });

    let hovered = if model.locked || !(model.pick_armed || model.inspect_enabled) {
        None
    } else {
        hovered
    };

    let toast = model.toast_message.as_deref();
    let best_selector = model.best_selector_json.as_deref();
    let focus_summary = model.focus_summary_line.as_deref();
    let focus_path = model.focus_path_line.as_deref();

    let should_show_hud = model.pick_armed
        || model.pick_pending
        || model.inspect_enabled
        || model.help_open
        || toast.is_some()
        || best_selector.is_some()
        || focus_summary.is_some()
        || focus_path.is_some();

    let hud_lines: Option<Vec<String>> = should_show_hud.then(|| {
        let mut header_lines: Vec<String> = Vec::new();
        let mut body_lines: Vec<String> = Vec::new();

        if model.pick_armed || model.pick_pending {
            header_lines
                .push("INSPECT: click to pick a target (Esc to cancel, Ctrl+Alt+H help)".to_string());
        } else if model.help_open {
            header_lines.push(format!(
                "INSPECT (enabled={}, consume_clicks={}, locked={})",
                model.inspect_enabled, model.consume_clicks, model.locked
            ));
            header_lines.push("Ctrl/Cmd+Alt+I: toggle inspect".to_string());
            header_lines.push("Ctrl/Cmd+Alt+H: toggle help".to_string());
            header_lines.push("Esc: exit inspect / disarm pick".to_string());
            header_lines.push("Ctrl/Cmd+C: copy best selector".to_string());
            header_lines.push("Ctrl/Cmd+Shift+C: copy inspect details".to_string());
            header_lines.push("F: select focused node".to_string());
            header_lines.push("L: lock/unlock selection".to_string());
            header_lines.push("Alt+Up/Down: navigate parent chain (locked)".to_string());
            header_lines.push("Ctrl/Cmd+T: toggle tree view".to_string());
            header_lines.push("PageUp/PageDown/Home/End: scroll help output".to_string());
            header_lines.push(
                "Ctrl/Cmd+Enter (help search): lock selected match + copy selector".to_string(),
            );

            if !explainability_lines.is_empty() {
                body_lines.extend(explainability_lines.iter().cloned());
            }
            if !neighborhood_model.lines.is_empty() {
                if !body_lines.is_empty() {
                    body_lines.push(String::new());
                }
                body_lines.extend(neighborhood_model.lines.iter().cloned());
            }
            if !tree_model.lines.is_empty() {
                if !body_lines.is_empty() {
                    body_lines.push(String::new());
                }
                body_lines.extend(tree_model.lines.iter().cloned());
            }

            const BODY_PAGE_LINES: usize = 28;
            let total = body_lines.len();
            let mut offset = model.help_scroll_offset;
            let max_offset = total.saturating_sub(BODY_PAGE_LINES);
            if offset > max_offset {
                offset = max_offset;
                app.with_global_mut_untracked(UiDiagnosticsService::default, |svc, _app| {
                    svc.set_inspect_help_scroll_offset(window, offset);
                });
            }

            if total > 0 {
                let start_1 = offset.saturating_add(1);
                let end_1 = (offset + BODY_PAGE_LINES).min(total);
                header_lines.push(format!("help: {start_1}-{end_1}/{total}"));
                body_lines = body_lines
                    .into_iter()
                    .skip(offset)
                    .take(BODY_PAGE_LINES)
                    .collect();
            } else {
                header_lines.push("help: <empty>".to_string());
                body_lines.clear();
            }
        } else {
            header_lines.push(format!(
                "INSPECT: Ctrl+Alt+I toggle | Ctrl+Alt+H help | Esc exit | Ctrl+C copy selector | F focus | L lock | Alt+Up/Down nav (consume_clicks={}, locked={})",
                model.consume_clicks, model.locked
            ));
        }

        if let Some(t) = toast {
            header_lines.push(format!("status: {t}"));
        }
        if let Some(summary) = focus_summary {
            header_lines.push(summary.to_string());
        }
        if let Some(path) = focus_path {
            header_lines.push(path.to_string());
        }
        if let Some(sel) = best_selector {
            let trimmed = if sel.chars().take(181).count() > 180 {
                let mut s: String = sel.chars().take(180).collect();
                s.push('…');
                s
            } else {
                sel.to_string()
            };
            header_lines.push(format!("selector: {trimmed}"));
        }

        let mut lines = header_lines;
        if !body_lines.is_empty() {
            lines.push(String::new());
            lines.extend(body_lines);
        }
        lines
    });

    let redact_text = model.redact_text;

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

            if let Some(lines) = hud_lines.as_ref() {
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

                children.push(cx.container(props, |cx| {
                    lines
                        .iter()
                        .cloned()
                        .map(|t| cx.text(t))
                        .collect::<Vec<_>>()
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
