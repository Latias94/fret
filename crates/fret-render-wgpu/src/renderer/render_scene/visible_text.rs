use super::super::types::ScissorRect;
use super::super::util::{intersect_scissor, rect_to_pixels, scissor_from_bounds_px};
use super::super::*;
use crate::text::TextSystem;
use fret_core::TextBlobId;

pub(super) fn visible_text_blob_ids_for_scene(
    scene: &Scene,
    text_system: &TextSystem,
    scale_factor: f32,
    viewport_size: (u32, u32),
) -> Vec<TextBlobId> {
    let mut state = VisibleTextState::new(scale_factor, viewport_size);
    let mut visible = Vec::new();
    for op in scene.ops() {
        state.handle_op(op, text_system, &mut visible);
    }
    visible
}

struct VisibleTextState {
    scale_factor: f32,
    viewport_size: (u32, u32),
    current_scissor: ScissorRect,
    scissor_stack: Vec<ScissorRect>,
    transform_stack: Vec<Transform2D>,
    opacity_stack: Vec<f32>,
}

impl VisibleTextState {
    fn new(scale_factor: f32, viewport_size: (u32, u32)) -> Self {
        let current_scissor = ScissorRect::full(viewport_size.0, viewport_size.1);
        Self {
            scale_factor,
            viewport_size,
            current_scissor,
            scissor_stack: vec![current_scissor],
            transform_stack: vec![Transform2D::IDENTITY],
            opacity_stack: vec![1.0],
        }
    }

    fn handle_op(&mut self, op: &SceneOp, text_system: &TextSystem, visible: &mut Vec<TextBlobId>) {
        match *op {
            SceneOp::PushTransform { transform } => {
                self.transform_stack
                    .push(self.current_transform() * transform);
            }
            SceneOp::PopTransform => {
                if self.transform_stack.len() > 1 {
                    self.transform_stack.pop();
                }
            }
            SceneOp::PushOpacity { opacity } => {
                self.opacity_stack
                    .push((self.current_opacity() * opacity).clamp(0.0, 1.0));
            }
            SceneOp::PopOpacity => {
                if self.opacity_stack.len() > 1 {
                    self.opacity_stack.pop();
                }
            }
            SceneOp::PushClipRect { rect } | SceneOp::PushClipRRect { rect, .. } => {
                self.push_bounds_scissor(rect);
            }
            SceneOp::PushClipPath { bounds, .. } => {
                self.push_bounds_scissor(bounds);
            }
            SceneOp::PopClip => {
                self.pop_scissor();
            }
            SceneOp::PushCompositeGroup { desc } => {
                self.push_bounds_scissor(desc.bounds);
            }
            SceneOp::PopCompositeGroup => {
                self.pop_scissor();
            }
            SceneOp::Text {
                origin,
                text,
                shadow,
                ..
            } => {
                if self.current_opacity() <= 0.0 {
                    return;
                }

                if self.text_blob_intersects_scissor(text_system, text, origin)
                    || self.shadow_intersects_scissor(text_system, text, origin, shadow)
                {
                    visible.push(text);
                }
            }
            SceneOp::PushLayer { .. }
            | SceneOp::PopLayer
            | SceneOp::PushMask { .. }
            | SceneOp::PopMask
            | SceneOp::PushEffect { .. }
            | SceneOp::PopEffect
            | SceneOp::PushBackdropSourceGroupV1 { .. }
            | SceneOp::PopBackdropSourceGroup
            | SceneOp::Quad { .. }
            | SceneOp::StrokeRRect { .. }
            | SceneOp::ShadowRRect { .. }
            | SceneOp::Image { .. }
            | SceneOp::ImageRegion { .. }
            | SceneOp::VertexColorQuad { .. }
            | SceneOp::ImageQuad { .. }
            | SceneOp::VertexColorTriangle { .. }
            | SceneOp::ImageTriangle { .. }
            | SceneOp::MaskImage { .. }
            | SceneOp::SvgMaskIcon { .. }
            | SceneOp::SvgImage { .. }
            | SceneOp::Path { .. }
            | SceneOp::ViewportSurface { .. } => {}
        }
    }

    fn current_opacity(&self) -> f32 {
        *self
            .opacity_stack
            .last()
            .expect("opacity stack must be non-empty")
    }

    fn current_transform(&self) -> Transform2D {
        *self
            .transform_stack
            .last()
            .expect("transform stack must be non-empty")
    }

    fn current_transform_px(&self) -> Transform2D {
        self.current_transform().to_physical_px(self.scale_factor)
    }

    fn push_bounds_scissor(&mut self, bounds: Rect) {
        let bounds_scissor = self.bounds_scissor(bounds);
        self.current_scissor = intersect_scissor(self.current_scissor, bounds_scissor);
        self.scissor_stack.push(self.current_scissor);
    }

    fn pop_scissor(&mut self) {
        if self.scissor_stack.len() > 1 {
            self.scissor_stack.pop();
            self.current_scissor = *self
                .scissor_stack
                .last()
                .expect("scissor stack must be non-empty");
        }
    }

