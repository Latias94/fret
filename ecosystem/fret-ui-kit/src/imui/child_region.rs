//! Immediate child-region helpers.

use std::cell::Cell;
use std::rc::Rc;

use fret_ui::element::AnyElement;
use fret_ui::{ElementContext, GlobalElementId, UiHost};

use super::{ChildRegionOptions, ChildRegionResponse, ImUiFacade};

mod entry;
mod resize;
mod resize_stack;
mod scroll;

pub(super) fn child_region_element<H: UiHost>(
    cx: &mut ElementContext<'_, H>,
    id: &str,
    build_focus: Option<Rc<Cell<Option<GlobalElementId>>>>,
    options: ChildRegionOptions,
    f: impl for<'cx2, 'a2> FnOnce(&mut ImUiFacade<'cx2, 'a2, H>),
) -> (AnyElement, ChildRegionResponse) {
    cx.keyed(id, |cx| {
        entry::child_region_keyed_element(cx, id, build_focus, options, f)
    })
}
