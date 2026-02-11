use super::*;
use crate::UiHost;
use crate::test_host::TestHost;
use crate::tree::UiTree;
use crate::widget::{LayoutCx, Widget};
use fret_core::{
    AppWindowId, Axis, Event, MouseButton, PathCommand, PathConstraints, PathMetrics, PathService,
    PathStyle, Point, Px, Rect, Size, TextConstraints, TextMetrics, TextService,
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

struct Dummy;
impl<H: UiHost> Widget<H> for Dummy {
    fn layout(&mut self, _cx: &mut LayoutCx<'_, H>) -> Size {
        Size::new(Px(0.0), Px(0.0))
    }
}

#[test]
fn resizable_panel_group_drag_updates_fractions_model() {
    let window = AppWindowId::default();

    let mut ui = UiTree::new();
    ui.set_window(window);

    let mut app = TestHost::new();
    app.set_global(PlatformCapabilities::default());

    let fractions = app.models_mut().insert(vec![0.33, 0.34, 0.33]);
    let mut group = BoundResizablePanelGroup::new(Axis::Horizontal, fractions.clone());
    group.set_style(ResizablePanelGroupStyle {
        gap: Px(0.0),
        hit_thickness: Px(10.0),
        ..Default::default()
    });

    let root_id = ui.create_node(group);
    let a = ui.create_node(Dummy);
    let b = ui.create_node(Dummy);
    let c = ui.create_node(Dummy);
    ui.add_child(root_id, a);
    ui.add_child(root_id, b);
    ui.add_child(root_id, c);
    ui.set_root(root_id);

    let mut services = FakeUiServices;
    let size = Size::new(Px(600.0), Px(40.0));
    let _ = ui.layout(&mut app, &mut services, root_id, size, 1.0);
    let _ = app.take_effects();

    let bounds = Rect::new(Point::new(Px(0.0), Px(0.0)), size);
    let fractions_now = app.models().get_cloned(&fractions).unwrap_or_default();
    let layout = compute_resizable_panel_group_layout(
        Axis::Horizontal,
        bounds,
        3,
        fractions_now,
        Px(0.0),
        Px(10.0),
        &[],
    );
    let center = layout.handle_centers.first().copied().unwrap_or(0.0);
    ui.dispatch_event(
        &mut app,
        &mut services,
        &Event::Pointer(fret_core::PointerEvent::Down {
            position: Point::new(Px(center), Px(20.0)),
            button: MouseButton::Left,
            modifiers: fret_core::Modifiers::default(),
            click_count: 1,
            pointer_id: fret_core::PointerId(0),
            pointer_type: fret_core::PointerType::Mouse,
        }),
    );
    assert_eq!(ui.captured(), Some(root_id), "expected pointer capture");
    ui.dispatch_event(
        &mut app,
        &mut services,
        &Event::Pointer(fret_core::PointerEvent::Move {
            position: Point::new(Px(center + 30.0), Px(20.0)),
            buttons: fret_core::MouseButtons::default(),
            modifiers: fret_core::Modifiers::default(),
            pointer_id: fret_core::PointerId(0),
            pointer_type: fret_core::PointerType::Mouse,
        }),
    );
    ui.dispatch_event(
        &mut app,
        &mut services,
        &Event::Pointer(fret_core::PointerEvent::Up {
            position: Point::new(Px(center + 30.0), Px(20.0)),
            button: MouseButton::Left,
            modifiers: fret_core::Modifiers::default(),
            is_click: false,
            click_count: 1,
            pointer_id: fret_core::PointerId(0),
            pointer_type: fret_core::PointerType::Mouse,
        }),
    );

    let v = app.models().get_cloned(&fractions).unwrap_or_default();
    assert_eq!(v.len(), 3);
    assert!(v[0] > 0.33, "expected left panel to grow, got {v:?}");
    let effects = app.take_effects();
    assert!(
        effects.iter().any(|e| matches!(
            e,
            Effect::CursorSetIcon { window: w, icon }
                if *w == window && *icon == fret_core::CursorIcon::ColResize
        )),
        "expected resize cursor effects during interaction"
    );
}

#[test]
fn resizable_panel_group_pushes_growth_through_following_panels() {
    let window = AppWindowId::default();

    let mut ui = UiTree::new();
    ui.set_window(window);

    let mut app = TestHost::new();
    app.set_global(PlatformCapabilities::default());

    let fractions = app.models_mut().insert(vec![0.33, 0.34, 0.33]);
    let mut group = BoundResizablePanelGroup::new(Axis::Horizontal, fractions.clone());
    group.set_style(ResizablePanelGroupStyle {
        gap: Px(0.0),
        hit_thickness: Px(10.0),
        ..Default::default()
    });
    group.set_min_px(vec![Px(100.0), Px(100.0), Px(100.0)]);

    let root_id = ui.create_node(group);
    let a = ui.create_node(Dummy);
    let b = ui.create_node(Dummy);
    let c = ui.create_node(Dummy);
    ui.add_child(root_id, a);
    ui.add_child(root_id, b);
    ui.add_child(root_id, c);
    ui.set_root(root_id);

    let mut services = FakeUiServices;
    let size = Size::new(Px(600.0), Px(40.0));
    let _ = ui.layout(&mut app, &mut services, root_id, size, 1.0);

    let bounds = Rect::new(Point::new(Px(0.0), Px(0.0)), size);
    let before = app.models().get_cloned(&fractions).unwrap_or_default();
    let layout_before = compute_resizable_panel_group_layout(
        Axis::Horizontal,
        bounds,
        3,
        before,
        Px(0.0),
        Px(10.0),
        &[Px(100.0), Px(100.0), Px(100.0)],
    );
    let center = layout_before.handle_centers.first().copied().unwrap_or(0.0);

    ui.dispatch_event(
        &mut app,
        &mut services,
        &Event::Pointer(fret_core::PointerEvent::Down {
            position: Point::new(Px(center), Px(20.0)),
            button: MouseButton::Left,
            modifiers: fret_core::Modifiers::default(),
            click_count: 1,
            pointer_id: fret_core::PointerId(0),
            pointer_type: fret_core::PointerType::Mouse,
        }),
    );
    assert_eq!(ui.captured(), Some(root_id), "expected pointer capture");

    ui.dispatch_event(
        &mut app,
        &mut services,
        &Event::Pointer(fret_core::PointerEvent::Move {
            position: Point::new(Px(center + 250.0), Px(20.0)),
            buttons: fret_core::MouseButtons::default(),
            modifiers: fret_core::Modifiers::default(),
            pointer_id: fret_core::PointerId(0),
            pointer_type: fret_core::PointerType::Mouse,
        }),
    );
    ui.dispatch_event(
        &mut app,
        &mut services,
        &Event::Pointer(fret_core::PointerEvent::Up {
            position: Point::new(Px(center + 250.0), Px(20.0)),
            button: MouseButton::Left,
            modifiers: fret_core::Modifiers::default(),
            is_click: false,
            click_count: 1,
            pointer_id: fret_core::PointerId(0),
            pointer_type: fret_core::PointerType::Mouse,
        }),
    );

    let after = app.models().get_cloned(&fractions).unwrap_or_default();
    let layout_after = compute_resizable_panel_group_layout(
        Axis::Horizontal,
        bounds,
        3,
        after,
        Px(0.0),
        Px(10.0),
        &[Px(100.0), Px(100.0), Px(100.0)],
    );

    assert_eq!(layout_after.sizes.len(), 3);
    assert!(
        (layout_after.sizes[0] - 400.0).abs() < 0.01,
        "{layout_after:?}"
    );
    assert!(
        (layout_after.sizes[1] - 100.0).abs() < 0.01,
        "{layout_after:?}"
    );
    assert!(
        (layout_after.sizes[2] - 100.0).abs() < 0.01,
        "{layout_after:?}"
    );
}

#[test]
fn resizable_panel_group_shrink_clamps_to_min_px() {
    let window = AppWindowId::default();

    let mut ui = UiTree::new();
    ui.set_window(window);

    let mut app = TestHost::new();
    app.set_global(PlatformCapabilities::default());

    let fractions = app.models_mut().insert(vec![0.5, 0.25, 0.25]);
    let mut group = BoundResizablePanelGroup::new(Axis::Horizontal, fractions.clone());
    group.set_style(ResizablePanelGroupStyle {
        gap: Px(0.0),
        hit_thickness: Px(10.0),
        ..Default::default()
    });
    group.set_min_px(vec![Px(100.0), Px(100.0), Px(100.0)]);

    let root_id = ui.create_node(group);
    let a = ui.create_node(Dummy);
    let b = ui.create_node(Dummy);
    let c = ui.create_node(Dummy);
    ui.add_child(root_id, a);
    ui.add_child(root_id, b);
    ui.add_child(root_id, c);
    ui.set_root(root_id);

    let mut services = FakeUiServices;
    let size = Size::new(Px(600.0), Px(40.0));
    let _ = ui.layout(&mut app, &mut services, root_id, size, 1.0);

    let bounds = Rect::new(Point::new(Px(0.0), Px(0.0)), size);
    let before = app.models().get_cloned(&fractions).unwrap_or_default();
    let layout_before = compute_resizable_panel_group_layout(
        Axis::Horizontal,
        bounds,
        3,
        before,
        Px(0.0),
        Px(10.0),
        &[Px(100.0), Px(100.0), Px(100.0)],
    );
    let center = layout_before.handle_centers.first().copied().unwrap_or(0.0);

    ui.dispatch_event(
        &mut app,
        &mut services,
        &Event::Pointer(fret_core::PointerEvent::Down {
            position: Point::new(Px(center), Px(20.0)),
            button: MouseButton::Left,
            modifiers: fret_core::Modifiers::default(),
            click_count: 1,
            pointer_id: fret_core::PointerId(0),
            pointer_type: fret_core::PointerType::Mouse,
        }),
    );
    assert_eq!(ui.captured(), Some(root_id), "expected pointer capture");

    ui.dispatch_event(
        &mut app,
        &mut services,
        &Event::Pointer(fret_core::PointerEvent::Move {
            position: Point::new(Px(center - 250.0), Px(20.0)),
            buttons: fret_core::MouseButtons::default(),
            modifiers: fret_core::Modifiers::default(),
            pointer_id: fret_core::PointerId(0),
            pointer_type: fret_core::PointerType::Mouse,
        }),
    );
    ui.dispatch_event(
        &mut app,
        &mut services,
        &Event::Pointer(fret_core::PointerEvent::Up {
            position: Point::new(Px(center - 250.0), Px(20.0)),
            button: MouseButton::Left,
            modifiers: fret_core::Modifiers::default(),
            is_click: false,
            click_count: 1,
            pointer_id: fret_core::PointerId(0),
            pointer_type: fret_core::PointerType::Mouse,
        }),
    );

    let after = app.models().get_cloned(&fractions).unwrap_or_default();
    let layout_after = compute_resizable_panel_group_layout(
        Axis::Horizontal,
        bounds,
        3,
        after,
        Px(0.0),
        Px(10.0),
        &[Px(100.0), Px(100.0), Px(100.0)],
    );

    assert_eq!(layout_after.sizes.len(), 3);
    assert!(
        (layout_after.sizes[0] - 100.0).abs() < 0.01,
        "{layout_after:?}"
    );
    assert!(
        (layout_after.sizes[1] - 350.0).abs() < 0.01,
        "{layout_after:?}"
    );
    assert!(
        (layout_after.sizes[2] - 150.0).abs() < 0.01,
        "{layout_after:?}"
    );
}
