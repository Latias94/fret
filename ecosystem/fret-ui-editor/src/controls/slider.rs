//! Editor-grade horizontal slider control (v1).
//!
//! This is intentionally a small, policy-layer widget:
//! - pointer down sets the value (clamped / stepped),
//! - pointer drag updates the value continuously (best-effort cleanup when pointer-up is missed),
//! - visuals reuse the shared editor "frame" chrome policy to stay consistent with other controls.
//! - optional value display and a typing mode (double-click).

use std::panic::Location;
use std::sync::{Arc, Mutex};

use crate::controls::numeric_input::{
    NumericFormatFn, NumericInput, NumericInputErrorDisplay, NumericInputOptions,
    NumericInputOutcome, NumericParseFn, NumericValidateFn,
};
use crate::primitives::EditorTokenKeys;
use crate::primitives::drag_value_core::DragValueScalar;
use crate::primitives::input_group::{
    derived_test_id, editor_input_group_divider, editor_input_group_frame,
    editor_input_group_segment,
};
use crate::primitives::numeric_format::suppress_duplicate_chrome_affixes;
use crate::primitives::numeric_text_entry::{
    NumericTextEntryFocusHandoffState, arm_numeric_text_entry_focus_handoff,
    sync_numeric_text_entry_focus_handoff,
};
use crate::primitives::readout::EditorCompactReadoutStyle;
use crate::primitives::visuals::{EditorFrameSemanticState, EditorFrameState};
use crate::primitives::{NumericPresentation, style::EditorStyle};
use fret_core::text::TextOverflow;
use fret_core::{Axis, Corners, CursorIcon, Edges, MouseButton, Px, TextAlign};
use fret_runtime::Model;
use fret_ui::action::{PressablePointerDownResult, PressablePointerUpResult};
use fret_ui::element::{
    AnyElement, ContainerProps, CrossAlign, FlexItemStyle, FlexProps, LayoutStyle, Length,
    MainAlign, Overflow, PressableA11y, PressableProps, SizeStyle, SpacingLength,
};
use fret_ui::{ElementContext, Invalidation, Theme, UiHost};

mod chrome;
mod model;
mod pointer;
#[cfg(test)]
mod tests;
mod typing;
mod value_math;

use chrome::resolve_slider_paint;
pub use model::SliderOptions;
use model::{
    SliderMode, SliderState, compose_affixed_value_text, default_slider_format,
    default_slider_parse, hidden_layout,
};
use pointer::{
    begin_slider_drag, clear_slider_drag, enter_slider_typing, finish_slider_drag,
    is_slider_drag_pointer, reset_slider_interaction,
};
use typing::{slider_typing_parse, slider_typing_validate};
use value_math::{quantize_value, t_from_value, value_from_slider_local_x};

#[derive(Clone)]
pub struct Slider<T> {
    model: Model<T>,
    min: f64,
    max: f64,
    format: NumericFormatFn<T>,
    parse: NumericParseFn<T>,
    validate: Option<NumericValidateFn<T>>,
    options: SliderOptions,
}

