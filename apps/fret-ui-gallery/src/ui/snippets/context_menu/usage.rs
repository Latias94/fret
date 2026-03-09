pub const SOURCE: &str = include_str!("usage.rs");

// region: example
use fret_ui_shadcn::{self as shadcn, prelude::*};

pub fn render<H: UiHost>(cx: &mut ElementContext<'_, H>) -> AnyElement {
    shadcn::ContextMenu::new_controllable(cx, None, false).into_element_parts(
        cx,
        |cx| {
            shadcn::ContextMenuTrigger::new(
                shadcn::Button::new("Right click here")
                    .variant(shadcn::ButtonVariant::Outline)
                    .into_element(cx),
            )
        },
        shadcn::ContextMenuContent::new(),
        |_cx| {
            vec![
                shadcn::ContextMenuEntry::Item(shadcn::ContextMenuItem::new("Profile")),
                shadcn::ContextMenuEntry::Item(shadcn::ContextMenuItem::new("Billing")),
                shadcn::ContextMenuEntry::Item(shadcn::ContextMenuItem::new("Team")),
                shadcn::ContextMenuEntry::Item(shadcn::ContextMenuItem::new("Subscription")),
            ]
        },
    )
}
// endregion: example
