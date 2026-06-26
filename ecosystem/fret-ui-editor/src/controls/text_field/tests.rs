use fret_app::App;
use fret_core::{AppWindowId, Point, Px, Rect, Size};
use fret_ui::element::{AnyElement, ElementKind, Length};
use fret_ui::elements::with_element_cx;

use super::TextField;

fn test_bounds() -> Rect {
    Rect::new(Point::new(Px(0.0), Px(0.0)), Size::new(Px(260.0), Px(64.0)))
}

fn joined_field_content(root: &AnyElement) -> &AnyElement {
    let pointer = &root.children[0];
    let frame = &pointer.children[0];
    &frame.children[0]
}

#[test]
fn text_field_without_clear_button_keeps_input_directly_inside_frame() {
    let mut app = App::new();
    let window = AppWindowId::default();
    let model = app.models_mut().insert(String::from("roughness"));

    let element = with_element_cx(
        &mut app,
        window,
        test_bounds(),
        "text-field-default-direct-input",
        |cx| TextField::new(model).into_element(cx),
    );

    assert!(matches!(element.kind, ElementKind::HoverRegion(_)));
    let content = joined_field_content(&element);
    assert!(
        matches!(content.kind, ElementKind::TextInput(_)),
        "default text field should not add a row shell when no clear segment is present, got {:?}",
        content.kind
    );
    if let ElementKind::TextInput(props) = &content.kind {
        assert!(matches!(props.layout.size.height, Length::Px(_)));
        assert!(matches!(props.layout.size.min_height, Some(Length::Px(_))));
    }
}
