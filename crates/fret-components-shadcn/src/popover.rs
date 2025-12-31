use std::sync::Arc;

use fret_components_ui::declarative::presence;
use fret_components_ui::declarative::style as decl_style;
use fret_components_ui::overlay;
use fret_components_ui::window_overlays;
use fret_components_ui::{ChromeRefinement, ColorRef, LayoutRefinement, Radius, Space};
use fret_core::{FontId, FontWeight, Px, SemanticsRole, Size, TextOverflow, TextStyle, TextWrap};
use fret_runtime::Model;
use fret_ui::Invalidation;
use fret_ui::element::{
    AnyElement, ContainerProps, InsetStyle, LayoutStyle, Length, OpacityProps, Overflow,
    PositionStyle, SemanticsProps, SizeStyle, TextProps,
};
use fret_ui::overlay_placement::{Align, Side, anchored_panel_bounds_sized};
use fret_ui::{ElementCx, Theme, UiHost};

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
    side_offset: Px,
    window_margin_override: Option<Px>,
    auto_focus: bool,
    initial_focus: Option<fret_ui::elements::GlobalElementId>,
}

impl std::fmt::Debug for Popover {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.debug_struct("Popover")
            .field("open", &"<model>")
            .field("align", &self.align)
            .field("side", &self.side)
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
            side_offset: Px(6.0),
            window_margin_override: None,
            auto_focus: false,
            initial_focus: None,
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

    pub fn side_offset(mut self, offset: Px) -> Self {
        self.side_offset = offset;
        self
    }

