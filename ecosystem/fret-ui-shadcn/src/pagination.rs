use fret_core::{Color, Corners, Edges, Px};
use fret_icons::IconId;
use fret_runtime::CommandId;
use fret_ui::element::{
    AnyElement, ContainerProps, CrossAlign, FlexProps, MainAlign, PressableProps,
};
use fret_ui::{ElementContext, Theme, UiHost};
use fret_ui_kit::command::ElementCommandGatingExt as _;
use fret_ui_kit::declarative::action_hooks::ActionHooksExt as _;
use fret_ui_kit::declarative::icon as decl_icon;
use fret_ui_kit::declarative::style as decl_style;
use fret_ui_kit::primitives::direction as direction_prim;
use fret_ui_kit::{LayoutRefinement, MetricRef, Radius, Size as ComponentSize, Space};
use std::sync::Arc;

#[derive(Debug, Clone, Copy, PartialEq, Eq, Default)]
pub enum PaginationLinkSize {
    Default,
    #[default]
    Icon,
}

fn alpha(color: Color, a: f32) -> Color {
    Color {
        a: a.clamp(0.0, 1.0),
        ..color
    }
}

fn radius(theme: &Theme) -> Px {
    MetricRef::radius(Radius::Md).resolve(theme)
}

fn icon_size(theme: &Theme) -> Px {
    ComponentSize::Medium.icon_button_size(theme)
}

fn button_h(theme: &Theme) -> Px {
    ComponentSize::Medium.button_h(theme)
}

fn border_color(theme: &Theme) -> Color {
    theme.color_required("border")
}

fn accent(theme: &Theme) -> Color {
    theme.color_required("accent")
}

fn accent_fg(theme: &Theme) -> Color {
    theme.color_required("accent-foreground")
}

fn base_fg(theme: &Theme) -> Color {
    theme.color_required("foreground")
}

fn disabled_fg(theme: &Theme) -> Color {
    alpha(base_fg(theme), 0.5)
}

#[derive(Debug, Clone)]
pub struct Pagination {
    children: Vec<AnyElement>,
    layout: LayoutRefinement,
}

impl Pagination {
    pub fn new(children: impl IntoIterator<Item = AnyElement>) -> Self {
        let children = children.into_iter().collect();
        Self {
            children,
            layout: LayoutRefinement::default().w_full(),
        }
    }

    pub fn refine_layout(mut self, layout: LayoutRefinement) -> Self {
        self.layout = self.layout.merge(layout);
        self
    }

    #[track_caller]
    pub fn into_element<H: UiHost>(self, cx: &mut ElementContext<'_, H>) -> AnyElement {
        let theme = Theme::global(&*cx.app).clone();
        let layout = decl_style::layout_style(&theme, self.layout);
        let children = self.children;

        cx.flex(
            FlexProps {
                layout,
                direction: fret_core::Axis::Horizontal,
                gap: Px(0.0),
                padding: Edges::all(Px(0.0)),
                justify: MainAlign::Center,
                align: CrossAlign::Center,
                wrap: false,
            },
            move |_cx| children,
        )
    }
}

#[derive(Debug, Clone)]
pub struct PaginationContent {
    children: Vec<AnyElement>,
}

impl PaginationContent {
    pub fn new(children: impl IntoIterator<Item = AnyElement>) -> Self {
        let children = children.into_iter().collect();
        Self { children }
    }

    #[track_caller]
    pub fn into_element<H: UiHost>(self, cx: &mut ElementContext<'_, H>) -> AnyElement {
        let theme = Theme::global(&*cx.app).clone();
        let gap = MetricRef::space(Space::N1).resolve(&theme);
        let children = self.children;

        cx.flex(
            FlexProps {
                layout: Default::default(),
                direction: fret_core::Axis::Horizontal,
                gap,
                padding: Edges::all(Px(0.0)),
                justify: MainAlign::Start,
                align: CrossAlign::Center,
                wrap: false,
            },
            move |_cx| children,
        )
    }
}

#[derive(Debug, Clone)]
pub struct PaginationItem {
    child: AnyElement,
}

impl PaginationItem {
    pub fn new(child: AnyElement) -> Self {
        Self { child }
    }

    #[track_caller]
    pub fn into_element<H: UiHost>(self, _cx: &mut ElementContext<'_, H>) -> AnyElement {
        self.child
    }
}

#[derive(Debug, Clone)]
pub struct PaginationLink {
    children: Vec<AnyElement>,
    size: PaginationLinkSize,
    is_active: bool,
    command: Option<CommandId>,
    disabled: bool,
}

