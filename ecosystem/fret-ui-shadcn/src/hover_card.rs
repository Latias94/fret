use crate::popper_arrow::{self, DiamondArrowStyle};
use fret_core::{Px, Size, Transform2D};
use fret_ui::element::{
    AnyElement, HoverRegionProps, LayoutStyle, Length, OpacityProps, Overflow, SizeStyle,
    VisualTransformProps,
};
use fret_ui::overlay_placement::{Align, LayoutDirection, Side};
use fret_ui::{ElementContext, Theme, UiHost};
use fret_ui_kit::declarative::style as decl_style;
use fret_ui_kit::overlay;
use fret_ui_kit::primitives::hover_card as radix_hover_card;
use fret_ui_kit::primitives::hover_intent::{self, HoverIntentConfig};
use fret_ui_kit::primitives::popper;
use fret_ui_kit::primitives::popper_content;
use fret_ui_kit::primitives::presence as radix_presence;
use fret_ui_kit::{ChromeRefinement, ColorRef, LayoutRefinement, MetricRef, Radius, Space};

use crate::layout as shadcn_layout;
use crate::overlay_motion;

// Radix default delays: open=700ms, close=300ms. We approximate with 60fps ticks.
const HOVER_CARD_DEFAULT_OPEN_DELAY_FRAMES: u32 =
    (overlay_motion::SHADCN_MOTION_TICKS_500 + overlay_motion::SHADCN_MOTION_TICKS_200) as u32;
const HOVER_CARD_DEFAULT_CLOSE_DELAY_FRAMES: u32 = overlay_motion::SHADCN_MOTION_TICKS_300 as u32;

fn shadcn_zoom_transform(origin: fret_core::Point, scale: f32) -> Transform2D {
    Transform2D::translation(origin)
        * Transform2D::scale_uniform(scale)
        * Transform2D::translation(fret_core::Point::new(Px(-origin.x.0), Px(-origin.y.0)))
}

fn hover_card_content_chrome(theme: &Theme) -> ChromeRefinement {
    let bg = theme.color_required("popover");
    let border = theme.color_required("border");

    ChromeRefinement::default()
        .rounded(Radius::Md)
        .border_1()
        .bg(ColorRef::Color(bg))
        .border_color(ColorRef::Color(border))
        .p(Space::N4)
}

#[derive(Debug, Clone, Copy, Default, PartialEq, Eq)]
pub enum HoverCardAlign {
    Start,
    #[default]
    Center,
    End,
}

#[derive(Debug, Default, Clone, Copy)]
struct HoverCardSharedState {
    overlay_hovered: bool,
}

/// shadcn/ui `HoverCard` root (v4).
///
/// This is a floating hover surface anchored to a trigger. In Radix/shadcn this uses a portal;
/// in Fret this is implemented as a component-layer policy built on runtime substrate primitives:
/// - `HoverRegion` (hover tracking)
/// - cross-frame geometry queries (`elements::bounds_for_element`)
/// - placement solver (`overlay_placement`)
#[derive(Debug, Clone)]
pub struct HoverCard {
    trigger: AnyElement,
    content: AnyElement,
    align: HoverCardAlign,
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

impl HoverCard {
    pub fn new(trigger: AnyElement, content: AnyElement) -> Self {
        Self {
            trigger,
            content,
            align: HoverCardAlign::default(),
            side_offset: Px(4.0),
            window_margin_override: None,
            arrow: false,
            arrow_size_override: None,
            arrow_padding_override: None,
            open_delay_frames: HOVER_CARD_DEFAULT_OPEN_DELAY_FRAMES,
            close_delay_frames: HOVER_CARD_DEFAULT_CLOSE_DELAY_FRAMES,
            layout: LayoutRefinement::default(),
            anchor_override: None,
        }
    }

