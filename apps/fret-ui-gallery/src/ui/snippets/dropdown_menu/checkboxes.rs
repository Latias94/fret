pub const SOURCE: &str = include_str!("checkboxes.rs");

// region: example
use fret::{UiChild, UiCx};
use fret_core::Px;
use fret_ui_kit::IntoUiElement;
use fret_ui_kit::declarative::ModelWatchExt as _;
use fret_ui_shadcn::facade as shadcn;

#[derive(Default, Clone)]
struct AppearanceState {
    show_status_bar: bool,
    show_activity_bar: bool,
    show_panel: bool,
}

pub fn render(cx: &mut UiCx<'_>) -> impl UiChild + use<> {
    let appearance = cx.local_model(|| AppearanceState {
        show_status_bar: true,
        show_activity_bar: false,
        show_panel: false,
    });
    let appearance_now = cx
        .watch_model(&appearance)
        .layout()
        .cloned()
        .unwrap_or_default();

    super::preview_frame_with(cx, |cx| {
        shadcn::DropdownMenu::uncontrolled(cx)
            .compose()
            .trigger(
                shadcn::Button::new("Open")
                    .variant(shadcn::ButtonVariant::Outline)
                    .test_id("ui-gallery-dropdown-menu-checkboxes-trigger"),
            )
            .content(
                shadcn::DropdownMenuContent::new()
                    .align(shadcn::DropdownMenuAlign::Start)
                    .side_offset(Px(4.0))
                    // base-nova dropdown-menu-checkboxes: `DropdownMenuContent className="w-40"`.
                    .min_width(Px(160.0)),
            )
            .entries([shadcn::DropdownMenuGroup::new([
                shadcn::DropdownMenuLabel::new("Appearance").into(),
                shadcn::DropdownMenuCheckboxItem::from_checked(
                    appearance_now.show_status_bar,
                    "Status Bar",
                )
                .on_checked_change({
                    let appearance = appearance.clone();
                    move |host, _action_cx, checked| {
                        let _ = host
                            .models_mut()
                            .update(&appearance, |state| state.show_status_bar = checked);
                    }
                })
                .test_id("ui-gallery-dropdown-menu-checkboxes-status-bar")
                .into(),
                shadcn::DropdownMenuCheckboxItem::from_checked(
                    appearance_now.show_activity_bar,
                    "Activity Bar",
                )
                .on_checked_change({
                    let appearance = appearance.clone();
                    move |host, _action_cx, checked| {
                        let _ = host
                            .models_mut()
                            .update(&appearance, |state| state.show_activity_bar = checked);
                    }
                })
                .disabled(true)
                .test_id("ui-gallery-dropdown-menu-checkboxes-activity-bar")
                .into(),
                shadcn::DropdownMenuCheckboxItem::from_checked(appearance_now.show_panel, "Panel")
                    .on_checked_change({
                        let appearance = appearance.clone();
                        move |host, _action_cx, checked| {
                            let _ = host
                                .models_mut()
                                .update(&appearance, |state| state.show_panel = checked);
                        }
                    })
                    .into(),
            ])
            .into()])
    })
    .into_element(cx)
}
// endregion: example
