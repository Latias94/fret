use super::*;

use fret_runtime::{
    CommandMeta, CommandScope, InputContext, InputDispatchPhase, WhenExpr,
    WindowCommandActionAvailabilityService, WindowInputArbitrationSnapshot,
    WindowKeyContextStackService, WindowMenuBarFocusService, WindowPointerOcclusion,
    command_is_enabled_for_window_with_input_ctx_fallback,
};
use std::sync::Arc;

fn attributed_plain(text: &str) -> fret_core::AttributedText {
    fret_core::AttributedText::new(
        Arc::<str>::from(text),
        [fret_core::TextSpan {
            len: text.len(),
            ..Default::default()
        }],
    )
}

#[derive(Debug, Default)]
struct AvailabilityLeaf;

impl<H: UiHost> Widget<H> for AvailabilityLeaf {
    fn is_focusable(&self) -> bool {
        true
    }

    fn command_availability(
        &self,
        _cx: &mut crate::widget::CommandAvailabilityCx<'_, H>,
        command: &CommandId,
    ) -> crate::widget::CommandAvailability {
        match command.as_str() {
            "test.available" => crate::widget::CommandAvailability::Available,
            "test.blocked" => crate::widget::CommandAvailability::Blocked,
            _ => crate::widget::CommandAvailability::NotHandled,
        }
    }

    fn layout(&mut self, cx: &mut LayoutCx<'_, H>) -> Size {
        cx.available
    }
}

#[derive(Debug, Default)]
struct FocusableLeaf;

impl<H: UiHost> Widget<H> for FocusableLeaf {
    fn is_focusable(&self) -> bool {
        true
    }

    fn layout(&mut self, cx: &mut LayoutCx<'_, H>) -> Size {
        cx.available
    }
}

#[derive(Debug, Default)]
struct CountingAvailabilityNode;

#[derive(Debug, Default)]
struct CountingAllAvailabilityNode;

#[derive(Debug, Default)]
struct CommandAvailabilityQueryCount {
    count: u32,
}

#[derive(Debug, Default)]
struct CountingNoFocusInterestNode;

#[derive(Debug, Default)]
struct BlockingAvailabilityNode;

impl<H: UiHost> Widget<H> for CountingAvailabilityNode {
    fn hit_test(&self, _bounds: Rect, _position: Point) -> bool {
        true
    }

    fn command_availability(
        &self,
        cx: &mut crate::widget::CommandAvailabilityCx<'_, H>,
        command: &CommandId,
    ) -> crate::widget::CommandAvailability {
        if command.as_str() == "test.available" {
            cx.app.with_global_mut_untracked(
                CommandAvailabilityQueryCount::default,
                |counter, _app| {
                    counter.count = counter.count.saturating_add(1);
                },
            );
            return crate::widget::CommandAvailability::Available;
        }
        crate::widget::CommandAvailability::NotHandled
    }

    fn layout(&mut self, cx: &mut LayoutCx<'_, H>) -> Size {
        cx.available
    }
}

impl<H: UiHost> Widget<H> for CountingAllAvailabilityNode {
    fn hit_test(&self, _bounds: Rect, _position: Point) -> bool {
        true
    }

    fn command_availability(
        &self,
        cx: &mut crate::widget::CommandAvailabilityCx<'_, H>,
        command: &CommandId,
    ) -> crate::widget::CommandAvailability {
        cx.app.with_global_mut_untracked(
            CommandAvailabilityQueryCount::default,
            |counter, _app| {
                counter.count = counter.count.saturating_add(1);
            },
        );
        if command.as_str() == "test.available" {
            return crate::widget::CommandAvailability::Available;
        }
        crate::widget::CommandAvailability::NotHandled
    }

    fn layout(&mut self, cx: &mut LayoutCx<'_, H>) -> Size {
        cx.available
    }
}

impl<H: UiHost> Widget<H> for CountingNoFocusInterestNode {
    fn hit_test(&self, _bounds: Rect, _position: Point) -> bool {
        true
    }

    fn command_availability(
        &self,
        cx: &mut crate::widget::CommandAvailabilityCx<'_, H>,
        command: &CommandId,
    ) -> crate::widget::CommandAvailability {
        if command.as_str() == "test.available" {
            cx.app.with_global_mut_untracked(
                CommandAvailabilityQueryCount::default,
                |counter, _app| {
                    counter.count = counter.count.saturating_add(1);
                },
            );
            return crate::widget::CommandAvailability::Available;
        }
        crate::widget::CommandAvailability::NotHandled
    }

    fn layout(&mut self, cx: &mut LayoutCx<'_, H>) -> Size {
        cx.available
    }
}

impl<H: UiHost> Widget<H> for BlockingAvailabilityNode {
    fn hit_test(&self, _bounds: Rect, _position: Point) -> bool {
        true
    }

    fn command_availability(
        &self,
        _cx: &mut crate::widget::CommandAvailabilityCx<'_, H>,
        command: &CommandId,
    ) -> crate::widget::CommandAvailability {
        if command.as_str() == "test.available" {
            return crate::widget::CommandAvailability::Blocked;
        }
        crate::widget::CommandAvailability::NotHandled
    }

    fn layout(&mut self, cx: &mut LayoutCx<'_, H>) -> Size {
        cx.available
    }
}

fn widget_command_meta(title: &str) -> CommandMeta {
    CommandMeta::new(title).with_scope(CommandScope::Widget)
}

fn snapshot_input_ctx(
    ui: &mut UiTree<crate::test_host::TestHost>,
    app: &mut crate::test_host::TestHost,
) -> InputContext {
    let caps = app
        .global::<PlatformCapabilities>()
        .cloned()
        .unwrap_or_default();

    InputContext {
        platform: Platform::current(),
        caps,
        ui_has_modal: false,
        window_arbitration: None,
        focus_is_text_input: ui.focus_is_text_input(app),
        text_boundary_mode: fret_runtime::TextBoundaryMode::UnicodeWord,
        edit_can_undo: true,
        edit_can_redo: true,
        router_can_back: false,
        router_can_forward: false,
        dispatch_phase: InputDispatchPhase::Bubble,
    }
}

fn publish_snapshot(
    ui: &mut UiTree<crate::test_host::TestHost>,
    app: &mut crate::test_host::TestHost,
    window: AppWindowId,
) {
    let input_ctx = snapshot_input_ctx(ui, app);

    ui.publish_window_command_action_availability_snapshot(app, &input_ctx);

    assert!(
        app.global::<WindowCommandActionAvailabilityService>()
            .and_then(|svc| svc.snapshot(window))
            .is_some()
    );
}

fn publish_filtered_snapshot(
    ui: &mut UiTree<crate::test_host::TestHost>,
    app: &mut crate::test_host::TestHost,
    window: AppWindowId,
    commands: impl IntoIterator<Item = CommandId>,
) {
    let input_ctx = snapshot_input_ctx(ui, app);

    ui.publish_window_command_action_availability_snapshot_filtered(app, &input_ctx, commands);

    assert!(
        app.global::<WindowCommandActionAvailabilityService>()
            .and_then(|svc| svc.snapshot(window))
            .is_some()
    );
}

fn publish_snapshot_with_input_ctx(
    ui: &mut UiTree<crate::test_host::TestHost>,
    app: &mut crate::test_host::TestHost,
    window: AppWindowId,
    input_ctx: InputContext,
) {
    ui.publish_window_command_action_availability_snapshot(app, &input_ctx);

    assert!(
        app.global::<WindowCommandActionAvailabilityService>()
            .and_then(|svc| svc.snapshot(window))
            .is_some()
    );
}

