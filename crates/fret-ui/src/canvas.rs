use std::any::{Any, TypeId};
use std::collections::HashMap;
use std::hash::{Hash, Hasher};
use std::sync::Arc;

use fret_core::scene::{Paint, PaintBindingV1, SceneChunk};
use fret_core::{
    AttributedText, Color, Corners, DrawOrder, EffectChain, EffectMode, EffectQuality, FontId,
    FontWeight, Point, Px, Rect, Scene, SceneOp, SvgFit, TextConstraints, TextMetrics,
    TextOverflow, TextSlant, TextStyle, TextWrap, Transform2D,
};
use fret_core::{PathCommand, PathConstraints, PathMetrics, PathStyle};
use fret_runtime::ModelId;
use smallvec::SmallVec;

use crate::Theme;
use crate::element::CanvasCachePolicy;
use crate::widget::Invalidation;
use crate::{SvgSource, UiHost, widget::PaintCx};

pub type OnCanvasPaint = Arc<dyn for<'a> Fn(&mut CanvasPainter<'a>) + 'static>;
pub type OnCanvasPrepaint = Arc<dyn for<'a> Fn(&mut CanvasPrepaintCx<'a>) + 'static>;

#[derive(Debug, Default, Clone, Copy, PartialEq, Eq)]
pub struct CanvasHostedResourceTouchCounts {
    pub text_blobs: u32,
    pub paths: u32,
    pub svgs: u32,
}

#[derive(Debug, Clone)]
pub struct CanvasSceneFragment<T> {
    pub payload: T,
    pub chunk: SceneChunk,
    pub hosted_resources: CanvasHostedResources,
    pub local_bounds: Rect,
    pub scene_origin: Point,
}

impl<T> CanvasSceneFragment<T> {
    pub fn new(
        payload: T,
        ops: Arc<[SceneOp]>,
        hosted_resources: CanvasHostedResources,
        local_bounds: Rect,
        scene_origin: Point,
    ) -> Self {
        let chunk = SceneChunk::from_ops(ops);
        Self::from_chunk(payload, chunk, hosted_resources, local_bounds, scene_origin)
    }

    pub fn from_chunk(
        payload: T,
        chunk: SceneChunk,
        hosted_resources: CanvasHostedResources,
        local_bounds: Rect,
        scene_origin: Point,
    ) -> Self {
        Self {
            payload,
            chunk,
            hosted_resources,
            local_bounds,
            scene_origin,
        }
    }

    pub fn ops(&self) -> &[SceneOp] {
        self.chunk.ops()
    }

    pub fn text_blob_ids(&self) -> &[fret_core::TextBlobId] {
        self.chunk.text_blob_ids()
    }

    pub fn fingerprint(&self) -> u64 {
        self.chunk.fingerprint()
    }

    pub fn replay_translated_into(&self, scene: &mut Scene, delta: Point) {
        self.chunk.replay_translated_into(scene, delta);
    }
}

impl<T: crate::tree::BoundarySceneFragmentDebug> crate::tree::BoundarySceneFragmentDebug
    for CanvasSceneFragment<T>
{
    fn boundary_scene_fragment_entry_count(&self) -> usize {
        self.payload.boundary_scene_fragment_entry_count()
    }

    fn boundary_scene_fragment_chunk_count(&self) -> usize {
        usize::from(!self.chunk.is_empty())
    }

    fn boundary_scene_fragment_fingerprint(&self) -> u64 {
        if self.chunk.is_empty() {
            return 0;
        }

        fret_core::SceneChunkManifestEntry::new(
            self.chunk.clone(),
            self.local_bounds,
            self.scene_origin,
        )
        .fingerprint()
    }

    fn append_boundary_scene_fragment_chunks(
        &self,
        out: &mut crate::tree::BoundarySceneChunkManifest,
    ) {
        out.push(crate::tree::BoundarySceneFragmentChunk::new(
            self.chunk.clone(),
            self.local_bounds,
            self.scene_origin,
        ));
    }
}

/// Precomputed hosted resource references extracted from retained `SceneOp`s.
///
/// Replay caches can store this alongside the op buffer so cache-hit paths only need to touch the
/// owned resource IDs instead of scanning the entire op slice again.
#[derive(Debug, Default, Clone, PartialEq, Eq)]
pub struct CanvasHostedResources {
    text_blobs: SmallVec<[fret_core::TextBlobId; 1]>,
    paths: SmallVec<[fret_core::PathId; 1]>,
    svgs: SmallVec<[fret_core::SvgId; 1]>,
}

impl CanvasHostedResources {
    /// Record the hosted resources referenced by a single `SceneOp`.
    pub fn push_scene_op(&mut self, op: SceneOp) {
        match op {
            SceneOp::Text { text, .. } => self.text_blobs.push(text),
            SceneOp::Path { path, .. } | SceneOp::PushClipPath { path, .. } => {
                self.paths.push(path)
            }
            SceneOp::SvgMaskIcon { svg, .. } | SceneOp::SvgImage { svg, .. } => self.svgs.push(svg),
            _ => {}
        }
    }

    /// Record the hosted resources referenced by a retained scene-op slice.
    pub fn extend_scene_ops(&mut self, ops: &[SceneOp]) {
        for &op in ops {
            self.push_scene_op(op);
        }
    }

    /// Merge precomputed hosted-resource references from another retained scene fragment.
    pub fn extend_resources(&mut self, resources: &CanvasHostedResources) {
        self.text_blobs.extend(resources.text_blobs.iter().copied());
        self.paths.extend(resources.paths.iter().copied());
        self.svgs.extend(resources.svgs.iter().copied());
    }

    /// Build a precomputed hosted-resource list from retained scene ops.
    pub fn from_scene_ops(ops: &[SceneOp]) -> Self {
        let mut resources = Self::default();
        resources.extend_scene_ops(ops);
        resources
    }

    pub fn text_blob_ids(&self) -> &[fret_core::TextBlobId] {
        &self.text_blobs
    }

    pub fn is_empty(&self) -> bool {
        self.text_blobs.is_empty() && self.paths.is_empty() && self.svgs.is_empty()
    }
}

/// A stable, user-provided cache key for hosted canvas resources.
///
/// Callers should treat this as an identity key for a logical draw item that is stable across
/// frames (e.g. "grid label #42"). The runtime mixes in scale-factor bits where needed, so the
/// same key can be reused across DPI/zoom changes.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub struct CanvasKey(pub u64);

impl CanvasKey {
    pub const fn new(value: u64) -> Self {
        Self(value)
    }

    /// Combine a child identifier into this key.
    pub fn combine(self, value: u64) -> Self {
        Self(mix_u64(self.0, value))
    }

    /// Combine a deterministic hash of `value` into this key.
    pub fn combine_hash<T: Hash>(self, value: &T) -> Self {
        self.combine(Self::from_hash(value).0)
    }

    /// Compute a deterministic hash for `value`.
    ///
    /// This uses a fixed-seed FNV-1a hasher (unlike `DefaultHasher`, which is randomized).
    pub fn from_hash<T: Hash>(value: &T) -> Self {
        let mut hasher = Fnv1a64::default();
        value.hash(&mut hasher);
        Self(hasher.finish())
    }
}

impl From<CanvasKey> for u64 {
    fn from(value: CanvasKey) -> Self {
        value.0
    }
}

impl From<u64> for CanvasKey {
    fn from(value: u64) -> Self {
        Self(value)
    }
}

#[derive(Default)]
pub(crate) struct CanvasPaintHooks {
    pub on_paint: Option<OnCanvasPaint>,
    pub on_prepaint: Option<OnCanvasPrepaint>,
}

pub(crate) trait UiCanvasPrepaintHost {
    fn bounds(&self) -> Rect;
    fn scale_factor(&self) -> f32;
    fn text_font_stack_key(&mut self) -> u64;
    fn theme(&self) -> &Theme;
    fn services(&mut self) -> &mut dyn fret_core::UiServices;
    fn request_redraw(&mut self);
    fn request_animation_frame(&mut self);
    fn set_output_box(&mut self, ty: TypeId, value: Box<dyn Any>);
    fn output_any(&self, ty: TypeId) -> Option<&dyn Any>;
    fn output_any_mut(&mut self, ty: TypeId) -> Option<&mut dyn Any>;
    fn set_scene_fragment_box(&mut self, ty: TypeId, value: Box<dyn Any>);
    fn set_scene_fragment_box_with_debug_counts(
        &mut self,
        ty: TypeId,
        value: Box<dyn Any>,
        entry_count: usize,
        chunk_count: usize,
        fingerprint: u64,
        chunks: crate::tree::BoundarySceneChunkManifest,
    );
    fn scene_fragment_any(&self, ty: TypeId) -> Option<&dyn Any>;
    fn scene_fragment_any_mut(&mut self, ty: TypeId) -> Option<&mut dyn Any>;
    fn record_scene_fragment_used_entries(&mut self, count: usize);
    fn record_scene_fragment_rejected_entries(&mut self, count: usize, reason: &'static str);
}

pub(crate) struct UiCanvasPrepaintHostAdapter<'a, 'b, H: UiHost> {
    cx: &'a mut crate::widget::PrepaintCx<'b, H>,
}

impl<'a, 'b, H: UiHost> UiCanvasPrepaintHostAdapter<'a, 'b, H> {
    pub(crate) fn new(cx: &'a mut crate::widget::PrepaintCx<'b, H>) -> Self {
        Self { cx }
    }
}

impl<'a, 'b, H: UiHost> UiCanvasPrepaintHost for UiCanvasPrepaintHostAdapter<'a, 'b, H> {
    fn bounds(&self) -> Rect {
        self.cx.bounds
    }

