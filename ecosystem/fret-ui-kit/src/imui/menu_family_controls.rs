//! Immediate-mode menu-bar helpers.

use std::cell::Cell;
use std::rc::Rc;
use std::sync::Arc;

use fret_core::{Corners, Edges, Px, SemanticsRole, TextOverflow, TextWrap};
use fret_ui::element::{
    AnyElement, ContainerProps, Length, PressableA11y, PressableProps, TextProps,
};
use fret_ui::{ElementContext, GlobalElementId, Theme, UiHost};

use super::{
    BeginMenuOptions, BeginSubmenuOptions, ImUiFacade, MenuBarOptions, MenuItemOptions,
    ResponseExt, UiWriterImUiFacadeExt,
};

pub(super) fn menu_bar_element<H: UiHost>(
    cx: &mut ElementContext<'_, H>,
    build_focus: Option<Rc<Cell<Option<GlobalElementId>>>>,
    options: MenuBarOptions,
    f: impl for<'cx2, 'a2> FnOnce(&mut ImUiFacade<'cx2, 'a2, H>),
) -> AnyElement {
    let mut builder = crate::ui::h_flex_build(move |cx, out| {
        super::containers::build_imui_children_with_focus(cx, out, build_focus, f);
    });
    builder = builder
        .gap_metric(options.gap)
        .justify(crate::Justify::Start)
        .items(crate::Items::Center)
        .no_wrap()
        .role(SemanticsRole::MenuBar);
    if let Some(test_id) = options.test_id {
        builder = builder.test_id(test_id);
    }
    builder.into_element(cx)
}

pub(super) fn begin_menu_with_options<H: UiHost, W: UiWriterImUiFacadeExt<H> + ?Sized>(
    ui: &mut W,
    id: &str,
    label: Arc<str>,
    options: BeginMenuOptions,
    f: impl for<'cx2, 'a2> FnOnce(&mut ImUiFacade<'cx2, 'a2, H>),
) -> bool {
    let enabled = options.enabled && ui.with_cx_mut(|cx| !super::imui_is_disabled(cx));
    let popup_open = ui.popup_open_model(id);
    let open_before = ui.with_cx_mut(|cx| {
        cx.read_model(&popup_open, fret_ui::Invalidation::Paint, |_app, value| {
            *value
        })
        .unwrap_or(false)
    });

    let trigger = ui.push_id(format!("{id}.trigger"), |ui| {
        menu_trigger_with_options(
            ui,
            label.clone(),
            open_before,
            enabled,
            options.test_id.clone(),
            options.activate_shortcut,
            options.shortcut_repeat,
        )
    });

    if enabled && trigger.clicked() {
        if open_before {
            ui.close_popup(id);
        } else if let Some(anchor) = trigger.core.rect {
            ui.open_popup_at(id, anchor);
        }
    }

    let submenu_root_name = format!("fret-ui-kit.imui.popup.inline.{id}");
    let built_popup = super::popup_overlay::build_popup_menu(
        ui,
        id,
        submenu_root_name.as_str(),
        options.popup,
        f,
    );
    let popup_opened = built_popup.is_some();
    if let Some(mut built) = built_popup {
        let mut attached_to_parent = false;
        ui.with_cx_mut(|cx| {
            if let Some(state) = cx.provided::<super::popup_overlay::ImUiMenuOpenDescendantState>()
            {
                state.children.borrow_mut().extend(built.children.drain(..));
                attached_to_parent = true;
            }
        });
        if !attached_to_parent {
            for child in built.children {
                ui.add(child);
            }
        }
    }
    if !enabled && popup_opened {
        ui.close_popup(id);
    }

    ui.with_cx_mut(|cx| {
        cx.read_model(&popup_open, fret_ui::Invalidation::Paint, |_app, value| {
            *value
        })
        .unwrap_or(false)
    })
}

