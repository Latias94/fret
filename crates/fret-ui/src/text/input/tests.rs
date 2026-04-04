use super::*;

use crate::UiTree;
use crate::test_host::TestHost;
use crate::widget::Widget;
use fret_core::{
    AppWindowId, Event, ImeEvent, Point, Px, Rect, Size, TextConstraints, TextMetrics, TextService,
};
use fret_runtime::{CommandId, Effect, PlatformCapabilities, TextInteractionSettings};
use std::sync::Arc;

#[derive(Debug, Clone, Copy)]
struct RenderTransformWrapper {
    transform: fret_core::Transform2D,
}

impl RenderTransformWrapper {
    fn new(transform: fret_core::Transform2D) -> Self {
        Self { transform }
    }
}

impl Widget<TestHost> for RenderTransformWrapper {
    fn render_transform(&self, _bounds: Rect) -> Option<fret_core::Transform2D> {
        Some(self.transform)
    }

    fn layout(&mut self, cx: &mut crate::widget::LayoutCx<'_, TestHost>) -> Size {
        let Some(&child) = cx.children.first() else {
            return Size::default();
        };
        cx.layout(child, cx.available)
    }
}

#[derive(Default)]
struct FakeTextService {
    prepared: Vec<String>,
}

impl TextService for FakeTextService {
    fn prepare(
        &mut self,
        input: &fret_core::TextInput,
        _constraints: TextConstraints,
    ) -> (fret_core::TextBlobId, TextMetrics) {
        let text = input.text();
        self.prepared.push(text.to_string());
        (
            fret_core::TextBlobId::default(),
            TextMetrics {
                size: Size::new(Px(10.0), Px(10.0)),
                baseline: Px(8.0),
            },
        )
    }

    fn release(&mut self, _blob: fret_core::TextBlobId) {}
}

impl fret_core::PathService for FakeTextService {
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

impl fret_core::SvgService for FakeTextService {
    fn register_svg(&mut self, _bytes: &[u8]) -> fret_core::SvgId {
        fret_core::SvgId::default()
    }

    fn unregister_svg(&mut self, _svg: fret_core::SvgId) -> bool {
        false
    }
}

impl fret_core::MaterialService for FakeTextService {
    fn register_material(
        &mut self,
        _desc: fret_core::MaterialDescriptor,
    ) -> Result<fret_core::MaterialId, fret_core::MaterialRegistrationError> {
        Err(fret_core::MaterialRegistrationError::Unsupported)
    }

    fn unregister_material(&mut self, _id: fret_core::MaterialId) -> bool {
        false
    }
}

#[derive(Default)]
struct IndexRecordingTextService {
    prepared: Vec<String>,
    caret_x_calls: Vec<usize>,
}

impl TextService for IndexRecordingTextService {
    fn prepare(
        &mut self,
        input: &fret_core::TextInput,
        _constraints: TextConstraints,
    ) -> (fret_core::TextBlobId, TextMetrics) {
        self.prepared.push(input.text().to_string());
        (
            fret_core::TextBlobId::default(),
            TextMetrics {
                size: Size::new(Px(10.0), Px(10.0)),
                baseline: Px(8.0),
            },
        )
    }

    fn caret_x(&mut self, _blob: fret_core::TextBlobId, index: usize) -> Px {
        self.caret_x_calls.push(index);
        Px(index as f32)
    }

    fn release(&mut self, _blob: fret_core::TextBlobId) {}
}

impl fret_core::PathService for IndexRecordingTextService {
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

impl fret_core::SvgService for IndexRecordingTextService {
    fn register_svg(&mut self, _bytes: &[u8]) -> fret_core::SvgId {
        fret_core::SvgId::default()
    }

    fn unregister_svg(&mut self, _svg: fret_core::SvgId) -> bool {
        false
    }
}

impl fret_core::MaterialService for IndexRecordingTextService {
    fn register_material(
        &mut self,
        _desc: fret_core::MaterialDescriptor,
    ) -> Result<fret_core::MaterialId, fret_core::MaterialRegistrationError> {
        Err(fret_core::MaterialRegistrationError::Unsupported)
    }

    fn unregister_material(&mut self, _id: fret_core::MaterialId) -> bool {
        false
    }
}

fn event_cx<'a>(
    app: &'a mut TestHost,
    services: &'a mut dyn fret_core::UiServices,
    node: fret_core::NodeId,
    window: fret_core::AppWindowId,
    bounds: Rect,
    prevented_default_actions: &'a mut fret_runtime::DefaultActionSet,
) -> crate::widget::EventCx<'a, TestHost> {
    crate::widget::EventCx {
        app,
        services,
        node,
        layer_root: None,
        window: Some(window),
        input_ctx: fret_runtime::InputContext {
            caps: PlatformCapabilities::default(),
            ..Default::default()
        },
        pointer_hit_is_text_input: false,
        pointer_hit_is_pressable: false,
        pointer_hit_pressable_target: None,
        pointer_hit_pressable_target_in_descendant_subtree: false,
        prevented_default_actions,
        pointer_id: None,
        scale_factor: 1.0,
        event_window_position: None,
        event_window_wheel_delta: None,
        children: &[],
        focus: Some(node),
        captured: None,
        bounds,
        invalidations: Vec::new(),
        scroll_handle_invalidations: Vec::new(),
        scroll_target_invalidations: Vec::new(),
        requested_focus: None,
        requested_focus_target: None,
        requested_capture: None,
        requested_cursor: None,
        notify_requested: false,
        notify_requested_location: None,
        stop_propagation: false,
    }
}

fn command_cx<'a>(
    app: &'a mut TestHost,
    services: &'a mut dyn fret_core::UiServices,
    tree: &'a mut UiTree<TestHost>,
    node: fret_core::NodeId,
    window: fret_core::AppWindowId,
) -> crate::widget::CommandCx<'a, TestHost> {
    crate::widget::CommandCx {
        app,
        services,
        tree,
        node,
        window: Some(window),
        input_ctx: fret_runtime::InputContext {
            caps: PlatformCapabilities::default(),
            ..Default::default()
        },
        focus: Some(node),
        invalidations: Vec::new(),
        requested_focus: None,
        notify_requested: false,
        notify_requested_location: None,
        stop_propagation: false,
    }
}