    pub fn align(mut self, align: HoverCardAlign) -> Self {
        self.align = align;
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

    /// Enables a HoverCard arrow (Radix `HoverCardArrow`-style).
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
    /// - Hover tracking still uses the trigger element.
    /// - The anchor bounds are resolved from last-frame layout/visual bounds.
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
        let side_offset = if self.side_offset == Px(4.0) {
            theme
                .metric_by_key("component.hover_card.side_offset")
                .unwrap_or(self.side_offset)
        } else {
            self.side_offset
        };
        let window_margin = self.window_margin_override.unwrap_or_else(|| {
            theme
                .metric_by_key("component.hover_card.window_margin")
                .unwrap_or(Px(8.0))
        });

        let align = self.align;
        let open_delay_frames = self.open_delay_frames;
        let close_delay_frames = self.close_delay_frames;
        let arrow = self.arrow;
        let arrow_size = self.arrow_size_override.unwrap_or_else(|| {
            theme
                .metric_by_key("component.hover_card.arrow_size")
                .unwrap_or(Px(12.0))
        });
        let arrow_padding = self.arrow_padding_override.unwrap_or_else(|| {
            theme
                .metric_by_key("component.hover_card.arrow_padding")
                .unwrap_or_else(|| MetricRef::radius(Radius::Md).resolve(&theme))
        });
        let arrow_bg = theme.color_required("popover");
        let arrow_border = theme.color_required("border");

        let content = self.content;
        let trigger = radix_hover_card::apply_hover_card_trigger_a11y(self.trigger, content.id);
        let trigger_id = trigger.id;
        let content_id = content.id;
        let anchor_id = self.anchor_override.unwrap_or(trigger_id);
        cx.hover_region(HoverRegionProps { layout }, move |cx, hovered| {
            let hover_card_id = cx.root_id();

            let overlay_hovered =
                cx.with_state_for(hover_card_id, HoverCardSharedState::default, |st| {
                    st.overlay_hovered
                });
            let focused = cx.is_focused_element(trigger_id);
            let keyboard_focused =
                focused && fret_ui::input_modality::is_keyboard(&mut *cx.app, Some(cx.window));
            let hovered =
                radix_hover_card::hover_card_hovered(hovered, overlay_hovered, keyboard_focused);

            let cfg = HoverIntentConfig::new(open_delay_frames as u64, close_delay_frames as u64);
            let update = hover_intent::drive(cx, hovered, cfg);
            let opening = update.open;
            let motion = radix_presence::scale_fade_presence_with_durations_and_easing(
                cx,
                opening,
                overlay_motion::SHADCN_MOTION_TICKS_100,
                overlay_motion::SHADCN_MOTION_TICKS_100,
                0.95,
                1.0,
                overlay_motion::shadcn_ease,
            );
            let opacity = motion.opacity;
            let scale = motion.scale;

            let out = vec![trigger];
            if !motion.present {
                cx.with_state_for(hover_card_id, HoverCardSharedState::default, |st| {
                    st.overlay_hovered = false;
                });
                return out;
            }

            let overlay_root_name = radix_hover_card::hover_card_root_name(hover_card_id);

            let overlay_children = cx.with_root_name(&overlay_root_name, |cx| {
                let anchor = overlay::anchor_bounds_for_element(cx, anchor_id);
                let Some(anchor) = anchor else {
                    cx.with_state_for(hover_card_id, HoverCardSharedState::default, |st| {
                        st.overlay_hovered = false;
                    });
                    return Vec::new();
                };

                let last_content_size = cx.last_bounds_for_element(content_id).map(|r| r.size);
                let estimated_size = Size::new(Px(256.0), Px(120.0));
                let content_size = last_content_size.unwrap_or(estimated_size);

                let outer = overlay::outer_bounds_with_window_margin(cx.bounds, window_margin);

                let align = match align {
                    HoverCardAlign::Start => Align::Start,
                    HoverCardAlign::Center => Align::Center,
                    HoverCardAlign::End => Align::End,
                };

                let (arrow_options, arrow_protrusion) =
                    popper::diamond_arrow_options(arrow, arrow_size, arrow_padding);

                let layout = popper::popper_content_layout_sized(
                    outer,
                    anchor,
                    content_size,
                    popper::PopperContentPlacement::new(
                        LayoutDirection::Ltr,
                        Side::Bottom,
                        align,
                        side_offset,
                    )
                    .with_arrow(arrow_options, arrow_protrusion),
                );

                let placed = layout.rect;
                let mut wrapper_insets = popper_arrow::wrapper_insets(&layout, arrow_protrusion);
                let slide_insets = overlay_motion::shadcn_slide_insets(layout.side);
                wrapper_insets.top.0 += slide_insets.top.0;
                wrapper_insets.right.0 += slide_insets.right.0;
                wrapper_insets.bottom.0 += slide_insets.bottom.0;
                wrapper_insets.left.0 += slide_insets.left.0;

                let origin = popper::popper_content_transform_origin(
                    &layout,
                    anchor,
                    arrow.then_some(arrow_size),
                );

                let zoom = shadcn_zoom_transform(origin, scale);
                let slide = if opening {
                    overlay_motion::shadcn_enter_slide_transform(layout.side, opacity, opening)
                } else {
                    Transform2D::IDENTITY
                };
                let transform = slide * zoom;

                let overlay_layout = LayoutStyle {
                    size: SizeStyle {
                        width: Length::Fill,
                        height: Length::Fill,
                        ..Default::default()
                    },
                    ..Default::default()
                };

                let wrapper = popper_content::popper_hover_region_at_with_panel(
                    cx,
                    placed,
                    wrapper_insets,
                    Overflow::Visible,
                    move |_cx| vec![content],
                    move |cx, hovered, panel| {
                        cx.with_state_for(hover_card_id, HoverCardSharedState::default, |st| {
                            st.overlay_hovered = hovered;
                        });

                        let arrow_el = popper_arrow::diamond_arrow_element(
                            cx,
                            &layout,
                            wrapper_insets,
                            arrow_size,
                            DiamondArrowStyle {
                                bg: arrow_bg,
                                border: Some(arrow_border),
                                border_width: Px(1.0),
                            },
                        );

                        if let Some(arrow_el) = arrow_el {
                            vec![arrow_el, panel]
                        } else {
                            vec![panel]
                        }
                    },
                );

                vec![cx.opacity_props(
                    OpacityProps {
                        layout: overlay_layout.clone(),
                        opacity,
                    },
                    move |cx| {
                        vec![cx.visual_transform_props(
                            VisualTransformProps {
                                layout: overlay_layout,
                                transform,
                            },
                            move |_cx| vec![wrapper],
                        )]
                    },
                )]
            });

            let request =
                radix_hover_card::hover_card_request(hover_card_id, trigger_id, overlay_children);
            radix_hover_card::request_hover_card(cx, request);

            out
        })
    }
}

/// shadcn/ui `HoverCardTrigger` (v4).
///
/// In the DOM this is a context-aware wrapper that does not impose layout. In Fret's declarative
/// authoring, the trigger is expressed as the first child passed to `HoverCard::new(...)`.
#[derive(Debug, Clone)]
pub struct HoverCardTrigger {
    child: AnyElement,
}

impl HoverCardTrigger {
    pub fn new(child: AnyElement) -> Self {
        Self { child }
    }

