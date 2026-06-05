use super::fixtures::{render_test_sortable_rows, test_sortable_items};
use super::*;

#[test]
fn sortable_rows_reorder_using_drop_positions() {
    let window = AppWindowId::default();
    let bounds = Rect::new(
        Point::new(Px(0.0), Px(0.0)),
        Size::new(Px(360.0), Px(220.0)),
    );

    let mut ui = UiTree::new();
    ui.set_window(window);

    let mut app = TestHost::new();
    app.set_global(PlatformCapabilities::default());
    let mut services = FakeTextService::default();

    let items = Rc::new(RefCell::new(test_sortable_items()));
    let preview_status = Rc::new(RefCell::new(String::new()));
    let delivered_status = Rc::new(RefCell::new(String::new()));
    let order_status = Rc::new(RefCell::new(String::new()));
    let delivered_flag = Rc::new(Cell::new(false));

    let _root = run_frame(
        &mut ui,
        &mut app,
        &mut services,
        window,
        bounds,
        "imui-sortable-rows",
        render_test_sortable_rows(
            &items,
            &preview_status,
            &delivered_status,
            &order_status,
            &delivered_flag,
        ),
    );

    assert_eq!(
        order_status.borrow().as_str(),
        "Camera -> Cube -> Key light"
    );
    assert!(preview_status.borrow().is_empty());
    assert!(delivered_status.borrow().is_empty());
    assert!(!delivered_flag.get());

    let source_point = point_for_test_id(
        &mut ui,
        &mut app,
        &mut services,
        bounds,
        "imui-sortable-row.camera",
    );
    let _target_point = point_for_test_id(
        &mut ui,
        &mut app,
        &mut services,
        bounds,
        "imui-sortable-row.cube",
    );
    let target_bounds = bounds_for_test_id(&ui, "imui-sortable-row.cube");
    let target_lower = Point::new(
        Px(target_bounds.origin.x.0 + target_bounds.size.width.0 * 0.5),
        Px(target_bounds.origin.y.0 + target_bounds.size.height.0 * 0.75),
    );

    pointer_down_at(&mut ui, &mut app, &mut services, source_point);
    pointer_move_at(
        &mut ui,
        &mut app,
        &mut services,
        target_lower,
        MouseButtons {
            left: true,
            ..MouseButtons::default()
        },
    );

    app.advance_frame();
    let _root = run_frame(
        &mut ui,
        &mut app,
        &mut services,
        window,
        bounds,
        "imui-sortable-rows",
        render_test_sortable_rows(
            &items,
            &preview_status,
            &delivered_status,
            &order_status,
            &delivered_flag,
        ),
    );

    assert_eq!(
        preview_status.borrow().as_str(),
        "Preview: move Camera after Cube"
    );
    assert!(delivered_status.borrow().is_empty());
    assert_eq!(
        order_status.borrow().as_str(),
        "Camera -> Cube -> Key light"
    );
    assert!(!delivered_flag.get());

    pointer_up_at_with_is_click(&mut ui, &mut app, &mut services, target_lower, false);

    app.advance_frame();
    let _root = run_frame(
        &mut ui,
        &mut app,
        &mut services,
        window,
        bounds,
        "imui-sortable-rows",
        render_test_sortable_rows(
            &items,
            &preview_status,
            &delivered_status,
            &order_status,
            &delivered_flag,
        ),
    );

    assert!(preview_status.borrow().is_empty());
    assert_eq!(
        delivered_status.borrow().as_str(),
        "Moved Camera after Cube"
    );
    assert_eq!(
        order_status.borrow().as_str(),
        "Cube -> Camera -> Key light"
    );
    assert!(delivered_flag.get());
}
