use std::sync::Arc;

use fret_core::{Color, Edges, FontId, FontWeight, Px, TextStyle};
use fret_icons::IconId;
use fret_runtime::CommandId;
use fret_ui::element::{
    AnyElement, CrossAlign, FlexProps, MainAlign, Overflow, PressableProps, RingStyle, SpacerProps,
};
use fret_ui::{ElementContext, Theme, UiHost};
use fret_ui_kit::command::ElementCommandGatingExt as _;
use fret_ui_kit::declarative::action_hooks::ActionHooksExt as _;
use fret_ui_kit::declarative::icon as decl_icon;
use fret_ui_kit::declarative::scroll as decl_scroll;
use fret_ui_kit::declarative::style as decl_style;
use fret_ui_kit::{ChromeRefinement, ColorRef, LayoutRefinement, MetricRef, Radius, Space, ui};

use crate::hover_card::{HoverCard, HoverCardAlign};
use crate::layout as shadcn_layout;

fn alpha_mul(mut c: Color, mul: f32) -> Color {
    c.a *= mul;
    c
}

fn sidebar_width(theme: &Theme) -> Px {
    theme
        .metric_by_key("component.sidebar.width")
        .unwrap_or(Px(256.0))
}

fn sidebar_width_icon(theme: &Theme) -> Px {
    theme
        .metric_by_key("component.sidebar.width_icon")
        .unwrap_or(Px(48.0))
}

fn sidebar_bg(theme: &Theme) -> Color {
    theme
        .color_by_key("sidebar.background")
        .or_else(|| theme.color_by_key("sidebar"))
        .unwrap_or_else(|| theme.color_required("sidebar"))
}

fn sidebar_fg(theme: &Theme) -> Color {
    theme
        .color_by_key("sidebar.foreground")
        .or_else(|| theme.color_by_key("sidebar-foreground"))
        .unwrap_or_else(|| theme.color_required("sidebar-foreground"))
}

fn sidebar_border(theme: &Theme) -> Color {
    theme
        .color_by_key("sidebar.border")
        .or_else(|| theme.color_by_key("sidebar-border"))
        .or_else(|| theme.color_by_key("border"))
        .unwrap_or_else(|| theme.color_required("sidebar-border"))
}

fn sidebar_accent(theme: &Theme) -> Color {
    theme
        .color_by_key("sidebar.accent")
        .or_else(|| theme.color_by_key("sidebar-accent"))
        .or_else(|| theme.color_by_key("accent"))
        .unwrap_or_else(|| theme.color_required("sidebar-accent"))
}

fn sidebar_accent_fg(theme: &Theme) -> Color {
    theme
        .color_by_key("sidebar.accent.foreground")
        .or_else(|| theme.color_by_key("sidebar-accent-foreground"))
        .or_else(|| theme.color_by_key("accent-foreground"))
        .unwrap_or_else(|| theme.color_required("sidebar-accent-foreground"))
}

fn sidebar_ring(theme: &Theme, radius: Px) -> RingStyle {
    decl_style::focus_ring(theme, radius)
}

fn menu_button_style(theme: &Theme) -> TextStyle {
    let size = theme
        .metric_by_key("component.sidebar.menu_button_px")
        .or_else(|| theme.metric_by_key("font.size"))
        .unwrap_or_else(|| theme.metric_required("font.size"));
    let line_height = theme
        .metric_by_key("component.sidebar.menu_button_line_height")
        .or_else(|| theme.metric_by_key("font.line_height"))
        .unwrap_or_else(|| theme.metric_required("font.line_height"));
    TextStyle {
        font: FontId::default(),
        size,
        weight: FontWeight::MEDIUM,
        line_height: Some(line_height),
        ..Default::default()
    }
}

/// shadcn/ui `Sidebar` (V1).
///
/// This is implemented as a declarative composition surface (not a retained widget), so it can
/// fully participate in Tailwind-like layout/style refinements.
#[derive(Debug, Clone)]
pub struct Sidebar {
    children: Vec<AnyElement>,
    collapsed: bool,
    chrome: ChromeRefinement,
    layout: LayoutRefinement,
}

impl Sidebar {
    pub fn new(children: impl IntoIterator<Item = AnyElement>) -> Self {
        let children = children.into_iter().collect();
        Self {
            children,
            collapsed: false,
            chrome: ChromeRefinement::default(),
            layout: LayoutRefinement::default(),
        }
    }

