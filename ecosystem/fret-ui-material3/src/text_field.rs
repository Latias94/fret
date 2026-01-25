use std::sync::Arc;

use fret_core::{Corners, Edges, FontId, Px, SemanticsRole, TextStyle};
use fret_runtime::{CommandId, Model};
use fret_ui::element::{
    AnyElement, InteractivityGateProps, Length, Overflow, SizeStyle, TextInputProps,
};
use fret_ui::{ElementContext, TextInputStyle, Theme, UiHost};
use fret_ui_kit::declarative::style as decl_style;
use fret_ui_kit::{
    ColorFallback, ColorRef, LayoutRefinement, MetricRef, Radius, Space, WidgetState,
    WidgetStateProperty, WidgetStates,
};

#[derive(Debug, Clone, Default)]
pub struct TextFieldStyle {
    pub background: Option<WidgetStateProperty<Option<ColorRef>>>,
    pub border_color: Option<WidgetStateProperty<Option<ColorRef>>>,
    pub border_color_focused: Option<WidgetStateProperty<Option<ColorRef>>>,
    pub focus_ring_color: Option<WidgetStateProperty<Option<ColorRef>>>,
    pub text_color: Option<WidgetStateProperty<Option<ColorRef>>>,
    pub placeholder_color: Option<WidgetStateProperty<Option<ColorRef>>>,
}

impl TextFieldStyle {
    pub fn background(mut self, background: WidgetStateProperty<Option<ColorRef>>) -> Self {
        self.background = Some(background);
        self
    }

    pub fn border_color(mut self, border_color: WidgetStateProperty<Option<ColorRef>>) -> Self {
        self.border_color = Some(border_color);
        self
    }

    pub fn border_color_focused(
        mut self,
        border_color_focused: WidgetStateProperty<Option<ColorRef>>,
    ) -> Self {
        self.border_color_focused = Some(border_color_focused);
        self
    }

    pub fn focus_ring_color(
        mut self,
        focus_ring_color: WidgetStateProperty<Option<ColorRef>>,
    ) -> Self {
        self.focus_ring_color = Some(focus_ring_color);
        self
    }

    pub fn text_color(mut self, text_color: WidgetStateProperty<Option<ColorRef>>) -> Self {
        self.text_color = Some(text_color);
        self
    }

    pub fn placeholder_color(
        mut self,
        placeholder_color: WidgetStateProperty<Option<ColorRef>>,
    ) -> Self {
        self.placeholder_color = Some(placeholder_color);
        self
    }

    pub fn merged(mut self, other: Self) -> Self {
        if other.background.is_some() {
            self.background = other.background;
        }
        if other.border_color.is_some() {
            self.border_color = other.border_color;
        }
        if other.border_color_focused.is_some() {
            self.border_color_focused = other.border_color_focused;
        }
        if other.focus_ring_color.is_some() {
            self.focus_ring_color = other.focus_ring_color;
        }
        if other.text_color.is_some() {
            self.text_color = other.text_color;
        }
        if other.placeholder_color.is_some() {
            self.placeholder_color = other.placeholder_color;
        }
        self
    }
}

fn token(key: &'static str, fallback: ColorFallback) -> ColorRef {
    ColorRef::Token { key, fallback }
}

