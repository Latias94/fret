//! Material 3 slider (MVP).
//!
//! Goals for the MVP:
//! - Token-driven track + handle visuals (Material Web v30 `md.comp.slider.*`).
//! - Deterministic headless snapshots (stable scene structure).
//! - Minimal interaction: pointer drag + keyboard step.
//!
//! Non-goals (defer until a follow-up):
//! - Value indicator bubble.
//! - Tick marks / stop indicators.
//! - Range (two-thumb) slider.

use std::sync::Arc;
use std::sync::OnceLock;

use fret_core::{
    Color, Corners, CursorIcon, DrawOrder, Edges, KeyCode, LayoutDirection, MouseButton, Px, Rect,
    SemanticsRole, Size,
};
use fret_runtime::Model;
use fret_ui::action::{ActionCx, PointerDownCx, PointerMoveCx, PointerUpCx, UiPointerActionHost};
use fret_ui::element::{
    AnyElement, CanvasProps, ContainerProps, CrossAlign, Length, MainAlign, PointerRegionProps,
    PositionStyle, RowProps, SemanticsProps, StackProps, TextProps,
};
use fret_ui::elements::ElementContext;
use fret_ui::{GlobalElementId, Invalidation, Theme, UiHost};
use fret_ui_kit::{
    ColorRef, OverrideSlot, WidgetState, WidgetStateProperty, WidgetStates,
    resolve_override_slot_with,
};

use crate::foundation::context::{resolved_layout_direction, theme_default_layout_direction};
use crate::foundation::indication::material_pressable_indication_config;
use crate::foundation::interactive_size::enforce_minimum_interactive_size;
use crate::interaction::state_layer::StateLayerAnimator;
use crate::tokens::slider as slider_tokens;

fn default_range_slider_a11y_label() -> Arc<str> {
    static LABEL: OnceLock<Arc<str>> = OnceLock::new();
    LABEL
        .get_or_init(|| Arc::<str>::from("range slider"))
        .clone()
}

fn alpha_mul(mut c: Color, mul: f32) -> Color {
    c.a = (c.a * mul).clamp(0.0, 1.0);
    c
}

fn corners_zero_left(mut c: Corners) -> Corners {
    c.top_left = Px(0.0);
    c.bottom_left = Px(0.0);
    c
}

fn corners_zero_right(mut c: Corners) -> Corners {
    c.top_right = Px(0.0);
    c.bottom_right = Px(0.0);
    c
}

fn quantize_value(value: f32, min: f32, max: f32, step: f32) -> f32 {
    if !value.is_finite() || !min.is_finite() || !max.is_finite() {
        return 0.0;
    }

    if max <= min {
        return min;
    }

    let mut v = value.clamp(min, max);
    if step.is_finite() && step > 0.0 {
        let steps = ((v - min) / step).round();
        v = min + steps * step;
        v = v.clamp(min, max);
    }

    v
}

fn value_to_t(value: f32, min: f32, max: f32) -> f32 {
    if !value.is_finite() || !min.is_finite() || !max.is_finite() || max <= min {
        return 0.0;
    }
    ((value - min) / (max - min)).clamp(0.0, 1.0)
}

fn clamp_range_pair(values: [f32; 2], min: f32, max: f32, step: f32) -> [f32; 2] {
    let mut a = quantize_value(values[0], min, max, step);
    let mut b = quantize_value(values[1], min, max, step);
    if a > b {
        std::mem::swap(&mut a, &mut b);
    }
    [a, b]
}

fn update_range_thumb(
    values: &mut [f32; 2],
    thumb: usize,
    next: f32,
    min: f32,
    max: f32,
    step: f32,
) {
    let other = values[1 - thumb];
    let mut v = quantize_value(next, min, max, step);
    if thumb == 0 {
        v = v.min(other);
    } else {
        v = v.max(other);
    }
    values[thumb] = v;
}

fn keyboard_step_delta(min: f32, max: f32, step: f32) -> f32 {
    if step.is_finite() && step > 0.0 {
        return step;
    }

    let range = (max - min).abs();
    if !range.is_finite() || range <= 0.0 {
        return 0.01;
    }

    // 1% feels like a sensible discrete delta for keyboard input when the slider is continuous.
    range / 100.0
}

fn keyboard_page_delta(min: f32, max: f32, step: f32) -> f32 {
    let step_delta = keyboard_step_delta(min, max, step);
    if !step_delta.is_finite() || step_delta <= 0.0 {
        return step_delta;
    }

    let range = (max - min).abs();
    if !range.is_finite() || range <= 0.0 {
        return step_delta;
    }

    let approx_steps = (range / step_delta).round();
    let approx_steps = approx_steps.clamp(1.0, 10_000.0) as i32;
    let page = (approx_steps / 10).clamp(1, 10) as f32;
    step_delta * page
}

#[derive(Debug, Clone, Default)]
pub struct SliderStyle {
    pub active_track_color: OverrideSlot<ColorRef>,
    pub inactive_track_color: OverrideSlot<ColorRef>,
    pub handle_color: OverrideSlot<ColorRef>,
    pub state_layer_color: OverrideSlot<ColorRef>,
}

impl SliderStyle {
    pub fn active_track_color(mut self, color: WidgetStateProperty<Option<ColorRef>>) -> Self {
        self.active_track_color = Some(color);
        self
    }

    pub fn inactive_track_color(mut self, color: WidgetStateProperty<Option<ColorRef>>) -> Self {
        self.inactive_track_color = Some(color);
        self
    }

    pub fn handle_color(mut self, color: WidgetStateProperty<Option<ColorRef>>) -> Self {
        self.handle_color = Some(color);
        self
    }

    pub fn state_layer_color(mut self, color: WidgetStateProperty<Option<ColorRef>>) -> Self {
        self.state_layer_color = Some(color);
        self
    }

    pub fn merged(mut self, other: Self) -> Self {
        if other.active_track_color.is_some() {
            self.active_track_color = other.active_track_color;
        }
        if other.inactive_track_color.is_some() {
            self.inactive_track_color = other.inactive_track_color;
        }
        if other.handle_color.is_some() {
            self.handle_color = other.handle_color;
        }
        if other.state_layer_color.is_some() {
            self.state_layer_color = other.state_layer_color;
        }
        self
    }
}

#[derive(Clone)]
pub struct Slider {
    value: Model<f32>,
    min: f32,
    max: f32,
    step: f32,
    with_tick_marks: bool,
    tick_marks_count: Option<u16>,
    disabled: bool,
    a11y_label: Option<Arc<str>>,
    test_id: Option<Arc<str>>,
    style: SliderStyle,
}

#[derive(Clone)]
pub struct RangeSlider {
    values: Model<[f32; 2]>,
    min: f32,
    max: f32,
    step: f32,
    with_tick_marks: bool,
    tick_marks_count: Option<u16>,
    disabled: bool,
    a11y_label: Option<Arc<str>>,
    test_id: Option<Arc<str>>,
    style: SliderStyle,
}

impl std::fmt::Debug for RangeSlider {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.debug_struct("RangeSlider")
            .field("min", &self.min)
            .field("max", &self.max)
            .field("step", &self.step)
            .field("with_tick_marks", &self.with_tick_marks)
            .field("tick_marks_count", &self.tick_marks_count)
            .field("disabled", &self.disabled)
            .field("a11y_label", &self.a11y_label)
            .field("test_id", &self.test_id)
            .field("style", &self.style)
            .finish()
    }
}

impl std::fmt::Debug for Slider {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.debug_struct("Slider")
            .field("min", &self.min)
            .field("max", &self.max)
            .field("step", &self.step)
            .field("disabled", &self.disabled)
            .field("a11y_label", &self.a11y_label)
            .field("test_id", &self.test_id)
            .field("style", &self.style)
            .finish()
    }
}

impl Slider {
    pub fn new(value: Model<f32>) -> Self {
        Self {
            value,
            min: 0.0,
            max: 1.0,
            step: 0.0,
            with_tick_marks: false,
            tick_marks_count: None,
            disabled: false,
            a11y_label: None,
            test_id: None,
            style: SliderStyle::default(),
        }
    }

    pub fn range(mut self, min: f32, max: f32) -> Self {
        self.min = min;
        self.max = max;
        self
    }

    pub fn step(mut self, step: f32) -> Self {
        self.step = step;
        self
    }

    pub fn with_tick_marks(mut self, enabled: bool) -> Self {
        self.with_tick_marks = enabled;
        self
    }

    /// Overrides the tick mark count when `with_tick_marks(true)` is enabled.
    ///
    /// This is useful for continuous sliders (no `step`) that still want discrete visual ticks.
    pub fn tick_marks_count(mut self, count: u16) -> Self {
        self.tick_marks_count = Some(count);
        self
    }

    pub fn disabled(mut self, disabled: bool) -> Self {
        self.disabled = disabled;
        self
    }

