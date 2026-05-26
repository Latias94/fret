use std::sync::Arc;

use fret_core::{KeyCode, MouseButton, Px};
use fret_ui::action::UiActionHostExt as _;
use fret_ui::action::{PressablePointerDownResult, PressablePointerUpResult};
use fret_ui::element::{
    ColumnProps, ContainerProps, LayoutStyle, Length, Overflow, PressableProps, SizeStyle,
    SpacingLength,
};
use fret_ui::{Invalidation, UiHost};

use super::label_identity::parse_label_identity;
use super::{
    CollapsingHeaderOptions, DisclosureResponse, ImUiFacade, TreeNodeOptions, UiWriterImUiFacadeExt,
};
use crate::declarative::ModelWatchExt;
use crate::primitives::collapsible as radix_collapsible;

mod spec;
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
            let active_item_model = super::active_item_model_for_window(cx);
            let mut build = Some(f);
            let content_id = cx.named("content", |cx| cx.root_id());
            let spec_for_header = spec.clone();

            let mut root_children = Vec::new();
            let header = cx.named("header", |cx| {
                let spec = spec_for_header.clone();
                let spec_for_pressable = spec.clone();
                let mut props = PressableProps::default();
                props.enabled = enabled;
                props.focusable = enabled;
                props.layout = LayoutStyle {
                    size: SizeStyle {
                        width: Length::Fill,
                        height: Length::Auto,
                        ..Default::default()
                    },
                    ..Default::default()
                };
                props.a11y = visual::disclosure_a11y(&spec, open_now);

                let mut header = cx.pressable_with_id(props, move |cx, state, trigger_id| {
                    let spec = spec_for_pressable.clone();
                    let context_anchor_model = super::context_menu_anchor_model_for(cx, trigger_id);
                    let context_anchor_model_for_report = context_anchor_model.clone();
                    cx.pressable_clear_on_pointer_down();
                    cx.pressable_clear_on_pointer_move();
                    cx.pressable_clear_on_pointer_up();
                    cx.key_clear_on_key_down_for(trigger_id);

                    let action_label = spec.label.clone();
                    let open_model_for_activate = open_model.clone();
                    let has_children = spec.has_children();
                    let activate_shortcut = spec.activate_shortcut;
                    let shortcut_repeat = spec.shortcut_repeat;
                    cx.pressable_on_activate(crate::on_activate(
                        move |host, action_cx, _reason| {
                            host.record_transient_event(action_cx, super::KEY_CLICKED);
                            if has_children {
                                let _ = host
                                    .models_mut()
                                    .update(&open_model_for_activate, |value| *value = !*value);
                            }
                            host.notify(action_cx);
                        },
                    ));

                    if enabled {
                        cx.key_on_key_down_for(
                            trigger_id,
                            Arc::new(move |host, acx, down| {
                                if let Some(shortcut) = activate_shortcut {
                                    let matches_shortcut =
                                        down.key == shortcut.key && down.modifiers == shortcut.mods;
                                    if matches_shortcut
                                        && (!down.repeat || shortcut_repeat)
                                        && !down.ime_composing
                                    {
                                        host.record_transient_event(acx, super::KEY_CLICKED);
                                        if has_children {
                                            let _ = host
                                                .models_mut()
                                                .update(&open_model, |value| *value = !*value);
                                        }
                                        host.notify(acx);
                                        return true;
                                    }
                                }

                                let is_menu_key = down.key == KeyCode::ContextMenu;
                                let is_shift_f10 = down.key == KeyCode::F10 && down.modifiers.shift;
                                if !(is_menu_key || is_shift_f10) {
                                    return false;
                                }

                                host.record_transient_event(acx, super::KEY_CONTEXT_MENU_REQUESTED);
                                host.notify(acx);
                                true
                            }),
                        );
                    }

                    cx.pressable_on_pointer_down(Arc::new(|_host, _acx, _down| {
                        PressablePointerDownResult::Continue
                    }));
                    cx.pressable_on_pointer_up(Arc::new(move |host, acx, up| {
                        if up.is_click && up.button == MouseButton::Right {
                            let _ = host.update_model(&context_anchor_model, |value| {
                                *value = Some(up.position)
                            });
                            host.record_transient_event(acx, super::KEY_SECONDARY_CLICKED);
                            host.record_transient_event(acx, super::KEY_CONTEXT_MENU_REQUESTED);
                            host.notify(acx);
                            return PressablePointerUpResult::SkipActivate;
                        }

                        if up.is_click && up.button == MouseButton::Left && up.click_count == 2 {
                            host.record_transient_event(acx, super::KEY_DOUBLE_CLICKED);
                            host.notify(acx);
                        }

                        PressablePointerUpResult::Continue
                    }));

                    trigger_response.set_core_hovered(state.hovered);
                    trigger_response.set_core_pressed(state.pressed);
                    trigger_response.set_core_focused(state.focused);
                    trigger_response.set_nav_highlighted(
                        state.focused
                            && fret_ui::focus_visible::is_focus_visible(cx.app, Some(cx.window)),
                    );
                    trigger_response.set_id(Some(trigger_id));
                    trigger_response
                        .set_core_clicked(cx.take_transient_for(trigger_id, super::KEY_CLICKED));
                    trigger_response.set_secondary_clicked(
                        cx.take_transient_for(trigger_id, super::KEY_SECONDARY_CLICKED),
                    );
                    trigger_response.set_double_clicked(
                        cx.take_transient_for(trigger_id, super::KEY_DOUBLE_CLICKED),
                    );
                    trigger_response.set_context_menu_requested(
                        cx.take_transient_for(trigger_id, super::KEY_CONTEXT_MENU_REQUESTED),
                    );
                    trigger_response.set_context_menu_anchor(
                        cx.read_model(
                            &context_anchor_model_for_report,
                            Invalidation::Paint,
                            |_app, value| *value,
                        )
                        .unwrap_or(None),
                    );
                    trigger_response.set_core_rect(cx.last_bounds_for_element(trigger_id));
                    let hover_delay = super::install_hover_query_hooks_for_pressable(
                        cx,
                        trigger_id,
                        state.hovered_raw,
                        None,
                    );
                    trigger_response.set_pointer_hovered_raw(state.hovered_raw);
                    trigger_response
                        .set_pointer_hovered_raw_below_barrier(state.hovered_raw_below_barrier);
                    trigger_response.set_hover_stationary_met(hover_delay.stationary_met);
                    trigger_response.set_hover_delay_short_met(hover_delay.delay_short_met);
                    trigger_response.set_hover_delay_normal_met(hover_delay.delay_normal_met);
                    trigger_response
                        .set_hover_delay_short_shared_met(hover_delay.shared_delay_short_met);
                    trigger_response
                        .set_hover_delay_normal_shared_met(hover_delay.shared_delay_normal_met);
                    trigger_response.set_hover_blocked_by_active_item(
                        super::hover_blocked_by_active_item_for(cx, trigger_id, &active_item_model),
                    );
                    super::sanitize_response_for_enabled(enabled, trigger_response);

                    vec![visual::header_row(cx, &spec, action_label, open_now, state)]
                });

                if spec.has_children() {
                    header = radix_collapsible::apply_collapsible_trigger_controls_expanded(
                        header, content_id, open_now,
                    );
                }
                if let Some(test_id) = spec.header_test_id.as_ref() {
                    header = header.test_id(test_id.clone());
                }
                header
            });
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