    pub fn collapsed(mut self, collapsed: bool) -> Self {
        self.collapsed = collapsed;
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
        let theme = Theme::global(&*cx.app).clone();

        let w = if self.collapsed {
            sidebar_width_icon(&theme)
        } else {
            sidebar_width(&theme)
        };
        let layout = LayoutRefinement::default()
            .w_px(MetricRef::Px(w))
            .h_full()
            .merge(self.layout);

        let chrome = ChromeRefinement::default()
            .bg(ColorRef::Color(sidebar_bg(&theme)))
            .border_1()
            .border_color(ColorRef::Color(sidebar_border(&theme)))
            .merge(self.chrome);

        let mut props = decl_style::container_props(&theme, chrome, layout);
        props.layout.overflow = Overflow::Clip;

        let children = self.children;
        shadcn_layout::container_flow(cx, props, children)
    }
}

#[derive(Debug, Clone)]
pub struct SidebarHeader {
    children: Vec<AnyElement>,
}

impl SidebarHeader {
    pub fn new(children: impl IntoIterator<Item = AnyElement>) -> Self {
        let children = children.into_iter().collect();
        Self { children }
    }

    pub fn into_element<H: UiHost>(self, cx: &mut ElementContext<'_, H>) -> AnyElement {
        let theme = Theme::global(&*cx.app).clone();
        let props = decl_style::container_props(
            &theme,
            ChromeRefinement::default().p(Space::N2),
            LayoutRefinement::default(),
        );
        let children = self.children;
        shadcn_layout::container_flow(cx, props, children)
    }
}

#[derive(Debug, Clone)]
pub struct SidebarFooter {
    children: Vec<AnyElement>,
}

impl SidebarFooter {
    pub fn new(children: impl IntoIterator<Item = AnyElement>) -> Self {
        let children = children.into_iter().collect();
        Self { children }
    }

    pub fn into_element<H: UiHost>(self, cx: &mut ElementContext<'_, H>) -> AnyElement {
        let theme = Theme::global(&*cx.app).clone();
        let props = decl_style::container_props(
            &theme,
            ChromeRefinement::default().p(Space::N2),
            LayoutRefinement::default(),
        );
        let children = self.children;
        shadcn_layout::container_flow(cx, props, children)
    }
}

#[derive(Debug, Clone)]
pub struct SidebarContent {
    children: Vec<AnyElement>,
    collapsed: bool,
}

impl SidebarContent {
    pub fn new(children: impl IntoIterator<Item = AnyElement>) -> Self {
        let children = children.into_iter().collect();
        Self {
            children,
            collapsed: false,
        }
    }

    pub fn collapsed(mut self, collapsed: bool) -> Self {
        self.collapsed = collapsed;
        self
    }

    pub fn into_element<H: UiHost>(self, cx: &mut ElementContext<'_, H>) -> AnyElement {
        let theme = Theme::global(&*cx.app).clone();

        let mut layout = LayoutRefinement::default().h_full();
        if self.collapsed {
            layout = layout.overflow_hidden();
        }

        let children = self.children;
        decl_scroll::overflow_scrollbar(cx, layout, move |cx| {
            let gap = decl_style::space(&theme, Space::N2);
            let col = FlexProps {
                direction: fret_core::Axis::Vertical,
                gap,
                padding: Edges::all(gap),
                layout: fret_ui::element::LayoutStyle {
                    size: fret_ui::element::SizeStyle {
                        width: fret_ui::element::Length::Fill,
                        height: fret_ui::element::Length::Auto,
                        ..Default::default()
                    },
                    ..Default::default()
                },
                ..Default::default()
            };

            vec![cx.flex(col, move |_cx| children)]
        })
    }
}

#[derive(Debug, Clone)]
pub struct SidebarGroup {
    children: Vec<AnyElement>,
}

impl SidebarGroup {
    pub fn new(children: impl IntoIterator<Item = AnyElement>) -> Self {
        let children = children.into_iter().collect();
        Self { children }
    }

    pub fn into_element<H: UiHost>(self, cx: &mut ElementContext<'_, H>) -> AnyElement {
        let theme = Theme::global(&*cx.app).clone();
        let chrome = ChromeRefinement::default().p(Space::N2);
        let props = decl_style::container_props(&theme, chrome, LayoutRefinement::default());
        let children = self.children;
        shadcn_layout::container_flow(cx, props, children)
    }
}

#[derive(Debug, Clone)]
pub struct SidebarGroupLabel {
    text: Arc<str>,
    collapsed: bool,
}

