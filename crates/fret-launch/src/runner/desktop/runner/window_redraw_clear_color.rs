use fret_app::App;
use fret_core::AppWindowId;
use fret_render::ClearColor;

pub(super) fn resolve_window_redraw_clear_color(
    app: &App,
    app_window: AppWindowId,
    configured_clear_color: ClearColor,
) -> ClearColor {
    let want_visual_transparent = app
        .global::<fret_runtime::RunnerWindowStyleDiagnosticsStore>()
        .and_then(|store| store.effective_snapshot(app_window))
        .is_some_and(|snapshot| snapshot.visual_transparent);

    if want_visual_transparent {
        ClearColor(wgpu::Color::TRANSPARENT)
    } else {
        configured_clear_color
    }
}
