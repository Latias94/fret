pub const SOURCE: &str = include_str!("complex.rs");

// region: example
use fret_core::Px;
use fret_ui_shadcn::{self as shadcn, prelude::*};
use std::sync::Arc;

#[derive(Default, Clone)]
struct Models {
    show_sidebar: Option<Model<bool>>,
    show_status_bar: Option<Model<bool>>,
    push_notifications: Option<Model<bool>>,
    email_notifications: Option<Model<bool>>,
    theme: Option<Model<Option<Arc<str>>>>,
}

pub fn render<H: UiHost>(cx: &mut ElementContext<'_, H>) -> AnyElement {
    let state = cx.with_state(Models::default, |st| st.clone());

    let show_sidebar = match state.show_sidebar {
        Some(model) => model,
        None => {
            let model = cx.app.models_mut().insert(true);
            cx.with_state(Models::default, |st| st.show_sidebar = Some(model.clone()));
            model
        }
    };

    let show_status_bar = match state.show_status_bar {
        Some(model) => model,
        None => {
            let model = cx.app.models_mut().insert(false);
            cx.with_state(Models::default, |st| {
                st.show_status_bar = Some(model.clone())
            });
            model
        }
    };

    let push_notifications = match state.push_notifications {
        Some(model) => model,
        None => {
            let model = cx.app.models_mut().insert(true);
            cx.with_state(Models::default, |st| {
                st.push_notifications = Some(model.clone())
            });
            model
        }
    };

    let email_notifications = match state.email_notifications {
        Some(model) => model,
        None => {
            let model = cx.app.models_mut().insert(true);
            cx.with_state(Models::default, |st| {
                st.email_notifications = Some(model.clone())
            });
            model
        }
    };

    let theme = match state.theme {
        Some(model) => model,
        None => {
            let model = cx.app.models_mut().insert(Some(Arc::<str>::from("light")));
            cx.with_state(Models::default, |st| st.theme = Some(model.clone()));
            model
        }
    };

    shadcn::DropdownMenu::new_controllable(cx, None, false)
        .into_element_parts(
            cx,
            |cx| {
                shadcn::DropdownMenuTrigger::new(
                    shadcn::Button::new("Complex Menu")
                        .variant(shadcn::ButtonVariant::Outline)
                        .test_id("ui-gallery-dropdown-menu-complex-trigger")
                        .into_element(cx),
                )
            },
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
                        shadcn::DropdownMenuCheckboxItem::new(show_sidebar.clone(), "Show Sidebar")
                            .leading_icon(IconId::new_static("lucide.eye"))
                            .into(),
                        shadcn::DropdownMenuCheckboxItem::new(
                            show_status_bar.clone(),
                            "Show Status Bar",
                        )
                        .leading_icon(IconId::new_static("lucide.layout"))
                        .into(),
                        shadcn::DropdownMenuSub::new(
                            shadcn::DropdownMenuSubTrigger::new("Theme").refine(|item| {
                                item.leading_icon(IconId::new_static("lucide.palette"))
                            }),
                            shadcn::DropdownMenuSubContent::new([shadcn::DropdownMenuGroup::new(
                                [
                                    shadcn::DropdownMenuLabel::new("Appearance").into(),
                                    shadcn::DropdownMenuRadioGroup::new(theme.clone())
                                        .item(
                                            shadcn::DropdownMenuRadioItemSpec::new(
                                                "light", "Light",
                                            )
                                            .leading_icon(IconId::new_static("lucide.sun")),
                                        )
                                        .item(
                                            shadcn::DropdownMenuRadioItemSpec::new("dark", "Dark")
                                                .leading_icon(IconId::new_static("lucide.moon")),
                                        )
                                        .item(
                                            shadcn::DropdownMenuRadioItemSpec::new(
                                                "system", "System",
                                            )
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
                                                shadcn::DropdownMenuCheckboxItem::new(
                                                    push_notifications.clone(),
                                                    "Push Notifications",
                                                )
                                                .leading_icon(IconId::new_static("lucide.bell"))
                                                .into(),
                                                shadcn::DropdownMenuCheckboxItem::new(
                                                    email_notifications.clone(),
                                                    "Email Notifications",
                                                )
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
                        .variant(shadcn::dropdown_menu::DropdownMenuItemVariant::Destructive)
                        .into()])
                    .into(),
                ]
            },
        )
        .test_id("ui-gallery-dropdown-menu-complex")
}
// endregion: example
