use std::{
    any::{Any, TypeId},
    collections::{HashMap, HashSet},
};

use fret_core::{
    AppWindowId, DrawOrder, Edges, Event, KeyCode, Modifiers, MouseButton, NodeId, Point,
    PointerEvent, PointerId, PointerType, Px, Rect, Scene, SceneOp, Size, Transform2D, UiServices,
};
use fret_runtime::{
    CommandRegistry, CommandsHost, DragHost, DragKindId, DragSession, DragSessionId, Effect,
    EffectSink, FrameId, GlobalsHost, ModelHost, ModelId, ModelStore, ModelsHost,
    PlatformCapabilities, TickId, TimeHost,
};
use fret_ui::element::{AnyElement, ContainerProps};
use fret_ui::{Theme, UiTree};

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
enum SceneSig {
    PushTransform,
    PopTransform,
    PushOpacity,
    PopOpacity,
    PushLayer,
    PopLayer,
    PushClipRect,
    PushClipRRect,
    PopClip,
    PushEffect,
    PopEffect,
    Quad(DrawOrder),
    Image(DrawOrder),
    ImageRegion(DrawOrder),
    MaskImage(DrawOrder),
    SvgMaskIcon(DrawOrder),
    SvgImage(DrawOrder),
    Text(DrawOrder),
    Path(DrawOrder),
    ViewportSurface(DrawOrder),
}

fn scene_signature(scene: &Scene) -> Vec<SceneSig> {
    scene
        .ops()
        .iter()
        .map(|op| match *op {
            SceneOp::PushTransform { .. } => SceneSig::PushTransform,
            SceneOp::PopTransform => SceneSig::PopTransform,
            SceneOp::PushOpacity { .. } => SceneSig::PushOpacity,
            SceneOp::PopOpacity => SceneSig::PopOpacity,
            SceneOp::PushLayer { .. } => SceneSig::PushLayer,
            SceneOp::PopLayer => SceneSig::PopLayer,
            SceneOp::PushClipRect { .. } => SceneSig::PushClipRect,
            SceneOp::PushClipRRect { .. } => SceneSig::PushClipRRect,
            SceneOp::PopClip => SceneSig::PopClip,
            SceneOp::PushEffect { .. } => SceneSig::PushEffect,
            SceneOp::PopEffect => SceneSig::PopEffect,
            SceneOp::Quad { order, .. } => SceneSig::Quad(order),
            SceneOp::Image { order, .. } => SceneSig::Image(order),
            SceneOp::ImageRegion { order, .. } => SceneSig::ImageRegion(order),
            SceneOp::MaskImage { order, .. } => SceneSig::MaskImage(order),
            SceneOp::SvgMaskIcon { order, .. } => SceneSig::SvgMaskIcon(order),
            SceneOp::SvgImage { order, .. } => SceneSig::SvgImage(order),
            SceneOp::Text { order, .. } => SceneSig::Text(order),
            SceneOp::Path { order, .. } => SceneSig::Path(order),
            SceneOp::ViewportSurface { order, .. } => SceneSig::ViewportSurface(order),
        })
        .collect()
}

#[derive(Default)]
struct TestHost {
    globals: HashMap<TypeId, Box<dyn Any>>,
    models: ModelStore,
    commands: CommandRegistry,
    redraw: HashSet<AppWindowId>,
    effects: Vec<Effect>,
    drags: HashMap<fret_core::PointerId, DragSession>,
    next_drag_session_id: u64,
    tick_id: TickId,
    frame_id: FrameId,
    next_timer_token: u64,
    next_clipboard_token: u64,
    next_image_upload_token: u64,
}

impl TestHost {
    fn set_global<T: Any>(&mut self, value: T) {
        GlobalsHost::set_global(self, value);
    }

    fn advance_frame(&mut self) {
        self.frame_id = FrameId(self.frame_id.0.saturating_add(1));
        self.tick_id = TickId(self.tick_id.0.saturating_add(1));
    }
}

impl GlobalsHost for TestHost {
    fn set_global<T: Any>(&mut self, value: T) {
        self.globals.insert(TypeId::of::<T>(), Box::new(value));
    }

    fn global<T: Any>(&self) -> Option<&T> {
        self.globals
            .get(&TypeId::of::<T>())
            .and_then(|v| v.downcast_ref::<T>())
    }

