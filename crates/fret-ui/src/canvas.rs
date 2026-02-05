use std::any::TypeId;
use std::collections::HashMap;
use std::hash::{Hash, Hasher};
use std::sync::Arc;

use fret_core::{
    AttributedText, Color, Corners, DrawOrder, EffectChain, EffectMode, EffectQuality, FontId,
    FontWeight, Point, Px, Rect, Scene, SceneOp, SvgFit, TextConstraints, TextMetrics,
    TextOverflow, TextSlant, TextStyle, TextWrap, Transform2D,
};
use fret_core::{PathCommand, PathConstraints, PathMetrics, PathStyle};
use fret_runtime::ModelId;

use crate::Theme;
use crate::element::CanvasCachePolicy;
use crate::widget::Invalidation;
use crate::{SvgSource, UiHost, widget::PaintCx};

pub type OnCanvasPaint = Arc<dyn for<'a> Fn(&mut CanvasPainter<'a>) + 'static>;

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
}

/// Object-safe paint surface for declarative canvas paint handlers.
///
/// This mirrors the "action hook host" pattern (ADR 0074): we cannot store closures that depend on
/// `H: UiHost` because `UiHost` is not object-safe.
pub(crate) trait UiCanvasHost {
    fn bounds(&self) -> Rect;
    fn scale_factor(&self) -> f32;
    fn text_font_stack_key(&mut self) -> u64;

    fn theme(&mut self) -> &Theme;
    fn request_redraw(&mut self);
    fn request_animation_frame(&mut self);

    fn observe_model_id(&mut self, model: ModelId, invalidation: Invalidation);
    fn observe_global(&mut self, global: TypeId, invalidation: Invalidation);

    fn scene(&mut self) -> &mut Scene;
    fn services_and_scene(&mut self) -> (&mut dyn fret_core::UiServices, &mut Scene);
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

    fn theme(&mut self) -> &Theme {
        self.cx.theme()
    }

    fn request_redraw(&mut self) {
        self.cx.request_redraw();
    }

    fn request_animation_frame(&mut self) {
        self.cx.request_animation_frame();
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
}

pub struct CanvasPainter<'a> {
    host: &'a mut dyn UiCanvasHost,
    cache: &'a mut CanvasCache,
}