#[test]
fn action_availability_snapshot_caches_declarative_interest_within_publication() {
    let mut app = crate::test_host::TestHost::new();
    app.set_global(PlatformCapabilities::default());

    let window = AppWindowId::default();
    app.register_command(
        CommandId::from("test.first_unhandled"),
        widget_command_meta("First Unhandled"),
    );
    app.register_command(
        CommandId::from("test.second_unhandled"),
        widget_command_meta("Second Unhandled"),
    );

    let mut ui: UiTree<crate::test_host::TestHost> = UiTree::new();
    ui.set_window(window);

    let root_element = crate::elements::GlobalElementId(0xCA1);
    let leaf_element = crate::elements::GlobalElementId(0xCA2);
    let root = ui.create_node_for_element(root_element, TestStack);
    let leaf = ui.create_node_for_element(leaf_element, TestStack);
    ui.set_root(root);
    ui.add_child(root, leaf);
    ui.set_focus(Some(leaf));

    crate::declarative::frame::with_window_frame_mut(&mut app, window, |window_frame| {
        for (node, element) in [(root, root_element), (leaf, leaf_element)] {
            window_frame.instances.insert(
                node,
                crate::declarative::frame::ElementRecord {
                    element,
                    instance: crate::declarative::frame::ElementInstance::Stack(
                        crate::element::StackProps::default(),
                    ),
                    inherited_foreground: None,
                    inherited_text_style: None,
                    semantics_decoration: None,
                    key_context: None,
                    layout_direction: fret_core::LayoutDirection::default(),
                },
            );
        }
    });

    let mut services = FakeUiServices;
    let bounds = Rect::new(Point::new(Px(0.0), Px(0.0)), Size::new(Px(100.0), Px(40.0)));
    ui.layout_all(&mut app, &mut services, bounds, 1.0);

    crate::tree::reset_command_availability_interest_probe_count();
    publish_snapshot(&mut ui, &mut app, window);

    assert_eq!(
        crate::tree::take_command_availability_interest_probe_count(),
        2,
        "root and leaf interest should be profiled once each and then reused for all commands in the same snapshot publication"
    );
}

#[test]
fn action_availability_snapshot_reuses_declarative_interest_across_same_frame_refine() {
    let mut app = crate::test_host::TestHost::new();
    app.set_global(PlatformCapabilities::default());

    let window = AppWindowId::default();
    app.register_command(
        CommandId::from("test.first_unhandled"),
        widget_command_meta("First Unhandled"),
    );
    app.register_command(
        CommandId::from("test.second_unhandled"),
        widget_command_meta("Second Unhandled"),
    );

    let mut ui: UiTree<crate::test_host::TestHost> = UiTree::new();
    ui.set_window(window);

    let root_element = crate::elements::GlobalElementId(0xCB1);
    let leaf_element = crate::elements::GlobalElementId(0xCB2);
    let root = ui.create_node_for_element(root_element, TestStack);
    let leaf = ui.create_node_for_element(leaf_element, TestStack);
    ui.set_root(root);
    ui.add_child(root, leaf);
    ui.set_focus(Some(leaf));

    crate::declarative::frame::with_window_frame_mut(&mut app, window, |window_frame| {
        for (node, element) in [(root, root_element), (leaf, leaf_element)] {
            window_frame.instances.insert(
                node,
                crate::declarative::frame::ElementRecord {
                    element,
                    instance: crate::declarative::frame::ElementInstance::Stack(
                        crate::element::StackProps::default(),
                    ),
                    inherited_foreground: None,
                    inherited_text_style: None,
                    semantics_decoration: None,
                    key_context: None,
                    layout_direction: fret_core::LayoutDirection::default(),
                },
            );
        }
    });

    let mut services = FakeUiServices;
    let bounds = Rect::new(Point::new(Px(0.0), Px(0.0)), Size::new(Px(100.0), Px(40.0)));
    ui.layout_all(&mut app, &mut services, bounds, 1.0);

    crate::tree::reset_command_availability_interest_probe_count();
    publish_snapshot(&mut ui, &mut app, window);
    assert_eq!(
        crate::tree::take_command_availability_interest_probe_count(),
        2
    );

    ui.pending_post_layout_window_runtime_snapshot_refine = true;
    crate::tree::reset_command_availability_interest_probe_count();
    publish_snapshot(&mut ui, &mut app, window);
    assert_eq!(
        crate::tree::take_command_availability_interest_probe_count(),
        0,
        "same-frame forced publication should reuse cached declarative command-interest metadata"
    );

    ui.invalidate_with_detail(
        leaf,
        Invalidation::Layout,
        UiDebugInvalidationDetail::LocalInvalidation,
    );
    crate::tree::reset_command_availability_interest_probe_count();
    publish_snapshot(&mut ui, &mut app, window);
    assert_eq!(
        crate::tree::take_command_availability_interest_probe_count(),
        2,
        "command availability revision changes must invalidate cached command-interest metadata"
    );
}

#[test]
fn action_availability_snapshot_keeps_interest_cache_for_scroll_hit_test_only_invalidation() {
    let mut app = crate::test_host::TestHost::new();
    app.set_global(PlatformCapabilities::default());

    let window = AppWindowId::default();
    app.register_command(
        CommandId::from("test.first_unhandled"),
        widget_command_meta("First Unhandled"),
    );
    app.register_command(
        CommandId::from("test.second_unhandled"),
        widget_command_meta("Second Unhandled"),
    );

    let mut ui: UiTree<crate::test_host::TestHost> = UiTree::new();
    ui.set_window(window);

    let root_element = crate::elements::GlobalElementId(0xCC1);
    let leaf_element = crate::elements::GlobalElementId(0xCC2);
    let root = ui.create_node_for_element(root_element, TestStack);
    let leaf = ui.create_node_for_element(leaf_element, TestStack);
    ui.set_root(root);
    ui.add_child(root, leaf);
    ui.set_focus(Some(leaf));

    crate::declarative::frame::with_window_frame_mut(&mut app, window, |window_frame| {
        for (node, element) in [(root, root_element), (leaf, leaf_element)] {
            window_frame.instances.insert(
                node,
                crate::declarative::frame::ElementRecord {
                    element,
                    instance: crate::declarative::frame::ElementInstance::Stack(
                        crate::element::StackProps::default(),
                    ),
                    inherited_foreground: None,
                    inherited_text_style: None,
                    semantics_decoration: None,
                    key_context: None,
                    layout_direction: fret_core::LayoutDirection::default(),
                },
            );
        }
    });

    let mut services = FakeUiServices;
    let bounds = Rect::new(Point::new(Px(0.0), Px(0.0)), Size::new(Px(100.0), Px(40.0)));
    ui.layout_all(&mut app, &mut services, bounds, 1.0);

    crate::tree::reset_command_availability_interest_probe_count();
    publish_snapshot(&mut ui, &mut app, window);
    assert_eq!(
        crate::tree::take_command_availability_interest_probe_count(),
        2
    );

    ui.invalidate_with_detail(
        leaf,
        Invalidation::HitTestOnly,
        UiDebugInvalidationDetail::ScrollHandleHitTestOnly,
    );
    ui.pending_post_layout_window_runtime_snapshot_refine = true;
    crate::tree::reset_command_availability_interest_probe_count();
    publish_snapshot(&mut ui, &mut app, window);
    assert_eq!(
        crate::tree::take_command_availability_interest_probe_count(),
        0,
        "scroll hit-test-only invalidations should not reset command-interest metadata"
    );

    ui.invalidate_with_detail(
        leaf,
        Invalidation::Layout,
        UiDebugInvalidationDetail::LocalInvalidation,
    );
    ui.pending_post_layout_window_runtime_snapshot_refine = true;
    crate::tree::reset_command_availability_interest_probe_count();
    publish_snapshot(&mut ui, &mut app, window);
    assert_eq!(
        crate::tree::take_command_availability_interest_probe_count(),
        2,
        "layout-sensitive invalidations should still reset command-interest metadata"
    );
}

