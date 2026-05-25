use std::sync::Arc;

use fret_core::Px;
use fret_ui::GlobalElementId;

use super::super::options::TableSortDirection;
use super::drag::DragResponse;
use super::hover::ResponseExt;

mod child_region;

pub use child_region::{
    ChildRegionResizeXResponse, ChildRegionResizeYResponse, ChildRegionResponse,
};

#[derive(Debug, Clone, Copy)]
pub struct DisclosureResponse {
    pub(crate) trigger: ResponseExt,
    pub(crate) open: bool,
    pub(crate) toggled: bool,
}

#[derive(Debug, Clone, Copy)]
pub struct ComboResponse {
    pub(crate) trigger: ResponseExt,
    pub(crate) open: bool,
    pub(crate) toggled: bool,
}

#[derive(Debug, Clone)]
pub struct InputTextPickerResponse {
    pub(crate) input: ResponseExt,
    pub(crate) open: bool,
    pub(crate) picked_index: Option<usize>,
    pub(crate) picked: Option<Arc<str>>,
}

/// Aggregated response surface for helper-owned tab bars.
#[derive(Debug, Clone)]
pub struct TabBarResponse {
    pub(crate) selected: Option<Arc<str>>,
    pub(crate) selected_changed: bool,
    pub(crate) triggers: Vec<TabTriggerResponse>,
}

/// Outward trigger response for a single helper-owned tab item.
#[derive(Debug, Clone)]
pub struct TabTriggerResponse {
    pub(crate) id: Arc<str>,
    pub(crate) selected: bool,
    pub(crate) trigger: ResponseExt,
}

/// Aggregated response surface for helper-owned table headers.
#[derive(Debug, Clone)]
pub struct TableResponse {
    pub(crate) headers: Vec<TableHeaderResponse>,
}

/// Outward response for a single helper-owned table header cell.
#[derive(Debug, Clone)]
pub struct TableHeaderResponse {
    pub(crate) column_index: usize,
    pub(crate) column_id: Option<Arc<str>>,
    pub(crate) sortable: bool,
    pub(crate) sort_direction: Option<TableSortDirection>,
    pub(crate) trigger: ResponseExt,
    pub(crate) resize: TableColumnResizeResponse,
}

#[derive(Debug, Clone)]
pub struct TableColumnResizeResponse {
    pub(crate) column_index: usize,
    pub(crate) column_id: Option<Arc<str>>,
    pub(crate) enabled: bool,
    pub(crate) min_width: Option<fret_core::Px>,
    pub(crate) max_width: Option<fret_core::Px>,
    pub(crate) drag: DragResponse,
}

#[derive(Debug, Clone)]
pub struct VirtualListResponse {
    pub(crate) handle: fret_ui::scroll::VirtualListScrollHandle,
    pub(crate) rendered_range: Option<(usize, usize)>,
}

impl DisclosureResponse {
    pub(crate) fn empty() -> Self {
        Self {
            trigger: ResponseExt::default(),
            open: false,
            toggled: false,
        }
    }

    pub fn id(self) -> Option<GlobalElementId> {
        self.trigger.id()
    }

    pub fn response(self) -> ResponseExt {
        self.trigger
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
        self.trigger.id()
    }

    pub fn response(self) -> ResponseExt {
        self.trigger
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

impl InputTextPickerResponse {
    pub fn id(&self) -> Option<GlobalElementId> {
        self.input.id()
    }

    pub fn response(&self) -> ResponseExt {
        self.input
    }

    pub fn open(&self) -> bool {
        self.open
    }

    pub fn changed(&self) -> bool {
        self.input.changed()
    }

    pub fn picked(&self) -> Option<&str> {
        self.picked.as_deref()
    }

    pub fn picked_index(&self) -> Option<usize> {
        self.picked_index
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
    pub fn column_index(&self) -> usize {
        self.column_index
    }

    pub fn column_id(&self) -> Option<&str> {
        self.column_id.as_deref()
    }

    pub fn sortable(&self) -> bool {
        self.sortable
    }

    pub fn sort_direction(&self) -> Option<TableSortDirection> {
        self.sort_direction
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

    pub fn resizing(&self) -> bool {
        self.resize.dragging()
    }

    pub fn resize(&self) -> &TableColumnResizeResponse {
        &self.resize
    }
}

impl TableColumnResizeResponse {
    pub fn column_index(&self) -> usize {
        self.column_index
    }

    pub fn column_id(&self) -> Option<&str> {
        self.column_id.as_deref()
    }

    pub fn enabled(&self) -> bool {
        self.enabled
    }

    pub fn min_width(&self) -> Option<Px> {
        self.min_width
    }

    pub fn max_width(&self) -> Option<Px> {
        self.max_width
    }

    pub fn dragging(&self) -> bool {
        self.drag.dragging
    }

    pub fn drag_started(&self) -> bool {
        self.drag.started
    }

    pub fn drag_stopped(&self) -> bool {
        self.drag.stopped
    }

    pub fn drag_delta_x(&self) -> f32 {
        self.drag.delta.x.0
    }

    pub fn drag_total_x(&self) -> f32 {
        self.drag.total.x.0
    }

    pub fn width_from_start(&self, start_width: Px) -> Px {
        let min = self.min_width.map(|width| width.0).unwrap_or(0.0).max(0.0);
        let max = self.max_width.map(|width| width.0).unwrap_or(f32::INFINITY);
        Px((start_width.0 + self.drag_total_x()).clamp(min, max.max(min)))
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
