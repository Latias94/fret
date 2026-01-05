use std::sync::Arc;

use fret_core::{Color, Edges, FontWeight, Px, SemanticsRole, TextOverflow, TextStyle, TextWrap};
use fret_runtime::{CommandId, Model};
use fret_ui::element::{
    AnyElement, CrossAlign, FlexProps, MainAlign, PressableA11y, PressableProps, TextProps,
};
use fret_ui::{ElementContext, Theme, UiHost};
use fret_ui_kit::declarative::action_hooks::ActionHooksExt as _;
use fret_ui_kit::declarative::chrome::control_chrome_pressable_with_id_props;
use fret_ui_kit::declarative::model_watch::ModelWatchExt as _;
use fret_ui_kit::declarative::style as decl_style;
use fret_ui_kit::{
    ChromeRefinement, ColorRef, LayoutRefinement, MetricRef, Radius, Size as ComponentSize, Space,
};

#[derive(Debug, Clone, Copy, PartialEq, Eq, Default)]
pub enum ToggleVariant {
    #[default]
    Default,
    Outline,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Default)]
pub enum ToggleSize {
    #[default]
    Default,
    Sm,
    Lg,
}

impl ToggleSize {
    pub fn component_size(self) -> ComponentSize {
        match self {
            Self::Default => ComponentSize::Medium,
            Self::Sm => ComponentSize::Small,
            Self::Lg => ComponentSize::Large,
        }
    }
}

fn alpha_mul(mut c: Color, mul: f32) -> Color {
    c.a = (c.a * mul).clamp(0.0, 1.0);
    c
}

fn toggle_bg_hover(theme: &Theme) -> Color {
    theme
        .color_by_key("muted")
        .unwrap_or(theme.colors.hover_background)
}

fn toggle_fg_muted(theme: &Theme) -> Color {
    theme
        .color_by_key("muted.foreground")
        .or_else(|| theme.color_by_key("muted-foreground"))
        .unwrap_or(theme.colors.text_muted)
}

fn toggle_ring_color(theme: &Theme) -> Color {
    theme
        .color_by_key("ring")
        .unwrap_or(theme.colors.focus_ring)
}

fn toggle_bg_on(theme: &Theme) -> Color {
    theme.color_by_key("accent").unwrap_or(theme.colors.accent)
}

fn toggle_fg_on(theme: &Theme) -> Color {
    theme
        .color_by_key("accent-foreground")
        .or_else(|| theme.color_by_key("accent.foreground"))
        .unwrap_or(theme.colors.text_primary)
}

fn toggle_border(theme: &Theme) -> Color {
    theme
        .color_by_key("input")
        .or_else(|| theme.color_by_key("border"))
        .unwrap_or(theme.colors.panel_border)
}

fn toggle_h(theme: &Theme, size: ToggleSize) -> Px {
    let (key, fallback) = match size {
        ToggleSize::Default => ("component.toggle.h", Px(36.0)),
        ToggleSize::Sm => ("component.toggle.h_sm", Px(32.0)),
        ToggleSize::Lg => ("component.toggle.h_lg", Px(40.0)),
    };
    theme.metric_by_key(key).unwrap_or(fallback)
}

fn toggle_pad_x(theme: &Theme, size: ToggleSize) -> Px {
    let (key, fallback) = match size {
        ToggleSize::Default => ("component.toggle.px", Px(8.0)),
        ToggleSize::Sm => ("component.toggle.px_sm", Px(6.0)),
        ToggleSize::Lg => ("component.toggle.px_lg", Px(10.0)),
    };
    theme.metric_by_key(key).unwrap_or(fallback)
}

fn toggle_text_style(theme: &Theme) -> TextStyle {
    let px = theme
        .metric_by_key("component.toggle.text_px")
        .or_else(|| theme.metric_by_key("font.size"))
        .unwrap_or(theme.metrics.font_size);
    let line_height = theme
        .metric_by_key("component.toggle.line_height")
        .or_else(|| theme.metric_by_key("font.line_height"))
        .unwrap_or(theme.metrics.font_line_height);
    TextStyle {
        size: px,
        weight: FontWeight::MEDIUM,
        line_height: Some(line_height),
        ..Default::default()
    }
}

