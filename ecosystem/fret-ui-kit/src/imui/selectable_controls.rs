//! Immediate-mode selectable row helpers.

use std::sync::Arc;

use fret_core::{Color, Corners, Edges, KeyCode, Modifiers, MouseButton, Px, SemanticsRole};
use fret_ui::UiHost;
use fret_ui::action::{
    ActivateReason, PressablePointerDownResult, PressablePointerUpResult, UiActionHostExt as _,
};
use fret_ui::element::{ContainerProps, Length, PressableA11y, PressableProps, TextProps};
use fret_ui::{ElementContext, Theme};

use super::{ResponseExt, SelectableOptions, UiWriterImUiFacadeExt};

#[derive(Debug, Clone, Copy, PartialEq)]
struct SelectablePalette {
    bg: Option<Color>,
    fg: Color,
}

pub(super) fn selectable_with_options<H: UiHost, W: UiWriterImUiFacadeExt<H> + ?Sized>(
    ui: &mut W,
    label: Arc<str>,
    options: SelectableOptions,
) -> ResponseExt {
    let mut response = ResponseExt::default();

    let element = ui.with_cx_mut(|cx| {
        let response = &mut response;
        let enabled = options.enabled && !super::imui_is_disabled(cx);
        let focusable = enabled && options.focusable;
        let selected = options.selected;
        let close_popup = options.close_popup.clone();
        let activate_shortcut = options.activate_shortcut;
        let shortcut_repeat = options.shortcut_repeat;

        let mut props = PressableProps::default();
        props.enabled = enabled;
        props.focusable = focusable;
        props.layout.size.width = Length::Fill;
        props.layout.size.height = Length::Auto;
        props.a11y = PressableA11y {
            role: options.a11y_role.or(Some(SemanticsRole::ListBoxOption)),
            label: options.a11y_label.clone().or_else(|| Some(label.clone())),
            test_id: options.test_id.clone(),
            selected,
            ..Default::default()
        };

        cx.pressable_with_id(props, move |cx, state, id| {
            cx.pressable_clear_on_pointer_down();
            cx.pressable_clear_on_pointer_move();
            cx.pressable_clear_on_pointer_up();
            cx.key_clear_on_key_down_for(id);

            let active_item_model = super::active_item_model_for_window(cx);
            let active_item_model_for_down = active_item_model.clone();
            let active_item_model_for_move = active_item_model.clone();
            let active_item_model_for_up = active_item_model.clone();

            let context_anchor_model = super::context_menu_anchor_model_for(cx, id);
            let context_anchor_model_for_report = context_anchor_model.clone();
            let long_press_signal_model = super::long_press_signal_model_for(cx, id);
            let long_press_signal_model_for_down = long_press_signal_model.clone();
            let long_press_signal_model_for_move = long_press_signal_model.clone();
            let long_press_signal_model_for_up = long_press_signal_model.clone();
            let lifecycle_model = super::lifecycle_session_model_for(cx, id);
            let lifecycle_model_for_activate = lifecycle_model.clone();
            let lifecycle_model_for_down = lifecycle_model.clone();
            let lifecycle_model_for_up = lifecycle_model.clone();
            let pointer_click_modifiers_model = super::pointer_click_modifiers_model_for(cx, id);
            let pointer_click_modifiers_model_for_up = pointer_click_modifiers_model.clone();
            let pointer_click_modifiers_model_for_report = pointer_click_modifiers_model.clone();

            if enabled {
                let close_popup_for_activate = close_popup.clone();
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
                    host.notify(acx);
                }));

                let nav_items = if focusable {
                    let nav_items = cx
                        .inherited_state::<super::popup_overlay::ImUiMenuNavState>()
                        .map(|st| st.items.clone());
                    if let Some(nav_items) = nav_items.as_ref() {
                        nav_items.borrow_mut().push(id);
                    }
                    nav_items
                } else {
                    None
                };
                let item_id = id;
                let close_popup_for_key = close_popup.clone();
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
                                host.notify(acx);
                                return true;
                            }
                        }

                        let is_menu_key = down.key == KeyCode::ContextMenu;
                        let is_shift_f10 = down.key == KeyCode::F10 && down.modifiers.shift;
                        if is_menu_key || is_shift_f10 {
                            host.record_transient_event(acx, super::KEY_CONTEXT_MENU_REQUESTED);
                            host.notify(acx);
                            return true;
                        }

                        let Some(nav_items) = nav_items.as_ref() else {
                            return false;
                        };
                        if down.repeat || down.modifiers != Modifiers::default() {
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
                            let current = idx.unwrap_or_else(|| if dir < 0 { len - 1 } else { 0 });
                            ((current as isize + dir + len as isize) % len as isize) as usize
                        };

                        host.request_focus(items[next_idx]);
                        host.notify(acx);
                        true
                    }),
                );
            }

            cx.pressable_on_pointer_down(Arc::new(move |host, acx, down| {
                super::mark_lifecycle_activated_on_left_pointer_down(
                    host,
                    acx,
                    down.button,
                    &lifecycle_model_for_down,
                );
                super::prepare_pressable_drag_on_pointer_down(
                    host,
                    acx,
                    down,
                    &active_item_model_for_down,
                    &long_press_signal_model_for_down,
                    super::drag_kind_for_element(acx.target),
                );

                PressablePointerDownResult::Continue
            }));

            let drag_threshold = super::drag_threshold_for(cx);
            cx.pressable_on_pointer_move(Arc::new(move |host, acx, mv| {
                super::handle_pressable_drag_move_with_threshold(
                    host,
                    acx,
                    mv,
                    &active_item_model_for_move,
                    &long_press_signal_model_for_move,
                    super::drag_kind_for_element(acx.target),
                    drag_threshold,
                )
            }));

            cx.pressable_on_pointer_up(Arc::new(move |host, acx, up| {
                super::mark_lifecycle_deactivated_on_left_pointer_up(
                    host,
                    acx,
                    up.button,
                    &lifecycle_model_for_up,
                );
                super::finish_pressable_drag_on_pointer_up(
                    host,
                    acx,
                    up,
                    &active_item_model_for_up,
                    &long_press_signal_model_for_up,
                    super::drag_kind_for_element(acx.target),
                );

                if up.is_click && up.button == MouseButton::Right {
                    let _ = host.update_model(&context_anchor_model, |v| *v = Some(up.position));
                    host.record_transient_event(acx, super::KEY_SECONDARY_CLICKED);
                    host.record_transient_event(acx, super::KEY_CONTEXT_MENU_REQUESTED);
                    host.notify(acx);
                    return PressablePointerUpResult::SkipActivate;
                }

                if up.is_click && up.button == MouseButton::Left {
                    let _ = host.update_model(&pointer_click_modifiers_model_for_up, |value| {
                        *value = up.modifiers;
                    });
                    host.record_transient_event(acx, super::KEY_POINTER_CLICKED);
                }

                if up.is_click && up.button == MouseButton::Left && up.click_count == 2 {
                    host.record_transient_event(acx, super::KEY_DOUBLE_CLICKED);
                    host.notify(acx);
                }

                PressablePointerUpResult::Continue
            }));

            response.core.hovered = state.hovered;
            response.core.pressed = state.pressed;
            response.core.focused = state.focused;
            response.nav_highlighted =
                state.focused && fret_ui::focus_visible::is_focus_visible(cx.app, Some(cx.window));
            response.id = Some(id);
            response.core.clicked = cx.take_transient_for(id, super::KEY_CLICKED);
            response.secondary_clicked = cx.take_transient_for(id, super::KEY_SECONDARY_CLICKED);
            response.double_clicked = cx.take_transient_for(id, super::KEY_DOUBLE_CLICKED);
            response.long_pressed = cx.take_transient_for(id, super::KEY_LONG_PRESSED);
            response.press_holding = cx
                .read_model(
                    &long_press_signal_model,
                    fret_ui::Invalidation::Paint,
                    |_app, value| value.holding,
                )
                .unwrap_or(false);
            response.context_menu_requested =
                cx.take_transient_for(id, super::KEY_CONTEXT_MENU_REQUESTED);
            response.context_menu_anchor = cx
                .read_model(
                    &context_anchor_model_for_report,
                    fret_ui::Invalidation::Paint,
                    |_app, v| *v,
                )
                .unwrap_or(None);
            response.pointer_clicked = cx.take_transient_for(id, super::KEY_POINTER_CLICKED);
            if response.pointer_clicked {
                response.pointer_click_modifiers = cx
                    .read_model(
                        &pointer_click_modifiers_model_for_report,
                        fret_ui::Invalidation::Paint,
                        |_app, modifiers| *modifiers,
                    )
                    .unwrap_or_default();
            }
            super::populate_pressable_drag_response(cx, id, response);
            response.core.rect = cx.last_bounds_for_element(id);
            let hover_delay = super::install_hover_query_hooks_for_pressable(
                cx,
                id,
                state.hovered_raw,
                Some(long_press_signal_model.clone()),
            );
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

            vec![selectable_row_element(
                cx,
                label.clone(),
                enabled,
                selected,
                state,
            )]
        })
    });

    ui.add(element);
    response
}

