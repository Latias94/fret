use super::*;

struct DeclarativeTabDropHarness {
    ui: UiTree<TestHost>,
    app: TestHost,
    services: FakeTextService,
    window: AppWindowId,
    tabs_node: fret_core::DockNodeId,
    dragged_panel: PanelKey,
    bounds: Rect,
}

impl DeclarativeTabDropHarness {
    fn single_tabs(tab_count: usize, bounds: Rect) -> Self {
        Self::single_tabs_with_dragged_index(tab_count, bounds, 0)
    }

    fn single_tabs_with_dragged_index(
        tab_count: usize,
        bounds: Rect,
        dragged_index: usize,
    ) -> Self {
        let window = AppWindowId::default();

        let mut ui: UiTree<TestHost> = UiTree::new();
        ui.set_window(window);

        let mut app = TestHost::new();
        app.set_global(PlatformCapabilities::default());
        app.set_global(DockManager::default());
        app.with_global_mut(
            DockPanelElementRegistryService::<TestHost>::default,
            |svc, _app| {
                svc.set(Arc::new(EmptyRegistry));
            },
        );

        let (tabs_node, dragged_panel) = app.with_global_mut(DockManager::default, |dock, _app| {
            let panels: Vec<PanelKey> = (0..tab_count)
                .map(|index| PanelKey::new(format!("demo.public.declarative.tab-drop.{index}")))
                .collect();
            let dragged_panel = panels
                .get(dragged_index)
                .cloned()
                .expect("dragged index must reference an existing tab");
            for (index, panel) in panels.iter().enumerate() {
                dock.ensure_panel(panel, || crate::DockPanel {
                    title: format!("Panel {index}"),
                    color: fret_core::Color::TRANSPARENT,
                    viewport: None,
                });
            }
            let tabs = dock.workspace.graph.insert_node(DockNode::Tabs {
                tabs: panels,
                active: 0,
            });
            dock.workspace.graph.set_window_root(window, tabs);
            (tabs, dragged_panel)
        });

        let mut services = FakeTextService;
        let root = declarative::render_root(
            &mut ui,
            &mut app,
            &mut services,
            window,
            bounds,
            "public-declarative-dock-host-tab-drop",
            move |cx| {
                vec![dock_space_element_from_registry(
                    cx,
                    window,
                    DockSpaceElementOptions::default(),
                )]
            },
        );
        ui.set_root(root);
        ui.layout_all(&mut app, &mut services, bounds, 1.0);
        let mut scene = Scene::default();
        ui.paint_all(&mut app, &mut services, bounds, &mut scene, 1.0);

        Self {
            ui,
            app,
            services,
            window,
            tabs_node,
            dragged_panel,
            bounds,
        }
    }

    fn begin_drag(&mut self) {
        self.app.begin_cross_window_drag_with_kind(
            fret_core::PointerId(0),
            DRAG_KIND_DOCK_PANEL,
            self.window,
            Point::new(Px(24.0), Px(12.0)),
            DockPanelDragPayload {
                panel: self.dragged_panel.clone(),
                grab_offset: Point::new(Px(0.0), Px(0.0)),
                tear_off_requested: false,
                tear_off_requested_at_tick: None,
                tear_off_oob_start_frame: None,
                dock_previews_enabled: true,
            },
        );
        if let Some(drag) = self.app.drag_mut(fret_core::PointerId(0)) {
            drag.dragging = true;
            drag.current_window = self.window;
        }
        let _ = self.app.take_effects();
    }

    fn drop_at(&mut self, position: Point) -> DockOp {
        for kind in [InternalDragKind::Over, InternalDragKind::Drop] {
            self.ui.dispatch_event(
                &mut self.app,
                &mut self.services,
                &Event::InternalDrag(InternalDragEvent {
                    position,
                    kind,
                    modifiers: Modifiers::default(),
                    pointer_id: fret_core::PointerId(0),
                }),
            );
        }
        self.app
            .take_effects()
            .into_iter()
            .find_map(|effect| match effect {
                Effect::Dock(op) => Some(op),
                _ => None,
            })
            .expect("expected declarative tab drop to emit a Dock op")
    }

