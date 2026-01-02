use fret_components_ui::declarative::scheduling;
use fret_components_ui::declarative::style as decl_style;
use fret_components_ui::headless::hover_intent::{HoverIntentConfig, HoverIntentState};
use fret_components_ui::overlay;
use fret_components_ui::{
    ChromeRefinement, ColorRef, LayoutRefinement, MetricRef, OverlayController, OverlayPresence,
    OverlayRequest, Radius, Space,
};
use std::sync::Arc;

use fret_core::{Corners, Edges, Point, Px, Size, TextOverflow, TextStyle, TextWrap, Transform2D};
use fret_ui::element::{
    AnyElement, ContainerProps, HoverRegionProps, InsetStyle, LayoutStyle, Length, Overflow,
    PositionStyle, SizeStyle, TextProps, VisualTransformProps,
};
use fret_ui::overlay_placement::{
    Align, AnchoredPanelOptions, ArrowOptions, LayoutDirection, Offset, Side,
};
use fret_ui::{ElementContext, Theme, UiHost};

fn tooltip_content_chrome(theme: &Theme) -> ChromeRefinement {
    // shadcn/ui v4 (2025-09-22): tooltip uses `bg-foreground text-background`.
    let bg = theme
        .color_by_key("foreground")
        .unwrap_or(theme.colors.text_primary);

    ChromeRefinement::default()
        .rounded(Radius::Sm)
        .bg(ColorRef::Color(bg))
        .px(Space::N3)
        .py(Space::N2)
}

#[derive(Debug, Clone, Copy, Default, PartialEq, Eq)]
pub enum TooltipAlign {
    Start,
    #[default]
    Center,
    End,
}

#[derive(Debug, Clone, Copy, Default, PartialEq, Eq)]
pub enum TooltipSide {
    #[default]
    Top,
    Right,
    Bottom,
    Left,
}

/// shadcn/ui `Tooltip` root (v4).
///
/// This is implemented as a component-layer policy built on runtime substrate primitives:
/// - `HoverRegion` (hover tracking)
/// - cross-frame geometry queries (`elements::bounds_for_element`)
/// - placement solver (`overlay_placement`)
///
/// Note: This uses a per-window overlay root, so it is not clipped by ancestors with
/// `overflow: Clip`.
#[derive(Debug, Clone)]
pub struct Tooltip {
    trigger: AnyElement,
    content: AnyElement,
    align: TooltipAlign,
    side: TooltipSide,
    side_offset: Px,
    window_margin_override: Option<Px>,
    arrow: bool,
    arrow_size_override: Option<Px>,
    arrow_padding_override: Option<Px>,
    open_delay_frames: u32,
    close_delay_frames: u32,
    layout: LayoutRefinement,
    anchor_override: Option<fret_ui::elements::GlobalElementId>,
}

impl Tooltip {
    pub fn new(trigger: AnyElement, content: AnyElement) -> Self {
        Self {
            trigger,
            content,
            align: TooltipAlign::default(),
            side: TooltipSide::default(),
            side_offset: Px(0.0),
            window_margin_override: None,
            arrow: false,
            arrow_size_override: None,
            arrow_padding_override: None,
            open_delay_frames: 0,
            close_delay_frames: 0,
            layout: LayoutRefinement::default(),
            anchor_override: None,
        }
    }

    pub fn align(mut self, align: TooltipAlign) -> Self {
        self.align = align;
        self
    }

    pub fn side(mut self, side: TooltipSide) -> Self {
        self.side = side;
        self
    }

    pub fn side_offset(mut self, offset: Px) -> Self {
        self.side_offset = offset;
        self
    }

    pub fn open_delay_frames(mut self, frames: u32) -> Self {
        self.open_delay_frames = frames;
        self
    }

    pub fn close_delay_frames(mut self, frames: u32) -> Self {
        self.close_delay_frames = frames;
        self
    }

    pub fn window_margin(mut self, margin: Px) -> Self {
        self.window_margin_override = Some(margin);
        self
    }

    /// Enables a Tooltip arrow (Radix `TooltipArrow`-style).
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

    /// Override the element used as the placement anchor.
    ///
    /// Notes:
    /// - Hover/focus tracking still uses the trigger element.
    /// - The anchor bounds are resolved from last-frame layout/visual bounds (same as Popover).
    pub fn anchor_element(mut self, element: fret_ui::elements::GlobalElementId) -> Self {
        self.anchor_override = Some(element);
        self
    }

