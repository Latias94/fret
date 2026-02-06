use super::*;

impl<M: NodeGraphCanvasMiddleware> NodeGraphCanvasWith<M> {
    pub(super) fn static_scene_style_key(&self, scale_factor: f32) -> u64 {
        let mut b = TileCacheKeyBuilder::new("fret-node.canvas.static_scene_style.v1");
        b.add_u32(self.style.group_background.r.to_bits());
        b.add_u32(self.style.group_background.g.to_bits());
        b.add_u32(self.style.group_background.b.to_bits());
        b.add_u32(self.style.group_background.a.to_bits());
        b.add_u32(self.style.group_border.r.to_bits());
        b.add_u32(self.style.group_border.g.to_bits());
        b.add_u32(self.style.group_border.b.to_bits());
        b.add_u32(self.style.group_border.a.to_bits());
        b.add_u32(self.style.node_background.r.to_bits());
        b.add_u32(self.style.node_background.g.to_bits());
        b.add_u32(self.style.node_background.b.to_bits());
        b.add_u32(self.style.node_background.a.to_bits());
        b.add_u32(self.style.node_border.r.to_bits());
        b.add_u32(self.style.node_border.g.to_bits());
        b.add_u32(self.style.node_border.b.to_bits());
        b.add_u32(self.style.node_border.a.to_bits());
        b.add_u32(self.style.wire_color_data.r.to_bits());
        b.add_u32(self.style.wire_color_data.g.to_bits());
        b.add_u32(self.style.wire_color_data.b.to_bits());
        b.add_u32(self.style.wire_color_data.a.to_bits());
        b.add_u32(self.style.wire_color_exec.r.to_bits());
        b.add_u32(self.style.wire_color_exec.g.to_bits());
        b.add_u32(self.style.wire_color_exec.b.to_bits());
        b.add_u32(self.style.wire_color_exec.a.to_bits());
        b.add_u32(self.style.wire_width.to_bits());
        b.add_u32(self.style.wire_width_selected_mul.to_bits());
        b.add_u32(self.style.wire_width_hover_mul.to_bits());
        b.add_u32(self.style.context_menu_background.r.to_bits());
        b.add_u32(self.style.context_menu_background.g.to_bits());
        b.add_u32(self.style.context_menu_background.b.to_bits());
        b.add_u32(self.style.context_menu_background.a.to_bits());
        b.add_u32(self.style.context_menu_border.r.to_bits());
        b.add_u32(self.style.context_menu_border.g.to_bits());
        b.add_u32(self.style.context_menu_border.b.to_bits());
        b.add_u32(self.style.context_menu_border.a.to_bits());
        b.add_u32(self.style.context_menu_text.r.to_bits());
        b.add_u32(self.style.context_menu_text.g.to_bits());
        b.add_u32(self.style.context_menu_text.b.to_bits());
        b.add_u32(self.style.context_menu_text.a.to_bits());
        b.add_f32_bits(self.style.node_padding);
        b.add_f32_bits(self.style.node_header_height);
        b.add_f32_bits(self.style.pin_row_height);
        b.add_f32_bits(self.style.pin_radius);
        b.add_u32(self.style.context_menu_text_style.size.0.to_bits());
        b.add_u32(u32::from(self.style.context_menu_text_style.weight.0));
        b.add_u32(scale_factor.to_bits());
        b.finish()
    }
}