#[test]
fn action_availability_snapshot_marks_unhandled_commands_unavailable() {
    let mut app = crate::test_host::TestHost::new();
    app.set_global(PlatformCapabilities::default());

    let window = AppWindowId::default();
    app.register_command(
        CommandId::from("test.available"),
        widget_command_meta("Available"),
    );
    app.register_command(
        CommandId::from("test.blocked"),
        widget_command_meta("Blocked"),
    );
    app.register_command(
        CommandId::from("test.unhandled"),
        widget_command_meta("Unhandled"),
    );

    let mut ui: UiTree<crate::test_host::TestHost> = UiTree::new();
    ui.set_window(window);

    let root = ui.create_node(TestStack);
    let leaf = ui.create_node(AvailabilityLeaf);
    ui.set_root(root);
    ui.add_child(root, leaf);
    ui.set_focus(Some(leaf));

    let mut services = FakeUiServices;
    let bounds = Rect::new(Point::new(Px(0.0), Px(0.0)), Size::new(Px(100.0), Px(40.0)));
    ui.layout_all(&mut app, &mut services, bounds, 1.0);

    publish_snapshot(&mut ui, &mut app, window);

    let svc = app
        .global::<WindowCommandActionAvailabilityService>()
        .expect("action availability service");
    assert_eq!(
        svc.available(window, &CommandId::from("test.available")),
        Some(true)
    );
    assert_eq!(
        svc.available(window, &CommandId::from("test.blocked")),
        Some(false)
    );
    assert_eq!(
        svc.available(window, &CommandId::from("test.unhandled")),
        Some(false)
    );
}

#[test]
fn action_availability_filtered_snapshot_publishes_only_requested_widget_commands() {
    let mut app = crate::test_host::TestHost::new();
    app.set_global(PlatformCapabilities::default());

    let window = AppWindowId::default();
    app.register_command(
        CommandId::from("test.available"),
        widget_command_meta("Available"),
    );
    app.register_command(
        CommandId::from("test.unhandled"),
        widget_command_meta("Unhandled"),
    );
    app.register_command(
        CommandId::from("test.window_scope"),
        CommandMeta::new("Window Scope"),
    );

    let mut ui: UiTree<crate::test_host::TestHost> = UiTree::new();
    ui.set_window(window);

    let root = ui.create_node(CountingAllAvailabilityNode);
    ui.set_root(root);
    ui.set_focus(Some(root));

    let mut services = FakeUiServices;
    let bounds = Rect::new(Point::new(Px(0.0), Px(0.0)), Size::new(Px(100.0), Px(40.0)));
    ui.layout_all(&mut app, &mut services, bounds, 1.0);

    publish_filtered_snapshot(
        &mut ui,
        &mut app,
        window,
        [
            CommandId::from("test.unhandled"),
            CommandId::from("test.available"),
            CommandId::from("test.unhandled"),
            CommandId::from("test.window_scope"),
            CommandId::from("test.missing"),
        ],
    );

    let eval_count = app
        .global::<CommandAvailabilityQueryCount>()
        .map(|counter| counter.count)
        .unwrap_or(0);
    assert_eq!(
        eval_count, 2,
        "filtered publication should evaluate each requested registered widget command once"
    );

    let svc = app
        .global::<WindowCommandActionAvailabilityService>()
        .expect("action availability service");
    assert_eq!(
        svc.available(window, &CommandId::from("test.available")),
        Some(true)
    );
    assert_eq!(
        svc.available(window, &CommandId::from("test.unhandled")),
        Some(false)
    );
    assert_eq!(
        svc.available(window, &CommandId::from("test.window_scope")),
        None,
        "non-widget commands are not action-availability entries"
    );
    assert_eq!(
        svc.available(window, &CommandId::from("test.missing")),
        None,
        "unregistered commands are unknown rather than disabled"
    );
}

#[test]
fn action_availability_filtered_snapshot_signature_dedupes_sorted_command_set() {
    let mut app = crate::test_host::TestHost::new();
    app.set_global(PlatformCapabilities::default());

    let window = AppWindowId::default();
    app.register_command(
        CommandId::from("test.available"),
        widget_command_meta("Available"),
    );
    app.register_command(CommandId::from("test.other"), widget_command_meta("Other"));

    let mut ui: UiTree<crate::test_host::TestHost> = UiTree::new();
    ui.set_window(window);

    let root = ui.create_node(CountingAllAvailabilityNode);
    ui.set_root(root);
    ui.set_focus(Some(root));

    let mut services = FakeUiServices;
    let bounds = Rect::new(Point::new(Px(0.0), Px(0.0)), Size::new(Px(100.0), Px(40.0)));
    ui.layout_all(&mut app, &mut services, bounds, 1.0);

    publish_filtered_snapshot(
        &mut ui,
        &mut app,
        window,
        [
            CommandId::from("test.other"),
            CommandId::from("test.available"),
            CommandId::from("test.other"),
        ],
    );
    let first_count = app
        .global::<CommandAvailabilityQueryCount>()
        .map(|counter| counter.count)
        .unwrap_or(0);
    assert_eq!(first_count, 2);

    publish_filtered_snapshot(
        &mut ui,
        &mut app,
        window,
        [
            CommandId::from("test.available"),
            CommandId::from("test.other"),
        ],
    );
    let second_count = app
        .global::<CommandAvailabilityQueryCount>()
        .map(|counter| counter.count)
        .unwrap_or(0);
    assert_eq!(
        second_count, first_count,
        "same filtered command set should skip duplicate same-frame publication even when caller order changes"
    );

    publish_filtered_snapshot(
        &mut ui,
        &mut app,
        window,
        [CommandId::from("test.available")],
    );
    let third_count = app
        .global::<CommandAvailabilityQueryCount>()
        .map(|counter| counter.count)
        .unwrap_or(0);
    assert_eq!(
        third_count,
        first_count + 1,
        "changing the filtered command set should publish a new snapshot"
    );
}

#[test]
fn runtime_snapshot_uses_full_action_availability_when_no_surface_declares_demand() {
    let mut app = crate::test_host::TestHost::new();
    app.set_global(PlatformCapabilities::default());

    let window = AppWindowId::default();
    app.register_command(
        CommandId::from("test.available"),
        widget_command_meta("Available"),
    );
    app.register_command(CommandId::from("test.other"), widget_command_meta("Other"));

    let mut ui: UiTree<crate::test_host::TestHost> = UiTree::new();
    ui.set_window(window);

    let root = ui.create_node(CountingAllAvailabilityNode);
    ui.set_root(root);
    ui.set_focus(Some(root));

    let mut services = FakeUiServices;
    let bounds = Rect::new(Point::new(Px(0.0), Px(0.0)), Size::new(Px(100.0), Px(40.0)));
    ui.layout_all(&mut app, &mut services, bounds, 1.0);

    ui.publish_window_runtime_snapshots(&mut app);

    let eval_count = app
        .global::<CommandAvailabilityQueryCount>()
        .map(|counter| counter.count)
        .unwrap_or(0);
    assert_eq!(
        eval_count, 2,
        "the default runtime snapshot publisher must stay conservative until a surface declares demand"
    );

    let svc = app
        .global::<WindowCommandActionAvailabilityService>()
        .expect("action availability service");
    assert_eq!(
        svc.available(window, &CommandId::from("test.available")),
        Some(true)
    );
    assert_eq!(
        svc.available(window, &CommandId::from("test.other")),
        Some(false)
    );
}

#[test]
fn runtime_snapshot_uses_filtered_action_availability_for_surface_demand() {
    let mut app = crate::test_host::TestHost::new();
    app.set_global(PlatformCapabilities::default());

    let window = AppWindowId::default();
    app.register_command(
        CommandId::from("test.available"),
        widget_command_meta("Available"),
    );
    app.register_command(CommandId::from("test.other"), widget_command_meta("Other"));

    let mut ui: UiTree<crate::test_host::TestHost> = UiTree::new();
    ui.set_window(window);

    let root = ui.create_node(CountingAllAvailabilityNode);
    ui.set_root(root);
    ui.set_focus(Some(root));

    let mut services = FakeUiServices;
    let bounds = Rect::new(Point::new(Px(0.0), Px(0.0)), Size::new(Px(100.0), Px(40.0)));
    ui.layout_all(&mut app, &mut services, bounds, 1.0);

    crate::elements::with_window_state(&mut app, window, |state| {
        state.request_command_action_availability_for_commands([
            CommandId::from("test.available"),
            CommandId::from("test.available"),
        ]);
    });
    ui.publish_window_runtime_snapshots(&mut app);

    let eval_count = app
        .global::<CommandAvailabilityQueryCount>()
        .map(|counter| counter.count)
        .unwrap_or(0);
    assert_eq!(
        eval_count, 1,
        "declared surface demand should evaluate only the requested widget command set"
    );

    let svc = app
        .global::<WindowCommandActionAvailabilityService>()
        .expect("action availability service");
    assert_eq!(
        svc.available(window, &CommandId::from("test.available")),
        Some(true)
    );
    assert_eq!(
        svc.available(window, &CommandId::from("test.other")),
        None,
        "commands outside the declared surface demand remain unknown rather than disabled"
    );
}

