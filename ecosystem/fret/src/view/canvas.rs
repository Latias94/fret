//! App-facing canvas helpers.

use std::hash::Hash;
use std::sync::Arc;

use fret_canvas::ui::{PanZoomCanvasPaintCx, PanZoomInputPreset};
use fret_canvas::view::PanZoom2D;
use fret_core::scene::{Paint, PaintBindingV1};
use fret_core::{
    Color, Corners, DrawOrder, Edges, PathCommand, PathMetrics, PathStyle, Point, Px, Rect,
    SceneOp, Transform2D,
};
use fret_ui::UiHost;
use fret_ui::canvas::CanvasKey;
use fret_ui::element::{CanvasCachePolicy, CanvasProps, Length, SemanticsDecoration};
use fret_ui_kit::IntoUiElement as _;

use super::{
    AppUi, LocalState, LocalStateRawModelExt as _, MouseButton, PointerActionCx, PointerDown,
    PointerMove, PointerUp,
};

/// App-facing painter for declarative canvas paint callbacks.
///
/// This keeps default app examples away from `fret_ui::canvas::CanvasPainter` while still exposing
/// the small drawing vocabulary custom app canvases need.
pub struct AppCanvasPainter<'paint, 'raw> {
    raw: &'paint mut fret_ui::canvas::CanvasPainter<'raw>,
}

impl<'paint, 'raw> AppCanvasPainter<'paint, 'raw> {
    pub fn bounds(&self) -> Rect {
        self.raw.bounds()
    }

    pub fn scale_factor(&self) -> f32 {
        self.raw.scale_factor()
    }

    pub fn theme_snapshot(&mut self) -> fret_ui::ThemeSnapshot {
        self.raw.theme().snapshot()
    }

    pub fn key<T: Hash>(&self, value: &T) -> u64 {
        self.raw.key(value)
    }

    pub fn key_scope<T: Hash>(&self, scope: &T) -> CanvasKey {
        self.raw.key_scope(scope)
    }

    pub fn child_key<T: Hash>(&self, parent: CanvasKey, child: &T) -> CanvasKey {
        self.raw.child_key(parent, child)
    }

    pub fn push(&mut self, op: SceneOp) {
        self.raw.scene().push(op);
    }

    pub fn quad(
        &mut self,
        order: DrawOrder,
        rect: Rect,
        background: Paint,
        border: Edges,
        border_paint: Paint,
        corner_radii: Corners,
    ) {
        self.push(SceneOp::Quad {
            order,
            rect,
            background: background.into(),
            border,
            border_paint: border_paint.into(),
            corner_radii,
        });
    }

    pub fn with_clip_rect<R>(
        &mut self,
        rect: Rect,
        f: impl for<'a> FnOnce(&mut AppCanvasPainter<'a, 'raw>) -> R,
    ) -> R {
        self.raw.with_clip_rect(rect, |raw| {
            let mut painter = AppCanvasPainter { raw };
            f(&mut painter)
        })
    }

    pub fn with_transform<R>(
        &mut self,
        transform: Transform2D,
        f: impl for<'a> FnOnce(&mut AppCanvasPainter<'a, 'raw>) -> R,
    ) -> R {
        self.raw.with_transform(transform, |raw| {
            let mut painter = AppCanvasPainter { raw };
            f(&mut painter)
        })
    }

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
        self.raw.path(
            key,
            order,
            origin,
            commands,
            style,
            color,
            raster_scale_factor,
        )
    }

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
        self.raw.path_paint(
            key,
            order,
            origin,
            commands,
            style,
            paint,
            raster_scale_factor,
        )
    }
}

/// App-facing raw canvas panel builder.
///
/// Use this for custom editor-style drawing surfaces that need direct pointer and paint loops but
/// should not import `fret_ui::canvas::CanvasPainter` or `CanvasProps`.
pub struct Canvas {
    props: CanvasProps,
}

/// App-facing canvas element produced by [`Canvas::paint`].
pub struct CanvasSurface {
    props: CanvasProps,
    paint: Arc<dyn for<'paint, 'raw> Fn(&mut AppCanvasPainter<'paint, 'raw>) + 'static>,
    semantics: Option<SemanticsDecoration>,
}

impl Default for Canvas {
    fn default() -> Self {
        Self::new()
    }
}

impl Canvas {
    pub fn new() -> Self {
        Self {
            props: CanvasProps::default(),
        }
    }

    pub fn fill_width(mut self) -> Self {
        self.props.layout.size.width = Length::Fill;
        self
    }

    pub fn fill_height(mut self) -> Self {
        self.props.layout.size.height = Length::Fill;
        self
    }

    pub fn size_full(mut self) -> Self {
        self.props.layout.size.width = Length::Fill;
        self.props.layout.size.height = Length::Fill;
        self
    }

    pub fn width_px(mut self, width: Px) -> Self {
        self.props.layout.size.width = Length::Px(width);
        self
    }

    pub fn height_px(mut self, height: Px) -> Self {
        self.props.layout.size.height = Length::Px(height);
        self
    }

    pub fn cache_policy(mut self, cache_policy: CanvasCachePolicy) -> Self {
        self.props.cache_policy = cache_policy;
        self
    }

    pub fn paint(
        self,
        paint: impl for<'paint, 'raw> Fn(&mut AppCanvasPainter<'paint, 'raw>) + 'static,
    ) -> CanvasSurface {
        CanvasSurface {
            props: self.props,
            paint: Arc::new(paint),
            semantics: None,
        }
    }

