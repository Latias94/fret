use std::sync::Arc;

use fret_ui::UiHost;

mod body;
mod state;

use super::super::label_identity::parse_label_identity;
use super::super::{
    CollapsingHeaderOptions, DisclosureResponse, ImUiFacade, TreeNodeOptions, UiWriterImUiFacadeExt,
};
use super::{layout, spec::DisclosureSpec};

pub(in crate::imui) fn collapsing_header_with_options<
    H: UiHost,
    W: UiWriterImUiFacadeExt<H> + ?Sized,
>(
    ui: &mut W,
    id: &str,
    label: Arc<str>,
    options: CollapsingHeaderOptions,
    f: impl for<'cx2, 'a2> FnOnce(&mut ImUiFacade<'cx2, 'a2, H>),
) -> DisclosureResponse {
    let parts = parse_label_identity(label.as_ref());
    let label = Arc::<str>::from(parts.visible);
    disclosure_with_options(ui, id, DisclosureSpec::collapsing_header(label, options), f)
}

pub(in crate::imui) fn tree_node_with_options<H: UiHost, W: UiWriterImUiFacadeExt<H> + ?Sized>(
    ui: &mut W,
    id: &str,
    label: Arc<str>,
    options: TreeNodeOptions,
    f: impl for<'cx2, 'a2> FnOnce(&mut ImUiFacade<'cx2, 'a2, H>),
) -> DisclosureResponse {
    let parts = parse_label_identity(label.as_ref());
    let label = Arc::<str>::from(parts.visible);
    disclosure_with_options(ui, id, DisclosureSpec::tree_node(label, options), f)
}

fn disclosure_with_options<H: UiHost, W: UiWriterImUiFacadeExt<H> + ?Sized>(
    ui: &mut W,
    id: &str,
    spec: DisclosureSpec,
    f: impl for<'cx2, 'a2> FnOnce(&mut ImUiFacade<'cx2, 'a2, H>),
) -> DisclosureResponse {
    let mut response = DisclosureResponse::empty();

    let element = ui.with_cx_mut(|cx| {
        let scope_key = format!("fret-ui-kit.imui.disclosure.{id}");
        cx.named(scope_key.as_str(), |cx| {
            let trigger_response = &mut response.trigger;
            let disclosure_state = state::prepare_disclosure_entry_state(cx, &spec);
            let content_id = cx.named("content", |cx| cx.root_id());
            let root_children = body::disclosure_root_children(
                cx,
                &spec,
                disclosure_state.open_model,
                content_id,
                disclosure_state.open_now,
                disclosure_state.enabled,
                trigger_response,
                f,
            );

            response.open = disclosure_state.open_now;
            response.toggled = disclosure_state.toggled;

            layout::disclosure_root_element(cx, &spec, root_children)
        })
    });

    ui.add(element);
    response
}
