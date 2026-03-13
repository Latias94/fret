use super::super::*;

pub(super) fn append_static_scene_paint_style_key(
    builder: &mut TileCacheKeyBuilder,
    style: &NodeGraphStyle,
) {
    builder.add_u32(style.paint.group_background.r.to_bits());
    builder.add_u32(style.paint.group_background.g.to_bits());
    builder.add_u32(style.paint.group_background.b.to_bits());
    builder.add_u32(style.paint.group_background.a.to_bits());
    builder.add_u32(style.paint.group_border.r.to_bits());
    builder.add_u32(style.paint.group_border.g.to_bits());
    builder.add_u32(style.paint.group_border.b.to_bits());
    builder.add_u32(style.paint.group_border.a.to_bits());
    builder.add_u32(style.paint.node_background.r.to_bits());
    builder.add_u32(style.paint.node_background.g.to_bits());
    builder.add_u32(style.paint.node_background.b.to_bits());
    builder.add_u32(style.paint.node_background.a.to_bits());
    builder.add_u32(style.paint.node_border.r.to_bits());
    builder.add_u32(style.paint.node_border.g.to_bits());
    builder.add_u32(style.paint.node_border.b.to_bits());
    builder.add_u32(style.paint.node_border.a.to_bits());
    builder.add_u32(style.paint.wire_color_data.r.to_bits());
    builder.add_u32(style.paint.wire_color_data.g.to_bits());
    builder.add_u32(style.paint.wire_color_data.b.to_bits());
    builder.add_u32(style.paint.wire_color_data.a.to_bits());
    builder.add_u32(style.paint.wire_color_exec.r.to_bits());
    builder.add_u32(style.paint.wire_color_exec.g.to_bits());
    builder.add_u32(style.paint.wire_color_exec.b.to_bits());
    builder.add_u32(style.paint.wire_color_exec.a.to_bits());
    builder.add_u32(style.paint.wire_width_selected_mul.to_bits());
    builder.add_u32(style.paint.wire_width_hover_mul.to_bits());
    builder.add_u32(style.paint.context_menu_background.r.to_bits());
    builder.add_u32(style.paint.context_menu_background.g.to_bits());
    builder.add_u32(style.paint.context_menu_background.b.to_bits());
    builder.add_u32(style.paint.context_menu_background.a.to_bits());
    builder.add_u32(style.paint.context_menu_border.r.to_bits());
    builder.add_u32(style.paint.context_menu_border.g.to_bits());
    builder.add_u32(style.paint.context_menu_border.b.to_bits());
    builder.add_u32(style.paint.context_menu_border.a.to_bits());
    builder.add_u32(style.paint.context_menu_text.r.to_bits());
    builder.add_u32(style.paint.context_menu_text.g.to_bits());
    builder.add_u32(style.paint.context_menu_text.b.to_bits());
    builder.add_u32(style.paint.context_menu_text.a.to_bits());
}

#[cfg(test)]
mod tests;
