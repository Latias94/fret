use super::*;

#[path = "command_dialog/fixtures.rs"]
mod fixtures;

fn assert_command_dialog_focused_item_chrome_matches_web(web_theme_name: &str) {
    assert_command_dialog_focused_item_chrome_matches_web_named(
        "command-dialog.focus-first",
        web_theme_name,
    );
}

fn assert_command_dialog_focused_item_chrome_matches_web_named(
    web_name: &str,
    web_theme_name: &str,
) {
    use fret_ui_shadcn::{Button, CommandDialog, CommandItem};

    let web = read_web_golden_open(web_name);
    let theme = web_theme_named(&web, web_theme_name);
    let expected = web_find_highlighted_listbox_option_chrome(theme, "command-item");

    let bounds = theme.viewport.map(bounds_for_viewport).unwrap_or_else(|| {
        Rect::new(
            Point::new(Px(0.0), Px(0.0)),
            CoreSize::new(Px(1440.0), Px(900.0)),
        )
    });

    let window = AppWindowId::default();
    let mut app = App::new();
    let scheme = match web_theme_name {
        "dark" => fret_ui_shadcn::shadcn_themes::ShadcnColorScheme::Dark,
        _ => fret_ui_shadcn::shadcn_themes::ShadcnColorScheme::Light,
    };
    setup_app_with_shadcn_theme_scheme(&mut app, scheme);

    let mut ui: UiTree<App> = UiTree::new();
    ui.set_window(window);
    let mut services = StyleAwareServices::default();

    let open: Model<bool> = app.models_mut().insert(false);

    render_frame(
        &mut ui,
        &mut app,
        &mut services,
        window,
        bounds,
        FrameId(1),
        false,
        |cx| {
            #[derive(Default)]
            struct Models {
                query: Option<Model<String>>,
            }

            let existing = cx.with_state(Models::default, |st| st.query.clone());
            let query = if let Some(existing) = existing {
                existing
            } else {
                let model = cx.app.models_mut().insert(String::new());
                cx.with_state(Models::default, |st| st.query = Some(model.clone()));
                model
            };

            let items = vec![
                CommandItem::new("Calendar"),
                CommandItem::new("Search Emoji"),
                CommandItem::new("Calculator"),
            ];

            vec![
                CommandDialog::new(open.clone(), query, items)
                    .into_element(cx, |cx| Button::new("Open").into_element(cx)),
            ]
        },
    );

    let _ = app.models_mut().update(&open, |v| *v = true);

    let settle_frames = fret_ui_kit::declarative::overlay_motion::SHADCN_MOTION_TICKS_100 + 2;
    for tick in 0..settle_frames {
        render_frame(
            &mut ui,
            &mut app,
            &mut services,
            window,
            bounds,
            FrameId(2 + tick),
            tick + 1 == settle_frames,
            |cx| {
                #[derive(Default)]
                struct Models {
                    query: Option<Model<String>>,
                }

                let existing = cx.with_state(Models::default, |st| st.query.clone());
                let query = if let Some(existing) = existing {
                    existing
                } else {
                    let model = cx.app.models_mut().insert(String::new());
                    cx.with_state(Models::default, |st| st.query = Some(model.clone()));
                    model
                };

                let items = vec![
                    CommandItem::new("Calendar"),
                    CommandItem::new("Search Emoji"),
                    CommandItem::new("Calculator"),
                ];

                vec![
                    CommandDialog::new(open.clone(), query, items)
                        .into_element(cx, |cx| Button::new("Open").into_element(cx)),
                ]
            },
        );
    }

    let (snap, _) = paint_frame(&mut ui, &mut app, &mut services, bounds);
    if let Some(text_field) = snap
        .nodes
        .iter()
        .filter(|n| n.role == SemanticsRole::TextField)
        .max_by(|a, b| rect_area(a.bounds).total_cmp(&rect_area(b.bounds)))
    {
        ui.set_focus(Some(text_field.id));
        render_frame(
            &mut ui,
            &mut app,
            &mut services,
            window,
            bounds,
            FrameId(2 + settle_frames),
            true,
            |cx| {
                #[derive(Default)]
                struct Models {
                    query: Option<Model<String>>,
                }

                let existing = cx.with_state(Models::default, |st| st.query.clone());
                let query = if let Some(existing) = existing {
                    existing
                } else {
                    let model = cx.app.models_mut().insert(String::new());
                    cx.with_state(Models::default, |st| st.query = Some(model.clone()));
                    model
                };

                let items = vec![
                    CommandItem::new("Calendar"),
                    CommandItem::new("Search Emoji"),
                    CommandItem::new("Calculator"),
                ];

                vec![
                    CommandDialog::new(open.clone(), query, items)
                        .into_element(cx, |cx| Button::new("Open").into_element(cx)),
                ]
            },
        );
    }

    dispatch_key_press(&mut ui, &mut app, &mut services, KeyCode::ArrowDown);
    render_frame(
        &mut ui,
        &mut app,
        &mut services,
        window,
        bounds,
        FrameId(3 + settle_frames),
        true,
        |cx| {
            #[derive(Default)]
            struct Models {
                query: Option<Model<String>>,
            }

            let existing = cx.with_state(Models::default, |st| st.query.clone());
            let query = if let Some(existing) = existing {
                existing
            } else {
                let model = cx.app.models_mut().insert(String::new());
                cx.with_state(Models::default, |st| st.query = Some(model.clone()));
                model
            };

            let items = vec![
                CommandItem::new("Calendar"),
                CommandItem::new("Search Emoji"),
                CommandItem::new("Calculator"),
            ];

            vec![
                CommandDialog::new(open.clone(), query, items)
                    .into_element(cx, |cx| Button::new("Open").into_element(cx)),
            ]
        },
    );

    let (snap, scene) = paint_frame(&mut ui, &mut app, &mut services, bounds);
    let option = fret_find_active_listbox_option(&snap).unwrap_or_else(|| {
        let focused_roles: Vec<SemanticsRole> = snap
            .nodes
            .iter()
            .filter(|n| n.flags.focused)
            .map(|n| n.role)
            .collect();
        let active_owner_roles: Vec<SemanticsRole> = snap
            .nodes
            .iter()
            .filter(|n| n.active_descendant.is_some())
            .map(|n| n.role)
            .collect();
        panic!(
            "{web_name} {web_theme_name}: expected active listbox option\n  focused_roles={focused_roles:?}\n  active_descendant_owner_roles={active_owner_roles:?}"
        )
    });

    let quad = find_best_solid_quad_within_matching_bg(&scene, option.bounds, expected.bg)
        .unwrap_or_else(|| panic!("{web_name} {web_theme_name}: focused option background quad"));
    assert_rgba_close(
        &format!("{web_name} {web_theme_name} focused option background"),
        color_to_rgba(quad.background),
        expected.bg,
        0.03,
    );

    let text = find_best_text_color_near(
        &scene,
        option.bounds,
        leftish_text_probe_point(option.bounds),
    )
    .unwrap_or_else(|| panic!("{web_name} {web_theme_name}: focused option text color"));
    assert_rgba_close(
        &format!("{web_name} {web_theme_name} focused option text color"),
        text,
        expected.fg,
        0.03,
    );
}

fn build_shadcn_command_dialog_page(
    cx: &mut ElementContext<'_, App>,
    open: &Model<bool>,
) -> AnyElement {
    use fret_ui_shadcn::{Button, CommandDialog, CommandItem};

    #[derive(Default)]
    struct Models {
        query: Option<Model<String>>,
    }

    let existing = cx.with_state(Models::default, |st| st.query.clone());
    let query = if let Some(existing) = existing {
        existing
    } else {
        let model = cx.app.models_mut().insert(String::new());
        cx.with_state(Models::default, |st| st.query = Some(model.clone()));
        model
    };

    let items = vec![
        CommandItem::new("Calendar"),
        CommandItem::new("Search Emoji"),
        CommandItem::new("Calculator"),
    ];

    CommandDialog::new(open.clone(), query, items)
        .into_element(cx, |cx| Button::new("Open").into_element(cx))
}