#[test]
fn text_input_hover_sets_text_cursor_effect() {
    let window = AppWindowId::default();

    let mut ui = UiTree::new();
    ui.set_window(window);

    let root = ui.create_node(TextInput::new());
    ui.set_root(root);

    let mut app = TestHost::new();
    app.set_global(PlatformCapabilities::default());
    let mut text = FakeTextService::default();

    let _ = ui.layout(
        &mut app,
        &mut text,
        root,
        Size::new(Px(200.0), Px(40.0)),
        1.0,
    );
    let _ = app.take_effects();

    ui.dispatch_event(
        &mut app,
        &mut text,
        &Event::Pointer(fret_core::PointerEvent::Move {
            position: Point::new(Px(10.0), Px(10.0)),
            buttons: fret_core::MouseButtons::default(),
            modifiers: fret_core::Modifiers::default(),
            pointer_id: fret_core::PointerId(0),
            pointer_type: fret_core::PointerType::Mouse,
        }),
    );

    let effects = app.take_effects();
    assert!(
        effects.iter().any(|e| matches!(
            e,
            Effect::CursorSetIcon { window: w, icon }
                if *w == window && *icon == fret_core::CursorIcon::Text
        )),
        "expected a text cursor effect when hovering a text input"
    );
}

#[test]
fn text_input_obscures_paint_text_when_requested() {
    let window = AppWindowId::default();
    let bounds = Rect::new(Point::new(Px(0.0), Px(0.0)), Size::new(Px(240.0), Px(40.0)));

    let mut ui = UiTree::new();
    ui.set_window(window);

    let mut widget = TextInput::new().with_text("abc");
    widget.set_obscure_text(true);
    let root = ui.create_node(widget);
    ui.set_root(root);
    ui.set_focus(Some(root));

    let mut app = TestHost::new();
    app.set_global(PlatformCapabilities::default());
    let mut services = FakeTextService::default();

    ui.layout_all(&mut app, &mut services, bounds, 1.0);
    let _ = app.take_effects();

    let mut scene = fret_core::Scene::default();
    ui.paint_all(&mut app, &mut services, bounds, &mut scene, 1.0);

    assert!(
        services.prepared.iter().any(|s| s == "•••"),
        "expected text service to see an obscured string for password-style text inputs"
    );
}

#[test]
fn text_input_obscure_text_maps_caret_queries_to_paint_indices() {
    let window = AppWindowId::default();
    let bounds = Rect::new(Point::new(Px(0.0), Px(0.0)), Size::new(Px(240.0), Px(40.0)));

    let mut ui = UiTree::new();
    ui.set_window(window);

    let mut widget = TextInput::new().with_text("abc");
    widget.set_obscure_text(true);
    let root = ui.create_node(widget);
    ui.set_root(root);
    ui.set_focus(Some(root));

    let mut app = TestHost::new();
    app.set_global(PlatformCapabilities::default());
    let mut services = IndexRecordingTextService::default();

    ui.layout_all(&mut app, &mut services, bounds, 1.0);
    let _ = app.take_effects();

    let mut scene = fret_core::Scene::default();
    ui.paint_all(&mut app, &mut services, bounds, &mut scene, 1.0);

    // "abc" becomes "•••" (3 UTF-8 bytes per dot), so the caret at the end should query index 9.
    assert!(
        services.caret_x_calls.iter().any(|&i| i == 9),
        "expected caret_x to be queried in paint-space indices for obscured text"
    );
}

#[test]
fn right_click_focuses_and_preserves_selection_for_context_menus() {
    let window = AppWindowId::default();
    let bounds = Rect::new(Point::new(Px(0.0), Px(0.0)), Size::new(Px(200.0), Px(40.0)));

    let mut ui = UiTree::new();
    ui.set_window(window);

    let mut widget = TextInput::new();
    widget.text = "hello world".to_string();
    widget.caret = 5;
    widget.selection_anchor = 0;
    widget.set_chrome_style(crate::TextInputStyle {
        padding: fret_core::Edges::all(Px(0.0)),
        ..Default::default()
    });
    let input = ui.create_node(widget);
    ui.set_root(input);
    ui.set_focus(None);

    let mut app = TestHost::new();
    app.set_global(PlatformCapabilities::default());
    let mut text = ImeTextService::default();

    ui.layout_all(&mut app, &mut text, bounds, 1.0);
    let _ = app.take_effects();

    ui.dispatch_event(
        &mut app,
        &mut text,
        &Event::Pointer(fret_core::PointerEvent::Down {
            position: Point::new(Px(2.0), Px(10.0)),
            button: fret_core::MouseButton::Right,
            modifiers: fret_core::Modifiers::default(),
            click_count: 1,
            pointer_id: fret_core::PointerId(0),
            pointer_type: fret_core::PointerType::Mouse,
        }),
    );

    assert_eq!(
        ui.focus(),
        Some(input),
        "expected right-click to focus the text input so context menu commands target it"
    );

    let mut scene = fret_core::Scene::default();
    ui.paint_all(&mut app, &mut text, bounds, &mut scene, 1.0);

    let snap0 = app
        .global::<fret_runtime::WindowTextInputSnapshotService>()
        .and_then(|svc| svc.snapshot(window))
        .cloned()
        .expect("expected a window text input snapshot after paint");
    assert_eq!(
        snap0.selection_utf16,
        Some((0, 5)),
        "expected right-click inside selection to preserve it"
    );

    ui.dispatch_event(
        &mut app,
        &mut text,
        &Event::Pointer(fret_core::PointerEvent::Down {
            position: Point::new(Px(8.0), Px(10.0)),
            button: fret_core::MouseButton::Right,
            modifiers: fret_core::Modifiers::default(),
            click_count: 1,
            pointer_id: fret_core::PointerId(1),
            pointer_type: fret_core::PointerType::Mouse,
        }),
    );

    let mut scene = fret_core::Scene::default();
    ui.paint_all(&mut app, &mut text, bounds, &mut scene, 1.0);

    let snap1 = app
        .global::<fret_runtime::WindowTextInputSnapshotService>()
        .and_then(|svc| svc.snapshot(window))
        .cloned()
        .expect("expected a window text input snapshot after paint");
    assert_eq!(
        snap1.selection_utf16,
        Some((8, 8)),
        "expected right-click outside selection to collapse it to the caret position"
    );
}

