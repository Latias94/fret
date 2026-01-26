use super::*;

struct CountingFocusWidget {
    is_focusable: bool,
    focus_traversal_children: bool,
    can_scroll_descendant_into_view: bool,
    offset_child_by: Option<Point>,
    is_focusable_calls: Arc<AtomicUsize>,
    focus_traversal_children_calls: Arc<AtomicUsize>,
    can_scroll_descendant_into_view_calls: Arc<AtomicUsize>,
}

impl CountingFocusWidget {
    fn new(
        is_focusable: bool,
        can_scroll_descendant_into_view: bool,
        offset_child_by: Option<Point>,
        is_focusable_calls: Arc<AtomicUsize>,
        focus_traversal_children_calls: Arc<AtomicUsize>,
        can_scroll_descendant_into_view_calls: Arc<AtomicUsize>,
    ) -> Self {
        Self {
            is_focusable,
            focus_traversal_children: true,
            can_scroll_descendant_into_view,
            offset_child_by,
            is_focusable_calls,
            focus_traversal_children_calls,
            can_scroll_descendant_into_view_calls,
        }
    }
}

impl<H: UiHost> Widget<H> for CountingFocusWidget {
    fn is_focusable(&self) -> bool {
        self.is_focusable_calls.fetch_add(1, Ordering::SeqCst);
        self.is_focusable
    }

    fn focus_traversal_children(&self) -> bool {
        self.focus_traversal_children_calls
            .fetch_add(1, Ordering::SeqCst);
        self.focus_traversal_children
    }

    fn can_scroll_descendant_into_view(&self) -> bool {
        self.can_scroll_descendant_into_view_calls
            .fetch_add(1, Ordering::SeqCst);
        self.can_scroll_descendant_into_view
    }

    fn layout(&mut self, cx: &mut LayoutCx<'_, H>) -> Size {
        let bounds = cx.bounds;
        for &child in cx.children {
            let child_bounds = if let Some(offset) = self.offset_child_by {
                Rect::new(
                    Point::new(
                        Px(bounds.origin.x.0 + offset.x.0),
                        Px(bounds.origin.y.0 + offset.y.0),
                    ),
                    bounds.size,
                )
            } else {
                bounds
            };
            let _ = cx.layout_in(child, child_bounds);
        }
        cx.available
    }
}

#[test]
fn focus_traversal_uses_prepaint_cache_for_clean_nodes() {
    let mut app = crate::test_host::TestHost::new();
    app.set_global(PlatformCapabilities::default());

    let is_focusable_calls = Arc::new(AtomicUsize::new(0));
    let focus_traversal_children_calls = Arc::new(AtomicUsize::new(0));
    let can_scroll_descendant_into_view_calls = Arc::new(AtomicUsize::new(0));

    let window = AppWindowId::default();
    let mut ui: UiTree<crate::test_host::TestHost> = UiTree::new();
    ui.set_window(window);

    let root = ui.create_node(TestStack::default());
    let scroll = ui.create_node(CountingFocusWidget::new(
        false,
        true,
        Some(Point::new(Px(240.0), Px(0.0))),
        is_focusable_calls.clone(),
        focus_traversal_children_calls.clone(),
        can_scroll_descendant_into_view_calls.clone(),
    ));
    let leaf = ui.create_node(CountingFocusWidget::new(
        true,
        false,
        None,
        is_focusable_calls.clone(),
        focus_traversal_children_calls.clone(),
        can_scroll_descendant_into_view_calls.clone(),
    ));
    ui.set_root(root);
    ui.add_child(root, scroll);
    ui.add_child(scroll, leaf);

    let mut services = FakeUiServices;
    let bounds = Rect::new(
        Point::new(Px(0.0), Px(0.0)),
        Size::new(Px(100.0), Px(100.0)),
    );
    ui.layout_all(&mut app, &mut services, bounds, 1.0);

    let before = (
        is_focusable_calls.load(Ordering::SeqCst),
        focus_traversal_children_calls.load(Ordering::SeqCst),
        can_scroll_descendant_into_view_calls.load(Ordering::SeqCst),
    );

    assert_eq!(
        ui.command_availability(&mut app, &CommandId::from("focus.next")),
        crate::widget::CommandAvailability::Available
    );

    let after = (
        is_focusable_calls.load(Ordering::SeqCst),
        focus_traversal_children_calls.load(Ordering::SeqCst),
        can_scroll_descendant_into_view_calls.load(Ordering::SeqCst),
    );
    assert_eq!(after, before);

    ui.nodes.get_mut(scroll).unwrap().invalidation.hit_test = true;
    ui.nodes.get_mut(leaf).unwrap().invalidation.hit_test = true;

    let _ = ui.command_availability(&mut app, &CommandId::from("focus.next"));

    let final_counts = (
        is_focusable_calls.load(Ordering::SeqCst),
        focus_traversal_children_calls.load(Ordering::SeqCst),
        can_scroll_descendant_into_view_calls.load(Ordering::SeqCst),
    );
    assert!(final_counts.0 > after.0);
    assert!(final_counts.1 > after.1);
    assert!(final_counts.2 > after.2);
}
