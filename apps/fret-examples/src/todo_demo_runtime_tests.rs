use super::*;
use fret::advanced::view::{
    AppUiRenderRootState, ViewWindowState, render_root_with_app_ui, view_init_window, view_view,
};
use fret_core::{Point, PointerType, Rect, Size, WindowMetricsService};
use fret_icons::IconRegistry;
use fret_runtime::{FrameId, TickId};
use fret_ui::UiTree;

#[derive(Default)]
struct FakeUiServices;

impl fret_core::TextService for FakeUiServices {
    fn prepare(
        &mut self,
        _input: &fret_core::TextInput,
        _constraints: fret_core::TextConstraints,
    ) -> (fret_core::TextBlobId, fret_core::TextMetrics) {
        (
            fret_core::TextBlobId::default(),
            fret_core::TextMetrics {
                size: fret_core::Size::new(Px(10.0), Px(10.0)),
                baseline: Px(8.0),
            },
        )
    }

    fn release(&mut self, _blob: fret_core::TextBlobId) {}
}

impl fret_core::PathService for FakeUiServices {
    fn prepare(
        &mut self,
        _commands: &[fret_core::PathCommand],
        _style: fret_core::PathStyle,
        _constraints: fret_core::PathConstraints,
    ) -> (fret_core::PathId, fret_core::PathMetrics) {
        (
            fret_core::PathId::default(),
            fret_core::PathMetrics::default(),
        )
    }

    fn release(&mut self, _path: fret_core::PathId) {}
}

impl fret_core::SvgService for FakeUiServices {
    fn register_svg(&mut self, _bytes: &[u8]) -> fret_core::SvgId {
        fret_core::SvgId::default()
    }

    fn unregister_svg(&mut self, _svg: fret_core::SvgId) -> bool {
        true
    }
}

impl fret_core::MaterialService for FakeUiServices {
    fn register_material(
        &mut self,
        _desc: fret_core::MaterialDescriptor,
    ) -> Result<fret_core::MaterialId, fret_core::MaterialRegistrationError> {
        Ok(fret_core::MaterialId::default())
    }

    fn unregister_material(&mut self, _material: fret_core::MaterialId) -> bool {
        true
    }
}

fn render_todo_demo_snapshot_for_frame(
    app: &mut App,
    ui: &mut UiTree<App>,
    services: &mut FakeUiServices,
    window: WindowId,
    bounds: Rect,
    root_state: &mut AppUiRenderRootState,
    view: &mut TodoDemoView,
    frame_id: u64,
) -> fret_core::SemanticsSnapshot {
    app.set_frame_id(FrameId(frame_id));
    let root = render_root_with_app_ui(
        fret_ui::declarative::RenderRootContext::new(ui, app, services, window, bounds),
        "todo-demo-test",
        root_state,
        |cx| view.render(cx),
    );
    ui.set_root(root);
    ui.request_semantics_snapshot();
    ui.layout_all(app, services, bounds, 1.0);
    ui.semantics_snapshot()
        .expect("todo demo semantics snapshot should exist")
        .clone()
}

fn seed_window_metrics(app: &mut App, window: WindowId, bounds: Rect, scale_factor: f32) {
    app.with_global_mut_untracked(WindowMetricsService::default, |svc, _app| {
        svc.set_inner_size(window, bounds.size);
        svc.set_scale_factor(window, scale_factor);
        svc.set_focused(window, true);
    });
    app.with_global_mut_untracked(fret_ui::elements::ElementRuntime::new, |rt, _app| {
        rt.set_window_primary_pointer_type(window, PointerType::Unknown);
    });
}

fn render_todo_demo_runtime_snapshot_for_frame(
    app: &mut App,
    ui: &mut UiTree<App>,
    services: &mut FakeUiServices,
    window: WindowId,
    bounds: Rect,
    state: &mut ViewWindowState<TodoDemoView>,
    frame_id: u64,
    scale_factor: f32,
) -> fret_core::SemanticsSnapshot {
    app.set_tick_id(TickId(frame_id));
    app.set_frame_id(FrameId(frame_id));
    seed_window_metrics(app, window, bounds, scale_factor);

    let root = fret_ui::declarative::render_root(
        ui,
        app,
        services,
        window,
        bounds,
        "todo-demo-runtime-test",
        |cx| view_view(cx, state),
    );
    ui.set_root(root);
    ui.request_semantics_snapshot();
    ui.layout_all(app, services, bounds, scale_factor);
    ui.semantics_snapshot()
        .expect("todo demo runtime semantics snapshot should exist")
        .clone()
}

