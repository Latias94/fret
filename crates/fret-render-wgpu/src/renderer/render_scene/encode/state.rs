use super::*;

#[derive(Clone, Copy)]
pub(super) enum ClipPop {
    NoShader,
    Shader { prev_head: u32 },
}

#[derive(Clone, Copy)]
pub(super) enum MaskPop {
    NoShader,
    Shader { prev_head: u32 },
}

pub(super) struct EncodeState<'a> {
    pub(super) scale_factor: f32,
    pub(super) viewport_size: (u32, u32),
    pub(super) output_is_srgb: u32,
    pub(super) text_gamma_ratios: [f32; 4],
    pub(super) text_grayscale_enhanced_contrast: f32,
    pub(super) text_subpixel_enhanced_contrast: f32,

    pub(super) instances: &'a mut Vec<QuadInstance>,
    pub(super) viewport_vertices: &'a mut Vec<ViewportVertex>,
    pub(super) text_vertices: &'a mut Vec<TextVertex>,
    pub(super) path_vertices: &'a mut Vec<PathVertex>,
    pub(super) clips: &'a mut Vec<ClipRRectUniform>,
    pub(super) masks: &'a mut Vec<MaskGradientUniform>,
    pub(super) uniforms: &'a mut Vec<ViewportUniform>,
    pub(super) ordered_draws: &'a mut Vec<OrderedDraw>,
    pub(super) effect_markers: &'a mut Vec<EffectMarker>,

    pub(super) scissor_stack: Vec<ScissorRect>,
    pub(super) current_scissor: ScissorRect,

    pub(super) clip_pop_stack: Vec<ClipPop>,
    pub(super) clip_head: u32,
    pub(super) clip_count: u32,

    pub(super) mask_pop_stack: Vec<MaskPop>,
    pub(super) mask_head: u32,
    pub(super) mask_count: u32,

    pub(super) mask_scope_stack: Vec<(u32, u32)>,
    pub(super) mask_scope_head: u32,
    pub(super) mask_scope_count: u32,

    pub(super) current_uniform_index: u32,

    pub(super) quad_batch: Option<(ScissorRect, u32, u32)>,

    pub(super) transform_stack: Vec<Transform2D>,
    pub(super) opacity_stack: Vec<f32>,

    pub(super) material_paint_budget_per_frame: u64,
    pub(super) material_distinct_budget_per_frame: usize,
    pub(super) material_paints_used: u64,
    pub(super) material_seen: Vec<fret_core::MaterialId>,
    pub(super) material_quad_ops: &'a mut u64,
    pub(super) material_distinct: &'a mut u64,
    pub(super) material_unknown_ids: &'a mut u64,
    pub(super) material_degraded_due_to_budget: &'a mut u64,
}

impl<'a> EncodeState<'a> {
    const MAX_CLIP_STACK_DEPTH: u32 = 64;
    const MAX_MASK_STACK_DEPTH: u32 = 64;

