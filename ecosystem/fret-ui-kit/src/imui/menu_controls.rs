//! Immediate-mode menu-item helpers.

use std::sync::Arc;

use fret_core::{Edges, KeyCode, Modifiers, Px, SemanticsRole};
use fret_runtime::ActionId;
use fret_ui::UiHost;
use fret_ui::action::UiActionHostExt as _;
use fret_ui::action::{ActivateReason, PressablePointerDownResult, PressablePointerUpResult};
use fret_ui::element::{
    AnyElement, ContainerProps, InsetStyle, LayoutStyle, Length, PositionStyle, PressableA11y,
    PressableProps, PressableState, RowProps, SemanticsDecoration, SpacerProps, SpacingLength,
    TextProps,
};
use fret_ui::elements::GlobalElementId;

use super::{MenuItemOptions, ResponseExt, UiWriterImUiFacadeExt};
use crate::command::ElementCommandGatingExt as _;

pub(super) fn menu_item_with_options<H: UiHost, W: UiWriterImUiFacadeExt<H> + ?Sized>(
    ui: &mut W,
    label: Arc<str>,
    options: MenuItemOptions,
) -> ResponseExt {
    menu_item_with_options_and_pressable_hook(
        ui,
        label,
        options,
        noop_menu_item_pressable_hook::<H>,
    )
}

pub(super) fn menu_item_checkbox_with_options<H: UiHost, W: UiWriterImUiFacadeExt<H> + ?Sized>(
    ui: &mut W,
    label: Arc<str>,
    checked: bool,
    options: MenuItemOptions,
) -> ResponseExt {
    menu_item_impl(
        ui,
        label,
        options,
        SemanticsRole::MenuItemCheckbox,
        Some(checked),
        None,
    )
}

pub(super) fn menu_item_radio_with_options<H: UiHost, W: UiWriterImUiFacadeExt<H> + ?Sized>(
    ui: &mut W,
    label: Arc<str>,
    checked: bool,
    options: MenuItemOptions,
) -> ResponseExt {
    menu_item_impl(
        ui,
        label,
        options,
        SemanticsRole::MenuItemRadio,
        Some(checked),
        None,
    )
}

pub(super) fn menu_item_action_with_options<H: UiHost, W: UiWriterImUiFacadeExt<H> + ?Sized>(
    ui: &mut W,
    label: Arc<str>,
    action: ActionId,
    options: MenuItemOptions,
) -> ResponseExt {
    menu_item_impl(
        ui,
        label,
        options,
        SemanticsRole::MenuItem,
        None,
        Some(action),
    )
}

fn menu_item_impl<H: UiHost, W: UiWriterImUiFacadeExt<H> + ?Sized>(
    ui: &mut W,
    label: Arc<str>,
    options: MenuItemOptions,
    role: SemanticsRole,
    checked: Option<bool>,
    action: Option<ActionId>,
) -> ResponseExt {
    menu_item_impl_with_pressable_hook(
        ui,
        label,
        options,
        role,
        checked,
        action,
        noop_menu_item_pressable_hook::<H>,
    )
}

pub(super) fn menu_item_with_options_and_pressable_hook<
    H: UiHost,
    W: UiWriterImUiFacadeExt<H> + ?Sized,
    F,
>(
    ui: &mut W,
    label: Arc<str>,
    options: MenuItemOptions,
    pressable_hook: F,
) -> ResponseExt
where
    F: Clone
        + for<'cx> Fn(&mut fret_ui::ElementContext<'cx, H>, PressableState, GlobalElementId, bool),
{
    menu_item_impl_with_pressable_hook(
        ui,
        label,
        options,
        SemanticsRole::MenuItem,
        None,
        None,
        pressable_hook,
    )
}

fn noop_menu_item_pressable_hook<H: UiHost>(
    _cx: &mut fret_ui::ElementContext<'_, H>,
    _state: PressableState,
    _item_id: GlobalElementId,
    _enabled: bool,
) {
}

