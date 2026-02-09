//! Material 3 text field (MVP).
//!
//! Outcome-oriented implementation:
//! - Token-driven chrome via `md.comp.(outlined|filled)-text-field.*`.
//! - Hover/focus/error/disabled outcomes via theme tokens (best-effort).

use std::cell::Cell;
use std::rc::Rc;
use std::sync::Arc;

use fret_core::{
    Axis, Color, Corners, Edges, NodeId, Point, Px, SemanticsRole, SvgFit, TextOverflow, TextStyle,
    TextWrap, Transform2D,
};
use fret_icons::IconId;
use fret_runtime::Model;
use fret_ui::action::{OnPressablePointerDown, PointerDownCx, PressablePointerDownResult};
use fret_ui::element::{
    AnyElement, ContainerProps, CrossAlign, FlexProps, HoverRegionProps, Length, MainAlign,
    Overflow, PointerRegionProps, PressableA11y, PressableProps, SvgIconProps, TextInputProps,
    TextProps, VisualTransformProps,
};
use fret_ui::elements::ElementContext;
use fret_ui::{GlobalElementId, Invalidation, Theme, UiHost};
use fret_ui_kit::{
    ColorRef, OverrideSlot, WidgetState, WidgetStateProperty, WidgetStates,
    resolve_override_slot_with,
};

use crate::foundation::floating_label;
use crate::foundation::icon::svg_source_for_icon;
use crate::foundation::indication::{
    RippleClip, material_ink_layer_for_pressable, material_pressable_indication_config,
};
use crate::foundation::motion_scheme::{MotionSchemeKey, sys_spring_in_scope};
use crate::motion::SpringAnimator;
use crate::tokens::autocomplete as autocomplete_tokens;
use crate::tokens::text_field as text_field_tokens;

#[derive(Debug, Clone, Copy, PartialEq, Eq, Default)]
pub enum TextFieldVariant {
    #[default]
    Outlined,
    Filled,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Default)]
pub(crate) enum TextFieldTokenNamespace {
    #[default]
    TextField,
    Autocomplete,
}

#[derive(Debug, Clone, Default)]
pub struct TextFieldStyle {
    pub container_background: OverrideSlot<ColorRef>,
    pub outline_color: OverrideSlot<ColorRef>,
    pub text_color: OverrideSlot<ColorRef>,
    pub placeholder_color: OverrideSlot<ColorRef>,
    pub caret_color: OverrideSlot<ColorRef>,
    pub label_color: OverrideSlot<ColorRef>,
    pub supporting_text_color: OverrideSlot<ColorRef>,
}

impl TextFieldStyle {
    pub fn container_background(
        mut self,
        background: WidgetStateProperty<Option<ColorRef>>,
    ) -> Self {
        self.container_background = Some(background);
        self
    }

    pub fn outline_color(mut self, color: WidgetStateProperty<Option<ColorRef>>) -> Self {
        self.outline_color = Some(color);
        self
    }

    pub fn text_color(mut self, color: WidgetStateProperty<Option<ColorRef>>) -> Self {
        self.text_color = Some(color);
        self
    }

    pub fn placeholder_color(mut self, color: WidgetStateProperty<Option<ColorRef>>) -> Self {
        self.placeholder_color = Some(color);
        self
    }

    pub fn caret_color(mut self, color: WidgetStateProperty<Option<ColorRef>>) -> Self {
        self.caret_color = Some(color);
        self
    }

    pub fn label_color(mut self, color: WidgetStateProperty<Option<ColorRef>>) -> Self {
        self.label_color = Some(color);
        self
    }

    pub fn supporting_text_color(mut self, color: WidgetStateProperty<Option<ColorRef>>) -> Self {
        self.supporting_text_color = Some(color);
        self
    }

    pub fn merged(mut self, other: Self) -> Self {
        if other.container_background.is_some() {
            self.container_background = other.container_background;
        }
        if other.outline_color.is_some() {
            self.outline_color = other.outline_color;
        }
        if other.text_color.is_some() {
            self.text_color = other.text_color;
        }
        if other.placeholder_color.is_some() {
            self.placeholder_color = other.placeholder_color;
        }
        if other.caret_color.is_some() {
            self.caret_color = other.caret_color;
        }
        if other.label_color.is_some() {
            self.label_color = other.label_color;
        }
        if other.supporting_text_color.is_some() {
            self.supporting_text_color = other.supporting_text_color;
        }
        self
    }
}

#[derive(Debug, Default)]
struct TextFieldRuntime {
    float_target: bool,
    float: SpringAnimator,
    last_phase: TextFieldInputPhase,
    placeholder_opacity: SpringAnimator,
    border_top: SpringAnimator,
    border_right: SpringAnimator,
    border_bottom: SpringAnimator,
    border_left: SpringAnimator,
    border_color: AnimatedColor,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Default)]
