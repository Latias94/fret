use fret_app::App;
use fret_core::{
    AppWindowId, Event, FrameId, KeyCode, Modifiers, Point, Px, Rect, SemanticsRole,
    Size as CoreSize,
};
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

fn find_by_label<'a>(
    snap: &'a fret_core::SemanticsSnapshot,
    role: SemanticsRole,
    label: &str,
) -> &'a fret_core::SemanticsNode {
    snap.nodes
        .iter()
        .find(|n| n.role == role && n.label.as_deref() == Some(label))
        .unwrap_or_else(|| panic!("missing semantics node role={role:?} label={label:?}"))
}

fn render_frame(
    ui: &mut UiTree<App>,
    app: &mut App,
    services: &mut dyn fret_core::UiServices,
    window: AppWindowId,
    bounds: Rect,
    api: Model<Option<fret_ui_shadcn::CarouselApi>>,
    opts: fret_ui_shadcn::CarouselOptions,
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
        "carousel-focus-watch-tab-scrolls",
        move |cx| {
            let slides = (1..=5).map(|idx| {
                let button = fret_ui_shadcn::Button::new(format!("Button {idx}")).into_element(cx);
                cx.container(Default::default(), move |_cx| vec![button])
            });

            let carousel = fret_ui_shadcn::Carousel::new(slides)
                .opts(opts)
                .api_handle_model(api)
                .track_start_neg_margin(Space::N0)
                .item_padding_start(Space::N0)
                .item_basis_main_px(Px(200.0))
                .refine_layout(
                    LayoutRefinement::default()
                        .w_px(MetricRef::Px(Px(200.0)))
                        .h_px(MetricRef::Px(Px(120.0))),
                )
                .refine_viewport_layout(LayoutRefinement::default().h_px(MetricRef::Px(Px(120.0))))
                .test_id("carousel-focus")
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
fn carousel_watch_focus_scrolls_to_focused_slide_after_tab() {
    let window = AppWindowId::default();
    let bounds = window_bounds();

    let mut app = App::new();
    let mut ui: UiTree<App> = UiTree::new();
    ui.set_window(window);
    let mut services = StyleAwareServices::default();

    let api = app.models_mut().insert(None::<fret_ui_shadcn::CarouselApi>);
    let opts = fret_ui_shadcn::CarouselOptions::default().watch_focus(true);

    for _ in 0..3 {
        render_frame(
            &mut ui,
            &mut app,
            &mut services,
            window,
            bounds,
            api.clone(),
            opts,
        );
    }

    let handle = app
        .models()
        .get_cloned(&api)
        .flatten()
        .expect("CarouselApi handle");
    let before = handle.snapshot(&mut app);
    assert_eq!(before.selected_index, 0, "expected initial selection");
    assert!(before.snap_count > 0, "expected measurable snaps");

    // Focus slide 1, then simulate a Tab-driven focus move into slide 2.
    let snap = ui.semantics_snapshot_arc().expect("semantics snapshot");
    let button1 = find_by_label(&snap, SemanticsRole::Button, "Button 1");
    let button2 = find_by_label(&snap, SemanticsRole::Button, "Button 2");
    ui.set_focus(Some(button1.id));

    ui.dispatch_event(
        &mut app,
        &mut services,
        &Event::KeyDown {
            key: KeyCode::Tab,
            modifiers: Modifiers::default(),
            repeat: false,
        },
    );
    ui.set_focus(Some(button2.id));

    for _ in 0..2 {
        render_frame(
            &mut ui,
            &mut app,
            &mut services,
            window,
            bounds,
            api.clone(),
            opts,
        );
    }

    let after = handle.snapshot(&mut app);
    assert_eq!(
        after.selected_index, 1,
        "expected focus watcher to scroll to slide 2; snapshot={after:?}"
    );
}
