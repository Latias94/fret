use fret_app::App;
use fret_core::{AppWindowId, Modifiers, MouseButton, Point, Px, Rect, Size as CoreSize};
use fret_runtime::Model;
use fret_ui::ElementContext;
use fret_ui::element::AnyElement;
use fret_ui::tree::UiTree;
use fret_ui_kit::ui::UiElementSinkExt as _;
use fret_ui_shadcn::facade as shadcn;

#[path = "support/fake_services.rs"]
mod fake_services;
use fake_services::FakeServices;

fn bounds() -> Rect {
    Rect::new(
        Point::new(Px(0.0), Px(0.0)),
        CoreSize::new(Px(480.0), Px(280.0)),
    )
}

fn center_of(bounds: fret_core::Rect) -> Point {
    Point::new(
        Px(bounds.origin.x.0 + bounds.size.width.0 * 0.5),
        Px(bounds.origin.y.0 + bounds.size.height.0 * 0.5),
    )
}

fn render_root(
    root_id: &'static str,
    build: impl FnOnce(&mut ElementContext<'_, App>) -> Vec<AnyElement>,
) -> (App, UiTree<App>, FakeServices) {
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

    let root = fret_ui::declarative::render_root(
        &mut ui,
        &mut app,
        &mut services,
        window,
        bounds(),
        root_id,
        build,
    );
    ui.set_root(root);
    ui.request_semantics_snapshot();
    ui.layout_all(&mut app, &mut services, bounds(), 1.0);

    (app, ui, services)
}

#[test]
fn field_input_inherits_label_and_description_association() {
    let (mut app, mut ui, mut services) = render_root("field-input-auto-association", |cx| {
        let model: Model<String> = cx.app.models_mut().insert(String::new());
        vec![cx.column(fret_ui::element::ColumnProps::default(), |cx| {
            vec![
                shadcn::Field::build(|cx, out| {
                    out.push_ui(
                        cx,
                        shadcn::FieldLabel::new("Email").test_id("field.input.label"),
                    );
                    out.push_ui(
                        cx,
                        shadcn::Input::new(model.clone())
                            .placeholder("name@example.com")
                            .test_id("field.input.control"),
                    );
                    out.push_ui(
                        cx,
                        shadcn::FieldDescription::new("We will only use this to contact you."),
                    );
                })
                .into_element(cx),
            ]
        })]
    });

    let snap = ui.semantics_snapshot_arc().expect("semantics snapshot");
    let label_node = snap
        .nodes
        .iter()
        .find(|node| node.test_id.as_deref() == Some("field.input.label"))
        .expect("field input label");
    let control_node = snap
        .nodes
        .iter()
        .find(|node| node.test_id.as_deref() == Some("field.input.control"))
        .expect("field input control");

    assert!(
        control_node
            .labelled_by
            .iter()
            .any(|id| *id == label_node.id),
        "expected Input to inherit FieldLabel association"
    );
    assert!(
        snap.nodes.iter().any(|node| {
            control_node.described_by.iter().any(|id| *id == node.id)
                && node.label.as_deref() == Some("We will only use this to contact you.")
        }),
        "expected Input to inherit FieldDescription association"
    );

    let label_center = center_of(ui.debug_node_bounds(label_node.id).expect("label bounds"));
    ui.dispatch_event(
        &mut app,
        &mut services,
        &fret_core::Event::Pointer(fret_core::PointerEvent::Down {
            pointer_id: fret_core::PointerId(21),
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
            pointer_id: fret_core::PointerId(21),
            position: label_center,
            button: MouseButton::Left,
            modifiers: Modifiers::default(),
            is_click: true,
            click_count: 1,
            pointer_type: fret_core::PointerType::Mouse,
        }),
    );
    assert_eq!(ui.focus(), Some(control_node.id));
}

#[test]
fn field_textarea_inherits_label_and_description_association() {
    let (mut app, mut ui, mut services) = render_root("field-textarea-auto-association", |cx| {
        let model: Model<String> = cx.app.models_mut().insert(String::new());
        vec![cx.column(fret_ui::element::ColumnProps::default(), |cx| {
            vec![
                shadcn::Field::build(|cx, out| {
                    out.push_ui(
                        cx,
                        shadcn::FieldLabel::new("Message").test_id("field.textarea.label"),
                    );
                    out.push_ui(
                        cx,
                        shadcn::Textarea::new(model.clone())
                            .placeholder("Tell us what happened")
                            .test_id("field.textarea.control"),
                    );
                    out.push_ui(
                        cx,
                        shadcn::FieldDescription::new("Include the exact steps to reproduce."),
                    );
                })
                .into_element(cx),
            ]
        })]
    });

    let snap = ui.semantics_snapshot_arc().expect("semantics snapshot");
    let label_node = snap
        .nodes
        .iter()
        .find(|node| node.test_id.as_deref() == Some("field.textarea.label"))
        .expect("field textarea label");
    let control_node = snap
        .nodes
        .iter()
        .find(|node| node.test_id.as_deref() == Some("field.textarea.control"))
        .expect("field textarea control");

    assert!(
        control_node
            .labelled_by
            .iter()
            .any(|id| *id == label_node.id),
        "expected Textarea to inherit FieldLabel association"
    );
    assert!(
        snap.nodes.iter().any(|node| {
            control_node.described_by.iter().any(|id| *id == node.id)
                && node.label.as_deref() == Some("Include the exact steps to reproduce.")
        }),
        "expected Textarea to inherit FieldDescription association"
    );

    let label_center = center_of(ui.debug_node_bounds(label_node.id).expect("label bounds"));
    ui.dispatch_event(
        &mut app,
        &mut services,
        &fret_core::Event::Pointer(fret_core::PointerEvent::Down {
            pointer_id: fret_core::PointerId(22),
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
            pointer_id: fret_core::PointerId(22),
            position: label_center,
            button: MouseButton::Left,
            modifiers: Modifiers::default(),
            is_click: true,
            click_count: 1,
            pointer_type: fret_core::PointerType::Mouse,
        }),
    );
    assert_eq!(ui.focus(), Some(control_node.id));
}
