use crate::LayoutDirection;
use fret_core::{Color, Corners, Edges, Px, SemanticsRole};
use fret_runtime::CommandId;
use fret_ui::element::{
    AnyElement, ContainerProps, CrossAlign, FlexProps, MainAlign, PressableA11y,
    PressableKeyActivation, PressableProps, SemanticsDecoration, SemanticsProps,
};
use fret_ui::{ElementContext, Invalidation, Theme, ThemeSnapshot, UiHost};
use fret_ui_kit::command::ElementCommandGatingExt as _;
use fret_ui_kit::declarative::action_hooks::ActionHooksExt as _;
use fret_ui_kit::declarative::icon as decl_icon;
use fret_ui_kit::declarative::style as decl_style;
use fret_ui_kit::declarative::viewport_queries;
use fret_ui_kit::{LayoutRefinement, MetricRef, Radius, Space};
use std::sync::Arc;

use crate::rtl;

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

fn radius(theme: &ThemeSnapshot) -> Px {
    MetricRef::radius(Radius::Md).resolve(theme)
}

fn button_h(theme: &ThemeSnapshot) -> Px {
    // shadcn/ui v4 button default (`h-9`).
    theme
        .metric_by_key("component.size.md.button.h")
        .unwrap_or(Px(36.0))
}

fn icon_button_size(theme: &ThemeSnapshot) -> Px {
    // shadcn/ui v4 icon button default (`size-9`).
    theme
        .metric_by_key("component.size.md.icon_button.size")
        .unwrap_or(Px(36.0))
}

fn border_color(theme: &ThemeSnapshot) -> Color {
    theme.color_token("border")
}

fn accent(theme: &ThemeSnapshot) -> Color {
    theme.color_token("accent")
}

fn accent_fg(theme: &ThemeSnapshot) -> Color {
    theme.color_token("accent-foreground")
}

fn base_fg(theme: &ThemeSnapshot) -> Color {
    theme.color_token("foreground")
}

fn disabled_fg(theme: &ThemeSnapshot) -> Color {
    alpha(base_fg(theme), 0.5)
}

#[derive(Debug)]
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
        let theme = Theme::global(&*cx.app).snapshot();
        let layout = decl_style::layout_style(&theme, self.layout);
        let children = self.children;

        let el = cx.flex(
            FlexProps {
                layout,
                direction: fret_core::Axis::Horizontal,
                gap: Px(0.0).into(),
                padding: Edges::all(Px(0.0)).into(),
                justify: MainAlign::Center,
                align: CrossAlign::Center,
                wrap: false,
            },
            move |_cx| children,
        );

        el.attach_semantics(
            SemanticsDecoration::default()
                .role(SemanticsRole::Region)
                .label("pagination"),
        )
    }
}

#[derive(Debug)]
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
        let theme = Theme::global(&*cx.app).snapshot();
        let gap = MetricRef::space(Space::N1).resolve(&theme);
        let children = self.children;

        cx.flex(
            FlexProps {
                layout: Default::default(),
                direction: fret_core::Axis::Horizontal,
                gap: gap.into(),
                padding: Edges::all(Px(0.0)).into(),
                justify: MainAlign::Start,
                align: CrossAlign::Center,
                wrap: false,
            },
            move |_cx| children,
        )
        .attach_semantics(SemanticsDecoration::default().role(SemanticsRole::List))
    }
}

#[derive(Debug)]
pub struct PaginationItem {
    child: AnyElement,
}

impl PaginationItem {
    pub fn new(child: AnyElement) -> Self {
        Self { child }
    }

    #[track_caller]
    pub fn into_element<H: UiHost>(self, cx: &mut ElementContext<'_, H>) -> AnyElement {
        let child = self.child;
        cx.semantics(
            SemanticsProps {
                role: SemanticsRole::ListItem,
                ..Default::default()
            },
            move |_cx| vec![child],
        )
    }
}

#[derive(Debug)]
pub struct PaginationLink {
    children: Vec<AnyElement>,
    size: PaginationLinkSize,
    is_active: bool,
    command: Option<CommandId>,
    disabled: bool,
    a11y_label: Option<Arc<str>>,
    test_id: Option<Arc<str>>,
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
            a11y_label: None,
            test_id: None,
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