#[derive(Clone)]
pub struct Toggle {
    model: Model<bool>,
    label: Option<Arc<str>>,
    children: Vec<AnyElement>,
    disabled: bool,
    a11y_label: Option<Arc<str>>,
    on_click: Option<CommandId>,
    variant: ToggleVariant,
    size: ToggleSize,
    chrome: ChromeRefinement,
    layout: LayoutRefinement,
}

impl std::fmt::Debug for Toggle {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.debug_struct("Toggle")
            .field("model", &"<model>")
            .field("label", &self.label.as_ref().map(|s| s.as_ref()))
            .field("children_len", &self.children.len())
            .field("disabled", &self.disabled)
            .field("a11y_label", &self.a11y_label.as_ref().map(|s| s.as_ref()))
            .field("on_click", &self.on_click)
            .field("variant", &self.variant)
            .field("size", &self.size)
            .field("chrome", &self.chrome)
            .field("layout", &self.layout)
            .finish()
    }
}

impl Toggle {
    pub fn new(model: Model<bool>) -> Self {
        Self {
            model,
            label: None,
            children: Vec::new(),
            disabled: false,
            a11y_label: None,
            on_click: None,
            variant: ToggleVariant::default(),
            size: ToggleSize::default(),
            chrome: ChromeRefinement::default(),
            layout: LayoutRefinement::default(),
        }
    }

    pub fn children(mut self, children: Vec<AnyElement>) -> Self {
        self.children = children;
        self
    }

    pub fn label(mut self, label: impl Into<Arc<str>>) -> Self {
        self.label = Some(label.into());
        self
    }

    pub fn a11y_label(mut self, label: impl Into<Arc<str>>) -> Self {
        self.a11y_label = Some(label.into());
        self
    }

    pub fn on_click(mut self, command: impl Into<CommandId>) -> Self {
        self.on_click = Some(command.into());
        self
    }

    pub fn disabled(mut self, disabled: bool) -> Self {
        self.disabled = disabled;
        self
    }

    pub fn variant(mut self, variant: ToggleVariant) -> Self {
        self.variant = variant;
        self
    }

    pub fn size(mut self, size: ToggleSize) -> Self {
        self.size = size;
        self
    }

    pub fn refine_style(mut self, style: ChromeRefinement) -> Self {
        self.chrome = self.chrome.merge(style);
        self
    }

    pub fn refine_layout(mut self, layout: LayoutRefinement) -> Self {
        self.layout = self.layout.merge(layout);
        self
    }

