use std::sync::Arc;

use fret_core::{
    Color, Corners, Edges, FontId, FontWeight, Px, SemanticsRole, TextOverflow, TextStyle, TextWrap,
};
use fret_runtime::Model;
use fret_ui::element::{
    AnyElement, ContainerProps, InsetStyle, LayoutStyle, Length, OpacityProps, Overflow,
    PositionStyle, PressableProps, SemanticsProps, SizeStyle, TextProps, VisualTransformProps,
};
use fret_ui::overlay_placement::Side;
use fret_ui::{ElementContext, Theme, UiHost};
use fret_ui_kit::declarative::action_hooks::ActionHooksExt as _;
use fret_ui_kit::declarative::model_watch::ModelWatchExt as _;
use fret_ui_kit::declarative::style as decl_style;
use fret_ui_kit::{
    ChromeRefinement, ColorRef, LayoutRefinement, OverlayController, OverlayPresence,
    OverlayRequest, Space,
};

use crate::layout as shadcn_layout;
use crate::overlay_motion;

fn default_overlay_color() -> Color {
    Color {
        r: 0.0,
        g: 0.0,
        b: 0.0,
        a: 0.8,
    }
}

#[derive(Debug, Clone, Copy, Default, PartialEq, Eq)]
pub enum SheetSide {
    Left,
    #[default]
    Right,
    Top,
    Bottom,
}

/// shadcn/ui `Sheet` (v4).
///
/// This is a modal overlay (barrier-backed) installed via the component-layer overlay manager.
/// The barrier provides the "overlay click to dismiss" policy when configured.
#[derive(Clone)]
pub struct Sheet {
    open: Model<bool>,
    side: SheetSide,
    size_override: Option<Px>,
    overlay_closable: bool,
    overlay_color: Option<Color>,
}

impl std::fmt::Debug for Sheet {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.debug_struct("Sheet")
            .field("open", &"<model>")
            .field("side", &self.side)
            .field("size_override", &self.size_override)
            .field("overlay_closable", &self.overlay_closable)
            .field("overlay_color", &self.overlay_color)
            .finish()
    }
}

impl Sheet {
    pub fn new(open: Model<bool>) -> Self {
        Self {
            open,
            side: SheetSide::default(),
            size_override: None,
            overlay_closable: true,
            overlay_color: None,
        }
    }

    pub fn side(mut self, side: SheetSide) -> Self {
        self.side = side;
        self
    }

    /// Sets the sheet size:
    /// - width for `Left` / `Right`
    /// - height for `Top` / `Bottom`
    pub fn size(mut self, size: Px) -> Self {
        self.size_override = Some(size);
        self
    }

    pub fn overlay_closable(mut self, overlay_closable: bool) -> Self {
        self.overlay_closable = overlay_closable;
        self
    }

    pub fn overlay_color(mut self, overlay_color: Color) -> Self {
        self.overlay_color = Some(overlay_color);
        self
    }

