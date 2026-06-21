use std::sync::Arc;

use super::{PropertyGroup, PropertyGroupOptions};
use fret_app::App;
use fret_core::{AppWindowId, Point, Px, Rect, Size};
use fret_ui::element::ElementKind;

fn bounds() -> Rect {
    Rect::new(
        Point::new(Px(0.0), Px(0.0)),
        Size::new(Px(160.0), Px(120.0)),
    )
}

#[test]
fn property_group_non_collapsible_single_content_uses_direct_header_and_content_containers() {
    let mut app = App::new();
    let window = AppWindowId::default();
    let group = fret_ui::elements::with_element_cx(
        &mut app,
        window,
        bounds(),
        "property-group-single-content",
        |cx| {
            PropertyGroup::new("Editor controls")
                .options(PropertyGroupOptions {
                    collapsible: false,
                    test_id: Some(Arc::from("group.root")),
                    ..Default::default()
                })
                .into_element(cx, |_cx| None, |cx| vec![cx.text("Body")])
        },
    );

    let ElementKind::Container(_) = &group.kind else {
        panic!("property group should still return a container root");
    };
    let ElementKind::Flex(_) = &group.children[0].kind else {
        panic!("property group root should keep its vertical shell");
    };

    let header = &group.children[0].children[0];
    let ElementKind::Container(_) = &header.kind else {
        panic!("non-collapsible header should use a direct container shell");
    };
    assert!(
        matches!(header.children[0].kind, ElementKind::Text(_)),
        "non-collapsible property group header should place the label directly in the container"
    );

    let content = &group.children[0].children[1];
    assert!(
        matches!(content.kind, ElementKind::Container(_)),
        "single property group content child should use a container shell"
    );
    assert!(
        matches!(content.children[0].kind, ElementKind::Text(_)),
        "single property group content child should stay directly under the content container"
    );
}
