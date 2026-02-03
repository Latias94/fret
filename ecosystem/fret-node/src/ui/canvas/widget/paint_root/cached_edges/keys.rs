use super::super::super::super::*;

pub(super) fn edges_tiles_base_key(
    base_key: DerivedBaseKey,
    style_key: u64,
    edges_cache_tile_size_canvas: f32,
) -> u64 {
    let mut b = TileCacheKeyBuilder::new("fret-node.canvas.static_edges.tile.v1");
    b.add_u64(base_key.graph_rev);
    b.add_u32(base_key.zoom_bits);
    b.add_u32(base_key.node_origin_x_bits);
    b.add_u32(base_key.node_origin_y_bits);
    b.add_u64(base_key.draw_order_hash);
    b.add_u64(base_key.presenter_rev);
    b.add_u64(base_key.edge_types_rev);
    b.add_u64(style_key);
    b.add_f32_bits(edges_cache_tile_size_canvas);
    b.finish()
}

pub(super) fn edge_labels_tiles_base_key(
    base_key: DerivedBaseKey,
    style_key: u64,
    edges_cache_tile_size_canvas: f32,
) -> u64 {
    let mut b = TileCacheKeyBuilder::new("fret-node.canvas.static_edge_labels.tile.v1");
    b.add_u64(base_key.graph_rev);
    b.add_u32(base_key.zoom_bits);
    b.add_u32(base_key.node_origin_x_bits);
    b.add_u32(base_key.node_origin_y_bits);
    b.add_u64(base_key.draw_order_hash);
    b.add_u64(base_key.presenter_rev);
    b.add_u64(base_key.edge_types_rev);
    b.add_u64(style_key);
    b.add_f32_bits(edges_cache_tile_size_canvas);
    b.finish()
}

pub(super) fn edges_single_rect_key(
    base_key: DerivedBaseKey,
    style_key: u64,
    edges_cache_tile_size_canvas: f32,
    edges_cache_rect: Rect,
) -> u64 {
    let mut b = TileCacheKeyBuilder::new("fret-node.canvas.static_edges.v1");
    b.add_u64(base_key.graph_rev);
    b.add_u32(base_key.zoom_bits);
    b.add_u32(base_key.node_origin_x_bits);
    b.add_u32(base_key.node_origin_y_bits);
    b.add_u64(base_key.draw_order_hash);
    b.add_u64(base_key.presenter_rev);
    b.add_u64(base_key.edge_types_rev);
    b.add_u64(style_key);
    b.add_f32_bits(edges_cache_tile_size_canvas);
    b.add_u32(edges_cache_rect.origin.x.0.to_bits());
    b.add_u32(edges_cache_rect.origin.y.0.to_bits());
    b.finish()
}

pub(super) fn edge_labels_single_rect_key(
    base_key: DerivedBaseKey,
    style_key: u64,
    edges_cache_tile_size_canvas: f32,
    edges_cache_rect: Rect,
) -> u64 {
    let mut b = TileCacheKeyBuilder::new("fret-node.canvas.static_edge_labels.v1");
    b.add_u64(base_key.graph_rev);
    b.add_u32(base_key.zoom_bits);
    b.add_u32(base_key.node_origin_x_bits);
    b.add_u32(base_key.node_origin_y_bits);
    b.add_u64(base_key.draw_order_hash);
    b.add_u64(base_key.presenter_rev);
    b.add_u64(base_key.edge_types_rev);
    b.add_u64(style_key);
    b.add_f32_bits(edges_cache_tile_size_canvas);
    b.add_u32(edges_cache_rect.origin.x.0.to_bits());
    b.add_u32(edges_cache_rect.origin.y.0.to_bits());
    b.finish()
}