fn menu_item_impl_with_pressable_hook<H: UiHost, W: UiWriterImUiFacadeExt<H> + ?Sized, F>(
    ui: &mut W,
    label: Arc<str>,
    options: MenuItemOptions,
    role: SemanticsRole,
    checked: Option<bool>,
    action: Option<ActionId>,
    pressable_hook: F,
) -> ResponseExt
where
    F: Clone
        + for<'cx> Fn(&mut fret_ui::ElementContext<'cx, H>, PressableState, GlobalElementId, bool),
{
    let mut response = ResponseExt::default();

    let element = ui.with_cx_mut(|cx| {
        let response = &mut response;
        let pressable_hook = pressable_hook.clone();
        let mut panel = ContainerProps::default();
        panel.layout.size.width = Length::Fill;
        panel.layout.size.height = Length::Auto;
        panel.padding = Edges {
            left: Px(6.0),
            right: Px(6.0),
            top: Px(2.0),
            bottom: Px(2.0),
        }
        .into();

        let close_popup = options.close_popup.clone();
        let test_id = options.test_id.clone();
        let shortcut = options.shortcut.clone();
        let shortcut_test_id = options.shortcut_test_id.clone().or_else(|| {
            test_id
                .as_ref()
                .map(|test_id| Arc::from(format!("{test_id}.shortcut")))
        });
        let submenu = options.submenu;
        let expanded = options.expanded;
        let activate_shortcut = options.activate_shortcut;
        let shortcut_repeat = options.shortcut_repeat;
        let mut enabled = options.enabled && !super::imui_is_disabled(cx);
        if let Some(action) = action.as_ref() {
            enabled = enabled && cx.action_is_enabled(action);
        }
        let label_for_visuals = label.clone();

        let mut stack = fret_ui::element::StackProps::default();
        stack.layout.size.width = Length::Fill;
        stack.layout.size.height = Length::Auto;

        cx.stack_props(stack, move |cx| {
            let visuals = cx.container(panel, move |cx| {
                let mut row = RowProps::default();
                row.layout.size.width = Length::Fill;
                row.layout.size.height = Length::Auto;
                row.gap = SpacingLength::Px(Px(6.0));

                let indicator = match (role, checked) {
                    (SemanticsRole::MenuItemCheckbox, Some(true)) => Some(Arc::from("\u{2713}")),
                    (SemanticsRole::MenuItemCheckbox, Some(false)) => Some(Arc::from(" ")),
                    (SemanticsRole::MenuItemRadio, Some(true)) => Some(Arc::from("\u{25CF}")),
                    (SemanticsRole::MenuItemRadio, Some(false)) => Some(Arc::from(" ")),
                    _ => None,
                };

                vec![cx.row(row, move |cx| {
                    let mut out: Vec<AnyElement> = Vec::new();
                    if let Some(indicator) = indicator.clone() {
                        out.push(cx.text(indicator));
                    }
                    let mut label_props = TextProps::new(label_for_visuals.clone());
                    label_props.layout.size.width = Length::Fill;
                    label_props.layout.flex.shrink = 1.0;
                    label_props.wrap = fret_core::TextWrap::None;
                    label_props.overflow = fret_core::TextOverflow::Ellipsis;
                    out.push(cx.text_props(label_props));

                    if let Some(shortcut) = shortcut.clone() {
                        out.push(cx.spacer(SpacerProps::default()));

                        let mut shortcut_props = TextProps::new(shortcut);
                        shortcut_props.wrap = fret_core::TextWrap::None;
                        shortcut_props.overflow = fret_core::TextOverflow::Clip;
                        let mut shortcut = cx.text_props(shortcut_props);
                        if let Some(test_id) = shortcut_test_id.clone() {
                            shortcut = shortcut
                                .attach_semantics(SemanticsDecoration::default().test_id(test_id));
                        }
                        out.push(shortcut);
                    } else if submenu {
                        out.push(cx.spacer(SpacerProps::default()));
                        out.push(cx.text(Arc::from("\u{203A}")));
                    }
                    out
                })]
            });

            let mut props = PressableProps::default();
            props.enabled = enabled;
            props.focusable = enabled;
            props.layout = LayoutStyle {
                position: PositionStyle::Absolute,
                inset: InsetStyle {
                    left: Some(Px(0.0)).into(),
                    right: Some(Px(0.0)).into(),
                    top: Some(Px(0.0)).into(),
                    bottom: Some(Px(0.0)).into(),
                },
                size: fret_ui::element::SizeStyle {
                    width: Length::Fill,
                    height: Length::Fill,
                    ..Default::default()
                },
                ..Default::default()
            };
            props.a11y = PressableA11y {
                role: Some(role),
                label: Some(label.clone()),
                test_id: test_id.clone(),
                checked,
                expanded,
                ..Default::default()
            };

            let pressable = cx.pressable_with_id(props, move |cx, state, id| {
                let pressable_hook = pressable_hook.clone();
                cx.pressable_clear_on_pointer_down();
                cx.pressable_clear_on_pointer_up();
                cx.key_clear_on_key_down_for(id);

                let active_item_model = super::active_item_model_for_window(cx);
                let active_item_model_for_down = active_item_model.clone();
                let active_item_model_for_up = active_item_model.clone();
                let lifecycle_model = super::lifecycle_session_model_for(cx, id);
                let lifecycle_model_for_activate = lifecycle_model.clone();
                let lifecycle_model_for_down = lifecycle_model.clone();
                let lifecycle_model_for_up = lifecycle_model.clone();

                cx.pressable_on_pointer_down(Arc::new(move |host, acx, down| {
                    super::mark_lifecycle_activated_on_left_pointer_down(
                        host,
                        acx,
                        down.button,
                        &lifecycle_model_for_down,
                    );
                    super::mark_active_item_on_left_pointer_down(
                        host,
                        acx,
                        down.button,
                        &active_item_model_for_down,
                        true,
                    );
                    PressablePointerDownResult::Continue
                }));

                cx.pressable_on_pointer_up(Arc::new(move |host, acx, up| {
                    super::mark_lifecycle_deactivated_on_left_pointer_up(
                        host,
                        acx,
                        up.button,
                        &lifecycle_model_for_up,
                    );
                    super::clear_active_item_on_left_pointer_up(
                        host,
                        acx,
                        up.button,
                        &active_item_model_for_up,
                    );
                    PressablePointerUpResult::Continue
                }));

                if enabled {
                    let close_popup_for_activate = close_popup.clone();
                    let action_for_activate = action.clone();
                    cx.pressable_on_activate(crate::on_activate(move |host, acx, reason| {
                        if reason == ActivateReason::Keyboard {
                            super::mark_lifecycle_instant_if_inactive(
                                host,
                                acx,
                                &lifecycle_model_for_activate,
                                false,
                            );
                        }
                        if let Some(open) = close_popup_for_activate.as_ref() {
                            let _ = host.update_model(open, |v| *v = false);
                        }
                        host.record_transient_event(acx, super::KEY_CLICKED);
                        if let Some(action) = action_for_activate.clone() {
                            host.record_pending_command_dispatch_source(acx, &action, reason);
                            host.dispatch_command(Some(acx.window), action);
                        }
                        host.notify(acx);
                    }));

                    let nav_items = cx
                        .inherited_state::<super::popup_overlay::ImUiMenuNavState>()
                        .map(|st| st.items.clone());
                    if let Some(nav_items) = nav_items.as_ref() {
                        nav_items.borrow_mut().push(id);
                    }
                    if let Some(nav_items) = nav_items {
                        let item_id = id;
                        let close_popup_for_key = close_popup.clone();
                        let action_for_shortcut = action.clone();
                        let lifecycle_model_for_shortcut = lifecycle_model.clone();
                        cx.key_on_key_down_for(
                            id,
                            Arc::new(move |host, acx, down| {
                                if let Some(shortcut) = activate_shortcut {
                                    let matches_shortcut =
                                        down.key == shortcut.key && down.modifiers == shortcut.mods;
                                    if matches_shortcut
                                        && (!down.repeat || shortcut_repeat)
                                        && !down.ime_composing
                                    {
                                        super::mark_lifecycle_instant_if_inactive(
                                            host,
                                            acx,
                                            &lifecycle_model_for_shortcut,
                                            false,
                                        );
                                        if let Some(open) = close_popup_for_key.as_ref() {
                                            let _ = host.update_model(open, |v| *v = false);
                                        }
                                        host.record_transient_event(acx, super::KEY_CLICKED);
                                        if let Some(action) = action_for_shortcut.clone() {
                                            host.record_pending_command_dispatch_source(
                                                acx,
                                                &action,
                                                ActivateReason::Keyboard,
                                            );
                                            host.dispatch_command(Some(acx.window), action);
                                        }
                                        host.notify(acx);
                                        return true;
                                    }
                                }

                                if down.repeat {
                                    return false;
                                }
                                if down.modifiers != Modifiers::default() {
                                    return false;
                                }

                                let (dir, jump_to) = match down.key {
                                    KeyCode::ArrowDown => (1isize, None),
                                    KeyCode::ArrowUp => (-1isize, None),
                                    KeyCode::Home => (0isize, Some(0usize)),
                                    KeyCode::End => (0isize, Some(usize::MAX)),
                                    _ => return false,
                                };

                                let items = nav_items.borrow();
                                if items.is_empty() {
                                    return false;
                                }
                                let len = items.len();
                                let idx = items.iter().position(|id| *id == item_id);
                                let next_idx = if let Some(jump) = jump_to {
                                    if jump == usize::MAX {
                                        len - 1
                                    } else {
                                        jump.min(len - 1)
                                    }
                                } else {
                                    let current =
                                        idx.unwrap_or_else(|| if dir < 0 { len - 1 } else { 0 });
                                    ((current as isize + dir + len as isize) % len as isize)
                                        as usize
                                };

                                host.request_focus(items[next_idx]);
                                host.notify(acx);
                                true
                            }),
                        );
                    }
                }

                pressable_hook(cx, state, id, enabled);

                response.core.hovered = state.hovered;
                response.core.pressed = state.pressed;
                response.core.focused = state.focused;
                response.nav_highlighted = state.focused
                    && fret_ui::focus_visible::is_focus_visible(cx.app, Some(cx.window));
                response.id = Some(id);
                response.core.clicked = cx.take_transient_for(id, super::KEY_CLICKED);
                response.core.rect = cx.last_bounds_for_element(id);
                let hover_delay =
                    super::install_hover_query_hooks_for_pressable(cx, id, state.hovered_raw, None);
                response.pointer_hovered_raw = state.hovered_raw;
                response.pointer_hovered_raw_below_barrier = state.hovered_raw_below_barrier;
                response.hover_stationary_met = hover_delay.stationary_met;
                response.hover_delay_short_met = hover_delay.delay_short_met;
                response.hover_delay_normal_met = hover_delay.delay_normal_met;
                response.hover_delay_short_shared_met = hover_delay.shared_delay_short_met;
                response.hover_delay_normal_shared_met = hover_delay.shared_delay_normal_met;
                response.hover_blocked_by_active_item =
                    super::hover_blocked_by_active_item_for(cx, id, &active_item_model);
                super::populate_response_lifecycle_transients(cx, id, response);
                super::populate_response_lifecycle_from_active_state(
                    cx,
                    id,
                    state.pressed,
                    false,
                    response,
                );
                super::sanitize_response_for_enabled(enabled, response);

                Vec::<AnyElement>::new()
            });

            vec![visuals, pressable]
        })
    });

    ui.add(element);
    response
}
