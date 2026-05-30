use std::cell::Cell;
use std::rc::Rc;

use fret_ui::{ElementContext, UiHost};

#[derive(Default)]
struct ImUiDisabledScopeStore {
    depth: Rc<Cell<u32>>,
}

pub(in crate::imui) fn disabled_scope_depth_for<H: UiHost>(
    cx: &mut ElementContext<'_, H>,
) -> Rc<Cell<u32>> {
    cx.app
        .with_global_mut_untracked(ImUiDisabledScopeStore::default, |st, _app| st.depth.clone())
}
