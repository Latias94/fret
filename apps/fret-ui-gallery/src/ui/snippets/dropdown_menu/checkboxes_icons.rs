pub const SOURCE: &str = include_str!("checkboxes_icons.rs");

// region: example
use fret::{UiChild, UiCx};
use fret_core::Px;
use fret_ui_kit::IntoUiElement;
use fret_ui_kit::declarative::ModelWatchExt as _;
use fret_ui_shadcn::{facade as shadcn, prelude::*};

#[derive(Default, Clone)]
struct NotificationState {
    email: bool,
    sms: bool,
    push: bool,
}

pub fn render(cx: &mut UiCx<'_>) -> impl UiChild + use<> {
    let notifications = cx.local_model(|| NotificationState {
        email: true,
        sms: false,
        push: true,
    });
    let notifications_now = cx
        .watch_model(&notifications)
        .layout()
        .cloned()
        .unwrap_or_default();

    super::preview_frame_with(cx, |cx| {
        shadcn::DropdownMenu::uncontrolled(cx)
            .compose()
            .trigger(
                shadcn::Button::new("Notifications")
                    .variant(shadcn::ButtonVariant::Outline)
                    .test_id("ui-gallery-dropdown-menu-checkboxes-icons-trigger"),
            )
            .content(
                shadcn::DropdownMenuContent::new()
                    .align(shadcn::DropdownMenuAlign::Start)
                    .side_offset(Px(4.0))
                    // base-nova dropdown-menu-checkboxes-icons: `DropdownMenuContent className="w-48"`.
                    .min_width(Px(192.0)),
            )
            .entries([shadcn::DropdownMenuGroup::new([
                shadcn::DropdownMenuLabel::new("Notification Preferences").into(),
                shadcn::DropdownMenuCheckboxItem::from_checked(
                    notifications_now.email,
                    "Email notifications",
                )
                .on_checked_change({
                    let notifications = notifications.clone();
                    move |host, _action_cx, checked| {
                        let _ = host
                            .models_mut()
                            .update(&notifications, |state| state.email = checked);
                    }
                })
                .leading_icon(IconId::new_static("lucide.mail"))
                .test_id("ui-gallery-dropdown-menu-checkboxes-icons-email")
                .into(),
                shadcn::DropdownMenuCheckboxItem::from_checked(
                    notifications_now.sms,
                    "SMS notifications",
                )
                .on_checked_change({
                    let notifications = notifications.clone();
                    move |host, _action_cx, checked| {
                        let _ = host
                            .models_mut()
                            .update(&notifications, |state| state.sms = checked);
                    }
                })
                .leading_icon(IconId::new_static("lucide.message-square"))
                .test_id("ui-gallery-dropdown-menu-checkboxes-icons-sms")
                .into(),
                shadcn::DropdownMenuCheckboxItem::from_checked(
                    notifications_now.push,
                    "Push notifications",
                )
                .on_checked_change({
                    let notifications = notifications.clone();
                    move |host, _action_cx, checked| {
                        let _ = host
                            .models_mut()
                            .update(&notifications, |state| state.push = checked);
                    }
                })
                .leading_icon(IconId::new_static("lucide.bell"))
                .test_id("ui-gallery-dropdown-menu-checkboxes-icons-push")
                .into(),
            ])
            .into()])
    })
    .into_element(cx)
}
// endregion: example
