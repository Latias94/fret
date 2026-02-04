use super::*;

use crate::UiTree;
use crate::test_host::TestHost;
use crate::widget::Widget;
use fret_core::{
    AppWindowId, Event, ImeEvent, Point, Px, Rect, Size, TextConstraints, TextMetrics, TextService,
};
use fret_runtime::{Effect, PlatformCapabilities};
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
        prevented_default_actions,
        pointer_id: None,
        children: &[],
        focus: Some(node),
        captured: None,
        bounds,
        invalidations: Vec::new(),
        requested_focus: None,
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
            .any(|e| matches!(e, Effect::ClipboardSetText { text } if text == "hello")),
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
