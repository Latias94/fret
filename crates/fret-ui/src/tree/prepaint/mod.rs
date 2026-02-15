use super::*;
use crate::cache_key::CacheKeyBuilder;

const RETAINED_HOST_PREFETCH_STEP_MAX: usize = 12;

#[derive(Clone)]
struct VirtualListPrepaintInputs {
    element: GlobalElementId,
    axis: fret_core::Axis,
    len: usize,
    items_revision: u64,
    measure_mode: crate::element::VirtualListMeasureMode,
    overscan: usize,
    estimate_row_height: Px,
    gap: Px,
    scroll_margin: Px,
    scroll_handle: crate::scroll::VirtualListScrollHandle,
}

#[derive(Debug, Clone, Copy)]
struct VirtualListPrepaintWindowUpdate {
    prev_items_revision: u64,
    prev_viewport: Px,
    prev_offset: Px,
    prev_window_range: Option<crate::virtual_list::VirtualRange>,
    render_window_range: Option<crate::virtual_list::VirtualRange>,
    window_range: Option<crate::virtual_list::VirtualRange>,
    viewport: Px,
    offset: Px,
    deferred_scroll_to_item: bool,
    window_mismatch: bool,
    window_shift_kind: crate::tree::UiDebugVirtualListWindowShiftKind,
    content_extent: Px,
}

#[derive(Debug, Clone, Copy, PartialEq)]
pub(super) struct InteractionCacheEntry {
    pub(super) generation: u64,
    pub(super) key: PaintCacheKey,
    pub(super) start: u32,
    pub(super) end: u32,
}

#[derive(Debug, Clone, Copy)]
#[allow(dead_code)]
pub(super) struct InteractionRecord {
    pub(super) node: NodeId,
    pub(super) bounds: Rect,
    pub(super) render_transform_inv: Option<Transform2D>,
    pub(super) children_render_transform_inv: Option<Transform2D>,
    pub(super) clips_hit_test: bool,
    pub(super) clip_hit_test_corner_radii: Option<Corners>,
    pub(super) is_focusable: bool,
    pub(super) focus_traversal_children: bool,
    pub(super) can_scroll_descendant_into_view: bool,
}

#[derive(Debug, Default)]
pub(super) struct InteractionCacheState {
    generation: u64,
    pub(super) prev_records: Vec<InteractionRecord>,
    pub(super) records: Vec<InteractionRecord>,
    replay_scratch: Vec<InteractionRecord>,
    pub(super) source_generation: u64,
    pub(super) target_generation: u64,
    pub(super) hits: u32,
    pub(super) misses: u32,
    pub(super) replayed_records: u32,
}

impl InteractionCacheState {
    pub(super) fn begin_frame(&mut self) {
        self.source_generation = self.generation;
        self.target_generation = self.generation.saturating_add(1);
        self.hits = 0;
        self.misses = 0;
        self.replayed_records = 0;

        std::mem::swap(&mut self.prev_records, &mut self.records);
        self.records.clear();
    }

    pub(super) fn finish_frame(&mut self) {
        self.generation = self.target_generation;
    }

    pub(super) fn invalidate_recording(&mut self) {
        self.prev_records.clear();
        self.records.clear();
        self.replay_scratch.clear();
        self.generation = self.generation.saturating_add(1);
    }
}

mod entry;
mod interaction;
mod virtual_list;

#[cfg(test)]
mod tests;
