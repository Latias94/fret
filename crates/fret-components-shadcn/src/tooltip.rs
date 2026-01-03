use crate::popper_arrow::{self, DiamondArrowStyle};
use fret_components_ui::declarative::scheduling;
use fret_components_ui::declarative::style as decl_style;
use fret_components_ui::headless::hover_intent::{HoverIntentConfig, HoverIntentState};
use fret_components_ui::overlay;
use fret_components_ui::primitives::popper;
use fret_components_ui::primitives::popper_content;
use fret_components_ui::tooltip_provider;
use fret_components_ui::{
    ChromeRefinement, ColorRef, LayoutRefinement, MetricRef, OverlayController, OverlayPresence,
    OverlayRequest, Radius, Space,
};
use std::sync::Arc;

use fret_core::{Edges, Px, Size, TextOverflow, TextStyle, TextWrap};
use fret_ui::element::{AnyElement, HoverRegionProps, LayoutStyle, Overflow, TextProps};
use fret_ui::overlay_placement::{Align, ArrowOptions, LayoutDirection, Side};
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

/// shadcn/ui `TooltipProvider` (v4).
///
/// In Radix/shadcn this is a context provider used to share open-delay behavior across tooltip
/// instances. In Fret, this is implemented as a declarative scoping helper that persists provider
/// state (delay group) across frames.
#[derive(Debug, Clone, Copy, Default, PartialEq, Eq)]
pub struct TooltipProvider {
    delay_duration_frames: u32,
    skip_delay_duration_frames: u32,
}

impl TooltipProvider {
    pub fn new() -> Self {
        Self::default()
    }

    pub fn delay_duration_frames(mut self, frames: u32) -> Self {
        self.delay_duration_frames = frames;
        self
    }

    pub fn skip_delay_duration_frames(mut self, frames: u32) -> Self {
        self.skip_delay_duration_frames = frames;
        self
    }

    pub fn with<H: UiHost>(
        self,
        cx: &mut ElementContext<'_, H>,
        f: impl FnOnce(&mut ElementContext<'_, H>) -> Vec<AnyElement>,
    ) -> Vec<AnyElement> {
        tooltip_provider::with_tooltip_provider(
            cx,
            tooltip_provider::TooltipProviderConfig::new(
                self.delay_duration_frames as u64,
                self.skip_delay_duration_frames as u64,
            ),
            f,
        )
    }
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
    open_delay_frames_override: Option<u32>,
    close_delay_frames_override: Option<u32>,
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
            arrow: true,
            arrow_size_override: None,
            arrow_padding_override: None,
            open_delay_frames_override: None,
            close_delay_frames_override: None,
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
        self.open_delay_frames_override = Some(frames);
        self
    }

    pub fn close_delay_frames(mut self, frames: u32) -> Self {
        self.close_delay_frames_override = Some(frames);
        self
    }

    pub fn window_margin(mut self, margin: Px) -> Self {
        self.window_margin_override = Some(margin);
        self
    }

    /// Enables a Tooltip arrow (Radix `TooltipArrow`-style).
    ///
    /// Default: `true`.
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
        let open_delay_frames_override = self.open_delay_frames_override;
        let close_delay_frames_override = self.close_delay_frames_override;

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

            let now = cx.app.frame_id().0;
            let focused = cx.is_focused_element(trigger_id);
            let open_delay_ticks = if focused {
                0
            } else if let Some(frames) = open_delay_frames_override {
                tooltip_provider::open_delay_ticks_with_base(cx, now, frames as u64)
            } else {
                tooltip_provider::open_delay_ticks(cx, now)
            };
            let close_delay_ticks = cx.with_state(FocusEdgeState::default, |st| {
                let was = st.was_focused;
                st.was_focused = focused;

                // shadcn/Radix behavior: blur closes immediately.
                if was && !focused {
                    0
                } else if focused {
                    0
                } else {
                    close_delay_frames_override.unwrap_or(0) as u64
                }
            });

            let cfg = HoverIntentConfig::new(open_delay_ticks, close_delay_ticks);
            let (was_open, update) = cx.with_state(HoverIntentState::default, |st| {
                let was_open = st.is_open();
                let update = st.update(hovered || focused, now, cfg);
                (was_open, update)
            });