    fn with_global_mut<T: Any, R>(
        &mut self,
        init: impl FnOnce() -> T,
        f: impl FnOnce(&mut T, &mut Self) -> R,
    ) -> R {
        #[derive(Debug)]
        struct GlobalLeaseMarker;

        struct Guard<T: Any> {
            type_id: TypeId,
            value: Option<T>,
            globals: *mut HashMap<TypeId, Box<dyn Any>>,
        }

        impl<T: Any> Drop for Guard<T> {
            fn drop(&mut self) {
                let Some(value) = self.value.take() else {
                    return;
                };
                unsafe {
                    (*self.globals).insert(self.type_id, Box::new(value));
                }
            }
        }

        let type_id = TypeId::of::<T>();
        let existing = self
            .globals
            .insert(type_id, Box::new(GlobalLeaseMarker) as Box<dyn Any>);

        let existing = match existing {
            None => None,
            Some(v) => {
                if v.is::<GlobalLeaseMarker>() {
                    panic!("global already leased: {type_id:?}");
                }
                Some(*v.downcast::<T>().expect("global type id must match"))
            }
        };

        let mut guard = Guard::<T> {
            type_id,
            value: Some(existing.unwrap_or_else(init)),
            globals: &mut self.globals as *mut _,
        };

        let result = {
            let value = guard.value.as_mut().expect("guard value exists");
            f(value, self)
        };

        drop(guard);
        result
    }
}

impl ModelsHost for TestHost {
    fn take_changed_models(&mut self) -> Vec<ModelId> {
        self.models.take_changed_models()
    }
}

impl CommandsHost for TestHost {
    fn commands(&self) -> &CommandRegistry {
        &self.commands
    }
}

impl EffectSink for TestHost {
    fn request_redraw(&mut self, window: AppWindowId) {
        self.redraw.insert(window);
    }

    fn push_effect(&mut self, effect: Effect) {
        match effect {
            Effect::Redraw(window) => self.request_redraw(window),
            effect => self.effects.push(effect),
        }
    }
}

impl TimeHost for TestHost {
    fn tick_id(&self) -> TickId {
        self.tick_id
    }

    fn frame_id(&self) -> FrameId {
        self.frame_id
    }

    fn next_timer_token(&mut self) -> fret_runtime::TimerToken {
        let token = fret_runtime::TimerToken(self.next_timer_token);
        self.next_timer_token = self.next_timer_token.saturating_add(1);
        token
    }

    fn next_clipboard_token(&mut self) -> fret_runtime::ClipboardToken {
        let token = fret_runtime::ClipboardToken(self.next_clipboard_token);
        self.next_clipboard_token = self.next_clipboard_token.saturating_add(1);
        token
    }

    fn next_image_upload_token(&mut self) -> fret_runtime::ImageUploadToken {
        let token = fret_runtime::ImageUploadToken(self.next_image_upload_token);
        self.next_image_upload_token = self.next_image_upload_token.saturating_add(1);
        token
    }
}

impl DragHost for TestHost {
    fn drag(&self, pointer_id: fret_core::PointerId) -> Option<&DragSession> {
        self.drags.get(&pointer_id)
    }

    fn drag_mut(&mut self, pointer_id: fret_core::PointerId) -> Option<&mut DragSession> {
        self.drags.get_mut(&pointer_id)
    }

    fn cancel_drag(&mut self, pointer_id: fret_core::PointerId) {
        self.drags.remove(&pointer_id);
    }

    fn begin_drag_with_kind<T: Any>(
        &mut self,
        pointer_id: fret_core::PointerId,
        kind: DragKindId,
        source_window: AppWindowId,
        start: Point,
        payload: T,
    ) {
        let session_id = DragSessionId(self.next_drag_session_id);
        self.next_drag_session_id = self.next_drag_session_id.saturating_add(1);
        self.drags.insert(
            pointer_id,
            DragSession::new(session_id, pointer_id, source_window, kind, start, payload),
        );
    }

    fn begin_cross_window_drag_with_kind<T: Any>(
        &mut self,
        pointer_id: fret_core::PointerId,
        kind: DragKindId,
        source_window: AppWindowId,
        start: Point,
        payload: T,
    ) {
        let session_id = DragSessionId(self.next_drag_session_id);
        self.next_drag_session_id = self.next_drag_session_id.saturating_add(1);
        self.drags.insert(
            pointer_id,
            DragSession::new_cross_window(
                session_id,
                pointer_id,
                source_window,
                kind,
                start,
                payload,
            ),
        );
    }
}

impl ModelHost for TestHost {
    fn models(&self) -> &ModelStore {
        &self.models
    }

    fn models_mut(&mut self) -> &mut ModelStore {
        &mut self.models
    }
}

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
                size: Size::new(Px(10.0), Px(10.0)),
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
        false
    }
}

fn find_first_bounds_with_size(
    ui: &UiTree<TestHost>,
    root: fret_core::NodeId,
    width: f32,
    height: f32,
) -> Option<Rect> {
    let mut stack = vec![root];
    while let Some(node) = stack.pop() {
        if let Some(r) = ui.debug_node_visual_bounds(node)
            && (r.size.width.0 - width).abs() < 0.1
            && (r.size.height.0 - height).abs() < 0.1
        {
            return Some(r);
        }

        for child in ui.children(node) {
            stack.push(child);
        }
    }
    None
}

fn pointer_down(pointer_id: PointerId, position: Point) -> Event {
    Event::Pointer(PointerEvent::Down {
        pointer_id,
        position,
        button: MouseButton::Left,
        modifiers: Modifiers::default(),
        click_count: 1,
        pointer_type: PointerType::Mouse,
    })
}

