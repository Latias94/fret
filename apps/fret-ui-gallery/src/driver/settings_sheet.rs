use fret_app::{App, Model};
use fret_ui::ElementContext;
use fret_ui::element::AnyElement;
use fret_ui_shadcn::{self as shadcn, prelude::*};
use std::sync::Arc;

use crate::spec::*;

pub(super) fn push_settings_sheet(
    cx: &mut ElementContext<'_, App>,
    settings_open: Model<bool>,
    settings_menu_bar_os: Model<Option<Arc<str>>>,
    settings_menu_bar_os_open: Model<bool>,
    settings_menu_bar_in_window: Model<Option<Arc<str>>>,
    settings_menu_bar_in_window_open: Model<bool>,
    settings_edit_can_undo: Model<bool>,
    settings_edit_can_redo: Model<bool>,
    content: &mut Vec<AnyElement>,
) {
    content.push(cx.keyed("ui_gallery.settings_sheet", move |cx| {
        shadcn::Sheet::new(settings_open.clone())
            .side(shadcn::SheetSide::Right)
            .size(Px(420.0))
            .into_element(
                cx,
                |cx| {
                    let mut layout = fret_ui::element::LayoutStyle::default();
                    layout.size.width = fret_ui::element::Length::Px(Px(0.0));
                    layout.size.height = fret_ui::element::Length::Px(Px(0.0));
                    cx.container(
                        fret_ui::element::ContainerProps {
                            layout,
                            ..Default::default()
                        },
                        |_cx| Vec::new(),
                    )
                },
                |cx| {
                    let os_select = shadcn::Select::new(
                        settings_menu_bar_os.clone(),
                        settings_menu_bar_os_open.clone(),
                    )
                    .placeholder("OS menubar")
                    .trigger_test_id("ui-gallery-settings-os-menubar")
                    .items([
                        shadcn::SelectItem::new("auto", "Auto (Windows/macOS on; Linux/Web off)")
                            .test_id("ui-gallery-settings-os-menubar-auto"),
                        shadcn::SelectItem::new("on", "On")
                            .test_id("ui-gallery-settings-os-menubar-on"),
                        shadcn::SelectItem::new("off", "Off")
                            .test_id("ui-gallery-settings-os-menubar-off"),
                    ])
                    .refine_layout(LayoutRefinement::default().w_full())
                    .into_element(cx);

                    let in_window_select = shadcn::Select::new(
                        settings_menu_bar_in_window.clone(),
                        settings_menu_bar_in_window_open.clone(),
                    )
                    .placeholder("In-window menubar")
                    .trigger_test_id("ui-gallery-settings-in-window-menubar")
                    .items([
                        shadcn::SelectItem::new("auto", "Auto (Linux/Web on; Windows/macOS off)")
                            .test_id("ui-gallery-settings-in-window-menubar-auto"),
                        shadcn::SelectItem::new("on", "On")
                            .test_id("ui-gallery-settings-in-window-menubar-on"),
                        shadcn::SelectItem::new("off", "Off")
                            .test_id("ui-gallery-settings-in-window-menubar-off"),
                    ])
                    .refine_layout(LayoutRefinement::default().w_full())
                    .into_element(cx);

                    let body = stack::vstack(
                        cx,
                        stack::VStackProps::default()
                            .layout(LayoutRefinement::default().w_full())
                            .gap(Space::N4),
                        |cx| {
                            vec![
                                stack::vstack(
                                    cx,
                                    stack::VStackProps::default()
                                        .layout(LayoutRefinement::default().w_full())
                                        .gap(Space::N2),
                                    |cx| {
                                        vec![
                                            shadcn::SheetHeader::new(vec![
                                                shadcn::SheetTitle::new("Settings")
                                                    .into_element(cx),
                                                shadcn::SheetDescription::new(
                                                    "Menu bar presentation (OS vs in-window).",
                                                )
                                                .into_element(cx),
                                            ])
                                            .into_element(cx),
                                            shadcn::Separator::new().into_element(cx),
                                            cx.text("Menu bar surfaces"),
                                            os_select,
                                            in_window_select,
                                            cx.text("Command availability (debug)"),
                                            stack::hstack(
                                                cx,
                                                stack::HStackProps::default()
                                                    .gap(Space::N2)
                                                    .items_center(),
                                                |cx| {
                                                    vec![
                                                        shadcn::Switch::new(
                                                            settings_edit_can_undo.clone(),
                                                        )
                                                        .a11y_label("Can Undo")
                                                        .disabled(true)
                                                        .into_element(cx),
                                                        cx.text(
                                                            "edit.can_undo (enables OS/in-window Undo)",
                                                        ),
                                                    ]
                                                },
                                            ),
                                            stack::hstack(
                                                cx,
                                                stack::HStackProps::default()
                                                    .gap(Space::N2)
                                                    .items_center(),
                                                |cx| {
                                                    vec![
                                                        shadcn::Switch::new(
                                                            settings_edit_can_redo.clone(),
                                                        )
                                                        .a11y_label("Can Redo")
                                                        .disabled(true)
                                                        .into_element(cx),
                                                        cx.text(
                                                            "edit.can_redo (enables OS/in-window Redo)",
                                                        ),
                                                    ]
                                                },
                                            ),
                                        ]
                                    },
                                ),
                                shadcn::Separator::new().into_element(cx),
                                shadcn::SheetFooter::new(vec![
                                    shadcn::Button::new("Apply (in memory)")
                                        .variant(shadcn::ButtonVariant::Secondary)
                                        .test_id("ui-gallery-settings-apply")
                                        .on_click(CMD_APP_SETTINGS_APPLY)
                                        .into_element(cx),
                                    shadcn::Button::new("Write project .fret/settings.json")
                                        .variant(shadcn::ButtonVariant::Outline)
                                        .on_click(CMD_APP_SETTINGS_WRITE_PROJECT)
                                        .into_element(cx),
                                    shadcn::Button::new("Close")
                                        .variant(shadcn::ButtonVariant::Ghost)
                                        .toggle_model(settings_open.clone())
                                        .into_element(cx),
                                ])
                                .into_element(cx),
                            ]
                        },
                    );

                    shadcn::SheetContent::new(vec![body]).into_element(cx)
                },
            )
    }));
}
