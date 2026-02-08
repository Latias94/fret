use std::{cell::RefCell, ops::Deref, rc::Rc};

use fret_core::{FrameId, Point, Px, Size};

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum ScrollStrategy {
    Start,
    Center,
    End,
    Nearest,
}

#[derive(Debug, Default)]
struct ScrollHandleState {
    offset: Point,
    viewport: Size,
    content: Size,
    revision: u64,
}

/// A lightweight imperative handle for driving scroll state.
///
/// This is intentionally small and allocation-free to clone, so component-layer code can store it
/// and pass it back into declarative elements each frame.
#[derive(Debug, Default, Clone)]
pub struct ScrollHandle {
    state: Rc<RefCell<ScrollHandleState>>,
}

impl ScrollHandle {
    pub(crate) fn binding_key(&self) -> usize {
        Rc::as_ptr(&self.state) as usize
    }

    pub(crate) fn bump_revision(&self) {
        let mut state = self.state.borrow_mut();
        state.revision = state.revision.saturating_add(1);
    }

    pub fn offset(&self) -> Point {
        self.state.borrow().offset
    }

    /// Monotonic revision counter that increments when the scroll offset changes via
    /// [`ScrollHandle::set_offset`] (or helpers that call it).
    ///
    /// The declarative runtime uses this to detect out-of-band scroll changes (e.g.
    /// component-driven "scroll into view") and invalidate the bound scroll nodes, even when the
    /// element instances themselves did not change.
    pub fn revision(&self) -> u64 {
        self.state.borrow().revision
    }

    pub fn max_offset(&self) -> Point {
        let state = self.state.borrow();
        Point::new(
            Px((state.content.width.0 - state.viewport.width.0).max(0.0)),
            Px((state.content.height.0 - state.viewport.height.0).max(0.0)),
        )
    }

    pub fn clamp_offset(&self, offset: Point) -> Point {
        let state = self.state.borrow();
        let max_x = (state.content.width.0 - state.viewport.width.0).max(0.0);
        let max_y = (state.content.height.0 - state.viewport.height.0).max(0.0);
        let clamp_x = state.viewport.width.0 > 0.0 && state.content.width.0 > 0.0;
        let clamp_y = state.viewport.height.0 > 0.0 && state.content.height.0 > 0.0;

        let x = offset.x.0.max(0.0);
        let y = offset.y.0.max(0.0);
        Point::new(
            Px(if clamp_x { x.min(max_x) } else { x }),
            Px(if clamp_y { y.min(max_y) } else { y }),
        )
    }

    pub fn set_offset(&self, offset: Point) {
        let clamped = self.clamp_offset(offset);
        let mut state = self.state.borrow_mut();
        if (state.offset.x.0 - clamped.x.0).abs() <= 0.01
            && (state.offset.y.0 - clamped.y.0).abs() <= 0.01
        {
            return;
        }
        state.offset = clamped;
        state.revision = state.revision.saturating_add(1);
    }

    /// Internal offset setter used by the runtime during layout passes.
    ///
    /// Unlike [`ScrollHandle::set_offset`], this does **not** bump the handle revision because the
    /// runtime is already invalidating and recomputing layout/paint in the same frame.
    pub(crate) fn set_offset_internal(&self, offset: Point) {
        let clamped = self.clamp_offset(offset);
        self.state.borrow_mut().offset = clamped;
    }

    pub fn scroll_to_offset(&self, offset: Point) {
        self.set_offset(offset);
    }

    pub fn viewport_size(&self) -> Size {
        self.state.borrow().viewport
    }

    pub fn set_viewport_size(&self, viewport: Size) {
        let mut state = self.state.borrow_mut();
        let next = Size::new(
            Px(viewport.width.0.max(0.0)),
            Px(viewport.height.0.max(0.0)),
        );
        if state.viewport != next {
            state.viewport = next;
            state.revision = state.revision.saturating_add(1);
        }
    }

    /// Internal viewport setter used by the runtime during layout passes.
    ///
    /// Unlike [`ScrollHandle::set_viewport_size`], this does **not** bump the handle revision
    /// because the runtime is already invalidating and recomputing layout/paint in the same frame.
    pub(crate) fn set_viewport_size_internal(&self, viewport: Size) {
        let mut state = self.state.borrow_mut();
        state.viewport = Size::new(
            Px(viewport.width.0.max(0.0)),
            Px(viewport.height.0.max(0.0)),
        );
    }

