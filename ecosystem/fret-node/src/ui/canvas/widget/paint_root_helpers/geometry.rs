use super::super::*;

pub(super) fn append_static_scene_geometry_style_key(
    builder: &mut TileCacheKeyBuilder,
    style: &NodeGraphStyle,
    scale_factor: f32,
) {
    builder.add_u32(style.geometry.wire_width.to_bits());
    builder.add_f32_bits(style.geometry.node_padding);
    builder.add_f32_bits(style.geometry.node_header_height);
    builder.add_f32_bits(style.geometry.pin_row_height);
    builder.add_f32_bits(style.geometry.pin_radius);
    builder.add_u32(style.geometry.context_menu_text_style.size.0.to_bits());
    builder.add_u32(u32::from(style.geometry.context_menu_text_style.weight.0));
    builder.add_u32(scale_factor.to_bits());
}

#[cfg(test)]
mod tests;
