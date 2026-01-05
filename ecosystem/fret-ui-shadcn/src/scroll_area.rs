use fret_ui::element::AnyElement;
use fret_ui::element::HoverRegionProps;
use fret_ui::scroll::ScrollHandle;
use fret_ui::{ElementContext, UiHost};
use fret_ui_kit::LayoutRefinement;
use fret_ui_kit::declarative::scroll;
use fret_ui_kit::primitives::scroll_area::DEFAULT_SCROLL_HIDE_DELAY_TICKS;
use fret_ui_kit::primitives::scroll_area::ScrollAreaType;

#[derive(Debug, Clone)]
pub struct ScrollArea {
    children: Vec<AnyElement>,
    show_scrollbar: bool,
    scrollbar_type: ScrollAreaType,
    scroll_hide_delay_ticks: u64,
    layout: LayoutRefinement,
    scroll_handle: Option<ScrollHandle>,
}

impl ScrollArea {
    pub fn new(children: Vec<AnyElement>) -> Self {
        Self {
            children,
            show_scrollbar: true,
            scrollbar_type: ScrollAreaType::default(),
            scroll_hide_delay_ticks: DEFAULT_SCROLL_HIDE_DELAY_TICKS,
            layout: LayoutRefinement::default(),
            scroll_handle: None,
        }
    }

    pub fn show_scrollbar(mut self, show: bool) -> Self {
        self.show_scrollbar = show;
        self
    }

    /// Matches Radix ScrollArea `type` outcome (best-effort).
    pub fn type_(mut self, scrollbar_type: ScrollAreaType) -> Self {
        self.scrollbar_type = scrollbar_type;
        self
    }

    /// Mirrors Radix `scrollHideDelay` (default 600ms).
    ///
    /// Fret currently expresses this value in frame-ish ticks (assuming ~60fps).
    pub fn scroll_hide_delay_ticks(mut self, ticks: u64) -> Self {
        self.scroll_hide_delay_ticks = ticks;
        self
    }

    pub fn refine_layout(mut self, layout: LayoutRefinement) -> Self {
        self.layout = self.layout.merge(layout);
        self
    }

    pub fn scroll_handle(mut self, handle: ScrollHandle) -> Self {
        self.scroll_handle = Some(handle);
        self
    }

    pub fn into_element<H: UiHost>(self, cx: &mut ElementContext<'_, H>) -> AnyElement {
        let children = self.children;
        let layout = self.layout;
        let show_scrollbar = self.show_scrollbar;
        let scrollbar_type = self.scrollbar_type;
        let scroll_hide_delay_ticks = self.scroll_hide_delay_ticks;
        let scroll_handle = self.scroll_handle;

        cx.hover_region(HoverRegionProps::default(), move |cx, hovered| {
            let handle = scroll_handle
                .unwrap_or_else(|| cx.with_state(ScrollHandle::default, |h| h.clone()));

            let show_scrollbar_now = show_scrollbar
                && fret_ui_kit::primitives::scroll_area::scrollbar_visibility(
                    cx,
                    scrollbar_type,
                    hovered,
                    handle.clone(),
                    scroll_hide_delay_ticks,
                )
                .visible;

            vec![scroll::overflow_scroll_with_handle(
                cx,
                layout,
                show_scrollbar_now,
                handle,
                move |_cx| children,
            )]
        })
    }
}

