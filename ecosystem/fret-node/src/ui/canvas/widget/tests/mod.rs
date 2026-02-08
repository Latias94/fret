use crate::core::{
    CanvasPoint, CanvasRect, CanvasSize, Edge, EdgeId, EdgeKind, Graph, GraphId, Group, GroupId,
    Node, NodeId, NodeKindKey, Port, PortCapacity, PortDirection, PortId, PortKey, PortKind,
};
use crate::rules::EdgeEndpoint;
use crate::ui::commands::{
    CMD_NODE_GRAPH_ACTIVATE, CMD_NODE_GRAPH_ALIGN_CENTER_X, CMD_NODE_GRAPH_ALIGN_LEFT,
    CMD_NODE_GRAPH_ALIGN_RIGHT, CMD_NODE_GRAPH_DELETE_SELECTION, CMD_NODE_GRAPH_DISTRIBUTE_X,
    CMD_NODE_GRAPH_FOCUS_NEXT, CMD_NODE_GRAPH_FOCUS_NEXT_PORT, CMD_NODE_GRAPH_FOCUS_PORT_LEFT,
    CMD_NODE_GRAPH_FOCUS_PORT_RIGHT, CMD_NODE_GRAPH_FOCUS_PREV, CMD_NODE_GRAPH_FOCUS_PREV_PORT,
    CMD_NODE_GRAPH_NUDGE_RIGHT, CMD_NODE_GRAPH_NUDGE_RIGHT_FAST, CMD_NODE_GRAPH_SELECT_ALL,
};
use fret_core::{
    AppWindowId, Event, Modifiers, MouseButton, MouseButtons, Point, PointerEvent, Px, Rect, Size,
};
use fret_runtime::{CommandId, DragSession, DragSessionId, Effect};
use fret_ui::retained_bridge::Widget as _;
use serde_json::Value;
use std::time::Instant;

mod a11y_active_descendant_conformance;
mod background_style_conformance;
mod cached_edge_labels_tile_equivalence_conformance;
mod cached_edges_tile_equivalence_conformance;
mod callbacks_conformance;
mod color_mode_conformance;
mod connect_conformance;
mod connection_mode_conformance;
mod custom_edge_path_conformance;
mod derived_geometry_invalidation_conformance;
mod derived_geometry_updates_conformance;
mod drag_preview_conformance;
mod draw_order_invalidation_conformance;
mod edge_drag_conformance;
mod edge_hit_width_conformance;
mod edge_insert_conformance;
mod edge_insert_gestures_conformance;
mod edge_label_route_anchor_conformance;
mod edge_label_style_override_conformance;
mod edge_marker_bezier_tangent_conformance;
mod edge_marker_size_zoom_conformance;
mod edge_marker_step_tangent_conformance;
mod edge_marker_tangent_fallback_conformance;
mod edge_types_invalidation_conformance;
mod edit_command_availability_conformance;
mod elevate_on_select_conformance;
mod fit_view_nodes_conformance;
mod fit_view_on_mount_conformance;
mod fit_view_options_conformance;
mod fit_view_padding_conformance;
mod focus_auto_pan_conformance;
mod group_preview_conformance;
mod harness;
mod hit_testing_conformance;
mod hit_testing_semantic_zoom_conformance;
mod hot_state_invalidation_conformance;
mod insert_node_drag_conformance;
mod insert_node_drag_drop_conformance;
mod interaction_conformance;
mod internals_conformance;
mod invalidation_ordering_conformance;
mod is_valid_connection_conformance;
mod measured_output_store_conformance;
mod measured_port_anchor_conformance;
mod middleware_conformance;
mod node_origin_conformance;
mod node_resize_preview_conformance;
mod node_sizing_conformance;
mod nudge_step_conformance;
mod only_render_visible_elements_conformance;
mod op_batching_determinism_conformance;
mod overlay_blackboard_conformance;
mod overlay_group_rename_conformance;
mod overlay_invalidation_conformance;
mod overlay_menu_searcher_conformance;
mod overlay_minimap_controls_conformance;
mod overlay_symbol_rename_conformance;
mod overlay_toolbars_conformance;
mod perf_cache;
mod perf_cache_prune_conformance;
mod portal_conformance;
mod portal_keyboard_conformance;
mod portal_lifecycle_conformance;
mod portal_measured_geometry_conformance;
mod portal_measured_internals_conformance;
mod portal_pointer_passthrough_conformance;
mod prelude;
mod render_culling_metrics_conformance;
mod selection_mode_conformance;
mod set_viewport_conformance;
mod spatial_index_equivalence_conformance;
mod threshold_zoom_conformance;
mod translate_extent_conformance;
mod viewport_animation_conformance;
mod viewport_helper_conformance;
mod xyflow_style_conformance;
mod z_order_conformance;

use harness::{
    NullServices, TestUiHostImpl, command_cx, event_cx, insert_graph_view, insert_view,
    make_host_graph_view, make_test_graph_two_nodes, make_test_graph_two_nodes_with_ports,
    make_test_graph_two_nodes_with_ports_spaced_x, make_test_graph_two_nodes_with_size,
    read_node_pos,
};

use prelude::{NodeDrag, NodeGraphCanvas, ViewSnapshot, WireDrag, WireDragKind};

#[test]
fn inflate_rect_expands_by_margin() {
    let rect = Rect::new(
        Point::new(Px(10.0), Px(20.0)),
        Size::new(Px(30.0), Px(40.0)),
    );
    let inflated = super::inflate_rect(rect, 5.0);
    assert_eq!(
        inflated,
        Rect::new(Point::new(Px(5.0), Px(15.0)), Size::new(Px(40.0), Px(50.0)))
    );
}

#[test]
fn edge_bounds_rect_applies_padding() {
    let from = Point::new(Px(10.0), Px(10.0));
    let to = Point::new(Px(30.0), Px(20.0));
    let bounds = super::edge_bounds_rect(
        crate::ui::presenter::EdgeRouteKind::Straight,
        from,
        to,
        1.0,
        2.0,
    );
    assert_eq!(
        bounds,
        Rect::new(Point::new(Px(8.0), Px(8.0)), Size::new(Px(24.0), Px(14.0)))
    );
}

#[test]
fn middle_mouse_panning_tracks_screen_delta_under_render_transform() {
    let mut host = TestUiHostImpl::default();
    let (graph_value, _a, _b) = make_test_graph_two_nodes();
    let (graph, view) = insert_graph_view(&mut host, graph_value);

    let mut canvas = NodeGraphCanvas::new(graph, view);
    let bounds = Rect::new(
        Point::new(Px(0.0), Px(0.0)),
        Size::new(Px(800.0), Px(600.0)),
    );
    let mut services = NullServices::default();
    let mut prevented_default_actions = fret_runtime::DefaultActionSet::default();
    let mut cx = event_cx(
        &mut host,
        &mut services,
        bounds,
        &mut prevented_default_actions,
    );

    let mut snapshot = canvas.sync_view_state(cx.app);
    assert_eq!(snapshot.zoom, 1.0);
    assert_eq!(snapshot.pan, CanvasPoint::default());

    canvas.interaction.panning = true;
    canvas.interaction.pan_last_sample_at = Some(Instant::now());
    canvas.interaction.pan_last_screen_pos = None;

    let screen_positions = [
        Point::new(Px(100.0), Px(100.0)),
        Point::new(Px(140.0), Px(100.0)),
        Point::new(Px(190.0), Px(100.0)),
    ];

    for screen in screen_positions {
        let zoom = snapshot.zoom;
        let pan = snapshot.pan;
        let local = Point::new(
            Px((screen.x.0 - bounds.origin.x.0) / zoom - pan.x),
            Px((screen.y.0 - bounds.origin.y.0) / zoom - pan.y),
        );
        assert!(super::pan_zoom::handle_panning_move(
            &mut canvas,
            &mut cx,
            &snapshot,
            local,
        ));
        snapshot = canvas.sync_view_state(cx.app);
    }

    let expected_pan_x = screen_positions.last().unwrap().x.0 - screen_positions[0].x.0;
    assert!((snapshot.pan.x - expected_pan_x).abs() <= 1.0e-3);
    assert!((snapshot.pan.y - 0.0).abs() <= 1.0e-3);
}

#[test]
fn space_to_pan_starts_left_mouse_panning_and_updates_viewport() {
    let mut host = TestUiHostImpl::default();
    let (graph_value, _a, _b) = make_test_graph_two_nodes();
    let (graph, view) = insert_graph_view(&mut host, graph_value);

    let mut canvas = NodeGraphCanvas::new(graph, view);
    let bounds = Rect::new(
        Point::new(Px(0.0), Px(0.0)),
        Size::new(Px(800.0), Px(600.0)),
    );
    let mut services = NullServices::default();
    let mut prevented_default_actions = fret_runtime::DefaultActionSet::default();
    let mut cx = event_cx(
        &mut host,
        &mut services,
        bounds,
        &mut prevented_default_actions,
    );

    let mut snapshot = canvas.sync_view_state(cx.app);
    assert!(snapshot.interaction.space_to_pan);
    assert_eq!(snapshot.zoom, 1.0);
    assert_eq!(snapshot.pan, CanvasPoint::default());

    canvas.event(
        &mut cx,
        &Event::KeyDown {
            key: fret_core::KeyCode::Space,
            modifiers: Modifiers::default(),
            repeat: false,
        },
    );
    assert!(canvas.interaction.pan_activation_key_held);

    canvas.event(
        &mut cx,
        &Event::Pointer(PointerEvent::Down {
            pointer_id: fret_core::PointerId::default(),
            position: Point::new(Px(100.0), Px(100.0)),
            button: MouseButton::Left,
            modifiers: Modifiers::default(),
            click_count: 1,
            pointer_type: fret_core::PointerType::Mouse,
        }),
    );
    assert!(canvas.interaction.panning);
    assert_eq!(canvas.interaction.panning_button, Some(MouseButton::Left));
    assert!(canvas.interaction.pending_marquee.is_none());
    assert!(canvas.interaction.marquee.is_none());

    let screen_positions = [
        Point::new(Px(100.0), Px(100.0)),
        Point::new(Px(140.0), Px(100.0)),
        Point::new(Px(190.0), Px(100.0)),
    ];
    for screen in screen_positions {
        let zoom = snapshot.zoom;
        let pan = snapshot.pan;
        let local = Point::new(
            Px((screen.x.0 - bounds.origin.x.0) / zoom - pan.x),
            Px((screen.y.0 - bounds.origin.y.0) / zoom - pan.y),
        );
        canvas.event(
            &mut cx,
            &Event::Pointer(PointerEvent::Move {
                pointer_id: fret_core::PointerId::default(),
                position: local,
                buttons: MouseButtons {
                    left: true,
                    ..MouseButtons::default()
                },
                modifiers: Modifiers::default(),
                pointer_type: fret_core::PointerType::Mouse,
            }),
        );
        snapshot = canvas.sync_view_state(cx.app);
    }

    let expected_pan_x = screen_positions.last().unwrap().x.0 - screen_positions[0].x.0;
    assert!((snapshot.pan.x - expected_pan_x).abs() <= 1.0e-3);
    assert!((snapshot.pan.y - 0.0).abs() <= 1.0e-3);

    canvas.event(
        &mut cx,
        &Event::Pointer(PointerEvent::Up {
            pointer_id: fret_core::PointerId::default(),
            position: *screen_positions.last().unwrap(),
            button: MouseButton::Left,
            modifiers: Modifiers::default(),
            is_click: true,
            click_count: 1,
            pointer_type: fret_core::PointerType::Mouse,
        }),
    );
    assert!(!canvas.interaction.panning);

    canvas.event(
        &mut cx,
        &Event::KeyUp {
            key: fret_core::KeyCode::Space,
            modifiers: Modifiers::default(),
        },
    );
    assert!(!canvas.interaction.pan_activation_key_held);
    assert_eq!(canvas.history.undo_len(), 0);
}

#[test]
fn pan_activation_key_code_must_match_to_enable_space_to_pan() {
    let mut host = TestUiHostImpl::default();
    let (graph_value, _a, _b) = make_test_graph_two_nodes();
    let (graph, view) = insert_graph_view(&mut host, graph_value);

    let _ = view.update(&mut host, |s, _cx| {
        s.interaction.space_to_pan = true;
        s.interaction.pan_activation_key_code =
            Some(crate::io::NodeGraphKeyCode(fret_core::KeyCode::KeyP));
    });

    let mut canvas = NodeGraphCanvas::new(graph, view);
    let bounds = Rect::new(
        Point::new(Px(0.0), Px(0.0)),
        Size::new(Px(800.0), Px(600.0)),
    );
    let mut services = NullServices::default();
    let mut prevented_default_actions = fret_runtime::DefaultActionSet::default();
    let mut cx = event_cx(
        &mut host,
        &mut services,
        bounds,
        &mut prevented_default_actions,
    );

    let _snapshot = canvas.sync_view_state(cx.app);
    assert!(!canvas.interaction.pan_activation_key_held);

    canvas.event(
        &mut cx,
        &Event::KeyDown {
            key: fret_core::KeyCode::Space,
            modifiers: Modifiers::default(),
            repeat: false,
        },
    );
    assert!(
        !canvas.interaction.pan_activation_key_held,
        "Space should not activate panning when pan_activation_key_code is KeyP"
    );

    canvas.event(
        &mut cx,
        &Event::KeyDown {
            key: fret_core::KeyCode::KeyP,
            modifiers: Modifiers::default(),
            repeat: false,
        },
    );
    assert!(canvas.interaction.pan_activation_key_held);

    canvas.event(
        &mut cx,
        &Event::KeyUp {
            key: fret_core::KeyCode::KeyP,
            modifiers: Modifiers::default(),
        },
    );
    assert!(!canvas.interaction.pan_activation_key_held);
}