impl SidebarGroupLabel {
    pub fn new(text: impl Into<Arc<str>>) -> Self {
        Self {
            text: text.into(),
            collapsed: false,
        }
    }

    pub fn collapsed(mut self, collapsed: bool) -> Self {
        self.collapsed = collapsed;
        self
    }

    pub fn into_element<H: UiHost>(self, cx: &mut ElementContext<'_, H>) -> AnyElement {
        let theme = Theme::global(&*cx.app).clone();
        if self.collapsed {
            return cx.spacer(fret_ui::element::SpacerProps {
                min: Px(0.0),
                ..Default::default()
            });
        }

        let fg = sidebar_fg(&theme);
        let mut fg = fg;
        fg.a = (fg.a * 0.7).clamp(0.0, 1.0);

        let size = theme
            .metric_by_key("component.sidebar.group_label_px")
            .unwrap_or(Px(12.0));
        let line_height = theme
            .metric_by_key("component.sidebar.group_label_line_height")
            .unwrap_or(Px(16.0));

        ui::text(cx, self.text)
            .text_size_px(size)
            .line_height_px(line_height)
            .font_medium()
            .text_color(ColorRef::Color(fg))
            .nowrap()
            .into_element(cx)
    }
}

#[derive(Debug, Clone)]
pub struct SidebarMenu {
    children: Vec<AnyElement>,
}

impl SidebarMenu {
    pub fn new(children: impl IntoIterator<Item = AnyElement>) -> Self {
        let children = children.into_iter().collect();
        Self { children }
    }

    pub fn into_element<H: UiHost>(self, cx: &mut ElementContext<'_, H>) -> AnyElement {
        let props = FlexProps {
            direction: fret_core::Axis::Vertical,
            gap: Px(4.0),
            layout: fret_ui::element::LayoutStyle {
                size: fret_ui::element::SizeStyle {
                    width: fret_ui::element::Length::Fill,
                    height: fret_ui::element::Length::Auto,
                    ..Default::default()
                },
                ..Default::default()
            },
            ..Default::default()
        };
        let children = self.children;
        cx.flex(props, move |_cx| children)
    }
}

#[derive(Debug, Clone)]
pub struct SidebarMenuItem {
    child: AnyElement,
}

impl SidebarMenuItem {
    pub fn new(child: AnyElement) -> Self {
        Self { child }
    }

    pub fn into_element<H: UiHost>(self, _cx: &mut ElementContext<'_, H>) -> AnyElement {
        self.child
    }
}

#[derive(Debug, Clone)]
pub struct SidebarMenuButton {
    label: Arc<str>,
    icon: Option<IconId>,
    active: bool,
    disabled: bool,
    collapsed: bool,
    on_click: Option<CommandId>,
}

impl SidebarMenuButton {
    pub fn new(label: impl Into<Arc<str>>) -> Self {
        Self {
            label: label.into(),
            icon: None,
            active: false,
            disabled: false,
            collapsed: false,
            on_click: None,
        }
    }

    pub fn icon(mut self, icon: IconId) -> Self {
        self.icon = Some(icon);
        self
    }

    pub fn active(mut self, active: bool) -> Self {
        self.active = active;
        self
    }

    pub fn disabled(mut self, disabled: bool) -> Self {
        self.disabled = disabled;
        self
    }

    pub fn collapsed(mut self, collapsed: bool) -> Self {
        self.collapsed = collapsed;
        self
    }

    pub fn on_click(mut self, command: impl Into<CommandId>) -> Self {
        self.on_click = Some(command.into());
        self
    }

