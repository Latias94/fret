//! Immediate listbox container helpers.

use std::cell::Cell;
use std::rc::Rc;

use fret_core::SemanticsRole;
use fret_ui::element::SemanticsDecoration;
use fret_ui::{ElementContext, GlobalElementId, UiHost};

use super::{ImUiFacade, ListBoxOptions, containers::build_imui_children_with_focus};

pub(super) fn list_box_element<H: UiHost>(
    cx: &mut ElementContext<'_, H>,
    id: &str,
    build_focus: Option<Rc<Cell<Option<GlobalElementId>>>>,
    options: ListBoxOptions,
    f: impl for<'cx2, 'a2> FnOnce(&mut ImUiFacade<'cx2, 'a2, H>),
) -> fret_ui::element::AnyElement {
    cx.keyed(id, |cx| {
        let mut scroll_options = options.scroll.clone();
        scroll_options.layout = options.layout.clone().merge(scroll_options.layout);

        let content_test_id = options.content_test_id.clone();
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
        if let Some(test_id) = options.test_id.clone() {
            element = element.test_id(test_id);
        }

        let mut semantics = SemanticsDecoration::default().role(SemanticsRole::ListBox);
        if let Some(label) = options.label {
            semantics = semantics.label(label);
        }
        if options.multiselectable {
            semantics = semantics.multiselectable(true);
        }

        element.semantics(semantics).into_element(cx)
    })
}