#[test]
fn pan_activation_key_code_none_disables_space_to_pan_activation() {
    let mut host = TestUiHostImpl::default();
    let (graph_value, _a, _b) = make_test_graph_two_nodes();
    let (graph, view) = insert_graph_view(&mut host, graph_value);

    let _ = view.update(&mut host, |s, _cx| {
        s.interaction.space_to_pan = true;
        s.interaction.pan_activation_key_code = None;
    });

    let mut canvas = NodeGraphCanvas::new(graph, view);
    let bounds = Rect::new(
        Point::new(Px(0.0), Px(0.0)),
        Size::new(Px(800.0), Px(600.0)),
    );
    let mut services = NullServices::default();
    let mut prevented_default_actions = fret_runtime::DefaultActionSet::default();
    let mut cx = event_cx(
        &mut host,
        &mut services,
        bounds,
        &mut prevented_default_actions,
    );

    let _snapshot = canvas.sync_view_state(cx.app);

    canvas.event(
        &mut cx,
        &Event::KeyDown {
            key: fret_core::KeyCode::Space,
            modifiers: Modifiers::default(),
            repeat: false,
        },
    );
    assert!(
        !canvas.interaction.pan_activation_key_held,
        "pan_activation_key_code=None should disable activation"
    );
}

#[test]
fn pan_on_scroll_mode_horizontal_ignores_vertical_wheel_delta() {
    let mut host = TestUiHostImpl::default();
    let (graph_value, _a, _b) = make_test_graph_two_nodes();
    let (graph, view) = insert_graph_view(&mut host, graph_value);

    let _ = view.update(&mut host, |s, _cx| {
        s.interaction.pan_on_scroll = true;
        s.interaction.pan_on_scroll_speed = 1.0;
        s.interaction.pan_on_scroll_mode = crate::io::NodeGraphPanOnScrollMode::Horizontal;
    });

    let mut canvas = NodeGraphCanvas::new(graph, view);
    let bounds = Rect::new(
        Point::new(Px(0.0), Px(0.0)),
        Size::new(Px(800.0), Px(600.0)),
    );
    let mut services = NullServices::default();
    let mut prevented_default_actions = fret_runtime::DefaultActionSet::default();
    let mut cx = event_cx(
        &mut host,
        &mut services,
        bounds,
        &mut prevented_default_actions,
    );

    let before = canvas.sync_view_state(cx.app).pan;
    canvas.event(
        &mut cx,
        &Event::Pointer(PointerEvent::Wheel {
            pointer_id: fret_core::PointerId::default(),
            position: Point::new(Px(0.0), Px(0.0)),
            delta: Point::new(Px(0.0), Px(120.0)),
            modifiers: Modifiers::default(),
            pointer_type: fret_core::PointerType::Mouse,
        }),
    );
    let after = canvas.sync_view_state(cx.app).pan;
    assert_eq!(before, after);

    canvas.event(
        &mut cx,
        &Event::Pointer(PointerEvent::Wheel {
            pointer_id: fret_core::PointerId::default(),
            position: Point::new(Px(0.0), Px(0.0)),
            delta: Point::new(Px(80.0), Px(0.0)),
            modifiers: Modifiers::default(),
            pointer_type: fret_core::PointerType::Mouse,
        }),
    );
    let after2 = canvas.sync_view_state(cx.app).pan;
    assert!((after2.x - after.x - 80.0).abs() <= 1.0e-3);
    assert!((after2.y - after.y).abs() <= 1.0e-3);
}

#[test]
fn pan_on_scroll_shift_maps_vertical_wheel_to_horizontal_on_windows() {
    let mut host = TestUiHostImpl::default();
    let (graph_value, _a, _b) = make_test_graph_two_nodes();
    let (graph, view) = insert_graph_view(&mut host, graph_value);

    let _ = view.update(&mut host, |s, _cx| {
        s.interaction.pan_on_scroll = true;
        s.interaction.pan_on_scroll_speed = 1.0;
        s.interaction.pan_on_scroll_mode = crate::io::NodeGraphPanOnScrollMode::Free;
    });

    let mut canvas = NodeGraphCanvas::new(graph, view);
    let bounds = Rect::new(
        Point::new(Px(0.0), Px(0.0)),
        Size::new(Px(800.0), Px(600.0)),
    );
    let mut services = NullServices::default();
    let mut prevented_default_actions = fret_runtime::DefaultActionSet::default();
    let mut cx = event_cx(
        &mut host,
        &mut services,
        bounds,
        &mut prevented_default_actions,
    );
    cx.input_ctx.platform = fret_runtime::Platform::Windows;

    let before = canvas.sync_view_state(cx.app).pan;
    canvas.event(
        &mut cx,
        &Event::Pointer(PointerEvent::Wheel {
            pointer_id: fret_core::PointerId::default(),
            position: Point::new(Px(0.0), Px(0.0)),
            delta: Point::new(Px(0.0), Px(120.0)),
            modifiers: Modifiers {
                shift: true,
                ..Modifiers::default()
            },
            pointer_type: fret_core::PointerType::Mouse,
        }),
    );
    let after = canvas.sync_view_state(cx.app).pan;
    assert!((after.x - before.x - 120.0).abs() <= 1.0e-3);
    assert!((after.y - before.y).abs() <= 1.0e-3);
}

#[test]
fn space_enables_pan_on_scroll_even_when_pan_on_scroll_is_disabled() {
    let mut host = TestUiHostImpl::default();
    let (graph_value, _a, _b) = make_test_graph_two_nodes();
    let (graph, view) = insert_graph_view(&mut host, graph_value);

    let _ = view.update(&mut host, |s, _cx| {
        s.interaction.pan_on_scroll = false;
        s.interaction.pan_on_scroll_speed = 1.0;
        s.interaction.pan_on_scroll_mode = crate::io::NodeGraphPanOnScrollMode::Free;
        s.interaction.zoom_on_scroll = false;
        s.interaction.space_to_pan = true;
    });

    let mut canvas = NodeGraphCanvas::new(graph, view);
    let bounds = Rect::new(
        Point::new(Px(0.0), Px(0.0)),
        Size::new(Px(800.0), Px(600.0)),
    );
    let mut services = NullServices::default();
    let mut prevented_default_actions = fret_runtime::DefaultActionSet::default();
    let mut cx = event_cx(
        &mut host,
        &mut services,
        bounds,
        &mut prevented_default_actions,
    );

    let before = canvas.sync_view_state(cx.app).pan;
    canvas.event(
        &mut cx,
        &Event::Pointer(PointerEvent::Wheel {
            pointer_id: fret_core::PointerId::default(),
            position: Point::new(Px(0.0), Px(0.0)),
            delta: Point::new(Px(0.0), Px(120.0)),
            modifiers: Modifiers::default(),
            pointer_type: fret_core::PointerType::Mouse,
        }),
    );
    let after = canvas.sync_view_state(cx.app).pan;
    assert_eq!(before, after);

    canvas.event(
        &mut cx,
        &Event::KeyDown {
            key: fret_core::KeyCode::Space,
            modifiers: Modifiers::default(),
            repeat: false,
        },
    );
    assert!(canvas.interaction.pan_activation_key_held);

    canvas.event(
        &mut cx,
        &Event::Pointer(PointerEvent::Wheel {
            pointer_id: fret_core::PointerId::default(),
            position: Point::new(Px(0.0), Px(0.0)),
            delta: Point::new(Px(0.0), Px(120.0)),
            modifiers: Modifiers::default(),
            pointer_type: fret_core::PointerType::Mouse,
        }),
    );
    let after2 = canvas.sync_view_state(cx.app).pan;
    assert!((after2.y - after.y - 120.0).abs() <= 1.0e-3);
}

#[test]
fn pinch_gesture_zooms_in_about_pointer() {
    let mut host = TestUiHostImpl::default();
    let (graph_value, _a, _b) = make_test_graph_two_nodes();
    let (graph, view) = insert_graph_view(&mut host, graph_value);

    let _ = view.update(&mut host, |s, _cx| {
        s.interaction.zoom_on_pinch = true;
        s.interaction.zoom_on_pinch_speed = 1.0;
    });

    let mut canvas = NodeGraphCanvas::new(graph, view);
    let bounds = Rect::new(
        Point::new(Px(0.0), Px(0.0)),
        Size::new(Px(800.0), Px(600.0)),
    );
    let mut services = NullServices::default();
    let mut prevented_default_actions = fret_runtime::DefaultActionSet::default();
    let mut cx = event_cx(
        &mut host,
        &mut services,
        bounds,
        &mut prevented_default_actions,
    );

    let pos = Point::new(Px(100.0), Px(100.0));
    let before = canvas.sync_view_state(cx.app);
    assert_eq!(before.zoom, 1.0);
    assert_eq!(before.pan, CanvasPoint::default());

    canvas.event(
        &mut cx,
        &Event::Pointer(PointerEvent::PinchGesture {
            pointer_id: fret_core::PointerId::default(),
            position: pos,
            delta: 1.0,
            modifiers: Modifiers::default(),
            pointer_type: fret_core::PointerType::Mouse,
        }),
    );

    let after = canvas.sync_view_state(cx.app);
    assert!((after.zoom - 2.0).abs() <= 1.0e-6);
    assert!((after.pan.x - -50.0).abs() <= 1.0e-3);
    assert!((after.pan.y - -50.0).abs() <= 1.0e-3);
}

#[test]
fn pinch_gesture_respects_toggle() {
    let mut host = TestUiHostImpl::default();
    let (graph_value, _a, _b) = make_test_graph_two_nodes();
    let (graph, view) = insert_graph_view(&mut host, graph_value);

    let _ = view.update(&mut host, |s, _cx| {
        s.interaction.zoom_on_pinch = false;
    });

    let mut canvas = NodeGraphCanvas::new(graph, view);
    let bounds = Rect::new(
        Point::new(Px(0.0), Px(0.0)),
        Size::new(Px(800.0), Px(600.0)),
    );
    let mut services = NullServices::default();
    let mut prevented_default_actions = fret_runtime::DefaultActionSet::default();
    let mut cx = event_cx(
        &mut host,
        &mut services,
        bounds,
        &mut prevented_default_actions,
    );

    canvas.event(
        &mut cx,
        &Event::Pointer(PointerEvent::PinchGesture {
            pointer_id: fret_core::PointerId::default(),
            position: Point::new(Px(100.0), Px(100.0)),
            delta: 1.0,
            modifiers: Modifiers::default(),
            pointer_type: fret_core::PointerType::Mouse,
        }),
    );

    let after = canvas.sync_view_state(cx.app);
    assert_eq!(after.zoom, 1.0);
    assert_eq!(after.pan, CanvasPoint::default());
}

#[test]
fn wheel_zoom_zooms_about_pointer() {
    let mut host = TestUiHostImpl::default();
    let (graph_value, _a, _b) = make_test_graph_two_nodes();
    let (graph, view) = insert_graph_view(&mut host, graph_value);

    let _ = view.update(&mut host, |s, _cx| {
        s.interaction.zoom_on_scroll = true;
        s.interaction.zoom_on_scroll_speed = 1.0;
        s.interaction.zoom_activation_key = crate::io::NodeGraphZoomActivationKey::None;
    });

    let mut canvas = NodeGraphCanvas::new(graph, view);
    let bounds = Rect::new(
        Point::new(Px(0.0), Px(0.0)),
        Size::new(Px(800.0), Px(600.0)),
    );
    let mut services = NullServices::default();
    let mut prevented_default_actions = fret_runtime::DefaultActionSet::default();
    let mut cx = event_cx(
        &mut host,
        &mut services,
        bounds,
        &mut prevented_default_actions,
    );

    let pos = Point::new(Px(100.0), Px(100.0));
    canvas.event(
        &mut cx,
        &Event::Pointer(PointerEvent::Wheel {
            pointer_id: fret_core::PointerId::default(),
            position: pos,
            delta: Point::new(Px(0.0), Px(-120.0)),
            modifiers: Modifiers::default(),
            pointer_type: fret_core::PointerType::Mouse,
        }),
    );

    let after = canvas.sync_view_state(cx.app);
    assert!((after.zoom - 1.18).abs() <= 1.0e-4);
    assert!((after.pan.x - -15.254).abs() <= 1.0e-3);
    assert!((after.pan.y - -15.254).abs() <= 1.0e-3);
}

