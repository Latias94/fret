//! Material 3 text field (MVP).
//!
//! Outcome-oriented implementation:
//! - Token-driven chrome via `md.comp.(outlined|filled)-text-field.*`.
//! - Hover/focus/error/disabled outcomes via theme tokens (best-effort).

use std::sync::Arc;

use fret_core::{Color, Edges, Px, SemanticsRole, TextOverflow, TextStyle, TextWrap};
use fret_runtime::Model;
use fret_ui::element::{
    AnyElement, ContainerProps, FlexProps, HoverRegionProps, Length, MainAlign, Overflow,
    PointerRegionProps, TextInputProps, TextProps,
};
use fret_ui::elements::ElementContext;
use fret_ui::{GlobalElementId, Invalidation, Theme, UiHost};
use fret_ui_kit::{
    ColorRef, OverrideSlot, WidgetState, WidgetStateProperty, WidgetStates,
    resolve_override_slot_with,
};

use crate::foundation::floating_label;
use crate::interaction::state_layer::StateLayerAnimator;
use crate::tokens::text_field as text_field_tokens;

#[derive(Debug, Clone, Copy, PartialEq, Eq, Default)]
pub enum TextFieldVariant {
    #[default]
    Outlined,
    Filled,
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
    float: StateLayerAnimator,
}

