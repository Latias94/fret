use fret_app::App;
use fret_core::{
    AppWindowId, Event, FrameId, Modifiers, MouseButton, Point, PointerEvent, PointerId,
    PointerType, Px, Rect, SemanticsRole, Size as CoreSize,
};
use fret_runtime::Model;
use fret_ui::element::{LayoutStyle, Length, PressableA11y, PressableProps};
use fret_ui::tree::UiTree;
use fret_ui_kit::{LayoutRefinement, MetricRef, OverlayController, Space};
use std::sync::Arc;

#[path = "support/style_aware_services.rs"]
mod style_aware_services;
use style_aware_services::StyleAwareServices;

fn render_frame(
    ui: &mut UiTree<App>,
    app: &mut App,
    services: &mut dyn fret_core::UiServices,
    window: AppWindowId,
    bounds: Rect,
    clicked: Model<bool>,
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
        "carousel-pointer-passthrough",
        move |cx| {
            let handler: fret_ui::action::OnPressablePointerDown =
                Arc::new(move |host, _action_cx, _down| {
                    let _ = host.models_mut().update(&clicked, |v| *v = true);
                    fret_ui::action::PressablePointerDownResult::SkipDefaultAndStopPropagation
                });

            let pressable = cx.pressable(
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
                        label: Some(Arc::from("Inner pressable")),
                        test_id: Some(Arc::from("carousel-inner-pressable")),
                        ..Default::default()
                    },
                    ..Default::default()
                },
                move |cx, _st| {
                    cx.pressable_on_pointer_down(handler.clone());
                    vec![cx.text("Click")]
                },
            );

            let slide_1 = cx.container(Default::default(), move |_cx| vec![pressable]);
            let slide_2 = cx.container(Default::default(), move |_cx| vec![]);

            let carousel = fret_ui_shadcn::Carousel::new([slide_1, slide_2])
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
fn carousel_pointer_region_does_not_swallow_descendant_pressable_down() {
    let window = AppWindowId::default();
    let mut app = App::new();
    let mut ui: UiTree<App> = UiTree::new();
    ui.set_window(window);
    let mut services = StyleAwareServices::default();

    let clicked = app.models_mut().insert(false);
    let bounds = Rect::new(
        Point::new(Px(0.0), Px(0.0)),
        CoreSize::new(Px(320.0), Px(200.0)),
    );

    render_frame(
        &mut ui,
        &mut app,
        &mut services,
        window,
        bounds,
        clicked.clone(),
    );

    ui.dispatch_event(
        &mut app,
        &mut services,
        &Event::Pointer(PointerEvent::Down {
            pointer_id: PointerId(0),
            position: Point::new(Px(20.0), Px(20.0)),
            button: MouseButton::Left,
            modifiers: Modifiers::default(),
            click_count: 1,
            pointer_type: PointerType::Mouse,
        }),
    );

    assert_eq!(app.models().get_copied(&clicked), Some(true));
}
