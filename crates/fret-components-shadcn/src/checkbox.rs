use std::sync::Arc;

use fret_components_icons::ids;
use fret_components_ui::declarative::action_hooks::ActionHooksExt as _;
use fret_components_ui::declarative::icon as decl_icon;
use fret_components_ui::declarative::style as decl_style;
use fret_components_ui::{ChromeRefinement, ColorRef, LayoutRefinement, MetricRef, Radius};
use fret_core::{Axis, Color, Corners, Edges, Px};
use fret_runtime::{CommandId, Model};
use fret_ui::Invalidation;
use fret_ui::element::{
    AnyElement, ContainerProps, CrossAlign, FlexProps, LayoutStyle, Length, MainAlign,
    PressableA11y, PressableProps,
};
use fret_ui::{ElementCx, Theme, UiHost};

fn alpha_mul(mut c: Color, mul: f32) -> Color {
    c.a = (c.a * mul).clamp(0.0, 1.0);
    c
}

fn checkbox_size(theme: &Theme) -> Px {
    theme
        .metric_by_key("component.checkbox.size")
        .unwrap_or(Px(16.0))
}

fn checkbox_radius(theme: &Theme) -> Px {
    theme
        .metric_by_key("component.checkbox.radius")
        .unwrap_or_else(|| MetricRef::radius(Radius::Sm).resolve(theme))
}

fn checkbox_border(theme: &Theme) -> Color {
    theme
        .color_by_key("input")
        .or_else(|| theme.color_by_key("border"))
        .unwrap_or(theme.colors.panel_border)
}

fn checkbox_bg_checked(theme: &Theme) -> Color {
    theme.color_by_key("primary").unwrap_or(theme.colors.accent)
}

fn checkbox_fg_checked(theme: &Theme) -> Color {
    theme
        .color_by_key("primary-foreground")
        .or_else(|| theme.color_by_key("primary.foreground"))
        .unwrap_or(theme.colors.text_primary)
}

#[derive(Clone)]
pub struct Checkbox {
    model: Model<bool>,
    disabled: bool,
    a11y_label: Option<Arc<str>>,
    on_click: Option<CommandId>,
    chrome: ChromeRefinement,
    layout: LayoutRefinement,
}

impl Checkbox {
    pub fn new(model: Model<bool>) -> Self {
        Self {
            model,
            disabled: false,
            a11y_label: None,
            on_click: None,
            chrome: ChromeRefinement::default(),
            layout: LayoutRefinement::default(),
        }
    }

    pub fn disabled(mut self, disabled: bool) -> Self {
        self.disabled = disabled;
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

    pub fn refine_style(mut self, style: ChromeRefinement) -> Self {
        self.chrome = self.chrome.merge(style);
        self
    }

    pub fn refine_layout(mut self, layout: LayoutRefinement) -> Self {
        self.layout = self.layout.merge(layout);
        self
    }

    pub fn into_element<H: UiHost>(self, cx: &mut ElementCx<'_, H>) -> AnyElement {
        cx.scope(|cx| {
            let model = self.model;
            cx.observe_model(&model, Invalidation::Paint);

            let theme = Theme::global(&*cx.app).clone();
            let checked = cx.app.models().get_copied(&model).unwrap_or(false);

            let size = checkbox_size(&theme);
            let radius = checkbox_radius(&theme);
            let border = checkbox_border(&theme);
            let bg_on = checkbox_bg_checked(&theme);
            let fg_on = checkbox_fg_checked(&theme);

            let layout = LayoutRefinement::default()
                .w_px(MetricRef::Px(size))
                .h_px(MetricRef::Px(size))
                .overflow_hidden()
                .merge(self.layout);
            let pressable_layout = decl_style::layout_style(&theme, layout);

            let a11y_label = self.a11y_label.clone();
            let disabled = self.disabled;
            let on_click = self.on_click.clone();
            let chrome = self.chrome.clone();

            let pressable = cx.pressable(
                PressableProps {
                    layout: pressable_layout,
                    enabled: !disabled,
                    focusable: true,
                    focus_ring: Some(decl_style::focus_ring(&theme, radius)),
                    a11y: PressableA11y {
                        role: Some(fret_core::SemanticsRole::Checkbox),
                        label: a11y_label.clone(),
                        checked: Some(checked),
                        ..Default::default()
                    },
                    ..Default::default()
                },
                move |cx, st| {
                    cx.pressable_dispatch_command_opt(on_click);
                    cx.pressable_toggle_bool(&model);

                    let theme = Theme::global(&*cx.app).clone();
                    let checked = cx.app.models().get_copied(&model).unwrap_or(false);

                    let mut bg = if checked { bg_on } else { Color::TRANSPARENT };
                    let border_color = if checked { bg_on } else { border };
                    let fg = if checked { fg_on } else { Color::TRANSPARENT };

                    let hovered = st.hovered && !disabled;
                    if hovered && !checked {
                        let hover = theme
                            .color_by_key("accent")
                            .or_else(|| theme.color_by_key("hover.background"))
                            .unwrap_or(theme.colors.hover_background);
                        bg = alpha_mul(hover, 0.35);
                    }

                    let mut props = decl_style::container_props(
                        &theme,
                        ChromeRefinement::default()
                            .rounded(Radius::Sm)
                            .border_1()
                            .bg(ColorRef::Color(bg))
                            .border_color(ColorRef::Color(border_color))
                            .merge(chrome.clone()),
                        LayoutRefinement::default(),
                    );
                    props.corner_radii = Corners::all(radius);

                    let mut inner_layout = LayoutStyle::default();
                    inner_layout.size.width = Length::Fill;
                    inner_layout.size.height = Length::Fill;

                    vec![cx.container(
                        ContainerProps {
                            layout: props.layout,
                            padding: Edges::all(Px(0.0)),
                            background: props.background,
                            shadow: props.shadow,
                            border: props.border,
                            border_color: props.border_color,
                            corner_radii: props.corner_radii,
                        },
                        move |cx| {
                            if !checked {
                                return Vec::new();
                            }

                            let icon = decl_icon::icon_with(
                                cx,
                                ids::ui::CHECK,
                                Some(Px(12.0)),
                                Some(ColorRef::Color(fg)),
                            );

                            vec![cx.flex(
                                FlexProps {
                                    layout: inner_layout,
                                    direction: Axis::Horizontal,
                                    gap: Px(0.0),
                                    padding: Edges::all(Px(0.0)),
                                    justify: MainAlign::Center,
                                    align: CrossAlign::Center,
                                    wrap: false,
                                },
                                move |_cx| vec![icon],
                            )]
                        },
                    )]
                },
            );

            if disabled {
                cx.opacity(0.5, |_cx| vec![pressable])
            } else {
                pressable
            }
        })
    }
}

pub fn checkbox<H: UiHost>(cx: &mut ElementCx<'_, H>, model: Model<bool>) -> AnyElement {
    Checkbox::new(model).into_element(cx)
}
