use std::any::TypeId;

use super::KernelApp;
use crate::shadcn;
use crate::shadcn::themes::{ShadcnBaseColor, ShadcnColorScheme, apply_shadcn_new_york};
use fret_core::{AppWindowId, ColorScheme, WindowMetricsService};
use fret_ui::{Theme, UiTree};

#[test]
fn shadcn_auto_theme_middleware_reacts_to_window_metrics() {
    let mut app = KernelApp::new();
    shadcn::app::install(&mut app);

    let window = AppWindowId::from(slotmap::KeyData::from_ffi(1));
    app.with_global_mut(WindowMetricsService::default, |svc, _app| {
        svc.set_color_scheme(window, Some(ColorScheme::Dark));
    });

    let mut ui = UiTree::<KernelApp>::default();
    let mut state = ();

    let before_bg = Theme::global(&app).colors.surface_background;
    let before_rev = Theme::global(&app).revision();

    super::shadcn_sync_theme_from_environment_on_global_changes::<()>(
        &mut app,
        window,
        &mut ui,
        &mut state,
        &[],
    );

    assert_eq!(Theme::global(&app).revision(), before_rev);
    assert_eq!(Theme::global(&app).colors.surface_background, before_bg);

    super::shadcn_sync_theme_from_environment_on_global_changes::<()>(
        &mut app,
        window,
        &mut ui,
        &mut state,
        &[TypeId::of::<WindowMetricsService>()],
    );

    assert_ne!(Theme::global(&app).colors.surface_background, before_bg);
    let rev_after = Theme::global(&app).revision();

    super::shadcn_sync_theme_from_environment_on_global_changes::<()>(
        &mut app,
        window,
        &mut ui,
        &mut state,
        &[TypeId::of::<WindowMetricsService>()],
    );

    assert_eq!(Theme::global(&app).revision(), rev_after);
}

#[cfg(feature = "imui")]
#[test]
fn shadcn_auto_theme_middleware_replays_installed_editor_preset() {
    let mut app = KernelApp::new();
    shadcn::app::install(&mut app);
    fret_ui_editor::theme::install_editor_theme_preset_v1(
        &mut app,
        fret_ui_editor::theme::EditorThemePresetV1::ImguiLikeDense,
    );

    let window = AppWindowId::from(slotmap::KeyData::from_ffi(1));
    app.with_global_mut(WindowMetricsService::default, |svc, _app| {
        svc.set_color_scheme(window, Some(ColorScheme::Dark));
    });

    let mut ui = UiTree::<KernelApp>::default();
    let mut state = ();

    super::shadcn_sync_theme_from_environment_on_global_changes::<()>(
        &mut app,
        window,
        &mut ui,
        &mut state,
        &[TypeId::of::<WindowMetricsService>()],
    );

    assert_eq!(
        Theme::global(&app)
            .metric_by_key(fret_ui_editor::primitives::EditorTokenKeys::TEXT_FIELD_RADIUS),
        Some(fret_core::Px(2.0))
    );
    assert_eq!(
        Theme::global(&app)
            .metric_by_key(fret_ui_editor::primitives::EditorTokenKeys::TEXT_FIELD_PADDING_Y),
        Some(fret_core::Px(3.0))
    );
}

#[test]
fn shadcn_auto_theme_middleware_requires_app_install_config() {
    let mut app = KernelApp::new();
    apply_shadcn_new_york(&mut app, ShadcnBaseColor::Slate, ShadcnColorScheme::Dark);

    let window = AppWindowId::from(slotmap::KeyData::from_ffi(1));
    app.with_global_mut(WindowMetricsService::default, |svc, _app| {
        svc.set_color_scheme(window, Some(ColorScheme::Light));
    });

    let mut ui = UiTree::<KernelApp>::default();
    let mut state = ();
    let before_bg = Theme::global(&app).colors.surface_background;
    let before_rev = Theme::global(&app).revision();

    super::shadcn_sync_theme_from_environment_on_global_changes::<()>(
        &mut app,
        window,
        &mut ui,
        &mut state,
        &[TypeId::of::<WindowMetricsService>()],
    );

    assert_eq!(Theme::global(&app).revision(), before_rev);
    assert_eq!(Theme::global(&app).colors.surface_background, before_bg);
}
