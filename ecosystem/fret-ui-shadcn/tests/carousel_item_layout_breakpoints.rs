use fret_app::App;
use fret_core::{AppWindowId, FrameId, Point, Px, Rect, Size as CoreSize};
use fret_ui::tree::UiTree;
use fret_ui_kit::{LayoutRefinement, MetricRef, OverlayController, Space};
use fret_ui_shadcn::facade as shadcn;

#[path = "support/style_aware_services.rs"]
mod style_aware_services;
use style_aware_services::StyleAwareServices;

fn find_by_test_id<'a>(
    snap: &'a fret_core::SemanticsSnapshot,
    id: &str,
) -> &'a fret_core::SemanticsNode {
    snap.nodes
        .iter()
        .find(|n| n.test_id.as_deref() == Some(id))
        .unwrap_or_else(|| panic!("missing semantics node with test_id={id}"))
}

fn render_frame(
    ui: &mut UiTree<App>,
    app: &mut App,
    services: &mut dyn fret_core::UiServices,
    window: AppWindowId,
    bounds: Rect,
) -> fret_core::SemanticsSnapshot {
    let next_frame = FrameId(app.frame_id().0.saturating_add(1));
    app.set_frame_id(next_frame);

    OverlayController::begin_frame(app, window);
    let root =
        fret_ui::declarative::render_root(ui, app, services, window, bounds, "carousel", |cx| {
            let slide_1 = cx.container(Default::default(), move |cx| vec![cx.text("1")]);
            let slide_2 = cx.container(Default::default(), move |cx| vec![cx.text("2")]);

            let slides = [
                shadcn::CarouselItem::new(slide_1)
                    .layout_breakpoint(Px(300.0), LayoutRefinement::default().basis_fraction(0.5)),
                shadcn::CarouselItem::new(slide_2),
            ];

            let carousel = shadcn::Carousel::new(slides)
                .track_start_neg_margin(Space::N0)
                .item_padding_start(Space::N0)
                .refine_layout(
                    LayoutRefinement::default()
                        .w_px(MetricRef::Px(Px(320.0)))
                        .h_px(MetricRef::Px(Px(120.0))),
                )
                .into_element(cx);

            vec![carousel]
        });
    ui.set_root(root);
    ui.request_semantics_snapshot();
    ui.layout_all(app, services, bounds, 1.0);

    ui.semantics_snapshot()
        .cloned()
        .expect("expected semantics snapshot")
}

fn render_frame_viewport_breakpoint(
    ui: &mut UiTree<App>,
    app: &mut App,
    services: &mut dyn fret_core::UiServices,
    window: AppWindowId,
    bounds: Rect,
) -> fret_core::SemanticsSnapshot {
    let next_frame = FrameId(app.frame_id().0.saturating_add(1));
    app.set_frame_id(next_frame);

    OverlayController::begin_frame(app, window);
    let root =
        fret_ui::declarative::render_root(ui, app, services, window, bounds, "carousel", |cx| {
            let slide_1 = cx.container(Default::default(), move |cx| vec![cx.text("1")]);
            let slide_2 = cx.container(Default::default(), move |cx| vec![cx.text("2")]);

            let slides = [
                shadcn::CarouselItem::new(slide_1).viewport_layout_breakpoint(
                    Px(300.0),
                    LayoutRefinement::default().basis_fraction(0.5),
                ),
                shadcn::CarouselItem::new(slide_2),
            ];

            let carousel = shadcn::Carousel::new(slides)
                .track_start_neg_margin(Space::N0)
                .item_padding_start(Space::N0)
                .refine_layout(
                    LayoutRefinement::default()
                        .w_px(MetricRef::Px(Px(320.0)))
                        .h_px(MetricRef::Px(Px(120.0))),
                )
                .into_element(cx);

            vec![carousel]
        });
    ui.set_root(root);
    ui.request_semantics_snapshot();
    ui.layout_all(app, services, bounds, 1.0);

    ui.semantics_snapshot()
        .cloned()
        .expect("expected semantics snapshot")
}

#[test]
fn carousel_item_layout_breakpoints_apply_after_viewport_width_is_measured() {
    let window = AppWindowId::default();
    let mut app = App::new();
    let mut ui: UiTree<App> = UiTree::new();
    ui.set_window(window);
    let mut services = StyleAwareServices::default();

    let bounds = Rect::new(
        Point::new(Px(0.0), Px(0.0)),
        CoreSize::new(Px(360.0), Px(200.0)),
    );

    let snap_1 = render_frame(&mut ui, &mut app, &mut services, window, bounds);
    let item_1_1 = find_by_test_id(&snap_1, "carousel-item-1");
    assert!(
        (item_1_1.bounds.size.width.0 - 320.0).abs() <= 1.0,
        "frame 1: expected item width≈320 got={}",
        item_1_1.bounds.size.width.0
    );

    // Second frame: the carousel viewport width is recorded for subsequent breakpoint evaluation.
    let snap_2 = render_frame(&mut ui, &mut app, &mut services, window, bounds);
    let item_1_2 = find_by_test_id(&snap_2, "carousel-item-1");
    assert!(
        (item_1_2.bounds.size.width.0 - 320.0).abs() <= 1.0,
        "frame 2: expected item width≈320 got={}",
        item_1_2.bounds.size.width.0
    );

    // Third frame: the recorded viewport width is visible to breakpoint evaluation.
    let snap_3 = render_frame(&mut ui, &mut app, &mut services, window, bounds);
    let item_1_3 = find_by_test_id(&snap_3, "carousel-item-1");
    assert!(
        (item_1_3.bounds.size.width.0 - 160.0).abs() <= 1.0,
        "frame 3: expected item width≈160 got={}",
        item_1_3.bounds.size.width.0
    );
}

#[test]
fn carousel_item_viewport_layout_breakpoints_apply_immediately() {
    let window = AppWindowId::default();
    let mut app = App::new();
    let mut ui: UiTree<App> = UiTree::new();
    ui.set_window(window);
    let mut services = StyleAwareServices::default();

    let bounds = Rect::new(
        Point::new(Px(0.0), Px(0.0)),
        CoreSize::new(Px(360.0), Px(200.0)),
    );

    let snap_1 = render_frame_viewport_breakpoint(&mut ui, &mut app, &mut services, window, bounds);
    let item_1 = find_by_test_id(&snap_1, "carousel-item-1");
    assert!(
        (item_1.bounds.size.width.0 - 160.0).abs() <= 1.0,
        "frame 1: expected item width≈160 got={}",
        item_1.bounds.size.width.0
    );
}
