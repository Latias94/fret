use fret_app::{App, Model};
use fret_ui::ElementContext;
use fret_ui::element::AnyElement;
use fret_ui::element::{Length, TextProps};
use fret_ui_kit::ui;
use fret_ui_shadcn::{facade as shadcn, prelude::*};
use std::sync::Arc;

use crate::spec::*;

fn flex_row_wrap_label(cx: &mut ElementContext<'_, App>, text: &'static str) -> AnyElement {
    let mut props = TextProps::new(text);
    props.layout.flex.grow = 1.0;
    props.layout.flex.shrink = 1.0;
    props.layout.flex.basis = Length::Px(Px(0.0));
    props.layout.size.min_width = Some(Length::Px(Px(0.0)));
    cx.text_props(props)
}

fn switch_row(
    cx: &mut ElementContext<'_, App>,
    control: AnyElement,
    label: &'static str,
) -> AnyElement {
    ui::h_flex(|cx| vec![control, flex_row_wrap_label(cx, label)])
        .gap(Space::N2)
        .items_start()
        .into_element(cx)
}

pub(super) fn push_settings_sheet(
    cx: &mut ElementContext<'_, App>,
    settings_open: Model<bool>,
    settings_menu_bar_os: Model<Option<Arc<str>>>,
    settings_menu_bar_os_open: Model<bool>,
    settings_menu_bar_in_window: Model<Option<Arc<str>>>,
    settings_menu_bar_in_window_open: Model<bool>,
    settings_edit_can_undo: Model<bool>,
    settings_edit_can_redo: Model<bool>,
    chrome_show_workspace_tab_strip: Model<bool>,
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
                    .value(shadcn::SelectValue::new().placeholder("OS menubar"))
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
                    .value(shadcn::SelectValue::new().placeholder("In-window menubar"))
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

                    let chrome_show_workspace_tab_strip_switch = shadcn::Switch::new(
                        chrome_show_workspace_tab_strip.clone(),
                    )
                    .a11y_label("Show workspace tab strip")
                    .test_id("ui-gallery-settings-workspace-tab-strip")
                    .into_element(cx);
                    let edit_can_undo_switch = shadcn::Switch::new(settings_edit_can_undo.clone())
                        .a11y_label("Can Undo")
                        .disabled(true)
                        .into_element(cx);
                    let edit_can_redo_switch = shadcn::Switch::new(settings_edit_can_redo.clone())
                        .a11y_label("Can Redo")
                        .disabled(true)
                        .into_element(cx);

                    let body = ui::v_stack(|cx| {
                        vec![
                            ui::v_stack(|cx| {
                                vec![
                                    shadcn::SheetHeader::new(vec![
                                        shadcn::SheetTitle::new("Settings").into_element(cx),
                                        shadcn::SheetDescription::new(
                                            "Menu bar presentation (OS vs in-window) + chrome/debug state.",
                                        )
                                        .into_element(cx),
                                    ])
                                    .into_element(cx),
                                    shadcn::Separator::new().into_element(cx),
                                    cx.text("Menu bar surfaces"),
                                    os_select,
                                    in_window_select,
                                    cx.text("Chrome"),
                                    switch_row(
                                        cx,
                                        chrome_show_workspace_tab_strip_switch,
                                        "Workspace tabs in the top bar",
                                    ),
                                    cx.text("Command availability (debug)"),
                                    switch_row(
                                        cx,
                                        edit_can_undo_switch,
                                        "edit.can_undo (enables OS/in-window Undo)",
                                    ),
                                    switch_row(
                                        cx,
                                        edit_can_redo_switch,
                                        "edit.can_redo (enables OS/in-window Redo)",
                                    ),
                                ]
                            })
                            .layout(LayoutRefinement::default().w_full())
                            .gap(Space::N2)
                            .into_element(cx),
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
                    })
                    .layout(LayoutRefinement::default().w_full())
                    .gap(Space::N4)
                    .into_element(cx);

                    shadcn::SheetContent::new(vec![body]).into_element(cx)
                },
            )
    }));
}
