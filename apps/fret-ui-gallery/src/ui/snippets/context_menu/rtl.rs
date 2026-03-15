pub const SOURCE: &str = include_str!("rtl.rs");

// region: example
use fret::{UiChild, UiCx};
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

pub fn render(cx: &mut UiCx<'_>) -> impl UiChild + use<> {
    with_direction_provider(cx, LayoutDirection::Rtl, |cx| {
        shadcn::ContextMenu::uncontrolled(cx)
            .content_test_id("ui-gallery-context-menu-rtl-content")
            .build(
                cx,
                trigger_surface("Right click in RTL", "ui-gallery-context-menu-rtl-trigger"),
                |cx| {
                    vec![
                        shadcn::ContextMenuEntry::Item(
                            shadcn::ContextMenuItem::new("Open")
                                .action(CommandId::new("ui_gallery.context_menu.rtl.open"))
                                .test_id("ui-gallery-context-menu-rtl-item-open")
                                .trailing(shadcn::ContextMenuShortcut::new("⌘O").into_element(cx)),
                        ),
                        shadcn::ContextMenuEntry::Item(
                            shadcn::ContextMenuItem::new("Settings")
                                .action(CommandId::new("ui_gallery.context_menu.rtl.settings"))
                                .test_id("ui-gallery-context-menu-rtl-item-settings")
                                .trailing(shadcn::ContextMenuShortcut::new("⌘,").into_element(cx)),
                        ),
                        shadcn::ContextMenuEntry::Separator,
                        shadcn::ContextMenuEntry::Item(
                            shadcn::ContextMenuItem::new("More")
                                .test_id("ui-gallery-context-menu-rtl-item-more")
                                .submenu([
                                    shadcn::ContextMenuEntry::Item(
                                        shadcn::ContextMenuItem::new("Sub Alpha")
                                            .action(CommandId::new(
                                                "ui_gallery.context_menu.rtl.sub_alpha",
                                            ))
                                            .test_id("ui-gallery-context-menu-rtl-item-sub-alpha"),
                                    ),
                                    shadcn::ContextMenuEntry::Item(
                                        shadcn::ContextMenuItem::new("Sub Beta")
                                            .action(CommandId::new(
                                                "ui_gallery.context_menu.rtl.sub_beta",
                                            ))
                                            .test_id("ui-gallery-context-menu-rtl-item-sub-beta"),
                                    ),
                                ]),
                        ),
                        shadcn::ContextMenuEntry::Separator,
                        shadcn::ContextMenuEntry::Item(
                            shadcn::ContextMenuItem::new("Delete")
                                .action(CommandId::new("ui_gallery.context_menu.rtl.delete"))
                                .test_id("ui-gallery-context-menu-rtl-item-delete")
                                .variant(fret_ui_shadcn::context_menu::ContextMenuItemVariant::Destructive),
                        ),
                    ]
                },
            )
    })
    .test_id("ui-gallery-context-menu-rtl")
}
// endregion: example
