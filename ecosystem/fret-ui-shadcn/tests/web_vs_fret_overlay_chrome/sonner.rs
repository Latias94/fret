use super::*;
use fret_ui_shadcn::facade as shadcn;

fn web_first_sonner_toast(theme: &WebGoldenTheme) -> &WebNode {
    find_first_in_theme(theme, &|n| n.attrs.contains_key("data-sonner-toast"))
        .expect("web sonner toast node ([data-sonner-toast])")
}

fn fret_first_toast_bounds(snap: &fret_core::SemanticsSnapshot) -> Rect {
    let mut candidates: Vec<_> = snap
        .nodes
        .iter()
        .filter(|n| n.role == SemanticsRole::Alert)
        .filter(|n| {
            n.test_id
                .as_deref()
                .is_some_and(|id| id.starts_with("toast-entry-"))
        })
        .collect();

    candidates.sort_by(|a, b| {
        a.bounds
            .origin
            .y
            .0
            .partial_cmp(&b.bounds.origin.y.0)
            .unwrap_or(std::cmp::Ordering::Equal)
    });

    candidates
        .first()
        .map(|n| n.bounds)
        .expect("missing fret toast semantics node (role=Alert, test_id=toast-entry-*)")
}

fn run_sonner_open_scene(
    theme: &WebGoldenTheme,
    scheme: shadcn::themes::ShadcnColorScheme,
    dispatch_toast: impl FnOnce(&shadcn::Sonner, &mut dyn fret_ui::action::UiActionHost, AppWindowId),
) -> (fret_core::SemanticsSnapshot, Scene) {
    let bounds = bounds_for_theme_viewport(theme).unwrap_or_else(|| {
        bounds_for_viewport(WebViewport {
            w: 1440.0,
            h: 900.0,
        })
    });

    let window = AppWindowId::default();
    let mut app = App::new();
    setup_app_with_shadcn_theme_scheme(&mut app, scheme);

    let mut ui: UiTree<App> = UiTree::new();
    ui.set_window(window);
    let mut services = FakeServices;

    render_frame(
        &mut ui,
        &mut app,
        &mut services,
        window,
        bounds,
        FrameId(1),
        false,
        |cx| vec![shadcn::Toaster::new().into_element(cx)],
    );

    let sonner = shadcn::Sonner::global(&mut app);
    {
        let mut host = fret_ui::action::UiActionHostAdapter { app: &mut app };
        dispatch_toast(&sonner, &mut host, window);
    }

    for frame_id in 2..=24 {
        render_frame(
            &mut ui,
            &mut app,
            &mut services,
            window,
            bounds,
            FrameId(frame_id),
            frame_id == 24,
            |cx| vec![shadcn::Toaster::new().into_element(cx)],
        );
    }

    paint_frame(&mut ui, &mut app, &mut services, bounds)
}

fn assert_sonner_demo_shadow_matches(
    web_name: &str,
    web_theme_name: &str,
    scheme: shadcn::themes::ShadcnColorScheme,
) {
    let web = read_web_golden_open(web_name);
    let theme = web_theme_named(&web, web_theme_name);
    let web_toast = web_first_sonner_toast(theme);
    let expected = web_drop_shadow_insets(web_toast);

    let (snap, scene) = run_sonner_open_scene(theme, scheme, |sonner, host, window| {
        let opts = shadcn::ToastMessageOptions::new()
            .description("Sunday, December 03, 2023 at 9:00 AM")
            .action("Undo", fret_runtime::CommandId::new("sonner.toast.undo"));
        let _ = sonner.toast_message(host, window, "Event has been created", opts);
    });

    let toast_bounds = fret_first_toast_bounds(&snap);
    let quad = find_best_chrome_quad(&scene, toast_bounds).expect("toast chrome quad");
    let candidates = fret_drop_shadow_insets_candidates(&scene, quad.rect);

    assert_shadow_insets_match(web_name, web_theme_name, &expected, &candidates);
}

#[test]
fn web_vs_fret_sonner_demo_toast_shadow_matches_web_light() {
    assert_sonner_demo_shadow_matches(
        "sonner-demo",
        "light",
        shadcn::themes::ShadcnColorScheme::Light,
    );
}

#[test]
fn web_vs_fret_sonner_demo_toast_shadow_matches_web_dark() {
    assert_sonner_demo_shadow_matches(
        "sonner-demo",
        "dark",
        shadcn::themes::ShadcnColorScheme::Dark,
    );
}
