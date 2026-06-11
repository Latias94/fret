use std::any::TypeId;

use fret_core::WindowMetricsService;
use fret_ui::{Theme, UiHost};

use super::presets::EditorThemePresetV1;
use super::reapply_installed_editor_theme_preset_v1;

/// Reapply the installed editor preset when a `WindowMetricsService` change may have caused the
/// host app to rebuild its base theme.
///
/// This is the common "host changed first, editor patch second" ordering used by apps that keep a
/// host-owned theme in sync with environment light/dark preferences. If the host sync turns out to
/// be a no-op, the installed editor preset is not replayed again.
pub fn sync_host_theme_then_reapply_installed_editor_theme_preset_on_window_metrics_change<
    H: UiHost,
>(
    app: &mut H,
    changed: &[TypeId],
    sync_host_theme: impl FnOnce(&mut H),
) -> Option<EditorThemePresetV1> {
    if !changed.contains(&TypeId::of::<WindowMetricsService>()) {
        return None;
    }

    let theme_revision_before = Theme::global(&*app).revision();
    sync_host_theme(app);
    if Theme::global(&*app).revision() == theme_revision_before {
        return None;
    }
    reapply_installed_editor_theme_preset_v1(app)
}

/// Reapply the installed editor preset when `WindowMetricsService` changes and no host theme sync
/// callback is needed.
pub fn reapply_installed_editor_theme_preset_on_window_metrics_change<H: UiHost>(
    app: &mut H,
    changed: &[TypeId],
) -> Option<EditorThemePresetV1> {
    sync_host_theme_then_reapply_installed_editor_theme_preset_on_window_metrics_change(
        app,
        changed,
        |_app| {},
    )
}