fn pointer_up(pointer_id: PointerId, position: Point) -> Event {
    Event::Pointer(PointerEvent::Up {
        pointer_id,
        position,
        button: MouseButton::Left,
        modifiers: Modifiers::default(),
        click_count: 1,
        pointer_type: PointerType::Mouse,
    })
}

fn key_down(key: KeyCode) -> Event {
    Event::KeyDown {
        key,
        modifiers: Modifiers::default(),
        repeat: false,
    }
}

fn key_up(key: KeyCode) -> Event {
    Event::KeyUp {
        key,
        modifiers: Modifiers::default(),
    }
}

fn with_padding<'a, H: fret_ui::UiHost>(
    cx: &mut fret_ui::elements::ElementContext<'a, H>,
    padding: Px,
    child: AnyElement,
) -> AnyElement {
    cx.container(
        ContainerProps {
            padding: Edges::all(padding),
            ..Default::default()
        },
        move |_cx| vec![child],
    )
}

#[test]
fn radio_selected_dot_is_centered_in_outline() {
    for scale_factor in [1.0, 1.25, 2.0] {
        let mut app = TestHost::default();
        app.set_global(PlatformCapabilities::default());

        let cfg = fret_ui_material3::tokens::v30::theme_config_with_colors(
            fret_ui_material3::tokens::v30::TypographyOptions::default(),
            fret_ui_material3::tokens::v30::ColorSchemeOptions::default(),
        );
        Theme::with_global_mut(&mut app, |theme| theme.apply_config(&cfg));

        let window = AppWindowId::default();
        let mut services = FakeUiServices::default();
        let mut ui: UiTree<TestHost> = UiTree::new();
        ui.set_window(window);

        let bounds = Rect::new(
            Point::new(Px(0.0), Px(0.0)),
            Size::new(Px(300.0), Px(200.0)),
        );

        let selected = app.models_mut().insert(true);

        let render =
            |ui: &mut UiTree<TestHost>, app: &mut TestHost, services: &mut dyn UiServices| {
                fret_ui::declarative::render_root(ui, app, services, window, bounds, "root", |cx| {
                    let child = fret_ui_material3::Radio::new(selected.clone())
                        .a11y_label("radio")
                        .into_element(cx);
                    vec![with_padding(cx, Px(37.0), child)]
                })
            };

        let mut found = None;
        for _ in 0..12 {
            let root = render(&mut ui, &mut app, &mut services);
            ui.set_root(root);
            ui.layout_all(&mut app, &mut services, bounds, scale_factor);

            let mut scene = Scene::default();
            ui.paint_all(&mut app, &mut services, bounds, &mut scene, scale_factor);

            let mut outline: Option<Rect> = None;
            let mut dot: Option<Rect> = None;

            for op in scene.ops() {
                let SceneOp::Quad {
                    rect,
                    background,
                    border,
                    ..
                } = op
                else {
                    continue;
                };

                let border_any =
                    border.top.0 > 0.0 || border.right.0 > 0.0 || border.bottom.0 > 0.0;
                if border_any && background.a <= 0.01 {
                    if outline.is_none_or(|r| rect.size.width.0 < r.size.width.0 + 1e-3) {
                        outline = Some(*rect);
                    }
                    continue;
                }

                if border == &Edges::all(Px(0.0))
                    && background.a > 0.5
                    && rect.size.width.0 <= 12.0
                    && rect.size.height.0 <= 12.0
                {
                    if dot.is_none_or(|r| rect.size.width.0 > r.size.width.0 + 1e-3) {
                        dot = Some(*rect);
                    }
                }
            }

            if let (Some(outline), Some(dot)) = (outline, dot) {
                found = Some((outline, dot));
                if dot.size.width.0 > 1.0 {
                    break;
                }
            }

            app.advance_frame();
        }

        let Some((outline, dot)) = found else {
            panic!("expected radio outline + selected dot quads in the scene");
        };

        let outline_cx = outline.origin.x.0 + outline.size.width.0 * 0.5;
        let outline_cy = outline.origin.y.0 + outline.size.height.0 * 0.5;
        let dot_cx = dot.origin.x.0 + dot.size.width.0 * 0.5;
        let dot_cy = dot.origin.y.0 + dot.size.height.0 * 0.5;

        assert!(
            (outline_cx - dot_cx).abs() < 0.75 && (outline_cy - dot_cy).abs() < 0.75,
            "dot center should match outline center (scale={scale_factor}): outline={outline:?} dot={dot:?}"
        );
    }
}

