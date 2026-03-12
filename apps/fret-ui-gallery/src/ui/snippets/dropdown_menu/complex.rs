pub const SOURCE: &str = include_str!("complex.rs");

// region: example
use fret_core::Px;
use fret_ui_kit::IntoUiElement;
use fret_ui_kit::declarative::ModelWatchExt as _;
use fret_ui_shadcn::{facade as shadcn, prelude::*};
use std::sync::Arc;

#[derive(Clone)]
struct ComplexMenuState {
    show_sidebar: bool,
    show_status_bar: bool,
    push_notifications: bool,
    email_notifications: bool,
    theme: Option<Arc<str>>,
}

impl Default for ComplexMenuState {
    fn default() -> Self {
        Self {
            show_sidebar: true,
            show_status_bar: false,
            push_notifications: true,
            email_notifications: true,
            theme: Some(Arc::<str>::from("light")),
        }
    }
}

pub fn render<H: UiHost>(cx: &mut ElementContext<'_, H>) -> AnyElement {
    let menu_state = cx.local_model(ComplexMenuState::default);
    let menu_state_now = cx
        .watch_model(&menu_state)
        .layout()
        .cloned()
        .unwrap_or_default();

    super::preview_frame_with(cx, |cx| {
        shadcn::DropdownMenu::new_controllable(cx, None, false).build_parts(
            cx,
            shadcn::DropdownMenuTrigger::build(
                shadcn::Button::new("Complex Menu")
                    .variant(shadcn::ButtonVariant::Outline)
                    .test_id("ui-gallery-dropdown-menu-complex-trigger"),
            ),
            shadcn::DropdownMenuContent::new()
                .align(shadcn::DropdownMenuAlign::Start)
                .side_offset(Px(4.0))
                // shadcn/ui docs: `DropdownMenuContent className="w-44"`.
                .min_width(Px(176.0)),
            |cx| {
                [
                    shadcn::DropdownMenuGroup::new([
                        shadcn::DropdownMenuLabel::new("File").into(),
                        shadcn::DropdownMenuItem::new("New File")
                            .leading_icon(IconId::new_static("lucide.file"))
                            .trailing(shadcn::DropdownMenuShortcut::new("⌘N").into_element(cx))
                            .into(),
                        shadcn::DropdownMenuItem::new("New Folder")
                            .leading_icon(IconId::new_static("lucide.folder"))
                            .trailing(shadcn::DropdownMenuShortcut::new("⇧⌘N").into_element(cx))
                            .into(),
                        shadcn::DropdownMenuSub::new(
                            shadcn::DropdownMenuSubTrigger::new("Open Recent").refine(|item| {
                                item.leading_icon(IconId::new_static("lucide.folder-open"))
                            }),
                            shadcn::DropdownMenuSubContent::new([
                                shadcn::DropdownMenuGroup::new([
                                    shadcn::DropdownMenuLabel::new("Recent Projects").into(),
                                    shadcn::DropdownMenuItem::new("Project Alpha")
                                        .leading_icon(IconId::new_static("lucide.file-code"))
                                        .into(),
                                    shadcn::DropdownMenuItem::new("Project Beta")
                                        .leading_icon(IconId::new_static("lucide.file-code"))
                                        .into(),
                                    shadcn::DropdownMenuSub::new(
                                        shadcn::DropdownMenuSubTrigger::new("More Projects")
                                            .refine(|item| {
                                                item.leading_icon(IconId::new_static(
                                                    "lucide.more-horizontal",
                                                ))
                                            }),
                                        shadcn::DropdownMenuSubContent::new([
                                            shadcn::DropdownMenuItem::new("Project Gamma")
                                                .leading_icon(IconId::new_static(
                                                    "lucide.file-code",
                                                ))
                                                .into(),
                                            shadcn::DropdownMenuItem::new("Project Delta")
                                                .leading_icon(IconId::new_static(
                                                    "lucide.file-code",
                                                ))
                                                .into(),
                                        ]),
                                    )
                                    .into(),
                                ])
                                .into(),
                                shadcn::DropdownMenuSeparator::new().into(),
                                shadcn::DropdownMenuGroup::new([shadcn::DropdownMenuItem::new(
                                    "Browse...",
                                )
                                .leading_icon(IconId::new_static("lucide.folder-search"))
                                .into()])
                                .into(),
                            ]),
                        )
                        .into(),
                        shadcn::DropdownMenuSeparator::new().into(),
                        shadcn::DropdownMenuItem::new("Save")
                            .leading_icon(IconId::new_static("lucide.save"))
                            .trailing(shadcn::DropdownMenuShortcut::new("⌘S").into_element(cx))
                            .into(),
                        shadcn::DropdownMenuItem::new("Export")
                            .leading_icon(IconId::new_static("lucide.download"))
                            .trailing(shadcn::DropdownMenuShortcut::new("⇧⌘E").into_element(cx))
                            .into(),
                    ])
                    .into(),
                    shadcn::DropdownMenuSeparator::new().into(),
                    shadcn::DropdownMenuGroup::new([
                        shadcn::DropdownMenuLabel::new("View").into(),
                        shadcn::DropdownMenuCheckboxItem::from_checked(
                            menu_state_now.show_sidebar,
                            "Show Sidebar",
                        )
                        .on_checked_change({
                            let menu_state = menu_state.clone();
                            move |host, _action_cx, checked| {
                                let _ = host
                                    .models_mut()
                                    .update(&menu_state, |state| state.show_sidebar = checked);
                            }
                        })
                        .leading_icon(IconId::new_static("lucide.eye"))
                        .into(),
                        shadcn::DropdownMenuCheckboxItem::from_checked(
                            menu_state_now.show_status_bar,
                            "Show Status Bar",
                        )
                        .on_checked_change({
                            let menu_state = menu_state.clone();
                            move |host, _action_cx, checked| {
                                let _ = host
                                    .models_mut()
                                    .update(&menu_state, |state| state.show_status_bar = checked);
                            }
                        })
                        .leading_icon(IconId::new_static("lucide.layout"))
                        .into(),
                        shadcn::DropdownMenuSub::new(
                            shadcn::DropdownMenuSubTrigger::new("Theme").refine(|item| {
                                item.leading_icon(IconId::new_static("lucide.palette"))
                            }),
                            shadcn::DropdownMenuSubContent::new([shadcn::DropdownMenuGroup::new(
                                [
                                    shadcn::DropdownMenuLabel::new("Appearance").into(),
                                    shadcn::DropdownMenuRadioGroup::from_value(
                                        menu_state_now.theme.clone(),
                                    )
                                    .on_value_change({
                                        let menu_state = menu_state.clone();
                                        move |host, _action_cx, value| {
                                            let _ =
                                                host.models_mut().update(&menu_state, |state| {
                                                    state.theme = Some(value)
                                                });
                                        }
                                    })
                                    .item(
                                        shadcn::DropdownMenuRadioItemSpec::new("light", "Light")
                                            .leading_icon(IconId::new_static("lucide.sun")),
                                    )
                                    .item(
                                        shadcn::DropdownMenuRadioItemSpec::new("dark", "Dark")
                                            .leading_icon(IconId::new_static("lucide.moon")),
                                    )
                                    .item(
                                        shadcn::DropdownMenuRadioItemSpec::new("system", "System")
                                            .leading_icon(IconId::new_static("lucide.monitor")),
                                    )
                                    .into(),
                                ],
                            )
                            .into()]),
                        )
                        .into(),
                    ])
                    .into(),
                    shadcn::DropdownMenuSeparator::new().into(),
                    shadcn::DropdownMenuGroup::new([
                        shadcn::DropdownMenuLabel::new("Account").into(),
                        shadcn::DropdownMenuItem::new("Profile")
                            .leading_icon(IconId::new_static("lucide.user"))
                            .trailing(shadcn::DropdownMenuShortcut::new("⇧⌘P").into_element(cx))
                            .into(),
                        shadcn::DropdownMenuItem::new("Billing")
                            .leading_icon(IconId::new_static("lucide.credit-card"))
                            .into(),
                        shadcn::DropdownMenuSub::new(
                            shadcn::DropdownMenuSubTrigger::new("Settings").refine(|item| {
                                item.leading_icon(IconId::new_static("lucide.settings"))
                            }),
                            shadcn::DropdownMenuSubContent::new([
                                shadcn::DropdownMenuGroup::new([
                                    shadcn::DropdownMenuLabel::new("Preferences").into(),
                                    shadcn::DropdownMenuItem::new("Keyboard Shortcuts")
                                        .leading_icon(IconId::new_static("lucide.keyboard"))
                                        .into(),
                                    shadcn::DropdownMenuItem::new("Language")
                                        .leading_icon(IconId::new_static("lucide.languages"))
                                        .into(),
                                    shadcn::DropdownMenuSub::new(
                                        shadcn::DropdownMenuSubTrigger::new("Notifications")
                                            .refine(|item| {
                                                item.leading_icon(IconId::new_static("lucide.bell"))
                                            }),
                                        shadcn::DropdownMenuSubContent::new([
                                            shadcn::DropdownMenuGroup::new([
                                                shadcn::DropdownMenuLabel::new(
                                                    "Notification Types",
                                                )
                                                .into(),
                                                shadcn::DropdownMenuCheckboxItem::from_checked(
                                                    menu_state_now.push_notifications,
                                                    "Push Notifications",
                                                )
                                                .on_checked_change({
                                                    let menu_state = menu_state.clone();
                                                    move |host, _action_cx, checked| {
                                                        let _ = host.models_mut().update(
                                                            &menu_state,
                                                            |state| {
                                                                state.push_notifications = checked
                                                            },
                                                        );
                                                    }
                                                })
                                                .leading_icon(IconId::new_static("lucide.bell"))
                                                .into(),
                                                shadcn::DropdownMenuCheckboxItem::from_checked(
                                                    menu_state_now.email_notifications,
                                                    "Email Notifications",
                                                )
                                                .on_checked_change({
                                                    let menu_state = menu_state.clone();
                                                    move |host, _action_cx, checked| {
                                                        let _ = host.models_mut().update(
                                                            &menu_state,
                                                            |state| {
                                                                state.email_notifications = checked
                                                            },
                                                        );
                                                    }
                                                })
                                                .leading_icon(IconId::new_static("lucide.mail"))
                                                .into(),
                                            ])
                                            .into(),
                                        ]),
                                    )
                                    .into(),
                                ])
                                .into(),
                                shadcn::DropdownMenuSeparator::new().into(),
                                shadcn::DropdownMenuGroup::new([shadcn::DropdownMenuItem::new(
                                    "Privacy & Security",
                                )
                                .leading_icon(IconId::new_static("lucide.shield"))
                                .into()])
                                .into(),
                            ]),
                        )
                        .into(),
                    ])
                    .into(),
                    shadcn::DropdownMenuSeparator::new().into(),
                    shadcn::DropdownMenuGroup::new([
                        shadcn::DropdownMenuItem::new("Help & Support")
                            .leading_icon(IconId::new_static("lucide.help-circle"))
                            .into(),
                        shadcn::DropdownMenuItem::new("Documentation")
                            .leading_icon(IconId::new_static("lucide.file-text"))
                            .into(),
                    ])
                    .into(),
                    shadcn::DropdownMenuSeparator::new().into(),
                    shadcn::DropdownMenuGroup::new([shadcn::DropdownMenuItem::new("Sign Out")
                        .leading_icon(IconId::new_static("lucide.log-out"))
                        .trailing(shadcn::DropdownMenuShortcut::new("⇧⌘Q").into_element(cx))
                        .variant(
                            fret_ui_shadcn::dropdown_menu::DropdownMenuItemVariant::Destructive,
                        )
                        .into()])
                    .into(),
                ]
            },
        )
    })
    .into_element(cx)
}
// endregion: example
