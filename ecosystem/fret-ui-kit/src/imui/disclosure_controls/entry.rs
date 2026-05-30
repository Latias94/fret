use std::sync::Arc;

use fret_ui::UiHost;

use super::super::label_identity::parse_label_identity;
use super::super::{
    CollapsingHeaderOptions, DisclosureResponse, ImUiFacade, TreeNodeOptions, UiWriterImUiFacadeExt,
};
use super::{layout, spec::DisclosureSpec, trigger};
use crate::declarative::ModelWatchExt;
use crate::primitives::collapsible as radix_collapsible;

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
            let root = radix_collapsible::CollapsibleRoot::new()
                .open(spec.open.clone())
                .default_open(spec.default_open);
            let open_model = root.use_open_model(cx).model();
            let open_now = if spec.has_children() {
                cx.watch_model(&open_model)
                    .layout()
                    .copied()
                    .unwrap_or(false)
            } else {
                false
            };
            let toggled = super::super::model_value_changed_for(cx, cx.root_id(), open_now);
            let enabled = spec.enabled && !super::super::imui_is_disabled(cx);
            let mut build = Some(f);
            let content_id = cx.named("content", |cx| cx.root_id());

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
                root_children.push(layout::disclosure_content_element(
                    cx,
                    &spec,
                    build
                        .take()
                        .expect("disclosure body builder should be available"),
                ));
            }

            response.open = open_now;
            response.toggled = toggled;

            layout::disclosure_root_element(cx, &spec, root_children)
        })
    });

    ui.add(element);
    response
}