#[test]
fn delete_key_defaults_to_backspace() {
    let mut host = TestUiHostImpl::default();
    let (graph_value, _a, _b) = make_test_graph_two_nodes();
    let (graph, view) = insert_graph_view(&mut host, graph_value);
    let mut canvas = NodeGraphCanvas::new(graph, view);

    let bounds = Rect::new(
        Point::new(Px(0.0), Px(0.0)),
        Size::new(Px(800.0), Px(600.0)),
    );
    let mut services = NullServices::default();
    {
        let mut prevented_default_actions = fret_runtime::DefaultActionSet::default();
        let mut cx = event_cx(
            &mut host,
            &mut services,
            bounds,
            &mut prevented_default_actions,
        );
        cx.window = Some(AppWindowId::default());
        canvas.event(
            &mut cx,
            &Event::KeyDown {
                key: fret_core::KeyCode::Delete,
                modifiers: Modifiers::default(),
                repeat: false,
            },
        );
    }
    assert!(
        host.effects.is_empty(),
        "Delete should not dispatch delete-selection by default (XYFlow default is Backspace)"
    );

    {
        let mut prevented_default_actions = fret_runtime::DefaultActionSet::default();
        let mut cx = event_cx(
            &mut host,
            &mut services,
            bounds,
            &mut prevented_default_actions,
        );
        cx.window = Some(AppWindowId::default());
        canvas.event(
            &mut cx,
            &Event::KeyDown {
                key: fret_core::KeyCode::Backspace,
                modifiers: Modifiers::default(),
                repeat: false,
            },
        );
    }
    assert!(
        host.effects.iter().any(|e| matches!(
            e,
            Effect::Command { command, .. }
                if *command == CommandId::from(crate::ui::commands::CMD_NODE_GRAPH_DELETE_SELECTION)
        )),
        "Backspace should dispatch delete-selection by default"
    );
}

#[test]
fn disable_keyboard_a11y_does_not_block_delete_shortcut() {
    let mut host = TestUiHostImpl::default();
    let (graph_value, _a, _b) = make_test_graph_two_nodes();
    let (graph, view) = insert_graph_view(&mut host, graph_value);

    let _ = view.update(&mut host, |s, _cx| {
        s.interaction.disable_keyboard_a11y = true;
        s.interaction.delete_key = crate::io::NodeGraphDeleteKey::BackspaceOrDelete;
    });

    let mut canvas = NodeGraphCanvas::new(graph, view);
    let bounds = Rect::new(
        Point::new(Px(0.0), Px(0.0)),
        Size::new(Px(800.0), Px(600.0)),
    );
    let mut services = NullServices::default();

    {
        let mut prevented_default_actions = fret_runtime::DefaultActionSet::default();
        let mut cx = event_cx(
            &mut host,
            &mut services,
            bounds,
            &mut prevented_default_actions,
        );
        cx.window = Some(AppWindowId::default());
        canvas.event(
            &mut cx,
            &Event::KeyDown {
                key: fret_core::KeyCode::Backspace,
                modifiers: Modifiers::default(),
                repeat: false,
            },
        );
    }

    assert!(
        host.effects.iter().any(|e| matches!(
            e,
            Effect::Command { command, .. }
                if *command == CommandId::from(crate::ui::commands::CMD_NODE_GRAPH_DELETE_SELECTION)
        )),
        "delete shortcut should still work when disable_keyboard_a11y is enabled (XYFlow parity)"
    );
}

#[test]
fn disable_keyboard_a11y_blocks_tab_focus_traversal() {
    let mut host = TestUiHostImpl::default();
    let (graph_value, _a, _b) = make_test_graph_two_nodes();
    let (graph, view) = insert_graph_view(&mut host, graph_value);

    let _ = view.update(&mut host, |s, _cx| {
        s.interaction.disable_keyboard_a11y = true;
    });

    let mut canvas = NodeGraphCanvas::new(graph, view);
    let bounds = Rect::new(
        Point::new(Px(0.0), Px(0.0)),
        Size::new(Px(800.0), Px(600.0)),
    );
    let mut services = NullServices::default();

    {
        let mut prevented_default_actions = fret_runtime::DefaultActionSet::default();
        let mut cx = event_cx(
            &mut host,
            &mut services,
            bounds,
            &mut prevented_default_actions,
        );
        cx.window = Some(AppWindowId::default());
        canvas.event(
            &mut cx,
            &Event::KeyDown {
                key: fret_core::KeyCode::Tab,
                modifiers: Modifiers::default(),
                repeat: false,
            },
        );
    }

    assert!(
        host.effects.is_empty(),
        "Tab focus traversal should not dispatch focus commands when disable_keyboard_a11y is enabled"
    );
}

#[test]
fn double_click_background_zooms_in_about_pointer() {
    let mut host = TestUiHostImpl::default();
    let (graph_value, _a, _b) = make_test_graph_two_nodes();
    let (graph, view) = insert_graph_view(&mut host, graph_value);

    let _ = view.update(&mut host, |s, _cx| {
        s.interaction.zoom_on_double_click = true;
    });

    let mut canvas = NodeGraphCanvas::new(graph, view);
    let bounds = Rect::new(
        Point::new(Px(0.0), Px(0.0)),
        Size::new(Px(800.0), Px(600.0)),
    );
    let mut services = NullServices::default();
    let mut prevented_default_actions = fret_runtime::DefaultActionSet::default();
    let mut cx = event_cx(
        &mut host,
        &mut services,
        bounds,
        &mut prevented_default_actions,
    );

    let pos = Point::new(Px(600.0), Px(500.0));
    let before = canvas.sync_view_state(cx.app);
    assert_eq!(before.zoom, 1.0);
    assert_eq!(before.pan, CanvasPoint::default());

    canvas.event(
        &mut cx,
        &Event::Pointer(PointerEvent::Down {
            pointer_id: fret_core::PointerId::default(),
            position: pos,
            button: MouseButton::Left,
            modifiers: Modifiers::default(),
            click_count: 2,
            pointer_type: fret_core::PointerType::Mouse,
        }),
    );

    let after = canvas.sync_view_state(cx.app);
    assert!((after.zoom - 2.0).abs() <= 1.0e-6);
    assert!((after.pan.x - -300.0).abs() <= 1.0e-3);
    assert!((after.pan.y - -250.0).abs() <= 1.0e-3);
    assert!(canvas.interaction.pending_marquee.is_none());
    assert!(canvas.interaction.marquee.is_none());
}

#[test]
fn shift_double_click_background_zooms_out_about_pointer() {
    let mut host = TestUiHostImpl::default();
    let (graph_value, _a, _b) = make_test_graph_two_nodes();
    let (graph, view) = insert_graph_view(&mut host, graph_value);

    let _ = view.update(&mut host, |s, _cx| {
        s.interaction.zoom_on_double_click = true;
    });

    let mut canvas = NodeGraphCanvas::new(graph, view);
    let bounds = Rect::new(
        Point::new(Px(0.0), Px(0.0)),
        Size::new(Px(800.0), Px(600.0)),
    );
    let mut services = NullServices::default();
    let mut prevented_default_actions = fret_runtime::DefaultActionSet::default();
    let mut cx = event_cx(
        &mut host,
        &mut services,
        bounds,
        &mut prevented_default_actions,
    );

    let pos = Point::new(Px(600.0), Px(500.0));
    canvas.event(
        &mut cx,
        &Event::Pointer(PointerEvent::Down {
            pointer_id: fret_core::PointerId::default(),
            position: pos,
            button: MouseButton::Left,
            modifiers: Modifiers {
                shift: true,
                ..Modifiers::default()
            },
            click_count: 2,
            pointer_type: fret_core::PointerType::Mouse,
        }),
    );

    let after = canvas.sync_view_state(cx.app);
    assert!((after.zoom - 0.5).abs() <= 1.0e-6);
    assert!((after.pan.x - 600.0).abs() <= 1.0e-3);
    assert!((after.pan.y - 500.0).abs() <= 1.0e-3);
}

#[test]
fn internal_drag_drop_candidate_on_edge_splits_edge() {
    use std::sync::Arc;

    use crate::core::{PortCapacity, PortDirection, PortKey, PortKind};
    use crate::rules::{InsertNodeTemplate, PortTemplate};
    use crate::ui::presenter::InsertNodeCandidate;
    use fret_core::{InternalDragEvent, InternalDragKind};

    let mut host = TestUiHostImpl::default();
    let (mut graph_value, _a, _a_in, a_out, _b, b_in) =
        make_test_graph_two_nodes_with_ports_spaced_x(420.0);
    let edge_id = EdgeId::new();
    graph_value.edges.insert(
        edge_id,
        Edge {
            kind: EdgeKind::Data,
            from: a_out,
            to: b_in,
            selectable: None,
            deletable: None,
            reconnectable: None,
        },
    );

    let template_kind = NodeKindKey::new("test.mid");
    let template = InsertNodeTemplate {
        kind: template_kind.clone(),
        kind_version: 1,
        collapsed: false,
        data: Value::Null,
        ports: vec![
            PortTemplate {
                key: PortKey::new("in"),
                dir: PortDirection::In,
                kind: PortKind::Data,
                capacity: PortCapacity::Single,
                ty: None,
                data: Value::Null,
            },
            PortTemplate {
                key: PortKey::new("out"),
                dir: PortDirection::Out,
                kind: PortKind::Data,
                capacity: PortCapacity::Single,
                ty: None,
                data: Value::Null,
            },
        ],
        input: PortKey::new("in"),
        output: PortKey::new("out"),
    };
    let candidate = InsertNodeCandidate {
        kind: template_kind.clone(),
        label: Arc::<str>::from("Mid"),
        enabled: true,
        template: Some(template),
        payload: Value::Null,
    };

    host.drag = Some(DragSession::new_cross_window(
        DragSessionId(1),
        fret_core::PointerId(0),
        AppWindowId::default(),
        super::insert_node_drag::DRAG_KIND_INSERT_NODE,
        Point::new(Px(0.0), Px(0.0)),
        super::insert_node_drag::InsertNodeDragPayload { candidate },
    ));

    let (graph, view) = insert_graph_view(&mut host, graph_value);

    let mut canvas = NodeGraphCanvas::new(graph.clone(), view.clone());
    let bounds = Rect::new(
        Point::new(Px(0.0), Px(0.0)),
        Size::new(Px(800.0), Px(600.0)),
    );
    let mut services = NullServices::default();
    let mut prevented_default_actions = fret_runtime::DefaultActionSet::default();
    let mut cx = event_cx(
        &mut host,
        &mut services,
        bounds,
        &mut prevented_default_actions,
    );

    let snap = canvas.sync_view_state(cx.app);
    let (geom, _index) = canvas.canvas_derived(&*cx.app, &snap);
    let from = geom.port_center(a_out).expect("from port center");
    let to = geom.port_center(b_in).expect("to port center");
    let (c1, c2) = super::wire_ctrl_points(from, to, snap.zoom);
    let pos = super::cubic_bezier(from, c1, c2, to, 0.5);

    canvas.event(
        &mut cx,
        &Event::InternalDrag(InternalDragEvent {
            pointer_id: fret_core::PointerId::default(),
            position: pos,
            kind: InternalDragKind::Drop,
            modifiers: Modifiers::default(),
        }),
    );

    let nodes_len = graph.read_ref(cx.app, |g| g.nodes.len()).unwrap_or(0);
    let edges_len = graph.read_ref(cx.app, |g| g.edges.len()).unwrap_or(0);
    assert_eq!(nodes_len, 3);
    assert_eq!(edges_len, 2);
    assert!(
        graph
            .read_ref(cx.app, |g| g.edges.contains_key(&edge_id))
            .unwrap_or(false)
    );
    assert!(
        graph
            .read_ref(cx.app, |g| g
                .nodes
                .values()
                .any(|n| n.kind == template_kind))
            .unwrap_or(false)
    );

    let after = canvas.sync_view_state(cx.app);
    assert_eq!(after.selected_nodes.len(), 1);
    assert_eq!(after.selected_edges.len(), 0);
}

#[cfg(any())]
mod legacy_inline_harness {
    use super::*;

    #[derive(Default)]
    struct NullServices;

impl fret_core::TextService for NullServices {
    fn prepare(
        &mut self,
        _input: &fret_core::TextInput,
        _constraints: fret_core::TextConstraints,
    ) -> (TextBlobId, fret_core::TextMetrics) {
        (
            TextBlobId::default(),
            fret_core::TextMetrics {
                size: Size::new(Px(0.0), Px(0.0)),
                baseline: Px(0.0),
            },
        )
    }

    fn release(&mut self, _blob: TextBlobId) {}
}

impl fret_core::PathService for NullServices {
    fn prepare(
        &mut self,
        _commands: &[fret_core::PathCommand],
        _style: fret_core::PathStyle,
        _constraints: fret_core::PathConstraints,
    ) -> (fret_core::PathId, fret_core::PathMetrics) {
        (
            fret_core::PathId::default(),
            fret_core::PathMetrics::default(),
        )
    }