    pub(super) fn new(
        encoding: &'a mut SceneEncoding,
        scale_factor: f32,
        viewport_size: (u32, u32),
        output_is_srgb: bool,
        text_gamma_ratios: [f32; 4],
        text_grayscale_enhanced_contrast: f32,
        text_subpixel_enhanced_contrast: f32,
        material_paint_budget_per_frame: u64,
        material_distinct_budget_per_frame: usize,
    ) -> Self {
        let instances = &mut encoding.instances;
        let viewport_vertices = &mut encoding.viewport_vertices;
        let text_vertices = &mut encoding.text_vertices;
        let path_vertices = &mut encoding.path_vertices;
        let clips = &mut encoding.clips;
        let masks = &mut encoding.masks;
        let uniforms = &mut encoding.uniforms;
        let ordered_draws = &mut encoding.ordered_draws;
        let effect_markers = &mut encoding.effect_markers;

        let material_quad_ops = &mut encoding.material_quad_ops;
        let material_distinct = &mut encoding.material_distinct;
        let material_unknown_ids = &mut encoding.material_unknown_ids;
        let material_degraded_due_to_budget = &mut encoding.material_degraded_due_to_budget;

        let current_scissor = ScissorRect::full(viewport_size.0, viewport_size.1);
        let mask_scope_head = 0;
        let mask_scope_count = 0;
        let mut state = Self {
            scale_factor,
            viewport_size,
            output_is_srgb: u32::from(output_is_srgb),
            text_gamma_ratios,
            text_grayscale_enhanced_contrast,
            text_subpixel_enhanced_contrast,
            instances,
            viewport_vertices,
            text_vertices,
            path_vertices,
            clips,
            masks,
            uniforms,
            ordered_draws,
            effect_markers,
            scissor_stack: vec![current_scissor],
            current_scissor,
            clip_pop_stack: Vec::new(),
            clip_head: 0,
            clip_count: 0,
            mask_pop_stack: Vec::new(),
            mask_head: 0,
            mask_count: 0,
            mask_scope_stack: vec![(mask_scope_head, mask_scope_count)],
            mask_scope_head,
            mask_scope_count,
            current_uniform_index: 0,
            quad_batch: None,
            transform_stack: vec![Transform2D::IDENTITY],
            opacity_stack: vec![1.0],

            material_paint_budget_per_frame,
            material_distinct_budget_per_frame,
            material_paints_used: 0,
            material_seen: Vec::new(),
            material_quad_ops,
            material_distinct,
            material_unknown_ids,
            material_degraded_due_to_budget,
        };

        state.current_uniform_index = state.push_uniform_snapshot(0, 0, 0, 0, 0, 0);
        state
    }

    pub(super) fn flush_quad_batch(&mut self) {
        if let Some((scissor, uniform_index, first_instance)) = self.quad_batch.take() {
            let instance_count = (self.instances.len() as u32).saturating_sub(first_instance);
            if instance_count > 0 {
                self.ordered_draws.push(OrderedDraw::Quad(DrawCall {
                    scissor,
                    uniform_index,
                    first_instance,
                    instance_count,
                }));
            }
        }
    }

    pub(super) fn push_text_draw(&mut self, draw: TextDraw) {
        if draw.vertex_count == 0 {
            return;
        }

        let Some(prev) = self.ordered_draws.last_mut() else {
            self.ordered_draws.push(OrderedDraw::Text(draw));
            return;
        };

        let OrderedDraw::Text(prev) = prev else {
            self.ordered_draws.push(OrderedDraw::Text(draw));
            return;
        };

        let prev_end = prev.first_vertex.saturating_add(prev.vertex_count);
        let can_merge = prev.scissor == draw.scissor
            && prev.uniform_index == draw.uniform_index
            && prev.kind == draw.kind
            && prev.atlas_page == draw.atlas_page
            && prev_end == draw.first_vertex;

        if can_merge {
            prev.vertex_count = prev.vertex_count.saturating_add(draw.vertex_count);
        } else {
            self.ordered_draws.push(OrderedDraw::Text(draw));
        }
    }

    pub(super) fn push_uniform_snapshot(
        &mut self,
        clip_head: u32,
        clip_count: u32,
        mask_head: u32,
        mask_count: u32,
        mask_scope_head: u32,
        mask_scope_count: u32,
    ) -> u32 {
        let uniform_index = self.uniforms.len() as u32;
        self.uniforms.push(ViewportUniform {
            viewport_size: [self.viewport_size.0 as f32, self.viewport_size.1 as f32],
            clip_head,
            clip_count: clip_count.min(Self::MAX_CLIP_STACK_DEPTH),
            mask_head,
            mask_count: mask_count.min(Self::MAX_MASK_STACK_DEPTH),
            mask_scope_head,
            mask_scope_count: mask_scope_count.min(Self::MAX_MASK_STACK_DEPTH),
            output_is_srgb: self.output_is_srgb,
            _pad: 0,
            mask_viewport_origin: [0.0, 0.0],
            mask_viewport_size: [self.viewport_size.0 as f32, self.viewport_size.1 as f32],
            _pad_text_gamma: [0; 2],
            text_gamma_ratios: self.text_gamma_ratios,
            text_grayscale_enhanced_contrast: self.text_grayscale_enhanced_contrast,
            text_subpixel_enhanced_contrast: self.text_subpixel_enhanced_contrast,
            _pad_text_quality: [0; 2],
        });
        uniform_index
    }