    fn bounds_scissor(&self, bounds: Rect) -> ScissorRect {
        let (x, y, w, h) = rect_to_pixels(bounds, self.scale_factor);
        if w <= 0.0 || h <= 0.0 {
            return ScissorRect {
                x: 0,
                y: 0,
                w: 0,
                h: 0,
            };
        }
        let quad = transform_quad_points_px(self.current_transform_px(), x, y, w, h);
        let (min_x, min_y, max_x, max_y) = bounds_of_quad_points(&quad);
        scissor_from_bounds_px(min_x, min_y, max_x, max_y, self.viewport_size).unwrap_or(
            ScissorRect {
                x: 0,
                y: 0,
                w: 0,
                h: 0,
            },
        )
    }

    fn text_blob_intersects_scissor(
        &self,
        text_system: &TextSystem,
        text: TextBlobId,
        origin: Point,
    ) -> bool {
        let Some((min_x, min_y, max_x, max_y)) =
            self.text_blob_bounds_px(text_system, text, origin)
        else {
            return false;
        };
        let quad = transform_quad_points_px(
            self.current_transform_px(),
            min_x,
            min_y,
            max_x - min_x,
            max_y - min_y,
        );
        let (min_x, min_y, max_x, max_y) = bounds_of_quad_points(&quad);
        let Some(bounds_scissor) =
            scissor_from_bounds_px(min_x, min_y, max_x, max_y, self.viewport_size)
        else {
            return false;
        };
        let clipped = intersect_scissor(self.current_scissor, bounds_scissor);
        clipped.w > 0 && clipped.h > 0
    }

    fn shadow_intersects_scissor(
        &self,
        text_system: &TextSystem,
        text: TextBlobId,
        origin: Point,
        shadow: Option<fret_core::scene::TextShadowV1>,
    ) -> bool {
        let Some(shadow) = shadow else {
            return false;
        };
        if shadow.color.a <= 0.0 || (shadow.offset.x.0 == 0.0 && shadow.offset.y.0 == 0.0) {
            return false;
        }
        self.text_blob_intersects_scissor(
            text_system,
            text,
            Point::new(origin.x + shadow.offset.x, origin.y + shadow.offset.y),
        )
    }

    fn text_blob_bounds_px(
        &self,
        text_system: &TextSystem,
        text: TextBlobId,
        origin: Point,
    ) -> Option<(f32, f32, f32, f32)> {
        let (min_x, min_y, max_x, max_y) = text_system.glyph_bounds_for_blob(text)?;
        Some((
            (origin.x.0 + min_x) * self.scale_factor,
            (origin.y.0 + min_y) * self.scale_factor,
            (origin.x.0 + max_x) * self.scale_factor,
            (origin.y.0 + max_y) * self.scale_factor,
        ))
    }
}

fn transform_quad_points_px(t_px: Transform2D, x: f32, y: f32, w: f32, h: f32) -> [(f32, f32); 4] {
    let x1 = x + w;
    let y1 = y + h;
    [
        apply_transform_px(t_px, x, y),
        apply_transform_px(t_px, x1, y),
        apply_transform_px(t_px, x1, y1),
        apply_transform_px(t_px, x, y1),
    ]
}

fn apply_transform_px(t_px: Transform2D, x: f32, y: f32) -> (f32, f32) {
    let p = t_px.apply_point(Point::new(Px(x), Px(y)));
    (p.x.0, p.y.0)
}

