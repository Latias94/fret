//! Immediate-mode text input and textarea helpers.

use std::sync::{Arc, Mutex};
use std::time::Duration;

use fret_core::{Color, Corners, Edges, KeyCode, Modifiers, NodeId, Px};
use fret_runtime::{CommandId, Effect, TimerToken};
use fret_ui::UiHost;
use fret_ui::element::{LayoutStyle, Length, SizeStyle};

use super::{
    InputTextMode, InputTextOptions, ResponseExt, TextAreaOptions, TextAreaSubmitKey,
    UiWriterImUiFacadeExt,
};

fn text_model_changed_for<H: UiHost>(
    cx: &mut fret_ui::ElementContext<'_, H>,
    id: fret_ui::GlobalElementId,
    current: &str,
) -> bool {
    super::model_value_changed_for(cx, id, current.to_string())
}

#[derive(Debug, Default, Clone, Copy)]
pub(super) struct InputTextAssistiveSemantics {
    pub active_descendant: Option<NodeId>,
    pub active_descendant_element: Option<u64>,
    pub controls_element: Option<u64>,
    pub expanded: Option<bool>,
}

#[derive(Debug, Default)]
struct ImuiTextFocusSelectionState {
    was_focused: bool,
    pending_select_all: bool,
    timer: Option<TimerToken>,
}

fn sync_select_all_on_focus<H: UiHost>(
    cx: &mut fret_ui::ElementContext<'_, H>,
    id: fret_ui::GlobalElementId,
    is_focused: bool,
    has_text: bool,
    select_all_on_focus: bool,
) {
    if !select_all_on_focus {
        return;
    }

    let state = cx.state_for(
        id,
        || Arc::new(Mutex::new(ImuiTextFocusSelectionState::default())),
        |state| state.clone(),
    );

    let (cancel_token, arm_token) = {
        let mut state = state.lock().unwrap_or_else(|e| e.into_inner());
        let mut cancel_token = None;
        let mut arm_token = None;

        if is_focused && !state.was_focused {
            state.pending_select_all = has_text;
            if state.pending_select_all {
                let token = cx.app.next_timer_token();
                state.timer = Some(token);
                arm_token = Some(token);
            }
        } else if !is_focused {
            cancel_token = state.timer.take();
            state.pending_select_all = false;
        }

        state.was_focused = is_focused;
        (cancel_token, arm_token)
    };

    if let Some(token) = cancel_token {
        cx.cancel_timer(token);
    }
    let install_handler = arm_token.is_some();
    if let Some(token) = arm_token {
        cx.set_timer_for(id, token, Duration::ZERO);
    }

    if install_handler {
        let state_for_timer = state.clone();
        cx.timer_on_timer_for(
            id,
            Arc::new(move |host, action_cx, token| {
                let mut state = state_for_timer.lock().unwrap_or_else(|e| e.into_inner());
                if state.timer != Some(token) {
                    return false;
                }
                state.timer = None;
                if !state.pending_select_all {
                    return false;
                }
                state.pending_select_all = false;
                host.record_transient_event(action_cx, super::KEY_SELECT_ALL_ON_FOCUS);
                host.request_redraw(action_cx.window);
                true
            }),
        );
    }
}

fn imui_text_input_style_from_theme(theme: &fret_ui::Theme) -> fret_ui::TextInputStyle {
    let background = theme
        .color_by_key("card")
        .or_else(|| theme.color_by_key("muted"))
        .or_else(|| theme.color_by_key("background"))
        .unwrap_or_else(|| theme.color_token("background"));
    let foreground = theme
        .color_by_key("foreground")
        .unwrap_or_else(|| theme.color_token("foreground"));
    let muted_foreground = theme
        .color_by_key("muted-foreground")
        .unwrap_or_else(|| theme.color_token("muted-foreground"));
    let border_idle = theme
        .color_by_key("input")
        .or_else(|| theme.color_by_key("border"))
        .unwrap_or_else(|| theme.color_token("input"));
    let ring = theme
        .color_by_key("ring")
        .unwrap_or_else(|| theme.color_token("ring"));
    let primary = theme
        .color_by_key("primary")
        .unwrap_or_else(|| theme.color_token("primary"));
    let selection = theme
        .color_by_key("component.input.selection")
        .unwrap_or_else(|| theme.color_token("selection.background"));
    let selection_color = Color {
        a: 1.0,
        ..selection
    };
    let mut preedit_bg_color = selection_color;
    preedit_bg_color.a = (preedit_bg_color.a * 0.35).clamp(0.0, 1.0);

    fret_ui::TextInputStyle {
        padding: Edges {
            left: Px(8.0),
            right: Px(8.0),
            top: Px(3.0),
            bottom: Px(3.0),
        },
        background,
        border: Edges::all(Px(1.0)),
        border_color: border_idle,
        border_color_focused: ring,
        focus_ring: None,
        corner_radii: Corners::all(super::control_chrome::CONTROL_RADIUS),
        text_color: foreground,
        placeholder_color: muted_foreground,
        selection_color,
        caret_color: foreground,
        preedit_bg_color,
        preedit_color: primary,
        preedit_underline_color: primary,
    }
}