    fn release(&mut self, _path: fret_core::PathId) {}
}

impl fret_core::SvgService for NullServices {
    fn register_svg(&mut self, _bytes: &[u8]) -> fret_core::SvgId {
        fret_core::SvgId::default()
    }

    fn unregister_svg(&mut self, _svg: fret_core::SvgId) -> bool {
        true
    }
}

#[derive(Default)]
struct TestUiHostImpl {
    globals: HashMap<TypeId, Box<dyn Any>>,
    models: ModelStore,
    commands: CommandRegistry,
    redraw: HashSet<AppWindowId>,
    effects: Vec<Effect>,
    drag: Option<DragSession>,
    tick_id: TickId,
    frame_id: FrameId,
    next_timer_token: u64,
    next_clipboard_token: u64,
    next_image_upload_token: u64,
}

impl GlobalsHost for TestUiHostImpl {
    fn set_global<T: Any>(&mut self, value: T) {
        self.globals.insert(TypeId::of::<T>(), Box::new(value));
    }

    fn global<T: Any>(&self) -> Option<&T> {
        self.globals
            .get(&TypeId::of::<T>())
            .and_then(|b| b.downcast_ref::<T>())
    }

    fn with_global_mut<T: Any, R>(
        &mut self,
        init: impl FnOnce() -> T,
        f: impl FnOnce(&mut T, &mut Self) -> R,
    ) -> R {
        let type_id = TypeId::of::<T>();
        if !self.globals.contains_key(&type_id) {
            self.globals.insert(type_id, Box::new(init()));
        }

        // Avoid aliasing `&mut self` by temporarily removing the value.
        let boxed = self
            .globals
            .remove(&type_id)
            .expect("global must exist")
            .downcast::<T>()
            .ok()
            .expect("global has wrong type");
        let mut value = *boxed;

        let out = f(&mut value, self);
        self.globals.insert(type_id, Box::new(value));
        out
    }
}

impl ModelHost for TestUiHostImpl {
    fn models(&self) -> &ModelStore {
        &self.models
    }

    fn models_mut(&mut self) -> &mut ModelStore {
        &mut self.models
    }
}

impl ModelsHost for TestUiHostImpl {
    fn take_changed_models(&mut self) -> Vec<fret_runtime::ModelId> {
        self.models.take_changed_models()
    }
}

impl CommandsHost for TestUiHostImpl {
    fn commands(&self) -> &CommandRegistry {
        &self.commands
    }
}

impl EffectSink for TestUiHostImpl {
    fn request_redraw(&mut self, window: AppWindowId) {
        self.redraw.insert(window);
    }

    fn push_effect(&mut self, effect: Effect) {
        self.effects.push(effect);
    }
}

impl TimeHost for TestUiHostImpl {
    fn tick_id(&self) -> TickId {
        self.tick_id
    }

    fn frame_id(&self) -> FrameId {
        self.frame_id
    }

    fn next_timer_token(&mut self) -> TimerToken {
        self.next_timer_token = self.next_timer_token.saturating_add(1);
        TimerToken(self.next_timer_token)
    }

    fn next_clipboard_token(&mut self) -> ClipboardToken {
        self.next_clipboard_token = self.next_clipboard_token.saturating_add(1);
        ClipboardToken(self.next_clipboard_token)
    }

    fn next_image_upload_token(&mut self) -> fret_runtime::ImageUploadToken {
        self.next_image_upload_token = self.next_image_upload_token.saturating_add(1);
        fret_runtime::ImageUploadToken(self.next_image_upload_token)
    }
}

impl DragHost for TestUiHostImpl {
    fn drag(&self, pointer_id: fret_core::PointerId) -> Option<&DragSession> {
        self.drag
            .as_ref()
            .filter(|drag| drag.pointer_id == pointer_id)
    }

    fn any_drag_session(&self, mut predicate: impl FnMut(&DragSession) -> bool) -> bool {
        self.drag.as_ref().is_some_and(|d| predicate(d))
    }

    fn find_drag_pointer_id(
        &self,
        mut predicate: impl FnMut(&DragSession) -> bool,
    ) -> Option<fret_core::PointerId> {
        self.drag
            .as_ref()
            .filter(|d| predicate(d))
            .map(|d| d.pointer_id)
    }

    fn cancel_drag_sessions(
        &mut self,
        mut predicate: impl FnMut(&DragSession) -> bool,
    ) -> Vec<fret_core::PointerId> {
        let Some(drag) = self.drag.as_ref() else {
            return Vec::new();
        };
        if !predicate(drag) {
            return Vec::new();
        }
        let pointer_id = drag.pointer_id;
        self.drag = None;
        vec![pointer_id]
    }

    fn drag_mut(&mut self, pointer_id: fret_core::PointerId) -> Option<&mut DragSession> {
        self.drag
            .as_mut()
            .filter(|drag| drag.pointer_id == pointer_id)
    }

    fn cancel_drag(&mut self, pointer_id: fret_core::PointerId) {
        if self.drag(pointer_id).is_some() {
            self.drag = None;
        }
    }

    fn begin_drag_with_kind<T: Any>(
        &mut self,
        pointer_id: fret_core::PointerId,
        kind: DragKindId,
        source_window: AppWindowId,
        start: Point,
        payload: T,
    ) {
        self.drag = Some(DragSession::new(
            DragSessionId(1),
            pointer_id,
            source_window,
            kind,
            start,
            payload,
        ));
    }

    fn begin_cross_window_drag_with_kind<T: Any>(
        &mut self,
        pointer_id: fret_core::PointerId,
        kind: DragKindId,
        source_window: AppWindowId,
        start: Point,
        payload: T,
    ) {
        self.drag = Some(DragSession::new_cross_window(
            DragSessionId(1),
            pointer_id,
            source_window,
            kind,
            start,
            payload,
        ));
    }
}

fn event_cx<'a>(
    host: &'a mut TestUiHostImpl,
    services: &'a mut NullServices,
    bounds: Rect,
    prevented_default_actions: &'a mut fret_runtime::DefaultActionSet,
) -> fret_ui::retained_bridge::EventCx<'a, TestUiHostImpl> {
    fret_ui::retained_bridge::EventCx {
        app: host,
        services,
        node: fret_core::NodeId::default(),
        layer_root: None,
        window: None,
        input_ctx: fret_runtime::InputContext::default(),
        pointer_id: None,
        prevented_default_actions,
        children: &[],
        focus: None,
        captured: None,
        bounds,
        invalidations: Vec::new(),
        requested_focus: None,
        requested_capture: None,
        requested_cursor: None,
        notify_requested: false,
        notify_requested_location: None,
        stop_propagation: false,
    }
}

fn command_cx<'a>(
    host: &'a mut TestUiHostImpl,
    services: &'a mut NullServices,
    tree: &'a mut fret_ui::UiTree<TestUiHostImpl>,
) -> fret_ui::retained_bridge::CommandCx<'a, TestUiHostImpl> {
    fret_ui::retained_bridge::CommandCx {
        app: host,
        services,
        tree,
        node: fret_core::NodeId::default(),
        window: None,
        input_ctx: fret_runtime::InputContext::default(),
        focus: None,
        invalidations: Vec::new(),
        requested_focus: None,
        stop_propagation: false,
    }
}

fn make_test_graph_two_nodes() -> (Graph, NodeId, NodeId) {
    let mut graph = Graph::new(GraphId::new());
    let kind = NodeKindKey::new("test.node");

    let a = NodeId::new();
    let b = NodeId::new();

    graph.nodes.insert(
        a,
        Node {
            kind: kind.clone(),
            kind_version: 1,
            pos: CanvasPoint { x: 0.0, y: 0.0 },
            selectable: None,
            draggable: None,
            connectable: None,
            deletable: None,
            parent: None,
            extent: None,
            expand_parent: None,
            size: None,
            hidden: false,
            collapsed: false,
            ports: Vec::new(),
            data: Value::Null,
        },
    );
    graph.nodes.insert(
        b,
        Node {
            kind,
            kind_version: 1,
            pos: CanvasPoint { x: 10.0, y: 0.0 },
            selectable: None,
            draggable: None,
            connectable: None,
            deletable: None,
            parent: None,
            extent: None,
            expand_parent: None,
            size: None,
            hidden: false,
            collapsed: false,
            ports: Vec::new(),
            data: Value::Null,
        },
    );

    (graph, a, b)
}

fn make_test_graph_two_nodes_with_size() -> (Graph, NodeId, NodeId) {
    let mut graph = Graph::new(GraphId::new());
    let kind = NodeKindKey::new("test.node");

    let a = NodeId::new();
    let b = NodeId::new();

    graph.nodes.insert(
        a,
        Node {
            kind: kind.clone(),
            kind_version: 1,
            pos: CanvasPoint { x: 0.0, y: 0.0 },
            selectable: None,
            draggable: None,
            connectable: None,
            deletable: None,
            parent: None,
            extent: None,
            expand_parent: None,
            size: Some(CanvasSize {
                width: 40.0,
                height: 20.0,
            }),
            hidden: false,
            collapsed: false,
            ports: Vec::new(),
            data: Value::Null,
        },
    );
    graph.nodes.insert(
        b,
        Node {
            kind,
            kind_version: 1,
            pos: CanvasPoint { x: 10.0, y: 5.0 },
            selectable: None,
            draggable: None,
            connectable: None,
            deletable: None,
            parent: None,
            extent: None,
            expand_parent: None,
            size: Some(CanvasSize {
                width: 40.0,
                height: 20.0,
            }),
            hidden: false,
            collapsed: false,
            ports: Vec::new(),
            data: Value::Null,
        },
    );

    (graph, a, b)
}

fn make_test_graph_two_nodes_with_ports() -> (Graph, NodeId, PortId, PortId, NodeId, PortId) {
    let mut graph = Graph::new(GraphId::new());
    let kind = NodeKindKey::new("test.node");

    let a = NodeId::new();
    let a_in = PortId::new();
    let a_out = PortId::new();
    graph.nodes.insert(
        a,
        Node {
            kind: kind.clone(),
            kind_version: 1,
            pos: CanvasPoint { x: 0.0, y: 0.0 },
            selectable: None,
            draggable: None,
            connectable: None,
            deletable: None,
            parent: None,
            extent: None,
            expand_parent: None,
            size: None,
            hidden: false,
            collapsed: false,
            ports: vec![a_in, a_out],
            data: Value::Null,
        },
    );
    graph.ports.insert(
        a_in,
        Port {
            node: a,
            key: PortKey::new("in"),
            dir: PortDirection::In,
            kind: PortKind::Data,
            capacity: PortCapacity::Single,
            connectable: None,
            connectable_start: None,
            connectable_end: None,
            ty: None,
            data: Value::Null,
        },
    );
    graph.ports.insert(
        a_out,
        Port {
            node: a,
            key: PortKey::new("out"),
            dir: PortDirection::Out,
            kind: PortKind::Data,
            capacity: PortCapacity::Multi,
            connectable: None,
            connectable_start: None,
            connectable_end: None,
            ty: None,
            data: Value::Null,
        },
    );

    let b = NodeId::new();
    let b_in = PortId::new();
    graph.nodes.insert(
        b,
        Node {
            kind,
            kind_version: 1,
            pos: CanvasPoint { x: 200.0, y: 0.0 },
            selectable: None,
            draggable: None,
            connectable: None,
            deletable: None,
            parent: None,
            extent: None,
            expand_parent: None,
            size: None,
            hidden: false,
            collapsed: false,
            ports: vec![b_in],
            data: Value::Null,
        },
    );
    graph.ports.insert(
        b_in,
        Port {
            node: b,
            key: PortKey::new("in"),
            dir: PortDirection::In,
            kind: PortKind::Data,
            capacity: PortCapacity::Single,
            connectable: None,
            connectable_start: None,
            connectable_end: None,
            ty: None,
            data: Value::Null,
        },
    );

    (graph, a, a_in, a_out, b, b_in)
}

fn make_test_graph_two_nodes_with_ports_spaced_x(
    dx: f32,
) -> (Graph, NodeId, PortId, PortId, NodeId, PortId) {
    let (mut graph, a, a_in, a_out, b, b_in) = make_test_graph_two_nodes_with_ports();
    graph
        .nodes
        .entry(b)
        .and_modify(|n| n.pos = CanvasPoint { x: dx, y: 0.0 });
    (graph, a, a_in, a_out, b, b_in)
}

fn read_node_pos(
    host: &mut TestUiHostImpl,
    model: &fret_runtime::Model<Graph>,
    id: NodeId,
) -> CanvasPoint {
    model
        .read_ref(host, |g| g.nodes.get(&id).map(|n| n.pos))
        .ok()
        .flatten()
        .unwrap_or_default()
}

}

