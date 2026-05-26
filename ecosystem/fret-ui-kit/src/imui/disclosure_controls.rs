use std::sync::Arc;

use fret_core::Px;
use fret_ui::UiHost;
use fret_ui::element::{
    ColumnProps, ContainerProps, LayoutStyle, Length, Overflow, SizeStyle, SpacingLength,
};

use super::label_identity::parse_label_identity;
use super::{
    CollapsingHeaderOptions, DisclosureResponse, ImUiFacade, TreeNodeOptions, UiWriterImUiFacadeExt,
};
use crate::declarative::ModelWatchExt;
use crate::primitives::collapsible as radix_collapsible;

mod spec;
mod trigger;
mod visual;

use spec::DisclosureSpec;

#[cfg(test)]
use visual::{header_row, resolve_disclosure_palette};

pub(super) fn collapsing_header_with_options<H: UiHost, W: UiWriterImUiFacadeExt<H> + ?Sized>(
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

pub(super) fn tree_node_with_options<H: UiHost, W: UiWriterImUiFacadeExt<H> + ?Sized>(
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
            let toggled = super::model_value_changed_for(cx, cx.root_id(), open_now);
            let enabled = spec.enabled && !super::imui_is_disabled(cx);
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
                let mut content = cx.named("content", |cx| {
                    let mut props = ContainerProps::default();
                    props.layout = LayoutStyle {
                        size: SizeStyle {
                            width: Length::Fill,
                            height: Length::Auto,
                            ..Default::default()
                        },
                        overflow: Overflow::Visible,
                        ..Default::default()
                    };
                    props.padding = visual::disclosure_content_padding(&spec).into();

                    cx.container(props, move |cx| {
                        vec![cx.column(
                            ColumnProps {
                                layout: LayoutStyle {
                                    size: SizeStyle {
                                        width: Length::Fill,
                                        height: Length::Auto,
                                        ..Default::default()
                                    },
                                    overflow: Overflow::Visible,
                                    ..Default::default()
                                },
                                gap: SpacingLength::Px(Px(0.0)),
                                ..Default::default()
                            },
                            move |cx| {
                                let mut out = Vec::new();
                                let mut body_ui = ImUiFacade {
                                    cx,
                                    out: &mut out,
                                    build_focus: None,
                                };
                                if let Some(build) = build.take() {
                                    build(&mut body_ui);
                                }
                                out
                            },
                        )]
                    })
                });
                if let Some(test_id) = spec.content_test_id.as_ref() {
                    content = content.test_id(test_id.clone());
                }
                root_children.push(content);
            }

            response.open = open_now;
            response.toggled = toggled;

            let mut root = cx.column(
                ColumnProps {
                    layout: LayoutStyle {
                        size: SizeStyle {
                            width: Length::Fill,
                            height: Length::Auto,
                            ..Default::default()
                        },
                        overflow: Overflow::Visible,
                        ..Default::default()
                    },
                    gap: SpacingLength::Px(Px(0.0)),
                    ..Default::default()
                },
                move |_cx| root_children,
            );
            if let Some(test_id) = spec.root_test_id.as_ref() {
                root = root.test_id(test_id.clone());
            }
            root
        })
    });

    ui.add(element);
    response
}

#[cfg(test)]
mod tests;
