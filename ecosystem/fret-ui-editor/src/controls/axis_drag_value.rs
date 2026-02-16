//! Axis-labeled drag value (joined input group).
//!
//! This is used by Vec/Transform-style inspectors where the axis marker ("X/Y/Z/W") should feel
//! like part of the numeric field instead of a separate, differently-styled widget.

use std::panic::Location;
use std::sync::{Arc, Mutex};

use fret_core::text::{TextOverflow, TextWrap};
use fret_core::{Color, Edges, KeyCode, Px, TextAlign, TextStyle};
use fret_runtime::Model;
use fret_ui::action::{
    ActionCx, PointerDownCx, PressablePointerDownResult, UiActionHost, UiFocusActionHost,
};
use fret_ui::element::{
    AnyElement, FlexItemStyle, InsetStyle, LayoutStyle, Length, Overflow, PositionStyle, SizeStyle,
    TextInputProps, TextProps,
};
use fret_ui::{ElementContext, Invalidation, Theme, UiHost};
use fret_ui_kit::{ChromeRefinement, Size};

use crate::controls::numeric_input::{NumericFormatFn, NumericParseFn, NumericValidateFn};
use crate::primitives::chrome::resolve_editor_text_field_style;
use crate::primitives::drag_value_core::DragValueScalar;
use crate::primitives::input_group::{
    EditorInputGroupFrameOverrides, editor_axis_segment, editor_input_group_divider,
    editor_input_group_frame, editor_input_group_frame_with_overrides, editor_input_group_inset,
    editor_input_group_row,
};
use crate::primitives::style::EditorStyle;
use crate::primitives::visuals::EditorFrameState;
use crate::primitives::{DragValueCore, DragValueCoreOptions};

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
enum AxisDragValueMode {
    Scrub,
    Typing,
}

#[derive(Debug)]
struct AxisDragValueState {
    mode: AxisDragValueMode,
    scrub_id: Option<fret_ui::GlobalElementId>,
    input_id: Option<fret_ui::GlobalElementId>,
    seen_input_focus: bool,
}

impl Default for AxisDragValueState {
    fn default() -> Self {
        Self {
            mode: AxisDragValueMode::Scrub,
            scrub_id: None,
            input_id: None,
            seen_input_focus: false,
        }
    }
}

#[derive(Debug, Clone)]
pub struct AxisDragValueOptions {
    pub layout: LayoutStyle,
    /// Explicit identity source for internal state (scrub/typing focus restore, draft string).
    ///
    /// This is the editor-control equivalent of egui's `id_source(...)` / ImGui's `PushID`.
    pub id_source: Option<Arc<str>>,
    pub test_id: Option<Arc<str>>,
    pub enabled: bool,
    pub focusable: bool,
    pub size: Size,
}

impl Default for AxisDragValueOptions {
    fn default() -> Self {
        Self {
            layout: LayoutStyle {
                size: SizeStyle {
                    width: Length::Fill,
                    height: Length::Auto,
                    ..Default::default()
                },
                flex: FlexItemStyle {
                    grow: 1.0,
                    basis: Length::Px(Px(0.0)),
                    ..Default::default()
                },
                ..Default::default()
            },
            id_source: None,
            test_id: None,
            enabled: true,
            focusable: true,
            size: Size::Small,
        }
    }
}

#[derive(Clone)]
pub struct AxisDragValue<T> {
    axis_label: Arc<str>,
    axis_tint: Color,
    model: Model<T>,
    format: NumericFormatFn<T>,
    parse: NumericParseFn<T>,
    validate: Option<NumericValidateFn<T>>,
    options: AxisDragValueOptions,
}