#[test]
fn distance_sq_point_to_rect_is_zero_inside_and_positive_outside() {
    let rect = Rect::new(
        Point::new(Px(10.0), Px(20.0)),
        Size::new(Px(100.0), Px(50.0)),
    );
    let inside = Point::new(Px(50.0), Px(40.0));
    assert!(NodeGraphCanvas::rect_contains_point(rect, inside));
    assert_eq!(
        NodeGraphCanvas::distance_sq_point_to_rect(inside, rect),
        0.0
    );

    let outside = Point::new(Px(0.0), Px(0.0));
    assert!(!NodeGraphCanvas::rect_contains_point(rect, outside));
    assert!(NodeGraphCanvas::distance_sq_point_to_rect(outside, rect) > 0.0);
}

#[test]
fn yank_edges_from_port_returns_all_incident_edges() {
    let mut graph = Graph::new(GraphId::new());
    let kind = NodeKindKey::new("test.node");

    let n1 = NodeId::new();
    let p_out = PortId::new();
    graph.nodes.insert(
        n1,
        Node {
            kind: kind.clone(),
            kind_version: 1,
            pos: CanvasPoint { x: 0.0, y: 0.0 },
            selectable: None,
            draggable: None,
            connectable: None,
            deletable: None,
            parent: None,
            extent: None,
            expand_parent: None,
            size: None,
            hidden: false,
            collapsed: false,
            ports: vec![p_out],
            data: Value::Null,
        },
    );
    graph.ports.insert(
        p_out,
        Port {
            node: n1,
            key: PortKey::new("out"),
            dir: PortDirection::Out,
            kind: PortKind::Data,
            capacity: PortCapacity::Multi,
            connectable: None,
            connectable_start: None,
            connectable_end: None,
            ty: None,
            data: Value::Null,
        },
    );

    let n2 = NodeId::new();
    let p_in1 = PortId::new();
    let p_in2 = PortId::new();
    graph.nodes.insert(
        n2,
        Node {
            kind: kind.clone(),
            kind_version: 1,
            pos: CanvasPoint { x: 0.0, y: 0.0 },
            selectable: None,
            draggable: None,
            connectable: None,
            deletable: None,
            parent: None,
            extent: None,
            expand_parent: None,
            size: None,
            hidden: false,
            collapsed: false,
            ports: vec![p_in1, p_in2],
            data: Value::Null,
        },
    );
    for (id, key) in [(p_in1, "in1"), (p_in2, "in2")] {
        graph.ports.insert(
            id,
            Port {
                node: n2,
                key: PortKey::new(key),
                dir: PortDirection::In,
                kind: PortKind::Data,
                capacity: PortCapacity::Single,
                connectable: None,
                connectable_start: None,
                connectable_end: None,
                ty: None,
                data: Value::Null,
            },
        );
    }

    let e1 = EdgeId::new();
    let e2 = EdgeId::new();
    graph.edges.insert(
        e1,
        Edge {
            kind: EdgeKind::Data,
            from: p_out,
            to: p_in1,
            selectable: None,
            deletable: None,
            reconnectable: None,
        },
    );
    graph.edges.insert(
        e2,
        Edge {
            kind: EdgeKind::Data,
            from: p_out,
            to: p_in2,
            selectable: None,
            deletable: None,
            reconnectable: None,
        },
    );

    let from_edges = NodeGraphCanvas::yank_edges_from_port(&graph, p_out);
    assert_eq!(from_edges.len(), 2);
    assert!(from_edges.contains(&(e1, EdgeEndpoint::From, p_in1)));
    assert!(from_edges.contains(&(e2, EdgeEndpoint::From, p_in2)));

    let to_edges = NodeGraphCanvas::yank_edges_from_port(&graph, p_in1);
    assert_eq!(to_edges, vec![(e1, EdgeEndpoint::To, p_out)]);
}

#[test]
fn should_add_bundle_port_requires_same_side_and_dedupes() {
    let mut graph = Graph::new(GraphId::new());
    let kind = NodeKindKey::new("test.node");
    let n1 = NodeId::new();

    let p_out1 = PortId::new();
    let p_out2 = PortId::new();
    let p_in = PortId::new();

    graph.nodes.insert(
        n1,
        Node {
            kind,
            kind_version: 1,
            pos: CanvasPoint { x: 0.0, y: 0.0 },
            selectable: None,
            draggable: None,
            connectable: None,
            deletable: None,
            parent: None,
            extent: None,
            expand_parent: None,
            size: None,
            hidden: false,
            collapsed: false,
            ports: vec![p_out1, p_out2, p_in],
            data: Value::Null,
        },
    );

    graph.ports.insert(
        p_out1,
        Port {
            node: n1,
            key: PortKey::new("out1"),
            dir: PortDirection::Out,
            kind: PortKind::Data,
            capacity: PortCapacity::Multi,
            connectable: None,
            connectable_start: None,
            connectable_end: None,
            ty: None,
            data: Value::Null,
        },
    );
    graph.ports.insert(
        p_out2,
        Port {
            node: n1,
            key: PortKey::new("out2"),
            dir: PortDirection::Out,
            kind: PortKind::Data,
            capacity: PortCapacity::Multi,
            connectable: None,
            connectable_start: None,
            connectable_end: None,
            ty: None,
            data: Value::Null,
        },
    );
    graph.ports.insert(
        p_in,
        Port {
            node: n1,
            key: PortKey::new("in"),
            dir: PortDirection::In,
            kind: PortKind::Data,
            capacity: PortCapacity::Single,
            connectable: None,
            connectable_start: None,
            connectable_end: None,
            ty: None,
            data: Value::Null,
        },
    );

    assert!(NodeGraphCanvas::should_add_bundle_port(
        &graph,
        p_out1,
        &[p_out1],
        p_out2
    ));
    assert!(!NodeGraphCanvas::should_add_bundle_port(
        &graph,
        p_out1,
        &[p_out2],
        p_out2
    ));
    assert!(!NodeGraphCanvas::should_add_bundle_port(
        &graph,
        p_out1,
        &[],
        p_out1
    ));
    assert!(!NodeGraphCanvas::should_add_bundle_port(
        &graph,
        p_out1,
        &[],
        p_in
    ));
}

#[test]
fn node_drag_records_single_history_entry_for_multi_node_move() {
    let mut host = TestUiHostImpl::default();
    let (graph_value, a, b) = make_test_graph_two_nodes();
    let (graph, view) = insert_graph_view(&mut host, graph_value);

    let mut canvas = NodeGraphCanvas::new(graph.clone(), view);
    let snapshot = canvas.sync_view_state(&mut host);
    let bounds = Rect::new(
        Point::new(Px(0.0), Px(0.0)),
        Size::new(Px(800.0), Px(600.0)),
    );
    let mut services = NullServices::default();
    let mut prevented_default_actions = fret_runtime::DefaultActionSet::default();
    let mut cx = event_cx(
        &mut host,
        &mut services,
        bounds,
        &mut prevented_default_actions,
    );

    canvas.interaction.node_drag = Some(NodeDrag {
        primary: a,
        node_ids: vec![a, b],
        nodes: vec![
            (a, CanvasPoint { x: 0.0, y: 0.0 }),
            (b, CanvasPoint { x: 10.0, y: 0.0 }),
        ],
        current_nodes: vec![
            (a, CanvasPoint { x: 0.0, y: 0.0 }),
            (b, CanvasPoint { x: 10.0, y: 0.0 }),
        ],
        current_groups: Vec::new(),
        preview_rev: 0,
        grab_offset: Point::new(Px(0.0), Px(0.0)),
        start_pos: Point::new(Px(0.0), Px(0.0)),
    });

    for pos in [
        Point::new(Px(20.0), Px(5.0)),
        Point::new(Px(40.0), Px(10.0)),
        Point::new(Px(60.0), Px(10.0)),
    ] {
        let did = super::node_drag::handle_node_drag_move(
            &mut canvas,
            &mut cx,
            &snapshot,
            pos,
            fret_core::Modifiers::default(),
            snapshot.zoom,
        );
        assert!(did);
        assert_eq!(canvas.history.undo_len(), 0);
    }

    let did_up = super::pointer_up::handle_pointer_up(
        &mut canvas,
        &mut cx,
        &snapshot,
        Point::new(Px(60.0), Px(10.0)),
        fret_core::MouseButton::Left,
        1,
        fret_core::Modifiers::default(),
        snapshot.zoom,
    );
    assert!(did_up);
    assert_eq!(canvas.history.undo_len(), 1);

    assert!(canvas.undo_last(&mut host, None));
    assert_eq!(
        read_node_pos(&mut host, &graph, a),
        CanvasPoint { x: 0.0, y: 0.0 }
    );
    assert_eq!(
        read_node_pos(&mut host, &graph, b),
        CanvasPoint { x: 10.0, y: 0.0 }
    );
}

#[test]
fn connect_bundle_records_single_history_entry() {
    let mut host = TestUiHostImpl::default();
    let mut graph = Graph::new(GraphId::new());
    let kind = NodeKindKey::new("test.node");

    let n1 = NodeId::new();
    let out1 = PortId::new();
    let out2 = PortId::new();
    graph.nodes.insert(
        n1,
        Node {
            kind: kind.clone(),
            kind_version: 1,
            pos: CanvasPoint { x: 0.0, y: 0.0 },
            selectable: None,
            draggable: None,
            connectable: None,
            deletable: None,
            parent: None,
            extent: None,
            expand_parent: None,
            size: None,
            hidden: false,
            collapsed: false,
            ports: vec![out1, out2],
            data: Value::Null,
        },
    );
    for (id, key) in [(out1, "out1"), (out2, "out2")] {
        graph.ports.insert(
            id,
            Port {
                node: n1,
                key: PortKey::new(key),
                dir: PortDirection::Out,
                kind: PortKind::Data,
                capacity: PortCapacity::Multi,
                connectable: None,
                connectable_start: None,
                connectable_end: None,
                ty: None,
                data: Value::Null,
            },
        );
    }

    let n2 = NodeId::new();
    let inn = PortId::new();
    graph.nodes.insert(
        n2,
        Node {
            kind,
            kind_version: 1,
            pos: CanvasPoint { x: 100.0, y: 0.0 },
            selectable: None,
            draggable: None,
            connectable: None,
            deletable: None,
            parent: None,
            extent: None,
            expand_parent: None,
            size: None,
            hidden: false,
            collapsed: false,
            ports: vec![inn],
            data: Value::Null,
        },
    );
    graph.ports.insert(
        inn,
        Port {
            node: n2,
            key: PortKey::new("in"),
            dir: PortDirection::In,
            kind: PortKind::Data,
            capacity: PortCapacity::Multi,
            connectable: None,
            connectable_start: None,
            connectable_end: None,
            ty: None,
            data: Value::Null,
        },
    );

    let (graph_model, view) = insert_graph_view(&mut host, graph);

    let mut canvas = NodeGraphCanvas::new(graph_model.clone(), view);
    let snapshot: ViewSnapshot = canvas.sync_view_state(&mut host);
    let bounds = Rect::new(
        Point::new(Px(0.0), Px(0.0)),
        Size::new(Px(800.0), Px(600.0)),
    );
    let mut services = NullServices::default();
    let mut prevented_default_actions = fret_runtime::DefaultActionSet::default();
    let mut cx = event_cx(
        &mut host,
        &mut services,
        bounds,
        &mut prevented_default_actions,
    );

    canvas.interaction.wire_drag = Some(WireDrag {
        kind: WireDragKind::New {
            from: out1,
            bundle: vec![out1, out2],
        },
        pos: Point::new(Px(0.0), Px(0.0)),
    });

    let did = super::wire_drag::handle_wire_left_up_with_forced_target(
        &mut canvas,
        &mut cx,
        &snapshot,
        snapshot.zoom,
        Some(inn),
    );
    assert!(did);
    assert_eq!(canvas.history.undo_len(), 1);
    let edges_len = graph_model
        .read_ref(&mut host, |g| g.edges.len())
        .unwrap_or(0);
    assert_eq!(edges_len, 2);

    assert!(canvas.undo_last(&mut host, None));
    let edges_len = graph_model
        .read_ref(&mut host, |g| g.edges.len())
        .unwrap_or(0);
    assert_eq!(edges_len, 0);
}

#[test]
fn nudge_moves_selection_and_records_history_entry() {
    let mut host = TestUiHostImpl::default();
    let (graph_value, a, b) = make_test_graph_two_nodes();
    let (graph, view) = insert_graph_view(&mut host, graph_value);

    let mut canvas = NodeGraphCanvas::new(graph.clone(), view.clone());
    canvas.sync_view_state(&mut host);

    view.update(&mut host, |s, _cx| {
        s.selected_nodes = vec![a, b];
    })
    .unwrap();

    let mut services = NullServices::default();
    let mut tree: fret_ui::UiTree<TestUiHostImpl> = fret_ui::UiTree::new();
    let mut cx = command_cx(&mut host, &mut services, &mut tree);

    assert!(canvas.command(&mut cx, &CommandId::from(CMD_NODE_GRAPH_NUDGE_RIGHT)));
    assert_eq!(canvas.history.undo_len(), 1);
    assert_eq!(read_node_pos(&mut host, &graph, a).x, 1.0);
    assert_eq!(read_node_pos(&mut host, &graph, b).x, 11.0);

    assert!(canvas.undo_last(&mut host, None));
    assert_eq!(
        read_node_pos(&mut host, &graph, a),
        CanvasPoint { x: 0.0, y: 0.0 }
    );
    assert_eq!(
        read_node_pos(&mut host, &graph, b),
        CanvasPoint { x: 10.0, y: 0.0 }
    );
}

