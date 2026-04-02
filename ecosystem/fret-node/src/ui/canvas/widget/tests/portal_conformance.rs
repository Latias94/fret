use fret_core::{AppWindowId, Point, Px, Rect, Size};
use fret_ui::declarative::render_dismissible_root_with_hooks;
use fret_ui::element::{LayoutStyle, Length, PointerRegionProps, SemanticsProps, SizeStyle};
use fret_ui::UiTree;

use super::{NullServices, TestUiHostImpl};

fn mount_transparent_portal_like_root(
    ui: &mut UiTree<TestUiHostImpl>,
    host: &mut TestUiHostImpl,
    services: &mut NullServices,
    window: AppWindowId,
    bounds: Rect,
) -> fret_core::NodeId {
    render_dismissible_root_with_hooks(
        ui,
        host,
        services,
        window,
        bounds,
        "test.portal.root",
        |ecx| {
            let mut props = SemanticsProps::default();
            props.layout = LayoutStyle {
                size: SizeStyle {
                    width: Length::Fill,
                    height: Length::Fill,
                    ..Default::default()
                },
                ..Default::default()
            };
            vec![ecx.semantics(props, |_ecx| Vec::new())]
        },
    )
}

#[test]
fn portal_root_is_hit_test_transparent_by_default() {
    let mut host = TestUiHostImpl::default();
    let mut services = NullServices::default();
    let mut ui = UiTree::<TestUiHostImpl>::default();

    let window = AppWindowId::default();
    ui.set_window(window);

    let bounds = Rect::new(
        Point::new(Px(0.0), Px(0.0)),
        Size::new(Px(800.0), Px(600.0)),
    );

    let root =
        mount_transparent_portal_like_root(&mut ui, &mut host, &mut services, window, bounds);
    ui.set_root(root);
    ui.layout_all(&mut host, &mut services, bounds, 1.0);

    let pos = Point::new(Px(10.0), Px(10.0));
    assert_eq!(
        ui.debug_hit_test(pos).hit,
        None,
        "expected portal-like dismissible+semantics overlay to be input-transparent"
    );
}

#[test]
fn portal_can_opt_in_to_hit_testing_via_pointer_region() {
    let mut host = TestUiHostImpl::default();
    let mut services = NullServices::default();
    let mut ui = UiTree::<TestUiHostImpl>::default();

    let window = AppWindowId::default();
    ui.set_window(window);

    let bounds = Rect::new(
        Point::new(Px(0.0), Px(0.0)),
        Size::new(Px(800.0), Px(600.0)),
    );

    let root = render_dismissible_root_with_hooks(
        &mut ui,
        &mut host,
        &mut services,
        window,
        bounds,
        "test.portal.pointer_region",
        |ecx| {
            let mut props = PointerRegionProps::default();
            props.layout = LayoutStyle {
                size: SizeStyle {
                    width: Length::Fill,
                    height: Length::Fill,
                    ..Default::default()
                },
                ..Default::default()
            };
            vec![ecx.pointer_region(props, |_ecx| Vec::new())]
        },
    );

    ui.set_root(root);
    ui.layout_all(&mut host, &mut services, bounds, 1.0);

    let pos = Point::new(Px(10.0), Px(10.0));
    assert!(
        ui.debug_hit_test(pos).hit.is_some(),
        "expected pointer region to be hit-testable (sanity check)"
    );
}
