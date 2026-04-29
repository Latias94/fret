use std::sync::Arc;

use fret_ui::GlobalElementId;

use super::super::options::TableSortDirection;
use super::hover::ResponseExt;

#[derive(Debug, Clone, Copy, Default)]
pub struct DisclosureResponse {
    pub trigger: ResponseExt,
    pub open: bool,
    pub toggled: bool,
}

#[derive(Debug, Clone, Copy, Default)]
pub struct ComboResponse {
    pub trigger: ResponseExt,
    pub open: bool,
    pub toggled: bool,
}

/// Aggregated response surface for helper-owned tab bars.
#[derive(Debug, Clone, Default)]
pub struct TabBarResponse {
    pub selected: Option<Arc<str>>,
    pub selected_changed: bool,
    pub triggers: Vec<TabTriggerResponse>,
}

/// Outward trigger response for a single helper-owned tab item.
#[derive(Debug, Clone)]
pub struct TabTriggerResponse {
    pub id: Arc<str>,
    pub selected: bool,
    pub trigger: ResponseExt,
}

/// Aggregated response surface for helper-owned table headers.
#[derive(Debug, Clone, Default)]
pub struct TableResponse {
    pub headers: Vec<TableHeaderResponse>,
}

/// Outward response for a single helper-owned table header cell.
#[derive(Debug, Clone)]
pub struct TableHeaderResponse {
    pub column_index: usize,
    pub column_id: Option<Arc<str>>,
    pub sortable: bool,
    pub sort_direction: Option<TableSortDirection>,
    pub trigger: ResponseExt,
}

#[derive(Debug, Clone, Default)]
pub struct VirtualListResponse {
    pub handle: fret_ui::scroll::VirtualListScrollHandle,
    pub rendered_range: Option<(usize, usize)>,
}

impl DisclosureResponse {
    pub fn id(self) -> Option<GlobalElementId> {
        self.trigger.id
    }

    pub fn open(self) -> bool {
        self.open
    }

    pub fn toggled(self) -> bool {
        self.toggled
    }

    pub fn clicked(self) -> bool {
        self.trigger.clicked()
    }

    pub fn opened(self) -> bool {
        self.toggled && self.open
    }

    pub fn closed(self) -> bool {
        self.toggled && !self.open
    }

    pub fn hovered_like_imgui(self) -> bool {
        self.trigger.hovered_like_imgui()
    }
}

impl ComboResponse {
    pub fn id(self) -> Option<GlobalElementId> {
        self.trigger.id
    }

    pub fn open(self) -> bool {
        self.open
    }

    pub fn toggled(self) -> bool {
        self.toggled
    }

    pub fn opened(self) -> bool {
        self.toggled && self.open
    }

    pub fn closed(self) -> bool {
        self.toggled && !self.open
    }

    pub fn clicked(self) -> bool {
        self.trigger.clicked()
    }

    pub fn hovered_like_imgui(self) -> bool {
        self.trigger.hovered_like_imgui()
    }
}

impl TabBarResponse {
    pub fn selected_id(&self) -> Option<&str> {
        self.selected.as_deref()
    }

    pub fn selected_changed(&self) -> bool {
        self.selected_changed
    }

    pub fn triggers(&self) -> &[TabTriggerResponse] {
        &self.triggers
    }

    pub fn trigger(&self, id: &str) -> Option<&TabTriggerResponse> {
        self.triggers
            .iter()
            .find(|trigger| trigger.id.as_ref() == id)
    }
}

impl TabTriggerResponse {
    pub fn id(&self) -> &str {
        self.id.as_ref()
    }

    pub fn selected(&self) -> bool {
        self.selected
    }

    pub fn response(&self) -> ResponseExt {
        self.trigger
    }

    pub fn clicked(&self) -> bool {
        self.trigger.clicked()
    }

    pub fn activated(&self) -> bool {
        self.trigger.activated()
    }

    pub fn deactivated(&self) -> bool {
        self.trigger.deactivated()
    }
}

impl TableResponse {
    pub fn headers(&self) -> &[TableHeaderResponse] {
        &self.headers
    }

    pub fn header(&self, column_id: &str) -> Option<&TableHeaderResponse> {
        self.headers
            .iter()
            .find(|header| header.column_id.as_deref() == Some(column_id))
    }

    pub fn header_at(&self, column_index: usize) -> Option<&TableHeaderResponse> {
        self.headers
            .iter()
            .find(|header| header.column_index == column_index)
    }
}

impl TableHeaderResponse {
    pub fn column_id(&self) -> Option<&str> {
        self.column_id.as_deref()
    }

    pub fn response(&self) -> ResponseExt {
        self.trigger
    }

    pub fn clicked(&self) -> bool {
        self.trigger.clicked()
    }

    pub fn activated(&self) -> bool {
        self.trigger.activated()
    }

    pub fn deactivated(&self) -> bool {
        self.trigger.deactivated()
    }
}

impl VirtualListResponse {
    pub fn handle(&self) -> fret_ui::scroll::VirtualListScrollHandle {
        self.handle.clone()
    }

    pub fn rendered_range(&self) -> Option<(usize, usize)> {
        self.rendered_range
    }
}
