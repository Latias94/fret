use std::sync::Arc;

use crate::popper_arrow::{self, DiamondArrowStyle};
use fret_core::{
    FontId, FontWeight, Point, Px, SemanticsRole, Size, TextOverflow, TextStyle, TextWrap,
    Transform2D,
};
use fret_runtime::Model;
use fret_ui::element::{
    AnyElement, ContainerProps, LayoutStyle, Length, OpacityProps, Overflow, SemanticsProps,
    SizeStyle, TextProps, VisualTransformProps,
};
use fret_ui::overlay_placement::{Align, LayoutDirection, Side};
use fret_ui::{ElementContext, Theme, UiHost};
use fret_ui_kit::declarative::model_watch::ModelWatchExt as _;
use fret_ui_kit::declarative::style as decl_style;
use fret_ui_kit::overlay;
use fret_ui_kit::primitives::popper;
use fret_ui_kit::primitives::popper_content;
use fret_ui_kit::{
    ChromeRefinement, ColorRef, LayoutRefinement, MetricRef, OverlayController, OverlayPresence,
    OverlayRequest, Radius, Space,
};

use crate::layout as shadcn_layout;

#[derive(Debug, Clone, Copy, Default, PartialEq, Eq)]
pub enum PopoverAlign {
    Start,
    #[default]
    Center,
    End,
}

#[derive(Debug, Clone, Copy, Default, PartialEq, Eq)]
pub enum PopoverSide {
    Top,
    Right,
    #[default]
    Bottom,
    Left,
}

/// shadcn/ui `Popover` (v4).
///
/// This is a non-modal, dismissible overlay built on:
/// - per-window overlay roots (ADR 0067)
/// - click-through outside-press observer pass (ADR 0069)
#[derive(Clone)]
pub struct Popover {
    open: Model<bool>,
    align: PopoverAlign,
    side: PopoverSide,
    align_offset: Px,
    side_offset: Px,
    window_margin_override: Option<Px>,
    arrow: bool,
    arrow_size_override: Option<Px>,
    arrow_padding_override: Option<Px>,
    auto_focus: bool,
    initial_focus: Option<fret_ui::elements::GlobalElementId>,
    anchor_override: Option<fret_ui::elements::GlobalElementId>,
}

impl std::fmt::Debug for Popover {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.debug_struct("Popover")
            .field("open", &"<model>")
            .field("align", &self.align)
            .field("side", &self.side)
            .field("align_offset", &self.align_offset)
            .field("side_offset", &self.side_offset)
            .field("window_margin_override", &self.window_margin_override)
            .field("auto_focus", &self.auto_focus)
            .field("initial_focus", &self.initial_focus)
            .finish()
    }
}

impl Popover {
    pub fn new(open: Model<bool>) -> Self {
        Self {
            open,
            align: PopoverAlign::default(),
            side: PopoverSide::default(),
            align_offset: Px(0.0),
            side_offset: Px(4.0),
            window_margin_override: None,
            arrow: false,
            arrow_size_override: None,
            arrow_padding_override: None,
            auto_focus: false,
            initial_focus: None,
            anchor_override: None,
        }
    }

    pub fn align(mut self, align: PopoverAlign) -> Self {
        self.align = align;
        self
    }

    pub fn side(mut self, side: PopoverSide) -> Self {
        self.side = side;
        self
    }

    pub fn align_offset(mut self, offset: Px) -> Self {
        self.align_offset = offset;
        self
    }

    pub fn side_offset(mut self, offset: Px) -> Self {
        self.side_offset = offset;
        self
    }

    pub fn window_margin(mut self, margin: Px) -> Self {
        self.window_margin_override = Some(margin);
        self
    }

    /// Enables a Popover arrow (Radix `PopoverArrow`-style).
    ///
    /// Default: `false`.
    pub fn arrow(mut self, arrow: bool) -> Self {
        self.arrow = arrow;
        self
    }

    pub fn arrow_size(mut self, size: Px) -> Self {
        self.arrow_size_override = Some(size);
        self
    }

    pub fn arrow_padding(mut self, padding: Px) -> Self {
        self.arrow_padding_override = Some(padding);
        self
    }

    /// When enabled, focus the first focusable descendant inside the popover on open.
    ///
    /// Default: `false` (preserve trigger focus).
    pub fn auto_focus(mut self, auto_focus: bool) -> Self {
        self.auto_focus = auto_focus;
        self
    }

    pub fn initial_focus(mut self, element: fret_ui::elements::GlobalElementId) -> Self {
        self.initial_focus = Some(element);
        self
    }

    /// Override the element used as the placement anchor.
    ///
    /// Notes:
    /// - Dismissal and focus-restore policies still treat the trigger as the "interactive branch".
    /// - The anchor bounds are resolved from `ElementCx::last_bounds_for_element` / visual bounds,
    ///   so it may take one frame to stabilize after layout changes (same as trigger anchoring).
    pub fn anchor_element(mut self, element: fret_ui::elements::GlobalElementId) -> Self {
        self.anchor_override = Some(element);
        self
    }