fn selectable_row_element<H: UiHost>(
    cx: &mut ElementContext<'_, H>,
    label: Arc<str>,
    enabled: bool,
    selected: bool,
    state: fret_ui::element::PressableState,
) -> fret_ui::element::AnyElement {
    let palette = resolve_selectable_palette(
        Theme::global(&*cx.app),
        enabled,
        selected,
        state.hovered || state.focused,
        state.pressed,
    );

    let mut row = ContainerProps::default();
    row.layout.size.width = Length::Fill;
    row.layout.size.height = Length::Auto;
    row.padding = Edges {
        left: Px(8.0),
        right: Px(8.0),
        top: Px(4.0),
        bottom: Px(4.0),
    }
    .into();
    row.background = palette.bg;
    row.corner_radii = Corners::all(Px(6.0));

    cx.container(row, move |cx| {
        let mut text = TextProps::new(label.clone());
        text.layout.size.width = Length::Fill;
        text.layout.size.height = Length::Auto;
        text.wrap = fret_core::TextWrap::None;
        text.overflow = fret_core::TextOverflow::Ellipsis;
        text.color = Some(palette.fg);
        vec![cx.text_props(text)]
    })
}

fn resolve_selectable_palette(
    theme: &Theme,
    enabled: bool,
    selected: bool,
    hovered: bool,
    pressed: bool,
) -> SelectablePalette {
    let selected_bg = theme
        .color_by_key("list.active.background")
        .or_else(|| theme.color_by_key("list.row.selected"))
        .or_else(|| theme.color_by_key("selection.background"))
        .unwrap_or_else(|| theme.color_token("selection.background"));
    let hover_bg = theme
        .color_by_key("list.hover.background")
        .or_else(|| theme.color_by_key("list.row.hover"))
        .or_else(|| theme.color_by_key("accent"))
        .unwrap_or_else(|| theme.color_token("accent"));
    let fg = if !enabled {
        theme
            .color_by_key("muted-foreground")
            .unwrap_or_else(|| theme.color_token("muted-foreground"))
    } else if !selected && (pressed || hovered) {
        theme
            .color_by_key("accent-foreground")
            .unwrap_or_else(|| theme.color_token("accent-foreground"))
    } else {
        theme
            .color_by_key("foreground")
            .unwrap_or_else(|| theme.color_token("foreground"))
    };

    let bg = if selected {
        Some(selected_bg)
    } else if pressed || hovered {
        Some(hover_bg)
    } else {
        None
    };

    SelectablePalette { bg, fg }
}

