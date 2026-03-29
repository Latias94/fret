use crate::{
    button::{ButtonVariant, variant_style},
    direction::LayoutDirection,
    overlay_motion, rtl,
};
use fret_core::{Color, Corners, Edges, Px, SemanticsRole};
use fret_runtime::CommandId;
use fret_ui::element::{
    AnyElement, ContainerProps, CrossAlign, FlexProps, MainAlign, PressableA11y,
    PressableKeyActivation, PressableProps, SemanticsDecoration, SemanticsProps,
};
use fret_ui::{ElementContext, Invalidation, Theme, ThemeSnapshot, UiHost};
use fret_ui_kit::command::ElementCommandGatingExt as _;
use fret_ui_kit::declarative::action_hooks::ActionHooksExt as _;
use fret_ui_kit::declarative::current_color;
use fret_ui_kit::declarative::icon as decl_icon;
use fret_ui_kit::declarative::motion::{
    drive_tween_color_for_element, drive_tween_f32_for_element,
};
use fret_ui_kit::declarative::style as decl_style;
use fret_ui_kit::declarative::viewport_queries;
use fret_ui_kit::{
    ColorRef, IntoUiElement, LayoutRefinement, MetricRef, Radius, Space, UiPatch, UiPatchTarget,
    UiSupportsLayout, WidgetStates,
};
use std::marker::PhantomData;
use std::sync::Arc;

