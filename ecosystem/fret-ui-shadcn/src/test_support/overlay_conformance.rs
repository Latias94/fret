use fret_app::App;
use fret_core::{AppWindowId, NodeId, Rect};
use fret_runtime::FrameId;
use fret_ui::declarative::RenderRootContext;
use fret_ui::element::AnyElement;
use fret_ui::{ElementContext, UiTree};
use fret_ui_kit::OverlayController;

pub(crate) fn bump_frame_id(app: &mut App) -> FrameId {
    let next_frame = FrameId(app.frame_id().0.saturating_add(1));
    app.set_frame_id(next_frame);
    next_frame
}

pub(crate) fn render_overlay_frame<I>(
    ui: &mut UiTree<App>,
    app: &mut App,
    services: &mut dyn fret_core::UiServices,
    window: AppWindowId,
    bounds: Rect,
    root_name: &str,
    render: impl FnOnce(&mut ElementContext<'_, App>) -> I,
) -> NodeId
where
    I: IntoIterator<Item = AnyElement>,
{
    bump_frame_id(app);
    OverlayController::begin_frame(app, window);

    let root =
        RenderRootContext::new(ui, app, services, window, bounds).render_root(root_name, render);
    ui.set_root(root);
    OverlayController::render(ui, app, services, window, bounds);
    ui.request_semantics_snapshot();
    ui.layout_all(app, services, bounds, 1.0);
    root
}
