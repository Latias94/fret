use super::super::types::ScissorRect;
use super::super::util::{intersect_scissor, rect_to_pixels, scissor_from_bounds_px};
use super::super::*;
use crate::text::{TextFrameResidency, TextSystem};

pub(super) fn visible_text_residency_for_scene(
    scene: &Scene,
    text_system: &TextSystem,
    scale_factor: f32,
    viewport_size: (u32, u32),
) -> TextFrameResidency {
    let mut state = VisibleTextState::new(scale_factor, viewport_size);
    let mut residency = TextFrameResidency::new();
    for op in scene.ops() {
        state.handle_op(op, text_system, &mut residency);
    }
    residency
}

pub(in crate::renderer) fn visible_text_residency_for_chunk_entry(
    entry: &fret_core::SceneChunkManifestEntry,
    text_system: &TextSystem,
    scale_factor: f32,
    viewport_size: (u32, u32),
) -> TextFrameResidency {
    let mut state = VisibleTextState::new_with_initial_transform(
        scale_factor,
        viewport_size,
        Transform2D::translation(entry.scene_origin()),
    );
    let mut residency = TextFrameResidency::new();
    for op in entry.chunk().ops() {
        state.handle_op(op, text_system, &mut residency);
    }
    residency
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
        Self::new_with_initial_transform(scale_factor, viewport_size, Transform2D::IDENTITY)
    }

    fn new_with_initial_transform(
        scale_factor: f32,
        viewport_size: (u32, u32),
        initial_transform: Transform2D,
    ) -> Self {
        let current_scissor = ScissorRect::full(viewport_size.0, viewport_size.1);
        Self {
            scale_factor,
            viewport_size,
            current_scissor,
            scissor_stack: vec![current_scissor],
            transform_stack: vec![initial_transform],
            opacity_stack: vec![1.0],
        }
    }

    fn handle_op(
        &mut self,
        op: &SceneOp,
        text_system: &TextSystem,
        residency: &mut TextFrameResidency,
    ) {
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

                text_system.push_cluster_residency_for_blob(residency, text, |cluster| {
                    let bounds = cluster.visual_bounds();
                    self.glyph_rect_intersects_scissor(origin, bounds)
                        || self.shadow_glyph_rect_intersects_scissor(origin, bounds, shadow)
                });
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

    fn glyph_rect_intersects_scissor(&self, origin: Point, rect: [f32; 4]) -> bool {
        let [x, y, w, h] = rect;
        if !x.is_finite() || !y.is_finite() || !w.is_finite() || !h.is_finite() {
            return false;
        }
        let x0 = (origin.x.0 + x) * self.scale_factor;
        let y0 = (origin.y.0 + y) * self.scale_factor;
        let x1 = x0 + w * self.scale_factor;
        let y1 = y0 + h * self.scale_factor;
        let min_x = x0.min(x1);
        let min_y = y0.min(y1);
        let max_x = x0.max(x1);
        let max_y = y0.max(y1);
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

    fn shadow_glyph_rect_intersects_scissor(
        &self,
        origin: Point,
        rect: [f32; 4],
        shadow: Option<fret_core::scene::TextShadowV1>,
    ) -> bool {
        let Some(shadow) = shadow else {
            return false;
        };
        if shadow.color.a <= 0.0 || (shadow.offset.x.0 == 0.0 && shadow.offset.y.0 == 0.0) {
            return false;
        }
        self.glyph_rect_intersects_scissor(
            Point::new(origin.x + shadow.offset.x, origin.y + shadow.offset.y),
            rect,
        )
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
    use fret_core::{
        Color, DrawOrder, Paint, TextBlobId, TextConstraints, TextStyle, geometry::Size,
    };

    const INTER_ROMAN_FULL: &[u8] = include_bytes!(concat!(
        env!("CARGO_MANIFEST_DIR"),
        "/../../assets/font-archive/fret-fonts-bootstrap-full/Inter-roman.ttf"
    ));

    fn inter_fixture_family(text: &mut TextSystem) -> String {
        let added = text.add_fonts([INTER_ROMAN_FULL.to_vec()]);
        assert!(added > 0, "expected Inter fixture font to load");
        text.all_font_names()
            .into_iter()
            .find(|name| {
                let lower = name.to_ascii_lowercase();
                lower == "inter" || lower.contains("inter ")
            })
            .unwrap_or_else(|| {
                panic!(
                    "expected an Inter family name after loading the fixture font (names_head={:?})",
                    text.all_font_names()
                        .into_iter()
                        .take(8)
                        .collect::<Vec<_>>()
                )
            })
    }

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

        let residency = visible_text_residency_for_scene(&scene, &text, 1.0, (240, 80));
        assert_eq!(residency.text_blob_ids(), vec![visible_blob]);

        let perf = text.prepare_for_text_residency_with_perf(&residency, 0, true);
        assert_eq!(perf.scene_text_blobs, 1);
        assert!(perf.added_glyph_keys > 0);

        let visible_snapshot = text.text_resource_snapshot_for_residency(&residency);
        assert!(visible_snapshot.glyphs > 0);
        assert_eq!(visible_snapshot.missing_glyph_resources, 0);

        let offscreen_snapshot = text.test_full_blob_text_resource_snapshot(&[offscreen_blob]);
        assert!(offscreen_snapshot.glyphs > 0);
        assert_eq!(
            offscreen_snapshot.missing_glyph_resources, offscreen_snapshot.glyphs,
            "offscreen text must not be pulled into atlas residency by frame prepare"
        );
    }

    #[test]
    fn visible_text_glyph_residency_excludes_offscreen_suffix_glyphs() {
        let ctx = pollster::block_on(WgpuContext::new()).expect("wgpu context");
        let mut text = TextSystem::new(&ctx.device);
        let style = TextStyle {
            size: Px(24.0),
            ..Default::default()
        };
        let (blob, _) = text.prepare(
            "abcdefghijklmnopqrstuvwxyz",
            &style,
            TextConstraints::default(),
        );
        let full_before = text.test_full_blob_text_resource_snapshot(&[blob]);
        assert!(full_before.glyphs > 4);

        let mut scene = Scene::default();
        scene.push(white_text_op(Point::new(Px(0.0), Px(32.0)), blob));

        let residency = visible_text_residency_for_scene(&scene, &text, 1.0, (36, 80));
        assert_eq!(residency.text_blob_ids(), vec![blob]);
        assert!(residency.glyph_count() > 0);
        assert!(
            residency.glyph_count() < full_before.glyphs as usize,
            "narrow viewport should select only a glyph prefix from the long blob"
        );

        let perf = text.prepare_for_text_residency_with_perf(&residency, 0, true);
        assert_eq!(perf.scene_text_blobs, 1);
        assert!(perf.added_glyph_keys > 0);

        let visible_snapshot = text.text_resource_snapshot_for_residency(&residency);
        assert_eq!(visible_snapshot.glyphs, residency.glyph_count() as u64);
        assert_eq!(visible_snapshot.missing_glyph_resources, 0);

        let full_after = text.test_full_blob_text_resource_snapshot(&[blob]);
        assert_eq!(full_after.glyphs, full_before.glyphs);
        assert!(
            full_after.missing_glyph_resources > 0,
            "offscreen suffix glyph resources must remain absent after visible-prefix prewarm"
        );
    }

    #[test]
    fn visible_text_residency_pins_complete_combining_cluster_under_narrow_scissor() {
        let ctx = pollster::block_on(WgpuContext::new()).expect("wgpu context");
        let mut text = TextSystem::new(&ctx.device);
        let family = inter_fixture_family(&mut text);
        let style = TextStyle {
            font: fret_core::FontId::family(family),
            size: Px(32.0),
            ..Default::default()
        };
        let (blob, _) = text.prepare(
            "a\u{0301}\u{0327}zzzzzzzz",
            &style,
            TextConstraints::default(),
        );
        let full_before = text.test_full_blob_text_resource_snapshot(&[blob]);
        assert!(
            full_before.glyphs > 3,
            "test setup expects a visible combining cluster plus an offscreen suffix"
        );

        let mut scene = Scene::default();
        scene.push(white_text_op(Point::new(Px(0.0), Px(32.0)), blob));

        let residency = visible_text_residency_for_scene(&scene, &text, 1.0, (4, 80));
        assert_eq!(residency.text_blob_ids(), vec![blob]);
        assert!(
            residency.cluster_count() >= 1,
            "narrow viewport should select at least the leading shaped cluster"
        );
        assert!(
            residency.glyph_count() >= 2,
            "the leading combining cluster must pin all of its glyphs"
        );
        assert!(
            residency.glyph_count() < full_before.glyphs as usize,
            "offscreen suffix clusters must remain outside visible residency"
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

        let residency = visible_text_residency_for_scene(&scene, &text_system, 1.0, (240, 80));
        assert_eq!(residency.text_blob_ids(), vec![visible_blob]);
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

        let residency = visible_text_residency_for_scene(&scene, &text_system, 1.0, (240, 80));
        assert_eq!(
            residency.text_blob_ids(),
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

        let residency = visible_text_residency_for_scene(&scene, &text_system, 1.0, (240, 80));
        assert!(
            residency.text_blob_ids().is_empty(),
            "wgpu encode uses composite group bounds as a work scissor"
        );
    }
}
