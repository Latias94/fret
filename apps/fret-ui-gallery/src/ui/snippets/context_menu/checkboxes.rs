pub const SOURCE: &str = include_str!("checkboxes.rs");

// region: example
use fret::{UiChild, UiCx};
use fret_runtime::CommandId;
use fret_ui_kit::IntoUiElement;
use fret_ui_kit::declarative::ModelWatchExt as _;
use fret_ui_shadcn::{facade as shadcn, prelude::*};

#[derive(Default, Clone)]
struct AppearanceState {
    show_status_bar: bool,
    show_activity_bar: bool,
    show_line_numbers: bool,
}

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
    let appearance = cx.local_model(|| AppearanceState {
        show_status_bar: true,
        show_activity_bar: true,
        show_line_numbers: false,
    });
    let appearance_now = cx
        .watch_model(&appearance)
        .layout()
        .cloned()
        .unwrap_or_default();

    shadcn::ContextMenu::uncontrolled(cx)
        .content_test_id("ui-gallery-context-menu-checkboxes-content")
        .build(
            cx,
            trigger_surface(
                "Right click for checkboxes",
                "ui-gallery-context-menu-checkboxes-trigger",
            ),
            |_cx| {
                vec![
                    shadcn::ContextMenuEntry::CheckboxItem(
                        shadcn::ContextMenuCheckboxItem::from_checked(
                            appearance_now.show_status_bar,
                            "Status Bar",
                        )
                        .on_checked_change({
                            let appearance = appearance.clone();
                            move |host, _action_cx, checked| {
                                let _ = host.models_mut().update(&appearance, |state| {
                                    state.show_status_bar = checked;
                                });
                            }
                        })
                        .action(CommandId::new(
                            "ui_gallery.context_menu.checkboxes.status_bar",
                        ))
                        .test_id("ui-gallery-context-menu-checkboxes-status-bar"),
                    ),
                    shadcn::ContextMenuEntry::CheckboxItem(
                        shadcn::ContextMenuCheckboxItem::from_checked(
                            appearance_now.show_activity_bar,
                            "Activity Bar",
                        )
                        .on_checked_change({
                            let appearance = appearance.clone();
                            move |host, _action_cx, checked| {
                                let _ = host.models_mut().update(&appearance, |state| {
                                    state.show_activity_bar = checked;
                                });
                            }
                        })
                        .action(CommandId::new(
                            "ui_gallery.context_menu.checkboxes.activity_bar",
                        )),
                    ),
                    shadcn::ContextMenuEntry::CheckboxItem(
                        shadcn::ContextMenuCheckboxItem::from_checked(
                            appearance_now.show_line_numbers,
                            "Show Line Numbers",
                        )
                        .on_checked_change({
                            let appearance = appearance.clone();
                            move |host, _action_cx, checked| {
                                let _ = host.models_mut().update(&appearance, |state| {
                                    state.show_line_numbers = checked;
                                });
                            }
                        })
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