#[test]
fn text_input_double_click_selection_respects_text_boundary_mode() {
    let window = AppWindowId::default();
    let node = fret_core::NodeId::default();
    let bounds = Rect::new(Point::new(Px(0.0), Px(0.0)), Size::new(Px(200.0), Px(40.0)));

    let mut app = TestHost::new();
    app.set_global(PlatformCapabilities::default());
    let mut services = FakeTextService::default();
    let mut prevented_default_actions = fret_runtime::DefaultActionSet::default();

    let mut input = TextInput::new();
    input.text = "can't".to_string();

    let mut cx = event_cx(
        &mut app,
        &mut services,
        node,
        window,
        bounds,
        &mut prevented_default_actions,
    );
    cx.input_ctx.text_boundary_mode = fret_runtime::TextBoundaryMode::UnicodeWord;

    input.event(
        &mut cx,
        &Event::Pointer(fret_core::PointerEvent::Down {
            position: Point::new(Px(5.0), Px(5.0)),
            button: fret_core::MouseButton::Left,
            modifiers: fret_core::Modifiers::default(),
            click_count: 2,
            pointer_id: fret_core::PointerId(0),
            pointer_type: fret_core::PointerType::Mouse,
        }),
    );
    assert_eq!(
        (input.selection_anchor, input.caret),
        (0, 5),
        "Unicode word mode should select the whole word"
    );

    let mut input = TextInput::new();
    input.text = "can't".to_string();

    let mut cx = event_cx(
        &mut app,
        &mut services,
        node,
        window,
        bounds,
        &mut prevented_default_actions,
    );
    cx.input_ctx.text_boundary_mode = fret_runtime::TextBoundaryMode::Identifier;

    input.event(
        &mut cx,
        &Event::Pointer(fret_core::PointerEvent::Down {
            position: Point::new(Px(5.0), Px(5.0)),
            button: fret_core::MouseButton::Left,
            modifiers: fret_core::Modifiers::default(),
            click_count: 2,
            pointer_id: fret_core::PointerId(0),
            pointer_type: fret_core::PointerType::Mouse,
        }),
    );
    assert_eq!(
        (input.selection_anchor, input.caret),
        (0, 3),
        "Identifier mode should stop at the apostrophe"
    );
}

#[test]
fn text_input_command_availability_tracks_selection_and_clipboard_caps() {
    let window = AppWindowId::default();
    let mut ui = UiTree::new();
    ui.set_window(window);

    let mut input = TextInput::new();
    input.text = "hello".to_string();
    input.caret = 0;
    input.selection_anchor = 0;
    let node = ui.create_node(input);
    ui.set_root(node);
    ui.set_focus(Some(node));

    let mut app = TestHost::new();
    let mut caps = PlatformCapabilities::default();
    caps.clipboard.text.read = true;
    caps.clipboard.text.write = true;
    app.set_global(caps);

    assert_eq!(
        ui.command_availability(&mut app, &CommandId::from("edit.copy")),
        crate::widget::CommandAvailability::Blocked,
        "expected copy to be blocked without a selection"
    );

    let mut ui = UiTree::new();
    ui.set_window(window);
    let mut input = TextInput::new();
    input.text = "hello".to_string();
    input.caret = 5;
    input.selection_anchor = 0;
    let node = ui.create_node(input);
    ui.set_root(node);
    ui.set_focus(Some(node));

    assert_eq!(
        ui.command_availability(&mut app, &CommandId::from("edit.copy")),
        crate::widget::CommandAvailability::Available,
        "expected copy to be available with a selection"
    );

    let mut caps = PlatformCapabilities::default();
    caps.clipboard.text.read = false;
    caps.clipboard.text.write = false;
    app.set_global(caps);

    assert_eq!(
        ui.command_availability(&mut app, &CommandId::from("edit.copy")),
        crate::widget::CommandAvailability::Blocked,
        "expected copy to be blocked when clipboard text is unavailable"
    );
}

#[test]
fn text_input_copy_clamps_out_of_range_selection_indices() {
    let window = AppWindowId::default();
    let node = fret_core::NodeId::default();

    let mut ui = UiTree::new();
    ui.set_window(window);

    let mut app = TestHost::new();
    app.set_global(PlatformCapabilities::default());
    let mut services = FakeTextService::default();

    let mut input = TextInput::new();
    input.text = "hello".to_string();
    input.selection_anchor = 0;
    input.caret = 999;

    let mut cx = command_cx(&mut app, &mut services, &mut ui, node, window);
    assert!(
        input.command(&mut cx, &fret_runtime::CommandId::from("edit.copy")),
        "expected edit.copy to be handled"
    );

    assert!(
        app.take_effects()
            .iter()
            .any(|e| matches!(e, Effect::ClipboardWriteText { text, .. } if text == "hello")),
        "expected edit.copy to clamp indices and copy the selected text"
    );
    assert_eq!(input.selection_anchor, 0);
    assert_eq!(input.caret, 5);
}

#[test]
fn text_input_renders_placeholder_when_empty() {
    let window = AppWindowId::default();

    let mut ui = UiTree::new();
    ui.set_window(window);

    let mut input = TextInput::new();
    input.set_placeholder(Some(Arc::from("Search…")));
    let root = ui.create_node(input);
    ui.set_root(root);

    let mut app = TestHost::new();
    app.set_global(PlatformCapabilities::default());
    let mut text = FakeTextService::default();
    let mut scene = fret_core::Scene::default();

    let _ = ui.layout(
        &mut app,
        &mut text,
        root,
        Size::new(Px(200.0), Px(40.0)),
        1.0,
    );

    ui.paint(
        &mut app,
        &mut text,
        root,
        Rect::new(
            fret_core::Point::new(Px(0.0), Px(0.0)),
            Size::new(Px(200.0), Px(40.0)),
        ),
        &mut scene,
        1.0,
    );

    assert!(
        text.prepared.iter().any(|t| t == "Search…"),
        "expected placeholder to be prepared as text"
    );
}

#[test]
fn text_input_draws_caret_when_focused_and_empty() {
    let window = AppWindowId::default();
    let bounds = Rect::new(Point::new(Px(0.0), Px(0.0)), Size::new(Px(240.0), Px(80.0)));

    let mut ui = UiTree::new();
    ui.set_window(window);

    let caret_color = fret_core::Color {
        r: 0.9,
        g: 0.1,
        b: 0.2,
        a: 1.0,
    };

    let mut input = TextInput::new();
    input.set_chrome_style(TextInputStyle {
        caret_color,
        ..Default::default()
    });

    let root = ui.create_node(input);
    ui.set_root(root);
    ui.set_focus(Some(root));

    let mut app = TestHost::new();
    app.set_global(PlatformCapabilities::default());
    let mut text = FakeTextService::default();

    let _ = ui.layout(&mut app, &mut text, root, bounds.size, 1.0);

    let mut scene = fret_core::Scene::default();
    ui.paint(&mut app, &mut text, root, bounds, &mut scene, 1.0);

    let caret_rect = scene.ops().iter().rev().find_map(|op| match op {
        fret_core::SceneOp::Quad {
            rect, background, ..
        } if *background == fret_core::Paint::Solid(caret_color).into() => Some(*rect),
        _ => None,
    });

    let Some(caret_rect) = caret_rect else {
        panic!("expected a caret quad to be present in the scene");
    };

    assert!(
        caret_rect.size.height.0 > 0.0,
        "expected caret height to be > 0 (got {:?})",
        caret_rect.size.height
    );
}

