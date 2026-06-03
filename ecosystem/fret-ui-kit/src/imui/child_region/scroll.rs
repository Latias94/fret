use fret_ui::element::AnyElement;
use fret_ui::{ElementContext, UiHost};

use super::super::{ImUiFacade, containers::build_imui_children_with_focus};

mod chrome;
mod types;

use chrome::apply_child_region_scroll_chrome;
pub(super) use types::ChildRegionScrollInput;

pub(super) fn child_region_scroll_element<H: UiHost>(
    cx: &mut ElementContext<'_, H>,
    input: ChildRegionScrollInput<impl for<'cx2, 'a2> FnOnce(&mut ImUiFacade<'cx2, 'a2, H>)>,
) -> AnyElement {
    let ChildRegionScrollInput {
        build_focus,
        build,
        chrome,
        scroll_layout,
        scroll_options,
        root_test_id,
        content_test_id,
    } = input;

    let viewport_test_id = scroll_options.viewport_test_id.clone();
    let builder = crate::ui::scroll_area_build(move |cx, out| {
        let mut content = crate::ui::v_flex_build(move |cx, out| {
            build_imui_children_with_focus(cx, out, build_focus, build);
        })
        .no_wrap();

        if let Some(test_id) = content_test_id.clone() {
            content = content.test_id(test_id);
        }

        out.push(content.into_element(cx));
    })
    .axis(scroll_options.axis)
    .show_scrollbars(
        scroll_options.show_scrollbar_x,
        scroll_options.show_scrollbar_y,
    )
    .layout(scroll_layout);

    let mut builder = apply_child_region_scroll_chrome(builder, chrome);

    if let Some(handle) = scroll_options.handle {
        builder = builder.handle(handle);
    }

    if let Some(test_id) = viewport_test_id {
        builder = builder.viewport_test_id(test_id);
    }

    if let Some(test_id) = root_test_id {
        builder = builder.test_id(test_id);
    }

    builder.into_element(cx)
}
