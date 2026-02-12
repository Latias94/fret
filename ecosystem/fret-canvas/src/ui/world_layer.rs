//! Declarative "world layer" helpers for pan/zoom canvases.
//!
//! This is a lightweight composition surface intended to support XYFlow-like outcomes:
//! - a pan/zoom canvas paint pass (background + edges),
//! - nodes as normal element subtrees positioned in canvas space,
//! - optional screen-space overlays layered above the world.
//!
//! v0 implementation note:
//! - The world transform is derived from a `Model<PanZoom2D>` and the last known layout bounds of
//!   a `LayoutQueryRegion` wrapper (one-frame latency on bounds changes).
//! - This avoids introducing new `fret-ui` element kinds while still using `render_transform`
//!   semantics for correct hit-testing and pointer coordinate mapping (ADR 0082).

use fret_core::{Rect, Transform2D};
use fret_ui::canvas::CanvasPainter;
use fret_ui::element::{
    AnyElement, ContainerProps, LayoutQueryRegionProps, LayoutStyle, Length, RenderTransformProps,
};
use fret_ui::{ElementContext, Invalidation, UiHost};

use crate::ui::use_controllable_model;
use crate::ui::{
    PanZoomCanvasPaintCx, PanZoomCanvasSurfacePanelProps, pan_zoom_canvas_surface_panel,
};
use crate::view::PanZoom2D;

#[derive(Debug, Clone, Copy, PartialEq, Eq, Default)]
pub enum CanvasWorldScaleMode {
    /// XYFlow-like: nodes and edges scale with the world zoom.
    #[default]
    ScaleWithZoom,
    /// Editor-like: nodes remain constant-size in screen space, while their positions follow the
    /// zoomed canvas mapping.
    ///
    /// In this mode the world subtree is **not** render-transformed. Callers should place node
    /// subtrees using `CanvasWorldPaintCx::canvas_to_screen(...)` (or `view.canvas_to_screen`).
    SemanticZoom,
}

#[derive(Debug, Clone, Copy)]
pub struct CanvasWorldPaintCx {
    pub bounds: Rect,
    pub view: PanZoom2D,
    pub raster_scale_factor: f32,
    pub scale_mode: CanvasWorldScaleMode,
}

impl CanvasWorldPaintCx {
    pub fn canvas_to_screen(&self, canvas: fret_core::Point) -> fret_core::Point {
        self.view.canvas_to_screen(self.bounds, canvas)
    }

    pub fn screen_to_canvas(&self, screen: fret_core::Point) -> fret_core::Point {
        self.view.screen_to_canvas(self.bounds, screen)
    }
}

#[derive(Clone)]
pub struct CanvasWorldSurfacePanelProps {
    /// Layout query wrapper for resolving stable bounds (used to compute the world transform).
    pub layout_query: LayoutQueryRegionProps,
    /// Underlying pan/zoom surface (input + paint).
    pub pan_zoom: PanZoomCanvasSurfacePanelProps,
    pub scale_mode: CanvasWorldScaleMode,
}

impl Default for CanvasWorldSurfacePanelProps {
    fn default() -> Self {
        let mut layout = LayoutStyle::default();
        layout.size.width = Length::Fill;
        layout.size.height = Length::Fill;

        Self {
            layout_query: LayoutQueryRegionProps {
                layout,
                name: Some("fret-canvas.ui.canvas_world_surface_panel".into()),
            },
            pan_zoom: PanZoomCanvasSurfacePanelProps::default(),
            scale_mode: CanvasWorldScaleMode::default(),
        }
    }
}

#[track_caller]
pub fn canvas_world_surface_panel<H: UiHost, W, O>(
    cx: &mut ElementContext<'_, H>,
    mut props: CanvasWorldSurfacePanelProps,
    paint: impl for<'p> Fn(&mut CanvasPainter<'p>, PanZoomCanvasPaintCx) + 'static,
    world: impl FnOnce(&mut ElementContext<'_, H>, CanvasWorldPaintCx) -> W,
    overlay: impl FnOnce(&mut ElementContext<'_, H>, CanvasWorldPaintCx) -> O,
) -> AnyElement
where
    W: IntoIterator<Item = AnyElement>,
    O: IntoIterator<Item = AnyElement>,
{
    let default_view = props.pan_zoom.default_view;
    let view = use_controllable_model(cx, props.pan_zoom.view.take(), move || default_view).model();
    props.pan_zoom.view = Some(view.clone());

    let scale_mode = props.scale_mode;
    let layout_query = props.layout_query.clone();
    cx.layout_query_region_with_id(layout_query, move |cx, region_id| {
        let view_value = cx
            .get_model_copied(&view, Invalidation::Layout)
            .unwrap_or(default_view);

        // Note: `layout_query_bounds` reads last-frame bounds and records a dependency so we
        // invalidate when the region changes.
        let bounds = cx
            .layout_query_bounds(region_id, Invalidation::Layout)
            .unwrap_or_else(|| cx.environment_viewport_bounds(Invalidation::Layout));

        let raster_scale_factor =
            view_value.zoom.max(1.0e-6) * cx.environment_scale_factor(Invalidation::Layout);

        let paint_cx = CanvasWorldPaintCx {
            bounds,
            view: view_value,
            raster_scale_factor,
            scale_mode,
        };

        let canvas = pan_zoom_canvas_surface_panel(cx, props.pan_zoom.clone(), paint);

        let mut fill = LayoutStyle::default();
        fill.size.width = Length::Fill;
        fill.size.height = Length::Fill;

        let world = match scale_mode {
            CanvasWorldScaleMode::ScaleWithZoom => {
                let transform = view_value
                    .render_transform(bounds)
                    .unwrap_or(Transform2D::IDENTITY);
                cx.render_transform_props(
                    RenderTransformProps {
                        layout: fill,
                        transform,
                    },
                    move |cx| world(cx, paint_cx),
                )
            }
            CanvasWorldScaleMode::SemanticZoom => cx.container(
                ContainerProps {
                    layout: fill,
                    ..Default::default()
                },
                move |cx| world(cx, paint_cx),
            ),
        };

        let overlay = overlay(cx, paint_cx);

        let mut container = ContainerProps::default();
        container.layout.size.width = Length::Fill;
        container.layout.size.height = Length::Fill;
        container.layout.position = fret_ui::element::PositionStyle::Relative;

        [cx.container(container, move |_cx| {
            [canvas, world].into_iter().chain(overlay)
        })]
    })
}
