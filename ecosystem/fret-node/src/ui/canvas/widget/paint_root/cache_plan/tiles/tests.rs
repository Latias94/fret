use super::{can_use_static_scene_cache, static_cache_rect, static_scene_cache_tile_sizes};
use crate::core::CanvasPoint;
use crate::ui::canvas::state::ViewSnapshot;
use fret_core::{Point, Px, Rect, Size};

#[test]
fn static_scene_cache_tile_sizes_scale_with_zoom() {
    let bounds = Rect::new(
        Point::new(Px(0.0), Px(0.0)),
        Size::new(Px(800.0), Px(600.0)),
    );

    let zoom1 = static_scene_cache_tile_sizes(bounds, 1.0);
    let zoom2 = static_scene_cache_tile_sizes(bounds, 2.0);

    assert!(zoom2.nodes_cache_tile_size_canvas < zoom1.nodes_cache_tile_size_canvas);
    assert!(zoom2.edges_cache_tile_size_canvas < zoom1.edges_cache_tile_size_canvas);
}

#[test]
fn static_cache_rect_requires_static_cache_and_full_coverage() {
    let viewport_rect = Rect::new(
        Point::new(Px(10.0), Px(20.0)),
        Size::new(Px(100.0), Px(80.0)),
    );

    assert!(static_cache_rect(false, viewport_rect, 100.0, 80.0, 200.0).is_none());
    assert!(static_cache_rect(true, viewport_rect, 100.0, 80.0, 90.0).is_none());
    assert!(static_cache_rect(true, viewport_rect, 100.0, 80.0, 200.0).is_some());
}

#[test]
fn can_use_static_scene_cache_requires_visible_elements_zoom_and_finite_bounds() {
    let mut snapshot = ViewSnapshot {
        pan: CanvasPoint::default(),
        zoom: 1.0,
        selected_nodes: Vec::new(),
        selected_edges: Vec::new(),
        selected_groups: Vec::new(),
        draw_order: Vec::new(),
        group_draw_order: Vec::new(),
        interaction: crate::io::NodeGraphInteractionState::default(),
    };
    snapshot.interaction.only_render_visible_elements = true;
    let bounds = Rect::new(
        Point::new(Px(0.0), Px(0.0)),
        Size::new(Px(400.0), Px(300.0)),
    );

    assert!(can_use_static_scene_cache(&snapshot, bounds, true));
    assert!(!can_use_static_scene_cache(&snapshot, bounds, false));

    snapshot.zoom = 0.0;
    assert!(!can_use_static_scene_cache(&snapshot, bounds, true));
}
