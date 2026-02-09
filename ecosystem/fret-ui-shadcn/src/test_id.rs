use std::sync::Arc;

use fret_ui::element::AnyElement;

pub(crate) fn attach_test_id(el: AnyElement, test_id: Arc<str>) -> AnyElement {
    el.test_id(test_id)
}

pub(crate) fn attach_test_id_suffix(
    el: AnyElement,
    prefix: Option<&Arc<str>>,
    suffix: &'static str,
) -> AnyElement {
    let Some(prefix) = prefix else {
        return el;
    };
    attach_test_id(el, Arc::<str>::from(format!("{prefix}-{suffix}")))
}