    fn scale_factor(&self) -> f32 {
        self.cx.scale_factor
    }

    fn text_font_stack_key(&mut self) -> u64 {
        self.cx
            .app
            .global::<fret_runtime::TextFontStackKey>()
            .map(|k| k.0)
            .unwrap_or(0)
    }

    fn theme(&self) -> &Theme {
        self.cx.theme()
    }

    fn services(&mut self) -> &mut dyn fret_core::UiServices {
        self.cx.services
    }

    fn request_redraw(&mut self) {
        self.cx.request_redraw();
    }

    fn request_animation_frame(&mut self) {
        self.cx.request_animation_frame();
    }

    fn set_output_box(&mut self, ty: TypeId, value: Box<dyn Any>) {
        self.cx
            .tree
            .set_prepaint_output_box(self.cx.node, ty, value);
    }

    fn output_any(&self, ty: TypeId) -> Option<&dyn Any> {
        self.cx.tree.prepaint_output_any(self.cx.node, ty)
    }

    fn output_any_mut(&mut self, ty: TypeId) -> Option<&mut dyn Any> {
        self.cx.tree.prepaint_output_any_mut(self.cx.node, ty)
    }

    fn set_scene_fragment_box(&mut self, ty: TypeId, value: Box<dyn Any>) {
        self.cx.tree.set_scene_fragment_box(self.cx.node, ty, value);
    }

    fn set_scene_fragment_box_with_debug_counts(
        &mut self,
        ty: TypeId,
        value: Box<dyn Any>,
        entry_count: usize,
        chunk_count: usize,
        fingerprint: u64,
        chunks: crate::tree::BoundarySceneChunkManifest,
    ) {
        self.cx.tree.set_scene_fragment_box_with_debug_counts(
            self.cx.node,
            ty,
            value,
            entry_count,
            chunk_count,
            fingerprint,
            chunks,
        );
    }

    fn scene_fragment_any(&self, ty: TypeId) -> Option<&dyn Any> {
        self.cx.tree.scene_fragment_any(self.cx.node, ty)
    }

    fn scene_fragment_any_mut(&mut self, ty: TypeId) -> Option<&mut dyn Any> {
        self.cx.tree.scene_fragment_any_mut(self.cx.node, ty)
    }

    fn record_scene_fragment_used_entries(&mut self, count: usize) {
        self.cx
            .tree
            .record_scene_fragment_used_entries(self.cx.node, count);
    }

    fn record_scene_fragment_rejected_entries(&mut self, count: usize, reason: &'static str) {
        self.cx
            .tree
            .record_scene_fragment_rejected_entries(self.cx.node, count, reason);
    }
}

pub struct CanvasPrepaintCx<'a> {
    host: &'a mut dyn UiCanvasPrepaintHost,
    cache: &'a mut CanvasCache,
}

impl<'a> CanvasPrepaintCx<'a> {
    pub(crate) fn new(host: &'a mut dyn UiCanvasPrepaintHost, cache: &'a mut CanvasCache) -> Self {
        Self { host, cache }
    }

    pub fn bounds(&self) -> Rect {
        self.host.bounds()
    }

    pub fn scale_factor(&self) -> f32 {
        self.host.scale_factor()
    }

    pub fn theme(&self) -> &Theme {
        self.host.theme()
    }

    pub fn request_redraw(&mut self) {
        self.host.request_redraw();
    }

    pub fn request_animation_frame(&mut self) {
        self.host.request_animation_frame();
    }

    pub fn set_output<T: Any>(&mut self, value: T) {
        self.host.set_output_box(TypeId::of::<T>(), Box::new(value));
    }

    pub fn output<T: Any>(&self) -> Option<&T> {
        self.host
            .output_any(TypeId::of::<T>())
            .and_then(|value| value.downcast_ref::<T>())
    }

    pub fn output_mut<T: Any>(&mut self) -> Option<&mut T> {
        self.host
            .output_any_mut(TypeId::of::<T>())
            .and_then(|value| value.downcast_mut::<T>())
    }

    pub fn set_scene_fragment<T: Any>(&mut self, value: T) {
        self.host
            .set_scene_fragment_box(TypeId::of::<T>(), Box::new(value));
    }

    pub fn set_scene_fragment_debug<T: crate::tree::BoundarySceneFragmentDebug>(
        &mut self,
        value: T,
    ) {
        let entry_count = value.boundary_scene_fragment_entry_count();
        let chunk_count = value.boundary_scene_fragment_chunk_count();
        let fingerprint = value.boundary_scene_fragment_fingerprint();
        let mut chunks = crate::tree::BoundarySceneChunkManifest::default();
        value.append_boundary_scene_fragment_chunks(&mut chunks);
        self.host.set_scene_fragment_box_with_debug_counts(
            TypeId::of::<T>(),
            Box::new(value),
            entry_count,
            chunk_count,
            fingerprint,
            chunks,
        );
    }

    pub fn scene_fragment<T: Any>(&self) -> Option<&T> {
        self.host
            .scene_fragment_any(TypeId::of::<T>())
            .and_then(|value| value.downcast_ref::<T>())
    }

    pub fn scene_fragment_mut<T: Any>(&mut self) -> Option<&mut T> {
        self.host
            .scene_fragment_any_mut(TypeId::of::<T>())
            .and_then(|value| value.downcast_mut::<T>())
    }

    pub fn record_scene_fragment_used_entries(&mut self, count: usize) {
        self.host.record_scene_fragment_used_entries(count);
    }

    pub fn record_scene_fragment_rejected_entries(&mut self, count: usize, reason: &'static str) {
        self.host
            .record_scene_fragment_rejected_entries(count, reason);
    }

    /// Prepare a retained scene fragment during prepaint without mutating the live paint scene.
    ///
    /// This is for prepaint-windowed surfaces that need to stage a small amount of canvas work
    /// before paint, then replay it through the boundary scene-fragment carrier. The closure draws
    /// into a scratch scene using the canvas hosted-resource cache shared with the later paint pass.
    pub fn prepare_scene_fragment<T>(
        &mut self,
        local_bounds: Rect,
        scene_origin: Point,
        prepare: impl FnOnce(&mut CanvasPrepaintPainter<'_>) -> T,
    ) -> CanvasSceneFragment<T> {
        let mut painter = CanvasPrepaintPainter {
            host: self.host,
            cache: self.cache,
            scene: Scene::default(),
        };
        let payload = prepare(&mut painter);
        let chunk = SceneChunk::from_scene(&painter.scene);
        let hosted_resources = CanvasHostedResources::from_scene_ops(chunk.ops());
        CanvasSceneFragment::from_chunk(
            payload,
            chunk,
            hosted_resources,
            local_bounds,
            scene_origin,
        )
    }

    /// Run a closure with a scratch prepaint painter.
    ///
    /// Use this when the caller needs to return both a prepared scene fragment and additional
    /// side-channel state for local caches.
    pub fn with_scene_painter<R>(
        &mut self,
        prepare: impl FnOnce(&mut CanvasPrepaintPainter<'_>) -> R,
    ) -> R {
        let mut painter = CanvasPrepaintPainter {
            host: self.host,
            cache: self.cache,
            scene: Scene::default(),
        };
        prepare(&mut painter)
    }
}

/// Scratch canvas painter used only by `CanvasPrepaintCx::prepare_scene_fragment`.
///
/// It exposes the hosted-resource/text preparation subset needed to build a replayable
/// `CanvasSceneFragment` while keeping the real paint scene untouched.
pub struct CanvasPrepaintPainter<'a> {
    host: &'a mut dyn UiCanvasPrepaintHost,
    cache: &'a mut CanvasCache,
    scene: Scene,
}

impl<'a> CanvasPrepaintPainter<'a> {
    pub fn bounds(&self) -> Rect {
        self.host.bounds()
    }

    pub fn scale_factor(&self) -> f32 {
        self.host.scale_factor()
    }

    pub fn theme(&self) -> &Theme {
        self.host.theme()
    }

    /// Compute a deterministic `u64` key for `value`.
    pub fn key<T: Hash>(&self, value: &T) -> u64 {
        CanvasKey::from_hash(value).0
    }

    /// Create a deterministic base key for a logical key namespace.
    pub fn key_scope<T: Hash>(&self, scope: &T) -> CanvasKey {
        CanvasKey::from_hash(scope)
    }

    /// Combine a child identifier into a scoped key.
    pub fn child_key<T: Hash>(&self, parent: CanvasKey, child: &T) -> CanvasKey {
        parent.combine_hash(child)
    }

    pub fn scene(&mut self) -> &mut Scene {
        &mut self.scene
    }

    pub fn scene_fragment<T>(
        &self,
        payload: T,
        local_bounds: Rect,
        scene_origin: Point,
    ) -> CanvasSceneFragment<T> {
        let chunk = SceneChunk::from_scene(&self.scene);
        let hosted_resources = CanvasHostedResources::from_scene_ops(chunk.ops());
        CanvasSceneFragment::from_chunk(
            payload,
            chunk,
            hosted_resources,
            local_bounds,
            scene_origin,
        )
    }

    /// Access UI services and the scratch scene backing this prepaint fragment.
    pub fn services_and_scene(&mut self) -> (&mut dyn fret_core::UiServices, &mut Scene) {
        let services = self.host.services();
        (services, &mut self.scene)
    }

