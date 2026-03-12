pub const SOURCE: &str = include_str!("checkboxes.rs");

// region: example
use fret_runtime::CommandId;
use fret_ui_shadcn::{facade as shadcn, prelude::*};

fn trigger_surface<H: UiHost>(cx: &mut ElementContext<'_, H>, label: &'static str) -> AnyElement {
    shadcn::Button::new(label)
        .variant(shadcn::ButtonVariant::Outline)
        .size(shadcn::ButtonSize::Sm)
        .into_element(cx)
}

pub fn render<H: UiHost>(cx: &mut ElementContext<'_, H>) -> AnyElement {
    let show_status_bar = cx.local_model_keyed("show_status_bar", || true);
    let show_activity_bar = cx.local_model_keyed("show_activity_bar", || true);
    let show_line_numbers = cx.local_model_keyed("show_line_numbers", || false);

    shadcn::ContextMenu::new_controllable(cx, None, false)
        .content_test_id("ui-gallery-context-menu-checkboxes-content")
        .into_element(
            cx,
            |cx| {
                trigger_surface(cx, "Right click for checkboxes")
                    .test_id("ui-gallery-context-menu-checkboxes-trigger")
            },
            |_cx| {
                vec![
                    shadcn::ContextMenuEntry::CheckboxItem(
                        shadcn::ContextMenuCheckboxItem::new(show_status_bar.clone(), "Status Bar")
                            .action(CommandId::new(
                                "ui_gallery.context_menu.checkboxes.status_bar",
                            ))
                            .test_id("ui-gallery-context-menu-checkboxes-status-bar"),
                    ),
                    shadcn::ContextMenuEntry::CheckboxItem(
                        shadcn::ContextMenuCheckboxItem::new(
                            show_activity_bar.clone(),
                            "Activity Bar",
                        )
                        .action(CommandId::new(
                            "ui_gallery.context_menu.checkboxes.activity_bar",
                        )),
                    ),
                    shadcn::ContextMenuEntry::CheckboxItem(
                        shadcn::ContextMenuCheckboxItem::new(
                            show_line_numbers.clone(),
                            "Show Line Numbers",
                        )
                        .action(CommandId::new(
                            "ui_gallery.context_menu.checkboxes.show_line_numbers",
                        )),
                    ),
                ]
            },
        )
        .test_id("ui-gallery-context-menu-checkboxes")
}
// endregion: example