#[test]
fn radio_ripple_origin_tracks_pointer_down_position() {
    for scale_factor in [1.0, 1.25, 2.0] {
        let mut app = TestHost::default();
        app.set_global(PlatformCapabilities::default());

        let cfg = fret_ui_material3::tokens::v30::theme_config_with_colors(
            fret_ui_material3::tokens::v30::TypographyOptions::default(),
            fret_ui_material3::tokens::v30::ColorSchemeOptions::default(),
        );
        Theme::with_global_mut(&mut app, |theme| theme.apply_config(&cfg));

        let window = AppWindowId::default();
        let mut services = FakeUiServices::default();
        let mut ui: UiTree<TestHost> = UiTree::new();
        ui.set_window(window);

        let bounds = Rect::new(
            Point::new(Px(0.0), Px(0.0)),
            Size::new(Px(300.0), Px(200.0)),
        );

        let selected = app.models_mut().insert(false);

        let render =
            |ui: &mut UiTree<TestHost>, app: &mut TestHost, services: &mut dyn UiServices| {
                fret_ui::declarative::render_root(ui, app, services, window, bounds, "root", |cx| {
                    let child = fret_ui_material3::Radio::new(selected.clone())
                        .a11y_label("radio")
                        .into_element(cx);
                    vec![with_padding(cx, Px(37.0), child)]
                })
            };

        let root = render(&mut ui, &mut app, &mut services);
        ui.set_root(root);
        ui.layout_all(&mut app, &mut services, bounds, scale_factor);

        let radio_bounds = find_first_bounds_with_size(&ui, root, 40.0, 40.0)
            .expect("expected a 40x40 radio chrome bounds");
        let press_at = Point::new(
            Px(radio_bounds.origin.x.0 + radio_bounds.size.width.0 * 0.5),
            Px(radio_bounds.origin.y.0 + radio_bounds.size.height.0 * 0.5),
        );

        ui.dispatch_event(
            &mut app,
            &mut services,
            &pointer_down(PointerId(1), press_at),
        );

        let mut ripple_center: Option<Point> = None;
        let mut saw_ripple_clip = false;
        for _ in 0..4 {
            app.advance_frame();

            let root = render(&mut ui, &mut app, &mut services);
            ui.set_root(root);
            ui.layout_all(&mut app, &mut services, bounds, scale_factor);

            let mut scene = Scene::default();
            ui.paint_all(&mut app, &mut services, bounds, &mut scene, scale_factor);

            saw_ripple_clip |= scene
                .ops()
                .iter()
                .any(|op| matches!(op, SceneOp::PushClipRRect { .. }));

            for op in scene.ops() {
                let SceneOp::Quad {
                    order,
                    rect: circle,
                    background,
                    border,
                    corner_radii,
                    ..
                } = op
                else {
                    continue;
                };

                if order != &DrawOrder(1) {
                    continue;
                }
                if border != &Edges::all(Px(0.0)) || background.a <= 0.01 {
                    continue;
                }
                if circle.size.width.0 <= 14.0 || circle.size.height.0 <= 14.0 {
                    continue;
                }

                let r = corner_radii.top_left.0;
                let r_ok = (corner_radii.top_right.0 - r).abs() < 1e-3
                    && (corner_radii.bottom_left.0 - r).abs() < 1e-3
                    && (corner_radii.bottom_right.0 - r).abs() < 1e-3;
                if !r_ok {
                    continue;
                }
                if (circle.size.width.0 * 0.5 - r).abs() > 1e-3
                    || (circle.size.height.0 * 0.5 - r).abs() > 1e-3
                {
                    continue;
                }

                ripple_center = Some(Point::new(
                    Px(circle.origin.x.0 + circle.size.width.0 * 0.5),
                    Px(circle.origin.y.0 + circle.size.height.0 * 0.5),
                ));
                break;
            }

            if ripple_center.is_some() {
                break;
            }
        }

        let Some(ripple_center) = ripple_center else {
            panic!("expected a ripple circle quad in the scene");
        };
        assert!(
            saw_ripple_clip,
            "expected ripple to be clipped to its state-layer bounds (scale={scale_factor})"
        );

        assert!(
            (ripple_center.x.0 - press_at.x.0).abs() < 0.75
                && (ripple_center.y.0 - press_at.y.0).abs() < 0.75,
            "expected ripple origin to match pointer down position (scale={scale_factor}): ripple_center={ripple_center:?} press_at={press_at:?}"
        );
    }
}

