use super::*;

struct CursorQueryWidget;

impl<H: UiHost> Widget<H> for CursorQueryWidget {
    fn layout(&mut self, _cx: &mut LayoutCx<'_, H>) -> Size {
        Size::new(Px(100.0), Px(100.0))
    }

    fn cursor_icon_at(
        &self,
        _bounds: Rect,
        _position: Point,
        _input_ctx: &fret_runtime::InputContext,
    ) -> Option<fret_core::CursorIcon> {
        Some(fret_core::CursorIcon::Pointer)
    }
}

#[test]
fn cursor_icon_query_drives_cursor_set_icon_effect_on_pointer_move() {
    let window = AppWindowId::default();

    let mut app = crate::test_host::TestHost::new();
    let mut ui = UiTree::new();
    ui.set_window(window);

    let root = ui.create_node(CursorQueryWidget);
    ui.set_root(root);

    let mut services = FakeUiServices;
    let bounds = Rect::new(
        Point::new(Px(0.0), Px(0.0)),
        Size::new(Px(100.0), Px(100.0)),
    );
    ui.layout_all(&mut app, &mut services, bounds, 1.0);

    ui.dispatch_event(
        &mut app,
        &mut services,
        &Event::Pointer(fret_core::PointerEvent::Move {
            position: Point::new(Px(50.0), Px(50.0)),
            buttons: fret_core::MouseButtons::default(),
            modifiers: fret_core::Modifiers::default(),
            pointer_id: fret_core::PointerId(0),
            pointer_type: fret_core::PointerType::Mouse,
        }),
    );

    let effects = app.flush_effects();
    assert!(
        effects.iter().any(|effect| matches!(
            effect,
            fret_runtime::Effect::CursorSetIcon {
                window: w,
                icon: fret_core::CursorIcon::Pointer
            } if *w == window
        )),
        "expected cursor icon query to produce a CursorSetIcon effect"
    );
}

struct SimpleContainerWidget;

impl<H: UiHost> Widget<H> for SimpleContainerWidget {
    fn layout(&mut self, cx: &mut LayoutCx<'_, H>) -> Size {
        let size = Size::new(Px(100.0), Px(100.0));
        if let Some(&child) = cx.children.first() {
            cx.layout_in(child, Rect::new(cx.bounds.origin, size));
        }
        size
    }
}

struct TranslateChildrenWidget {
    delta: Point,
}

impl<H: UiHost> Widget<H> for TranslateChildrenWidget {
    fn layout(&mut self, cx: &mut LayoutCx<'_, H>) -> Size {
        let size = Size::new(Px(100.0), Px(100.0));
        if let Some(&child) = cx.children.first() {
            cx.layout_in(child, Rect::new(cx.bounds.origin, size));
        }
        size
    }

    fn children_render_transform(&self, _bounds: Rect) -> Option<Transform2D> {
        Some(Transform2D::translation(self.delta))
    }
}

struct CursorQueryExpectX {
    expected_x: Px,
}

impl<H: UiHost> Widget<H> for CursorQueryExpectX {
    fn layout(&mut self, _cx: &mut LayoutCx<'_, H>) -> Size {
        Size::new(Px(100.0), Px(100.0))
    }

    fn cursor_icon_at(
        &self,
        _bounds: Rect,
        position: Point,
        _input_ctx: &fret_runtime::InputContext,
    ) -> Option<fret_core::CursorIcon> {
        let dx = (position.x.0 - self.expected_x.0).abs();
        (dx < 0.01).then_some(fret_core::CursorIcon::Pointer)
    }
}

#[test]
fn cursor_icon_query_remains_correct_with_stale_parent_pointers() {
    let window = AppWindowId::default();

    let mut app = crate::test_host::TestHost::new();
    let mut ui = UiTree::new();
    ui.set_window(window);

    let root = ui.create_node(SimpleContainerWidget);
    ui.set_root(root);

    let translate = ui.create_node(TranslateChildrenWidget {
        delta: Point::new(Px(20.0), Px(0.0)),
    });
    ui.add_child(root, translate);

    let leaf = ui.create_node(CursorQueryExpectX {
        expected_x: Px(5.0),
    });
    ui.add_child(translate, leaf);

    // Simulate retained/view-cache reuse glitches where `parent` pointers can be stale even when
    // child-edge traversal remains correct. Dispatch must not depend on the long-lived `parent`.
    ui.nodes.get_mut(leaf).unwrap().parent = None;

    let mut services = FakeUiServices;
    let bounds = Rect::new(
        Point::new(Px(0.0), Px(0.0)),
        Size::new(Px(100.0), Px(100.0)),
    );
    ui.layout_all(&mut app, &mut services, bounds, 1.0);

    ui.dispatch_event(
        &mut app,
        &mut services,
        &Event::Pointer(fret_core::PointerEvent::Move {
            position: Point::new(Px(25.0), Px(50.0)),
            buttons: fret_core::MouseButtons::default(),
            modifiers: fret_core::Modifiers::default(),
            pointer_id: fret_core::PointerId(0),
            pointer_type: fret_core::PointerType::Mouse,
        }),
    );

    let effects = app.flush_effects();
    assert!(
        effects.iter().any(|effect| matches!(
            effect,
            fret_runtime::Effect::CursorSetIcon {
                window: w,
                icon: fret_core::CursorIcon::Pointer
            } if *w == window
        )),
        "expected cursor icon query to remain correct under stale parent pointers"
    );
}