fn snapshot_test_ids(snapshot: &fret_core::SemanticsSnapshot) -> Vec<String> {
    let mut ids: Vec<String> = snapshot
        .nodes
        .iter()
        .filter_map(|node| node.test_id.as_ref().map(|id| id.to_string()))
        .collect();
    ids.sort();
    ids
}

fn node_by_test_id<'a>(
    snapshot: &'a fret_core::SemanticsSnapshot,
    test_id: &str,
) -> Option<&'a fret_core::SemanticsNode> {
    snapshot
        .nodes
        .iter()
        .find(|node| node.test_id.as_deref() == Some(test_id))
}

#[test]
fn todo_demo_registers_vendor_icons_used_by_layout() {
    let mut app = App::new();
    fret_icons_lucide::app::install(&mut app);
    let icons = app
        .global::<IconRegistry>()
        .expect("expected icon registry in todo demo app");

    for id in [
        "lucide.list-todo",
        "lucide.plus",
        "lucide.circle",
        "lucide.check",
        "lucide.sparkles",
        "lucide.trash-2",
    ] {
        assert!(
            icons.resolve(&IconId::new(id)).is_ok(),
            "missing icon: {id}"
        );
    }
}

#[test]
fn todo_demo_responsive_layout_prefers_compact_footer_and_inline_actions_on_narrow_width() {
    let compact = TodoResponsiveLayout::from_viewport(Px(520.0), Px(700.0), true, false);

    assert_eq!(compact.page_padding, Space::N3);
    assert_eq!(compact.section_padding_x, Space::N4);
    assert!(!compact.center_card_vertically);
    assert!(compact.stack_footer);
    assert!(compact.fill_card_height);
    assert!(compact.always_show_row_actions);
}

#[test]
fn todo_demo_responsive_layout_gives_roomy_shells_more_vertical_headroom() {
    let compact = TodoResponsiveLayout::from_viewport(Px(720.0), Px(560.0), true, true);
    let roomy = TodoResponsiveLayout::from_viewport(Px(720.0), Px(820.0), true, true);

    assert_eq!(compact.page_top_padding, Space::N3);
    assert_eq!(roomy.page_top_padding, Space::N8);
    assert!(roomy.card_max_height.0 > compact.card_max_height.0);
    assert!(compact.fill_card_height);
    assert!(!roomy.fill_card_height);
    assert!(!roomy.always_show_row_actions);
}

#[test]
fn todo_demo_responsive_layout_centers_card_once_viewport_is_large_enough() {
    let compact = TodoResponsiveLayout::from_viewport(Px(520.0), Px(620.0), true, false);
    let regular = TodoResponsiveLayout::from_viewport(Px(560.0), Px(660.0), true, true);

    assert!(!compact.center_card_vertically);
    assert!(regular.center_card_vertically);
}

#[test]
fn todo_demo_responsive_layout_keeps_inline_row_actions_for_non_hover_pointers() {
    let touch = TodoResponsiveLayout::from_viewport(Px(820.0), Px(760.0), false, true);

    assert!(touch.always_show_row_actions);
    assert!(!touch.stack_footer);
}