    pub fn window_margin(mut self, margin: Px) -> Self {
        self.window_margin_override = Some(margin);
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

    pub fn into_element<H: UiHost>(
        self,
        cx: &mut ElementCx<'_, H>,
        trigger: impl FnOnce(&mut ElementCx<'_, H>) -> AnyElement,
        content: impl FnOnce(&mut ElementCx<'_, H>) -> AnyElement,
    ) -> AnyElement {
        cx.scope(|cx| {
            cx.observe_model(&self.open, Invalidation::Paint);

            let theme = Theme::global(&*cx.app).clone();
            let is_open = cx.app.models().get_copied(&self.open).unwrap_or(false);

            let trigger = trigger(cx);
            let trigger_id = trigger.id;

            let presence = presence::fade_presence(cx, is_open, 4);

            if presence.present {
                let overlay_root_name = window_overlays::popover_root_name(trigger_id);
                let align = self.align;
                let side = self.side;
                let side_offset = self.side_offset;
                let window_margin = self.window_margin_override.unwrap_or_else(|| {
                    theme
                        .metric_by_key("component.popover.window_margin")
                        .unwrap_or(Px(8.0))
                });

                let opacity = presence.opacity;
                let overlay_children = cx.with_root_name(&overlay_root_name, |cx| {
                    let anchor = overlay::anchor_bounds_for_element(cx, trigger_id);
                    let Some(anchor) = anchor else {
                        return Vec::new();
                    };

                    let content = content(cx);
                    let content_id = content.id;

                    let outer = overlay::outer_bounds_with_window_margin(cx.bounds, window_margin);

                    let last_content_size = cx.last_bounds_for_element(content_id).map(|r| r.size);
                    let estimated = Size::new(Px(256.0), Px(160.0));
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

                    let placed = anchored_panel_bounds_sized(
                        outer,
                        anchor,
                        content_size,
                        side_offset,
                        side,
                        align,
                    );

                    let wrapper = cx.container(
                        ContainerProps {
                            layout: LayoutStyle {
                                position: PositionStyle::Absolute,
                                inset: InsetStyle {
                                    left: Some(placed.origin.x),
                                    top: Some(placed.origin.y),
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
                            layout: opacity_layout,
                            opacity,
                        },
                        |_cx| vec![wrapper],
                    )]
                });

                let initial_focus = if let Some(id) = self.initial_focus {
                    Some(id)
                } else if self.auto_focus {
                    None
                } else {
                    Some(trigger_id)
                };

                window_overlays::request_dismissible_popover(
                    cx,
                    window_overlays::DismissiblePopoverRequest {
                        id: trigger_id,
                        root_name: overlay_root_name,
                        trigger: trigger_id,
                        open: self.open,
                        present: true,
                        initial_focus,
                        children: overlay_children,
                    },
                );
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

    pub fn into_element<H: UiHost>(self, _cx: &mut ElementCx<'_, H>) -> AnyElement {
        self.child
    }
}

fn popover_content_chrome(theme: &Theme) -> ChromeRefinement {
    let bg = theme
        .color_by_key("popover")
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
            layout: LayoutRefinement::default(),
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

    pub fn into_element<H: UiHost>(self, cx: &mut ElementCx<'_, H>) -> AnyElement {
        let theme = Theme::global(&*cx.app).clone();
        let radius = theme.metrics.radius_md;
        let shadow = decl_style::shadow_md(&theme, radius);

        let chrome = popover_content_chrome(&theme).merge(self.chrome);
        let props = decl_style::container_props(&theme, chrome, self.layout);
        let children = self.children;
        let label = self.a11y_label;

        let container = cx.container(
            ContainerProps {
                shadow: Some(shadow),
                ..props
            },
            move |_cx| children,
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

    pub fn into_element<H: UiHost>(self, cx: &mut ElementCx<'_, H>) -> AnyElement {
        let props = decl_style::container_props(
            Theme::global(&*cx.app),
            ChromeRefinement::default().pb(Space::N4),
            LayoutRefinement::default(),
        );
        let children = self.children;
        cx.container(props, move |_cx| children)
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

    pub fn into_element<H: UiHost>(self, cx: &mut ElementCx<'_, H>) -> AnyElement {
        let theme = Theme::global(&*cx.app).clone();
        let fg = theme
            .color_by_key("foreground")
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

#[cfg(test)]
mod tests {
    use super::*;
    use std::cell::Cell;
    use std::rc::Rc;

    use fret_app::App;
    use fret_components_ui::declarative::action_hooks::ActionHooksExt;
    use fret_core::{
        AppWindowId, FrameId, PathCommand, Point, Rect, Size as CoreSize, SvgId, SvgService,
    };
    use fret_core::{PathConstraints, PathId, PathMetrics, PathService, PathStyle};
    use fret_core::{
        Px, TextBlobId, TextConstraints, TextMetrics, TextService, TextStyle as CoreTextStyle,
    };
    use fret_ui::UiTree;
    use fret_ui::element::PressableProps;

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
        underlay_id_out: Rc<Cell<Option<fret_ui::elements::GlobalElementId>>>,
        popover_focus_id_out: Rc<Cell<Option<fret_ui::elements::GlobalElementId>>>,
    ) -> fret_ui::elements::GlobalElementId {
        window_overlays::begin_frame(app, window);

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
                            layout.position = PositionStyle::Absolute;
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

                let popover = Popover::new(open).auto_focus(true).into_element(
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
                        PopoverContent::new(vec![focusable]).into_element(cx)
                    },
                );

                vec![underlay, popover]
            });

        ui.set_root(root);
        window_overlays::render(ui, app, services, window, bounds);
        trigger_id.expect("trigger id")
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
            underlay_id.clone(),
            popover_focus_cell.clone(),
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
            underlay_id.clone(),
            popover_focus_cell.clone(),
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
            underlay_id.clone(),
            popover_focus_cell.clone(),
        );
        ui.layout_all(&mut app, &mut services, bounds, 1.0);

        let underlay_id = underlay_id.get().expect("underlay element id");
        let underlay_node =
            fret_ui::elements::node_for_element(&mut app, window, underlay_id).expect("underlay");
        assert_eq!(ui.focus(), Some(underlay_node));
    }
}