#[test]
fn runtime_snapshot_all_surface_demand_wins_over_filtered_demand() {
    let mut app = crate::test_host::TestHost::new();
    app.set_global(PlatformCapabilities::default());

    let window = AppWindowId::default();
    app.register_command(
        CommandId::from("test.available"),
        widget_command_meta("Available"),
    );
    app.register_command(CommandId::from("test.other"), widget_command_meta("Other"));

    let mut ui: UiTree<crate::test_host::TestHost> = UiTree::new();
    ui.set_window(window);

    let root = ui.create_node(CountingAllAvailabilityNode);
    ui.set_root(root);
    ui.set_focus(Some(root));

    let mut services = FakeUiServices;
    let bounds = Rect::new(Point::new(Px(0.0), Px(0.0)), Size::new(Px(100.0), Px(40.0)));
    ui.layout_all(&mut app, &mut services, bounds, 1.0);

    crate::elements::with_window_state(&mut app, window, |state| {
        state.request_command_action_availability_for_commands([CommandId::from("test.available")]);
        state.request_all_command_action_availability();
    });
    ui.publish_window_runtime_snapshots(&mut app);

    let eval_count = app
        .global::<CommandAvailabilityQueryCount>()
        .map(|counter| counter.count)
        .unwrap_or(0);
    assert_eq!(
        eval_count, 2,
        "a complete host-command surface must keep the publisher in full conservative mode"
    );

    let svc = app
        .global::<WindowCommandActionAvailabilityService>()
        .expect("action availability service");
    assert_eq!(
        svc.available(window, &CommandId::from("test.available")),
        Some(true)
    );
    assert_eq!(
        svc.available(window, &CommandId::from("test.other")),
        Some(false)
    );
}

#[test]
fn action_availability_snapshot_publishes_focus_traversal_gating() {
    let mut app = crate::test_host::TestHost::new();
    app.set_global(PlatformCapabilities::default());

    let window = AppWindowId::default();
    app.register_command(
        CommandId::from("focus.next"),
        widget_command_meta("Focus Next"),
    );
    app.register_command(
        CommandId::from("focus.previous"),
        widget_command_meta("Focus Previous"),
    );

    let mut ui: UiTree<crate::test_host::TestHost> = UiTree::new();
    ui.set_window(window);

    let root = ui.create_node(TestStack);
    ui.set_root(root);

    let mut services = FakeUiServices;
    let bounds = Rect::new(Point::new(Px(0.0), Px(0.0)), Size::new(Px(100.0), Px(40.0)));
    ui.layout_all(&mut app, &mut services, bounds, 1.0);

    publish_snapshot(&mut ui, &mut app, window);

    let svc = app
        .global::<WindowCommandActionAvailabilityService>()
        .expect("action availability service");
    assert_eq!(
        svc.available(window, &CommandId::from("focus.next")),
        Some(false)
    );
    assert_eq!(
        svc.available(window, &CommandId::from("focus.previous")),
        Some(false)
    );

    let leaf = ui.create_node(FocusableLeaf);
    ui.add_child(root, leaf);
    ui.layout_all(&mut app, &mut services, bounds, 1.0);
    publish_snapshot(&mut ui, &mut app, window);

    let svc = app
        .global::<WindowCommandActionAvailabilityService>()
        .expect("action availability service");
    assert_eq!(
        svc.available(window, &CommandId::from("focus.next")),
        Some(true)
    );
    assert_eq!(
        svc.available(window, &CommandId::from("focus.previous")),
        Some(true)
    );
}

#[test]
fn action_availability_snapshot_reuses_focus_traversal_within_frame() {
    let mut app = crate::test_host::TestHost::new();
    app.set_global(PlatformCapabilities::default());

    let window = AppWindowId::default();
    app.register_command(
        CommandId::from("focus.next"),
        widget_command_meta("Focus Next"),
    );
    app.register_command(
        CommandId::from("focus.previous"),
        widget_command_meta("Focus Previous"),
    );

    let mut ui: UiTree<crate::test_host::TestHost> = UiTree::new();
    ui.set_debug_enabled(true);
    ui.set_window(window);

    let root = ui.create_node(TestStack);
    let leaf = ui.create_node(FocusableLeaf);
    ui.set_root(root);
    ui.add_child(root, leaf);

    let mut services = FakeUiServices;
    let bounds = Rect::new(Point::new(Px(0.0), Px(0.0)), Size::new(Px(100.0), Px(40.0)));
    ui.layout_all(&mut app, &mut services, bounds, 1.0);

    app.advance_frame();
    ui.begin_debug_frame_if_needed(app.frame_id());
    publish_snapshot(&mut ui, &mut app, window);
    assert_eq!(
        ui.debug_command_availability_hotspots()
            .iter()
            .filter(|hotspot| hotspot.route == "focus_traversal_snapshot")
            .count(),
        1,
        "focus.next/focus.previous should share one traversal query inside a publication"
    );

    ui.pending_post_layout_window_runtime_snapshot_refine = true;
    publish_snapshot(&mut ui, &mut app, window);
    assert_eq!(
        ui.debug_command_availability_hotspots()
            .iter()
            .filter(|hotspot| hotspot.route == "focus_traversal_snapshot")
            .count(),
        1,
        "same-frame re-publication should reuse the frame-level traversal availability cache"
    );

    let svc = app
        .global::<WindowCommandActionAvailabilityService>()
        .expect("action availability service");
    assert_eq!(
        svc.available(window, &CommandId::from("focus.next")),
        Some(true)
    );
    assert_eq!(
        svc.available(window, &CommandId::from("focus.previous")),
        Some(true)
    );
}

#[test]
fn action_availability_snapshot_refreshes_focus_traversal_on_next_frame() {
    let mut app = crate::test_host::TestHost::new();
    app.set_global(PlatformCapabilities::default());

    let window = AppWindowId::default();
    app.register_command(
        CommandId::from("focus.next"),
        widget_command_meta("Focus Next"),
    );
    app.register_command(
        CommandId::from("focus.previous"),
        widget_command_meta("Focus Previous"),
    );

    let mut ui: UiTree<crate::test_host::TestHost> = UiTree::new();
    ui.set_debug_enabled(true);
    ui.set_window(window);

    let root = ui.create_node(TestStack);
    let leaf = ui.create_node(FocusableLeaf);
    ui.set_root(root);
    ui.add_child(root, leaf);

    let mut services = FakeUiServices;
    let bounds = Rect::new(Point::new(Px(0.0), Px(0.0)), Size::new(Px(100.0), Px(40.0)));
    ui.layout_all(&mut app, &mut services, bounds, 1.0);

    app.advance_frame();
    ui.begin_debug_frame_if_needed(app.frame_id());
    ui.pending_post_layout_window_runtime_snapshot_refine = true;
    publish_snapshot(&mut ui, &mut app, window);
    assert_eq!(
        ui.debug_command_availability_hotspots()
            .iter()
            .filter(|hotspot| hotspot.route == "focus_traversal_snapshot")
            .count(),
        1
    );

    app.advance_frame();
    ui.begin_debug_frame_if_needed(app.frame_id());
    ui.pending_post_layout_window_runtime_snapshot_refine = true;
    publish_snapshot(&mut ui, &mut app, window);

    assert_eq!(
        ui.debug_command_availability_hotspots()
            .iter()
            .filter(|hotspot| hotspot.route == "focus_traversal_snapshot")
            .count(),
        1,
        "new frames should compute their own traversal availability instead of reusing stale cache"
    );
}