enum TextFieldInputPhase {
    Focused,
    #[default]
    UnfocusedEmpty,
    UnfocusedNotEmpty,
}

#[derive(Debug, Default)]
struct AnimatedColor {
    r: SpringAnimator,
    g: SpringAnimator,
    b: SpringAnimator,
    a: SpringAnimator,
}

impl AnimatedColor {
    fn reset(&mut self, now_frame: u64, color: Color) {
        self.r.reset(now_frame, color.r);
        self.g.reset(now_frame, color.g);
        self.b.reset(now_frame, color.b);
        self.a.reset(now_frame, color.a);
    }

    fn set_target(&mut self, now_frame: u64, color: Color, spec: crate::motion::SpringSpec) {
        self.r.set_target(now_frame, color.r, spec);
        self.g.set_target(now_frame, color.g, spec);
        self.b.set_target(now_frame, color.b, spec);
        self.a.set_target(now_frame, color.a, spec);
    }

    fn advance(&mut self, now_frame: u64) {
        self.r.advance(now_frame);
        self.g.advance(now_frame);
        self.b.advance(now_frame);
        self.a.advance(now_frame);
    }

    fn is_active(&self) -> bool {
        self.r.is_active() || self.g.is_active() || self.b.is_active() || self.a.is_active()
    }

    fn value(&self) -> Color {
        Color {
            r: self.r.value().clamp(0.0, 1.0),
            g: self.g.value().clamp(0.0, 1.0),
            b: self.b.value().clamp(0.0, 1.0),
            a: self.a.value().clamp(0.0, 1.0),
        }
    }
}

#[derive(Clone)]
pub struct TextField {
    model: Model<String>,
    variant: TextFieldVariant,
    label: Option<Arc<str>>,
    placeholder: Option<Arc<str>>,
    supporting_text: Option<Arc<str>>,
    style: TextFieldStyle,
    field_id_out: Option<Rc<Cell<Option<GlobalElementId>>>>,
    trailing_icon: Option<IconId>,
    trailing_icon_a11y_label: Option<Arc<str>>,
    trailing_icon_test_id: Option<Arc<str>>,
    trailing_icon_rotation_progress: Option<f32>,
    on_trailing_icon_pointer_down: Option<OnPressablePointerDown>,
    disabled: bool,
    error: bool,
    a11y_label: Option<Arc<str>>,
    test_id: Option<Arc<str>>,
    a11y_role: Option<SemanticsRole>,
    active_descendant: Option<NodeId>,
    controls_element: Option<u64>,
    expanded: Option<bool>,
    input_id_out: Option<Rc<Cell<Option<GlobalElementId>>>>,
    token_namespace: TextFieldTokenNamespace,
}

impl std::fmt::Debug for TextField {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.debug_struct("TextField")
            .field("variant", &self.variant)
            .field("label", &self.label)
            .field("placeholder", &self.placeholder)
            .field("supporting_text", &self.supporting_text)
            .field("style", &self.style)
            .field("disabled", &self.disabled)
            .field("error", &self.error)
            .field("a11y_label", &self.a11y_label)
            .field("test_id", &self.test_id)
            .field("a11y_role", &self.a11y_role)
            .field("token_namespace", &self.token_namespace)
            .finish()
    }
}

impl TextField {
    pub fn new(model: Model<String>) -> Self {
        Self {
            model,
            variant: TextFieldVariant::default(),
            label: None,
            placeholder: None,
            supporting_text: None,
            style: TextFieldStyle::default(),
            field_id_out: None,
            trailing_icon: None,
            trailing_icon_a11y_label: None,
            trailing_icon_test_id: None,
            trailing_icon_rotation_progress: None,
            on_trailing_icon_pointer_down: None,
            disabled: false,
            error: false,
            a11y_label: None,
            test_id: None,
            a11y_role: None,
            active_descendant: None,
            controls_element: None,
            expanded: None,
            input_id_out: None,
            token_namespace: TextFieldTokenNamespace::TextField,
        }
    }

    pub fn variant(mut self, variant: TextFieldVariant) -> Self {
        self.variant = variant;
        self
    }

    pub(crate) fn token_namespace(mut self, namespace: TextFieldTokenNamespace) -> Self {
        self.token_namespace = namespace;
        self
    }

    pub fn label(mut self, label: impl Into<Arc<str>>) -> Self {
        self.label = Some(label.into());
        self
    }

    pub fn placeholder(mut self, placeholder: impl Into<Arc<str>>) -> Self {
        self.placeholder = Some(placeholder.into());
        self
    }

    pub fn supporting_text(mut self, text: impl Into<Arc<str>>) -> Self {
        self.supporting_text = Some(text.into());
        self
    }

