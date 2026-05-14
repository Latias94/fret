use super::*;

pub(crate) fn run_frame(
    ui: &mut UiTree<TestHost>,
    app: &mut TestHost,
    services: &mut FakeTextService,
    window: AppWindowId,
    bounds: Rect,
    root_name: &str,
    render: impl FnOnce(&mut ElementContext<'_, TestHost>) -> crate::Elements,
) -> fret_core::NodeId {
    OverlayController::begin_frame(app, window);
    let root = render_root(ui, app, services, window, bounds, root_name, render);
    OverlayController::render(ui, app, services, window, bounds);
    ui.layout_all(app, services, bounds, 1.0);
    root
}

pub(crate) fn advance_and_run_frame<R>(
    ui: &mut UiTree<TestHost>,
    app: &mut TestHost,
    services: &mut FakeTextService,
    window: AppWindowId,
    bounds: Rect,
    root_name: &str,
    render: &R,
) -> fret_core::NodeId
where
    R: for<'a> Fn(&mut ElementContext<'a, TestHost>) -> crate::Elements,
{
    app.advance_frame();
    run_frame(ui, app, services, window, bounds, root_name, |cx| {
        render(cx)
    })
}
