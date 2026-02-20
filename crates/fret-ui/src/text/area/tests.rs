use super::*;
use crate::UiTree;
use crate::test_host::TestHost;
use crate::widget::Widget;
use fret_core::{
    AppWindowId, CaretAffinity, Color, Event, Paint, Point, Px, Rect, Scene, SceneOp, Size,
    TextConstraints, TextLineMetrics, TextMetrics, TextService,
};
use fret_runtime::{Effect, PlatformCapabilities};
use slotmap::KeyData;
use std::collections::HashMap;

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
struct ZeroHeightCaretTextService {}

impl TextService for ZeroHeightCaretTextService {
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

    fn first_line_metrics(&mut self, _blob: fret_core::TextBlobId) -> Option<TextLineMetrics> {
        Some(TextLineMetrics {
            ascent: Px(8.0),
            descent: Px(2.0),
            line_height: Px(10.0),
        })
    }

    fn caret_rect(
        &mut self,
        _blob: fret_core::TextBlobId,
        index: usize,
        _affinity: CaretAffinity,
    ) -> Rect {
        Rect::new(
            Point::new(Px(index as f32), Px(0.0)),
            Size::new(Px(1.0), Px(0.0)),
        )
    }

    fn release(&mut self, _blob: fret_core::TextBlobId) {}
}

impl fret_core::PathService for ZeroHeightCaretTextService {
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

impl fret_core::SvgService for ZeroHeightCaretTextService {
    fn register_svg(&mut self, _bytes: &[u8]) -> fret_core::SvgId {
        fret_core::SvgId::default()
    }

    fn unregister_svg(&mut self, _svg: fret_core::SvgId) -> bool {
        false
    }
}

impl fret_core::MaterialService for ZeroHeightCaretTextService {
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
struct ZeroHeightSelectionRectTextService {}

impl TextService for ZeroHeightSelectionRectTextService {
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

    fn first_line_metrics(&mut self, _blob: fret_core::TextBlobId) -> Option<TextLineMetrics> {
        Some(TextLineMetrics {
            ascent: Px(8.0),
            descent: Px(2.0),
            line_height: Px(10.0),
        })
    }