    fn tab_drop_position(&self, tab_index: usize, fraction: f32) -> Point {
        let tab_count = self
            .app
            .global::<DockManager>()
            .and_then(|dock| match dock.workspace.graph.node(self.tabs_node) {
                Some(DockNode::Tabs { tabs, .. }) => Some(tabs.len()),
                _ => None,
            })
            .expect("expected target tabs node");
        let theme = fret_ui::Theme::global(&self.app).snapshot();
        let (_chrome, dock_bounds) = dock_space_regions(self.bounds);
        let (tab_bar, _content) = split_tab_bar(dock_bounds);
        let tab_width =
            crate::dock::tab_bar_geometry::dock_tab_width_for_title(theme.clone(), Px(240.0), true);
        let tab_widths: Arc<[Px]> = vec![tab_width; tab_count].into();
        let candidate = crate::dock::tab_bar_kernel::compute_tab_bar_overflow_candidate_geometry(
            theme,
            tab_bar,
            tab_count,
            Some(&tab_widths),
        );
        let geom = if candidate.overflows {
            candidate.geom
        } else {
            crate::dock::tab_bar_geometry::TabBarGeometry::variable(tab_bar, tab_widths)
        };
        let tab_rect = geom.tab_rect(tab_index, Px(0.0));
        Point::new(
            Px(tab_rect.origin.x.0 + tab_rect.size.width.0 * fraction),
            Px(tab_rect.origin.y.0 + tab_rect.size.height.0 * 0.5),
        )
    }

    fn tab_bar_reserved_header_position(&self, tab_count: usize) -> Point {
        let theme = fret_ui::Theme::global(&self.app).snapshot();
        let (_chrome, dock_bounds) = dock_space_regions(self.bounds);
        let (tab_bar, _content) = split_tab_bar(dock_bounds);
        let candidate = crate::dock::tab_bar_kernel::compute_tab_bar_overflow_candidate_geometry(
            theme, tab_bar, tab_count, None,
        );
        assert!(candidate.overflows, "expected tab strip to overflow");
        let strip_end = candidate.strip_rect.origin.x.0 + candidate.strip_rect.size.width.0;
        let button_start = candidate.overflow_button_rect.origin.x.0;
        assert!(
            button_start > strip_end,
            "expected reserved header space before overflow button"
        );
        Point::new(
            Px((strip_end + button_start) * 0.5),
            Px(tab_bar.origin.y.0 + tab_bar.size.height.0 * 0.5),
        )
    }
}

#[test]
fn public_declarative_dock_space_entry_point_tab_drop_uses_over_tab_halves_for_insert_index() {
    let bounds = Rect::new(
        Point::new(Px(0.0), Px(0.0)),
        Size::new(Px(800.0), Px(600.0)),
    );

    let mut left = DeclarativeTabDropHarness::single_tabs(3, bounds);
    left.begin_drag();
    let op = left.drop_at(left.tab_drop_position(1, 0.25));
    assert!(
        left.app.drag(fret_core::PointerId(0)).is_none(),
        "expected declarative tab drop to end the drag session"
    );
    assert!(
        matches!(
            op,
            DockOp::MovePanel {
                target_tabs,
                zone: DropZone::Center,
                insert_index: Some(1),
                ..
            } if target_tabs == left.tabs_node
        ),
        "expected left half of tab 1 to insert before tab 1, got: {op:?}"
    );

    let mut right = DeclarativeTabDropHarness::single_tabs(3, bounds);
    right.begin_drag();
    let op = right.drop_at(right.tab_drop_position(1, 0.75));
    assert!(
        matches!(
            op,
            DockOp::MovePanel {
                target_tabs,
                zone: DropZone::Center,
                insert_index: Some(2),
                ..
            } if target_tabs == right.tabs_node
        ),
        "expected right half of tab 1 to insert after tab 1, got: {op:?}"
    );
}

