use super::*;

#[path = "button_group/fixtures.rs"]
mod fixtures;

fn build_shadcn_button_group_demo_dropdown_menu(
    cx: &mut ElementContext<'_, App>,
    open: &Model<bool>,
    label_value: Model<Option<Arc<str>>>,
) -> AnyElement {
    use fret_ui::UiHost;
    use fret_ui::element::{ContainerProps, LayoutStyle, Length};
    use fret_ui_shadcn::{
        Button, ButtonSize, ButtonVariant, DropdownMenu, DropdownMenuAlign, DropdownMenuEntry,
        DropdownMenuGroup, DropdownMenuItem, DropdownMenuRadioGroup, DropdownMenuRadioItemSpec,
    };

    fn icon_stub<H: UiHost>(cx: &mut ElementContext<'_, H>) -> AnyElement {
        cx.container(
            ContainerProps {
                layout: {
                    let mut layout = LayoutStyle::default();
                    layout.size.width = Length::Px(Px(16.0));
                    layout.size.height = Length::Px(Px(16.0));
                    layout
                },
                ..Default::default()
            },
            |_cx| Vec::new(),
        )
    }

    DropdownMenu::new(open.clone())
        .align(DropdownMenuAlign::End)
        // new-york-v4 button-group-demo: `DropdownMenuContent className="w-52"`.
        .min_width(Px(208.0))
        .into_element(
            cx,
            |cx| {
                Button::new("More Options")
                    .variant(ButtonVariant::Outline)
                    .size(ButtonSize::Icon)
                    .children([icon_stub(cx)])
                    .into_element(cx)
            },
            |cx| {
                vec![
                    DropdownMenuEntry::Group(DropdownMenuGroup::new(vec![
                        DropdownMenuEntry::Item(
                            DropdownMenuItem::new("Mark as Read").leading(icon_stub(cx)),
                        ),
                        DropdownMenuEntry::Item(
                            DropdownMenuItem::new("Archive").leading(icon_stub(cx)),
                        ),
                    ])),
                    DropdownMenuEntry::Separator,
                    DropdownMenuEntry::Group(DropdownMenuGroup::new(vec![
                        DropdownMenuEntry::Item(
                            DropdownMenuItem::new("Snooze").leading(icon_stub(cx)),
                        ),
                        DropdownMenuEntry::Item(
                            DropdownMenuItem::new("Add to Calendar").leading(icon_stub(cx)),
                        ),
                        DropdownMenuEntry::Item(
                            DropdownMenuItem::new("Add to List").leading(icon_stub(cx)),
                        ),
                        DropdownMenuEntry::Item(
                            DropdownMenuItem::new("Label As...")
                                .leading(icon_stub(cx))
                                .submenu(vec![DropdownMenuEntry::RadioGroup(
                                    DropdownMenuRadioGroup::new(label_value)
                                        .item(DropdownMenuRadioItemSpec::new(
                                            "personal", "Personal",
                                        ))
                                        .item(DropdownMenuRadioItemSpec::new("work", "Work"))
                                        .item(DropdownMenuRadioItemSpec::new("other", "Other")),
                                )]),
                        ),
                    ])),
                    DropdownMenuEntry::Separator,
                    DropdownMenuEntry::Group(DropdownMenuGroup::new(vec![
                        DropdownMenuEntry::Item(
                            DropdownMenuItem::new("Trash")
                                .leading(icon_stub(cx))
                                .variant(
                                fret_ui_shadcn::dropdown_menu::DropdownMenuItemVariant::Destructive,
                            ),
                        ),
                    ])),
                ]
            },
        )
}

fn assert_button_group_demo_dropdown_menu_destructive_item_idle_fg_matches_web(
    web_theme_name: &str,
    scheme: fret_ui_shadcn::shadcn_themes::ShadcnColorScheme,
) {
    let web = read_web_golden_open("button-group-demo");
    let theme = web_theme_named(&web, web_theme_name);
    let expected = web_find_menu_item_chrome_by_slot_variant_and_text(
        theme,
        "dropdown-menu-item",
        "destructive",
        "Trash",
    );
    assert!(
        expected.bg.a < 0.02,
        "button-group-demo {web_theme_name}: expected destructive item bg to be transparent, got={expected:?}"
    );

    let bounds = bounds_for_theme_viewport(theme).unwrap_or_else(|| {
        Rect::new(
            Point::new(Px(0.0), Px(0.0)),
            CoreSize::new(Px(1440.0), Px(900.0)),
        )
    });

    let window = AppWindowId::default();
    let mut app = App::new();
    setup_app_with_shadcn_theme_scheme(&mut app, scheme);

    let mut ui: UiTree<App> = UiTree::new();
    ui.set_window(window);
    let mut services = StyleAwareServices::default();

    let open: Model<bool> = app.models_mut().insert(false);
    let label_value: Model<Option<Arc<str>>> = app.models_mut().insert(Some(Arc::from("personal")));

    render_frame(
        &mut ui,
        &mut app,
        &mut services,
        window,
        bounds,
        FrameId(1),
        true,
        |cx| {
            vec![build_shadcn_button_group_demo_dropdown_menu(
                cx,
                &open,
                label_value.clone(),
            )]
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
                vec![build_shadcn_button_group_demo_dropdown_menu(
                    cx,
                    &open,
                    label_value.clone(),
                )]
            },
        );
    }

    let (snap, _scene) = paint_frame(&mut ui, &mut app, &mut services, bounds);
    let trash = snap
        .nodes
        .iter()
        .find(|n| n.role == SemanticsRole::MenuItem && n.label.as_deref() == Some("Trash"))
        .expect("button-group-demo: destructive Trash menu item semantics node");
    assert!(
        !trash.flags.focused,
        "button-group-demo {web_theme_name}: expected Trash to be idle (not focused)"
    );

    render_frame(
        &mut ui,
        &mut app,
        &mut services,
        window,
        bounds,
        FrameId(200),
        true,
        |cx| {
            vec![build_shadcn_button_group_demo_dropdown_menu(
                cx,
                &open,
                label_value.clone(),
            )]
        },
    );

    let (_, scene) = paint_frame(&mut ui, &mut app, &mut services, bounds);
    let text =
        find_best_text_color_near(&scene, trash.bounds, leftish_text_probe_point(trash.bounds))
            .unwrap_or_else(|| {
                panic!("button-group-demo {web_theme_name}: destructive idle menu item text color")
            });
    assert_rgba_close(
        &format!("button-group-demo {web_theme_name} destructive idle menu item text color"),
        text,
        expected.fg,
        0.03,
    );
}

