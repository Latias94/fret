//! Event runtime adapter contract for retained compatibility route preparation.
//!
//! This module keeps event route preparation and dispatch behind a named seam. Concrete
//! retained-context bindings live at the retained runtime entrypoint.

use fret_core::{Event, Rect, UiServices};
use fret_ui::{ThemeSnapshot, UiHost};

use super::{NodeGraphCanvasMiddleware, NodeGraphCanvasWith, ViewSnapshot, event_router_cx};

pub(super) trait CanvasEventRuntimeCx<H: UiHost, M: NodeGraphCanvasMiddleware>:
    event_router_cx::EventRouteCx<H, M>
{
    fn event_runtime_theme_snapshot(&self) -> ThemeSnapshot;
    fn event_runtime_services(&mut self) -> Option<&mut dyn UiServices>;
    fn event_runtime_host(&mut self) -> &mut H;
    fn event_runtime_bounds(&self) -> Rect;
}

pub(super) fn dispatch_canvas_event<H, M>(
    canvas: &mut NodeGraphCanvasWith<M>,
    cx: &mut impl CanvasEventRuntimeCx<H, M>,
    event: &Event,
) where
    H: UiHost,
    M: NodeGraphCanvasMiddleware,
{
    let snapshot = prepare_canvas_event_route(canvas, cx);

    canvas.handle_event(cx, event, &snapshot);
}

fn prepare_canvas_event_route<H, M>(
    canvas: &mut NodeGraphCanvasWith<M>,
    cx: &mut impl CanvasEventRuntimeCx<H, M>,
) -> ViewSnapshot
where
    H: UiHost,
    M: NodeGraphCanvasMiddleware,
{
    super::retained_widget_runtime_shared::sync_runtime_theme(
        canvas,
        cx.event_runtime_theme_snapshot(),
        cx.event_runtime_services(),
    );
    let snapshot = canvas.sync_view_state(cx.event_runtime_host());
    canvas.interaction.last_bounds = Some(cx.event_runtime_bounds());

    snapshot
}