    pub fn style(mut self, style: TextFieldStyle) -> Self {
        self.style = self.style.merged(style);
        self
    }

    pub fn trailing_icon(mut self, icon: IconId) -> Self {
        self.trailing_icon = Some(icon);
        self
    }

    pub fn trailing_icon_a11y_label(mut self, label: impl Into<Arc<str>>) -> Self {
        self.trailing_icon_a11y_label = Some(label.into());
        self
    }

    pub fn trailing_icon_test_id(mut self, id: impl Into<Arc<str>>) -> Self {
        self.trailing_icon_test_id = Some(id.into());
        self
    }

    pub fn trailing_icon_rotation_progress(mut self, progress: f32) -> Self {
        self.trailing_icon_rotation_progress = Some(progress);
        self
    }

    pub fn on_trailing_icon_pointer_down(
        mut self,
        on_pointer_down: OnPressablePointerDown,
    ) -> Self {
        self.on_trailing_icon_pointer_down = Some(on_pointer_down);
        self
    }

    pub fn disabled(mut self, disabled: bool) -> Self {
        self.disabled = disabled;
        self
    }

    pub fn error(mut self, error: bool) -> Self {
        self.error = error;
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

    pub fn a11y_role(mut self, role: SemanticsRole) -> Self {
        self.a11y_role = Some(role);
        self
    }

    pub(crate) fn active_descendant(mut self, node: Option<NodeId>) -> Self {
        self.active_descendant = node;
        self
    }

    pub(crate) fn controls_element(mut self, element: Option<u64>) -> Self {
        self.controls_element = element;
        self
    }

    pub(crate) fn expanded(mut self, expanded: Option<bool>) -> Self {
        self.expanded = expanded;
        self
    }

    pub(crate) fn input_id_out(mut self, out: Rc<Cell<Option<GlobalElementId>>>) -> Self {
        self.input_id_out = Some(out);
        self
    }

    pub(crate) fn field_id_out(mut self, out: Rc<Cell<Option<GlobalElementId>>>) -> Self {
        self.field_id_out = Some(out);
        self
    }

    pub fn into_element<H: UiHost>(self, cx: &mut ElementContext<'_, H>) -> AnyElement {
        cx.scope(|cx| {
            let variant = self.variant;
            let token_namespace = self.token_namespace;
            let height = {
                let theme = Theme::global(&*cx.app);
                match token_namespace {
                    TextFieldTokenNamespace::TextField => {
                        text_field_tokens::container_height(theme, variant)
                    }
                    TextFieldTokenNamespace::Autocomplete => {
                        autocomplete_tokens::text_field_container_height(theme, variant)
                    }
                }
            };

            let mut hover_layout = fret_ui::element::LayoutStyle::default();
            hover_layout.size.width = Length::Fill;
            hover_layout.overflow = Overflow::Visible;
            let hover = HoverRegionProps {
                layout: hover_layout,
            };

            let model = self.model.clone();
            let variant_for_children = variant;
            let label = self.label.clone();
            let placeholder = self.placeholder.clone();
            let supporting_text = self.supporting_text.clone();
            let style_override = self.style;
            let disabled = self.disabled;
            let error = self.error;
            let a11y_role = self.a11y_role;
            let active_descendant = self.active_descendant;
            let controls_element = self.controls_element;
            let expanded = self.expanded;
            let a11y_label = self
                .a11y_label
                .clone()
                .or_else(|| label.clone())
                .or_else(|| placeholder.clone());
            let test_id = self.test_id.clone();
            let trailing_icon = self.trailing_icon;
            let trailing_icon_a11y_label = self.trailing_icon_a11y_label.clone();
            let trailing_icon_test_id = self.trailing_icon_test_id.clone();
            let trailing_icon_rotation_progress = self.trailing_icon_rotation_progress;
            let on_trailing_icon_pointer_down = self.on_trailing_icon_pointer_down.clone();
            let input_id_out = self.input_id_out.clone();
            let field_id_out = self.field_id_out.clone();

            cx.hover_region(hover, move |cx, hovered| {
                let mut focused = false;
                let mut input_id = GlobalElementId(0);
                let mut states = WidgetStates::empty();
                let mut input_bg = {
                    let theme = Theme::global(&*cx.app);
                    theme
                        .color_by_key("md.sys.color.surface")
                        .unwrap_or_else(|| theme.color_required("md.sys.color.surface"))
                };
                let mut outline_width_for_notch = Px(0.0);
                vec![cx.flex(
                    FlexProps {
                        layout: {
                            let mut layout = fret_ui::element::LayoutStyle::default();
                            layout.size.width = Length::Fill;
                            layout.overflow = Overflow::Visible;
                            layout
                        },
                        direction: fret_core::Axis::Vertical,
                        gap: Px(4.0),
                        padding: Edges::all(Px(0.0)),
                        justify: MainAlign::Start,
                        align: fret_ui::element::CrossAlign::Start,
                        wrap: false,
                    },
                    move |cx| {
                        let mut children: Vec<AnyElement> = Vec::new();
                        let mut float_progress = 0.0f32;

                        let input = cx.named("text_input", |cx| {
                            let populated = cx
                                .get_model_cloned(&model, Invalidation::Layout)
                                .map(|v| !v.is_empty())
                                .unwrap_or(false);

                            let mut container = ContainerProps::default();
                            container.layout.size.width = Length::Fill;
                            container.layout.size.height = Length::Px(height);
                            container.layout.overflow = Overflow::Clip;

                            let state_layer = (hovered && !disabled)
                                .then(|| {
                                    let theme = Theme::global(&*cx.app);
                                    text_field_tokens::hover_state_layer(
                                        theme,
                                        variant_for_children,
                                        error,
                                    )
                                })
                                .flatten()
                                .map(|(color, opacity)| {
                                    let mut out = color;
                                    out.a = (out.a * opacity).clamp(0.0, 1.0);
                                    out
                                })
                                .filter(|c| c.a > 0.0);

                            let text_input = cx.text_input_with_id_props(|cx, id| {
                                input_id = id;
                                focused = cx.is_focused_element(id);
                                states = text_field_widget_states(cx, hovered, focused, disabled);

                                let (mut chrome, spatial, fast_effects, slow_effects) = {
                                    let theme = Theme::global(&*cx.app);
                                    let mut chrome = match token_namespace {
                                        TextFieldTokenNamespace::TextField => {
                                            text_field_tokens::text_input_style(
                                                theme,
                                                variant_for_children,
                                                focused,
                                                hovered,
                                                disabled,
                                                error,
                                            )
                                        }
                                        TextFieldTokenNamespace::Autocomplete => {
                                            autocomplete_tokens::text_input_style(
                                                theme,
                                                variant_for_children,
                                                focused,
                                                hovered,
                                                disabled,
                                                error,
                                            )
                                        }
                                    };
                                    apply_text_field_input_overrides(
                                        theme,
                                        states,
                                        &style_override,
                                        &mut chrome,
                                    );

                                    let spatial = sys_spring_in_scope(
                                        &*cx,
                                        theme,
                                        MotionSchemeKey::FastSpatial,
                                    );
                                    let fast_effects = sys_spring_in_scope(
                                        &*cx,
                                        theme,
                                        MotionSchemeKey::FastEffects,
                                    );
                                    let slow_effects = sys_spring_in_scope(
                                        &*cx,
                                        theme,
                                        MotionSchemeKey::SlowEffects,
                                    );

                                    (chrome, spatial, fast_effects, slow_effects)
                                };

                                let trailing_icon_hit_width = if trailing_icon.is_some() {
                                    Px(48.0)
                                } else {
                                    Px(0.0)
                                };
                                if trailing_icon_hit_width.0 > 0.0 {
                                    chrome.padding.right =
                                        Px(chrome.padding.right.0.max(trailing_icon_hit_width.0));
                                }

                                let should_float = focused || populated;
                                let input_phase = if focused {
                                    TextFieldInputPhase::Focused
                                } else if populated {
                                    TextFieldInputPhase::UnfocusedNotEmpty
                                } else {
                                    TextFieldInputPhase::UnfocusedEmpty
                                };

                                let placeholder_target_opacity = if label.is_some() {
                                    if focused && !populated { 1.0 } else { 0.0 }
                                } else {
                                    1.0
                                };

                                let now_frame = cx.frame_id.0;

                                let target_border = chrome.border;
                                let target_border_color = chrome.border_color;

                                let (
                                    want_frames,
                                    next_float_progress,
                                    animated_border,
                                    animated_border_color,
                                    placeholder_opacity,
                                ) = cx.with_state(TextFieldRuntime::default, |rt| {
                                    if disabled {
                                        rt.float_target = should_float;
                                        rt.float
                                            .reset(now_frame, if should_float { 1.0 } else { 0.0 });
                                        rt.last_phase = input_phase;
                                        rt.placeholder_opacity
                                            .reset(now_frame, placeholder_target_opacity);
                                        rt.border_top.reset(now_frame, target_border.top.0);
                                        rt.border_right.reset(now_frame, target_border.right.0);
                                        rt.border_bottom.reset(now_frame, target_border.bottom.0);
                                        rt.border_left.reset(now_frame, target_border.left.0);
                                        rt.border_color.reset(now_frame, target_border_color);

                                        return (
                                            false,
                                            rt.float.value(),
                                            target_border,
                                            target_border_color,
                                            rt.placeholder_opacity.value(),
                                        );
                                    }

                                    if rt.float_target != should_float {
                                        rt.float_target = should_float;
                                        rt.float.set_target(
                                            now_frame,
                                            if should_float { 1.0 } else { 0.0 },
                                            spatial,
                                        );
                                    }

                                    let placeholder_effects = match (rt.last_phase, input_phase) {
                                        (
                                            TextFieldInputPhase::Focused,
                                            TextFieldInputPhase::UnfocusedEmpty,
                                        ) => fast_effects,
                                        (
                                            TextFieldInputPhase::UnfocusedEmpty,
                                            TextFieldInputPhase::Focused,
                                        )
                                        | (
                                            TextFieldInputPhase::UnfocusedNotEmpty,
                                            TextFieldInputPhase::UnfocusedEmpty,
                                        ) => slow_effects,
                                        _ => fast_effects,
                                    };
                                    rt.last_phase = input_phase;

                                    rt.placeholder_opacity.set_target(
                                        now_frame,
                                        placeholder_target_opacity,
                                        placeholder_effects,
                                    );

                                    rt.border_top.set_target(
                                        now_frame,
                                        target_border.top.0,
                                        spatial,
                                    );
                                    rt.border_right.set_target(
                                        now_frame,
                                        target_border.right.0,
                                        spatial,
                                    );
                                    rt.border_bottom.set_target(
                                        now_frame,
                                        target_border.bottom.0,
                                        spatial,
                                    );
                                    rt.border_left.set_target(
                                        now_frame,
                                        target_border.left.0,
                                        spatial,
                                    );

                                    rt.border_color.set_target(
                                        now_frame,
                                        target_border_color,
                                        fast_effects,
                                    );

                                    rt.float.advance(now_frame);
                                    rt.placeholder_opacity.advance(now_frame);
                                    rt.border_top.advance(now_frame);
                                    rt.border_right.advance(now_frame);
                                    rt.border_bottom.advance(now_frame);
                                    rt.border_left.advance(now_frame);
                                    rt.border_color.advance(now_frame);

                                    let want_frames = rt.float.is_active()
                                        || rt.placeholder_opacity.is_active()
                                        || rt.border_top.is_active()
                                        || rt.border_right.is_active()
                                        || rt.border_bottom.is_active()
                                        || rt.border_left.is_active()
                                        || rt.border_color.is_active();

                                    (
                                        want_frames,
                                        rt.float.value(),
                                        Edges {
                                            top: Px(rt.border_top.value().max(0.0)),
                                            right: Px(rt.border_right.value().max(0.0)),
                                            bottom: Px(rt.border_bottom.value().max(0.0)),
                                            left: Px(rt.border_left.value().max(0.0)),
                                        },
                                        rt.border_color.value(),
                                        rt.placeholder_opacity.value(),
                                    )
                                });

                                float_progress = next_float_progress.clamp(0.0, 1.0);

                                if want_frames {
                                    cx.request_animation_frame();
                                }

                                input_bg = chrome.background;
                                outline_width_for_notch = match variant_for_children {
                                    TextFieldVariant::Outlined => animated_border.top,
                                    TextFieldVariant::Filled => Px(0.0),
                                };

                                container.background =
                                    (chrome.background.a > 0.0).then_some(chrome.background);
                                container.corner_radii = chrome.corner_radii;
                                container.border = animated_border;
                                container.border_color = Some(animated_border_color);

                                chrome.background = Color::TRANSPARENT;
                                chrome.border = Edges::all(Px(0.0));
                                chrome.border_color = Color::TRANSPARENT;
                                chrome.border_color_focused = Color::TRANSPARENT;

                                chrome.placeholder_color = alpha_mul(
                                    chrome.placeholder_color,
                                    placeholder_opacity.clamp(0.0, 1.0),
                                );

                                let mut props = TextInputProps::new(model.clone());
                                props.layout.size.width = Length::Fill;
                                props.layout.size.height = Length::Fill;
                                props.a11y_label = a11y_label.clone();
                                props.a11y_role =
                                    Some(a11y_role.unwrap_or(SemanticsRole::TextField));
                                props.test_id = test_id.clone();
                                props.placeholder = placeholder.clone();
                                props.active_descendant = active_descendant;
                                props.controls_element = controls_element;
                                props.expanded = expanded;
                                props.chrome = chrome;
                                props.text_style =
                                    crate::foundation::context::inherited_text_style(cx)
                                        .unwrap_or_else(|| {
                                            let theme = Theme::global(&*cx.app);
                                            theme
                                                .text_style_by_key("md.sys.typescale.body-large")
                                                .unwrap_or(TextStyle::default())
                                        });

                                props
                            });
                            if let Some(out) = input_id_out.as_ref() {
                                out.set(Some(input_id));
                            }

                            // Keep subtree shape stable across hover transitions (ADR 0181).
                            // We always include the overlay node, but only paint when `state_layer`
                            // is present.
                            let overlay = {
                                let mut overlay_layout = fret_ui::element::LayoutStyle::default();
                                overlay_layout.position = fret_ui::element::PositionStyle::Absolute;
                                overlay_layout.inset.top = Some(Px(0.0));
                                overlay_layout.inset.right = Some(Px(0.0));
                                overlay_layout.inset.bottom = Some(Px(0.0));
                                overlay_layout.inset.left = Some(Px(0.0));

                                let mut overlay = ContainerProps::default();
                                overlay.layout = overlay_layout;
                                overlay.background = state_layer;
                                overlay.corner_radii = container.corner_radii;
                                cx.container(overlay, |_cx| Vec::new())
                            };

                            let trailing_icon_el =
                                trailing_icon.map(|icon| {
                                    let (
                                        size,
                                        color,
                                        opacity,
                                        hover_opacity,
                                        pressed_opacity,
                                        config,
                                    ) = {
                                        let theme = Theme::global(&*cx.app);
                                        let size = match token_namespace {
                                            TextFieldTokenNamespace::TextField => {
                                                text_field_tokens::trailing_icon_size(
                                                    theme,
                                                    variant_for_children,
                                                )
                                            }
                                            TextFieldTokenNamespace::Autocomplete => {
                                                autocomplete_tokens::trailing_icon_size(
                                                    theme,
                                                    variant_for_children,
                                                )
                                            }
                                        };
                                        let (color, opacity) = match token_namespace {
                                            TextFieldTokenNamespace::TextField => {
                                                text_field_tokens::trailing_icon_color(
                                                    theme,
                                                    variant_for_children,
                                                    hovered,
                                                    disabled,
                                                    error,
                                                    focused,
                                                )
                                            }
                                            TextFieldTokenNamespace::Autocomplete => {
                                                autocomplete_tokens::trailing_icon_color(
                                                    theme,
                                                    variant_for_children,
                                                    hovered,
                                                    disabled,
                                                    error,
                                                    focused,
                                                )
                                            }
                                        };

                                        let hover_opacity = theme
                                            .number_by_key("md.sys.state.hover.state-layer-opacity")
                                            .unwrap_or(0.08);
                                        let pressed_opacity = theme
                                            .number_by_key(
                                                "md.sys.state.pressed.state-layer-opacity",
                                            )
                                            .unwrap_or(0.1);
                                        let config =
                                            material_pressable_indication_config(theme, None);

                                        (
                                            size,
                                            color,
                                            opacity,
                                            hover_opacity,
                                            pressed_opacity,
                                            config,
                                        )
                                    };

                                    let svg = svg_source_for_icon(cx, &icon);
                                    let mut icon_props = SvgIconProps::new(svg);
                                    icon_props.fit = SvgFit::Contain;
                                    icon_props.color = color;
                                    icon_props.opacity = opacity;
                                    icon_props.layout.size.width = Length::Px(size);
                                    icon_props.layout.size.height = Length::Px(size);
                                    let icon_el = cx.svg_icon_props(icon_props);

                                    let icon_el = if let Some(progress) =
                                        trailing_icon_rotation_progress
                                    {
                                        let degrees = 180.0 * progress.clamp(0.0, 1.0);
                                        let mut layout = fret_ui::element::LayoutStyle::default();
                                        layout.size.width = Length::Px(size);
                                        layout.size.height = Length::Px(size);
                                        cx.visual_transform_props(
                                            VisualTransformProps {
                                                layout,
                                                transform: Transform2D::rotation_about_degrees(
                                                    degrees,
                                                    Point::new(Px(size.0 * 0.5), Px(size.0 * 0.5)),
                                                ),
                                            },
                                            move |_cx| vec![icon_el],
                                        )
                                    } else {
                                        icon_el
                                    };

                                    let icon_test_id =
                                        trailing_icon_test_id.clone().or_else(|| {
                                            test_id.as_ref().map(|id| {
                                                Arc::<str>::from(format!("{id}-trailing-icon"))
                                            })
                                        });

                                    let icon_a11y_label = trailing_icon_a11y_label.clone();

                                    let input_id_for_focus = input_id;
                                    let handler = on_trailing_icon_pointer_down.clone();
                                    let enabled = !disabled;
                                    let ripple_base_opacity = pressed_opacity;
                                    let corner_radii = Corners::all(Px(24.0));
                                    let state_layer_color = alpha_mul(color, opacity);

                                    let mut layout = fret_ui::element::LayoutStyle::default();
                                    layout.position = fret_ui::element::PositionStyle::Absolute;
                                    layout.inset.top = Some(Px(0.0));
                                    layout.inset.right = Some(Px(0.0));
                                    layout.inset.bottom = Some(Px(0.0));
                                    layout.size.width = Length::Px(Px(48.0));
                                    layout.size.height = Length::Fill;

                                    cx.pressable(
                                        PressableProps {
                                            layout,
                                            enabled,
                                            focusable: false,
                                            a11y: PressableA11y {
                                                role: Some(SemanticsRole::Button),
                                                label: icon_a11y_label,
                                                test_id: icon_test_id,
                                                ..Default::default()
                                            },
                                            ..Default::default()
                                        },
                                        move |cx, state| {
                                            if enabled {
                                                cx.pressable_on_pointer_down(Arc::new(
                                                    move |host, action_cx, down: PointerDownCx| {
                                                        host.request_focus(input_id_for_focus);
                                                        if let Some(h) = handler.clone() {
                                                            return h(host, action_cx, down);
                                                        }
                                                        PressablePointerDownResult::Continue
                                                    },
                                                ));
                                            }

                                            let pressable_id = cx.root_id();
                                            let now_frame = cx.frame_id.0;

                                            let mut props = PointerRegionProps::default();
                                            props.enabled = enabled;
                                            props.layout.size.width = Length::Fill;
                                            props.layout.size.height = Length::Fill;

                                            vec![cx.pointer_region(props, move |cx| {
                                                cx.pointer_region_on_pointer_down(Arc::new(
                                                    |_host, _cx, _down| false,
                                                ));

                                                let pressed = enabled && state.pressed;
                                                let hovered = enabled && state.hovered;
                                                let state_layer_target = if pressed {
                                                    pressed_opacity
                                                } else if hovered {
                                                    hover_opacity
                                                } else {
                                                    0.0
                                                };

                                                let overlay = material_ink_layer_for_pressable(
                                                    cx,
                                                    pressable_id,
                                                    now_frame,
                                                    corner_radii,
                                                    RippleClip::Bounded,
                                                    state_layer_color,
                                                    pressed,
                                                    state_layer_target,
                                                    ripple_base_opacity,
                                                    config,
                                                    false,
                                                );

                                                let mut row = FlexProps::default();
                                                row.direction = Axis::Horizontal;
                                                row.justify = MainAlign::Center;
                                                row.align = CrossAlign::Center;
                                                row.layout.size.width = Length::Fill;
                                                row.layout.size.height = Length::Fill;
                                                vec![
                                                    overlay,
                                                    cx.flex(row, move |_cx| vec![icon_el]),
                                                ]
                                            })]
                                        },
                                    )
                                });

                            cx.container(container, move |cx| {
                                if let Some(out) = field_id_out.as_ref() {
                                    out.set(Some(cx.root_id()));
                                }
                                let mut out = vec![overlay, text_input];
                                if let Some(icon) = trailing_icon_el {
                                    out.push(icon);
                                }
                                out
                            })
                        });

                        children.push(input);

                        if let Some(label) = label.as_ref() {
                            children.push(text_field_label(
                                cx,
                                variant_for_children,
                                label.clone(),
                                float_progress,
                                states,
                                &style_override,
                                hovered,
                                disabled,
                                error,
                                focused,
                                input_id,
                                input_bg,
                                outline_width_for_notch,
                            ));
                        }

                        if let Some(text) = supporting_text.as_ref() {
                            children.push(text_field_supporting_text(
                                cx,
                                variant_for_children,
                                text.clone(),
                                states,
                                &style_override,
                                hovered,
                                disabled,
                                error,
                                focused,
                            ));
                        }

                        children
                    },
                )]
            })
        })
    }
}

fn text_field_label<H: UiHost>(
    cx: &mut ElementContext<'_, H>,
    variant: TextFieldVariant,
    text: Arc<str>,
    progress: f32,
    states: WidgetStates,
    style_override: &TextFieldStyle,
    hovered: bool,
    disabled: bool,
    error: bool,
    focused: bool,
    input_id: GlobalElementId,
    input_bg: Color,
    outline_width: Px,
) -> AnyElement {
    let (style, color) = {
        let theme = Theme::global(&*cx.app);
        let style = floating_label::material_floating_label_text_style(theme, progress)
            .or_else(|| theme.text_style_by_key("md.sys.typescale.body-large"));

        let color = resolve_override_slot_with(
            style_override.label_color.as_ref(),
            states,
            |color| color.resolve(theme),
            || text_field_tokens::label_color(theme, variant, hovered, disabled, error, focused),
        );

        (style, color)
    };

    let (x, y) = floating_label::material_floating_label_offsets(progress);

    let mut layout = fret_ui::element::LayoutStyle::default();
    layout.position = fret_ui::element::PositionStyle::Absolute;
    layout.inset.top = Some(y);
    layout.inset.left = Some(x);
    layout.inset.right = Some(Px(16.0));
    layout.overflow = Overflow::Visible;

    let floated = floating_label::is_floated(progress);

    let mut patch = ContainerProps::default();
    if variant == TextFieldVariant::Outlined {
        let patch_padding_x = Px(4.0);
        let patch_padding_y = Px((outline_width.0 + 1.0).max(0.0));
        patch.padding = if floated {
            Edges {
                top: patch_padding_y,
                right: patch_padding_x,
                bottom: patch_padding_y,
                left: patch_padding_x,
            }
        } else {
            Edges::all(Px(0.0))
        };
        patch.background = floated.then_some(input_bg);
    }

    cx.pointer_region(
        PointerRegionProps {
            layout,
            enabled: !disabled,
        },
        move |cx| {
            let input_for_focus = input_id;
            cx.pointer_region_on_pointer_down(Arc::new(move |host, _cx, _down| {
                host.request_focus(input_for_focus);
                true
            }));

            vec![cx.container(patch, move |cx| {
                vec![cx.text_props(TextProps {
                    layout: fret_ui::element::LayoutStyle::default(),
                    text: text.clone(),
                    style,
                    color: Some(color),
                    wrap: TextWrap::None,
                    overflow: TextOverflow::Clip,
                })]
            })]
        },
    )
}

fn text_field_supporting_text<H: UiHost>(
    cx: &mut ElementContext<'_, H>,
    variant: TextFieldVariant,
    text: Arc<str>,
    states: WidgetStates,
    style_override: &TextFieldStyle,
    hovered: bool,
    disabled: bool,
    error: bool,
    focused: bool,
) -> AnyElement {
    let (style, color) = {
        let theme = Theme::global(&*cx.app);
        let style = theme.text_style_by_key("md.sys.typescale.body-small");
        let color = resolve_override_slot_with(
            style_override.supporting_text_color.as_ref(),
            states,
            |color| color.resolve(theme),
            || {
                text_field_tokens::supporting_text_color(
                    theme, variant, hovered, disabled, error, focused,
                )
            },
        );

        (style, color)
    };

    let mut layout = fret_ui::element::LayoutStyle::default();
    layout.margin.left = fret_ui::element::MarginEdge::Px(Px(16.0));
    layout.margin.right = fret_ui::element::MarginEdge::Px(Px(16.0));

    cx.text_props(TextProps {
        layout,
        text,
        style,
        color: Some(color),
        wrap: TextWrap::Word,
        overflow: TextOverflow::Clip,
    })
}

fn text_field_widget_states<H: UiHost>(
    cx: &mut ElementContext<'_, H>,
    hovered: bool,
    focused: bool,
    disabled: bool,
) -> WidgetStates {
    let mut states = WidgetStates::empty();
    states.set(WidgetState::Disabled, disabled);
    states.set(WidgetState::Hovered, hovered && !disabled);
    states.set(WidgetState::Focused, focused && !disabled);
    states.set(
        WidgetState::FocusVisible,
        focused && !disabled && fret_ui::focus_visible::is_focus_visible(cx.app, Some(cx.window)),
    );
    states
}

fn alpha_mul(mut color: Color, opacity: f32) -> Color {
    color.a = (color.a * opacity).clamp(0.0, 1.0);
    color
}

fn apply_text_field_input_overrides(
    theme: &Theme,
    states: WidgetStates,
    style_override: &TextFieldStyle,
    chrome: &mut fret_ui::TextInputStyle,
) {
    if let Some(background) = style_override
        .container_background
        .as_ref()
        .and_then(|slot| slot.resolve(states).as_ref())
    {
        chrome.background = background.resolve(theme);
    }

    if let Some(outline) = style_override
        .outline_color
        .as_ref()
        .and_then(|slot| slot.resolve(states).as_ref())
    {
        let outline = outline.resolve(theme);
        chrome.border_color = outline;
        chrome.border_color_focused = outline;
    }

    if let Some(text_color) = style_override
        .text_color
        .as_ref()
        .and_then(|slot| slot.resolve(states).as_ref())
    {
        chrome.text_color = text_color.resolve(theme);
    }

    if let Some(placeholder) = style_override
        .placeholder_color
        .as_ref()
        .and_then(|slot| slot.resolve(states).as_ref())
    {
        chrome.placeholder_color = placeholder.resolve(theme);
    }

    if let Some(caret_color) = style_override
        .caret_color
        .as_ref()
        .and_then(|slot| slot.resolve(states).as_ref())
    {
        chrome.caret_color = caret_color.resolve(theme);
    }
}
