pub const SOURCE: &str = include_str!("usage.rs");

// region: example
use fret_ui_shadcn::{self as shadcn, prelude::*};

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
                shadcn::DropdownMenuEntry::Item(shadcn::DropdownMenuItem::new("Profile")),
                shadcn::DropdownMenuEntry::Item(shadcn::DropdownMenuItem::new("Billing")),
                shadcn::DropdownMenuEntry::Item(shadcn::DropdownMenuItem::new("Team")),
                shadcn::DropdownMenuEntry::Item(shadcn::DropdownMenuItem::new("Subscription")),
            ]
        },
    )
}
// endregion: example
