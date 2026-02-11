use super::*;

#[test]
fn web_vs_fret_layout_progress_demo_track_and_indicator_geometry_light() {
    let web = read_web_golden("progress-demo");
    let theme = web.themes.get("light").expect("missing light theme");

    let web_track = web_find_by_class_tokens(
        &theme.root,
        &[
            "bg-primary/20",
            "relative",
            "h-2",
            "overflow-hidden",
            "rounded-full",
            "w-[60%]",
        ],
    )
    .expect("web progress track");
    let web_indicator = web_find_by_class_tokens(
        web_track,
        &["bg-primary", "h-full", "w-full", "flex-1", "transition-all"],
    )
    .or_else(|| web_find_by_class_token(web_track, "bg-primary"))
    .expect("web progress indicator");

    let expected_track_bg = web_track
        .computed_style
        .get("backgroundColor")
        .map(String::as_str)
        .and_then(parse_css_color)
        .expect("web track backgroundColor");
    let expected_indicator_bg = web_indicator
        .computed_style
        .get("backgroundColor")
        .map(String::as_str)
        .and_then(parse_css_color)
        .expect("web indicator backgroundColor");

    let t = (web_indicator.rect.x + web_indicator.rect.w - web_track.rect.x) / web_track.rect.w;
    let v = (t * 100.0).clamp(0.0, 100.0);

    let bounds = Rect::new(
        Point::new(Px(0.0), Px(0.0)),
        CoreSize::new(Px(theme.viewport.w), Px(theme.viewport.h)),
    );

    let window = AppWindowId::default();
    let mut app = App::new();

    fret_ui_shadcn::shadcn_themes::apply_shadcn_new_york_v4(
        &mut app,
        fret_ui_shadcn::shadcn_themes::ShadcnBaseColor::Neutral,
        fret_ui_shadcn::shadcn_themes::ShadcnColorScheme::Light,
    );

    let mut ui: UiTree<App> = UiTree::new();
    ui.set_window(window);
    let mut services = FakeServices::default();

    let root = fret_ui::declarative::render_root(
        &mut ui,
        &mut app,
        &mut services,
        window,
        bounds,
        "web-vs-fret-layout",
        |cx| {
            let width = Px(web_track.rect.w);
            let model: Model<f32> = cx.app.models_mut().insert(v);

            let progress = cx.semantics(
                fret_ui::element::SemanticsProps {
                    role: SemanticsRole::Panel,
                    label: Some(Arc::from("Golden:progress-demo")),
                    ..Default::default()
                },
                move |cx| vec![fret_ui_shadcn::Progress::new(model).into_element(cx)],
            );

            vec![cx.container(
                ContainerProps {
                    layout: fret_ui_kit::declarative::style::layout_style(
                        &Theme::global(&*cx.app),
                        LayoutRefinement::default().w_px(width),
                    ),
                    ..Default::default()
                },
                move |_cx| vec![progress],
            )]
        },
    );
    ui.set_root(root);
    ui.request_semantics_snapshot();
    ui.layout_all(&mut app, &mut services, bounds, 1.0);

    let mut scene = Scene::default();
    ui.paint_all(&mut app, &mut services, bounds, &mut scene, 1.0);

    let (_track_rect, track_bg) =
        find_scene_quad_background_with_rect_close(&scene, web_track.rect, 1.0)
            .expect("track quad");
    assert_rgba_close(
        "progress-demo track background",
        paint_to_rgba(track_bg),
        expected_track_bg,
        0.02,
    );

    let ind = find_scene_quad_background_with_world_rect_close(&scene, web_indicator.rect, 1.0);
    if ind.is_none() {
        debug_dump_scene_quads_near_expected(
            &scene,
            web_indicator.rect,
            Some(expected_indicator_bg),
        );
    }
    let (_ind_rect, ind_bg) = ind.expect("indicator quad");
    assert_rgba_close(
        "progress-demo indicator background",
        paint_to_rgba(ind_bg),
        expected_indicator_bg,
        0.02,
    );
}