fn default_style() -> (
    WidgetStateProperty<Option<ColorRef>>,
    WidgetStateProperty<Option<ColorRef>>,
    WidgetStateProperty<Option<ColorRef>>,
    WidgetStateProperty<Option<ColorRef>>,
    WidgetStateProperty<Option<ColorRef>>,
    WidgetStateProperty<Option<ColorRef>>,
) {
    let bg = WidgetStateProperty::new(Some(token(
        "material3.text_field.container",
        ColorFallback::ThemeSurfaceBackground,
    )))
    .when(
        WidgetStates::DISABLED,
        Some(token(
            "material3.text_field.disabled.container",
            ColorFallback::ThemeTokenAlphaMul {
                key: "panel.background",
                mul: 0.38,
            },
        )),
    );

    let border = WidgetStateProperty::new(Some(token(
        "material3.text_field.outline",
        ColorFallback::ThemePanelBorder,
    )))
    .when(
        WidgetStates::DISABLED,
        Some(token(
            "material3.text_field.disabled.outline",
            ColorFallback::ThemeTokenAlphaMul {
                key: "border",
                mul: 0.38,
            },
        )),
    );

    let border_focused = WidgetStateProperty::new(Some(token(
        "material3.text_field.focus.outline",
        ColorFallback::ThemeAccent,
    )));

    let ring = WidgetStateProperty::new(Some(token(
        "material3.text_field.focus.ring",
        ColorFallback::ThemeAccent,
    )));

    let text = WidgetStateProperty::new(Some(token(
        "material3.text_field.text",
        ColorFallback::ThemeTextPrimary,
    )))
    .when(
        WidgetStates::DISABLED,
        Some(token(
            "material3.text_field.disabled.text",
            ColorFallback::ThemeTextDisabled,
        )),
    );

    let placeholder = WidgetStateProperty::new(Some(token(
        "material3.text_field.placeholder",
        ColorFallback::ThemeTextMuted,
    )))
    .when(
        WidgetStates::DISABLED,
        Some(token(
            "material3.text_field.disabled.placeholder",
            ColorFallback::ThemeTextDisabled,
        )),
    );

    (bg, border, border_focused, ring, text, placeholder)
}

#[derive(Clone)]
pub struct TextField {
    model: Model<String>,
    a11y_label: Option<Arc<str>>,
    a11y_role: Option<SemanticsRole>,
    test_id: Option<Arc<str>>,
    placeholder: Option<Arc<str>>,
    disabled: bool,
    submit_command: Option<CommandId>,
    cancel_command: Option<CommandId>,
    style: TextFieldStyle,
    layout: LayoutRefinement,
}

impl TextField {
    pub fn new(model: Model<String>) -> Self {
        Self {
            model,
            a11y_label: None,
            a11y_role: None,
            test_id: None,
            placeholder: None,
            disabled: false,
            submit_command: None,
            cancel_command: None,
            style: TextFieldStyle::default(),
            layout: LayoutRefinement::default(),
        }
    }

    pub fn a11y_label(mut self, label: impl Into<Arc<str>>) -> Self {
        self.a11y_label = Some(label.into());
        self
    }

    pub fn a11y_role(mut self, role: SemanticsRole) -> Self {
        self.a11y_role = Some(role);
        self
    }

    pub fn test_id(mut self, id: impl Into<Arc<str>>) -> Self {
        self.test_id = Some(id.into());
        self
    }

    pub fn placeholder(mut self, placeholder: impl Into<Arc<str>>) -> Self {
        self.placeholder = Some(placeholder.into());
        self
    }

    pub fn disabled(mut self, disabled: bool) -> Self {
        self.disabled = disabled;
        self
    }

    pub fn submit_command(mut self, command: CommandId) -> Self {
        self.submit_command = Some(command);
        self
    }

    pub fn cancel_command(mut self, command: CommandId) -> Self {
        self.cancel_command = Some(command);
        self
    }

    pub fn style(mut self, style: TextFieldStyle) -> Self {
        self.style = self.style.merged(style);
        self
    }

    pub fn refine_layout(mut self, layout: LayoutRefinement) -> Self {
        self.layout = self.layout.merge(layout);
        self
    }