impl<'a> CanvasPainter<'a> {
    pub(crate) fn new(host: &'a mut dyn UiCanvasHost, cache: &'a mut CanvasCache) -> Self {
        Self { host, cache }
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

    pub fn request_redraw(&mut self) {
        self.host.request_redraw();
    }

    pub fn request_animation_frame(&mut self) {
        self.host.request_animation_frame();
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

    /// Access the underlying UI services and scene for advanced canvas paint handlers.
    ///
    /// This is primarily intended for diagnostics/profiling overlays and experimental paint
    /// surfaces that need text geometry queries (selection rects, hit-testing, etc.).
    pub fn services_and_scene(&mut self) -> (&mut dyn fret_core::UiServices, &mut Scene) {
        self.host.services_and_scene()
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
    ///   explicit policy decision of the caller (ADR 0156).
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

    /// Draw a cached text blob prepared at `raster_scale_factor` and return its `TextBlobId`.
    ///
    /// This is intended for advanced paint handlers that need to query text geometry (caret stops,
    /// selection rects, hit-testing, etc.) using the returned blob.
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
    ///   explicit policy decision of the caller (ADR 0156).
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
            color,
            raster_scale_factor,
            scene,
        )
    }

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
    shared_text_by_fingerprint: HashMap<SharedTextFingerprintKey, SharedTextEntry>,
    path_by_key: HashMap<CanvasPathCacheKey, HostedPathEntry>,
    svg_by_key: HashMap<CanvasSvgCacheKey, HostedSvgEntry>,
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
        for (_, entry) in self.shared_text_by_fingerprint.drain() {
            services.text().release(entry.blob);
        }
        for (_, mut entry) in self.path_by_key.drain() {
            if let Some(path) = entry.path.take() {
                services.path().release(path);
            }
        }
        for (_, mut entry) in self.svg_by_key.drain() {
            if let Some(svg) = entry.svg.take() {
                let _ = services.svg().unregister_svg(svg);
            }
        }
        self.frame = 0;
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
                services.text().release(entry.blob);
            }
            idx += 1;
        }
    }

    fn evict_hosted_text(&mut self, services: &mut dyn fret_core::UiServices) {
        let now = self.frame;
        let keep_frames = self.policy.text.keep_frames;
        let max_entries = self.policy.text.max_entries;

        self.text_by_key.retain(|_, entry| {
            let keep = now.saturating_sub(entry.last_used_frame) <= keep_frames;
            if !keep {
                if let Some(blob) = entry.blob.take() {
                    services.text().release(blob);
                }
            }
            keep
        });

        if max_entries == 0 {
            for (_, mut entry) in self.text_by_key.drain() {
                if let Some(blob) = entry.blob.take() {
                    services.text().release(blob);
                }
            }
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
            if let Some(mut entry) = self.text_by_key.remove(&key) {
                if let Some(blob) = entry.blob.take() {
                    services.text().release(blob);
                }
            }
        }
    }

    fn evict_hosted_paths(&mut self, services: &mut dyn fret_core::UiServices) {
        let now = self.frame;
        let keep_frames = self.policy.path.keep_frames;
        let max_entries = self.policy.path.max_entries;

        self.path_by_key.retain(|_, entry| {
            let keep = now.saturating_sub(entry.last_used_frame) <= keep_frames;
            if !keep {
                if let Some(path) = entry.path.take() {
                    services.path().release(path);
                }
            }
            keep
        });

        if max_entries == 0 {
            for (_, mut entry) in self.path_by_key.drain() {
                if let Some(path) = entry.path.take() {
                    services.path().release(path);
                }
            }
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
            if let Some(mut entry) = self.path_by_key.remove(&key) {
                if let Some(path) = entry.path.take() {
                    services.path().release(path);
                }
            }
        }
    }

    fn evict_hosted_svgs(&mut self, services: &mut dyn fret_core::UiServices) {
        let now = self.frame;
        let keep_frames = self.policy.svg.keep_frames;
        let max_entries = self.policy.svg.max_entries;

        self.svg_by_key.retain(|_, entry| {
            let keep = now.saturating_sub(entry.last_used_frame) <= keep_frames;
            if !keep {
                if let Some(svg) = entry.svg.take() {
                    let _ = services.svg().unregister_svg(svg);
                }
            }
            keep
        });

        if max_entries == 0 {
            for (_, mut entry) in self.svg_by_key.drain() {
                if let Some(svg) = entry.svg.take() {
                    let _ = services.svg().unregister_svg(svg);
                }
            }
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
        let raster_scale_factor = normalize_scale_factor(raster_scale_factor);
        let scale_bits = raster_scale_factor.to_bits();

        if let HostedTextContent::Plain(text) = &content
            && self.policy.shared_text.max_entries > 0
        {
            let shared_key = SharedTextFingerprintKey {
                content: SharedTextContentKey::Plain(Arc::clone(text)),
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
                    color,
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
                scale_factor: raster_scale_factor,
            };

            let (blob, metrics) =
                services
                    .text()
                    .prepare_str(text.as_ref(), &style, text_constraints);
            self.shared_text_by_fingerprint.insert(
                shared_key,
                SharedTextEntry {
                    blob,
                    metrics,
                    last_used_frame: self.frame,
                },
            );

            scene.push(SceneOp::Text {
                order,
                origin,
                text: blob,
                color,
            });
            return TextDraw { blob, metrics };
        }

        let cache_key = CanvasTextCacheKey { key, scale_bits };
        let entry = self.text_by_key.entry(cache_key).or_default();
        entry.last_used_frame = self.frame;

        let fingerprint = HostedTextFingerprint {
            content: content.clone(),
            style: style.clone(),
            constraints,
            font_stack_key,
            scale_bits,
        };

        let needs_prepare =
            entry.blob.is_none() || entry.fingerprint.as_ref() != Some(&fingerprint);
        if needs_prepare {
            if let Some(blob) = entry.blob.take() {
                services.text().release(blob);
            }

            let text_constraints = TextConstraints {
                max_width: constraints.max_width,
                wrap: constraints.wrap,
                overflow: constraints.overflow,
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

        scene.push(SceneOp::Text {
            order,
            origin,
            text: blob,
            color,
        });
        TextDraw { blob, metrics }
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
        color: Color,
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
                services.path().release(path);
            }
            let constraints = PathConstraints {
                scale_factor: raster_scale_factor,
            };
            let (path, metrics) = services.path().prepare(commands, style, constraints);
            entry.path = Some(path);
            entry.metrics = Some(metrics);
            entry.fingerprint = Some(fingerprint);
        }

        let Some(path) = entry.path else {
            return PathMetrics::default();
        };
        let metrics = entry.metrics.unwrap_or_default();

        scene.push(SceneOp::Path {
            order,
            origin,
            path,
            color,
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
            SvgSource::Static(bytes) => self.svg_bytes(services, key, SvgBytesKey::Static(*bytes)),
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
            }
            entry.fingerprint = Some(fingerprint);
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
        Self {
            max_width_bits: constraints.max_width.map(|w| w.0.to_bits()),
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

#[derive(Debug, Clone, PartialEq)]
enum HostedTextContent {
    Plain(Arc<str>),
    Rich(AttributedText),
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
