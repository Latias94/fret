//! Material 3 text field (MVP).
//!
//! Outcome-oriented implementation:
//! - Token-driven outline + typography via `md.comp.outlined-text-field.*`.
//! - Hover/focus/error/disabled outcomes via theme tokens (best-effort; no notch yet).

use std::sync::Arc;

use fret_core::{Color, Corners, Edges, Px, SemanticsRole, TextOverflow, TextStyle, TextWrap};
use fret_runtime::Model;
use fret_ui::element::{
    AnyElement, FlexProps, HoverRegionProps, Length, MainAlign, Overflow, TextInputProps, TextProps,
};
use fret_ui::elements::ElementContext;
use fret_ui::{Invalidation, TextInputStyle, Theme, UiHost};

#[derive(Debug, Clone, Copy, PartialEq, Eq, Default)]
pub enum TextFieldVariant {
    #[default]
    Outlined,
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

            let height = outlined_container_height(&theme);

            let mut hover_layout = fret_ui::element::LayoutStyle::default();
            hover_layout.size.width = Length::Fill;
            hover_layout.overflow = Overflow::Visible;
            let hover = HoverRegionProps {
                layout: hover_layout,
            };

            let model = self.model.clone();
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

                            props.chrome = outlined_text_input_style(
                                &theme, focused, hovered, disabled, error,
                            );
                            props.text_style = theme
                                .text_style_by_key("md.sys.typescale.body-large")
                                .unwrap_or(TextStyle::default());

