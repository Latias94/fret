use std::sync::Arc;

use super::{PropertyGroup, PropertyGroupOptions};
use fret_app::App;
use fret_core::{AppWindowId, Axis, Edges, Point, Px, Rect, Size};
use fret_ui::Theme;
use fret_ui::element::{
    AnyElement, CrossAlign, ElementKind, FlexItemStyle, FlexProps, LayoutStyle, Length, MainAlign,
    MarginEdge, MarginEdges, SizeStyle, SpacingEdges, SpacingLength,
};

use crate::primitives::inspector_layout::InspectorLayoutMetrics;

fn bounds() -> Rect {
    Rect::new(
        Point::new(Px(0.0), Px(0.0)),
        Size::new(Px(160.0), Px(120.0)),
    )
}

fn fill_auto_layout() -> LayoutStyle {
    LayoutStyle {
        size: SizeStyle {
            width: Length::Fill,
            height: Length::Auto,
            ..Default::default()
        },
        ..Default::default()
    }
}

fn zero_padding() -> SpacingEdges {
    Edges::all(Px(0.0)).into()
}

fn element_test_id(element: &AnyElement) -> Option<&str> {
    element
        .semantics_decoration
        .as_ref()
        .and_then(|decoration| decoration.test_id.as_deref())
}

fn expected_content_padding(app: &App) -> SpacingEdges {
    let metrics = InspectorLayoutMetrics::resolve(Theme::global(app));
    let density = metrics.density;
    Edges {
        top: Px(density.padding_y.0 + 2.0),
        right: density.padding_x,
        bottom: Px(density.padding_y.0 + 4.0),
        left: density.padding_x,
    }
    .into()
}

#[test]
fn property_group_outer_layout_stays_on_the_chrome_root_only() {
    let mut app = App::new();
    let window = AppWindowId::default();
    let requested_layout = LayoutStyle {
        size: SizeStyle {
            width: Length::Px(Px(88.0)),
            height: Length::Px(Px(44.0)),
            min_width: Some(Length::Px(Px(11.0))),
            min_height: Some(Length::Px(Px(12.0))),
            max_width: Some(Length::Px(Px(144.0))),
            max_height: Some(Length::Px(Px(155.0))),
        },
        flex: FlexItemStyle {
            order: 7,
            grow: 2.0,
            shrink: 0.5,
            basis: Length::Fill,
            align_self: None,
        },
        margin: MarginEdges {
            top: MarginEdge::Px(Px(3.0)),
            right: MarginEdge::Px(Px(4.0)),
            bottom: MarginEdge::Px(Px(5.0)),
            left: MarginEdge::Px(Px(6.0)),
        },
        ..Default::default()
    };

    let group = fret_ui::elements::with_element_cx(
        &mut app,
        window,
        bounds(),
        "property-group-layout",
        |cx| {
            PropertyGroup::new("Editor controls")
                .options(PropertyGroupOptions {
                    collapsible: false,
                    layout: requested_layout,
                    test_id: Some(Arc::from("group.root")),
                    ..Default::default()
                })
                .into_element(cx, |_cx| None, |cx| vec![cx.text("Body")])
        },
    );

    let ElementKind::Container(root) = &group.kind else {
        panic!("property group should still return a container root");
    };
    assert_eq!(root.layout, requested_layout);

    let ElementKind::Flex(shell) = &group.children[0].kind else {
        panic!("property group root should keep its vertical shell");
    };
    assert_eq!(
        shell.layout.size,
        SizeStyle {
            width: Length::Fill,
            height: Length::Auto,
            ..Default::default()
        }
    );
    assert_eq!(shell.layout.flex, FlexItemStyle::default());
    assert_eq!(shell.layout.margin, MarginEdges::default());
}

#[test]
fn property_group_inlines_single_fill_auto_flex_content_and_keeps_padding() {
    let mut app = App::new();
    let window = AppWindowId::default();
    let expected_padding = expected_content_padding(&app);

    let group = fret_ui::elements::with_element_cx(
        &mut app,
        window,
        bounds(),
        "property-group-inline",
        |cx| {
            PropertyGroup::new("Editor controls")
                .options(PropertyGroupOptions {
                    collapsible: false,
                    test_id: Some(Arc::from("group.root")),
                    ..Default::default()
                })
                .into_element(
                    cx,
                    |_cx| None,
                    |cx| {
                        vec![
                            cx.flex(
                                FlexProps {
                                    layout: fill_auto_layout(),
                                    direction: Axis::Vertical,
                                    gap: SpacingLength::Px(Px(0.0)),
                                    padding: zero_padding(),
                                    justify: MainAlign::Start,
                                    align: CrossAlign::Stretch,
                                    wrap: false,
                                },
                                |cx| vec![cx.text("Body")],
                            )
                            .test_id("group.grid"),
                        ]
                    },
                )
        },
    );

    let ElementKind::Container(_) = &group.kind else {
        panic!("property group should still return a container root");
    };
    let content = &group.children[0].children[1];
    let ElementKind::Flex(props) = &content.kind else {
        panic!("single layout-equivalent property group content should reuse the flex root");
    };
    assert_eq!(element_test_id(content), Some("group.grid"));
    assert_eq!(props.padding, expected_padding);
    assert!(
        matches!(content.children[0].kind, ElementKind::Text(_)),
        "single layout-equivalent property group content should not be wrapped in another container"
    );
}

#[test]
fn property_group_keeps_single_content_shell_when_child_root_has_test_id() {
    let mut app = App::new();
    let window = AppWindowId::default();
    let expected_padding = expected_content_padding(&app);

    let group = fret_ui::elements::with_element_cx(
        &mut app,
        window,
        bounds(),
        "property-group-inline-collision",
        |cx| {
            PropertyGroup::new("Editor controls")
                .options(PropertyGroupOptions {
                    collapsible: false,
                    content_test_id: Some(Arc::from("group.content")),
                    ..Default::default()
                })
                .into_element(
                    cx,
                    |_cx| None,
                    |cx| {
                        vec![
                            cx.flex(
                                FlexProps {
                                    layout: fill_auto_layout(),
                                    direction: Axis::Vertical,
                                    gap: SpacingLength::Px(Px(0.0)),
                                    padding: zero_padding(),
                                    justify: MainAlign::Start,
                                    align: CrossAlign::Stretch,
                                    wrap: false,
                                },
                                |cx| vec![cx.text("Body")],
                            )
                            .test_id("group.grid"),
                        ]
                    },
                )
        },
    );

    let content = &group.children[0].children[1];
    let ElementKind::Container(props) = &content.kind else {
        panic!("property group should keep a wrapper when the child root already owns a test id");
    };
    assert_eq!(element_test_id(content), Some("group.content"));
    assert_eq!(props.padding, expected_padding);
    assert_eq!(content.children.len(), 1);

    let child = &content.children[0];
    let ElementKind::Flex(props) = &child.kind else {
        panic!("property group should keep the original flex child below the wrapper");
    };
    assert_eq!(element_test_id(child), Some("group.grid"));
    assert_eq!(props.padding, zero_padding());
    assert!(
        matches!(child.children[0].kind, ElementKind::Text(_)),
        "the original child content should remain intact"
    );
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
