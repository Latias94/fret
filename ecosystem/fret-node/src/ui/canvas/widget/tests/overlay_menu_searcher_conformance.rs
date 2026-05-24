use std::sync::Arc;

use fret_core::{
    KeyCode, Modifiers, MouseButton, Point, PointerEvent, PointerId, PointerType, Px, Rect, Size,
};
use fret_runtime::CommandId;
use fret_ui::Widget;

use crate::core::{
    CanvasPoint, CanvasRect, CanvasSize, Edge, EdgeId, EdgeKind, Graph, GraphId, Group, GroupId,
    Node, NodeId, NodeKindKey, Port, PortCapacity, PortDirection, PortId, PortKey, PortKind,
};
use crate::ui::commands::CMD_NODE_GRAPH_OPEN_CONVERSION_PICKER;
use crate::ui::presenter::{
    InsertNodeCandidate, NodeGraphContextMenuAction, NodeGraphContextMenuItem,
};
use crate::ui::style::NodeGraphStyle;

use super::prelude::{HitTestCtx, HitTestScratch, NodeGraphCanvas, overlay_hit};
use super::{NullServices, TestUiHostImpl, command_cx, event_cx, insert_graph_view_editor_config};
use crate::ui::canvas::searcher::{SEARCHER_MAX_VISIBLE_ROWS, SearcherRow, SearcherRowKind};
use crate::ui::canvas::state::{
    ContextMenuState, ContextMenuTarget, LastConversionContext, SearcherRowsMode, SearcherState,
};

fn bounds() -> Rect {
    Rect::new(
        Point::new(Px(0.0), Px(0.0)),
        Size::new(Px(800.0), Px(600.0)),
    )
}

fn context_menu_test_node(
    kind: NodeKindKey,
    pos: CanvasPoint,
    size: CanvasSize,
    ports: Vec<PortId>,
) -> Node {
    Node {
        kind,
        kind_version: 1,
        pos,
        selectable: None,
        draggable: None,
        connectable: None,
        deletable: None,
        parent: None,
        extent: None,
        expand_parent: None,
        size: Some(size),
        hidden: false,
        collapsed: false,
        ports,
        data: serde_json::Value::Null,
    }
}

fn context_menu_test_port(node: NodeId, key: &str, dir: PortDirection) -> Port {
    Port {
        node,
        key: PortKey::new(key),
        dir,
        kind: PortKind::Data,
        capacity: PortCapacity::Single,
        connectable: None,
        connectable_start: None,
        connectable_end: None,
        ty: None,
        data: serde_json::Value::Null,
    }
}