                            cx.text_input(props)
                        });

                        let populated = cx
                            .get_model_cloned(&model, Invalidation::Layout)
                            .map(|v| !v.is_empty())
                            .unwrap_or(false);

                        children.push(input);

                        if let Some(label) = label.as_ref() {
                            let floated = focused || populated;
                            children.push(text_field_label(
                                cx,
                                &theme,
                                label.clone(),
                                floated,
                                disabled,
                                error,
                                focused,
                            ));
                        }

                        if let Some(text) = supporting_text.as_ref() {
                            children.push(text_field_supporting_text(
                                cx,
                                &theme,
                                text.clone(),
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

fn outlined_container_height(theme: &Theme) -> Px {
    theme
        .metric_by_key("md.comp.outlined-text-field.container.height")
        .or_else(|| theme.metric_by_key("md.comp.filled-text-field.container.height"))
        .unwrap_or(Px(56.0))
}

fn outlined_container_corner(theme: &Theme) -> Corners {
    let r = theme
        .metric_by_key("md.comp.outlined-text-field.container.shape")
        .or_else(|| theme.metric_by_key("md.sys.shape.corner.extra-small"))
        .unwrap_or(Px(4.0));
    Corners::all(r)
}

fn alpha_mul(mut c: Color, mul: f32) -> Color {
    c.a = (c.a * mul).clamp(0.0, 1.0);
    c
}

fn outlined_text_input_style(
    theme: &Theme,
    focused: bool,
    hovered: bool,
    disabled: bool,
    error: bool,
) -> TextInputStyle {
    let corner = outlined_container_corner(theme);

    let outline_width = theme
        .metric_by_key("md.comp.outlined-text-field.outline.width")
        .unwrap_or(Px(1.0));
    let hover_width = theme
        .metric_by_key("md.comp.outlined-text-field.hover.outline.width")
        .unwrap_or(outline_width);
    let focus_width = theme
        .metric_by_key("md.comp.outlined-text-field.focus.outline.width")
        .unwrap_or(Px(2.0));
    let disabled_width = theme
        .metric_by_key("md.comp.outlined-text-field.disabled.outline.width")
        .unwrap_or(outline_width);

    let mut style = TextInputStyle::default();
    style.corner_radii = corner;
    style.focus_ring = None;

    style.padding = Edges {
        top: Px(18.0),
        right: Px(16.0),
        bottom: Px(14.0),
        left: Px(16.0),
    };

    let default_bg = theme
        .color_by_key("md.sys.color.surface")
        .unwrap_or_else(|| theme.color_required("card"));
    style.background = default_bg;

    let outline_color = outlined_outline_color(theme, hovered, disabled, error, focused);
    let focused_outline_color = outlined_outline_color(theme, false, disabled, error, true);

    let border_width = if disabled {
        disabled_width
    } else if focused {
        focus_width
    } else if hovered {
        hover_width
    } else {
        outline_width
    };
    style.border = Edges::all(border_width);
    style.border_color = outline_color;
    style.border_color_focused = focused_outline_color;

    style.text_color = outlined_input_text_color(theme, hovered, disabled, error, focused);
    style.placeholder_color = theme
        .color_by_key("md.comp.outlined-text-field.input-text.placeholder.color")
        .or_else(|| theme.color_by_key("md.sys.color.on-surface-variant"))
        .unwrap_or(style.placeholder_color);
    style.selection_color = theme
        .color_by_key("md.sys.color.primary")
        .map(|c| alpha_mul(c, 0.35))
        .unwrap_or(style.selection_color);
    style.caret_color = outlined_caret_color(theme, disabled, error, focused);
    style.preedit_color = theme
        .color_by_key("md.sys.color.primary")
        .unwrap_or(style.preedit_color);

    if disabled {
        let opacity = theme
            .number_by_key("md.comp.outlined-text-field.disabled.input-text.opacity")
            .unwrap_or(0.38);
        style.text_color = alpha_mul(style.text_color, opacity);
        style.placeholder_color = alpha_mul(style.placeholder_color, opacity);

        let outline_opacity = theme
            .number_by_key("md.comp.outlined-text-field.disabled.outline.opacity")
            .unwrap_or(0.12);
        style.border_color = alpha_mul(style.border_color, outline_opacity);
        style.border_color_focused = alpha_mul(style.border_color_focused, outline_opacity);
    }

    style
}

fn outlined_caret_color(theme: &Theme, disabled: bool, error: bool, focused: bool) -> Color {
    let base = if error && focused {
        theme
            .color_by_key("md.comp.outlined-text-field.error.focus.caret.color")
            .or_else(|| theme.color_by_key("md.sys.color.error"))
    } else {
        theme
            .color_by_key("md.comp.outlined-text-field.caret.color")
            .or_else(|| theme.color_by_key("md.sys.color.primary"))
    }
    .unwrap_or_else(|| theme.color_required("foreground"));

    if disabled {
        alpha_mul(base, 0.38)
    } else {
        base
    }
}

fn outlined_input_text_color(
    theme: &Theme,
    hovered: bool,
    disabled: bool,
    error: bool,
    focused: bool,
) -> Color {
    let key = if error && focused {
        "md.comp.outlined-text-field.error.focus.input-text.color"
    } else if error && hovered {
        "md.comp.outlined-text-field.error.hover.input-text.color"
    } else if error {
        "md.comp.outlined-text-field.error.input-text.color"
    } else if focused {
        "md.comp.outlined-text-field.focus.input-text.color"
    } else if hovered {
        "md.comp.outlined-text-field.hover.input-text.color"
    } else {
        "md.comp.outlined-text-field.input-text.color"
    };

    let mut c = theme
        .color_by_key(key)
        .or_else(|| theme.color_by_key("md.sys.color.on-surface"))
        .unwrap_or_else(|| theme.color_required("foreground"));

    if disabled {
        let opacity = theme
            .number_by_key("md.comp.outlined-text-field.disabled.input-text.opacity")
            .unwrap_or(0.38);
        c = alpha_mul(c, opacity);
    }

    c
}

fn outlined_outline_color(
    theme: &Theme,
    hovered: bool,
    disabled: bool,
    error: bool,
    focused: bool,
) -> Color {
    let key = if error && focused {
        "md.comp.outlined-text-field.error.focus.outline.color"
    } else if error && hovered {
        "md.comp.outlined-text-field.error.hover.outline.color"
    } else if error {
        "md.comp.outlined-text-field.error.outline.color"
    } else if focused {
        "md.comp.outlined-text-field.focus.outline.color"
    } else if hovered {
        "md.comp.outlined-text-field.hover.outline.color"
    } else if disabled {
        "md.comp.outlined-text-field.disabled.outline.color"
    } else {
        "md.comp.outlined-text-field.outline.color"
    };

    theme
        .color_by_key(key)
        .or_else(|| theme.color_by_key("md.sys.color.outline"))
        .unwrap_or_else(|| theme.color_required("border"))
}

fn outlined_label_color(theme: &Theme, disabled: bool, error: bool, focused: bool) -> Color {
    let key = if error && focused {
        "md.comp.outlined-text-field.error.focus.label-text.color"
    } else if error {
        "md.comp.outlined-text-field.error.label-text.color"
    } else if focused {
        "md.comp.outlined-text-field.focus.label-text.color"
    } else if disabled {
        "md.comp.outlined-text-field.disabled.label-text.color"
    } else {
        "md.comp.outlined-text-field.label-text.color"
    };

    let mut c = theme
        .color_by_key(key)
        .or_else(|| theme.color_by_key("md.sys.color.on-surface-variant"))
        .unwrap_or_else(|| theme.color_required("muted-foreground"));

    if disabled {
        let opacity = theme
            .number_by_key("md.comp.outlined-text-field.disabled.label-text.opacity")
            .unwrap_or(0.38);
        c = alpha_mul(c, opacity);
    }

    c
}

fn outlined_supporting_text_color(
    theme: &Theme,
    disabled: bool,
    error: bool,
    focused: bool,
) -> Color {
    let key = if error && focused {
        "md.comp.outlined-text-field.error.focus.supporting-text.color"
    } else if error {
        "md.comp.outlined-text-field.error.supporting-text.color"
    } else if focused {
        "md.comp.outlined-text-field.focus.supporting-text.color"
    } else if disabled {
        "md.comp.outlined-text-field.disabled.supporting-text.color"
    } else {
        "md.comp.outlined-text-field.supporting-text.color"
    };

    let mut c = theme
        .color_by_key(key)
        .or_else(|| theme.color_by_key("md.sys.color.on-surface-variant"))
        .unwrap_or_else(|| theme.color_required("muted-foreground"));

    if disabled {
        let opacity = theme
            .number_by_key("md.comp.outlined-text-field.disabled.supporting-text.opacity")
            .unwrap_or(0.38);
        c = alpha_mul(c, opacity);
    }

    c
}

fn text_field_label<H: UiHost>(
    cx: &mut ElementContext<'_, H>,
    theme: &Theme,
    text: Arc<str>,
    floated: bool,
    disabled: bool,
    error: bool,
    focused: bool,
) -> AnyElement {
    let style_key = if floated {
        "md.sys.typescale.body-small"
    } else {
        "md.sys.typescale.body-large"
    };
    let style = theme.text_style_by_key(style_key);

    let y = if floated { Px(6.0) } else { Px(18.0) };
    let x = Px(16.0);

    let mut layout = fret_ui::element::LayoutStyle::default();
    layout.position = fret_ui::element::PositionStyle::Absolute;
    layout.inset.top = Some(y);
    layout.inset.left = Some(x);
    layout.inset.right = Some(Px(16.0));
    layout.overflow = Overflow::Visible;

    cx.text_props(TextProps {
        layout,
        text,
        style,
        color: Some(outlined_label_color(theme, disabled, error, focused)),
        wrap: TextWrap::None,
        overflow: TextOverflow::Clip,
    })
}

fn text_field_supporting_text<H: UiHost>(
    cx: &mut ElementContext<'_, H>,
    theme: &Theme,
    text: Arc<str>,
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
        color: Some(outlined_supporting_text_color(
            theme, disabled, error, focused,
        )),
        wrap: TextWrap::Word,
        overflow: TextOverflow::Clip,
    })
}
