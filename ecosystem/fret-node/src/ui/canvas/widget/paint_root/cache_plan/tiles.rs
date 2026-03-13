use crate::ui::canvas::widget::*;

pub(super) struct StaticSceneCacheTileSizes {
    pub(super) nodes_cache_tile_size_canvas: f32,
    pub(super) edges_cache_tile_size_canvas: f32,
}

pub(super) fn can_use_static_scene_cache(
    snapshot: &ViewSnapshot,
    bounds: Rect,
    no_drag_preview: bool,
) -> bool {
    no_drag_preview
        && snapshot.interaction.only_render_visible_elements
        && snapshot.zoom.is_finite()
        && snapshot.zoom > 1.0e-6
        && bounds.size.width.0.is_finite()
        && bounds.size.height.0.is_finite()
}

pub(super) fn static_scene_cache_tile_sizes(bounds: Rect, zoom: f32) -> StaticSceneCacheTileSizes {
    let viewport_max_screen_px = bounds.size.width.0.max(bounds.size.height.0);
    let nodes_tile_size_screen_px =
        crate::ui::canvas::widget::static_scene_cache_plan::next_power_of_two_at_least(
            super::STATIC_SCENE_TILE_SIZE_SCREEN_PX_MIN,
            viewport_max_screen_px * super::STATIC_NODES_TILE_MUL,
        );

    StaticSceneCacheTileSizes {
        nodes_cache_tile_size_canvas: (nodes_tile_size_screen_px as f32 / zoom).max(1.0),
        edges_cache_tile_size_canvas: (super::STATIC_EDGES_TILE_SIZE_SCREEN_PX as f32 / zoom)
            .max(1.0),
    }
}

pub(super) fn static_cache_rect(
    can_use_static_scene_cache: bool,
    viewport_rect: Rect,
    viewport_w: f32,
    viewport_h: f32,
    tile_size_canvas: f32,
) -> Option<Rect> {
    if can_use_static_scene_cache
        && tile_size_canvas >= viewport_w
        && tile_size_canvas >= viewport_h
    {
        crate::ui::canvas::widget::static_scene_cache_plan::centered_single_tile_rect(
            viewport_rect,
            tile_size_canvas,
        )
    } else {
        None
    }
}

#[cfg(test)]
mod tests;
