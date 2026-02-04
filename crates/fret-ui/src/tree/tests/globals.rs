use super::*;
use std::any::TypeId;

struct GlobalObservingWidget;

impl<H: UiHost> Widget<H> for GlobalObservingWidget {
    fn layout(&mut self, cx: &mut LayoutCx<'_, H>) -> Size {
        cx.observe_global::<u32>(Invalidation::Layout);
        Size::new(fret_core::Px(10.0), fret_core::Px(10.0))
    }
}

#[test]
fn global_change_invalidates_observers() {
    let mut app = crate::test_host::TestHost::new();
    app.set_global::<u32>(1);

    let window = AppWindowId::default();

    let mut ui = UiTree::new();
    ui.set_window(window);

    let node = ui.create_node(GlobalObservingWidget);
    ui.set_root(node);

    let mut services = FakeUiServices;
    let bounds = Rect::new(
        Point::new(fret_core::Px(0.0), fret_core::Px(0.0)),
        Size::new(fret_core::Px(100.0), fret_core::Px(100.0)),
    );
    ui.layout_all(&mut app, &mut services, bounds, 1.0);
    let mut scene = Scene::default();
    ui.paint_all(&mut app, &mut services, bounds, &mut scene, 1.0);
    ui.test_clear_node_invalidations(node);

    let changed = vec![TypeId::of::<u32>()];
    assert!(ui.propagate_global_changes(&mut app, &changed));

    let n = ui.nodes.get(node).unwrap();
    assert!(n.invalidation.layout);
    assert!(n.invalidation.paint);

    let effects = app.flush_effects();
    assert!(
        effects
            .iter()
            .any(|e| matches!(e, Effect::Redraw(w) if *w == window))
    );
}
