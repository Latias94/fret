use std::sync::Arc;

use fret_components_ui::declarative::action_hooks::ActionHooksExt as _;
use fret_components_ui::declarative::chrome::control_chrome_pressable_with_id_props;
use fret_components_ui::declarative::model_watch::ModelWatchExt as _;
use fret_components_ui::declarative::style as decl_style;
use fret_components_ui::{ChromeRefinement, ColorRef, LayoutRefinement, MetricRef, Radius};
use fret_core::{Color, Corners, Edges, Px};
use fret_runtime::{CommandId, Model};
use fret_ui::element::{
    AnyElement, ContainerProps, InsetStyle, LayoutStyle, Length, PositionStyle, PressableA11y,
    PressableProps, SizeStyle,
};
use fret_ui::{ElementContext, Theme, UiHost};

fn alpha_mul(mut c: Color, mul: f32) -> Color {
    c.a = (c.a * mul).clamp(0.0, 1.0);
    c
}

fn switch_track_w(theme: &Theme) -> Px {
    theme
        .metric_by_key("component.switch.track_w")
        .unwrap_or(Px(36.0))
}

fn switch_track_h(theme: &Theme) -> Px {
    theme
        .metric_by_key("component.switch.track_h")
        .unwrap_or(Px(20.0))
}

fn switch_thumb(theme: &Theme) -> Px {
    theme
        .metric_by_key("component.switch.thumb")
        .unwrap_or(Px(16.0))
}

fn switch_padding(theme: &Theme) -> Px {
    theme
        .metric_by_key("component.switch.thumb_pad")
        .unwrap_or(Px(2.0))
}

fn switch_bg_on(theme: &Theme) -> Color {
    theme.color_by_key("primary").unwrap_or(theme.colors.accent)
}

fn switch_bg_off(theme: &Theme) -> Color {
    theme
        .color_by_key("input")
        .or_else(|| theme.color_by_key("muted"))
        .unwrap_or(theme.colors.panel_border)
}

fn switch_thumb_bg(theme: &Theme) -> Color {
    theme
        .color_by_key("background")
        .unwrap_or(theme.colors.surface_background)
}

#[derive(Clone)]
pub struct Switch {
    model: Model<bool>,
    disabled: bool,
    a11y_label: Option<Arc<str>>,
    on_click: Option<CommandId>,
    chrome: ChromeRefinement,
    layout: LayoutRefinement,
}

impl Switch {
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

    pub fn into_element<H: UiHost>(self, cx: &mut ElementContext<'_, H>) -> AnyElement {
        cx.scope(|cx| {
            let model = self.model;

            let theme = Theme::global(&*cx.app).clone();

            let w = switch_track_w(&theme);
            let h = switch_track_h(&theme);
            let thumb = switch_thumb(&theme);
            let pad = switch_padding(&theme);

            let radius = Px((h.0 * 0.5).max(0.0));
            let ring = decl_style::focus_ring(&theme, radius);

            let layout = LayoutRefinement::default()
                .w_px(MetricRef::Px(w))
                .h_px(MetricRef::Px(h))
                .merge(self.layout);
            let pressable_layout = decl_style::layout_style(&theme, layout);

            let a11y_label = self.a11y_label.clone();
            let disabled = self.disabled;
            let on_click = self.on_click.clone();
            let chrome = self.chrome.clone();

            let pressable = control_chrome_pressable_with_id_props(cx, move |cx, st, _id| {
                cx.pressable_dispatch_command_opt(on_click);
                cx.pressable_toggle_bool(&model);

                let theme = Theme::global(&*cx.app).clone();
                let on = cx.watch_model(&model).copied().unwrap_or(false);

                let mut bg = if on {
                    switch_bg_on(&theme)
                } else {
                    switch_bg_off(&theme)
                };
                let hovered = st.hovered && !disabled;
                if hovered {
                    bg = alpha_mul(bg, if on { 0.9 } else { 0.7 });
                }

                let mut chrome_props = decl_style::container_props(
                    &theme,
                    ChromeRefinement::default()
                        .bg(ColorRef::Color(bg))
                        .rounded(Radius::Full)
                        .merge(chrome.clone()),
                    LayoutRefinement::default(),
                );
                chrome_props.corner_radii = Corners::all(radius);
                chrome_props.layout.size = pressable_layout.size;

                let pressable_props = PressableProps {
                    layout: pressable_layout,
                    enabled: !disabled,
                    focusable: true,
                    focus_ring: Some(ring),
                    a11y: PressableA11y {
                        role: Some(fret_core::SemanticsRole::Switch),
                        label: a11y_label.clone(),
                        checked: Some(on),
                        ..Default::default()
                    },
                    ..Default::default()
                };

                let children = move |cx: &mut ElementContext<'_, H>| {
                    let x = if on {
                        Px((w.0 - pad.0 - thumb.0).max(0.0))
                    } else {
                        pad
                    };

                    let thumb_layout = LayoutStyle {
                        position: PositionStyle::Absolute,
                        inset: InsetStyle {
                            top: Some(pad),
                            left: Some(x),
                            ..Default::default()
                        },
                        size: SizeStyle {
                            width: Length::Px(thumb),
                            height: Length::Px(thumb),
                            ..Default::default()
                        },
                        ..Default::default()
                    };

                    let thumb_bg = switch_thumb_bg(&theme);
                    let thumb_props = ContainerProps {
                        layout: thumb_layout,
                        padding: Edges::all(Px(0.0)),
                        background: Some(thumb_bg),
                        shadow: None,
                        border: Edges::all(Px(0.0)),
                        border_color: None,
                        corner_radii: Corners::all(Px((thumb.0 * 0.5).max(0.0))),
                    };

                    vec![cx.container(thumb_props, |_cx| Vec::new())]
                };

                (pressable_props, chrome_props, children)
            });

            if disabled {
                cx.opacity(0.5, |_cx| vec![pressable])
            } else {
                pressable
            }
        })
    }
}

pub fn switch<H: UiHost>(cx: &mut ElementContext<'_, H>, model: Model<bool>) -> AnyElement {
    Switch::new(model).into_element(cx)
}
