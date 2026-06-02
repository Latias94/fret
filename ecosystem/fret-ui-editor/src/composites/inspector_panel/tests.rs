use std::sync::{Arc, Mutex};

use super::{InspectorPanel, InspectorPanelOptions};
use crate::test_support::WrappingTextServices;
use fret_app::App;
use fret_core::{AppWindowId, Point, Px, Rect, Size};
use fret_ui::element::{AnyElement, ElementKind, LayoutStyle, Length, SizeStyle};
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