#[cfg(test)]
mod tests {
    use fret_app::App;
    use fret_core::Color;
    use fret_ui::{Theme, ThemeConfig};

    use super::resolve_selectable_palette;

    #[test]
    fn selectable_palette_prefers_selected_background_hover_foreground_and_disabled_muted() {
        let mut app = App::new();
        Theme::with_global_mut(&mut app, |theme| {
            let mut cfg = ThemeConfig::default();
            cfg.colors
                .insert("list.active.background".to_string(), "#224466".to_string());
            cfg.colors
                .insert("accent".to_string(), "#335577".to_string());
            cfg.colors
                .insert("foreground".to_string(), "#f5f6f7".to_string());
            cfg.colors
                .insert("accent-foreground".to_string(), "#fefefe".to_string());
            cfg.colors
                .insert("muted-foreground".to_string(), "#8899aa".to_string());
            theme.apply_config_patch(&cfg);
        });

        let theme = Theme::global(&app);
        let selected = resolve_selectable_palette(theme, true, true, false, false);
        assert_eq!(selected.bg, Some(Color::from_srgb_hex_rgb(0x22_44_66)));
        assert_eq!(selected.fg, Color::from_srgb_hex_rgb(0xf5_f6_f7));

        let hovered = resolve_selectable_palette(theme, true, false, true, false);
        assert_eq!(hovered.bg, Some(Color::from_srgb_hex_rgb(0x33_55_77)));
        assert_eq!(hovered.fg, Color::from_srgb_hex_rgb(0xfe_fe_fe));

        let disabled = resolve_selectable_palette(theme, false, false, false, false);
        assert_eq!(disabled.bg, None);
        assert_eq!(disabled.fg, Color::from_srgb_hex_rgb(0x88_99_aa));
    }
}