fn imui_text_area_style_from_theme(theme: &fret_ui::Theme) -> fret_ui::TextAreaStyle {
    let input_style = imui_text_input_style_from_theme(theme);

    fret_ui::TextAreaStyle {
        padding_x: input_style.padding.left,
        padding_y: input_style.padding.top,
        background: input_style.background,
        border: input_style.border,
        border_color: input_style.border_color,
        border_color_focused: input_style.border_color_focused,
        focus_ring: None,
        corner_radii: input_style.corner_radii,
        text_color: input_style.text_color,
        placeholder_color: input_style.placeholder_color,
        selection_color: input_style.selection_color,
        caret_color: input_style.caret_color,
        preedit_bg_color: input_style.preedit_bg_color,
        preedit_underline_color: input_style.preedit_underline_color,
    }
}

fn default_input_text_style_from_theme(theme: &fret_ui::Theme) -> fret_core::TextStyle {
    crate::typography::control_text_style_for_font_size(
        theme,
        fret_core::FontId::ui(),
        theme
            .metric_by_key("font.size")
            .unwrap_or_else(|| theme.metric_token("font.size")),
    )
}

fn input_text_layout() -> LayoutStyle {
    LayoutStyle {
        size: SizeStyle {
            width: Length::Fill,
            height: Length::Px(super::control_chrome::FIELD_MIN_HEIGHT),
            min_height: Some(Length::Px(super::control_chrome::FIELD_MIN_HEIGHT)),
            max_height: Some(Length::Px(super::control_chrome::FIELD_MIN_HEIGHT)),
            ..Default::default()
        },
        ..Default::default()
    }
}

fn install_input_text_policy_commands<H: UiHost>(
    cx: &mut fret_ui::ElementContext<'_, H>,
    id: fret_ui::GlobalElementId,
    options: &InputTextOptions,
) {
    let completion_command = options.completion_command.clone();
    let history_previous_command = options.history_previous_command.clone();
    let history_next_command = options.history_next_command.clone();
    let undo_command = options.undo_command.clone();
    let redo_command = options.redo_command.clone();
    let completion_command_repeat = options.completion_command_repeat;
    let history_command_repeat = options.history_command_repeat;
    let undo_redo_command_repeat = options.undo_redo_command_repeat;

    if completion_command.is_none()
        && history_previous_command.is_none()
        && history_next_command.is_none()
        && undo_command.is_none()
        && redo_command.is_none()
    {
        return;
    }

    cx.key_add_on_key_down_for(
        id,
        Arc::new(move |host, action_cx, down| {
            if down.ime_composing || down.modifiers.alt || down.modifiers.meta {
                return false;
            }

            let command = if down.modifiers.ctrl {
                match down.key {
                    KeyCode::KeyZ
                        if !down.modifiers.shift && (!down.repeat || undo_redo_command_repeat) =>
                    {
                        undo_command.clone()
                    }
                    KeyCode::KeyY
                        if !down.modifiers.shift && (!down.repeat || undo_redo_command_repeat) =>
                    {
                        redo_command.clone()
                    }
                    KeyCode::KeyZ
                        if down.modifiers.shift && (!down.repeat || undo_redo_command_repeat) =>
                    {
                        redo_command.clone()
                    }
                    _ => None,
                }
            } else if !down.modifiers.shift {
                match down.key {
                    KeyCode::Tab if !down.repeat || completion_command_repeat => {
                        completion_command.clone()
                    }
                    KeyCode::ArrowUp if !down.repeat || history_command_repeat => {
                        history_previous_command.clone()
                    }
                    KeyCode::ArrowDown if !down.repeat || history_command_repeat => {
                        history_next_command.clone()
                    }
                    _ => None,
                }
            } else {
                None
            };

            let Some(command) = command else {
                return false;
            };

            host.dispatch_command(Some(action_cx.window), command);
            true
        }),
    );
}