    pub fn into_element<H: UiHost>(self, cx: &mut ElementContext<'_, H>) -> AnyElement {
        let theme = Theme::global(&*cx.app).clone();

        let (
            default_bg,
            default_border,
            default_border_focused,
            default_ring,
            default_text,
            default_placeholder,
        ) = default_style();
        let style_override = self.style;

        let model = self.model;
        let disabled = self.disabled;
        let a11y_label = self.a11y_label;
        let a11y_role = self.a11y_role;
        let test_id = self.test_id;
        let placeholder = self.placeholder;
        let submit_command = self.submit_command;
        let cancel_command = self.cancel_command;

        let layout = LayoutRefinement::default()
            .min_h(MetricRef::Px(Px(40.0)))
            .merge(self.layout);

        let text_input = cx.text_input_with_id_props(move |cx, id| {
            let focused = cx.is_focused_element(id);
            let focus_visible =
                focused && fret_ui::focus_visible::is_focus_visible(cx.app, Some(cx.window));

            let mut states = WidgetStates::empty();
            states.set(WidgetState::Disabled, disabled);
            states.set(WidgetState::Focused, focused);
            states.set(WidgetState::FocusVisible, focus_visible);

            let bg_ref = style_override
                .background
                .as_ref()
                .and_then(|p| p.resolve(states).clone())
                .or_else(|| default_bg.resolve(states).clone());
            let border_ref = style_override
                .border_color
                .as_ref()
                .and_then(|p| p.resolve(states).clone())
                .or_else(|| default_border.resolve(states).clone());
            let border_focused_ref = style_override
                .border_color_focused
                .as_ref()
                .and_then(|p| p.resolve(states).clone())
                .or_else(|| default_border_focused.resolve(states).clone());
            let ring_ref = style_override
                .focus_ring_color
                .as_ref()
                .and_then(|p| p.resolve(states).clone())
                .or_else(|| default_ring.resolve(states).clone());
            let text_ref = style_override
                .text_color
                .as_ref()
                .and_then(|p| p.resolve(states).clone())
                .or_else(|| default_text.resolve(states).clone());
            let placeholder_ref = style_override
                .placeholder_color
                .as_ref()
                .and_then(|p| p.resolve(states).clone())
                .or_else(|| default_placeholder.resolve(states).clone());

            let bg = bg_ref.map(|c| c.resolve(&theme));
            let border = border_ref.map(|c| c.resolve(&theme));
            let border_focused = border_focused_ref.map(|c| c.resolve(&theme));
            let ring_color = ring_ref.map(|c| c.resolve(&theme));
            let text_color = text_ref.map(|c| c.resolve(&theme));
            let placeholder_color = placeholder_ref.map(|c| c.resolve(&theme));

            let radius = MetricRef::radius(Radius::Md).resolve(&theme);
            let px = MetricRef::space(Space::N3).resolve(&theme);
            let py = MetricRef::space(Space::N2).resolve(&theme);

            let mut chrome = TextInputStyle::from_theme(theme.snapshot());
            chrome.padding = Edges {
                top: py,
                right: px,
                bottom: py,
                left: px,
            };
            chrome.corner_radii = Corners::all(radius);
            chrome.border = Edges::all(Px(1.0));
            chrome.background = bg.unwrap_or(chrome.background);
            if let Some(border) = border {
                chrome.border_color = border;
            }
            if let Some(border_focused) = border_focused {
                chrome.border_color_focused = border_focused;
            }
            if let Some(text_color) = text_color {
                chrome.text_color = text_color;
                chrome.caret_color = text_color;
            }
            if let Some(placeholder_color) = placeholder_color {
                chrome.placeholder_color = placeholder_color;
            }

            let mut ring = decl_style::focus_ring(&theme, radius);
            if let Some(ring_color) = ring_color {
                ring.color = ring_color;
            }
            chrome.focus_ring = Some(ring);

            let font_line_height = theme
                .metric_by_key("font.line_height")
                .unwrap_or_else(|| theme.metric_required("font.line_height"));
            let text_style = TextStyle {
                font: FontId::default(),
                size: theme.metric_required("font.size"),
                line_height: Some(font_line_height),
                ..Default::default()
            };

            let mut props = TextInputProps::new(model.clone());
            props.a11y_label = a11y_label.clone();
            props.a11y_role = a11y_role.or(Some(SemanticsRole::TextField));
            props.test_id = test_id.clone();
            props.placeholder = placeholder.clone();
            props.chrome = chrome;
            props.text_style = text_style;
            props.submit_command = submit_command.clone();
            props.cancel_command = cancel_command.clone();

            props.layout.size = SizeStyle {
                width: Length::Fill,
                min_width: Some(Px(0.0)),
                min_height: Some(Px(40.0)),
                ..Default::default()
            };
            props.layout.overflow = Overflow::Clip;
            decl_style::apply_layout_refinement(&theme, layout.clone(), &mut props.layout);

            props
        });

        if !disabled {
            return text_input;
        }

        cx.interactivity_gate_props(
            InteractivityGateProps {
                present: true,
                interactive: false,
                ..Default::default()
            },
            move |_cx| vec![text_input],
        )
    }
}