    pub fn into_element<H: UiHost>(
        self,
        cx: &mut ElementContext<'_, H>,
        trigger: impl FnOnce(&mut ElementContext<'_, H>) -> AnyElement,
        content: impl FnOnce(&mut ElementContext<'_, H>) -> AnyElement,
    ) -> AnyElement {
        cx.scope(|cx| {
            let theme = Theme::global(&*cx.app).clone();
            let is_open = cx.watch_model(&self.open).copied().unwrap_or(false);

            let trigger = trigger(cx);
            let id = trigger.id;
            let overlay_root_name = OverlayController::modal_root_name(id);

            let motion = OverlayController::transition_with_durations(
                cx,
                is_open,
                overlay_motion::SHADCN_MOTION_TICKS_500,
                overlay_motion::SHADCN_MOTION_TICKS_300,
            );
            let overlay_presence = OverlayPresence {
                present: motion.present,
                interactive: is_open,
            };

            if overlay_presence.present {
                let overlay_color = self.overlay_color.unwrap_or_else(default_overlay_color);
                let overlay_closable = self.overlay_closable;
                let sheet_side = self.side;

                let default_size = theme
                    .metric_by_key("component.sheet.size")
                    .or_else(|| theme.metric_by_key("component.sheet.width"))
                    .unwrap_or(Px(350.0));
                let size = self.size_override.unwrap_or(default_size);

                let opacity = motion.progress;
                let overlay_children = cx.with_root_name(&overlay_root_name, |cx| {
                    let barrier_layout = LayoutStyle {
                        position: PositionStyle::Absolute,
                        inset: InsetStyle {
                            top: Some(Px(0.0)),
                            right: Some(Px(0.0)),
                            bottom: Some(Px(0.0)),
                            left: Some(Px(0.0)),
                        },
                        size: SizeStyle {
                            width: Length::Fill,
                            height: Length::Fill,
                            ..Default::default()
                        },
                        ..Default::default()
                    };

                    let barrier = if overlay_closable {
                        let open = self.open.clone();
                        cx.pressable(
                            PressableProps {
                                layout: barrier_layout,
                                enabled: true,
                                focusable: false,
                                ..Default::default()
                            },
                            move |cx, _st| {
                                cx.pressable_set_bool(&open, false);
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
                                        background: Some(overlay_color),
                                        shadow: None,
                                        border: Edges::all(Px(0.0)),
                                        border_color: None,
                                        corner_radii: Corners::all(Px(0.0)),
                                    },
                                    |_cx| Vec::new(),
                                )]
                            },
                        )
                    } else {
                        cx.container(
                            ContainerProps {
                                layout: barrier_layout,
                                padding: Edges::all(Px(0.0)),
                                background: Some(overlay_color),
                                shadow: None,
                                border: Edges::all(Px(0.0)),
                                border_color: None,
                                corner_radii: Corners::all(Px(0.0)),
                            },
                            |_cx| Vec::new(),
                        )
                    };

                    let content = content(cx);

                    let outer = cx.bounds;
                    let max_w = outer.size.width;
                    let max_h = outer.size.height;

                    let sheet_w = Px(size.0.min(max_w.0).max(0.0));
                    let sheet_h = Px(size.0.min(max_h.0).max(0.0));

                    let (inset, size) = match sheet_side {
                        SheetSide::Right => (
                            InsetStyle {
                                top: Some(Px(0.0)),
                                right: Some(Px(0.0)),
                                bottom: Some(Px(0.0)),
                                left: None,
                            },
                            SizeStyle {
                                width: Length::Px(sheet_w),
                                height: Length::Fill,
                                ..Default::default()
                            },
                        ),
                        SheetSide::Left => (
                            InsetStyle {
                                top: Some(Px(0.0)),
                                right: None,
                                bottom: Some(Px(0.0)),
                                left: Some(Px(0.0)),
                            },
                            SizeStyle {
                                width: Length::Px(sheet_w),
                                height: Length::Fill,
                                ..Default::default()
                            },
                        ),
                        SheetSide::Top => (
                            InsetStyle {
                                top: Some(Px(0.0)),
                                right: Some(Px(0.0)),
                                bottom: None,
                                left: Some(Px(0.0)),
                            },
                            SizeStyle {
                                width: Length::Fill,
                                height: Length::Px(sheet_h),
                                ..Default::default()
                            },
                        ),
                        SheetSide::Bottom => (
                            InsetStyle {
                                top: None,
                                right: Some(Px(0.0)),
                                bottom: Some(Px(0.0)),
                                left: Some(Px(0.0)),
                            },
                            SizeStyle {
                                width: Length::Fill,
                                height: Length::Px(sheet_h),
                                ..Default::default()
                            },
                        ),
                    };

                    let motion_side = match sheet_side {
                        SheetSide::Left => Side::Left,
                        SheetSide::Right => Side::Right,
                        SheetSide::Top => Side::Top,
                        SheetSide::Bottom => Side::Bottom,
                    };
                    let motion_distance = match sheet_side {
                        SheetSide::Left | SheetSide::Right => sheet_w,
                        SheetSide::Top | SheetSide::Bottom => sheet_h,
                    };
                    let slide = overlay_motion::shadcn_modal_slide_transform(
                        motion_side,
                        motion_distance,
                        opacity,
                    );

                    let wrapper = cx.container(
                        ContainerProps {
                            layout: LayoutStyle {
                                position: PositionStyle::Absolute,
                                inset,
                                size,
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
                            layout: opacity_layout.clone(),
                            opacity,
                        },
                        move |cx| {
                            vec![
                                barrier,
                                cx.visual_transform_props(
                                    VisualTransformProps {
                                        layout: opacity_layout,
                                        transform: slide,
                                    },
                                    move |_cx| vec![wrapper],
                                ),
                            ]
                        },
                    )]
                });

                let mut request = OverlayRequest::modal(
                    id,
                    Some(id),
                    self.open,
                    overlay_presence,
                    overlay_children,
                );
                request.root_name = Some(overlay_root_name);
                OverlayController::request(cx, request);
            }

            trigger
        })
    }
}

