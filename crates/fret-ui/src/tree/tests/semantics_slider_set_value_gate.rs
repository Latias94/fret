use super::*;

use fret_core::{AppWindowId, Point, Px, Rect, SemanticsRole, Size};

#[derive(Default)]
struct SliderLikeNoNumeric;

impl<H: UiHost> Widget<H> for SliderLikeNoNumeric {
    fn layout(&mut self, cx: &mut LayoutCx<'_, H>) -> Size {
        cx.available
    }

    fn semantics(&mut self, cx: &mut SemanticsCx<'_, H>) {
        cx.set_role(SemanticsRole::Slider);
        cx.set_value_editable(true);
    }
}

#[derive(Default)]
struct SliderLikeWithNumeric;

impl<H: UiHost> Widget<H> for SliderLikeWithNumeric {
    fn layout(&mut self, cx: &mut LayoutCx<'_, H>) -> Size {
        cx.available
    }

    fn semantics(&mut self, cx: &mut SemanticsCx<'_, H>) {
        cx.set_role(SemanticsRole::Slider);
        cx.set_numeric_value(Some(50.0));
        cx.set_numeric_range(Some(0.0), Some(100.0));
        cx.set_numeric_step(Some(1.0));
        cx.set_value_editable(true);
    }
}

#[test]
fn slider_set_value_is_only_exposed_when_numeric_metadata_is_present() {
    let window = AppWindowId::default();
    let mut app = crate::test_host::TestHost::default();
    let mut services = FakeUiServices::default();

    let mut ui: UiTree<crate::test_host::TestHost> = UiTree::new();
    ui.set_window(window);

    let bounds = Rect::new(
        Point::new(Px(0.0), Px(0.0)),
        fret_core::Size::new(Px(200.0), Px(120.0)),
    );

    let root = ui.create_node(TestStack::default());
    let no_numeric = ui.create_node(SliderLikeNoNumeric::default());
    let with_numeric = ui.create_node(SliderLikeWithNumeric::default());
    ui.add_child(root, no_numeric);
    ui.add_child(root, with_numeric);
    ui.set_root(root);

    ui.request_semantics_snapshot();
    ui.layout_all(&mut app, &mut services, bounds, 1.0);

    let snap = ui.semantics_snapshot().expect("semantics snapshot");

    let no_numeric_node = snap
        .nodes
        .iter()
        .find(|n| n.id == no_numeric)
        .expect("node");
    assert!(no_numeric_node.actions.increment);
    assert!(no_numeric_node.actions.decrement);
    assert!(
        !no_numeric_node.actions.set_value,
        "expected SetValue to be gated off without numeric metadata"
    );

    let with_numeric_node = snap
        .nodes
        .iter()
        .find(|n| n.id == with_numeric)
        .expect("node");
    assert!(with_numeric_node.actions.increment);
    assert!(with_numeric_node.actions.decrement);
    assert!(with_numeric_node.actions.set_value);
}
