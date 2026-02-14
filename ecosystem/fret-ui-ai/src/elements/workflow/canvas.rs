use std::sync::Arc;

use fret_core::scene::{DrawOrder, Paint, SceneOp};
use fret_core::{Corners, Edges, Px, SemanticsRole};
use fret_runtime::Model;
use fret_ui::canvas::CanvasPainter;
use fret_ui::element::{AnyElement, SemanticsProps};
use fret_ui::{ElementContext, Theme, UiHost};
use fret_ui_kit::declarative::stack;
use fret_ui_kit::declarative::style as decl_style;
use fret_ui_kit::{ChromeRefinement, ColorRef, LayoutRefinement, Radius, Space};

use fret_canvas::ui::{
    PanZoomCanvasPaintCx, PanZoomCanvasSurfacePanelProps, editor_pan_zoom_canvas_surface_panel,
};
use fret_canvas::view::PanZoom2D;

/// AI Elements-aligned workflow `Canvas` chrome (UI-only).
///
/// Upstream reference: `repo-ref/ai-elements/packages/elements/src/canvas.tsx`.
///
/// Notes:
/// - Upstream is `@xyflow/react`-backed (`ReactFlow` + `Background`).
/// - In Fret this is a host surface that wires a pan/zoom canvas background and an overlay slot.
/// - Apps own the actual graph engine (nodes/edges/layout/hit-testing).
#[derive(Clone)]
pub struct WorkflowCanvas {
    children: Vec<AnyElement>,
    test_id: Option<Arc<str>>,
    layout: LayoutRefinement,
    chrome: ChromeRefinement,
    view: Option<Model<PanZoom2D>>,
}

impl std::fmt::Debug for WorkflowCanvas {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.debug_struct("WorkflowCanvas")
            .field("children_len", &self.children.len())
            .field("test_id", &self.test_id.as_deref())
            .field("layout", &self.layout)
            .field("chrome", &self.chrome)
            .field("view_present", &self.view.is_some())
            .finish()
    }
}

impl WorkflowCanvas {
    pub fn new(children: impl IntoIterator<Item = AnyElement>) -> Self {
        Self {
            children: children.into_iter().collect(),
            test_id: None,
            layout: LayoutRefinement::default()
                .w_full()
                .h_full()
                .min_w_0()
                .min_h_0()
                .relative()
                .overflow_hidden(),
            chrome: ChromeRefinement::default(),
            view: None,
        }
    }

    /// Provide an app-owned pan/zoom view model (controlled).
    ///
    /// When omitted, an internal view model is used.
    pub fn view(mut self, view: Model<PanZoom2D>) -> Self {
        self.view = Some(view);
        self
    }

    pub fn test_id(mut self, id: impl Into<Arc<str>>) -> Self {
        self.test_id = Some(id.into());
        self
    }

    pub fn refine_layout(mut self, layout: LayoutRefinement) -> Self {
        self.layout = self.layout.merge(layout);
        self
    }

    pub fn refine_style(mut self, chrome: ChromeRefinement) -> Self {
        self.chrome = self.chrome.merge(chrome);
        self
    }

    /// Render the canvas with a paint handler.
    ///
    /// The default paint pass fills the background using the `sidebar` token (matching upstream's
    /// `Background bgColor="var(--sidebar)"`), and then delegates to `paint`.
    #[track_caller]
    pub fn into_element_with_paint<H: UiHost>(
        self,
        cx: &mut ElementContext<'_, H>,
        paint: impl for<'p> Fn(&mut CanvasPainter<'p>, WorkflowCanvasPaintCx) + 'static,
    ) -> AnyElement {
        let theme = Theme::global(&*cx.app).clone();

        let sidebar = theme
            .color_by_key("sidebar")
            .or_else(|| theme.color_by_key("sidebar-background"))
            .unwrap_or_else(|| theme.color_required("background"));
        let border = theme.color_required("border");

        let base_chrome = ChromeRefinement::default()
            .rounded(Radius::Md)
            .border_1()
            .border_color(ColorRef::Color(border))
            .bg(ColorRef::Color(sidebar))
            .p(Space::N0);

        let root_props =
            decl_style::container_props(&theme, base_chrome.merge(self.chrome), self.layout);

        let overlay_children = self.children;

        let view = self.view;
        let canvas = editor_pan_zoom_canvas_surface_panel(
            cx,
            PanZoomCanvasSurfacePanelProps {
                view,
                ..Default::default()
            },
            move |p, paint_cx: PanZoomCanvasPaintCx| {
                let bounds = p.bounds();
                if bounds.size.width.0 > 0.0 && bounds.size.height.0 > 0.0 {
                    p.scene().push(SceneOp::Quad {
                        order: DrawOrder(0),
                        rect: bounds,
                        background: Paint::Solid(sidebar),
                        border: Edges::all(Px(0.0)),
                        border_paint: Paint::Solid(sidebar),
                        corner_radii: Corners::all(Px(0.0)),
                    });
                }

                paint(
                    p,
                    WorkflowCanvasPaintCx {
                        view: paint_cx.view,
                        raster_scale_factor: paint_cx.raster_scale_factor,
                    },
                );
            },
        );

        let overlay = cx.container(
            fret_ui::element::ContainerProps {
                layout: decl_style::layout_style(
                    &theme,
                    LayoutRefinement::default()
                        .absolute()
                        .inset_px(Px(0.0))
                        .w_full()
                        .h_full()
                        .min_w_0()
                        .min_h_0(),
                ),
                ..Default::default()
            },
            move |_cx| overlay_children,
        );

        let body = cx.container(root_props, move |cx| {
            vec![stack::vstack(
                cx,
                stack::VStackProps::default().gap(Space::N0).layout(
                    LayoutRefinement::default()
                        .w_full()
                        .h_full()
                        .min_w_0()
                        .min_h_0(),
                ),
                move |_cx| vec![canvas, overlay],
            )]
        });

        let Some(test_id) = self.test_id else {
            return body;
        };
        cx.semantics(
            SemanticsProps {
                role: SemanticsRole::Group,
                test_id: Some(test_id),
                ..Default::default()
            },
            move |_cx| [body],
        )
    }

    /// Render the canvas with the default paint pass (background only).
    #[track_caller]
    pub fn into_element<H: UiHost>(self, cx: &mut ElementContext<'_, H>) -> AnyElement {
        self.into_element_with_paint(cx, |_p, _cx| {})
    }
}

#[derive(Debug, Clone, Copy)]
pub struct WorkflowCanvasPaintCx {
    pub view: PanZoom2D,
    pub raster_scale_factor: f32,
}
