use super::*;
use crate::UiHost;
use crate::test_host::TestHost;
use crate::tree::UiTree;
use crate::widget::{LayoutCx, Widget};
use fret_core::{
    AppWindowId, Axis, Event, PathCommand, PathConstraints, PathMetrics, PathService, PathStyle,
    Point, Px, Size, TextConstraints, TextMetrics, TextService,
};
use fret_runtime::{Effect, PlatformCapabilities};

#[derive(Default)]
struct FakeUiServices;

impl TextService for FakeUiServices {
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

    fn release(&mut self, _blob: fret_core::TextBlobId) {}
}

impl PathService for FakeUiServices {
    fn prepare(
        &mut self,
        _commands: &[PathCommand],
        _style: PathStyle,
        _constraints: PathConstraints,
    ) -> (fret_core::PathId, PathMetrics) {
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
        false
    }
}

impl fret_core::MaterialService for FakeUiServices {
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

struct Leaf;

impl<H: UiHost> Widget<H> for Leaf {
    fn layout(&mut self, cx: &mut LayoutCx<'_, H>) -> Size {
        cx.available
    }
}

#[test]
fn resizable_split_hover_sets_resize_cursor() {
    let window = AppWindowId::default();

    let mut ui = UiTree::new();
    ui.set_window(window);

    let mut app = TestHost::new();
    app.set_global(PlatformCapabilities::default());
    let fraction = app.models_mut().insert(0.5f32);

    let root = ui.create_node(ResizableSplit::new(Axis::Horizontal, fraction.clone()));
    let a = ui.create_node(Leaf);
    let b = ui.create_node(Leaf);
    ui.add_child(root, a);
    ui.add_child(root, b);
    ui.set_root(root);

    let mut services = FakeUiServices;
    let size = Size::new(Px(400.0), Px(120.0));
    let _ = ui.layout(&mut app, &mut services, root, size, 1.0);
    let _ = app.take_effects();

    ui.dispatch_event(
        &mut app,
        &mut services,
        &Event::Pointer(fret_core::PointerEvent::Move {
            position: Point::new(Px(200.0), Px(10.0)),
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
                if *w == window && *icon == fret_core::CursorIcon::ColResize
        )),
        "expected a resize cursor effect when hovering the split handle"
    );
}

#[test]
fn resizable_split_drag_updates_fraction_model() {
    let window = AppWindowId::default();

    let mut ui = UiTree::new();
    ui.set_window(window);

    let mut app = TestHost::new();
    app.set_global(PlatformCapabilities::default());
    let fraction = app.models_mut().insert(0.5f32);

    let root = ui.create_node(ResizableSplit::new(Axis::Horizontal, fraction.clone()));
    let a = ui.create_node(Leaf);
    let b = ui.create_node(Leaf);
    ui.add_child(root, a);
    ui.add_child(root, b);
    ui.set_root(root);

    let mut services = FakeUiServices;
    let size = Size::new(Px(400.0), Px(120.0));
    let _ = ui.layout(&mut app, &mut services, root, size, 1.0);

    ui.dispatch_event(
        &mut app,
        &mut services,
        &Event::Pointer(fret_core::PointerEvent::Down {
            position: Point::new(Px(200.0), Px(10.0)),
            button: fret_core::MouseButton::Left,
            modifiers: fret_core::Modifiers::default(),
            click_count: 1,
            pointer_id: fret_core::PointerId(0),
            pointer_type: fret_core::PointerType::Mouse,
        }),
    );

    ui.dispatch_event(
        &mut app,
        &mut services,
        &Event::Pointer(fret_core::PointerEvent::Move {
            position: Point::new(Px(280.0), Px(10.0)),
            buttons: fret_core::MouseButtons::default(),
            modifiers: fret_core::Modifiers::default(),
            pointer_id: fret_core::PointerId(0),
            pointer_type: fret_core::PointerType::Mouse,
        }),
    );

    let updated = app.models().get_copied(&fraction).unwrap_or(0.0);
    assert!(
        updated > 0.5,
        "expected drag to increase split fraction, got {updated}"
    );
}