#[test]
fn web_vs_fret_layout_progress_demo_track_and_indicator_geometry_dark() {
    let web = read_web_golden("progress-demo");
    let theme = web.themes.get("dark").expect("missing dark theme");

    let web_track = web_find_by_class_tokens(
        &theme.root,
        &[
            "bg-primary/20",
            "relative",
            "h-2",
            "overflow-hidden",
            "rounded-full",
            "w-[60%]",
        ],
    )
    .expect("web progress track");
    let web_indicator = web_find_by_class_tokens(
        web_track,
        &["bg-primary", "h-full", "w-full", "flex-1", "transition-all"],
    )
    .or_else(|| web_find_by_class_token(web_track, "bg-primary"))
    .expect("web progress indicator");

    let expected_track_bg = web_track
        .computed_style
        .get("backgroundColor")
        .map(String::as_str)
        .and_then(parse_css_color)
        .expect("web track backgroundColor");
    let expected_indicator_bg = web_indicator
        .computed_style
        .get("backgroundColor")
        .map(String::as_str)
        .and_then(parse_css_color)
        .expect("web indicator backgroundColor");

    let t = (web_indicator.rect.x + web_indicator.rect.w - web_track.rect.x) / web_track.rect.w;
    let v = (t * 100.0).clamp(0.0, 100.0);

    let bounds = Rect::new(
        Point::new(Px(0.0), Px(0.0)),
        CoreSize::new(Px(theme.viewport.w), Px(theme.viewport.h)),
    );

    let window = AppWindowId::default();
    let mut app = App::new();

    fret_ui_shadcn::shadcn_themes::apply_shadcn_new_york_v4(
        &mut app,
        fret_ui_shadcn::shadcn_themes::ShadcnBaseColor::Neutral,
        fret_ui_shadcn::shadcn_themes::ShadcnColorScheme::Dark,
    );

    let mut ui: UiTree<App> = UiTree::new();
    ui.set_window(window);
    let mut services = FakeServices::default();

    let root = fret_ui::declarative::render_root(
        &mut ui,
        &mut app,
        &mut services,
        window,
        bounds,
        "web-vs-fret-layout",
        |cx| {
            let width = Px(web_track.rect.w);
            let model: Model<f32> = cx.app.models_mut().insert(v);

            let progress = cx.semantics(
                fret_ui::element::SemanticsProps {
                    role: SemanticsRole::Panel,
                    label: Some(Arc::from("Golden:progress-demo")),
                    ..Default::default()
                },
                move |cx| vec![fret_ui_shadcn::Progress::new(model).into_element(cx)],
            );

            vec![cx.container(
                ContainerProps {
                    layout: fret_ui_kit::declarative::style::layout_style(
                        &Theme::global(&*cx.app),
                        LayoutRefinement::default().w_px(width),
                    ),
                    ..Default::default()
                },
                move |_cx| vec![progress],
            )]
        },
    );
    ui.set_root(root);
    ui.request_semantics_snapshot();
    ui.layout_all(&mut app, &mut services, bounds, 1.0);

    let mut scene = Scene::default();
    ui.paint_all(&mut app, &mut services, bounds, &mut scene, 1.0);

    let (_track_rect, track_bg) =
        find_scene_quad_background_with_rect_close(&scene, web_track.rect, 1.0)
            .expect("track quad");
    assert_rgba_close(
        "progress-demo track background",
        paint_to_rgba(track_bg),
        expected_track_bg,
        0.02,
    );

    let ind = find_scene_quad_background_with_world_rect_close(&scene, web_indicator.rect, 1.0);
    if ind.is_none() {
        debug_dump_scene_quads_near_expected(
            &scene,
            web_indicator.rect,
            Some(expected_indicator_bg),
        );
    }
    let (_ind_rect, ind_bg) = ind.expect("indicator quad");
    assert_rgba_close(
        "progress-demo indicator background",
        paint_to_rgba(ind_bg),
        expected_indicator_bg,
        0.02,
    );
}
