use std::cell::Cell;
use std::rc::Rc;
use std::sync::Arc;

use fret_ui::GlobalElementId;

use super::super::super::{ChildRegionChrome, ScrollOptions};

pub(in crate::imui::child_region) struct ChildRegionScrollInput<Build> {
    pub(in crate::imui::child_region) build_focus: Option<Rc<Cell<Option<GlobalElementId>>>>,
    pub(in crate::imui::child_region) build: Build,
    pub(in crate::imui::child_region) chrome: ChildRegionChrome,
    pub(in crate::imui::child_region) scroll_layout: crate::LayoutRefinement,
    pub(in crate::imui::child_region) scroll_options: ScrollOptions,
    pub(in crate::imui::child_region) root_test_id: Option<Arc<str>>,
    pub(in crate::imui::child_region) content_test_id: Option<Arc<str>>,
}
