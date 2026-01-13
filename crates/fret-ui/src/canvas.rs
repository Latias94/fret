use std::any::TypeId;
use std::collections::{HashMap, HashSet};
use std::sync::Arc;

use fret_core::{
    Color, DrawOrder, Point, Px, Rect, RichText, Scene, SceneOp, TextConstraints, TextMetrics,
    TextOverflow, TextStyle, TextWrap,
};
use fret_runtime::ModelId;

use crate::Theme;
use crate::elements::stable_hash;
use crate::widget::Invalidation;
use crate::{UiHost, widget::PaintCx};

pub type OnCanvasPaint = Arc<dyn for<'a> Fn(&mut CanvasPainter<'a>) + 'static>;

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

    pub fn scene(&mut self) -> &mut Scene {
        self.host.scene()
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

    pub fn rich_text(
        &mut self,
        key: u64,
        order: DrawOrder,
        origin: Point,
        rich: RichText,
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
    text_by_key: HashMap<u64, HostedTextEntry>,
    text_used: HashSet<u64>,
}

impl CanvasCache {
    pub(crate) fn begin_paint(&mut self) {
        self.text_used.clear();
    }

    pub(crate) fn end_paint(&mut self, services: &mut dyn fret_core::UiServices) {
        let mut to_remove: Vec<u64> = Vec::new();
        for (&key, entry) in self.text_by_key.iter() {
            if self.text_used.contains(&key) {
                continue;
            }
            if entry.blob.is_some() {
                to_remove.push(key);
            }
        }

        for key in to_remove {
            if let Some(mut entry) = self.text_by_key.remove(&key) {
                if let Some(blob) = entry.blob.take() {
                    services.text().release(blob);
                }
            }
        }
    }

    pub(crate) fn cleanup_resources(&mut self, services: &mut dyn fret_core::UiServices) {
        for (_, mut entry) in self.text_by_key.drain() {
            if let Some(blob) = entry.blob.take() {
                services.text().release(blob);
            }
        }
        self.text_used.clear();
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
        let raster_scale_factor = normalize_scale_factor(raster_scale_factor);
        let scale_bits = raster_scale_factor.to_bits();

        let cache_key = stable_hash(&("fret_ui.canvas.text", key, scale_bits));
        self.text_used.insert(cache_key);

        let entry = self
            .text_by_key
            .entry(cache_key)
            .or_insert_with(|| HostedTextEntry {
                blob: None,
                metrics: None,
                fingerprint: None,
            });

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
                        .prepare(text.as_ref(), &style, text_constraints)
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
            return TextMetrics {
                size: fret_core::Size::new(Px(0.0), Px(0.0)),
                baseline: Px(0.0),
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
        metrics
    }
}

#[derive(Debug, Clone, PartialEq)]
enum HostedTextContent {
    Plain(Arc<str>),
    Rich(RichText),
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
}

fn normalize_scale_factor(scale_factor: f32) -> f32 {
    if !scale_factor.is_finite() || scale_factor <= 0.0 {
        1.0
    } else {
        scale_factor
    }
}