#[test]
fn switch_ripple_origin_tracks_pointer_down_position() {
    for scale_factor in [1.0, 1.25, 2.0] {
        let mut app = TestHost::default();
        app.set_global(PlatformCapabilities::default());

        let cfg = fret_ui_material3::tokens::v30::theme_config_with_colors(
            fret_ui_material3::tokens::v30::TypographyOptions::default(),
            fret_ui_material3::tokens::v30::ColorSchemeOptions::default(),
        );
        Theme::with_global_mut(&mut app, |theme| theme.apply_config(&cfg));

        let theme = Theme::global(&app);
        let track_width = theme
            .metric_by_key("md.comp.switch.track.width")
            .unwrap_or(Px(52.0));
        let state_layer = theme
            .metric_by_key("md.comp.switch.state-layer.size")
            .unwrap_or(Px(40.0));

        let window = AppWindowId::default();
        let mut services = FakeUiServices::default();
        let mut ui: UiTree<TestHost> = UiTree::new();
        ui.set_window(window);

        let bounds = Rect::new(
            Point::new(Px(0.0), Px(0.0)),
            Size::new(Px(300.0), Px(200.0)),
        );

        let selected = app.models_mut().insert(false);

        let render =
            |ui: &mut UiTree<TestHost>, app: &mut TestHost, services: &mut dyn UiServices| {
                fret_ui::declarative::render_root(ui, app, services, window, bounds, "root", |cx| {
                    let child = fret_ui_material3::Switch::new(selected.clone())
                        .a11y_label("switch")
                        .into_element(cx);
                    vec![with_padding(cx, Px(37.0), child)]
                })
            };

        let root = render(&mut ui, &mut app, &mut services);
        ui.set_root(root);
        ui.layout_all(&mut app, &mut services, bounds, scale_factor);

        let switch_bounds = find_first_bounds_with_size(&ui, root, track_width.0, state_layer.0)
            .expect("expected a switch outer bounds");
        let press_at = Point::new(
            Px(switch_bounds.origin.x.0 + switch_bounds.size.width.0 * 0.5),
            Px(switch_bounds.origin.y.0 + switch_bounds.size.height.0 * 0.5),
        );

        ui.dispatch_event(
            &mut app,
            &mut services,
            &pointer_down(PointerId(1), press_at),
        );

        let mut ripple_center: Option<Point> = None;
        let mut saw_ripple_clip = false;
        for _ in 0..4 {
            app.advance_frame();

            let root = render(&mut ui, &mut app, &mut services);
            ui.set_root(root);
            ui.layout_all(&mut app, &mut services, bounds, scale_factor);

            let mut scene = Scene::default();
            ui.paint_all(&mut app, &mut services, bounds, &mut scene, scale_factor);

            saw_ripple_clip |= scene
                .ops()
                .iter()
                .any(|op| matches!(op, SceneOp::PushClipRRect { .. }));

            for op in scene.ops() {
                let SceneOp::Quad {
                    order,
                    rect: circle,
                    background,
                    border,
                    corner_radii,
                    ..
                } = op
                else {
                    continue;
                };

                if order != &DrawOrder(1) {
                    continue;
                }
                if border != &Edges::all(Px(0.0)) || background.a <= 0.01 {
                    continue;
                }
                if circle.size.width.0 <= 14.0 || circle.size.height.0 <= 14.0 {
                    continue;
                }

                let r = corner_radii.top_left.0;
                let r_ok = (corner_radii.top_right.0 - r).abs() < 1e-3
                    && (corner_radii.bottom_left.0 - r).abs() < 1e-3
                    && (corner_radii.bottom_right.0 - r).abs() < 1e-3;
                if !r_ok {
                    continue;
                }
                if (circle.size.width.0 * 0.5 - r).abs() > 1e-3
                    || (circle.size.height.0 * 0.5 - r).abs() > 1e-3
                {
                    continue;
                }

                ripple_center = Some(Point::new(
                    Px(circle.origin.x.0 + circle.size.width.0 * 0.5),
                    Px(circle.origin.y.0 + circle.size.height.0 * 0.5),
                ));
                break;
            }

            if ripple_center.is_some() {
                break;
            }
        }

        let Some(ripple_center) = ripple_center else {
            panic!("expected a ripple circle quad in the scene");
        };
        assert!(
            saw_ripple_clip,
            "expected ripple to be clipped to its state-layer bounds (scale={scale_factor})"
        );

        assert!(
            (ripple_center.x.0 - press_at.x.0).abs() < 0.75
                && (ripple_center.y.0 - press_at.y.0).abs() < 0.75,
            "expected ripple origin to match pointer down position (scale={scale_factor}): ripple_center={ripple_center:?} press_at={press_at:?}"
        );
    }
}