            scheduling::set_continuous_frames(cx, update.wants_continuous_ticks);

            if was_open && !update.open {
                tooltip_provider::note_closed(cx, now);
            }

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
                let arrow_protrusion = popper_arrow::arrow_protrusion(arrow, arrow_size);

                let layout = popper::popper_content_layout_sized(
                    outer,
                    anchor,
                    content_size,
                    popper::PopperContentPlacement::new(
                        LayoutDirection::Ltr,
                        side,
                        align,
                        side_offset,
                    )
                    .with_arrow(arrow_options, arrow_protrusion),
                );

                let placed = layout.rect;
                let wrapper_insets = popper_arrow::wrapper_insets(&layout, arrow_protrusion);

                let arrow_el = popper_arrow::diamond_arrow_element(
                    cx,
                    &layout,
                    wrapper_insets,
                    arrow_size,
                    DiamondArrowStyle {
                        bg: arrow_bg,
                        border: None,
                        border_width: Px(0.0),
                    },
                );

                let wrapper = popper_content::popper_wrapper_at(cx, placed, wrapper_insets, |cx| {
                    let content = popper_content::popper_panel_at(
                        cx,
                        placed,
                        wrapper_insets,
                        Overflow::Visible,
                        move |_cx| vec![content],
                    );

                    if let Some(arrow_el) = arrow_el {
                        vec![arrow_el, content]
                    } else {
                        vec![content]
                    }
                });

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
    fn tooltip_opens_after_delay_and_closes_after_close_delay() {
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

        fn render_frame(
            ui: &mut UiTree<App>,
            app: &mut App,
            services: &mut dyn fret_core::UiServices,
            window: AppWindowId,
            bounds: Rect,
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
                    TooltipProvider::new()
                        .delay_duration_frames(1)
                        .skip_delay_duration_frames(0)
                        .with(cx, |cx| {
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

                            let content =
                                TooltipContent::new(vec![cx.text_props(TextProps::new("tip"))])
                                    .into_element(cx);
                            content_id_out.set(Some(content.id));

                            vec![
                                Tooltip::new(trigger, content)
                                    .close_delay_frames(2)
                                    .into_element(cx),
                            ]
                        })
                },
            );
            ui.set_root(root);
            OverlayController::render(ui, app, services, window, bounds);
        }

        // Frame 1: build and establish mappings.
        app.set_frame_id(FrameId(1));
        render_frame(
            &mut ui,
            &mut app,
            &mut services,
            window,
            bounds,
            trigger_id.clone(),
            content_id.clone(),
        );
        ui.layout_all(&mut app, &mut services, bounds, 1.0);

        // Ensure pointer starts outside.
        ui.dispatch_event(
            &mut app,
            &mut services,
            &fret_core::Event::Pointer(fret_core::PointerEvent::Move {
                position: Point::new(Px(200.0), Px(200.0)),
                buttons: fret_core::MouseButtons::default(),
                modifiers: fret_core::Modifiers::default(),
            }),
        );

        // Hover trigger.
        ui.dispatch_event(
            &mut app,
            &mut services,
            &fret_core::Event::Pointer(fret_core::PointerEvent::Move {
                position: Point::new(Px(10.0), Px(10.0)),
                buttons: fret_core::MouseButtons::default(),
                modifiers: fret_core::Modifiers::default(),
            }),
        );

        // Frame 2: hovered, but delay not yet elapsed.
        app.set_frame_id(FrameId(2));
        render_frame(
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
        assert!(
            fret_ui::elements::node_for_element(&mut app, window, content_element).is_none(),
            "expected tooltip to still be closed before delay elapses"
        );

        // Frame 3: delay elapsed -> open.
        app.set_frame_id(FrameId(3));
        render_frame(
            &mut ui,
            &mut app,
            &mut services,
            window,
            bounds,
            trigger_id.clone(),
            content_id.clone(),
        );
        ui.layout_all(&mut app, &mut services, bounds, 1.0);

        let content_node = fret_ui::elements::node_for_element(&mut app, window, content_element)
            .expect("expected tooltip content node to exist after delay elapses");
        let tooltip_layer_root = *ui
            .debug_node_path(content_node)
            .first()
            .expect("tooltip node path root");
        assert!(
            ui.debug_layers_in_paint_order()
                .iter()
                .find(|layer| layer.root == tooltip_layer_root)
                .is_some_and(|layer| layer.visible),
            "expected tooltip layer to be visible after delay elapses"
        );

        // Leave hover.
        ui.dispatch_event(
            &mut app,
            &mut services,
            &fret_core::Event::Pointer(fret_core::PointerEvent::Move {
                position: Point::new(Px(200.0), Px(200.0)),
                buttons: fret_core::MouseButtons::default(),
                modifiers: fret_core::Modifiers::default(),
            }),
        );

        // Frame 4/5: close delay not yet elapsed -> still open.
        for frame in 4..=5 {
            app.set_frame_id(FrameId(frame));
            render_frame(
                &mut ui,
                &mut app,
                &mut services,
                window,
                bounds,
                trigger_id.clone(),
                content_id.clone(),
            );
            ui.layout_all(&mut app, &mut services, bounds, 1.0);
            assert!(
                fret_ui::elements::node_for_element(&mut app, window, content_element).is_some(),
                "expected tooltip to remain mounted during close delay"
            );
            assert!(
                ui.debug_layers_in_paint_order()
                    .iter()
                    .find(|layer| layer.root == tooltip_layer_root)
                    .is_some_and(|layer| layer.visible),
                "expected tooltip layer to remain visible during close delay"
            );
        }

        // Frame 6: close delay elapsed -> closed.
        app.set_frame_id(FrameId(6));
        render_frame(
            &mut ui,
            &mut app,
            &mut services,
            window,
            bounds,
            trigger_id.clone(),
            content_id.clone(),
        );
        ui.layout_all(&mut app, &mut services, bounds, 1.0);

        let tooltip_layer = ui
            .debug_layers_in_paint_order()
            .into_iter()
            .find(|layer| layer.root == tooltip_layer_root);
        assert!(
            tooltip_layer.is_none_or(|layer| !layer.visible),
            "expected tooltip layer to be hidden after close delay elapses"
        );
    }

    #[test]
    fn tooltip_provider_skips_delay_after_recent_close() {
        let window = AppWindowId::default();
        let mut app = App::new();
        let mut ui: UiTree<App> = UiTree::new();
        ui.set_window(window);

        // This test "fast-forwards" by jumping `App::frame_id` without rendering intermediate
        // frames. Keep element state alive across that jump so we can validate provider delay
        // semantics rather than `ElementRuntime` GC behavior.
        app.with_global_mut(fret_ui::elements::ElementRuntime::new, |rt, _app| {
            rt.set_gc_lag_frames(128);
        });

        let content_1_id: Rc<Cell<Option<fret_ui::elements::GlobalElementId>>> =
            Rc::new(Cell::new(None));
        let content_2_id: Rc<Cell<Option<fret_ui::elements::GlobalElementId>>> =
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
            content_1_id_out: Rc<Cell<Option<fret_ui::elements::GlobalElementId>>>,
            content_2_id_out: Rc<Cell<Option<fret_ui::elements::GlobalElementId>>>,
        ) {
            OverlayController::begin_frame(app, window);

            let root =
                fret_ui::declarative::render_root(
                    ui,
                    app,
                    services,
                    window,
                    bounds,
                    "test",
                    |cx| {
                        TooltipProvider::new()
                            .delay_duration_frames(10)
                            .skip_delay_duration_frames(30)
                            .with(cx, |cx| {
                                vec![cx.column(fret_ui::element::ColumnProps::default(), |cx| {
                                    let trigger_1 = cx.pressable_with_id(
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
                                                label: Some(Arc::from("trigger_1")),
                                                ..Default::default()
                                            },
                                            ..Default::default()
                                        },
                                        |cx, _st, _id| {
                                            vec![cx.container(ContainerProps::default(), |_cx| {
                                                Vec::new()
                                            })]
                                        },
                                    );

                                    let trigger_2 = cx.pressable_with_id(
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
                                                label: Some(Arc::from("trigger_2")),
                                                ..Default::default()
                                            },
                                            ..Default::default()
                                        },
                                        |cx, _st, _id| {
                                            vec![cx.container(ContainerProps::default(), |_cx| {
                                                Vec::new()
                                            })]
                                        },
                                    );

                                    let content_1 = TooltipContent::new(vec![
                                        cx.text_props(TextProps::new("tip_1")),
                                    ])
                                    .into_element(cx);
                                    content_1_id_out.set(Some(content_1.id));

                                    let content_2 = TooltipContent::new(vec![
                                        cx.text_props(TextProps::new("tip_2")),
                                    ])
                                    .into_element(cx);
                                    content_2_id_out.set(Some(content_2.id));

                                    vec![
                                        Tooltip::new(trigger_1, content_1).into_element(cx),
                                        Tooltip::new(trigger_2, content_2).into_element(cx),
                                    ]
                                })]
                            })
                    },
                );

            ui.set_root(root);
            OverlayController::render(ui, app, services, window, bounds);
        }

        // Frame 1: build.
        app.set_frame_id(FrameId(1));
        render_frame(
            &mut ui,
            &mut app,
            &mut services,
            window,
            bounds,
            content_1_id.clone(),
            content_2_id.clone(),
        );
        ui.request_semantics_snapshot();
        ui.layout_all(&mut app, &mut services, bounds, 1.0);

        let snap = ui.semantics_snapshot().expect("semantics snapshot");
        let trigger_1 = snap
            .nodes
            .iter()
            .find(|n| n.label.as_deref() == Some("trigger_1"))
            .expect("trigger_1 node");
        let trigger_2 = snap
            .nodes
            .iter()
            .find(|n| n.label.as_deref() == Some("trigger_2"))
            .expect("trigger_2 node");

        let trigger_1_node = trigger_1.id;
        let trigger_1_bounds = trigger_1.bounds;
        let trigger_2_bounds = trigger_2.bounds;

        let trigger_1_point = Point::new(
            Px(trigger_1_bounds.origin.x.0 + trigger_1_bounds.size.width.0 * 0.5),
            Px(trigger_1_bounds.origin.y.0 + trigger_1_bounds.size.height.0 * 0.5),
        );
        let trigger_2_point = Point::new(
            Px(trigger_2_bounds.origin.x.0 + trigger_2_bounds.size.width.0 * 0.5),
            Px(trigger_2_bounds.origin.y.0 + trigger_2_bounds.size.height.0 * 0.5),
        );

        ui.dispatch_event(
            &mut app,
            &mut services,
            &fret_core::Event::Pointer(fret_core::PointerEvent::Move {
                position: trigger_1_point,
                buttons: fret_core::MouseButtons::default(),
                modifiers: fret_core::Modifiers::default(),
            }),
        );

        // Frame 2: focus should open tooltip 1 immediately (regardless of provider delay).
        ui.set_focus(Some(trigger_1_node));

        app.set_frame_id(FrameId(2));
        render_frame(
            &mut ui,
            &mut app,
            &mut services,
            window,
            bounds,
            content_1_id.clone(),
            content_2_id.clone(),
        );
        ui.layout_all(&mut app, &mut services, bounds, 1.0);

        let content_1_element = content_1_id.get().expect("content_1 element id");
        assert!(
            fret_ui::elements::node_for_element(&mut app, window, content_1_element).is_some(),
            "expected tooltip 1 to be open when focused"
        );

        // Blur + move to trigger 2, then render: provider should skip delay for the new tooltip.
        ui.set_focus(None);
        ui.dispatch_event(
            &mut app,
            &mut services,
            &fret_core::Event::Pointer(fret_core::PointerEvent::Move {
                position: trigger_2_point,
                buttons: fret_core::MouseButtons::default(),
                modifiers: fret_core::Modifiers::default(),
            }),
        );

        app.set_frame_id(FrameId(3));
        render_frame(
            &mut ui,
            &mut app,
            &mut services,
            window,
            bounds,
            content_1_id.clone(),
            content_2_id.clone(),
        );
        ui.layout_all(&mut app, &mut services, bounds, 1.0);

        let content_2_element = content_2_id.get().expect("content_2 element id");
        assert!(
            fret_ui::elements::node_for_element(&mut app, window, content_2_element).is_some(),
            "expected tooltip 2 to open without delay under the provider skip window"
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
                            .arrow(false)
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
