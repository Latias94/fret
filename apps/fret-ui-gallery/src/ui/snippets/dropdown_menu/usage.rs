pub const SOURCE: &str = include_str!("usage.rs");

// region: example
use fret_core::Px;
use fret_ui_kit::IntoUiElement;
use fret_ui_shadcn::{facade as shadcn, prelude::*};

pub fn render<H: UiHost>(cx: &mut ElementContext<'_, H>) -> AnyElement {
    super::preview_frame_with(cx, |cx| {
        shadcn::DropdownMenu::uncontrolled(cx).build_parts(
            cx,
            shadcn::DropdownMenuTrigger::build(
                shadcn::Button::new("Open").variant(shadcn::ButtonVariant::Outline),
            ),
            shadcn::DropdownMenuContent::new()
                .align(shadcn::DropdownMenuAlign::Start)
                .side_offset(Px(4.0)),
            |_cx| {
                [
                    shadcn::DropdownMenuGroup::new([
                        shadcn::DropdownMenuLabel::new("My Account").into(),
                        shadcn::DropdownMenuItem::new("Profile").into(),
                        shadcn::DropdownMenuItem::new("Billing").into(),
                        shadcn::DropdownMenuSeparator::new().into(),
                    ])
                    .into(),
                    shadcn::DropdownMenuGroup::new([
                        shadcn::DropdownMenuItem::new("Team").into(),
                        shadcn::DropdownMenuItem::new("Subscription").into(),
                    ])
                    .into(),
                ]
            },
        )
    })
    .into_element(cx)
}
// endregion: example
