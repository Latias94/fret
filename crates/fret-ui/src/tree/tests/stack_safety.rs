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

    let root = ui.create_node(TestStack::default());
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
