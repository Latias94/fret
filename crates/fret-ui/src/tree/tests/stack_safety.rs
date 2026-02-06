use super::*;
use std::sync::{
    Arc,
    atomic::{AtomicBool, Ordering},
};

#[derive(Default)]
struct StackSafetyProbe {
    layout_is_protected: Arc<AtomicBool>,
    paint_is_protected: Arc<AtomicBool>,
}

impl StackSafetyProbe {
    fn new(layout_is_protected: Arc<AtomicBool>, paint_is_protected: Arc<AtomicBool>) -> Self {
        Self {
            layout_is_protected,
            paint_is_protected,
        }
    }
}

impl<H: UiHost> Widget<H> for StackSafetyProbe {
    fn layout(&mut self, cx: &mut LayoutCx<'_, H>) -> Size {
        self.layout_is_protected
            .store(stacksafe::internal::is_protected(), Ordering::SeqCst);
        cx.available
    }

    fn paint(&mut self, cx: &mut PaintCx<'_, H>) {
        self.paint_is_protected
            .store(stacksafe::internal::is_protected(), Ordering::SeqCst);
        let _ = cx.scene;
    }
}

#[test]
fn layout_and_paint_run_inside_stacksafe_protection() {
    let window = AppWindowId::default();

    let mut app = crate::test_host::TestHost::new();
    let mut ui = UiTree::new();
    ui.set_window(window);

    let root = ui.create_node(TestStack);
    ui.set_root(root);

    let layout_is_protected = Arc::new(AtomicBool::new(false));
    let paint_is_protected = Arc::new(AtomicBool::new(false));
    let probe = ui.create_node(StackSafetyProbe::new(
        layout_is_protected.clone(),
        paint_is_protected.clone(),
    ));
    ui.add_child(root, probe);

    let mut services = FakeUiServices;
    let bounds = Rect::new(
        Point::new(Px(0.0), Px(0.0)),
        Size::new(Px(100.0), Px(100.0)),
    );

    ui.layout_all(&mut app, &mut services, bounds, 1.0);

    let mut scene = Scene::default();
    ui.paint_all(&mut app, &mut services, bounds, &mut scene, 1.0);

    assert!(
        layout_is_protected.load(Ordering::SeqCst),
        "expected UiTree::layout_all to run under stacksafe protection"
    );
    assert!(
        paint_is_protected.load(Ordering::SeqCst),
        "expected UiTree::paint_all to run under stacksafe protection"
    );
}

#[test]
fn hit_test_handles_deep_trees_on_small_stacks() {
    // Hit testing used to be recursive, which can overflow on deep trees.
    // Run the probe on a small stack to guard against regressions.
    std::thread::Builder::new()
        .name("hit_test_stack_safety".to_string())
        .stack_size(512 * 1024)
        .spawn(|| {
            let window = AppWindowId::default();

            let _app = crate::test_host::TestHost::new();
            let mut ui: UiTree<crate::test_host::TestHost> = UiTree::new();
            ui.set_window(window);

            let root = ui.create_node(TestStack);
            ui.set_root(root);

            let mut current = root;
            let depth = 20_000;
            for _ in 0..depth {
                let child = ui.create_node(TestStack);
                ui.add_child(current, child);
                current = child;
            }

            let bounds = Rect::new(
                Point::new(Px(0.0), Px(0.0)),
                Size::new(Px(100.0), Px(100.0)),
            );
            for (_, node) in ui.nodes.iter_mut() {
                node.bounds = bounds;
            }

            let hit = ui.hit_test(root, Point::new(Px(1.0), Px(1.0)));
            assert_eq!(hit, Some(current));
        })
        .expect("spawn test thread")
        .join()
        .expect("join test thread");
}

#[test]
fn remove_and_cleanup_handle_deep_trees_on_small_stacks() {
    std::thread::Builder::new()
        .name("remove_cleanup_stack_safety".to_string())
        .stack_size(512 * 1024)
        .spawn(|| {
            let window = AppWindowId::default();

            let mut ui: UiTree<crate::test_host::TestHost> = UiTree::new();
            ui.set_window(window);

            let root = ui.create_node(TestStack);
            ui.set_root(root);

            let first_child = ui.create_node(TestStack);
            ui.add_child(root, first_child);

            let mut current = first_child;
            let depth = 20_000;
            for _ in 0..depth {
                let child = ui.create_node(TestStack);
                ui.add_child(current, child);
                current = child;
            }

            let mut services = FakeUiServices;
            ui.cleanup_subtree(&mut services, first_child);
            let removed = ui.remove_subtree(&mut services, first_child);
            assert_eq!(
                removed.len(),
                depth + 1,
                "expected to remove the full chain without recursion"
            );
        })
        .expect("spawn test thread")
        .join()
        .expect("join test thread");
}