impl<T> AxisDragValue<T>
where
    T: DragValueScalar + Default,
{
    pub fn new(
        axis_label: Arc<str>,
        axis_tint: Color,
        model: Model<T>,
        format: NumericFormatFn<T>,
        parse: NumericParseFn<T>,
    ) -> Self {
        Self {
            axis_label,
            axis_tint,
            model,
            format,
            parse,
            validate: None,
            options: AxisDragValueOptions::default(),
        }
    }

    pub fn validate(mut self, validate: Option<NumericValidateFn<T>>) -> Self {
        self.validate = validate;
        self
    }

    pub fn options(mut self, options: AxisDragValueOptions) -> Self {
        self.options = options;
        self
    }

    #[track_caller]
    pub fn into_element<H: UiHost>(self, cx: &mut ElementContext<'_, H>) -> AnyElement {
        let model_id = self.model.id();
        let loc = Location::caller();
        let callsite = (loc.file(), loc.line(), loc.column());
        let id_source = self.options.id_source.clone();

        if let Some(id_source) = id_source.as_deref() {
            cx.keyed(
                ("fret-ui-editor.axis_drag_value", id_source, model_id),
                |cx| self.into_element_keyed(cx),
            )
        } else {
            cx.keyed(
                ("fret-ui-editor.axis_drag_value", callsite, model_id),
                |cx| self.into_element_keyed(cx),
            )
        }
    }

    fn into_element_keyed<H: UiHost>(self, cx: &mut ElementContext<'_, H>) -> AnyElement {
        let state: Arc<Mutex<AxisDragValueState>> = cx.with_state(
            || Arc::new(Mutex::new(AxisDragValueState::default())),
            |s| s.clone(),
        );

        let draft = draft_model(cx);
        let error = error_model(cx);

        let value = cx
            .get_model_copied(&self.model, Invalidation::Paint)
            .unwrap_or_default();
        let value_text = (self.format)(value);
        let value_text_for_scrub = value_text.clone();

        let mode = state.lock().unwrap_or_else(|e| e.into_inner()).mode;
        let typing = mode == AxisDragValueMode::Typing;

        let (density, frame_chrome, (text_style, input_chrome)) = {
            let theme = Theme::global(&*cx.app);
            let style = EditorStyle::resolve(theme);
            let frame_chrome = style.frame_chrome_small();
            let (input_chrome, text_style) = resolve_editor_text_field_style(
                theme,
                self.options.size,
                &ChromeRefinement::default(),
            );

            (style.density, frame_chrome, (text_style, input_chrome))
        };

        let mut scrub_opts = DragValueCoreOptions::default();
        scrub_opts.layout = if typing {
            hidden_layout(self.options.layout)
        } else {
            self.options.layout
        };
        scrub_opts.enabled = self.options.enabled && mode == AxisDragValueMode::Scrub;
        scrub_opts.scrub_on_double_click = false;

        let model_for_change = self.model.clone();
        let on_change_live: Arc<dyn Fn(&mut dyn UiActionHost, ActionCx, T) + 'static> =
            Arc::new(move |host, action_cx, next| {
                let _ = host.models_mut().update(&model_for_change, |v| *v = next);
                host.request_redraw(action_cx.window);
            });

        let axis_label = self.axis_label.clone();
        let axis_tint = self.axis_tint;
        let state_for_scrub_record = state.clone();
        let scrub = DragValueCore::new(value, on_change_live)
            .a11y_label(value_text.clone())
            .options(scrub_opts)
            .into_element(cx, move |cx, resp| {
                let scrub_id = cx.root_id();
                let mut st = state_for_scrub_record
                    .lock()
                    .unwrap_or_else(|e| e.into_inner());
                st.scrub_id = Some(scrub_id);

                let state_for_double_click = state_for_scrub_record.clone();
                cx.pressable_add_on_pointer_down(Arc::new(
                    move |host, action_cx, down: PointerDownCx| {
                        if down.click_count < 2 {
                            return PressablePointerDownResult::Continue;
                        }
                        let mut st = state_for_double_click
                            .lock()
                            .unwrap_or_else(|e| e.into_inner());
                        st.mode = AxisDragValueMode::Typing;
                        st.seen_input_focus = false;
                        if let Some(input_id) = st.input_id {
                            host.request_focus(input_id);
                        }
                        host.request_redraw(action_cx.window);
                        PressablePointerDownResult::SkipDefaultAndStopPropagation
                    },
                ));

                let divider = frame_chrome.border;

                vec![editor_input_group_frame(
                    cx,
                    LayoutStyle {
                        size: SizeStyle {
                            width: Length::Fill,
                            height: Length::Fill,
                            min_height: Some(density.row_height),
                            ..Default::default()
                        },
                        ..Default::default()
                    },
                    density,
                    frame_chrome,
                    EditorFrameState {
                        enabled: true,
                        hovered: resp.hovered,
                        pressed: resp.dragging || resp.pressed,
                        focused: resp.focused || cx.is_focused_element(scrub_id),
                        open: false,
                    },
                    move |cx, visuals| {
                        let axis = editor_axis_segment(
                            cx,
                            density,
                            axis_label.clone(),
                            axis_tint,
                            visuals.bg,
                        );
                        let sep = editor_input_group_divider(cx, divider);
                        let value_text_el = cx.text_props(TextProps {
                            layout: LayoutStyle {
                                size: SizeStyle {
                                    width: Length::Fill,
                                    height: Length::Fill,
                                    ..Default::default()
                                },
                                ..Default::default()
                            },
                            text: value_text_for_scrub.clone(),
                            style: Some(TextStyle {
                                size: frame_chrome.text_px,
                                line_height: Some(density.row_height),
                                ..Default::default()
                            }),
                            color: Some(visuals.fg),
                            wrap: TextWrap::None,
                            overflow: TextOverflow::Ellipsis,
                            align: TextAlign::Start,
                        });
                        let value =
                            editor_input_group_inset(cx, frame_chrome.padding, value_text_el);

                        vec![editor_input_group_row(cx, Px(0.0), vec![axis, sep, value])]
                    },
                )]
            });

        let input_group_layout = if typing {
            self.options.layout
        } else {
            hidden_layout(self.options.layout)
        };

        let parse = self.parse.clone();
        let format = self.format.clone();
        let validate = self.validate.clone();
        let model_for_commit = self.model.clone();
        let state_for_input = state.clone();
        let error_for_keys = error.clone();
        let draft_for_keys = draft.clone();

        let mut props = TextInputProps::new(draft.clone());
        props.layout = LayoutStyle {
            size: SizeStyle {
                width: Length::Fill,
                height: Length::Auto,
                min_height: Some(density.row_height),
                ..Default::default()
            },
            ..Default::default()
        };
        props.enabled = self.options.enabled && typing;
        props.focusable = self.options.focusable && typing;
        props.test_id = self.options.test_id.clone();

        // Joined field: the frame is drawn by the input group. Keep the inner text input transparent
        // and borderless to avoid double chrome.
        let mut joined_chrome = input_chrome;
        joined_chrome.padding = Edges::all(Px(0.0));
        joined_chrome.border = Edges::all(Px(0.0));
        joined_chrome.corner_radii = fret_core::Corners::all(Px(0.0));
        joined_chrome.background = Color {
            a: 0.0,
            ..joined_chrome.background
        };
        joined_chrome.border_color = Color {
            a: 0.0,
            ..joined_chrome.border_color
        };
        joined_chrome.border_color_focused = joined_chrome.border_color;
        joined_chrome.focus_ring = None;

        props.chrome = joined_chrome;
        props.text_style = text_style.clone();

        let input = cx.text_input(props);
        let input_id = input.id;
        let is_focused = cx.is_focused_element(input_id);

        {
            let mut st = state.lock().unwrap_or_else(|e| e.into_inner());
            st.input_id = Some(input_id);
        }

        // Drive mode transitions from focus: if the user clicks away after the input actually
        // became focused, return to scrub mode.
        if typing {
            let mut st = state.lock().unwrap_or_else(|e| e.into_inner());
            if is_focused {
                st.seen_input_focus = true;
            } else if st.seen_input_focus {
                st.mode = AxisDragValueMode::Scrub;
            }
        }

        if !is_focused {
            let _ = cx
                .app
                .models_mut()
                .update(&draft, |s| *s = value_text.as_ref().to_string());
            let _ = cx.app.models_mut().update(&error, |e| *e = None);
        }

        cx.key_add_on_key_down_capture_for(
            input_id,
            Arc::new(
                move |host: &mut dyn UiFocusActionHost, action_cx: ActionCx, down| match down.key {
                    KeyCode::Enter | KeyCode::NumpadEnter => {
                        let text = host
                            .models_mut()
                            .read(&draft_for_keys, |s| s.clone())
                            .unwrap_or_default();
                        if let Some(v) = (parse)(&text) {
                            if let Some(validate) = validate.as_ref() {
                                if let Some(msg) = validate(v) {
                                    let _ = host
                                        .models_mut()
                                        .update(&error_for_keys, |e| *e = Some(msg));
                                    host.request_redraw(action_cx.window);
                                    return true;
                                }
                            }

                            let _ = host.models_mut().update(&model_for_commit, |m| *m = v);
                            let formatted = (format)(v);
                            let _ = host
                                .models_mut()
                                .update(&draft_for_keys, |s| *s = formatted.as_ref().to_string());
                            let _ = host.models_mut().update(&error_for_keys, |e| *e = None);

                            let mut st = state_for_input.lock().unwrap_or_else(|e| e.into_inner());
                            st.mode = AxisDragValueMode::Scrub;
                            if let Some(scrub_id) = st.scrub_id {
                                host.request_focus(scrub_id);
                            }
                            host.request_redraw(action_cx.window);
                            true
                        } else {
                            let _ = host.models_mut().update(&error_for_keys, |e| {
                                *e = Some(Arc::from("Invalid number"))
                            });
                            host.request_redraw(action_cx.window);
                            true
                        }
                    }
                    KeyCode::Escape => {
                        let current = host
                            .models_mut()
                            .get_copied(&model_for_commit)
                            .unwrap_or_default();
                        let formatted = (format)(current);
                        let _ = host
                            .models_mut()
                            .update(&draft_for_keys, |s| *s = formatted.as_ref().to_string());
                        let _ = host.models_mut().update(&error_for_keys, |e| *e = None);

                        let mut st = state_for_input.lock().unwrap_or_else(|e| e.into_inner());
                        st.mode = AxisDragValueMode::Scrub;
                        if let Some(scrub_id) = st.scrub_id {
                            host.request_focus(scrub_id);
                        }
                        host.request_redraw(action_cx.window);
                        true
                    }
                    _ => false,
                },
            ),
        );

        let has_error = cx
            .get_model_cloned(&error, Invalidation::Paint)
            .unwrap_or(None)
            .is_some();

        let typing_field = {
            let theme = Theme::global(&*cx.app);
            let divider = frame_chrome.border;
            let error_border = theme.color_token("destructive");

            editor_input_group_frame_with_overrides(
                cx,
                input_group_layout,
                density,
                frame_chrome,
                EditorFrameState {
                    enabled: true,
                    hovered: false,
                    pressed: false,
                    focused: is_focused,
                    open: false,
                },
                EditorInputGroupFrameOverrides {
                    bg: None,
                    border: has_error.then_some(error_border),
                },
                move |cx, visuals| {
                    let axis = editor_axis_segment(
                        cx,
                        density,
                        self.axis_label.clone(),
                        self.axis_tint,
                        visuals.bg,
                    );
                    let sep = editor_input_group_divider(cx, divider);

                    // Wrap the text input so the group padding applies, without adding its own padding.
                    let input_wrap = editor_input_group_inset(cx, frame_chrome.padding, input);

                    vec![editor_input_group_row(
                        cx,
                        Px(0.0),
                        vec![axis, sep, input_wrap],
                    )]
                },
            )
        };

        // Render both: scrub stays mounted so focus can restore, typing stays mounted so focus
        // requests have a stable target.
        cx.container(Default::default(), move |_cx| vec![scrub, typing_field])
    }
}