impl<T> Slider<T>
where
    T: DragValueScalar + Default,
{
    pub fn new(model: Model<T>, min: f64, max: f64) -> Self {
        Self {
            model,
            min,
            max,
            format: default_slider_format(),
            parse: default_slider_parse(),
            validate: None,
            options: SliderOptions::default(),
        }
    }

    /// Construct a slider from a shared editor authoring bundle.
    pub fn from_presentation(
        model: Model<T>,
        min: f64,
        max: f64,
        presentation: NumericPresentation<T>,
    ) -> Self {
        let mut slider = Self::new(model, min, max);
        slider.format = presentation.format();
        slider.parse = presentation.parse();
        slider.options.prefix = presentation.chrome_prefix().cloned();
        slider.options.suffix = presentation.chrome_suffix().cloned();
        slider
    }

    pub fn format(mut self, format: NumericFormatFn<T>) -> Self {
        self.format = format;
        self
    }

    pub fn parse(mut self, parse: NumericParseFn<T>) -> Self {
        self.parse = parse;
        self
    }

    pub fn validate(mut self, validate: Option<NumericValidateFn<T>>) -> Self {
        self.validate = validate;
        self
    }

    pub fn options(mut self, options: SliderOptions) -> Self {
        self.options = options;
        self
    }

    #[track_caller]
    pub fn into_element<H: UiHost>(self, cx: &mut ElementContext<'_, H>) -> AnyElement {
        // Important: key internal state per slider instance so multiple sliders don't share
        // drag/typing state.
        //
        // Do not use `test_id` for identity: test ids are for diagnostics/automation, not widget
        // identity. Instead, follow egui/imgui-style identity rules:
        // - Prefer an explicit `id_source` (PushID/id_source equivalent) when provided.
        // - Otherwise key by `(callsite, model.id())` to prevent helper-function callsite
        //   collisions while keeping per-instance state separation.
        let model_id = self.model.id();
        let loc = Location::caller();
        let callsite = (loc.file(), loc.line(), loc.column());

        let id_source = self.options.id_source.clone();
        if let Some(id_source) = id_source.as_deref() {
            cx.keyed(("fret-ui-editor.slider", id_source, model_id), |cx| {
                self.into_element_keyed(cx)
            })
        } else {
            cx.keyed(("fret-ui-editor.slider", callsite, model_id), |cx| {
                self.into_element_keyed(cx)
            })
        }
    }

    fn into_element_keyed<H: UiHost>(self, cx: &mut ElementContext<'_, H>) -> AnyElement {
        let theme = Theme::global(&*cx.app);
        let style = EditorStyle::resolve(theme);
        let density = style.density;
        let frame = style.frame_chrome_small();

        let track_h = theme
            .metric_by_key(EditorTokenKeys::SLIDER_TRACK_HEIGHT)
            .unwrap_or(Px(4.0));
        let thumb_d = theme
            .metric_by_key(EditorTokenKeys::SLIDER_THUMB_DIAMETER)
            .unwrap_or(Px(12.0));

        let track_h = Px(track_h.0.max(1.0));
        let thumb_d = Px(thumb_d.0.max(track_h.0));

        let track_radius = Px(track_h.0 * 0.5);
        let thumb_radius = Px(thumb_d.0 * 0.5);

        let (min, max) = if self.min <= self.max {
            (self.min, self.max)
        } else {
            (self.max, self.min)
        };

        let clamp = self.options.clamp;
        let step = self.options.step;

        let raw_value = cx
            .get_model_copied(&self.model, Invalidation::Paint)
            .unwrap_or_default();
        let value_f = raw_value.to_f64();
        let value_f = quantize_value(min, max, clamp, step, value_f);
        let t = t_from_value(min, max, clamp, value_f);

        // Anchor state to a stable element id under the slider's identity key. This avoids any
        // accidental state sharing across sibling sliders when the surrounding composition
        // changes.
        let state_id = cx.named("slider.state", |cx| cx.root_id());
        let state: Arc<Mutex<SliderState>> = cx.state_for(
            state_id,
            || Arc::new(Mutex::new(SliderState::default())),
            |s| s.clone(),
        );
        let focus_handoff_state_id = cx.named("slider.focus_handoff.state", |cx| cx.root_id());
        let focus_handoff: Arc<Mutex<NumericTextEntryFocusHandoffState>> = cx.state_for(
            focus_handoff_state_id,
            || Arc::new(Mutex::new(NumericTextEntryFocusHandoffState::default())),
            |s| s.clone(),
        );

        let mode = state.lock().unwrap_or_else(|e| e.into_inner()).mode;
        let typing = mode == SliderMode::Typing;

        let enabled = self.options.enabled;
        if !enabled && typing {
            let mut st = state.lock().unwrap_or_else(|e| e.into_inner());
            reset_slider_interaction(&mut st);
        }

        let mut slider_layout = self.options.layout;
        if typing {
            slider_layout = hidden_layout(slider_layout);
        }

        let mut input_layout = self.options.layout;
        if !typing {
            input_layout = hidden_layout(input_layout);
        }

        let model_for_change = self.model.clone();
        let a11y_label = self.options.a11y_label.clone();
        let show_value = self.options.show_value;
        let value_width = self.options.value_width;
        let allow_typing = self.options.allow_typing;
        let typing_test_id = derived_test_id(self.options.test_id.as_ref(), "typing");
        let active_typing_test_id = if typing { typing_test_id.clone() } else { None };
        let value_display_test_id = derived_test_id(self.options.test_id.as_ref(), "value_display");

        let interactive_enabled = enabled && !typing;

        let mut layout = slider_layout;
        if layout.size.min_height.is_none() {
            layout.size.min_height = Some(Length::Px(density.row_height));
        }

        let format = self.format.clone();
        let value_text = (format)(T::from_f64(value_f));
        let (prefix, suffix) = suppress_duplicate_chrome_affixes(
            value_text.as_ref(),
            self.options.prefix.clone(),
            self.options.suffix.clone(),
        );
        let value_display_text =
            compose_affixed_value_text(&value_text, prefix.as_ref(), suffix.as_ref());

        let state_for_slider = state.clone();
        let focus_handoff_for_slider = focus_handoff.clone();
        let mut slider_el = cx.pressable(
            PressableProps {
                enabled: interactive_enabled,
                layout,
                a11y: PressableA11y {
                    label: a11y_label,
                    ..Default::default()
                },
                ..Default::default()
            },
            move |cx, st| {
                let slider_id = cx.root_id();
                {
                    let mut st = state_for_slider.lock().unwrap_or_else(|e| e.into_inner());
                    st.slider_id = Some(slider_id);
                }

                let state_for_down = state_for_slider.clone();
                let focus_handoff_for_down = focus_handoff_for_slider.clone();
                let model_for_down = model_for_change.clone();
                cx.pressable_add_on_pointer_down(Arc::new(move |host, action_cx, down| {
                    if !interactive_enabled {
                        return PressablePointerDownResult::Continue;
                    }
                    if allow_typing && down.button == MouseButton::Left && down.click_count >= 2 {
                        let mut st = state_for_down.lock().unwrap_or_else(|e| e.into_inner());
                        enter_slider_typing(&mut st);
                        {
                            let mut handoff = focus_handoff_for_down
                                .lock()
                                .unwrap_or_else(|e| e.into_inner());
                            arm_numeric_text_entry_focus_handoff(&mut handoff);
                        }
                        host.request_redraw(action_cx.window);
                        return PressablePointerDownResult::SkipDefaultAndStopPropagation;
                    }

                    if down.button != MouseButton::Left {
                        return PressablePointerDownResult::Continue;
                    }

                    let bounds = host.bounds();
                    let next = value_from_slider_local_x(
                        min,
                        max,
                        clamp,
                        step,
                        down.position_local.x.0 as f64,
                        bounds.size.width.0 as f64,
                        show_value,
                        value_width.0 as f64,
                        frame.padding.left.0 as f64,
                        frame.padding.right.0 as f64,
                        thumb_d.0 as f64,
                    );
                    let next_t = T::from_f64(next);
                    let _ = host.models_mut().update(&model_for_down, |v| *v = next_t);
                    host.request_redraw(action_cx.window);

                    host.set_cursor_icon(CursorIcon::ColResize);

                    let mut st = state_for_down.lock().unwrap_or_else(|e| e.into_inner());
                    begin_slider_drag(&mut st, down.pointer_id);

                    PressablePointerDownResult::Continue
                }));

                let state_for_move = state_for_slider.clone();
                let model_for_move = model_for_change.clone();
                cx.pressable_add_on_pointer_move(Arc::new(move |host, action_cx, mv| {
                    let mut st_lock = state_for_move.lock().unwrap_or_else(|e| e.into_inner());
                    if !is_slider_drag_pointer(&st_lock, mv.pointer_id) {
                        return false;
                    }

                    // Best-effort cleanup when the pointer-up event is missed.
                    if !mv.buttons.left {
                        clear_slider_drag(&mut st_lock);
                        return false;
                    }

                    let bounds = host.bounds();
                    let next = value_from_slider_local_x(
                        min,
                        max,
                        clamp,
                        step,
                        mv.position_local.x.0 as f64,
                        bounds.size.width.0 as f64,
                        show_value,
                        value_width.0 as f64,
                        frame.padding.left.0 as f64,
                        frame.padding.right.0 as f64,
                        thumb_d.0 as f64,
                    );
                    let next_t = T::from_f64(next);
                    let _ = host.models_mut().update(&model_for_move, |v| *v = next_t);
                    host.request_redraw(action_cx.window);
                    true
                }));

                let state_for_up = state_for_slider.clone();
                cx.pressable_add_on_pointer_up(Arc::new(move |_host, _action_cx, up| {
                    let mut st = state_for_up.lock().unwrap_or_else(|e| e.into_inner());
                    finish_slider_drag(&mut st, up.pointer_id);
                    PressablePointerUpResult::Continue
                }));

                let theme = Theme::global(&*cx.app);
                let hovered = st.hovered || st.hovered_raw;
                let pressed = st.pressed;
                let focused = st.focused;

                let paint =
                    resolve_slider_paint(theme, interactive_enabled, enabled, hovered, pressed);
                let track_bg = paint.track_bg;
                let fill_bg = paint.fill_bg;
                let thumb_bg = paint.thumb_bg;
                let thumb_border = paint.thumb_border;
                let readout_style = EditorCompactReadoutStyle::resolve(theme, density.row_height);

                let left_grow = t.clamp(0.0, 1.0);
                let right_grow = (1.0 - left_grow).max(0.0);

                vec![editor_input_group_frame(
                    cx,
                    LayoutStyle {
                        size: SizeStyle {
                            width: Length::Fill,
                            height: Length::Fill,
                            min_height: Some(Length::Px(density.row_height)),
                            ..Default::default()
                        },
                        overflow: Overflow::Clip,
                        ..Default::default()
                    },
                    density,
                    frame,
                    EditorFrameState {
                        enabled: interactive_enabled,
                        hovered,
                        pressed,
                        focused,
                        open: false,
                        semantic: EditorFrameSemanticState::default(),
                    },
                    move |cx, frame_visuals| {
                        let track = cx.flex(
                            FlexProps {
                                layout: LayoutStyle {
                                    size: SizeStyle {
                                        width: Length::Fill,
                                        height: Length::Fill,
                                        ..Default::default()
                                    },
                                    flex: FlexItemStyle {
                                        order: 0,
                                        grow: 1.0,
                                        shrink: 1.0,
                                        basis: Length::Px(Px(0.0)),
                                        align_self: None,
                                    },
                                    ..Default::default()
                                },
                                direction: Axis::Horizontal,
                                gap: SpacingLength::Px(Px(0.0)),
                                padding: Edges::all(Px(0.0)).into(),
                                justify: MainAlign::Start,
                                align: CrossAlign::Center,
                                wrap: false,
                            },
                            move |cx| {
                                let mut seg_layout =
                                    |grow: f32, bg: fret_core::Color, left: bool| {
                                        cx.container(
                                            ContainerProps {
                                                layout: LayoutStyle {
                                                    size: SizeStyle {
                                                        width: Length::Auto,
                                                        height: Length::Px(track_h),
                                                        ..Default::default()
                                                    },
                                                    flex: FlexItemStyle {
                                                        order: 0,
                                                        grow,
                                                        shrink: 1.0,
                                                        basis: Length::Px(Px(0.0)),
                                                        align_self: None,
                                                    },
                                                    ..Default::default()
                                                },
                                                background: Some(bg),
                                                corner_radii: if left {
                                                    Corners {
                                                        top_left: track_radius,
                                                        bottom_left: track_radius,
                                                        top_right: Px(0.0),
                                                        bottom_right: Px(0.0),
                                                    }
                                                } else {
                                                    Corners {
                                                        top_left: Px(0.0),
                                                        bottom_left: Px(0.0),
                                                        top_right: track_radius,
                                                        bottom_right: track_radius,
                                                    }
                                                },
                                                ..Default::default()
                                            },
                                            |_cx| vec![],
                                        )
                                    };

                                let left = seg_layout(left_grow, fill_bg, true);
                                let right = seg_layout(right_grow, track_bg, false);

                                let thumb = cx.container(
                                    ContainerProps {
                                        layout: LayoutStyle {
                                            size: SizeStyle {
                                                width: Length::Px(thumb_d),
                                                height: Length::Px(thumb_d),
                                                ..Default::default()
                                            },
                                            flex: FlexItemStyle {
                                                order: 0,
                                                grow: 0.0,
                                                shrink: 0.0,
                                                basis: Length::Px(thumb_d),
                                                align_self: None,
                                            },
                                            ..Default::default()
                                        },
                                        background: Some(thumb_bg),
                                        border: Edges::all(Px(1.0)),
                                        border_color: Some(thumb_border),
                                        corner_radii: Corners::all(thumb_radius),
                                        ..Default::default()
                                    },
                                    |_cx| vec![],
                                );

                                vec![left, thumb, right]
                            },
                        );

                        let value_el = if show_value {
                            let mut value_text_el = cx.text_props(readout_style.text_props(
                                value_display_text.clone(),
                                LayoutStyle {
                                    size: SizeStyle {
                                        width: Length::Fill,
                                        height: Length::Fill,
                                        ..Default::default()
                                    },
                                    ..Default::default()
                                },
                                TextAlign::End,
                                TextOverflow::Clip,
                            ));
                            if let Some(test_id) = value_display_test_id.as_ref() {
                                value_text_el = value_text_el
                                    .test_id(test_id.clone())
                                    .a11y_label(value_display_text.clone());
                            }

                            let value_seg = editor_input_group_segment(
                                cx,
                                LayoutStyle {
                                    size: SizeStyle {
                                        width: Length::Px(value_width),
                                        height: Length::Fill,
                                        ..Default::default()
                                    },
                                    flex: FlexItemStyle {
                                        order: 0,
                                        grow: 0.0,
                                        shrink: 0.0,
                                        basis: Length::Px(value_width),
                                        align_self: None,
                                    },
                                    ..Default::default()
                                },
                                frame.padding,
                                value_text_el,
                            );
                            Some(value_seg)
                        } else {
                            None
                        };

                        let track_seg = editor_input_group_segment(
                            cx,
                            LayoutStyle {
                                size: SizeStyle {
                                    width: Length::Fill,
                                    height: Length::Fill,
                                    ..Default::default()
                                },
                                flex: FlexItemStyle {
                                    order: 0,
                                    grow: 1.0,
                                    shrink: 1.0,
                                    basis: Length::Px(Px(0.0)),
                                    align_self: None,
                                },
                                ..Default::default()
                            },
                            frame.padding,
                            track,
                        );

                        let mut children = vec![track_seg];
                        if let Some(value_el) = value_el {
                            children.push(editor_input_group_divider(cx, frame_visuals.border));
                            children.push(value_el);
                        }

                        vec![cx.flex(
                            FlexProps {
                                layout: LayoutStyle {
                                    size: SizeStyle {
                                        width: Length::Fill,
                                        height: Length::Fill,
                                        ..Default::default()
                                    },
                                    ..Default::default()
                                },
                                direction: Axis::Horizontal,
                                gap: SpacingLength::Px(Px(0.0)),
                                padding: Edges::all(Px(0.0)).into(),
                                justify: MainAlign::Start,
                                align: CrossAlign::Center,
                                wrap: false,
                            },
                            move |_cx| children,
                        )]
                    },
                )]
            },
        );

        if let Some(test_id) = self.options.test_id.as_ref() {
            slider_el = slider_el.test_id(test_id.clone());
        }

        let parse = self.parse.clone();
        let format = self.format.clone();
        let validate = self.validate.clone();

        let parse_for_input = slider_typing_parse(parse, min, max, clamp, step);
        let validate_for_input = slider_typing_validate(validate, min, max, clamp);

        let state_for_input = state.clone();
        let input_focus_target: Arc<Mutex<Option<fret_ui::GlobalElementId>>> =
            Arc::new(Mutex::new(None));
        let input = NumericInput::new(self.model.clone(), format, parse_for_input)
            .validate(validate_for_input)
            .focus_target(input_focus_target.clone())
            .options(NumericInputOptions {
                layout: input_layout,
                enabled: enabled && typing,
                focusable: enabled && typing,
                prefix: prefix.clone(),
                suffix: suffix.clone(),
                selection_behavior: self.options.selection_behavior,
                test_id: active_typing_test_id,
                // Avoid growing the row height when a commit-time validation error occurs.
                // A small trailing status icon keeps the inspector layout stable.
                error_display: NumericInputErrorDisplay::TrailingIcon,
                ..Default::default()
            })
            .on_outcome(Some(Arc::new(move |host, action_cx, outcome| {
                if matches!(
                    outcome,
                    NumericInputOutcome::Committed | NumericInputOutcome::Canceled
                ) {
                    let mut st = state_for_input.lock().unwrap_or_else(|e| e.into_inner());
                    reset_slider_interaction(&mut st);
                    if let Some(slider_id) = st.slider_id {
                        host.request_focus(slider_id);
                    }
                    host.request_redraw(action_cx.window);
                }
            })))
            .into_element(cx);

        if let Some(input_id) = input_focus_target
            .lock()
            .unwrap_or_else(|e| e.into_inner())
            .as_ref()
            .copied()
        {
            let is_focused = cx.is_focused_element(input_id);
            sync_numeric_text_entry_focus_handoff(
                cx,
                input.id,
                &focus_handoff,
                typing,
                input_id,
                is_focused,
            );
        }

        cx.container(Default::default(), move |_cx| vec![slider_el, input])
    }
}