    /// Draw a cached text blob into the scratch scene and return its hosted `TextBlobId`.
    #[allow(clippy::too_many_arguments)]
    pub fn text_with_blob(
        &mut self,
        key: u64,
        order: DrawOrder,
        origin: Point,
        text: impl Into<Arc<str>>,
        style: TextStyle,
        color: Color,
        constraints: CanvasTextConstraints,
        raster_scale_factor: f32,
    ) -> (fret_core::TextBlobId, TextMetrics) {
        let text = text.into();
        let font_stack_key = self.host.text_font_stack_key();
        let services = self.host.services();
        let draw = self.cache.text_draw(
            services,
            key,
            order,
            origin,
            HostedTextContent::Plain(text),
            style,
            color,
            constraints,
            raster_scale_factor,
            font_stack_key,
            &mut self.scene,
        );
        (draw.blob, draw.metrics)
    }

    /// Prepare a cached text blob without emitting a `SceneOp::Text`.
    ///
    /// Use this when the final draw origin depends on the prepared metrics for the current frame.
    #[allow(clippy::too_many_arguments)]
    pub fn prepare_text_with_blob(
        &mut self,
        key: u64,
        text: impl Into<Arc<str>>,
        style: TextStyle,
        constraints: CanvasTextConstraints,
        raster_scale_factor: f32,
    ) -> (fret_core::TextBlobId, TextMetrics) {
        let text = text.into();
        let font_stack_key = self.host.text_font_stack_key();
        let services = self.host.services();
        let draw = self.cache.text_prepare(
            services,
            key,
            HostedTextContent::Plain(text),
            style,
            constraints,
            raster_scale_factor,
            font_stack_key,
        );
        (draw.blob, draw.metrics)
    }

    /// Draw cached rich text into the scratch scene and return its hosted `TextBlobId`.
    #[allow(clippy::too_many_arguments)]
    pub fn rich_text_with_blob(
        &mut self,
        key: u64,
        order: DrawOrder,
        origin: Point,
        rich: AttributedText,
        base_style: TextStyle,
        color: Color,
        constraints: CanvasTextConstraints,
        raster_scale_factor: f32,
    ) -> (fret_core::TextBlobId, TextMetrics) {
        let font_stack_key = self.host.text_font_stack_key();
        let services = self.host.services();
        let draw = self.cache.text_draw(
            services,
            key,
            order,
            origin,
            HostedTextContent::Rich(rich),
            base_style,
            color,
            constraints,
            raster_scale_factor,
            font_stack_key,
            &mut self.scene,
        );
        (draw.blob, draw.metrics)
    }
}

/// Object-safe paint surface for declarative canvas paint handlers.
///
/// This mirrors the "action hook host" pattern (ADR 0074): we cannot store closures that depend on
/// `H: UiHost` because `UiHost` is not object-safe.
pub(crate) trait UiCanvasHost {
    fn bounds(&self) -> Rect;
    fn scale_factor(&self) -> f32;
    fn text_font_stack_key(&mut self) -> u64;
    fn inherited_foreground(&self) -> Option<Color>;
    fn inherited_text_style(&mut self) -> Option<fret_core::TextStyleRefinement>;

    fn theme(&mut self) -> &Theme;
    fn request_redraw(&mut self);
    fn request_animation_frame(&mut self);
    fn request_animation_frame_paint_only(&mut self);

    fn observe_model_id(&mut self, model: ModelId, invalidation: Invalidation);
    fn observe_global(&mut self, global: TypeId, invalidation: Invalidation);

    fn scene(&mut self) -> &mut Scene;
    fn services_and_scene(&mut self) -> (&mut dyn fret_core::UiServices, &mut Scene);

    fn prepaint_output_any(&self, ty: TypeId) -> Option<&dyn Any>;
    fn prepaint_output_any_mut(&mut self, ty: TypeId) -> Option<&mut dyn Any>;
    fn scene_fragment_any(&self, ty: TypeId) -> Option<&dyn Any>;
    fn scene_fragment_any_mut(&mut self, ty: TypeId) -> Option<&mut dyn Any>;
    fn record_scene_fragment_used_entries(&mut self, count: usize);
    fn record_scene_fragment_rejected_entries(&mut self, count: usize, reason: &'static str);
}

pub(crate) struct UiCanvasHostAdapter<'a, 'b, H: UiHost> {
    cx: &'a mut PaintCx<'b, H>,
}

impl<'a, 'b, H: UiHost> UiCanvasHostAdapter<'a, 'b, H> {
    pub(crate) fn new(cx: &'a mut PaintCx<'b, H>) -> Self {
        Self { cx }
    }
}

impl<'a, 'b, H: UiHost> UiCanvasHost for UiCanvasHostAdapter<'a, 'b, H> {
    fn bounds(&self) -> Rect {
        self.cx.bounds
    }

    fn scale_factor(&self) -> f32 {
        self.cx.scale_factor
    }

    fn text_font_stack_key(&mut self) -> u64 {
        self.cx
            .observe_global::<fret_runtime::TextFontStackKey>(Invalidation::Layout);
        self.cx
            .app
            .global::<fret_runtime::TextFontStackKey>()
            .map(|k| k.0)
            .unwrap_or(0)
    }

    fn inherited_foreground(&self) -> Option<Color> {
        self.cx.inherited_foreground()
    }

    fn inherited_text_style(&mut self) -> Option<fret_core::TextStyleRefinement> {
        let Some(window) = self.cx.window else {
            return None;
        };
        crate::declarative::frame::inherited_text_style_for_node(self.cx.app, window, self.cx.node)
    }

    fn theme(&mut self) -> &Theme {
        self.cx.theme()
    }

    fn request_redraw(&mut self) {
        self.cx.request_redraw();
    }

    fn request_animation_frame(&mut self) {
        self.cx.request_animation_frame();
    }

    fn request_animation_frame_paint_only(&mut self) {
        self.cx.request_animation_frame_paint_only();
    }

    fn observe_model_id(&mut self, model: ModelId, invalidation: Invalidation) {
        (self.cx.observe_model)(model, invalidation);
    }

    fn observe_global(&mut self, global: TypeId, invalidation: Invalidation) {
        (self.cx.observe_global)(global, invalidation);
    }

    fn scene(&mut self) -> &mut Scene {
        self.cx.scene
    }

    fn services_and_scene(&mut self) -> (&mut dyn fret_core::UiServices, &mut Scene) {
        (self.cx.services, self.cx.scene)
    }

    fn prepaint_output_any(&self, ty: TypeId) -> Option<&dyn Any> {
        self.cx.tree.prepaint_output_any(self.cx.node, ty)
    }

    fn prepaint_output_any_mut(&mut self, ty: TypeId) -> Option<&mut dyn Any> {
        self.cx.tree.prepaint_output_any_mut(self.cx.node, ty)
    }

    fn scene_fragment_any(&self, ty: TypeId) -> Option<&dyn Any> {
        self.cx.tree.scene_fragment_any(self.cx.node, ty)
    }

    fn scene_fragment_any_mut(&mut self, ty: TypeId) -> Option<&mut dyn Any> {
        self.cx.tree.scene_fragment_any_mut(self.cx.node, ty)
    }

    fn record_scene_fragment_used_entries(&mut self, count: usize) {
        self.cx
            .tree
            .record_scene_fragment_used_entries(self.cx.node, count);
    }

    fn record_scene_fragment_rejected_entries(&mut self, count: usize, reason: &'static str) {
        self.cx
            .tree
            .record_scene_fragment_rejected_entries(self.cx.node, count, reason);
    }
}

pub struct CanvasPainter<'a> {
    host: &'a mut dyn UiCanvasHost,
    cache: &'a mut CanvasCache,
}

impl<'a> CanvasPainter<'a> {
    pub(crate) fn new(host: &'a mut dyn UiCanvasHost, cache: &'a mut CanvasCache) -> Self {
        Self { host, cache }
    }

    /// Current runner-owned frame id for this paint pass.
    pub fn frame_id(&self) -> u64 {
        self.cache.frame
    }

    pub fn bounds(&self) -> Rect {
        self.host.bounds()
    }

    pub fn scale_factor(&self) -> f32 {
        self.host.scale_factor()
    }

    pub fn theme(&mut self) -> &Theme {
        self.host.theme()
    }

    pub fn inherited_foreground(&self) -> Option<Color> {
        self.host.inherited_foreground()
    }

    pub fn inherited_text_style(&mut self) -> Option<fret_core::TextStyleRefinement> {
        self.host.inherited_text_style()
    }

    pub fn resolved_passive_text_style(&mut self, explicit: Option<TextStyle>) -> TextStyle {
        let theme = self.host.theme().snapshot();
        let inherited = self.host.inherited_text_style();
        crate::text_props::resolve_text_style(theme, explicit, inherited.as_ref())
    }

    pub fn request_redraw(&mut self) {
        self.host.request_redraw();
    }

    pub fn request_animation_frame(&mut self) {
        self.host.request_animation_frame();
    }

    /// Request the next animation frame without forcing the nearest view-cache root to rerender.
    ///
    /// Use this only when the canvas paint closure can advance the visual state from retained
    /// paint-time data. If animation state is computed during declarative rendering, use
    /// [`Self::request_animation_frame`] so view-cache roots are rerendered.
    pub fn request_animation_frame_paint_only(&mut self) {
        self.host.request_animation_frame_paint_only();
    }

    pub fn observe_model_id(&mut self, model: ModelId, invalidation: Invalidation) {
        self.host.observe_model_id(model, invalidation);
    }