#[test]
fn todo_demo_compact_start_snapshot_keeps_footer_filters_across_cache_reuse() {
    let mut app = App::new();
    install_demo_theme(&mut app);
    fret_icons_lucide::app::install(&mut app);

    let window = WindowId::default();
    let bounds = Rect::new(
        Point::new(Px(0.0), Px(0.0)),
        Size::new(Px(420.0), Px(560.0)),
    );
    let mut ui = UiTree::<App>::new();
    ui.set_window(window);
    ui.set_view_cache_enabled(true);
    let mut services = FakeUiServices;
    let mut root_state = AppUiRenderRootState::default();
    let mut view = TodoDemoView;

    let frame1 = render_todo_demo_snapshot_for_frame(
        &mut app,
        &mut ui,
        &mut services,
        window,
        bounds,
        &mut root_state,
        &mut view,
        1,
    );
    let frame2 = render_todo_demo_snapshot_for_frame(
        &mut app,
        &mut ui,
        &mut services,
        window,
        bounds,
        &mut root_state,
        &mut view,
        2,
    );

    let mut failures: Vec<String> = Vec::new();
    for (label, snapshot) in [("frame1", &frame1), ("frame2", &frame2)] {
        let ids = snapshot_test_ids(snapshot);
        let rows = node_by_test_id(snapshot, TEST_ID_ROWS)
            .expect("rows viewport should exist in compact startup snapshot");
        if !ids.iter().any(|id| id == TEST_ID_FILTER_ALL) {
            failures.push(format!(
                "{label} should keep the All filter chip in compact startup snapshot; ids={ids:?}"
            ));
        }
        if !ids.iter().any(|id| id == TEST_ID_FILTER_ACTIVE) {
            failures.push(format!(
                    "{label} should keep the Active filter chip in compact startup snapshot; ids={ids:?}"
                ));
        }
        if !ids.iter().any(|id| id == TEST_ID_FILTER_COMPLETED) {
            failures.push(format!(
                    "{label} should keep the Completed filter chip in compact startup snapshot; ids={ids:?}"
                ));
        }
        if rows.bounds.size.height.0 <= 0.0 {
            failures.push(format!(
                    "{label} rows viewport should keep positive height in compact startup snapshot; rows_bounds={:?}",
                    rows.bounds
                ));
        }
    }

    if !failures.is_empty() {
        panic!("{}", failures.join("\n"));
    }
}

#[test]
fn todo_demo_resize_to_compact_keeps_footer_filters_across_cache_reuse() {
    let mut app = App::new();
    install_demo_theme(&mut app);
    fret_icons_lucide::app::install(&mut app);

    let window = WindowId::default();
    let roomy_bounds = Rect::new(
        Point::new(Px(0.0), Px(0.0)),
        Size::new(Px(560.0), Px(660.0)),
    );
    let compact_bounds = Rect::new(
        Point::new(Px(0.0), Px(0.0)),
        Size::new(Px(420.0), Px(560.0)),
    );
    let mut ui = UiTree::<App>::new();
    ui.set_window(window);
    ui.set_view_cache_enabled(true);
    let mut services = FakeUiServices;
    let mut root_state = AppUiRenderRootState::default();
    let mut view = TodoDemoView;

    let _frame1 = render_todo_demo_snapshot_for_frame(
        &mut app,
        &mut ui,
        &mut services,
        window,
        roomy_bounds,
        &mut root_state,
        &mut view,
        1,
    );
    let frame2 = render_todo_demo_snapshot_for_frame(
        &mut app,
        &mut ui,
        &mut services,
        window,
        compact_bounds,
        &mut root_state,
        &mut view,
        2,
    );
    let frame3 = render_todo_demo_snapshot_for_frame(
        &mut app,
        &mut ui,
        &mut services,
        window,
        compact_bounds,
        &mut root_state,
        &mut view,
        3,
    );

    let mut failures: Vec<String> = Vec::new();
    for (label, snapshot) in [("frame2", &frame2), ("frame3", &frame3)] {
        let ids = snapshot_test_ids(snapshot);
        let rows = node_by_test_id(snapshot, TEST_ID_ROWS)
            .expect("rows viewport should exist after resize to compact");
        for filter_id in [
            TEST_ID_FILTER_ALL,
            TEST_ID_FILTER_ACTIVE,
            TEST_ID_FILTER_COMPLETED,
        ] {
            if !ids.iter().any(|id| id == filter_id) {
                failures.push(format!(
                    "{label} should keep {filter_id} after resize to compact; ids={ids:?}"
                ));
            }
        }
        if rows.bounds.size.height.0 <= 0.0 {
            failures.push(format!(
                    "{label} rows viewport should keep positive height after resize to compact; rows_bounds={:?}",
                    rows.bounds
                ));
        }
    }

    if !failures.is_empty() {
        panic!("{}", failures.join("\n"));
    }
}