    fn build_button<H: UiHost>(&self, cx: &mut ElementContext<'_, H>) -> AnyElement {
        let theme = Theme::global(&*cx.app).clone();

        let radius = decl_style::radius(&theme, Radius::Md);
        let ring = sidebar_ring(&theme, radius);

        let on_click = self.on_click.clone();
        let disabled = self.disabled
            || on_click
                .as_ref()
                .is_some_and(|cmd| !cx.command_is_enabled(cmd));
        let pressable = PressableProps {
            enabled: !disabled,
            focus_ring: Some(ring),
            layout: decl_style::layout_style(&theme, LayoutRefinement::default().w_full()),
            ..Default::default()
        };

        let label = self.label.clone();
        let icon = self.icon.clone();
        let active = self.active;
        let disabled = disabled;
        let collapsed = self.collapsed;

        cx.pressable(pressable, move |cx, st| {
            cx.pressable_dispatch_command_if_enabled_opt(on_click);
            let theme = Theme::global(&*cx.app).clone();

            let bg = if active || st.hovered || st.pressed {
                sidebar_accent(&theme)
            } else {
                Color::TRANSPARENT
            };

            let fg = if disabled {
                alpha_mul(sidebar_fg(&theme), 0.5)
            } else if active || st.hovered || st.pressed {
                sidebar_accent_fg(&theme)
            } else {
                sidebar_fg(&theme)
            };

            let chrome = if bg.a > 0.0 {
                ChromeRefinement::default()
                    .bg(ColorRef::Color(bg))
                    .rounded(Radius::Md)
            } else {
                ChromeRefinement::default().rounded(Radius::Md)
            };

            let mut props = decl_style::container_props(
                &theme,
                chrome,
                LayoutRefinement::default()
                    .w_full()
                    .min_h(fret_ui_kit::MetricRef::Px(Px(32.0))),
            );
            props.layout.overflow = Overflow::Clip;

            let inner_gap = decl_style::space(&theme, Space::N2);

            vec![cx.container(props, move |cx| {
                let row = FlexProps {
                    direction: fret_core::Axis::Horizontal,
                    gap: inner_gap,
                    align: CrossAlign::Center,
                    justify: MainAlign::Start,
                    padding: Edges {
                        top: Px(0.0),
                        right: inner_gap,
                        bottom: Px(0.0),
                        left: inner_gap,
                    },
                    layout: fret_ui::element::LayoutStyle {
                        size: fret_ui::element::SizeStyle {
                            width: fret_ui::element::Length::Fill,
                            height: fret_ui::element::Length::Fill,
                            ..Default::default()
                        },
                        ..Default::default()
                    },
                    ..Default::default()
                };

                let label = label.clone();
                let icon = icon.clone();
                vec![cx.flex(row, move |cx| {
                    let mut out = Vec::new();
                    if let Some(icon) = icon.clone() {
                        out.push(decl_icon::icon(cx, icon));
                    }
                    if !collapsed {
                        let style = menu_button_style(&theme);
                        let mut text = ui::text(cx, label.clone())
                            .text_size_px(style.size)
                            .font_weight(style.weight)
                            .text_color(ColorRef::Color(fg))
                            .truncate();
                        if let Some(line_height) = style.line_height {
                            text = text.line_height_px(line_height);
                        }
                        if let Some(letter_spacing_em) = style.letter_spacing_em {
                            text = text.letter_spacing_em(letter_spacing_em);
                        }
                        out.push(text.into_element(cx));
                    } else {
                        out.push(cx.spacer(SpacerProps {
                            min: Px(0.0),
                            ..Default::default()
                        }));
                    }
                    out
                })]
            })]
        })
    }

    pub fn into_element<H: UiHost>(self, cx: &mut ElementContext<'_, H>) -> AnyElement {
        let button = self.build_button(cx);

        if !self.collapsed {
            return button;
        }

        // In collapsed (icon) mode, show the label via a hover card.
        let theme = Theme::global(&*cx.app).clone();

        let label = self.label.clone();

        let chrome = ChromeRefinement::default()
            .bg(ColorRef::Color(
                theme
                    .color_by_key("popover.background")
                    .unwrap_or_else(|| theme.color_required("popover.background")),
            ))
            .border_1()
            .border_color(ColorRef::Color(
                theme
                    .color_by_key("border")
                    .unwrap_or_else(|| theme.color_required("border")),
            ))
            .rounded(Radius::Md)
            .p(Space::N2);
        let mut props = decl_style::container_props(&theme, chrome, LayoutRefinement::default());
        props.layout.overflow = Overflow::Clip;
        let content = cx.container(props, move |cx| {
            let style = menu_button_style(&theme);
            let mut text = ui::text(cx, label.clone())
                .text_size_px(style.size)
                .font_weight(style.weight)
                .text_color(ColorRef::Color(sidebar_fg(&theme)))
                .nowrap();
            if let Some(line_height) = style.line_height {
                text = text.line_height_px(line_height);
            }
            if let Some(letter_spacing_em) = style.letter_spacing_em {
                text = text.letter_spacing_em(letter_spacing_em);
            }
            vec![text.into_element(cx)]
        });

        HoverCard::new(button, content)
            .align(HoverCardAlign::Center)
            .side_offset(Px(8.0))
            .into_element(cx)
    }
}
