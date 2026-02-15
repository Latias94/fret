//! Editor-grade horizontal slider control (v1).
//!
//! This is intentionally a small, policy-layer widget:
//! - pointer down sets the value (clamped / stepped),
//! - pointer drag updates the value continuously (best-effort cleanup when pointer-up is missed),
//! - visuals reuse the shared editor "frame" chrome policy to stay consistent with other controls.

use std::sync::{Arc, Mutex};

use fret_core::{Corners, CursorIcon, Edges, PointerId, Px};
use fret_runtime::Model;
use fret_ui::action::{PressablePointerDownResult, PressablePointerUpResult};
use fret_ui::element::{
    AnyElement, ContainerProps, FlexItemStyle, LayoutStyle, Length, Overflow, PressableA11y,
    PressableProps, SizeStyle,
};
use fret_ui::{ElementContext, Invalidation, Theme, UiHost};
use fret_ui_kit::recipes::input::InputTokenKeys;
use fret_ui_kit::{ChromeRefinement, Size};

use crate::primitives::chrome::resolve_editor_frame_chrome;
use crate::primitives::drag_value_core::DragValueScalar;
use crate::primitives::visuals::{EditorFrameState, editor_frame_visuals};
use crate::primitives::{EditorDensity, EditorTokenKeys};

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

#[derive(Debug)]
struct SliderState {
    dragging: bool,
    pointer_id: Option<PointerId>,
}

impl Default for SliderState {
    fn default() -> Self {
        Self {
            dragging: false,
            pointer_id: None,
        }
    }
}

#[derive(Debug, Clone)]
pub struct SliderOptions {
    pub layout: LayoutStyle,
    pub enabled: bool,
    pub clamp: bool,
    /// Quantize to a step size in value space (e.g. `0.01` for normalized floats).
    pub step: Option<f64>,
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
            clamp: true,
            step: None,
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
            options: SliderOptions::default(),
        }
    }

    pub fn options(mut self, options: SliderOptions) -> Self {
        self.options = options;
        self
    }

    #[track_caller]
    pub fn into_element<H: UiHost>(self, cx: &mut ElementContext<'_, H>) -> AnyElement {
        cx.scope(|cx| {
            let theme = Theme::global(&*cx.app);
            let density = EditorDensity::resolve(theme);

            let frame = resolve_editor_frame_chrome(
                theme,
                Size::Small,
                &ChromeRefinement::default(),
                InputTokenKeys {
                    padding_x: Some("component.text_field.padding_x"),
                    padding_y: Some("component.text_field.padding_y"),
                    min_height: Some("component.text_field.min_height"),
                    radius: Some("component.text_field.radius"),
                    border_width: Some("component.text_field.border_width"),
                    bg: Some("component.text_field.bg"),
                    border: Some("component.text_field.border"),
                    border_focus: Some("component.text_field.border_focus"),
                    fg: Some("component.text_field.fg"),
                    text_px: Some("component.text_field.text_px"),
                    selection: Some("component.text_field.selection"),
                },
            );

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

            let state: Arc<Mutex<SliderState>> = cx.with_state(
                || Arc::new(Mutex::new(SliderState::default())),
                |s| s.clone(),
            );

            let model_for_change = self.model.clone();
            let enabled = self.options.enabled;
            let a11y_label = self.options.a11y_label.clone();
            let opts = self.options.clone();

            let mut layout = opts.layout;
            if layout.size.min_height.is_none() {
                layout.size.min_height = Some(density.row_height);
            }

            let mut el = cx.pressable(
                PressableProps {
                    enabled,
                    layout,
                    a11y: PressableA11y {
                        label: a11y_label,
                        ..Default::default()
                    },
                    ..Default::default()
                },
                move |cx, st| {
                    let state_for_down = state.clone();
                    let model_for_down = model_for_change.clone();
                    cx.pressable_add_on_pointer_down(Arc::new(move |host, action_cx, down| {
                        if !enabled {
                            return PressablePointerDownResult::Continue;
                        }
                        if down.button != fret_core::MouseButton::Left {
                            return PressablePointerDownResult::Continue;
                        }

                        let bounds = host.bounds();
                        let width = bounds.size.width.0 as f64;
                        let inner_w =
                            (width - frame.padding.left.0 as f64 - frame.padding.right.0 as f64)
                                .max(0.0);
                        let inner_x = (down.position_local.x.0 as f64
                            - frame.padding.left.0 as f64)
                            .clamp(0.0, inner_w);

                        let next =
                            value_from_x(min, max, clamp, step, inner_x, inner_w, thumb_d.0 as f64);
                        let next_t = T::from_f64(next);
                        let _ = host.models_mut().update(&model_for_down, |v| *v = next_t);
                        host.request_redraw(action_cx.window);

                        host.set_cursor_icon(CursorIcon::ColResize);

                        let mut st = state_for_down.lock().unwrap_or_else(|e| e.into_inner());
                        st.dragging = true;
                        st.pointer_id = Some(down.pointer_id);

                        PressablePointerDownResult::Continue
                    }));

                    let state_for_move = state.clone();
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
                        let inner_w =
                            (width - frame.padding.left.0 as f64 - frame.padding.right.0 as f64)
                                .max(0.0);
                        let inner_x = (mv.position_local.x.0 as f64 - frame.padding.left.0 as f64)
                            .clamp(0.0, inner_w);

                        let next =
                            value_from_x(min, max, clamp, step, inner_x, inner_w, thumb_d.0 as f64);
                        let next_t = T::from_f64(next);
                        let _ = host.models_mut().update(&model_for_move, |v| *v = next_t);
                        host.request_redraw(action_cx.window);
                        true
                    }));

                    let state_for_up = state.clone();
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

                    let frame_visuals = editor_frame_visuals(
                        theme,
                        frame,
                        EditorFrameState {
                            enabled,
                            hovered,
                            pressed,
                            focused,
                            open: false,
                        },
                    );

                    let accent = theme.color_token("accent");
                    let disabled_alpha = if enabled { 1.0 } else { 0.55 };

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

                    vec![cx.container(
                        ContainerProps {
                            layout: LayoutStyle {
                                size: SizeStyle {
                                    width: Length::Fill,
                                    height: Length::Fill,
                                    min_height: Some(density.row_height),
                                    ..Default::default()
                                },
                                overflow: Overflow::Clip,
                                ..Default::default()
                            },
                            padding: frame.padding,
                            background: Some(frame_visuals.bg),
                            border: Edges::all(frame.border_width),
                            border_color: Some(frame_visuals.border),
                            focus_border_color: Some(frame.border_focus),
                            corner_radii: Corners::all(frame.radius),
                            ..Default::default()
                        },
                        move |cx| {
                            vec![cx.flex(
                                fret_ui::element::FlexProps {
                                    layout: LayoutStyle {
                                        size: SizeStyle {
                                            width: Length::Fill,
                                            height: Length::Fill,
                                            ..Default::default()
                                        },
                                        ..Default::default()
                                    },
                                    direction: fret_core::Axis::Horizontal,
                                    gap: Px(0.0),
                                    padding: Edges::all(Px(0.0)),
                                    justify: fret_ui::element::MainAlign::Start,
                                    align: fret_ui::element::CrossAlign::Center,
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
                            )]
                        },
                    )]
                },
            );

            if let Some(test_id) = self.options.test_id.as_ref() {
                el = el.test_id(test_id.clone());
            }

            el
        })
    }
}
