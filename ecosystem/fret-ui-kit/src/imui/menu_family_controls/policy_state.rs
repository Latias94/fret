use std::sync::Arc;

use fret_runtime::Model;

use crate::primitives::menubar::trigger_row as menubar_trigger_row;

#[derive(Debug, Clone)]
pub(in crate::imui) struct ImUiMenubarPolicyState {
    pub(in crate::imui) open_menu: Model<Option<Arc<str>>>,
    pub(in crate::imui) group_active: Model<Option<menubar_trigger_row::MenubarActiveTrigger>>,
    pub(in crate::imui) registry: Model<Vec<menubar_trigger_row::MenubarTriggerRowEntry>>,
    pub(in crate::imui) suppress_close_auto_focus_once: Model<bool>,
}
