use fret_core::Size;
use fret_ui::element::AnyElement;
use fret_ui::{ElementContext, GlobalElementId, UiHost};

mod body;
mod props;

use props::window_frame_props;

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
        let stacked_body = body::floating_window_shell_body_element(
            cx,
            window_id,
            title_bar_row,
            content,
            resizable_layout,
            collapsed,
            resize_enabled,
            options,
            handle_test_ids,
            muted,
            border,
        );

        vec![stacked_body]
    })
}
