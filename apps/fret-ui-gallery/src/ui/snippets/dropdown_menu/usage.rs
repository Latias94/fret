pub const SOURCE: &str = include_str!("usage.rs");

// region: example
use fret_ui_shadcn::{facade as shadcn, prelude::*};

pub fn render<H: UiHost>(cx: &mut ElementContext<'_, H>) -> AnyElement {
    shadcn::DropdownMenu::new_controllable(cx, None, false).into_element_parts(
        cx,
        |cx| {
            shadcn::DropdownMenuTrigger::new(
                shadcn::Button::new("Open")
                    .variant(shadcn::ButtonVariant::Outline)
                    .into_element(cx),
            )
        },
        shadcn::DropdownMenuContent::new(),
        |_cx| {
            vec![
                shadcn::DropdownMenuEntry::Group(shadcn::DropdownMenuGroup::new([
                    shadcn::DropdownMenuLabel::new("My Account").into(),
                    shadcn::DropdownMenuItem::new("Profile").into(),
                    shadcn::DropdownMenuItem::new("Billing").into(),
                    shadcn::DropdownMenuSeparator::new().into(),
                ])),
                shadcn::DropdownMenuEntry::Group(shadcn::DropdownMenuGroup::new([
                    shadcn::DropdownMenuItem::new("Team").into(),
                    shadcn::DropdownMenuItem::new("Subscription").into(),
                ])),
            ]
        },
    )
}
// endregion: example
