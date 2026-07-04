//! App-facing canvas helpers.

use std::sync::Arc;

use fret_canvas::ui::{PanZoomCanvasPaintCx, PanZoomInputPreset};
use fret_canvas::view::PanZoom2D;
use fret_core::scene::Paint;
use fret_core::{Corners, DrawOrder, Edges, Rect, SceneOp, Transform2D};
use fret_ui::UiHost;

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