    pub fn observe_global<T: std::any::Any>(&mut self, invalidation: Invalidation) {
        self.host.observe_global(TypeId::of::<T>(), invalidation);
    }

    /// Compute a deterministic `u64` key for `value`.
    pub fn key<T: Hash>(&self, value: &T) -> u64 {
        CanvasKey::from_hash(value).0
    }

    /// Create a deterministic base key for a logical key "namespace".
    ///
    /// Use this to avoid accidental key collisions across unrelated subsystems.
    pub fn key_scope<T: Hash>(&self, scope: &T) -> CanvasKey {
        CanvasKey::from_hash(scope)
    }

    /// Combine a child identifier into a scoped key.
    pub fn child_key<T: Hash>(&self, parent: CanvasKey, child: &T) -> CanvasKey {
        parent.combine_hash(child)
    }

    pub fn scene(&mut self) -> &mut Scene {
        self.host.scene()
    }

    /// Touch hosted resources referenced by retained scene ops before replaying them.
    ///
    /// Replay caches store `SceneOp`s, not resource ownership. Calling this on cache-hit paths
    /// keeps canvas-owned `TextBlobId`/`PathId`/`SvgId` entries alive so end-of-paint pruning does
    /// not release resources still referenced by replayed ops.
    pub fn touch_hosted_resources_in_scene_ops(
        &mut self,
        ops: &[SceneOp],
    ) -> CanvasHostedResourceTouchCounts {
        self.cache.touch_hosted_resources_in_scene_ops(ops)
    }

    /// Touch hosted resources that were precomputed from retained scene ops.
    pub fn touch_hosted_resources(
        &mut self,
        resources: &CanvasHostedResources,
    ) -> CanvasHostedResourceTouchCounts {
        self.cache.touch_hosted_resources(resources)
    }

    /// Access the underlying UI services and scene for advanced canvas paint handlers.
    ///
    /// This is primarily intended for diagnostics/profiling overlays and experimental paint
    /// surfaces that need text geometry queries (selection rects, hit-testing, etc.).
    pub fn services_and_scene(&mut self) -> (&mut dyn fret_core::UiServices, &mut Scene) {
        self.host.services_and_scene()
    }

    pub fn prepaint_output<T: Any>(&self) -> Option<&T> {
        self.host
            .prepaint_output_any(TypeId::of::<T>())
            .and_then(|value| value.downcast_ref::<T>())
    }

    pub fn prepaint_output_mut<T: Any>(&mut self) -> Option<&mut T> {
        self.host
            .prepaint_output_any_mut(TypeId::of::<T>())
            .and_then(|value| value.downcast_mut::<T>())
    }

    pub fn scene_fragment<T: Any>(&self) -> Option<&T> {
        self.host
            .scene_fragment_any(TypeId::of::<T>())
            .and_then(|value| value.downcast_ref::<T>())
    }

    pub fn scene_fragment_mut<T: Any>(&mut self) -> Option<&mut T> {
        self.host
            .scene_fragment_any_mut(TypeId::of::<T>())
            .and_then(|value| value.downcast_mut::<T>())
    }

    pub fn record_scene_fragment_used_entries(&mut self, count: usize) {
        self.host.record_scene_fragment_used_entries(count);
    }

    pub fn record_scene_fragment_rejected_entries(&mut self, count: usize, reason: &'static str) {
        self.host
            .record_scene_fragment_rejected_entries(count, reason);
    }

    pub fn with_clip_rect<R>(&mut self, rect: Rect, f: impl FnOnce(&mut Self) -> R) -> R {
        {
            let scene = self.host.scene();
            scene.push(SceneOp::PushClipRect { rect });
        }
        let out = f(self);
        {
            let scene = self.host.scene();
            scene.push(SceneOp::PopClip);
        }
        out
    }

    pub fn with_clip_rrect<R>(
        &mut self,
        rect: Rect,
        corner_radii: Corners,
        f: impl FnOnce(&mut Self) -> R,
    ) -> R {
        {
            let scene = self.host.scene();
            scene.push(SceneOp::PushClipRRect { rect, corner_radii });
        }
        let out = f(self);
        {
            let scene = self.host.scene();
            scene.push(SceneOp::PopClip);
        }
        out
    }

    pub fn with_transform<R>(
        &mut self,
        transform: Transform2D,
        f: impl FnOnce(&mut Self) -> R,
    ) -> R {
        let is_finite = transform.a.is_finite()
            && transform.b.is_finite()
            && transform.c.is_finite()
            && transform.d.is_finite()
            && transform.tx.is_finite()
            && transform.ty.is_finite();

        if !is_finite || transform == Transform2D::IDENTITY {
            return f(self);
        }

        {
            let scene = self.host.scene();
            scene.push(SceneOp::PushTransform { transform });
        }
        let out = f(self);
        {
            let scene = self.host.scene();
            scene.push(SceneOp::PopTransform);
        }
        out
    }

    pub fn with_opacity<R>(&mut self, opacity: f32, f: impl FnOnce(&mut Self) -> R) -> R {
        let opacity = if opacity.is_finite() {
            opacity.clamp(0.0, 1.0)
        } else {
            1.0
        };

        if opacity >= 1.0 {
            return f(self);
        }

        {
            let scene = self.host.scene();
            scene.push(SceneOp::PushOpacity { opacity });
        }
        let out = f(self);
        {
            let scene = self.host.scene();
            scene.push(SceneOp::PopOpacity);
        }
        out
    }

    pub fn with_effect<R>(
        &mut self,
        bounds: Rect,
        mode: EffectMode,
        chain: EffectChain,
        quality: EffectQuality,
        f: impl FnOnce(&mut Self) -> R,
    ) -> R {
        if chain.is_empty() {
            return f(self);
        }

        {
            let scene = self.host.scene();
            scene.push(SceneOp::PushEffect {
                bounds,
                mode,
                chain,
                quality,
            });
        }
        let out = f(self);
        {
            let scene = self.host.scene();
            scene.push(SceneOp::PopEffect);
        }
        out
    }

    /// Draw a cached text blob prepared at `raster_scale_factor`.
    ///
    /// - `key` must be stable across frames for the *same* logical text instance.
    /// - `raster_scale_factor` should usually be `device_scale_factor * zoom`, where zoom is an
    ///   explicit policy decision of the caller (ADR 0141).
    #[allow(clippy::too_many_arguments)]
    pub fn text(
        &mut self,
        key: u64,
        order: DrawOrder,
        origin: Point,
        text: impl Into<Arc<str>>,
        style: TextStyle,
        color: Color,
        constraints: CanvasTextConstraints,
        raster_scale_factor: f32,
    ) -> TextMetrics {
        let text = text.into();
        let font_stack_key = self.host.text_font_stack_key();
        let (services, scene) = self.host.services_and_scene();
        self.cache.text(
            services,
            key,
            order,
            origin,
            HostedTextContent::Plain(text),
            style,
            color,
            constraints,
            raster_scale_factor,
            font_stack_key,
            scene,
        )
    }

    /// Draw a cached text blob keyed by (content, style, constraints).
    ///
    /// This is intended for repeated labels where callers do not have a stable per-instance key.
    /// High-entropy or scroll-driven surfaces (code editors, large virtual lists, etc.) should
    /// prefer `text(...)` with stable keys to avoid churn in the shared cache.
    #[allow(clippy::too_many_arguments)]
    pub fn shared_text(
        &mut self,
        order: DrawOrder,
        origin: Point,
        text: impl Into<Arc<str>>,
        style: TextStyle,
        color: Color,
        constraints: CanvasTextConstraints,
        raster_scale_factor: f32,
    ) -> TextMetrics {
        let text = text.into();
        let font_stack_key = self.host.text_font_stack_key();
        let (services, scene) = self.host.services_and_scene();
        self.cache
            .shared_text_draw(
                services,
                order,
                origin,
                text,
                style,
                color,
                constraints,
                raster_scale_factor,
                font_stack_key,
                scene,
            )
            .metrics
    }

    /// Draw a cached text blob prepared at `raster_scale_factor` and return its `TextBlobId`.
    ///
    /// This is intended for advanced paint handlers that need to query text geometry (caret stops,
    /// selection rects, hit-testing, etc.) using the returned blob.
    #[allow(clippy::too_many_arguments)]
    pub fn text_with_blob(
        &mut self,
        key: u64,
        order: DrawOrder,
        origin: Point,
        text: impl Into<Arc<str>>,
        style: TextStyle,
        color: Color,
        constraints: CanvasTextConstraints,
        raster_scale_factor: f32,
    ) -> (fret_core::TextBlobId, TextMetrics) {
        let text = text.into();
        let font_stack_key = self.host.text_font_stack_key();
        let (services, scene) = self.host.services_and_scene();
        let draw = self.cache.text_draw(
            services,
            key,
            order,
            origin,
            HostedTextContent::Plain(text),
            style,
            color,
            constraints,
            raster_scale_factor,
            font_stack_key,
            scene,
        );
        (draw.blob, draw.metrics)
    }

    /// Prepare a cached text blob without emitting a `SceneOp::Text`.
    ///
    /// Use this when the final draw origin depends on the prepared metrics for the current frame.
    #[allow(clippy::too_many_arguments)]
    pub fn prepare_text_with_blob(
        &mut self,
        key: u64,
        text: impl Into<Arc<str>>,
        style: TextStyle,
        constraints: CanvasTextConstraints,
        raster_scale_factor: f32,
    ) -> (fret_core::TextBlobId, TextMetrics) {
        let text = text.into();
        let font_stack_key = self.host.text_font_stack_key();
        let (services, _) = self.host.services_and_scene();
        let draw = self.cache.text_prepare(
            services,
            key,
            HostedTextContent::Plain(text),
            style,
            constraints,
            raster_scale_factor,
            font_stack_key,
        );
        (draw.blob, draw.metrics)
    }