#[test]
fn public_declarative_dock_space_entry_point_tab_drop_reorders_tabs_when_move_op_is_applied() {
    let bounds = Rect::new(
        Point::new(Px(0.0), Px(0.0)),
        Size::new(Px(800.0), Px(600.0)),
    );
    let mut harness = DeclarativeTabDropHarness::single_tabs_with_dragged_index(4, bounds, 3);
    let source_panel = harness.dragged_panel.clone();
    harness.begin_drag();
    let op = harness.drop_at(harness.tab_drop_position(1, 0.75));

    harness
        .app
        .with_global_mut(DockManager::default, |dock, _app| {
            assert!(
                dock.workspace
                    .graph
                    .apply_op_checked(&op)
                    .expect("apply must succeed"),
                "expected emitted tab-drop op to mutate the graph"
            );
            let Some(DockNode::Tabs { tabs, .. }) = dock.workspace.graph.node(harness.tabs_node)
            else {
                panic!("expected target tabs node to remain tabs");
            };
            assert_eq!(
                tabs,
                &vec![
                    PanelKey::new("demo.public.declarative.tab-drop.0"),
                    PanelKey::new("demo.public.declarative.tab-drop.1"),
                    source_panel,
                    PanelKey::new("demo.public.declarative.tab-drop.2"),
                ],
                "expected applied declarative tab drop to insert dragged panel after tab 1"
            );
        });
}

#[test]
fn public_declarative_dock_space_entry_point_tab_drop_reserved_overflow_header_inserts_at_end() {
    let bounds = Rect::new(
        Point::new(Px(0.0), Px(0.0)),
        Size::new(Px(260.0), Px(180.0)),
    );
    let tab_count = 10;
    let mut harness = DeclarativeTabDropHarness::single_tabs(tab_count, bounds);
    let position = harness.tab_bar_reserved_header_position(tab_count);
    harness.begin_drag();
    let op = harness.drop_at(position);

    assert!(
        matches!(
            op,
            DockOp::MovePanel {
                target_tabs,
                zone: DropZone::Center,
                insert_index: Some(ix),
                ..
            } if target_tabs == harness.tabs_node && ix == tab_count
        ),
        "expected reserved overflow header space to insert at tab_count={tab_count}, got: {op:?}"
    );
}

