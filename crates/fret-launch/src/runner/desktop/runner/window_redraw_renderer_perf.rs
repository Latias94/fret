use fret_app::App;
use fret_core::AppWindowId;

use super::WinitAppDriver;

pub(super) fn maybe_publish_window_redraw_renderer_perf_sample<D: WinitAppDriver>(
    app: &mut App,
    driver: &mut D,
    renderer: &mut fret_render::Renderer,
    app_window: AppWindowId,
    user: &mut D::WindowState,
    tick_id: u64,
    frame_id: u64,
) {
    let diag_renderer_perf =
        std::env::var_os("FRET_DIAG_RENDERER_PERF").is_some_and(|v| !v.is_empty());
    if !diag_renderer_perf {
        return;
    }

    let sample =
        renderer
            .take_last_frame_perf_snapshot()
            .map(|perf| fret_render::RendererPerfFrameSample {
                tick_id,
                frame_id,
                perf,
            });
    if let Some(sample) = sample {
        app.with_global_mut_untracked(
            fret_render::RendererPerfFrameStore::default,
            |store, _app| {
                store.record(app_window, tick_id, frame_id, sample.perf);
            },
        );
    }
    driver.renderer_perf_sample(app, app_window, user, sample);
}
