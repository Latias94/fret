use super::*;

#[test]
fn installed_preset_can_be_reapplied_after_base_theme_reset() {
    let mut app = App::new();
    apply_shadcn_new_york(&mut app, ShadcnBaseColor::Slate, ShadcnColorScheme::Dark);
    install_editor_theme_preset_v1(&mut app, EditorThemePresetV1::Default);
    assert_eq!(
        installed_editor_theme_preset_v1(&app),
        Some(EditorThemePresetV1::Default)
    );

    let expected_field_bg = Some(Color::from_srgb_hex_rgb(0x14_1b_24));
    let expected_panel_bg = Some(Color::from_srgb_hex_rgb(0x0f_15_1d));
    assert_eq!(
        Theme::global(&app).color_by_key(EditorTokenKeys::TEXT_FIELD_BG),
        expected_field_bg
    );
    assert_eq!(
        Theme::global(&app).color_by_key(EditorTokenKeys::PROPERTY_PANEL_BG),
        expected_panel_bg
    );

    apply_shadcn_new_york(&mut app, ShadcnBaseColor::Slate, ShadcnColorScheme::Light);
    assert_ne!(
        Theme::global(&app).color_by_key(EditorTokenKeys::TEXT_FIELD_BG),
        expected_field_bg
    );

    assert_eq!(
        reapply_installed_editor_theme_preset_v1(&mut app),
        Some(EditorThemePresetV1::Default)
    );
    assert_eq!(
        Theme::global(&app).color_by_key(EditorTokenKeys::TEXT_FIELD_BG),
        expected_field_bg
    );
    assert_eq!(
        Theme::global(&app).color_by_key(EditorTokenKeys::PROPERTY_PANEL_BG),
        expected_panel_bg
    );
}

#[test]
fn window_metrics_helper_reapplies_after_host_theme_sync() {
    let mut app = App::new();
    apply_shadcn_new_york(&mut app, ShadcnBaseColor::Slate, ShadcnColorScheme::Dark);
    install_editor_theme_preset_v1(&mut app, EditorThemePresetV1::Default);

    let expected_field_bg = Some(Color::from_srgb_hex_rgb(0x14_1b_24));
    let changed = [TypeId::of::<fret_core::WindowMetricsService>()];

    let replayed =
        sync_host_theme_then_reapply_installed_editor_theme_preset_on_window_metrics_change(
            &mut app,
            &changed,
            |app| {
                apply_shadcn_new_york(app, ShadcnBaseColor::Slate, ShadcnColorScheme::Light);
            },
        );

    assert_eq!(replayed, Some(EditorThemePresetV1::Default));
    assert_eq!(
        Theme::global(&app).color_by_key(EditorTokenKeys::TEXT_FIELD_BG),
        expected_field_bg
    );
}

#[test]
fn window_metrics_helper_skips_replay_when_host_theme_sync_is_noop() {
    let mut app = App::new();
    let window = AppWindowId::from(slotmap::KeyData::from_ffi(1));
    app.with_global_mut(fret_core::WindowMetricsService::default, |svc, _app| {
        svc.set_color_scheme(window, Some(fret_core::ColorScheme::Dark));
    });
    let _ = fret_ui_shadcn::advanced::sync_theme_from_environment(
        &mut app,
        window,
        ShadcnBaseColor::Slate,
        ShadcnColorScheme::Dark,
    );
    install_editor_theme_preset_v1(&mut app, EditorThemePresetV1::Default);

    let expected_field_bg = Theme::global(&app).color_by_key(EditorTokenKeys::TEXT_FIELD_BG);
    let before_rev = Theme::global(&app).revision();
    let changed = [TypeId::of::<fret_core::WindowMetricsService>()];

    let replayed =
        sync_host_theme_then_reapply_installed_editor_theme_preset_on_window_metrics_change(
            &mut app,
            &changed,
            |app| {
                let _ = fret_ui_shadcn::advanced::sync_theme_from_environment(
                    app,
                    window,
                    ShadcnBaseColor::Slate,
                    ShadcnColorScheme::Dark,
                );
            },
        );

    assert_eq!(replayed, None);
    assert_eq!(Theme::global(&app).revision(), before_rev);
    assert_eq!(
        Theme::global(&app).color_by_key(EditorTokenKeys::TEXT_FIELD_BG),
        expected_field_bg
    );
}

#[test]
fn window_metrics_helper_ignores_unrelated_global_changes() {
    let mut app = App::new();
    apply_shadcn_new_york(&mut app, ShadcnBaseColor::Slate, ShadcnColorScheme::Dark);
    install_editor_theme_preset_v1(&mut app, EditorThemePresetV1::Default);

    let expected_field_bg = Theme::global(&app).color_by_key(EditorTokenKeys::TEXT_FIELD_BG);
    let changed = [TypeId::of::<Theme>()];

    let replayed =
        reapply_installed_editor_theme_preset_on_window_metrics_change(&mut app, &changed);

    assert_eq!(replayed, None);
    assert_eq!(
        Theme::global(&app).color_by_key(EditorTokenKeys::TEXT_FIELD_BG),
        expected_field_bg
    );
}
