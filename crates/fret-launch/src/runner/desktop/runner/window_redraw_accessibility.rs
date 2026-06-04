use fret_app::App;
use fret_core::AppWindowId;
use fret_runner_winit::accessibility;

use super::WinitAppDriver;

pub(super) fn update_window_redraw_accessibility_snapshot<D: WinitAppDriver>(
    driver: &mut D,
    app: &mut App,
    app_window: AppWindowId,
    user: &mut D::WindowState,
    accessibility: &mut Option<accessibility::WinitAccessibility>,
    last_semantics_snapshot: &mut Option<std::sync::Arc<fret_core::SemanticsSnapshot>>,
    scale_factor: f64,
) {
    if let Some(a11y) = accessibility.as_mut()
        && a11y.is_active()
        && let Some(snapshot) = driver.semantics_snapshot(app, app_window, user)
    {
        let update = accessibility::tree_update_from_snapshot(&snapshot, scale_factor);
        a11y.update_if_active(|| update);
        *last_semantics_snapshot = Some(snapshot);
    } else {
        *last_semantics_snapshot = None;
    }
}