impl PaginationLink {
    pub fn new(children: impl IntoIterator<Item = AnyElement>) -> Self {
        let children = children.into_iter().collect();
        Self {
            children,
            size: PaginationLinkSize::default(),
            is_active: false,
            command: None,
            disabled: false,
        }
    }

    pub fn size(mut self, size: PaginationLinkSize) -> Self {
        self.size = size;
        self
    }

    pub fn active(mut self, active: bool) -> Self {
        self.is_active = active;
        self
    }

    pub fn on_click(mut self, command: impl Into<CommandId>) -> Self {
        self.command = Some(command.into());
        self
    }

    pub fn disabled(mut self, disabled: bool) -> Self {
        self.disabled = disabled;
        self
    }

    #[track_caller]
    pub fn into_element<H: UiHost>(self, cx: &mut ElementContext<'_, H>) -> AnyElement {
        let theme = Theme::global(&*cx.app).clone();

        let r = radius(&theme);
        let gap = MetricRef::space(Space::N1).resolve(&theme);
        let px_2p5 = MetricRef::space(Space::N2p5).resolve(&theme);

        let enabled = self
            .command
            .as_ref()
            .is_some_and(|cmd| cx.command_is_enabled(cmd))
            && !self.disabled;
        let focus_ring = decl_style::focus_ring(&theme, r);

        let base_bg = if self.is_active {
            theme.color_required("background")
        } else {
            Color::TRANSPARENT
        };

        let base_border = if self.is_active {
            border_color(&theme)
        } else {
            Color::TRANSPARENT
        };

        let acc = accent(&theme);
        let hover_bg = alpha(acc, 1.0);
        let pressed_bg = alpha(acc, 1.0);

        let base_fg = if enabled {
            base_fg(&theme)
        } else {
            disabled_fg(&theme)
        };

        let children = std::rc::Rc::new(self.children);
        let children_for_content = children.clone();

        let (layout, padding, inner_gap, inner_wrap) = match self.size {
            PaginationLinkSize::Icon => {
                let s = icon_size(&theme);
                (
                    decl_style::layout_style(
                        &theme,
                        LayoutRefinement::default()
                            .w_px(s)
                            .h_px(s)
                            .flex_none()
                            .flex_shrink_0(),
                    ),
                    Edges::all(Px(0.0)),
                    Px(0.0),
                    false,
                )
            }
            PaginationLinkSize::Default => (
                decl_style::layout_style(
                    &theme,
                    LayoutRefinement::default()
                        .min_h(button_h(&theme))
                        .flex_none()
                        .flex_shrink_0(),
                ),
                Edges::symmetric(px_2p5, Px(0.0)),
                gap,
                false,
            ),
        };

        let content = move |cx: &mut ElementContext<'_, H>, hovered: bool, pressed: bool| {
            let bg = if !enabled {
                base_bg
            } else if pressed {
                pressed_bg
            } else if hovered {
                hover_bg
            } else {
                base_bg
            };

            let fg = if !enabled {
                base_fg
            } else if pressed || hovered {
                accent_fg(&theme)
            } else {
                base_fg
            };

            let inner_layout =
                decl_style::layout_style(&theme, LayoutRefinement::default().size_full());
            let children = children_for_content.clone();

            let row = cx.flex(
                FlexProps {
                    layout: inner_layout,
                    direction: fret_core::Axis::Horizontal,
                    gap: inner_gap,
                    padding: Edges::all(Px(0.0)),
                    justify: MainAlign::Center,
                    align: CrossAlign::Center,
                    wrap: inner_wrap,
                },
                move |_cx| {
                    let mut out = Vec::new();
                    for child in (*children).clone() {
                        let mut child = child;
                        if let fret_ui::element::ElementKind::Text(ref mut p) = child.kind {
                            p.color = Some(fg);
                        }
                        if let fret_ui::element::ElementKind::SvgIcon(ref mut p) = child.kind {
                            p.color = fg;
                        }
                        out.push(child);
                    }
                    out
                },
            );

            vec![cx.container(
                ContainerProps {
                    layout,
                    padding,
                    background: Some(bg),
                    border: Edges::all(Px(1.0)),
                    border_color: Some(base_border),
                    corner_radii: Corners::all(r),
                    ..Default::default()
                },
                move |_cx| vec![row],
            )]
        };

        cx.pressable(
            PressableProps {
                layout,
                enabled,
                focus_ring: Some(focus_ring),
                ..Default::default()
            },
            move |cx, st| {
                cx.pressable_dispatch_command_if_enabled_opt(self.command);
                content(cx, st.hovered, st.pressed)
            },
        )
    }
}

#[derive(Debug, Clone)]
pub struct PaginationPrevious {
    command: Option<CommandId>,
    disabled: bool,
    text: Option<Arc<str>>,
}