fn assert_button_group_demo_dropdown_menu_destructive_focused_item_chrome_matches_web(
    web_theme_name: &str,
    scheme: fret_ui_shadcn::shadcn_themes::ShadcnColorScheme,
) {
    let web = read_web_golden_open("button-group-demo.destructive-focus");
    let theme = web_theme_named(&web, web_theme_name);
    let expected = web_find_active_element_chrome(theme);

    let bounds = bounds_for_theme_viewport(theme).unwrap_or_else(|| {
        Rect::new(
            Point::new(Px(0.0), Px(0.0)),
            CoreSize::new(Px(1440.0), Px(900.0)),
        )
    });

    let window = AppWindowId::default();
    let mut app = App::new();
    setup_app_with_shadcn_theme_scheme(&mut app, scheme);

    let mut ui: UiTree<App> = UiTree::new();
    ui.set_window(window);
    let mut services = StyleAwareServices::default();

    let open: Model<bool> = app.models_mut().insert(false);
    let label_value: Model<Option<Arc<str>>> = app.models_mut().insert(Some(Arc::from("personal")));

    render_frame(
        &mut ui,
        &mut app,
        &mut services,
        window,
        bounds,
        FrameId(1),
        true,
        |cx| {
            vec![build_shadcn_button_group_demo_dropdown_menu(
                cx,
                &open,
                label_value.clone(),
            )]
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
                vec![build_shadcn_button_group_demo_dropdown_menu(
                    cx,
                    &open,
                    label_value.clone(),
                )]
            },
        );
    }

    let (snap2, _) = paint_frame(&mut ui, &mut app, &mut services, bounds);
    let trash = snap2
        .nodes
        .iter()
        .find(|n| n.role == SemanticsRole::MenuItem && n.label.as_deref() == Some("Trash"))
        .expect("button-group-demo: destructive Trash menu item semantics node");

    ui.set_focus(Some(trash.id));
    render_frame(
        &mut ui,
        &mut app,
        &mut services,
        window,
        bounds,
        FrameId(200),
        true,
        |cx| {
            vec![build_shadcn_button_group_demo_dropdown_menu(
                cx,
                &open,
                label_value.clone(),
            )]
        },
    );

    let (snap, scene) = paint_frame(&mut ui, &mut app, &mut services, bounds);
    let focused_fallback = snap
        .nodes
        .iter()
        .find(|n| n.role == SemanticsRole::MenuItem && n.label.as_deref() == Some("Trash"))
        .expect("button-group-demo: Trash menu item semantics node after focus");
    let focused = fret_find_active_menu_item(&snap).unwrap_or(focused_fallback);
    assert!(
        focused.role == SemanticsRole::MenuItem && focused.label.as_deref() == Some("Trash"),
        "button-group-demo.destructive-focus {web_theme_name}: expected focused menu item to be Trash, got role={:?} label={:?}",
        focused.role,
        focused.label
    );

    let quad = find_best_quad_within_matching_bg(&scene, focused.bounds, expected.bg)
        .unwrap_or_else(|| {
            panic!(
                "button-group-demo.destructive-focus {web_theme_name}: destructive focused menu item background quad"
            )
        });
    assert_rgba_close(
        &format!(
            "button-group-demo.destructive-focus {web_theme_name} destructive focused menu item background"
        ),
        color_to_rgba(quad.background),
        expected.bg,
        0.03,
    );

    let text = find_best_text_color_near(
        &scene,
        focused.bounds,
        leftish_text_probe_point(focused.bounds),
    )
    .unwrap_or_else(|| {
        panic!(
            "button-group-demo.destructive-focus {web_theme_name}: destructive focused menu item text color"
        )
    });
    assert_rgba_close(
        &format!(
            "button-group-demo.destructive-focus {web_theme_name} destructive focused menu item text color"
        ),
        text,
        expected.fg,
        0.03,
    );
}