fn hidden_layout(mut layout: LayoutStyle) -> LayoutStyle {
    layout.size = SizeStyle {
        width: Length::Px(Px(0.0)),
        height: Length::Px(Px(0.0)),
        min_width: Some(Px(0.0)),
        min_height: Some(Px(0.0)),
        ..Default::default()
    };
    layout.position = PositionStyle::Absolute;
    layout.inset = InsetStyle {
        top: Some(Px(0.0)),
        left: Some(Px(0.0)),
        ..Default::default()
    };
    layout.overflow = Overflow::Clip;
    layout
}

fn draft_model<H: UiHost>(cx: &mut ElementContext<'_, H>) -> Model<String> {
    let draft = cx.with_state(|| None::<Model<String>>, |st| st.clone());
    match draft {
        Some(draft) => draft,
        None => {
            let draft = cx.app.models_mut().insert(String::new());
            cx.with_state(
                || None::<Model<String>>,
                |st| {
                    if st.is_none() {
                        *st = Some(draft.clone());
                    }
                },
            );
            draft
        }
    }
}

fn error_model<H: UiHost>(cx: &mut ElementContext<'_, H>) -> Model<Option<Arc<str>>> {
    let m = cx.with_state(|| None::<Model<Option<Arc<str>>>>, |st| st.clone());
    match m {
        Some(m) => m,
        None => {
            let m = cx.app.models_mut().insert(None::<Arc<str>>);
            cx.with_state(
                || None::<Model<Option<Arc<str>>>>,
                |st| {
                    if st.is_none() {
                        *st = Some(m.clone());
                    }
                },
            );
            m
        }
    }
}