#[test]
fn switch_keyboard_ripple_origin_ignores_stale_pointer_down() {
    let mut app = TestHost::default();
    app.set_global(PlatformCapabilities::default());

    let cfg = fret_ui_material3::tokens::v30::theme_config_with_colors(
        fret_ui_material3::tokens::v30::TypographyOptions::default(),
        fret_ui_material3::tokens::v30::ColorSchemeOptions::default(),
    );
    Theme::with_global_mut(&mut app, |theme| theme.apply_config(&cfg));

    let theme = Theme::global(&app);
    let track_width = theme
        .metric_by_key("md.comp.switch.track.width")
        .unwrap_or(Px(52.0));
    let track_height = theme
        .metric_by_key("md.comp.switch.track.height")
        .unwrap_or(Px(32.0));
    let state_layer = theme
        .metric_by_key("md.comp.switch.state-layer.size")
        .unwrap_or(Px(40.0));

    let window = AppWindowId::default();
    let mut services = FakeUiServices::default();
    let mut ui: UiTree<TestHost> = UiTree::new();
    ui.set_window(window);

    let bounds = Rect::new(
        Point::new(Px(0.0), Px(0.0)),
        Size::new(Px(300.0), Px(200.0)),
    );

    let selected = app.models_mut().insert(false);

    let render = |ui: &mut UiTree<TestHost>, app: &mut TestHost, services: &mut dyn UiServices| {
        fret_ui::declarative::render_root(ui, app, services, window, bounds, "root", |cx| {
            let child = fret_ui_material3::Switch::new(selected.clone())
                .a11y_label("switch")
                .into_element(cx);
            vec![with_padding(cx, Px(37.0), child)]
        })
    };

    let root = render(&mut ui, &mut app, &mut services);
    ui.set_root(root);
    ui.layout_all(&mut app, &mut services, bounds, 1.0);

    let track_bounds = find_first_bounds_with_size(&ui, root, track_width.0, track_height.0)
        .expect("expected switch track bounds");
    let old_press_at = Point::new(
        Px(track_bounds.origin.x.0 + 2.0),
        Px(track_bounds.origin.y.0 + 2.0),
    );

    ui.dispatch_event(
        &mut app,
        &mut services,
        &pointer_down(PointerId(1), old_press_at),
    );
    ui.dispatch_event(
        &mut app,
        &mut services,
        &pointer_up(PointerId(1), old_press_at),
    );

    // Let the pointer-started ripple fully finish so we don't confuse it with the keyboard ripple.
    for _ in 0..120 {
        app.advance_frame();
        let root = render(&mut ui, &mut app, &mut services);
        ui.set_root(root);
        ui.layout_all(&mut app, &mut services, bounds, 1.0);
        let mut scene = Scene::default();
        ui.paint_all(&mut app, &mut services, bounds, &mut scene, 1.0);
    }

    // Ensure keyboard events are delivered by explicitly focusing the switch node via semantics.
    let root = render(&mut ui, &mut app, &mut services);
    ui.set_root(root);
    ui.request_semantics_snapshot();
    ui.layout_all(&mut app, &mut services, bounds, 1.0);
    let focus: NodeId = ui
        .semantics_snapshot()
        .and_then(|snapshot| {
            snapshot.nodes.iter().find_map(|node| {
                if node.label.as_deref() == Some("switch") {
                    Some(node.id)
                } else {
                    None
                }
            })
        })
        .expect("expected switch node in semantics snapshot");
    ui.set_focus(Some(focus));

    ui.dispatch_event(&mut app, &mut services, &key_down(KeyCode::Space));
    assert!(
        fret_ui::input_modality::is_keyboard(&mut app, Some(window)),
        "expected keydown to switch input modality to keyboard"
    );

    let mut expected_center: Option<Point> = None;
    let mut ripple_center: Option<Point> = None;
    for attempt in 0..6 {
        if attempt > 0 {
            app.advance_frame();
        }

        let root = render(&mut ui, &mut app, &mut services);
        ui.set_root(root);
        ui.layout_all(&mut app, &mut services, bounds, 1.0);

        let mut scene = Scene::default();
        ui.paint_all(&mut app, &mut services, bounds, &mut scene, 1.0);

        // Scene ops may contain transforms. Always compare centers in the same coordinate space
        // by applying the transform stack while scanning.
        let mut transform = Transform2D::IDENTITY;
        let mut transform_stack: Vec<Transform2D> = Vec::new();
        let mut clip_stack: Vec<Option<Point>> = Vec::new();

        for op in scene.ops() {
            match *op {
                SceneOp::PushTransform { transform: next } => {
                    transform_stack.push(transform);
                    transform = transform.compose(next);
                }
                SceneOp::PopTransform => {
                    transform = transform_stack.pop().unwrap_or(Transform2D::IDENTITY);
                }
                SceneOp::PushClipRect { .. } => {
                    clip_stack.push(None);
                }
                SceneOp::PushClipRRect { rect, .. } => {
                    let is_state_layer = (rect.size.width.0 - state_layer.0).abs() < 0.25
                        && (rect.size.height.0 - state_layer.0).abs() < 0.25;
                    let center = Point::new(
                        Px(rect.origin.x.0 + rect.size.width.0 * 0.5),
                        Px(rect.origin.y.0 + rect.size.height.0 * 0.5),
                    );
                    clip_stack.push(is_state_layer.then_some(transform.apply_point(center)));
                }
                SceneOp::PopClip => {
                    clip_stack.pop();
                }
                SceneOp::Quad {
                    order,
                    rect,
                    background,
                    border,
                    corner_radii,
                    ..
                } => {
                    let Some(center_expected) = clip_stack.iter().rev().find_map(|center| *center)
                    else {
                        continue;
                    };
                    if order != DrawOrder(1)
                        || border != Edges::all(Px(0.0))
                        || background.a <= 0.001
                        || background.a >= 0.9
                        || (rect.size.width.0 - rect.size.height.0).abs() >= 0.25
                    {
                        continue;
                    }

                    let r = corner_radii.top_left.0;
                    let r_ok = (corner_radii.top_right.0 - r).abs() < 0.25
                        && (corner_radii.bottom_left.0 - r).abs() < 0.25
                        && (corner_radii.bottom_right.0 - r).abs() < 0.25;
                    if !r_ok {
                        continue;
                    }
                    if (rect.size.width.0 * 0.5 - r).abs() > 0.25
                        || (rect.size.height.0 * 0.5 - r).abs() > 0.25
                    {
                        continue;
                    }

                    let center_ripple = Point::new(
                        Px(rect.origin.x.0 + rect.size.width.0 * 0.5),
                        Px(rect.origin.y.0 + rect.size.height.0 * 0.5),
                    );
                    expected_center = Some(center_expected);
                    ripple_center = Some(transform.apply_point(center_ripple));
                    break;
                }
                _ => {}
            }
        }

        if expected_center.is_some() && ripple_center.is_some() {
            break;
        }
    }

    let expected_center = expected_center.expect("expected state-layer bounds quad");
    let ripple_center = ripple_center.expect("expected a ripple quad");

    assert!(
        (ripple_center.x.0 - expected_center.x.0).abs() < 0.75
            && (ripple_center.y.0 - expected_center.y.0).abs() < 0.75,
        "expected keyboard ripple origin to be centered in the state-layer bounds: ripple_center={ripple_center:?} expected_center={expected_center:?}"
    );
    assert!(
        (ripple_center.x.0 - old_press_at.x.0).abs() > 2.0
            || (ripple_center.y.0 - old_press_at.y.0).abs() > 2.0,
        "expected keyboard ripple origin to ignore stale pointer down: ripple_center={ripple_center:?} old_press_at={old_press_at:?}"
    );

    ui.dispatch_event(&mut app, &mut services, &key_up(KeyCode::Space));
}

