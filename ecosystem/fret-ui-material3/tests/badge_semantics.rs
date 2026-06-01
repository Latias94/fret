//! Material 3 badge slot and semantics regression tests.

use fret_core::{AppWindowId, Point, Px, Rect, SemanticsRole, Size, UiServices};
use fret_runtime::PlatformCapabilities;
use fret_ui::UiTree;
use fret_ui::element::{AnyElement, ContainerProps, Length};
use fret_ui_material3::tokens::v30::{DynamicVariant, SchemeMode};
use fret_ui_material3::{Badge, BadgePlacement};

mod support;

use support::host::{FakeUiServices, TestHost};
use support::layout::{semantics_node_id_by_test_id, with_padding};
use support::theme::{apply_material_theme, apply_material_theme_rtl};

fn bounds() -> Rect {
    Rect::new(
        Point::new(Px(0.0), Px(0.0)),
        Size::new(Px(240.0), Px(180.0)),
    )
}

fn render_badge(
    ui: &mut UiTree<TestHost>,
    app: &mut TestHost,
    services: &mut dyn UiServices,
    window: AppWindowId,
) {
    let badge = Badge::text("99+")
        .placement(BadgePlacement::TopRight)
        .anchor_size(Px(40.0))
        .a11y_label("99 or more new notifications")
        .test_id("m3-badge");
    render_badge_with(ui, app, services, window, badge);
}

fn render_badge_with(
    ui: &mut UiTree<TestHost>,
    app: &mut TestHost,
    services: &mut dyn UiServices,
    window: AppWindowId,
    badge: Badge,
) {
    let bounds = bounds();
    let root = fret_ui::declarative::render_root(ui, app, services, window, bounds, "root", |cx| {
        let anchor = |cx: &mut fret_ui::elements::ElementContext<'_, TestHost>| {
            let mut props = ContainerProps::default();
            props.layout.size.width = Length::Px(Px(40.0));
            props.layout.size.height = Length::Px(Px(40.0));
            cx.container(props, |_cx| Vec::<AnyElement>::new())
        };

        let badge = badge.into_element(cx, |cx| vec![anchor(cx)]);

        vec![with_padding(cx, Px(24.0), badge)]
    });
    ui.set_root(root);
    ui.request_semantics_snapshot();
    ui.layout_all(app, services, bounds, 1.0);
}

fn node<'a>(ui: &'a UiTree<TestHost>, test_id: &str) -> &'a fret_core::SemanticsNode {
    ui.semantics_snapshot()
        .expect("expected semantics snapshot")
        .nodes
        .iter()
        .find(|node| node.test_id.as_deref() == Some(test_id))
        .unwrap_or_else(|| panic!("expected semantics node {test_id}"))
}

fn assert_px_close(actual: f32, expected: f32, context: &str) {
    let delta = (actual - expected).abs();
    assert!(
        delta <= 0.5,
        "{context}: expected {expected}px, got {actual}px (delta {delta}px)"
    );
}

fn layout_bounds_by_test_id(ui: &UiTree<TestHost>, test_id: &str) -> Rect {
    let node = semantics_node_id_by_test_id(ui, test_id)
        .unwrap_or_else(|| panic!("expected semantics node {test_id}"));
    ui.debug_node_bounds(node)
        .unwrap_or_else(|| panic!("expected layout bounds for {test_id}"))
}