pub(super) fn begin_submenu_with_options<H: UiHost, W: UiWriterImUiFacadeExt<H> + ?Sized>(
    ui: &mut W,
    id: &str,
    label: Arc<str>,
    options: BeginSubmenuOptions,
    f: impl for<'cx2, 'a2> FnOnce(&mut ImUiFacade<'cx2, 'a2, H>),
) -> bool {
    let enabled = options.enabled && ui.with_cx_mut(|cx| !super::imui_is_disabled(cx));
    let popup_open = ui.popup_open_model(id);
    let open_before = ui.with_cx_mut(|cx| {
        cx.read_model(&popup_open, fret_ui::Invalidation::Paint, |_app, value| {
            *value
        })
        .unwrap_or(false)
    });

    let trigger = ui.push_id(format!("{id}.trigger"), |ui| {
        super::menu_controls::menu_item_with_options(
            ui,
            label.clone(),
            MenuItemOptions {
                enabled,
                test_id: options.test_id.clone(),
                submenu: true,
                expanded: Some(open_before),
                activate_shortcut: options.activate_shortcut,
                shortcut_repeat: options.shortcut_repeat,
                ..Default::default()
            },
        )
    });

    if enabled && trigger.clicked() {
        if open_before {
            ui.close_popup(id);
        } else if let Some(anchor) = trigger.core.rect {
            ui.open_popup_at(id, anchor);
        }
    }

    let popup_opened = ui.begin_popup_menu_with_options(id, trigger.id, options.popup, f);
    if !enabled && popup_opened {
        ui.close_popup(id);
    }

    ui.with_cx_mut(|cx| {
        cx.read_model(&popup_open, fret_ui::Invalidation::Paint, |_app, value| {
            *value
        })
        .unwrap_or(false)
    })
}

fn menu_trigger_with_options<H: UiHost, W: UiWriterImUiFacadeExt<H> + ?Sized>(
    ui: &mut W,
    label: Arc<str>,
    open: bool,
    enabled: bool,
    test_id: Option<Arc<str>>,
    activate_shortcut: Option<fret_runtime::KeyChord>,
    shortcut_repeat: bool,
) -> ResponseExt {
    let mut response = ResponseExt::default();

    let element = ui.with_cx_mut(|cx| {
        let response = &mut response;

        let mut props = PressableProps::default();
        props.enabled = enabled;
        props.focusable = enabled;
        props.a11y = PressableA11y {
            role: Some(SemanticsRole::MenuItem),
            label: Some(label.clone()),
            test_id,
            expanded: Some(open),
            ..Default::default()
        };

        cx.pressable_with_id(props, move |cx, state, id| {
            cx.pressable_clear_on_pointer_down();
            cx.pressable_clear_on_pointer_up();
            cx.key_clear_on_key_down_for(id);

            cx.pressable_on_activate(crate::on_activate(move |host, acx, _reason| {
                host.record_transient_event(acx, super::KEY_CLICKED);
                host.notify(acx);
            }));

            if enabled {
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
                                host.record_transient_event(acx, super::KEY_CLICKED);
                                host.notify(acx);
                                return true;
                            }
                        }

                        false
                    }),
                );
            }

            response.core.hovered = state.hovered;
            response.core.pressed = state.pressed;
            response.core.focused = state.focused;
            response.nav_highlighted =
                state.focused && fret_ui::focus_visible::is_focus_visible(cx.app, Some(cx.window));
            response.id = Some(id);
            response.core.clicked = cx.take_transient_for(id, super::KEY_CLICKED);
            response.core.rect = cx.last_bounds_for_element(id);
            super::sanitize_response_for_enabled(enabled, response);

            vec![menu_trigger_visual(cx, label.clone(), open, enabled, state)]
        })
    });

    ui.add(element);
    response
}

fn menu_trigger_visual<H: UiHost>(
    cx: &mut ElementContext<'_, H>,
    label: Arc<str>,
    open: bool,
    enabled: bool,
    state: fret_ui::element::PressableState,
) -> AnyElement {
    let theme = Theme::global(&*cx.app);
    let active = open || state.hovered || state.focused || state.pressed;
    let background = if active {
        Some(
            theme
                .color_by_key("accent")
                .unwrap_or_else(|| theme.color_token("accent")),
        )
    } else {
        None
    };
    let foreground = if !enabled {
        theme
            .color_by_key("muted-foreground")
            .unwrap_or_else(|| theme.color_token("muted-foreground"))
    } else if active {
        theme
            .color_by_key("accent-foreground")
            .unwrap_or_else(|| theme.color_token("accent-foreground"))
    } else {
        theme
            .color_by_key("foreground")
            .unwrap_or_else(|| theme.color_token("foreground"))
    };

    let mut chrome = ContainerProps::default();
    chrome.layout.size.width = Length::Auto;
    chrome.layout.size.height = Length::Auto;
    chrome.padding = Edges {
        left: Px(8.0),
        right: Px(8.0),
        top: Px(4.0),
        bottom: Px(4.0),
    }
    .into();
    chrome.background = background;
    chrome.corner_radii = Corners::all(Px(6.0));

    cx.container(chrome, move |cx| {
        let mut text = TextProps::new(label.clone());
        text.wrap = TextWrap::None;
        text.overflow = TextOverflow::Clip;
        text.color = Some(foreground);
        vec![cx.text_props(text)]
    })
}