fn bounds_of_quad_points(pts: &[(f32, f32); 4]) -> (f32, f32, f32, f32) {
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

#[cfg(test)]
mod tests {
    use super::*;
    use crate::WgpuContext;
    use fret_core::{Color, DrawOrder, Paint, TextConstraints, TextStyle, geometry::Size};

    fn white_text_op(origin: Point, text: TextBlobId) -> SceneOp {
        SceneOp::Text {
            order: DrawOrder(0),
            origin,
            text,
            paint: Paint::Solid(Color {
                r: 1.0,
                g: 1.0,
                b: 1.0,
                a: 1.0,
            })
            .into(),
            outline: None,
            shadow: None,
        }
    }

    #[test]
    fn visible_text_blob_prepass_excludes_offscreen_residency() {
        let ctx = pollster::block_on(WgpuContext::new()).expect("wgpu context");
        let mut text = TextSystem::new(&ctx.device);
        let style = TextStyle {
            size: Px(16.0),
            ..Default::default()
        };
        let constraints = TextConstraints {
            max_width: Some(Px(240.0)),
            ..Default::default()
        };
        let (visible_blob, _) = text.prepare("aaaa", &style, constraints);
        let (offscreen_blob, _) = text.prepare("zzzz", &style, constraints);

        let mut scene = Scene::default();
        scene.push(white_text_op(Point::new(Px(8.0), Px(24.0)), visible_blob));
        scene.push(white_text_op(
            Point::new(Px(8.0), Px(1024.0)),
            offscreen_blob,
        ));

        let visible = visible_text_blob_ids_for_scene(&scene, &text, 1.0, (240, 80));
        assert_eq!(visible, vec![visible_blob]);

        let perf = text.prepare_for_text_blobs_with_perf(&visible, 0, true);
        assert_eq!(perf.scene_text_blobs, 1);
        assert!(perf.added_glyph_keys > 0);

        let visible_snapshot = text.text_resource_snapshot_for_blobs(&[visible_blob]);
        assert!(visible_snapshot.glyphs > 0);
        assert_eq!(visible_snapshot.missing_glyph_resources, 0);

        let offscreen_snapshot = text.text_resource_snapshot_for_blobs(&[offscreen_blob]);
        assert!(offscreen_snapshot.glyphs > 0);
        assert_eq!(
            offscreen_snapshot.missing_glyph_resources, offscreen_snapshot.glyphs,
            "offscreen text must not be pulled into atlas residency by frame prepare"
        );
    }

    #[test]
    fn visible_text_blob_prepass_respects_clip_and_opacity() {
        let ctx = pollster::block_on(WgpuContext::new()).expect("wgpu context");
        let text_system = TextSystem::new(&ctx.device);
        let style = TextStyle {
            size: Px(16.0),
            ..Default::default()
        };
        let constraints = TextConstraints {
            max_width: Some(Px(240.0)),
            ..Default::default()
        };
        let mut text_system = text_system;
        let (visible_blob, _) = text_system.prepare("clip visible", &style, constraints);
        let (clipped_blob, _) = text_system.prepare("clip hidden", &style, constraints);
        let (transparent_blob, _) = text_system.prepare("transparent", &style, constraints);

        let mut scene = Scene::default();
        scene.push(SceneOp::PushClipRect {
            rect: Rect::new(Point::new(Px(0.0), Px(0.0)), Size::new(Px(240.0), Px(80.0))),
        });
        scene.push(white_text_op(Point::new(Px(8.0), Px(24.0)), visible_blob));
        scene.push(white_text_op(Point::new(Px(8.0), Px(240.0)), clipped_blob));
        scene.push(SceneOp::PopClip);
        scene.push(SceneOp::PushOpacity { opacity: 0.0 });
        scene.push(white_text_op(
            Point::new(Px(8.0), Px(24.0)),
            transparent_blob,
        ));
        scene.push(SceneOp::PopOpacity);

        let visible = visible_text_blob_ids_for_scene(&scene, &text_system, 1.0, (240, 80));
        assert_eq!(visible, vec![visible_blob]);
    }

    #[test]
    fn visible_text_blob_prepass_does_not_treat_effect_bounds_as_clip() {
        let ctx = pollster::block_on(WgpuContext::new()).expect("wgpu context");
        let mut text_system = TextSystem::new(&ctx.device);
        let style = TextStyle {
            size: Px(16.0),
            ..Default::default()
        };
        let (blob, _) = text_system.prepare("effect visible", &style, TextConstraints::default());

        let mut scene = Scene::default();
        scene.push(SceneOp::PushEffect {
            bounds: Rect::new(
                Point::new(Px(500.0), Px(500.0)),
                Size::new(Px(1.0), Px(1.0)),
            ),
            mode: fret_core::EffectMode::FilterContent,
            chain: fret_core::EffectChain::EMPTY,
            quality: fret_core::EffectQuality::Auto,
        });
        scene.push(white_text_op(Point::new(Px(8.0), Px(24.0)), blob));
        scene.push(SceneOp::PopEffect);

        let visible = visible_text_blob_ids_for_scene(&scene, &text_system, 1.0, (240, 80));
        assert_eq!(
            visible,
            vec![blob],
            "effect bounds are computation bounds, not text draw scissor"
        );
    }

    #[test]
    fn visible_text_blob_prepass_matches_composite_group_work_scissor() {
        let ctx = pollster::block_on(WgpuContext::new()).expect("wgpu context");
        let mut text_system = TextSystem::new(&ctx.device);
        let style = TextStyle {
            size: Px(16.0),
            ..Default::default()
        };
        let (blob, _) =
            text_system.prepare("composite clipped", &style, TextConstraints::default());

        let mut scene = Scene::default();
        scene.push(SceneOp::PushCompositeGroup {
            desc: fret_core::CompositeGroupDesc::new(
                Rect::new(
                    Point::new(Px(500.0), Px(500.0)),
                    Size::new(Px(1.0), Px(1.0)),
                ),
                fret_core::BlendMode::Over,
                fret_core::EffectQuality::Auto,
            ),
        });
        scene.push(white_text_op(Point::new(Px(8.0), Px(24.0)), blob));
        scene.push(SceneOp::PopCompositeGroup);

        let visible = visible_text_blob_ids_for_scene(&scene, &text_system, 1.0, (240, 80));
        assert!(
            visible.is_empty(),
            "wgpu encode uses composite group bounds as a work scissor"
        );
    }
}
