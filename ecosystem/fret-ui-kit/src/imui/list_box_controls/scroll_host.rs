use std::cell::Cell;
use std::rc::Rc;
use std::sync::Arc;

use fret_ui::element::{AnyElement, SemanticsDecoration};
use fret_ui::{ElementContext, GlobalElementId, UiHost};

use super::super::{ImUiFacade, ScrollOptions, containers::build_imui_children_with_focus};

pub(in crate::imui::list_box_controls) fn list_box_scroll_host_element<H: UiHost>(
    cx: &mut ElementContext<'_, H>,
    build_focus: Option<Rc<Cell<Option<GlobalElementId>>>>,
    layout: crate::LayoutRefinement,
    mut scroll_options: ScrollOptions,
    root_test_id: Option<Arc<str>>,
    content_test_id: Option<Arc<str>>,
    semantics: SemanticsDecoration,
    f: impl for<'cx2, 'a2> FnOnce(&mut ImUiFacade<'cx2, 'a2, H>),
) -> AnyElement {
    scroll_options.layout = layout.merge(scroll_options.layout);

    let mut element = crate::ui::scroll_area_build(move |cx, out| {
        let mut content = crate::ui::v_flex_build(move |cx, out| {
            build_imui_children_with_focus(cx, out, build_focus, f);
        })
        .items(crate::Items::Stretch)
        .no_wrap();

        if let Some(test_id) = content_test_id.clone() {
            content = content.test_id(test_id);
        }

        out.push(content.into_element(cx));
    })
    .layout(scroll_options.layout)
    .axis(scroll_options.axis)
    .show_scrollbars(
        scroll_options.show_scrollbar_x,
        scroll_options.show_scrollbar_y,
    );

    if let Some(handle) = scroll_options.handle {
        element = element.handle(handle);
    }
    if let Some(test_id) = scroll_options.viewport_test_id {
        element = element.viewport_test_id(test_id);
    }
    if let Some(test_id) = root_test_id {
        element = element.test_id(test_id);
    }

    element.semantics(semantics).into_element(cx)
}
