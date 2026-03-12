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
use crate::primitives::style::EditorStyle;
use crate::primitives::visuals::{EditorFrameSemanticState, EditorFrameState};
use fret_core::text::{TextOverflow, TextWrap};
use fret_core::{
    Axis, Corners, CursorIcon, Edges, MouseButton, PointerId, Px, TextAlign, TextStyle,
};
use fret_runtime::Model;
use fret_ui::action::{PressablePointerDownResult, PressablePointerUpResult};
use fret_ui::element::{
    AnyElement, ContainerProps, CrossAlign, FlexItemStyle, FlexProps, InsetStyle, LayoutStyle,
    Length, MainAlign, Overflow, PositionStyle, PressableA11y, PressableProps, SizeStyle,
    SpacingLength, TextProps,
};
use fret_ui::{ElementContext, Invalidation, Theme, UiHost};
use fret_ui_kit::typography;

fn lerp(a: f32, b: f32, t: f32) -> f32 {
    a + (b - a) * t
}

fn mix(a: fret_core::Color, b: fret_core::Color, t: f32) -> fret_core::Color {
    let t = t.clamp(0.0, 1.0);
    fret_core::Color {
        r: lerp(a.r, b.r, t),
        g: lerp(a.g, b.g, t),
        b: lerp(a.b, b.b, t),
        a: lerp(a.a, b.a, t),
    }
}

fn alpha_mul(mut c: fret_core::Color, mul: f32) -> fret_core::Color {
    c.a = (c.a * mul).clamp(0.0, 1.0);
    c
}

fn compose_affixed_value_text(
    value: &Arc<str>,
    prefix: Option<&Arc<str>>,
    suffix: Option<&Arc<str>>,
) -> Arc<str> {
    match (prefix, suffix) {
        (None, None) => value.clone(),
        _ => {
            let mut out = String::new();
            if let Some(prefix) = prefix {
                out.push_str(prefix);
            }
            out.push_str(value);
            if let Some(suffix) = suffix {
                out.push_str(suffix);
            }
            Arc::from(out)
        }
    }
}

fn quantize_value(min: f64, max: f64, clamp: bool, step: Option<f64>, v: f64) -> f64 {
    let mut out = v;
    if let Some(step) = step {
        if step.is_finite() && step > 0.0 && (max - min).is_finite() {
            out = ((out - min) / step).round() * step + min;
        }
    }
    if clamp {
        out = out.clamp(min, max);
    }
    out
}

fn t_from_value(min: f64, max: f64, clamp: bool, v: f64) -> f32 {
    let range = max - min;
    if !range.is_finite() || range.abs() <= f64::EPSILON {
        return 0.0;
    }
    let mut out = (v - min) / range;
    if clamp {
        out = out.clamp(0.0, 1.0);
    }
    out as f32
}

fn value_from_x(
    min: f64,
    max: f64,
    clamp: bool,
    step: Option<f64>,
    x: f64,
    width: f64,
    thumb_d: f64,
) -> f64 {
    let avail = (width - thumb_d).max(0.0);
    if avail <= f64::EPSILON {
        return quantize_value(min, max, clamp, step, min);
    }
    let thumb_r = thumb_d * 0.5;
    let thumb_left = (x - thumb_r).clamp(0.0, avail);
    let t = thumb_left / avail;
    let v = min + (max - min) * t;
    quantize_value(min, max, clamp, step, v)
}

