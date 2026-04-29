//! Immediate-mode button-style pressable helpers.

use std::{any::Any, sync::Arc};

use fret_core::{Corners, Edges, KeyCode, Px, SemanticsRole, Size};
use fret_runtime::ActionId;
use fret_ui::UiHost;
use fret_ui::action::ActivateReason;
use fret_ui::element::{AnyElement, ContainerProps, Length, PressableA11y, PressableProps};

use super::label_identity::parse_label_identity;
use super::{
    ButtonArrowDirection, ButtonOptions, ButtonVariant, ResponseExt, UiWriterImUiFacadeExt,
};
use crate::command::ElementCommandGatingExt as _;
use crate::declarative::chrome::control_chrome_pressable_with_id_props;

pub(super) fn button_with_options<H: UiHost, W: UiWriterImUiFacadeExt<H> + ?Sized>(
    ui: &mut W,
    label: Arc<str>,
    options: ButtonOptions,
) -> ResponseExt {
    button_impl(ui, label, options, None)
}

pub(super) fn small_button_with_options<H: UiHost, W: UiWriterImUiFacadeExt<H> + ?Sized>(
    ui: &mut W,
    label: Arc<str>,
    mut options: ButtonOptions,
) -> ResponseExt {
    options.variant = ButtonVariant::Small;
    button_impl(ui, label, options, None)
}

pub(super) fn arrow_button_with_options<H: UiHost, W: UiWriterImUiFacadeExt<H> + ?Sized>(
    ui: &mut W,
    id: &str,
    direction: ButtonArrowDirection,
    mut options: ButtonOptions,
) -> ResponseExt {
    options.variant = ButtonVariant::Arrow(direction);
    ui.push_id(id, |ui| button_impl(ui, Arc::from(""), options, None))
}

pub(super) fn invisible_button_with_options<H: UiHost, W: UiWriterImUiFacadeExt<H> + ?Sized>(
    ui: &mut W,
    id: &str,
    size: Size,
    mut options: ButtonOptions,
) -> ResponseExt {
    options.variant = ButtonVariant::Invisible { size };
    ui.push_id(id, |ui| button_impl(ui, Arc::from(""), options, None))
}

pub(super) fn action_button_with_options<H: UiHost, W: UiWriterImUiFacadeExt<H> + ?Sized>(
    ui: &mut W,
    label: Arc<str>,
    action: ActionId,
    options: ButtonOptions,
) -> ResponseExt {
    button_impl(
        ui,
        label,
        options,
        Some(ButtonAction {
            action,
            payload: None,
        }),
    )
}

pub(super) fn action_payload_button_with_options<
    H: UiHost,
    W: UiWriterImUiFacadeExt<H> + ?Sized,
    T,
>(
    ui: &mut W,
    label: Arc<str>,
    action: ActionId,
    payload: T,
    options: ButtonOptions,
) -> ResponseExt
where
    T: Any + Clone + Send + Sync + 'static,
{
    let payload = Arc::new(move || Box::new(payload.clone()) as Box<dyn Any + Send + Sync>);
    button_impl(
        ui,
        label,
        options,
        Some(ButtonAction {
            action,
            payload: Some(payload),
        }),
    )
}

#[derive(Clone)]
struct ButtonAction {
    action: ActionId,
    payload: Option<Arc<dyn Fn() -> Box<dyn Any + Send + Sync> + 'static>>,
}

fn arrow_symbol(direction: ButtonArrowDirection) -> Arc<str> {
    Arc::from(match direction {
        ButtonArrowDirection::Left => "<",
        ButtonArrowDirection::Right => ">",
        ButtonArrowDirection::Up => "^",
        ButtonArrowDirection::Down => "v",
    })
}

fn arrow_a11y_label(direction: ButtonArrowDirection) -> Arc<str> {
    Arc::from(match direction {
        ButtonArrowDirection::Left => "Left arrow button",
        ButtonArrowDirection::Right => "Right arrow button",
        ButtonArrowDirection::Up => "Up arrow button",
        ButtonArrowDirection::Down => "Down arrow button",
    })
}

fn button_a11y_label(
    label: &Arc<str>,
    options: &ButtonOptions,
    variant: ButtonVariant,
) -> Option<Arc<str>> {
    options.a11y_label.clone().or_else(|| match variant {
        ButtonVariant::Arrow(direction) => Some(arrow_a11y_label(direction)),
        ButtonVariant::Invisible { .. } if label.is_empty() => None,
        _ => Some(label.clone()),
    })
}

fn apply_button_variant_layout(props: &mut PressableProps, variant: ButtonVariant) {
    match variant {
        ButtonVariant::Default => {
            props.layout.size.min_height =
                Some(Length::Px(super::control_chrome::BUTTON_MIN_HEIGHT));
        }
        ButtonVariant::Small => {
            props.layout.size.min_height =
                Some(Length::Px(super::control_chrome::SMALL_BUTTON_MIN_HEIGHT));
        }
        ButtonVariant::Arrow(_) => {
            props.layout.size.width = Length::Px(super::control_chrome::ARROW_BUTTON_SIZE);
            props.layout.size.height = Length::Px(super::control_chrome::ARROW_BUTTON_SIZE);
        }
        ButtonVariant::Invisible { size } => {
            props.layout.size.width = Length::Px(size.width);
            props.layout.size.height = Length::Px(size.height);
        }
    }
}

