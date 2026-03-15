use fret_app::App;
use fret_core::{AppWindowId, FrameId, Point, Px, Rect, Size as CoreSize};
use fret_ui::element::{ContainerProps, LayoutStyle, Length};
use fret_ui::tree::UiTree;
use fret_ui_kit::{LayoutRefinement, MetricRef, OverlayController, Space};
use fret_ui_shadcn::facade as shadcn;

#[path = "support/style_aware_services.rs"]
mod style_aware_services;
use style_aware_services::StyleAwareServices;

fn window_bounds() -> Rect {
    Rect::new(
        Point::new(Px(0.0), Px(0.0)),
        CoreSize::new(Px(320.0), Px(240.0)),
    )
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
        "carousel-auto-height",
        |cx| {
            let slides = (0..3).map(|_| {
                cx.container(
                    ContainerProps {
                        layout: {
                            let mut layout = LayoutStyle::default();
                            layout.size.width = Length::Fill;
                            layout.size.height = Length::Px(Px(80.0));
                            layout
                        },
                        ..Default::default()
                    },
                    |_cx| vec![],
                )
            });

            let carousel = shadcn::Carousel::new(slides)
                .track_start_neg_margin(Space::N0)
                .item_padding_start(Space::N0)
                .item_basis_main_px(Px(200.0))
                .refine_layout(LayoutRefinement::default().w_px(MetricRef::Px(Px(200.0))))
                .test_id("carousel-auto-height")
                .into_element(cx);

            vec![carousel]
        },
    );

    ui.set_root(root);
    ui.request_semantics_snapshot();
    OverlayController::render(ui, app, services, window, bounds);
    ui.layout_all(app, services, bounds, 1.0);
}

#[test]
fn carousel_viewport_height_is_intrinsic_by_default() {
    let window = AppWindowId::default();
    let bounds = window_bounds();

    let mut app = App::new();
    let mut ui: UiTree<App> = UiTree::new();
    ui.set_window(window);
    let mut services = StyleAwareServices::default();

    for _ in 0..3 {
        render_frame(&mut ui, &mut app, &mut services, window, bounds);
    }

    let snapshot = ui.semantics_snapshot().expect("semantics snapshot");
    let first_item = snapshot
        .nodes
        .iter()
        .find(|n| n.test_id.as_deref() == Some("carousel-auto-height-item-1"))
        .expect("expected first carousel item semantics node by test_id");

    assert!(
        first_item.bounds.size.height.0 >= 40.0,
        "expected carousel item to have a non-trivial height; bounds={:?}",
        first_item.bounds
    );
}
