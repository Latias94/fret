use std::sync::Arc;

use super::{
    EffectStep, Mask, Paint, PaintBindingV1, SceneOp, SceneRecording, TextBlobId, mix_scene_op,
};
use crate::geometry::{Point, Transform2D};
use crate::{EffectId, ImageId, MaterialId, PathId, RenderTargetId, SvgId};
use slotmap::Key;

#[derive(Debug, Default, Clone)]
pub struct SceneChunk {
    ops: Arc<[SceneOp]>,
    text_blob_ids: Arc<[TextBlobId]>,
    closure: SceneChunkClosureMetadata,
    fingerprint: u64,
}

#[derive(Debug, Default, Clone, Copy, PartialEq, Eq)]
pub struct SceneChunkOpRange {
    pub start: u32,
    pub end: u32,
}

impl SceneChunkOpRange {
    pub fn new(start: usize, end: usize) -> Self {
        Self {
            start: u32::try_from(start).unwrap_or(u32::MAX),
            end: u32::try_from(end).unwrap_or(u32::MAX),
        }
    }

    pub fn len(self) -> u32 {
        self.end.saturating_sub(self.start)
    }

    pub fn is_empty(self) -> bool {
        self.start >= self.end
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum SceneChunkScopeKind {
    Transform,
    Opacity,
    Layer,
    Clip,
    Mask,
    Effect,
    BackdropSource,
    Composite,
}

impl SceneChunkScopeKind {
    const ALL: [Self; 8] = [
        Self::Transform,
        Self::Opacity,
        Self::Layer,
        Self::Clip,
        Self::Mask,
        Self::Effect,
        Self::BackdropSource,
        Self::Composite,
    ];

    const fn index(self) -> usize {
        match self {
            Self::Transform => 0,
            Self::Opacity => 1,
            Self::Layer => 2,
            Self::Clip => 3,
            Self::Mask => 4,
            Self::Effect => 5,
            Self::BackdropSource => 6,
            Self::Composite => 7,
        }
    }
}

#[derive(Debug, Default, Clone, Copy, PartialEq, Eq)]
pub struct SceneChunkScopeClosure {
    pub pushes: u32,
    pub pops: u32,
    pub inherited_pops: u32,
    pub open_pushes: u32,
}

impl SceneChunkScopeClosure {
    pub fn is_balanced(self) -> bool {
        self.inherited_pops == 0 && self.open_pushes == 0
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum SceneChunkClosureUnsupportedReason {
    InheritedScope(SceneChunkScopeKind),
    OpenScope(SceneChunkScopeKind),
}

#[derive(Debug, Default, Clone, Copy, PartialEq, Eq)]
pub struct SceneChunkDrawStreamSummary {
    pub quad_ops: u32,
    pub image_ops: u32,
    pub vertex_color_ops: u32,
    pub mask_image_ops: u32,
    pub svg_ops: u32,
    pub text_ops: u32,
    pub path_ops: u32,
    pub viewport_ops: u32,
}

impl SceneChunkDrawStreamSummary {
    pub fn draw_ops(self) -> u32 {
        self.quad_ops
            .saturating_add(self.image_ops)
            .saturating_add(self.vertex_color_ops)
            .saturating_add(self.mask_image_ops)
            .saturating_add(self.svg_ops)
            .saturating_add(self.text_ops)
            .saturating_add(self.path_ops)
            .saturating_add(self.viewport_ops)
    }

    pub fn is_quad_only(self) -> bool {
        self.quad_ops > 0 && self.draw_ops() == self.quad_ops
    }
}

#[derive(Debug, Default, Clone, PartialEq, Eq)]
pub struct SceneChunkResourceClosure {
    images: Arc<[ImageId]>,
    svgs: Arc<[SvgId]>,
    paths: Arc<[PathId]>,
    text_blobs: Arc<[TextBlobId]>,
    materials: Arc<[MaterialId]>,
    effects: Arc<[EffectId]>,
    render_targets: Arc<[RenderTargetId]>,
}

impl SceneChunkResourceClosure {
    pub fn images(&self) -> &[ImageId] {
        &self.images
    }

    pub fn svgs(&self) -> &[SvgId] {
        &self.svgs
    }

    pub fn paths(&self) -> &[PathId] {
        &self.paths
    }

    pub fn text_blobs(&self) -> &[TextBlobId] {
        &self.text_blobs
    }

    pub fn materials(&self) -> &[MaterialId] {
        &self.materials
    }

    pub fn effects(&self) -> &[EffectId] {
        &self.effects
    }

    pub fn render_targets(&self) -> &[RenderTargetId] {
        &self.render_targets
    }

    pub fn is_empty(&self) -> bool {
        self.images.is_empty()
            && self.svgs.is_empty()
            && self.paths.is_empty()
            && self.text_blobs.is_empty()
            && self.materials.is_empty()
            && self.effects.is_empty()
            && self.render_targets.is_empty()
    }
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct SceneChunkClosureMetadata {
    op_range: SceneChunkOpRange,
    scopes: [SceneChunkScopeClosure; SceneChunkScopeKind::ALL.len()],
    scope_unsupported_reasons: Arc<[SceneChunkClosureUnsupportedReason]>,
    draw_streams: SceneChunkDrawStreamSummary,
    resources: SceneChunkResourceClosure,
    resource_fingerprint: u64,
    fingerprint: u64,
}

impl Default for SceneChunkClosureMetadata {
    fn default() -> Self {
        Self {
            op_range: SceneChunkOpRange::default(),
            scopes: [SceneChunkScopeClosure::default(); SceneChunkScopeKind::ALL.len()],
            scope_unsupported_reasons: Arc::default(),
            draw_streams: SceneChunkDrawStreamSummary::default(),
            resources: SceneChunkResourceClosure::default(),
            resource_fingerprint: 0,
            fingerprint: 0,
        }
    }
}

impl SceneChunkClosureMetadata {
    pub fn op_range(&self) -> SceneChunkOpRange {
        self.op_range
    }

    pub fn scope(&self, kind: SceneChunkScopeKind) -> SceneChunkScopeClosure {
        self.scopes[kind.index()]
    }

    pub fn scope_unsupported_reasons(&self) -> &[SceneChunkClosureUnsupportedReason] {
        &self.scope_unsupported_reasons
    }

    pub fn draw_streams(&self) -> SceneChunkDrawStreamSummary {
        self.draw_streams
    }

    pub fn resources(&self) -> &SceneChunkResourceClosure {
        &self.resources
    }

    pub fn resource_fingerprint(&self) -> u64 {
        self.resource_fingerprint
    }

    pub fn fingerprint(&self) -> u64 {
        self.fingerprint
    }

    pub fn is_scope_closed(&self) -> bool {
        self.scope_unsupported_reasons.is_empty()
    }

    pub fn is_resource_free_quad_only(&self) -> bool {
        self.is_scope_closed() && self.resources.is_empty() && self.draw_streams.is_quad_only()
    }

    fn from_ops(ops: &[SceneOp], text_blob_ids: &[TextBlobId]) -> Self {
        let mut builder = SceneChunkClosureBuilder::new(ops.len());
        for op in ops {
            builder.record_op(*op);
        }
        builder.finish(text_blob_ids)
    }
}

impl SceneChunk {
    pub fn from_scene(scene: &SceneRecording) -> Self {
        Self::from_ops_and_text_blob_ids(
            Arc::from(scene.ops().to_vec()),
            Arc::from(scene.text_blob_ids().to_vec()),
        )
    }

    pub fn from_ops(ops: Arc<[SceneOp]>) -> Self {
        let text_blob_ids = Arc::from(
            ops.iter()
                .filter_map(|op| match op {
                    SceneOp::Text { text, .. } => Some(*text),
                    _ => None,
                })
                .collect::<Vec<_>>(),
        );
        Self::from_ops_and_text_blob_ids(ops, text_blob_ids)
    }

    pub fn from_ops_and_text_blob_ids(
        ops: Arc<[SceneOp]>,
        text_blob_ids: Arc<[TextBlobId]>,
    ) -> Self {
        #[cfg(debug_assertions)]
        debug_assert!(
            ops.iter()
                .filter_map(|op| match op {
                    SceneOp::Text { text, .. } => Some(*text),
                    _ => None,
                })
                .eq(text_blob_ids.iter().copied()),
            "SceneChunk::from_ops_and_text_blob_ids() received a text blob index that does not match the retained ops"
        );

        let fingerprint = ops
            .iter()
            .fold(0, |fingerprint, op| mix_scene_op(fingerprint, *op));
        let closure = SceneChunkClosureMetadata::from_ops(&ops, &text_blob_ids);
        Self {
            ops,
            text_blob_ids,
            closure,
            fingerprint,
        }
    }

    pub fn ops(&self) -> &[SceneOp] {
        &self.ops
    }

    pub fn ops_len(&self) -> usize {
        self.ops.len()
    }

    pub fn is_empty(&self) -> bool {
        self.ops.is_empty()
    }

    pub fn text_blob_ids(&self) -> &[TextBlobId] {
        &self.text_blob_ids
    }

    pub fn closure(&self) -> &SceneChunkClosureMetadata {
        &self.closure
    }

    pub fn fingerprint(&self) -> u64 {
        self.fingerprint
    }

    pub fn replay_into(&self, scene: &mut SceneRecording) {
        scene.replay_ops_with_text_blob_ids(self.ops(), self.text_blob_ids());
    }

    pub fn replay_translated_into(&self, scene: &mut SceneRecording, delta: Point) {
        scene.replay_ops_translated_with_text_blob_ids(self.ops(), delta, self.text_blob_ids());
    }

    pub fn replay_transformed_into(&self, scene: &mut SceneRecording, transform: Transform2D) {
        scene.replay_ops_transformed_with_text_blob_ids(
            self.ops(),
            transform,
            self.text_blob_ids(),
        );
    }
}

struct SceneChunkClosureBuilder {
    op_range: SceneChunkOpRange,
    scopes: [SceneChunkScopeClosure; SceneChunkScopeKind::ALL.len()],
    scope_depths: [u32; SceneChunkScopeKind::ALL.len()],
    draw_streams: SceneChunkDrawStreamSummary,
    images: Vec<ImageId>,
    svgs: Vec<SvgId>,
    paths: Vec<PathId>,
    text_blobs: Vec<TextBlobId>,
    materials: Vec<MaterialId>,
    effects: Vec<EffectId>,
    render_targets: Vec<RenderTargetId>,
}

impl SceneChunkClosureBuilder {
    fn new(op_count: usize) -> Self {
        Self {
            op_range: SceneChunkOpRange::new(0, op_count),
            scopes: [SceneChunkScopeClosure::default(); SceneChunkScopeKind::ALL.len()],
            scope_depths: [0; SceneChunkScopeKind::ALL.len()],
            draw_streams: SceneChunkDrawStreamSummary::default(),
            images: Vec::new(),
            svgs: Vec::new(),
            paths: Vec::new(),
            text_blobs: Vec::new(),
            materials: Vec::new(),
            effects: Vec::new(),
            render_targets: Vec::new(),
        }
    }

    fn record_op(&mut self, op: SceneOp) {
        match op {
            SceneOp::PushTransform { .. } => self.push_scope(SceneChunkScopeKind::Transform),
            SceneOp::PopTransform => self.pop_scope(SceneChunkScopeKind::Transform),
            SceneOp::PushOpacity { .. } => self.push_scope(SceneChunkScopeKind::Opacity),
            SceneOp::PopOpacity => self.pop_scope(SceneChunkScopeKind::Opacity),
            SceneOp::PushLayer { .. } => self.push_scope(SceneChunkScopeKind::Layer),
            SceneOp::PopLayer => self.pop_scope(SceneChunkScopeKind::Layer),
            SceneOp::PushClipRect { .. } | SceneOp::PushClipRRect { .. } => {
                self.push_scope(SceneChunkScopeKind::Clip);
            }
            SceneOp::PushClipPath { path, .. } => {
                self.push_scope(SceneChunkScopeKind::Clip);
                push_unique(&mut self.paths, path);
            }
            SceneOp::PopClip => self.pop_scope(SceneChunkScopeKind::Clip),
            SceneOp::PushMask { mask, .. } => {
                self.push_scope(SceneChunkScopeKind::Mask);
                self.record_mask(mask);
            }
            SceneOp::PopMask => self.pop_scope(SceneChunkScopeKind::Mask),
            SceneOp::PushEffect { chain, .. } => {
                self.push_scope(SceneChunkScopeKind::Effect);
                self.record_effect_chain(chain);
            }
            SceneOp::PopEffect => self.pop_scope(SceneChunkScopeKind::Effect),
            SceneOp::PushBackdropSourceGroupV1 { .. } => {
                self.push_scope(SceneChunkScopeKind::BackdropSource);
            }
            SceneOp::PopBackdropSourceGroup => self.pop_scope(SceneChunkScopeKind::BackdropSource),
            SceneOp::PushCompositeGroup { .. } => self.push_scope(SceneChunkScopeKind::Composite),
            SceneOp::PopCompositeGroup => self.pop_scope(SceneChunkScopeKind::Composite),
            SceneOp::Quad {
                background,
                border_paint,
                ..
            } => {
                self.draw_streams.quad_ops = self.draw_streams.quad_ops.saturating_add(1);
                self.record_paint_binding(background);
                self.record_paint_binding(border_paint);
            }
            SceneOp::StrokeRRect { stroke_paint, .. } => {
                self.draw_streams.quad_ops = self.draw_streams.quad_ops.saturating_add(1);
                self.record_paint_binding(stroke_paint);
            }
            SceneOp::ShadowRRect { .. } => {
                self.draw_streams.quad_ops = self.draw_streams.quad_ops.saturating_add(1);
            }
            SceneOp::Image { image, .. }
            | SceneOp::ImageRegion { image, .. }
            | SceneOp::ImageQuad { image, .. }
            | SceneOp::ImageTriangle { image, .. } => {
                self.draw_streams.image_ops = self.draw_streams.image_ops.saturating_add(1);
                push_unique(&mut self.images, image);
            }
            SceneOp::VertexColorQuad { .. } | SceneOp::VertexColorTriangle { .. } => {
                self.draw_streams.vertex_color_ops =
                    self.draw_streams.vertex_color_ops.saturating_add(1);
            }
            SceneOp::MaskImage { image, .. } => {
                self.draw_streams.mask_image_ops =
                    self.draw_streams.mask_image_ops.saturating_add(1);
                push_unique(&mut self.images, image);
            }
            SceneOp::SvgMaskIcon { svg, .. } | SceneOp::SvgImage { svg, .. } => {
                self.draw_streams.svg_ops = self.draw_streams.svg_ops.saturating_add(1);
                push_unique(&mut self.svgs, svg);
            }
            SceneOp::Text {
                text,
                paint,
                outline,
                ..
            } => {
                self.draw_streams.text_ops = self.draw_streams.text_ops.saturating_add(1);
                push_unique(&mut self.text_blobs, text);
                self.record_paint_binding(paint);
                if let Some(outline) = outline {
                    self.record_paint_binding(outline.paint);
                }
            }
            SceneOp::Path { path, paint, .. } => {
                self.draw_streams.path_ops = self.draw_streams.path_ops.saturating_add(1);
                push_unique(&mut self.paths, path);
                self.record_paint_binding(paint);
            }
            SceneOp::ViewportSurface { target, .. } => {
                self.draw_streams.viewport_ops = self.draw_streams.viewport_ops.saturating_add(1);
                push_unique(&mut self.render_targets, target);
            }
        }
    }

    fn push_scope(&mut self, kind: SceneChunkScopeKind) {
        let index = kind.index();
        self.scopes[index].pushes = self.scopes[index].pushes.saturating_add(1);
        self.scope_depths[index] = self.scope_depths[index].saturating_add(1);
    }

    fn pop_scope(&mut self, kind: SceneChunkScopeKind) {
        let index = kind.index();
        self.scopes[index].pops = self.scopes[index].pops.saturating_add(1);
        if self.scope_depths[index] == 0 {
            self.scopes[index].inherited_pops = self.scopes[index].inherited_pops.saturating_add(1);
        } else {
            self.scope_depths[index] -= 1;
        }
    }

    fn record_mask(&mut self, mask: Mask) {
        if let Mask::Image { image, .. } = mask {
            push_unique(&mut self.images, image);
        }
    }

    fn record_paint_binding(&mut self, paint: PaintBindingV1) {
        if let Paint::Material { id, .. } = paint.paint {
            push_unique(&mut self.materials, id);
        }
    }

    fn record_effect_chain(&mut self, chain: super::EffectChain) {
        for step in chain.iter() {
            self.record_effect_step(step);
        }
    }

    fn record_effect_step(&mut self, step: EffectStep) {
        match step {
            EffectStep::BackdropWarpV2(warp) => {
                if let super::BackdropWarpFieldV2::ImageDisplacementMap { image, .. } = warp.field {
                    push_unique(&mut self.images, image);
                }
            }
            EffectStep::CustomV1 { id, .. } => push_unique(&mut self.effects, id),
            EffectStep::CustomV2 {
                id, input_image, ..
            } => {
                push_unique(&mut self.effects, id);
                self.record_custom_effect_image_input(input_image);
            }
            EffectStep::CustomV3 {
                id, user0, user1, ..
            } => {
                push_unique(&mut self.effects, id);
                self.record_custom_effect_image_input(user0);
                self.record_custom_effect_image_input(user1);
            }
            EffectStep::GaussianBlur { .. }
            | EffectStep::DropShadowV1(_)
            | EffectStep::BackdropWarpV1(_)
            | EffectStep::NoiseV1(_)
            | EffectStep::ColorAdjust { .. }
            | EffectStep::ColorMatrix { .. }
            | EffectStep::AlphaThreshold { .. }
            | EffectStep::Pixelate { .. }
            | EffectStep::Dither { .. } => {}
        }
    }

    fn record_custom_effect_image_input(&mut self, input: Option<super::CustomEffectImageInputV1>) {
        if let Some(input) = input {
            push_unique(&mut self.images, input.image);
        }
    }

    fn finish(mut self, text_blob_ids: &[TextBlobId]) -> SceneChunkClosureMetadata {
        for &text in text_blob_ids {
            push_unique(&mut self.text_blobs, text);
        }

        let mut unsupported = Vec::new();
        for kind in SceneChunkScopeKind::ALL {
            let index = kind.index();
            self.scopes[index].open_pushes = self.scope_depths[index];
            if self.scopes[index].inherited_pops > 0 {
                unsupported.push(SceneChunkClosureUnsupportedReason::InheritedScope(kind));
            }
            if self.scopes[index].open_pushes > 0 {
                unsupported.push(SceneChunkClosureUnsupportedReason::OpenScope(kind));
            }
        }

        let resources = SceneChunkResourceClosure {
            images: Arc::from(self.images),
            svgs: Arc::from(self.svgs),
            paths: Arc::from(self.paths),
            text_blobs: Arc::from(self.text_blobs),
            materials: Arc::from(self.materials),
            effects: Arc::from(self.effects),
            render_targets: Arc::from(self.render_targets),
        };
        let resource_fingerprint = resource_fingerprint(&resources);
        let mut fingerprint = 0x2b4e_1709_a51c_b011u64;
        fingerprint = mix_u64(fingerprint, u64::from(self.op_range.start));
        fingerprint = mix_u64(fingerprint, u64::from(self.op_range.end));
        fingerprint = mix_scope_closures(fingerprint, &self.scopes);
        fingerprint = mix_draw_streams(fingerprint, self.draw_streams);
        fingerprint = mix_u64(fingerprint, resource_fingerprint);
        for reason in &unsupported {
            fingerprint = mix_unsupported_reason(fingerprint, *reason);
        }

        SceneChunkClosureMetadata {
            op_range: self.op_range,
            scopes: self.scopes,
            scope_unsupported_reasons: Arc::from(unsupported),
            draw_streams: self.draw_streams,
            resources,
            resource_fingerprint,
            fingerprint,
        }
    }
}

fn push_unique<T: Copy + Eq>(values: &mut Vec<T>, value: T) {
    if !values.contains(&value) {
        values.push(value);
    }
}

fn mix_u64(mut state: u64, value: u64) -> u64 {
    state ^= value.wrapping_add(0x9E37_79B9_7F4A_7C15);
    state = state.rotate_left(7);
    state.wrapping_mul(0xD6E8_FEB8_6659_FD93)
}

fn mix_key<K: Key>(state: u64, key: K) -> u64 {
    mix_u64(state, key.data().as_ffi())
}

fn mix_scope_kind(state: u64, kind: SceneChunkScopeKind) -> u64 {
    mix_u64(state, kind.index() as u64)
}

fn mix_scope_closures(
    mut state: u64,
    scopes: &[SceneChunkScopeClosure; SceneChunkScopeKind::ALL.len()],
) -> u64 {
    for kind in SceneChunkScopeKind::ALL {
        let scope = scopes[kind.index()];
        state = mix_scope_kind(state, kind);
        state = mix_u64(state, u64::from(scope.pushes));
        state = mix_u64(state, u64::from(scope.pops));
        state = mix_u64(state, u64::from(scope.inherited_pops));
        state = mix_u64(state, u64::from(scope.open_pushes));
    }
    state
}

fn mix_unsupported_reason(mut state: u64, reason: SceneChunkClosureUnsupportedReason) -> u64 {
    match reason {
        SceneChunkClosureUnsupportedReason::InheritedScope(kind) => {
            state = mix_u64(state, 1);
            mix_scope_kind(state, kind)
        }
        SceneChunkClosureUnsupportedReason::OpenScope(kind) => {
            state = mix_u64(state, 2);
            mix_scope_kind(state, kind)
        }
    }
}

fn mix_draw_streams(mut state: u64, draw_streams: SceneChunkDrawStreamSummary) -> u64 {
    state = mix_u64(state, u64::from(draw_streams.quad_ops));
    state = mix_u64(state, u64::from(draw_streams.image_ops));
    state = mix_u64(state, u64::from(draw_streams.vertex_color_ops));
    state = mix_u64(state, u64::from(draw_streams.mask_image_ops));
    state = mix_u64(state, u64::from(draw_streams.svg_ops));
    state = mix_u64(state, u64::from(draw_streams.text_ops));
    state = mix_u64(state, u64::from(draw_streams.path_ops));
    mix_u64(state, u64::from(draw_streams.viewport_ops))
}

fn resource_fingerprint(resources: &SceneChunkResourceClosure) -> u64 {
    let mut state = 0x87c2_93bd_f3c4_591du64;
    state = mix_key_slice(state, 1, resources.images());
    state = mix_key_slice(state, 2, resources.svgs());
    state = mix_key_slice(state, 3, resources.paths());
    state = mix_key_slice(state, 4, resources.text_blobs());
    state = mix_key_slice(state, 5, resources.materials());
    state = mix_key_slice(state, 6, resources.effects());
    mix_key_slice(state, 7, resources.render_targets())
}

fn mix_key_slice<K: Key>(mut state: u64, tag: u64, values: &[K]) -> u64 {
    state = mix_u64(state, tag);
    state = mix_u64(state, values.len() as u64);
    for &value in values {
        state = mix_key(state, value);
    }
    state
}
