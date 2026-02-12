use super::*;

#[path = "navigation_menu/fixtures.rs"]
mod fixtures;

fn assert_navigation_menu_trigger_surface_colors_match(
    web_name: &str,
    open_label: &str,
    open_value: &str,
    closed_label: &str,
    web_theme_name: &str,
    scheme: fret_ui_shadcn::shadcn_themes::ShadcnColorScheme,
) {
    use fret_ui_shadcn::{NavigationMenu, NavigationMenuItem};

    let web = read_web_golden_open(web_name);
    let theme = web_theme_named(&web, web_theme_name);

    let web_open_trigger = find_by_data_slot_and_state_and_text(
        &theme.root,
        "navigation-menu-trigger",
        "open",
        open_label,
    )
    .unwrap_or_else(|| {
        panic!(
            "missing web open trigger: slot=navigation-menu-trigger state=open text={open_label:?}"
        )
    });
    let web_closed_trigger = find_by_data_slot_and_state_and_text(
        &theme.root,
        "navigation-menu-trigger",
        "closed",
        closed_label,
    )
    .unwrap_or_else(|| {
        panic!(
            "missing web closed trigger: slot=navigation-menu-trigger state=closed text={closed_label:?}"
        )
    });

    let web_open_bg = web_open_trigger
        .computed_style
        .get("backgroundColor")
        .and_then(|v| parse_css_color(v));
    let web_open_text = web_open_trigger
        .computed_style
        .get("color")
        .and_then(|v| parse_css_color(v));

    let web_closed_bg = web_closed_trigger
        .computed_style
        .get("backgroundColor")
        .and_then(|v| parse_css_color(v));
    let web_closed_text = web_closed_trigger
        .computed_style
        .get("color")
        .and_then(|v| parse_css_color(v));

    let bounds = bounds_for_theme_viewport(theme).unwrap_or_else(|| {
        Rect::new(
            Point::new(Px(0.0), Px(0.0)),
            CoreSize::new(Px(640.0), Px(480.0)),
        )
    });

    let window = AppWindowId::default();
    let mut app = App::new();
    setup_app_with_shadcn_theme_scheme(&mut app, scheme);

    let mut ui: UiTree<App> = UiTree::new();
    ui.set_window(window);
    let mut services = FakeServices;

    let model: Model<Option<Arc<str>>> = app.models_mut().insert(None);

    render_frame(
        &mut ui,
        &mut app,
        &mut services,
        window,
        bounds,
        FrameId(1),
        true,
        |cx| {
            vec![
                NavigationMenu::new(model.clone())
                    .viewport(false)
                    .indicator(false)
                    .items(vec![
                        NavigationMenuItem::new("home", "Home", vec![cx.text("Home content")]),
                        NavigationMenuItem::new(
                            "components",
                            "Components",
                            vec![cx.text("Components content")],
                        ),
                        NavigationMenuItem::new("list", "List", vec![cx.text("List content")]),
                    ])
                    .into_element(cx),
            ]
        },
    );

    let snap = ui.semantics_snapshot().expect("semantics snapshot").clone();
    let open_trigger = snap
        .nodes
        .iter()
        .find(|n| n.role == SemanticsRole::Button && n.label.as_deref() == Some(open_label))
        .unwrap_or_else(|| panic!("missing fret trigger semantics node: Button {open_label:?}"));
    left_click_center(
        &mut ui,
        &mut app,
        &mut services,
        bounds_center(open_trigger.bounds),
    );

    let _ = app
        .models_mut()
        .update(&model, |v| *v = Some(Arc::from(open_value)));

    let settle_frames = fret_ui_kit::declarative::overlay_motion::SHADCN_MOTION_TICKS_100 + 2;
    for tick in 0..settle_frames {
        let request_semantics = tick + 1 == settle_frames;
        render_frame(
            &mut ui,
            &mut app,
            &mut services,
            window,
            bounds,
            FrameId(2 + tick),
            request_semantics,
            |cx| {
                vec![
                    NavigationMenu::new(model.clone())
                        .viewport(false)
                        .indicator(false)
                        .items(vec![
                            NavigationMenuItem::new("home", "Home", vec![cx.text("Home content")]),
                            NavigationMenuItem::new(
                                "components",
                                "Components",
                                vec![cx.text("Components content")],
                            ),
                            NavigationMenuItem::new("list", "List", vec![cx.text("List content")]),
                        ])
                        .into_element(cx),
                ]
            },
        );
    }

    let (snap, scene) = paint_frame(&mut ui, &mut app, &mut services, bounds);

    let open_trigger = snap
        .nodes
        .iter()
        .find(|n| n.role == SemanticsRole::Button && n.label.as_deref() == Some(open_label))
        .unwrap_or_else(|| {
            panic!("missing fret trigger semantics node after open: {open_label:?}")
        });
    let closed_trigger = snap
        .nodes
        .iter()
        .find(|n| n.role == SemanticsRole::Button && n.label.as_deref() == Some(closed_label))
        .unwrap_or_else(|| {
            panic!("missing fret trigger semantics node after open: {closed_label:?}")
        });

    let open_quad = find_best_chrome_quad(&scene, open_trigger.bounds)
        .expect("painted quad for navigation-menu trigger chrome (open)");
    let closed_quad = find_best_chrome_quad(&scene, closed_trigger.bounds)
        .expect("painted quad for navigation-menu trigger chrome (closed)");

    if let Some(web_open_bg) = web_open_bg
        && web_open_bg.a > 0.01
    {
        let fret_bg = color_to_rgba(open_quad.background);
        assert_close(
            &format!("{web_name} {web_theme_name} trigger[{open_label}] bg.r"),
            fret_bg.r,
            web_open_bg.r,
            0.02,
        );
        assert_close(
            &format!("{web_name} {web_theme_name} trigger[{open_label}] bg.g"),
            fret_bg.g,
            web_open_bg.g,
            0.02,
        );
        assert_close(
            &format!("{web_name} {web_theme_name} trigger[{open_label}] bg.b"),
            fret_bg.b,
            web_open_bg.b,
            0.02,
        );
        assert_close(
            &format!("{web_name} {web_theme_name} trigger[{open_label}] bg.a"),
            fret_bg.a,
            web_open_bg.a,
            0.02,
        );
    }

    if let Some(web_closed_bg) = web_closed_bg
        && web_closed_bg.a > 0.01
    {
        let fret_bg = color_to_rgba(closed_quad.background);
        assert_close(
            &format!("{web_name} {web_theme_name} trigger[{closed_label}] bg.r"),
            fret_bg.r,
            web_closed_bg.r,
            0.02,
        );
        assert_close(
            &format!("{web_name} {web_theme_name} trigger[{closed_label}] bg.g"),
            fret_bg.g,
            web_closed_bg.g,
            0.02,
        );
        assert_close(
            &format!("{web_name} {web_theme_name} trigger[{closed_label}] bg.b"),
            fret_bg.b,
            web_closed_bg.b,
            0.02,
        );
        assert_close(
            &format!("{web_name} {web_theme_name} trigger[{closed_label}] bg.a"),
            fret_bg.a,
            web_closed_bg.a,
            0.02,
        );
    }

    if let Some(web_open_text) = web_open_text
        && web_open_text.a > 0.01
    {
        let text = find_best_text_color_near(
            &scene,
            open_trigger.bounds,
            bounds_center(open_trigger.bounds),
        )
        .expect("open trigger text color");
        assert_close(
            &format!("{web_name} {web_theme_name} trigger[{open_label}] text.r"),
            text.r,
            web_open_text.r,
            0.05,
        );
        assert_close(
            &format!("{web_name} {web_theme_name} trigger[{open_label}] text.g"),
            text.g,
            web_open_text.g,
            0.05,
        );
        assert_close(
            &format!("{web_name} {web_theme_name} trigger[{open_label}] text.b"),
            text.b,
            web_open_text.b,
            0.05,
        );
        assert_close(
            &format!("{web_name} {web_theme_name} trigger[{open_label}] text.a"),
            text.a,
            web_open_text.a,
            0.05,
        );
    }

    if let Some(web_closed_text) = web_closed_text
        && web_closed_text.a > 0.01
    {
        let text = find_best_text_color_near(
            &scene,
            closed_trigger.bounds,
            bounds_center(closed_trigger.bounds),
        )
        .expect("closed trigger text color");
        assert_close(
            &format!("{web_name} {web_theme_name} trigger[{closed_label}] text.r"),
            text.r,
            web_closed_text.r,
            0.05,
        );
        assert_close(
            &format!("{web_name} {web_theme_name} trigger[{closed_label}] text.g"),
            text.g,
            web_closed_text.g,
            0.05,
        );
        assert_close(
            &format!("{web_name} {web_theme_name} trigger[{closed_label}] text.b"),
            text.b,
            web_closed_text.b,
            0.05,
        );
        assert_close(
            &format!("{web_name} {web_theme_name} trigger[{closed_label}] text.a"),
            text.a,
            web_closed_text.a,
            0.05,
        );
    }
}