impl PaginationPrevious {
    pub fn new() -> Self {
        Self {
            command: None,
            disabled: false,
            text: None,
        }
    }

    pub fn on_click(mut self, command: impl Into<CommandId>) -> Self {
        self.command = Some(command.into());
        self
    }

    pub fn text(mut self, text: impl Into<Arc<str>>) -> Self {
        self.text = Some(text.into());
        self
    }

    pub fn disabled(mut self, disabled: bool) -> Self {
        self.disabled = disabled;
        self
    }

    #[track_caller]
    pub fn into_element<H: UiHost>(self, cx: &mut ElementContext<'_, H>) -> AnyElement {
        let dir = direction_prim::use_direction_in_scope(cx, None);
        let text = self.text.unwrap_or_else(|| Arc::<str>::from("Previous"));
        let chevron = if dir == direction_prim::LayoutDirection::Rtl {
            "lucide.chevron-right"
        } else {
            "lucide.chevron-left"
        };
        let icon = decl_icon::icon(cx, IconId::new_static(chevron));

        let children = if dir == direction_prim::LayoutDirection::Rtl {
            vec![cx.text(text), icon]
        } else {
            vec![icon, cx.text(text)]
        };

        let mut link = PaginationLink::new(children)
            .size(PaginationLinkSize::Default)
            .disabled(self.disabled);
        if let Some(command) = self.command {
            link = link.on_click(command);
        }
        link.into_element(cx)
    }
}

impl Default for PaginationPrevious {
    fn default() -> Self {
        Self::new()
    }
}

#[derive(Debug, Clone)]
pub struct PaginationNext {
    command: Option<CommandId>,
    disabled: bool,
    text: Option<Arc<str>>,
}

impl PaginationNext {
    pub fn new() -> Self {
        Self {
            command: None,
            disabled: false,
            text: None,
        }
    }

    pub fn on_click(mut self, command: impl Into<CommandId>) -> Self {
        self.command = Some(command.into());
        self
    }

    pub fn text(mut self, text: impl Into<Arc<str>>) -> Self {
        self.text = Some(text.into());
        self
    }

    pub fn disabled(mut self, disabled: bool) -> Self {
        self.disabled = disabled;
        self
    }

    #[track_caller]
    pub fn into_element<H: UiHost>(self, cx: &mut ElementContext<'_, H>) -> AnyElement {
        let dir = direction_prim::use_direction_in_scope(cx, None);
        let text = self.text.unwrap_or_else(|| Arc::<str>::from("Next"));
        let chevron = if dir == direction_prim::LayoutDirection::Rtl {
            "lucide.chevron-left"
        } else {
            "lucide.chevron-right"
        };
        let icon = decl_icon::icon(cx, IconId::new_static(chevron));

        let children = if dir == direction_prim::LayoutDirection::Rtl {
            vec![icon, cx.text(text)]
        } else {
            vec![cx.text(text), icon]
        };

        let mut link = PaginationLink::new(children)
            .size(PaginationLinkSize::Default)
            .disabled(self.disabled);
        if let Some(command) = self.command {
            link = link.on_click(command);
        }
        link.into_element(cx)
    }
}

impl Default for PaginationNext {
    fn default() -> Self {
        Self::new()
    }
}

#[derive(Debug, Clone)]
pub struct PaginationEllipsis;

impl PaginationEllipsis {
    pub fn new() -> Self {
        Self
    }

    #[track_caller]
    pub fn into_element<H: UiHost>(self, cx: &mut ElementContext<'_, H>) -> AnyElement {
        let theme = Theme::global(&*cx.app).clone();
        let s = icon_size(&theme);
        let layout = decl_style::layout_style(
            &theme,
            LayoutRefinement::default()
                .w_px(s)
                .h_px(s)
                .flex_none()
                .flex_shrink_0(),
        );
        let r = radius(&theme);
        cx.container(
            ContainerProps {
                layout,
                padding: Edges::all(Px(0.0)),
                background: None,
                border: Edges::all(Px(0.0)),
                border_color: None,
                corner_radii: Corners::all(r),
                ..Default::default()
            },
            move |cx| {
                let inner = cx.flex(
                    FlexProps {
                        layout: Default::default(),
                        direction: fret_core::Axis::Horizontal,
                        gap: Px(0.0),
                        padding: Edges::all(Px(0.0)),
                        justify: MainAlign::Center,
                        align: CrossAlign::Center,
                        wrap: false,
                    },
                    move |cx| vec![cx.text("…")],
                );
                vec![inner]
            },
        )
    }
}

impl Default for PaginationEllipsis {
    fn default() -> Self {
        Self::new()
    }
}