#[test]
fn todo_demo_compact_start_keeps_footer_filters_after_many_reuse_frames() {
    let mut app = App::new();
    install_demo_theme(&mut app);
    fret_icons_lucide::app::install(&mut app);

    let window = WindowId::default();
    let bounds = Rect::new(
        Point::new(Px(0.0), Px(0.0)),
        Size::new(Px(420.0), Px(560.0)),
    );
    let mut ui = UiTree::<App>::new();
    ui.set_window(window);
    ui.set_view_cache_enabled(true);
    let mut services = FakeUiServices;
    let mut root_state = AppUiRenderRootState::default();
    let mut view = TodoDemoView;

    let mut last_snapshot = None;
    for frame_id in 1..=22 {
        last_snapshot = Some(render_todo_demo_snapshot_for_frame(
            &mut app,
            &mut ui,
            &mut services,
            window,
            bounds,
            &mut root_state,
            &mut view,
            frame_id,
        ));
    }

    let snapshot = last_snapshot.expect("expected final compact snapshot");
    let ids = snapshot_test_ids(&snapshot);
    let rows =
        node_by_test_id(&snapshot, TEST_ID_ROWS).expect("rows viewport should exist at frame22");

    for filter_id in [
        TEST_ID_FILTER_ALL,
        TEST_ID_FILTER_ACTIVE,
        TEST_ID_FILTER_COMPLETED,
    ] {
        assert!(
            ids.iter().any(|id| id == filter_id),
            "frame22 should keep {filter_id} after many compact reuse frames; ids={ids:?}"
        );
    }
    assert!(
        rows.bounds.size.height.0 > 0.0,
        "frame22 rows viewport should keep positive height after many compact reuse frames; rows_bounds={:?}",
        rows.bounds
    );
}

#[test]
fn todo_demo_view_runtime_cache_enable_transition_keeps_footer_filters_after_compact_resize() {
    let mut app = App::new();
    install_demo_theme(&mut app);
    fret_icons_lucide::app::install(&mut app);

    let window = WindowId::default();
    let roomy_bounds = Rect::new(
        Point::new(Px(0.0), Px(0.0)),
        Size::new(Px(560.0), Px(660.0)),
    );
    let compact_bounds = Rect::new(
        Point::new(Px(0.0), Px(0.0)),
        Size::new(Px(420.0), Px(560.0)),
    );
    let mut ui = UiTree::<App>::new();
    ui.set_window(window);
    ui.set_debug_enabled(true);
    let mut services = FakeUiServices;
    let mut state = view_init_window::<TodoDemoView>(&mut app, window);

    let _frame1 = render_todo_demo_runtime_snapshot_for_frame(
        &mut app,
        &mut ui,
        &mut services,
        window,
        roomy_bounds,
        &mut state,
        1,
        2.0,
    );

    // The desktop runner flips view-cache on from the engine-frame hook after the initial
    // render, so replay that order rather than assuming cache was enabled from frame 1.
    ui.set_view_cache_enabled(true);

    let frame2 = render_todo_demo_runtime_snapshot_for_frame(
        &mut app,
        &mut ui,
        &mut services,
        window,
        compact_bounds,
        &mut state,
        2,
        2.0,
    );
    let frame3 = render_todo_demo_runtime_snapshot_for_frame(
        &mut app,
        &mut ui,
        &mut services,
        window,
        compact_bounds,
        &mut state,
        3,
        2.0,
    );

    let mut tracked_frames = vec![(2u64, frame2), (3u64, frame3)];
    for frame_id in 4..=22 {
        tracked_frames.push((
            frame_id,
            render_todo_demo_runtime_snapshot_for_frame(
                &mut app,
                &mut ui,
                &mut services,
                window,
                compact_bounds,
                &mut state,
                frame_id,
                2.0,
            ),
        ));
    }

    let mut failures: Vec<String> = Vec::new();
    for (frame_id, snapshot) in &tracked_frames {
        let label = format!("frame{frame_id}");
        let ids = snapshot_test_ids(snapshot);
        let rows = node_by_test_id(snapshot, TEST_ID_ROWS)
            .expect("rows viewport should exist in runtime compact snapshot");
        for filter_id in [
            TEST_ID_FILTER_ALL,
            TEST_ID_FILTER_ACTIVE,
            TEST_ID_FILTER_COMPLETED,
        ] {
            if !ids.iter().any(|id| id == filter_id) {
                let cache_roots = ui.debug_cache_root_stats();
                let removed = ui.debug_removed_subtrees();
                failures.push(format!(
                        "{label} should keep {filter_id} through runtime cache-enable transition; ids={ids:?}; cache_roots={cache_roots:?}; removed={removed:?}"
                    ));
            }
        }
        if rows.bounds.size.height.0 <= 0.0 {
            failures.push(format!(
                    "{label} rows viewport should keep positive height through runtime cache-enable transition; rows_bounds={:?}",
                    rows.bounds
                ));
        }
    }

    if !failures.is_empty() {
        panic!("{}", failures.join("\n"));
    }
}
