use std::cell::Cell;
use std::rc::Rc;

use fret_ui::element::AnyElement;
use fret_ui::{ElementContext, GlobalElementId, UiHost};

use super::super::{ImUiFacade, ScrollOptions};
use super::children::build_imui_children_with_focus;

pub(in crate::imui) fn scroll_container_element<H: UiHost>(
    cx: &mut ElementContext<'_, H>,
    build_focus: Option<Rc<Cell<Option<GlobalElementId>>>>,
    options: ScrollOptions,
    f: impl for<'cx2, 'a2> FnOnce(&mut ImUiFacade<'cx2, 'a2, H>),
) -> AnyElement {
    let layout = options.layout.clone();
    let test_id = options.test_id.clone();
    let viewport_test_id = options.viewport_test_id.clone();
    let mut builder = crate::ui::scroll_area_build(move |cx, out| {
        build_imui_children_with_focus(cx, out, build_focus, f);
    });
    builder = builder
        .layout(layout)
        .axis(options.axis)
        .show_scrollbars(options.show_scrollbar_x, options.show_scrollbar_y);
    if let Some(handle) = options.handle {
        builder = builder.handle(handle);
    }
    if let Some(test_id) = test_id {
        builder = builder.test_id(test_id);
    }
    if let Some(test_id) = viewport_test_id {
        builder = builder.viewport_test_id(test_id);
    }
    builder.into_element(cx)
}