/// shadcn/ui `SheetContent` (v4).
#[derive(Debug, Clone)]
pub struct SheetContent {
    children: Vec<AnyElement>,
    chrome: ChromeRefinement,
    layout: LayoutRefinement,
}

impl SheetContent {
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

        let bg = theme
            .color_by_key("background")
            .unwrap_or(theme.colors.panel_background);
        let border = theme
            .color_by_key("border")
            .unwrap_or(theme.colors.panel_border);

        let radius = theme.metrics.radius_lg;
        let shadow = decl_style::shadow_lg(&theme, radius);

        let chrome = ChromeRefinement::default()
            .border_1()
            .bg(ColorRef::Color(bg))
            .border_color(ColorRef::Color(border))
            .p(Space::N6)
            .merge(self.chrome);

        let layout = LayoutRefinement::default()
            .w_full()
            .h_full()
            .overflow_hidden()
            .merge(self.layout);

        let props = decl_style::container_props(&theme, chrome, layout);
        let children = self.children;
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
                role: SemanticsRole::Dialog,
                ..Default::default()
            },
            move |_cx| vec![container],
        )
    }
}

/// shadcn/ui `SheetHeader` (v4).
#[derive(Debug, Clone)]
pub struct SheetHeader {
    children: Vec<AnyElement>,
}

impl SheetHeader {
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

/// shadcn/ui `SheetFooter` (v4).
#[derive(Debug, Clone)]
pub struct SheetFooter {
    children: Vec<AnyElement>,
}

impl SheetFooter {
    pub fn new(children: Vec<AnyElement>) -> Self {
        Self { children }
    }

    pub fn into_element<H: UiHost>(self, cx: &mut ElementContext<'_, H>) -> AnyElement {
        let props = decl_style::container_props(
            Theme::global(&*cx.app),
            ChromeRefinement::default().pt(Space::N4),
            LayoutRefinement::default(),
        );
        let children = self.children;
        shadcn_layout::container_hstack(
            cx,
            props,
            fret_ui_kit::declarative::stack::HStackProps::default()
                .gap(Space::N2)
                .justify_end()
                .items_center(),
            children,
        )
    }
}

/// shadcn/ui `SheetTitle` (v4).
#[derive(Debug, Clone)]
pub struct SheetTitle {
    text: Arc<str>,
}

impl SheetTitle {
    pub fn new(text: impl Into<Arc<str>>) -> Self {
        Self { text: text.into() }
    }

    pub fn into_element<H: UiHost>(self, cx: &mut ElementContext<'_, H>) -> AnyElement {
        let theme = Theme::global(&*cx.app).clone();
        let fg = theme
            .color_by_key("foreground")
            .unwrap_or(theme.colors.text_primary);

        let px = theme
            .metric_by_key("component.sheet.title_px")
            .or_else(|| theme.metric_by_key("font.size"))
            .unwrap_or(theme.metrics.font_size);
        let line_height = theme
            .metric_by_key("component.sheet.title_line_height")
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

/// shadcn/ui `SheetDescription` (v4).
#[derive(Debug, Clone)]
pub struct SheetDescription {
    text: Arc<str>,
}

impl SheetDescription {
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
            .metric_by_key("component.sheet.description_px")
            .or_else(|| theme.metric_by_key("font.size"))
            .unwrap_or(theme.metrics.font_size);
        let line_height = theme
            .metric_by_key("component.sheet.description_line_height")
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
    use fret_core::{AppWindowId, PathCommand, Point, Rect, Size, SvgId, SvgService};
    use fret_core::{PathConstraints, PathId, PathMetrics, PathService, PathStyle};
    use fret_core::{
        Px, TextBlobId, TextConstraints, TextMetrics, TextService, TextStyle as CoreTextStyle,
    };
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
                    size: Size::new(Px(0.0), Px(0.0)),
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

    fn render_sheet_frame(
        ui: &mut UiTree<App>,
        app: &mut App,
        services: &mut dyn fret_core::UiServices,
        window: AppWindowId,
        bounds: Rect,
        open: Model<bool>,
        overlay_closable: bool,
        side: SheetSide,
        content_id_out: Rc<Cell<Option<fret_ui::elements::GlobalElementId>>>,
        initial_focus_id_out: Rc<Cell<Option<fret_ui::elements::GlobalElementId>>>,
    ) -> fret_ui::elements::GlobalElementId {
        OverlayController::begin_frame(app, window);

        let mut trigger_id: Option<fret_ui::elements::GlobalElementId> = None;

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
                        cx.pressable_toggle_bool(&open);
                        trigger_id = Some(id);
                        vec![cx.container(ContainerProps::default(), |_cx| Vec::new())]
                    },
                );

                let sheet = Sheet::new(open)
                    .side(side)
                    .overlay_closable(overlay_closable)
                    .size(Px(300.0))
                    .into_element(
                        cx,
                        |_cx| trigger,
                        move |cx| {
                            let focusable = cx.pressable_with_id(
                                PressableProps {
                                    layout: {
                                        let mut layout = LayoutStyle::default();
                                        layout.size.width = Length::Px(Px(200.0));
                                        layout.size.height = Length::Px(Px(44.0));
                                        layout
                                    },
                                    enabled: true,
                                    focusable: true,
                                    ..Default::default()
                                },
                                |cx, _st, id| {
                                    initial_focus_id_out.set(Some(id));
                                    vec![cx.container(ContainerProps::default(), |_cx| Vec::new())]
                                },
                            );

                            let content = SheetContent::new(vec![focusable]).into_element(cx);
                            content_id_out.set(Some(content.id));
                            content
                        },
                    );

                vec![sheet]
            });