    pub fn a11y_label(mut self, label: impl Into<Arc<str>>) -> Self {
        self.a11y_label = Some(label.into());
        self
    }

    pub fn test_id(mut self, id: impl Into<Arc<str>>) -> Self {
        self.test_id = Some(id.into());
        self
    }

    #[track_caller]
    pub fn into_element<H: UiHost>(self, cx: &mut ElementContext<'_, H>) -> AnyElement {
        let theme = Theme::global(&*cx.app).snapshot();
        let icon_size = icon_button_size(&theme);
        let button_h = button_h(&theme);

        let r = radius(&theme);
        let gap = MetricRef::space(Space::N1).resolve(&theme);
        let px_2p5 = MetricRef::space(Space::N2p5).resolve(&theme);
        let py_2 = MetricRef::space(Space::N2).resolve(&theme);

        let enabled = self
            .command
            .as_ref()
            .is_some_and(|cmd| cx.command_is_enabled(cmd))
            && !self.disabled;
        let focus_ring = decl_style::focus_ring(&theme, r);

        let base_bg = if self.is_active {
            theme.color_token("background")
        } else {
            Color::TRANSPARENT
        };

        let border_width = if self.is_active { Px(1.0) } else { Px(0.0) };
        let base_border = self.is_active.then(|| border_color(&theme));
        let shadow = self.is_active.then(|| decl_style::shadow_xs(&theme, r));

        let acc = accent(&theme);
        let hover_bg = alpha(acc, 1.0);
        let pressed_bg = alpha(acc, 1.0);

        let base_fg = if enabled {
            base_fg(&theme)
        } else {
            disabled_fg(&theme)
        };

        let children = self.children;
        let a11y_label = self.a11y_label;
        let test_id = self.test_id;

        let (layout, padding, inner_gap, inner_wrap) = match self.size {
            PaginationLinkSize::Icon => (
                decl_style::layout_style(
                    &theme,
                    LayoutRefinement::default()
                        .w_px(icon_size)
                        .h_px(icon_size)
                        .flex_none()
                        .flex_shrink_0(),
                ),
                Edges::all(Px(0.0)),
                Px(0.0),
                false,
            ),
            PaginationLinkSize::Default => (
                decl_style::layout_style(
                    &theme,
                    LayoutRefinement::default()
                        .h_px(button_h)
                        .min_h(button_h)
                        .flex_none()
                        .flex_shrink_0(),
                ),
                Edges::symmetric(px_2p5, py_2),
                gap,
                false,
            ),
        };

        cx.pressable(
            PressableProps {
                layout,
                enabled,
                focus_ring: Some(focus_ring),
                key_activation: PressableKeyActivation::EnterOnly,
                a11y: PressableA11y {
                    role: Some(SemanticsRole::Link),
                    label: a11y_label,
                    test_id,
                    selected: self.is_active,
                    ..Default::default()
                },
                ..Default::default()
            },
            move |cx, st| {
                cx.pressable_dispatch_command_if_enabled_opt(self.command);

                let bg = if !enabled {
                    base_bg
                } else if st.pressed {
                    pressed_bg
                } else if st.hovered {
                    hover_bg
                } else {
                    base_bg
                };

                let fg = if !enabled {
                    base_fg
                } else if st.pressed || st.hovered {
                    accent_fg(&theme)
                } else {
                    base_fg
                };

                let inner_layout =
                    decl_style::layout_style(&theme, LayoutRefinement::default().size_full());

                let mut children = children;
                for child in &mut children {
                    if let fret_ui::element::ElementKind::Text(ref mut p) = child.kind {
                        p.color = Some(fg);
                    }
                    if let fret_ui::element::ElementKind::SvgIcon(ref mut p) = child.kind {
                        p.color = fg;
                    }
                }

                let row = cx.flex(
                    FlexProps {
                        layout: inner_layout,
                        direction: fret_core::Axis::Horizontal,
                        gap: inner_gap.into(),
                        padding: Edges::all(Px(0.0)).into(),
                        justify: MainAlign::Center,
                        align: CrossAlign::Center,
                        wrap: inner_wrap,
                    },
                    move |_cx| children,
                );

                vec![cx.container(
                    ContainerProps {
                        layout,
                        padding: padding.into(),
                        background: Some(bg),
                        shadow,
                        border: Edges::all(border_width),
                        border_color: base_border,
                        corner_radii: Corners::all(r),
                        ..Default::default()
                    },
                    move |_cx| vec![row],
                )]
            },
        )
    }
}