    pub fn into_element<H: UiHost>(
        self,
        cx: &mut ElementContext<'_, H>,
        trigger: impl FnOnce(&mut ElementContext<'_, H>) -> AnyElement,
        content: impl FnOnce(&mut ElementContext<'_, H>) -> AnyElement,
    ) -> AnyElement {
        self.into_element_with_anchor(cx, trigger, move |cx, _anchor| content(cx))
    }

    pub fn into_element_with_anchor<H: UiHost>(
        self,
        cx: &mut ElementContext<'_, H>,
        trigger: impl FnOnce(&mut ElementContext<'_, H>) -> AnyElement,
        content: impl FnOnce(&mut ElementContext<'_, H>, fret_core::Rect) -> AnyElement,
    ) -> AnyElement {
        cx.scope(|cx| {
            let theme = Theme::global(&*cx.app).clone();
            let is_open = cx.watch_model(&self.open).copied().unwrap_or(false);

            let trigger = trigger(cx);
            let trigger_id = trigger.id;
            let anchor_id = self.anchor_override.unwrap_or(trigger_id);

            let presence = OverlayController::fade_presence(cx, is_open, 4);
            let overlay_presence = OverlayPresence::from_fade(is_open, presence);

            if overlay_presence.present {
                let overlay_root_name = OverlayController::popover_root_name(trigger_id);
                let align = self.align;
                let side = self.side;
                let align_offset = self.align_offset;
                let side_offset = self.side_offset;
                let window_margin = self.window_margin_override.unwrap_or_else(|| {
                    theme
                        .metric_by_key("component.popover.window_margin")
                        .unwrap_or(Px(8.0))
                });
                let arrow = self.arrow;
                let arrow_size = self.arrow_size_override.unwrap_or_else(|| {
                    theme
                        .metric_by_key("component.popover.arrow_size")
                        .unwrap_or(Px(12.0))
                });
                let arrow_padding = self.arrow_padding_override.unwrap_or_else(|| {
                    theme
                        .metric_by_key("component.popover.arrow_padding")
                        .unwrap_or(theme.metrics.radius_md)
                });

                let opacity = presence.opacity;
                let overlay_children = cx.with_root_name(&overlay_root_name, move |cx| {
                    let anchor = overlay::anchor_bounds_for_element(cx, anchor_id);
                    let Some(anchor) = anchor else {
                        return Vec::new();
                    };
                    let anchor_raw = anchor;

                    let content = content(cx, anchor_raw);
                    let content_id = content.id;

                    let last_content_size = cx.last_bounds_for_element(content_id).map(|r| r.size);
                    let estimated = Size::new(Px(288.0), Px(160.0));
                    let content_size = last_content_size.unwrap_or(estimated);

                    let align = match align {
                        PopoverAlign::Start => Align::Start,
                        PopoverAlign::Center => Align::Center,
                        PopoverAlign::End => Align::End,
                    };
                    let side = match side {
                        PopoverSide::Top => Side::Top,
                        PopoverSide::Right => Side::Right,
                        PopoverSide::Bottom => Side::Bottom,
                        PopoverSide::Left => Side::Left,
                    };

                    let (arrow_options, arrow_protrusion) =
                        popper::diamond_arrow_options(arrow, arrow_size, arrow_padding);

                    let layout = popper::popper_content_layout_sized(
                        overlay::outer_bounds_with_window_margin(cx.bounds, window_margin),
                        anchor,
                        content_size,
                        popper::PopperContentPlacement::new(
                            LayoutDirection::Ltr,
                            side,
                            align,
                            side_offset,
                        )
                        .with_align_offset(align_offset)
                        .with_arrow(arrow_options, arrow_protrusion),
                    );

                    let placed = layout.rect;
                    let wrapper_insets = popper_arrow::wrapper_insets(&layout, arrow_protrusion);
                    let wrapper_origin = Point::new(
                        Px(placed.origin.x.0 - wrapper_insets.left.0),
                        Px(placed.origin.y.0 - wrapper_insets.top.0),
                    );
                    let wrapper_size = Size::new(
                        Px(placed.size.width.0 + wrapper_insets.left.0 + wrapper_insets.right.0),
                        Px(placed.size.height.0 + wrapper_insets.top.0 + wrapper_insets.bottom.0),
                    );
                    let center = Point::new(
                        Px(wrapper_origin.x.0 + wrapper_size.width.0 * 0.5),
                        Px(wrapper_origin.y.0 + wrapper_size.height.0 * 0.5),
                    );

                    // shadcn/ui v4 uses a small zoom-in (95% -> 100%) plus opacity transitions.
                    // We approximate that with a fade-driven zoom transform about the wrapper center.
                    let scale = 0.95 + 0.05 * opacity.clamp(0.0, 1.0);
                    let zoom = Transform2D::translation(center)
                        * Transform2D::scale_uniform(scale)
                        * Transform2D::translation(Point::new(Px(-center.x.0), Px(-center.y.0)));

                    let bg = theme
                        .color_by_key("popover")
                        .or_else(|| theme.color_by_key("popover.background"))
                        .unwrap_or(theme.colors.panel_background);
                    let border = theme
                        .color_by_key("border")
                        .unwrap_or(theme.colors.panel_border);

                    let wrapper = popper_content::popper_wrapper_at_with_panel(
                        cx,
                        placed,
                        wrapper_insets,
                        Overflow::Visible,
                        move |_cx| vec![content],
                        move |cx, content| {
                            let arrow_el = popper_arrow::diamond_arrow_element(
                                cx,
                                &layout,
                                wrapper_insets,
                                arrow_size,
                                DiamondArrowStyle {
                                    bg,
                                    border: Some(border),
                                    border_width: Px(1.0),
                                },
                            );

                            if let Some(arrow_el) = arrow_el {
                                vec![arrow_el, content]
                            } else {
                                vec![content]
                            }
                        },
                    );

                    let opacity_layout = LayoutStyle {
                        size: SizeStyle {
                            width: Length::Fill,
                            height: Length::Fill,
                            ..Default::default()
                        },
                        ..Default::default()
                    };
                    vec![cx.opacity_props(
                        OpacityProps {
                            layout: opacity_layout.clone(),
                            opacity,
                        },
                        move |cx| {
                            vec![cx.visual_transform_props(
                                VisualTransformProps {
                                    layout: opacity_layout,
                                    transform: zoom,
                                },
                                move |_cx| vec![wrapper],
                            )]
                        },
                    )]
                });

                let initial_focus = if let Some(id) = self.initial_focus {
                    Some(id)
                } else if self.auto_focus {
                    None
                } else {
                    Some(trigger_id)
                };

                let mut request = OverlayRequest::dismissible_popover(
                    trigger_id,
                    trigger_id,
                    self.open,
                    overlay_presence,
                    overlay_children,
                );
                request.root_name = Some(overlay_root_name);
                request.initial_focus = initial_focus;
                OverlayController::request(cx, request);
            }

            trigger
        })
    }
}

/// shadcn/ui `PopoverTrigger` (v4).
#[derive(Debug, Clone)]
pub struct PopoverTrigger {
    child: AnyElement,
}

impl PopoverTrigger {
    pub fn new(child: AnyElement) -> Self {
        Self { child }
    }

    pub fn into_element<H: UiHost>(self, _cx: &mut ElementContext<'_, H>) -> AnyElement {
        self.child
    }
}

/// shadcn/ui `PopoverAnchor` (v4).
///
/// This is a layout-only helper. Use [`Popover::anchor_element`] to wire the anchor element ID
/// into placement.
#[derive(Debug, Clone)]
pub struct PopoverAnchor {
    child: AnyElement,
}

impl PopoverAnchor {
    pub fn new(child: AnyElement) -> Self {
        Self { child }
    }

    pub fn element_id(&self) -> fret_ui::elements::GlobalElementId {
        self.child.id
    }

    pub fn into_element<H: UiHost>(self, _cx: &mut ElementContext<'_, H>) -> AnyElement {
        self.child
    }
}

fn popover_content_chrome(theme: &Theme) -> ChromeRefinement {
    let bg = theme
        .color_by_key("popover")
        .or_else(|| theme.color_by_key("popover.background"))
        .unwrap_or(theme.colors.panel_background);
    let border = theme
        .color_by_key("border")
        .unwrap_or(theme.colors.panel_border);

    ChromeRefinement::default()
        .rounded(Radius::Md)
        .border_1()
        .bg(ColorRef::Color(bg))
        .border_color(ColorRef::Color(border))
        .p(Space::N4)
}

/// shadcn/ui `PopoverContent` (v4).
#[derive(Debug, Clone)]
pub struct PopoverContent {
    children: Vec<AnyElement>,
    chrome: ChromeRefinement,
    layout: LayoutRefinement,
    a11y_label: Option<Arc<str>>,
}

impl PopoverContent {
    pub fn new(children: Vec<AnyElement>) -> Self {
        Self {
            children,
            chrome: ChromeRefinement::default(),
            layout: LayoutRefinement::default().w_px(MetricRef::Px(Px(288.0))),
            a11y_label: None,
        }
    }

    pub fn refine_style(mut self, style: ChromeRefinement) -> Self {
        self.chrome = self.chrome.merge(style);
        self
    }

    pub fn refine_layout(mut self, layout: LayoutRefinement) -> Self {
        self.layout = self.layout.merge(layout);
        self
    }

    pub fn a11y_label(mut self, label: impl Into<Arc<str>>) -> Self {
        self.a11y_label = Some(label.into());
        self
    }

    pub fn into_element<H: UiHost>(self, cx: &mut ElementContext<'_, H>) -> AnyElement {
        let theme = Theme::global(&*cx.app).clone();
        let radius = theme.metrics.radius_md;
        let shadow = decl_style::shadow_md(&theme, radius);

        let chrome = popover_content_chrome(&theme).merge(self.chrome);
        let props = decl_style::container_props(&theme, chrome, self.layout);
        let children = self.children;
        let label = self.a11y_label;

        let container = shadcn_layout::container_vstack_gap(
            cx,
            ContainerProps {
                shadow: Some(shadow),
                ..props
            },
            Space::N4,
            children,
        );

        cx.semantics(
            SemanticsProps {
                role: SemanticsRole::Panel,
                label,
                ..Default::default()
            },
            move |_cx| vec![container],
        )
    }
}

/// shadcn/ui `PopoverHeader` (v4).
#[derive(Debug, Clone)]
pub struct PopoverHeader {
    children: Vec<AnyElement>,
}

impl PopoverHeader {
    pub fn new(children: Vec<AnyElement>) -> Self {
        Self { children }
    }

    pub fn into_element<H: UiHost>(self, cx: &mut ElementContext<'_, H>) -> AnyElement {
        let props = decl_style::container_props(
            Theme::global(&*cx.app),
            ChromeRefinement::default().pb(Space::N4),
            LayoutRefinement::default(),
        );
        let children = self.children;
        shadcn_layout::container_vstack_gap(cx, props, Space::N1p5, children)
    }
}

/// shadcn/ui `PopoverTitle` (v4).
#[derive(Debug, Clone)]
pub struct PopoverTitle {
    text: Arc<str>,
}

impl PopoverTitle {
    pub fn new(text: impl Into<Arc<str>>) -> Self {
        Self { text: text.into() }
    }

    pub fn into_element<H: UiHost>(self, cx: &mut ElementContext<'_, H>) -> AnyElement {
        let theme = Theme::global(&*cx.app).clone();
        let fg = theme
            .color_by_key("popover.foreground")
            .or_else(|| theme.color_by_key("popover-foreground"))
            .or_else(|| theme.color_by_key("foreground"))
            .unwrap_or(theme.colors.text_primary);

        let px = theme
            .metric_by_key("component.popover.title_px")
            .or_else(|| theme.metric_by_key("font.size"))
            .unwrap_or(theme.metrics.font_size);
        let line_height = theme
            .metric_by_key("component.popover.title_line_height")
            .or_else(|| theme.metric_by_key("font.line_height"))
            .unwrap_or(theme.metrics.font_line_height);

        cx.text_props(TextProps {
            layout: Default::default(),
            text: self.text,
            style: Some(TextStyle {
                font: FontId::default(),
                size: px,
                weight: FontWeight::SEMIBOLD,
                line_height: Some(line_height),
                letter_spacing_em: Some(-0.02),
            }),
            color: Some(fg),
            wrap: TextWrap::None,
            overflow: TextOverflow::Clip,
        })
    }
}

/// shadcn/ui `PopoverDescription` (v4).
#[derive(Debug, Clone)]
pub struct PopoverDescription {
    text: Arc<str>,
}

impl PopoverDescription {
    pub fn new(text: impl Into<Arc<str>>) -> Self {
        Self { text: text.into() }
    }

    pub fn into_element<H: UiHost>(self, cx: &mut ElementContext<'_, H>) -> AnyElement {
        let theme = Theme::global(&*cx.app).clone();
        let fg = theme
            .color_by_key("muted.foreground")
            .or_else(|| theme.color_by_key("muted-foreground"))
            .unwrap_or(theme.colors.text_muted);

        let px = theme
            .metric_by_key("component.popover.description_px")
            .or_else(|| theme.metric_by_key("font.size"))
            .unwrap_or(theme.metrics.font_size);
        let line_height = theme
            .metric_by_key("component.popover.description_line_height")
            .or_else(|| theme.metric_by_key("font.line_height"))
            .unwrap_or(theme.metrics.font_line_height);

        cx.text_props(TextProps {
            layout: Default::default(),
            text: self.text,
            style: Some(TextStyle {
                font: FontId::default(),
                size: px,
                weight: FontWeight::NORMAL,
                line_height: Some(line_height),
                letter_spacing_em: None,
            }),
            color: Some(fg),
            wrap: TextWrap::Word,
            overflow: TextOverflow::Clip,
        })
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::cell::Cell;
    use std::rc::Rc;

    use fret_app::App;
    use fret_core::{
        AppWindowId, Corners, MouseButton, PathCommand, Point, Rect, Size as CoreSize, SvgId,
        SvgService,
    };
    use fret_core::{PathConstraints, PathId, PathMetrics, PathService, PathStyle};
    use fret_core::{
        Px, TextBlobId, TextConstraints, TextMetrics, TextService, TextStyle as CoreTextStyle,
    };
    use fret_runtime::FrameId;
    use fret_ui::element::PressableProps;
    use fret_ui::UiTree;
    use fret_ui_kit::declarative::action_hooks::ActionHooksExt;

    #[derive(Default)]
    struct FakeServices;

    impl TextService for FakeServices {
        fn prepare(
            &mut self,
            _text: &str,
            _style: &CoreTextStyle,
            _constraints: TextConstraints,
        ) -> (TextBlobId, TextMetrics) {
            (
                TextBlobId::default(),
                TextMetrics {
                    size: CoreSize::new(Px(0.0), Px(0.0)),
                    baseline: Px(0.0),
                },
            )
        }

        fn release(&mut self, _blob: TextBlobId) {}
    }

    impl PathService for FakeServices {
        fn prepare(
            &mut self,
            _commands: &[PathCommand],
            _style: PathStyle,
            _constraints: PathConstraints,
        ) -> (PathId, PathMetrics) {
            (PathId::default(), PathMetrics::default())
        }

        fn release(&mut self, _path: PathId) {}
    }

    impl SvgService for FakeServices {
        fn register_svg(&mut self, _bytes: &[u8]) -> SvgId {
            SvgId::default()
        }

        fn unregister_svg(&mut self, _svg: SvgId) -> bool {
            true
        }
    }

    fn render_popover_frame(
        ui: &mut UiTree<App>,
        app: &mut App,
        services: &mut dyn fret_core::UiServices,
        window: AppWindowId,
        bounds: Rect,
        open: Model<bool>,
        arrow: bool,
        underlay_id_out: Rc<Cell<Option<fret_ui::elements::GlobalElementId>>>,
        popover_focus_id_out: Rc<Cell<Option<fret_ui::elements::GlobalElementId>>>,
        popover_content_id_out: Rc<Cell<Option<fret_ui::elements::GlobalElementId>>>,
    ) -> fret_ui::elements::GlobalElementId {
        OverlayController::begin_frame(app, window);

        let mut trigger_id: Option<fret_ui::elements::GlobalElementId> = None;

        let root =
            fret_ui::declarative::render_root(ui, app, services, window, bounds, "test", |cx| {
                let underlay = cx.pressable_with_id(
                    PressableProps {
                        layout: {
                            let mut layout = LayoutStyle::default();
                            layout.size.width = Length::Px(Px(120.0));
                            layout.size.height = Length::Px(Px(40.0));
                            layout.inset.top = Some(Px(300.0));
                            layout.inset.left = Some(Px(400.0));
                            layout.position = fret_ui::element::PositionStyle::Absolute;
                            layout
                        },
                        enabled: true,
                        focusable: true,
                        ..Default::default()
                    },
                    |cx, _st, id| {
                        underlay_id_out.set(Some(id));
                        vec![cx.container(ContainerProps::default(), |_cx| Vec::new())]
                    },
                );

                let trigger = cx.pressable_with_id(
                    PressableProps {
                        layout: {
                            let mut layout = LayoutStyle::default();
                            layout.size.width = Length::Px(Px(120.0));
                            layout.size.height = Length::Px(Px(40.0));
                            layout
                        },
                        enabled: true,
                        focusable: true,
                        ..Default::default()
                    },
                    |cx, _st, id| {
                        cx.pressable_toggle_bool(&open);
                        trigger_id = Some(id);
                        vec![cx.container(ContainerProps::default(), |_cx| Vec::new())]
                    },
                );

                let popover_focus_id_out = popover_focus_id_out.clone();
                let popover_content_id_out = popover_content_id_out.clone();
                let popover = Popover::new(open.clone())
                    .auto_focus(true)
                    .arrow(arrow)
                    .into_element(
                        cx,
                        |_cx| trigger,
                        move |cx| {
                            let focusable = cx.pressable_with_id(
                                PressableProps {
                                    layout: {
                                        let mut layout = LayoutStyle::default();
                                        layout.size.width = Length::Px(Px(160.0));
                                        layout.size.height = Length::Px(Px(44.0));
                                        layout
                                    },
                                    enabled: true,
                                    focusable: true,
                                    ..Default::default()
                                },
                                |cx, _st, id| {
                                    popover_focus_id_out.set(Some(id));
                                    vec![cx.container(ContainerProps::default(), |_cx| Vec::new())]
                                },
                            );
                            let content = PopoverContent::new(vec![focusable]).into_element(cx);
                            popover_content_id_out.set(Some(content.id));
                            content
                        },
                    );

                vec![underlay, popover]
            });

        ui.set_root(root);
        OverlayController::render(ui, app, services, window, bounds);
        trigger_id.expect("trigger id")
    }

    fn render_popover_in_clipped_surface_frame(
        ui: &mut UiTree<App>,
        app: &mut App,
        services: &mut dyn fret_core::UiServices,
        window: AppWindowId,
        bounds: Rect,
        open: Model<bool>,
        popover_content_id_out: Rc<Cell<Option<fret_ui::elements::GlobalElementId>>>,
    ) -> fret_ui::elements::GlobalElementId {
        OverlayController::begin_frame(app, window);

        let trigger_id_out: Rc<Cell<Option<fret_ui::elements::GlobalElementId>>> =
            Rc::new(Cell::new(None));

        let root =
            fret_ui::declarative::render_root(ui, app, services, window, bounds, "test", |cx| {
                let clipped_surface = cx.container(
                    ContainerProps {
                        layout: {
                            let mut layout = LayoutStyle::default();
                            layout.size.width = Length::Px(Px(200.0));
                            layout.size.height = Length::Px(Px(80.0));
                            layout.overflow = Overflow::Clip;
                            layout
                        },
                        corner_radii: Corners::all(Px(12.0)),
                        ..Default::default()
                    },
                    {
                        let trigger_id_out = trigger_id_out.clone();
                        move |cx| {
                            let popover_content_id_out = popover_content_id_out.clone();
                            vec![Popover::new(open.clone())
                                .side(PopoverSide::Bottom)
                                .into_element(
                                    cx,
                                    |cx| {
                                        cx.pressable_with_id(
                                            PressableProps {
                                                layout: {
                                                    let mut layout = LayoutStyle::default();
                                                    layout.size.width = Length::Px(Px(120.0));
                                                    layout.size.height = Length::Px(Px(40.0));
                                                    layout.position =
                                                        fret_ui::element::PositionStyle::Absolute;
                                                    layout.inset.top = Some(Px(20.0));
                                                    layout.inset.left = Some(Px(10.0));
                                                    layout
                                                },
                                                enabled: true,
                                                focusable: true,
                                                ..Default::default()
                                            },
                                            |cx, _st, id| {
                                                cx.pressable_toggle_bool(&open);
                                                trigger_id_out.set(Some(id));
                                                vec![cx
                                                    .container(ContainerProps::default(), |_cx| {
                                                        Vec::new()
                                                    })]
                                            },
                                        )
                                    },
                                    move |cx| {
                                        let focusable = cx.pressable_with_id(
                                            PressableProps {
                                                layout: {
                                                    let mut layout = LayoutStyle::default();
                                                    layout.size.width = Length::Px(Px(220.0));
                                                    layout.size.height = Length::Px(Px(120.0));
                                                    layout
                                                },
                                                enabled: true,
                                                focusable: true,
                                                ..Default::default()
                                            },
                                            |cx, _st, _id| {
                                                vec![cx
                                                    .container(ContainerProps::default(), |_cx| {
                                                        Vec::new()
                                                    })]
                                            },
                                        );
                                        let content =
                                            PopoverContent::new(vec![focusable]).into_element(cx);
                                        popover_content_id_out.set(Some(content.id));
                                        content
                                    },
                                )]
                        }
                    },
                );
                vec![clipped_surface]
            });

        ui.set_root(root);
        OverlayController::render(ui, app, services, window, bounds);
        trigger_id_out.get().expect("trigger id")
    }

    #[test]
    fn popover_outside_press_closes_without_overriding_new_focus() {
        let window = AppWindowId::default();
        let mut app = App::new();
        let mut ui: UiTree<App> = UiTree::new();
        ui.set_window(window);

        let open = app.models_mut().insert(false);
        let underlay_id: Rc<Cell<Option<fret_ui::elements::GlobalElementId>>> =
            Rc::new(Cell::new(None));
        let popover_focus_cell: Rc<Cell<Option<fret_ui::elements::GlobalElementId>>> =
            Rc::new(Cell::new(None));

        let mut services = FakeServices;
        let bounds = Rect::new(
            Point::new(Px(0.0), Px(0.0)),
            CoreSize::new(Px(800.0), Px(600.0)),
        );

        // First frame: closed, establish trigger bounds.
        app.set_frame_id(FrameId(1));
        let _trigger = render_popover_frame(
            &mut ui,
            &mut app,
            &mut services,
            window,
            bounds,
            open.clone(),
            false,
            underlay_id.clone(),
            popover_focus_cell.clone(),
            Rc::new(Cell::new(None)),
        );
        ui.layout_all(&mut app, &mut services, bounds, 1.0);

        // Open via trigger click.
        ui.dispatch_event(
            &mut app,
            &mut services,
            &fret_core::Event::Pointer(fret_core::PointerEvent::Down {
                position: Point::new(Px(10.0), Px(10.0)),
                button: fret_core::MouseButton::Left,
                modifiers: fret_core::Modifiers::default(),
            }),
        );
        ui.dispatch_event(
            &mut app,
            &mut services,
            &fret_core::Event::Pointer(fret_core::PointerEvent::Up {
                position: Point::new(Px(10.0), Px(10.0)),
                button: fret_core::MouseButton::Left,
                modifiers: fret_core::Modifiers::default(),
            }),
        );
        assert_eq!(app.models().get_copied(&open), Some(true));

        // Second frame: open + auto-focus inside popover.
        app.set_frame_id(FrameId(2));
        let _trigger = render_popover_frame(
            &mut ui,
            &mut app,
            &mut services,
            window,
            bounds,
            open.clone(),
            false,
            underlay_id.clone(),
            popover_focus_cell.clone(),
            Rc::new(Cell::new(None)),
        );
        ui.layout_all(&mut app, &mut services, bounds, 1.0);

        let popover_focus_element_id = popover_focus_cell.get().expect("popover focus element id");
        let popover_focus_node =
            fret_ui::elements::node_for_element(&mut app, window, popover_focus_element_id)
                .expect("popover focus node");
        assert_eq!(ui.focus(), Some(popover_focus_node));

        // Click the underlay while the popover is open: should close the popover (observer pass)
        // and still focus the underlay (click-through), without being overridden on close.
        ui.dispatch_event(
            &mut app,
            &mut services,
            &fret_core::Event::Pointer(fret_core::PointerEvent::Down {
                position: Point::new(Px(410.0), Px(310.0)),
                button: fret_core::MouseButton::Left,
                modifiers: fret_core::Modifiers::default(),
            }),
        );
        ui.dispatch_event(
            &mut app,
            &mut services,
            &fret_core::Event::Pointer(fret_core::PointerEvent::Up {
                position: Point::new(Px(410.0), Px(310.0)),
                button: fret_core::MouseButton::Left,
                modifiers: fret_core::Modifiers::default(),
            }),
        );
        assert_eq!(app.models().get_copied(&open), Some(false));

        // Third frame: popover hidden, focus should remain on the underlay.
        app.set_frame_id(FrameId(3));
        let _trigger = render_popover_frame(
            &mut ui,
            &mut app,
            &mut services,
            window,
            bounds,
            open.clone(),
            false,
            underlay_id.clone(),
            popover_focus_cell.clone(),
            Rc::new(Cell::new(None)),
        );
        ui.layout_all(&mut app, &mut services, bounds, 1.0);

        let underlay_id = underlay_id.get().expect("underlay element id");
        let underlay_node =
            fret_ui::elements::node_for_element(&mut app, window, underlay_id).expect("underlay");
        assert_eq!(ui.focus(), Some(underlay_node));
    }

    #[test]
    fn popover_portal_escapes_overflow_clip_ancestor() {
        let window = AppWindowId::default();
        let mut app = App::new();
        let mut ui: UiTree<App> = UiTree::new();
        ui.set_window(window);

        let open = app.models_mut().insert(false);
        let popover_content_cell: Rc<Cell<Option<fret_ui::elements::GlobalElementId>>> =
            Rc::new(Cell::new(None));

        let mut services = FakeServices;
        let bounds = Rect::new(
            Point::new(Px(0.0), Px(0.0)),
            CoreSize::new(Px(800.0), Px(600.0)),
        );

        // Frame 1: closed, establish trigger bounds for the placement solver.
        app.set_frame_id(FrameId(1));
        let trigger_id = render_popover_in_clipped_surface_frame(
            &mut ui,
            &mut app,
            &mut services,
            window,
            bounds,
            open.clone(),
            popover_content_cell.clone(),
        );
        ui.layout_all(&mut app, &mut services, bounds, 1.0);

        // Open via trigger click.
        let trigger_node = fret_ui::elements::node_for_element(&mut app, window, trigger_id)
            .expect("trigger node");
        let trigger_bounds = ui.debug_node_bounds(trigger_node).expect("trigger bounds");
        let click_point = Point::new(
            Px(trigger_bounds.origin.x.0 + 2.0),
            Px(trigger_bounds.origin.y.0 + 2.0),
        );

        let pre_hit = ui.debug_hit_test(click_point).hit.expect("pre-hit");
        let pre_path = ui.debug_node_path(pre_hit);
        assert!(
            pre_path.contains(&trigger_node),
            "expected click point to hit trigger subtree; point={click_point:?} hit={pre_hit:?} trigger={trigger_node:?} path={pre_path:?}"
        );
        ui.dispatch_event(
            &mut app,
            &mut services,
            &fret_core::Event::Pointer(fret_core::PointerEvent::Down {
                position: click_point,
                button: MouseButton::Left,
                modifiers: fret_core::Modifiers::default(),
            }),
        );
        ui.dispatch_event(
            &mut app,
            &mut services,
            &fret_core::Event::Pointer(fret_core::PointerEvent::Up {
                position: click_point,
                button: MouseButton::Left,
                modifiers: fret_core::Modifiers::default(),
            }),
        );
        assert_eq!(app.models().get_copied(&open), Some(true));

        // Frame 2: open, compute popover bounds and hit-test outside the clipped surface.
        app.set_frame_id(FrameId(2));
        let _trigger = render_popover_in_clipped_surface_frame(
            &mut ui,
            &mut app,
            &mut services,
            window,
            bounds,
            open.clone(),
            popover_content_cell.clone(),
        );
        ui.layout_all(&mut app, &mut services, bounds, 1.0);

        let content_id = popover_content_cell.get().expect("popover content id");
        let content_node = fret_ui::elements::node_for_element(&mut app, window, content_id)
            .expect("content node");
        let content_bounds = ui.debug_node_bounds(content_node).expect("content bounds");

        let clip_bottom = 80.0f32;
        let target_y = (clip_bottom + 5.0).max(content_bounds.origin.y.0 + 2.0);
        let point = Point::new(Px(content_bounds.origin.x.0 + 5.0), Px(target_y));
        assert!(
            content_bounds.contains(point),
            "expected point inside popover bounds; point={point:?} bounds={content_bounds:?}"
        );
        assert!(
            point.y.0 > clip_bottom,
            "expected point below the clipped surface; y={} clip_bottom={}",
            point.y.0,
            clip_bottom
        );

        let hit = ui
            .debug_hit_test(point)
            .hit
            .expect("expected hit in popover content outside clipped surface");
        let path = ui.debug_node_path(hit);
        assert!(
            path.contains(&content_node),
            "expected hit to be within popover content subtree; hit={hit:?} content={content_node:?} path={path:?}"
        );
    }

    #[test]
    fn popover_arrow_is_hit_testable_and_does_not_dismiss_on_click() {
        let window = AppWindowId::default();
        let mut app = App::new();
        let mut ui: UiTree<App> = UiTree::new();
        ui.set_window(window);

        let open = app.models_mut().insert(false);
        let underlay_id: Rc<Cell<Option<fret_ui::elements::GlobalElementId>>> =
            Rc::new(Cell::new(None));
        let popover_focus_cell: Rc<Cell<Option<fret_ui::elements::GlobalElementId>>> =
            Rc::new(Cell::new(None));
        let popover_content_cell: Rc<Cell<Option<fret_ui::elements::GlobalElementId>>> =
            Rc::new(Cell::new(None));

        let mut services = FakeServices;
        let bounds = Rect::new(
            Point::new(Px(0.0), Px(0.0)),
            CoreSize::new(Px(800.0), Px(600.0)),
        );

        // Frame 1: closed, establish trigger bounds.
        app.set_frame_id(FrameId(1));
        let _trigger = render_popover_frame(
            &mut ui,
            &mut app,
            &mut services,
            window,
            bounds,
            open.clone(),
            true,
            underlay_id.clone(),
            popover_focus_cell.clone(),
            popover_content_cell.clone(),
        );
        ui.layout_all(&mut app, &mut services, bounds, 1.0);

        // Open via trigger click.
        ui.dispatch_event(
            &mut app,
            &mut services,
            &fret_core::Event::Pointer(fret_core::PointerEvent::Down {
                position: Point::new(Px(10.0), Px(10.0)),
                button: MouseButton::Left,
                modifiers: fret_core::Modifiers::default(),
            }),
        );
        ui.dispatch_event(
            &mut app,
            &mut services,
            &fret_core::Event::Pointer(fret_core::PointerEvent::Up {
                position: Point::new(Px(10.0), Px(10.0)),
                button: MouseButton::Left,
                modifiers: fret_core::Modifiers::default(),
            }),
        );
        assert_eq!(app.models().get_copied(&open), Some(true));

        // Frame 2: open + arrow.
        app.set_frame_id(FrameId(2));
        let _trigger = render_popover_frame(
            &mut ui,
            &mut app,
            &mut services,
            window,
            bounds,
            open.clone(),
            true,
            underlay_id.clone(),
            popover_focus_cell.clone(),
            popover_content_cell.clone(),
        );
        ui.layout_all(&mut app, &mut services, bounds, 1.0);

        let content_id = popover_content_cell.get().expect("popover content id");
        let content_node = fret_ui::elements::node_for_element(&mut app, window, content_id)
            .expect("content node");
        let content_bounds = ui.debug_node_bounds(content_node).expect("content bounds");

        // Click just above the panel: this should land on the arrow and not trigger outside-press
        // dismissal.
        let click = Point::new(
            Px(content_bounds.origin.x.0 + content_bounds.size.width.0 * 0.5),
            Px(content_bounds.origin.y.0 - 1.0),
        );

        ui.dispatch_event(
            &mut app,
            &mut services,
            &fret_core::Event::Pointer(fret_core::PointerEvent::Down {
                position: click,
                button: MouseButton::Left,
                modifiers: fret_core::Modifiers::default(),
            }),
        );
        ui.dispatch_event(
            &mut app,
            &mut services,
            &fret_core::Event::Pointer(fret_core::PointerEvent::Up {
                position: click,
                button: MouseButton::Left,
                modifiers: fret_core::Modifiers::default(),
            }),
        );

        assert_eq!(app.models().get_copied(&open), Some(true));
    }

    #[test]
    fn popover_anchor_override_changes_anchor_rect_passed_to_content() {
        let window = AppWindowId::default();
        let mut app = App::new();
        let mut ui: UiTree<App> = UiTree::new();
        ui.set_window(window);

        let open_model = app.models_mut().insert(false);
        let anchor_id_out: Rc<Cell<Option<fret_ui::elements::GlobalElementId>>> =
            Rc::new(Cell::new(None));
        let anchor_rect_out: Rc<Cell<Option<Rect>>> = Rc::new(Cell::new(None));

        let mut services = FakeServices;
        let bounds = Rect::new(
            Point::new(Px(0.0), Px(0.0)),
            CoreSize::new(Px(800.0), Px(600.0)),
        );

        let render =
            |ui: &mut UiTree<App>, app: &mut App, services: &mut dyn fret_core::UiServices| {
                OverlayController::begin_frame(app, window);
                let anchor_id_out_for_frame = anchor_id_out.clone();
                let anchor_rect_out_for_frame = anchor_rect_out.clone();
                let open = open_model.clone();

                let root = fret_ui::declarative::render_root(
                    ui,
                    app,
                    services,
                    window,
                    bounds,
                    "test",
                    |cx| {
                        let anchor_id_out_for_anchor = anchor_id_out_for_frame.clone();
                        let anchor = cx.pressable_with_id(
                            PressableProps {
                                layout: {
                                    let mut layout = LayoutStyle::default();
                                    layout.size.width = Length::Px(Px(50.0));
                                    layout.size.height = Length::Px(Px(10.0));
                                    layout.inset.top = Some(Px(120.0));
                                    layout.inset.left = Some(Px(240.0));
                                    layout.position = fret_ui::element::PositionStyle::Absolute;
                                    layout
                                },
                                enabled: false,
                                focusable: false,
                                ..Default::default()
                            },
                            move |_cx, _st, id| {
                                anchor_id_out_for_anchor.set(Some(id));
                                vec![]
                            },
                        );

                        let anchor_id = anchor_id_out_for_frame.get().expect("anchor id");
                        let popover = Popover::new(open.clone())
                            .anchor_element(anchor_id)
                            .into_element_with_anchor(
                                cx,
                                move |cx| {
                                    let open = open.clone();
                                    cx.pressable(
                                        PressableProps {
                                            layout: {
                                                let mut layout = LayoutStyle::default();
                                                layout.size.width = Length::Px(Px(120.0));
                                                layout.size.height = Length::Px(Px(40.0));
                                                layout
                                            },
                                            enabled: true,
                                            focusable: true,
                                            ..Default::default()
                                        },
                                        move |cx, _st| {
                                            cx.pressable_toggle_bool(&open);
                                            vec![]
                                        },
                                    )
                                },
                                move |cx, anchor_rect| {
                                    anchor_rect_out_for_frame.set(Some(anchor_rect));
                                    PopoverContent::new(vec![]).into_element(cx)
                                },
                            );

                        vec![anchor, popover]
                    },
                );

                ui.set_root(root);
                OverlayController::render(ui, app, services, window, bounds);
            };

        // Frame 1: closed, establish stable last-bounds for the anchor element.
        app.set_frame_id(FrameId(1));
        render(&mut ui, &mut app, &mut services);
        ui.layout_all(&mut app, &mut services, bounds, 1.0);

        // Open via trigger click.
        ui.dispatch_event(
            &mut app,
            &mut services,
            &fret_core::Event::Pointer(fret_core::PointerEvent::Down {
                position: Point::new(Px(12.0), Px(12.0)),
                button: MouseButton::Left,
                modifiers: fret_core::Modifiers::default(),
            }),
        );
        ui.dispatch_event(
            &mut app,
            &mut services,
            &fret_core::Event::Pointer(fret_core::PointerEvent::Up {
                position: Point::new(Px(12.0), Px(12.0)),
                button: MouseButton::Left,
                modifiers: fret_core::Modifiers::default(),
            }),
        );
        assert_eq!(app.models().get_copied(&open_model), Some(true));

        // Frame 2: open, content closure should observe the anchor override rect.
        app.set_frame_id(FrameId(2));
        render(&mut ui, &mut app, &mut services);
        ui.layout_all(&mut app, &mut services, bounds, 1.0);

        let anchor_rect = anchor_rect_out.get().expect("anchor rect");
        assert_eq!(
            anchor_rect,
            Rect::new(
                Point::new(Px(240.0), Px(120.0)),
                CoreSize::new(Px(50.0), Px(10.0))
            )
        );
    }
}
