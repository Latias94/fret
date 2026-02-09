use super::*;

#[test]
fn fret_context_menu_does_not_move_when_underlay_scrolls() {
    let window = AppWindowId::default();
    let mut app = App::new();
    setup_app_with_shadcn_theme(&mut app);

    let mut ui: UiTree<App> = UiTree::new();
    ui.set_window(window);

    let mut services = StyleAwareServices::default();

    let bounds = Rect::new(
        Point::new(Px(0.0), Px(0.0)),
        CoreSize::new(Px(480.0), Px(400.0)),
    );

    let open = app.models_mut().insert(false);
    let scroll_handle = ScrollHandle::default();

    let trigger_test_id = "scroll-underlay-context-menu-trigger";

    let render = |cx: &mut ElementContext<'_, App>, open: &Model<bool>| {
        use fret_ui_shadcn::context_menu::{ContextMenu, ContextMenuEntry, ContextMenuItem};

        let open = open.clone();
        let scroll_handle = scroll_handle.clone();

        let mut scroll_layout = LayoutStyle::default();
        scroll_layout.size.width = Length::Fill;
        scroll_layout.size.height = Length::Fill;
        scroll_layout.overflow = fret_ui::element::Overflow::Clip;

        vec![cx.scroll(
            fret_ui::element::ScrollProps {
                layout: scroll_layout,
                axis: fret_ui::element::ScrollAxis::Y,
                scroll_handle: Some(scroll_handle),
                ..Default::default()
            },
            move |cx| {
                vec![cx.container(
                    ContainerProps {
                        layout: {
                            let mut layout = LayoutStyle::default();
                            layout.size.width = Length::Fill;
                            layout.size.height = Length::Px(Px(1200.0));
                            layout
                        },
                        ..Default::default()
                    },
                    move |cx| {
                        let menu = ContextMenu::new(open.clone());

                        vec![menu.into_element(
                            cx,
                            move |cx| {
                                cx.semantics(
                                    fret_ui::element::SemanticsProps {
                                        layout: {
                                            let mut layout = LayoutStyle::default();
                                            layout.position =
                                                fret_ui::element::PositionStyle::Absolute;
                                            layout.inset.left = Some(Px(16.0));
                                            layout.inset.top = Some(Px(160.0));
                                            layout.size.width = Length::Px(Px(160.0));
                                            layout.size.height = Length::Px(Px(32.0));
                                            layout
                                        },
                                        role: SemanticsRole::Button,
                                        label: Some(Arc::from("ScrollUnderlayContextTrigger")),
                                        test_id: Some(Arc::from(trigger_test_id)),
                                        ..Default::default()
                                    },
                                    move |cx| {
                                        vec![cx.pressable_with_id(
                                            fret_ui::element::PressableProps {
                                                layout: {
                                                    let mut layout = LayoutStyle::default();
                                                    layout.size.width = Length::Fill;
                                                    layout.size.height = Length::Fill;
                                                    layout
                                                },
                                                enabled: true,
                                                focusable: true,
                                                ..Default::default()
                                            },
                                            |cx, _st, _id| vec![cx.text("Right click")],
                                        )]
                                    },
                                )
                            },
                            move |_cx| {
                                vec![
                                    ContextMenuEntry::Item(ContextMenuItem::new("Item A")),
                                    ContextMenuEntry::Item(ContextMenuItem::new("Item B")),
                                    ContextMenuEntry::Item(ContextMenuItem::new("Item C")),
                                ]
                            },
                        )]
                    },
                )]
            },
        )]
    };

    // Frame 1: mount closed so the trigger element id mapping is stable.
    render_frame(
        &mut ui,
        &mut app,
        &mut services,
        window,
        bounds,
        FrameId(1),
        true,
        |cx| render(cx, &open),
    );

    let snap0 = ui.semantics_snapshot().expect("semantics snapshot").clone();
    let trigger0 = find_semantics_by_test_id(&snap0, trigger_test_id).expect("trigger semantics");
    let trigger_center = Point::new(
        Px(trigger0.bounds.origin.x.0 + trigger0.bounds.size.width.0 * 0.5),
        Px(trigger0.bounds.origin.y.0 + trigger0.bounds.size.height.0 * 0.5),
    );

    // Open via right click to ensure the anchor point is stored (virtual pointer rect behavior).
    ui.dispatch_event(
        &mut app,
        &mut services,
        &Event::Pointer(PointerEvent::Down {
            pointer_id: fret_core::PointerId::default(),
            position: trigger_center,
            button: MouseButton::Right,
            modifiers: Modifiers::default(),
            pointer_type: PointerType::Mouse,
            click_count: 1,
        }),
    );
    ui.dispatch_event(
        &mut app,
        &mut services,
        &Event::Pointer(PointerEvent::Up {
            pointer_id: fret_core::PointerId::default(),
            position: trigger_center,
            button: MouseButton::Right,
            modifiers: Modifiers::default(),
            is_click: true,
            pointer_type: PointerType::Mouse,
            click_count: 1,
        }),
    );

    // Frame 2+: open and settle motion.
    render_frame(
        &mut ui,
        &mut app,
        &mut services,
        window,
        bounds,
        FrameId(2),
        false,
        |cx| render(cx, &open),
    );

    let settle_frames = fret_ui_kit::declarative::overlay_motion::SHADCN_MOTION_TICKS_100 + 2;
    for tick in 0..settle_frames {
        let request_semantics = tick + 1 == settle_frames;
        render_frame(
            &mut ui,
            &mut app,
            &mut services,
            window,
            bounds,
            FrameId(3 + tick),
            request_semantics,
            |cx| render(cx, &open),
        );
    }

    let snap_before = ui
        .semantics_snapshot()
        .expect("semantics snapshot (before scroll)")
        .clone();
    let trigger_before =
        find_semantics_by_test_id(&snap_before, trigger_test_id).expect("trigger semantics");
    let menu_before = snap_before
        .nodes
        .iter()
        .find(|n| n.role == SemanticsRole::Menu)
        .expect("menu semantics (before scroll)");

    // Scroll the underlay (wheel over the scroll viewport, not over the menu panel).
    ui.dispatch_event(
        &mut app,
        &mut services,
        &Event::Pointer(PointerEvent::Wheel {
            pointer_id: fret_core::PointerId::default(),
            position: Point::new(Px(10.0), Px(10.0)),
            delta: Point::new(Px(0.0), Px(-80.0)),
            modifiers: Modifiers::default(),
            pointer_type: PointerType::Mouse,
        }),
    );
    assert!(
        scroll_handle.offset().y.0 > 0.01,
        "expected scroll handle offset to update after wheel; y={}",
        scroll_handle.offset().y.0
    );

    // Frame N: apply the scroll and paint once so scroll transforms update visual bounds caches.
    render_frame(
        &mut ui,
        &mut app,
        &mut services,
        window,
        bounds,
        FrameId(3 + settle_frames),
        false,
        |cx| render(cx, &open),
    );
    let mut scene = fret_core::Scene::default();
    ui.paint_all(&mut app, &mut services, bounds, &mut scene, 1.0);

    let effects = app.flush_effects();
    assert!(
        effects
            .iter()
            .any(|e| matches!(e, Effect::Redraw(w) if *w == window)),
        "expected a follow-up redraw after scroll to re-anchor overlays; effects={effects:?}",
    );
    for effect in effects {
        match effect {
            Effect::Redraw(_) => {}
            other => app.push_effect(other),
        }
    }

    // Frame N+1: the context menu anchor is a virtual pointer rect, so the menu should stay put.
    render_frame(
        &mut ui,
        &mut app,
        &mut services,
        window,
        bounds,
        FrameId(4 + settle_frames),
        true,
        |cx| render(cx, &open),
    );

    let snap_after = ui
        .semantics_snapshot()
        .expect("semantics snapshot (after scroll)")
        .clone();
    let trigger_after =
        find_semantics_by_test_id(&snap_after, trigger_test_id).expect("trigger semantics");
    let menu_after = snap_after
        .nodes
        .iter()
        .find(|n| n.role == SemanticsRole::Menu)
        .expect("menu semantics (after scroll)");

    assert!(
        (trigger_after.bounds.origin.y.0 - trigger_before.bounds.origin.y.0).abs() > 1.0,
        "expected trigger to move under scroll (before_y={} after_y={})",
        trigger_before.bounds.origin.y.0,
        trigger_after.bounds.origin.y.0
    );

    assert_close(
        "context menu origin x stable under scroll",
        menu_after.bounds.origin.x.0,
        menu_before.bounds.origin.x.0,
        1.0,
    );
    assert_close(
        "context menu origin y stable under scroll",
        menu_after.bounds.origin.y.0,
        menu_before.bounds.origin.y.0,
        1.0,
    );
}
#[test]
fn web_vs_fret_context_menu_demo_overlay_placement_matches() {
    assert_point_anchored_overlay_placement_matches(
        "context-menu-demo",
        "menu",
        SemanticsRole::Menu,
        |cx, open| {
            #[derive(Default)]
            struct Models {
                checked_bookmarks: Option<Model<bool>>,
                checked_full_urls: Option<Model<bool>>,
                radio_person: Option<Model<Option<Arc<str>>>>,
            }

            let existing = cx.with_state(Models::default, |st| {
                match (
                    st.checked_bookmarks.as_ref(),
                    st.checked_full_urls.as_ref(),
                    st.radio_person.as_ref(),
                ) {
                    (Some(a), Some(b), Some(c)) => Some((a.clone(), b.clone(), c.clone())),
                    _ => None,
                }
            });

            let (checked_bookmarks, checked_full_urls, radio_person) =
                if let Some(existing) = existing {
                    existing
                } else {
                    let checked_bookmarks = cx.app.models_mut().insert(true);
                    let checked_full_urls = cx.app.models_mut().insert(false);
                    let radio_person = cx.app.models_mut().insert(Some(Arc::from("pedro")));

                    cx.with_state(Models::default, |st| {
                        st.checked_bookmarks = Some(checked_bookmarks.clone());
                        st.checked_full_urls = Some(checked_full_urls.clone());
                        st.radio_person = Some(radio_person.clone());
                    });

                    (checked_bookmarks, checked_full_urls, radio_person)
                };

            fret_ui_shadcn::ContextMenu::new(open.clone())
                // new-york-v4 context-menu-demo: `ContextMenuContent className="w-52"`.
                .min_width(Px(208.0))
                // new-york-v4 context-menu-demo: `ContextMenuSubContent className="w-44"`.
                .submenu_min_width(Px(176.0))
                .into_element(
                cx,
                |cx| {
                    cx.container(
                        ContainerProps {
                            layout: {
                                let mut layout = LayoutStyle::default();
                                layout.size.width = Length::Px(Px(300.0));
                                layout.size.height = Length::Px(Px(150.0));
                                layout
                            },
                            ..Default::default()
                        },
                        |cx| vec![cx.text("Right click here")],
                    )
                },
                |cx| {
                    vec![
                        fret_ui_shadcn::ContextMenuEntry::Item(
                            fret_ui_shadcn::ContextMenuItem::new("Back")
                                .inset(true)
                                .trailing(
                                    fret_ui_shadcn::ContextMenuShortcut::new("⌘[")
                                        .into_element(cx),
                                ),
                        ),
                        fret_ui_shadcn::ContextMenuEntry::Item(
                            fret_ui_shadcn::ContextMenuItem::new("Forward")
                                .inset(true)
                                .disabled(true)
                                .trailing(
                                    fret_ui_shadcn::ContextMenuShortcut::new("⌘]")
                                        .into_element(cx),
                                ),
                        ),
                        fret_ui_shadcn::ContextMenuEntry::Item(
                            fret_ui_shadcn::ContextMenuItem::new("Reload")
                                .inset(true)
                                .trailing(
                                    fret_ui_shadcn::ContextMenuShortcut::new("⌘R")
                                        .into_element(cx),
                                ),
                        ),
                        fret_ui_shadcn::ContextMenuEntry::Item(
                            fret_ui_shadcn::ContextMenuItem::new("More Tools").inset(true).submenu(
                                vec![
                                    fret_ui_shadcn::ContextMenuEntry::Item(
                                        fret_ui_shadcn::ContextMenuItem::new("Save Page..."),
                                    ),
                                    fret_ui_shadcn::ContextMenuEntry::Item(
                                        fret_ui_shadcn::ContextMenuItem::new(
                                            "Create Shortcut...",
                                        ),
                                    ),
                                    fret_ui_shadcn::ContextMenuEntry::Item(
                                        fret_ui_shadcn::ContextMenuItem::new("Name Window..."),
                                    ),
                                    fret_ui_shadcn::ContextMenuEntry::Separator,
                                    fret_ui_shadcn::ContextMenuEntry::Item(
                                        fret_ui_shadcn::ContextMenuItem::new("Developer Tools"),
                                    ),
                                    fret_ui_shadcn::ContextMenuEntry::Separator,
                                    fret_ui_shadcn::ContextMenuEntry::Item(
                                        fret_ui_shadcn::ContextMenuItem::new("Delete").variant(
                                            fret_ui_shadcn::context_menu::ContextMenuItemVariant::Destructive,
                                        ),
                                    ),
                                ],
                            ),
                        ),
                        fret_ui_shadcn::ContextMenuEntry::Separator,
                        fret_ui_shadcn::ContextMenuEntry::CheckboxItem(
                            fret_ui_shadcn::ContextMenuCheckboxItem::new(
                                checked_bookmarks,
                                "Show Bookmarks",
                            ),
                        ),
                        fret_ui_shadcn::ContextMenuEntry::CheckboxItem(
                            fret_ui_shadcn::ContextMenuCheckboxItem::new(
                                checked_full_urls,
                                "Show Full URLs",
                            ),
                        ),
                        fret_ui_shadcn::ContextMenuEntry::Separator,
                        fret_ui_shadcn::ContextMenuEntry::Label(
                            fret_ui_shadcn::ContextMenuLabel::new("People").inset(true),
                        ),
                        fret_ui_shadcn::ContextMenuEntry::RadioGroup(
                            fret_ui_shadcn::ContextMenuRadioGroup::new(radio_person)
                                .item(fret_ui_shadcn::ContextMenuRadioItemSpec::new(
                                    "pedro",
                                    "Pedro Duarte",
                                ))
                                .item(fret_ui_shadcn::ContextMenuRadioItemSpec::new(
                                    "colm",
                                    "Colm Tuite",
                                )),
                        ),
                    ]
                },
            )
        },
        |ui, app, services, _window, point| {
            ui.dispatch_event(
                app,
                services,
                &Event::Pointer(PointerEvent::Down {
                    pointer_id: fret_core::PointerId::default(),
                    position: Point::new(Px(point.x), Px(point.y)),
                    button: MouseButton::Right,
                    modifiers: Modifiers::default(),
                    pointer_type: PointerType::Mouse,
                    click_count: 1,
                }),
            );
            ui.dispatch_event(
                app,
                services,
                &Event::Pointer(PointerEvent::Up {
                    pointer_id: fret_core::PointerId::default(),
                    position: Point::new(Px(point.x), Px(point.y)),
                    button: MouseButton::Right,
                    modifiers: Modifiers::default(),
                    is_click: true,
                    pointer_type: PointerType::Mouse,
                    click_count: 1,
                }),
            );
        },
    );
}
#[test]
fn web_vs_fret_context_menu_demo_small_viewport_overlay_placement_matches() {
    assert_context_menu_demo_constrained_overlay_placement_matches("context-menu-demo.vp1440x320");
}
#[test]
fn web_vs_fret_context_menu_demo_tiny_viewport_overlay_placement_matches() {
    assert_context_menu_demo_constrained_overlay_placement_matches("context-menu-demo.vp1440x240");
}
#[test]
fn web_vs_fret_context_menu_demo_mobile_tiny_viewport_overlay_placement_matches() {
    assert_context_menu_demo_constrained_overlay_placement_matches("context-menu-demo.vp375x240");
}
#[test]
fn web_vs_fret_context_menu_demo_small_viewport_menu_item_height_matches() {
    assert_context_menu_demo_constrained_menu_item_height_matches("context-menu-demo.vp1440x320");
}
#[test]
fn web_vs_fret_context_menu_demo_tiny_viewport_menu_item_height_matches() {
    assert_context_menu_demo_constrained_menu_item_height_matches("context-menu-demo.vp1440x240");
}
#[test]
fn web_vs_fret_context_menu_demo_mobile_tiny_viewport_menu_item_height_matches() {
    assert_context_menu_demo_constrained_menu_item_height_matches("context-menu-demo.vp375x240");
}
#[test]
fn web_vs_fret_context_menu_demo_menu_item_height_matches() {
    assert_context_menu_demo_constrained_menu_item_height_matches("context-menu-demo");
}
#[test]
fn web_vs_fret_context_menu_demo_back_item_padding_and_shortcut_match() {
    assert_context_menu_demo_back_item_padding_and_shortcut_match_impl("context-menu-demo");
}
#[test]
fn web_vs_fret_context_menu_demo_checkbox_indicator_slot_inset_matches_web() {
    assert_context_menu_demo_checkbox_indicator_slot_inset_matches_web_impl("context-menu-demo");
}
#[test]
fn web_vs_fret_context_menu_demo_radio_indicator_slot_inset_matches_web() {
    assert_context_menu_demo_radio_indicator_slot_inset_matches_web_impl("context-menu-demo");
}
#[test]
fn web_vs_fret_context_menu_demo_small_viewport_menu_content_insets_match() {
    assert_context_menu_demo_constrained_menu_content_insets_match("context-menu-demo.vp1440x320");
}
#[test]
fn web_vs_fret_context_menu_demo_tiny_viewport_menu_content_insets_match() {
    assert_context_menu_demo_constrained_menu_content_insets_match("context-menu-demo.vp1440x240");
}
#[test]
fn web_vs_fret_context_menu_demo_mobile_tiny_viewport_menu_content_insets_match() {
    assert_context_menu_demo_constrained_menu_content_insets_match("context-menu-demo.vp375x240");
}
#[test]
fn web_vs_fret_context_menu_demo_small_viewport_scroll_state_matches() {
    assert_context_menu_demo_constrained_scroll_state_matches("context-menu-demo.vp1440x320");
}
#[test]
fn web_vs_fret_context_menu_demo_tiny_viewport_scroll_state_matches() {
    assert_context_menu_demo_constrained_scroll_state_matches("context-menu-demo.vp1440x240");
}
#[test]
fn web_vs_fret_context_menu_demo_mobile_tiny_viewport_scroll_state_matches() {
    assert_context_menu_demo_constrained_scroll_state_matches("context-menu-demo.vp375x240");
}
#[test]
fn web_vs_fret_context_menu_demo_mobile_tiny_viewport_wheel_scroll_matches_web_scrolled_80() {
    assert_context_menu_demo_wheel_scroll_matches_web_scrolled(
        "context-menu-demo.vp375x240-scrolled-80",
        -80.0,
    );
}
#[test]
fn web_vs_fret_context_menu_demo_menu_content_insets_match() {
    assert_context_menu_demo_constrained_menu_content_insets_match("context-menu-demo");
}
#[test]
fn web_vs_fret_context_menu_demo_submenu_overlay_placement_matches() {
    assert_context_menu_demo_submenu_overlay_placement_matches("context-menu-demo.submenu-kbd");
}
#[test]
fn web_vs_fret_context_menu_demo_submenu_hover_overlay_placement_matches() {
    assert_context_menu_demo_submenu_overlay_placement_matches("context-menu-demo.submenu");
}
#[test]
fn web_vs_fret_context_menu_demo_submenu_small_viewport_overlay_placement_matches() {
    assert_context_menu_demo_submenu_overlay_placement_matches(
        "context-menu-demo.submenu-kbd-vp1440x320",
    );
}
#[test]
fn web_vs_fret_context_menu_demo_submenu_tiny_viewport_overlay_placement_matches() {
    assert_context_menu_demo_submenu_overlay_placement_matches(
        "context-menu-demo.submenu-kbd-vp1440x240",
    );
}
#[test]
fn web_vs_fret_context_menu_demo_submenu_mobile_tiny_viewport_overlay_placement_matches() {
    assert_context_menu_demo_submenu_overlay_placement_matches(
        "context-menu-demo.submenu-kbd-vp375x240",
    );
}
#[test]
fn web_vs_fret_context_menu_demo_submenu_menu_content_insets_match() {
    assert_context_menu_demo_submenu_constrained_menu_content_insets_match(
        "context-menu-demo.submenu-kbd",
    );
}
#[test]
fn web_vs_fret_context_menu_demo_submenu_hover_menu_content_insets_match() {
    assert_context_menu_demo_submenu_constrained_menu_content_insets_match(
        "context-menu-demo.submenu",
    );
}
#[test]
fn web_vs_fret_context_menu_demo_submenu_small_viewport_menu_content_insets_match() {
    assert_context_menu_demo_submenu_constrained_menu_content_insets_match(
        "context-menu-demo.submenu-kbd-vp1440x320",
    );
}
#[test]
fn web_vs_fret_context_menu_demo_submenu_tiny_viewport_menu_content_insets_match() {
    assert_context_menu_demo_submenu_constrained_menu_content_insets_match(
        "context-menu-demo.submenu-kbd-vp1440x240",
    );
}
#[test]
fn web_vs_fret_context_menu_demo_submenu_mobile_tiny_viewport_menu_content_insets_match() {
    assert_context_menu_demo_submenu_constrained_menu_content_insets_match(
        "context-menu-demo.submenu-kbd-vp375x240",
    );
}
