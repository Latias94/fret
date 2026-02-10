use fret_core::geometry::{Px, Rect, Size};
use fret_core::scene::{Color, DrawOrder, SceneOp};
use fret_core::{Event, RenderTargetId, SemanticsRole, UiServices, ViewportFit, ViewportMapping};
use fret_runtime::Model;
use fret_ui::UiHost;
use fret_ui::retained_bridge::viewport_surface::{
    ViewportInputCapture, handle_viewport_surface_input,
};
use fret_ui::retained_bridge::{
    Invalidation, LayoutCx, PaintCx, SemanticsCx, UiTreeRetainedExt, Widget,
};

#[derive(Debug, Clone, Copy, PartialEq)]
pub struct Plot3dViewport {
    pub target: RenderTargetId,
    pub target_px_size: (u32, u32),
    pub fit: ViewportFit,
    pub opacity: f32,
}

impl Plot3dViewport {
    pub fn mapping(self, bounds: Rect) -> ViewportMapping {
        ViewportMapping {
            content_rect: bounds,
            target_px_size: self.target_px_size,
            fit: self.fit,
        }
    }

    pub fn draw_rect(self, bounds: Rect) -> Rect {
        self.mapping(bounds).map().draw_rect
    }
}

impl Default for Plot3dViewport {
    fn default() -> Self {
        Self {
            target: RenderTargetId::default(),
            target_px_size: (1, 1),
            fit: ViewportFit::Contain,
            opacity: 1.0,
        }
    }
}

#[derive(Debug, Clone, Default)]
pub struct Plot3dModel {
    pub viewport: Plot3dViewport,
}

#[derive(Debug, Clone, Copy)]
pub struct Plot3dStyle {
    pub background: Option<Color>,
    pub border: Option<Color>,
    pub border_width: Px,
}

impl Default for Plot3dStyle {
    fn default() -> Self {
        Self {
            background: None,
            border: None,
            border_width: Px(1.0),
        }
    }
}

#[derive(Debug)]
pub struct Plot3dCanvas {
    model: Model<Plot3dModel>,
    style: Plot3dStyle,
    capture: Option<ViewportInputCapture>,
}

impl Plot3dCanvas {
    pub fn new(model: Model<Plot3dModel>) -> Self {
        Self {
            model,
            style: Plot3dStyle::default(),
            capture: None,
        }
    }

    pub fn style(mut self, style: Plot3dStyle) -> Self {
        self.style = style;
        self
    }

    pub fn create_node<H: UiHost>(ui: &mut fret_ui::UiTree<H>, canvas: Self) -> fret_core::NodeId {
        ui.create_node_retained(canvas)
    }
}

impl<H: UiHost> Widget<H> for Plot3dCanvas {
    fn event(&mut self, cx: &mut fret_ui::retained_bridge::EventCx<'_, H>, event: &Event) {
        let Ok(viewport) = self.model.read(cx.app, |_app, m| m.viewport) else {
            return;
        };

        let mapping = viewport.mapping(cx.bounds);
        let _ = handle_viewport_surface_input(
            cx,
            event,
            viewport.target,
            mapping,
            &mut self.capture,
            true,
        );
    }

    fn layout(&mut self, cx: &mut LayoutCx<'_, H>) -> Size {
        cx.observe_model(&self.model, Invalidation::Paint);
        cx.available
    }

    fn paint(&mut self, cx: &mut PaintCx<'_, H>) {
        cx.observe_model(&self.model, Invalidation::Paint);

        let theme = cx.theme();
        let background = self
            .style
            .background
            .unwrap_or_else(|| theme.color_required("card"));
        let border = self
            .style
            .border
            .unwrap_or_else(|| theme.color_required("border"));
        let border_width = self.style.border_width;

        let bounds = cx.bounds;
        cx.scene.push(SceneOp::Quad {
            order: DrawOrder(1),
            rect: bounds,
            background: fret_core::Paint::Solid(background),
            border: fret_core::Edges::all(border_width),
            border_paint: fret_core::Paint::Solid(border),
            corner_radii: fret_core::Corners::all(Px(0.0)),
        });

        let viewport = self
            .model
            .read(cx.app, |_app, m| m.viewport)
            .unwrap_or_default();
        let draw_rect = viewport.draw_rect(bounds);

        cx.scene.push(SceneOp::ViewportSurface {
            order: DrawOrder(2),
            rect: draw_rect,
            target: viewport.target,
            opacity: viewport.opacity.clamp(0.0, 1.0),
        });
    }

    fn semantics(&mut self, cx: &mut SemanticsCx<'_, H>) {
        cx.set_role(SemanticsRole::Viewport);
        cx.set_label("Plot3D");
    }

    fn cleanup_resources(&mut self, _services: &mut dyn UiServices) {}
}