#[test]
fn action_availability_snapshot_skips_recompute_when_inputs_are_unchanged() {
    let mut app = crate::test_host::TestHost::new();
    app.set_global(PlatformCapabilities::default());

    let window = AppWindowId::default();
    app.register_command(
        CommandId::from("test.available"),
        widget_command_meta("Available"),
    );

    let mut ui: UiTree<crate::test_host::TestHost> = UiTree::new();
    ui.set_window(window);

    let root = ui.create_node(CountingAvailabilityNode);
    let leaf_a = ui.create_node(FocusableLeaf);
    let leaf_b = ui.create_node(FocusableLeaf);
    ui.set_root(root);
    ui.add_child(root, leaf_a);
    ui.add_child(root, leaf_b);
    ui.set_focus(Some(leaf_a));

    let mut services = FakeUiServices;
    let bounds = Rect::new(Point::new(Px(0.0), Px(0.0)), Size::new(Px(100.0), Px(40.0)));
    ui.layout_all(&mut app, &mut services, bounds, 1.0);

    publish_snapshot(&mut ui, &mut app, window);
    let first_count = app
        .global::<CommandAvailabilityQueryCount>()
        .map(|counter| counter.count)
        .unwrap_or(0);
    assert_eq!(first_count, 1);

    publish_snapshot(&mut ui, &mut app, window);
    let second_count = app
        .global::<CommandAvailabilityQueryCount>()
        .map(|counter| counter.count)
        .unwrap_or(0);
    assert_eq!(second_count, 1);

    ui.invalidate_with_detail(
        root,
        Invalidation::Paint,
        UiDebugInvalidationDetail::AnimationFrameRequest,
    );
    publish_snapshot(&mut ui, &mut app, window);
    let third_count = app
        .global::<CommandAvailabilityQueryCount>()
        .map(|counter| counter.count)
        .unwrap_or(0);
    assert_eq!(third_count, 1);

    ui.set_focus(Some(leaf_b));
    publish_snapshot(&mut ui, &mut app, window);
    let fourth_count = app
        .global::<CommandAvailabilityQueryCount>()
        .map(|counter| counter.count)
        .unwrap_or(0);
    assert_eq!(fourth_count, 2);
}

#[test]
fn action_availability_snapshot_dedupes_same_pending_refine_but_post_layout_republishes() {
    let mut app = crate::test_host::TestHost::new();
    app.set_global(PlatformCapabilities::default());

    let window = AppWindowId::default();
    app.register_command(
        CommandId::from("test.available"),
        widget_command_meta("Available"),
    );

    let mut ui: UiTree<crate::test_host::TestHost> = UiTree::new();
    ui.set_window(window);

    let root = ui.create_node(CountingAvailabilityNode);
    ui.set_root(root);
    ui.set_focus(Some(root));

    let mut services = FakeUiServices;
    let bounds = Rect::new(Point::new(Px(0.0), Px(0.0)), Size::new(Px(100.0), Px(40.0)));
    ui.layout_all(&mut app, &mut services, bounds, 1.0);

    publish_snapshot(&mut ui, &mut app, window);
    let first_count = app
        .global::<CommandAvailabilityQueryCount>()
        .map(|counter| counter.count)
        .unwrap_or(0);
    assert_eq!(first_count, 1);

    ui.pending_post_layout_window_runtime_snapshot_refine = true;
    publish_snapshot(&mut ui, &mut app, window);
    let pending_count = app
        .global::<CommandAvailabilityQueryCount>()
        .map(|counter| counter.count)
        .unwrap_or(0);
    assert_eq!(
        pending_count, 2,
        "entering a pending post-layout refine state should publish a fresh interim snapshot"
    );

    publish_snapshot(&mut ui, &mut app, window);
    let duplicate_pending_count = app
        .global::<CommandAvailabilityQueryCount>()
        .map(|counter| counter.count)
        .unwrap_or(0);
    assert_eq!(
        duplicate_pending_count, pending_count,
        "same-frame duplicate publishes with the same pending-refine signature should be skipped"
    );

    ui.refine_pending_window_runtime_snapshots_after_layout(&mut app);
    let post_layout_count = app
        .global::<CommandAvailabilityQueryCount>()
        .map(|counter| counter.count)
        .unwrap_or(0);
    assert_eq!(
        post_layout_count,
        pending_count + 1,
        "post-layout refine should still force the authoritative final snapshot"
    );
}

#[test]
fn action_availability_snapshot_ignores_pointer_arbitration_only_changes() {
    let mut app = crate::test_host::TestHost::new();
    app.set_global(PlatformCapabilities::default());

    let window = AppWindowId::default();
    app.register_command(
        CommandId::from("test.available"),
        widget_command_meta("Available"),
    );

    let mut ui: UiTree<crate::test_host::TestHost> = UiTree::new();
    ui.set_window(window);

    let root = ui.create_node(CountingAvailabilityNode);
    let leaf = ui.create_node(FocusableLeaf);
    ui.set_root(root);
    ui.add_child(root, leaf);
    ui.set_focus(Some(leaf));

    let mut services = FakeUiServices;
    let bounds = Rect::new(Point::new(Px(0.0), Px(0.0)), Size::new(Px(100.0), Px(40.0)));
    ui.layout_all(&mut app, &mut services, bounds, 1.0);

    let mut input_ctx = InputContext {
        platform: Platform::current(),
        caps: app
            .global::<PlatformCapabilities>()
            .cloned()
            .unwrap_or_default(),
        ui_has_modal: false,
        window_arbitration: None,
        focus_is_text_input: ui.focus_is_text_input(&mut app),
        text_boundary_mode: fret_runtime::TextBoundaryMode::UnicodeWord,
        edit_can_undo: true,
        edit_can_redo: true,
        router_can_back: false,
        router_can_forward: false,
        dispatch_phase: InputDispatchPhase::Bubble,
    };

    publish_snapshot_with_input_ctx(&mut ui, &mut app, window, input_ctx.clone());
    let first_count = app
        .global::<CommandAvailabilityQueryCount>()
        .map(|counter| counter.count)
        .unwrap_or(0);
    assert_eq!(first_count, 1);

    input_ctx.window_arbitration = Some(WindowInputArbitrationSnapshot {
        modal_barrier_root: Some(root),
        focus_barrier_root: Some(root),
        pointer_occlusion: WindowPointerOcclusion::BlockMouse,
        pointer_occlusion_root: Some(root),
        pointer_capture_active: true,
        pointer_capture_root: Some(root),
        pointer_capture_multiple_roots: false,
    });
    publish_snapshot_with_input_ctx(&mut ui, &mut app, window, input_ctx);
    let second_count = app
        .global::<CommandAvailabilityQueryCount>()
        .map(|counter| counter.count)
        .unwrap_or(0);
    assert_eq!(second_count, 1);
}