    pub fn a11y_label(mut self, label: impl Into<Arc<str>>) -> Self {
        self.a11y_label = Some(label.into());
        self
    }

    pub fn test_id(mut self, id: impl Into<Arc<str>>) -> Self {
        self.test_id = Some(id.into());
        self
    }

    pub fn style(mut self, style: SliderStyle) -> Self {
        self.style = self.style.merged(style);
        self
    }

    pub fn into_element<H: UiHost>(self, cx: &mut ElementContext<'_, H>) -> AnyElement {
        slider(
            cx,
            self.value,
            self.min,
            self.max,
            self.step,
            self.with_tick_marks,
            self.tick_marks_count,
            self.disabled,
            self.a11y_label,
            self.test_id,
            self.style,
        )
    }
}

impl RangeSlider {
    pub fn new(values: Model<[f32; 2]>) -> Self {
        Self {
            values,
            min: 0.0,
            max: 1.0,
            step: 0.0,
            with_tick_marks: false,
            tick_marks_count: None,
            disabled: false,
            a11y_label: None,
            test_id: None,
            style: SliderStyle::default(),
        }
    }

    pub fn range(mut self, min: f32, max: f32) -> Self {
        self.min = min;
        self.max = max;
        self
    }

    pub fn step(mut self, step: f32) -> Self {
        self.step = step;
        self
    }

    pub fn with_tick_marks(mut self, enabled: bool) -> Self {
        self.with_tick_marks = enabled;
        self
    }

    /// Overrides the tick mark count when `with_tick_marks(true)` is enabled.
    pub fn tick_marks_count(mut self, count: u16) -> Self {
        self.tick_marks_count = Some(count);
        self
    }

    pub fn disabled(mut self, disabled: bool) -> Self {
        self.disabled = disabled;
        self
    }

    pub fn a11y_label(mut self, label: impl Into<Arc<str>>) -> Self {
        self.a11y_label = Some(label.into());
        self
    }

    pub fn test_id(mut self, id: impl Into<Arc<str>>) -> Self {
        self.test_id = Some(id.into());
        self
    }

    pub fn style(mut self, style: SliderStyle) -> Self {
        self.style = self.style.merged(style);
        self
    }

    pub fn into_element<H: UiHost>(self, cx: &mut ElementContext<'_, H>) -> AnyElement {
        range_slider(
            cx,
            self.values,
            self.min,
            self.max,
            self.step,
            self.with_tick_marks,
            self.tick_marks_count,
            self.disabled,
            self.a11y_label,
            self.test_id,
            self.style,
        )
    }
}

