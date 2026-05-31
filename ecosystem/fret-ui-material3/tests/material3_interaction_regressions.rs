use std::sync::Arc;

use fret_core::{AppWindowId, Event, NodeId, Point, Px, Rect, Size, UiServices};
use fret_runtime::{ModelHost, PlatformCapabilities};
use fret_ui::UiTree;
use fret_ui_material3::tokens::v30::{DynamicVariant, SchemeMode};

mod interaction_harness;
mod support;

use support::host::{FakeUiServices, TestHost};
use support::layout::with_padding;
use support::theme::apply_material_theme;

// Residual plain TextInput regression kept here until the mechanism-layer ownership audit.

#[test]
fn text_input_text_input_event_updates_model() {
    use fret_ui::element::TextInputProps;

    let mut app = TestHost::default();
    app.set_global(PlatformCapabilities::default());
    apply_material_theme(&mut app, SchemeMode::Light, DynamicVariant::TonalSpot);

    let window = AppWindowId::default();
    let mut services = FakeUiServices;
    let mut ui: UiTree<TestHost> = UiTree::new();
    ui.set_window(window);

    let bounds = Rect::new(
        Point::new(Px(0.0), Px(0.0)),
        Size::new(Px(520.0), Px(420.0)),
    );

    let model = app.models_mut().insert(String::new());
    let model_for_render = model.clone();

    let render =
        move |ui: &mut UiTree<TestHost>, app: &mut TestHost, services: &mut dyn UiServices| {
            fret_ui::declarative::render_root(ui, app, services, window, bounds, "root", |cx| {
                let mut props = TextInputProps::new(model_for_render.clone());
                props.layout.size.width = fret_ui::element::Length::Px(Px(200.0));
                props.layout.size.height = fret_ui::element::Length::Px(Px(40.0));
                props.a11y_label = Some(Arc::<str>::from("input"));
                props.test_id = Some(Arc::<str>::from("plain-text-input"));
                let input = cx.text_input(props);
                vec![with_padding(cx, Px(24.0), input)]
            })
        };

    let root = render(&mut ui, &mut app, &mut services);
    ui.set_root(root);
    ui.request_semantics_snapshot();
    ui.layout_all(&mut app, &mut services, bounds, 1.0);

    let input_node: NodeId = ui
        .semantics_snapshot()
        .and_then(|snapshot| {
            snapshot.nodes.iter().find_map(|node| {
                (node.test_id.as_deref() == Some("plain-text-input")).then_some(node.id)
            })
        })
        .expect("expected plain-text-input in semantics snapshot");

    ui.set_focus(Some(input_node));
    assert_eq!(
        ui.focus(),
        Some(input_node),
        "expected focus to be set to the input node",
    );

    ui.dispatch_event(&mut app, &mut services, &Event::TextInput("a".to_string()));

    let root = render(&mut ui, &mut app, &mut services);
    ui.set_root(root);
    ui.layout_all(&mut app, &mut services, bounds, 1.0);

    let value = app.models().get_cloned(&model).expect("model exists");
    assert_eq!(value, "a", "expected text input event to update the model");
}
