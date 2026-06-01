use fret_core::Color;
use fret_ui::element::AnyElement;
use fret_ui::{ElementContext, GlobalElementId, UiHost};

use super::props::{clipped_body_props, shell_column_props, title_bar_container_props};

pub(in crate::imui::floating_window_shell) fn floating_window_shell_body_element<H: UiHost>(
    cx: &mut ElementContext<'_, H>,
    window_id: GlobalElementId,
    title_bar_row: AnyElement,
    content: AnyElement,
    resizable_layout: bool,
    collapsed: bool,
    resize_enabled: bool,
    options: super::super::FloatingWindowOptions,
    handle_test_ids: super::super::floating_window_resize::FloatingWindowResizeHandleTestIds,
    muted: Color,
    border: Color,
) -> AnyElement {
    let col = shell_column_props(resizable_layout, collapsed);

    let title_bar = cx.container(
        title_bar_container_props(resizable_layout, muted, border),
        move |_cx| vec![title_bar_row],
    );

    let body = if collapsed {
        title_bar
    } else {
        cx.column(col, move |_cx| vec![title_bar, content])
    };

    let clipped_body = cx.container(
        clipped_body_props(resizable_layout, collapsed),
        move |_cx| vec![body],
    );

    let blocker = super::super::floating_window_blocker::floating_window_blocker_element(
        cx,
        options.inputs_enabled,
    );

    super::super::floating_window_resize::resize_stack_element(
        cx,
        window_id,
        clipped_body,
        blocker,
        resizable_layout,
        collapsed,
        resize_enabled,
        options.activate_on_click,
        handle_test_ids,
    )
}