    pub fn content_size(&self) -> Size {
        self.state.borrow().content
    }

    pub fn set_content_size(&self, content: Size) {
        let mut state = self.state.borrow_mut();
        let next = Size::new(Px(content.width.0.max(0.0)), Px(content.height.0.max(0.0)));
        if state.content != next {
            state.content = next;
            state.revision = state.revision.saturating_add(1);
        }
    }

    /// Internal content-size setter used by the runtime during layout passes.
    ///
    /// Unlike [`ScrollHandle::set_content_size`], this does **not** bump the handle revision
    /// because the runtime is already invalidating and recomputing layout/paint in the same frame.
    pub(crate) fn set_content_size_internal(&self, content: Size) {
        let mut state = self.state.borrow_mut();
        state.content = Size::new(Px(content.width.0.max(0.0)), Px(content.height.0.max(0.0)));
    }

    pub fn scroll_to_range_y(&self, start_y: Px, end_y: Px, strategy: ScrollStrategy) {
        let start_y = Px(start_y.0.max(0.0));
        let end_y = Px(end_y.0.max(start_y.0));

        let viewport_h = Px(self.viewport_size().height.0.max(0.0));
        if viewport_h.0 <= 0.0 {
            return;
        }

        let prev = self.offset();
        let view_top = prev.y;
        let view_bottom = Px(view_top.0 + viewport_h.0);

        let next_y = match strategy {
            ScrollStrategy::Start => start_y,
            ScrollStrategy::End => Px(end_y.0 - viewport_h.0),
            ScrollStrategy::Center => {
                let center = 0.5 * (start_y.0 + end_y.0);
                Px(center - 0.5 * viewport_h.0)
            }
            ScrollStrategy::Nearest => {
                if start_y.0 < view_top.0 {
                    start_y
                } else if end_y.0 > view_bottom.0 {
                    Px(end_y.0 - viewport_h.0)
                } else {
                    view_top
                }
            }
        };

        self.set_offset(Point::new(prev.x, next_y));
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
struct DeferredScrollToItem {
    index: usize,
    strategy: ScrollStrategy,
}

#[derive(Debug, Default)]
struct VirtualListScrollHandleState {
    items_count: usize,
    deferred: Option<DeferredScrollToItem>,
    last_consumed: Option<DeferredScrollToItem>,
    last_consumed_revision: u64,
    last_consumed_frame_id: FrameId,
}

/// A scroll handle with VirtualList-specific helpers (scroll-to-item).
#[derive(Debug, Default, Clone)]
pub struct VirtualListScrollHandle {
    state: Rc<RefCell<VirtualListScrollHandleState>>,
    base_handle: ScrollHandle,
}

impl Deref for VirtualListScrollHandle {
    type Target = ScrollHandle;

    fn deref(&self) -> &Self::Target {
        &self.base_handle
    }
}

impl VirtualListScrollHandle {
    pub fn new() -> Self {
        Self::default()
    }

    pub fn base_handle(&self) -> &ScrollHandle {
        &self.base_handle
    }

    pub fn scroll_to_item(&self, index: usize, strategy: ScrollStrategy) {
        let current_revision = self.base_handle.revision();
        let mut state = self.state.borrow_mut();
        let next = DeferredScrollToItem { index, strategy };
        if state.deferred == Some(next) {
            return;
        }
        if state.deferred.is_none()
            && state.last_consumed == Some(next)
            && state.last_consumed_revision == current_revision
        {
            return;
        }
        state.deferred = Some(next);
        self.base_handle.bump_revision();
    }

    pub fn scroll_to_index(&self, index: usize, strategy: ScrollStrategy) {
        self.scroll_to_item(index, strategy);
    }

    pub fn scroll_to_bottom(&self) {
        // Avoid relying on the runtime-driven `items_count` bookkeeping for "scroll to end".
        // The layout pass will clamp the requested index against the current list count.
        self.scroll_to_item(usize::MAX, ScrollStrategy::End);
    }

    pub(crate) fn set_items_count(&self, items_count: usize) {
        self.state.borrow_mut().items_count = items_count;
    }

    pub(crate) fn deferred_scroll_to_item(&self) -> Option<(usize, ScrollStrategy)> {
        self.state.borrow().deferred.map(|d| (d.index, d.strategy))
    }

    pub(crate) fn clear_deferred_scroll_to_item(&self, frame_id: FrameId) {
        let mut state = self.state.borrow_mut();
        if let Some(deferred) = state.deferred {
            state.last_consumed = Some(deferred);
            state.last_consumed_revision = self.base_handle.revision();
            state.last_consumed_frame_id = frame_id;
        }
        state.deferred = None;
    }

    pub(crate) fn scroll_to_item_consumed_in_frame(&self, frame_id: FrameId) -> bool {
        let state = self.state.borrow();
        state.last_consumed.is_some() && state.last_consumed_frame_id == frame_id
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn scroll_handle_clamps_offset_to_content_bounds() {
        let handle = ScrollHandle::default();
        handle.set_viewport_size(Size::new(Px(10.0), Px(10.0)));
        handle.set_content_size(Size::new(Px(20.0), Px(30.0)));

        handle.set_offset(Point::new(Px(-5.0), Px(999.0)));
        assert_eq!(handle.offset(), Point::new(Px(0.0), Px(20.0)));
    }

    #[test]
    fn scroll_handle_internal_setters_do_not_bump_revision() {
        let handle = ScrollHandle::default();
        let rev0 = handle.revision();

        handle.set_viewport_size_internal(Size::new(Px(10.0), Px(10.0)));
        handle.set_content_size_internal(Size::new(Px(20.0), Px(30.0)));
        handle.set_offset_internal(Point::new(Px(0.0), Px(5.0)));
        assert_eq!(handle.revision(), rev0);

        handle.set_viewport_size(Size::new(Px(11.0), Px(10.0)));
        assert_eq!(handle.revision(), rev0.saturating_add(1));
    }

    #[test]
    fn virtual_list_scroll_to_bottom_requests_end_sentinel() {
        let handle = VirtualListScrollHandle::default();
        handle.scroll_to_bottom();
        assert_eq!(
            handle.deferred_scroll_to_item(),
            Some((usize::MAX, ScrollStrategy::End))
        );
    }

    #[test]
    fn scroll_handle_scroll_to_range_nearest_keeps_range_visible() {
        let handle = ScrollHandle::default();
        handle.set_viewport_size(Size::new(Px(10.0), Px(10.0)));
        handle.set_content_size(Size::new(Px(10.0), Px(100.0)));

        handle.set_offset(Point::new(Px(0.0), Px(20.0)));
        handle.scroll_to_range_y(Px(25.0), Px(28.0), ScrollStrategy::Nearest);
        assert_eq!(handle.offset().y, Px(20.0));

        handle.scroll_to_range_y(Px(5.0), Px(8.0), ScrollStrategy::Nearest);
        assert_eq!(handle.offset().y, Px(5.0));

        handle.scroll_to_range_y(Px(95.0), Px(99.0), ScrollStrategy::Nearest);
        assert_eq!(handle.offset().y, Px(89.0));
    }

    #[test]
    fn virtual_list_scroll_to_item_does_not_bump_revision_when_reissued_without_context_change() {
        let handle = VirtualListScrollHandle::new();
        handle.set_viewport_size(Size::new(Px(10.0), Px(10.0)));
        handle.set_content_size(Size::new(Px(10.0), Px(100.0)));

        let initial_revision = handle.revision();
        handle.scroll_to_item(5, ScrollStrategy::Nearest);
        let first_rev = handle.revision();
        assert!(first_rev > initial_revision);

        // Simulate runtime consumption: the request was consumed, but did not necessarily change
        // the offset. Reissuing the same request should not keep bumping the revision forever.
        handle.clear_deferred_scroll_to_item(FrameId(1));
        handle.scroll_to_item(5, ScrollStrategy::Nearest);
        assert_eq!(handle.revision(), first_rev);

        // A context change (e.g. content size change) should allow the same request to be issued
        // again, because it may become meaningful after layout changes.
        handle.set_content_size(Size::new(Px(10.0), Px(120.0)));
        let after_context = handle.revision();
        handle.scroll_to_item(5, ScrollStrategy::Nearest);
        assert!(handle.revision() > after_context);
    }
}
