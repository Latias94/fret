#![cfg(feature = "diagnostics")]

//! Material 3 divider semantics and layout regression tests.

use fret_core::{AppWindowId, Axis, Point, Px, Rect, SemanticsRole, Size, UiServices};
use fret_runtime::PlatformCapabilities;
use fret_ui::UiTree;
use fret_ui::element::{ContainerProps, FlexProps, Length};
use fret_ui_material3::Divider;
use fret_ui_material3::tokens::v30::{DynamicVariant, SchemeMode};

mod support;

use support::host::{FakeUiServices, TestHost};
use support::layout::{semantics_node_id_by_test_id, with_padding};
use support::theme::apply_material_theme;

fn bounds() -> Rect {
    Rect::new(
        Point::new(Px(0.0), Px(0.0)),
        Size::new(Px(320.0), Px(220.0)),
    )
}

fn render_dividers(
    ui: &mut UiTree<TestHost>,
    app: &mut TestHost,
    services: &mut dyn UiServices,
    window: AppWindowId,
) {
    let bounds = bounds();
    let root = fret_ui::declarative::render_root(ui, app, services, window, bounds, "root", |cx| {
        let mut column = FlexProps::default();
        column.direction = Axis::Vertical;
        column.gap = Px(16.0).into();

        let content = cx.flex(column, |cx| {
            let mut horizontal_box = ContainerProps::default();
            horizontal_box.layout.size.width = Length::Px(Px(200.0));
            horizontal_box.layout.size.height = Length::Px(Px(16.0));

            let horizontal = cx.container(horizontal_box, |cx| {
                vec![
                    Divider::horizontal()
                        .thickness(Px(3.0))
                        .test_id("m3-divider-horizontal")
                        .into_element(cx),
                ]
            });

            let mut vertical_box = ContainerProps::default();
            vertical_box.layout.size.width = Length::Px(Px(32.0));
            vertical_box.layout.size.height = Length::Px(Px(80.0));

            let vertical = cx.container(vertical_box, |cx| {
                vec![
                    Divider::vertical()
                        .thickness(Px(4.0))
                        .test_id("m3-divider-vertical")
                        .into_element(cx),
                ]
            });

            vec![horizontal, vertical]
        });

        vec![with_padding(cx, Px(24.0), content)]
    });
    ui.set_root(root);
    ui.request_semantics_snapshot();
    ui.layout_all(app, services, bounds, 1.0);
}

fn semantics_node<'a>(ui: &'a UiTree<TestHost>, test_id: &str) -> &'a fret_core::SemanticsNode {
    ui.semantics_snapshot()
        .and_then(|snapshot| {
            snapshot
                .nodes
                .iter()
                .find(|node| node.test_id.as_deref() == Some(test_id))
        })
        .unwrap_or_else(|| panic!("expected semantics node for test_id {test_id}"))
}

fn layout_bounds_by_test_id(ui: &UiTree<TestHost>, test_id: &str) -> Rect {
    let node = semantics_node_id_by_test_id(ui, test_id)
        .unwrap_or_else(|| panic!("expected semantics node {test_id}"));
    ui.debug_node_bounds(node)
        .unwrap_or_else(|| panic!("expected layout bounds for {test_id}"))
}

fn assert_px_close(actual: f32, expected: f32, context: &str) {
    let delta = (actual - expected).abs();
    assert!(
        delta <= 0.5,
        "{context}: expected {expected}px, got {actual}px (delta {delta}px)"
    );
}

#[test]
fn divider_semantics_and_orientation_bounds_are_stable() {
    let mut app = TestHost::default();
    app.set_global(PlatformCapabilities::default());
    apply_material_theme(&mut app, SchemeMode::Light, DynamicVariant::TonalSpot);

    let window = AppWindowId::default();
    let mut services = FakeUiServices;
    let mut ui: UiTree<TestHost> = UiTree::new();
    ui.set_window(window);

    render_dividers(&mut ui, &mut app, &mut services, window);

    let horizontal = semantics_node(&ui, "m3-divider-horizontal");
    assert_eq!(horizontal.role, SemanticsRole::Generic);
    assert!(!horizontal.actions.invoke);

    let vertical = semantics_node(&ui, "m3-divider-vertical");
    assert_eq!(vertical.role, SemanticsRole::Generic);
    assert!(!vertical.actions.invoke);

    let horizontal = layout_bounds_by_test_id(&ui, "m3-divider-horizontal");
    assert_px_close(horizontal.size.width.0, 200.0, "horizontal divider width");
    assert_px_close(
        horizontal.size.height.0,
        3.0,
        "horizontal divider thickness",
    );

    let vertical = layout_bounds_by_test_id(&ui, "m3-divider-vertical");
    assert_px_close(vertical.size.width.0, 4.0, "vertical divider thickness");
    assert_px_close(vertical.size.height.0, 80.0, "vertical divider height");
}