fn hidden_layout(mut layout: LayoutStyle) -> LayoutStyle {
    layout.size = SizeStyle {
        width: Length::Px(Px(0.0)),
        height: Length::Px(Px(0.0)),
        min_width: Some(Length::Px(Px(0.0))),
        min_height: Some(Length::Px(Px(0.0))),
        ..Default::default()
    };
    layout.position = PositionStyle::Absolute;
    layout.inset = InsetStyle {
        top: Some(Px(0.0)).into(),
        left: Some(Px(0.0)).into(),
        ..Default::default()
    };
    layout.overflow = Overflow::Clip;
    layout
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
enum SliderMode {
    Slide,
    Typing,
}

#[derive(Debug)]
struct SliderState {
    mode: SliderMode,
    slider_id: Option<fret_ui::GlobalElementId>,
    input_id: Option<fret_ui::GlobalElementId>,
    dragging: bool,
    pointer_id: Option<PointerId>,
}

impl Default for SliderState {
    fn default() -> Self {
        Self {
            mode: SliderMode::Slide,
            slider_id: None,
            input_id: None,
            dragging: false,
            pointer_id: None,
        }
    }
}

#[derive(Debug, Clone)]
pub struct SliderOptions {
    pub layout: LayoutStyle,
    pub enabled: bool,
    pub prefix: Option<Arc<str>>,
    pub suffix: Option<Arc<str>>,
    pub clamp: bool,
    /// Quantize to a step size in value space (e.g. `0.01` for normalized floats).
    pub step: Option<f64>,
    pub show_value: bool,
    pub value_width: Px,
    pub allow_typing: bool,
    /// Explicit identity source for internal state (drag/typing focus restore).
    ///
    /// This is the editor-control equivalent of egui's `id_source(...)` / ImGui's `PushID`.
    /// Use this when a helper function builds multiple sliders from the same callsite and
    /// you need stable, per-instance state separation.
    pub id_source: Option<Arc<str>>,
    pub test_id: Option<Arc<str>>,
    pub a11y_label: Option<Arc<str>>,
}

impl Default for SliderOptions {
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
            enabled: true,
            prefix: None,
            suffix: None,
            clamp: true,
            step: None,
            show_value: true,
            value_width: Px(52.0),
            allow_typing: true,
            id_source: None,
            test_id: None,
            a11y_label: None,
        }
    }
}

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
        let format: NumericFormatFn<T> = Arc::new(|v| {
            let f = v.to_f64();
            if (f - f.round()).abs() <= 1e-6 {
                Arc::from(format!("{}", f.round() as i64))
            } else {
                Arc::from(format!("{f:.3}"))
            }
        });
        let parse: NumericParseFn<T> = Arc::new(|s| s.trim().parse::<f64>().ok().map(T::from_f64));
        Self {
            model,
            min,
            max,
            format,
            parse,
            validate: None,
            options: SliderOptions::default(),
        }
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
        let state: Arc<Mutex<SliderState>> = cx.with_state_for(
            state_id,
            || Arc::new(Mutex::new(SliderState::default())),
            |s| s.clone(),
        );

        let mode = state.lock().unwrap_or_else(|e| e.into_inner()).mode;
        let typing = mode == SliderMode::Typing;

        let enabled = self.options.enabled;
        if !enabled && typing {
            let mut st = state.lock().unwrap_or_else(|e| e.into_inner());
            st.mode = SliderMode::Slide;
            st.dragging = false;
            st.pointer_id = None;
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
        let prefix = self.options.prefix.clone();
        let suffix = self.options.suffix.clone();
        let typing_test_id = derived_test_id(self.options.test_id.as_ref(), "typing");
        let value_display_test_id = derived_test_id(self.options.test_id.as_ref(), "value_display");

        let interactive_enabled = enabled && !typing;

        let mut layout = slider_layout;
        if layout.size.min_height.is_none() {
            layout.size.min_height = Some(Length::Px(density.row_height));
        }

        let format = self.format.clone();
        let value_text = (format)(T::from_f64(value_f));
        let value_display_text =
            compose_affixed_value_text(&value_text, prefix.as_ref(), suffix.as_ref());

        let state_for_slider = state.clone();
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
                let model_for_down = model_for_change.clone();
                cx.pressable_add_on_pointer_down(Arc::new(move |host, action_cx, down| {
                    if !interactive_enabled {
                        return PressablePointerDownResult::Continue;
                    }
                    if allow_typing && down.button == MouseButton::Left && down.click_count >= 2 {
                        let mut st = state_for_down.lock().unwrap_or_else(|e| e.into_inner());
                        st.mode = SliderMode::Typing;
                        st.dragging = false;
                        st.pointer_id = None;
                        if let Some(input_id) = st.input_id {
                            host.request_focus(input_id);
                        }
                        host.request_redraw(action_cx.window);
                        return PressablePointerDownResult::SkipDefaultAndStopPropagation;
                    }

                    if down.button != MouseButton::Left {
                        return PressablePointerDownResult::Continue;
                    }

                    let bounds = host.bounds();
                    let width = bounds.size.width.0 as f64;
                    let divider_w = if show_value { 1.0 } else { 0.0 };
                    let track_outer_w = (width
                        - divider_w
                        - if show_value {
                            value_width.0 as f64
                        } else {
                            0.0
                        })
                    .max(0.0);
                    let x_outer = (down.position_local.x.0 as f64).clamp(0.0, track_outer_w);
                    let interactive_w = (track_outer_w
                        - frame.padding.left.0 as f64
                        - frame.padding.right.0 as f64)
                        .max(0.0);
                    let inner_x = (x_outer - frame.padding.left.0 as f64).clamp(0.0, interactive_w);

                    let next = value_from_x(
                        min,
                        max,
                        clamp,
                        step,
                        inner_x,
                        interactive_w,
                        thumb_d.0 as f64,
                    );
                    let next_t = T::from_f64(next);
                    let _ = host.models_mut().update(&model_for_down, |v| *v = next_t);
                    host.request_redraw(action_cx.window);

                    host.set_cursor_icon(CursorIcon::ColResize);

                    let mut st = state_for_down.lock().unwrap_or_else(|e| e.into_inner());
                    st.dragging = true;
                    st.pointer_id = Some(down.pointer_id);

                    PressablePointerDownResult::Continue
                }));

                let state_for_move = state_for_slider.clone();
                let model_for_move = model_for_change.clone();
                cx.pressable_add_on_pointer_move(Arc::new(move |host, action_cx, mv| {
                    let mut st_lock = state_for_move.lock().unwrap_or_else(|e| e.into_inner());
                    if !st_lock.dragging || st_lock.pointer_id != Some(mv.pointer_id) {
                        return false;
                    }

                    // Best-effort cleanup when the pointer-up event is missed.
                    if !mv.buttons.left {
                        st_lock.dragging = false;
                        st_lock.pointer_id = None;
                        return false;
                    }

                    let bounds = host.bounds();
                    let width = bounds.size.width.0 as f64;
                    let divider_w = if show_value { 1.0 } else { 0.0 };
                    let track_outer_w = (width
                        - divider_w
                        - if show_value {
                            value_width.0 as f64
                        } else {
                            0.0
                        })
                    .max(0.0);
                    let x_outer = (mv.position_local.x.0 as f64).clamp(0.0, track_outer_w);
                    let interactive_w = (track_outer_w
                        - frame.padding.left.0 as f64
                        - frame.padding.right.0 as f64)
                        .max(0.0);
                    let inner_x = (x_outer - frame.padding.left.0 as f64).clamp(0.0, interactive_w);

                    let next = value_from_x(
                        min,
                        max,
                        clamp,
                        step,
                        inner_x,
                        interactive_w,
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
                    if st.pointer_id == Some(up.pointer_id) {
                        st.dragging = false;
                        st.pointer_id = None;
                    }
                    PressablePointerUpResult::Continue
                }));

                let theme = Theme::global(&*cx.app);
                let hovered = st.hovered || st.hovered_raw;
                let pressed = st.pressed;
                let focused = st.focused;

                let accent = theme.color_token("accent");
                let disabled_alpha = if interactive_enabled { 1.0 } else { 0.55 };

                let mut track_bg = theme
                    .color_by_key("component.slider.track_bg")
                    .or_else(|| theme.color_by_key("muted"))
                    .unwrap_or_else(|| theme.color_token("muted"));
                let mut fill_bg = theme
                    .color_by_key("component.slider.fill_bg")
                    .or_else(|| theme.color_by_key("primary"))
                    .unwrap_or_else(|| theme.color_token("primary"));
                let thumb_bg = theme
                    .color_by_key("component.slider.thumb_bg")
                    .unwrap_or_else(|| theme.color_token("background"));
                let thumb_border = theme
                    .color_by_key("component.slider.thumb_border")
                    .or_else(|| theme.color_by_key("border"))
                    .unwrap_or_else(|| theme.color_token("foreground"));

                if hovered && enabled {
                    track_bg = mix(track_bg, accent, 0.06);
                    fill_bg = mix(fill_bg, accent, 0.04);
                }
                if pressed && enabled {
                    track_bg = mix(track_bg, accent, 0.10);
                    fill_bg = mix(fill_bg, accent, 0.08);
                }

                track_bg = alpha_mul(track_bg, disabled_alpha);
                fill_bg = alpha_mul(fill_bg, disabled_alpha);

                let thumb_bg = alpha_mul(thumb_bg, disabled_alpha);
                let thumb_border = alpha_mul(thumb_border, disabled_alpha);

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
                            let mut value_text_el = cx.text_props(TextProps {
                                layout: LayoutStyle {
                                    size: SizeStyle {
                                        width: Length::Fill,
                                        height: Length::Fill,
                                        ..Default::default()
                                    },
                                    ..Default::default()
                                },
                                text: value_display_text.clone(),
                                style: Some(typography::as_control_text(TextStyle {
                                    size: frame.text_px,
                                    line_height: Some(density.row_height),
                                    ..Default::default()
                                })),
                                color: Some(frame_visuals.fg),
                                wrap: TextWrap::None,
                                overflow: TextOverflow::Clip,
                                align: TextAlign::End,
                                ink_overflow: Default::default(),
                            });
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

        let parse_for_input: NumericParseFn<T> = Arc::new(move |s| {
            let v = parse(s)?;
            let next = quantize_value(min, max, clamp, step, v.to_f64());
            Some(T::from_f64(next))
        });

        let validate_for_input: Option<NumericValidateFn<T>> = if clamp {
            validate
        } else {
            let validate = validate.clone();
            Some(Arc::new(move |v| {
                let f = v.to_f64();
                if f < min || f > max {
                    return Some(Arc::from("Out of range"));
                }
                if let Some(validate) = validate.as_ref() {
                    validate(v)
                } else {
                    None
                }
            }))
        };

        let state_for_input = state.clone();
        let input = NumericInput::new(self.model.clone(), format, parse_for_input)
            .validate(validate_for_input)
            .options(NumericInputOptions {
                layout: input_layout,
                enabled: enabled && typing,
                focusable: enabled && typing,
                prefix: self.options.prefix.clone(),
                suffix: self.options.suffix.clone(),
                test_id: typing_test_id,
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
                    st.mode = SliderMode::Slide;
                    st.dragging = false;
                    st.pointer_id = None;
                    if let Some(slider_id) = st.slider_id {
                        host.request_focus(slider_id);
                    }
                    host.request_redraw(action_cx.window);
                }
            })))
            .into_element(cx);

        {
            let mut st = state.lock().unwrap_or_else(|e| e.into_inner());
            st.input_id = Some(input.id);
        }

        cx.container(Default::default(), move |_cx| vec![slider_el, input])
    }
}

#[cfg(test)]
mod tests {
    use std::sync::Arc;

    use super::compose_affixed_value_text;

    #[test]
    fn compose_affixed_value_text_keeps_plain_value_when_no_affix() {
        let value = Arc::<str>::from("12.0");
        assert_eq!(compose_affixed_value_text(&value, None, None), value);
    }

    #[test]
    fn compose_affixed_value_text_joins_prefix_and_suffix_without_extra_spacing() {
        let value = Arc::<str>::from("12.0");
        let prefix = Arc::<str>::from("$");
        let suffix = Arc::<str>::from("px");
        assert_eq!(
            compose_affixed_value_text(&value, Some(&prefix), Some(&suffix)).as_ref(),
            "$12.0px"
        );
    }
}
