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

use crate::interaction::state_layer::StateLayerAnimator;
use crate::tokens::text_field as text_field_tokens;

#[derive(Debug, Clone, Copy, PartialEq, Eq, Default)]
pub enum TextFieldVariant {
    #[default]
    Outlined,
    Filled,
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
                            let id = cx.root_id();
                            input_id = id;
                            focused = cx.is_focused_element(id);

                            let populated = cx
                                .get_model_cloned(&model, Invalidation::Layout)
                                .map(|v| !v.is_empty())
                                .unwrap_or(false);

                            let show_placeholder = if label.is_some() {
                                focused && !populated
                            } else {
                                true
                            };

                            let mut props = TextInputProps::new(model.clone());
                            props.layout.size.width = Length::Fill;
                            props.layout.size.height = Length::Px(height);
                            props.a11y_label = a11y_label.clone();
                            props.a11y_role = Some(SemanticsRole::TextField);
                            props.test_id = test_id.clone();
                            props.placeholder = if show_placeholder {
                                placeholder.clone()
                            } else {
                                None
                            };

                            let chrome = text_field_tokens::text_input_style(
                                &theme,
                                variant_for_children,
                                focused,
                                hovered,
                                disabled,
                                error,
                            );
                            input_bg = chrome.background;
                            outline_width_for_notch = match variant_for_children {
                                TextFieldVariant::Outlined => chrome.border.top,
                                TextFieldVariant::Filled => Px(0.0),
                            };
                            props.chrome = chrome;
                            props.text_style = theme
                                .text_style_by_key("md.sys.typescale.body-large")
                                .unwrap_or(TextStyle::default());

                            cx.text_input(props)
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

fn lerp_px(a: Px, b: Px, t: f32) -> Px {
    let t = t.clamp(0.0, 1.0);
    Px(a.0 + (b.0 - a.0) * t)
}

fn lerp_f32(a: f32, b: f32, t: f32) -> f32 {
    let t = t.clamp(0.0, 1.0);
    a + (b - a) * t
}

fn interpolated_label_text_style(theme: &Theme, progress: f32) -> Option<TextStyle> {
    let large = theme.text_style_by_key("md.sys.typescale.body-large")?;
    let small = theme.text_style_by_key("md.sys.typescale.body-small")?;

    if large.font != small.font || large.weight != small.weight || large.slant != small.slant {
        return Some(if progress >= 0.5 { small } else { large });
    }

    let size = lerp_px(large.size, small.size, progress);
    let line_height = match (large.line_height, small.line_height) {
        (Some(a), Some(b)) => Some(lerp_px(a, b, progress)),
        (Some(a), None) => Some(a),
        (None, Some(b)) => Some(b),
        (None, None) => None,
    };
    let letter_spacing_em = match (large.letter_spacing_em, small.letter_spacing_em) {
        (Some(a), Some(b)) => Some(lerp_f32(a, b, progress)),
        (Some(a), None) => Some(a),
        (None, Some(b)) => Some(b),
        (None, None) => None,
    };

    Some(TextStyle {
        font: large.font,
        size,
        weight: large.weight,
        slant: large.slant,
        line_height,
        letter_spacing_em,
    })
}

fn text_field_label<H: UiHost>(
    cx: &mut ElementContext<'_, H>,
    theme: &Theme,
    variant: TextFieldVariant,
    text: Arc<str>,
    progress: f32,
    hovered: bool,
    disabled: bool,
    error: bool,
    focused: bool,
    input_id: GlobalElementId,
    input_bg: Color,
    outline_width: Px,
) -> AnyElement {
    let style = interpolated_label_text_style(theme, progress)
        .or_else(|| theme.text_style_by_key("md.sys.typescale.body-large"));

    let y = lerp_px(Px(18.0), Px(6.0), progress);
    let x = Px(16.0);

    let mut layout = fret_ui::element::LayoutStyle::default();
    layout.position = fret_ui::element::PositionStyle::Absolute;
    layout.inset.top = Some(y);
    layout.inset.left = Some(x);
    layout.inset.right = Some(Px(16.0));
    layout.overflow = Overflow::Visible;

    let floated = progress >= 0.5;

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
                    color: Some(text_field_tokens::label_color(
                        theme, variant, hovered, disabled, error, focused,
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
        color: Some(text_field_tokens::supporting_text_color(
            theme, variant, hovered, disabled, error, focused,
        )),
        wrap: TextWrap::Word,
        overflow: TextOverflow::Clip,
    })
}