#[test]
fn text_input_caret_blinks_when_enabled() {
    let window = AppWindowId::default();
    let bounds = Rect::new(Point::new(Px(0.0), Px(0.0)), Size::new(Px(240.0), Px(80.0)));

    let mut ui = UiTree::new();
    ui.set_window(window);

    let caret_color = fret_core::Color {
        r: 0.91,
        g: 0.23,
        b: 0.14,
        a: 1.0,
    };

    let mut input = TextInput::new();
    input.set_chrome_style(TextInputStyle {
        caret_color,
        ..Default::default()
    });

    let root = ui.create_node(input);
    ui.set_root(root);
    ui.set_focus(Some(root));

    let mut app = TestHost::new();
    app.set_global(PlatformCapabilities::default());
    app.set_global(TextInteractionSettings {
        caret_blink: true,
        caret_blink_interval_ms: 16,
        ..Default::default()
    });
    let mut text = FakeTextService::default();

    let _ = ui.layout(&mut app, &mut text, root, bounds.size, 1.0);

    let mut scene0 = fret_core::Scene::default();
    ui.paint(&mut app, &mut text, root, bounds, &mut scene0, 1.0);

    let token0 = app
        .take_effects()
        .into_iter()
        .find_map(|e| match e {
            Effect::SetTimer { token, .. } => Some(token),
            _ => None,
        })
        .expect("expected caret blink to schedule a timer when focused");

    assert!(
        scene0.ops().iter().any(|op| matches!(
            op,
            fret_core::SceneOp::Quad { background, .. }
                if *background == fret_core::Paint::Solid(caret_color).into()
        )),
        "expected caret to be visible before the blink timer fires"
    );

    ui.dispatch_event(&mut app, &mut text, &Event::Timer { token: token0 });
    let _ = app.take_effects();

    let mut scene1 = fret_core::Scene::default();
    ui.paint(&mut app, &mut text, root, bounds, &mut scene1, 1.0);

    assert!(
        !scene1.ops().iter().any(|op| matches!(
            op,
            fret_core::SceneOp::Quad { background, .. }
                if *background == fret_core::Paint::Solid(caret_color).into()
        )),
        "expected caret to be hidden after the blink timer fires"
    );
}

#[test]
fn text_input_blur_hides_caret_and_clears_preedit_before_refocus() {
    let window = AppWindowId::default();
    let bounds = Rect::new(Point::new(Px(0.0), Px(0.0)), Size::new(Px(240.0), Px(80.0)));

    let mut ui = UiTree::new();
    ui.set_window(window);

    let caret_color = fret_core::Color {
        r: 0.91,
        g: 0.23,
        b: 0.14,
        a: 1.0,
    };
    let underline_color = fret_core::Color {
        r: 0.12,
        g: 0.78,
        b: 0.31,
        a: 1.0,
    };

    let mut input = TextInput::new().with_text("hello");
    input.set_chrome_style(TextInputStyle {
        caret_color,
        preedit_underline_color: underline_color,
        ..Default::default()
    });

    let root = ui.create_node(input);
    ui.set_root(root);
    ui.set_focus(Some(root));

    let mut app = TestHost::new();
    app.set_global(PlatformCapabilities::default());
    app.set_global(TextInteractionSettings {
        caret_blink: true,
        caret_blink_interval_ms: 16,
        ..Default::default()
    });
    let mut text = FakeTextService::default();

    ui.layout_all(&mut app, &mut text, bounds, 1.0);
    let _ = app.take_effects();

    ui.dispatch_event(
        &mut app,
        &mut text,
        &Event::Ime(ImeEvent::Preedit {
            text: "yo".to_string(),
            cursor: Some((0, 2)),
        }),
    );

    let mut focused_scene = fret_core::Scene::default();
    ui.paint_all(&mut app, &mut text, bounds, &mut focused_scene, 1.0);

    let focused_effects = app.take_effects();
    let blink_token = focused_effects
        .iter()
        .find_map(|effect| match effect {
            Effect::SetTimer { token, .. } => Some(*token),
            _ => None,
        })
        .expect("expected focused preedit to keep a caret blink timer active");

    let focused_snapshot = app
        .global::<fret_runtime::WindowTextInputSnapshotService>()
        .and_then(|svc| svc.snapshot(window))
        .cloned()
        .expect("expected a text input snapshot while focused");
    assert!(focused_snapshot.focus_is_text_input);
    assert!(focused_snapshot.is_composing);
    assert!(focused_snapshot.marked_utf16.is_some());
    assert!(
        focused_scene.ops().iter().any(|op| matches!(
            op,
            fret_core::SceneOp::Quad { background, .. }
                if *background == fret_core::Paint::Solid(caret_color).into()
        )),
        "expected caret to remain visible while focused"
    );
    assert!(
        focused_scene.ops().iter().any(|op| matches!(
            op,
            fret_core::SceneOp::Quad { background, rect, .. }
                if *background == fret_core::Paint::Solid(underline_color).into()
                    && rect.size.height.0 > 0.0
                    && rect.size.height.0 <= 1.1
        )),
        "expected focused preedit to paint an underline"
    );

    ui.set_focus(None);

    let mut blur_scene = fret_core::Scene::default();
    ui.paint_all(&mut app, &mut text, bounds, &mut blur_scene, 1.0);

    let blur_effects = app.take_effects();
    assert!(
        blur_effects.iter().any(|effect| matches!(
            effect,
            Effect::CancelTimer { token } if *token == blink_token
        )),
        "expected blur to cancel the active caret blink timer"
    );
    assert!(
        !blur_scene.ops().iter().any(|op| matches!(
            op,
            fret_core::SceneOp::Quad { background, .. }
                if *background == fret_core::Paint::Solid(caret_color).into()
        )),
        "expected blur to stop painting the caret"
    );

    let blur_snapshot = app
        .global::<fret_runtime::WindowTextInputSnapshotService>()
        .and_then(|svc| svc.snapshot(window))
        .cloned()
        .expect("expected the window text input snapshot service to stay present");
    assert!(!blur_snapshot.focus_is_text_input);
    assert!(!blur_snapshot.is_composing);
    assert_eq!(blur_snapshot.marked_utf16, None);

    ui.set_focus(Some(root));

    let mut refocus_scene = fret_core::Scene::default();
    ui.paint_all(&mut app, &mut text, bounds, &mut refocus_scene, 1.0);
    let _ = app.take_effects();

    let refocus_snapshot = app
        .global::<fret_runtime::WindowTextInputSnapshotService>()
        .and_then(|svc| svc.snapshot(window))
        .cloned()
        .expect("expected a text input snapshot after refocus");
    assert!(refocus_snapshot.focus_is_text_input);
    assert!(!refocus_snapshot.is_composing);
    assert_eq!(refocus_snapshot.marked_utf16, None);
    assert!(
        refocus_scene.ops().iter().any(|op| matches!(
            op,
            fret_core::SceneOp::Quad { background, .. }
                if *background == fret_core::Paint::Solid(caret_color).into()
        )),
        "expected refocus to paint the caret again"
    );
    assert!(
        !refocus_scene.ops().iter().any(|op| matches!(
            op,
            fret_core::SceneOp::Quad { background, rect, .. }
                if *background == fret_core::Paint::Solid(underline_color).into()
                    && rect.size.height.0 > 0.0
                    && rect.size.height.0 <= 1.1
        )),
        "expected blur to clear preedit so refocus does not repaint an underline"
    );
}