#[test]
fn switch_ripple_holds_for_minimum_press_duration_before_fade() {
    let mut app = TestHost::default();
    app.set_global(PlatformCapabilities::default());

    let cfg = fret_ui_material3::tokens::v30::theme_config_with_colors(
        fret_ui_material3::tokens::v30::TypographyOptions::default(),
        fret_ui_material3::tokens::v30::ColorSchemeOptions::default(),
    );
    Theme::with_global_mut(&mut app, |theme| theme.apply_config(&cfg));

    let theme = Theme::global(&app);
    let min_frames = fret_ui_material3::motion::ms_to_frames(225);
    let track_width = theme
        .metric_by_key("md.comp.switch.track.width")
        .unwrap_or(Px(52.0));
    let track_height = theme
        .metric_by_key("md.comp.switch.track.height")
        .unwrap_or(Px(32.0));

    let window = AppWindowId::default();
    let mut services = FakeUiServices::default();
    let mut ui: UiTree<TestHost> = UiTree::new();
    ui.set_window(window);

    let bounds = Rect::new(
        Point::new(Px(0.0), Px(0.0)),
        Size::new(Px(300.0), Px(200.0)),
    );

    let selected = app.models_mut().insert(false);

    let render = |ui: &mut UiTree<TestHost>, app: &mut TestHost, services: &mut dyn UiServices| {
        fret_ui::declarative::render_root(ui, app, services, window, bounds, "root", |cx| {
            vec![
                fret_ui_material3::Switch::new(selected.clone())
                    .a11y_label("switch")
                    .into_element(cx),
            ]
        })
    };

    let root = render(&mut ui, &mut app, &mut services);
    ui.set_root(root);
    ui.layout_all(&mut app, &mut services, bounds, 1.0);

    // Ensure the pressable is focused so it responds to keyboard events.
    let _ = find_first_bounds_with_size(&ui, root, track_width.0, track_height.0)
        .expect("expected switch track bounds");
    ui.request_semantics_snapshot();
    ui.layout_all(&mut app, &mut services, bounds, 1.0);
    let focus: NodeId = ui
        .semantics_snapshot()
        .and_then(|snapshot| {
            snapshot.nodes.iter().find_map(|node| {
                if node.label.as_deref() == Some("switch") {
                    Some(node.id)
                } else {
                    None
                }
            })
        })
        .expect("expected switch node in semantics snapshot");
    ui.set_focus(Some(focus));

    ui.dispatch_event(&mut app, &mut services, &key_down(KeyCode::Space));

    // Ensure the ripple has started (pressed rising observed).
    let root = render(&mut ui, &mut app, &mut services);
    ui.set_root(root);
    ui.layout_all(&mut app, &mut services, bounds, 1.0);
    let mut scene = Scene::default();
    ui.paint_all(&mut app, &mut services, bounds, &mut scene, 1.0);

    ui.dispatch_event(&mut app, &mut services, &key_up(KeyCode::Space));

    let mut held_alpha: Option<f32> = None;
    let mut saw_fade = false;
    for frame_offset in 0..(min_frames.saturating_add(3)) {
        if frame_offset > 0 {
            app.advance_frame();
        }

        let root = render(&mut ui, &mut app, &mut services);
        ui.set_root(root);
        ui.layout_all(&mut app, &mut services, bounds, 1.0);
        let mut scene = Scene::default();
        ui.paint_all(&mut app, &mut services, bounds, &mut scene, 1.0);

        let ripple_alpha = scene
            .ops()
            .iter()
            .filter_map(|op| match op {
                SceneOp::Quad {
                    order,
                    background,
                    border,
                    ..
                } if *order == DrawOrder(1) && *border == Edges::all(Px(0.0)) => Some(background.a),
                _ => None,
            })
            .next()
            .unwrap_or(0.0);

        if held_alpha.is_none() && ripple_alpha > 0.001 {
            held_alpha = Some(ripple_alpha);
        }
        let Some(held_alpha) = held_alpha else {
            continue;
        };

        if frame_offset < min_frames {
            assert!(
                (ripple_alpha - held_alpha).abs() < 1e-3,
                "expected ripple alpha to hold until min press duration: offset={frame_offset} ripple_alpha={ripple_alpha} held_alpha={held_alpha}"
            );
        }

        if frame_offset >= min_frames {
            assert!(
                ripple_alpha < held_alpha - 1e-4,
                "expected ripple alpha to start fading after min press duration: offset={frame_offset} ripple_alpha={ripple_alpha} held_alpha={held_alpha} min_frames={min_frames}"
            );
            saw_fade = true;
            break;
        }
    }

    assert!(
        held_alpha.is_some(),
        "expected to observe a keyboard ripple"
    );
    assert!(saw_fade, "expected the ripple to start fading");
}