    /// Variant of `shared_text` that returns its prepared `TextBlobId`.
    #[allow(clippy::too_many_arguments)]
    pub fn shared_text_with_blob(
        &mut self,
        order: DrawOrder,
        origin: Point,
        text: impl Into<Arc<str>>,
        style: TextStyle,
        color: Color,
        constraints: CanvasTextConstraints,
        raster_scale_factor: f32,
    ) -> (fret_core::TextBlobId, TextMetrics) {
        let text = text.into();
        let font_stack_key = self.host.text_font_stack_key();
        let (services, scene) = self.host.services_and_scene();
        let draw = self.cache.shared_text_draw(
            services,
            order,
            origin,
            text,
            style,
            color,
            constraints,
            raster_scale_factor,
            font_stack_key,
            scene,
        );
        (draw.blob, draw.metrics)
    }

    #[allow(clippy::too_many_arguments)]
    pub fn rich_text(
        &mut self,
        key: u64,
        order: DrawOrder,
        origin: Point,
        rich: AttributedText,
        base_style: TextStyle,
        color: Color,
        constraints: CanvasTextConstraints,
        raster_scale_factor: f32,
    ) -> TextMetrics {
        let font_stack_key = self.host.text_font_stack_key();
        let (services, scene) = self.host.services_and_scene();
        self.cache.text(
            services,
            key,
            order,
            origin,
            HostedTextContent::Rich(rich),
            base_style,
            color,
            constraints,
            raster_scale_factor,
            font_stack_key,
            scene,
        )
    }

    /// Draw a cached rich text blob prepared at `raster_scale_factor` and return its `TextBlobId`.
    ///
    /// This is intended for advanced paint handlers that need to query text geometry (caret stops,
    /// selection rects, hit-testing, etc.) using the returned blob.
    #[allow(clippy::too_many_arguments)]
    pub fn rich_text_with_blob(
        &mut self,
        key: u64,
        order: DrawOrder,
        origin: Point,
        rich: AttributedText,
        base_style: TextStyle,
        color: Color,
        constraints: CanvasTextConstraints,
        raster_scale_factor: f32,
    ) -> (fret_core::TextBlobId, TextMetrics) {
        let font_stack_key = self.host.text_font_stack_key();
        let (services, scene) = self.host.services_and_scene();
        let draw = self.cache.text_draw(
            services,
            key,
            order,
            origin,
            HostedTextContent::Rich(rich),
            base_style,
            color,
            constraints,
            raster_scale_factor,
            font_stack_key,
            scene,
        );
        (draw.blob, draw.metrics)
    }

    /// Draw a cached tessellated path prepared at `raster_scale_factor`.
    ///
    /// - `key` must be stable across frames for the *same* logical path instance.
    /// - `raster_scale_factor` should usually be `device_scale_factor * zoom`, where zoom is an
    ///   explicit policy decision of the caller (ADR 0141).
    #[allow(clippy::too_many_arguments)]
    pub fn path(
        &mut self,
        key: u64,
        order: DrawOrder,
        origin: Point,
        commands: &[PathCommand],
        style: PathStyle,
        color: Color,
        raster_scale_factor: f32,
    ) -> PathMetrics {
        let (services, scene) = self.host.services_and_scene();
        self.cache.path(
            services,
            key,
            order,
            origin,
            commands,
            style,
            color.into(),
            raster_scale_factor,
            scene,
        )
    }

    /// Draw a cached tessellated path with an explicit paint binding.
    ///
    /// This is the paint-general form of `path(...)`: geometry caching is keyed by path commands,
    /// style, and scale. Paint binding changes should not force re-tessellation.
    #[allow(clippy::too_many_arguments)]
    pub fn path_paint(
        &mut self,
        key: u64,
        order: DrawOrder,
        origin: Point,
        commands: &[PathCommand],
        style: PathStyle,
        paint: PaintBindingV1,
        raster_scale_factor: f32,
    ) -> PathMetrics {
        let (services, scene) = self.host.services_and_scene();
        self.cache.path(
            services,
            key,
            order,
            origin,
            commands,
            style,
            paint,
            raster_scale_factor,
            scene,
        )
    }

    #[allow(clippy::too_many_arguments)]
    pub fn svg_mask_icon(
        &mut self,
        key: u64,
        order: DrawOrder,
        rect: Rect,
        svg: &SvgSource,
        fit: SvgFit,
        color: Color,
        opacity: f32,
    ) {
        let opacity = opacity.clamp(0.0, 1.0);
        if opacity <= 0.0 || color.a <= 0.0 {
            return;
        }

        let (services, scene) = self.host.services_and_scene();
        let svg_id = self.cache.svg(services, key, svg);
        scene.push(SceneOp::SvgMaskIcon {
            order,
            rect,
            svg: svg_id,
            fit,
            color,
            opacity,
        });
    }

    pub fn svg_image(
        &mut self,
        key: u64,
        order: DrawOrder,
        rect: Rect,
        svg: &SvgSource,
        fit: SvgFit,
        opacity: f32,
    ) {
        let opacity = opacity.clamp(0.0, 1.0);
        if opacity <= 0.0 {
            return;
        }

        let (services, scene) = self.host.services_and_scene();
        let svg_id = self.cache.svg(services, key, svg);
        scene.push(SceneOp::SvgImage {
            order,
            rect,
            svg: svg_id,
            fit,
            opacity,
        });
    }
}

#[derive(Debug, Clone, Copy, PartialEq)]
pub struct CanvasTextConstraints {
    pub max_width: Option<Px>,
    pub wrap: TextWrap,
    pub overflow: TextOverflow,
}

impl Default for CanvasTextConstraints {
    fn default() -> Self {
        Self {
            max_width: None,
            wrap: TextWrap::Word,
            overflow: TextOverflow::Clip,
        }
    }
}

#[derive(Default)]
pub(crate) struct CanvasCache {
    frame: u64,
    policy: CanvasCachePolicy,
    text_by_key: HashMap<CanvasTextCacheKey, HostedTextEntry>,
    text_key_by_blob: HashMap<fret_core::TextBlobId, CanvasTextCacheKey>,
    shared_text_by_fingerprint: HashMap<SharedTextFingerprintKey, SharedTextEntry>,
    shared_text_key_by_blob: HashMap<fret_core::TextBlobId, SharedTextFingerprintKey>,
    path_by_key: HashMap<CanvasPathCacheKey, HostedPathEntry>,
    path_key_by_id: HashMap<fret_core::PathId, CanvasPathCacheKey>,
    svg_by_key: HashMap<CanvasSvgCacheKey, HostedSvgEntry>,
    svg_key_by_id: HashMap<fret_core::SvgId, CanvasSvgCacheKey>,
}

impl CanvasCache {
    pub(crate) fn begin_paint(&mut self, frame: u64, policy: CanvasCachePolicy) {
        self.frame = frame;
        self.policy = policy;
    }

    pub(crate) fn end_paint(&mut self, services: &mut dyn fret_core::UiServices) {
        self.evict_hosted_text(services);
        self.evict_hosted_paths(services);
        self.evict_hosted_svgs(services);

        self.evict_shared_text(services);
    }

    pub(crate) fn cleanup_resources(&mut self, services: &mut dyn fret_core::UiServices) {
        for (_, mut entry) in self.text_by_key.drain() {
            if let Some(blob) = entry.blob.take() {
                services.text().release(blob);
            }
        }
        self.text_key_by_blob.clear();
        for (_, entry) in self.shared_text_by_fingerprint.drain() {
            services.text().release(entry.blob);
        }
        self.shared_text_key_by_blob.clear();
        for (_, mut entry) in self.path_by_key.drain() {
            if let Some(path) = entry.path.take() {
                services.path().release(path);
            }
        }
        self.path_key_by_id.clear();
        for (_, mut entry) in self.svg_by_key.drain() {
            if let Some(svg) = entry.svg.take() {
                let _ = services.svg().unregister_svg(svg);
            }
        }
        self.svg_key_by_id.clear();
        self.frame = 0;
    }

    fn touch_hosted_resources_in_scene_ops(
        &mut self,
        ops: &[SceneOp],
    ) -> CanvasHostedResourceTouchCounts {
        let mut counts = CanvasHostedResourceTouchCounts::default();

        for op in ops {
            match *op {
                SceneOp::Text { text, .. } => {
                    if self.touch_text_blob(text) {
                        counts.text_blobs = counts.text_blobs.saturating_add(1);
                    }
                }
                SceneOp::Path { path, .. } | SceneOp::PushClipPath { path, .. } => {
                    if self.touch_path(path) {
                        counts.paths = counts.paths.saturating_add(1);
                    }
                }
                SceneOp::SvgMaskIcon { svg, .. } | SceneOp::SvgImage { svg, .. } => {
                    if self.touch_svg(svg) {
                        counts.svgs = counts.svgs.saturating_add(1);
                    }
                }
                _ => {}
            }
        }

        counts
    }