    pub fn into_element<H: UiHost>(self, _cx: &mut ElementContext<'_, H>) -> AnyElement {
        self.child
    }
}

/// Optional layout-only anchor for advanced hover card placement recipes.
///
/// Use [`HoverCard::anchor_element`] to wire the anchor element ID into placement.
#[derive(Debug, Clone)]
pub struct HoverCardAnchor {
    child: AnyElement,
}

impl HoverCardAnchor {
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

/// shadcn/ui `HoverCardContent` (v4).
#[derive(Debug, Clone)]
pub struct HoverCardContent {
    children: Vec<AnyElement>,
    chrome: ChromeRefinement,
    layout: LayoutRefinement,
}

impl HoverCardContent {
    pub fn new(children: Vec<AnyElement>) -> Self {
        Self {
            children,
            chrome: ChromeRefinement::default(),
            layout: LayoutRefinement::default(),
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

    pub fn into_element<H: UiHost>(self, cx: &mut ElementContext<'_, H>) -> AnyElement {
        let theme = Theme::global(&*cx.app).clone();

        let base_layout = LayoutRefinement::default()
            .w_px(MetricRef::Px(Px(256.0)))
            .flex_shrink_0();

        let chrome = hover_card_content_chrome(&theme).merge(self.chrome);
        let radius = MetricRef::radius(Radius::Md).resolve(&theme);
        let mut props = decl_style::container_props(&theme, chrome, base_layout.merge(self.layout));
        props.shadow = Some(decl_style::shadow_md(&theme, radius));
        let children = self.children;
        shadcn_layout::container_flow(cx, props, children)
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::cell::Cell;
    use std::rc::Rc;

    use fret_app::App;
    use fret_core::{
        AppWindowId, MouseButtons, PathCommand, PathConstraints, PathId, PathMetrics, PathService,
        PathStyle, Point, Px, Rect, SemanticsRole, SvgId, SvgService, TextBlobId, TextConstraints,
        TextMetrics, TextService, TextStyle as CoreTextStyle,
    };
    use fret_runtime::FrameId;
    use fret_ui::element::{
        ContainerProps, LayoutStyle, Length, PositionStyle, PressableProps, SemanticsProps,
        TextProps,
    };
    use fret_ui::overlay_placement;
    use fret_ui::tree::UiTree;
    use fret_ui_kit::OverlayController;

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
                    size: fret_core::Size::new(Px(10.0), Px(10.0)),
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

    fn render_hover_card_frame(
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

        let root =
            fret_ui::declarative::render_root(ui, app, services, window, bounds, "test", |cx| {
                let anchor_id_out_for_anchor = anchor_id_out.clone();
                let anchor = cx.pressable_with_id(
                    PressableProps {
                        layout: {
                            let mut layout = LayoutStyle::default();
                            layout.size.width = Length::Px(Px(50.0));
                            layout.size.height = Length::Px(Px(10.0));
                            layout.inset.top = Some(Px(120.0));
                            layout.inset.left = Some(Px(240.0));
                            layout.position = PositionStyle::Absolute;
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
                            HoverCardContent::new(vec![cx.text_props(TextProps::new("card"))])
                                .into_element(cx),
                        ]
                    },
                );
                content_id_out.set(Some(content.id));

                vec![
                    anchor,
                    HoverCard::new(trigger, content)
                        .anchor_element(anchor_id)
                        .align(HoverCardAlign::Start)
                        .open_delay_frames(0)
                        .side_offset(Px(8.0))
                        .window_margin(Px(0.0))
                        .into_element(cx),
                ]
            });

        ui.set_root(root);
        OverlayController::render(ui, app, services, window, bounds);
    }

    #[test]
    fn hover_card_anchor_override_uses_anchor_bounds_for_placement() {
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

        let mut services = FakeServices::default();
        let bounds = Rect::new(
            Point::new(Px(0.0), Px(0.0)),
            fret_core::Size::new(Px(800.0), Px(600.0)),
        );

        // Frame 1: establish bounds for the anchor + element/node mappings.
        app.set_frame_id(FrameId(1));
        render_hover_card_frame(
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

        // Move pointer over the trigger to open.
        ui.dispatch_event(
            &mut app,
            &mut services,
            &fret_core::Event::Pointer(fret_core::PointerEvent::Move {
                position: Point::new(Px(12.0), Px(12.0)),
                buttons: MouseButtons::default(),
                modifiers: fret_core::Modifiers::default(),
                pointer_type: fret_core::PointerType::Mouse,
            }),
        );

        // Frame 2: hover should request the overlay and mount the content.
        app.set_frame_id(FrameId(2));
        render_hover_card_frame(
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
            fret_core::Size::new(Px(50.0), Px(10.0)),
        );

        let expected = overlay_placement::anchored_panel_bounds_sized(
            bounds,
            anchor_bounds,
            fret_core::Size::new(Px(256.0), Px(120.0)),
            Px(8.0),
            overlay_placement::Side::Bottom,
            overlay_placement::Align::Start,
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

    fn render_hover_card_focus_frame(
        ui: &mut UiTree<App>,
        app: &mut App,
        services: &mut dyn fret_core::UiServices,
        window: AppWindowId,
        bounds: Rect,
        trigger_id_out: Rc<Cell<Option<fret_ui::elements::GlobalElementId>>>,
        content_id_out: Rc<Cell<Option<fret_ui::elements::GlobalElementId>>>,
        after_id_out: Rc<Cell<Option<fret_ui::elements::GlobalElementId>>>,
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
                            HoverCardContent::new(vec![cx.text_props(TextProps::new("card"))])
                                .into_element(cx),
                        ]
                    },
                );
                content_id_out.set(Some(content.id));

                let after = cx.pressable_with_id(
                    PressableProps {
                        layout: {
                            let mut layout = LayoutStyle::default();
                            layout.size.width = Length::Px(Px(120.0));
                            layout.size.height = Length::Px(Px(40.0));
                            layout.inset.top = Some(Px(60.0));
                            layout.position = PositionStyle::Absolute;
                            layout
                        },
                        enabled: true,
                        focusable: true,
                        ..Default::default()
                    },
                    |_cx, _st, id| {
                        after_id_out.set(Some(id));
                        Vec::new()
                    },
                );

                vec![
                    HoverCard::new(trigger, content)
                        .open_delay_frames(0)
                        .close_delay_frames(0)
                        .into_element(cx),
                    after,
                ]
            });

        ui.set_root(root);
        OverlayController::render(ui, app, services, window, bounds);
    }

    #[test]
    fn hover_card_opens_on_focus_and_closes_on_blur() {
        let window = AppWindowId::default();
        let mut app = App::new();
        let mut ui: UiTree<App> = UiTree::new();
        ui.set_window(window);

        let trigger_id: Rc<Cell<Option<fret_ui::elements::GlobalElementId>>> =
            Rc::new(Cell::new(None));
        let content_id: Rc<Cell<Option<fret_ui::elements::GlobalElementId>>> =
            Rc::new(Cell::new(None));
        let after_id: Rc<Cell<Option<fret_ui::elements::GlobalElementId>>> =
            Rc::new(Cell::new(None));

        let mut services = FakeServices::default();
        let bounds = Rect::new(
            Point::new(Px(0.0), Px(0.0)),
            fret_core::Size::new(Px(800.0), Px(600.0)),
        );

        // Frame 1: mount trigger/after and resolve element/node mappings.
        app.set_frame_id(FrameId(1));
        render_hover_card_focus_frame(
            &mut ui,
            &mut app,
            &mut services,
            window,
            bounds,
            trigger_id.clone(),
            content_id.clone(),
            after_id.clone(),
        );
        ui.layout_all(&mut app, &mut services, bounds, 1.0);

        let trigger_element = trigger_id.get().expect("trigger element id");
        let trigger_node = fret_ui::elements::node_for_element(&mut app, window, trigger_element)
            .expect("trigger node");
        // Focus-driven hover cards are a keyboard affordance; mirror the runtime input-modality
        // signal that Radix would receive via key interaction (e.g. tabbing).
        let _ = fret_ui::input_modality::update_for_event(
            &mut app,
            window,
            &fret_core::Event::KeyDown {
                key: fret_core::KeyCode::Tab,
                modifiers: fret_core::Modifiers::default(),
                repeat: false,
            },
        );
        ui.set_focus(Some(trigger_node));

        // Frame 2: focus should open the overlay and mount the content.
        app.set_frame_id(FrameId(2));
        render_hover_card_focus_frame(
            &mut ui,
            &mut app,
            &mut services,
            window,
            bounds,
            trigger_id.clone(),
            content_id.clone(),
            after_id.clone(),
        );
        ui.request_semantics_snapshot();
        ui.layout_all(&mut app, &mut services, bounds, 1.0);

        let content_element = content_id.get().expect("content element id");
        let content_node = fret_ui::elements::node_for_element(&mut app, window, content_element)
            .expect("content node");
        let snap = ui.semantics_snapshot().expect("semantics snapshot");
        assert!(
            snap.nodes.iter().any(|n| n.id == content_node),
            "expected hover card content to mount when trigger is focused"
        );

        // Blur by moving focus elsewhere, then wait for the exit animation to complete.
        let after_element = after_id.get().expect("after element id");
        let after_node = fret_ui::elements::node_for_element(&mut app, window, after_element)
            .expect("after node");
        ui.set_focus(Some(after_node));
        ui.dispatch_event(
            &mut app,
            &mut services,
            &fret_core::Event::Pointer(fret_core::PointerEvent::Move {
                position: Point::new(Px(2000.0), Px(2000.0)),
                buttons: MouseButtons::default(),
                modifiers: fret_core::Modifiers::default(),
                pointer_type: fret_core::PointerType::Mouse,
            }),
        );

        let settle_frames = crate::overlay_motion::SHADCN_MOTION_TICKS_100 + 1;
        for i in 0..settle_frames {
            app.set_frame_id(FrameId(3 + i));
            render_hover_card_focus_frame(
                &mut ui,
                &mut app,
                &mut services,
                window,
                bounds,
                trigger_id.clone(),
                content_id.clone(),
                after_id.clone(),
            );
            ui.layout_all(&mut app, &mut services, bounds, 1.0);
        }

        ui.request_semantics_snapshot();
        ui.layout_all(&mut app, &mut services, bounds, 1.0);

        let snap = ui.semantics_snapshot().expect("semantics snapshot");
        assert!(
            !snap.nodes.iter().any(|n| n.id == content_node),
            "expected hover card content to unmount after blur"
        );
    }
}