fn install_textarea_policy_commands<H: UiHost>(
    cx: &mut fret_ui::ElementContext<'_, H>,
    id: fret_ui::GlobalElementId,
    options: &TextAreaOptions,
) {
    let submit_command = options.submit_command.clone();
    let cancel_command = options.cancel_command.clone();
    let submit_key = options.submit_key;
    let command_repeat = options.submit_cancel_command_repeat;

    if submit_command.is_none() && cancel_command.is_none() {
        return;
    }

    cx.key_add_on_key_down_capture_for(
        id,
        Arc::new(move |host, action_cx, down| {
            if down.ime_composing || down.modifiers.alt || down.modifiers.meta {
                return false;
            }

            let command = match down.key {
                KeyCode::Enter | KeyCode::NumpadEnter => match submit_key {
                    TextAreaSubmitKey::CtrlEnter
                        if down.modifiers
                            == (Modifiers {
                                ctrl: true,
                                ..Default::default()
                            }) =>
                    {
                        submit_command.clone()
                    }
                    TextAreaSubmitKey::Enter if down.modifiers == Modifiers::default() => {
                        submit_command.clone()
                    }
                    _ => None,
                },
                KeyCode::Escape if down.modifiers == Modifiers::default() => cancel_command.clone(),
                _ => None,
            };

            let Some(command) = command else {
                return false;
            };

            if down.repeat && !command_repeat {
                return true;
            }

            host.dispatch_command(Some(action_cx.window), command);
            true
        }),
    );
}

pub(super) fn input_text_model_with_options<H: UiHost, W: UiWriterImUiFacadeExt<H> + ?Sized>(
    ui: &mut W,
    model: &fret_runtime::Model<String>,
    options: InputTextOptions,
) -> ResponseExt {
    let model = model.clone();
    let mut response = ResponseExt::default();
    let element = ui
        .with_cx_mut(|cx| input_text_model_element_with_options(cx, model, options, &mut response));

    ui.add(element);
    response
}

pub(super) fn input_text_model_element_with_options<H: UiHost>(
    cx: &mut fret_ui::ElementContext<'_, H>,
    model: fret_runtime::Model<String>,
    options: InputTextOptions,
    response: &mut ResponseExt,
) -> fret_ui::element::AnyElement {
    input_text_model_element_with_options_and_semantics(
        cx,
        model,
        options,
        InputTextAssistiveSemantics::default(),
        response,
    )
}