#[test]
fn nudge_multi_selection_respects_node_extent_by_selection_bounds() {
    let mut host = TestUiHostImpl::default();
    let (graph_value, a, b) = make_test_graph_two_nodes_with_size();
    let (graph, view) = insert_graph_view(&mut host, graph_value);

    let mut canvas = NodeGraphCanvas::new(graph.clone(), view.clone());
    canvas.sync_view_state(&mut host);

    view.update(&mut host, |s, _cx| {
        s.selected_nodes = vec![a, b];
        s.interaction.node_extent = Some(CanvasRect {
            origin: CanvasPoint { x: 0.0, y: 0.0 },
            size: CanvasSize {
                width: 50.0,
                height: 100.0,
            },
        });
    })
    .unwrap();

    let mut services = NullServices::default();
    let mut tree: fret_ui::UiTree<TestUiHostImpl> = fret_ui::UiTree::new();
    let mut cx = command_cx(&mut host, &mut services, &mut tree);

    assert!(canvas.command(&mut cx, &CommandId::from(CMD_NODE_GRAPH_NUDGE_RIGHT)));
    assert_eq!(canvas.history.undo_len(), 0);

    assert_eq!(read_node_pos(&mut host, &graph, a).x, 0.0);
    assert_eq!(read_node_pos(&mut host, &graph, b).x, 10.0);
}

#[test]
fn nudge_respects_per_node_extent_rect() {
    let mut host = TestUiHostImpl::default();

    let mut graph_value = Graph::new(GraphId::new());
    let node_id = NodeId::new();
    graph_value.nodes.insert(
        node_id,
        Node {
            kind: NodeKindKey::new("test.node"),
            kind_version: 1,
            pos: CanvasPoint { x: 0.0, y: 0.0 },
            selectable: None,
            draggable: None,
            connectable: None,
            deletable: None,
            parent: None,
            extent: Some(crate::core::NodeExtent::Rect {
                rect: CanvasRect {
                    origin: CanvasPoint { x: 0.0, y: 0.0 },
                    size: CanvasSize {
                        width: 45.0,
                        height: 100.0,
                    },
                },
            }),
            expand_parent: None,
            size: Some(CanvasSize {
                width: 40.0,
                height: 20.0,
            }),
            hidden: false,
            collapsed: false,
            ports: Vec::new(),
            data: Value::Null,
        },
    );

    let (graph, view) = insert_graph_view(&mut host, graph_value);

    let mut canvas = NodeGraphCanvas::new(graph.clone(), view.clone());
    canvas.sync_view_state(&mut host);

    view.update(&mut host, |s, _cx| {
        s.selected_nodes = vec![node_id];
    })
    .unwrap();

    let mut services = NullServices::default();
    let mut tree: fret_ui::UiTree<TestUiHostImpl> = fret_ui::UiTree::new();
    let mut cx = command_cx(&mut host, &mut services, &mut tree);

    assert!(canvas.command(&mut cx, &CommandId::from(CMD_NODE_GRAPH_NUDGE_RIGHT_FAST)));
    assert_eq!(canvas.history.undo_len(), 1);
    assert_eq!(read_node_pos(&mut host, &graph, node_id).x, 5.0);
}

#[test]
fn select_all_selects_nodes_groups_and_edges_and_respects_edge_selectable() {
    let mut host = TestUiHostImpl::default();

    let mut graph = Graph::new(GraphId::new());
    let kind = NodeKindKey::new("test.node");

    let a = NodeId::new();
    let b = NodeId::new();
    graph.nodes.insert(
        a,
        Node {
            kind: kind.clone(),
            kind_version: 1,
            pos: CanvasPoint { x: 0.0, y: 0.0 },
            selectable: None,
            draggable: None,
            connectable: None,
            deletable: None,
            parent: None,
            extent: None,
            expand_parent: None,
            size: None,
            hidden: false,
            collapsed: false,
            ports: Vec::new(),
            data: Value::Null,
        },
    );
    graph.nodes.insert(
        b,
        Node {
            kind,
            kind_version: 1,
            pos: CanvasPoint { x: 10.0, y: 0.0 },
            selectable: Some(false),
            draggable: None,
            connectable: None,
            deletable: None,
            parent: None,
            extent: None,
            expand_parent: None,
            size: None,
            hidden: false,
            collapsed: false,
            ports: Vec::new(),
            data: Value::Null,
        },
    );

    let p_out = PortId::new();
    let p_in = PortId::new();
    graph.ports.insert(
        p_out,
        Port {
            node: a,
            key: PortKey::new("out"),
            dir: PortDirection::Out,
            kind: PortKind::Data,
            capacity: PortCapacity::Multi,
            connectable: None,
            connectable_start: None,
            connectable_end: None,
            ty: None,
            data: Value::Null,
        },
    );
    graph.ports.insert(
        p_in,
        Port {
            node: b,
            key: PortKey::new("in"),
            dir: PortDirection::In,
            kind: PortKind::Data,
            capacity: PortCapacity::Single,
            connectable: None,
            connectable_start: None,
            connectable_end: None,
            ty: None,
            data: Value::Null,
        },
    );
    graph.nodes.get_mut(&a).unwrap().ports.push(p_out);
    graph.nodes.get_mut(&b).unwrap().ports.push(p_in);

    let e_ok = EdgeId::new();
    let e_no = EdgeId::new();
    graph.edges.insert(
        e_ok,
        Edge {
            kind: EdgeKind::Data,
            from: p_out,
            to: p_in,
            selectable: None,
            deletable: None,
            reconnectable: None,
        },
    );
    graph.edges.insert(
        e_no,
        Edge {
            kind: EdgeKind::Data,
            from: p_out,
            to: p_in,
            selectable: Some(false),
            deletable: None,
            reconnectable: None,
        },
    );

    let g0 = GroupId::new();
    graph.groups.insert(
        g0,
        Group {
            title: "Group".to_string(),
            rect: CanvasRect {
                origin: CanvasPoint { x: 0.0, y: 0.0 },
                size: CanvasSize {
                    width: 100.0,
                    height: 80.0,
                },
            },
            color: None,
        },
    );

    let (graph, view) = insert_graph_view(&mut host, graph);

    let mut canvas = NodeGraphCanvas::new(graph, view.clone());
    canvas.sync_view_state(&mut host);

    view.update(&mut host, |s, _cx| {
        s.interaction.elements_selectable = true;
        s.interaction.edges_selectable = true;
        s.selected_nodes.clear();
        s.selected_edges.clear();
        s.selected_groups.clear();
    })
    .unwrap();

    canvas.interaction.focused_edge = Some(e_ok);
    canvas.interaction.focused_node = Some(a);
    canvas.interaction.focused_port = Some(p_out);
    canvas.interaction.focused_port_valid = true;
    canvas.interaction.focused_port_convertible = true;

    let mut services = NullServices::default();
    let mut tree: fret_ui::UiTree<TestUiHostImpl> = fret_ui::UiTree::new();
    let mut cx = command_cx(&mut host, &mut services, &mut tree);

    assert!(canvas.command(&mut cx, &CommandId::from(CMD_NODE_GRAPH_SELECT_ALL)));

    assert!(canvas.interaction.focused_edge.is_none());
    assert!(canvas.interaction.focused_node.is_none());
    assert!(canvas.interaction.focused_port.is_none());
    assert!(!canvas.interaction.focused_port_valid);
    assert!(!canvas.interaction.focused_port_convertible);

    let mut selected_nodes = view
        .read_ref(&host, |s| s.selected_nodes.clone())
        .unwrap_or_default();
    selected_nodes.sort();
    assert_eq!(selected_nodes, vec![a]);

    let mut selected_groups = view
        .read_ref(&host, |s| s.selected_groups.clone())
        .unwrap_or_default();
    selected_groups.sort();
    assert_eq!(selected_groups, vec![g0]);

    let mut selected_edges = view
        .read_ref(&host, |s| s.selected_edges.clone())
        .unwrap_or_default();
    selected_edges.sort();
    assert_eq!(selected_edges, vec![e_ok]);
}

#[test]
fn delete_selection_respects_node_deletable_and_keeps_undeletable_selected() {
    let mut host = TestUiHostImpl::default();
    let (mut graph_value, a, b) = make_test_graph_two_nodes();
    graph_value
        .nodes
        .get_mut(&a)
        .expect("node must exist")
        .deletable = Some(false);
    let (graph, view) = insert_graph_view(&mut host, graph_value);

    let mut canvas = NodeGraphCanvas::new(graph.clone(), view.clone());
    canvas.sync_view_state(&mut host);

    view.update(&mut host, |s, _cx| {
        s.selected_nodes = vec![a, b];
        s.selected_edges.clear();
        s.selected_groups.clear();
        s.interaction.nodes_deletable = true;
    })
    .unwrap();

    let mut services = NullServices::default();
    let mut tree: fret_ui::UiTree<TestUiHostImpl> = fret_ui::UiTree::new();
    let mut cx = command_cx(&mut host, &mut services, &mut tree);

    assert!(canvas.command(&mut cx, &CommandId::from(CMD_NODE_GRAPH_DELETE_SELECTION)));

    assert!(
        graph
            .read_ref(&mut host, |g| g.nodes.contains_key(&a))
            .unwrap_or(false)
    );
    assert!(
        !graph
            .read_ref(&mut host, |g| g.nodes.contains_key(&b))
            .unwrap_or(true)
    );

    let selected_nodes = view
        .read_ref(&host, |s| s.selected_nodes.clone())
        .unwrap_or_default();
    assert_eq!(selected_nodes, vec![a]);
    assert_eq!(canvas.history.undo_len(), 1);
}

#[test]
fn delete_selection_respects_edge_deletable_and_keeps_undeletable_selected() {
    let mut host = TestUiHostImpl::default();
    let (mut graph_value, _a, _a_in, a_out, _b, b_in) = make_test_graph_two_nodes_with_ports();

    let edge = EdgeId::new();
    graph_value.edges.insert(
        edge,
        Edge {
            kind: EdgeKind::Data,
            from: a_out,
            to: b_in,
            selectable: None,
            deletable: Some(false),
            reconnectable: None,
        },
    );

    let (graph, view) = insert_graph_view(&mut host, graph_value);

    let mut canvas = NodeGraphCanvas::new(graph.clone(), view.clone());
    canvas.sync_view_state(&mut host);

    view.update(&mut host, |s, _cx| {
        s.selected_nodes.clear();
        s.selected_groups.clear();
        s.selected_edges = vec![edge];
        s.interaction.edges_deletable = true;
    })
    .unwrap();

    let mut services = NullServices::default();
    let mut tree: fret_ui::UiTree<TestUiHostImpl> = fret_ui::UiTree::new();
    let mut cx = command_cx(&mut host, &mut services, &mut tree);

    assert!(canvas.command(&mut cx, &CommandId::from(CMD_NODE_GRAPH_DELETE_SELECTION)));

    assert_eq!(graph.read_ref(&mut host, |g| g.edges.len()).unwrap_or(0), 1);
    let selected_edges = view
        .read_ref(&host, |s| s.selected_edges.clone())
        .unwrap_or_default();
    assert_eq!(selected_edges, vec![edge]);
    assert_eq!(canvas.history.undo_len(), 0);
}

#[test]
fn align_left_moves_selected_nodes_and_records_history_entry() {
    let mut host = TestUiHostImpl::default();
    let (graph_value, a, b) = make_test_graph_two_nodes_with_size();
    let (graph, view) = insert_graph_view(&mut host, graph_value);

    let mut canvas = NodeGraphCanvas::new(graph.clone(), view.clone());
    canvas.sync_view_state(&mut host);

    view.update(&mut host, |s, _cx| {
        s.selected_nodes = vec![a, b];
    })
    .unwrap();

    let mut services = NullServices::default();
    let mut tree: fret_ui::UiTree<TestUiHostImpl> = fret_ui::UiTree::new();
    let mut cx = command_cx(&mut host, &mut services, &mut tree);

    assert!(canvas.command(&mut cx, &CommandId::from(CMD_NODE_GRAPH_ALIGN_LEFT)));
    assert_eq!(canvas.history.undo_len(), 1);
    assert_eq!(read_node_pos(&mut host, &graph, a).x, 0.0);
    assert_eq!(read_node_pos(&mut host, &graph, b).x, 0.0);

    assert!(canvas.undo_last(&mut host, None));
    assert_eq!(
        read_node_pos(&mut host, &graph, a),
        CanvasPoint { x: 0.0, y: 0.0 }
    );
    assert_eq!(
        read_node_pos(&mut host, &graph, b),
        CanvasPoint { x: 10.0, y: 5.0 }
    );
}