pub fn slider<H: UiHost>(
    cx: &mut ElementContext<'_, H>,
    value: Model<f32>,
    min: f32,
    max: f32,
    step: f32,
    with_tick_marks: bool,
    tick_marks_count: Option<u16>,
    disabled: bool,
    a11y_label: Option<Arc<str>>,
    test_id: Option<Arc<str>>,
    style: SliderStyle,
) -> AnyElement {
    #[derive(Default)]
    struct Runtime {
        dragging: Option<Model<bool>>,
        hovered: Option<Model<bool>>,
        state_layer_target: f32,
        state_layer: StateLayerAnimator,
    }

    let (mut dragging_model, mut hovered_model) = cx.with_state(Runtime::default, |st| {
        (st.dragging.clone(), st.hovered.clone())
    });

    if dragging_model.is_none() {
        let model = cx.app.models_mut().insert(false);
        let value = model.clone();
        cx.with_state(Runtime::default, move |st| {
            st.dragging = Some(value.clone())
        });
        dragging_model = Some(model);
    }
    if hovered_model.is_none() {
        let model = cx.app.models_mut().insert(false);
        let value = model.clone();
        cx.with_state(Runtime::default, move |st| st.hovered = Some(value.clone()));
        hovered_model = Some(model);
    }

    let dragging_model = dragging_model.expect("slider dragging model");
    let hovered_model = hovered_model.expect("slider hovered model");

    let enabled = !disabled;
    let dragging = cx
        .get_model_copied(&dragging_model, Invalidation::Paint)
        .unwrap_or(false);
    let hovered = cx
        .get_model_copied(&hovered_model, Invalidation::Paint)
        .unwrap_or(false);

    let (
        active_track_h,
        inactive_track_h,
        track_shape,
        handle_h,
        handle_shape,
        default_layout_direction,
    ) = {
        let theme = Theme::global(&*cx.app);
        let active_track_h = slider_tokens::active_track_height(theme);
        let inactive_track_h = slider_tokens::inactive_track_height(theme);
        let track_shape = slider_tokens::track_shape(theme);
        let handle_h = slider_tokens::handle_height(theme);
        let handle_shape = slider_tokens::handle_shape(theme);
        let default_layout_direction = theme_default_layout_direction(theme);
        (
            active_track_h,
            inactive_track_h,
            track_shape,
            handle_h,
            handle_shape,
            default_layout_direction,
        )
    };
    let track_h = Px(active_track_h.0.max(inactive_track_h.0));

    let value_now = cx
        .get_model_copied(&value, Invalidation::Paint)
        .unwrap_or(min)
        .clamp(min, max);
    let value_now = quantize_value(value_now, min, max, step);
    let t_value = value_to_t(value_now, min, max);

    let mut semantics = SemanticsProps::default();
    semantics.layout.size.width = Length::Fill;
    semantics.layout.size.height = Length::Fill;
    semantics.role = SemanticsRole::Slider;
    semantics.label = a11y_label;
    semantics.test_id = test_id;
    semantics.focusable = !disabled;
    semantics.disabled = disabled;

    #[derive(Default)]
    struct DerivedValueString {
        bits: u32,
        value: Option<Arc<str>>,
    }

    let value_text = cx.with_state(DerivedValueString::default, |st| {
        let bits = value_now.to_bits();
        if st.value.is_none() || st.bits != bits {
            st.bits = bits;
            st.value = Some(Arc::from(format!("{value_now:.3}")));
        }
        st.value.as_ref().expect("value").clone()
    });

    semantics.value = Some(value_text);

    cx.semantics_with_id(semantics, move |cx, semantics_id| {
        let layout_direction = resolved_layout_direction(cx, default_layout_direction);
        let rtl = layout_direction == LayoutDirection::Rtl;
        let sign = if rtl { -1.0 } else { 1.0 };
        let t_visual = if rtl { 1.0 - t_value } else { t_value };
        let t = t_visual;
        let focus_visible = fret_ui::focus_visible::is_focus_visible(&mut *cx.app, Some(cx.window));
        let has_focus = enabled && cx.is_focused_element(semantics_id);
        let is_focus_visible = has_focus && focus_visible;

        let token_interaction = if enabled && dragging {
            slider_tokens::SliderInteraction::Pressed
        } else if is_focus_visible {
            slider_tokens::SliderInteraction::Focused
        } else if enabled && hovered {
            slider_tokens::SliderInteraction::Hovered
        } else {
            slider_tokens::SliderInteraction::None
        };

        let mut states = WidgetStates::default();
        states.set(WidgetState::Disabled, !enabled);
        states.set(WidgetState::Hovered, enabled && hovered);
        states.set(WidgetState::Active, enabled && dragging);
        states.set(WidgetState::Focused, has_focus);
        states.set(WidgetState::FocusVisible, is_focus_visible);

        let (
            active_track_color,
            inactive_track_color,
            handle_color,
            state_layer_target,
            state_layer_color,
            state_layer_size,
            config,
            handle_w,
        ) = {
            let theme = Theme::global(&*cx.app);

            let active_track_default =
                slider_tokens::active_track_color(theme, enabled, token_interaction);
            let inactive_track_default =
                slider_tokens::inactive_track_color(theme, enabled, token_interaction);
            let handle_default = slider_tokens::handle_color(theme, enabled, token_interaction);

            let active_track_color = resolve_override_slot_with(
                style.active_track_color.as_ref(),
                states,
                |c| c.resolve(theme),
                || active_track_default,
            );
            let inactive_track_color = resolve_override_slot_with(
                style.inactive_track_color.as_ref(),
                states,
                |c| c.resolve(theme),
                || inactive_track_default,
            );
            let handle_color = resolve_override_slot_with(
                style.handle_color.as_ref(),
                states,
                |c| c.resolve(theme),
                || handle_default,
            );

            let state_layer_target = if enabled && dragging {
                slider_tokens::pressed_state_layer_opacity(theme)
            } else {
                slider_tokens::state_layer_target_opacity(theme, enabled, token_interaction)
            };
            let state_layer_default = slider_tokens::state_layer_color(theme, token_interaction);
            let state_layer_color = resolve_override_slot_with(
                style.state_layer_color.as_ref(),
                states,
                |c| c.resolve(theme),
                || state_layer_default,
            );
            let state_layer_size = slider_tokens::state_layer_size(theme);
            let config = material_pressable_indication_config(theme, None);

            let handle_w = slider_tokens::handle_width(theme, enabled, token_interaction);

            (
                active_track_color,
                inactive_track_color,
                handle_color,
                state_layer_target,
                state_layer_color,
                state_layer_size,
                config,
                handle_w,
            )
        };

        let now_frame = cx.frame_id.0;
        let (state_layer_opacity, state_layer_want_frames) =
            cx.with_state(Runtime::default, |st| {
                if (state_layer_target - st.state_layer_target).abs() > 1e-6 {
                    st.state_layer_target = state_layer_target;
                    st.state_layer.set_target(
                        now_frame,
                        state_layer_target,
                        config.state_duration_ms,
                        config.easing,
                    );
                }
                st.state_layer.advance(now_frame);
                (st.state_layer.value(), st.state_layer.is_active())
            });

        let value_on_key = value.clone();
        cx.key_on_key_down_for(
            semantics_id,
            Arc::new(move |host, acx, down| {
                if disabled || down.repeat {
                    return false;
                }
                if down.modifiers.alt || down.modifiers.ctrl || down.modifiers.meta {
                    return false;
                }

                let delta = keyboard_step_delta(min, max, step);
                let page_delta = keyboard_page_delta(min, max, step);
                let next = match down.key {
                    KeyCode::ArrowLeft => Some(-delta * sign),
                    KeyCode::ArrowRight => Some(delta * sign),
                    KeyCode::ArrowDown => Some(-delta),
                    KeyCode::ArrowUp => Some(delta),
                    KeyCode::PageDown => Some(-page_delta),
                    KeyCode::PageUp => Some(page_delta),
                    KeyCode::Home => {
                        let _ = host.models_mut().update(&value_on_key, |v| *v = min);
                        host.request_redraw(acx.window);
                        return true;
                    }
                    KeyCode::End => {
                        let _ = host.models_mut().update(&value_on_key, |v| *v = max);
                        host.request_redraw(acx.window);
                        return true;
                    }
                    _ => None,
                };

                let Some(delta) = next else { return false };
                let cur = host
                    .models_mut()
                    .read(&value_on_key, |v| *v)
                    .ok()
                    .unwrap_or(min);
                let next = quantize_value(cur + delta, min, max, step);
                let _ = host.models_mut().update(&value_on_key, |v| *v = next);
                host.request_redraw(acx.window);
                true
            }),
        );

        let value_on_pointer = value.clone();
        let dragging_on_pointer = dragging_model.clone();
        let hovered_on_pointer = hovered_model.clone();

        let update_from_pos =
            move |host: &mut dyn UiPointerActionHost, acx: ActionCx, pos: fret_core::Point| {
                let bounds = host.bounds();
                if bounds.size.width.0 <= 0.0 {
                    return;
                }
                let t_input = ((pos.x.0 - bounds.origin.x.0) / bounds.size.width.0).clamp(0.0, 1.0);
                let t_value = if rtl { 1.0 - t_input } else { t_input };
                let v = min + t_value * (max - min);
                let v = quantize_value(v, min, max, step);
                let _ = host.models_mut().update(&value_on_pointer, |out| *out = v);
                host.request_redraw(acx.window);
            };

        let on_down = Arc::new({
            let update_from_pos = update_from_pos.clone();
            let dragging_on_pointer = dragging_on_pointer.clone();
            let hovered_on_pointer = hovered_on_pointer.clone();
            move |host: &mut dyn UiPointerActionHost, acx: ActionCx, down: PointerDownCx| {
                if disabled || down.button != MouseButton::Left {
                    return false;
                }
                host.request_focus(semantics_id);
                host.capture_pointer();
                host.set_cursor_icon(CursorIcon::Pointer);
                let _ = host
                    .models_mut()
                    .update(&dragging_on_pointer, |v| *v = true);
                let _ = host.models_mut().update(&hovered_on_pointer, |v| *v = true);
                update_from_pos(host, acx, down.position);
                true
            }
        });

        let on_move = Arc::new({
            let update_from_pos = update_from_pos.clone();
            let dragging_on_pointer = dragging_on_pointer.clone();
            let hovered_on_pointer = hovered_on_pointer.clone();
            move |host: &mut dyn UiPointerActionHost, acx: ActionCx, mv: PointerMoveCx| {
                if disabled {
                    return false;
                }
                host.set_cursor_icon(CursorIcon::Pointer);
                let is_dragging = host
                    .models_mut()
                    .read(&dragging_on_pointer, |v| *v)
                    .ok()
                    .unwrap_or(false);
                let hovered = host.bounds().contains(mv.position);
                let _ = host
                    .models_mut()
                    .update(&hovered_on_pointer, |v| *v = hovered);
                if is_dragging {
                    update_from_pos(host, acx, mv.position);
                }
                true
            }
        });

        let on_up = Arc::new({
            let dragging_on_pointer = dragging_on_pointer.clone();
            let hovered_on_pointer = hovered_on_pointer.clone();
            move |host: &mut dyn UiPointerActionHost, acx: ActionCx, up: PointerUpCx| {
                if disabled {
                    return false;
                }
                let hovered = host.bounds().contains(up.position);
                let _ = host
                    .models_mut()
                    .update(&hovered_on_pointer, |v| *v = hovered);
                let _ = host
                    .models_mut()
                    .update(&dragging_on_pointer, |v| *v = false);
                host.release_pointer_capture();
                host.request_redraw(acx.window);
                true
            }
        });

        let (
            value_indicator_bottom_space,
            value_indicator_container_color,
            value_indicator_label_style,
            value_indicator_label_color,
            tick_size,
            tick_shape,
            tick_active_color,
            tick_active_opacity,
            tick_inactive_color,
            tick_inactive_opacity,
        ) = {
            let theme = Theme::global(&*cx.app);
            (
                slider_tokens::value_indicator_bottom_space(theme),
                slider_tokens::value_indicator_container_color(theme),
                slider_tokens::value_indicator_label_style(theme),
                slider_tokens::value_indicator_label_color(theme),
                slider_tokens::tick_mark_size(theme),
                slider_tokens::tick_mark_shape(theme),
                slider_tokens::tick_mark_color(theme, enabled, true),
                slider_tokens::tick_mark_opacity(theme, enabled, true),
                slider_tokens::tick_mark_color(theme, enabled, false),
                slider_tokens::tick_mark_opacity(theme, enabled, false),
            )
        };

        let mut pointer_props = PointerRegionProps::default();
        pointer_props.layout.size.width = Length::Fill;
        pointer_props.layout.size.height = Length::Fill;
        {
            let theme = Theme::global(&*cx.app);
            enforce_minimum_interactive_size(&mut pointer_props.layout, theme);
        }
        pointer_props.enabled = enabled;

        vec![cx.pointer_region(pointer_props, move |cx| {
            cx.pointer_region_on_pointer_down(on_down);
            cx.pointer_region_on_pointer_move(on_move);
            cx.pointer_region_on_pointer_up(on_up);

            let mut canvas_props = CanvasProps::default();
            canvas_props.layout.size.width = Length::Fill;
            canvas_props.layout.size.height = Length::Px(handle_h);

            let content = cx.canvas(canvas_props, move |p| {
                let bounds = p.bounds();
                if bounds.size.width.0 <= 0.0 || bounds.size.height.0 <= 0.0 {
                    return;
                }

                let track_h = Px(track_h.0.min(bounds.size.height.0));
                let track_y = Px(bounds.origin.y.0 + (bounds.size.height.0 - track_h.0) * 0.5);

                let thumb_x = Px(bounds.origin.x.0 + bounds.size.width.0 * t);
                let thumb_y = Px(bounds.origin.y.0 + bounds.size.height.0 * 0.5);

                let left_w = (thumb_x.0 - bounds.origin.x.0).clamp(0.0, bounds.size.width.0);
                let right_w = (bounds.size.width.0 - left_w).clamp(0.0, bounds.size.width.0);

                let (left_color, right_color) = if rtl {
                    (inactive_track_color, active_track_color)
                } else {
                    (active_track_color, inactive_track_color)
                };

                if left_w > 0.0 {
                    let rect = Rect::new(
                        fret_core::Point::new(bounds.origin.x, track_y),
                        Size::new(Px(left_w), track_h),
                    );
                    let corners = if right_w <= 0.0 {
                        track_shape
                    } else {
                        corners_zero_right(track_shape)
                    };
                    p.scene().push(fret_core::SceneOp::Quad {
                        order: DrawOrder(0),
                        rect,
                        background: fret_core::Paint::Solid(left_color),
                        border: Edges::all(Px(0.0)),
                        border_paint: fret_core::Paint::TRANSPARENT,
                        corner_radii: corners,
                    });
                }

                if right_w > 0.0 {
                    let rect = Rect::new(
                        fret_core::Point::new(Px(bounds.origin.x.0 + left_w), track_y),
                        Size::new(Px(right_w), track_h),
                    );
                    let corners = if left_w <= 0.0 {
                        track_shape
                    } else {
                        corners_zero_left(track_shape)
                    };
                    p.scene().push(fret_core::SceneOp::Quad {
                        order: DrawOrder(0),
                        rect,
                        background: fret_core::Paint::Solid(right_color),
                        border: Edges::all(Px(0.0)),
                        border_paint: fret_core::Paint::TRANSPARENT,
                        corner_radii: corners,
                    });
                }

                if state_layer_opacity > 0.0 {
                    let size = Px(state_layer_size.0.min(bounds.size.width.0).max(0.0));
                    if size.0 > 0.0 {
                        let x = Px((thumb_x.0 - size.0 * 0.5).clamp(
                            bounds.origin.x.0,
                            bounds.origin.x.0 + bounds.size.width.0 - size.0,
                        ));
                        let y = Px((thumb_y.0 - size.0 * 0.5).clamp(
                            bounds.origin.y.0,
                            bounds.origin.y.0 + bounds.size.height.0 - size.0,
                        ));
                        let rect = Rect::new(fret_core::Point::new(x, y), Size::new(size, size));
                        p.scene().push(fret_core::SceneOp::Quad {
                            order: DrawOrder(2),
                            rect,
                            background: fret_core::Paint::Solid(alpha_mul(
                                state_layer_color,
                                state_layer_opacity,
                            )),
                            border: Edges::all(Px(0.0)),
                            border_paint: fret_core::Paint::TRANSPARENT,
                            corner_radii: Corners::all(Px(size.0 * 0.5)),
                        });
                    }
                }

                if with_tick_marks {
                    let mut tick_count: Option<i32> = tick_marks_count
                        .map(|c| i32::from(c).clamp(0, 256))
                        .filter(|c| *c > 0);

                    if tick_count.is_none()
                        && step.is_finite()
                        && step > 0.0
                        && max.is_finite()
                        && min.is_finite()
                        && max > min
                    {
                        let approx_steps = ((max - min) / step).round();
                        let approx_steps = approx_steps.clamp(1.0, 200.0) as i32;
                        tick_count = Some(approx_steps + 1);
                    }

                    if let Some(tick_count) = tick_count {
                        let tick_count = tick_count.clamp(1, 201);

                        let center_y = Px(track_y.0 + track_h.0 * 0.5);
                        let y = Px((center_y.0 - tick_size.0 * 0.5).clamp(
                            bounds.origin.y.0,
                            bounds.origin.y.0 + bounds.size.height.0 - tick_size.0,
                        ));

                        for i in 0..tick_count {
                            let tick_t = if tick_count <= 1 {
                                0.0
                            } else {
                                (i as f32) / ((tick_count - 1) as f32)
                            };
                            let x = Px(bounds.origin.x.0 + bounds.size.width.0 * tick_t
                                - tick_size.0 * 0.5);
                            let x = Px(x.0.clamp(
                                bounds.origin.x.0,
                                bounds.origin.x.0 + bounds.size.width.0 - tick_size.0,
                            ));

                            let active = if rtl {
                                tick_t + 1e-6 >= t
                            } else {
                                tick_t <= (t + 1e-6)
                            };
                            let (color, opacity) = if active {
                                (tick_active_color, tick_active_opacity)
                            } else {
                                (tick_inactive_color, tick_inactive_opacity)
                            };

                            let rect = Rect::new(
                                fret_core::Point::new(x, y),
                                Size::new(tick_size, tick_size),
                            );
                            p.scene().push(fret_core::SceneOp::Quad {
                                order: DrawOrder(1),
                                rect,
                                background: fret_core::Paint::Solid(alpha_mul(color, opacity)),
                                border: Edges::all(Px(0.0)),
                                border_paint: fret_core::Paint::TRANSPARENT,
                                corner_radii: tick_shape,
                            });
                        }
                    }
                }

                let handle_w = Px(handle_w.0.min(bounds.size.width.0).max(0.0));
                let handle_h = Px(handle_h.0.min(bounds.size.height.0).max(0.0));
                if handle_w.0 > 0.0 && handle_h.0 > 0.0 {
                    let x = Px((thumb_x.0 - handle_w.0 * 0.5).clamp(
                        bounds.origin.x.0,
                        bounds.origin.x.0 + bounds.size.width.0 - handle_w.0,
                    ));
                    let y = Px(bounds.origin.y.0 + (bounds.size.height.0 - handle_h.0) * 0.5);
                    let handle =
                        Rect::new(fret_core::Point::new(x, y), Size::new(handle_w, handle_h));

                    p.scene().push(fret_core::SceneOp::Quad {
                        order: DrawOrder(3),
                        rect: handle,
                        background: fret_core::Paint::Solid(handle_color),
                        border: Edges::all(Px(0.0)),
                        border_paint: fret_core::Paint::TRANSPARENT,
                        corner_radii: handle_shape,
                    });
                }

                if state_layer_want_frames {
                    p.request_animation_frame();
                }
            });

            let mut stack_props = StackProps::default();
            stack_props.layout.size.width = Length::Fill;
            stack_props.layout.size.height = Length::Fill;

            let show_value_indicator = enabled && dragging;
            let value_indicator = if show_value_indicator {
                let t_value = t_value;
                let t_visual = t;
                let percent = (t_value * 100.0).round().clamp(0.0, 100.0) as i32;
                let label = Arc::from(percent.to_string());

                let bubble_w = Px(48.0);
                let bubble_h = Px(28.0);
                let bottom_space = value_indicator_bottom_space;

                let last = cx.last_bounds_for_element(semantics_id);
                let (left, top) = if let Some(bounds) = last {
                    let x = (bounds.size.width.0 * t_visual).clamp(0.0, bounds.size.width.0);
                    let left =
                        Px((x - bubble_w.0 * 0.5).clamp(0.0, bounds.size.width.0 - bubble_w.0));
                    let top = Px(-bubble_h.0 - bottom_space.0);
                    (left, top)
                } else {
                    (Px(0.0), Px(-bubble_h.0 - bottom_space.0))
                };

                let mut container = ContainerProps::default();
                container.layout.position = PositionStyle::Absolute;
                container.layout.inset.left = Some(left);
                container.layout.inset.top = Some(top);
                container.layout.size.width = Length::Px(bubble_w);
                container.layout.size.height = Length::Px(bubble_h);
                container.layout.overflow = fret_ui::element::Overflow::Visible;
                container.background = Some(value_indicator_container_color);
                container.corner_radii = Corners::all(Px(9999.0));

                let mut row = RowProps::default();
                row.layout.size.width = Length::Fill;
                row.layout.size.height = Length::Fill;
                row.justify = MainAlign::Center;
                row.align = CrossAlign::Center;

                let text_style = value_indicator_label_style;
                let text_color = value_indicator_label_color;

                let mut text = TextProps::new(label);
                text.layout.size.width = Length::Fill;
                text.layout.size.height = Length::Fill;
                text.style = Some(text_style);
                text.color = Some(text_color);
                text.wrap = fret_core::TextWrap::None;
                text.overflow = fret_core::TextOverflow::Clip;

                Some(cx.container(container, |cx| {
                    vec![cx.row(row, |cx| vec![cx.text_props(text)])]
                }))
            } else {
                None
            };

            vec![cx.stack_props(stack_props, |_cx| {
                let mut out = vec![content];
                if let Some(indicator) = value_indicator {
                    out.push(indicator);
                }
                out
            })]
        })]
    })
}