    fn selection_rects_clipped(
        &mut self,
        _blob: fret_core::TextBlobId,
        range: (usize, usize),
        _clip: Rect,
        out: &mut Vec<Rect>,
    ) {
        out.clear();
        let (start, end) = range;
        if start >= end {
            return;
        }

        out.push(Rect::new(
            Point::new(Px(0.0), Px(0.0)),
            Size::new(Px((end - start) as f32), Px(0.0)),
        ));
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

impl fret_core::PathService for ZeroHeightSelectionRectTextService {
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

impl fret_core::SvgService for ZeroHeightSelectionRectTextService {
    fn register_svg(&mut self, _bytes: &[u8]) -> fret_core::SvgId {
        fret_core::SvgId::default()
    }

    fn unregister_svg(&mut self, _svg: fret_core::SvgId) -> bool {
        false
    }
}

impl fret_core::MaterialService for ZeroHeightSelectionRectTextService {
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
struct AutoscrollTextService {
    next_blob: u64,
    by_blob_text: HashMap<fret_core::TextBlobId, String>,
    last_prepare_max_width: Option<Px>,
}

impl TextService for AutoscrollTextService {
    fn prepare(
        &mut self,
        input: &fret_core::TextInput,
        constraints: TextConstraints,
    ) -> (fret_core::TextBlobId, TextMetrics) {
        self.last_prepare_max_width = constraints.max_width;
        let blob = fret_core::TextBlobId::from(KeyData::from_ffi(self.next_blob));
        self.next_blob = self.next_blob.wrapping_add(1);
        self.by_blob_text.insert(blob, input.text().to_string());

        let w = constraints
            .max_width
            .map(|w| w.0.max(0.0))
            .unwrap_or(input.text().len() as f32);

        (
            blob,
            TextMetrics {
                size: Size::new(Px(w), Px(10.0)),
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

    fn selection_rects(
        &mut self,
        _blob: fret_core::TextBlobId,
        (a, b): (usize, usize),
        out: &mut Vec<Rect>,
    ) {
        out.clear();
        let (start, end) = (a.min(b), a.max(b));
        out.push(Rect::new(
            Point::new(Px(start as f32), Px(0.0)),
            Size::new(Px((end - start) as f32), Px(10.0)),
        ));
    }

    fn hit_test_point(
        &mut self,
        blob: fret_core::TextBlobId,
        point: Point,
    ) -> fret_core::HitTestResult {
        let len = self.by_blob_text.get(&blob).map(|s| s.len()).unwrap_or(0);

        let mut idx = point.x.0.floor().max(0.0) as usize;
        idx = idx.min(len);

        let text = self
            .by_blob_text
            .get(&blob)
            .map(|s| s.as_str())
            .unwrap_or("");
        while idx > 0 && !text.is_char_boundary(idx) {
            idx = idx.saturating_sub(1);
        }

        fret_core::HitTestResult {
            index: idx,
            affinity: fret_core::CaretAffinity::Downstream,
        }
    }

    fn release(&mut self, blob: fret_core::TextBlobId) {
        self.by_blob_text.remove(&blob);
    }
}

impl fret_core::PathService for AutoscrollTextService {
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

impl fret_core::SvgService for AutoscrollTextService {
    fn register_svg(&mut self, _bytes: &[u8]) -> fret_core::SvgId {
        fret_core::SvgId::default()
    }

    fn unregister_svg(&mut self, _svg: fret_core::SvgId) -> bool {
        false
    }
}

impl fret_core::MaterialService for AutoscrollTextService {
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
fn caret_is_visible_even_when_backend_reports_zero_height_caret_rect() {
    let window = AppWindowId::default();
    let bounds = Rect::new(Point::new(Px(0.0), Px(0.0)), Size::new(Px(240.0), Px(80.0)));

    let mut ui = UiTree::new();
    ui.set_window(window);

    let caret_color = Color {
        r: 0.9,
        g: 0.1,
        b: 0.2,
        a: 1.0,
    };
    let style = TextAreaStyle {
        padding_x: Px(0.0),
        padding_y: Px(0.0),
        caret_color,
        ..Default::default()
    };

    let area = ui.create_node(TextArea::new("").with_style(style));
    ui.set_root(area);
    ui.set_focus(Some(area));

    let mut app = TestHost::new();
    app.set_global(PlatformCapabilities::default());
    let mut text = ZeroHeightCaretTextService::default();

    ui.layout_all(&mut app, &mut text, bounds, 1.0);

    let mut scene = Scene::default();
    ui.paint_all(&mut app, &mut text, bounds, &mut scene, 1.0);

    let caret_rect = scene.ops().iter().rev().find_map(|op| match op {
        SceneOp::Quad {
            rect, background, ..
        } if *background == Paint::Solid(caret_color) => Some(*rect),
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
fn right_click_focuses_and_preserves_selection_for_context_menus() {
    let window = AppWindowId::default();
    let bounds = Rect::new(Point::new(Px(0.0), Px(0.0)), Size::new(Px(200.0), Px(40.0)));

    let mut ui = UiTree::new();
    ui.set_window(window);

    let style = TextAreaStyle {
        padding_x: Px(0.0),
        padding_y: Px(0.0),
        ..Default::default()
    };

    let mut widget = TextArea::new("hello world").with_style(style);
    widget.caret = 5;
    widget.selection_anchor = 0;
    let area = ui.create_node(widget);
    ui.set_root(area);
    ui.set_focus(None);

    let mut app = TestHost::new();
    app.set_global(PlatformCapabilities::default());
    let mut text = AutoscrollTextService::default();

    ui.layout_all(&mut app, &mut text, bounds, 1.0);
    let _ = app.take_effects();

    let mut scene = Scene::default();
    ui.paint_all(&mut app, &mut text, bounds, &mut scene, 1.0);
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
        Some(area),
        "expected right-click to focus the text area so context menu commands target it"
    );

    let mut scene = Scene::default();
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

    let mut scene = Scene::default();
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
fn selection_highlight_is_visible_when_backend_reports_zero_height_selection_rects() {
    let window = AppWindowId::default();
    let bounds = Rect::new(Point::new(Px(0.0), Px(0.0)), Size::new(Px(200.0), Px(60.0)));

    let mut ui = UiTree::new();
    ui.set_window(window);

    let style = TextAreaStyle {
        padding_x: Px(0.0),
        padding_y: Px(0.0),
        ..Default::default()
    };

    let mut widget = TextArea::new("hello").with_style(style);
    widget.selection_anchor = 0;
    widget.caret = 5;

    let area = ui.create_node(widget);
    ui.set_root(area);
    ui.set_focus(None);

    let mut app = TestHost::new();
    app.set_global(PlatformCapabilities::default());
    let mut services = ZeroHeightSelectionRectTextService::default();

    ui.layout_all(&mut app, &mut services, bounds, 1.0);
    let mut scene = Scene::default();
    ui.paint_all(&mut app, &mut services, bounds, &mut scene, 1.0);

    let selection_quad_height = scene.ops().iter().find_map(|op| match op {
        SceneOp::Quad { rect, .. } if (rect.size.width.0 - 5.0).abs() < 0.01 => {
            Some(rect.size.height)
        }
        _ => None,
    });

    assert_eq!(
        selection_quad_height,
        Some(Px(10.0)),
        "expected zero-height selection rects to be expanded using line metrics so selection highlights remain visible"
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
fn text_area_command_availability_tracks_selection_and_clipboard_caps() {
    let window = AppWindowId::default();
    let mut app = TestHost::new();

    let mut caps = PlatformCapabilities::default();
    caps.clipboard.text.read = true;
    caps.clipboard.text.write = true;
    app.set_global(caps);

    let mut ui = UiTree::new();
    ui.set_window(window);
    let mut area = TextArea::new("hello");
    area.caret = 0;
    area.selection_anchor = 0;
    let node = ui.create_node(area);
    ui.set_root(node);
    ui.set_focus(Some(node));

    assert_eq!(
        ui.command_availability(&mut app, &fret_runtime::CommandId::from("edit.copy")),
        crate::widget::CommandAvailability::Blocked,
        "expected copy to be blocked without a selection"
    );

    let mut ui = UiTree::new();
    ui.set_window(window);
    let mut area = TextArea::new("hello");
    area.caret = 5;
    area.selection_anchor = 0;
    let node = ui.create_node(area);
    ui.set_root(node);
    ui.set_focus(Some(node));

    assert_eq!(
        ui.command_availability(&mut app, &fret_runtime::CommandId::from("edit.copy")),
        crate::widget::CommandAvailability::Available,
        "expected copy to be available with a selection"
    );

    let mut caps = PlatformCapabilities::default();
    caps.clipboard.text.read = false;
    caps.clipboard.text.write = false;
    app.set_global(caps);

    assert_eq!(
        ui.command_availability(&mut app, &fret_runtime::CommandId::from("edit.copy")),
        crate::widget::CommandAvailability::Blocked,
        "expected copy to be blocked when clipboard text is unavailable"
    );
}

#[test]
fn dragging_selection_autoscrolls_horizontally_when_wrap_none() {
    let window = AppWindowId::default();
    let bounds = Rect::new(Point::new(Px(0.0), Px(0.0)), Size::new(Px(20.0), Px(40.0)));

    let mut ui = UiTree::new();
    ui.set_window(window);

    let root = ui.create_node(RenderTransformWrapper::new(
        fret_core::Transform2D::IDENTITY,
    ));
    let style = TextAreaStyle {
        padding_x: Px(0.0),
        padding_y: Px(0.0),
        ..Default::default()
    };

    let mut widget = TextArea::new("abcdefghijklmnopqrstuvwxyz")
        .with_wrap(TextWrap::None)
        .with_style(style);
    widget.caret = 0;
    widget.selection_anchor = 0;
    widget.offset_x = Px(0.0);
    let area = ui.create_node(widget);
    ui.add_child(root, area);
    ui.set_root(root);
    ui.set_focus(Some(area));

    let mut app = TestHost::new();
    app.set_global(PlatformCapabilities::default());
    let mut text = AutoscrollTextService::default();

    ui.layout_all(&mut app, &mut text, bounds, 1.0);
    let _ = app.take_effects();

    text.last_prepare_max_width = Some(Px(-1.0));
    let mut scene = Scene::default();
    ui.paint_all(&mut app, &mut text, bounds, &mut scene, 1.0);
    let _ = app.take_effects();

    assert!(
        text.last_prepare_max_width.is_none(),
        "expected wrap=None text area to prepare text with an unbounded max width"
    );

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

    let mut scene = Scene::default();
    ui.paint_all(&mut app, &mut text, bounds, &mut scene, 1.0);

    let snap0 = app
        .global::<fret_runtime::WindowTextInputSnapshotService>()
        .and_then(|svc| svc.snapshot(window))
        .cloned()
        .expect("expected a window text input snapshot after paint");
    let (_, focus0) = snap0
        .selection_utf16
        .expect("expected selection to be present for focused text area");
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

        let mut scene = Scene::default();
        ui.paint_all(&mut app, &mut text, bounds, &mut scene, 1.0);
    }

    let snap1 = app
        .global::<fret_runtime::WindowTextInputSnapshotService>()
        .and_then(|svc| svc.snapshot(window))
        .cloned()
        .expect("expected a window text input snapshot after paint");
    let (_, focus1) = snap1
        .selection_utf16
        .expect("expected selection to be present for focused text area");

    assert!(
        focus1 > focus0,
        "expected dragging selection near the right edge to auto-scroll and extend the selection (focus0={focus0}, focus1={focus1}, text_len={})",
        snap1.text_len_utf16
    );
}

#[test]
fn text_area_set_text_is_idempotent_for_same_value() {
    let mut area = TextArea::new("hello");

    area.caret = 2;
    area.selection_anchor = 4;
    area.ensure_caret_visible = false;
    area.preedit = "x".to_string();
    area.preedit_cursor = Some((0, 1));
    area.ime_replace_range = Some((1, 2));
    area.text_dirty = false;
    area.preferred_x = Some(Px(12.0));

    area.set_text("hello");

    assert_eq!(area.text, "hello");
    assert_eq!(area.caret, 2);
    assert_eq!(area.selection_anchor, 4);
    assert!(!area.ensure_caret_visible);
    assert_eq!(area.preedit, "x");
    assert_eq!(area.preedit_cursor, Some((0, 1)));
    assert_eq!(area.ime_replace_range, Some((1, 2)));
    assert!(!area.text_dirty);
    assert_eq!(area.preferred_x, Some(Px(12.0)));
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

    let mut area = TextArea {
        text: "hello\nworld".to_string(),
        selection_anchor: 0,
        caret: 999,
        ..Default::default()
    };

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
        pointer_hit_is_text_input: false,
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

impl fret_core::MaterialService for YTextService {
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
