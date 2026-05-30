use fret_core::Size;
use fret_ui::element::AnyElement;
use fret_ui::{ElementContext, GlobalElementId, UiHost};

mod props;

use props::{
    clipped_body_props, shell_column_props, title_bar_container_props, window_frame_props,
};

pub(super) fn floating_window_shell_element<H: UiHost>(
    cx: &mut ElementContext<'_, H>,
    window_id: GlobalElementId,
    title_bar_row: AnyElement,
    content: AnyElement,
    window_size: Size,
    resizable_layout: bool,
    collapsed: bool,
    resize_enabled: bool,
    options: super::FloatingWindowOptions,
    handle_test_ids: super::floating_window_resize::FloatingWindowResizeHandleTestIds,
) -> AnyElement {
    let (popover, border, muted) = {
        let theme = fret_ui::Theme::global(&*cx.app);
        (
            theme.color_token("popover"),
            theme.color_token("border"),
            theme.color_token("muted"),
        )
    };

    let window_props =
        window_frame_props(window_size, resizable_layout, collapsed, popover, border);

    cx.container(window_props, move |cx| {
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

        let blocker = super::floating_window_blocker::floating_window_blocker_element(
            cx,
            options.inputs_enabled,
        );

        let stacked_body = super::floating_window_resize::resize_stack_element(
            cx,
            window_id,
            clipped_body,
            blocker,
            resizable_layout,
            collapsed,
            resize_enabled,
            options.activate_on_click,
            handle_test_ids,
        );

        vec![stacked_body]
    })
}