fn graph_with_context_targets() -> (Graph, GroupId, EdgeId, PortId, PortId) {
    let mut graph = Graph::new(GraphId::new());
    let kind = NodeKindKey::new("test.node");

    let group_id = GroupId::new();
    graph.groups.insert(
        group_id,
        Group {
            title: "Group".to_string(),
            rect: CanvasRect {
                origin: CanvasPoint { x: 10.0, y: 20.0 },
                size: CanvasSize {
                    width: 160.0,
                    height: 120.0,
                },
            },
            color: None,
        },
    );

    let a = NodeId::new();
    let a_out = PortId::new();
    graph.nodes.insert(
        a,
        context_menu_test_node(
            kind.clone(),
            CanvasPoint { x: 240.0, y: 80.0 },
            CanvasSize {
                width: 120.0,
                height: 60.0,
            },
            vec![a_out],
        ),
    );
    graph
        .ports
        .insert(a_out, context_menu_test_port(a, "out", PortDirection::Out));

    let b = NodeId::new();
    let b_in = PortId::new();
    graph.nodes.insert(
        b,
        context_menu_test_node(
            kind,
            CanvasPoint { x: 460.0, y: 80.0 },
            CanvasSize {
                width: 120.0,
                height: 60.0,
            },
            vec![b_in],
        ),
    );
    graph
        .ports
        .insert(b_in, context_menu_test_port(b, "in", PortDirection::In));

    let edge_id = EdgeId::new();
    graph.edges.insert(
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

    (graph, group_id, edge_id, a_out, b_in)
}

fn hit_edge_at(
    canvas: &mut NodeGraphCanvas,
    host: &mut TestUiHostImpl,
    snapshot: &crate::ui::canvas::state::ViewSnapshot,
    pos: Point,
) -> Option<EdgeId> {
    let (geom, index) = canvas.canvas_derived(&*host, snapshot);
    let this = canvas;
    this.graph
        .read_ref(host, |graph| {
            let mut scratch = HitTestScratch::default();
            let mut ctx =
                HitTestCtx::new(geom.as_ref(), index.as_ref(), snapshot.zoom, &mut scratch);
            this.hit_edge(graph, snapshot, &mut ctx, pos)
        })
        .ok()
        .flatten()
}

fn find_edge_hit_position(
    canvas: &mut NodeGraphCanvas,
    host: &mut TestUiHostImpl,
    snapshot: &crate::ui::canvas::state::ViewSnapshot,
    edge_id: EdgeId,
    from_port: PortId,
    to_port: PortId,
) -> Point {
    let (from, to) = {
        let (geom, _index) = canvas.canvas_derived(&*host, snapshot);
        let from = geom
            .ports
            .get(&from_port)
            .expect("source port geometry should exist")
            .center;
        let to = geom
            .ports
            .get(&to_port)
            .expect("target port geometry should exist")
            .center;
        (from, to)
    };

    (1..20)
        .map(|step| {
            let t = step as f32 / 20.0;
            Point::new(
                Px(from.x.0 + (to.x.0 - from.x.0) * t),
                Px(from.y.0 + (to.y.0 - from.y.0) * t),
            )
        })
        .find(|position| hit_edge_at(canvas, host, snapshot, *position) == Some(edge_id))
        .expect("edge should be hittable along the route between its ports")
}

fn open_context_menu_with_right_click(
    canvas: &mut NodeGraphCanvas,
    host: &mut TestUiHostImpl,
    position: Point,
) {
    let mut services = NullServices::default();
    let mut prevented_default_actions = fret_runtime::DefaultActionSet::default();
    let mut cx = event_cx(
        host,
        &mut services,
        bounds(),
        &mut prevented_default_actions,
    );
    canvas.event(
        &mut cx,
        &fret_core::Event::Pointer(PointerEvent::Down {
            pointer_id: PointerId::default(),
            position,
            button: MouseButton::Right,
            modifiers: Modifiers::default(),
            click_count: 1,
            pointer_type: PointerType::Mouse,
        }),
    );
}

fn rect_contains_rect(outer: Rect, inner: Rect) -> bool {
    let outer_x0 = outer.origin.x.0;
    let outer_y0 = outer.origin.y.0;
    let outer_x1 = outer.origin.x.0 + outer.size.width.0;
    let outer_y1 = outer.origin.y.0 + outer.size.height.0;

    let inner_x0 = inner.origin.x.0;
    let inner_y0 = inner.origin.y.0;
    let inner_x1 = inner.origin.x.0 + inner.size.width.0;
    let inner_y1 = inner.origin.y.0 + inner.size.height.0;

    inner_x0 >= outer_x0 && inner_y0 >= outer_y0 && inner_x1 <= outer_x1 && inner_y1 <= outer_y1
}

#[test]
fn context_menu_rect_scales_in_canvas_space_to_keep_screen_size_constant() {
    let mut style = NodeGraphStyle::default();
    style.paint.context_menu_width = 240.0;
    style.paint.context_menu_item_height = 20.0;
    style.paint.context_menu_padding = 6.0;

    let origin = Point::new(Px(10.0), Px(20.0));
    let item_count = 4;

    for zoom in [0.5, 1.0, 2.0, 4.0] {
        let rect = overlay_hit::context_menu_rect_at(&style, origin, item_count, zoom);
        assert!((rect.size.width.0 * zoom - style.paint.context_menu_width).abs() <= 1.0e-6);
        assert!(
            (rect.size.height.0 * zoom
                - (2.0 * style.paint.context_menu_padding
                    + style.paint.context_menu_item_height * item_count as f32))
                .abs()
                <= 1.0e-6
        );
    }
}

#[test]
fn hit_context_menu_item_returns_expected_item_index() {
    let mut style = NodeGraphStyle::default();
    style.paint.context_menu_width = 200.0;
    style.paint.context_menu_item_height = 10.0;
    style.paint.context_menu_padding = 2.0;

    let origin = Point::new(Px(100.0), Px(50.0));
    let menu = ContextMenuState {
        origin,
        invoked_at: origin,
        target: ContextMenuTarget::Background,
        items: vec![
            NodeGraphContextMenuItem {
                label: Arc::<str>::from("A"),
                enabled: true,
                action: NodeGraphContextMenuAction::Custom(1),
            },
            NodeGraphContextMenuItem {
                label: Arc::<str>::from("B"),
                enabled: true,
                action: NodeGraphContextMenuAction::Custom(2),
            },
            NodeGraphContextMenuItem {
                label: Arc::<str>::from("C"),
                enabled: true,
                action: NodeGraphContextMenuAction::Custom(3),
            },
        ],
        candidates: Vec::new(),
        hovered_item: None,
        active_item: 0,
        typeahead: String::new(),
    };

    let zoom = 2.0;
    let pad = style.paint.context_menu_padding / zoom;
    let item_h = style.paint.context_menu_item_height / zoom;

    // Inside first item.
    let p0 = Point::new(Px(origin.x.0 + 1.0), Px(origin.y.0 + pad + 0.5 * item_h));
    assert_eq!(
        overlay_hit::hit_context_menu_item(&style, &menu, p0, zoom),
        Some(0)
    );

    // Inside second item.
    let p1 = Point::new(Px(origin.x.0 + 1.0), Px(origin.y.0 + pad + 1.5 * item_h));
    assert_eq!(
        overlay_hit::hit_context_menu_item(&style, &menu, p1, zoom),
        Some(1)
    );

    // In padding above items.
    let p_pad = Point::new(Px(origin.x.0 + 1.0), Px(origin.y.0 + 0.5 * pad));
    assert_eq!(
        overlay_hit::hit_context_menu_item(&style, &menu, p_pad, zoom),
        None
    );

    // Outside rect.
    let outside = Point::new(Px(origin.x.0 - 10.0), Px(origin.y.0 - 10.0));
    assert_eq!(
        overlay_hit::hit_context_menu_item(&style, &menu, outside, zoom),
        None
    );
}

#[test]
fn hit_searcher_row_respects_scroll_and_header_region() {
    let mut style = NodeGraphStyle::default();
    style.paint.context_menu_width = 200.0;
    style.paint.context_menu_item_height = 10.0;
    style.paint.context_menu_padding = 2.0;

    let origin = Point::new(Px(10.0), Px(20.0));
    let rows: Vec<SearcherRow> = (0..20)
        .map(|ix| SearcherRow {
            kind: SearcherRowKind::Candidate { candidate_ix: ix },
            label: Arc::<str>::from("Row"),
            enabled: true,
        })
        .collect();

    let searcher = SearcherState {
        origin,
        invoked_at: origin,
        target: ContextMenuTarget::ConnectionInsertNodePicker {
            from: PortId::new(),
            at: crate::core::CanvasPoint::default(),
        },
        rows_mode: SearcherRowsMode::Catalog,
        query: String::new(),
        candidates: vec![InsertNodeCandidate {
            kind: NodeKindKey::new("test.kind"),
            label: Arc::<str>::from("Candidate"),
            enabled: true,
            template: None,
            payload: serde_json::Value::Null,
        }],
        recent_kinds: Vec::new(),
        rows,
        hovered_row: None,
        active_row: 0,
        scroll: 5,
    };

    let zoom = 1.0;
    let pad = style.paint.context_menu_padding / zoom;
    let item_h = style.paint.context_menu_item_height / zoom;
    let list_top = origin.y.0 + pad + item_h + pad;

    // Header region (title + padding) should not hit any row.
    let header_pos = Point::new(Px(origin.x.0 + 1.0), Px(origin.y.0 + pad + 0.5 * item_h));
    assert_eq!(
        overlay_hit::hit_searcher_row(&style, &searcher, header_pos, zoom),
        None
    );

    // First visible row at current scroll offset.
    let row0 = Point::new(Px(origin.x.0 + 1.0), Px(list_top + 0.5 * item_h));
    assert_eq!(
        overlay_hit::hit_searcher_row(&style, &searcher, row0, zoom),
        Some(searcher.scroll)
    );

    // A later visible row within the capped visible window.
    let row7 = Point::new(Px(origin.x.0 + 1.0), Px(list_top + 7.5 * item_h));
    assert_eq!(
        overlay_hit::hit_searcher_row(&style, &searcher, row7, zoom),
        Some(searcher.scroll + 7)
    );

    // Past the visible row window should not hit.
    let visible = searcher
        .rows
        .len()
        .saturating_sub(searcher.scroll)
        .min(SEARCHER_MAX_VISIBLE_ROWS);
    let below = Point::new(
        Px(origin.x.0 + 1.0),
        Px(list_top + (visible as f32 + 0.25) * item_h),
    );
    assert_eq!(
        overlay_hit::hit_searcher_row(&style, &searcher, below, zoom),
        None
    );
}

#[test]
fn build_searcher_rows_respects_explicit_rows_mode() {
    let candidates = vec![
        InsertNodeCandidate {
            kind: NodeKindKey::new("math.add"),
            label: Arc::<str>::from("Math/Add"),
            enabled: true,
            template: None,
            payload: serde_json::Value::Null,
        },
        InsertNodeCandidate {
            kind: NodeKindKey::new("math.mul"),
            label: Arc::<str>::from("Math/Mul"),
            enabled: true,
            template: None,
            payload: serde_json::Value::Null,
        },
    ];
    let recent_kinds = vec![NodeKindKey::new("math.add")];

    let catalog_rows = super::super::menu_session::build_searcher_rows(
        &candidates,
        "",
        &recent_kinds,
        SearcherRowsMode::Catalog,
    );
    let flat_rows = super::super::menu_session::build_searcher_rows(
        &candidates,
        "",
        &recent_kinds,
        SearcherRowsMode::Flat,
    );

    assert!(matches!(
        catalog_rows.first().map(|row| &row.kind),
        Some(SearcherRowKind::Header)
    ));
    assert!(
        flat_rows
            .iter()
            .all(|row| matches!(row.kind, SearcherRowKind::Candidate { .. }))
    );
    assert_eq!(flat_rows.len(), candidates.len());
}

#[test]
fn first_enabled_context_menu_item_skips_disabled_entries() {
    let items = vec![
        NodeGraphContextMenuItem {
            label: Arc::<str>::from("A"),
            enabled: false,
            action: NodeGraphContextMenuAction::Custom(1),
        },
        NodeGraphContextMenuItem {
            label: Arc::<str>::from("B"),
            enabled: false,
            action: NodeGraphContextMenuAction::Custom(2),
        },
        NodeGraphContextMenuItem {
            label: Arc::<str>::from("C"),
            enabled: true,
            action: NodeGraphContextMenuAction::Custom(3),
        },
    ];

    assert_eq!(
        super::super::menu_session::first_enabled_context_menu_item(&items),
        2
    );
}

#[test]
fn clamp_context_menu_origin_keeps_menu_rect_inside_visible_canvas_rect() {
    let mut host = TestUiHostImpl::default();
    let (graph, view, editor_config) =
        insert_graph_view_editor_config(&mut host, Graph::new(GraphId::new()));

    let _ = view.update(&mut host, |s, _cx| {
        s.pan = CanvasPoint { x: 100.0, y: -50.0 };
        s.zoom = 2.0;
    });

    let mut canvas = new_canvas!(host, graph, view.clone(), editor_config);
    let snapshot = canvas.sync_view_state(&mut host);

    let viewport = NodeGraphCanvas::viewport_from_snapshot(bounds(), &snapshot);
    let vis = viewport.visible_canvas_rect();

    let item_count = 12;
    let desired = Point::new(
        Px(vis.origin.x.0 + vis.size.width.0 + 1_000.0),
        Px(vis.origin.y.0 + vis.size.height.0 + 1_000.0),
    );
    let origin = canvas.clamp_context_menu_origin(desired, item_count, bounds(), &snapshot);
    let rect = overlay_hit::context_menu_rect_at(&canvas.style, origin, item_count, snapshot.zoom);
    assert!(
        rect_contains_rect(vis, rect),
        "expected clamped context menu rect to remain inside the visible canvas rect"
    );
}

#[test]
fn right_click_background_opens_background_context_menu_with_paste_disabled_without_window() {
    let mut host = TestUiHostImpl::default();
    let (graph_value, _group_id, _edge_id, _from_port, _to_port) = graph_with_context_targets();
    let (graph, view, editor_config) = insert_graph_view_editor_config(&mut host, graph_value);
    let _ = view.update(&mut host, |s, _cx| {
        s.selected_nodes.push(NodeId::new());
    });
    let mut canvas = new_canvas!(host, graph, view.clone(), editor_config);

    open_context_menu_with_right_click(&mut canvas, &mut host, Point::new(Px(700.0), Px(500.0)));

    let menu = canvas
        .interaction
        .context_menu
        .as_ref()
        .expect("right-click background should open a context menu");
    assert!(matches!(menu.target, ContextMenuTarget::Background));
    assert_eq!(menu.invoked_at, Point::new(Px(700.0), Px(500.0)));
    assert_eq!(
        menu.items
            .iter()
            .filter(|item| item.label.as_ref() == "Paste")
            .count(),
        1
    );
    let paste = menu
        .items
        .iter()
        .find(|item| item.label.as_ref() == "Paste")
        .expect("background menu should include Paste");
    assert!(!paste.enabled);
    let delete = menu
        .items
        .iter()
        .find(|item| item.label.as_ref() == "Delete Selection")
        .expect("background menu should include Delete Selection");
    assert!(delete.enabled);
}

#[test]
fn right_click_group_opens_group_context_menu_and_selects_group() {
    let mut host = TestUiHostImpl::default();
    let (graph_value, group_id, _edge_id, _from_port, _to_port) = graph_with_context_targets();
    let (graph, view, editor_config) = insert_graph_view_editor_config(&mut host, graph_value);
    let mut canvas = new_canvas!(host, graph, view.clone(), editor_config);

    open_context_menu_with_right_click(&mut canvas, &mut host, Point::new(Px(20.0), Px(25.0)));

    let menu = canvas
        .interaction
        .context_menu
        .as_ref()
        .expect("right-click group should open a context menu");
    assert!(matches!(menu.target, ContextMenuTarget::Group(id) if id == group_id));
    assert_eq!(
        menu.items.first().map(|item| item.label.as_ref()),
        Some("Bring to Front")
    );
    let selected_groups = view
        .read_ref(&host, |state| state.selected_groups.clone())
        .expect("view state should be readable");
    assert_eq!(selected_groups, vec![group_id]);
}

#[test]
fn right_click_edge_opens_edge_context_menu_and_selects_edge() {
    let mut host = TestUiHostImpl::default();
    let (graph_value, _group_id, edge_id, from_port, to_port) = graph_with_context_targets();
    let (graph, view, editor_config) = insert_graph_view_editor_config(&mut host, graph_value);
    let mut canvas = new_canvas!(host, graph, view.clone(), editor_config);

    let snapshot = canvas.sync_view_state(&mut host);
    let position = find_edge_hit_position(
        &mut canvas,
        &mut host,
        &snapshot,
        edge_id,
        from_port,
        to_port,
    );

    open_context_menu_with_right_click(&mut canvas, &mut host, position);

    let menu = canvas
        .interaction
        .context_menu
        .as_ref()
        .expect("right-click edge should open a context menu");
    assert!(matches!(menu.target, ContextMenuTarget::Edge(id) if id == edge_id));
    assert!(
        menu.items
            .iter()
            .any(|item| item.label.as_ref() == "Insert Node...")
    );
    assert!(
        menu.items
            .iter()
            .any(|item| item.label.as_ref() == "Insert Reroute")
    );
    assert!(
        menu.items
            .iter()
            .any(|item| item.label.as_ref() == "Delete")
    );
    let selected_edges = view
        .read_ref(&host, |state| state.selected_edges.clone())
        .expect("view state should be readable");
    assert_eq!(selected_edges, vec![edge_id]);
}

#[test]
fn clamp_searcher_origin_keeps_rect_inside_visible_canvas_rect() {
    let mut host = TestUiHostImpl::default();
    let (graph, view, editor_config) =
        insert_graph_view_editor_config(&mut host, Graph::new(GraphId::new()));

    let _ = view.update(&mut host, |s, _cx| {
        s.pan = CanvasPoint { x: -250.0, y: 75.0 };
        s.zoom = 0.75;
    });

    let mut canvas = new_canvas!(host, graph, view, editor_config);
    let snapshot = canvas.sync_view_state(&mut host);

    let viewport = NodeGraphCanvas::viewport_from_snapshot(bounds(), &snapshot);
    let vis = viewport.visible_canvas_rect();

    let visible_rows = 8;
    let desired = Point::new(Px(vis.origin.x.0 - 10_000.0), Px(vis.origin.y.0 - 10_000.0));
    let origin = canvas.clamp_searcher_origin(desired, visible_rows, bounds(), &snapshot);
    let rect = overlay_hit::searcher_rect_at(&canvas.style, origin, visible_rows, snapshot.zoom);
    assert!(
        rect_contains_rect(vis, rect),
        "expected clamped searcher rect to remain inside the visible canvas rect"
    );
}

#[test]
fn open_conversion_command_reuses_searcher_install_to_replace_context_menu() {
    let mut host = TestUiHostImpl::default();
    let (graph, view, editor_config) =
        insert_graph_view_editor_config(&mut host, Graph::new(GraphId::new()));
    let mut canvas = new_canvas!(host, graph, view, editor_config);
    canvas.sync_view_state(&mut host);
    canvas.interaction.last_bounds = Some(bounds());

    let from = PortId::new();
    let to = PortId::new();
    let at = CanvasPoint { x: 120.0, y: 48.0 };
    canvas.interaction.context_menu = Some(ContextMenuState {
        origin: Point::new(Px(16.0), Px(24.0)),
        invoked_at: Point::new(Px(16.0), Px(24.0)),
        target: ContextMenuTarget::Background,
        items: vec![NodeGraphContextMenuItem {
            label: Arc::<str>::from("Convert"),
            enabled: true,
            action: NodeGraphContextMenuAction::Custom(1),
        }],
        candidates: Vec::new(),
        hovered_item: None,
        active_item: 0,
        typeahead: String::new(),
    });
    canvas.interaction.last_conversion = Some(LastConversionContext {
        from,
        to,
        at,
        candidates: vec![InsertNodeCandidate {
            kind: NodeKindKey::new("math.add"),
            label: Arc::<str>::from("Math/Add"),
            enabled: true,
            template: None,
            payload: serde_json::Value::Null,
        }],
    });

    let mut services = NullServices::default();
    let mut tree: fret_ui::UiTree<TestUiHostImpl> = fret_ui::UiTree::new();
    let mut cx = command_cx(&mut host, &mut services, &mut tree);

    assert!(canvas.command(
        &mut cx,
        &CommandId::from(CMD_NODE_GRAPH_OPEN_CONVERSION_PICKER),
    ));
    assert!(canvas.interaction.context_menu.is_none());

    let searcher = canvas
        .interaction
        .searcher
        .as_ref()
        .expect("conversion command should open searcher");
    assert!(matches!(
        searcher.target,
        ContextMenuTarget::ConnectionConvertPicker {
            from: searcher_from,
            to: searcher_to,
            at: searcher_at,
        } if searcher_from == from
            && searcher_to == to
            && (searcher_at.x - at.x).abs() <= 1.0e-3
            && (searcher_at.y - at.y).abs() <= 1.0e-3
    ));
    assert!(matches!(searcher.rows_mode, SearcherRowsMode::Flat));
    assert_eq!(searcher.candidates.len(), 1);
    assert_eq!(searcher.invoked_at.x.0, at.x);
    assert_eq!(searcher.invoked_at.y.0, at.y);
}

#[test]
fn context_menu_command_pointer_activation_keeps_menu_closed_via_selection_take_path() {
    let mut host = TestUiHostImpl::default();
    let (graph, view, editor_config) =
        insert_graph_view_editor_config(&mut host, Graph::new(GraphId::new()));
    let mut canvas = new_canvas!(host, graph, view, editor_config);
    canvas.sync_view_state(&mut host);

    let origin = Point::new(Px(100.0), Px(50.0));
    canvas.interaction.context_menu = Some(ContextMenuState {
        origin,
        invoked_at: origin,
        target: ContextMenuTarget::Background,
        items: vec![NodeGraphContextMenuItem {
            label: Arc::<str>::from("Dispatch"),
            enabled: true,
            action: NodeGraphContextMenuAction::Command(CommandId::from("demo.command")),
        }],
        candidates: Vec::new(),
        hovered_item: None,
        active_item: 0,
        typeahead: String::new(),
    });

    let zoom = 1.0;
    let pad = canvas.style.paint.context_menu_padding / zoom;
    let item_h = canvas.style.paint.context_menu_item_height / zoom;
    let position = Point::new(Px(origin.x.0 + 1.0), Px(origin.y.0 + pad + 0.5 * item_h));

    let mut services = NullServices::default();
    let mut prevented_default_actions = fret_runtime::DefaultActionSet::default();
    let mut cx = event_cx(
        &mut host,
        &mut services,
        bounds(),
        &mut prevented_default_actions,
    );

    assert!(
        super::super::context_menu::handle_context_menu_pointer_down(
            &mut canvas,
            &mut cx,
            position,
            fret_core::MouseButton::Left,
            zoom,
        )
    );
    assert!(canvas.interaction.context_menu.is_none());
    assert!(host.effects.iter().any(|effect| matches!(
        effect,
        fret_runtime::Effect::Command { command, .. }
            if *command == CommandId::from("demo.command")
    )));
}

#[test]
fn context_menu_disabled_pointer_activation_keeps_menu_open() {
    let mut host = TestUiHostImpl::default();
    let (graph, view, editor_config) =
        insert_graph_view_editor_config(&mut host, Graph::new(GraphId::new()));
    let mut canvas = new_canvas!(host, graph, view, editor_config);
    canvas.sync_view_state(&mut host);

    let origin = Point::new(Px(100.0), Px(50.0));
    canvas.interaction.context_menu = Some(ContextMenuState {
        origin,
        invoked_at: origin,
        target: ContextMenuTarget::Background,
        items: vec![NodeGraphContextMenuItem {
            label: Arc::<str>::from("Disabled"),
            enabled: false,
            action: NodeGraphContextMenuAction::Command(CommandId::from("demo.disabled")),
        }],
        candidates: Vec::new(),
        hovered_item: None,
        active_item: 0,
        typeahead: String::new(),
    });

    let zoom = 1.0;
    let pad = canvas.style.paint.context_menu_padding / zoom;
    let item_h = canvas.style.paint.context_menu_item_height / zoom;
    let position = Point::new(Px(origin.x.0 + 1.0), Px(origin.y.0 + pad + 0.5 * item_h));

    let mut services = NullServices::default();
    let mut prevented_default_actions = fret_runtime::DefaultActionSet::default();
    let mut cx = event_cx(
        &mut host,
        &mut services,
        bounds(),
        &mut prevented_default_actions,
    );

    assert!(
        super::super::context_menu::handle_context_menu_pointer_down(
            &mut canvas,
            &mut cx,
            position,
            fret_core::MouseButton::Left,
            zoom,
        )
    );
    assert!(canvas.interaction.context_menu.is_some());
    assert!(!host.effects.iter().any(|effect| matches!(
        effect,
        fret_runtime::Effect::Command { command, .. }
            if *command == CommandId::from("demo.disabled")
    )));
}

#[test]
fn context_menu_enter_on_disabled_active_item_keeps_menu_open() {
    let mut host = TestUiHostImpl::default();
    let (graph, view, editor_config) =
        insert_graph_view_editor_config(&mut host, Graph::new(GraphId::new()));
    let mut canvas = new_canvas!(host, graph, view, editor_config);
    canvas.sync_view_state(&mut host);

    let origin = Point::new(Px(100.0), Px(50.0));
    canvas.interaction.context_menu = Some(ContextMenuState {
        origin,
        invoked_at: origin,
        target: ContextMenuTarget::Background,
        items: vec![NodeGraphContextMenuItem {
            label: Arc::<str>::from("Disabled"),
            enabled: false,
            action: NodeGraphContextMenuAction::Command(CommandId::from("demo.disabled")),
        }],
        candidates: Vec::new(),
        hovered_item: None,
        active_item: 0,
        typeahead: String::new(),
    });

    let mut services = NullServices::default();
    let mut prevented_default_actions = fret_runtime::DefaultActionSet::default();
    let mut cx = event_cx(
        &mut host,
        &mut services,
        bounds(),
        &mut prevented_default_actions,
    );

    assert!(super::super::context_menu::handle_context_menu_key_down(
        &mut canvas,
        &mut cx,
        KeyCode::Enter,
    ));
    assert!(canvas.interaction.context_menu.is_some());
    assert!(!host.effects.iter().any(|effect| matches!(
        effect,
        fret_runtime::Effect::Command { command, .. }
            if *command == CommandId::from("demo.disabled")
    )));
}