#[test]
fn text_input_draws_preedit_underline_when_composing() {
    let window = AppWindowId::default();
    let bounds = Rect::new(Point::new(Px(0.0), Px(0.0)), Size::new(Px(240.0), Px(80.0)));

    let mut ui = UiTree::new();
    ui.set_window(window);

    let underline_color = fret_core::Color {
        r: 0.2,
        g: 0.8,
        b: 0.3,
        a: 1.0,
    };

    let mut input = TextInput::new().with_text("hello");
    input.set_chrome_style(TextInputStyle {
        preedit_underline_color: underline_color,
        ..Default::default()
    });

    let root = ui.create_node(input);
    ui.set_root(root);
    ui.set_focus(Some(root));

    let mut app = TestHost::new();
    app.set_global(PlatformCapabilities::default());
    let mut text = FakeTextService::default();

    ui.layout_all(&mut app, &mut text, bounds, 1.0);
    let _ = app.take_effects();

    ui.dispatch_event(
        &mut app,
        &mut text,
        &Event::Ime(ImeEvent::Preedit {
            text: "yo".to_string(),
            cursor: Some((2, 2)),
        }),
    );

    let mut scene = fret_core::Scene::default();
    ui.paint(&mut app, &mut text, root, bounds, &mut scene, 1.0);

    let underline_rect = scene.ops().iter().rev().find_map(|op| match op {
        fret_core::SceneOp::Quad {
            rect, background, ..
        } if *background == fret_core::Paint::Solid(underline_color).into()
            && rect.size.height.0 > 0.0
            && rect.size.height.0 <= 1.1 =>
        {
            Some(*rect)
        }
        _ => None,
    });

    let Some(underline_rect) = underline_rect else {
        panic!("expected a preedit underline quad to be present in the scene");
    };

    assert!(
        underline_rect.size.width.0 > 0.1,
        "expected underline width to be > 0 (got {:?})",
        underline_rect.size.width
    );
}

#[test]
fn text_input_draws_preedit_background_when_composing() {
    let window = AppWindowId::default();
    let bounds = Rect::new(Point::new(Px(0.0), Px(0.0)), Size::new(Px(240.0), Px(80.0)));

    let mut ui = UiTree::new();
    ui.set_window(window);

    let bg_color = fret_core::Color {
        r: 0.9,
        g: 0.1,
        b: 0.7,
        a: 0.5,
    };

    let mut input = TextInput::new().with_text("hello");
    input.set_chrome_style(TextInputStyle {
        preedit_bg_color: bg_color,
        ..Default::default()
    });

    let root = ui.create_node(input);
    ui.set_root(root);
    ui.set_focus(Some(root));

    let mut app = TestHost::new();
    app.set_global(PlatformCapabilities::default());
    let mut text = FakeTextService::default();

    ui.layout_all(&mut app, &mut text, bounds, 1.0);
    let _ = app.take_effects();

    ui.dispatch_event(
        &mut app,
        &mut text,
        &Event::Ime(ImeEvent::Preedit {
            text: "yo".to_string(),
            cursor: Some((2, 2)),
        }),
    );

    let mut scene = fret_core::Scene::default();
    ui.paint(&mut app, &mut text, root, bounds, &mut scene, 1.0);

    let preedit_bg = scene.ops().iter().rev().find_map(|op| match op {
        fret_core::SceneOp::Quad {
            rect, background, ..
        } if *background == fret_core::Paint::Solid(bg_color).into()
            && rect.size.height.0 > 1.0 =>
        {
            Some(*rect)
        }
        _ => None,
    });

    assert!(
        preedit_bg.is_some(),
        "expected a preedit background quad to be present in the scene"
    );
}

#[test]
fn ime_commit_replaces_original_selection_after_preedit_starts() {
    let window = AppWindowId::default();
    let node = fret_core::NodeId::default();
    let bounds = Rect::new(Point::new(Px(0.0), Px(0.0)), Size::new(Px(200.0), Px(40.0)));

    let mut app = TestHost::new();
    app.set_global(PlatformCapabilities::default());
    let mut services = FakeTextService::default();
    let mut prevented_default_actions = fret_runtime::DefaultActionSet::default();

    let mut input = TextInput::new();
    input.text = "hello world".to_string();
    input.caret = 5;
    input.selection_anchor = 0;

    let mut cx = event_cx(
        &mut app,
        &mut services,
        node,
        window,
        bounds,
        &mut prevented_default_actions,
    );

    input.event(
        &mut cx,
        &Event::Ime(ImeEvent::Preedit {
            text: "yo".to_string(),
            cursor: Some((0, 2)),
        }),
    );
    input.event(&mut cx, &Event::Ime(ImeEvent::Commit("yo".to_string())));

    assert_eq!(input.text, "yo world");
    assert!(input.ime_replace_range.is_none());
    assert!(!input.is_ime_composing());
}

#[test]
fn ime_delete_surrounding_deletes_base_text_without_clearing_preedit() {
    let window = AppWindowId::default();
    let node = fret_core::NodeId::default();
    let bounds = Rect::new(Point::new(Px(0.0), Px(0.0)), Size::new(Px(200.0), Px(40.0)));

    let mut app = TestHost::new();
    app.set_global(PlatformCapabilities::default());
    let mut services = FakeTextService::default();
    let mut prevented_default_actions = fret_runtime::DefaultActionSet::default();

    let mut input = TextInput::new();
    input.text = "hello".to_string();
    input.caret = 2;
    input.selection_anchor = 2;
    input.preedit = "X".to_string();
    input.preedit_cursor = Some((0, 1));

    let mut cx = event_cx(
        &mut app,
        &mut services,
        node,
        window,
        bounds,
        &mut prevented_default_actions,
    );

    input.event(
        &mut cx,
        &Event::Ime(ImeEvent::DeleteSurrounding {
            before_bytes: 1,
            after_bytes: 2,
        }),
    );

    assert_eq!(input.text, "ho");
    assert_eq!(input.caret, 1);
    assert_eq!(input.selection_anchor, 1);
    assert_eq!(input.preedit, "X");
    assert_eq!(input.preedit_cursor, Some((0, 1)));
}

fn varying_ascii_text(len: usize) -> String {
    let mut out = String::with_capacity(len);
    for i in 0..len {
        let b = b'a' + (u8::try_from(i % 26).unwrap_or(0));
        out.push(b as char);
    }
    out
}

