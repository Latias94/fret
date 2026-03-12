pub const SOURCE: &str = include_str!("groups.rs");

// region: example
use fret_runtime::CommandId;
use fret_ui_kit::IntoUiElement;
use fret_ui_shadcn::{facade as shadcn, prelude::*};

fn trigger_surface<H: UiHost>(label: &'static str) -> impl IntoUiElement<H> + use<H> {
    shadcn::Button::new(label)
        .variant(shadcn::ButtonVariant::Outline)
        .size(shadcn::ButtonSize::Sm)
}

pub fn render<H: UiHost>(cx: &mut ElementContext<'_, H>) -> AnyElement {
    shadcn::ContextMenu::new_controllable(cx, None, false)
        .content_test_id("ui-gallery-context-menu-groups-content")
        .into_element(
            cx,
            |cx| {
                trigger_surface("Right click for groups")
                    .into_element(cx)
                    .test_id("ui-gallery-context-menu-groups-trigger")
            },
            |_cx| {
                vec![
                    shadcn::ContextMenuEntry::Group(shadcn::ContextMenuGroup::new(vec![
                        shadcn::ContextMenuEntry::Label(shadcn::ContextMenuLabel::new(
                            "My Account",
                        )),
                        shadcn::ContextMenuEntry::Item(
                            shadcn::ContextMenuItem::new("Profile")
                                .action(CommandId::new("ui_gallery.context_menu.groups.profile")),
                        ),
                        shadcn::ContextMenuEntry::Item(
                            shadcn::ContextMenuItem::new("Billing")
                                .action(CommandId::new("ui_gallery.context_menu.groups.billing")),
                        ),
                        shadcn::ContextMenuEntry::Separator,
                    ])),
                    shadcn::ContextMenuEntry::Group(shadcn::ContextMenuGroup::new(vec![
                        shadcn::ContextMenuEntry::Item(
                            shadcn::ContextMenuItem::new("Team")
                                .action(CommandId::new("ui_gallery.context_menu.groups.team")),
                        ),
                        shadcn::ContextMenuEntry::Item(
                            shadcn::ContextMenuItem::new("Subscription").action(CommandId::new(
                                "ui_gallery.context_menu.groups.subscription",
                            )),
                        ),
                    ])),
                ]
            },
        )
        .test_id("ui-gallery-context-menu-groups")
}
// endregion: example