#[derive(Debug, Clone)]
pub struct PaginationPrevious {
    command: Option<CommandId>,
    disabled: bool,
    text: Option<Arc<str>>,
    test_id: Option<Arc<str>>,
}

impl PaginationPrevious {
    pub fn new() -> Self {
        Self {
            command: None,
            disabled: false,
            text: None,
            test_id: None,
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

    pub fn test_id(mut self, id: impl Into<Arc<str>>) -> Self {
        self.test_id = Some(id.into());
        self
    }

    #[track_caller]
    pub fn into_element<H: UiHost>(self, cx: &mut ElementContext<'_, H>) -> AnyElement {
        let dir = crate::use_direction(cx, None);
        let text = self.text.unwrap_or_else(|| Arc::<str>::from("Previous"));
        let show_text = viewport_queries::viewport_width_at_least(
            cx,
            Invalidation::Layout,
            viewport_queries::tailwind::SM,
            Default::default(),
        );
        let icon = decl_icon::icon(cx, rtl::chevron_inline_start(dir));

        let mut children = Vec::with_capacity(2);
        if dir == LayoutDirection::Rtl {
            if show_text {
                children.push(cx.text(text));
            }
            children.push(icon);
        } else {
            children.push(icon);
            if show_text {
                children.push(cx.text(text));
            }
        }

        let mut link = PaginationLink::new(children)
            .size(PaginationLinkSize::Default)
            .a11y_label("Go to previous page")
            .disabled(self.disabled);
        if let Some(id) = self.test_id {
            link = link.test_id(id);
        }
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
    test_id: Option<Arc<str>>,
}

impl PaginationNext {
    pub fn new() -> Self {
        Self {
            command: None,
            disabled: false,
            text: None,
            test_id: None,
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

    pub fn test_id(mut self, id: impl Into<Arc<str>>) -> Self {
        self.test_id = Some(id.into());
        self
    }

    #[track_caller]
    pub fn into_element<H: UiHost>(self, cx: &mut ElementContext<'_, H>) -> AnyElement {
        let dir = crate::use_direction(cx, None);
        let text = self.text.unwrap_or_else(|| Arc::<str>::from("Next"));
        let show_text = viewport_queries::viewport_width_at_least(
            cx,
            Invalidation::Layout,
            viewport_queries::tailwind::SM,
            Default::default(),
        );
        let icon = decl_icon::icon(cx, rtl::chevron_inline_end(dir));

        let mut children = Vec::with_capacity(2);
        if dir == LayoutDirection::Rtl {
            children.push(icon);
            if show_text {
                children.push(cx.text(text));
            }
        } else {
            if show_text {
                children.push(cx.text(text));
            }
            children.push(icon);
        }

        let mut link = PaginationLink::new(children)
            .size(PaginationLinkSize::Default)
            .a11y_label("Go to next page")
            .disabled(self.disabled);
        if let Some(id) = self.test_id {
            link = link.test_id(id);
        }
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
        let theme = Theme::global(&*cx.app).snapshot();
        let s = icon_button_size(&theme);
        let fg = base_fg(&theme);
        let layout = decl_style::layout_style(
            &theme,
            LayoutRefinement::default()
                .w_px(s)
                .h_px(s)
                .flex_none()
                .flex_shrink_0(),
        );
        let r = radius(&theme);
        let el = cx.container(
            ContainerProps {
                layout,
                padding: Edges::all(Px(0.0)).into(),
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
                        gap: Px(0.0).into(),
                        padding: Edges::all(Px(0.0)).into(),
                        justify: MainAlign::Center,
                        align: CrossAlign::Center,
                        wrap: false,
                    },
                    move |cx| {
                        vec![decl_icon::icon_with(
                            cx,
                            fret_icons::ids::ui::MORE_HORIZONTAL,
                            Some(Px(16.0)),
                            Some(fret_ui_kit::ColorRef::Color(fg)),
                        )]
                    },
                );
                vec![inner]
            },
        );

        el.attach_semantics(
            SemanticsDecoration::default()
                .role(SemanticsRole::Text)
                .label("More pages")
                .hidden(true),
        )
    }
}

impl Default for PaginationEllipsis {
    fn default() -> Self {
        Self::new()
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    use fret_app::App;
    use fret_core::{AppWindowId, Point, Px, Rect, Size};
    use fret_ui::element::ElementKind;

    fn bounds() -> Rect {
        Rect::new(
            Point::new(Px(0.0), Px(0.0)),
            Size::new(Px(300.0), Px(160.0)),
        )
    }

    #[test]
    fn pagination_root_is_w_full_and_labeled() {
        let window = AppWindowId::default();
        let mut app = App::new();

        let el = fret_ui::elements::with_element_cx(&mut app, window, bounds(), "test", |cx| {
            Pagination::new([PaginationContent::new([PaginationItem::new(
                PaginationLink::new([cx.text("1")]).into_element(cx),
            )
            .into_element(cx)])
            .into_element(cx)])
            .into_element(cx)
        });

        let ElementKind::Flex(props) = &el.kind else {
            panic!("expected Pagination to build a Flex element");
        };
        assert_eq!(
            props.layout.size.width,
            fret_ui::element::Length::Fill,
            "expected Pagination to default to w-full"
        );
        assert_eq!(
            el.semantics_decoration
                .as_ref()
                .and_then(|d| d.label.as_deref()),
            Some("pagination"),
            "expected Pagination to attach an a11y label"
        );
        assert_eq!(
            el.semantics_decoration.as_ref().and_then(|d| d.role),
            Some(SemanticsRole::Region),
            "expected Pagination to approximate the upstream navigation landmark with Region semantics"
        );
    }

    #[test]
    fn pagination_content_and_item_emit_list_semantics() {
        let window = AppWindowId::default();
        let mut app = App::new();

        let el = fret_ui::elements::with_element_cx(&mut app, window, bounds(), "test", |cx| {
            PaginationContent::new([PaginationItem::new(
                PaginationLink::new([cx.text("1")]).into_element(cx),
            )
            .into_element(cx)])
            .into_element(cx)
        });

        assert_eq!(
            el.semantics_decoration.as_ref().and_then(|d| d.role),
            Some(SemanticsRole::List),
            "expected PaginationContent to approximate upstream <ul> semantics"
        );

        let Some(item) = el.children.first() else {
            panic!("expected PaginationContent to render one child");
        };
        let ElementKind::Semantics(props) = &item.kind else {
            panic!("expected PaginationItem to wrap its child in a semantics node");
        };
        assert_eq!(
            props.role,
            SemanticsRole::ListItem,
            "expected PaginationItem to approximate upstream <li> semantics"
        );
    }

    #[test]
    fn pagination_link_active_stamps_selected() {
        let window = AppWindowId::default();
        let mut app = App::new();

        let el = fret_ui::elements::with_element_cx(&mut app, window, bounds(), "test", |cx| {
            PaginationLink::new([cx.text("1")])
                .active(true)
                .into_element(cx)
        });

        let ElementKind::Pressable(props) = &el.kind else {
            panic!("expected PaginationLink to build a Pressable element");
        };
        assert!(
            props.a11y.selected,
            "expected PaginationLink(active=true) to set a11y.selected"
        );
    }

    #[test]
    fn pagination_ellipsis_is_hidden_in_semantics_tree() {
        let window = AppWindowId::default();
        let mut app = App::new();

        let el = fret_ui::elements::with_element_cx(&mut app, window, bounds(), "test", |cx| {
            PaginationEllipsis::new().into_element(cx)
        });

        assert_eq!(
            el.semantics_decoration.as_ref().and_then(|d| d.hidden),
            Some(true),
            "expected PaginationEllipsis to be semantics-hidden (aria-hidden parity)"
        );
    }
}