pub(super) fn input_text_model_element_with_options_and_semantics<H: UiHost>(
    cx: &mut fret_ui::ElementContext<'_, H>,
    model: fret_runtime::Model<String>,
    options: InputTextOptions,
    assistive_semantics: InputTextAssistiveSemantics,
    response: &mut ResponseExt,
) -> fret_ui::element::AnyElement {
    let enabled = options.enabled && !super::imui_is_disabled(cx);
    cx.scope(|cx| {
        let id = cx.root_id();
        let current = cx
            .read_model(&model, fret_ui::Invalidation::Paint, |_app, v| v.clone())
            .unwrap_or_default();

        response.id = Some(id);
        response.enabled = enabled;
        response.core.focused = enabled && cx.is_focused_element(id);
        response.core.changed = enabled && text_model_changed_for(cx, id, &current);
        response.core.rect = cx.last_bounds_for_element(id);
        super::populate_response_lifecycle_from_active_state(
            cx,
            id,
            response.core.focused,
            response.core.changed,
            response,
        );
        sync_select_all_on_focus(
            cx,
            id,
            response.core.focused,
            !current.is_empty(),
            options.select_all_on_focus,
        );
        let select_all_requested = cx.take_transient_for(id, super::KEY_SELECT_ALL_ON_FOCUS);
        if select_all_requested && options.select_all_on_focus && response.core.focused {
            cx.app.push_effect(Effect::Command {
                window: Some(cx.window),
                command: CommandId::from("edit.select_all"),
            });
        }

        let mut props = fret_ui::element::TextInputProps::new(model.clone());
        props.enabled = enabled;
        props.focusable = enabled && options.focusable;
        props.read_only = options.read_only;
        props.obscure_text = matches!(options.mode, InputTextMode::Password);
        props.layout = input_text_layout();
        props.a11y_label = options.a11y_label.clone();
        props.a11y_role = options.a11y_role;
        props.active_descendant = assistive_semantics.active_descendant;
        props.active_descendant_element = assistive_semantics.active_descendant_element;
        props.controls_element = assistive_semantics.controls_element;
        props.expanded = assistive_semantics.expanded;
        props.test_id = options.test_id.clone();
        props.placeholder = options.placeholder.clone();
        props.submit_command = options.submit_command.clone();
        props.cancel_command = options.cancel_command.clone();
        if !options.filters.is_empty() || options.custom_filter.is_some() {
            let filters = options.filters;
            let custom_filter = options.custom_filter.clone();
            props.insert_filter = Some(Arc::new(move |text| {
                let filtered = filters.filter_text(text);
                match custom_filter.as_ref() {
                    Some(filter) => filter.filter_text(&filtered),
                    None => filtered,
                }
            }));
        }
        let (chrome, text_style) = {
            let theme = fret_ui::Theme::global(&*cx.app);
            (
                imui_text_input_style_from_theme(theme),
                default_input_text_style_from_theme(theme),
            )
        };
        props.chrome = chrome;
        props.text_style = text_style;

        let mut element = cx.text_input(props);
        element.id = id;
        install_input_text_policy_commands(cx, id, &options);
        element
    })
}

pub(super) fn textarea_model_with_options<H: UiHost, W: UiWriterImUiFacadeExt<H> + ?Sized>(
    ui: &mut W,
    model: &fret_runtime::Model<String>,
    options: TextAreaOptions,
) -> ResponseExt {
    let model = model.clone();
    let mut response = ResponseExt::default();

    let element = ui.with_cx_mut(|cx| {
        let enabled = options.enabled && !super::imui_is_disabled(cx);
        cx.scope(|cx| {
            let id = cx.root_id();
            let current = cx
                .read_model(&model, fret_ui::Invalidation::Paint, |_app, v| v.clone())
                .unwrap_or_default();

            response.id = Some(id);
            response.enabled = enabled;
            response.core.focused = enabled && cx.is_focused_element(id);
            response.core.changed = enabled && text_model_changed_for(cx, id, &current);
            response.core.rect = cx.last_bounds_for_element(id);
            super::populate_response_lifecycle_from_active_state(
                cx,
                id,
                response.core.focused,
                response.core.changed,
                &mut response,
            );
            sync_select_all_on_focus(
                cx,
                id,
                response.core.focused,
                !current.is_empty(),
                options.select_all_on_focus,
            );
            let select_all_requested = cx.take_transient_for(id, super::KEY_SELECT_ALL_ON_FOCUS);
            if select_all_requested && options.select_all_on_focus && response.core.focused {
                cx.app.push_effect(Effect::Command {
                    window: Some(cx.window),
                    command: CommandId::from("edit.select_all"),
                });
            }

            let mut props = fret_ui::element::TextAreaProps::new(model.clone());
            props.enabled = enabled;
            props.focusable = enabled && options.focusable;
            props.read_only = options.read_only;
            props.allow_tab_input = options.allow_tab_input;
            props.layout.size.width = Length::Fill;
            props.a11y_label = options.a11y_label.clone();
            props.test_id = options.test_id.clone();
            props.min_height = options.min_height;
            let (chrome, text_style) = {
                let theme = fret_ui::Theme::global(&*cx.app);
                let chrome = imui_text_area_style_from_theme(theme);
                let text_style = if options.stable_line_boxes {
                    crate::typography::text_area_control_text_style(theme)
                } else {
                    crate::typography::text_area_content_text_style(theme)
                };
                (chrome, text_style)
            };
            props.chrome = chrome;
            props.text_style = text_style;

            let mut element = cx.text_area(props);
            element.id = id;
            install_textarea_policy_commands(cx, id, &options);
            element
        })
    });

    ui.add(element);
    response
}

#[cfg(test)]
mod tests {
    use super::*;

    use std::sync::Arc;

