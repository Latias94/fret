use fret_app::App;
use fret_core::{AppWindowId, Modifiers, MouseButton, Point, Px, Rect, Size as CoreSize};
use fret_runtime::Model;
use fret_ui::ElementContext;
use fret_ui::tree::UiTree;
use fret_ui_kit::ui::UiElementSinkExt as _;
use fret_ui_shadcn::facade as shadcn;
use std::sync::Arc;

#[path = "support/fake_services.rs"]
mod fake_services;
use fake_services::FakeServices;

fn bounds() -> Rect {
    Rect::new(
        Point::new(Px(0.0), Px(0.0)),
        CoreSize::new(Px(480.0), Px(260.0)),
    )
}

fn center_of(bounds: fret_core::Rect) -> Point {
    Point::new(
        Px(bounds.origin.x.0 + bounds.size.width.0 * 0.5),
        Px(bounds.origin.y.0 + bounds.size.height.0 * 0.5),
    )
}

#[test]
fn field_select_inherits_label_and_description_association() {
    let window = AppWindowId::default();
    let mut app = App::new();
    fret_ui_shadcn::facade::themes::apply_shadcn_new_york(
        &mut app,
        fret_ui_shadcn::facade::themes::ShadcnBaseColor::Neutral,
        fret_ui_shadcn::facade::themes::ShadcnColorScheme::Light,
    );

    let mut ui: UiTree<App> = UiTree::new();
    ui.set_window(window);
    let mut services = FakeServices;
    let value: Model<Option<Arc<str>>> = app.models_mut().insert(None);
    let open = app.models_mut().insert(false);

    let root = fret_ui::declarative::render_root(
        &mut ui,
        &mut app,
        &mut services,
        window,
        bounds(),
        "field-select-auto-association",
        |cx: &mut ElementContext<'_, App>| {
            vec![cx.column(fret_ui::element::ColumnProps::default(), |cx| {
                vec![
                    shadcn::Field::build(|cx, out| {
                        out.push_ui(
                            cx,
                            shadcn::FieldLabel::new("Department").test_id("field.select.label"),
                        );
                        out.push_ui(
                            cx,
                            shadcn::Select::new(value.clone(), open.clone())
                                .trigger_test_id("field.select.trigger")
                                .value(shadcn::SelectValue::new().placeholder("Choose department"))
                                .items([
                                    shadcn::SelectItem::new("engineering", "Engineering"),
                                    shadcn::SelectItem::new("design", "Design"),
                                    shadcn::SelectItem::new("marketing", "Marketing"),
                                ]),
                        );
                        out.push_ui(
                            cx,
                            shadcn::FieldDescription::new(
                                "Select your department or area of work.",
                            ),
                        );
                    })
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
        .find(|n| n.test_id.as_deref() == Some("field.select.label"))
        .expect("label node");
    let trigger_node = snap
        .nodes
        .iter()
        .find(|n| n.test_id.as_deref() == Some("field.select.trigger"))
        .expect("select trigger node");

    assert!(
        trigger_node.labelled_by.contains(&label_node.id),
        "expected Select trigger to inherit FieldLabel association"
    );
    assert!(
        snap.nodes.iter().any(|node| {
            trigger_node.described_by.contains(&node.id)
                && node.label.as_deref() == Some("Select your department or area of work.")
        }),
        "expected Select trigger to inherit FieldDescription association"
    );

    let label_center = center_of(ui.debug_node_bounds(label_node.id).expect("label bounds"));
    ui.dispatch_event(
        &mut app,
        &mut services,
        &fret_core::Event::Pointer(fret_core::PointerEvent::Down {
            pointer_id: fret_core::PointerId(11),
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
            pointer_id: fret_core::PointerId(11),
            position: label_center,
            button: MouseButton::Left,
            modifiers: Modifiers::default(),
            is_click: true,
            click_count: 1,
            pointer_type: fret_core::PointerType::Mouse,
        }),
    );

    assert_eq!(app.models().get_copied(&open), Some(true));
}
