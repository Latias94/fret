use std::sync::Arc;

use fret_authoring::UiWriter as _;
use fret_runtime::Model;
use fret_ui::element::AnyElement;
use fret_ui::{ElementContext, UiHost};

pub(in crate::imui) fn floating_window_title_bar_row<H: UiHost>(
    cx: &mut ElementContext<'_, H>,
    area: super::super::FloatingAreaContext,
    title: Arc<str>,
    open_model: Option<Model<bool>>,
    title_bar_test_id: Arc<str>,
    close_button_test_id: Arc<str>,
    resizable_layout: bool,
    options: super::super::FloatingWindowOptions,
) -> AnyElement {
    let row = super::super::floating_window_title_bar_props::title_bar_row_props(resizable_layout);

    let title = title.clone();
    let title_bar_test_id = title_bar_test_id.clone();
    let open_for_key = open_model.clone();
    let can_interact = options.inputs_enabled;
    let can_close = can_interact && options.closable && open_for_key.is_some();
    let can_collapse = can_interact && options.collapsible;
    let can_move = can_interact && options.movable;
    let on_left_double_click = super::behavior::title_bar_double_click_toggle_handler(can_collapse);

    let drag_surface = super::super::floating_area_drag_surface_element(
        cx,
        area,
        super::super::floating_window_title_bar_props::title_bar_drag_surface_props(
            resizable_layout,
            can_interact,
        ),
        on_left_double_click,
        can_move,
        options.activate_on_click,
        move |cx, region_id| {
            super::behavior::install_title_bar_key_behavior(cx, region_id, can_close, open_for_key);
        },
        move |ui| {
            let element = ui.with_cx_mut(|cx| {
                let title = if resizable_layout {
                    crate::declarative::text::text_chrome_title(cx, title.clone())
                } else {
                    crate::declarative::text::text_section_chrome_label(cx, title.clone())
                };
                title.attach_semantics(
                    fret_ui::element::SemanticsDecoration::default()
                        .test_id(title_bar_test_id.clone()),
                )
            });
            ui.add(element);
        },
    );

    let close = (options.inputs_enabled && options.closable)
        .then(|| open_model.clone())
        .flatten()
        .map(|open| {
            let props = super::super::floating_window_title_bar_props::title_bar_close_button_props(
                close_button_test_id.clone(),
            );
            super::behavior::title_bar_close_button(cx, props, open)
        });

    cx.row(row, move |_cx| {
        let mut out = vec![drag_surface];
        if let Some(close) = close {
            out.push(close);
        }
        out
    })
}