#[test]
fn align_right_respects_per_node_extent_rect() {
    let mut host = TestUiHostImpl::default();

    let mut graph_value = Graph::new(GraphId::new());
    let kind = NodeKindKey::new("test.node");

    let a = NodeId::new();
    let b = NodeId::new();
    graph_value.nodes.insert(
        a,
        Node {
            kind: kind.clone(),
            kind_version: 1,
            pos: CanvasPoint { x: 0.0, y: 0.0 },
            selectable: None,
            draggable: None,
            connectable: None,
            deletable: None,
            parent: None,
            extent: Some(crate::core::NodeExtent::Rect {
                rect: CanvasRect {
                    origin: CanvasPoint { x: 0.0, y: 0.0 },
                    size: CanvasSize {
                        width: 40.0,
                        height: 100.0,
                    },
                },
            }),
            expand_parent: None,
            size: Some(CanvasSize {
                width: 10.0,
                height: 20.0,
            }),
            hidden: false,
            collapsed: false,
            ports: Vec::new(),
            data: Value::Null,
        },
    );
    graph_value.nodes.insert(
        b,
        Node {
            kind,
            kind_version: 1,
            pos: CanvasPoint { x: 20.0, y: 0.0 },
            selectable: None,
            draggable: None,
            connectable: None,
            deletable: None,
            parent: None,
            extent: None,
            expand_parent: None,
            size: Some(CanvasSize {
                width: 40.0,
                height: 20.0,
            }),
            hidden: false,
            collapsed: false,
            ports: Vec::new(),
            data: Value::Null,
        },
    );

    let (graph, view) = insert_graph_view(&mut host, graph_value);

    let mut canvas = NodeGraphCanvas::new(graph.clone(), view.clone());
    canvas.sync_view_state(&mut host);

    view.update(&mut host, |s, _cx| {
        s.selected_nodes = vec![a, b];
    })
    .unwrap();

    let mut services = NullServices::default();
    let mut tree: fret_ui::UiTree<TestUiHostImpl> = fret_ui::UiTree::new();
    let mut cx = command_cx(&mut host, &mut services, &mut tree);

    assert!(canvas.command(&mut cx, &CommandId::from(CMD_NODE_GRAPH_ALIGN_RIGHT)));
    assert_eq!(canvas.history.undo_len(), 1);
    assert_eq!(read_node_pos(&mut host, &graph, a).x, 30.0);
    assert_eq!(read_node_pos(&mut host, &graph, b).x, 20.0);
}

#[test]
fn align_center_x_preserves_alignment_under_node_extent_bounds() {
    let mut host = TestUiHostImpl::default();

    let mut graph_value = Graph::new(GraphId::new());
    let kind = NodeKindKey::new("test.node");

    let a = NodeId::new();
    let b = NodeId::new();
    graph_value.nodes.insert(
        a,
        Node {
            kind: kind.clone(),
            kind_version: 1,
            pos: CanvasPoint { x: 90.0, y: 0.0 },
            selectable: None,
            draggable: None,
            connectable: None,
            deletable: None,
            parent: None,
            extent: None,
            expand_parent: None,
            size: Some(CanvasSize {
                width: 10.0,
                height: 20.0,
            }),
            hidden: false,
            collapsed: false,
            ports: Vec::new(),
            data: Value::Null,
        },
    );
    graph_value.nodes.insert(
        b,
        Node {
            kind,
            kind_version: 1,
            pos: CanvasPoint { x: 150.0, y: 0.0 },
            selectable: None,
            draggable: None,
            connectable: None,
            deletable: None,
            parent: None,
            extent: None,
            expand_parent: None,
            size: Some(CanvasSize {
                width: 40.0,
                height: 20.0,
            }),
            hidden: false,
            collapsed: false,
            ports: Vec::new(),
            data: Value::Null,
        },
    );

    let (graph, view) = insert_graph_view(&mut host, graph_value);

    let mut canvas = NodeGraphCanvas::new(graph.clone(), view.clone());
    canvas.sync_view_state(&mut host);

    view.update(&mut host, |s, _cx| {
        s.selected_nodes = vec![a, b];
        s.interaction.node_extent = Some(CanvasRect {
            origin: CanvasPoint { x: 0.0, y: 0.0 },
            size: CanvasSize {
                width: 100.0,
                height: 100.0,
            },
        });
    })
    .unwrap();

    let mut services = NullServices::default();
    let mut tree: fret_ui::UiTree<TestUiHostImpl> = fret_ui::UiTree::new();
    let mut cx = command_cx(&mut host, &mut services, &mut tree);

    assert!(canvas.command(&mut cx, &CommandId::from(CMD_NODE_GRAPH_ALIGN_CENTER_X)));
    assert_eq!(canvas.history.undo_len(), 1);

    let pos_a = read_node_pos(&mut host, &graph, a);
    let pos_b = read_node_pos(&mut host, &graph, b);
    assert_eq!(pos_a.x, 75.0);
    assert_eq!(pos_b.x, 60.0);

    let center_a = pos_a.x + 5.0;
    let center_b = pos_b.x + 20.0;
    assert!((center_a - center_b).abs() <= 1.0e-6);
}

#[test]
fn distribute_x_clamps_nodes_to_node_extent_rect_like_xyflow() {
    let mut host = TestUiHostImpl::default();

    let mut graph_value = Graph::new(GraphId::new());
    let kind = NodeKindKey::new("test.node");

    let a = NodeId::new();
    let b = NodeId::new();
    let c = NodeId::new();
    let d = NodeId::new();

    graph_value.nodes.insert(
        a,
        Node {
            kind: kind.clone(),
            kind_version: 1,
            pos: CanvasPoint { x: 0.0, y: 0.0 },
            selectable: None,
            draggable: None,
            connectable: None,
            deletable: None,
            parent: None,
            extent: None,
            expand_parent: None,
            size: Some(CanvasSize {
                width: 10.0,
                height: 20.0,
            }),
            hidden: false,
            collapsed: false,
            ports: Vec::new(),
            data: Value::Null,
        },
    );
    graph_value.nodes.insert(
        b,
        Node {
            kind: kind.clone(),
            kind_version: 1,
            pos: CanvasPoint { x: 10.0, y: 0.0 },
            selectable: None,
            draggable: None,
            connectable: None,
            deletable: None,
            parent: None,
            extent: None,
            expand_parent: None,
            size: Some(CanvasSize {
                width: 10.0,
                height: 20.0,
            }),
            hidden: false,
            collapsed: false,
            ports: Vec::new(),
            data: Value::Null,
        },
    );
    graph_value.nodes.insert(
        c,
        Node {
            kind: kind.clone(),
            kind_version: 1,
            pos: CanvasPoint { x: 60.0, y: 0.0 },
            selectable: None,
            draggable: None,
            connectable: None,
            deletable: None,
            parent: None,
            extent: None,
            expand_parent: None,
            size: Some(CanvasSize {
                width: 80.0,
                height: 20.0,
            }),
            hidden: false,
            collapsed: false,
            ports: Vec::new(),
            data: Value::Null,
        },
    );
    graph_value.nodes.insert(
        d,
        Node {
            kind,
            kind_version: 1,
            pos: CanvasPoint { x: 90.0, y: 0.0 },
            selectable: None,
            draggable: None,
            connectable: None,
            deletable: None,
            parent: None,
            extent: None,
            expand_parent: None,
            size: Some(CanvasSize {
                width: 10.0,
                height: 20.0,
            }),
            hidden: false,
            collapsed: false,
            ports: Vec::new(),
            data: Value::Null,
        },
    );

    let (graph, view) = insert_graph_view(&mut host, graph_value);

    let mut canvas = NodeGraphCanvas::new(graph.clone(), view.clone());
    canvas.sync_view_state(&mut host);

    view.update(&mut host, |s, _cx| {
        s.selected_nodes = vec![a, b, c, d];
        s.interaction.node_extent = Some(CanvasRect {
            origin: CanvasPoint { x: 0.0, y: 0.0 },
            size: CanvasSize {
                width: 100.0,
                height: 100.0,
            },
        });
    })
    .unwrap();

    let mut services = NullServices::default();
    let mut tree: fret_ui::UiTree<TestUiHostImpl> = fret_ui::UiTree::new();
    let mut cx = command_cx(&mut host, &mut services, &mut tree);

    assert!(canvas.command(&mut cx, &CommandId::from(CMD_NODE_GRAPH_DISTRIBUTE_X)));
    assert_eq!(canvas.history.undo_len(), 1);

    assert_eq!(read_node_pos(&mut host, &graph, a).x, 0.0);
    assert_eq!(read_node_pos(&mut host, &graph, b).x, 30.0);

    // Desired position would be x=25, but node extent clamps to max_x=20 for a 80px-wide node.
    assert_eq!(read_node_pos(&mut host, &graph, c).x, 20.0);
    assert_eq!(read_node_pos(&mut host, &graph, d).x, 90.0);
}

#[test]
fn distribute_x_clamps_selected_group_children_to_node_extent_rect_like_xyflow() {
    let mut host = TestUiHostImpl::default();

    let mut graph_value = Graph::new(GraphId::new());
    let kind = NodeKindKey::new("test.node");

    let left = NodeId::new();
    let right = NodeId::new();
    let child = NodeId::new();

    let group_id = GroupId::new();
    graph_value.groups.insert(
        group_id,
        Group {
            title: "G".to_string(),
            rect: CanvasRect {
                origin: CanvasPoint { x: 10.0, y: 0.0 },
                size: CanvasSize {
                    width: 20.0,
                    height: 20.0,
                },
            },
            color: None,
        },
    );

    graph_value.nodes.insert(
        left,
        Node {
            kind: kind.clone(),
            kind_version: 1,
            pos: CanvasPoint { x: 0.0, y: 0.0 },
            selectable: None,
            draggable: None,
            connectable: None,
            deletable: None,
            parent: None,
            extent: None,
            expand_parent: None,
            size: Some(CanvasSize {
                width: 10.0,
                height: 20.0,
            }),
            hidden: false,
            collapsed: false,
            ports: Vec::new(),
            data: Value::Null,
        },
    );
    graph_value.nodes.insert(
        right,
        Node {
            kind: kind.clone(),
            kind_version: 1,
            pos: CanvasPoint { x: 90.0, y: 0.0 },
            selectable: None,
            draggable: None,
            connectable: None,
            deletable: None,
            parent: None,
            extent: None,
            expand_parent: None,
            size: Some(CanvasSize {
                width: 10.0,
                height: 20.0,
            }),
            hidden: false,
            collapsed: false,
            ports: Vec::new(),
            data: Value::Null,
        },
    );

    graph_value.nodes.insert(
        child,
        Node {
            kind,
            kind_version: 1,
            pos: CanvasPoint { x: 50.0, y: 0.0 },
            selectable: None,
            draggable: None,
            connectable: None,
            deletable: None,
            parent: Some(group_id),
            extent: None,
            expand_parent: None,
            size: Some(CanvasSize {
                width: 40.0,
                height: 20.0,
            }),
            hidden: false,
            collapsed: false,
            ports: Vec::new(),
            data: Value::Null,
        },
    );

    let (graph, view) = insert_graph_view(&mut host, graph_value);

    let mut canvas = NodeGraphCanvas::new(graph.clone(), view.clone());
    canvas.sync_view_state(&mut host);

    view.update(&mut host, |s, _cx| {
        s.selected_nodes = vec![left, right];
        s.selected_groups = vec![group_id];
        s.interaction.node_extent = Some(CanvasRect {
            origin: CanvasPoint { x: 0.0, y: 0.0 },
            size: CanvasSize {
                width: 100.0,
                height: 100.0,
            },
        });
    })
    .unwrap();

    let mut services = NullServices::default();
    let mut tree: fret_ui::UiTree<TestUiHostImpl> = fret_ui::UiTree::new();
    let mut cx = command_cx(&mut host, &mut services, &mut tree);

    assert!(canvas.command(&mut cx, &CommandId::from(CMD_NODE_GRAPH_DISTRIBUTE_X)));
    assert_eq!(canvas.history.undo_len(), 1);

    // Left/right are the endpoints and remain fixed; the group is the interior element.
    assert_eq!(read_node_pos(&mut host, &graph, left).x, 0.0);
    assert_eq!(read_node_pos(&mut host, &graph, right).x, 90.0);

    // The group's desired shift would move the child to x=80. Node extent clamps to max_x=60.
    assert_eq!(read_node_pos(&mut host, &graph, child).x, 60.0);
    let group_origin_x = graph
        .read_ref(&mut host, |g| {
            g.groups.get(&group_id).map(|gr| gr.rect.origin.x)
        })
        .ok()
        .flatten()
        .unwrap_or_default();
    assert_eq!(group_origin_x, 20.0);
}

