use super::*;

#[test]
fn theme_color_change_does_not_reprepare_text_in_paint() {
    let mut app = TestHost::new();
    let mut ui: UiTree<TestHost> = UiTree::new();
    let window = AppWindowId::default();
    ui.set_window(window);

    let bounds = Rect::new(
        fret_core::Point::new(Px(0.0), Px(0.0)),
        Size::new(Px(200.0), Px(60.0)),
    );
    let mut services = FakeTextService::default();

    // Ensure the theme is stored as a global so we can mutate it between frames.
    app.set_global(crate::Theme::global(&app).clone());

    let root = render_root(
        &mut ui,
        &mut app,
        &mut services,
        window,
        bounds,
        "text-cache",
        |cx| vec![cx.keyed(1u64, |cx| cx.text("hello"))],
    );
    ui.set_root(root);
    ui.layout_all(&mut app, &mut services, bounds, 1.0);
    let mut scene = Scene::default();
    ui.paint_all(&mut app, &mut services, bounds, &mut scene, 1.0);

    let prepares_after_first_paint = services.prepare_calls;

    // Paint-only theme change: should not invalidate the text blob cache path.
    crate::Theme::with_global_mut(&mut app, |theme| {
        let mut cfg = crate::ThemeConfig::default();
        cfg.colors
            .insert("foreground".to_string(), "#ff0000".to_string());
        theme.extend_tokens_from_config(&cfg);
    });

    // Intentionally skip `render_root`/`layout_all` so the only possible text service work is from paint.
    let mut scene = Scene::default();
    ui.paint_all(&mut app, &mut services, bounds, &mut scene, 1.0);

    assert_eq!(
        services.prepare_calls, prepares_after_first_paint,
        "paint-only theme changes should not force re-preparing text blobs"
    );
}