    use fret_app::App;
    use fret_authoring::UiWriter;
    use fret_core::{AppWindowId, Rect};
    use fret_ui::ElementContext;
    use fret_ui::element::{AnyElement, ElementKind};

    struct TestWriter<'cx, 'a, H: UiHost> {
        cx: &'cx mut ElementContext<'a, H>,
        out: &'cx mut Vec<AnyElement>,
    }

    impl<'cx, 'a, H: UiHost> UiWriter<H> for TestWriter<'cx, 'a, H> {
        fn with_cx_mut<R>(&mut self, f: impl FnOnce(&mut ElementContext<'_, H>) -> R) -> R {
            f(self.cx)
        }

        fn add(&mut self, element: AnyElement) {
            self.out.push(element);
        }
    }

    fn first_text_input(root: &AnyElement) -> Option<&fret_ui::element::TextInputProps> {
        match &root.kind {
            ElementKind::TextInput(props) => Some(props),
            _ => root.children.iter().find_map(first_text_input),
        }
    }

    fn first_text_area(root: &AnyElement) -> Option<&fret_ui::element::TextAreaProps> {
        match &root.kind {
            ElementKind::TextArea(props) => Some(props),
            _ => root.children.iter().find_map(first_text_area),
        }
    }

    #[test]
    fn input_text_model_uses_compact_imui_chrome_without_focus_ring() {
        let mut app = App::new();
        let model = app.models_mut().insert(String::new());

        fret_ui::elements::with_element_cx(
            &mut app,
            AppWindowId::default(),
            Rect::default(),
            "imui-input-text-chrome",
            |cx| {
                let mut out = Vec::new();
                let mut ui = TestWriter { cx, out: &mut out };

                let response = input_text_model_with_options(
                    &mut ui,
                    &model,
                    InputTextOptions {
                        test_id: Some(Arc::from("imui-input-text-chrome")),
                        ..Default::default()
                    },
                );

                assert!(response.id.is_some());
                assert_eq!(out.len(), 1);

                let props = first_text_input(&out[0]).expect("expected text input element");
                assert!(props.chrome.focus_ring.is_none());
                assert_eq!(props.chrome.border, Edges::all(Px(1.0)));
                assert_eq!(props.chrome.padding.left, Px(8.0));
                assert_eq!(props.chrome.padding.right, Px(8.0));
                assert_eq!(props.chrome.padding.top, Px(3.0));
                assert_eq!(props.chrome.padding.bottom, Px(3.0));
                assert_eq!(
                    props.chrome.corner_radii,
                    Corners::all(super::super::control_chrome::CONTROL_RADIUS)
                );
                assert_eq!(
                    props.layout.size.height,
                    Length::Px(super::super::control_chrome::FIELD_MIN_HEIGHT)
                );
                assert_eq!(
                    props.layout.size.min_height,
                    Some(Length::Px(super::super::control_chrome::FIELD_MIN_HEIGHT))
                );
                assert_eq!(
                    props.layout.size.max_height,
                    Some(Length::Px(super::super::control_chrome::FIELD_MIN_HEIGHT))
                );
            },
        );
    }

    #[test]
    fn textarea_model_uses_compact_imui_chrome_without_focus_ring() {
        let mut app = App::new();
        let model = app.models_mut().insert(String::new());

        fret_ui::elements::with_element_cx(
            &mut app,
            AppWindowId::default(),
            Rect::default(),
            "imui-textarea-chrome",
            |cx| {
                let mut out = Vec::new();
                let mut ui = TestWriter { cx, out: &mut out };

                let response = textarea_model_with_options(
                    &mut ui,
                    &model,
                    TextAreaOptions {
                        test_id: Some(Arc::from("imui-textarea-chrome")),
                        ..Default::default()
                    },
                );

                assert!(response.id.is_some());
                assert_eq!(out.len(), 1);

                let props = first_text_area(&out[0]).expect("expected text area element");
                assert!(props.chrome.focus_ring.is_none());
                assert_eq!(props.chrome.border, Edges::all(Px(1.0)));
                assert_eq!(props.chrome.padding_x, Px(8.0));
                assert_eq!(props.chrome.padding_y, Px(3.0));
                assert_eq!(
                    props.chrome.corner_radii,
                    Corners::all(super::super::control_chrome::CONTROL_RADIUS)
                );
                assert_eq!(props.layout.size.width, Length::Fill);
            },
        );
    }
}