#[derive(Clone)]
pub struct TextField {
    model: Model<String>,
    variant: TextFieldVariant,
    label: Option<Arc<str>>,
    placeholder: Option<Arc<str>>,
    supporting_text: Option<Arc<str>>,
    style: TextFieldStyle,
    disabled: bool,
    error: bool,
    a11y_label: Option<Arc<str>>,
    test_id: Option<Arc<str>>,
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
            disabled: false,
            error: false,
            a11y_label: None,
            test_id: None,
        }
    }

    pub fn variant(mut self, variant: TextFieldVariant) -> Self {
        self.variant = variant;
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

    pub fn into_element<H: UiHost>(self, cx: &mut ElementContext<'_, H>) -> AnyElement {
        cx.scope(|cx| {
            let theme = Theme::global(&*cx.app).clone();

            let variant = self.variant;
            let height = text_field_tokens::container_height(&theme, variant);

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
            let a11y_label = self
                .a11y_label
                .clone()
                .or_else(|| label.clone())
                .or_else(|| placeholder.clone());
            let test_id = self.test_id.clone();

            cx.hover_region(hover, move |cx, hovered| {
                let theme = Theme::global(&*cx.app).clone();

                let mut focused = false;
                let mut input_id = GlobalElementId(0);
                let mut states = WidgetStates::empty();
                let mut input_bg = theme
                    .color_by_key("md.sys.color.surface")
                    .unwrap_or_else(|| theme.color_required("md.sys.color.surface"));
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
                                    text_field_tokens::hover_state_layer(
                                        &theme,
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

                                let mut chrome = text_field_tokens::text_input_style(
                                    &theme,
                                    variant_for_children,
                                    focused,
                                    hovered,
                                    disabled,
                                    error,
                                );
                                apply_text_field_input_overrides(
                                    &theme,
                                    states,
                                    &style_override,
                                    &mut chrome,
                                );

                                input_bg = chrome.background;
                                outline_width_for_notch = match variant_for_children {
                                    TextFieldVariant::Outlined => chrome.border.top,
                                    TextFieldVariant::Filled => Px(0.0),
                                };

                                container.background =
                                    (chrome.background.a > 0.0).then_some(chrome.background);
                                container.corner_radii = chrome.corner_radii;
                                container.border = chrome.border;
                                container.border_color = Some(chrome.border_color);

                                chrome.background = Color::TRANSPARENT;
                                chrome.border = Edges::all(Px(0.0));
                                chrome.border_color = Color::TRANSPARENT;
                                chrome.border_color_focused = Color::TRANSPARENT;

                                let show_placeholder = if label.is_some() {
                                    focused && !populated
                                } else {
                                    true
                                };

                                let mut props = TextInputProps::new(model.clone());
                                props.layout.size.width = Length::Fill;
                                props.layout.size.height = Length::Fill;
                                props.a11y_label = a11y_label.clone();
                                props.a11y_role = Some(SemanticsRole::TextField);
                                props.test_id = test_id.clone();
                                props.placeholder = if show_placeholder {
                                    placeholder.clone()
                                } else {
                                    None
                                };
                                props.chrome = chrome;
                                props.text_style =
                                    crate::foundation::context::inherited_text_style(cx)
                                        .unwrap_or_else(|| {
                                            theme
                                                .text_style_by_key("md.sys.typescale.body-large")
                                                .unwrap_or(TextStyle::default())
                                        });

                                props
                            });

                            let overlay = state_layer.map(|background| {
                                let mut overlay_layout = fret_ui::element::LayoutStyle::default();
                                overlay_layout.position = fret_ui::element::PositionStyle::Absolute;
                                overlay_layout.inset.top = Some(Px(0.0));
                                overlay_layout.inset.right = Some(Px(0.0));
                                overlay_layout.inset.bottom = Some(Px(0.0));
                                overlay_layout.inset.left = Some(Px(0.0));

                                let mut overlay = ContainerProps::default();
                                overlay.layout = overlay_layout;
                                overlay.background = Some(background);
                                overlay.corner_radii = container.corner_radii;
                                cx.container(overlay, |_cx| Vec::new())
                            });

                            match overlay {
                                Some(overlay) => {
                                    cx.container(container, move |_cx| vec![overlay, text_input])
                                }
                                None => cx.container(container, move |_cx| vec![text_input]),
                            }
                        });

                        let populated = cx
                            .get_model_cloned(&model, Invalidation::Layout)
                            .map(|v| !v.is_empty())
                            .unwrap_or(false);
                        let should_float = focused || populated;

                        let now_frame = cx.frame_id.0;
                        let duration_ms = theme
                            .duration_ms_by_key("md.sys.motion.duration.short4")
                            .unwrap_or(200);
                        let easing = theme
                            .easing_by_key("md.sys.motion.easing.standard")
                            .unwrap_or(fret_ui::theme::CubicBezier {
                                x1: 0.0,
                                y1: 0.0,
                                x2: 1.0,
                                y2: 1.0,
                            });

                        let (progress, want_frames) =
                            cx.with_state(TextFieldRuntime::default, |rt| {
                                if rt.float_target != should_float {
                                    rt.float_target = should_float;
                                    rt.float.set_target(
                                        now_frame,
                                        if should_float { 1.0 } else { 0.0 },
                                        duration_ms,
                                        easing,
                                    );
                                }
                                rt.float.advance(now_frame);
                                (rt.float.value(), rt.float.is_active())
                            });

                        if want_frames {
                            cx.request_animation_frame();
                        }

                        children.push(input);

                        if let Some(label) = label.as_ref() {
                            children.push(text_field_label(
                                cx,
                                &theme,
                                variant_for_children,
                                label.clone(),
                                progress,
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
                                &theme,
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
    theme: &Theme,
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
    let style = floating_label::material_floating_label_text_style(theme, progress)
        .or_else(|| theme.text_style_by_key("md.sys.typescale.body-large"));

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
                    color: Some(resolve_override_slot_with(
                        style_override.label_color.as_ref(),
                        states,
                        |color| color.resolve(theme),
                        || {
                            text_field_tokens::label_color(
                                theme, variant, hovered, disabled, error, focused,
                            )
                        },
                    )),
                    wrap: TextWrap::None,
                    overflow: TextOverflow::Clip,
                })]
            })]
        },
    )
}

fn text_field_supporting_text<H: UiHost>(
    cx: &mut ElementContext<'_, H>,
    theme: &Theme,
    variant: TextFieldVariant,
    text: Arc<str>,
    states: WidgetStates,
    style_override: &TextFieldStyle,
    hovered: bool,
    disabled: bool,
    error: bool,
    focused: bool,
) -> AnyElement {
    let mut layout = fret_ui::element::LayoutStyle::default();
    layout.margin.left = fret_ui::element::MarginEdge::Px(Px(16.0));
    layout.margin.right = fret_ui::element::MarginEdge::Px(Px(16.0));

    cx.text_props(TextProps {
        layout,
        text,
        style: theme.text_style_by_key("md.sys.typescale.body-small"),
        color: Some(resolve_override_slot_with(
            style_override.supporting_text_color.as_ref(),
            states,
            |color| color.resolve(theme),
            || {
                text_field_tokens::supporting_text_color(
                    theme, variant, hovered, disabled, error, focused,
                )
            },
        )),
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