#[derive(Debug, Clone, Copy, PartialEq, Eq, Default)]
pub enum PaginationLinkSize {
    Default,
    #[default]
    Icon,
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

fn base_fg(theme: &ThemeSnapshot) -> Color {
    theme.color_token("foreground")
}

#[derive(Debug, Clone, Copy)]
struct PaginationLinkChromeTargets {
    background: Color,
    foreground: Color,
    border_color: Color,
}

fn pagination_button_variant(is_active: bool) -> ButtonVariant {
    if is_active {
        ButtonVariant::Outline
    } else {
        ButtonVariant::Ghost
    }
}

fn pagination_link_chrome_targets(
    theme: &ThemeSnapshot,
    is_active: bool,
    states: WidgetStates,
) -> PaginationLinkChromeTargets {
    let variant = variant_style(pagination_button_variant(is_active));

    PaginationLinkChromeTargets {
        background: variant.background.resolve(states).resolve(theme),
        foreground: variant.foreground.resolve(states).resolve(theme),
        border_color: variant.border_color.resolve(states).resolve(theme),
    }
}

fn tailwind_transition_ease_in_out(t: f32) -> f32 {
    fret_ui_kit::headless::easing::CubicBezier::new(0.4, 0.0, 0.2, 1.0).sample(t)
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

    pub fn build<H: UiHost, B>(build: B) -> PaginationBuild<H, B>
    where
        B: FnOnce(&mut ElementContext<'_, H>, &mut Vec<AnyElement>),
    {
        PaginationBuild {
            build: Some(build),
            layout: LayoutRefinement::default().w_full(),
            _phantom: PhantomData,
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

    pub fn build<H: UiHost, B>(build: B) -> PaginationContentBuild<H, B>
    where
        B: FnOnce(&mut ElementContext<'_, H>, &mut Vec<AnyElement>),
    {
        PaginationContentBuild {
            build: Some(build),
            _phantom: PhantomData,
        }
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

    pub fn build<H: UiHost, T>(child: T) -> PaginationItemBuild<H, T>
    where
        T: IntoUiElement<H>,
    {
        PaginationItemBuild {
            child,
            _phantom: PhantomData,
        }
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

    pub fn build<H: UiHost, B>(build: B) -> PaginationLinkBuild<H, B>
    where
        B: FnOnce(&mut ElementContext<'_, H>, &mut Vec<AnyElement>),
    {
        PaginationLinkBuild {
            build: Some(build),
            size: PaginationLinkSize::default(),
            is_active: false,
            command: None,
            disabled: false,
            a11y_label: None,
            test_id: None,
            _phantom: PhantomData,
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

    /// Bind a stable action ID to this pagination link (action-first authoring).
    ///
    /// v1 compatibility: `ActionId` is `CommandId`-compatible (ADR 0307), so this dispatches
    /// through the existing command pipeline.
    pub fn action(mut self, action: impl Into<fret_runtime::ActionId>) -> Self {
        self.command = Some(action.into());
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

        let command_enabled = self
            .command
            .as_ref()
            .is_none_or(|cmd| cx.command_is_enabled(cmd));
        let visually_disabled = self.disabled || !command_enabled;
        let interactive = self.command.is_some() && !visually_disabled;
        let focus_ring = decl_style::focus_ring(&theme, r);
        let border_width = if self.is_active { Px(1.0) } else { Px(0.0) };
        let shadow = self.is_active.then(|| decl_style::shadow_xs(&theme, r));

        let children = self.children;
        let a11y_label = self.a11y_label;
        let test_id = self.test_id;
        let command = self.command;
        let is_active = self.is_active;

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

        let pressable = cx.pressable_with_id_props(move |cx, st, id| {
            let states = WidgetStates::from_pressable(cx, st, !visually_disabled);
            let chrome = pagination_link_chrome_targets(&theme, is_active, states);
            let duration = overlay_motion::shadcn_motion_duration_150(cx);

            let bg_motion = drive_tween_color_for_element(
                cx,
                id,
                "pagination.link.bg",
                chrome.background,
                duration,
                tailwind_transition_ease_in_out,
            );
            let fg_motion = drive_tween_color_for_element(
                cx,
                id,
                "pagination.link.fg",
                chrome.foreground,
                duration,
                tailwind_transition_ease_in_out,
            );
            let border_motion = drive_tween_color_for_element(
                cx,
                id,
                "pagination.link.border",
                chrome.border_color,
                duration,
                tailwind_transition_ease_in_out,
            );
            let ring_alpha = drive_tween_f32_for_element(
                cx,
                id,
                "pagination.link.ring.alpha",
                if states.contains(WidgetStates::FOCUS_VISIBLE) {
                    1.0
                } else {
                    0.0
                },
                duration,
                tailwind_transition_ease_in_out,
            );

            cx.pressable_dispatch_command_if_enabled_opt(command);

            let mut animated_focus_ring = focus_ring;
            animated_focus_ring.color.a =
                (animated_focus_ring.color.a * ring_alpha.value).clamp(0.0, 1.0);
            if let Some(offset_color) = animated_focus_ring.offset_color {
                animated_focus_ring.offset_color = Some(Color {
                    a: (offset_color.a * ring_alpha.value).clamp(0.0, 1.0),
                    ..offset_color
                });
            }

            let pressable_props = PressableProps {
                layout,
                enabled: interactive,
                focus_ring: Some(animated_focus_ring),
                focus_ring_always_paint: ring_alpha.animating,
                key_activation: PressableKeyActivation::EnterOnly,
                a11y: PressableA11y {
                    role: Some(SemanticsRole::Link),
                    label: a11y_label,
                    test_id,
                    selected: is_active,
                    ..Default::default()
                },
                ..Default::default()
            };

            let row = {
                let inner_layout =
                    decl_style::layout_style(&theme, LayoutRefinement::default().size_full());
                let children = current_color::scope_children(
                    cx,
                    ColorRef::Color(fg_motion.value),
                    move |_cx| children,
                );

                cx.flex(
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
                )
            };

            let container = cx.container(
                ContainerProps {
                    layout,
                    padding: padding.into(),
                    background: Some(bg_motion.value),
                    shadow,
                    border: Edges::all(border_width),
                    border_color: Some(border_motion.value),
                    corner_radii: Corners::all(r),
                    ..Default::default()
                },
                move |_cx| vec![row],
            );

            (pressable_props, vec![container])
        });

        if visually_disabled {
            cx.opacity(0.5, move |_cx| vec![pressable])
        } else {
            pressable
        }
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

    /// Bind a stable action ID to this pagination previous-link (action-first authoring).
    ///
    /// v1 compatibility: `ActionId` is `CommandId`-compatible (ADR 0307), so this dispatches
    /// through the existing command pipeline.
    pub fn action(mut self, action: impl Into<fret_runtime::ActionId>) -> Self {
        self.command = Some(action.into());
        self
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
        let dir = crate::direction::use_direction(cx, None);
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

    /// Bind a stable action ID to this pagination next-link (action-first authoring).
    ///
    /// v1 compatibility: `ActionId` is `CommandId`-compatible (ADR 0307), so this dispatches
    /// through the existing command pipeline.
    pub fn action(mut self, action: impl Into<fret_runtime::ActionId>) -> Self {
        self.command = Some(action.into());
        self
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
        let dir = crate::direction::use_direction(cx, None);
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

pub fn pagination<H: UiHost, I, F, T>(
    f: F,
) -> PaginationBuild<H, impl FnOnce(&mut ElementContext<'_, H>, &mut Vec<AnyElement>)>
where
    F: FnOnce(&mut ElementContext<'_, H>) -> I,
    I: IntoIterator<Item = T>,
    T: IntoUiElement<H>,
{
    Pagination::build(move |cx, out| {
        let children = f(cx);
        extend_landed_pagination_children(cx, out, children);
    })
}

pub fn pagination_content<H: UiHost, I, F, T>(
    f: F,
) -> PaginationContentBuild<H, impl FnOnce(&mut ElementContext<'_, H>, &mut Vec<AnyElement>)>
where
    F: FnOnce(&mut ElementContext<'_, H>) -> I,
    I: IntoIterator<Item = T>,
    T: IntoUiElement<H>,
{
    PaginationContent::build(move |cx, out| {
        let children = f(cx);
        extend_landed_pagination_children(cx, out, children);
    })
}

pub fn pagination_item<H: UiHost, T>(child: T) -> PaginationItemBuild<H, T>
where
    T: IntoUiElement<H>,
{
    PaginationItem::build(child)
}

pub fn pagination_link<H: UiHost, I, F, T>(
    f: F,
) -> PaginationLinkBuild<H, impl FnOnce(&mut ElementContext<'_, H>, &mut Vec<AnyElement>)>
where
    F: FnOnce(&mut ElementContext<'_, H>) -> I,
    I: IntoIterator<Item = T>,
    T: IntoUiElement<H>,
{
    PaginationLink::build(move |cx, out| {
        let children = f(cx);
        extend_landed_pagination_children(cx, out, children);
    })
}

pub struct PaginationBuild<H, B> {
    build: Option<B>,
    layout: LayoutRefinement,
    _phantom: PhantomData<fn() -> H>,
}

impl<H: UiHost, B> PaginationBuild<H, B>
where
    B: FnOnce(&mut ElementContext<'_, H>, &mut Vec<AnyElement>),
{
    pub fn refine_layout(mut self, layout: LayoutRefinement) -> Self {
        self.layout = self.layout.merge(layout);
        self
    }

    #[track_caller]
    pub fn into_element(self, cx: &mut ElementContext<'_, H>) -> AnyElement {
        let children = collect_built_pagination_children(
            cx,
            self.build.expect("expected pagination build closure"),
        );
        Pagination::new(children)
            .refine_layout(self.layout)
            .into_element(cx)
    }
}

impl<H: UiHost, B> UiPatchTarget for PaginationBuild<H, B>
where
    B: FnOnce(&mut ElementContext<'_, H>, &mut Vec<AnyElement>),
{
    fn apply_ui_patch(self, patch: UiPatch) -> Self {
        self.refine_layout(patch.layout)
    }
}

impl<H: UiHost, B> UiSupportsLayout for PaginationBuild<H, B> where
    B: FnOnce(&mut ElementContext<'_, H>, &mut Vec<AnyElement>)
{
}

impl<H: UiHost, B> IntoUiElement<H> for PaginationBuild<H, B>
where
    B: FnOnce(&mut ElementContext<'_, H>, &mut Vec<AnyElement>),
{
    #[track_caller]
    fn into_element(self, cx: &mut ElementContext<'_, H>) -> AnyElement {
        PaginationBuild::into_element(self, cx)
    }
}

pub struct PaginationContentBuild<H, B> {
    build: Option<B>,
    _phantom: PhantomData<fn() -> H>,
}

impl<H: UiHost, B> PaginationContentBuild<H, B>
where
    B: FnOnce(&mut ElementContext<'_, H>, &mut Vec<AnyElement>),
{
    #[track_caller]
    pub fn into_element(self, cx: &mut ElementContext<'_, H>) -> AnyElement {
        let children = collect_built_pagination_children(
            cx,
            self.build
                .expect("expected pagination content build closure"),
        );
        PaginationContent::new(children).into_element(cx)
    }
}

impl<H: UiHost, B> UiPatchTarget for PaginationContentBuild<H, B>
where
    B: FnOnce(&mut ElementContext<'_, H>, &mut Vec<AnyElement>),
{
    fn apply_ui_patch(self, _patch: UiPatch) -> Self {
        self
    }
}

impl<H: UiHost, B> IntoUiElement<H> for PaginationContentBuild<H, B>
where
    B: FnOnce(&mut ElementContext<'_, H>, &mut Vec<AnyElement>),
{
    #[track_caller]
    fn into_element(self, cx: &mut ElementContext<'_, H>) -> AnyElement {
        PaginationContentBuild::into_element(self, cx)
    }
}

pub struct PaginationItemBuild<H, T> {
    child: T,
    _phantom: PhantomData<fn() -> H>,
}

impl<H: UiHost, T> PaginationItemBuild<H, T>
where
    T: IntoUiElement<H>,
{
    #[track_caller]
    pub fn into_element(self, cx: &mut ElementContext<'_, H>) -> AnyElement {
        PaginationItem::new(self.child.into_element(cx)).into_element(cx)
    }
}

impl<H: UiHost, T> UiPatchTarget for PaginationItemBuild<H, T>
where
    T: IntoUiElement<H>,
{
    fn apply_ui_patch(self, _patch: UiPatch) -> Self {
        self
    }
}

impl<H: UiHost, T> IntoUiElement<H> for PaginationItemBuild<H, T>
where
    T: IntoUiElement<H>,
{
    #[track_caller]
    fn into_element(self, cx: &mut ElementContext<'_, H>) -> AnyElement {
        PaginationItemBuild::into_element(self, cx)
    }
}

pub struct PaginationLinkBuild<H, B> {
    build: Option<B>,
    size: PaginationLinkSize,
    is_active: bool,
    command: Option<CommandId>,
    disabled: bool,
    a11y_label: Option<Arc<str>>,
    test_id: Option<Arc<str>>,
    _phantom: PhantomData<fn() -> H>,
}

impl<H: UiHost, B> PaginationLinkBuild<H, B>
where
    B: FnOnce(&mut ElementContext<'_, H>, &mut Vec<AnyElement>),
{
    pub fn size(mut self, size: PaginationLinkSize) -> Self {
        self.size = size;
        self
    }

    pub fn active(mut self, active: bool) -> Self {
        self.is_active = active;
        self
    }

    /// Bind a stable action ID to this pagination link build wrapper (action-first authoring).
    ///
    /// v1 compatibility: `ActionId` is `CommandId`-compatible (ADR 0307), so this dispatches
    /// through the existing command pipeline.
    pub fn action(mut self, action: impl Into<fret_runtime::ActionId>) -> Self {
        self.command = Some(action.into());
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
    pub fn into_element(self, cx: &mut ElementContext<'_, H>) -> AnyElement {
        let children = collect_built_pagination_children(
            cx,
            self.build.expect("expected pagination link build closure"),
        );
        let mut link = PaginationLink::new(children)
            .size(self.size)
            .active(self.is_active)
            .disabled(self.disabled);
        if let Some(command) = self.command {
            link = link.on_click(command);
        }
        if let Some(label) = self.a11y_label {
            link = link.a11y_label(label);
        }
        if let Some(test_id) = self.test_id {
            link = link.test_id(test_id);
        }
        link.into_element(cx)
    }
}

impl<H: UiHost, B> UiPatchTarget for PaginationLinkBuild<H, B>
where
    B: FnOnce(&mut ElementContext<'_, H>, &mut Vec<AnyElement>),
{
    fn apply_ui_patch(self, _patch: UiPatch) -> Self {
        self
    }
}

impl<H: UiHost, B> IntoUiElement<H> for PaginationLinkBuild<H, B>
where
    B: FnOnce(&mut ElementContext<'_, H>, &mut Vec<AnyElement>),
{
    #[track_caller]
    fn into_element(self, cx: &mut ElementContext<'_, H>) -> AnyElement {
        PaginationLinkBuild::into_element(self, cx)
    }
}

fn collect_built_pagination_children<H: UiHost>(
    cx: &mut ElementContext<'_, H>,
    build: impl FnOnce(&mut ElementContext<'_, H>, &mut Vec<AnyElement>),
) -> Vec<AnyElement> {
    let mut out = Vec::new();
    build(cx, &mut out);
    out
}

fn extend_landed_pagination_children<H: UiHost, I, T>(
    cx: &mut ElementContext<'_, H>,
    out: &mut Vec<AnyElement>,
    children: I,
) where
    I: IntoIterator<Item = T>,
    T: IntoUiElement<H>,
{
    for child in children {
        out.push(child.into_element(cx));
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

    #[test]
    fn pagination_active_link_focus_visible_border_uses_ring_token() {
        let app = App::new();
        let theme = Theme::global(&app).snapshot();
        let chrome = pagination_link_chrome_targets(&theme, true, WidgetStates::FOCUS_VISIBLE);

        assert_eq!(
            chrome.border_color,
            theme.color_token("ring"),
            "expected active pagination link focus-visible border to follow the shared outline button ring token"
        );
    }

    #[test]
    fn pagination_link_without_action_keeps_enabled_visual_chrome() {
        let window = AppWindowId::default();
        let mut app = App::new();

        let el = fret_ui::elements::with_element_cx(&mut app, window, bounds(), "test", |cx| {
            PaginationLink::new([cx.text("1")]).into_element(cx)
        });

        assert!(
            !matches!(el.kind, ElementKind::Opacity(_)),
            "expected a pagination link without an app action to keep enabled visual chrome rather than looking disabled"
        );
    }

    #[test]
    fn pagination_disabled_link_wraps_in_opacity() {
        let window = AppWindowId::default();
        let mut app = App::new();

        let el = fret_ui::elements::with_element_cx(&mut app, window, bounds(), "test", |cx| {
            PaginationLink::new([cx.text("1")])
                .disabled(true)
                .into_element(cx)
        });

        let ElementKind::Opacity(props) = &el.kind else {
            panic!("expected disabled PaginationLink to render through an opacity wrapper");
        };
        assert!(
            (props.opacity - 0.5).abs() < 1e-6,
            "expected disabled PaginationLink opacity to match shadcn disabled opacity"
        );

        let Some(child) = el.children.first() else {
            panic!("expected opacity wrapper to contain the pressable");
        };
        assert!(
            matches!(child.kind, ElementKind::Pressable(_)),
            "expected opacity wrapper child to remain the pagination pressable"
        );
    }
}
