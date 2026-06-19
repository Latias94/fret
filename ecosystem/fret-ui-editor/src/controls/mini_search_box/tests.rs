use fret_app::App;
use fret_core::{AppWindowId, Point, Px, Rect, Size};
use fret_ui::element::{AnyElement, ElementKind};
use fret_ui::elements::with_element_cx;

use super::MiniSearchBox;

fn test_bounds() -> Rect {
    Rect::new(Point::new(Px(0.0), Px(0.0)), Size::new(Px(260.0), Px(64.0)))
}

fn joined_field_content(root: &AnyElement) -> &AnyElement {
    let pointer = &root.children[0];
    let frame = &pointer.children[0];
    &frame.children[0]
}

#[test]
fn mini_search_box_without_value_keeps_input_directly_inside_frame() {
    let mut app = App::new();
    let window = AppWindowId::default();
    let model = app.models_mut().insert(String::new());

    let element = with_element_cx(
        &mut app,
        window,
        test_bounds(),
        "mini-search-default-direct-input",
        |cx| MiniSearchBox::new(model).into_element(cx),
    );

    assert!(matches!(element.kind, ElementKind::HoverRegion(_)));
    let content = joined_field_content(&element);
    assert!(
        matches!(content.kind, ElementKind::TextInput(_)),
        "empty mini search box should not add a row shell when no clear segment is present, got {:?}",
        content.kind
    );
}