pub fn range_slider<H: UiHost>(
    cx: &mut ElementContext<'_, H>,
    values: Model<[f32; 2]>,
    min: f32,
    max: f32,
    step: f32,
    with_tick_marks: bool,
    tick_marks_count: Option<u16>,
    disabled: bool,
    a11y_label: Option<Arc<str>>,
    test_id: Option<Arc<str>>,
    style: SliderStyle,
) -> AnyElement {
    #[derive(Default)]
    struct Runtime {
        dragging: Option<Model<bool>>,
        hovered: Option<Model<bool>>,
        active_thumb: Option<Model<u8>>,
        state_layer_target_start: f32,
        state_layer_start: StateLayerAnimator,
        state_layer_target_end: f32,
        state_layer_end: StateLayerAnimator,
    }

    let (mut dragging_model, mut hovered_model, mut active_thumb_model) =
        cx.with_state(Runtime::default, |st| {
            (
                st.dragging.clone(),
                st.hovered.clone(),
                st.active_thumb.clone(),
            )
        });

    if dragging_model.is_none() {
        let model = cx.app.models_mut().insert(false);
        let value = model.clone();
        cx.with_state(Runtime::default, move |st| {
            st.dragging = Some(value.clone())
        });
        dragging_model = Some(model);
    }
    if hovered_model.is_none() {
        let model = cx.app.models_mut().insert(false);
        let value = model.clone();
        cx.with_state(Runtime::default, move |st| st.hovered = Some(value.clone()));
        hovered_model = Some(model);
    }
    if active_thumb_model.is_none() {
        let model = cx.app.models_mut().insert(0u8);
        let value = model.clone();
        cx.with_state(Runtime::default, move |st| {
            st.active_thumb = Some(value.clone())
        });
        active_thumb_model = Some(model);
    }

    let dragging_model = dragging_model.expect("range slider dragging model");
    let hovered_model = hovered_model.expect("range slider hovered model");
    let active_thumb_model = active_thumb_model.expect("range slider active thumb model");

    let enabled = !disabled;

    let dragging = cx
        .get_model_copied(&dragging_model, Invalidation::Paint)
        .unwrap_or(false);
    let hovered = cx
        .get_model_copied(&hovered_model, Invalidation::Paint)
        .unwrap_or(false);
    let values_now = cx
        .get_model_copied(&values, Invalidation::Paint)
        .unwrap_or([min, max]);
    let values_now = clamp_range_pair(values_now, min, max, step);
    let t0_value = value_to_t(values_now[0], min, max);
    let t1_value = value_to_t(values_now[1], min, max);
    let (
        default_layout_direction,
        active_track_h,
        inactive_track_h,
        track_shape,
        handle_h,
        handle_shape,
    ) = {
        let theme = Theme::global(&*cx.app);
        let default_layout_direction = theme_default_layout_direction(theme);
        let active_track_h = slider_tokens::active_track_height(theme);
        let inactive_track_h = slider_tokens::inactive_track_height(theme);
        let track_shape = slider_tokens::track_shape(theme);
        let handle_h = slider_tokens::handle_height(theme);
        let handle_shape = slider_tokens::handle_shape(theme);
        (
            default_layout_direction,
            active_track_h,
            inactive_track_h,
            track_shape,
            handle_h,
            handle_shape,
        )
    };

    let mut semantics = SemanticsProps::default();
    semantics.layout.size.width = Length::Fill;
    semantics.layout.size.height = Length::Fill;
    semantics.role = SemanticsRole::Group;
    semantics.label = a11y_label.clone();
    semantics.test_id = test_id.clone();
    semantics.focusable = false;
    semantics.disabled = !enabled;

    #[derive(Default)]
    struct DerivedRangeValueString {
        start_bits: u32,
        end_bits: u32,
        value: Option<Arc<str>>,
    }

    let range_value_text = cx.with_state(DerivedRangeValueString::default, |st| {
        let start_bits = values_now[0].to_bits();
        let end_bits = values_now[1].to_bits();
        if st.value.is_none() || st.start_bits != start_bits || st.end_bits != end_bits {
            st.start_bits = start_bits;
            st.end_bits = end_bits;
            st.value = Some(Arc::from(format!(
                "{:.3}..{:.3}",
                values_now[0], values_now[1]
            )));
        }
        st.value.as_ref().expect("value").clone()
    });

    semantics.value = Some(range_value_text);

    let track_h = Px(active_track_h.0.max(inactive_track_h.0));

    cx.semantics_with_id(semantics, move |cx, group_semantics_id| {
        let layout_direction = resolved_layout_direction(cx, default_layout_direction);
        let rtl = layout_direction == LayoutDirection::Rtl;
        let sign = if rtl { -1.0 } else { 1.0 };

        let t0_visual = if rtl { 1.0 - t0_value } else { t0_value };
        let t1_visual = if rtl { 1.0 - t1_value } else { t1_value };
        let t_left = t0_visual.min(t1_visual);
        let t_right = t0_visual.max(t1_visual);

        let thumb_label_base: Arc<str> = a11y_label
            .clone()
            .unwrap_or_else(default_range_slider_a11y_label);

        #[derive(Default)]
        struct DerivedThumbStrings {
            label_base: Option<Arc<str>>,
            test_id_base: Option<Arc<str>>,
            start_label: Option<Arc<str>>,
            end_label: Option<Arc<str>>,
            start_test_id: Option<Arc<str>>,
            end_test_id: Option<Arc<str>>,
        }

        let (start_label, end_label, start_test_id, end_test_id) =
            cx.with_state(DerivedThumbStrings::default, |st| {
                if st.start_label.is_none()
                    || st.label_base.as_deref() != Some(thumb_label_base.as_ref())
                    || st.test_id_base.as_deref() != test_id.as_deref()
                {
                    st.label_base = Some(thumb_label_base.clone());
                    st.test_id_base = test_id.clone();
                    st.start_label =
                        Some(Arc::from(format!("{} start", thumb_label_base.as_ref())));
                    st.end_label = Some(Arc::from(format!("{} end", thumb_label_base.as_ref())));
                    st.start_test_id = test_id
                        .as_ref()
                        .map(|id| Arc::from(format!("{}.start", id.as_ref())));
                    st.end_test_id = test_id
                        .as_ref()
                        .map(|id| Arc::from(format!("{}.end", id.as_ref())));
                }
                (
                    st.start_label.as_ref().expect("start_label").clone(),
                    st.end_label.as_ref().expect("end_label").clone(),
                    st.start_test_id.clone(),
                    st.end_test_id.clone(),
                )
            });

        #[derive(Default)]
        struct DerivedThumbValues {
            start_bits: u32,
            end_bits: u32,
            start: Option<Arc<str>>,
            end: Option<Arc<str>>,
        }

        let (start_value_text, end_value_text) = cx.with_state(DerivedThumbValues::default, |st| {
            let start_bits = values_now[0].to_bits();
            let end_bits = values_now[1].to_bits();
            if st.start.is_none() || st.start_bits != start_bits {
                st.start_bits = start_bits;
                st.start = Some(Arc::from(format!("{:.3}", values_now[0])));
            }
            if st.end.is_none() || st.end_bits != end_bits {
                st.end_bits = end_bits;
                st.end = Some(Arc::from(format!("{:.3}", values_now[1])));
            }
            (
                st.start.as_ref().expect("start_value").clone(),
                st.end.as_ref().expect("end_value").clone(),
            )
        });

        let last = cx.last_bounds_for_element(group_semantics_id);
        let (start_left, end_left) = if let Some(bounds) = last {
            let start_x = (bounds.size.width.0 * t0_visual).clamp(0.0, bounds.size.width.0);
            let end_x = (bounds.size.width.0 * t1_visual).clamp(0.0, bounds.size.width.0);
            let w = handle_h.0.max(0.0);
            let start_left = Px((start_x - w * 0.5).clamp(0.0, bounds.size.width.0 - w));
            let end_left = Px((end_x - w * 0.5).clamp(0.0, bounds.size.width.0 - w));
            (start_left, end_left)
        } else {
            (Px(0.0), Px(0.0))
        };

        let mut start_thumb_id: Option<GlobalElementId> = None;
        let mut end_thumb_id: Option<GlobalElementId> = None;

        let mut start_thumb_semantics = SemanticsProps::default();
        start_thumb_semantics.layout.position = PositionStyle::Absolute;
        start_thumb_semantics.layout.inset.left = Some(start_left);
        start_thumb_semantics.layout.inset.top = Some(Px(0.0));
        start_thumb_semantics.layout.size.width = Length::Px(handle_h);
        start_thumb_semantics.layout.size.height = Length::Px(handle_h);
        start_thumb_semantics.role = SemanticsRole::Slider;
        start_thumb_semantics.label = Some(start_label);
        start_thumb_semantics.test_id = start_test_id;
        start_thumb_semantics.focusable = enabled;
        start_thumb_semantics.disabled = !enabled;
        start_thumb_semantics.value = Some(start_value_text);

        let start_thumb = cx.semantics_with_id(start_thumb_semantics, |cx, semantics_id| {
            start_thumb_id = Some(semantics_id);

            let values_on_key = values.clone();
            let active_thumb_on_key = active_thumb_model.clone();
            cx.key_on_key_down_for(
                semantics_id,
                Arc::new(move |host, acx, down| {
                    if !enabled || down.repeat {
                        return false;
                    }
                    if down.modifiers.alt || down.modifiers.ctrl || down.modifiers.meta {
                        return false;
                    }

                    let _ = host.models_mut().update(&active_thumb_on_key, |v| *v = 0);

                    let delta = keyboard_step_delta(min, max, step);
                    let page_delta = keyboard_page_delta(min, max, step);
                    let (kind, delta) = match down.key {
                        KeyCode::ArrowLeft => (0, -delta * sign),
                        KeyCode::ArrowRight => (0, delta * sign),
                        KeyCode::ArrowDown => (0, -delta),
                        KeyCode::ArrowUp => (0, delta),
                        KeyCode::PageDown => (0, -page_delta),
                        KeyCode::PageUp => (0, page_delta),
                        KeyCode::Home => (1, 0.0),
                        KeyCode::End => (2, 0.0),
                        _ => return false,
                    };

                    let _ = host.models_mut().update(&values_on_key, |v| {
                        let clamped = clamp_range_pair(*v, min, max, step);
                        *v = clamped;
                        match kind {
                            0 => {
                                let next = v[0] + delta;
                                update_range_thumb(v, 0, next, min, max, step);
                            }
                            1 => update_range_thumb(v, 0, min, min, max, step),
                            2 => update_range_thumb(v, 0, v[1], min, max, step),
                            _ => {}
                        }
                    });

                    host.request_redraw(acx.window);
                    true
                }),
            );

            Vec::<AnyElement>::new()
        });

        let mut end_thumb_semantics = SemanticsProps::default();
        end_thumb_semantics.layout.position = PositionStyle::Absolute;
        end_thumb_semantics.layout.inset.left = Some(end_left);
        end_thumb_semantics.layout.inset.top = Some(Px(0.0));
        end_thumb_semantics.layout.size.width = Length::Px(handle_h);
        end_thumb_semantics.layout.size.height = Length::Px(handle_h);
        end_thumb_semantics.role = SemanticsRole::Slider;
        end_thumb_semantics.label = Some(end_label);
        end_thumb_semantics.test_id = end_test_id;
        end_thumb_semantics.focusable = enabled;
        end_thumb_semantics.disabled = !enabled;
        end_thumb_semantics.value = Some(end_value_text);

        let end_thumb = cx.semantics_with_id(end_thumb_semantics, |cx, semantics_id| {
            end_thumb_id = Some(semantics_id);

            let values_on_key = values.clone();
            let active_thumb_on_key = active_thumb_model.clone();
            cx.key_on_key_down_for(
                semantics_id,
                Arc::new(move |host, acx, down| {
                    if !enabled || down.repeat {
                        return false;
                    }
                    if down.modifiers.alt || down.modifiers.ctrl || down.modifiers.meta {
                        return false;
                    }

                    let _ = host.models_mut().update(&active_thumb_on_key, |v| *v = 1);

                    let delta = keyboard_step_delta(min, max, step);
                    let page_delta = keyboard_page_delta(min, max, step);
                    let (kind, delta) = match down.key {
                        KeyCode::ArrowLeft => (0, -delta * sign),
                        KeyCode::ArrowRight => (0, delta * sign),
                        KeyCode::ArrowDown => (0, -delta),
                        KeyCode::ArrowUp => (0, delta),
                        KeyCode::PageDown => (0, -page_delta),
                        KeyCode::PageUp => (0, page_delta),
                        KeyCode::Home => (1, 0.0),
                        KeyCode::End => (2, 0.0),
                        _ => return false,
                    };

                    let _ = host.models_mut().update(&values_on_key, |v| {
                        let clamped = clamp_range_pair(*v, min, max, step);
                        *v = clamped;
                        match kind {
                            0 => {
                                let next = v[1] + delta;
                                update_range_thumb(v, 1, next, min, max, step);
                            }
                            1 => update_range_thumb(v, 1, v[0], min, max, step),
                            2 => update_range_thumb(v, 1, max, min, max, step),
                            _ => {}
                        }
                    });

                    host.request_redraw(acx.window);
                    true
                }),
            );

            Vec::<AnyElement>::new()
        });

        let start_thumb_id = start_thumb_id.expect("range slider start thumb semantics id");
        let end_thumb_id = end_thumb_id.expect("range slider end thumb semantics id");

        let focus_visible = fret_ui::focus_visible::is_focus_visible(&mut *cx.app, Some(cx.window));
        let start_has_focus = enabled && cx.is_focused_element(start_thumb_id);
        let end_has_focus = enabled && cx.is_focused_element(end_thumb_id);
        let has_focus = start_has_focus || end_has_focus;
        let is_focus_visible = has_focus && focus_visible;

        let active_thumb = if start_has_focus {
            let _ = cx.app.models_mut().update(&active_thumb_model, |v| *v = 0);
            0usize
        } else if end_has_focus {
            let _ = cx.app.models_mut().update(&active_thumb_model, |v| *v = 1);
            1usize
        } else {
            cx.get_model_copied(&active_thumb_model, Invalidation::Paint)
                .unwrap_or(0)
                .min(1) as usize
        };

        let interaction_active = if enabled && dragging {
            slider_tokens::SliderInteraction::Pressed
        } else if is_focus_visible {
            slider_tokens::SliderInteraction::Focused
        } else if enabled && hovered {
            slider_tokens::SliderInteraction::Hovered
        } else {
            slider_tokens::SliderInteraction::None
        };

        let mut states = WidgetStates::default();
        states.set(WidgetState::Disabled, !enabled);
        states.set(WidgetState::Hovered, enabled && hovered);
        states.set(WidgetState::Active, enabled && dragging);
        states.set(WidgetState::Focused, has_focus);
        states.set(WidgetState::FocusVisible, is_focus_visible);

        let (
            active_track_color,
            inactive_track_color,
            handle_color_active,
            handle_color_inactive,
            state_layer_target,
            state_layer_color,
            state_layer_size,
            config,
            handle_w_active,
            handle_w_inactive,
        ) = {
            let theme = Theme::global(&*cx.app);

            let active_track_default =
                slider_tokens::active_track_color(theme, enabled, interaction_active);
            let inactive_track_default =
                slider_tokens::inactive_track_color(theme, enabled, interaction_active);

            let active_track_color = resolve_override_slot_with(
                style.active_track_color.as_ref(),
                states,
                |c| c.resolve(theme),
                || active_track_default,
            );
            let inactive_track_color = resolve_override_slot_with(
                style.inactive_track_color.as_ref(),
                states,
                |c| c.resolve(theme),
                || inactive_track_default,
            );

            let handle_color_active = resolve_override_slot_with(
                style.handle_color.as_ref(),
                states,
                |c| c.resolve(theme),
                || slider_tokens::handle_color(theme, enabled, interaction_active),
            );
            let handle_color_inactive = resolve_override_slot_with(
                style.handle_color.as_ref(),
                states,
                |c| c.resolve(theme),
                || {
                    slider_tokens::handle_color(
                        theme,
                        enabled,
                        slider_tokens::SliderInteraction::None,
                    )
                },
            );

            let state_layer_target = if enabled && dragging {
                slider_tokens::pressed_state_layer_opacity(theme)
            } else {
                slider_tokens::state_layer_target_opacity(theme, enabled, interaction_active)
            };
            let state_layer_default = slider_tokens::state_layer_color(theme, interaction_active);
            let state_layer_color = resolve_override_slot_with(
                style.state_layer_color.as_ref(),
                states,
                |c| c.resolve(theme),
                || state_layer_default,
            );
            let state_layer_size = slider_tokens::state_layer_size(theme);
            let config = material_pressable_indication_config(theme, None);

            let handle_w_active = slider_tokens::handle_width(theme, enabled, interaction_active);
            let handle_w_inactive =
                slider_tokens::handle_width(theme, enabled, slider_tokens::SliderInteraction::None);

            (
                active_track_color,
                inactive_track_color,
                handle_color_active,
                handle_color_inactive,
                state_layer_target,
                state_layer_color,
                state_layer_size,
                config,
                handle_w_active,
                handle_w_inactive,
            )
        };
        let now_frame = cx.frame_id.0;
        let target_start = if active_thumb == 0 {
            state_layer_target
        } else {
            0.0
        };
        let target_end = if active_thumb == 1 {
            state_layer_target
        } else {
            0.0
        };
        let (state_layer_opacity_start, state_layer_opacity_end, state_layer_want_frames) = cx
            .with_state(Runtime::default, |st| {
                if (target_start - st.state_layer_target_start).abs() > 1e-6 {
                    st.state_layer_target_start = target_start;
                    st.state_layer_start.set_target(
                        now_frame,
                        target_start,
                        config.state_duration_ms,
                        config.easing,
                    );
                }
                if (target_end - st.state_layer_target_end).abs() > 1e-6 {
                    st.state_layer_target_end = target_end;
                    st.state_layer_end.set_target(
                        now_frame,
                        target_end,
                        config.state_duration_ms,
                        config.easing,
                    );
                }

                st.state_layer_start.advance(now_frame);
                st.state_layer_end.advance(now_frame);

                (
                    st.state_layer_start.value(),
                    st.state_layer_end.value(),
                    st.state_layer_start.is_active() || st.state_layer_end.is_active(),
                )
            });
        let state_layer_opacity = if active_thumb == 0 {
            state_layer_opacity_start
        } else {
            state_layer_opacity_end
        };

        let values_on_pointer = values.clone();
        let dragging_on_pointer = dragging_model.clone();
        let hovered_on_pointer = hovered_model.clone();
        let active_thumb_on_pointer = active_thumb_model.clone();

        let values_for_update = values_on_pointer.clone();
        let active_thumb_for_update = active_thumb_on_pointer.clone();
        let update_from_pos =
            move |host: &mut dyn UiPointerActionHost, acx: ActionCx, pos: fret_core::Point| {
                let bounds = host.bounds();
                if bounds.size.width.0 <= 0.0 {
                    return;
                }

                let thumb = host
                    .models_mut()
                    .read(&active_thumb_for_update, |v| *v)
                    .ok()
                    .unwrap_or(0)
                    .min(1) as usize;

                let t_input = ((pos.x.0 - bounds.origin.x.0) / bounds.size.width.0).clamp(0.0, 1.0);
                let t_value = if rtl { 1.0 - t_input } else { t_input };
                let v_next = min + t_value * (max - min);
                let _ = host.models_mut().update(&values_for_update, |out| {
                    let clamped = clamp_range_pair(*out, min, max, step);
                    *out = clamped;
                    update_range_thumb(out, thumb, v_next, min, max, step);
                });
                host.request_redraw(acx.window);
            };

        let on_down = Arc::new({
            let update_from_pos = update_from_pos.clone();
            let dragging_on_pointer = dragging_on_pointer.clone();
            let hovered_on_pointer = hovered_on_pointer.clone();
            let values_on_pointer = values_on_pointer.clone();
            let active_thumb_on_pointer = active_thumb_on_pointer.clone();
            move |host: &mut dyn UiPointerActionHost, acx: ActionCx, down: PointerDownCx| {
                if !enabled || down.button != MouseButton::Left {
                    return false;
                }

                host.capture_pointer();
                host.set_cursor_icon(CursorIcon::Pointer);

                let bounds = host.bounds();
                let cur = host
                    .models_mut()
                    .read(&values_on_pointer, |v| *v)
                    .ok()
                    .unwrap_or([min, max]);
                let cur = clamp_range_pair(cur, min, max, step);
                let t0_visual = {
                    let t = value_to_t(cur[0], min, max);
                    if rtl { 1.0 - t } else { t }
                };
                let t1_visual = {
                    let t = value_to_t(cur[1], min, max);
                    if rtl { 1.0 - t } else { t }
                };
                let x0 = bounds.origin.x.0 + bounds.size.width.0 * t0_visual;
                let x1 = bounds.origin.x.0 + bounds.size.width.0 * t1_visual;
                let dx0 = (down.position.x.0 - x0).abs();
                let dx1 = (down.position.x.0 - x1).abs();
                let thumb: u8 = if dx0 <= dx1 { 0 } else { 1 };
                let _ = host
                    .models_mut()
                    .update(&active_thumb_on_pointer, |v| *v = thumb);

                if thumb == 0 {
                    host.request_focus(start_thumb_id);
                } else {
                    host.request_focus(end_thumb_id);
                }

                let _ = host
                    .models_mut()
                    .update(&dragging_on_pointer, |v| *v = true);
                let _ = host.models_mut().update(&hovered_on_pointer, |v| *v = true);
                update_from_pos(host, acx, down.position);
                true
            }
        });

        let on_move = Arc::new({
            let update_from_pos = update_from_pos.clone();
            let dragging_on_pointer = dragging_on_pointer.clone();
            let hovered_on_pointer = hovered_on_pointer.clone();
            let active_thumb_on_pointer = active_thumb_on_pointer.clone();
            let values_on_pointer = values_on_pointer.clone();
            move |host: &mut dyn UiPointerActionHost, acx: ActionCx, mv: PointerMoveCx| {
                if !enabled {
                    return false;
                }
                host.set_cursor_icon(CursorIcon::Pointer);
                let is_dragging = host
                    .models_mut()
                    .read(&dragging_on_pointer, |v| *v)
                    .ok()
                    .unwrap_or(false);
                let hovered = host.bounds().contains(mv.position);
                let _ = host
                    .models_mut()
                    .update(&hovered_on_pointer, |v| *v = hovered);
                if is_dragging {
                    update_from_pos(host, acx, mv.position);
                } else if hovered {
                    let bounds = host.bounds();
                    let cur = host
                        .models_mut()
                        .read(&values_on_pointer, |v| *v)
                        .ok()
                        .unwrap_or([min, max]);
                    let cur = clamp_range_pair(cur, min, max, step);
                    let t0_visual = {
                        let t = value_to_t(cur[0], min, max);
                        if rtl { 1.0 - t } else { t }
                    };
                    let t1_visual = {
                        let t = value_to_t(cur[1], min, max);
                        if rtl { 1.0 - t } else { t }
                    };
                    let x0 = bounds.origin.x.0 + bounds.size.width.0 * t0_visual;
                    let x1 = bounds.origin.x.0 + bounds.size.width.0 * t1_visual;
                    let dx0 = (mv.position.x.0 - x0).abs();
                    let dx1 = (mv.position.x.0 - x1).abs();
                    let thumb: u8 = if dx0 <= dx1 { 0 } else { 1 };
                    let _ = host
                        .models_mut()
                        .update(&active_thumb_on_pointer, |v| *v = thumb);
                }
                true
            }
        });

        let on_up = Arc::new({
            let dragging_on_pointer = dragging_on_pointer.clone();
            let hovered_on_pointer = hovered_on_pointer.clone();
            move |host: &mut dyn UiPointerActionHost, acx: ActionCx, up: PointerUpCx| {
                if !enabled {
                    return false;
                }
                let hovered = host.bounds().contains(up.position);
                let _ = host
                    .models_mut()
                    .update(&hovered_on_pointer, |v| *v = hovered);
                let _ = host
                    .models_mut()
                    .update(&dragging_on_pointer, |v| *v = false);
                host.release_pointer_capture();
                host.request_redraw(acx.window);
                true
            }
        });

        let (
            value_indicator_bottom_space,
            value_indicator_container_color,
            value_indicator_label_style,
            value_indicator_label_color,
            tick_size,
            tick_shape,
            tick_active_color,
            tick_active_opacity,
            tick_inactive_color,
            tick_inactive_opacity,
        ) = {
            let theme = Theme::global(&*cx.app);
            (
                slider_tokens::value_indicator_bottom_space(theme),
                slider_tokens::value_indicator_container_color(theme),
                slider_tokens::value_indicator_label_style(theme),
                slider_tokens::value_indicator_label_color(theme),
                slider_tokens::tick_mark_size(theme),
                slider_tokens::tick_mark_shape(theme),
                slider_tokens::tick_mark_color(theme, enabled, true),
                slider_tokens::tick_mark_opacity(theme, enabled, true),
                slider_tokens::tick_mark_color(theme, enabled, false),
                slider_tokens::tick_mark_opacity(theme, enabled, false),
            )
        };

        let mut pointer_props = PointerRegionProps::default();
        pointer_props.layout.size.width = Length::Fill;
        pointer_props.layout.size.height = Length::Fill;
        {
            let theme = Theme::global(&*cx.app);
            enforce_minimum_interactive_size(&mut pointer_props.layout, theme);
        }
        pointer_props.enabled = enabled;

        vec![cx.pointer_region(pointer_props, move |cx| {
            cx.pointer_region_on_pointer_down(on_down);
            cx.pointer_region_on_pointer_move(on_move);
            cx.pointer_region_on_pointer_up(on_up);

            let mut canvas_props = CanvasProps::default();
            canvas_props.layout.size.width = Length::Fill;
            canvas_props.layout.size.height = Length::Px(handle_h);

            let content = cx.canvas(canvas_props, move |p| {
                let bounds = p.bounds();
                if bounds.size.width.0 <= 0.0 || bounds.size.height.0 <= 0.0 {
                    return;
                }

                let track_h = Px(track_h.0.min(bounds.size.height.0));
                let track_y = Px(bounds.origin.y.0 + (bounds.size.height.0 - track_h.0) * 0.5);
                let thumb_y = Px(bounds.origin.y.0 + bounds.size.height.0 * 0.5);

                let x0_thumb = Px(bounds.origin.x.0 + bounds.size.width.0 * t0_visual);
                let x1_thumb = Px(bounds.origin.x.0 + bounds.size.width.0 * t1_visual);
                let x_left = Px(bounds.origin.x.0 + bounds.size.width.0 * t_left);
                let x_right = Px(bounds.origin.x.0 + bounds.size.width.0 * t_right);

                let left_w = (x_left.0 - bounds.origin.x.0).clamp(0.0, bounds.size.width.0);
                let active_w = (x_right.0 - x_left.0).clamp(0.0, bounds.size.width.0);
                let right_w =
                    (bounds.size.width.0 - left_w - active_w).clamp(0.0, bounds.size.width.0);

                if left_w > 0.0 {
                    let rect = Rect::new(
                        fret_core::Point::new(bounds.origin.x, track_y),
                        Size::new(Px(left_w), track_h),
                    );
                    let corners = if left_w + active_w + right_w <= left_w + 1e-6 {
                        track_shape
                    } else {
                        corners_zero_right(track_shape)
                    };
                    p.scene().push(fret_core::SceneOp::Quad {
                        order: DrawOrder(0),
                        rect,
                        background: fret_core::Paint::Solid(inactive_track_color),
                        border: Edges::all(Px(0.0)),
                        border_paint: fret_core::Paint::TRANSPARENT,
                        corner_radii: corners,
                    });
                }

                if right_w > 0.0 {
                    let rect = Rect::new(
                        fret_core::Point::new(Px(bounds.origin.x.0 + left_w + active_w), track_y),
                        Size::new(Px(right_w), track_h),
                    );
                    let corners = if left_w + active_w <= 1e-6 {
                        track_shape
                    } else {
                        corners_zero_left(track_shape)
                    };
                    p.scene().push(fret_core::SceneOp::Quad {
                        order: DrawOrder(0),
                        rect,
                        background: fret_core::Paint::Solid(inactive_track_color),
                        border: Edges::all(Px(0.0)),
                        border_paint: fret_core::Paint::TRANSPARENT,
                        corner_radii: corners,
                    });
                }

                if active_w > 0.0 {
                    let rect = Rect::new(
                        fret_core::Point::new(Px(bounds.origin.x.0 + left_w), track_y),
                        Size::new(Px(active_w), track_h),
                    );
                    let corners = Corners::all(Px(0.0));
                    p.scene().push(fret_core::SceneOp::Quad {
                        order: DrawOrder(0),
                        rect,
                        background: fret_core::Paint::Solid(active_track_color),
                        border: Edges::all(Px(0.0)),
                        border_paint: fret_core::Paint::TRANSPARENT,
                        corner_radii: corners,
                    });
                }

                if with_tick_marks {
                    let mut tick_count: Option<i32> = tick_marks_count
                        .map(|c| i32::from(c).clamp(0, 256))
                        .filter(|c| *c > 0);
                    if tick_count.is_none()
                        && step.is_finite()
                        && step > 0.0
                        && max.is_finite()
                        && min.is_finite()
                        && max > min
                    {
                        let approx_steps = ((max - min) / step).round();
                        let approx_steps = approx_steps.clamp(1.0, 200.0) as i32;
                        tick_count = Some(approx_steps + 1);
                    }
                    if let Some(tick_count) = tick_count {
                        let tick_count = tick_count.clamp(1, 201);
                        let center_y = Px(track_y.0 + track_h.0 * 0.5);
                        let y = Px((center_y.0 - tick_size.0 * 0.5).clamp(
                            bounds.origin.y.0,
                            bounds.origin.y.0 + bounds.size.height.0 - tick_size.0,
                        ));

                        for i in 0..tick_count {
                            let tick_t = if tick_count <= 1 {
                                0.0
                            } else {
                                (i as f32) / ((tick_count - 1) as f32)
                            };
                            let x = Px(bounds.origin.x.0 + bounds.size.width.0 * tick_t
                                - tick_size.0 * 0.5);
                            let x = Px(x.0.clamp(
                                bounds.origin.x.0,
                                bounds.origin.x.0 + bounds.size.width.0 - tick_size.0,
                            ));

                            let active = tick_t >= (t_left - 1e-6) && tick_t <= (t_right + 1e-6);
                            let (color, opacity) = if active {
                                (tick_active_color, tick_active_opacity)
                            } else {
                                (tick_inactive_color, tick_inactive_opacity)
                            };
                            let rect = Rect::new(
                                fret_core::Point::new(x, y),
                                Size::new(tick_size, tick_size),
                            );
                            p.scene().push(fret_core::SceneOp::Quad {
                                order: DrawOrder(1),
                                rect,
                                background: fret_core::Paint::Solid(alpha_mul(color, opacity)),
                                border: Edges::all(Px(0.0)),
                                border_paint: fret_core::Paint::TRANSPARENT,
                                corner_radii: tick_shape,
                            });
                        }
                    }
                }

                if state_layer_opacity > 0.0 {
                    let size = Px(state_layer_size.0.min(bounds.size.width.0).max(0.0));
                    if size.0 > 0.0 {
                        let thumb_x = if active_thumb == 0 {
                            x0_thumb
                        } else {
                            x1_thumb
                        };
                        let x = Px((thumb_x.0 - size.0 * 0.5).clamp(
                            bounds.origin.x.0,
                            bounds.origin.x.0 + bounds.size.width.0 - size.0,
                        ));
                        let y = Px((thumb_y.0 - size.0 * 0.5).clamp(
                            bounds.origin.y.0,
                            bounds.origin.y.0 + bounds.size.height.0 - size.0,
                        ));
                        let rect = Rect::new(fret_core::Point::new(x, y), Size::new(size, size));
                        p.scene().push(fret_core::SceneOp::Quad {
                            order: DrawOrder(2),
                            rect,
                            background: fret_core::Paint::Solid(alpha_mul(
                                state_layer_color,
                                state_layer_opacity,
                            )),
                            border: Edges::all(Px(0.0)),
                            border_paint: fret_core::Paint::TRANSPARENT,
                            corner_radii: Corners::all(Px(size.0 * 0.5)),
                        });
                    }
                }

                let mut draw_handle = |thumb_x: Px, color: Color, handle_w: Px| {
                    let handle_w = Px(handle_w.0.min(bounds.size.width.0).max(0.0));
                    let handle_h = Px(handle_h.0.min(bounds.size.height.0).max(0.0));
                    if handle_w.0 <= 0.0 || handle_h.0 <= 0.0 {
                        return;
                    }
                    let x = Px((thumb_x.0 - handle_w.0 * 0.5).clamp(
                        bounds.origin.x.0,
                        bounds.origin.x.0 + bounds.size.width.0 - handle_w.0,
                    ));
                    let y = Px(bounds.origin.y.0 + (bounds.size.height.0 - handle_h.0) * 0.5);
                    let rect =
                        Rect::new(fret_core::Point::new(x, y), Size::new(handle_w, handle_h));
                    p.scene().push(fret_core::SceneOp::Quad {
                        order: DrawOrder(3),
                        rect,
                        background: fret_core::Paint::Solid(color),
                        border: Edges::all(Px(0.0)),
                        border_paint: fret_core::Paint::TRANSPARENT,
                        corner_radii: handle_shape,
                    });
                };

                if active_thumb == 0 {
                    draw_handle(x0_thumb, handle_color_active, handle_w_active);
                    draw_handle(x1_thumb, handle_color_inactive, handle_w_inactive);
                } else {
                    draw_handle(x0_thumb, handle_color_inactive, handle_w_inactive);
                    draw_handle(x1_thumb, handle_color_active, handle_w_active);
                }

                if state_layer_want_frames {
                    p.request_animation_frame();
                }
            });

            let mut stack_props = StackProps::default();
            stack_props.layout.size.width = Length::Fill;
            stack_props.layout.size.height = Length::Fill;

            let show_value_indicator = enabled && dragging;
            let value_indicator = if show_value_indicator {
                let (active_t_value, active_t_visual) = if active_thumb == 0 {
                    (t0_value, t0_visual)
                } else {
                    (t1_value, t1_visual)
                };
                let percent = (active_t_value * 100.0).round().clamp(0.0, 100.0) as i32;
                let label = Arc::from(percent.to_string());

                let bubble_w = Px(48.0);
                let bubble_h = Px(28.0);
                let bottom_space = value_indicator_bottom_space;

                let last = cx.last_bounds_for_element(group_semantics_id);
                let (left, top) = if let Some(bounds) = last {
                    let x = (bounds.size.width.0 * active_t_visual).clamp(0.0, bounds.size.width.0);
                    let left =
                        Px((x - bubble_w.0 * 0.5).clamp(0.0, bounds.size.width.0 - bubble_w.0));
                    let top = Px(-bubble_h.0 - bottom_space.0);
                    (left, top)
                } else {
                    (Px(0.0), Px(-bubble_h.0 - bottom_space.0))
                };

                let mut container = ContainerProps::default();
                container.layout.position = PositionStyle::Absolute;
                container.layout.inset.left = Some(left);
                container.layout.inset.top = Some(top);
                container.layout.size.width = Length::Px(bubble_w);
                container.layout.size.height = Length::Px(bubble_h);
                container.layout.overflow = fret_ui::element::Overflow::Visible;
                container.background = Some(value_indicator_container_color);
                container.corner_radii = Corners::all(Px(9999.0));

                let mut row = RowProps::default();
                row.layout.size.width = Length::Fill;
                row.layout.size.height = Length::Fill;
                row.justify = MainAlign::Center;
                row.align = CrossAlign::Center;

                let text_style = value_indicator_label_style;
                let text_color = value_indicator_label_color;

                let mut text = TextProps::new(label);
                text.layout.size.width = Length::Fill;
                text.layout.size.height = Length::Fill;
                text.style = Some(text_style);
                text.color = Some(text_color);
                text.wrap = fret_core::TextWrap::None;
                text.overflow = fret_core::TextOverflow::Clip;

                Some(cx.container(container, |cx| {
                    vec![cx.row(row, |cx| vec![cx.text_props(text)])]
                }))
            } else {
                None
            };

            vec![cx.stack_props(stack_props, |_cx| {
                let mut out = vec![content, start_thumb, end_thumb];
                if let Some(indicator) = value_indicator {
                    out.push(indicator);
                }
                out
            })]
        })]
    })
}