#[test]
fn surrounding_text_snapshot_is_cached_across_frames() {
    let mut input = TextInput::new();
    input.text = varying_ascii_text(5000);
    input.caret = 2500;
    input.selection_anchor = 2500;

    let s1 = <TextInput as Widget<TestHost>>::platform_text_input_snapshot(&input)
        .expect("expected snapshot");
    let s2 = <TextInput as Widget<TestHost>>::platform_text_input_snapshot(&input)
        .expect("expected snapshot");

    let t1 = s1
        .surrounding_text
        .expect("expected surrounding text to be present");
    let t2 = s2
        .surrounding_text
        .expect("expected surrounding text to be present");

    assert!(Arc::ptr_eq(&t1.text, &t2.text));
    assert_eq!(t1.cursor, t2.cursor);
    assert_eq!(t1.anchor, t2.anchor);

    // Changing the caret/anchor should produce a different excerpt (and then cache again).
    input.caret = 2501;
    input.selection_anchor = 2501;
    let s3 = <TextInput as Widget<TestHost>>::platform_text_input_snapshot(&input)
        .expect("expected snapshot");
    let s4 = <TextInput as Widget<TestHost>>::platform_text_input_snapshot(&input)
        .expect("expected snapshot");

    let t3 = s3
        .surrounding_text
        .expect("expected surrounding text to be present");
    let t4 = s4
        .surrounding_text
        .expect("expected surrounding text to be present");

    assert_ne!(t1.text.as_ref(), t3.text.as_ref());
    assert!(Arc::ptr_eq(&t3.text, &t4.text));
    assert_eq!(t3.cursor, t4.cursor);
    assert_eq!(t3.anchor, t4.anchor);
}

#[test]
fn text_input_selection_highlight_uses_square_corners() {
    let window = AppWindowId::default();
    let bounds = Rect::new(Point::new(Px(0.0), Px(0.0)), Size::new(Px(240.0), Px(80.0)));

    let mut ui = UiTree::new();
    ui.set_window(window);

    let selection_color = fret_core::Color {
        r: 0.05,
        g: 0.95,
        b: 0.25,
        a: 0.85,
    };
    let chrome = TextInputStyle {
        corner_radii: fret_core::Corners::all(Px(12.0)),
        selection_color,
        ..Default::default()
    };

    let mut input = TextInput::new().with_text("hello world");
    input.selection_anchor = 0;
    input.caret = 5;
    input.set_chrome_style(chrome);

    let root = ui.create_node(input);
    ui.set_root(root);
    ui.set_focus(Some(root));

    let mut app = TestHost::new();
    app.set_global(PlatformCapabilities::default());
    let mut text = ImeTextService::default();

    ui.layout_all(&mut app, &mut text, bounds, 1.0);

    let mut scene = fret_core::Scene::default();
    ui.paint(&mut app, &mut text, root, bounds, &mut scene, 1.0);

    let selection_quad = scene.ops().iter().find_map(|op| match op {
        fret_core::SceneOp::Quad {
            background,
            corner_radii,
            rect,
            ..
        } if *background == fret_core::Paint::Solid(selection_color).into()
            && rect.size.width.0 > 0.0
            && rect.size.height.0 > 0.0 =>
        {
            Some(*corner_radii)
        }
        _ => None,
    });

    assert_eq!(
        selection_quad,
        Some(fret_core::Corners::all(Px(0.0))),
        "expected selection highlight to not inherit the input corner radius"
    );
}

#[derive(Default)]
struct ImeTextService {}

impl TextService for ImeTextService {
    fn prepare(
        &mut self,
        input: &fret_core::TextInput,
        _constraints: TextConstraints,
    ) -> (fret_core::TextBlobId, TextMetrics) {
        let text = input.text();
        (
            fret_core::TextBlobId::default(),
            TextMetrics {
                size: Size::new(Px(text.len() as f32), Px(10.0)),
                baseline: Px(8.0),
            },
        )
    }

    fn caret_x(&mut self, _blob: fret_core::TextBlobId, index: usize) -> Px {
        Px(index as f32)
    }

    fn hit_test_x(&mut self, _blob: fret_core::TextBlobId, x: Px) -> usize {
        x.0.max(0.0).floor() as usize
    }

    fn release(&mut self, _blob: fret_core::TextBlobId) {}
}

impl fret_core::PathService for ImeTextService {
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

impl fret_core::SvgService for ImeTextService {
    fn register_svg(&mut self, _bytes: &[u8]) -> fret_core::SvgId {
        fret_core::SvgId::default()
    }

    fn unregister_svg(&mut self, _svg: fret_core::SvgId) -> bool {
        false
    }
}

impl fret_core::MaterialService for ImeTextService {
    fn register_material(
        &mut self,
        _desc: fret_core::MaterialDescriptor,
    ) -> Result<fret_core::MaterialId, fret_core::MaterialRegistrationError> {
        Err(fret_core::MaterialRegistrationError::Unsupported)
    }

