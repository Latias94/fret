use super::*;

#[derive(Default)]
struct FocusableLeaf;

impl<H: UiHost> Widget<H> for FocusableLeaf {
    fn is_focusable(&self) -> bool {
        true
    }

    fn layout(&mut self, cx: &mut LayoutCx<'_, H>) -> Size {
        cx.available
    }
}

#[test]
fn focus_next_is_available_when_focusables_exist() {
    let mut app = crate::test_host::TestHost::new();
    app.set_global(PlatformCapabilities::default());

    let window = AppWindowId::default();
    let mut ui: UiTree<crate::test_host::TestHost> = UiTree::new();
    ui.set_window(window);

    let root = ui.create_node(TestStack);
    let leaf = ui.create_node(FocusableLeaf);
    ui.set_root(root);
    ui.add_child(root, leaf);

    let mut services = FakeUiServices;
    let bounds = Rect::new(Point::new(Px(0.0), Px(0.0)), Size::new(Px(100.0), Px(40.0)));
    ui.layout_all(&mut app, &mut services, bounds, 1.0);

    assert_eq!(
        ui.command_availability(&mut app, &CommandId::from("focus.next")),
        crate::widget::CommandAvailability::Available
    );
    assert!(ui.is_command_available(&mut app, &CommandId::from("focus.next")));
}

#[test]
fn focus_next_is_unavailable_when_no_focusables_exist() {
    let mut app = crate::test_host::TestHost::new();
    app.set_global(PlatformCapabilities::default());

    let window = AppWindowId::default();
    let mut ui: UiTree<crate::test_host::TestHost> = UiTree::new();
    ui.set_window(window);

    let root = ui.create_node(TestStack);
    ui.set_root(root);

    let mut services = FakeUiServices;
    let bounds = Rect::new(Point::new(Px(0.0), Px(0.0)), Size::new(Px(100.0), Px(40.0)));
    ui.layout_all(&mut app, &mut services, bounds, 1.0);

    assert_eq!(
        ui.command_availability(&mut app, &CommandId::from("focus.next")),
        crate::widget::CommandAvailability::NotHandled
    );
    assert!(!ui.is_command_available(&mut app, &CommandId::from("focus.next")));
}
