use fret_app::App;
use fret_core::{AppWindowId, FrameId, Point, Px, Rect, SemanticsRole, Size as CoreSize};
use fret_ui::tree::UiTree;
use fret_ui_kit::{LayoutRefinement, MetricRef, OverlayController, Space};
use fret_ui_shadcn::facade as shadcn;

#[path = "support/style_aware_services.rs"]
mod style_aware_services;
use style_aware_services::StyleAwareServices;

fn window_bounds() -> Rect {
    Rect::new(
        Point::new(Px(0.0), Px(0.0)),
        CoreSize::new(Px(320.0), Px(200.0)),
    )
}

fn find_by_test_id<'a>(
    snap: &'a fret_core::SemanticsSnapshot,
    role: SemanticsRole,
    test_id: &str,
) -> &'a fret_core::SemanticsNode {
    snap.nodes
        .iter()
        .find(|n| n.role == role && n.test_id.as_deref() == Some(test_id))
        .unwrap_or_else(|| panic!("missing semantics node role={role:?} test_id={test_id:?}"))
}

fn render_frame(
    ui: &mut UiTree<App>,
    app: &mut App,
    services: &mut dyn fret_core::UiServices,
    window: AppWindowId,
    bounds: Rect,
) {
    let next_frame = FrameId(app.frame_id().0.saturating_add(1));
    app.set_frame_id(next_frame);

    OverlayController::begin_frame(app, window);
    let root = fret_ui::declarative::render_root(
        ui,
        app,
        services,
        window,
        bounds,
        "carousel-a11y-semantics",
        move |cx| {
            let slides = (0..5).map(|_| cx.container(Default::default(), |_cx| vec![]));
            let carousel = shadcn::Carousel::new(slides)
                .track_start_neg_margin(Space::N0)
                .item_padding_start(Space::N0)
                .item_basis_main_px(Px(200.0))
                .refine_layout(
                    LayoutRefinement::default()
                        .w_px(MetricRef::Px(Px(200.0)))
                        .h_px(MetricRef::Px(Px(120.0))),
                )
                .refine_viewport_layout(LayoutRefinement::default().h_px(MetricRef::Px(Px(120.0))))
                .test_id("carousel-a11y")
                .into_element(cx);
            vec![carousel]
        },
    );
    ui.set_root(root);
    OverlayController::render(ui, app, services, window, bounds);
    ui.request_semantics_snapshot();
    ui.layout_all(app, services, bounds, 1.0);
}

#[test]
fn carousel_publishes_panel_and_slide_group_labels() {
    let window = AppWindowId::default();
    let bounds = window_bounds();

    let mut app = App::new();
    let mut ui: UiTree<App> = UiTree::new();
    ui.set_window(window);
    let mut services = StyleAwareServices::default();

    for _ in 0..3 {
        render_frame(&mut ui, &mut app, &mut services, window, bounds);
    }

    let snap = ui.semantics_snapshot_arc().expect("semantics snapshot");
    let root = find_by_test_id(&snap, SemanticsRole::Region, "carousel-a11y");
    assert_eq!(root.extra.role_description.as_deref(), Some("carousel"));
    assert_eq!(root.label.as_deref(), Some("Carousel"));

    for idx in 1..=5 {
        let test_id = format!("carousel-a11y-item-{idx}");
        let slide = find_by_test_id(&snap, SemanticsRole::Group, &test_id);
        assert_eq!(slide.extra.role_description.as_deref(), Some("slide"));
        let expected = format!("Slide {idx} of 5");
        assert_eq!(
            slide.label.as_deref(),
            Some(expected.as_str()),
            "unexpected slide label for test_id={test_id:?}"
        );
    }
}