    fn unregister_material(&mut self, _id: fret_core::MaterialId) -> bool {
        false
    }
}

#[test]
fn ime_cursor_area_moves_with_preedit_cursor() {
    let window = AppWindowId::default();

    let mut ui = UiTree::new();
    ui.set_window(window);

    let root = ui.create_node(TextInput::new().with_text("hello"));
    ui.set_root(root);
    ui.set_focus(Some(root));

    let mut app = TestHost::new();
    app.set_global(PlatformCapabilities::default());
    let mut text = ImeTextService::default();

    let _ = ui.layout(
        &mut app,
        &mut text,
        root,
        Size::new(Px(300.0), Px(40.0)),
        1.0,
    );
    let _ = app.take_effects();

    fn paint_once(
        ui: &mut UiTree<TestHost>,
        root: fret_core::NodeId,
        app: &mut TestHost,
        text: &mut ImeTextService,
    ) -> f32 {
        let mut scene = fret_core::Scene::default();
        ui.paint(
            app,
            text,
            root,
            Rect::new(Point::new(Px(0.0), Px(0.0)), Size::new(Px(300.0), Px(40.0))),
            &mut scene,
            1.0,
        );
        app.take_effects()
            .into_iter()
            .find_map(|e| match e {
                Effect::ImeSetCursorArea { rect, .. } => Some(rect.origin.x.0),
                _ => None,
            })
            .expect("expected an IME cursor area effect")
    }

    ui.dispatch_event(
        &mut app,
        &mut text,
        &Event::Ime(ImeEvent::Preedit {
            text: "abcd".to_string(),
            cursor: Some((0, 0)),
        }),
    );
    let x0 = paint_once(&mut ui, root, &mut app, &mut text);

    ui.dispatch_event(
        &mut app,
        &mut text,
        &Event::Ime(ImeEvent::Preedit {
            text: "abcd".to_string(),
            cursor: Some((0, 2)),
        }),
    );
    let x2 = paint_once(&mut ui, root, &mut app, &mut text);

    assert!(
        (x2 - x0 - 2.0).abs() < 0.001,
        "expected IME cursor x to move by preedit prefix width"
    );
}

#[test]
fn double_click_cancels_preedit_and_maps_hit_test_from_display_to_base_indices() {
    let window = AppWindowId::default();
    let node = fret_core::NodeId::default();
    let bounds = Rect::new(Point::new(Px(0.0), Px(0.0)), Size::new(Px(200.0), Px(40.0)));

    let mut app = TestHost::new();
    app.set_global(PlatformCapabilities::default());
    let mut services = ImeTextService::default();
    let mut prevented_default_actions = fret_runtime::DefaultActionSet::default();

    let mut input = TextInput::new();
    input.text = "hello world".to_string();
    input.caret = 5;
    input.selection_anchor = 5;
    input.text_blob = Some(fret_core::TextBlobId::default());
    input.last_bounds = bounds;
    input.chrome_style.padding.left = Px(0.0);

    let mut cx = event_cx(
        &mut app,
        &mut services,
        node,
        window,
        bounds,
        &mut prevented_default_actions,
    );

    input.event(
        &mut cx,
        &Event::Ime(ImeEvent::Preedit {
            text: "yo".to_string(),
            cursor: Some((0, 2)),
        }),
    );
    assert!(input.is_ime_composing());
    // The preedit event invalidates text blobs; restore a blob id so the pointer path uses
    // hit-testing instead of the caret-stop fallback.
    input.text_blob = Some(fret_core::TextBlobId::default());

    // The composed display text would be: "helloyo world".
    // Click after the inserted preedit; the display index should be shifted by the preedit length.
    input.event(
        &mut cx,
        &Event::Pointer(fret_core::PointerEvent::Down {
            position: Point::new(Px(8.0), Px(5.0)),
            button: fret_core::MouseButton::Left,
            modifiers: fret_core::Modifiers::default(),
            click_count: 2,
            pointer_id: fret_core::PointerId(0),
            pointer_type: fret_core::PointerType::Mouse,
        }),
    );

    assert!(
        !input.is_ime_composing(),
        "expected double-click selection to cancel inline preedit deterministically"
    );
    assert!(input.preedit.is_empty());
    assert!(input.preedit_cursor.is_none());
    assert_eq!(
        (input.selection_anchor, input.caret),
        (6, 11),
        "expected display hit-test index to map to base indices after cancelling preedit"
    );
}

#[test]
fn command_navigation_cancels_preedit_deterministically() {
    let window = AppWindowId::default();
    let node = fret_core::NodeId::default();

    let mut app = TestHost::new();
    app.set_global(PlatformCapabilities::default());
    let mut services = ImeTextService::default();
    let mut tree = UiTree::new();

    let mut input = TextInput::new();
    input.text = "hello".to_string();
    input.caret = input.text.len();
    input.selection_anchor = input.caret;

    input.event(
        &mut event_cx(
            &mut app,
            &mut services,
            node,
            window,
            Rect::default(),
            &mut fret_runtime::DefaultActionSet::default(),
        ),
        &Event::Ime(ImeEvent::Preedit {
            text: "yo".to_string(),
            cursor: Some((0, 2)),
        }),
    );
    assert!(input.is_ime_composing());

    let mut cx = command_cx(&mut app, &mut services, &mut tree, node, window);
    let handled = input.command(&mut cx, &CommandId::new("text.move_left"));
    assert!(handled);
    assert!(
        !input.is_ime_composing(),
        "expected command-driven navigation to cancel preedit deterministically"
    );
    assert_eq!(input.preedit, "");
    assert_eq!(input.preedit_cursor, None);
    assert_eq!(input.caret, "hell".len());
    assert_eq!(input.selection_anchor, input.caret);
}

#[test]
fn vertical_navigation_command_does_not_cancel_preedit() {
    let window = AppWindowId::default();
    let node = fret_core::NodeId::default();

    let mut app = TestHost::new();
    app.set_global(PlatformCapabilities::default());
    let mut services = ImeTextService::default();
    let mut tree = UiTree::new();

    let mut input = TextInput::new();
    input.text = "hello".to_string();
    input.caret = input.text.len();
    input.selection_anchor = input.caret;

    input.event(
        &mut event_cx(
            &mut app,
            &mut services,
            node,
            window,
            Rect::default(),
            &mut fret_runtime::DefaultActionSet::default(),
        ),
        &Event::Ime(ImeEvent::Preedit {
            text: "yo".to_string(),
            cursor: Some((0, 2)),
        }),
    );
    assert!(input.is_ime_composing());

    let mut cx = command_cx(&mut app, &mut services, &mut tree, node, window);
    let handled = input.command(&mut cx, &CommandId::new("text.move_up"));
    assert!(handled);
    assert!(
        input.is_ime_composing(),
        "expected vertical navigation to be reserved for IME arbitration while composing"
    );
    assert_eq!(input.preedit, "yo");
    assert_eq!(input.caret, input.text.len());
    assert_eq!(input.selection_anchor, input.caret);
}

#[test]
fn copy_command_does_not_cancel_preedit() {
    let window = AppWindowId::default();
    let node = fret_core::NodeId::default();

    let mut app = TestHost::new();
    app.set_global(PlatformCapabilities::default());
    let mut services = ImeTextService::default();
    let mut tree = UiTree::new();

    let mut input = TextInput::new();
    input.text = "hello".to_string();
    input.caret = input.text.len();
    input.selection_anchor = input.caret;

    input.event(
        &mut event_cx(
            &mut app,
            &mut services,
            node,
            window,
            Rect::default(),
            &mut fret_runtime::DefaultActionSet::default(),
        ),
        &Event::Ime(ImeEvent::Preedit {
            text: "yo".to_string(),
            cursor: Some((0, 2)),
        }),
    );
    assert!(input.is_ime_composing());

    let mut cx = command_cx(&mut app, &mut services, &mut tree, node, window);
    let handled = input.command(&mut cx, &CommandId::new("text.copy"));
    assert!(handled);
    assert!(
        input.is_ime_composing(),
        "expected copy to preserve preedit (non-mutating command)"
    );
}

#[test]
fn ime_cursor_area_is_in_visual_space_after_render_transform() {
    let window = AppWindowId::default();
    let bounds = Rect::new(Point::new(Px(0.0), Px(0.0)), Size::new(Px(300.0), Px(40.0)));

    fn paint_ime_origin(
        transform: fret_core::Transform2D,
        bounds: Rect,
        window: AppWindowId,
    ) -> Point {
        let mut ui = UiTree::new();
        ui.set_window(window);

        let root = ui.create_node(RenderTransformWrapper::new(transform));
        let input = ui.create_node(TextInput::new().with_text("hello"));
        ui.add_child(root, input);
        ui.set_root(root);
        ui.set_focus(Some(input));

        let mut app = TestHost::new();
        app.set_global(PlatformCapabilities::default());
        let mut text = ImeTextService::default();

        ui.layout_all(&mut app, &mut text, bounds, 1.0);
        let _ = app.take_effects();

        let mut scene = fret_core::Scene::default();
        ui.paint(&mut app, &mut text, root, bounds, &mut scene, 1.0);

        app.take_effects()
            .into_iter()
            .find_map(|e| match e {
                Effect::ImeSetCursorArea { rect, .. } => Some(rect.origin),
                _ => None,
            })
            .expect("expected an IME cursor area effect")
    }

    let base = paint_ime_origin(fret_core::Transform2D::IDENTITY, bounds, window);
    let dx = Px(50.0);
    let dy = Px(20.0);
    let translated = paint_ime_origin(
        fret_core::Transform2D::translation(Point::new(dx, dy)),
        bounds,
        window,
    );

    assert!(
        (translated.x.0 - base.x.0 - dx.0).abs() < 0.001,
        "expected IME cursor x to include render transform translation"
    );
    assert!(
        (translated.y.0 - base.y.0 - dy.0).abs() < 0.001,
        "expected IME cursor y to include render transform translation"
    );
}

#[test]
fn ime_cursor_area_scrolls_horizontally_to_keep_caret_visible() {
    let window = AppWindowId::default();
    let bounds = Rect::new(Point::new(Px(0.0), Px(0.0)), Size::new(Px(20.0), Px(40.0)));

    let mut ui = UiTree::new();
    ui.set_window(window);

    let root = ui.create_node(RenderTransformWrapper::new(
        fret_core::Transform2D::IDENTITY,
    ));
    let mut widget = TextInput::new().with_text("abcdefghijklmnopqrstuvwxyz");
    widget.set_chrome_style(crate::TextInputStyle {
        padding: fret_core::Edges::all(Px(0.0)),
        ..Default::default()
    });
    let input = ui.create_node(widget);
    ui.add_child(root, input);
    ui.set_root(root);
    ui.set_focus(Some(input));

    let mut app = TestHost::new();
    app.set_global(PlatformCapabilities::default());
    let mut text = ImeTextService::default();

    ui.layout_all(&mut app, &mut text, bounds, 1.0);
    let _ = app.take_effects();

    let mut scene = fret_core::Scene::default();
    ui.paint_all(&mut app, &mut text, bounds, &mut scene, 1.0);

    let origin = app
        .take_effects()
        .into_iter()
        .find_map(|e| match e {
            Effect::ImeSetCursorArea { rect, .. } => Some(rect.origin),
            _ => None,
        })
        .expect("expected an IME cursor area effect");

    assert!(
        origin.x.0 <= bounds.origin.x.0 + bounds.size.width.0 + 0.001,
        "expected IME cursor x to remain within the text input bounds after horizontal scrolling"
    );
}

#[test]
fn dragging_selection_autoscrolls_horizontally_beyond_viewport() {
    let window = AppWindowId::default();
    let bounds = Rect::new(Point::new(Px(0.0), Px(0.0)), Size::new(Px(20.0), Px(40.0)));

    let mut ui = UiTree::new();
    ui.set_window(window);

    let root = ui.create_node(RenderTransformWrapper::new(
        fret_core::Transform2D::IDENTITY,
    ));
    let mut widget = TextInput::new().with_text("abcdefghijklmnopqrstuvwxyz");
    widget.set_chrome_style(crate::TextInputStyle {
        padding: fret_core::Edges::all(Px(0.0)),
        ..Default::default()
    });
    widget.caret = 0;
    widget.selection_anchor = 0;
    widget.offset_x = Px(0.0);
    let input = ui.create_node(widget);
    ui.add_child(root, input);
    ui.set_root(root);
    ui.set_focus(Some(input));

    let mut app = TestHost::new();
    app.set_global(PlatformCapabilities::default());
    let mut text = ImeTextService::default();

    ui.layout_all(&mut app, &mut text, bounds, 1.0);
    let _ = app.take_effects();

    let mut scene = fret_core::Scene::default();
    ui.paint_all(&mut app, &mut text, bounds, &mut scene, 1.0);
    let _ = app.take_effects();

    ui.dispatch_event(
        &mut app,
        &mut text,
        &Event::Pointer(fret_core::PointerEvent::Down {
            position: Point::new(Px(0.0), Px(10.0)),
            button: fret_core::MouseButton::Left,
            modifiers: fret_core::Modifiers::default(),
            click_count: 1,
            pointer_id: fret_core::PointerId(0),
            pointer_type: fret_core::PointerType::Mouse,
        }),
    );

    let mut token = app
        .take_effects()
        .into_iter()
        .find_map(|e| match e {
            Effect::SetTimer { token, .. } => Some(token),
            _ => None,
        })
        .expect("expected selection autoscroll to schedule a timer on pointer down");

    ui.dispatch_event(
        &mut app,
        &mut text,
        &Event::Pointer(fret_core::PointerEvent::Move {
            position: Point::new(Px(13.0), Px(10.0)),
            buttons: fret_core::MouseButtons {
                left: true,
                ..Default::default()
            },
            modifiers: fret_core::Modifiers::default(),
            pointer_id: fret_core::PointerId(0),
            pointer_type: fret_core::PointerType::Mouse,
        }),
    );
    let _ = app.take_effects();

    let mut scene = fret_core::Scene::default();
    ui.paint_all(&mut app, &mut text, bounds, &mut scene, 1.0);

    let snap0 = app
        .global::<fret_runtime::WindowTextInputSnapshotService>()
        .and_then(|svc| svc.snapshot(window))
        .cloned()
        .expect("expected a window text input snapshot after paint");
    let (_, focus0) = snap0
        .selection_utf16
        .expect("expected selection to be present for focused text input");
    assert!(
        focus0 < snap0.text_len_utf16,
        "expected a partial selection before timer-driven auto-scroll ticks"
    );

    for _ in 0..5 {
        ui.dispatch_event(&mut app, &mut text, &Event::Timer { token });

        token = app
            .take_effects()
            .into_iter()
            .find_map(|e| match e {
                Effect::SetTimer { token, .. } => Some(token),
                _ => None,
            })
            .expect("expected selection autoscroll to schedule the next timer tick");

        let mut scene = fret_core::Scene::default();
        ui.paint_all(&mut app, &mut text, bounds, &mut scene, 1.0);
    }

    let snap1 = app
        .global::<fret_runtime::WindowTextInputSnapshotService>()
        .and_then(|svc| svc.snapshot(window))
        .cloned()
        .expect("expected a window text input snapshot after paint");
    let (_, focus1) = snap1
        .selection_utf16
        .expect("expected selection to be present for focused text input");

    assert!(
        focus1 > focus0,
        "expected dragging selection near the right edge to auto-scroll and extend the selection"
    );
}