    pub(super) fn push_effect_uniform_snapshot(
        &mut self,
        mask_viewport: ScissorRect,
        clip_head: u32,
        clip_count: u32,
        mask_head: u32,
        mask_count: u32,
    ) -> u32 {
        let uniform_index = self.uniforms.len() as u32;
        let w = mask_viewport.w.max(1) as f32;
        let h = mask_viewport.h.max(1) as f32;
        self.uniforms.push(ViewportUniform {
            viewport_size: [self.viewport_size.0 as f32, self.viewport_size.1 as f32],
            clip_head,
            clip_count: clip_count.min(Self::MAX_CLIP_STACK_DEPTH),
            mask_head,
            mask_count: mask_count.min(Self::MAX_MASK_STACK_DEPTH),
            mask_scope_head: 0,
            mask_scope_count: 0,
            output_is_srgb: self.output_is_srgb,
            _pad: 0,
            mask_viewport_origin: [mask_viewport.x as f32, mask_viewport.y as f32],
            mask_viewport_size: [w, h],
            _pad_text_gamma: [0; 2],
            text_gamma_ratios: self.text_gamma_ratios,
            text_grayscale_enhanced_contrast: self.text_grayscale_enhanced_contrast,
            text_subpixel_enhanced_contrast: self.text_subpixel_enhanced_contrast,
            _pad_text_quality: [0; 2],
        });
        uniform_index
    }

    pub(super) fn current_opacity(&self) -> f32 {
        *self
            .opacity_stack
            .last()
            .expect("opacity stack must be non-empty")
    }

    pub(super) fn current_transform(&self) -> Transform2D {
        *self
            .transform_stack
            .last()
            .expect("transform stack must be non-empty")
    }

    pub(super) fn to_physical_px(&self, t: Transform2D) -> Transform2D {
        t.to_physical_px(self.scale_factor)
    }

    pub(super) fn current_transform_px(&self) -> Transform2D {
        self.to_physical_px(self.current_transform())
    }

    pub(super) fn current_transform_max_scale(t: Transform2D) -> f32 {
        if let Some((s, _)) = t.as_translation_uniform_scale()
            && s.is_finite()
            && s > 0.0
        {
            return s;
        }

        let sx = (t.a * t.a + t.b * t.b).sqrt();
        let sy = (t.c * t.c + t.d * t.d).sqrt();
        let s = sx.max(sy);
        if s.is_finite() && s > 0.0 { s } else { 1.0 }
    }

    pub(super) fn color_with_opacity(mut c: Color, opacity: f32) -> Color {
        c.a = (c.a * opacity).clamp(0.0, 1.0);
        c
    }
}

pub(super) fn transform_rows(t_px: Transform2D) -> ([f32; 4], [f32; 4]) {
    (
        [t_px.a, t_px.c, t_px.tx, 0.0],
        [t_px.b, t_px.d, t_px.ty, 0.0],
    )
}

pub(super) fn apply_transform_px(t_px: Transform2D, x: f32, y: f32) -> (f32, f32) {
    let p = t_px.apply_point(Point::new(Px(x), Px(y)));
    (p.x.0, p.y.0)
}

pub(super) fn transform_quad_points_px(
    t_px: Transform2D,
    x: f32,
    y: f32,
    w: f32,
    h: f32,
) -> [(f32, f32); 4] {
    let (x0, y0) = (x, y);
    let (x1, y1) = (x + w, y + h);
    [
        apply_transform_px(t_px, x0, y0),    // TL
        apply_transform_px(t_px, x + w, y0), // TR
        apply_transform_px(t_px, x1, y1),    // BR
        apply_transform_px(t_px, x0, y1),    // BL
    ]
}

pub(super) fn bounds_of_quad_points(pts: &[(f32, f32); 4]) -> (f32, f32, f32, f32) {
    let mut min_x = pts[0].0;
    let mut max_x = pts[0].0;
    let mut min_y = pts[0].1;
    let mut max_y = pts[0].1;
    for (x, y) in pts.iter().copied() {
        min_x = min_x.min(x);
        max_x = max_x.max(x);
        min_y = min_y.min(y);
        max_y = max_y.max(y);
    }
    (min_x, min_y, max_x, max_y)
}
