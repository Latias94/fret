use fret_runtime::Model;
use fret_ui::element::AnyElement;
use fret_ui::{ElementContext, GlobalElementId, UiHost};

use super::super::super::{ImUiFacade, ResponseExt};
use super::super::{layout, spec::DisclosureSpec, trigger};

pub(super) fn disclosure_root_children<H: UiHost>(
    cx: &mut ElementContext<'_, H>,
    spec: &DisclosureSpec,
    open_model: Model<bool>,
    content_id: GlobalElementId,
    open_now: bool,
    enabled: bool,
    trigger_response: &mut ResponseExt,
    build: impl for<'cx2, 'a2> FnOnce(&mut ImUiFacade<'cx2, 'a2, H>),
) -> Vec<AnyElement> {
    let mut root_children = Vec::new();
    let header = trigger::disclosure_header_element(
        cx,
        spec.clone(),
        open_model,
        content_id,
        open_now,
        enabled,
        trigger_response,
    );
    root_children.push(header);

    if spec.has_children() && open_now {
        root_children.push(layout::disclosure_content_element(cx, spec, build));
    }

    root_children
}