fn button_label_children<H: UiHost>(
    cx: &mut fret_ui::ElementContext<'_, H>,
    label: Arc<str>,
    color: fret_core::Color,
) -> Vec<AnyElement> {
    vec![
        cx.flex(super::control_chrome::centered_row_props(), move |cx| {
            vec![super::control_chrome::control_text(
                cx,
                label.clone(),
                color,
            )]
        }),
    ]
}

fn button_impl<H: UiHost, W: UiWriterImUiFacadeExt<H> + ?Sized>(
    ui: &mut W,
    label: Arc<str>,
    options: ButtonOptions,
    action: Option<ButtonAction>,
) -> ResponseExt {
    let parts = parse_label_identity(label.as_ref());
    let identity = Arc::<str>::from(parts.identity);
    let visible_label = Arc::<str>::from(parts.visible);
    ui.push_id(("button-label", identity), |ui| {
        button_impl_inner(ui, visible_label, options, action)
    })
}

fn button_impl_inner<H: UiHost, W: UiWriterImUiFacadeExt<H> + ?Sized>(
    ui: &mut W,
    label: Arc<str>,
    options: ButtonOptions,
    action: Option<ButtonAction>,
) -> ResponseExt {
    let mut response = ResponseExt::default();

    let element = ui.with_cx_mut(|cx| {
        let response = &mut response;
        let mut enabled = options.enabled && !super::imui_is_disabled(cx);
        if let Some(action) = action.as_ref() {
            enabled = enabled && cx.action_is_enabled(&action.action);
        }
        let variant = options.variant;
        let mut props = PressableProps::default();
        props.enabled = enabled;
        props.focusable = enabled && options.focusable;
        apply_button_variant_layout(&mut props, variant);
        props.a11y = PressableA11y {
            role: Some(SemanticsRole::Button),
            label: button_a11y_label(&label, &options, variant),
            test_id: options.test_id.clone(),
            ..Default::default()
        };
        let activate_shortcut = options.activate_shortcut;
        let shortcut_repeat = options.shortcut_repeat;

        control_chrome_pressable_with_id_props(cx, move |cx, state, id| {
            let behavior = super::item_behavior::install_pressable_item_behavior(cx, id);
            let lifecycle_model_for_activate = behavior.lifecycle_model.clone();

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
                host.record_transient_event(acx, super::KEY_CLICKED);
                if let Some(action) = action_for_activate.clone() {
                    host.record_pending_command_dispatch_source(acx, &action.action, reason);
                    if let Some(payload) = action.payload.as_ref() {
                        host.record_pending_action_payload(acx, &action.action, payload());
                    }
                    host.dispatch_command(Some(acx.window), action.action);
                }
                host.notify(acx);
            }));

            if enabled {
                let lifecycle_model_for_shortcut = behavior.lifecycle_model.clone();
                let action_for_shortcut = action.clone();
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
                                host.record_transient_event(acx, super::KEY_CLICKED);
                                if let Some(action) = action_for_shortcut.clone() {
                                    host.record_pending_command_dispatch_source(
                                        acx,
                                        &action.action,
                                        ActivateReason::Keyboard,
                                    );
                                    if let Some(payload) = action.payload.as_ref() {
                                        host.record_pending_action_payload(
                                            acx,
                                            &action.action,
                                            payload(),
                                        );
                                    }
                                    host.dispatch_command(Some(acx.window), action.action);
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

            let (palette, chrome, visual_label, invisible) = match variant {
                ButtonVariant::Default => {
                    let (palette, chrome) =
                        super::control_chrome::button_chrome(cx, enabled, state);
                    (Some(palette), chrome, label.clone(), false)
                }
                ButtonVariant::Small => {
                    let (palette, mut chrome) =
                        super::control_chrome::button_chrome(cx, enabled, state);
                    chrome.padding = Edges {
                        left: Px(8.0),
                        right: Px(8.0),
                        top: Px(2.0),
                        bottom: Px(2.0),
                    }
                    .into();
                    chrome.corner_radii = Corners::all(super::control_chrome::CONTROL_RADIUS);
                    (Some(palette), chrome, label.clone(), false)
                }
                ButtonVariant::Arrow(direction) => {
                    let (palette, mut chrome) =
                        super::control_chrome::button_chrome(cx, enabled, state);
                    chrome.padding = Edges::all(Px(0.0)).into();
                    (Some(palette), chrome, arrow_symbol(direction), false)
                }
                ButtonVariant::Invisible { .. } => {
                    (None, ContainerProps::default(), Arc::from(""), true)
                }
            };

            (props, chrome, move |cx| {
                if invisible {
                    return Vec::<AnyElement>::new();
                }
                let palette = palette.expect("visible buttons should carry a palette");
                button_label_children(cx, visual_label.clone(), palette.foreground)
            })
        })
    });

    ui.add(element);
    response
}
