use std::sync::Arc;

use crate::element::AnyElement;
use crate::elements::ElementContext;
use crate::virtual_list::VirtualRange;
use crate::{ItemKey, UiHost};

pub(crate) type RetainedVirtualListKeyAtFn = Arc<dyn Fn(usize) -> ItemKey>;

pub(crate) type RetainedVirtualListRowFn<H> =
    Arc<dyn for<'a> Fn(&mut ElementContext<'a, H>, usize) -> AnyElement>;

pub(crate) type RetainedVirtualListRangeExtractor = fn(VirtualRange) -> Vec<usize>;

#[derive(Default)]
pub(crate) struct RetainedVirtualListHostMarker;

pub(crate) struct RetainedVirtualListHostCallbacks<H: UiHost> {
    pub key_at: RetainedVirtualListKeyAtFn,
    pub row: RetainedVirtualListRowFn<H>,
    pub range_extractor: RetainedVirtualListRangeExtractor,
}
