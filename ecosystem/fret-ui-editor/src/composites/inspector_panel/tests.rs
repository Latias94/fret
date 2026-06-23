use std::sync::{Arc, Mutex};

use super::{InspectorPanel, InspectorPanelOptions};
use crate::test_support::WrappingTextServices;
use fret_app::App;
use fret_core::{AppWindowId, Point, Px, Rect, Size};
use fret_ui::element::{AnyElement, ContainerProps, ElementKind, LayoutStyle, Length, SizeStyle};
use fret_ui::elements::GlobalElementId;
use fret_ui::{UiTree, declarative};

fn bounds() -> Rect {
    Rect::new(
        Point::new(Px(0.0), Px(0.0)),
        Size::new(Px(140.0), Px(120.0)),
    )
}

fn find_text<'a>(element: &'a AnyElement, expected: &str) -> Option<&'a AnyElement> {
    if matches!(&element.kind, ElementKind::Text(props) if props.text.as_ref() == expected) {
        return Some(element);
    }
    element
        .children
        .iter()
        .find_map(|child| find_text(child, expected))
}

fn lock_id(id: &Arc<Mutex<Option<GlobalElementId>>>, label: &str) -> GlobalElementId {
    id.lock().unwrap().unwrap_or_else(|| panic!("{label} id"))
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

fn element_test_id(element: &AnyElement) -> Option<&str> {
    element
        .semantics_decoration
        .as_ref()
        .and_then(|decoration| decoration.test_id.as_deref())
}

#[test]
fn inspector_panel_title_stays_single_line_when_header_is_narrow() {
    let mut app = App::new();
    let mut ui: UiTree<App> = UiTree::new();
    let window = AppWindowId::default();
    ui.set_window(window);

    let mut services = WrappingTextServices;
    let panel_id = Arc::new(Mutex::new(None::<GlobalElementId>));
    let title_id = Arc::new(Mutex::new(None::<GlobalElementId>));

    let panel_id_for_render = Arc::clone(&panel_id);
    let title_id_for_render = Arc::clone(&title_id);
    let title: Arc<str> = Arc::from("Very Long Material Inspector Title That Must Not Wrap");
    let root = declarative::render_root(
        &mut ui,
        &mut app,
        &mut services,
        window,
        bounds(),
        "inspector-panel-title-layout",
        move |cx| {
            let panel = InspectorPanel::new(None)
                .options(InspectorPanelOptions {
                    layout: LayoutStyle {
                        size: SizeStyle {
                            width: Length::Fill,
                            height: Length::Auto,
                            ..Default::default()
                        },
                        ..Default::default()
                    },
                    title: Some(title.clone()),
                    test_id: Some(Arc::from("inspector.panel")),
                    ..Default::default()
                })
                .into_element(
                    cx,
                    |cx, _panel| vec![cx.text("Edit"), cx.text("Reset")],
                    |cx, _panel| vec![cx.text("Body")],
                );

            let title_text = find_text(&panel, title.as_ref())
                .expect("inspector panel should render its title as a text element");
            let ElementKind::Text(props) = &title_text.kind else {
                panic!("inspector panel title should be a text element");
            };
            assert_eq!(props.wrap, fret_core::TextWrap::None);
            assert_eq!(props.overflow, fret_core::TextOverflow::Ellipsis);
            assert_eq!(props.layout.size.min_width, Some(Length::Px(Px(0.0))));
            assert_eq!(props.layout.flex.grow, 1.0);
            assert_eq!(props.layout.flex.shrink, 1.0);

            let ElementKind::Flex(_) = &panel.children[0].kind else {
                panic!("inspector panel root should keep its vertical shell");
            };
            let ElementKind::Container(_) = &panel.children[0].children[0].kind else {
                panic!("inspector panel header should stay wrapped in a container");
            };
            assert!(
                matches!(
                    panel.children[0].children[0].children[0].kind,
                    ElementKind::Flex(_)
                ),
                "inspector panel title+toolbar header should keep the row flex"
            );

            *panel_id_for_render.lock().unwrap() = Some(panel.id);
            *title_id_for_render.lock().unwrap() = Some(title_text.id);
            vec![panel]
        },
    );
    ui.set_root(root);
    ui.layout_all(&mut app, &mut services, bounds(), 1.0);

    let panel_bounds = fret_ui::elements::current_bounds_for_element(
        &mut app,
        window,
        lock_id(&panel_id, "panel"),
    )
    .expect("panel bounds");
    let title_bounds = fret_ui::elements::current_bounds_for_element(
        &mut app,
        window,
        lock_id(&title_id, "title"),
    )
    .expect("title bounds");

    assert!(
        title_bounds.size.height.0 <= 14.1,
        "inspector panel title should stay one measured line under resize, got {:?}",
        title_bounds
    );
    assert!(
        title_bounds.origin.y.0 + title_bounds.size.height.0
            <= panel_bounds.origin.y.0 + panel_bounds.size.height.0 + 0.01,
        "single-line title should stay inside the inspector panel header/body layout"
    );
}

#[test]
fn inspector_panel_title_only_uses_direct_header_and_single_content_shells() {
    let mut app = App::new();
    let mut ui: UiTree<App> = UiTree::new();
    let window = AppWindowId::default();
    ui.set_window(window);

    let mut services = WrappingTextServices;
    let title: Arc<str> = Arc::from("Inspector");
    let root = declarative::render_root(
        &mut ui,
        &mut app,
        &mut services,
        window,
        bounds(),
        "inspector-panel-title-only-layout",
        move |cx| {
            let panel = InspectorPanel::new(None)
                .options(InspectorPanelOptions {
                    layout: LayoutStyle {
                        size: SizeStyle {
                            width: Length::Fill,
                            height: Length::Auto,
                            ..Default::default()
                        },
                        ..Default::default()
                    },
                    title: Some(title.clone()),
                    test_id: Some(Arc::from("inspector.panel.title-only")),
                    content_test_id: Some(Arc::from("inspector.panel.content")),
                    ..Default::default()
                })
                .into_element(
                    cx,
                    |_cx, _panel| Vec::new(),
                    |cx, _panel| vec![cx.text("Body")],
                );

            let ElementKind::Container(_) = &panel.kind else {
                panic!("inspector panel should still return a container root");
            };
            let ElementKind::Flex(_) = &panel.children[0].kind else {
                panic!("inspector panel root should keep its vertical shell");
            };

            let header = &panel.children[0].children[0];
            let ElementKind::Container(_) = &header.kind else {
                panic!("title-only header should use a container shell");
            };
            assert!(
                matches!(header.children[0].kind, ElementKind::Text(_)),
                "title-only inspector header should place the title directly in the header container"
            );

            let content = &panel.children[0].children[1];
            assert!(
                matches!(content.kind, ElementKind::Container(_)),
                "single inspector content child should use a container shell"
            );
            assert!(
                matches!(content.children[0].kind, ElementKind::Text(_)),
                "single inspector content child should stay directly under the content container"
            );

            vec![panel]
        },
    );
    ui.set_root(root);
    ui.layout_all(&mut app, &mut services, bounds(), 1.0);
}

#[test]
fn inspector_panel_inlines_single_fill_auto_content_root_for_content_test_id() {
    let mut app = App::new();
    let window = AppWindowId::default();
    let title: Arc<str> = Arc::from("Inspector");

    let panel = fret_ui::elements::with_element_cx(
        &mut app,
        window,
        bounds(),
        "inspector-panel-inline-content-layout",
        |cx| {
            InspectorPanel::new(None)
                .options(InspectorPanelOptions {
                    title: Some(title.clone()),
                    content_test_id: Some(Arc::from("inspector.panel.content")),
                    ..Default::default()
                })
                .into_element(
                    cx,
                    |_cx, _panel| Vec::new(),
                    |cx, _panel| {
                        vec![cx.container(
                            ContainerProps {
                                layout: fill_auto_layout(),
                                ..Default::default()
                            },
                            |cx| vec![cx.text("Body")],
                        )]
                    },
                )
        },
    );

    let ElementKind::Container(_) = &panel.kind else {
        panic!("inspector panel should still return a container root");
    };
    let content = &panel.children[0].children[1];
    assert_eq!(element_test_id(content), Some("inspector.panel.content"));
    assert!(
        matches!(content.kind, ElementKind::Container(_)),
        "single layout-equivalent content root should be reused directly"
    );
    assert!(
        matches!(content.children[0].kind, ElementKind::Text(_)),
        "single layout-equivalent content root should not be wrapped in another container"
    );
}

#[test]
fn inspector_panel_keeps_single_content_shell_when_child_root_has_test_id() {
    let mut app = App::new();
    let window = AppWindowId::default();
    let title: Arc<str> = Arc::from("Inspector");

    let panel = fret_ui::elements::with_element_cx(
        &mut app,
        window,
        bounds(),
        "inspector-panel-content-test-id-collision",
        |cx| {
            InspectorPanel::new(None)
                .options(InspectorPanelOptions {
                    title: Some(title.clone()),
                    content_test_id: Some(Arc::from("inspector.panel.content")),
                    ..Default::default()
                })
                .into_element(
                    cx,
                    |_cx, _panel| Vec::new(),
                    |cx, _panel| {
                        vec![
                            cx.container(
                                ContainerProps {
                                    layout: fill_auto_layout(),
                                    ..Default::default()
                                },
                                |cx| vec![cx.text("Body")],
                            )
                            .test_id("inspector.panel.child"),
                        ]
                    },
                )
        },
    );

    let content = &panel.children[0].children[1];
    assert_eq!(element_test_id(content), Some("inspector.panel.content"));
    assert!(
        matches!(content.kind, ElementKind::Container(_)),
        "inspector content should keep a wrapper when the child root already owns a test id"
    );

    let child = &content.children[0];
    assert_eq!(element_test_id(child), Some("inspector.panel.child"));
    assert!(
        matches!(child.kind, ElementKind::Container(_)),
        "the original child root should stay below the content wrapper"
    );
    assert!(
        matches!(child.children[0].kind, ElementKind::Text(_)),
        "the original child content should remain intact"
    );
}