#[test]
fn pointer_move_publishes_input_context_without_command_availability_recompute() {
    let mut app = crate::test_host::TestHost::new();
    app.set_global(PlatformCapabilities::default());

    let window = AppWindowId::default();
    app.register_command(
        CommandId::from("test.available"),
        widget_command_meta("Available"),
    );

    let mut ui: UiTree<crate::test_host::TestHost> = UiTree::new();
    ui.set_window(window);

    let root = ui.create_node(CountingAvailabilityNode);
    ui.set_root(root);
    ui.set_focus(Some(root));

    let mut services = FakeUiServices;
    let bounds = Rect::new(Point::new(Px(0.0), Px(0.0)), Size::new(Px(100.0), Px(40.0)));
    ui.layout_all(&mut app, &mut services, bounds, 1.0);

    publish_snapshot(&mut ui, &mut app, window);
    let baseline_count = app
        .global::<CommandAvailabilityQueryCount>()
        .map(|counter| counter.count)
        .unwrap_or(0);
    assert_eq!(baseline_count, 1);

    ui.dispatch_event(
        &mut app,
        &mut services,
        &Event::Pointer(PointerEvent::Move {
            position: Point::new(Px(10.0), Px(10.0)),
            buttons: fret_core::MouseButtons::default(),
            modifiers: fret_core::Modifiers::default(),
            pointer_id: fret_core::PointerId(0),
            pointer_type: fret_core::PointerType::Mouse,
        }),
    );

    let post_move_count = app
        .global::<CommandAvailabilityQueryCount>()
        .map(|counter| counter.count)
        .unwrap_or(0);
    assert_eq!(
        post_move_count, baseline_count,
        "pointer move should refresh input context without republishing widget command availability"
    );

    assert!(
        app.global::<fret_runtime::WindowInputContextService>()
            .and_then(|svc| svc.snapshot(window))
            .is_some(),
        "pointer move should still publish the latest window input context"
    );

    let svc = app
        .global::<WindowCommandActionAvailabilityService>()
        .expect("action availability service");
    assert_eq!(
        svc.available(window, &CommandId::from("test.available")),
        Some(true)
    );
}

#[test]
fn action_availability_snapshot_does_not_scan_unfocused_subtree() {
    let mut app = crate::test_host::TestHost::new();
    app.set_global(PlatformCapabilities::default());

    let window = AppWindowId::default();
    app.register_command(
        CommandId::from("test.available"),
        widget_command_meta("Available"),
    );

    let mut ui: UiTree<crate::test_host::TestHost> = UiTree::new();
    ui.set_window(window);

    let root = ui.create_node(TestStack);
    let focused = ui.create_node(FocusableLeaf);
    let unfocused_sibling = ui.create_node(CountingAvailabilityNode);
    ui.set_root(root);
    ui.add_child(root, focused);
    ui.add_child(root, unfocused_sibling);
    ui.set_focus(Some(focused));

    let mut services = FakeUiServices;
    let bounds = Rect::new(Point::new(Px(0.0), Px(0.0)), Size::new(Px(100.0), Px(40.0)));
    ui.layout_all(&mut app, &mut services, bounds, 1.0);

    publish_snapshot(&mut ui, &mut app, window);

    let query_count = app
        .global::<CommandAvailabilityQueryCount>()
        .map(|counter| counter.count)
        .unwrap_or(0);
    assert_eq!(query_count, 0);

    let svc = app
        .global::<WindowCommandActionAvailabilityService>()
        .expect("action availability service");
    assert_eq!(
        svc.available(window, &CommandId::from("test.available")),
        Some(false)
    );
}

#[test]
fn action_availability_snapshot_matches_no_focus_dispatch_subtree_fallback() {
    let mut app = crate::test_host::TestHost::new();
    app.set_global(PlatformCapabilities::default());

    let window = AppWindowId::default();
    app.register_command(
        CommandId::from("test.available"),
        widget_command_meta("Available"),
    );

    let mut ui: UiTree<crate::test_host::TestHost> = UiTree::new();
    ui.set_window(window);

    let root = ui.create_node(TestStack);
    let action_root = ui.create_node(CountingAvailabilityNode);
    ui.set_root(root);
    ui.add_child(root, action_root);
    ui.set_focus(None);

    let mut services = FakeUiServices;
    let bounds = Rect::new(Point::new(Px(0.0), Px(0.0)), Size::new(Px(100.0), Px(40.0)));
    ui.layout_all(&mut app, &mut services, bounds, 1.0);

    publish_snapshot(&mut ui, &mut app, window);

    let query_count = app
        .global::<CommandAvailabilityQueryCount>()
        .map(|counter| counter.count)
        .unwrap_or(0);
    assert_eq!(
        query_count, 1,
        "expected no-focus publication to use the same subtree route fallback as dispatch"
    );

    let svc = app
        .global::<WindowCommandActionAvailabilityService>()
        .expect("action availability service");
    assert_eq!(
        svc.available(window, &CommandId::from("test.available")),
        Some(true)
    );
}

#[test]
fn action_availability_snapshot_uses_dispatch_snapshot_parent_not_retained_parent() {
    let mut app = crate::test_host::TestHost::new();
    app.set_global(PlatformCapabilities::default());

    let window = AppWindowId::default();
    app.register_command(
        CommandId::from("test.available"),
        widget_command_meta("Available"),
    );

    let mut ui: UiTree<crate::test_host::TestHost> = UiTree::new();
    ui.set_window(window);

    let root = ui.create_node(TestStack);
    let leaf = ui.create_node(FocusableLeaf);
    let detached_handler = ui.create_node(CountingAvailabilityNode);
    ui.set_root(root);
    ui.add_child(root, leaf);
    ui.set_focus(Some(leaf));

    let mut services = FakeUiServices;
    let bounds = Rect::new(Point::new(Px(0.0), Px(0.0)), Size::new(Px(100.0), Px(40.0)));
    ui.layout_all(&mut app, &mut services, bounds, 1.0);

    let (_active_roots, barrier_root) = ui.active_input_layers();
    let (active_focus_roots, focus_barrier_root) = ui.active_focus_layers();
    let barrier_root = focus_barrier_root.or(barrier_root);
    let snapshot = ui.cached_dispatch_snapshot_for_layer_roots(
        app.frame_id(),
        &active_focus_roots,
        barrier_root,
    );
    assert!(snapshot.pre.get(leaf).is_some());
    assert!(snapshot.pre.get(detached_handler).is_none());

    ui.test_set_node_parent(leaf, Some(detached_handler));
    publish_snapshot(&mut ui, &mut app, window);

    let query_count = app
        .global::<CommandAvailabilityQueryCount>()
        .map(|counter| counter.count)
        .unwrap_or(0);
    assert_eq!(
        query_count, 0,
        "snapshot publication must not query availability through retained parents outside the dispatch snapshot"
    );

    let svc = app
        .global::<WindowCommandActionAvailabilityService>()
        .expect("action availability service");
    assert_eq!(
        svc.available(window, &CommandId::from("test.available")),
        Some(false)
    );
}

#[test]
fn action_availability_no_focus_subtree_fallback_scans_each_node_once_per_command() {
    let mut app = crate::test_host::TestHost::new();
    app.set_global(PlatformCapabilities::default());

    let window = AppWindowId::default();
    let commands = [
        CommandId::from("test.first_unhandled"),
        CommandId::from("test.second_unhandled"),
        CommandId::from("test.third_unhandled"),
    ];
    for command in &commands {
        app.register_command(command.clone(), widget_command_meta(command.as_str()));
    }

    let mut ui: UiTree<crate::test_host::TestHost> = UiTree::new();
    ui.set_window(window);

    let root = ui.create_node(TestStack);
    ui.set_root(root);
    let mut parent = root;
    let depth = 6usize;
    for _ in 0..depth {
        let child = ui.create_node(CountingAllAvailabilityNode);
        ui.add_child(parent, child);
        parent = child;
    }
    ui.set_focus(None);

    let mut services = FakeUiServices;
    let bounds = Rect::new(Point::new(Px(0.0), Px(0.0)), Size::new(Px(100.0), Px(40.0)));
    ui.layout_all(&mut app, &mut services, bounds, 1.0);

    publish_snapshot(&mut ui, &mut app, window);

    let query_count = app
        .global::<CommandAvailabilityQueryCount>()
        .map(|counter| counter.count)
        .unwrap_or(0);
    assert_eq!(
        query_count,
        (depth * commands.len()) as u32,
        "no-focus subtree fallback should be a single DFS over candidate nodes, not a per-node parent-chain bubble"
    );

    let svc = app
        .global::<WindowCommandActionAvailabilityService>()
        .expect("action availability service");
    for command in commands {
        assert_eq!(svc.available(window, &command), Some(false));
    }
}