#[test]
fn distribute_x_clamps_selected_group_children_to_node_extent_rect_from_node_extents() {
    let mut host = TestUiHostImpl::default();

    let mut graph_value = Graph::new(GraphId::new());
    let kind = NodeKindKey::new("test.node");

    let left = NodeId::new();
    let right = NodeId::new();
    let child = NodeId::new();

    let group_id = GroupId::new();
    graph_value.groups.insert(
        group_id,
        Group {
            title: "G".to_string(),
            rect: CanvasRect {
                origin: CanvasPoint { x: 10.0, y: 0.0 },
                size: CanvasSize {
                    width: 20.0,
                    height: 20.0,
                },
            },
            color: None,
        },
    );

    graph_value.nodes.insert(
        left,
        Node {
            kind: kind.clone(),
            kind_version: 1,
            pos: CanvasPoint { x: 0.0, y: 0.0 },
            selectable: None,
            draggable: None,
            connectable: None,
            deletable: None,
            parent: None,
            extent: None,
            expand_parent: None,
            size: Some(CanvasSize {
                width: 10.0,
                height: 20.0,
            }),
            hidden: false,
            collapsed: false,
            ports: Vec::new(),
            data: Value::Null,
        },
    );
    graph_value.nodes.insert(
        right,
        Node {
            kind: kind.clone(),
            kind_version: 1,
            pos: CanvasPoint { x: 90.0, y: 0.0 },
            selectable: None,
            draggable: None,
            connectable: None,
            deletable: None,
            parent: None,
            extent: None,
            expand_parent: None,
            size: Some(CanvasSize {
                width: 10.0,
                height: 20.0,
            }),
            hidden: false,
            collapsed: false,
            ports: Vec::new(),
            data: Value::Null,
        },
    );

    graph_value.nodes.insert(
        child,
        Node {
            kind,
            kind_version: 1,
            pos: CanvasPoint { x: 50.0, y: 0.0 },
            selectable: None,
            draggable: None,
            connectable: None,
            deletable: None,
            parent: Some(group_id),
            extent: Some(crate::core::NodeExtent::Rect {
                rect: CanvasRect {
                    origin: CanvasPoint { x: 40.0, y: 0.0 },
                    size: CanvasSize {
                        width: 60.0,
                        height: 100.0,
                    },
                },
            }),
            expand_parent: None,
            size: Some(CanvasSize {
                width: 40.0,
                height: 20.0,
            }),
            hidden: false,
            collapsed: false,
            ports: Vec::new(),
            data: Value::Null,
        },
    );

    let (graph, view) = insert_graph_view(&mut host, graph_value);

    let mut canvas = NodeGraphCanvas::new(graph.clone(), view.clone());
    canvas.sync_view_state(&mut host);

    view.update(&mut host, |s, _cx| {
        s.selected_nodes = vec![left, right];
        s.selected_groups = vec![group_id];
    })
    .unwrap();

    let mut services = NullServices::default();
    let mut tree: fret_ui::UiTree<TestUiHostImpl> = fret_ui::UiTree::new();
    let mut cx = command_cx(&mut host, &mut services, &mut tree);

    assert!(canvas.command(&mut cx, &CommandId::from(CMD_NODE_GRAPH_DISTRIBUTE_X)));
    assert_eq!(canvas.history.undo_len(), 1);
    assert_eq!(read_node_pos(&mut host, &graph, child).x, 60.0);

    let group_origin_x = graph
        .read_ref(&mut host, |g| {
            g.groups.get(&group_id).map(|gr| gr.rect.origin.x)
        })
        .ok()
        .flatten()
        .unwrap_or_default();
    assert_eq!(group_origin_x, 20.0);
}

#[test]
fn focus_next_cycles_nodes_and_updates_selection() {
    let mut host = TestUiHostImpl::default();
    let (graph_value, a, b) = make_test_graph_two_nodes();
    let (graph, view) = insert_graph_view(&mut host, graph_value);

    let mut canvas = NodeGraphCanvas::new(graph, view.clone());
    canvas.sync_view_state(&mut host);

    view.update(&mut host, |s, _cx| {
        s.draw_order = vec![a, b];
        s.selected_nodes.clear();
        s.selected_edges.clear();
        s.selected_groups.clear();
    })
    .unwrap();

    let mut services = NullServices::default();
    let mut tree: fret_ui::UiTree<TestUiHostImpl> = fret_ui::UiTree::new();

    {
        let mut cx = command_cx(&mut host, &mut services, &mut tree);
        assert!(canvas.command(&mut cx, &CommandId::from(CMD_NODE_GRAPH_FOCUS_NEXT)));
    }
    let selected = view
        .read_ref(&host, |s| s.selected_nodes.clone())
        .unwrap_or_default();
    assert_eq!(selected, vec![a]);

    {
        let mut cx = command_cx(&mut host, &mut services, &mut tree);
        assert!(canvas.command(&mut cx, &CommandId::from(CMD_NODE_GRAPH_FOCUS_NEXT)));
    }
    let selected = view
        .read_ref(&host, |s| s.selected_nodes.clone())
        .unwrap_or_default();
    assert_eq!(selected, vec![b]);

    {
        let mut cx = command_cx(&mut host, &mut services, &mut tree);
        assert!(canvas.command(&mut cx, &CommandId::from(CMD_NODE_GRAPH_FOCUS_PREV)));
    }
    let selected = view
        .read_ref(&host, |s| s.selected_nodes.clone())
        .unwrap_or_default();
    assert_eq!(selected, vec![a]);
}

#[test]
fn focus_next_skips_unselectable_nodes() {
    let mut host = TestUiHostImpl::default();
    let (mut graph_value, a, b) = make_test_graph_two_nodes();
    graph_value
        .nodes
        .get_mut(&a)
        .expect("node exists")
        .selectable = Some(false);

    let (graph, view) = insert_graph_view(&mut host, graph_value);

    let mut canvas = NodeGraphCanvas::new(graph, view.clone());
    canvas.sync_view_state(&mut host);

    view.update(&mut host, |s, _cx| {
        s.interaction.elements_selectable = true;
        s.draw_order = vec![a, b];
        s.selected_nodes.clear();
        s.selected_edges.clear();
        s.selected_groups.clear();
    })
    .unwrap();

    let mut services = NullServices::default();
    let mut tree: fret_ui::UiTree<TestUiHostImpl> = fret_ui::UiTree::new();
    let mut cx = command_cx(&mut host, &mut services, &mut tree);

    assert!(canvas.command(&mut cx, &CommandId::from(CMD_NODE_GRAPH_FOCUS_NEXT)));
    let selected = view
        .read_ref(&host, |s| s.selected_nodes.clone())
        .unwrap_or_default();
    assert_eq!(selected, vec![b]);
}

#[test]
fn focus_next_port_cycles_ports_within_focused_node() {
    let mut host = TestUiHostImpl::default();
    let (graph_value, a, a_in, a_out, _b, _b_in) = make_test_graph_two_nodes_with_ports();
    let (graph, view) = insert_graph_view(&mut host, graph_value);

    let mut canvas = NodeGraphCanvas::new(graph, view.clone());
    canvas.sync_view_state(&mut host);

    view.update(&mut host, |s, _cx| {
        s.selected_nodes = vec![a];
        s.selected_edges.clear();
        s.selected_groups.clear();
    })
    .unwrap();

    let mut services = NullServices::default();
    let mut tree: fret_ui::UiTree<TestUiHostImpl> = fret_ui::UiTree::new();

    {
        let mut cx = command_cx(&mut host, &mut services, &mut tree);
        assert!(canvas.command(&mut cx, &CommandId::from(CMD_NODE_GRAPH_FOCUS_NEXT_PORT)));
    }
    assert_eq!(canvas.interaction.focused_port, Some(a_in));

    {
        let mut cx = command_cx(&mut host, &mut services, &mut tree);
        assert!(canvas.command(&mut cx, &CommandId::from(CMD_NODE_GRAPH_FOCUS_NEXT_PORT)));
    }
    assert_eq!(canvas.interaction.focused_port, Some(a_out));

    {
        let mut cx = command_cx(&mut host, &mut services, &mut tree);
        assert!(canvas.command(&mut cx, &CommandId::from(CMD_NODE_GRAPH_FOCUS_PREV_PORT)));
    }
    assert_eq!(canvas.interaction.focused_port, Some(a_in));
}

#[test]
fn focus_next_port_filters_by_wire_direction() {
    let mut host = TestUiHostImpl::default();
    let (graph_value, a, a_in, a_out, _b, _b_in) = make_test_graph_two_nodes_with_ports();
    let (graph, view) = insert_graph_view(&mut host, graph_value);

    let mut canvas = NodeGraphCanvas::new(graph, view.clone());
    canvas.sync_view_state(&mut host);

    view.update(&mut host, |s, _cx| {
        s.selected_nodes = vec![a];
        s.selected_edges.clear();
        s.selected_groups.clear();
    })
    .unwrap();

    canvas.interaction.wire_drag = Some(WireDrag {
        kind: WireDragKind::New {
            from: a_out,
            bundle: Vec::new(),
        },
        pos: Point::new(Px(0.0), Px(0.0)),
    });

    let mut services = NullServices::default();
    let mut tree: fret_ui::UiTree<TestUiHostImpl> = fret_ui::UiTree::new();
    let mut cx = command_cx(&mut host, &mut services, &mut tree);

    assert!(canvas.command(&mut cx, &CommandId::from(CMD_NODE_GRAPH_FOCUS_NEXT_PORT)));
    assert_eq!(canvas.interaction.focused_port, Some(a_in));
}

#[test]
fn activate_starts_and_commits_wire_drag() {
    let mut host = TestUiHostImpl::default();
    let (graph_value, _a, _a_in, a_out, _b, b_in) = make_test_graph_two_nodes_with_ports();
    let (graph_model, view) = insert_graph_view(&mut host, graph_value);

    let mut canvas = NodeGraphCanvas::new(graph_model.clone(), view);
    canvas.sync_view_state(&mut host);

    canvas.interaction.focused_port = Some(a_out);

    let mut services = NullServices::default();
    let mut tree: fret_ui::UiTree<TestUiHostImpl> = fret_ui::UiTree::new();

    {
        let mut cx = command_cx(&mut host, &mut services, &mut tree);
        assert!(canvas.command(&mut cx, &CommandId::from(CMD_NODE_GRAPH_ACTIVATE)));
    }
    assert!(canvas.interaction.wire_drag.is_some());
    assert!(canvas.interaction.click_connect);
    assert!(canvas.interaction.focused_port.is_none());

    canvas.interaction.focused_port = Some(b_in);
    {
        let mut cx = command_cx(&mut host, &mut services, &mut tree);
        assert!(canvas.command(&mut cx, &CommandId::from(CMD_NODE_GRAPH_ACTIVATE)));
    }
    assert!(canvas.interaction.wire_drag.is_none());

    let edges_len = graph_model
        .read_ref(&mut host, |g| g.edges.len())
        .unwrap_or(0);
    assert_eq!(edges_len, 1);
}

#[test]
fn focus_port_right_moves_to_neighbor_node() {
    let mut host = TestUiHostImpl::default();
    let (graph_value, _a, _a_in, a_out, b, b_in) =
        make_test_graph_two_nodes_with_ports_spaced_x(500.0);
    let (graph, view) = insert_graph_view(&mut host, graph_value);

    let mut canvas = NodeGraphCanvas::new(graph, view.clone());
    canvas.sync_view_state(&mut host);

    canvas.interaction.focused_port = Some(a_out);
    canvas.interaction.focused_node = None;

    let mut services = NullServices::default();
    let mut tree: fret_ui::UiTree<TestUiHostImpl> = fret_ui::UiTree::new();
    let mut cx = command_cx(&mut host, &mut services, &mut tree);

    assert!(canvas.command(&mut cx, &CommandId::from(CMD_NODE_GRAPH_FOCUS_PORT_RIGHT)));
    assert_eq!(canvas.interaction.focused_node, Some(b));
    assert_eq!(canvas.interaction.focused_port, Some(b_in));
}

#[test]
fn focus_port_left_moves_back() {
    let mut host = TestUiHostImpl::default();
    let (graph_value, a, _a_in, a_out, _b, b_in) =
        make_test_graph_two_nodes_with_ports_spaced_x(500.0);
    let (graph, view) = insert_graph_view(&mut host, graph_value);

    let mut canvas = NodeGraphCanvas::new(graph, view.clone());
    canvas.sync_view_state(&mut host);

    canvas.interaction.focused_port = Some(b_in);

    let mut services = NullServices::default();
    let mut tree: fret_ui::UiTree<TestUiHostImpl> = fret_ui::UiTree::new();
    let mut cx = command_cx(&mut host, &mut services, &mut tree);

    assert!(canvas.command(&mut cx, &CommandId::from(CMD_NODE_GRAPH_FOCUS_PORT_LEFT)));
    assert_eq!(canvas.interaction.focused_node, Some(a));
    assert_eq!(canvas.interaction.focused_port, Some(a_out));
}