    pub fn refine_layout(mut self, layout: LayoutRefinement) -> Self {
        self.layout = self.layout.merge(layout);
        self
    }

    pub fn into_element<H: UiHost>(self, cx: &mut ElementContext<'_, H>) -> AnyElement {
        let theme = Theme::global(&*cx.app).clone();

        let layout = decl_style::layout_style(&theme, self.layout);
        let side_offset = if self.side_offset == Px(0.0) {
            theme
                .metric_by_key("component.tooltip.side_offset")
                .unwrap_or(self.side_offset)
        } else {
            self.side_offset
        };
        let window_margin = self.window_margin_override.unwrap_or_else(|| {
            theme
                .metric_by_key("component.tooltip.window_margin")
                .unwrap_or(Px(8.0))
        });
        let arrow = self.arrow;
        let arrow_size = self.arrow_size_override.unwrap_or_else(|| {
            theme
                .metric_by_key("component.tooltip.arrow_size")
                .unwrap_or(Px(10.0))
        });
        let arrow_padding = self.arrow_padding_override.unwrap_or_else(|| {
            theme
                .metric_by_key("component.tooltip.arrow_padding")
                .unwrap_or(theme.metrics.radius_sm)
        });
        let arrow_bg = theme
            .color_by_key("foreground")
            .unwrap_or(theme.colors.text_primary);

        let align = self.align;
        let side = self.side;
        let open_delay_frames = self.open_delay_frames;
        let close_delay_frames = self.close_delay_frames;

        let trigger = self.trigger;
        let content = self.content;
        let trigger_id = trigger.id;
        let content_id = content.id;
        let anchor_id = self.anchor_override.unwrap_or(trigger_id);

        cx.hover_region(HoverRegionProps { layout }, move |cx, hovered| {
            #[derive(Debug, Default, Clone, Copy)]
            struct FocusEdgeState {
                was_focused: bool,
            }

            let frame = cx.app.frame_id();
            let focused = cx.is_focused_element(trigger_id);
            let (open_delay_ticks, close_delay_ticks) =
                cx.with_state(FocusEdgeState::default, |st| {
                    let was = st.was_focused;
                    st.was_focused = focused;

                    // shadcn/Radix behavior: focus opens immediately, blur closes immediately.
                    let open = if focused { 0 } else { open_delay_frames as u64 };
                    let close = if was && !focused {
                        0
                    } else if focused {
                        0
                    } else {
                        close_delay_frames as u64
                    };

                    (open, close)
                });

            let cfg = HoverIntentConfig::new(open_delay_ticks, close_delay_ticks);
            let update = cx.with_state(HoverIntentState::default, |state| {
                state.update(hovered || focused, frame.0, cfg)
            });

            scheduling::set_continuous_frames(cx, update.wants_continuous_ticks);

            let out = vec![trigger];
            if !update.open {
                return out;
            }

            let tooltip_id = cx.root_id();
            let overlay_root_name = OverlayController::tooltip_root_name(tooltip_id);

            let overlay_children = cx.with_root_name(&overlay_root_name, |cx| {
                let anchor = overlay::anchor_bounds_for_element(cx, anchor_id);
                let Some(anchor) = anchor else {
                    return Vec::new();
                };

                let last_content_size = cx.last_bounds_for_element(content_id).map(|r| r.size);
                let estimated_size = Size::new(Px(240.0), Px(44.0));
                let content_size = last_content_size.unwrap_or(estimated_size);

                let outer = overlay::outer_bounds_with_window_margin(cx.bounds, window_margin);

                let align = match align {
                    TooltipAlign::Start => Align::Start,
                    TooltipAlign::Center => Align::Center,
                    TooltipAlign::End => Align::End,
                };
                let side = match side {
                    TooltipSide::Top => Side::Top,
                    TooltipSide::Right => Side::Right,
                    TooltipSide::Bottom => Side::Bottom,
                    TooltipSide::Left => Side::Left,
                };

                let arrow_options = arrow.then_some(ArrowOptions {
                    size: Size::new(arrow_size, arrow_size),
                    padding: Edges::all(arrow_padding),
                });
                let arrow_protrusion = if arrow {
                    Px(arrow_size.0 * 0.75)
                } else {
                    Px(0.0)
                };

                let layout = overlay::popper_layout_sized(
                    outer,
                    anchor,
                    content_size,
                    side_offset,
                    side,
                    align,
                    AnchoredPanelOptions {
                        direction: LayoutDirection::Ltr,
                        offset: Offset {
                            main_axis: if arrow { arrow_protrusion } else { Px(0.0) },
                            cross_axis: Px(0.0),
                            alignment_axis: None,
                        },
                        arrow: arrow_options,
                    },
                );

                let placed = layout.rect;
                let (extra_left, extra_right, extra_top, extra_bottom) =
                    match layout.arrow.as_ref().map(|a| a.side) {
                        Some(Side::Top) => (Px(0.0), Px(0.0), arrow_protrusion, Px(0.0)),
                        Some(Side::Bottom) => (Px(0.0), Px(0.0), Px(0.0), arrow_protrusion),
                        Some(Side::Left) => (arrow_protrusion, Px(0.0), Px(0.0), Px(0.0)),
                        Some(Side::Right) => (Px(0.0), arrow_protrusion, Px(0.0), Px(0.0)),
                        None => (Px(0.0), Px(0.0), Px(0.0), Px(0.0)),
                    };

                let arrow_el = layout.arrow.map(|arrow| {
                    let (left, top) = match arrow.side {
                        Side::Top => (
                            Px(extra_left.0 + arrow.offset.0),
                            Px(extra_top.0 - arrow_size.0 * 0.5),
                        ),
                        Side::Bottom => (
                            Px(extra_left.0 + arrow.offset.0),
                            Px(extra_top.0 + placed.size.height.0 - arrow_size.0 * 0.5),
                        ),
                        Side::Left => (
                            Px(extra_left.0 - arrow_size.0 * 0.5),
                            Px(extra_top.0 + arrow.offset.0),
                        ),
                        Side::Right => (
                            Px(extra_left.0 + placed.size.width.0 - arrow_size.0 * 0.5),
                            Px(extra_top.0 + arrow.offset.0),
                        ),
                    };

                    let layout = LayoutStyle {
                        position: PositionStyle::Absolute,
                        inset: InsetStyle {
                            top: Some(top),
                            left: Some(left),
                            ..Default::default()
                        },
                        size: SizeStyle {
                            width: Length::Px(arrow_size),
                            height: Length::Px(arrow_size),
                            ..Default::default()
                        },
                        overflow: Overflow::Visible,
                        ..Default::default()
                    };

                    let center = Point::new(Px(arrow_size.0 * 0.5), Px(arrow_size.0 * 0.5));
                    let transform = Transform2D::rotation_about_degrees(45.0, center);

                    cx.visual_transform_props(
                        VisualTransformProps { layout, transform },
                        move |cx| {
                            vec![cx.container(
                                ContainerProps {
                                    layout: LayoutStyle {
                                        size: SizeStyle {
                                            width: Length::Fill,
                                            height: Length::Fill,
                                            ..Default::default()
                                        },
                                        ..Default::default()
                                    },
                                    padding: Edges::all(Px(0.0)),
                                    background: Some(arrow_bg),
                                    shadow: None,
                                    border: Edges::all(Px(0.0)),
                                    border_color: None,
                                    corner_radii: Corners::all(Px(0.0)),
                                },
                                |_cx| Vec::new(),
                            )]
                        },
                    )
                });

                let wrapper = if let Some(arrow_el) = arrow_el {
                    cx.container(
                        ContainerProps {
                            layout: LayoutStyle {
                                position: PositionStyle::Absolute,
                                inset: InsetStyle {
                                    top: Some(Px(placed.origin.y.0 - extra_top.0)),
                                    left: Some(Px(placed.origin.x.0 - extra_left.0)),
                                    ..Default::default()
                                },
                                size: SizeStyle {
                                    width: Length::Px(Px(placed.size.width.0
                                        + extra_left.0
                                        + extra_right.0)),
                                    height: Length::Px(Px(placed.size.height.0
                                        + extra_top.0
                                        + extra_bottom.0)),
                                    ..Default::default()
                                },
                                overflow: Overflow::Visible,
                                ..Default::default()
                            },
                            ..Default::default()
                        },
                        move |cx| {
                            let content = cx.container(
                                ContainerProps {
                                    layout: LayoutStyle {
                                        position: PositionStyle::Absolute,
                                        inset: InsetStyle {
                                            top: Some(extra_top),
                                            left: Some(extra_left),
                                            ..Default::default()
                                        },
                                        size: SizeStyle {
                                            width: Length::Px(placed.size.width),
                                            height: Length::Px(placed.size.height),
                                            ..Default::default()
                                        },
                                        overflow: Overflow::Visible,
                                        ..Default::default()
                                    },
                                    ..Default::default()
                                },
                                move |_cx| vec![content],
                            );

                            vec![arrow_el, content]
                        },
                    )
                } else {
                    cx.container(
                        ContainerProps {
                            layout: LayoutStyle {
                                position: PositionStyle::Absolute,
                                inset: InsetStyle {
                                    top: Some(placed.origin.y),
                                    left: Some(placed.origin.x),
                                    ..Default::default()
                                },
                                size: SizeStyle {
                                    width: Length::Px(placed.size.width),
                                    height: Length::Px(placed.size.height),
                                    ..Default::default()
                                },
                                ..Default::default()
                            },
                            ..Default::default()
                        },
                        move |_cx| vec![content],
                    )
                };

                vec![wrapper]
            });

            let mut request = OverlayRequest::tooltip(
                tooltip_id,
                OverlayPresence::instant(true),
                overlay_children,
            );
            request.root_name = Some(overlay_root_name);
            OverlayController::request(cx, request);

            out
        })
    }
}

