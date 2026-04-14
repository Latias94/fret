use super::*;

#[test]
fn descendant_via_children_ignores_stale_parent_pointers() {
    struct PassiveWidget;

    impl<H: UiHost> Widget<H> for PassiveWidget {
        fn layout(&mut self, cx: &mut LayoutCx<'_, H>) -> Size {
            cx.available
        }
    }

    let mut ui: UiTree<crate::test_host::TestHost> = UiTree::new();

    let root = ui.create_node(PassiveWidget);
    let child = ui.create_node(PassiveWidget);
    let grandchild = ui.create_node(PassiveWidget);

    ui.set_root(root);
    ui.add_child(root, child);
    ui.add_child(child, grandchild);

    assert!(ui.is_descendant(root, grandchild));
    assert!(ui.is_descendant_via_children(root, grandchild));

    ui.nodes[grandchild].parent = None;

    assert!(
        !ui.is_descendant(root, grandchild),
        "expected retained parent-pointer traversal to observe the injected stale parent"
    );
    assert!(
        ui.is_descendant_via_children(root, grandchild),
        "expected child-edge traversal to remain authoritative even with stale parent pointers"
    );
}
