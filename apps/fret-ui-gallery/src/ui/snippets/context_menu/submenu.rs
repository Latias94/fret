pub const SOURCE: &str = include_str!("submenu.rs");

// region: example
use fret_runtime::CommandId;
use fret_ui_shadcn::{self as shadcn, prelude::*};

fn trigger_surface<H: UiHost>(cx: &mut ElementContext<'_, H>, label: &'static str) -> AnyElement {
    shadcn::Button::new(label)
        .variant(shadcn::ButtonVariant::Outline)
        .size(shadcn::ButtonSize::Sm)
        .into_element(cx)
}

pub fn render<H: UiHost>(cx: &mut ElementContext<'_, H>) -> AnyElement {
    shadcn::ContextMenu::new_controllable(cx, None, false)
        .content_test_id("ui-gallery-context-menu-submenu-content")
        .into_element(
            cx,
            |cx| {
                trigger_surface(cx, "Right click for submenu")
                    .test_id("ui-gallery-context-menu-submenu-trigger")
            },
            |_cx| {
                vec![
                    shadcn::ContextMenuEntry::Item(
                        shadcn::ContextMenuItem::new("Open")
                            .action(CommandId::new("ui_gallery.context_menu.submenu.open"))
                            .test_id("ui-gallery-context-menu-submenu-open"),
                    ),
                    shadcn::ContextMenuSub::new(
                        shadcn::ContextMenuSubTrigger::new("More tools").refine(|item| {
                            item.test_id("ui-gallery-context-menu-submenu-more-tools")
                        }),
                        shadcn::ContextMenuSubContent::new(vec![
                            shadcn::ContextMenuItem::new("Rename")
                                .action(CommandId::new("ui_gallery.context_menu.submenu.rename"))
                                .test_id("ui-gallery-context-menu-submenu-rename")
                                .into(),
                            shadcn::ContextMenuItem::new("Duplicate")
                                .action(CommandId::new(
                                    "ui_gallery.context_menu.submenu.duplicate",
                                ))
                                .test_id("ui-gallery-context-menu-submenu-duplicate")
                                .into(),
                        ]),
                    )
                    .into_entry(),
                ]
            },
        )
        .test_id("ui-gallery-context-menu-submenu")
}
// endregion: example