    pub fn into_element<H: UiHost>(self, cx: &mut ElementContext<'_, H>) -> AnyElement {
        let model = self.model;
        let label = self.label;
        let children = self.children;
        let disabled = self.disabled;
        let a11y_label = self.a11y_label.clone();
        let on_click = self.on_click;
        let variant = self.variant;
        let size_token = self.size;
        let chrome = self.chrome;
        let layout = self.layout;

        let theme = Theme::global(&*cx.app).clone();

        let radius = MetricRef::radius(Radius::Md).resolve(&theme);
        let ring_border = toggle_ring_color(&theme);
        let mut ring = decl_style::focus_ring(&theme, radius);
        ring.color = alpha_mul(ring_border, 0.5);
        let text_style = toggle_text_style(&theme);

        let h = toggle_h(&theme, size_token);
        let min_h = h;
        let min_w = h;
        let pad_x = toggle_pad_x(&theme, size_token);
        let pad_y = Px(0.0);

        let pressable_layout = decl_style::layout_style(
            &theme,
            LayoutRefinement::default()
                .min_h(MetricRef::Px(min_h))
                .min_w(MetricRef::Px(min_w))
                .merge(layout),
        );

        let fg_disabled = theme.colors.text_disabled;
        let fg_default = theme
            .color_by_key("foreground")
            .unwrap_or(theme.colors.text_primary);
        let fg_muted = toggle_fg_muted(&theme);
        let bg_hover = toggle_bg_hover(&theme);
        let bg_on = toggle_bg_on(&theme);
        let fg_on = toggle_fg_on(&theme);
        let border = toggle_border(&theme);

        let base_chrome = match variant {
            ToggleVariant::Default => ChromeRefinement {
                radius: Some(MetricRef::Px(radius)),
                border_width: Some(MetricRef::Px(Px(1.0))),
                border_color: Some(ColorRef::Color(Color::TRANSPARENT)),
                ..Default::default()
            },
            ToggleVariant::Outline => ChromeRefinement {
                radius: Some(MetricRef::Px(radius)),
                border_width: Some(MetricRef::Px(Px(1.0))),
                border_color: Some(ColorRef::Color(border)),
                ..Default::default()
            },
        }
        .merge(chrome);

        control_chrome_pressable_with_id_props(cx, move |cx, state, _id| {
            cx.pressable_dispatch_command_opt(on_click);
            cx.pressable_toggle_bool(&model);

            let on = cx.watch_model(&model).copied().unwrap_or(false);
            let hovered = state.hovered && !state.pressed;
            let pressed = state.pressed;

            let (hover_bg, hover_fg) = match variant {
                ToggleVariant::Default => (bg_hover, fg_muted),
                ToggleVariant::Outline => (bg_on, fg_on),
            };

            let mut fg = if disabled {
                fg_disabled
            } else if on {
                fg_on
            } else if hovered {
                hover_fg
            } else {
                fg_default
            };

            let mut bg = if on && !disabled {
                Some(bg_on)
            } else if hovered && !disabled {
                Some(hover_bg)
            } else {
                None
            };

            if pressed && !disabled {
                fg = hover_fg;
                bg = Some(hover_bg);
            }

            let mut chrome_props = decl_style::container_props(
                &theme,
                base_chrome.clone(),
                LayoutRefinement::default(),
            );
            chrome_props.padding = Edges {
                top: pad_y,
                right: pad_x,
                bottom: pad_y,
                left: pad_x,
            };
            if matches!(variant, ToggleVariant::Outline) {
                chrome_props.shadow = Some(decl_style::shadow_xs(&theme, radius));
            }
            if bg.is_some() {
                chrome_props.background = bg;
            }
            if state.focused && !disabled {
                chrome_props.border_color = Some(ring_border);
            }
            chrome_props.layout.size = pressable_layout.size;

            let pressable_props = PressableProps {
                layout: pressable_layout,
                enabled: !disabled,
                focusable: true,
                focus_ring: Some(ring),
                a11y: PressableA11y {
                    role: Some(SemanticsRole::Button),
                    label: a11y_label,
                    selected: on,
                    ..Default::default()
                },
                ..Default::default()
            };

            let content_children = move |cx: &mut ElementContext<'_, H>| {
                vec![cx.flex(
                    FlexProps {
                        direction: fret_core::Axis::Horizontal,
                        gap: MetricRef::space(Space::N2).resolve(&theme),
                        padding: Edges::all(Px(0.0)),
                        justify: MainAlign::Center,
                        align: CrossAlign::Center,
                        wrap: false,
                        ..Default::default()
                    },
                    move |cx: &mut ElementContext<'_, H>| {
                        let mut out = Vec::new();
                        out.extend(children);
                        if let Some(label) = label {
                            out.push(cx.text_props(TextProps {
                                layout: Default::default(),
                                text: label,
                                style: Some(text_style),
                                color: Some(fg),
                                wrap: TextWrap::None,
                                overflow: TextOverflow::Clip,
                            }));
                        }
                        out
                    },
                )]
            };

            (pressable_props, chrome_props, content_children)
        })
    }
}

pub fn toggle<H: UiHost>(
    cx: &mut ElementContext<'_, H>,
    model: Model<bool>,
    f: impl FnOnce(&mut ElementContext<'_, H>) -> Vec<AnyElement>,
) -> AnyElement {
    Toggle::new(model).children(f(cx)).into_element(cx)
}
