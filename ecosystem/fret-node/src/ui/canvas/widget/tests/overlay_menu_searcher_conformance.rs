use std::sync::Arc;

use fret_core::{Point, Px, Rect, Size};

use crate::core::{CanvasPoint, Graph, GraphId, NodeKindKey, PortId};
use crate::io::NodeGraphViewState;
use crate::ui::presenter::{
    InsertNodeCandidate, NodeGraphContextMenuAction, NodeGraphContextMenuItem,
};
use crate::ui::style::NodeGraphStyle;

use super::super::NodeGraphCanvas;
use super::super::overlay_hit;
use super::TestUiHostImpl;
use crate::ui::canvas::searcher::{SEARCHER_MAX_VISIBLE_ROWS, SearcherRow, SearcherRowKind};
use crate::ui::canvas::state::{ContextMenuState, ContextMenuTarget, SearcherState};

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
    style.context_menu_width = 240.0;
    style.context_menu_item_height = 20.0;
    style.context_menu_padding = 6.0;

    let origin = Point::new(Px(10.0), Px(20.0));
    let item_count = 4;

    for zoom in [0.5, 1.0, 2.0, 4.0] {
        let rect = overlay_hit::context_menu_rect_at(&style, origin, item_count, zoom);
        assert!((rect.size.width.0 * zoom - style.context_menu_width).abs() <= 1.0e-6);
        assert!(
            (rect.size.height.0 * zoom
                - (2.0 * style.context_menu_padding
                    + style.context_menu_item_height * item_count as f32))
                .abs()
                <= 1.0e-6
        );
    }
}

#[test]
fn hit_context_menu_item_returns_expected_item_index() {
    let mut style = NodeGraphStyle::default();
    style.context_menu_width = 200.0;
    style.context_menu_item_height = 10.0;
    style.context_menu_padding = 2.0;

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
    let pad = style.context_menu_padding / zoom;
    let item_h = style.context_menu_item_height / zoom;

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
    style.context_menu_width = 200.0;
    style.context_menu_item_height = 10.0;
    style.context_menu_padding = 2.0;

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
    let pad = style.context_menu_padding / zoom;
    let item_h = style.context_menu_item_height / zoom;
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
fn clamp_context_menu_origin_keeps_menu_rect_inside_visible_canvas_rect() {
    let mut host = TestUiHostImpl::default();
    let graph = host.models.insert(Graph::new(GraphId::new()));
    let view = host.models.insert(NodeGraphViewState::default());

    let _ = view.update(&mut host, |s, _cx| {
        s.pan = CanvasPoint { x: 100.0, y: -50.0 };
        s.zoom = 2.0;
    });

    let mut canvas = NodeGraphCanvas::new(graph, view);
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
    let graph = host.models.insert(Graph::new(GraphId::new()));
    let view = host.models.insert(NodeGraphViewState::default());

    let _ = view.update(&mut host, |s, _cx| {
        s.pan = CanvasPoint { x: -250.0, y: 75.0 };
        s.zoom = 0.75;
    });

    let mut canvas = NodeGraphCanvas::new(graph, view);
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