#[test]
fn badge_exposes_badged_box_anchor_and_badge_part_semantics() {
    let mut app = TestHost::default();
    app.set_global(PlatformCapabilities::default());
    apply_material_theme(&mut app, SchemeMode::Light, DynamicVariant::TonalSpot);

    let window = AppWindowId::default();
    let mut services = FakeUiServices;
    let mut ui: UiTree<TestHost> = UiTree::new();
    ui.set_window(window);

    render_badge(&mut ui, &mut app, &mut services, window);

    let root = node(&ui, "m3-badge");
    assert_eq!(root.role, SemanticsRole::Group);
    assert_eq!(root.label, None);

    let anchor = node(&ui, "m3-badge.anchor");
    assert_eq!(anchor.role, SemanticsRole::Generic);
    assert_eq!(anchor.label, None);

    let badge = node(&ui, "m3-badge.badge");
    assert_eq!(badge.role, SemanticsRole::Generic);
    assert_eq!(badge.label.as_deref(), Some("99 or more new notifications"));
    assert_eq!(badge.value.as_deref(), Some("99+"));

    let root_id =
        semantics_node_id_by_test_id(&ui, "m3-badge").expect("expected root test id node");
    let anchor_id =
        semantics_node_id_by_test_id(&ui, "m3-badge.anchor").expect("expected anchor part node");
    let badge_id =
        semantics_node_id_by_test_id(&ui, "m3-badge.badge").expect("expected badge part node");

    let root_bounds = ui.debug_node_bounds(root_id).expect("root layout bounds");
    let anchor_bounds = ui
        .debug_node_bounds(anchor_id)
        .expect("anchor layout bounds");
    let badge_bounds = ui.debug_node_bounds(badge_id).expect("badge layout bounds");

    assert_px_close(root_bounds.size.width.0, 40.0, "root width follows anchor");
    assert_px_close(
        root_bounds.size.height.0,
        40.0,
        "root height follows anchor",
    );
    assert_px_close(anchor_bounds.size.width.0, 40.0, "anchor width");
    assert_px_close(anchor_bounds.size.height.0, 40.0, "anchor height");
    assert_px_close(badge_bounds.size.height.0, 16.0, "large badge height");
    assert!(
        badge_bounds.size.width.0 > 16.0,
        "large badge with text should expand beyond the minimum size"
    );
    assert!(
        badge_bounds.origin.x.0 >= anchor_bounds.origin.x.0 + anchor_bounds.size.width.0 - 16.5,
        "large badge should be anchored to the top-end corner"
    );
    assert_px_close(
        badge_bounds.origin.y.0,
        anchor_bounds.origin.y.0,
        "badge top",
    );
}

#[test]
fn navigation_icon_badge_uses_logical_inline_edge_in_ltr_and_rtl() {
    fn render_direction(rtl: bool) -> (Rect, Rect) {
        let mut app = TestHost::default();
        app.set_global(PlatformCapabilities::default());
        if rtl {
            apply_material_theme_rtl(&mut app, SchemeMode::Light, DynamicVariant::TonalSpot);
        } else {
            apply_material_theme(&mut app, SchemeMode::Light, DynamicVariant::TonalSpot);
        }

        let window = AppWindowId::default();
        let mut services = FakeUiServices;
        let mut ui: UiTree<TestHost> = UiTree::new();
        ui.set_window(window);

        render_badge_with(
            &mut ui,
            &mut app,
            &mut services,
            window,
            Badge::dot()
                .placement(BadgePlacement::NavigationIcon)
                .anchor_size(Px(40.0))
                .test_id("m3-nav-badge"),
        );

        (
            layout_bounds_by_test_id(&ui, "m3-nav-badge.anchor"),
            layout_bounds_by_test_id(&ui, "m3-nav-badge.badge"),
        )
    }

    let (ltr_anchor, ltr_badge) = render_direction(false);
    let (rtl_anchor, rtl_badge) = render_direction(true);

    assert!(
        ltr_badge.origin.x.0 > ltr_anchor.origin.x.0 + ltr_anchor.size.width.0 * 0.5,
        "expected LTR navigation badge to sit on the physical right half; anchor={ltr_anchor:?}, badge={ltr_badge:?}"
    );
    assert!(
        rtl_badge.origin.x.0 < rtl_anchor.origin.x.0 + rtl_anchor.size.width.0 * 0.5,
        "expected RTL navigation badge to sit on the physical left half; anchor={rtl_anchor:?}, badge={rtl_badge:?}"
    );
    assert_px_close(
        ltr_badge.origin.y.0,
        ltr_anchor.origin.y.0 + 4.0,
        "LTR navigation badge top offset",
    );
    assert_px_close(
        rtl_badge.origin.y.0,
        rtl_anchor.origin.y.0 + 4.0,
        "RTL navigation badge top offset",
    );
}
