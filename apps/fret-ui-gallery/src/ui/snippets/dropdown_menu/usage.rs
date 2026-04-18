pub const SOURCE: &str = include_str!("usage.rs");

// region: example
use fret::{AppComponentCx, UiChild};
use fret_core::Px;
use fret_ui_kit::IntoUiElement;
use fret_ui_shadcn::facade as shadcn;

pub fn render(cx: &mut AppComponentCx<'_>) -> impl UiChild + use<> {
    super::preview_frame_with(cx, |cx| {
        shadcn::DropdownMenu::uncontrolled(cx)
            .compose()
            .trigger(shadcn::Button::new("Open").variant(shadcn::ButtonVariant::Outline))
            .content(
                shadcn::DropdownMenuContent::new()
                    .align(shadcn::DropdownMenuAlign::Start)
                    .side_offset(Px(4.0)),
            )
            .entries([
                shadcn::DropdownMenuGroup::new([
                    shadcn::DropdownMenuLabel::new("My Account").into(),
                    shadcn::DropdownMenuItem::new("Profile").into(),
                    shadcn::DropdownMenuItem::new("Billing").into(),
                ])
                .into(),
                shadcn::DropdownMenuSeparator::new().into(),
                shadcn::DropdownMenuGroup::new([
                    shadcn::DropdownMenuItem::new("Team").into(),
                    shadcn::DropdownMenuItem::new("Subscription").into(),
                ])
                .into(),
            ])
    })
    .into_element(cx)
}
// endregion: example