    fn touch_hosted_resources(
        &mut self,
        resources: &CanvasHostedResources,
    ) -> CanvasHostedResourceTouchCounts {
        let mut counts = CanvasHostedResourceTouchCounts::default();

        for &text_blob in resources.text_blobs.iter() {
            if self.touch_text_blob(text_blob) {
                counts.text_blobs = counts.text_blobs.saturating_add(1);
            }
        }
        for &path in resources.paths.iter() {
            if self.touch_path(path) {
                counts.paths = counts.paths.saturating_add(1);
            }
        }
        for &svg in resources.svgs.iter() {
            if self.touch_svg(svg) {
                counts.svgs = counts.svgs.saturating_add(1);
            }
        }

        counts
    }

    fn touch_text_blob(&mut self, blob: fret_core::TextBlobId) -> bool {
        let mut touched = false;

        if let Some(key) = self.text_key_by_blob.get(&blob).copied() {
            if let Some(entry) = self.text_by_key.get_mut(&key) {
                entry.last_used_frame = self.frame;
                touched = true;
            } else {
                self.text_key_by_blob.remove(&blob);
            }
        }

        if let Some(key) = self.shared_text_key_by_blob.get(&blob).cloned() {
            if let Some(entry) = self.shared_text_by_fingerprint.get_mut(&key) {
                entry.last_used_frame = self.frame;
                touched = true;
            } else {
                self.shared_text_key_by_blob.remove(&blob);
            }
        }

        touched
    }

    fn touch_path(&mut self, path: fret_core::PathId) -> bool {
        let Some(key) = self.path_key_by_id.get(&path).copied() else {
            return false;
        };
        let Some(entry) = self.path_by_key.get_mut(&key) else {
            self.path_key_by_id.remove(&path);
            return false;
        };
        entry.last_used_frame = self.frame;
        true
    }

    fn touch_svg(&mut self, svg: fret_core::SvgId) -> bool {
        let Some(key) = self.svg_key_by_id.get(&svg).copied() else {
            return false;
        };
        let Some(entry) = self.svg_by_key.get_mut(&key) else {
            self.svg_key_by_id.remove(&svg);
            return false;
        };
        entry.last_used_frame = self.frame;
        true
    }

    fn evict_shared_text(&mut self, services: &mut dyn fret_core::UiServices) {
        let now = self.frame;
        let keep_frames = self.policy.shared_text.keep_frames;
        let max_entries = self.policy.shared_text.max_entries;

        if self.shared_text_by_fingerprint.is_empty() {
            return;
        }

        if max_entries == 0 {
            for (_, entry) in self.shared_text_by_fingerprint.drain() {
                services.text().release(entry.blob);
            }
            self.shared_text_key_by_blob.clear();
            return;
        }

        let mut to_remove: Vec<SharedTextFingerprintKey> = Vec::new();
        for (key, entry) in &self.shared_text_by_fingerprint {
            if entry.last_used_frame == now {
                continue;
            }
            if now.saturating_sub(entry.last_used_frame) > keep_frames {
                to_remove.push(key.clone());
            }
        }

        for key in to_remove {
            if let Some(entry) = self.shared_text_by_fingerprint.remove(&key) {
                self.shared_text_key_by_blob.remove(&entry.blob);
                services.text().release(entry.blob);
            }
        }

        if self.shared_text_by_fingerprint.len() <= max_entries {
            return;
        }

        let mut candidates: Vec<(u64, SharedTextFingerprintKey)> = self
            .shared_text_by_fingerprint
            .iter()
            .filter_map(|(key, entry)| {
                if entry.last_used_frame == now {
                    None
                } else {
                    Some((entry.last_used_frame, key.clone()))
                }
            })
            .collect();
        candidates.sort_by_key(|(last_used, _)| *last_used);

        let mut idx = 0usize;
        while self.shared_text_by_fingerprint.len() > max_entries && idx < candidates.len() {
            let key = candidates[idx].1.clone();
            if let Some(entry) = self.shared_text_by_fingerprint.remove(&key) {
                self.shared_text_key_by_blob.remove(&entry.blob);
                services.text().release(entry.blob);
            }
            idx += 1;
        }
    }

    fn evict_hosted_text(&mut self, services: &mut dyn fret_core::UiServices) {
        let now = self.frame;
        let keep_frames = self.policy.text.keep_frames;
        let max_entries = self.policy.text.max_entries;

        let mut expired: Vec<CanvasTextCacheKey> = Vec::new();
        for (key, entry) in &self.text_by_key {
            if now.saturating_sub(entry.last_used_frame) > keep_frames {
                expired.push(*key);
            }
        }
        for key in expired {
            if let Some(mut entry) = self.text_by_key.remove(&key)
                && let Some(blob) = entry.blob.take()
            {
                self.text_key_by_blob.remove(&blob);
                services.text().release(blob);
            }
        }

        if max_entries == 0 {
            for (_, mut entry) in self.text_by_key.drain() {
                if let Some(blob) = entry.blob.take() {
                    services.text().release(blob);
                }
            }
            self.text_key_by_blob.clear();
            return;
        }

        let over = self.text_by_key.len().saturating_sub(max_entries);
        if over == 0 {
            return;
        }

        let mut candidates: Vec<(u64, CanvasTextCacheKey)> = self
            .text_by_key
            .iter()
            .map(|(k, v)| (v.last_used_frame, *k))
            .collect();
        candidates.sort_by_key(|(last, _)| *last);

        for (_, key) in candidates.into_iter().take(over) {
            if let Some(mut entry) = self.text_by_key.remove(&key)
                && let Some(blob) = entry.blob.take()
            {
                self.text_key_by_blob.remove(&blob);
                services.text().release(blob);
            }
        }
    }

    fn evict_hosted_paths(&mut self, services: &mut dyn fret_core::UiServices) {
        let now = self.frame;
        let keep_frames = self.policy.path.keep_frames;
        let max_entries = self.policy.path.max_entries;

        let mut expired: Vec<CanvasPathCacheKey> = Vec::new();
        for (key, entry) in &self.path_by_key {
            if now.saturating_sub(entry.last_used_frame) > keep_frames {
                expired.push(*key);
            }
        }
        for key in expired {
            if let Some(mut entry) = self.path_by_key.remove(&key)
                && let Some(path) = entry.path.take()
            {
                self.path_key_by_id.remove(&path);
                services.path().release(path);
            }
        }

        if max_entries == 0 {
            for (_, mut entry) in self.path_by_key.drain() {
                if let Some(path) = entry.path.take() {
                    services.path().release(path);
                }
            }
            self.path_key_by_id.clear();
            return;
        }

        let over = self.path_by_key.len().saturating_sub(max_entries);
        if over == 0 {
            return;
        }

        let mut candidates: Vec<(u64, CanvasPathCacheKey)> = self
            .path_by_key
            .iter()
            .map(|(k, v)| (v.last_used_frame, *k))
            .collect();
        candidates.sort_by_key(|(last, _)| *last);

        for (_, key) in candidates.into_iter().take(over) {
            if let Some(mut entry) = self.path_by_key.remove(&key)
                && let Some(path) = entry.path.take()
            {
                self.path_key_by_id.remove(&path);
                services.path().release(path);
            }
        }
    }

    fn evict_hosted_svgs(&mut self, services: &mut dyn fret_core::UiServices) {
        let now = self.frame;
        let keep_frames = self.policy.svg.keep_frames;
        let max_entries = self.policy.svg.max_entries;

        let mut expired: Vec<CanvasSvgCacheKey> = Vec::new();
        for (key, entry) in &self.svg_by_key {
            if now.saturating_sub(entry.last_used_frame) > keep_frames {
                expired.push(*key);
            }
        }
        for key in expired {
            if let Some(mut entry) = self.svg_by_key.remove(&key)
                && let Some(svg) = entry.svg.take()
            {
                let _ = services.svg().unregister_svg(svg);
            }
        }

        if max_entries == 0 {
            for (_, mut entry) in self.svg_by_key.drain() {
                if let Some(svg) = entry.svg.take() {
                    let _ = services.svg().unregister_svg(svg);
                }
            }
            self.svg_key_by_id.clear();
            return;
        }

        let over = self.svg_by_key.len().saturating_sub(max_entries);
        if over == 0 {
            return;
        }

        let mut candidates: Vec<(u64, CanvasSvgCacheKey)> = self
            .svg_by_key
            .iter()
            .map(|(k, v)| (v.last_used_frame, *k))
            .collect();
        candidates.sort_by_key(|(last, _)| *last);

        for (_, key) in candidates.into_iter().take(over) {
            if let Some(mut entry) = self.svg_by_key.remove(&key)
                && let Some(svg) = entry.svg.take()
            {
                self.svg_key_by_id.remove(&svg);
                let _ = services.svg().unregister_svg(svg);
            }
        }
    }

    #[allow(clippy::too_many_arguments)]
    fn text(
        &mut self,
        services: &mut dyn fret_core::UiServices,
        key: u64,
        order: DrawOrder,
        origin: Point,
        content: HostedTextContent,
        style: TextStyle,
        color: Color,
        constraints: CanvasTextConstraints,
        raster_scale_factor: f32,
        font_stack_key: u64,
        scene: &mut Scene,
    ) -> TextMetrics {
        self.text_draw(
            services,
            key,
            order,
            origin,
            content,
            style,
            color,
            constraints,
            raster_scale_factor,
            font_stack_key,
            scene,
        )
        .metrics
    }