/// shadcn/ui `TooltipTrigger` (v4).
#[derive(Debug, Clone)]
pub struct TooltipTrigger {
    child: AnyElement,
}

impl TooltipTrigger {
    pub fn new(child: AnyElement) -> Self {
        Self { child }
    }

    pub fn into_element<H: UiHost>(self, _cx: &mut ElementContext<'_, H>) -> AnyElement {
        self.child
    }
}

/// Optional layout-only anchor for advanced tooltip placement recipes.
///
/// Use [`Tooltip::anchor_element`] to wire the anchor element ID into placement.
#[derive(Debug, Clone)]
pub struct TooltipAnchor {
    child: AnyElement,
}

impl TooltipAnchor {
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

/// shadcn/ui `TooltipContent` (v4).
#[derive(Debug, Clone)]
pub struct TooltipContent {
    children: Vec<AnyElement>,
    chrome: ChromeRefinement,
    layout: LayoutRefinement,
}

impl TooltipContent {
    pub fn new(children: Vec<AnyElement>) -> Self {
        Self {
            children,
            chrome: ChromeRefinement::default(),
            layout: LayoutRefinement::default(),
        }
    }

    pub fn text<H: UiHost>(
        cx: &mut ElementContext<'_, H>,
        text: impl Into<Arc<str>>,
    ) -> AnyElement {
        let theme = Theme::global(&*cx.app).clone();
        let text = text.into();

        let text_style = TextStyle {
            font: fret_core::FontId::default(),
            size: theme.metrics.font_size,
            weight: fret_core::FontWeight::NORMAL,
            line_height: Some(theme.metrics.font_line_height),
            letter_spacing_em: None,
        };

        let fg = theme
            .color_by_key("background")
            .unwrap_or(theme.colors.surface_background);

        cx.text_props(TextProps {
            layout: LayoutStyle::default(),
            text,
            style: Some(text_style),
            color: Some(fg),
            wrap: TextWrap::None,
            overflow: TextOverflow::Ellipsis,
        })
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

        let base_layout = LayoutRefinement::default().flex_shrink_0();
        let chrome = tooltip_content_chrome(&theme).merge(self.chrome);
        let mut props = decl_style::container_props(&theme, chrome, base_layout.merge(self.layout));

        let radius = MetricRef::radius(Radius::Md).resolve(&theme);
        props.shadow = Some(decl_style::shadow_sm(&theme, radius));
        let children = self.children;
        cx.container(props, move |_cx| children)
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::cell::Cell;
    use std::rc::Rc;

    use fret_app::App;
    use fret_core::{
        AppWindowId, PathCommand, PathConstraints, PathId, PathMetrics, PathService, PathStyle,
        Point, Px, Rect, SemanticsRole, Size as CoreSize, SvgId, SvgService, TextBlobId,
        TextConstraints, TextMetrics, TextService, TextStyle as CoreTextStyle,
    };
    use fret_runtime::FrameId;
    use fret_ui::element::{
        ContainerProps, LayoutStyle, Length, PressableA11y, PressableProps, SemanticsProps,
        TextProps,
    };
    use fret_ui::overlay_placement::{Align, Side, anchored_panel_bounds_sized};
    use fret_ui::tree::UiTree;

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
                    size: CoreSize::new(Px(10.0), Px(10.0)),
                    baseline: Px(8.0),
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

    fn render_tooltip_frame(
        ui: &mut UiTree<App>,
        app: &mut App,
        services: &mut dyn fret_core::UiServices,
        window: AppWindowId,
        bounds: Rect,
        trigger_id_out: Rc<Cell<Option<fret_ui::elements::GlobalElementId>>>,
        content_id_out: Rc<Cell<Option<fret_ui::elements::GlobalElementId>>>,
    ) {
        OverlayController::begin_frame(app, window);

        let root =
            fret_ui::declarative::render_root(ui, app, services, window, bounds, "test", |cx| {
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
                        a11y: PressableA11y {
                            role: Some(SemanticsRole::Button),
                            label: Some(Arc::from("trigger")),
                            ..Default::default()
                        },
                        ..Default::default()
                    },
                    |cx, _st, id| {
                        trigger_id_out.set(Some(id));
                        vec![cx.container(ContainerProps::default(), |_cx| Vec::new())]
                    },
                );

                let content = TooltipContent::new(vec![cx.text_props(TextProps::new("tip"))])
                    .into_element(cx);
                content_id_out.set(Some(content.id));

                vec![
                    Tooltip::new(trigger, content)
                        .open_delay_frames(30)
                        .close_delay_frames(30)
                        .into_element(cx),
                ]
            });

