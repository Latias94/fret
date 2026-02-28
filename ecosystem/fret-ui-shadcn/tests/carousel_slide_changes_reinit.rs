use fret_app::App;
use fret_core::{AppWindowId, FrameId, Point, Px, Rect, Size as CoreSize};
use fret_runtime::Model;
use fret_ui::tree::UiTree;
use fret_ui_kit::{LayoutRefinement, MetricRef, OverlayController, Space};

#[path = "support/style_aware_services.rs"]
mod style_aware_services;
use style_aware_services::StyleAwareServices;

fn window_bounds() -> Rect {
    Rect::new(
        Point::new(Px(0.0), Px(0.0)),
        CoreSize::new(Px(320.0), Px(200.0)),
    )
}

fn render_frame(
    ui: &mut UiTree<App>,
    app: &mut App,
    services: &mut dyn fret_core::UiServices,
    window: AppWindowId,
    bounds: Rect,
    api: Model<fret_ui_shadcn::CarouselApiSnapshot>,
    opts: fret_ui_shadcn::CarouselOptions,
    slide_order: &[u8],
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
        "carousel-slide-changes-reinit",
        move |cx| {
            let slides = slide_order.iter().map(|key| {
                cx.keyed(("slide", *key), |cx| {
                    cx.container(Default::default(), |_cx| vec![])
                })
            });
            let carousel = fret_ui_shadcn::Carousel::new(slides)
                .opts(opts)
                .api_snapshot_model(api)
                .track_start_neg_margin(Space::N0)
                .item_padding_start(Space::N0)
                .item_basis_main_px(Px(200.0))
                .refine_layout(
                    LayoutRefinement::default()
                        .w_px(MetricRef::Px(Px(200.0)))
                        .h_px(MetricRef::Px(Px(120.0))),
                )
                .refine_viewport_layout(LayoutRefinement::default().h_px(MetricRef::Px(Px(120.0))))
                .into_element(cx);
            vec![carousel]
        },
    );
    ui.set_root(root);
    OverlayController::render(ui, app, services, window, bounds);
    ui.layout_all(app, services, bounds, 1.0);
}

#[test]
fn carousel_reinit_generation_increments_on_slide_reorder() {
    let window = AppWindowId::default();
    let bounds = window_bounds();

    let mut app = App::new();
    let mut ui: UiTree<App> = UiTree::new();
    ui.set_window(window);
    let mut services = StyleAwareServices::default();

    let api = app
        .models_mut()
        .insert(fret_ui_shadcn::CarouselApiSnapshot::default());
    let opts = fret_ui_shadcn::CarouselOptions::default().embla_engine(true);

    let order_a = [0u8, 1, 2, 3, 4];
    for _ in 0..3 {
        render_frame(
            &mut ui,
            &mut app,
            &mut services,
            window,
            bounds,
            api.clone(),
            opts,
            &order_a,
        );
    }

    let before = app.models().get_copied(&api).expect("api snapshot");
    assert!(
        before.snap_count > 0,
        "expected measurable snaps; snapshot={before:?}"
    );

    let order_b = [0u8, 2, 1, 3, 4];
    let mut after = before;
    for _ in 0..8 {
        render_frame(
            &mut ui,
            &mut app,
            &mut services,
            window,
            bounds,
            api.clone(),
            opts,
            &order_b,
        );
        after = app.models().get_copied(&api).expect("api snapshot");
        if after.reinit_generation > before.reinit_generation {
            break;
        }
    }

    assert_eq!(
        after.reinit_generation,
        before.reinit_generation.saturating_add(1),
        "expected reinit_generation to increment once on slide reorder; before={before:?} after={after:?}"
    );
}