    #[allow(clippy::too_many_arguments)]
    fn shared_text_draw(
        &mut self,
        services: &mut dyn fret_core::UiServices,
        order: DrawOrder,
        origin: Point,
        text: Arc<str>,
        style: TextStyle,
        color: Color,
        constraints: CanvasTextConstraints,
        raster_scale_factor: f32,
        font_stack_key: u64,
        scene: &mut Scene,
    ) -> TextDraw {
        let raster_scale_factor = normalize_scale_factor(raster_scale_factor);
        let scale_bits = raster_scale_factor.to_bits();

        if self.policy.shared_text.max_entries == 0 {
            let text_constraints = TextConstraints {
                max_width: constraints.max_width,
                wrap: constraints.wrap,
                overflow: constraints.overflow,
                align: fret_core::TextAlign::Start,
                scale_factor: raster_scale_factor,
            };

            let (blob, metrics) =
                services
                    .text()
                    .prepare_str(text.as_ref(), &style, text_constraints);
            scene.push(SceneOp::Text {
                order,
                origin,
                text: blob,
                paint: Paint::Solid(color).into(),
                outline: None,
                shadow: None,
            });
            return TextDraw { blob, metrics };
        }

        let shared_key = SharedTextFingerprintKey {
            content: SharedTextContentKey::Plain(Arc::clone(&text)),
            style: TextStyleCacheKey::from_style(&style),
            constraints: CanvasTextConstraintsKey::from_constraints(constraints),
            font_stack_key,
            scale_bits,
        };

        if let Some(entry) = self.shared_text_by_fingerprint.get_mut(&shared_key) {
            entry.last_used_frame = self.frame;
            scene.push(SceneOp::Text {
                order,
                origin,
                text: entry.blob,
                paint: Paint::Solid(color).into(),
                outline: None,
                shadow: None,
            });
            return TextDraw {
                blob: entry.blob,
                metrics: entry.metrics,
            };
        }

        let text_constraints = TextConstraints {
            max_width: constraints.max_width,
            wrap: constraints.wrap,
            overflow: constraints.overflow,
            align: fret_core::TextAlign::Start,
            scale_factor: raster_scale_factor,
        };

        let (blob, metrics) = services
            .text()
            .prepare_str(text.as_ref(), &style, text_constraints);
        if let Some(old) = self.shared_text_by_fingerprint.insert(
            shared_key.clone(),
            SharedTextEntry {
                blob,
                metrics,
                last_used_frame: self.frame,
            },
        ) {
            self.shared_text_key_by_blob.remove(&old.blob);
        }
        self.shared_text_key_by_blob.insert(blob, shared_key);

        scene.push(SceneOp::Text {
            order,
            origin,
            text: blob,
            paint: Paint::Solid(color).into(),
            outline: None,
            shadow: None,
        });
        TextDraw { blob, metrics }
    }

    #[allow(clippy::too_many_arguments)]
    fn text_prepare(
        &mut self,
        services: &mut dyn fret_core::UiServices,
        key: u64,
        content: HostedTextContent,
        style: TextStyle,
        constraints: CanvasTextConstraints,
        raster_scale_factor: f32,
        font_stack_key: u64,
    ) -> TextDraw {
        let raster_scale_factor = normalize_scale_factor(raster_scale_factor);
        let scale_bits = raster_scale_factor.to_bits();

        let fingerprint_constraints = match constraints.wrap {
            TextWrap::None if constraints.overflow != TextOverflow::Ellipsis => {
                CanvasTextConstraints {
                    max_width: None,
                    ..constraints
                }
            }
            _ => constraints,
        };

        let cache_key = CanvasTextCacheKey { key, scale_bits };
        let entry = self.text_by_key.entry(cache_key).or_default();
        entry.last_used_frame = self.frame;

        let fingerprint = HostedTextFingerprint {
            content: content.clone(),
            style: style.clone(),
            constraints: fingerprint_constraints,
            font_stack_key,
            scale_bits,
        };

        let needs_prepare =
            entry.blob.is_none() || entry.fingerprint.as_ref() != Some(&fingerprint);
        if needs_prepare {
            if let Some(blob) = entry.blob.take() {
                self.text_key_by_blob.remove(&blob);
                services.text().release(blob);
            }

            let text_constraints = TextConstraints {
                max_width: fingerprint_constraints.max_width,
                wrap: fingerprint_constraints.wrap,
                overflow: fingerprint_constraints.overflow,
                align: fret_core::TextAlign::Start,
                scale_factor: raster_scale_factor,
            };

            let (blob, metrics) = match &content {
                HostedTextContent::Plain(text) => {
                    services
                        .text()
                        .prepare_str(text.as_ref(), &style, text_constraints)
                }
                HostedTextContent::Rich(rich) => {
                    services.text().prepare_rich(rich, &style, text_constraints)
                }
            };

            entry.blob = Some(blob);
            entry.metrics = Some(metrics);
            entry.fingerprint = Some(fingerprint);
            self.text_key_by_blob.insert(blob, cache_key);
        }

        let Some(blob) = entry.blob else {
            return TextDraw {
                blob: fret_core::TextBlobId::default(),
                metrics: TextMetrics {
                    size: fret_core::Size::new(Px(0.0), Px(0.0)),
                    baseline: Px(0.0),
                },
            };
        };
        let metrics = entry.metrics.unwrap_or(TextMetrics {
            size: fret_core::Size::new(Px(0.0), Px(0.0)),
            baseline: Px(0.0),
        });
        TextDraw { blob, metrics }
    }

    #[allow(clippy::too_many_arguments)]
    fn text_draw(
        &mut self,
        services: &mut dyn fret_core::UiServices,
        key: u64,
        order: DrawOrder,
        origin: Point,
        content: HostedTextContent,
        style: TextStyle,
        color: Color,
        constraints: CanvasTextConstraints,
        raster_scale_factor: f32,
        font_stack_key: u64,
        scene: &mut Scene,
    ) -> TextDraw {
        let draw = self.text_prepare(
            services,
            key,
            content,
            style,
            constraints,
            raster_scale_factor,
            font_stack_key,
        );

        scene.push(SceneOp::Text {
            order,
            origin,
            text: draw.blob,
            paint: Paint::Solid(color).into(),
            outline: None,
            shadow: None,
        });
        draw
    }

    #[allow(clippy::too_many_arguments)]
    fn path(
        &mut self,
        services: &mut dyn fret_core::UiServices,
        key: u64,
        order: DrawOrder,
        origin: Point,
        commands: &[PathCommand],
        style: PathStyle,
        paint: PaintBindingV1,
        raster_scale_factor: f32,
        scene: &mut Scene,
    ) -> PathMetrics {
        let raster_scale_factor = normalize_scale_factor(raster_scale_factor);
        let scale_bits = raster_scale_factor.to_bits();

        let cache_key = CanvasPathCacheKey { key, scale_bits };
        let entry = self.path_by_key.entry(cache_key).or_default();
        entry.last_used_frame = self.frame;

        let fingerprint = HostedPathFingerprint {
            commands_hash: hash_path_commands(commands),
            commands_len: commands.len(),
            style,
            scale_bits,
        };

        let needs_prepare =
            entry.path.is_none() || entry.fingerprint.as_ref() != Some(&fingerprint);
        if needs_prepare {
            if let Some(path) = entry.path.take() {
                self.path_key_by_id.remove(&path);
                services.path().release(path);
            }
            let constraints = PathConstraints {
                scale_factor: raster_scale_factor,
            };
            let (path, metrics) = services.path().prepare(commands, style, constraints);
            entry.path = Some(path);
            entry.metrics = Some(metrics);
            entry.fingerprint = Some(fingerprint);
            self.path_key_by_id.insert(path, cache_key);
        }

        let Some(path) = entry.path else {
            return PathMetrics::default();
        };
        let metrics = entry.metrics.unwrap_or_default();

        scene.push(SceneOp::Path {
            order,
            origin,
            path,
            paint,
        });
        metrics
    }

    fn svg(
        &mut self,
        services: &mut dyn fret_core::UiServices,
        key: u64,
        svg: &SvgSource,
    ) -> fret_core::SvgId {
        match svg {
            SvgSource::Id(id) => *id,
            SvgSource::Static(bytes) => self.svg_bytes(services, key, SvgBytesKey::Static(bytes)),
            SvgSource::Bytes(bytes) => {
                self.svg_bytes(services, key, SvgBytesKey::Bytes(bytes.clone()))
            }
        }
    }

