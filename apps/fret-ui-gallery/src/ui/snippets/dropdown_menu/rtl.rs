pub const SOURCE: &str = include_str!("rtl.rs");

// region: example
use fret::{UiChild, UiCx};
use fret_core::Px;
use fret_ui_kit::IntoUiElement;
use fret_ui_kit::declarative::ModelWatchExt as _;
use fret_ui_shadcn::{facade as shadcn, prelude::*};
use std::sync::Arc;

#[derive(Default, Clone)]
struct RtlMenuState {
    show_status_bar: bool,
    show_activity_bar: bool,
    show_panel: bool,
    position: Option<Arc<str>>,
}

pub fn render(cx: &mut UiCx<'_>) -> impl UiChild + use<> {
    let menu_state = cx.local_model(|| RtlMenuState {
        show_status_bar: true,
        show_activity_bar: false,
        show_panel: false,
        position: Some(Arc::<str>::from("bottom")),
    });
    let menu_state_now = cx
        .watch_model(&menu_state)
        .layout()
        .cloned()
        .unwrap_or_default();

    super::preview_frame_with(cx, move |cx| {
        with_direction_provider(cx, LayoutDirection::Rtl, move |cx| {
            shadcn::DropdownMenu::uncontrolled(cx)
                .compose()
                .trigger(
                    shadcn::Button::new("افتح القائمة")
                        .variant(shadcn::ButtonVariant::Outline)
                        .test_id("ui-gallery-dropdown-menu-rtl-trigger"),
                )
                .content(
                    shadcn::DropdownMenuContent::new()
                        .align(shadcn::DropdownMenuAlign::End)
                        .side_offset(Px(4.0))
                        .min_width(Px(144.0)),
                )
                .entries([
                    shadcn::DropdownMenuGroup::new([shadcn::DropdownMenuSub::new(
                        shadcn::DropdownMenuSubTrigger::new("الحساب").refine(|item| {
                            item.test_id("ui-gallery-dropdown-menu-rtl-item-account")
                        }),
                        shadcn::DropdownMenuSubContent::new([shadcn::DropdownMenuGroup::new([
                            shadcn::DropdownMenuItem::new("الملف الشخصي")
                                .leading_icon(IconId::new_static("lucide.user"))
                                .test_id("ui-gallery-dropdown-menu-rtl-item-profile")
                                .into(),
                            shadcn::DropdownMenuItem::new("الفوترة")
                                .leading_icon(IconId::new_static("lucide.credit-card"))
                                .into(),
                            shadcn::DropdownMenuItem::new("الإعدادات")
                                .leading_icon(IconId::new_static("lucide.settings"))
                                .into(),
                        ])
                        .into()]),
                    )
                    .into()])
                    .into(),
                    shadcn::DropdownMenuSeparator::new().into(),
                    shadcn::DropdownMenuGroup::new([
                        shadcn::DropdownMenuLabel::new("الفريق").into(),
                        shadcn::DropdownMenuItem::new("الفريق").into(),
                        shadcn::DropdownMenuSub::new(
                            shadcn::DropdownMenuSubTrigger::new("دعوة المستخدمين").refine(|item| {
                                item.test_id("ui-gallery-dropdown-menu-rtl-item-invite-users")
                            }),
                            shadcn::DropdownMenuSubContent::new([
                                shadcn::DropdownMenuItem::new("البريد الإلكتروني")
                                    .test_id("ui-gallery-dropdown-menu-rtl-item-email")
                                    .into(),
                                shadcn::DropdownMenuItem::new("رسالة").into(),
                                shadcn::DropdownMenuSub::new(
                                    shadcn::DropdownMenuSubTrigger::new("المزيد").refine(|item| {
                                        item.test_id("ui-gallery-dropdown-menu-rtl-item-more")
                                    }),
                                    shadcn::DropdownMenuSubContent::new([
                                        shadcn::DropdownMenuItem::new("تقويم")
                                            .test_id("ui-gallery-dropdown-menu-rtl-item-calendar")
                                            .into(),
                                        shadcn::DropdownMenuItem::new("دردشة").into(),
                                        shadcn::DropdownMenuSeparator::new().into(),
                                        shadcn::DropdownMenuItem::new("خطاف ويب").into(),
                                    ]),
                                )
                                .into(),
                                shadcn::DropdownMenuSeparator::new().into(),
                                shadcn::DropdownMenuItem::new("متقدم...").into(),
                            ]),
                        )
                        .into(),
                        shadcn::DropdownMenuItem::new("فريق جديد")
                            .shortcut("⌘+T")
                            .into(),
                    ])
                    .into(),
                    shadcn::DropdownMenuSeparator::new().into(),
                    shadcn::DropdownMenuGroup::new([
                        shadcn::DropdownMenuLabel::new("عرض").into(),
                        shadcn::DropdownMenuCheckboxItem::from_checked(
                            menu_state_now.show_status_bar,
                            "شريط الحالة",
                        )
                        .on_checked_change({
                            let menu_state = menu_state.clone();
                            move |host, _action_cx, checked| {
                                let _ = host.models_mut().update(&menu_state, |state| {
                                    state.show_status_bar = checked;
                                });
                            }
                        })
                        .test_id("ui-gallery-dropdown-menu-rtl-item-status-bar")
                        .into(),
                        shadcn::DropdownMenuCheckboxItem::from_checked(
                            menu_state_now.show_activity_bar,
                            "شريط النشاط",
                        )
                        .on_checked_change({
                            let menu_state = menu_state.clone();
                            move |host, _action_cx, checked| {
                                let _ = host.models_mut().update(&menu_state, |state| {
                                    state.show_activity_bar = checked;
                                });
                            }
                        })
                        .test_id("ui-gallery-dropdown-menu-rtl-item-activity-bar")
                        .into(),
                        shadcn::DropdownMenuCheckboxItem::from_checked(
                            menu_state_now.show_panel,
                            "اللوحة",
                        )
                        .on_checked_change({
                            let menu_state = menu_state.clone();
                            move |host, _action_cx, checked| {
                                let _ = host.models_mut().update(&menu_state, |state| {
                                    state.show_panel = checked;
                                });
                            }
                        })
                        .test_id("ui-gallery-dropdown-menu-rtl-item-panel")
                        .into(),
                    ])
                    .into(),
                    shadcn::DropdownMenuSeparator::new().into(),
                    shadcn::DropdownMenuGroup::new([
                        shadcn::DropdownMenuLabel::new("الموضع").into(),
                        shadcn::DropdownMenuRadioGroup::from_value(menu_state_now.position.clone())
                            .on_value_change({
                                let menu_state = menu_state.clone();
                                move |host, _action_cx, value| {
                                    let _ = host
                                        .models_mut()
                                        .update(&menu_state, |state| state.position = Some(value));
                                }
                            })
                            .item(
                                shadcn::DropdownMenuRadioItemSpec::new("top", "أعلى")
                                    .test_id("ui-gallery-dropdown-menu-rtl-position-top"),
                            )
                            .item(
                                shadcn::DropdownMenuRadioItemSpec::new("bottom", "أسفل")
                                    .test_id("ui-gallery-dropdown-menu-rtl-position-bottom"),
                            )
                            .item(
                                shadcn::DropdownMenuRadioItemSpec::new("right", "يمين")
                                    .test_id("ui-gallery-dropdown-menu-rtl-position-right"),
                            )
                            .item(
                                shadcn::DropdownMenuRadioItemSpec::new("left", "يسار")
                                    .test_id("ui-gallery-dropdown-menu-rtl-position-left"),
                            )
                            .into(),
                    ])
                    .into(),
                    shadcn::DropdownMenuSeparator::new().into(),
                    shadcn::DropdownMenuGroup::new([shadcn::DropdownMenuItem::new("تسجيل الخروج")
                        .variant(shadcn::raw::dropdown_menu::DropdownMenuItemVariant::Destructive)
                        .test_id("ui-gallery-dropdown-menu-rtl-item-logout")
                        .into()])
                    .into(),
                ])
        })
    })
    .into_element(cx)
}
// endregion: example
