// region: example
use fret_ui_shadcn::{self as shadcn, prelude::*};

fn trigger_surface<H: UiHost>(cx: &mut ElementContext<'_, H>, label: &'static str) -> AnyElement {
    shadcn::Button::new(label)
        .variant(shadcn::ButtonVariant::Outline)
        .size(shadcn::ButtonSize::Sm)
        .into_element(cx)
}

pub fn render<H: UiHost>(cx: &mut ElementContext<'_, H>) -> AnyElement {
    shadcn::ContextMenu::new_controllable(cx, None, false)
        .content_test_id("ui-gallery-context-menu-groups-content")
        .into_element(
            cx,
            |cx| {
                trigger_surface(cx, "Right click for groups")
                    .test_id("ui-gallery-context-menu-groups-trigger")
            },
            |_cx| {
                vec![
                    shadcn::ContextMenuEntry::Group(shadcn::ContextMenuGroup::new(vec![
                        shadcn::ContextMenuEntry::Label(shadcn::ContextMenuLabel::new(
                            "My Account",
                        )),
                        shadcn::ContextMenuEntry::Item(shadcn::ContextMenuItem::new("Profile")),
                        shadcn::ContextMenuEntry::Item(shadcn::ContextMenuItem::new("Billing")),
                        shadcn::ContextMenuEntry::Separator,
                    ])),
                    shadcn::ContextMenuEntry::Group(shadcn::ContextMenuGroup::new(vec![
                        shadcn::ContextMenuEntry::Item(shadcn::ContextMenuItem::new("Team")),
                        shadcn::ContextMenuEntry::Item(shadcn::ContextMenuItem::new(
                            "Subscription",
                        )),
                    ])),
                ]
            },
        )
        .test_id("ui-gallery-context-menu-groups")
}
// endregion: example
