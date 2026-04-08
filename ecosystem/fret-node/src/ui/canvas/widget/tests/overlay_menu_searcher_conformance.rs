use std::sync::Arc;

use fret_core::{Point, Px, Rect, Size};
use fret_runtime::CommandId;
use fret_ui::retained_bridge::Widget;

use crate::core::{CanvasPoint, Graph, GraphId, NodeKindKey, PortId};
use crate::ui::commands::CMD_NODE_GRAPH_OPEN_CONVERSION_PICKER;
use crate::ui::presenter::{
    InsertNodeCandidate, NodeGraphContextMenuAction, NodeGraphContextMenuItem,
};
use crate::ui::style::NodeGraphStyle;

use super::prelude::NodeGraphCanvas;
use super::prelude::overlay_hit;
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

    let mut canvas = new_canvas!(host, graph, view, editor_config);
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