    fn svg_bytes(
        &mut self,
        services: &mut dyn fret_core::UiServices,
        key: u64,
        bytes: SvgBytesKey,
    ) -> fret_core::SvgId {
        let cache_key = CanvasSvgCacheKey { key };
        let entry = self.svg_by_key.entry(cache_key).or_default();
        entry.last_used_frame = self.frame;
        let fingerprint = SvgFingerprint {
            bytes: bytes.fingerprint(),
        };

        let needs_prepare = entry.svg.is_none() || entry.fingerprint.as_ref() != Some(&fingerprint);
        if needs_prepare {
            let svg_id = match &bytes {
                SvgBytesKey::Static(bytes) => services.svg().register_svg(bytes),
                SvgBytesKey::Bytes(bytes) => services.svg().register_svg(bytes),
            };
            if let Some(old) = entry.svg.replace(svg_id) {
                let _ = services.svg().unregister_svg(old);
                self.svg_key_by_id.remove(&old);
            }
            entry.fingerprint = Some(fingerprint);
            self.svg_key_by_id.insert(svg_id, cache_key);
        }

        entry.svg.unwrap_or_default()
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
struct CanvasTextCacheKey {
    key: u64,
    scale_bits: u32,
}

#[derive(Debug, Clone, PartialEq, Eq, Hash)]
struct TextStyleCacheKey {
    font: FontId,
    size_bits: u32,
    weight: FontWeight,
    slant: TextSlant,
    line_height_bits: Option<u32>,
    letter_spacing_em_bits: Option<u32>,
}

impl TextStyleCacheKey {
    fn from_style(style: &TextStyle) -> Self {
        Self {
            font: style.font.clone(),
            size_bits: style.size.0.to_bits(),
            weight: style.weight,
            slant: style.slant,
            line_height_bits: style.line_height.map(|h| h.0.to_bits()),
            letter_spacing_em_bits: style.letter_spacing_em.map(f32::to_bits),
        }
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
struct CanvasTextConstraintsKey {
    max_width_bits: Option<u32>,
    wrap: TextWrap,
    overflow: TextOverflow,
}

impl CanvasTextConstraintsKey {
    fn from_constraints(constraints: CanvasTextConstraints) -> Self {
        let max_width_bits = match constraints.wrap {
            // `TextWrap::None` does not change shaping results based on width unless we need to
            // materialize an overflow policy (ellipsis). Callers clip at higher levels.
            TextWrap::None if constraints.overflow != TextOverflow::Ellipsis => None,
            _ => constraints.max_width.map(|w| w.0.to_bits()),
        };
        Self {
            max_width_bits,
            wrap: constraints.wrap,
            overflow: constraints.overflow,
        }
    }
}

#[derive(Debug, Clone, PartialEq, Eq, Hash)]
enum SharedTextContentKey {
    Plain(Arc<str>),
}

#[derive(Debug, Clone, PartialEq, Eq, Hash)]
struct SharedTextFingerprintKey {
    content: SharedTextContentKey,
    style: TextStyleCacheKey,
    constraints: CanvasTextConstraintsKey,
    font_stack_key: u64,
    scale_bits: u32,
}

#[derive(Debug, Clone, Copy)]
struct SharedTextEntry {
    blob: fret_core::TextBlobId,
    metrics: TextMetrics,
    last_used_frame: u64,
}

#[derive(Debug, Clone, Copy)]
struct TextDraw {
    blob: fret_core::TextBlobId,
    metrics: TextMetrics,
}

#[derive(Debug, Clone)]
enum HostedTextContent {
    Plain(Arc<str>),
    Rich(AttributedText),
}

impl PartialEq for HostedTextContent {
    fn eq(&self, other: &Self) -> bool {
        match (self, other) {
            (Self::Plain(a), Self::Plain(b)) => Arc::ptr_eq(a, b) || a.as_ref() == b.as_ref(),
            (Self::Rich(a), Self::Rich(b)) => {
                (Arc::ptr_eq(&a.text, &b.text) && Arc::ptr_eq(&a.spans, &b.spans)) || a == b
            }
            _ => false,
        }
    }
}

#[derive(Debug, Clone, PartialEq)]
struct HostedTextFingerprint {
    content: HostedTextContent,
    style: TextStyle,
    constraints: CanvasTextConstraints,
    font_stack_key: u64,
    scale_bits: u32,
}

#[derive(Default)]
struct HostedTextEntry {
    blob: Option<fret_core::TextBlobId>,
    metrics: Option<TextMetrics>,
    fingerprint: Option<HostedTextFingerprint>,
    last_used_frame: u64,
}

#[derive(Debug, Clone, Copy, PartialEq)]
struct HostedPathFingerprint {
    commands_hash: u64,
    commands_len: usize,
    style: PathStyle,
    scale_bits: u32,
}

#[derive(Default)]
struct HostedPathEntry {
    path: Option<fret_core::PathId>,
    metrics: Option<PathMetrics>,
    fingerprint: Option<HostedPathFingerprint>,
    last_used_frame: u64,
}

#[derive(Default)]
struct HostedSvgEntry {
    svg: Option<fret_core::SvgId>,
    fingerprint: Option<SvgFingerprint>,
    last_used_frame: u64,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
struct SvgFingerprint {
    bytes: SvgBytesFingerprint,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
struct CanvasSvgCacheKey {
    key: u64,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
struct CanvasPathCacheKey {
    key: u64,
    scale_bits: u32,
}

fn normalize_scale_factor(scale_factor: f32) -> f32 {
    if !scale_factor.is_finite() || scale_factor <= 0.0 {
        1.0
    } else {
        scale_factor
    }
}

fn hash_path_commands(commands: &[PathCommand]) -> u64 {
    let mut state = 0u64;
    for cmd in commands {
        match *cmd {
            PathCommand::MoveTo(p) => {
                state = mix_u64(state, 1);
                state = mix_point(state, p);
            }
            PathCommand::LineTo(p) => {
                state = mix_u64(state, 2);
                state = mix_point(state, p);
            }
            PathCommand::QuadTo { ctrl, to } => {
                state = mix_u64(state, 3);
                state = mix_point(state, ctrl);
                state = mix_point(state, to);
            }
            PathCommand::CubicTo { ctrl1, ctrl2, to } => {
                state = mix_u64(state, 4);
                state = mix_point(state, ctrl1);
                state = mix_point(state, ctrl2);
                state = mix_point(state, to);
            }
            PathCommand::Close => {
                state = mix_u64(state, 5);
            }
        }
    }
    state
}

fn mix_u64(mut state: u64, value: u64) -> u64 {
    // Keep mixing deterministic and reasonably avalanche-y (not cryptographic).
    state ^= value.wrapping_add(0x9E37_79B9_7F4A_7C15);
    state = state.rotate_left(7);
    state = state.wrapping_mul(0xD6E8_FEB8_6659_FD93);
    state
}

fn mix_f32(state: u64, value: f32) -> u64 {
    mix_u64(state, u64::from(value.to_bits()))
}

fn mix_px(state: u64, value: fret_core::Px) -> u64 {
    mix_f32(state, value.0)
}

fn mix_point(mut state: u64, p: fret_core::Point) -> u64 {
    state = mix_px(state, p.x);
    state = mix_px(state, p.y);
    state
}

#[derive(Clone)]
enum SvgBytesKey {
    Static(&'static [u8]),
    Bytes(Arc<[u8]>),
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
enum SvgBytesFingerprint {
    Static { ptr: usize, len: usize },
    Bytes { ptr: usize, len: usize },
}

impl SvgBytesKey {
    fn fingerprint(&self) -> SvgBytesFingerprint {
        match self {
            SvgBytesKey::Static(bytes) => SvgBytesFingerprint::Static {
                ptr: bytes.as_ptr() as usize,
                len: bytes.len(),
            },
            SvgBytesKey::Bytes(bytes) => SvgBytesFingerprint::Bytes {
                ptr: bytes.as_ptr() as usize,
                len: bytes.len(),
            },
        }
    }
}

#[derive(Default)]
struct Fnv1a64(u64);

impl Hasher for Fnv1a64 {
    fn finish(&self) -> u64 {
        self.0
    }

    fn write(&mut self, bytes: &[u8]) {
        let mut hash = if self.0 == 0 {
            0xcbf29ce484222325
        } else {
            self.0
        };
        for b in bytes {
            hash ^= *b as u64;
            hash = hash.wrapping_mul(0x100000001b3);
        }
        self.0 = hash;
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use slotmap::KeyData;

    fn text_blob_id(raw: u64) -> fret_core::TextBlobId {
        fret_core::TextBlobId::from(KeyData::from_ffi(raw))
    }

    fn path_id(raw: u64) -> fret_core::PathId {
        fret_core::PathId::from(KeyData::from_ffi(raw))
    }

    fn svg_id(raw: u64) -> fret_core::SvgId {
        fret_core::SvgId::from(KeyData::from_ffi(raw))
    }

    #[test]
    fn hosted_resources_from_scene_ops_collects_resource_ids() {
        let text = text_blob_id(1);
        let clip_path = path_id(2);
        let path = path_id(3);
        let mask_svg = svg_id(4);
        let image_svg = svg_id(5);

        let resources = CanvasHostedResources::from_scene_ops(&[
            SceneOp::PushClipPath {
                bounds: Rect::new(
                    Point::new(Px(0.0), Px(0.0)),
                    fret_core::Size::new(Px(1.0), Px(1.0)),
                ),
                origin: Point::new(Px(0.0), Px(0.0)),
                path: clip_path,
            },
            SceneOp::Path {
                order: DrawOrder(0),
                origin: Point::new(Px(0.0), Px(0.0)),
                path,
                paint: Paint::Solid(Color::TRANSPARENT).into(),
            },
            SceneOp::SvgMaskIcon {
                order: DrawOrder(0),
                rect: Rect::new(
                    Point::new(Px(0.0), Px(0.0)),
                    fret_core::Size::new(Px(1.0), Px(1.0)),
                ),
                svg: mask_svg,
                fit: SvgFit::Contain,
                color: Color::TRANSPARENT,
                opacity: 1.0,
            },
            SceneOp::SvgImage {
                order: DrawOrder(0),
                rect: Rect::new(
                    Point::new(Px(0.0), Px(0.0)),
                    fret_core::Size::new(Px(1.0), Px(1.0)),
                ),
                svg: image_svg,
                fit: SvgFit::Contain,
                opacity: 1.0,
            },
            SceneOp::Text {
                order: DrawOrder(0),
                origin: Point::new(Px(0.0), Px(0.0)),
                text,
                paint: Paint::Solid(Color::TRANSPARENT).into(),
                outline: None,
                shadow: None,
            },
            SceneOp::PopClip,
        ]);

        assert_eq!(resources.text_blobs.as_slice(), &[text]);
        assert_eq!(resources.paths.as_slice(), &[clip_path, path]);
        assert_eq!(resources.svgs.as_slice(), &[mask_svg, image_svg]);
    }
}