        ui.set_root(root);
        OverlayController::render(ui, app, services, window, bounds);
    }

    #[test]
    fn tooltip_opens_on_keyboard_focus_without_hover() {
        let window = AppWindowId::default();
        let mut app = App::new();
        let mut ui: UiTree<App> = UiTree::new();
        ui.set_window(window);

        let trigger_id: Rc<Cell<Option<fret_ui::elements::GlobalElementId>>> =
            Rc::new(Cell::new(None));
        let content_id: Rc<Cell<Option<fret_ui::elements::GlobalElementId>>> =
            Rc::new(Cell::new(None));

        let mut services = FakeServices;
        let bounds = Rect::new(
            Point::new(Px(0.0), Px(0.0)),
            CoreSize::new(Px(800.0), Px(600.0)),
        );

        // Frame 1: establish element->node mappings.
        app.set_frame_id(FrameId(1));
        render_tooltip_frame(
            &mut ui,
            &mut app,
            &mut services,
            window,
            bounds,
            trigger_id.clone(),
            content_id.clone(),
        );
        ui.layout_all(&mut app, &mut services, bounds, 1.0);

        let trigger_element = trigger_id.get().expect("trigger element id");
        let trigger_node = fret_ui::elements::node_for_element(&mut app, window, trigger_element)
            .expect("trigger node");
        ui.set_focus(Some(trigger_node));

        // Frame 2: focus should cause the tooltip overlay to be requested and mounted.
        app.set_frame_id(FrameId(2));
        render_tooltip_frame(
            &mut ui,
            &mut app,
            &mut services,
            window,
            bounds,
            trigger_id.clone(),
            content_id.clone(),
        );
        ui.layout_all(&mut app, &mut services, bounds, 1.0);

        let content_element = content_id.get().expect("content element id");
        let content_node = fret_ui::elements::node_for_element(&mut app, window, content_element);
        assert!(
            content_node.is_some(),
            "expected tooltip content to be mounted when focused"
        );
    }