        ui.set_root(root);
        OverlayController::render(ui, app, services, window, bounds);
        trigger_id.expect("trigger id")
    }

    #[test]
    fn sheet_overlay_click_closes_when_overlay_closable() {
        let window = AppWindowId::default();
        let mut app = App::new();
        let mut ui: UiTree<App> = UiTree::new();
        ui.set_window(window);

        let open = app.models_mut().insert(false);
        let content_id: Rc<Cell<Option<fret_ui::elements::GlobalElementId>>> =
            Rc::new(Cell::new(None));

        let mut services = FakeServices;
        let bounds = Rect::new(
            Point::new(Px(0.0), Px(0.0)),
            Size::new(Px(800.0), Px(600.0)),
        );

        // First frame: render closed.
        let _ = render_sheet_frame(
            &mut ui,
            &mut app,
            &mut services,
            window,
            bounds,
            open.clone(),
            true,
            SheetSide::Right,
            content_id.clone(),
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

        // Second frame: render open + overlay.
        let _ = render_sheet_frame(
            &mut ui,
            &mut app,
            &mut services,
            window,
            bounds,
            open.clone(),
            true,
            SheetSide::Right,
            content_id.clone(),
            Rc::new(Cell::new(None)),
        );
        ui.layout_all(&mut app, &mut services, bounds, 1.0);
        assert!(content_id.get().is_some());

        // Click inside sheet should not close.
        ui.dispatch_event(
            &mut app,
            &mut services,
            &fret_core::Event::Pointer(fret_core::PointerEvent::Down {
                position: Point::new(Px(780.0), Px(50.0)),
                button: fret_core::MouseButton::Left,
                modifiers: fret_core::Modifiers::default(),
            }),
        );
        ui.dispatch_event(
            &mut app,
            &mut services,
            &fret_core::Event::Pointer(fret_core::PointerEvent::Up {
                position: Point::new(Px(780.0), Px(50.0)),
                button: fret_core::MouseButton::Left,
                modifiers: fret_core::Modifiers::default(),
            }),
        );
        assert_eq!(app.models().get_copied(&open), Some(true));

        // Click outside sheet should close via barrier.
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
        assert_eq!(app.models().get_copied(&open), Some(false));
    }

    #[test]
    fn sheet_overlay_click_does_not_close_when_not_overlay_closable() {
        let window = AppWindowId::default();
        let mut app = App::new();
        let mut ui: UiTree<App> = UiTree::new();
        ui.set_window(window);

        let open = app.models_mut().insert(true);
        let content_id: Rc<Cell<Option<fret_ui::elements::GlobalElementId>>> =
            Rc::new(Cell::new(None));

        let mut services = FakeServices;
        let bounds = Rect::new(
            Point::new(Px(0.0), Px(0.0)),
            Size::new(Px(800.0), Px(600.0)),
        );

        let _ = render_sheet_frame(
            &mut ui,
            &mut app,
            &mut services,
            window,
            bounds,
            open.clone(),
            false,
            SheetSide::Right,
            content_id.clone(),
            Rc::new(Cell::new(None)),
        );
        ui.layout_all(&mut app, &mut services, bounds, 1.0);

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
    }

    #[test]
    fn sheet_escape_closes() {
        let window = AppWindowId::default();
        let mut app = App::new();
        let mut ui: UiTree<App> = UiTree::new();
        ui.set_window(window);

        let open = app.models_mut().insert(true);
        let content_id: Rc<Cell<Option<fret_ui::elements::GlobalElementId>>> =
            Rc::new(Cell::new(None));

        let mut services = FakeServices;
        let bounds = Rect::new(
            Point::new(Px(0.0), Px(0.0)),
            Size::new(Px(800.0), Px(600.0)),
        );

        let _ = render_sheet_frame(
            &mut ui,
            &mut app,
            &mut services,
            window,
            bounds,
            open.clone(),
            true,
            SheetSide::Right,
            content_id.clone(),
            Rc::new(Cell::new(None)),
        );
        ui.layout_all(&mut app, &mut services, bounds, 1.0);

        ui.dispatch_event(
            &mut app,
            &mut services,
            &fret_core::Event::KeyDown {
                key: fret_core::KeyCode::Escape,
                modifiers: fret_core::Modifiers::default(),
                repeat: false,
            },
        );

        assert_eq!(app.models().get_copied(&open), Some(false));
    }

    #[test]
    fn sheet_focuses_first_focusable_on_open_and_restores_trigger_on_close() {
        let window = AppWindowId::default();
        let mut app = App::new();
        let mut ui: UiTree<App> = UiTree::new();
        ui.set_window(window);

        let open = app.models_mut().insert(false);
        let content_id: Rc<Cell<Option<fret_ui::elements::GlobalElementId>>> =
            Rc::new(Cell::new(None));
        let initial_focus_cell: Rc<Cell<Option<fret_ui::elements::GlobalElementId>>> =
            Rc::new(Cell::new(None));

        let mut services = FakeServices;
        let bounds = Rect::new(
            Point::new(Px(0.0), Px(0.0)),
            Size::new(Px(800.0), Px(600.0)),
        );

        // First frame: closed.
        let trigger = render_sheet_frame(
            &mut ui,
            &mut app,
            &mut services,
            window,
            bounds,
            open.clone(),
            true,
            SheetSide::Right,
            content_id.clone(),
            initial_focus_cell.clone(),
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

        // Second frame: open.
        let _ = render_sheet_frame(
            &mut ui,
            &mut app,
            &mut services,
            window,
            bounds,
            open.clone(),
            true,
            SheetSide::Right,
            content_id.clone(),
            initial_focus_cell.clone(),
        );
        ui.layout_all(&mut app, &mut services, bounds, 1.0);

        let initial_focus_element_id = initial_focus_cell.get().expect("initial focus element id");
        let initial_focus_node =
            fret_ui::elements::node_for_element(&mut app, window, initial_focus_element_id)
                .expect("initial focus node");
        assert_eq!(ui.focus(), Some(initial_focus_node));

        // Close via Escape and render a few frames to allow the close animation to finish and the
        // overlay manager to restore focus when the layer is uninstalled.
        ui.dispatch_event(
            &mut app,
            &mut services,
            &fret_core::Event::KeyDown {
                key: fret_core::KeyCode::Escape,
                modifiers: fret_core::Modifiers::default(),
                repeat: false,
            },
        );
        assert_eq!(app.models().get_copied(&open), Some(false));

        let settle_frames = crate::overlay_motion::SHADCN_MOTION_TICKS_300 as usize + 1;
        for _ in 0..settle_frames {
            let _ = render_sheet_frame(
                &mut ui,
                &mut app,
                &mut services,
                window,
                bounds,
                open.clone(),
                true,
                SheetSide::Right,
                content_id.clone(),
                initial_focus_cell.clone(),
            );
            ui.layout_all(&mut app, &mut services, bounds, 1.0);
        }

        let trigger_node =
            fret_ui::elements::node_for_element(&mut app, window, trigger).expect("trigger node");
        assert_eq!(ui.focus(), Some(trigger_node));
    }
}
