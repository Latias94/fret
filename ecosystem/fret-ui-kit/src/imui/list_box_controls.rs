//! Immediate listbox container helpers.

use std::cell::Cell;
use std::rc::Rc;

use fret_ui::{ElementContext, GlobalElementId, UiHost};

use super::{ImUiFacade, ListBoxOptions};

mod scroll_host;
mod semantics;

pub(super) fn list_box_element<H: UiHost>(
    cx: &mut ElementContext<'_, H>,
    id: &str,
    build_focus: Option<Rc<Cell<Option<GlobalElementId>>>>,
    options: ListBoxOptions,
    f: impl for<'cx2, 'a2> FnOnce(&mut ImUiFacade<'cx2, 'a2, H>),
) -> fret_ui::element::AnyElement {
    cx.keyed(id, |cx| {
        let ListBoxOptions {
            layout,
            scroll,
            label,
            multiselectable,
            test_id,
            content_test_id,
        } = options;

        let semantics = semantics::list_box_semantics(label, multiselectable);
        scroll_host::list_box_scroll_host_element(
            cx,
            build_focus,
            layout,
            scroll,
            test_id,
            content_test_id,
            semantics,
            f,
        )
    })
}