    #[test]
    fn tooltip_anchor_override_uses_anchor_bounds_for_placement() {
        let window = AppWindowId::default();
        let mut app = App::new();
        let mut ui: UiTree<App> = UiTree::new();
        ui.set_window(window);

        let anchor_id: Rc<Cell<Option<fret_ui::elements::GlobalElementId>>> =
            Rc::new(Cell::new(None));
        let trigger_id: Rc<Cell<Option<fret_ui::elements::GlobalElementId>>> =
            Rc::new(Cell::new(None));
        let content_id: Rc<Cell<Option<fret_ui::elements::GlobalElementId>>> =
            Rc::new(Cell::new(None));

        let mut services = FakeServices;
        let bounds = Rect::new(
            Point::new(Px(0.0), Px(0.0)),
            CoreSize::new(Px(800.0), Px(600.0)),
        );

        fn render_frame(
            ui: &mut UiTree<App>,
            app: &mut App,
            services: &mut dyn fret_core::UiServices,
            window: AppWindowId,
            bounds: Rect,
            anchor_id_out: Rc<Cell<Option<fret_ui::elements::GlobalElementId>>>,
            trigger_id_out: Rc<Cell<Option<fret_ui::elements::GlobalElementId>>>,
            content_id_out: Rc<Cell<Option<fret_ui::elements::GlobalElementId>>>,
        ) {
            OverlayController::begin_frame(app, window);

            let root = fret_ui::declarative::render_root(
                ui,
                app,
                services,
                window,
                bounds,
                "test",
                |cx| {
                    let anchor_id_out_for_anchor = anchor_id_out.clone();
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

                    let anchor_id = anchor_id_out.get().expect("anchor element id");

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
                            a11y: PressableA11y {
                                role: Some(SemanticsRole::Button),
                                label: Some(Arc::from("trigger")),
                                ..Default::default()
                            },
                            ..Default::default()
                        },
                        |cx, _st, id| {
                            trigger_id_out.set(Some(id));
                            vec![cx.container(ContainerProps::default(), |_cx| Vec::new())]
                        },
                    );

                    let content = cx.semantics(
                        SemanticsProps {
                            role: SemanticsRole::Panel,
                            ..Default::default()
                        },
                        |cx| {
                            vec![
                                TooltipContent::new(vec![cx.text_props(TextProps::new("tip"))])
                                    .into_element(cx),
                            ]
                        },
                    );
                    content_id_out.set(Some(content.id));

                    vec![
                        anchor,
                        Tooltip::new(trigger, content)
                            .anchor_element(anchor_id)
                            .side(TooltipSide::Bottom)
                            .align(TooltipAlign::Start)
                            .side_offset(Px(8.0))
                            .window_margin(Px(0.0))
                            .open_delay_frames(0)
                            .close_delay_frames(0)
                            .into_element(cx),
                    ]
                },
            );

