pub const SOURCE: &str = include_str!("submenu.rs");

// region: example
use fret_runtime::CommandId;
use fret_ui_kit::IntoUiElement;
use fret_ui_shadcn::{facade as shadcn, prelude::*};

fn trigger_surface<H: UiHost>(
    label: &'static str,
    test_id: &'static str,
) -> impl IntoUiElement<H> + use<H> {
    shadcn::Button::new(label)
        .variant(shadcn::ButtonVariant::Outline)
        .size(shadcn::ButtonSize::Sm)
        .test_id(test_id)
}

pub fn render<H: UiHost>(cx: &mut ElementContext<'_, H>) -> AnyElement {
    shadcn::ContextMenu::uncontrolled(cx)
        .content_test_id("ui-gallery-context-menu-submenu-content")
        .build(
            cx,
            trigger_surface(
                "Right click for submenu",
                "ui-gallery-context-menu-submenu-trigger",
            ),
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
                                .action(CommandId::new("ui_gallery.context_menu.submenu.duplicate"))
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
