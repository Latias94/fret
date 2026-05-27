use std::cell::Cell;
use std::rc::Rc;

use fret_ui::element::AnyElement;
use fret_ui::{ElementContext, GlobalElementId, UiHost};

use super::super::ImUiFacade;

pub(in crate::imui) fn build_imui_children_with_focus<H: UiHost>(
    cx: &mut ElementContext<'_, H>,
    out: &mut Vec<AnyElement>,
    build_focus: Option<Rc<Cell<Option<GlobalElementId>>>>,
    f: impl for<'cx2, 'a2> FnOnce(&mut ImUiFacade<'cx2, 'a2, H>),
) {
    let mut ui = ImUiFacade {
        cx,
        out,
        build_focus,
    };
    f(&mut ui);
}
