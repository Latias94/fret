use super::*;
use serde::Deserialize;

#[derive(Debug, Clone, Deserialize)]
#[serde(rename_all = "snake_case")]
enum ContextMenuDemoRecipe {
    DemoOverlayPlacement,
    ConstrainedOverlayPlacement,
    ConstrainedMenuItemHeight,
    BackItemPaddingAndShortcut,
    CheckboxIndicatorSlotInset,
    RadioIndicatorSlotInset,
    ConstrainedMenuContentInsets,
    ConstrainedScrollState,
    WheelScroll,
    SubmenuOverlayPlacement,
    SubmenuConstrainedMenuContentInsets,
}

#[derive(Debug, Clone, Deserialize)]
struct ContextMenuDemoCase {
    id: String,
    web_name: String,
    recipe: ContextMenuDemoRecipe,
    delta_y: Option<f32>,
}

#[track_caller]
fn build_context_menu_demo_overlay(
    cx: &mut ElementContext<'_, App>,
    open: &Model<bool>,
) -> AnyElement {
    let checked_bookmarks = cx.local_model_keyed("checked_bookmarks", || true);
    let checked_full_urls = cx.local_model_keyed("checked_full_urls", || false);
    let radio_person = cx.local_model_keyed("radio_person", || Some(Arc::from("pedro")));

    use fret_ui_shadcn::{
        ContextMenu, ContextMenuCheckboxItem, ContextMenuEntry, ContextMenuItem, ContextMenuLabel,
        ContextMenuRadioGroup, ContextMenuRadioItemSpec, ContextMenuShortcut,
    };

    ContextMenu::new(open.clone())
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
                    ContextMenuEntry::Item(
                        ContextMenuItem::new("Back")
                            .inset(true)
                            .trailing(ContextMenuShortcut::new("⌘[").into_element(cx)),
                    ),
                    ContextMenuEntry::Item(
                        ContextMenuItem::new("Forward")
                            .inset(true)
                            .disabled(true)
                            .trailing(ContextMenuShortcut::new("⌘]").into_element(cx)),
                    ),
                    ContextMenuEntry::Item(
                        ContextMenuItem::new("Reload")
                            .inset(true)
                            .trailing(ContextMenuShortcut::new("⌘R").into_element(cx)),
                    ),
                    ContextMenuEntry::Item(ContextMenuItem::new("More Tools").inset(true).submenu(
                        vec![
                            ContextMenuEntry::Item(ContextMenuItem::new("Save Page...")),
                            ContextMenuEntry::Item(ContextMenuItem::new("Create Shortcut...")),
                            ContextMenuEntry::Item(ContextMenuItem::new("Name Window...")),
                            ContextMenuEntry::Separator,
                            ContextMenuEntry::Item(ContextMenuItem::new("Developer Tools")),
                            ContextMenuEntry::Separator,
                            ContextMenuEntry::Item(ContextMenuItem::new("Delete").variant(
                                fret_ui_shadcn::context_menu::ContextMenuItemVariant::Destructive,
                            )),
                        ],
                    )),
                    ContextMenuEntry::Separator,
                    ContextMenuEntry::CheckboxItem(ContextMenuCheckboxItem::new(
                        checked_bookmarks,
                        "Show Bookmarks",
                    )),
                    ContextMenuEntry::CheckboxItem(ContextMenuCheckboxItem::new(
                        checked_full_urls,
                        "Show Full URLs",
                    )),
                    ContextMenuEntry::Separator,
                    ContextMenuEntry::Label(ContextMenuLabel::new("People").inset(true)),
                    ContextMenuEntry::RadioGroup(
                        ContextMenuRadioGroup::new(radio_person)
                            .item(ContextMenuRadioItemSpec::new("pedro", "Pedro Duarte"))
                            .item(ContextMenuRadioItemSpec::new("colm", "Colm Tuite")),
                    ),
                ]
            },
        )
}

#[test]
fn web_vs_fret_context_menu_demo_cases_match_web_fixtures() {
    let raw = include_str!(concat!(
        env!("CARGO_MANIFEST_DIR"),
        "/tests/fixtures/overlay_placement_context_menu_demo_cases_v1.json"
    ));
    let suite: FixtureSuite<ContextMenuDemoCase> =
        serde_json::from_str(raw).expect("context-menu-demo fixture parse");
    assert_eq!(suite.schema_version, 1);
    assert!(!suite.cases.is_empty());

    for case in suite.cases {
        eprintln!("context-menu-demo case={}", case.id);
        match case.recipe {
            ContextMenuDemoRecipe::DemoOverlayPlacement => {
                assert_point_anchored_overlay_placement_matches(
                    &case.web_name,
                    "menu",
                    SemanticsRole::Menu,
                    |cx, open| build_context_menu_demo_overlay(cx, open),
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
            ContextMenuDemoRecipe::ConstrainedOverlayPlacement => {
                assert_context_menu_demo_constrained_overlay_placement_matches(&case.web_name);
            }
            ContextMenuDemoRecipe::ConstrainedMenuItemHeight => {
                assert_context_menu_demo_constrained_menu_item_height_matches(&case.web_name);
            }
            ContextMenuDemoRecipe::BackItemPaddingAndShortcut => {
                assert_context_menu_demo_back_item_padding_and_shortcut_match_impl(&case.web_name);
            }
            ContextMenuDemoRecipe::CheckboxIndicatorSlotInset => {
                assert_context_menu_demo_checkbox_indicator_slot_inset_matches_web_impl(
                    &case.web_name,
                );
            }
            ContextMenuDemoRecipe::RadioIndicatorSlotInset => {
                assert_context_menu_demo_radio_indicator_slot_inset_matches_web_impl(
                    &case.web_name,
                );
            }
            ContextMenuDemoRecipe::ConstrainedMenuContentInsets => {
                assert_context_menu_demo_constrained_menu_content_insets_match(&case.web_name);
            }
            ContextMenuDemoRecipe::ConstrainedScrollState => {
                assert_context_menu_demo_constrained_scroll_state_matches(&case.web_name);
            }
            ContextMenuDemoRecipe::WheelScroll => {
                let delta_y = case.delta_y.unwrap_or_else(|| {
                    panic!("missing delta_y for recipe wheel_scroll: {}", case.id)
                });
                assert_context_menu_demo_wheel_scroll_matches_web_scrolled(&case.web_name, delta_y);
            }
            ContextMenuDemoRecipe::SubmenuOverlayPlacement => {
                assert_context_menu_demo_submenu_overlay_placement_matches(&case.web_name);
            }
            ContextMenuDemoRecipe::SubmenuConstrainedMenuContentInsets => {
                assert_context_menu_demo_submenu_constrained_menu_content_insets_match(
                    &case.web_name,
                );
            }
        }
    }
}