            ui.set_root(root);
            OverlayController::render(ui, app, services, window, bounds);
        }

        // Frame 1: establish bounds for the anchor + element/node mappings.
        app.set_frame_id(FrameId(1));
        render_frame(
            &mut ui,
            &mut app,
            &mut services,
            window,
            bounds,
            anchor_id.clone(),
            trigger_id.clone(),
            content_id.clone(),
        );
        ui.layout_all(&mut app, &mut services, bounds, 1.0);

        ui.dispatch_event(
            &mut app,
            &mut services,
            &fret_core::Event::Pointer(fret_core::PointerEvent::Move {
                position: Point::new(Px(12.0), Px(12.0)),
                buttons: fret_core::MouseButtons::default(),
                modifiers: fret_core::Modifiers::default(),
            }),
        );

        // Frame 2: hover should open the tooltip, and placement should use the anchor override.
        app.set_frame_id(FrameId(2));
        render_frame(
            &mut ui,
            &mut app,
            &mut services,
            window,
            bounds,
            anchor_id.clone(),
            trigger_id.clone(),
            content_id.clone(),
        );
        ui.request_semantics_snapshot();
        ui.layout_all(&mut app, &mut services, bounds, 1.0);

        let content_element = content_id.get().expect("content element id");

        let anchor_bounds = Rect::new(
            Point::new(Px(240.0), Px(120.0)),
            CoreSize::new(Px(50.0), Px(10.0)),
        );

        let expected = anchored_panel_bounds_sized(
            bounds,
            anchor_bounds,
            CoreSize::new(Px(240.0), Px(44.0)),
            Px(8.0),
            Side::Bottom,
            Align::Start,
        );

        let snap = ui.semantics_snapshot().expect("semantics snapshot");
        let content_node = fret_ui::elements::node_for_element(&mut app, window, content_element)
            .expect("content node");
        let content_bounds = snap
            .nodes
            .iter()
            .find(|n| n.id == content_node)
            .map(|n| n.bounds)
            .expect("content bounds");

        assert_eq!(content_bounds.origin, expected.origin);
    }
}
