use super::super::*;
use super::SvgRasterGpu;
use crate::svg::SMOOTH_SVG_SCALE_FACTOR;

impl Renderer {
    pub(in crate::renderer) fn svg_target_box_px(rect: Rect, scale_factor: f32) -> (u32, u32) {
        let w = (rect.size.width.0 * scale_factor).ceil().max(1.0);
        let h = (rect.size.height.0 * scale_factor).ceil().max(1.0);
        (w as u32, h as u32)
    }

    pub(in crate::renderer) fn svg_raster_key(
        svg: fret_core::SvgId,
        rect: Rect,
        scale_factor: f32,
        kind: SvgRasterKind,
        fit: fret_core::SvgFit,
    ) -> SvgRasterKey {
        let (target_w, target_h) = Self::svg_target_box_px(rect, scale_factor);
        SvgRasterKey {
            svg,
            target_w,
            target_h,
            smooth_scale_bits: SMOOTH_SVG_SCALE_FACTOR.to_bits(),
            kind,
            fit,
        }
    }

    pub(in crate::renderer) fn prepare_svg_ops(
        &mut self,
        device: &wgpu::Device,
        queue: &wgpu::Queue,
        scene: &Scene,
        scale_factor: f32,
    ) {
        #[cfg(debug_assertions)]
        if let Err(e) = scene.validate() {
            panic!("invalid scene: {e}");
        }

        let gpu = SvgRasterGpu { device, queue };
        let mut transform_stack: Vec<Transform2D> = vec![Transform2D::IDENTITY];

        let current_transform_scale = |t: Transform2D| -> f32 {
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
        };

        for op in scene.ops() {
            match op {
                SceneOp::PushTransform { transform } => {
                    let current = *transform_stack
                        .last()
                        .expect("transform stack must be non-empty");
                    transform_stack.push(current * *transform);
                }
                SceneOp::PopTransform => {
                    if transform_stack.len() > 1 {
                        transform_stack.pop();
                    }
                }
                SceneOp::PushOpacity { .. }
                | SceneOp::PopOpacity
                | SceneOp::PushLayer { .. }
                | SceneOp::PopLayer
                | SceneOp::PushClipRect { .. }
                | SceneOp::PushClipRRect { .. }
                | SceneOp::PopClip
                | SceneOp::PushEffect { .. }
                | SceneOp::PopEffect => {}
                SceneOp::SvgMaskIcon { rect, svg, fit, .. } => {
                    let s = current_transform_scale(
                        *transform_stack
                            .last()
                            .expect("transform stack must be non-empty"),
                    );
                    let rect = Rect::new(
                        rect.origin,
                        Size::new(Px(rect.size.width.0 * s), Px(rect.size.height.0 * s)),
                    );
                    let _ = self.ensure_svg_raster(
                        &gpu,
                        *svg,
                        rect,
                        scale_factor,
                        SvgRasterKind::AlphaMask,
                        *fit,
                    );
                }
                SceneOp::SvgImage { rect, svg, fit, .. } => {
                    let s = current_transform_scale(
                        *transform_stack
                            .last()
                            .expect("transform stack must be non-empty"),
                    );
                    let rect = Rect::new(
                        rect.origin,
                        Size::new(Px(rect.size.width.0 * s), Px(rect.size.height.0 * s)),
                    );
                    let _ = self.ensure_svg_raster(
                        &gpu,
                        *svg,
                        rect,
                        scale_factor,
                        SvgRasterKind::Rgba,
                        *fit,
                    );
                }
                SceneOp::Quad { .. }
                | SceneOp::Image { .. }
                | SceneOp::ImageRegion { .. }
                | SceneOp::MaskImage { .. }
                | SceneOp::PushMask { .. }
                | SceneOp::PopMask
                | SceneOp::Text { .. }
                | SceneOp::Path { .. }
                | SceneOp::ViewportSurface { .. } => {}
            }
        }

        self.prune_svg_rasters();
    }
}
