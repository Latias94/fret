use super::*;
use crate::UiTree;
use crate::test_host::TestHost;
use crate::widget::Widget;
use fret_core::{
    AppWindowId, CaretAffinity, Event, Point, Px, Rect, Scene, Size, TextConstraints, TextMetrics,
    TextService,
};
use fret_runtime::{Effect, PlatformCapabilities};

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
struct FakeTextService {}

impl TextService for FakeTextService {
    fn prepare(
        &mut self,
        _input: &fret_core::TextInput,
        _constraints: TextConstraints,
    ) -> (fret_core::TextBlobId, TextMetrics) {
        (
            fret_core::TextBlobId::default(),
            TextMetrics {
                size: Size::new(Px(10.0), Px(10.0)),
                baseline: Px(8.0),
            },
        )
    }

    fn caret_rect(
        &mut self,
        _blob: fret_core::TextBlobId,
        index: usize,
        _affinity: CaretAffinity,
    ) -> Rect {
        Rect::new(
            Point::new(Px(index as f32), Px(0.0)),
            Size::new(Px(1.0), Px(10.0)),
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

fn command_cx<'a>(
    app: &'a mut TestHost,
    services: &'a mut FakeTextService,
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
fn text_area_hover_sets_text_cursor_effect() {
    let window = AppWindowId::default();

    let mut ui = UiTree::new();
    ui.set_window(window);

    let root = ui.create_node(TextArea::default());
    ui.set_root(root);

    let mut app = TestHost::new();
    app.set_global(PlatformCapabilities::default());
    let mut text = FakeTextService::default();

    let _ = ui.layout(
        &mut app,
        &mut text,
        root,
        Size::new(Px(300.0), Px(200.0)),
        1.0,
    );
    let _ = app.take_effects();

    ui.dispatch_event(
        &mut app,
        &mut text,
        &Event::Pointer(fret_core::PointerEvent::Move {
            position: Point::new(Px(12.0), Px(12.0)),
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
        "expected a text cursor effect when hovering a text area"
    );
}

#[test]
fn ime_cursor_area_is_in_visual_space_after_render_transform() {
    let window = AppWindowId::default();
    let bounds = Rect::new(
        Point::new(Px(0.0), Px(0.0)),
        Size::new(Px(300.0), Px(200.0)),
    );

    fn paint_ime_origin(
        transform: fret_core::Transform2D,
        bounds: Rect,
        window: AppWindowId,
    ) -> Point {
        let mut ui = UiTree::new();
        ui.set_window(window);

        let root = ui.create_node(RenderTransformWrapper::new(transform));
        let area = ui.create_node(TextArea::new("hello"));
        ui.add_child(root, area);
        ui.set_root(root);
        ui.set_focus(Some(area));

        let mut app = TestHost::new();
        app.set_global(PlatformCapabilities::default());
        let mut services = FakeTextService::default();

        ui.layout_all(&mut app, &mut services, bounds, 1.0);
        let _ = app.take_effects();

        let mut scene = Scene::default();
        ui.paint(&mut app, &mut services, root, bounds, &mut scene, 1.0);

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
fn text_area_copy_clamps_out_of_range_selection_indices() {
    let window = AppWindowId::default();
    let node = fret_core::NodeId::default();

    let mut ui = UiTree::new();
    ui.set_window(window);

    let mut app = TestHost::new();
    app.set_global(PlatformCapabilities::default());
    let mut services = FakeTextService::default();

    let mut area = TextArea::default();
    area.text = "hello\nworld".to_string();
    area.selection_anchor = 0;
    area.caret = 999;

    let mut cx = command_cx(&mut app, &mut services, &mut ui, node, window);
    assert!(
        area.command(&mut cx, &fret_runtime::CommandId::from("edit.copy")),
        "expected edit.copy to be handled"
    );

    assert!(
        app.take_effects()
            .iter()
            .any(|e| matches!(e, Effect::ClipboardSetText { text } if text == "hello\nworld")),
        "expected edit.copy to clamp indices and copy the selected text"
    );
    assert_eq!(area.selection_anchor, 0);
    assert_eq!(area.caret, 11);
}

#[test]
fn ime_cursor_area_moves_with_preedit_cursor() {
    let window = AppWindowId::default();

    let mut ui = UiTree::new();
    ui.set_window(window);

    let root = ui.create_node(TextArea::default());
    ui.set_root(root);
    ui.set_focus(Some(root));

    let mut app = TestHost::new();
    app.set_global(PlatformCapabilities::default());
    let mut text = FakeTextService::default();

    let _ = ui.layout(
        &mut app,
        &mut text,
        root,
        Size::new(Px(300.0), Px(200.0)),
        1.0,
    );
    let _ = app.take_effects();

    fn paint_once(
        ui: &mut UiTree<TestHost>,
        root: fret_core::NodeId,
        app: &mut TestHost,
        text: &mut FakeTextService,
    ) -> f32 {
        let mut scene = Scene::default();
        ui.paint(
            app,
            text,
            root,
            Rect::new(
                Point::new(Px(0.0), Px(0.0)),
                Size::new(Px(300.0), Px(200.0)),
            ),
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
        &Event::Ime(fret_core::ImeEvent::Preedit {
            text: "abcd".to_string(),
            cursor: Some((0, 0)),
        }),
    );
    let x0 = paint_once(&mut ui, root, &mut app, &mut text);

    ui.dispatch_event(
        &mut app,
        &mut text,
        &Event::Ime(fret_core::ImeEvent::Preedit {
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
fn ime_commit_clears_preedit_state() {
    let window = AppWindowId::default();

    let mut ui = UiTree::new();
    ui.set_window(window);

    let root = ui.create_node(TextArea::default());
    ui.set_root(root);
    ui.set_focus(Some(root));

    let mut app = TestHost::new();
    app.set_global(PlatformCapabilities::default());
    let mut text = FakeTextService::default();

    let _ = ui.layout(
        &mut app,
        &mut text,
        root,
        Size::new(Px(300.0), Px(200.0)),
        1.0,
    );
    let _ = app.take_effects();

    fn paint_once(
        ui: &mut UiTree<TestHost>,
        root: fret_core::NodeId,
        app: &mut TestHost,
        text: &mut FakeTextService,
    ) -> Option<f32> {
        let mut scene = Scene::default();
        ui.paint(
            app,
            text,
            root,
            Rect::new(
                Point::new(Px(0.0), Px(0.0)),
                Size::new(Px(300.0), Px(200.0)),
            ),
            &mut scene,
            1.0,
        );
        app.take_effects().into_iter().find_map(|e| match e {
            Effect::ImeSetCursorArea { rect, .. } => Some(rect.origin.x.0),
            _ => None,
        })
    }

    ui.dispatch_event(
        &mut app,
        &mut text,
        &Event::Ime(fret_core::ImeEvent::Preedit {
            text: "abcd".to_string(),
            cursor: Some((0, 4)),
        }),
    );
    let x_preedit =
        paint_once(&mut ui, root, &mut app, &mut text).expect("expected an IME cursor area effect");

    ui.dispatch_event(
        &mut app,
        &mut text,
        &Event::Ime(fret_core::ImeEvent::Commit("abcd".to_string())),
    );
    let _ = app.take_effects();

    let x_after_commit = paint_once(&mut ui, root, &mut app, &mut text).unwrap_or(x_preedit);
    assert!(
        (x_after_commit - x_preedit).abs() < 0.001,
        "expected preedit to be cleared on commit (otherwise cursor area jumps)"
    );
}

fn event_cx<'a>(
    app: &'a mut TestHost,
    services: &'a mut dyn fret_core::UiServices,
    node: fret_core::NodeId,
    window: fret_core::AppWindowId,
    bounds: Rect,
    prevented_default_actions: &'a mut fret_runtime::DefaultActionSet,
) -> EventCx<'a, TestHost> {
    EventCx {
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

#[test]
fn ime_commit_normalizes_newlines_to_lf() {
    let window = AppWindowId::default();
    let node = fret_core::NodeId::default();
    let bounds = Rect::new(
        Point::new(Px(0.0), Px(0.0)),
        Size::new(Px(300.0), Px(200.0)),
    );

    let mut app = TestHost::new();
    app.set_global(PlatformCapabilities::default());
    let mut services = FakeTextService::default();
    let mut prevented_default_actions = fret_runtime::DefaultActionSet::default();

    let mut area = TextArea::default();
    let mut cx = event_cx(
        &mut app,
        &mut services,
        node,
        window,
        bounds,
        &mut prevented_default_actions,
    );

    area.event(
        &mut cx,
        &Event::Ime(fret_core::ImeEvent::Commit("a\r\nb\rc".to_string())),
    );

    assert_eq!(area.text(), "a\nb\nc");
    assert!(area.preedit.is_empty());
    assert!(area.preedit_cursor.is_none());
}

#[test]
fn ime_delete_surrounding_deletes_base_text_without_clearing_preedit() {
    let window = AppWindowId::default();
    let node = fret_core::NodeId::default();
    let bounds = Rect::new(
        Point::new(Px(0.0), Px(0.0)),
        Size::new(Px(300.0), Px(200.0)),
    );

    let mut app = TestHost::new();
    app.set_global(PlatformCapabilities::default());
    let mut services = FakeTextService::default();
    let mut prevented_default_actions = fret_runtime::DefaultActionSet::default();

    let mut area = TextArea::new("hello\nworld");
    area.caret = 2;
    area.selection_anchor = 2;
    area.preedit = "X".to_string();
    area.preedit_cursor = Some((0, 1));

    let mut cx = event_cx(
        &mut app,
        &mut services,
        node,
        window,
        bounds,
        &mut prevented_default_actions,
    );

    area.event(
        &mut cx,
        &Event::Ime(fret_core::ImeEvent::DeleteSurrounding {
            before_bytes: 1,
            after_bytes: 2,
        }),
    );

    assert_eq!(area.text(), "ho\nworld");
    assert_eq!(area.caret, 1);
    assert_eq!(area.selection_anchor, 1);
    assert_eq!(area.preedit, "X");
    assert_eq!(area.preedit_cursor, Some((0, 1)));
}

#[test]
fn ime_commit_replaces_selection_and_clears_it() {
    let window = AppWindowId::default();
    let node = fret_core::NodeId::default();
    let bounds = Rect::new(
        Point::new(Px(0.0), Px(0.0)),
        Size::new(Px(300.0), Px(200.0)),
    );

    let mut app = TestHost::new();
    app.set_global(PlatformCapabilities::default());
    let mut services = FakeTextService::default();
    let mut prevented_default_actions = fret_runtime::DefaultActionSet::default();

    let mut area = TextArea::new("hello world");
    area.caret = 5;
    area.selection_anchor = 0;

    let mut cx = event_cx(
        &mut app,
        &mut services,
        node,
        window,
        bounds,
        &mut prevented_default_actions,
    );

    area.event(
        &mut cx,
        &Event::Ime(fret_core::ImeEvent::Commit("yo".to_string())),
    );

    assert_eq!(area.text(), "yo world");
    assert_eq!(area.caret, 2);
    assert_eq!(area.selection_anchor, 2);
    assert!(area.preedit.is_empty());
}

#[test]
fn text_area_double_click_selection_respects_text_boundary_mode() {
    let window = AppWindowId::default();
    let node = fret_core::NodeId::default();
    let bounds = Rect::new(
        Point::new(Px(0.0), Px(0.0)),
        Size::new(Px(300.0), Px(200.0)),
    );

    let mut app = TestHost::new();
    app.set_global(PlatformCapabilities::default());
    let mut services = FakeTextService::default();
    let mut prevented_default_actions = fret_runtime::DefaultActionSet::default();

    let mut area = TextArea::new("can't\nsecond".to_string());
    area.caret = 0;
    area.selection_anchor = 0;

    let mut cx = event_cx(
        &mut app,
        &mut services,
        node,
        window,
        bounds,
        &mut prevented_default_actions,
    );
    cx.input_ctx.text_boundary_mode = fret_runtime::TextBoundaryMode::UnicodeWord;

    area.event(
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
    assert_eq!((area.selection_anchor, area.caret), (0, 5));

    let mut area = TextArea::new("can't\nsecond".to_string());
    area.caret = 0;
    area.selection_anchor = 0;
    let mut cx = event_cx(
        &mut app,
        &mut services,
        node,
        window,
        bounds,
        &mut prevented_default_actions,
    );
    cx.input_ctx.text_boundary_mode = fret_runtime::TextBoundaryMode::Identifier;

    area.event(
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
    assert_eq!((area.selection_anchor, area.caret), (0, 3));
}

#[test]
fn preedit_does_not_mutate_buffer_until_commit() {
    let window = AppWindowId::default();
    let node = fret_core::NodeId::default();
    let bounds = Rect::new(
        Point::new(Px(0.0), Px(0.0)),
        Size::new(Px(300.0), Px(200.0)),
    );

    let mut app = TestHost::new();
    app.set_global(PlatformCapabilities::default());
    let mut services = FakeTextService::default();
    let mut prevented_default_actions = fret_runtime::DefaultActionSet::default();

    let mut area = TextArea::new("abc");
    area.caret = 1;
    area.selection_anchor = 1;

    let mut cx = event_cx(
        &mut app,
        &mut services,
        node,
        window,
        bounds,
        &mut prevented_default_actions,
    );

    area.event(
        &mut cx,
        &Event::Ime(fret_core::ImeEvent::Preedit {
            text: "ZZ".to_string(),
            cursor: Some((0, 2)),
        }),
    );

    assert_eq!(area.text(), "abc");
    assert_eq!(area.preedit, "ZZ");

    area.event(
        &mut cx,
        &Event::Ime(fret_core::ImeEvent::Commit("ZZ".to_string())),
    );

    assert_eq!(area.text(), "aZZbc");
    assert!(area.preedit.is_empty());
    assert!(area.preedit_cursor.is_none());
}

#[test]
fn ime_commit_replaces_original_selection_after_preedit_starts() {
    let window = AppWindowId::default();
    let node = fret_core::NodeId::default();
    let bounds = Rect::new(
        Point::new(Px(0.0), Px(0.0)),
        Size::new(Px(300.0), Px(200.0)),
    );

    let mut app = TestHost::new();
    app.set_global(PlatformCapabilities::default());
    let mut services = FakeTextService::default();
    let mut prevented_default_actions = fret_runtime::DefaultActionSet::default();

    let mut area = TextArea::new("hello world");
    area.caret = 5;
    area.selection_anchor = 0;

    let mut cx = event_cx(
        &mut app,
        &mut services,
        node,
        window,
        bounds,
        &mut prevented_default_actions,
    );

    area.event(
        &mut cx,
        &Event::Ime(fret_core::ImeEvent::Preedit {
            text: "yo".to_string(),
            cursor: Some((0, 2)),
        }),
    );

    area.event(
        &mut cx,
        &Event::Ime(fret_core::ImeEvent::Commit("yo".to_string())),
    );

    assert_eq!(area.text(), "yo world");
    assert!(area.preedit.is_empty());
    assert!(area.ime_replace_range.is_none());
}

#[test]
fn clipboard_text_normalizes_newlines_to_lf() {
    let window = AppWindowId::default();
    let node = fret_core::NodeId::default();
    let bounds = Rect::new(
        Point::new(Px(0.0), Px(0.0)),
        Size::new(Px(300.0), Px(200.0)),
    );

    let mut app = TestHost::new();
    app.set_global(PlatformCapabilities::default());
    let mut services = FakeTextService::default();
    let mut prevented_default_actions = fret_runtime::DefaultActionSet::default();

    let mut area = TextArea::default();
    let mut cx = event_cx(
        &mut app,
        &mut services,
        node,
        window,
        bounds,
        &mut prevented_default_actions,
    );

    let token = fret_runtime::ClipboardToken(1);
    area.pending_clipboard_token = Some(token);
    area.event(
        &mut cx,
        &Event::ClipboardText {
            token,
            text: "a\r\nb\rc".to_string(),
        },
    );

    assert_eq!(area.text(), "a\nb\nc");
}

#[derive(Default)]
struct YTextService {}

impl TextService for YTextService {
    fn prepare(
        &mut self,
        input: &fret_core::TextInput,
        _constraints: TextConstraints,
    ) -> (fret_core::TextBlobId, TextMetrics) {
        let text = input.text();
        (
            fret_core::TextBlobId::default(),
            TextMetrics {
                size: Size::new(Px(text.len() as f32), Px(1000.0)),
                baseline: Px(8.0),
            },
        )
    }

    fn caret_rect(
        &mut self,
        _blob: fret_core::TextBlobId,
        index: usize,
        _affinity: CaretAffinity,
    ) -> Rect {
        Rect::new(
            Point::new(Px(0.0), Px(index as f32)),
            Size::new(Px(1.0), Px(10.0)),
        )
    }

    fn release(&mut self, _blob: fret_core::TextBlobId) {}
}

impl fret_core::PathService for YTextService {
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

impl fret_core::SvgService for YTextService {
    fn register_svg(&mut self, _bytes: &[u8]) -> fret_core::SvgId {
        fret_core::SvgId::default()
    }

    fn unregister_svg(&mut self, _svg: fret_core::SvgId) -> bool {
        false
    }
}

#[test]
fn ime_cursor_area_reflects_scroll_offset_in_paint_space() {
    let window = AppWindowId::default();
    let node = fret_core::NodeId::default();
    let bounds = Rect::new(
        Point::new(Px(0.0), Px(0.0)),
        Size::new(Px(300.0), Px(200.0)),
    );

    let mut app = TestHost::new();
    app.set_global(PlatformCapabilities::default());
    let mut services = YTextService::default();

    let mut area = TextArea::new("hello");
    area.caret = 50;
    area.selection_anchor = 50;
    area.ensure_caret_visible = false;
    area.last_content_height = Px(1000.0);
    area.last_viewport_height = Px(200.0);
    area.last_bounds = bounds;

    fn paint_once(
        area: &mut TextArea,
        app: &mut TestHost,
        services: &mut YTextService,
        node: fret_core::NodeId,
        window: fret_core::AppWindowId,
        bounds: Rect,
    ) -> Rect {
        let mut scene = Scene::default();
        let mut observe_model = |_id, _inv| {};
        let mut observe_global = |_id, _inv| {};
        let mut tree = crate::tree::UiTree::<TestHost>::default();

        let mut cx = crate::widget::PaintCx {
            app,
            tree: &mut tree,
            node,
            window: Some(window),
            focus: Some(node),
            children: &[],
            bounds,
            scale_factor: 1.0,
            accumulated_transform: fret_core::Transform2D::IDENTITY,
            children_render_transform: None,
            services,
            observe_model: &mut observe_model,
            observe_global: &mut observe_global,
            scene: &mut scene,
        };

        area.paint(&mut cx);

        app.take_effects()
            .into_iter()
            .find_map(|e| match e {
                Effect::ImeSetCursorArea { rect, .. } => Some(rect),
                _ => None,
            })
            .expect("expected an IME cursor area effect")
    }

    let y0 = paint_once(&mut area, &mut app, &mut services, node, window, bounds)
        .origin
        .y
        .0;

    {
        let mut prevented_default_actions = fret_runtime::DefaultActionSet::default();
        let mut cx = event_cx(
            &mut app,
            &mut services,
            node,
            window,
            bounds,
            &mut prevented_default_actions,
        );
        area.event(
            &mut cx,
            &Event::Pointer(fret_core::PointerEvent::Wheel {
                position: Point::new(Px(0.0), Px(0.0)),
                delta: Point::new(Px(0.0), Px(-10.0)),
                modifiers: fret_core::Modifiers::default(),
                pointer_id: fret_core::PointerId(0),
                pointer_type: fret_core::PointerType::Mouse,
            }),
        );
    }

    let y1 = paint_once(&mut area, &mut app, &mut services, node, window, bounds)
        .origin
        .y
        .0;

    assert!(
        (y1 - (y0 - 10.0)).abs() < 0.001,
        "expected IME cursor area to move with scroll offset"
    );
}

#[test]
fn semantics_value_and_composition_include_inline_preedit() {
    let window = AppWindowId::default();

    let mut ui = UiTree::new();
    ui.set_window(window);

    let root = ui.create_node(TextArea::new("abc"));
    ui.set_root(root);
    ui.set_focus(Some(root));

    let mut app = TestHost::new();
    app.set_global(PlatformCapabilities::default());
    let mut services = FakeTextService::default();

    let bounds = Rect::new(Point::new(Px(0.0), Px(0.0)), Size::new(Px(200.0), Px(80.0)));

    ui.layout_all(&mut app, &mut services, bounds, 1.0);
    let _ = app.take_effects();

    ui.dispatch_event(
        &mut app,
        &mut services,
        &Event::SetTextSelection {
            anchor: 1,
            focus: 1,
        },
    );
    ui.dispatch_event(
        &mut app,
        &mut services,
        &Event::Ime(fret_core::ImeEvent::Preedit {
            text: "ZZ".to_string(),
            cursor: Some((0, 1)),
        }),
    );

    ui.request_semantics_snapshot();
    ui.layout_all(&mut app, &mut services, bounds, 1.0);

    let snap = ui.semantics_snapshot().expect("semantics snapshot");
    snap.validate().expect("valid semantics snapshot");

    let node = snap
        .nodes
        .iter()
        .find(|n| n.id == root)
        .expect("text area node");

    assert_eq!(node.role, fret_core::SemanticsRole::TextField);
    assert_eq!(node.value.as_deref(), Some("aZZbc"));
    assert_eq!(node.text_selection, Some((2, 2)));
    assert_eq!(node.text_composition, Some((1, 3)));
}