#[test]
fn tabs_pressed_scene_structure_is_stable() {
    use std::sync::Arc;

    use fret_ui_material3::{TabItem, Tabs};

    let mut app = TestHost::default();
    app.set_global(PlatformCapabilities::default());

    let cfg = fret_ui_material3::tokens::v30::theme_config_with_colors(
        fret_ui_material3::tokens::v30::TypographyOptions::default(),
        fret_ui_material3::tokens::v30::ColorSchemeOptions::default(),
    );
    Theme::with_global_mut(&mut app, |theme| theme.apply_config(&cfg));

    let window = AppWindowId::default();
    let mut services = FakeUiServices::default();
    let mut ui: UiTree<TestHost> = UiTree::new();
    ui.set_window(window);

    let bounds = Rect::new(
        Point::new(Px(0.0), Px(0.0)),
        Size::new(Px(360.0), Px(240.0)),
    );

    let selected = app.models_mut().insert(Arc::<str>::from("b"));
    let items = vec![
        TabItem::new("a", "A").test_id("tab-a"),
        TabItem::new("b", "B").test_id("tab-b"),
        TabItem::new("c", "C").test_id("tab-c"),
    ];

    let render = |ui: &mut UiTree<TestHost>, app: &mut TestHost, services: &mut dyn UiServices| {
        fret_ui::declarative::render_root(ui, app, services, window, bounds, "root", |cx| {
            let tabs = Tabs::new(selected.clone())
                .items(items.clone())
                .a11y_label("tabs")
                .into_element(cx);
            vec![with_padding(cx, Px(24.0), tabs)]
        })
    };

    let root = render(&mut ui, &mut app, &mut services);
    ui.set_root(root);
    ui.request_semantics_snapshot();
    ui.layout_all(&mut app, &mut services, bounds, 1.0);

    let tab_b: NodeId = ui
        .semantics_snapshot()
        .and_then(|snapshot| {
            snapshot.nodes.iter().find_map(|node| {
                if node.test_id.as_deref() == Some("tab-b") {
                    Some(node.id)
                } else {
                    None
                }
            })
        })
        .expect("expected tab-b in semantics snapshot");
    let tab_b_bounds = ui
        .debug_node_visual_bounds(tab_b)
        .expect("expected tab-b visual bounds");
    let press_at = Point::new(
        Px(tab_b_bounds.origin.x.0 + tab_b_bounds.size.width.0 * 0.5),
        Px(tab_b_bounds.origin.y.0 + tab_b_bounds.size.height.0 * 0.5),
    );

    ui.dispatch_event(
        &mut app,
        &mut services,
        &pointer_down(PointerId(1), press_at),
    );

    let mut baseline: Option<Vec<SceneSig>> = None;
    for frame in 0..6 {
        app.advance_frame();
        let root = render(&mut ui, &mut app, &mut services);
        ui.set_root(root);
        ui.layout_all(&mut app, &mut services, bounds, 1.0);

        let mut scene = Scene::default();
        ui.paint_all(&mut app, &mut services, bounds, &mut scene, 1.0);

        if frame < 2 {
            continue;
        }

        let sig = scene_signature(&scene);
        if let Some(prev) = baseline.as_ref() {
            assert_eq!(
                sig, *prev,
                "expected Tabs to keep a stable scene structure while pressed"
            );
        } else {
            baseline = Some(sig);
        }
    }

    ui.dispatch_event(&mut app, &mut services, &pointer_up(PointerId(1), press_at));
}