    #[track_caller]
    pub fn into_element<H>(
        self,
        cx: &mut AppUi<'_, '_, H>,
        paint: impl for<'paint, 'raw> Fn(&mut AppCanvasPainter<'paint, 'raw>) + 'static,
    ) -> fret_ui::element::AnyElement
    where
        H: UiHost,
    {
        self.paint(paint).into_element(cx.elements())
    }
}

impl CanvasSurface {
    pub fn a11y(mut self, decoration: SemanticsDecoration) -> Self {
        self.semantics = Some(match self.semantics.take() {
            Some(existing) => existing.merge(decoration),
            None => decoration,
        });
        self
    }

    pub fn test_id(self, id: impl Into<Arc<str>>) -> Self {
        self.a11y(SemanticsDecoration::default().test_id(id))
    }
}

impl<H: UiHost> fret_ui_kit::IntoUiElement<H> for CanvasSurface {
    #[track_caller]
    fn into_element(self, cx: &mut fret_ui::ElementContext<'_, H>) -> fret_ui::element::AnyElement {
        let paint = self.paint;
        let element = cx.canvas(self.props, move |raw| {
            let mut painter = AppCanvasPainter { raw };
            paint(&mut painter);
        });
        match self.semantics {
            Some(semantics) => element.a11y(semantics),
            None => element,
        }
    }
}

/// App-facing pan/zoom canvas panel builder.
///
/// The builder keeps `LocalState<PanZoom2D>` as the public state handle and adapts optional custom
/// pointer handlers through `PointerActionCx`, leaving raw `Model<T>` and `UiPointerActionHost`
/// plumbing inside the facade.
pub struct PanZoomCanvas {
    props: fret_canvas::ui::PanZoomCanvasSurfacePanelProps,
}

impl PanZoomCanvas {
    pub fn new(view: &LocalState<PanZoom2D>) -> Self {
        let mut props = fret_canvas::ui::PanZoomCanvasSurfacePanelProps {
            view: Some(view.clone_model()),
            ..Default::default()
        };
        props.canvas.cache_policy = fret_ui::element::CanvasCachePolicy::smooth_default();
        Self { props }
    }

    pub fn default_view(mut self, view: PanZoom2D) -> Self {
        self.props.default_view = view;
        self
    }

    pub fn input_preset(mut self, preset: PanZoomInputPreset) -> Self {
        self.props.preset = preset;
        self
    }

    pub fn desktop_canvas_cad(self) -> Self {
        self.input_preset(PanZoomInputPreset::DesktopCanvasCad)
    }

    pub fn pan_button(mut self, button: MouseButton) -> Self {
        self.props.pan_button = button;
        self
    }

    pub fn on_pointer_down(
        mut self,
        handler: impl Fn(&mut PointerActionCx<'_>, PointerDown) -> bool + 'static,
    ) -> Self {
        self.props.on_pointer_down = Some(Arc::new(move |host, action_cx, down| {
            let mut cx = PointerActionCx::new(host, action_cx);
            handler(&mut cx, down)
        }));
        self
    }

    pub fn on_pointer_move(
        mut self,
        handler: impl Fn(&mut PointerActionCx<'_>, PointerMove) -> bool + 'static,
    ) -> Self {
        self.props.on_pointer_move = Some(Arc::new(move |host, action_cx, mv| {
            let mut cx = PointerActionCx::new(host, action_cx);
            handler(&mut cx, mv)
        }));
        self
    }

    pub fn on_pointer_up(
        mut self,
        handler: impl Fn(&mut PointerActionCx<'_>, PointerUp) -> bool + 'static,
    ) -> Self {
        self.props.on_pointer_up = Some(Arc::new(move |host, action_cx, up| {
            let mut cx = PointerActionCx::new(host, action_cx);
            handler(&mut cx, up)
        }));
        self
    }

    #[track_caller]
    pub fn into_element<H>(
        self,
        cx: &mut AppUi<'_, '_, H>,
        paint: impl for<'paint, 'raw> Fn(&mut AppCanvasPainter<'paint, 'raw>, PanZoomCanvasPaintCx)
        + 'static,
    ) -> fret_ui::element::AnyElement
    where
        H: UiHost,
    {
        fret_canvas::ui::pan_zoom_canvas_surface_panel(cx.elements(), self.props, move |raw, cx| {
            let mut painter = AppCanvasPainter { raw };
            paint(&mut painter, cx);
        })
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[allow(dead_code)]
    fn app_canvas_painter_custom_surface_api_compiles(
        painter: &mut AppCanvasPainter<'_, '_>,
    ) -> PathMetrics {
        let _theme = painter.theme_snapshot();
        let _standalone_key = painter.key(&"standalone");
        let scope = painter.key_scope(&"custom-surface");
        let key: u64 = painter.child_key(scope, &0u8).into();
        let origin = Point::new(Px(0.0), Px(0.0));
        let commands = [
            PathCommand::MoveTo(origin),
            PathCommand::LineTo(Point::new(Px(8.0), Px(8.0))),
        ];
        let style = PathStyle::Fill(Default::default());
        let scale = painter.scale_factor();

        let _ = painter.path(
            key,
            DrawOrder(0),
            origin,
            &commands,
            style,
            Color::TRANSPARENT,
            scale,
        );
        painter.path_paint(
            key,
            DrawOrder(1),
            origin,
            &commands,
            style,
            Color::TRANSPARENT.into(),
            scale,
        )
    }

    #[test]
    fn canvas_builder_defaults_to_full_size_and_accepts_cache_policy() {
        let _canvas = Canvas::new()
            .fill_width()
            .fill_height()
            .size_full()
            .width_px(Px(10.0))
            .height_px(Px(20.0))
            .cache_policy(CanvasCachePolicy::smooth_default())
            .paint(|_painter| {})
            .test_id("canvas.builder.test");
    }
}
