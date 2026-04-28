//! Immediate-mode selectable row helpers.

use std::sync::Arc;

use fret_core::{Color, Corners, Edges, KeyCode, Modifiers, Px, SemanticsRole};
use fret_ui::UiHost;
use fret_ui::action::{ActivateReason, UiActionHostExt as _};
use fret_ui::element::{ContainerProps, Length, PressableA11y, PressableProps, TextProps};
use fret_ui::{ElementContext, Theme};

use super::label_identity::parse_label_identity;
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
    let parts = parse_label_identity(label.as_ref());
    let identity = Arc::<str>::from(parts.identity);
    let visible_label = Arc::<str>::from(parts.visible);
    ui.push_id(("selectable-label", identity), |ui| {
        selectable_with_options_inner(ui, visible_label, options)
    })
}

fn selectable_with_options_inner<H: UiHost, W: UiWriterImUiFacadeExt<H> + ?Sized>(
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
            let behavior = super::item_behavior::install_pressable_item_behavior_with_options(
                cx,
                id,
                super::item_behavior::PressableItemBehaviorOptions {
                    report_pointer_click: true,
                },
            );
            let lifecycle_model_for_activate = behavior.lifecycle_model.clone();

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
                let lifecycle_model_for_shortcut = behavior.lifecycle_model.clone();
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

            let clicked = cx.take_transient_for(id, super::KEY_CLICKED);
            super::item_behavior::populate_pressable_item_response(
                cx,
                id,
                state,
                &behavior,
                super::item_behavior::PressableItemResponseInput {
                    enabled,
                    clicked,
                    changed: false,
                    lifecycle_edited: false,
                },
                response,
            );

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
        left: Px(6.0),
        right: Px(6.0),
        top: Px(2.0),
        bottom: Px(2.0),
    }
    .into();
    row.background = palette.bg;
    row.corner_radii = Corners::all(super::control_chrome::CONTROL_RADIUS);

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
