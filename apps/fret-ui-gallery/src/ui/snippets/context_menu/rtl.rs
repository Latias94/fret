pub const SOURCE: &str = include_str!("rtl.rs");

// region: example
use fret::{UiChild, UiCx};
use fret_core::Px;
use fret_core::scene::DashPatternV1;
use fret_icons::IconId;
use fret_runtime::CommandId;
use fret_ui::{Invalidation, Theme};
use fret_ui_kit::IntoUiElement;
use fret_ui_kit::declarative::ModelWatchExt as _;
use fret_ui_kit::declarative::primary_pointer_is_coarse;
use fret_ui_kit::{ChromeRefinement, ColorRef, LayoutRefinement, Radius, ui};
use fret_ui_shadcn::{facade as shadcn, prelude::*};
use std::sync::Arc;

#[derive(Default, Clone)]
struct RtlMenuState {
    show_bookmarks: bool,
    show_full_urls: bool,
    person: Option<Arc<str>>,
}

fn trigger_surface<H: UiHost>(
    fine_label: &'static str,
    coarse_label: &'static str,
    test_id: &'static str,
) -> impl IntoUiElement<H> + use<H> {
    ui::h_flex(move |cx| {
        let theme = Theme::global(&*cx.app);
        let border = theme.color_token("border");
        let bg = theme.color_token("background");
        let fg = theme.color_token("muted-foreground");
        let label = if primary_pointer_is_coarse(cx, Invalidation::Layout, false) {
            coarse_label
        } else {
            fine_label
        };

        let label = ui::text(label)
            .text_sm()
            .text_color(ColorRef::Color(fg))
            .into_element(cx);

        let content = ui::v_flex(move |_cx| vec![label])
            .layout(LayoutRefinement::default().w_full().h_full())
            .items_center()
            .justify_center()
            .into_element(cx);

        [shadcn::AspectRatio::with_child(content)
            .ratio(16.0 / 9.0)
            .refine_style(
                ChromeRefinement::default()
                    .rounded(Radius::Lg)
                    .border_1()
                    .border_dash(DashPatternV1::new(Px(4.0), Px(4.0), Px(0.0)))
                    .border_color(ColorRef::Color(border))
                    .bg(ColorRef::Color(bg)),
            )
            .refine_layout(LayoutRefinement::default().w_full().max_w(Px(320.0)))
            .into_element(cx)
            .test_id(test_id)]
    })
    .layout(LayoutRefinement::default().w_full().min_w_0())
    .justify_center()
}