pub fn scroll_area<H: UiHost>(
    cx: &mut ElementContext<'_, H>,
    f: impl FnOnce(&mut ElementContext<'_, H>) -> Vec<AnyElement>,
) -> AnyElement {
    ScrollArea::new(f(cx)).into_element(cx)
}

#[cfg(test)]
mod tests {
    use super::*;
    use fret_app::App;
    use fret_core::{
        AppWindowId, Modifiers, MouseButtons, Point, Px, Rect, Size, SvgId, SvgService,
    };
    use fret_core::{PathCommand, PathConstraints, PathId, PathMetrics, PathService, PathStyle};
    use fret_core::{TextBlobId, TextConstraints, TextMetrics, TextService, TextStyle};
    use fret_runtime::TickId;
    use fret_ui::element::ColumnProps;
    use fret_ui::tree::UiTree;

    #[derive(Default)]
    struct FakeServices;

    impl TextService for FakeServices {
        fn prepare(
            &mut self,
            _text: &str,
            _style: &TextStyle,
            _constraints: TextConstraints,
        ) -> (TextBlobId, TextMetrics) {
            (
                TextBlobId::default(),
                TextMetrics {
                    size: Size::new(Px(10.0), Px(10.0)),
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

    fn bounds() -> Rect {
        Rect::new(
            Point::new(Px(0.0), Px(0.0)),
            Size::new(Px(400.0), Px(240.0)),
        )
    }

    fn render(
        ui: &mut UiTree<App>,
        app: &mut App,
        services: &mut dyn fret_core::UiServices,
        window: AppWindowId,
        ty: ScrollAreaType,
    ) -> fret_core::NodeId {
        let root =
            fret_ui::declarative::render_root(ui, app, services, window, bounds(), "sa", |cx| {
                vec![
                    ScrollArea::new(vec![cx.column(ColumnProps::default(), |cx| {
                        (0..50).map(|_| cx.text("Row")).collect()
                    })])
                    .type_(ty)
                    .into_element(cx),
                ]
            });
        ui.set_root(root);
        ui.layout_all(app, services, bounds(), 1.0);
        root
    }

    #[test]
    fn scroll_area_hover_type_shows_scrollbar_only_when_hovered() {
        let window = AppWindowId::default();
        let mut app = App::new();
        let mut ui: UiTree<App> = UiTree::new();
        ui.set_window(window);

        let mut services = FakeServices::default();

        let root = render(
            &mut ui,
            &mut app,
            &mut services,
            window,
            ScrollAreaType::Hover,
        );

        // Root -> HoverRegion -> Stack -> (Scroll) when not hovered.
        let hover_region = ui.children(root)[0];
        let stack = ui.children(hover_region)[0];
        assert_eq!(
            ui.children(stack).len(),
            1,
            "expected no scrollbar before hover"
        );

        ui.dispatch_event(
            &mut app,
            &mut services,
            &fret_core::Event::Pointer(fret_core::PointerEvent::Move {
                position: Point::new(Px(10.0), Px(10.0)),
                buttons: MouseButtons::default(),
                modifiers: Modifiers::default(),
            }),
        );

        let root = render(
            &mut ui,
            &mut app,
            &mut services,
            window,
            ScrollAreaType::Hover,
        );
        let hover_region = ui.children(root)[0];
        let stack = ui.children(hover_region)[0];
        assert_eq!(
            ui.children(stack).len(),
            2,
            "expected scrollbar to mount on hover"
        );
    }

    #[test]
    fn scroll_area_auto_type_shows_scrollbar_when_overflowing() {
        let window = AppWindowId::default();
        let mut app = App::new();
        let mut ui: UiTree<App> = UiTree::new();
        ui.set_window(window);

        let mut services = FakeServices::default();

        // First render establishes the scroll handle's viewport/content sizes during layout.
        let _ = render(
            &mut ui,
            &mut app,
            &mut services,
            window,
            ScrollAreaType::Auto,
        );
        let root = render(
            &mut ui,
            &mut app,
            &mut services,
            window,
            ScrollAreaType::Auto,
        );

        // Root -> HoverRegion -> Stack -> Scroll + Scrollbar (overflowing content).
        let hover_region = ui.children(root)[0];
        let stack = ui.children(hover_region)[0];
        assert_eq!(
            ui.children(stack).len(),
            2,
            "expected auto scrollbar to mount for overflow"
        );
    }

    #[test]
    fn scroll_area_scroll_type_shows_while_scrolling_then_hides() {
        let window = AppWindowId::default();
        let mut app = App::new();
        let mut ui: UiTree<App> = UiTree::new();
        ui.set_window(window);

        let mut services = FakeServices::default();
        let handle = ScrollHandle::default();

        let root = fret_ui::declarative::render_root(
            &mut ui,
            &mut app,
            &mut services,
            window,
            bounds(),
            "sa-scroll",
            |cx| {
                vec![
                    ScrollArea::new(vec![cx.column(ColumnProps::default(), |cx| {
                        (0..50).map(|_| cx.text("Row")).collect()
                    })])
                    .type_(ScrollAreaType::Scroll)
                    .scroll_hide_delay_ticks(4)
                    .scroll_handle(handle.clone())
                    .into_element(cx),
                ]
            },
        );
        ui.set_root(root);
        ui.layout_all(&mut app, &mut services, bounds(), 1.0);

        let hover_region = ui.children(root)[0];
        let stack = ui.children(hover_region)[0];
        assert_eq!(
            ui.children(stack).len(),
            1,
            "expected no scrollbar before any scrolling"
        );

        // Simulate a scroll delta by mutating the shared handle between frames.
        handle.set_offset(Point::new(Px(0.0), Px(10.0)));
        app.set_tick_id(TickId(1));

        let root = fret_ui::declarative::render_root(
            &mut ui,
            &mut app,
            &mut services,
            window,
            bounds(),
            "sa-scroll",
            |cx| {
                vec![
                    ScrollArea::new(vec![cx.column(ColumnProps::default(), |cx| {
                        (0..50).map(|_| cx.text("Row")).collect()
                    })])
                    .type_(ScrollAreaType::Scroll)
                    .scroll_hide_delay_ticks(4)
                    .scroll_handle(handle.clone())
                    .into_element(cx),
                ]
            },
        );
        ui.set_root(root);
        ui.layout_all(&mut app, &mut services, bounds(), 1.0);

        let hover_region = ui.children(root)[0];
        let stack = ui.children(hover_region)[0];
        assert_eq!(
            ui.children(stack).len(),
            2,
            "expected scrollbar while scrolling"
        );

        // Keep rendering without scroll input; after debounce + hide delay it should disappear.
        for n in 0..12 {
            app.set_tick_id(TickId(2 + n));
            let root = fret_ui::declarative::render_root(
                &mut ui,
                &mut app,
                &mut services,
                window,
                bounds(),
                "sa-scroll",
                |cx| {
                    vec![
                        ScrollArea::new(vec![cx.column(ColumnProps::default(), |cx| {
                            (0..50).map(|_| cx.text("Row")).collect()
                        })])
                        .type_(ScrollAreaType::Scroll)
                        .scroll_hide_delay_ticks(4)
                        .scroll_handle(handle.clone())
                        .into_element(cx),
                    ]
                },
            );
            ui.set_root(root);
            ui.layout_all(&mut app, &mut services, bounds(), 1.0);
        }

        let root = fret_ui::declarative::render_root(
            &mut ui,
            &mut app,
            &mut services,
            window,
            bounds(),
            "sa-scroll",
            |cx| {
                vec![
                    ScrollArea::new(vec![cx.column(ColumnProps::default(), |cx| {
                        (0..50).map(|_| cx.text("Row")).collect()
                    })])
                    .type_(ScrollAreaType::Scroll)
                    .scroll_hide_delay_ticks(4)
                    .scroll_handle(handle.clone())
                    .into_element(cx),
                ]
            },
        );
        ui.set_root(root);
        ui.layout_all(&mut app, &mut services, bounds(), 1.0);

        let hover_region = ui.children(root)[0];
        let stack = ui.children(hover_region)[0];
        assert_eq!(
            ui.children(stack).len(),
            1,
            "expected scrollbar to hide after scroll ends"
        );
    }
}
