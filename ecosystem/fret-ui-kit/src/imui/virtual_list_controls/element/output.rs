use std::sync::Arc;

use fret_core::SemanticsRole;
use fret_ui::element::{AnyElement, SemanticsProps};
use fret_ui::scroll::VirtualListScrollHandle;
use fret_ui::{ElementContext, UiHost};

use super::super::super::VirtualListResponse;

pub(super) fn decorate_list_semantics<H: UiHost>(
    cx: &mut ElementContext<'_, H>,
    list: AnyElement,
    test_id: Option<Arc<str>>,
) -> AnyElement {
    if let Some(test_id) = test_id {
        let mut semantics = SemanticsProps::default();
        semantics.role = SemanticsRole::List;
        semantics.test_id = Some(test_id);
        cx.semantics(semantics, move |_cx| vec![list])
    } else {
        list
    }
}

pub(super) fn virtual_list_response(
    handle: VirtualListScrollHandle,
    rendered_range: Option<(usize, usize)>,
) -> VirtualListResponse {
    VirtualListResponse {
        handle,
        rendered_range,
    }
}
