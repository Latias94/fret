use fret_app::App;
use fret_core::{AppWindowId, Modifiers, MouseButton, Point, Px, Rect, Size as CoreSize};
use fret_runtime::Model;
use fret_ui::ElementContext;
use fret_ui::tree::UiTree;

#[path = "support/fake_services.rs"]
mod fake_services;
use fake_services::FakeServices;

fn bounds() -> Rect {
    Rect::new(
        Point::new(Px(0.0), Px(0.0)),
        CoreSize::new(Px(480.0), Px(240.0)),
    )
}

fn center_of(bounds: fret_core::Rect) -> Point {
    Point::new(
        Px(bounds.origin.x.0 + bounds.size.width.0 * 0.5),
        Px(bounds.origin.y.0 + bounds.size.height.0 * 0.5),
    )
}

#[test]
fn field_label_click_focuses_input_control() {
    let window = AppWindowId::default();
    let mut app = App::new();
    fret_ui_shadcn::shadcn_themes::apply_shadcn_new_york(
        &mut app,
        fret_ui_shadcn::shadcn_themes::ShadcnBaseColor::Neutral,
        fret_ui_shadcn::shadcn_themes::ShadcnColorScheme::Light,
    );

    let mut ui: UiTree<App> = UiTree::new();
    ui.set_window(window);
    let mut services = FakeServices;
    let model: Model<String> = app.models_mut().insert(String::new());
    let control_id = fret_ui_kit::primitives::control_registry::ControlId::from("input.email");

    let root = fret_ui::declarative::render_root(
        &mut ui,
        &mut app,
        &mut services,
        window,
        bounds(),
        "input-label-focus",
        |cx: &mut ElementContext<'_, App>| {
            vec![cx.column(fret_ui::element::ColumnProps::default(), |cx| {
                vec![
                    fret_ui_shadcn::FieldLabel::new("Email")
                        .for_control(control_id.clone())
                        .test_id("input.label")
                        .into_element(cx),
                    fret_ui_shadcn::Input::new(model.clone())
                        .control_id(control_id.clone())
                        .test_id("input.control")
                        .into_element(cx),
                ]
            })]
        },
    );
    ui.set_root(root);
    ui.request_semantics_snapshot();
    ui.layout_all(&mut app, &mut services, bounds(), 1.0);

    let snap = ui.semantics_snapshot_arc().expect("semantics snapshot");
    let label_node = snap
        .nodes
        .iter()
        .find(|n| n.test_id.as_deref() == Some("input.label"))
        .map(|n| n.id)
        .expect("label node");
    let input_node = snap
        .nodes
        .iter()
        .find(|n| n.test_id.as_deref() == Some("input.control"))
        .map(|n| n.id)
        .expect("input node");

    let label_bounds = ui.debug_node_bounds(label_node).expect("label bounds");
    let label_center = center_of(label_bounds);

    ui.dispatch_event(
        &mut app,
        &mut services,
        &fret_core::Event::Pointer(fret_core::PointerEvent::Down {
            pointer_id: fret_core::PointerId(7),
            position: label_center,
            button: MouseButton::Left,
            modifiers: Modifiers::default(),
            pointer_type: fret_core::PointerType::Mouse,
            click_count: 1,
        }),
    );
    ui.dispatch_event(
        &mut app,
        &mut services,
        &fret_core::Event::Pointer(fret_core::PointerEvent::Up {
            pointer_id: fret_core::PointerId(7),
            position: label_center,
            button: MouseButton::Left,
            modifiers: Modifiers::default(),
            is_click: true,
            click_count: 1,
            pointer_type: fret_core::PointerType::Mouse,
        }),
    );

    assert_eq!(ui.focus(), Some(input_node));
}