#[test]
fn action_availability_no_focus_subtree_fallback_reuses_subtree_interest_across_commands() {
    let mut app = crate::test_host::TestHost::new();
    app.set_global(PlatformCapabilities::default());

    let window = AppWindowId::default();
    let commands = [
        CommandId::from("test.first_unhandled"),
        CommandId::from("test.second_unhandled"),
    ];
    for command in &commands {
        app.register_command(command.clone(), widget_command_meta(command.as_str()));
    }

    let mut ui: UiTree<crate::test_host::TestHost> = UiTree::new();
    ui.set_window(window);

    let root = ui.create_node(TestStack);
    ui.set_root(root);
    let mut parent = root;
    let depth = 4usize;
    for _ in 0..depth {
        let child = ui.create_node(CountingNoFocusInterestNode);
        ui.add_child(parent, child);
        parent = child;
    }
    ui.set_focus(None);

    let mut services = FakeUiServices;
    let bounds = Rect::new(Point::new(Px(0.0), Px(0.0)), Size::new(Px(100.0), Px(40.0)));
    ui.layout_all(&mut app, &mut services, bounds, 1.0);

    crate::tree::reset_command_availability_subtree_interest_probe_count();
    publish_snapshot(&mut ui, &mut app, window);

    assert_eq!(
        crate::tree::take_command_availability_subtree_interest_probe_count(),
        (depth + 1) as usize,
        "subtree interest should be computed once per node in the fallback subtree"
    );

    let svc = app
        .global::<WindowCommandActionAvailabilityService>()
        .expect("action availability service");
    for command in commands {
        assert_eq!(svc.available(window, &command), Some(false));
    }
}

#[test]
fn action_availability_no_focus_subtree_fallback_skips_focus_bound_edit_commands() {
    let mut app = crate::test_host::TestHost::new();
    app.set_global(PlatformCapabilities::default());

    let window = AppWindowId::default();
    let commands = [
        CommandId::from("edit.copy"),
        CommandId::from("text.copy"),
        CommandId::from("test.available"),
    ];
    for command in &commands {
        app.register_command(command.clone(), widget_command_meta(command.as_str()));
    }

    let mut ui: UiTree<crate::test_host::TestHost> = UiTree::new();
    ui.set_window(window);

    let root = ui.create_node(TestStack);
    let child = ui.create_node(CountingAllAvailabilityNode);
    ui.set_root(root);
    ui.add_child(root, child);
    ui.set_focus(None);

    let mut services = FakeUiServices;
    let bounds = Rect::new(Point::new(Px(0.0), Px(0.0)), Size::new(Px(100.0), Px(40.0)));
    ui.layout_all(&mut app, &mut services, bounds, 1.0);

    publish_snapshot(&mut ui, &mut app, window);

    let query_count = app
        .global::<CommandAvailabilityQueryCount>()
        .map(|counter| counter.count)
        .unwrap_or(0);
    assert_eq!(
        query_count, 1,
        "no-focus subtree fallback should skip focus-bound text/edit commands but keep custom widget command discovery"
    );

    let svc = app
        .global::<WindowCommandActionAvailabilityService>()
        .expect("action availability service");
    assert_eq!(
        svc.available(window, &CommandId::from("edit.copy")),
        Some(false)
    );
    assert_eq!(
        svc.available(window, &CommandId::from("text.copy")),
        Some(false)
    );
    assert_eq!(
        svc.available(window, &CommandId::from("test.available")),
        Some(true)
    );
}

#[test]
fn action_availability_no_focus_subtree_fallback_honors_ancestor_blocking() {
    let mut app = crate::test_host::TestHost::new();
    app.set_global(PlatformCapabilities::default());

    let window = AppWindowId::default();
    app.register_command(
        CommandId::from("test.available"),
        widget_command_meta("Available"),
    );

    let mut ui: UiTree<crate::test_host::TestHost> = UiTree::new();
    ui.set_window(window);

    let root = ui.create_node(TestStack);
    let blocker = ui.create_node(BlockingAvailabilityNode);
    let descendant = ui.create_node(CountingAvailabilityNode);
    ui.set_root(root);
    ui.add_child(root, blocker);
    ui.add_child(blocker, descendant);
    ui.set_focus(None);

    let mut services = FakeUiServices;
    let bounds = Rect::new(Point::new(Px(0.0), Px(0.0)), Size::new(Px(100.0), Px(40.0)));
    ui.layout_all(&mut app, &mut services, bounds, 1.0);

    publish_snapshot(&mut ui, &mut app, window);

    let query_count = app
        .global::<CommandAvailabilityQueryCount>()
        .map(|counter| counter.count)
        .unwrap_or(0);
    assert_eq!(
        query_count, 0,
        "ancestor blocked nodes should stop the no-focus subtree fallback before descending further"
    );

    let svc = app
        .global::<WindowCommandActionAvailabilityService>()
        .expect("action availability service");
    assert_eq!(
        svc.available(window, &CommandId::from("test.available")),
        Some(false)
    );
}

#[test]
fn action_availability_no_focus_subtree_fallback_prunes_focus_bound_text_interest() {
    let mut app = crate::test_host::TestHost::new();
    app.set_global(PlatformCapabilities::default());

    let window = AppWindowId::default();
    app.register_command(CommandId::from("edit.copy"), widget_command_meta("Copy"));

    let mut ui: UiTree<crate::test_host::TestHost> = UiTree::new();
    ui.set_window(window);

    let mut services = FakeUiServices;
    let bounds = Rect::new(Point::new(Px(0.0), Px(0.0)), Size::new(Px(100.0), Px(40.0)));
    let root = declarative::render_root(
        &mut ui,
        &mut app,
        &mut services,
        window,
        bounds,
        "no-focus-selectable-copy-interest",
        |cx| vec![cx.selectable_text(attributed_plain("copyable text"))],
    );
    ui.set_root(root);
    ui.set_focus(None);
    ui.layout_all(&mut app, &mut services, bounds, 1.0);

    crate::tree::reset_command_availability_widget_probe_count();
    publish_snapshot(&mut ui, &mut app, window);

    assert_eq!(
        crate::tree::take_command_availability_widget_probe_count(),
        0,
        "no-focus subtree publication should not call focused-only text/selectable command availability"
    );

    let svc = app
        .global::<WindowCommandActionAvailabilityService>()
        .expect("action availability service");
    assert_eq!(
        svc.available(window, &CommandId::from("edit.copy")),
        Some(false)
    );
}

#[test]
fn action_availability_focused_selectable_text_still_uses_text_interest() {
    let mut app = crate::test_host::TestHost::new();
    app.set_global(PlatformCapabilities::default());

    let window = AppWindowId::default();
    app.register_command(
        CommandId::from("edit.select_all"),
        widget_command_meta("Select All"),
    );

    let mut ui: UiTree<crate::test_host::TestHost> = UiTree::new();
    ui.set_window(window);

    let mut services = FakeUiServices;
    let bounds = Rect::new(Point::new(Px(0.0), Px(0.0)), Size::new(Px(100.0), Px(40.0)));
    let root = declarative::render_root(
        &mut ui,
        &mut app,
        &mut services,
        window,
        bounds,
        "focused-selectable-text-interest",
        |cx| vec![cx.selectable_text(attributed_plain("copyable text"))],
    );
    ui.set_root(root);
    ui.layout_all(&mut app, &mut services, bounds, 1.0);
    let selectable_node = ui.children(root)[0];
    ui.set_focus(Some(selectable_node));

    crate::tree::reset_command_availability_widget_probe_count();
    publish_snapshot(&mut ui, &mut app, window);

    assert!(
        crate::tree::take_command_availability_widget_probe_count() > 0,
        "focused text/selectable command availability must still participate in publication"
    );

    let svc = app
        .global::<WindowCommandActionAvailabilityService>()
        .expect("action availability service");
    assert_eq!(
        svc.available(window, &CommandId::from("edit.select_all")),
        Some(true)
    );
}