pub fn render(cx: &mut UiCx<'_>) -> impl UiChild + use<> {
    let menu_state = cx.local_model(|| RtlMenuState {
        show_bookmarks: true,
        show_full_urls: false,
        person: Some(Arc::<str>::from("pedro")),
    });
    let menu_state_now = cx
        .watch_model(&menu_state)
        .layout()
        .cloned()
        .unwrap_or_default();

    with_direction_provider(cx, LayoutDirection::Rtl, move |cx| {
        shadcn::ContextMenu::uncontrolled(cx)
            .content_test_id("ui-gallery-context-menu-rtl-content")
            .compose()
            .trigger(trigger_surface(
                "Right click here",
                "Long press here",
                "ui-gallery-context-menu-rtl-trigger",
            ))
            .content(
                shadcn::ContextMenuContent::new()
                    .side(shadcn::DropdownMenuSide::InlineEnd)
                    .min_width(Px(192.0))
                    .submenu_min_width(Px(176.0)),
            )
            .entries_with({
                let menu_state = menu_state.clone();
                move |cx| {
                    vec![
                        shadcn::ContextMenuEntry::Group(shadcn::ContextMenuGroup::new(vec![
                            shadcn::ContextMenuEntry::Item(
                                shadcn::ContextMenuItem::new("Navigation")
                                    .test_id("ui-gallery-context-menu-rtl-item-navigation")
                                    .submenu([
                                        shadcn::ContextMenuEntry::Group(
                                            shadcn::ContextMenuGroup::new(vec![
                                                shadcn::ContextMenuEntry::Item(
                                                    shadcn::ContextMenuItem::new("Back")
                                                        .leading_icon(IconId::new_static(
                                                            "lucide.arrow-left",
                                                        ))
                                                        .action(CommandId::new(
                                                            "ui_gallery.context_menu.rtl.back",
                                                        ))
                                                        .test_id(
                                                            "ui-gallery-context-menu-rtl-item-back",
                                                        )
                                                        .trailing(
                                                            shadcn::ContextMenuShortcut::new("⌘[")
                                                                .into_element(cx),
                                                        ),
                                                ),
                                                shadcn::ContextMenuEntry::Item(
                                                    shadcn::ContextMenuItem::new("Forward")
                                                        .leading_icon(IconId::new_static(
                                                            "lucide.arrow-right",
                                                        ))
                                                        .disabled(true)
                                                        .action(CommandId::new(
                                                            "ui_gallery.context_menu.rtl.forward",
                                                        ))
                                                        .test_id(
                                                            "ui-gallery-context-menu-rtl-item-forward",
                                                        )
                                                        .trailing(
                                                            shadcn::ContextMenuShortcut::new("⌘]")
                                                                .into_element(cx),
                                                        ),
                                                ),
                                                shadcn::ContextMenuEntry::Item(
                                                    shadcn::ContextMenuItem::new("Reload")
                                                        .leading_icon(IconId::new_static(
                                                            "lucide.rotate-cw",
                                                        ))
                                                        .action(CommandId::new(
                                                            "ui_gallery.context_menu.rtl.reload",
                                                        ))
                                                        .test_id(
                                                            "ui-gallery-context-menu-rtl-item-reload",
                                                        )
                                                        .trailing(
                                                            shadcn::ContextMenuShortcut::new("⌘R")
                                                                .into_element(cx),
                                                        ),
                                                ),
                                            ]),
                                        ),
                                    ]),
                            ),
                            shadcn::ContextMenuEntry::Item(
                                shadcn::ContextMenuItem::new("More Tools")
                                    .test_id("ui-gallery-context-menu-rtl-item-more-tools")
                                    .submenu([
                                        shadcn::ContextMenuEntry::Group(
                                            shadcn::ContextMenuGroup::new(vec![
                                                shadcn::ContextMenuEntry::Item(
                                                    shadcn::ContextMenuItem::new("Save Page...")
                                                        .action(CommandId::new(
                                                            "ui_gallery.context_menu.rtl.save_page",
                                                        ))
                                                        .test_id(
                                                            "ui-gallery-context-menu-rtl-item-save-page",
                                                        ),
                                                ),
                                                shadcn::ContextMenuEntry::Item(
                                                    shadcn::ContextMenuItem::new(
                                                        "Create Shortcut...",
                                                    )
                                                    .action(CommandId::new(
                                                        "ui_gallery.context_menu.rtl.create_shortcut",
                                                    ))
                                                    .test_id(
                                                        "ui-gallery-context-menu-rtl-item-create-shortcut",
                                                    ),
                                                ),
                                                shadcn::ContextMenuEntry::Item(
                                                    shadcn::ContextMenuItem::new("Name Window...")
                                                        .action(CommandId::new(
                                                            "ui_gallery.context_menu.rtl.name_window",
                                                        ))
                                                        .test_id(
                                                            "ui-gallery-context-menu-rtl-item-name-window",
                                                        ),
                                                ),
                                            ]),
                                        ),
                                        shadcn::ContextMenuEntry::Separator,
                                        shadcn::ContextMenuEntry::Group(
                                            shadcn::ContextMenuGroup::new(vec![
                                                shadcn::ContextMenuEntry::Item(
                                                    shadcn::ContextMenuItem::new("Developer Tools")
                                                        .action(CommandId::new(
                                                            "ui_gallery.context_menu.rtl.developer_tools",
                                                        ))
                                                        .test_id(
                                                            "ui-gallery-context-menu-rtl-item-developer-tools",
                                                        ),
                                                ),
                                            ]),
                                        ),
                                        shadcn::ContextMenuEntry::Separator,
                                        shadcn::ContextMenuEntry::Group(
                                            shadcn::ContextMenuGroup::new(vec![
                                                shadcn::ContextMenuEntry::Item(
                                                    shadcn::ContextMenuItem::new("Delete")
                                                        .action(CommandId::new(
                                                            "ui_gallery.context_menu.rtl.delete",
                                                        ))
                                                        .test_id(
                                                            "ui-gallery-context-menu-rtl-item-delete",
                                                        )
                                                        .variant(
                                                            shadcn::raw::context_menu::ContextMenuItemVariant::Destructive,
                                                        ),
                                                ),
                                            ]),
                                        ),
                                    ]),
                            ),
                        ])),
                        shadcn::ContextMenuEntry::Separator,
                        shadcn::ContextMenuEntry::Group(shadcn::ContextMenuGroup::new(vec![
                            shadcn::ContextMenuEntry::CheckboxItem(
                                shadcn::ContextMenuCheckboxItem::from_checked(
                                    menu_state_now.show_bookmarks,
                                    "Show Bookmarks",
                                )
                                .on_checked_change({
                                    let menu_state = menu_state.clone();
                                    move |host, _action_cx, checked| {
                                        let _ = host.models_mut().update(&menu_state, |state| {
                                            state.show_bookmarks = checked;
                                        });
                                    }
                                })
                                .action(CommandId::new(
                                    "ui_gallery.context_menu.rtl.show_bookmarks",
                                ))
                                .test_id("ui-gallery-context-menu-rtl-item-show-bookmarks"),
                            ),
                            shadcn::ContextMenuEntry::CheckboxItem(
                                shadcn::ContextMenuCheckboxItem::from_checked(
                                    menu_state_now.show_full_urls,
                                    "Show Full URLs",
                                )
                                .on_checked_change({
                                    let menu_state = menu_state.clone();
                                    move |host, _action_cx, checked| {
                                        let _ = host.models_mut().update(&menu_state, |state| {
                                            state.show_full_urls = checked;
                                        });
                                    }
                                })
                                .action(CommandId::new(
                                    "ui_gallery.context_menu.rtl.show_full_urls",
                                ))
                                .test_id("ui-gallery-context-menu-rtl-item-show-full-urls"),
                            ),
                        ])),
                        shadcn::ContextMenuEntry::Separator,
                        shadcn::ContextMenuEntry::Group(shadcn::ContextMenuGroup::new(vec![
                            shadcn::ContextMenuEntry::Label(shadcn::ContextMenuLabel::new(
                                "People",
                            )),
                            shadcn::ContextMenuEntry::RadioGroup(
                                shadcn::ContextMenuRadioGroup::from_value(
                                    menu_state_now.person.clone(),
                                )
                                .on_value_change({
                                    let menu_state = menu_state.clone();
                                    move |host, _action_cx, value| {
                                        let _ = host
                                            .models_mut()
                                            .update(&menu_state, |state| state.person = Some(value));
                                    }
                                })
                                .item(
                                    shadcn::ContextMenuRadioItemSpec::new(
                                        "pedro",
                                        "Pedro Duarte",
                                    )
                                    .action(CommandId::new(
                                        "ui_gallery.context_menu.rtl.person.pedro",
                                    ))
                                    .test_id("ui-gallery-context-menu-rtl-item-person-pedro"),
                                )
                                .item(
                                    shadcn::ContextMenuRadioItemSpec::new("colm", "Colm Tuite")
                                        .action(CommandId::new(
                                            "ui_gallery.context_menu.rtl.person.colm",
                                        ))
                                        .test_id("ui-gallery-context-menu-rtl-item-person-colm"),
                                ),
                            ),
                        ])),
                    ]
                }
            })
    })
    .test_id("ui-gallery-context-menu-rtl")
}
// endregion: example
