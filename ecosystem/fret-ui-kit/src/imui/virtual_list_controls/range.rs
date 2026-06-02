use std::cell::Cell;
use std::rc::Rc;

#[derive(Clone)]
pub(super) struct VirtualListRenderedRangeTracker {
    first: Rc<Cell<Option<usize>>>,
    last: Rc<Cell<Option<usize>>>,
}

impl VirtualListRenderedRangeTracker {
    pub(super) fn new() -> Self {
        Self {
            first: Rc::new(Cell::new(None)),
            last: Rc::new(Cell::new(None)),
        }
    }

    pub(super) fn record(&self, index: usize) {
        if self.first.get().is_none() {
            self.first.set(Some(index));
        }
        self.last.set(Some(index));
    }

    pub(super) fn range(&self) -> Option<(usize, usize)> {
        self.first.get().zip(self.last.get())
    }
}
