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
use fret_ui::{ElementContext, GlobalElementId, Invalidation, ItemKey, UiHost};
use std::collections::BTreeMap;

use crate::ui::use_controllable_model;
use crate::ui::{
    PanZoomCanvasPaintCx, PanZoomCanvasSurfacePanelProps, pan_zoom_canvas_surface_panel,
};
use crate::view::screen_rect_to_canvas_rect;
use crate::view::{FitViewOptions2D, PanZoom2D, fit_view_to_canvas_rect};

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

#[derive(Debug, Clone, Copy, PartialEq)]
pub struct CanvasWorldItemBounds {
    /// Root element ID for the wrapped subtree (useful for overlay anchoring queries).
    pub element: GlobalElementId,
    /// The subtree's last known bounds mapped into canvas space under the current surface view.
    pub canvas_bounds: Rect,
}

/// A lightweight, app-owned registry for node subtree bounds in a canvas world layer.
///
/// This is intentionally "outcomes-first" and frame-lagged:
///
/// - The world layer can only read last-frame bounds (`LayoutQueryRegion` / element bounds caches),
///   so updates arrive with one-frame latency.
/// - The common use case (fit-view / selection queries) is tolerant of that latency.
///
/// Apps decide which keys are "active" (stale entries are allowed).
#[derive(Debug, Default, Clone)]
pub struct CanvasWorldBoundsStore {
    pub items: BTreeMap<ItemKey, CanvasWorldItemBounds>,
}

impl CanvasWorldBoundsStore {
    pub fn union_canvas_bounds_for_keys<'a>(
        &self,
        keys: impl IntoIterator<Item = &'a ItemKey>,
    ) -> Option<Rect> {
        let mut out: Option<Rect> = None;
        for key in keys {
            let Some(item) = self.items.get(key) else {
                continue;
            };
            out = Some(match out {
                None => item.canvas_bounds,
                Some(prev) => rect_union(prev, item.canvas_bounds),
            });
        }
        out
    }

    pub fn union_canvas_bounds_for_key_values(
        &self,
        keys: impl IntoIterator<Item = ItemKey>,
    ) -> Option<Rect> {
        let mut out: Option<Rect> = None;
        for key in keys {
            let Some(item) = self.items.get(&key) else {
                continue;
            };
            out = Some(match out {
                None => item.canvas_bounds,
                Some(prev) => rect_union(prev, item.canvas_bounds),
            });
        }
        out
    }
}

/// Computes a `PanZoom2D` view that fits the union of the given item keys.
///
/// Returns `None` when no keys are present in the store.
pub fn canvas_world_fit_view_to_keys(
    surface_bounds: Rect,
    bounds_store: &CanvasWorldBoundsStore,
    keys: impl IntoIterator<Item = ItemKey>,
    options: FitViewOptions2D,
) -> Option<PanZoom2D> {
    let target = bounds_store.union_canvas_bounds_for_key_values(keys)?;
    Some(fit_view_to_canvas_rect(surface_bounds, target, options))
}

fn rect_union(a: Rect, b: Rect) -> Rect {
    let x0 = a.origin.x.0.min(b.origin.x.0);
    let y0 = a.origin.y.0.min(b.origin.y.0);
    let x1 = (a.origin.x.0 + a.size.width.0).max(b.origin.x.0 + b.size.width.0);
    let y1 = (a.origin.y.0 + a.size.height.0).max(b.origin.y.0 + b.size.height.0);
    Rect::new(
        fret_core::Point::new(fret_core::Px(x0), fret_core::Px(y0)),
        fret_core::Size::new(
            fret_core::Px((x1 - x0).max(0.0)),
            fret_core::Px((y1 - y0).max(0.0)),
        ),
    )
}

fn rect_approx_eq(a: Rect, b: Rect, eps: f32) -> bool {
    (a.origin.x.0 - b.origin.x.0).abs() <= eps
        && (a.origin.y.0 - b.origin.y.0).abs() <= eps
        && (a.size.width.0 - b.size.width.0).abs() <= eps
        && (a.size.height.0 - b.size.height.0).abs() <= eps
}

/// Wraps a subtree as a "world item" and reports its last-known bounds into an app-owned store.
///
/// Notes:
///
/// - `key` should be stable for the item within this world surface.
/// - Bounds are frame-lagged by design (see `LayoutQueryRegion` contract).
#[track_caller]
pub fn canvas_world_bounds_item<H: UiHost, I>(
    cx: &mut ElementContext<'_, H>,
    bounds_store: fret_runtime::Model<CanvasWorldBoundsStore>,
    key: ItemKey,
    paint_cx: CanvasWorldPaintCx,
    f: impl FnOnce(&mut ElementContext<'_, H>) -> I,
) -> AnyElement
where
    I: IntoIterator<Item = AnyElement>,
{
    cx.keyed(key, |cx| {
        let mut props = LayoutQueryRegionProps::default();
        props.name = Some("fret-canvas.ui.canvas_world_bounds_item".into());

        cx.layout_query_region_with_id(props, move |cx, element| {
            let visual_bounds = cx
                .last_visual_bounds_for_element(element)
                .or_else(|| cx.last_bounds_for_element(element));

            if let Some(visual_bounds) = visual_bounds {
                let canvas_bounds =
                    screen_rect_to_canvas_rect(paint_cx.bounds, paint_cx.view, visual_bounds);

                let should_update = cx
                    .app
                    .models()
                    .read(&bounds_store, |st| {
                        let Some(prev) = st.items.get(&key) else {
                            return true;
                        };
                        prev.element != element
                            || !rect_approx_eq(prev.canvas_bounds, canvas_bounds, 0.25)
                    })
                    .unwrap_or(true);

                if should_update {
                    let _ = cx.app.update_model(&bounds_store, |st, _| {
                        st.items.insert(
                            key,
                            CanvasWorldItemBounds {
                                element,
                                canvas_bounds,
                            },
                        );
                    });
                    cx.request_frame();
                }
            }

            f(cx)
        })
    })
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
