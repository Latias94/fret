use std::cell::Cell;
use std::rc::Rc;

use fret_ui::GlobalElementId;

pub(in crate::imui::popup_overlay::modal) fn modal_focus_state() -> Rc<Cell<Option<GlobalElementId>>>
{
    Rc::new(Cell::new(None::<GlobalElementId>))
}