#[test]
fn action_availability_snapshot_uses_explicit_action_route_fallback_root() {
    use crate::elements::{GlobalElementId, NodeEntry};

    let mut app = crate::test_host::TestHost::new();
    app.set_global(PlatformCapabilities::default());

    let window = AppWindowId::default();
    app.register_command(
        CommandId::from("test.available"),
        widget_command_meta("Available"),
    );

    let mut ui: UiTree<crate::test_host::TestHost> = UiTree::new();
    ui.set_debug_enabled(true);
    ui.set_window(window);

    let root = ui.create_node(TestStack);
    let focused = ui.create_node(FocusableLeaf);
    let action_root_element = GlobalElementId(0xA11CE);
    let action_root = ui.create_node_for_element(action_root_element, CountingAvailabilityNode);
    ui.set_root(root);
    ui.add_child(root, focused);
    ui.add_child(root, action_root);
    ui.set_focus(Some(focused));

    let frame_id = app.frame_id();
    crate::elements::with_window_state(&mut app, window, |st| {
        st.set_node_entry(
            action_root_element,
            NodeEntry {
                node: action_root,
                last_seen_frame: frame_id,
                root: action_root_element,
            },
        );
        st.record_action_route_fallback_root(action_root_element);
    });

    let mut services = FakeUiServices;
    let bounds = Rect::new(Point::new(Px(0.0), Px(0.0)), Size::new(Px(100.0), Px(40.0)));
    ui.layout_all(&mut app, &mut services, bounds, 1.0);

    app.advance_frame();
    ui.begin_debug_frame_if_needed(app.frame_id());
    publish_snapshot(&mut ui, &mut app, window);

    let query_count = app
        .global::<CommandAvailabilityQueryCount>()
        .map(|counter| counter.count)
        .unwrap_or(0);
    assert_eq!(query_count, 1);

    let svc = app
        .global::<WindowCommandActionAvailabilityService>()
        .expect("action availability service");
    assert_eq!(
        svc.available(window, &CommandId::from("test.available")),
        Some(true)
    );

    let route_hotspots: Vec<_> = ui
        .debug_command_availability_hotspots()
        .iter()
        .filter(|hotspot| hotspot.route == "action_route_fallback_roots")
        .collect();
    assert_eq!(route_hotspots.len(), 1);
    assert_eq!(route_hotspots[0].start_node, action_root);
    assert_eq!(route_hotspots[0].resolved_node, Some(action_root));
    assert_eq!(route_hotspots[0].start_element, Some(action_root_element));
}

#[test]
fn action_availability_snapshot_publishes_focus_menu_bar_gating() {
    let mut app = crate::test_host::TestHost::new();
    app.set_global(PlatformCapabilities::default());

    let window = AppWindowId::default();
    app.register_command(
        CommandId::from("focus.menu_bar"),
        widget_command_meta("Focus Menu Bar"),
    );

    let mut ui: UiTree<crate::test_host::TestHost> = UiTree::new();
    ui.set_window(window);

    let root = ui.create_node(TestStack);
    let leaf = ui.create_node(FocusableLeaf);
    ui.set_root(root);
    ui.add_child(root, leaf);
    ui.set_focus(Some(leaf));

    let mut services = FakeUiServices;
    let bounds = Rect::new(Point::new(Px(0.0), Px(0.0)), Size::new(Px(100.0), Px(40.0)));
    ui.layout_all(&mut app, &mut services, bounds, 1.0);

    publish_snapshot(&mut ui, &mut app, window);

    let svc = app
        .global::<WindowCommandActionAvailabilityService>()
        .expect("action availability service");
    assert_eq!(
        svc.available(window, &CommandId::from("focus.menu_bar")),
        Some(false)
    );

    let mut focus_svc = WindowMenuBarFocusService::default();
    focus_svc.set_present(window, true);
    app.set_global(focus_svc);

    publish_snapshot(&mut ui, &mut app, window);

    let svc = app
        .global::<WindowCommandActionAvailabilityService>()
        .expect("action availability service");
    assert_eq!(
        svc.available(window, &CommandId::from("focus.menu_bar")),
        Some(true)
    );
}

#[derive(Debug, Default)]
struct FocusOnPointerDownAvailable;

impl<H: UiHost> Widget<H> for FocusOnPointerDownAvailable {
    fn is_focusable(&self) -> bool {
        true
    }

    fn event(&mut self, cx: &mut EventCx<'_, H>, event: &Event) {
        if matches!(event, Event::Pointer(fret_core::PointerEvent::Down { .. })) {
            cx.request_focus(cx.node);
            cx.stop_propagation();
        }
    }

    fn command_availability(
        &self,
        _cx: &mut crate::widget::CommandAvailabilityCx<'_, H>,
        command: &CommandId,
    ) -> crate::widget::CommandAvailability {
        match command.as_str() {
            "test.available" => crate::widget::CommandAvailability::Available,
            _ => crate::widget::CommandAvailability::NotHandled,
        }
    }

    fn layout(&mut self, cx: &mut LayoutCx<'_, H>) -> Size {
        cx.available
    }
}

#[test]
fn dispatch_event_publishes_action_availability_snapshot() {
    let mut app = crate::test_host::TestHost::new();
    app.set_global(PlatformCapabilities::default());

    let window = AppWindowId::default();
    app.register_command(
        CommandId::from("test.available"),
        widget_command_meta("Available"),
    );

    let mut ui: UiTree<crate::test_host::TestHost> = UiTree::new();
    ui.set_window(window);

    let root = ui.create_node(TestStack);
    let leaf = ui.create_node(FocusOnPointerDownAvailable);
    ui.set_root(root);
    ui.add_child(root, leaf);

    let mut services = FakeUiServices;
    let bounds = Rect::new(Point::new(Px(0.0), Px(0.0)), Size::new(Px(100.0), Px(40.0)));
    ui.layout_all(&mut app, &mut services, bounds, 1.0);

    ui.dispatch_event(
        &mut app,
        &mut services,
        &Event::Pointer(fret_core::PointerEvent::Down {
            pointer_id: fret_core::PointerId(0),
            position: Point::new(Px(10.0), Px(10.0)),
            button: fret_core::MouseButton::Left,
            modifiers: fret_core::Modifiers::default(),
            pointer_type: fret_core::PointerType::Mouse,
            click_count: 1,
        }),
    );

    let svc = app
        .global::<WindowCommandActionAvailabilityService>()
        .expect("action availability service");
    assert_eq!(
        svc.available(window, &CommandId::from("test.available")),
        Some(true)
    );
}

#[test]
fn publish_snapshot_refreshes_key_context_stack_for_cross_surface_gating() {
    let mut app = crate::test_host::TestHost::new();
    app.set_global(PlatformCapabilities::default());

    let window = AppWindowId::default();
    let command = CommandId::from("test.keyctx_gated");
    app.register_command(
        command.clone(),
        CommandMeta::new("Key Context Gated")
            .with_scope(CommandScope::App)
            .with_when(WhenExpr::parse("keyctx.demo").unwrap()),
    );

    app.with_global_mut(WindowKeyContextStackService::default, |svc, _app| {
        svc.set_snapshot(window, vec![Arc::<str>::from("demo")]);
    });

    let mut ui: UiTree<crate::test_host::TestHost> = UiTree::new();
    ui.set_window(window);

    let root = ui.create_node(TestStack);
    ui.set_root(root);

    let mut services = FakeUiServices;
    let bounds = Rect::new(Point::new(Px(0.0), Px(0.0)), Size::new(Px(100.0), Px(40.0)));
    ui.layout_all(&mut app, &mut services, bounds, 1.0);

    publish_snapshot(&mut ui, &mut app, window);

    assert!(
        !command_is_enabled_for_window_with_input_ctx_fallback(
            &app,
            window,
            &command,
            InputContext::default(),
        ),
        "publishing a fresh action-availability snapshot should also refresh key-context snapshots so cross-surface gating does not keep stale keyctx values alive"
    );
}
