use super::*;

const STATIC_SCENE_TILE_SIZE_SCREEN_PX_MIN: u32 = 1024;
const STATIC_NODES_TILE_MUL: f32 = 2.0;

pub(super) fn should_track_cull_window<M: NodeGraphCanvasMiddleware>(
    canvas: &mut NodeGraphCanvasWith<M>,
    snapshot: &ViewSnapshot,
) -> bool {
    if snapshot.interaction.only_render_visible_elements {
        true
    } else {
        canvas.last_cull_window_key = None;
        false
    }
}

pub(super) fn build_cull_window_key(bounds: Rect, snapshot: &ViewSnapshot) -> Option<u64> {
    let zoom = snapshot.zoom;
    if !zoom.is_finite() || zoom <= 1.0e-6 {
        return None;
    }

    let viewport_max_screen_px = bounds.size.width.0.max(bounds.size.height.0);
    if !viewport_max_screen_px.is_finite() || viewport_max_screen_px <= 0.0 {
        return None;
    }

    let nodes_tile_size_screen_px = next_power_of_two_at_least(
        STATIC_SCENE_TILE_SIZE_SCREEN_PX_MIN,
        viewport_max_screen_px * STATIC_NODES_TILE_MUL,
    );
    let nodes_cache_tile_size_canvas = (nodes_tile_size_screen_px as f32 / zoom).max(1.0);

    let viewport = CanvasViewport2D::new(
        bounds,
        PanZoom2D {
            pan: Point::new(Px(snapshot.pan.x), Px(snapshot.pan.y)),
            zoom,
        },
    );
    let viewport_rect = viewport.visible_canvas_rect();
    let center_x = viewport_rect.origin.x.0 + 0.5 * viewport_rect.size.width.0;
    let center_y = viewport_rect.origin.y.0 + 0.5 * viewport_rect.size.height.0;
    if !center_x.is_finite() || !center_y.is_finite() {
        return None;
    }

    let tile_x = (center_x / nodes_cache_tile_size_canvas).floor() as i32;
    let tile_y = (center_y / nodes_cache_tile_size_canvas).floor() as i32;

    let mut builder = TileCacheKeyBuilder::new("fret-node.canvas.cull_window.v1");
    builder
        .add_u32(nodes_tile_size_screen_px)
        .add_f32_bits(zoom)
        .add_i32(tile_x)
        .add_i32(tile_y);
    Some(builder.finish())
}

fn next_power_of_two_at_least(min: u32, value: f32) -> u32 {
    let target = value.ceil().max(1.0) as u32;
    let pow2 = target.checked_next_power_of_two().unwrap_or(0x8000_0000);
    pow2.max(min)
}