#[test]
fn public_declarative_dock_space_entry_point_auto_scrolls_tab_bar_during_dock_drag() {
    struct EmptyRegistry;

    impl DockPanelElementRegistry<TestHost> for EmptyRegistry {
        fn render_panel(
            &self,
            _cx: &mut fret_ui::ElementContext<'_, TestHost>,
            _window: AppWindowId,
            _panel: &PanelKey,
        ) -> Option<fret_ui::element::AnyElement> {
            None
        }
    }

    let window = AppWindowId::default();

    let mut ui: UiTree<TestHost> = UiTree::new();
    ui.set_window(window);

    let mut app = TestHost::new();
    app.set_global(PlatformCapabilities::default());
    app.set_global(DockManager::default());
    app.with_global_mut(
        DockPanelElementRegistryService::<TestHost>::default,
        |svc, _app| {
            svc.set(Arc::new(EmptyRegistry));
        },
    );
    let tabs_node = app.with_global_mut(DockManager::default, |dock, _app| {
        let tabs: Vec<PanelKey> = (0..30)
            .map(|index| PanelKey::new(format!("demo.public.declarative.auto-scroll.{index}")))
            .collect();
        for (index, panel) in tabs.iter().enumerate() {
            dock.ensure_panel(panel, || crate::DockPanel {
                title: format!("T{index}"),
                color: fret_core::Color::TRANSPARENT,
                viewport: None,
            });
        }
        let tabs_node = dock
            .workspace
            .graph
            .insert_node(DockNode::Tabs { tabs, active: 0 });
        dock.workspace.graph.set_window_root(window, tabs_node);
        dock.ensure_panel(
            &PanelKey::new("demo.public.declarative.auto-scroll.dragged"),
            || crate::DockPanel {
                title: "Dragged".to_string(),
                color: fret_core::Color::TRANSPARENT,
                viewport: None,
            },
        );
        tabs_node
    });

    let bounds = Rect::new(
        Point::new(Px(0.0), Px(0.0)),
        Size::new(Px(800.0), Px(600.0)),
    );
    let mut services = FakeTextService;
    let root = declarative::render_root(
        &mut ui,
        &mut app,
        &mut services,
        window,
        bounds,
        "public-declarative-dock-host-tab-bar-drag-auto-scroll",
        move |cx| {
            vec![dock_space_element_from_registry(
                cx,
                window,
                DockSpaceElementOptions::default(),
            )]
        },
    );
    ui.set_root(root);
    ui.layout_all(&mut app, &mut services, bounds, 1.0);
    let mut scene = Scene::default();
    ui.paint_all(&mut app, &mut services, bounds, &mut scene, 1.0);

    app.begin_cross_window_drag_with_kind(
        fret_core::PointerId(0),
        DRAG_KIND_DOCK_PANEL,
        window,
        Point::new(Px(12.0), Px(12.0)),
        DockPanelDragPayload {
            panel: PanelKey::new("demo.public.declarative.auto-scroll.dragged"),
            grab_offset: Point::new(Px(0.0), Px(0.0)),
            tear_off_requested: false,
            tear_off_requested_at_tick: None,
            tear_off_oob_start_frame: None,
            dock_previews_enabled: true,
        },
    );
    if let Some(drag) = app.drag_mut(fret_core::PointerId(0)) {
        drag.dragging = true;
        drag.current_window = window;
    }

    let (_chrome, dock_bounds) = dock_space_regions(bounds);
    let (tab_bar, _content) = split_tab_bar(dock_bounds);
    let pos_right = Point::new(
        Px(tab_bar.origin.x.0 + tab_bar.size.width.0 - 2.0),
        Px(tab_bar.origin.y.0 + 6.0),
    );

    ui.dispatch_event(
        &mut app,
        &mut services,
        &Event::InternalDrag(InternalDragEvent {
            position: pos_right,
            kind: InternalDragKind::Over,
            modifiers: Modifiers::default(),
            pointer_id: fret_core::PointerId(0),
        }),
    );

    let first_ix = match app
        .global::<DockManager>()
        .and_then(|dock| dock.presentation.hover.clone())
    {
        Some(DockDropTarget::Dock(target)) => {
            assert_eq!(target.tabs, tabs_node);
            target.insert_index.expect("expected a tab insert index")
        }
        other => panic!("expected declarative tab-bar dock hover target, got: {other:?}"),
    };

    let mut ix_after_scroll = first_ix;
    for _ in 0..6 {
        app.advance_frame();
        ui.dispatch_event(
            &mut app,
            &mut services,
            &Event::InternalDrag(InternalDragEvent {
                position: pos_right,
                kind: InternalDragKind::Over,
                modifiers: Modifiers::default(),
                pointer_id: fret_core::PointerId(0),
            }),
        );
        if let Some(DockDropTarget::Dock(target)) = app
            .global::<DockManager>()
            .and_then(|dock| dock.presentation.hover.clone())
        {
            ix_after_scroll = target.insert_index.expect("expected insert index");
        }
    }

    assert!(
        ix_after_scroll > first_ix,
        "expected declarative auto-scroll at the right edge to increase the insert index, before={first_ix}, after={ix_after_scroll}",
    );
}
