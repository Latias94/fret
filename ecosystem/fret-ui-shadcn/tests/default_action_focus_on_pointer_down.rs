use fret_app::App;
use fret_core::{
    AppWindowId, Event, FrameId, Modifiers, MouseButton, Point, PointerEvent, PointerId,
    PointerType, Px, Rect, SemanticsRole, Size as CoreSize,
};
use fret_core::{PathCommand, PathConstraints, PathId, PathMetrics, PathService, PathStyle, SvgId};
use fret_core::{SvgService, TextBlobId, TextConstraints, TextMetrics, TextService};
use fret_runtime::Model;
use fret_ui::element::{LayoutStyle, Length, PressableA11y, PressableProps};
use fret_ui::tree::UiTree;
use fret_ui_kit::OverlayController;
use std::sync::Arc;

#[derive(Default)]
struct FakeServices;

impl TextService for FakeServices {
    fn prepare(
        &mut self,
        _input: &fret_core::TextInput,
        _constraints: TextConstraints,
    ) -> (TextBlobId, TextMetrics) {
        (
            TextBlobId::default(),
            TextMetrics {
                size: CoreSize::new(Px(0.0), Px(0.0)),
                baseline: Px(0.0),
            },
        )
    }

    fn release(&mut self, _blob: TextBlobId) {}
}

impl PathService for FakeServices {
    fn prepare(
        &mut self,
        _commands: &[PathCommand],
        _style: PathStyle,
        _constraints: PathConstraints,
    ) -> (PathId, PathMetrics) {
        (PathId::default(), PathMetrics::default())
    }

    fn release(&mut self, _path: PathId) {}
}

impl SvgService for FakeServices {
    fn register_svg(&mut self, _bytes: &[u8]) -> SvgId {
        SvgId::default()
    }

    fn unregister_svg(&mut self, _svg: SvgId) -> bool {
        true
    }
}

fn render_frame(
    ui: &mut UiTree<App>,
    app: &mut App,
    services: &mut dyn fret_core::UiServices,
    window: AppWindowId,
    bounds: Rect,
    clicked: Model<bool>,
    prevent_default_focus: bool,
) {
    let next_frame = FrameId(app.frame_id().0.saturating_add(1));
    app.set_frame_id(next_frame);

    OverlayController::begin_frame(app, window);
    let root = fret_ui::declarative::render_root(
        ui,
        app,
        services,
        window,
        bounds,
        "default-action-focus-on-pointer-down",
        move |cx| {
            let handler: fret_ui::action::OnPressablePointerDown =
                Arc::new(move |host, _action_cx, _down| {
                    let _ = host.models_mut().update(&clicked, |v| *v = true);
                    if prevent_default_focus {
                        host.prevent_default(fret_runtime::DefaultAction::FocusOnPointerDown);
                    }
                    fret_ui::action::PressablePointerDownResult::SkipDefault
                });

            vec![cx.pressable(
                PressableProps {
                    layout: {
                        let mut layout = LayoutStyle::default();
                        layout.size.width = Length::Px(Px(200.0));
                        layout.size.height = Length::Px(Px(100.0));
                        layout
                    },
                    enabled: true,
                    focusable: true,
                    a11y: PressableA11y {
                        role: Some(SemanticsRole::Button),
                        label: Some(Arc::from("Target")),
                        test_id: Some(Arc::from("target")),
                        ..Default::default()
                    },
                    ..Default::default()
                },
                move |cx, _st| {
                    cx.pressable_on_pointer_down(handler.clone());
                    vec![]
                },
            )]
        },
    );
    ui.set_root(root);
    OverlayController::render(ui, app, services, window, bounds);
    ui.request_semantics_snapshot();
    ui.layout_all(app, services, bounds, 1.0);
}

fn dispatch_left_mouse_down(
    ui: &mut UiTree<App>,
    app: &mut App,
    services: &mut dyn fret_core::UiServices,
) {
    ui.dispatch_event(
        app,
        services,
        &Event::Pointer(PointerEvent::Down {
            pointer_id: PointerId(0),
            position: Point::new(Px(10.0), Px(10.0)),
            button: MouseButton::Left,
            modifiers: Modifiers::default(),
            click_count: 1,
            pointer_type: PointerType::Mouse,
        }),
    );
}

#[test]
fn pressable_skip_default_still_triggers_default_focus_on_pointer_down() {
    let window = AppWindowId::default();
    let mut app = App::new();
    let mut ui: UiTree<App> = UiTree::new();
    ui.set_window(window);
    let mut services = FakeServices::default();

    let clicked = app.models_mut().insert(false);
    let bounds = Rect::new(
        Point::new(Px(0.0), Px(0.0)),
        CoreSize::new(Px(300.0), Px(200.0)),
    );

    render_frame(
        &mut ui,
        &mut app,
        &mut services,
        window,
        bounds,
        clicked.clone(),
        false,
    );
    let snap = ui.semantics_snapshot().expect("semantics snapshot").clone();
    let target_node = snap
        .nodes
        .iter()
        .find(|n| n.test_id.as_deref() == Some("target"))
        .map(|n| n.id)
        .expect("target node");

    assert_eq!(ui.focus(), None);
    dispatch_left_mouse_down(&mut ui, &mut app, &mut services);
    assert_eq!(ui.focus(), Some(target_node));
    assert_eq!(app.models().get_copied(&clicked), Some(true));
}

#[test]
fn pressable_skip_default_can_prevent_default_focus_on_pointer_down() {
    let window = AppWindowId::default();
    let mut app = App::new();
    let mut ui: UiTree<App> = UiTree::new();
    ui.set_window(window);
    let mut services = FakeServices::default();

    let clicked = app.models_mut().insert(false);
    let bounds = Rect::new(
        Point::new(Px(0.0), Px(0.0)),
        CoreSize::new(Px(300.0), Px(200.0)),
    );

    render_frame(
        &mut ui,
        &mut app,
        &mut services,
        window,
        bounds,
        clicked.clone(),
        true,
    );

    assert_eq!(ui.focus(), None);
    dispatch_left_mouse_down(&mut ui, &mut app, &mut services);
    assert_eq!(ui.focus(), None);
    assert_eq!(app.models().get_copied(&clicked), Some(true));
}
