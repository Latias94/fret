use std::{
    any::{Any, TypeId},
    collections::{BTreeMap, HashMap, HashSet},
    path::{Path, PathBuf},
    sync::Arc,
};

use fret_core::MouseButtons;
use fret_core::{
    AppWindowId, DrawOrder, Edges, Event, KeyCode, Modifiers, MouseButton, NodeId, Point,
    PointerEvent, PointerId, PointerType, Px, Rect, Scene, SceneOp, Size, Transform2D, UiServices,
};
use fret_runtime::{
    CommandRegistry, CommandsHost, DragHost, DragKindId, DragSession, DragSessionId, Effect,
    EffectSink, FrameId, GlobalsHost, Model, ModelHost, ModelId, ModelStore, ModelsHost,
    PlatformCapabilities, TickId, TimeHost,
};
use fret_ui::element::{AnyElement, ContainerProps};
use fret_ui::{Theme, UiTree};
use fret_ui_kit::declarative::action_hooks::ActionHooksExt as _;
use fret_ui_material3::tokens::v30::{
    ColorSchemeOptions, DynamicVariant, SchemeMode, TypographyOptions, theme_config_with_colors,
};
use serde::{Deserialize, Serialize};

mod interaction_harness;
use interaction_harness::{
    QuadGeomSig, SceneSig, scene_quad_geometry_signature, scene_quad_signature, scene_signature,
};

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

    fn any_drag_session(&self, mut predicate: impl FnMut(&DragSession) -> bool) -> bool {
        self.drags.values().any(|session| predicate(session))
    }

    fn find_drag_pointer_id(
        &self,
        mut predicate: impl FnMut(&DragSession) -> bool,
    ) -> Option<PointerId> {
        self.drags
            .iter()
            .find_map(|(pointer_id, session)| predicate(session).then_some(*pointer_id))
    }

    fn cancel_drag_sessions(
        &mut self,
        mut predicate: impl FnMut(&DragSession) -> bool,
    ) -> Vec<PointerId> {
        let canceled: Vec<PointerId> = self
            .drags
            .iter()
            .filter_map(|(pointer_id, session)| predicate(session).then_some(*pointer_id))
            .collect();

        for pointer_id in &canceled {
            self.drags.remove(pointer_id);
        }

        canceled
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

fn apply_material_theme(app: &mut TestHost, mode: SchemeMode, variant: DynamicVariant) {
    let mut colors = ColorSchemeOptions::default();
    colors.mode = mode;
    colors.variant = variant;

    let cfg = theme_config_with_colors(TypographyOptions::default(), colors);
    Theme::with_global_mut(app, |theme| theme.apply_config(&cfg));
}

fn repo_root() -> PathBuf {
    let manifest_dir = PathBuf::from(env!("CARGO_MANIFEST_DIR"));
    manifest_dir
        .parent()
        .and_then(|p| p.parent())
        .map(Path::to_path_buf)
        .expect("repo root")
}

fn material3_goldens_dir() -> PathBuf {
    repo_root()
        .join("goldens")
        .join("material3-headless")
        .join("v1")
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
struct Material3HeadlessGoldenV1 {
    signature: Vec<SceneOpSigV1>,
    quads: Vec<QuadV1>,
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
struct Material3HeadlessSuiteV1 {
    cases: BTreeMap<String, Material3HeadlessGoldenV1>,
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
#[serde(tag = "kind", rename_all = "snake_case")]
enum SceneOpSigV1 {
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
    Quad { order: u32 },
    Image { order: u32 },
    ImageRegion { order: u32 },
    MaskImage { order: u32 },
    SvgMaskIcon { order: u32 },
    SvgImage { order: u32 },
    Text { order: u32 },
    Path { order: u32 },
    ViewportSurface { order: u32 },
}

impl From<SceneSig> for SceneOpSigV1 {
    fn from(sig: SceneSig) -> Self {
        match sig {
            SceneSig::PushTransform => Self::PushTransform,
            SceneSig::PopTransform => Self::PopTransform,
            SceneSig::PushOpacity => Self::PushOpacity,
            SceneSig::PopOpacity => Self::PopOpacity,
            SceneSig::PushLayer => Self::PushLayer,
            SceneSig::PopLayer => Self::PopLayer,
            SceneSig::PushClipRect => Self::PushClipRect,
            SceneSig::PushClipRRect => Self::PushClipRRect,
            SceneSig::PopClip => Self::PopClip,
            SceneSig::PushEffect => Self::PushEffect,
            SceneSig::PopEffect => Self::PopEffect,
            SceneSig::Quad(order) => Self::Quad { order: order.0 },
            SceneSig::Image(order) => Self::Image { order: order.0 },
            SceneSig::ImageRegion(order) => Self::ImageRegion { order: order.0 },
            SceneSig::MaskImage(order) => Self::MaskImage { order: order.0 },
            SceneSig::SvgMaskIcon(order) => Self::SvgMaskIcon { order: order.0 },
            SceneSig::SvgImage(order) => Self::SvgImage { order: order.0 },
            SceneSig::Text(order) => Self::Text { order: order.0 },
            SceneSig::Path(order) => Self::Path { order: order.0 },
            SceneSig::ViewportSurface(order) => Self::ViewportSurface { order: order.0 },
        }
    }
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
struct QuadV1 {
    order: u32,
    rect: [i32; 4],
    background: [i32; 4],
    border: [i32; 4],
    corner_radii: [i32; 4],
}

impl From<interaction_harness::QuadSig> for QuadV1 {
    fn from(quad: interaction_harness::QuadSig) -> Self {
        Self {
            order: quad.order.0,
            rect: [quad.rect.x, quad.rect.y, quad.rect.w, quad.rect.h],
            background: [
                quad.background.r,
                quad.background.g,
                quad.background.b,
                quad.background.a,
            ],
            border: [
                quad.border.top,
                quad.border.right,
                quad.border.bottom,
                quad.border.left,
            ],
            corner_radii: [
                quad.corner_radii.top_left,
                quad.corner_radii.top_right,
                quad.corner_radii.bottom_right,
                quad.corner_radii.bottom_left,
            ],
        }
    }
}

fn material3_scene_snapshot_v1(scene: &Scene) -> Material3HeadlessGoldenV1 {
    Material3HeadlessGoldenV1 {
        signature: scene_signature(scene)
            .into_iter()
            .map(SceneOpSigV1::from)
            .collect(),
        quads: scene_quad_signature(scene)
            .into_iter()
            .map(QuadV1::from)
            .collect(),
    }
}

fn settle_material3_scene_snapshot_v1(
    app: &mut TestHost,
    ui: &mut UiTree<TestHost>,
    services: &mut dyn UiServices,
    bounds: Rect,
    scale_factor: f32,
    settle_from_frame: usize,
    total_frames: usize,
    stable_message: &str,
    render: &impl Fn(&mut UiTree<TestHost>, &mut TestHost, &mut dyn UiServices) -> NodeId,
) -> Material3HeadlessGoldenV1 {
    let mut settled: Option<Material3HeadlessGoldenV1> = None;
    for frame in 0..total_frames {
        app.advance_frame();
        let root = render(ui, app, services);
        ui.set_root(root);
        ui.layout_all(app, services, bounds, scale_factor);

        let mut scene = Scene::default();
        ui.paint_all(app, services, bounds, &mut scene, scale_factor);

        if frame < settle_from_frame {
            continue;
        }

        let snapshot = material3_scene_snapshot_v1(&scene);
        if let Some(prev) = settled.as_ref() {
            assert_eq!(snapshot, *prev, "{stable_message}");
        } else {
            settled = Some(snapshot);
        }
    }

    settled.unwrap_or_else(|| panic!("expected a settled snapshot: {stable_message}"))
}

fn write_or_assert_material3_suite_v1(name: &str, suite: &Material3HeadlessSuiteV1) {
    let path = material3_goldens_dir().join(format!("{name}.json"));

    if std::env::var_os("FRET_UPDATE_GOLDENS").is_some() {
        std::fs::create_dir_all(material3_goldens_dir()).expect("create material3 goldens dir");
        let json = serde_json::to_string_pretty(suite).expect("serialize material3 suite golden");
        std::fs::write(&path, json).expect("write material3 suite golden");
        return;
    }

    let golden_json = std::fs::read_to_string(&path).unwrap_or_else(|err| {
        panic!(
            "missing material3 suite golden: {}\nerror: {err}\n\nTo (re)generate:\n  $env:FRET_UPDATE_GOLDENS='1'; cargo nextest run -p fret-ui-material3 --test radio_alignment -- material3_headless\n",
            path.display()
        )
    });
    let golden: Material3HeadlessSuiteV1 =
        serde_json::from_str(&golden_json).expect("parse material3 suite golden");

    assert_eq!(
        *suite,
        golden,
        "material3 suite golden mismatch: {}\n\nTo update:\n  $env:FRET_UPDATE_GOLDENS='1'; cargo nextest run -p fret-ui-material3 --test radio_alignment -- material3_headless",
        path.display()
    );
}

fn run_overlay_frame(
    ui: &mut UiTree<TestHost>,
    app: &mut TestHost,
    services: &mut dyn UiServices,
    window: AppWindowId,
    bounds: Rect,
    capture_semantics: bool,
    render: impl FnOnce(&mut UiTree<TestHost>, &mut TestHost, &mut dyn UiServices) -> NodeId,
) {
    use fret_ui_kit::OverlayController;

    app.advance_frame();
    OverlayController::begin_frame(app, window);

    let root = render(ui, app, services);
    ui.set_root(root);
    OverlayController::render(ui, app, services, window, bounds);

    if capture_semantics {
        ui.request_semantics_snapshot();
    }
    ui.layout_all(app, services, bounds, 1.0);
    let mut scene = Scene::default();
    ui.paint_all(app, services, bounds, &mut scene, 1.0);
}

fn run_overlay_frame_scaled(
    ui: &mut UiTree<TestHost>,
    app: &mut TestHost,
    services: &mut dyn UiServices,
    window: AppWindowId,
    bounds: Rect,
    scale_factor: f32,
    capture_semantics: bool,
    render: impl FnOnce(&mut UiTree<TestHost>, &mut TestHost, &mut dyn UiServices) -> NodeId,
) {
    use fret_ui_kit::OverlayController;

    app.advance_frame();
    OverlayController::begin_frame(app, window);

    let root = render(ui, app, services);
    ui.set_root(root);
    OverlayController::render(ui, app, services, window, bounds);

    if capture_semantics {
        ui.request_semantics_snapshot();
    }
    ui.layout_all(app, services, bounds, scale_factor);

    let mut scene = Scene::default();
    ui.paint_all(app, services, bounds, &mut scene, scale_factor);
}

fn run_overlay_frame_with_scene_scaled(
    ui: &mut UiTree<TestHost>,
    app: &mut TestHost,
    services: &mut dyn UiServices,
    window: AppWindowId,
    bounds: Rect,
    scale_factor: f32,
    capture_semantics: bool,
    render: impl FnOnce(&mut UiTree<TestHost>, &mut TestHost, &mut dyn UiServices) -> NodeId,
) -> Scene {
    use fret_ui_kit::OverlayController;

    app.advance_frame();
    OverlayController::begin_frame(app, window);

    let root = render(ui, app, services);
    ui.set_root(root);
    OverlayController::render(ui, app, services, window, bounds);

    if capture_semantics {
        ui.request_semantics_snapshot();
    }
    ui.layout_all(app, services, bounds, scale_factor);

    let mut scene = Scene::default();
    ui.paint_all(app, services, bounds, &mut scene, scale_factor);
    scene
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

fn pointer_move(pointer_id: PointerId, position: Point) -> Event {
    Event::Pointer(PointerEvent::Move {
        pointer_id,
        position,
        buttons: MouseButtons::default(),
        modifiers: Modifiers::default(),
        pointer_type: PointerType::Mouse,
    })
}

fn pointer_up(pointer_id: PointerId, position: Point) -> Event {
    Event::Pointer(PointerEvent::Up {
        pointer_id,
        position,
        button: MouseButton::Left,
        modifiers: Modifiers::default(),
        is_click: true,
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

    let cases = [
        (SchemeMode::Dark, DynamicVariant::TonalSpot, "dark/tonal"),
        (SchemeMode::Light, DynamicVariant::TonalSpot, "light/tonal"),
        (
            SchemeMode::Dark,
            DynamicVariant::Expressive,
            "dark/expressive",
        ),
        (
            SchemeMode::Light,
            DynamicVariant::Expressive,
            "light/expressive",
        ),
    ];

    for (mode, variant, label) in cases {
        let mut app = TestHost::default();
        app.set_global(PlatformCapabilities::default());
        apply_material_theme(&mut app, mode, variant);

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

        let render =
            |ui: &mut UiTree<TestHost>, app: &mut TestHost, services: &mut dyn UiServices| {
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

        let mut baseline_structure: Option<Vec<SceneSig>> = None;
        let mut baseline_quads: Option<Vec<QuadGeomSig>> = None;
        for frame in 0..24 {
            app.advance_frame();
            let root = render(&mut ui, &mut app, &mut services);
            ui.set_root(root);
            ui.layout_all(&mut app, &mut services, bounds, 1.0);

            let mut scene = Scene::default();
            ui.paint_all(&mut app, &mut services, bounds, &mut scene, 1.0);

            if frame >= 2 && frame < 7 {
                let sig = scene_signature(&scene);
                if let Some(prev) = baseline_structure.as_ref() {
                    assert_eq!(
                        sig, *prev,
                        "expected Tabs to keep a stable scene structure while pressed ({label})"
                    );
                } else {
                    baseline_structure = Some(sig);
                }
            }

            if frame >= 16 {
                let sig = scene_quad_geometry_signature(&scene);
                if let Some(prev) = baseline_quads.as_ref() {
                    assert_eq!(
                        sig, *prev,
                        "expected Tabs to keep stable quad geometry after animations settle ({label})"
                    );
                } else {
                    baseline_quads = Some(sig);
                }
            }
        }

        ui.dispatch_event(&mut app, &mut services, &pointer_up(PointerId(1), press_at));
    }
}

#[test]
fn icon_button_pressed_scene_structure_is_stable() {
    use fret_icons::ids;
    use fret_ui_material3::IconButton;

    let cases = [
        (SchemeMode::Dark, DynamicVariant::TonalSpot, "dark/tonal"),
        (SchemeMode::Light, DynamicVariant::TonalSpot, "light/tonal"),
        (
            SchemeMode::Dark,
            DynamicVariant::Expressive,
            "dark/expressive",
        ),
        (
            SchemeMode::Light,
            DynamicVariant::Expressive,
            "light/expressive",
        ),
    ];

    for (mode, variant, label) in cases {
        let mut app = TestHost::default();
        app.set_global(PlatformCapabilities::default());
        apply_material_theme(&mut app, mode, variant);

        let window = AppWindowId::default();
        let mut services = FakeUiServices::default();
        let mut ui: UiTree<TestHost> = UiTree::new();
        ui.set_window(window);

        let bounds = Rect::new(
            Point::new(Px(0.0), Px(0.0)),
            Size::new(Px(320.0), Px(240.0)),
        );

        let render =
            |ui: &mut UiTree<TestHost>, app: &mut TestHost, services: &mut dyn UiServices| {
                fret_ui::declarative::render_root(ui, app, services, window, bounds, "root", |cx| {
                    let button = IconButton::new(ids::ui::CHECK)
                        .a11y_label("icon button")
                        .test_id("icon-button")
                        .into_element(cx);
                    vec![with_padding(cx, Px(32.0), button)]
                })
            };

        let root = render(&mut ui, &mut app, &mut services);
        ui.set_root(root);
        ui.request_semantics_snapshot();
        ui.layout_all(&mut app, &mut services, bounds, 1.0);

        let button_node: NodeId = ui
            .semantics_snapshot()
            .and_then(|snapshot| {
                snapshot.nodes.iter().find_map(|node| {
                    if node.test_id.as_deref() == Some("icon-button") {
                        Some(node.id)
                    } else {
                        None
                    }
                })
            })
            .expect("expected icon-button in semantics snapshot");
        let button_bounds = ui
            .debug_node_visual_bounds(button_node)
            .expect("expected icon-button visual bounds");
        let press_at = Point::new(
            Px(button_bounds.origin.x.0 + button_bounds.size.width.0 * 0.5),
            Px(button_bounds.origin.y.0 + button_bounds.size.height.0 * 0.5),
        );

        ui.dispatch_event(
            &mut app,
            &mut services,
            &pointer_down(PointerId(1), press_at),
        );

        let mut baseline_structure: Option<Vec<SceneSig>> = None;
        let mut baseline_quads: Option<Vec<QuadGeomSig>> = None;
        for frame in 0..24 {
            app.advance_frame();
            let root = render(&mut ui, &mut app, &mut services);
            ui.set_root(root);
            ui.layout_all(&mut app, &mut services, bounds, 1.0);

            let mut scene = Scene::default();
            ui.paint_all(&mut app, &mut services, bounds, &mut scene, 1.0);

            if frame >= 2 && frame < 7 {
                let sig = scene_signature(&scene);
                if let Some(prev) = baseline_structure.as_ref() {
                    assert_eq!(
                        sig, *prev,
                        "expected IconButton to keep a stable scene structure while pressed ({label})"
                    );
                } else {
                    baseline_structure = Some(sig);
                }
            }

            if frame >= 16 {
                let sig = scene_quad_geometry_signature(&scene);
                if let Some(prev) = baseline_quads.as_ref() {
                    assert_eq!(
                        sig, *prev,
                        "expected IconButton to keep stable quad geometry after animations settle ({label})"
                    );
                } else {
                    baseline_quads = Some(sig);
                }
            }
        }

        ui.dispatch_event(&mut app, &mut services, &pointer_up(PointerId(1), press_at));
    }
}

#[test]
fn switch_pressed_scene_structure_is_stable() {
    use fret_ui_material3::Switch;

    let cases = [
        (SchemeMode::Dark, DynamicVariant::TonalSpot, "dark/tonal"),
        (SchemeMode::Light, DynamicVariant::TonalSpot, "light/tonal"),
        (
            SchemeMode::Dark,
            DynamicVariant::Expressive,
            "dark/expressive",
        ),
        (
            SchemeMode::Light,
            DynamicVariant::Expressive,
            "light/expressive",
        ),
    ];

    for (mode, variant, label) in cases {
        let mut app = TestHost::default();
        app.set_global(PlatformCapabilities::default());
        apply_material_theme(&mut app, mode, variant);

        let window = AppWindowId::default();
        let mut services = FakeUiServices::default();
        let mut ui: UiTree<TestHost> = UiTree::new();
        ui.set_window(window);

        let bounds = Rect::new(
            Point::new(Px(0.0), Px(0.0)),
            Size::new(Px(320.0), Px(240.0)),
        );

        let selected = app.models_mut().insert(false);
        let render =
            |ui: &mut UiTree<TestHost>, app: &mut TestHost, services: &mut dyn UiServices| {
                fret_ui::declarative::render_root(ui, app, services, window, bounds, "root", |cx| {
                    let switch = Switch::new(selected.clone())
                        .a11y_label("switch")
                        .test_id("switch")
                        .into_element(cx);
                    vec![with_padding(cx, Px(32.0), switch)]
                })
            };

        let root = render(&mut ui, &mut app, &mut services);
        ui.set_root(root);
        ui.request_semantics_snapshot();
        ui.layout_all(&mut app, &mut services, bounds, 1.0);

        let switch_node: NodeId = ui
            .semantics_snapshot()
            .and_then(|snapshot| {
                snapshot.nodes.iter().find_map(|node| {
                    if node.test_id.as_deref() == Some("switch") {
                        Some(node.id)
                    } else {
                        None
                    }
                })
            })
            .expect("expected switch in semantics snapshot");
        let switch_bounds = ui
            .debug_node_visual_bounds(switch_node)
            .expect("expected switch visual bounds");
        let press_at = Point::new(
            Px(switch_bounds.origin.x.0 + switch_bounds.size.width.0 * 0.5),
            Px(switch_bounds.origin.y.0 + switch_bounds.size.height.0 * 0.5),
        );

        ui.dispatch_event(
            &mut app,
            &mut services,
            &pointer_down(PointerId(1), press_at),
        );

        let mut baseline_structure: Option<Vec<SceneSig>> = None;
        let mut baseline_quads: Option<Vec<QuadGeomSig>> = None;
        for frame in 0..24 {
            app.advance_frame();
            let root = render(&mut ui, &mut app, &mut services);
            ui.set_root(root);
            ui.layout_all(&mut app, &mut services, bounds, 1.0);

            let mut scene = Scene::default();
            ui.paint_all(&mut app, &mut services, bounds, &mut scene, 1.0);

            if frame >= 2 && frame < 7 {
                let sig = scene_signature(&scene);
                if let Some(prev) = baseline_structure.as_ref() {
                    assert_eq!(
                        sig, *prev,
                        "expected Switch to keep a stable scene structure while pressed ({label})"
                    );
                } else {
                    baseline_structure = Some(sig);
                }
            }

            if frame >= 16 {
                let sig = scene_quad_geometry_signature(&scene);
                if let Some(prev) = baseline_quads.as_ref() {
                    assert_eq!(
                        sig, *prev,
                        "expected Switch to keep stable quad geometry after animations settle ({label})"
                    );
                } else {
                    baseline_quads = Some(sig);
                }
            }
        }

        ui.dispatch_event(&mut app, &mut services, &pointer_up(PointerId(1), press_at));
    }
}

#[test]
fn checkbox_pressed_scene_structure_is_stable() {
    use fret_ui_material3::Checkbox;

    let cases = [
        (SchemeMode::Dark, DynamicVariant::TonalSpot, "dark/tonal"),
        (SchemeMode::Light, DynamicVariant::TonalSpot, "light/tonal"),
        (
            SchemeMode::Dark,
            DynamicVariant::Expressive,
            "dark/expressive",
        ),
        (
            SchemeMode::Light,
            DynamicVariant::Expressive,
            "light/expressive",
        ),
    ];

    for (mode, variant, label) in cases {
        let mut app = TestHost::default();
        app.set_global(PlatformCapabilities::default());
        apply_material_theme(&mut app, mode, variant);

        let window = AppWindowId::default();
        let mut services = FakeUiServices::default();
        let mut ui: UiTree<TestHost> = UiTree::new();
        ui.set_window(window);

        let bounds = Rect::new(
            Point::new(Px(0.0), Px(0.0)),
            Size::new(Px(320.0), Px(240.0)),
        );

        let checked = app.models_mut().insert(false);
        let render =
            |ui: &mut UiTree<TestHost>, app: &mut TestHost, services: &mut dyn UiServices| {
                fret_ui::declarative::render_root(ui, app, services, window, bounds, "root", |cx| {
                    let checkbox = Checkbox::new(checked.clone())
                        .a11y_label("checkbox")
                        .test_id("checkbox")
                        .into_element(cx);
                    vec![with_padding(cx, Px(32.0), checkbox)]
                })
            };

        let root = render(&mut ui, &mut app, &mut services);
        ui.set_root(root);
        ui.request_semantics_snapshot();
        ui.layout_all(&mut app, &mut services, bounds, 1.0);

        let checkbox_node: NodeId = ui
            .semantics_snapshot()
            .and_then(|snapshot| {
                snapshot.nodes.iter().find_map(|node| {
                    if node.test_id.as_deref() == Some("checkbox") {
                        Some(node.id)
                    } else {
                        None
                    }
                })
            })
            .expect("expected checkbox in semantics snapshot");
        let checkbox_bounds = ui
            .debug_node_visual_bounds(checkbox_node)
            .expect("expected checkbox visual bounds");
        let press_at = Point::new(
            Px(checkbox_bounds.origin.x.0 + checkbox_bounds.size.width.0 * 0.5),
            Px(checkbox_bounds.origin.y.0 + checkbox_bounds.size.height.0 * 0.5),
        );

        ui.dispatch_event(
            &mut app,
            &mut services,
            &pointer_down(PointerId(1), press_at),
        );

        let mut baseline_structure: Option<Vec<SceneSig>> = None;
        let mut baseline_quads: Option<Vec<QuadGeomSig>> = None;
        for frame in 0..24 {
            app.advance_frame();
            let root = render(&mut ui, &mut app, &mut services);
            ui.set_root(root);
            ui.layout_all(&mut app, &mut services, bounds, 1.0);

            let mut scene = Scene::default();
            ui.paint_all(&mut app, &mut services, bounds, &mut scene, 1.0);

            if frame >= 2 && frame < 7 {
                let sig = scene_signature(&scene);
                if let Some(prev) = baseline_structure.as_ref() {
                    assert_eq!(
                        sig, *prev,
                        "expected Checkbox to keep a stable scene structure while pressed ({label})"
                    );
                } else {
                    baseline_structure = Some(sig);
                }
            }

            if frame >= 16 {
                let sig = scene_quad_geometry_signature(&scene);
                if let Some(prev) = baseline_quads.as_ref() {
                    assert_eq!(
                        sig, *prev,
                        "expected Checkbox to keep stable quad geometry after animations settle ({label})"
                    );
                } else {
                    baseline_quads = Some(sig);
                }
            }
        }

        ui.dispatch_event(&mut app, &mut services, &pointer_up(PointerId(1), press_at));
    }
}

#[test]
fn menu_pressed_scene_structure_is_stable() {
    use fret_ui_material3::menu::{Menu, MenuEntry, MenuItem};

    let cases = [
        (SchemeMode::Dark, DynamicVariant::TonalSpot, "dark/tonal"),
        (SchemeMode::Light, DynamicVariant::TonalSpot, "light/tonal"),
        (
            SchemeMode::Dark,
            DynamicVariant::Expressive,
            "dark/expressive",
        ),
        (
            SchemeMode::Light,
            DynamicVariant::Expressive,
            "light/expressive",
        ),
    ];

    for (mode, variant, label) in cases {
        let mut app = TestHost::default();
        app.set_global(PlatformCapabilities::default());
        apply_material_theme(&mut app, mode, variant);

        let window = AppWindowId::default();
        let mut services = FakeUiServices::default();
        let mut ui: UiTree<TestHost> = UiTree::new();
        ui.set_window(window);

        let bounds = Rect::new(
            Point::new(Px(0.0), Px(0.0)),
            Size::new(Px(360.0), Px(260.0)),
        );

        let entries = vec![
            MenuEntry::Item(MenuItem::new("A").test_id("menu-item-a")),
            MenuEntry::Item(MenuItem::new("B").test_id("menu-item-b")),
            MenuEntry::Item(MenuItem::new("C").test_id("menu-item-c")),
        ];

        let render =
            |ui: &mut UiTree<TestHost>, app: &mut TestHost, services: &mut dyn UiServices| {
                fret_ui::declarative::render_root(ui, app, services, window, bounds, "root", |cx| {
                    let menu = Menu::new()
                        .entries(entries.clone())
                        .a11y_label("menu")
                        .test_id("menu")
                        .into_element(cx);
                    vec![with_padding(cx, Px(24.0), menu)]
                })
            };

        let root = render(&mut ui, &mut app, &mut services);
        ui.set_root(root);
        ui.request_semantics_snapshot();
        ui.layout_all(&mut app, &mut services, bounds, 1.0);

        let item_b: NodeId = ui
            .semantics_snapshot()
            .and_then(|snapshot| {
                snapshot.nodes.iter().find_map(|node| {
                    if node.test_id.as_deref() == Some("menu-item-b") {
                        Some(node.id)
                    } else {
                        None
                    }
                })
            })
            .expect("expected menu-item-b in semantics snapshot");
        let item_bounds = ui
            .debug_node_visual_bounds(item_b)
            .expect("expected menu-item-b visual bounds");
        let press_at = Point::new(
            Px(item_bounds.origin.x.0 + item_bounds.size.width.0 * 0.5),
            Px(item_bounds.origin.y.0 + item_bounds.size.height.0 * 0.5),
        );

        ui.dispatch_event(
            &mut app,
            &mut services,
            &pointer_down(PointerId(1), press_at),
        );

        let mut baseline_structure: Option<Vec<SceneSig>> = None;
        let mut baseline_quads: Option<Vec<QuadGeomSig>> = None;
        for frame in 0..24 {
            app.advance_frame();
            let root = render(&mut ui, &mut app, &mut services);
            ui.set_root(root);
            ui.layout_all(&mut app, &mut services, bounds, 1.0);

            let mut scene = Scene::default();
            ui.paint_all(&mut app, &mut services, bounds, &mut scene, 1.0);

            if frame >= 2 && frame < 7 {
                let sig = scene_signature(&scene);
                if let Some(prev) = baseline_structure.as_ref() {
                    assert_eq!(
                        sig, *prev,
                        "expected Menu to keep a stable scene structure while pressed ({label})"
                    );
                } else {
                    baseline_structure = Some(sig);
                }
            }

            if frame >= 16 {
                let sig = scene_quad_geometry_signature(&scene);
                if let Some(prev) = baseline_quads.as_ref() {
                    assert_eq!(
                        sig, *prev,
                        "expected Menu to keep stable quad geometry after animations settle ({label})"
                    );
                } else {
                    baseline_quads = Some(sig);
                }
            }
        }

        ui.dispatch_event(&mut app, &mut services, &pointer_up(PointerId(1), press_at));
    }
}

#[test]
fn dialog_focus_is_contained_and_restored_across_schemes() {
    use fret_ui_material3::{Button, Dialog, DialogAction};

    let cases = [
        (SchemeMode::Dark, DynamicVariant::TonalSpot, "dark/tonal"),
        (SchemeMode::Light, DynamicVariant::TonalSpot, "light/tonal"),
        (
            SchemeMode::Dark,
            DynamicVariant::Expressive,
            "dark/expressive",
        ),
        (
            SchemeMode::Light,
            DynamicVariant::Expressive,
            "light/expressive",
        ),
    ];

    for (mode, variant, label) in cases {
        let mut app = TestHost::default();
        app.set_global(PlatformCapabilities::default());
        apply_material_theme(&mut app, mode, variant);

        let window = AppWindowId::default();
        let mut services = FakeUiServices::default();
        let mut ui: UiTree<TestHost> = UiTree::new();
        ui.set_window(window);

        let bounds = Rect::new(
            Point::new(Px(0.0), Px(0.0)),
            Size::new(Px(420.0), Px(320.0)),
        );

        let open = app.models_mut().insert(false);

        let render =
            |ui: &mut UiTree<TestHost>, app: &mut TestHost, services: &mut dyn UiServices| {
                fret_ui::declarative::render_root(ui, app, services, window, bounds, "root", |cx| {
                    let dialog = Dialog::new(open.clone())
                        .headline("Dialog")
                        .supporting_text("Body")
                        .actions(vec![DialogAction::new("OK").test_id("dialog-ok")])
                        .test_id("dialog")
                        .into_element(
                            cx,
                            |cx| {
                                let trigger = Button::new("Open dialog")
                                    .test_id("dialog-trigger")
                                    .into_element(cx);
                                with_padding(cx, Px(24.0), trigger)
                            },
                            |_cx| Vec::new(),
                        );
                    vec![dialog]
                })
            };

        run_overlay_frame(
            &mut ui,
            &mut app,
            &mut services,
            window,
            bounds,
            true,
            |ui, app, services| render(ui, app, services),
        );

        let trigger_node: NodeId = ui
            .semantics_snapshot()
            .and_then(|snapshot| {
                assert_eq!(snapshot.barrier_root, None);
                snapshot.nodes.iter().find_map(|node| {
                    if node.test_id.as_deref() == Some("dialog-trigger") {
                        Some(node.id)
                    } else {
                        None
                    }
                })
            })
            .unwrap_or_else(|| panic!("expected dialog-trigger in semantics snapshot ({label})"));
        ui.set_focus(Some(trigger_node));
        assert_eq!(ui.focus(), Some(trigger_node));

        let _ = app.models_mut().update(&open, |v| *v = true);
        run_overlay_frame(
            &mut ui,
            &mut app,
            &mut services,
            window,
            bounds,
            true,
            |ui, app, services| render(ui, app, services),
        );

        let snapshot = ui
            .semantics_snapshot()
            .expect("expected semantics snapshot");
        assert!(
            snapshot.barrier_root.is_some(),
            "expected modal barrier root while dialog is open ({label})"
        );
        assert!(
            snapshot
                .nodes
                .iter()
                .any(|node| node.test_id.as_deref() == Some("dialog-scrim")),
            "expected dialog scrim node while dialog is open ({label})"
        );
        assert_ne!(
            ui.focus(),
            Some(trigger_node),
            "expected focus to move into dialog layer while open ({label})"
        );

        ui.set_focus(Some(trigger_node));
        run_overlay_frame(
            &mut ui,
            &mut app,
            &mut services,
            window,
            bounds,
            false,
            |ui, app, services| render(ui, app, services),
        );
        assert_ne!(
            ui.focus(),
            Some(trigger_node),
            "expected modal barrier to enforce focus containment ({label})"
        );

        let _ = app.models_mut().update(&open, |v| *v = false);
        run_overlay_frame(
            &mut ui,
            &mut app,
            &mut services,
            window,
            bounds,
            false,
            |ui, app, services| render(ui, app, services),
        );
        assert_eq!(
            ui.focus(),
            Some(trigger_node),
            "expected focus to restore to trigger on close ({label})"
        );

        let mut saw_barrier_cleared = false;
        for _ in 0..40 {
            run_overlay_frame(
                &mut ui,
                &mut app,
                &mut services,
                window,
                bounds,
                true,
                |ui, app, services| render(ui, app, services),
            );

            let snapshot = ui
                .semantics_snapshot()
                .expect("expected semantics snapshot");
            if snapshot.barrier_root.is_none() {
                saw_barrier_cleared = true;
                break;
            }
        }
        assert!(
            saw_barrier_cleared,
            "expected dialog barrier to unmount after close transition ({label})"
        );
    }
}

#[test]
fn modal_navigation_drawer_focus_is_contained_and_restored_across_schemes() {
    use fret_ui_material3::{Button, ModalNavigationDrawer};

    let cases = [
        (SchemeMode::Dark, DynamicVariant::TonalSpot, "dark/tonal"),
        (SchemeMode::Light, DynamicVariant::TonalSpot, "light/tonal"),
        (
            SchemeMode::Dark,
            DynamicVariant::Expressive,
            "dark/expressive",
        ),
        (
            SchemeMode::Light,
            DynamicVariant::Expressive,
            "light/expressive",
        ),
    ];

    for (mode, variant, label) in cases {
        let mut app = TestHost::default();
        app.set_global(PlatformCapabilities::default());
        apply_material_theme(&mut app, mode, variant);

        let window = AppWindowId::default();
        let mut services = FakeUiServices::default();
        let mut ui: UiTree<TestHost> = UiTree::new();
        ui.set_window(window);

        let bounds = Rect::new(
            Point::new(Px(0.0), Px(0.0)),
            Size::new(Px(480.0), Px(360.0)),
        );

        let open = app.models_mut().insert(false);

        let render =
            |ui: &mut UiTree<TestHost>, app: &mut TestHost, services: &mut dyn UiServices| {
                fret_ui::declarative::render_root(ui, app, services, window, bounds, "root", |cx| {
                    let drawer = ModalNavigationDrawer::new(open.clone())
                        .test_id("drawer")
                        .into_element(
                            cx,
                            |cx| {
                                Button::new("Drawer item")
                                    .test_id("drawer-item")
                                    .into_element(cx)
                            },
                            |cx| {
                                let trigger = Button::new("Open drawer")
                                    .test_id("drawer-trigger")
                                    .into_element(cx);
                                with_padding(cx, Px(24.0), trigger)
                            },
                        );
                    vec![drawer]
                })
            };

        run_overlay_frame(
            &mut ui,
            &mut app,
            &mut services,
            window,
            bounds,
            true,
            |ui, app, services| render(ui, app, services),
        );

        let trigger_node: NodeId = ui
            .semantics_snapshot()
            .and_then(|snapshot| {
                assert_eq!(snapshot.barrier_root, None);
                snapshot.nodes.iter().find_map(|node| {
                    if node.test_id.as_deref() == Some("drawer-trigger") {
                        Some(node.id)
                    } else {
                        None
                    }
                })
            })
            .unwrap_or_else(|| panic!("expected drawer-trigger in semantics snapshot ({label})"));
        ui.set_focus(Some(trigger_node));
        assert_eq!(ui.focus(), Some(trigger_node));

        let _ = app.models_mut().update(&open, |v| *v = true);
        run_overlay_frame(
            &mut ui,
            &mut app,
            &mut services,
            window,
            bounds,
            true,
            |ui, app, services| render(ui, app, services),
        );

        let snapshot = ui
            .semantics_snapshot()
            .expect("expected semantics snapshot");
        assert!(
            snapshot.barrier_root.is_some(),
            "expected modal barrier root while drawer is open ({label})"
        );
        assert!(
            snapshot
                .nodes
                .iter()
                .any(|node| node.test_id.as_deref() == Some("drawer-scrim")),
            "expected drawer scrim node while drawer is open ({label})"
        );
        assert_ne!(
            ui.focus(),
            Some(trigger_node),
            "expected focus to move into drawer layer while open ({label})"
        );

        ui.set_focus(Some(trigger_node));
        run_overlay_frame(
            &mut ui,
            &mut app,
            &mut services,
            window,
            bounds,
            false,
            |ui, app, services| render(ui, app, services),
        );
        assert_ne!(
            ui.focus(),
            Some(trigger_node),
            "expected modal barrier to enforce focus containment ({label})"
        );

        let _ = app.models_mut().update(&open, |v| *v = false);
        run_overlay_frame(
            &mut ui,
            &mut app,
            &mut services,
            window,
            bounds,
            false,
            |ui, app, services| render(ui, app, services),
        );
        assert_eq!(
            ui.focus(),
            Some(trigger_node),
            "expected focus to restore to trigger on close ({label})"
        );

        let mut saw_barrier_cleared = false;
        for _ in 0..60 {
            run_overlay_frame(
                &mut ui,
                &mut app,
                &mut services,
                window,
                bounds,
                true,
                |ui, app, services| render(ui, app, services),
            );

            let snapshot = ui
                .semantics_snapshot()
                .expect("expected semantics snapshot");
            if snapshot.barrier_root.is_none() {
                saw_barrier_cleared = true;
                break;
            }
        }
        assert!(
            saw_barrier_cleared,
            "expected drawer barrier to unmount after close transition ({label})"
        );
    }
}

#[test]
fn tooltip_opens_and_closes_on_hover_across_schemes() {
    use fret_ui_kit::{OverlayController, OverlayStackEntryKind};
    use fret_ui_material3::{Button, PlainTooltip, TooltipProvider};

    let cases = [
        (SchemeMode::Dark, DynamicVariant::TonalSpot, "dark/tonal"),
        (SchemeMode::Light, DynamicVariant::TonalSpot, "light/tonal"),
        (
            SchemeMode::Dark,
            DynamicVariant::Expressive,
            "dark/expressive",
        ),
        (
            SchemeMode::Light,
            DynamicVariant::Expressive,
            "light/expressive",
        ),
    ];

    for (mode, variant, label) in cases {
        let mut app = TestHost::default();
        app.set_global(PlatformCapabilities::default());
        apply_material_theme(&mut app, mode, variant);

        let window = AppWindowId::default();
        let mut services = FakeUiServices::default();
        let mut ui: UiTree<TestHost> = UiTree::new();
        ui.set_window(window);

        let bounds = Rect::new(
            Point::new(Px(0.0), Px(0.0)),
            Size::new(Px(420.0), Px(320.0)),
        );

        run_overlay_frame(
            &mut ui,
            &mut app,
            &mut services,
            window,
            bounds,
            true,
            |ui, app, services| {
                fret_ui::declarative::render_root(ui, app, services, window, bounds, "root", |cx| {
                    TooltipProvider::new()
                        .delay_duration_frames(0)
                        .skip_delay_duration_frames(0)
                        .with_elements(cx, |cx| {
                            let trigger = Button::new("Trigger")
                                .test_id("tooltip-trigger")
                                .into_element(cx);
                            let tooltip = PlainTooltip::new(trigger, "Tip")
                                .open_delay_frames(Some(0))
                                .close_delay_frames(Some(0))
                                .into_element(cx);
                            vec![with_padding(cx, Px(24.0), tooltip)]
                        })
                })
            },
        );

        let trigger_node: NodeId = ui
            .semantics_snapshot()
            .and_then(|snapshot| {
                snapshot.nodes.iter().find_map(|node| {
                    if node.test_id.as_deref() == Some("tooltip-trigger") {
                        Some(node.id)
                    } else {
                        None
                    }
                })
            })
            .unwrap_or_else(|| panic!("expected tooltip-trigger in semantics snapshot ({label})"));
        let trigger_bounds = ui
            .debug_node_visual_bounds(trigger_node)
            .expect("expected tooltip-trigger bounds");
        let hover_at = Point::new(
            Px(trigger_bounds.origin.x.0 + trigger_bounds.size.width.0 * 0.5),
            Px(trigger_bounds.origin.y.0 + trigger_bounds.size.height.0 * 0.5),
        );

        ui.dispatch_event(
            &mut app,
            &mut services,
            &pointer_move(PointerId(1), hover_at),
        );

        let mut opened = false;
        for _ in 0..6 {
            run_overlay_frame(
                &mut ui,
                &mut app,
                &mut services,
                window,
                bounds,
                false,
                |ui, app, services| {
                    fret_ui::declarative::render_root(
                        ui,
                        app,
                        services,
                        window,
                        bounds,
                        "root",
                        |cx| {
                            TooltipProvider::new()
                                .delay_duration_frames(0)
                                .skip_delay_duration_frames(0)
                                .with_elements(cx, |cx| {
                                    let trigger = Button::new("Trigger")
                                        .test_id("tooltip-trigger")
                                        .into_element(cx);
                                    let tooltip = PlainTooltip::new(trigger, "Tip")
                                        .open_delay_frames(Some(0))
                                        .close_delay_frames(Some(0))
                                        .into_element(cx);
                                    vec![with_padding(cx, Px(24.0), tooltip)]
                                })
                        },
                    )
                },
            );

            let stack = OverlayController::stack_snapshot_for_window(&ui, &mut app, window);
            if stack.stack.iter().any(|entry| {
                entry.kind == OverlayStackEntryKind::Tooltip && entry.open && entry.visible
            }) {
                opened = true;
                break;
            }
        }
        assert!(opened, "expected tooltip to open on hover ({label})");

        let unhover_at = Point::new(Px(1.0), Px(1.0));
        ui.dispatch_event(
            &mut app,
            &mut services,
            &pointer_move(PointerId(1), unhover_at),
        );

        let mut closed = false;
        for _ in 0..6 {
            run_overlay_frame(
                &mut ui,
                &mut app,
                &mut services,
                window,
                bounds,
                false,
                |ui, app, services| {
                    fret_ui::declarative::render_root(
                        ui,
                        app,
                        services,
                        window,
                        bounds,
                        "root",
                        |cx| {
                            TooltipProvider::new()
                                .delay_duration_frames(0)
                                .skip_delay_duration_frames(0)
                                .with_elements(cx, |cx| {
                                    let trigger = Button::new("Trigger")
                                        .test_id("tooltip-trigger")
                                        .into_element(cx);
                                    let tooltip = PlainTooltip::new(trigger, "Tip")
                                        .open_delay_frames(Some(0))
                                        .close_delay_frames(Some(0))
                                        .into_element(cx);
                                    vec![with_padding(cx, Px(24.0), tooltip)]
                                })
                        },
                    )
                },
            );

            let stack = OverlayController::stack_snapshot_for_window(&ui, &mut app, window);
            if !stack
                .stack
                .iter()
                .any(|entry| entry.kind == OverlayStackEntryKind::Tooltip && entry.visible)
            {
                closed = true;
                break;
            }
        }
        assert!(closed, "expected tooltip to close after unhover ({label})");
    }
}

#[test]
fn tooltip_is_click_through_and_does_not_block_underlay_activation_across_schemes() {
    use fret_ui_kit::{OverlayController, OverlayStackEntryKind};
    use fret_ui_material3::{Button, PlainTooltip, TooltipProvider};

    let cases = [
        (SchemeMode::Dark, DynamicVariant::TonalSpot, "dark/tonal"),
        (SchemeMode::Light, DynamicVariant::TonalSpot, "light/tonal"),
        (
            SchemeMode::Dark,
            DynamicVariant::Expressive,
            "dark/expressive",
        ),
        (
            SchemeMode::Light,
            DynamicVariant::Expressive,
            "light/expressive",
        ),
    ];

    for (mode, variant, label) in cases {
        let mut app = TestHost::default();
        app.set_global(PlatformCapabilities::default());
        apply_material_theme(&mut app, mode, variant);

        let window = AppWindowId::default();
        let mut services = FakeUiServices::default();
        let mut ui: UiTree<TestHost> = UiTree::new();
        ui.set_window(window);

        let bounds = Rect::new(
            Point::new(Px(0.0), Px(0.0)),
            Size::new(Px(560.0), Px(420.0)),
        );

        let underlay_toggled = app.models_mut().insert(false);
        let underlay_toggled_for_render = underlay_toggled.clone();
        let render = move |ui: &mut UiTree<TestHost>,
                           app: &mut TestHost,
                           services: &mut dyn UiServices| {
            fret_ui::declarative::render_root(ui, app, services, window, bounds, "root", |cx| {
                TooltipProvider::new()
                    .delay_duration_frames(0)
                    .skip_delay_duration_frames(0)
                    .with_elements(cx, |cx| {
                        let trigger = Button::new("Trigger")
                            .test_id("tooltip-trigger")
                            .into_element(cx);
                        let tooltip = PlainTooltip::new(trigger, "Tip")
                            .open_delay_frames(Some(0))
                            .close_delay_frames(Some(0))
                            .into_element(cx);

                        let underlay_toggled = underlay_toggled_for_render.clone();
                        let underlay = cx.pressable(
                            fret_ui::element::PressableProps {
                                layout: {
                                    let mut l = fret_ui::element::LayoutStyle::default();
                                    l.size.width = fret_ui::element::Length::Px(Px(160.0));
                                    l.size.height = fret_ui::element::Length::Px(Px(40.0));
                                    l
                                },
                                a11y: fret_ui::element::PressableA11y {
                                    test_id: Some(std::sync::Arc::<str>::from("tooltip-underlay")),
                                    ..Default::default()
                                },
                                ..Default::default()
                            },
                            move |cx, _st| {
                                cx.pressable_toggle_bool(&underlay_toggled);
                                Vec::new()
                            },
                        );

                        let mut props = fret_ui::element::FlexProps::default();
                        props.direction = fret_core::Axis::Vertical;
                        props.gap = Px(24.0);
                        vec![cx.flex(props, move |_cx| vec![tooltip, underlay])]
                    })
            })
        };

        run_overlay_frame(
            &mut ui,
            &mut app,
            &mut services,
            window,
            bounds,
            true,
            |ui, app, services| render(ui, app, services),
        );

        let trigger_node: NodeId = ui
            .semantics_snapshot()
            .and_then(|snapshot| {
                snapshot.nodes.iter().find_map(|node| {
                    if node.test_id.as_deref() == Some("tooltip-trigger") {
                        Some(node.id)
                    } else {
                        None
                    }
                })
            })
            .unwrap_or_else(|| panic!("expected tooltip-trigger in semantics snapshot ({label})"));
        let trigger_bounds = ui
            .debug_node_visual_bounds(trigger_node)
            .expect("expected tooltip-trigger bounds");
        let hover_at = Point::new(
            Px(trigger_bounds.origin.x.0 + trigger_bounds.size.width.0 * 0.5),
            Px(trigger_bounds.origin.y.0 + trigger_bounds.size.height.0 * 0.5),
        );

        let underlay_node: NodeId = ui
            .semantics_snapshot()
            .and_then(|snapshot| {
                snapshot.nodes.iter().find_map(|node| {
                    if node.test_id.as_deref() == Some("tooltip-underlay") {
                        Some(node.id)
                    } else {
                        None
                    }
                })
            })
            .unwrap_or_else(|| panic!("expected tooltip-underlay in semantics snapshot ({label})"));
        let underlay_bounds = ui
            .debug_node_visual_bounds(underlay_node)
            .expect("expected tooltip-underlay bounds");
        let click_at = Point::new(
            Px(underlay_bounds.origin.x.0 + underlay_bounds.size.width.0 * 0.5),
            Px(underlay_bounds.origin.y.0 + underlay_bounds.size.height.0 * 0.5),
        );

        ui.dispatch_event(
            &mut app,
            &mut services,
            &pointer_move(PointerId(1), hover_at),
        );

        let mut opened = false;
        for _ in 0..8 {
            run_overlay_frame(
                &mut ui,
                &mut app,
                &mut services,
                window,
                bounds,
                false,
                |ui, app, services| render(ui, app, services),
            );
            let stack = OverlayController::stack_snapshot_for_window(&ui, &mut app, window);
            if stack
                .stack
                .iter()
                .any(|entry| entry.kind == OverlayStackEntryKind::Tooltip && entry.visible)
            {
                opened = true;
                break;
            }
        }
        assert!(opened, "expected tooltip to open on hover ({label})");

        ui.dispatch_event(
            &mut app,
            &mut services,
            &pointer_down(PointerId(1), click_at),
        );
        ui.dispatch_event(&mut app, &mut services, &pointer_up(PointerId(1), click_at));

        run_overlay_frame(
            &mut ui,
            &mut app,
            &mut services,
            window,
            bounds,
            false,
            |ui, app, services| render(ui, app, services),
        );

        assert_eq!(
            app.models().get_copied(&underlay_toggled),
            Some(true),
            "expected tooltip to be click-through and allow underlay activation ({label})"
        );
        assert_eq!(
            ui.focus(),
            Some(underlay_node),
            "expected underlay to receive focus when clicking through tooltip ({label})"
        );

        let mut closed = false;
        for _ in 0..16 {
            run_overlay_frame(
                &mut ui,
                &mut app,
                &mut services,
                window,
                bounds,
                false,
                |ui, app, services| render(ui, app, services),
            );
            let stack = OverlayController::stack_snapshot_for_window(&ui, &mut app, window);
            if !stack
                .stack
                .iter()
                .any(|entry| entry.kind == OverlayStackEntryKind::Tooltip && entry.visible)
            {
                closed = true;
                break;
            }
        }
        assert!(
            closed,
            "expected tooltip to close after outside press without blocking underlay ({label})"
        );
    }
}

#[test]
fn material3_headless_controls_suite_goldens_v1() {
    use fret_ui::element::FlexProps;
    use fret_ui_material3::{Button, Checkbox, Switch};

    let schemes = [
        (
            SchemeMode::Dark,
            DynamicVariant::TonalSpot,
            "dark.tonal_spot",
        ),
        (
            SchemeMode::Light,
            DynamicVariant::TonalSpot,
            "light.tonal_spot",
        ),
        (
            SchemeMode::Dark,
            DynamicVariant::Expressive,
            "dark.expressive",
        ),
        (
            SchemeMode::Light,
            DynamicVariant::Expressive,
            "light.expressive",
        ),
    ];

    for scale_factor in [1.0, 1.25, 2.0] {
        let scale = scale_segment(scale_factor);

        for (mode, variant, label) in schemes {
            let mut app = TestHost::default();
            app.set_global(PlatformCapabilities::default());
            apply_material_theme(&mut app, mode, variant);

            let window = AppWindowId::default();
            let mut services = FakeUiServices::default();
            let mut ui: UiTree<TestHost> = UiTree::new();
            ui.set_window(window);

            let bounds = Rect::new(
                Point::new(Px(0.0), Px(0.0)),
                Size::new(Px(420.0), Px(320.0)),
            );

            let checkbox_checked = app.models_mut().insert(true);
            let checkbox_unchecked = app.models_mut().insert(false);
            let switch_on = app.models_mut().insert(true);
            let switch_off = app.models_mut().insert(false);

            let render = |ui: &mut UiTree<TestHost>,
                          app: &mut TestHost,
                          services: &mut dyn UiServices| {
                fret_ui::declarative::render_root(ui, app, services, window, bounds, "root", |cx| {
                    let mut props = FlexProps::default();
                    props.direction = fret_core::Axis::Vertical;
                    props.gap = Px(16.0);

                    let content = cx.flex(props, |cx| {
                        vec![
                            Button::new("Filled").test_id("btn-filled").into_element(cx),
                            Button::new("Filled (disabled)")
                                .disabled(true)
                                .test_id("btn-filled-disabled")
                                .into_element(cx),
                            Checkbox::new(checkbox_checked.clone())
                                .a11y_label("checkbox checked")
                                .test_id("cb-checked")
                                .into_element(cx),
                            Checkbox::new(checkbox_unchecked.clone())
                                .a11y_label("checkbox unchecked")
                                .test_id("cb-unchecked")
                                .into_element(cx),
                            Switch::new(switch_on.clone())
                                .a11y_label("switch on")
                                .test_id("sw-on")
                                .into_element(cx),
                            Switch::new(switch_off.clone())
                                .a11y_label("switch off")
                                .test_id("sw-off")
                                .into_element(cx),
                        ]
                    });

                    vec![with_padding(cx, Px(24.0), content)]
                })
            };

            let root = render(&mut ui, &mut app, &mut services);
            ui.set_root(root);
            ui.request_semantics_snapshot();
            ui.layout_all(&mut app, &mut services, bounds, scale_factor);

            ui.set_focus(None);
            ui.dispatch_event(
                &mut app,
                &mut services,
                &pointer_move(PointerId(1), Point::new(Px(1.0), Px(1.0))),
            );

            let btn_node: NodeId = ui
                .semantics_snapshot()
                .and_then(|snapshot| {
                    snapshot.nodes.iter().find_map(|node| {
                        (node.test_id.as_deref() == Some("btn-filled")).then_some(node.id)
                    })
                })
                .unwrap_or_else(|| {
                    panic!("expected btn-filled in semantics snapshot ({label}, {scale})")
                });
            let btn_bounds = ui
                .debug_node_visual_bounds(btn_node)
                .unwrap_or_else(|| panic!("expected btn-filled bounds ({label}, {scale})"));
            let btn_center = Point::new(
                Px(btn_bounds.origin.x.0 + btn_bounds.size.width.0 * 0.5),
                Px(btn_bounds.origin.y.0 + btn_bounds.size.height.0 * 0.5),
            );

            let mut cases: BTreeMap<String, Material3HeadlessGoldenV1> = BTreeMap::new();

            let idle_message = format!(
                "expected the Material3 controls idle scene to be stable after animations settle ({label}, {scale})"
            );
            cases.insert(
                "idle".to_string(),
                settle_material3_scene_snapshot_v1(
                    &mut app,
                    &mut ui,
                    &mut services,
                    bounds,
                    scale_factor,
                    24,
                    40,
                    &idle_message,
                    &render,
                ),
            );

            ui.dispatch_event(
                &mut app,
                &mut services,
                &pointer_move(PointerId(1), btn_center),
            );

            let hover_message = format!(
                "expected the Material3 controls hover scene to be stable after animations settle ({label}, {scale})"
            );
            cases.insert(
                "hover_btn_filled".to_string(),
                settle_material3_scene_snapshot_v1(
                    &mut app,
                    &mut ui,
                    &mut services,
                    bounds,
                    scale_factor,
                    24,
                    40,
                    &hover_message,
                    &render,
                ),
            );

            ui.dispatch_event(
                &mut app,
                &mut services,
                &pointer_move(PointerId(1), Point::new(Px(1.0), Px(1.0))),
            );
            ui.set_focus(Some(btn_node));
            ui.dispatch_event(&mut app, &mut services, &key_down(KeyCode::ArrowRight));
            ui.dispatch_event(&mut app, &mut services, &key_up(KeyCode::ArrowRight));

            let focus_visible_message = format!(
                "expected the Material3 controls focus-visible scene to be stable after animations settle ({label}, {scale})"
            );
            cases.insert(
                "focus_visible_btn_filled".to_string(),
                settle_material3_scene_snapshot_v1(
                    &mut app,
                    &mut ui,
                    &mut services,
                    bounds,
                    scale_factor,
                    24,
                    40,
                    &focus_visible_message,
                    &render,
                ),
            );

            let suite = Material3HeadlessSuiteV1 { cases };
            write_or_assert_material3_suite_v1(
                &format!("material3-controls.{scale}.{label}"),
                &suite,
            );
        }
    }
}

#[test]
fn material3_headless_overlays_suite_goldens_v1() {
    use fret_ui::element::{CrossAlign, FlexProps, Length, MainAlign};
    use fret_ui_kit::{OverlayController, OverlayStackEntryKind};
    use fret_ui_material3::menu::{MenuEntry, MenuItem};
    use fret_ui_material3::{
        Button, DropdownMenu, PlainTooltip, Select, SelectItem, TooltipProvider,
    };

    let schemes = [
        (
            SchemeMode::Dark,
            DynamicVariant::TonalSpot,
            "dark.tonal_spot",
        ),
        (
            SchemeMode::Light,
            DynamicVariant::TonalSpot,
            "light.tonal_spot",
        ),
        (
            SchemeMode::Dark,
            DynamicVariant::Expressive,
            "dark.expressive",
        ),
        (
            SchemeMode::Light,
            DynamicVariant::Expressive,
            "light.expressive",
        ),
    ];

    for scale_factor in [1.0, 1.25, 2.0] {
        let scale = scale_segment(scale_factor);

        for (mode, variant, label) in schemes {
            let mut app = TestHost::default();
            app.set_global(PlatformCapabilities::default());
            apply_material_theme(&mut app, mode, variant);

            let window = AppWindowId::default();
            let mut services = FakeUiServices::default();
            let mut ui: UiTree<TestHost> = UiTree::new();
            ui.set_window(window);

            let bounds = Rect::new(
                Point::new(Px(0.0), Px(0.0)),
                Size::new(Px(860.0), Px(520.0)),
            );

            let open = app.models_mut().insert(true);
            let open_model = open.clone();

            let render = move |ui: &mut UiTree<TestHost>,
                               app: &mut TestHost,
                               services: &mut dyn UiServices| {
                fret_ui::declarative::render_root(ui, app, services, window, bounds, "root", |cx| {
                    TooltipProvider::new()
                        .delay_duration_frames(0)
                        .skip_delay_duration_frames(0)
                        .with_elements(cx, |cx| {
                            let tooltip_trigger = Button::new("Tooltip")
                                .test_id("tooltip-trigger")
                                .into_element(cx);
                            let tooltip = PlainTooltip::new(tooltip_trigger, "Tip")
                                .open_delay_frames(Some(0))
                                .close_delay_frames(Some(0))
                                .into_element(cx);

                            let menu = DropdownMenu::new(open_model.clone())
                                .a11y_label("menu")
                                .test_id("dropdown")
                                .into_element(
                                    cx,
                                    |cx| {
                                        Button::new("Menu")
                                            .test_id("dropdown-trigger")
                                            .into_element(cx)
                                    },
                                    |_cx| {
                                        vec![
                                            MenuEntry::Item(
                                                MenuItem::new("A").test_id("dropdown-item-a"),
                                            ),
                                            MenuEntry::Item(
                                                MenuItem::new("B").test_id("dropdown-item-b"),
                                            ),
                                            MenuEntry::Item(
                                                MenuItem::new("C").test_id("dropdown-item-c"),
                                            ),
                                        ]
                                    },
                                );

                            let mut props = FlexProps::default();
                            props.layout.size.width = Length::Fill;
                            props.direction = fret_core::Axis::Horizontal;
                            props.gap = Px(48.0);
                            props.justify = MainAlign::SpaceBetween;
                            props.align = CrossAlign::Center;

                            let content = cx.flex(props, move |_cx| vec![tooltip, menu]);
                            vec![with_padding(cx, Px(24.0), content)]
                        })
                })
            };

            run_overlay_frame_scaled(
                &mut ui,
                &mut app,
                &mut services,
                window,
                bounds,
                scale_factor,
                true,
                |ui, app, services| render(ui, app, services),
            );

            let tooltip_trigger_node: NodeId = ui
                .semantics_snapshot()
                .and_then(|snapshot| {
                    snapshot.nodes.iter().find_map(|node| {
                        (node.test_id.as_deref() == Some("tooltip-trigger")).then_some(node.id)
                    })
                })
                .unwrap_or_else(|| {
                    panic!("expected tooltip-trigger in semantics snapshot ({label}, {scale})")
                });
            let tooltip_trigger_bounds = ui
                .debug_node_visual_bounds(tooltip_trigger_node)
                .expect("expected tooltip-trigger bounds");
            let hover_at = Point::new(
                Px(tooltip_trigger_bounds.origin.x.0 + tooltip_trigger_bounds.size.width.0 * 0.5),
                Px(tooltip_trigger_bounds.origin.y.0 + tooltip_trigger_bounds.size.height.0 * 0.5),
            );

            ui.dispatch_event(
                &mut app,
                &mut services,
                &pointer_move(PointerId(1), hover_at),
            );

            let mut opened = false;
            for _ in 0..12 {
                run_overlay_frame_scaled(
                    &mut ui,
                    &mut app,
                    &mut services,
                    window,
                    bounds,
                    scale_factor,
                    false,
                    |ui, app, services| render(ui, app, services),
                );

                let stack = OverlayController::stack_snapshot_for_window(&ui, &mut app, window);
                let tooltip_open = stack.stack.iter().any(|entry| {
                    entry.kind == OverlayStackEntryKind::Tooltip && entry.open && entry.visible
                });
                let menu_open = stack.stack.iter().any(|entry| {
                    entry.kind == OverlayStackEntryKind::Popover && entry.open && entry.visible
                });
                if tooltip_open && menu_open {
                    opened = true;
                    break;
                }
            }
            assert!(
                opened,
                "expected both tooltip and menu overlays to be open ({label}, {scale})"
            );

            let mut settled: Option<Material3HeadlessGoldenV1> = None;
            for frame in 0..80 {
                let scene = run_overlay_frame_with_scene_scaled(
                    &mut ui,
                    &mut app,
                    &mut services,
                    window,
                    bounds,
                    scale_factor,
                    false,
                    |ui, app, services| render(ui, app, services),
                );

                if frame < 44 {
                    continue;
                }

                let snapshot = material3_scene_snapshot_v1(&scene);
                if let Some(prev) = settled.as_ref() {
                    assert_eq!(
                        snapshot, *prev,
                        "expected the Material3 overlays scene to be stable after animations settle ({label}, {scale})"
                    );
                } else {
                    settled = Some(snapshot);
                }
            }

            let Some(both_open_snapshot) = settled else {
                panic!("expected a settled overlays snapshot ({label}, {scale})");
            };

            let select_open_snapshot = {
                let mut app = TestHost::default();
                app.set_global(PlatformCapabilities::default());
                apply_material_theme(&mut app, mode, variant);

                let window = AppWindowId::default();
                let mut services = FakeUiServices::default();
                let mut ui: UiTree<TestHost> = UiTree::new();
                ui.set_window(window);

                let selected: Model<Option<Arc<str>>> =
                    app.models_mut().insert(Some(Arc::<str>::from("beta")));

                let items: Arc<[SelectItem]> = vec![
                    SelectItem::new("alpha", "Alpha").test_id("select-item-alpha"),
                    SelectItem::new("beta", "Beta").test_id("select-item-beta"),
                    SelectItem::new("charlie", "Charlie (disabled)")
                        .disabled(true)
                        .test_id("select-item-charlie-disabled"),
                ]
                .into();

                let render = move |ui: &mut UiTree<TestHost>,
                                   app: &mut TestHost,
                                   services: &mut dyn UiServices| {
                    let selected = selected.clone();
                    let items = items.clone();
                    fret_ui::declarative::render_root(
                        ui,
                        app,
                        services,
                        window,
                        bounds,
                        "root",
                        |cx| {
                            let mut props = FlexProps::default();
                            props.direction = fret_core::Axis::Vertical;
                            props.gap = Px(16.0);
                            props.align = CrossAlign::Start;

                            let select = Select::new(selected)
                                .a11y_label("select")
                                .placeholder("Pick one")
                                .items(items)
                                .test_id("material3-select-trigger")
                                .into_element(cx);

                            vec![cx.flex(props, move |_cx| vec![select])]
                        },
                    )
                };

                run_overlay_frame_scaled(
                    &mut ui,
                    &mut app,
                    &mut services,
                    window,
                    bounds,
                    scale_factor,
                    true,
                    |ui, app, services| render(ui, app, services),
                );

                let Some(snapshot) = ui.semantics_snapshot() else {
                    panic!(
                        "expected semantics snapshot for select overlay case ({label}, {scale})"
                    );
                };

                let select_trigger_node = snapshot
                    .nodes
                    .iter()
                    .find_map(|node| {
                        (node.test_id.as_deref() == Some("material3-select-trigger"))
                            .then_some(node.id)
                    })
                    .unwrap_or_else(|| {
                        panic!(
                            "expected material3-select-trigger in semantics snapshot ({label}, {scale})"
                        )
                    });

                let select_trigger_bounds = ui
                    .debug_node_visual_bounds(select_trigger_node)
                    .expect("expected select trigger bounds");
                let click_at = Point::new(
                    Px(select_trigger_bounds.origin.x.0 + select_trigger_bounds.size.width.0 * 0.5),
                    Px(select_trigger_bounds.origin.y.0
                        + select_trigger_bounds.size.height.0 * 0.5),
                );

                ui.dispatch_event(
                    &mut app,
                    &mut services,
                    &pointer_down(PointerId(1), click_at),
                );
                ui.dispatch_event(&mut app, &mut services, &pointer_up(PointerId(1), click_at));

                let mut opened = false;
                for _ in 0..12 {
                    run_overlay_frame_scaled(
                        &mut ui,
                        &mut app,
                        &mut services,
                        window,
                        bounds,
                        scale_factor,
                        false,
                        |ui, app, services| render(ui, app, services),
                    );

                    let stack = OverlayController::stack_snapshot_for_window(&ui, &mut app, window);
                    let select_open = stack.stack.iter().any(|entry| {
                        entry.kind == OverlayStackEntryKind::Popover && entry.open && entry.visible
                    });
                    if select_open {
                        opened = true;
                        break;
                    }
                }
                assert!(
                    opened,
                    "expected the select overlay to be open after clicking the trigger ({label}, {scale})"
                );

                let mut settled: Option<Material3HeadlessGoldenV1> = None;
                for frame in 0..80 {
                    let scene = run_overlay_frame_with_scene_scaled(
                        &mut ui,
                        &mut app,
                        &mut services,
                        window,
                        bounds,
                        scale_factor,
                        false,
                        |ui, app, services| render(ui, app, services),
                    );

                    if frame < 44 {
                        continue;
                    }

                    let snapshot = material3_scene_snapshot_v1(&scene);
                    if let Some(prev) = settled.as_ref() {
                        assert_eq!(
                            snapshot, *prev,
                            "expected the Material3 select overlay scene to be stable after animations settle ({label}, {scale})"
                        );
                    } else {
                        settled = Some(snapshot);
                    }
                }

                settled.unwrap_or_else(|| {
                    panic!("expected a settled select overlay snapshot ({label}, {scale})")
                })
            };

            let mut cases: BTreeMap<String, Material3HeadlessGoldenV1> = BTreeMap::new();
            cases.insert("both_open".to_string(), both_open_snapshot);
            cases.insert("select_open".to_string(), select_open_snapshot);
            let suite = Material3HeadlessSuiteV1 { cases };

            write_or_assert_material3_suite_v1(
                &format!("material3-overlays.{scale}.{label}"),
                &suite,
            );
        }
    }
}

#[test]
fn material3_headless_text_field_suite_goldens_v1() {
    use fret_ui::element::FlexProps;
    use fret_ui_material3::{TextField, TextFieldVariant};

    let schemes = [
        (
            SchemeMode::Dark,
            DynamicVariant::TonalSpot,
            "dark.tonal_spot",
        ),
        (
            SchemeMode::Light,
            DynamicVariant::TonalSpot,
            "light.tonal_spot",
        ),
        (
            SchemeMode::Dark,
            DynamicVariant::Expressive,
            "dark.expressive",
        ),
        (
            SchemeMode::Light,
            DynamicVariant::Expressive,
            "light.expressive",
        ),
    ];

    for scale_factor in [1.0, 1.25, 2.0] {
        let scale = scale_segment(scale_factor);

        for (mode, variant, label) in schemes {
            let mut cases: BTreeMap<String, Material3HeadlessGoldenV1> = BTreeMap::new();

            for (case_name, hover_id, focus_id) in [
                ("idle", None, None),
                ("hover_outlined", Some("tf-outlined"), None),
                ("focus_visible_outlined", None, Some("tf-outlined")),
                ("hover_filled", Some("tf-filled"), None),
                ("focus_visible_filled", None, Some("tf-filled")),
            ] {
                let mut app = TestHost::default();
                app.set_global(PlatformCapabilities::default());
                apply_material_theme(&mut app, mode, variant);

                let window = AppWindowId::default();
                let mut services = FakeUiServices::default();
                let mut ui: UiTree<TestHost> = UiTree::new();
                ui.set_window(window);

                let bounds = Rect::new(
                    Point::new(Px(0.0), Px(0.0)),
                    Size::new(Px(640.0), Px(520.0)),
                );

                let outlined_empty = app.models_mut().insert(String::new());
                let outlined_populated = app.models_mut().insert("Hello".to_string());
                let filled_empty = app.models_mut().insert(String::new());
                let filled_populated = app.models_mut().insert("Hello".to_string());

                let render = |ui: &mut UiTree<TestHost>,
                              app: &mut TestHost,
                              services: &mut dyn UiServices| {
                    fret_ui::declarative::render_root(
                        ui,
                        app,
                        services,
                        window,
                        bounds,
                        "root",
                        |cx| {
                            let mut props = FlexProps::default();
                            props.direction = fret_core::Axis::Vertical;
                            props.gap = Px(12.0);

                            let content = cx.flex(props, |cx| {
                                vec![
                                    TextField::new(outlined_empty.clone())
                                        .variant(TextFieldVariant::Outlined)
                                        .label("Outlined")
                                        .placeholder("Placeholder")
                                        .supporting_text("Supporting")
                                        .test_id("tf-outlined")
                                        .into_element(cx),
                                    TextField::new(outlined_empty.clone())
                                        .variant(TextFieldVariant::Outlined)
                                        .label("Outlined (error)")
                                        .placeholder("Placeholder")
                                        .supporting_text("Supporting")
                                        .error(true)
                                        .test_id("tf-outlined-error")
                                        .into_element(cx),
                                    TextField::new(outlined_empty.clone())
                                        .variant(TextFieldVariant::Outlined)
                                        .label("Outlined (disabled)")
                                        .placeholder("Placeholder")
                                        .supporting_text("Supporting")
                                        .disabled(true)
                                        .test_id("tf-outlined-disabled")
                                        .into_element(cx),
                                    TextField::new(outlined_populated.clone())
                                        .variant(TextFieldVariant::Outlined)
                                        .label("Outlined (populated)")
                                        .placeholder("Placeholder")
                                        .supporting_text("Supporting")
                                        .test_id("tf-outlined-populated")
                                        .into_element(cx),
                                    TextField::new(filled_empty.clone())
                                        .variant(TextFieldVariant::Filled)
                                        .label("Filled")
                                        .placeholder("Placeholder")
                                        .supporting_text("Supporting")
                                        .test_id("tf-filled")
                                        .into_element(cx),
                                    TextField::new(filled_empty.clone())
                                        .variant(TextFieldVariant::Filled)
                                        .label("Filled (error)")
                                        .placeholder("Placeholder")
                                        .supporting_text("Supporting")
                                        .error(true)
                                        .test_id("tf-filled-error")
                                        .into_element(cx),
                                    TextField::new(filled_empty.clone())
                                        .variant(TextFieldVariant::Filled)
                                        .label("Filled (disabled)")
                                        .placeholder("Placeholder")
                                        .supporting_text("Supporting")
                                        .disabled(true)
                                        .test_id("tf-filled-disabled")
                                        .into_element(cx),
                                    TextField::new(filled_populated.clone())
                                        .variant(TextFieldVariant::Filled)
                                        .label("Filled (populated)")
                                        .placeholder("Placeholder")
                                        .supporting_text("Supporting")
                                        .test_id("tf-filled-populated")
                                        .into_element(cx),
                                ]
                            });

                            vec![with_padding(cx, Px(24.0), content)]
                        },
                    )
                };

                let root = render(&mut ui, &mut app, &mut services);
                ui.set_root(root);
                ui.request_semantics_snapshot();
                ui.layout_all(&mut app, &mut services, bounds, scale_factor);

                if case_name == "idle" {
                    ui.dispatch_event(
                        &mut app,
                        &mut services,
                        &pointer_move(PointerId(1), Point::new(Px(1.0), Px(1.0))),
                    );
                }

                if let Some(test_id) = hover_id {
                    let node_id: NodeId = ui
                        .semantics_snapshot()
                        .and_then(|snapshot| {
                            snapshot.nodes.iter().find_map(|node| {
                                (node.test_id.as_deref() == Some(test_id)).then_some(node.id)
                            })
                        })
                        .unwrap_or_else(|| {
                            panic!(
                                "expected {test_id} in semantics snapshot ({label}, {scale}, {case_name})"
                            )
                        });
                    let node_bounds = ui.debug_node_visual_bounds(node_id).unwrap_or_else(|| {
                        panic!("expected {test_id} bounds ({label}, {scale}, {case_name})")
                    });
                    let hover_at = Point::new(
                        Px(node_bounds.origin.x.0 + node_bounds.size.width.0 * 0.5),
                        Px(node_bounds.origin.y.0 + node_bounds.size.height.0 * 0.5),
                    );
                    ui.dispatch_event(
                        &mut app,
                        &mut services,
                        &pointer_move(PointerId(1), hover_at),
                    );
                }

                if let Some(test_id) = focus_id {
                    let node_id: NodeId = ui
                        .semantics_snapshot()
                        .and_then(|snapshot| {
                            snapshot.nodes.iter().find_map(|node| {
                                (node.test_id.as_deref() == Some(test_id)).then_some(node.id)
                            })
                        })
                        .unwrap_or_else(|| {
                            panic!(
                                "expected {test_id} in semantics snapshot ({label}, {scale}, {case_name})"
                            )
                        });
                    ui.set_focus(Some(node_id));
                    ui.dispatch_event(&mut app, &mut services, &key_down(KeyCode::ArrowRight));
                    ui.dispatch_event(&mut app, &mut services, &key_up(KeyCode::ArrowRight));
                }

                let mut settled: Option<Material3HeadlessGoldenV1> = None;
                for frame in 0..64 {
                    app.advance_frame();
                    let root = render(&mut ui, &mut app, &mut services);
                    ui.set_root(root);
                    ui.layout_all(&mut app, &mut services, bounds, scale_factor);

                    let mut scene = Scene::default();
                    ui.paint_all(&mut app, &mut services, bounds, &mut scene, scale_factor);

                    if frame < 28 {
                        continue;
                    }

                    let snapshot = material3_scene_snapshot_v1(&scene);
                    if let Some(prev) = settled.as_ref() {
                        assert_eq!(
                            snapshot, *prev,
                            "expected text field scene to be stable after animations settle ({label}, {scale}, {case_name})"
                        );
                    } else {
                        settled = Some(snapshot);
                    }
                }

                let Some(snapshot) = settled else {
                    panic!(
                        "expected a settled text field snapshot ({label}, {scale}, {case_name})"
                    );
                };
                cases.insert(case_name.to_string(), snapshot);
            }

            let suite = Material3HeadlessSuiteV1 { cases };
            write_or_assert_material3_suite_v1(
                &format!("material3-text-field.{scale}.{label}"),
                &suite,
            );
        }
    }
}

#[test]
fn dropdown_menu_dismisses_and_restores_focus_across_schemes() {
    use fret_ui_kit::{OverlayController, OverlayStackEntryKind};
    use fret_ui_material3::DropdownMenu;
    use fret_ui_material3::menu::{MenuEntry, MenuItem};

    let cases = [
        (SchemeMode::Dark, DynamicVariant::TonalSpot, "dark/tonal"),
        (SchemeMode::Light, DynamicVariant::TonalSpot, "light/tonal"),
        (
            SchemeMode::Dark,
            DynamicVariant::Expressive,
            "dark/expressive",
        ),
        (
            SchemeMode::Light,
            DynamicVariant::Expressive,
            "light/expressive",
        ),
    ];

    for (mode, variant, label) in cases {
        let mut app = TestHost::default();
        app.set_global(PlatformCapabilities::default());
        apply_material_theme(&mut app, mode, variant);

        let window = AppWindowId::default();
        let mut services = FakeUiServices::default();
        let mut ui: UiTree<TestHost> = UiTree::new();
        ui.set_window(window);

        let bounds = Rect::new(
            Point::new(Px(0.0), Px(0.0)),
            Size::new(Px(560.0), Px(420.0)),
        );

        let open = app.models_mut().insert(false);
        let underlay_toggled = app.models_mut().insert(false);

        let open_model = open.clone();
        let underlay_model = underlay_toggled.clone();
        let render =
            move |ui: &mut UiTree<TestHost>, app: &mut TestHost, services: &mut dyn UiServices| {
                fret_ui::declarative::render_root(ui, app, services, window, bounds, "root", |cx| {
                    let menu = DropdownMenu::new(open_model.clone())
                        .a11y_label("menu")
                        .test_id("dropdown")
                        .into_element(
                            cx,
                            |cx| {
                                cx.pressable_with_id(
                                    fret_ui::element::PressableProps {
                                        layout: {
                                            let mut l = fret_ui::element::LayoutStyle::default();
                                            l.size.width = fret_ui::element::Length::Px(Px(120.0));
                                            l.size.height = fret_ui::element::Length::Px(Px(40.0));
                                            l
                                        },
                                        a11y: fret_ui::element::PressableA11y {
                                            test_id: Some(std::sync::Arc::<str>::from(
                                                "dropdown-trigger",
                                            )),
                                            ..Default::default()
                                        },
                                        ..Default::default()
                                    },
                                    |_cx, _st, _id| Vec::new(),
                                )
                            },
                            |_cx| {
                                vec![
                                    MenuEntry::Item(MenuItem::new("A").test_id("dropdown-item-a")),
                                    MenuEntry::Item(MenuItem::new("B").test_id("dropdown-item-b")),
                                    MenuEntry::Item(MenuItem::new("C").test_id("dropdown-item-c")),
                                ]
                            },
                        );

                    let underlay_toggled = underlay_model.clone();
                    let underlay = cx.pressable(
                        fret_ui::element::PressableProps {
                            layout: {
                                let mut l = fret_ui::element::LayoutStyle::default();
                                l.size.width = fret_ui::element::Length::Px(Px(160.0));
                                l.size.height = fret_ui::element::Length::Px(Px(40.0));
                                l
                            },
                            a11y: fret_ui::element::PressableA11y {
                                test_id: Some(std::sync::Arc::<str>::from("underlay-toggle")),
                                ..Default::default()
                            },
                            ..Default::default()
                        },
                        move |cx, _st| {
                            cx.pressable_toggle_bool(&underlay_toggled);
                            Vec::new()
                        },
                    );

                    let mut props = fret_ui::element::FlexProps::default();
                    props.direction = fret_core::Axis::Vertical;
                    props.gap = Px(24.0);
                    vec![cx.flex(props, move |_cx| vec![menu, underlay])]
                })
            };

        run_overlay_frame(
            &mut ui,
            &mut app,
            &mut services,
            window,
            bounds,
            true,
            |ui, app, services| render(ui, app, services),
        );

        let trigger_node: NodeId = ui
            .semantics_snapshot()
            .and_then(|snapshot| {
                snapshot.nodes.iter().find_map(|node| {
                    if node.test_id.as_deref() == Some("dropdown-trigger") {
                        Some(node.id)
                    } else {
                        None
                    }
                })
            })
            .unwrap_or_else(|| panic!("expected dropdown-trigger in semantics snapshot ({label})"));
        ui.set_focus(Some(trigger_node));

        let _ = app.models_mut().update(&open, |v| *v = true);
        run_overlay_frame(
            &mut ui,
            &mut app,
            &mut services,
            window,
            bounds,
            true,
            |ui, app, services| render(ui, app, services),
        );

        let stack = OverlayController::stack_snapshot_for_window(&ui, &mut app, window);
        assert!(
            stack
                .stack
                .iter()
                .any(|e| e.kind == OverlayStackEntryKind::Popover && e.open),
            "expected dropdown menu overlay to be open ({label})"
        );

        ui.dispatch_event(&mut app, &mut services, &key_down(KeyCode::Escape));
        ui.dispatch_event(&mut app, &mut services, &key_up(KeyCode::Escape));

        run_overlay_frame(
            &mut ui,
            &mut app,
            &mut services,
            window,
            bounds,
            false,
            |ui, app, services| render(ui, app, services),
        );
        assert_eq!(
            app.models().get_copied(&open),
            Some(false),
            "expected dropdown menu to close on Escape ({label})"
        );
        assert_eq!(
            ui.focus(),
            Some(trigger_node),
            "expected dropdown menu to restore focus to trigger on Escape ({label})"
        );

        let _ = app.models_mut().update(&open, |v| *v = true);
        run_overlay_frame(
            &mut ui,
            &mut app,
            &mut services,
            window,
            bounds,
            true,
            |ui, app, services| render(ui, app, services),
        );

        let underlay_node: NodeId = ui
            .semantics_snapshot()
            .and_then(|snapshot| {
                snapshot.nodes.iter().find_map(|node| {
                    if node.test_id.as_deref() == Some("underlay-toggle") {
                        Some(node.id)
                    } else {
                        None
                    }
                })
            })
            .unwrap_or_else(|| panic!("expected underlay-toggle in semantics snapshot ({label})"));
        let underlay_bounds = ui
            .debug_node_visual_bounds(underlay_node)
            .expect("expected underlay-toggle bounds");
        let click_at = Point::new(
            Px(underlay_bounds.origin.x.0 + underlay_bounds.size.width.0 * 0.5),
            Px(underlay_bounds.origin.y.0 + underlay_bounds.size.height.0 * 0.5),
        );

        ui.dispatch_event(
            &mut app,
            &mut services,
            &pointer_down(PointerId(1), click_at),
        );
        ui.dispatch_event(&mut app, &mut services, &pointer_up(PointerId(1), click_at));

        run_overlay_frame(
            &mut ui,
            &mut app,
            &mut services,
            window,
            bounds,
            false,
            |ui, app, services| render(ui, app, services),
        );

        assert_eq!(
            app.models().get_copied(&open),
            Some(false),
            "expected dropdown menu to close on outside press ({label})"
        );
        assert_eq!(
            app.models().get_copied(&underlay_toggled),
            Some(false),
            "expected dropdown menu to prevent underlay activation on outside press ({label})"
        );

        let mut saw_unmount = false;
        for _ in 0..60 {
            run_overlay_frame(
                &mut ui,
                &mut app,
                &mut services,
                window,
                bounds,
                false,
                |ui, app, services| render(ui, app, services),
            );
            let stack = OverlayController::stack_snapshot_for_window(&ui, &mut app, window);
            if !stack
                .stack
                .iter()
                .any(|e| e.kind == OverlayStackEntryKind::Popover && e.visible)
            {
                saw_unmount = true;
                break;
            }
        }
        assert!(
            saw_unmount,
            "expected dropdown menu popover layer to unmount after close ({label})"
        );
        assert_eq!(
            ui.focus(),
            Some(trigger_node),
            "expected dropdown menu to restore focus to trigger on outside press ({label})"
        );
    }
}

#[test]
fn select_dismisses_and_restores_focus_across_schemes() {
    use fret_ui_kit::{OverlayController, OverlayStackEntryKind};
    use fret_ui_material3::{Select, SelectItem};

    let cases = [
        (SchemeMode::Dark, DynamicVariant::TonalSpot, "dark/tonal"),
        (SchemeMode::Light, DynamicVariant::TonalSpot, "light/tonal"),
        (
            SchemeMode::Dark,
            DynamicVariant::Expressive,
            "dark/expressive",
        ),
        (
            SchemeMode::Light,
            DynamicVariant::Expressive,
            "light/expressive",
        ),
    ];

    for (mode, variant, label) in cases {
        let mut app = TestHost::default();
        app.set_global(PlatformCapabilities::default());
        apply_material_theme(&mut app, mode, variant);

        let window = AppWindowId::default();
        let mut services = FakeUiServices::default();
        let mut ui: UiTree<TestHost> = UiTree::new();
        ui.set_window(window);

        let bounds = Rect::new(
            Point::new(Px(0.0), Px(0.0)),
            Size::new(Px(560.0), Px(420.0)),
        );

        let selected = app.models_mut().insert(Some(Arc::<str>::from("beta")));
        let items: Arc<[SelectItem]> = vec![
            SelectItem::new("alpha", "Alpha").test_id("select-item-alpha"),
            SelectItem::new("beta", "Beta").test_id("select-item-beta"),
            SelectItem::new("charlie", "Charlie (disabled)")
                .disabled(true)
                .test_id("select-item-charlie-disabled"),
        ]
        .into();

        let selected_model = selected.clone();
        let render =
            move |ui: &mut UiTree<TestHost>, app: &mut TestHost, services: &mut dyn UiServices| {
                let selected_model = selected_model.clone();
                let items = items.clone();
                fret_ui::declarative::render_root(ui, app, services, window, bounds, "root", |cx| {
                    vec![
                        Select::new(selected_model)
                            .a11y_label("select")
                            .placeholder("Pick one")
                            .items(items)
                            .test_id("select-trigger")
                            .into_element(cx),
                    ]
                })
            };

        run_overlay_frame(
            &mut ui,
            &mut app,
            &mut services,
            window,
            bounds,
            true,
            |ui, app, services| render(ui, app, services),
        );

        let trigger_node: NodeId = ui
            .semantics_snapshot()
            .and_then(|snapshot| {
                snapshot.nodes.iter().find_map(|node| {
                    (node.test_id.as_deref() == Some("select-trigger")).then_some(node.id)
                })
            })
            .unwrap_or_else(|| panic!("expected select-trigger in semantics snapshot ({label})"));

        let trigger_bounds = ui
            .debug_node_visual_bounds(trigger_node)
            .expect("expected select-trigger bounds");
        let click_at = Point::new(
            Px(trigger_bounds.origin.x.0 + trigger_bounds.size.width.0 * 0.5),
            Px(trigger_bounds.origin.y.0 + trigger_bounds.size.height.0 * 0.5),
        );

        ui.set_focus(Some(trigger_node));
        ui.dispatch_event(
            &mut app,
            &mut services,
            &pointer_down(PointerId(1), click_at),
        );
        ui.dispatch_event(&mut app, &mut services, &pointer_up(PointerId(1), click_at));

        let mut opened = false;
        for _ in 0..16 {
            run_overlay_frame(
                &mut ui,
                &mut app,
                &mut services,
                window,
                bounds,
                false,
                |ui, app, services| render(ui, app, services),
            );

            let stack = OverlayController::stack_snapshot_for_window(&ui, &mut app, window);
            if stack
                .stack
                .iter()
                .any(|e| e.kind == OverlayStackEntryKind::Popover && e.open)
            {
                opened = true;
                break;
            }
        }
        assert!(opened, "expected select overlay to open on click ({label})");

        ui.dispatch_event(&mut app, &mut services, &key_down(KeyCode::Escape));
        ui.dispatch_event(&mut app, &mut services, &key_up(KeyCode::Escape));

        let mut closed = false;
        for _ in 0..16 {
            run_overlay_frame(
                &mut ui,
                &mut app,
                &mut services,
                window,
                bounds,
                false,
                |ui, app, services| render(ui, app, services),
            );

            let stack = OverlayController::stack_snapshot_for_window(&ui, &mut app, window);
            if !stack
                .stack
                .iter()
                .any(|e| e.kind == OverlayStackEntryKind::Popover && e.visible)
            {
                closed = true;
                break;
            }
        }

        assert!(
            closed,
            "expected select overlay to close on Escape ({label})"
        );
        assert_eq!(
            ui.focus(),
            Some(trigger_node),
            "expected select to restore focus to trigger on Escape ({label})"
        );
    }
}

#[test]
fn select_keyboard_open_sets_initial_focus_and_outside_dismiss_restores_focus_across_schemes() {
    use fret_ui_kit::{OverlayController, OverlayStackEntryKind};
    use fret_ui_material3::{Select, SelectItem};

    let cases = [
        (SchemeMode::Dark, DynamicVariant::TonalSpot, "dark/tonal"),
        (SchemeMode::Light, DynamicVariant::TonalSpot, "light/tonal"),
        (
            SchemeMode::Dark,
            DynamicVariant::Expressive,
            "dark/expressive",
        ),
        (
            SchemeMode::Light,
            DynamicVariant::Expressive,
            "light/expressive",
        ),
    ];
    let keyboard_open_keys = [
        (KeyCode::ArrowDown, "arrow_down"),
        (KeyCode::ArrowUp, "arrow_up"),
    ];

    for (mode, variant, label) in cases {
        for (open_key, key_label) in keyboard_open_keys {
            let mut app = TestHost::default();
            app.set_global(PlatformCapabilities::default());
            apply_material_theme(&mut app, mode, variant);

            let window = AppWindowId::default();
            let mut services = FakeUiServices::default();
            let mut ui: UiTree<TestHost> = UiTree::new();
            ui.set_window(window);

            let bounds = Rect::new(
                Point::new(Px(0.0), Px(0.0)),
                Size::new(Px(560.0), Px(420.0)),
            );

            let selected = app.models_mut().insert(Some(Arc::<str>::from("beta")));
            let underlay_toggled = app.models_mut().insert(false);

            let items: Arc<[SelectItem]> = vec![
                SelectItem::new("alpha", "Alpha").test_id("select-item-alpha"),
                SelectItem::new("beta", "Beta").test_id("select-item-beta"),
                SelectItem::new("charlie", "Charlie (disabled)")
                    .disabled(true)
                    .test_id("select-item-charlie-disabled"),
            ]
            .into();

            let selected_model = selected.clone();
            let underlay_model = underlay_toggled.clone();
            let render = move |ui: &mut UiTree<TestHost>,
                               app: &mut TestHost,
                               services: &mut dyn UiServices| {
                let selected_model = selected_model.clone();
                let items = items.clone();
                let underlay_model = underlay_model.clone();
                fret_ui::declarative::render_root(ui, app, services, window, bounds, "root", |cx| {
                    let select = Select::new(selected_model)
                        .a11y_label("select")
                        .placeholder("Pick one")
                        .items(items)
                        .test_id("select-trigger")
                        .into_element(cx);

                    let underlay = cx.pressable(
                        fret_ui::element::PressableProps {
                            layout: {
                                let mut l = fret_ui::element::LayoutStyle::default();
                                l.size.width = fret_ui::element::Length::Px(Px(160.0));
                                l.size.height = fret_ui::element::Length::Px(Px(40.0));
                                l
                            },
                            a11y: fret_ui::element::PressableA11y {
                                test_id: Some(Arc::<str>::from("select-underlay-toggle")),
                                ..Default::default()
                            },
                            ..Default::default()
                        },
                        move |cx, _st| {
                            cx.pressable_toggle_bool(&underlay_model);
                            Vec::new()
                        },
                    );

                    let mut props = fret_ui::element::FlexProps::default();
                    props.direction = fret_core::Axis::Vertical;
                    props.gap = Px(24.0);
                    // Place the underlay above the trigger so the "outside press" point is
                    // guaranteed to be outside the select popover (which opens below the trigger).
                    vec![cx.flex(props, move |_cx| vec![underlay, select])]
                })
            };

            run_overlay_frame(
                &mut ui,
                &mut app,
                &mut services,
                window,
                bounds,
                true,
                |ui, app, services| render(ui, app, services),
            );

            let trigger_node: NodeId = ui
                .semantics_snapshot()
                .and_then(|snapshot| {
                    snapshot.nodes.iter().find_map(|node| {
                        (node.test_id.as_deref() == Some("select-trigger")).then_some(node.id)
                    })
                })
                .unwrap_or_else(|| {
                    panic!("expected select-trigger in semantics snapshot ({label}, {key_label})")
                });
            let underlay_node: NodeId = ui
                .semantics_snapshot()
                .and_then(|snapshot| {
                    snapshot.nodes.iter().find_map(|node| {
                        (node.test_id.as_deref() == Some("select-underlay-toggle"))
                            .then_some(node.id)
                    })
                })
                .unwrap_or_else(|| {
                    panic!(
                        "expected select-underlay-toggle in semantics snapshot ({label}, {key_label})"
                    )
                });

            ui.set_focus(Some(trigger_node));
            ui.dispatch_event(&mut app, &mut services, &key_down(open_key));
            ui.dispatch_event(&mut app, &mut services, &key_up(open_key));

            let mut opened = false;
            for _ in 0..24 {
                run_overlay_frame(
                    &mut ui,
                    &mut app,
                    &mut services,
                    window,
                    bounds,
                    true,
                    |ui, app, services| render(ui, app, services),
                );

                let stack = OverlayController::stack_snapshot_for_window(&ui, &mut app, window);
                if stack
                    .stack
                    .iter()
                    .any(|e| e.kind == OverlayStackEntryKind::Popover && e.open)
                {
                    opened = true;
                    break;
                }
            }
            assert!(
                opened,
                "expected select overlay to open on {key_label} ({label})"
            );

            let selected_option_node: NodeId = ui
                .semantics_snapshot()
                .and_then(|snapshot| {
                    snapshot.nodes.iter().find_map(|node| {
                        (node.test_id.as_deref() == Some("select-item-beta")).then_some(node.id)
                    })
                })
                .unwrap_or_else(|| {
                    panic!("expected select-item-beta in semantics snapshot ({label}, {key_label})")
                });
            let mut focused_selected = ui.focus() == Some(selected_option_node);
            for _ in 0..12 {
                if focused_selected {
                    break;
                }
                run_overlay_frame(
                    &mut ui,
                    &mut app,
                    &mut services,
                    window,
                    bounds,
                    true,
                    |ui, app, services| render(ui, app, services),
                );
                focused_selected = ui.focus() == Some(selected_option_node);
            }
            if !focused_selected {
                let focused_test_id = ui.semantics_snapshot().and_then(|snapshot| {
                    ui.focus().and_then(|focused| {
                        snapshot
                            .nodes
                            .iter()
                            .find(|node| node.id == focused)
                            .and_then(|node| node.test_id.as_deref())
                            .map(|s| s.to_string())
                    })
                });
                panic!(
                    "expected Select to move focus to the selected option when opening via keyboard ({label}, {key_label}); focus={:?}, focus_test_id={focused_test_id:?}",
                    ui.focus()
                );
            }

            let underlay_bounds = ui
                .debug_node_visual_bounds(underlay_node)
                .expect("expected underlay bounds");
            let click_at = Point::new(
                Px(underlay_bounds.origin.x.0 + underlay_bounds.size.width.0 * 0.5),
                Px(underlay_bounds.origin.y.0 + underlay_bounds.size.height.0 * 0.5),
            );
            ui.dispatch_event(
                &mut app,
                &mut services,
                &pointer_down(PointerId(1), click_at),
            );
            ui.dispatch_event(&mut app, &mut services, &pointer_up(PointerId(1), click_at));

            let mut closed = false;
            for _ in 0..24 {
                run_overlay_frame(
                    &mut ui,
                    &mut app,
                    &mut services,
                    window,
                    bounds,
                    false,
                    |ui, app, services| render(ui, app, services),
                );

                let stack = OverlayController::stack_snapshot_for_window(&ui, &mut app, window);
                if !stack
                    .stack
                    .iter()
                    .any(|e| e.kind == OverlayStackEntryKind::Popover && e.visible)
                {
                    closed = true;
                    break;
                }
            }

            assert!(
                closed,
                "expected select overlay to close on outside press after opening via {key_label} ({label})"
            );
            assert_eq!(
                app.models().get_copied(&underlay_toggled),
                Some(false),
                "expected select to prevent underlay activation on outside press ({label}, {key_label})"
            );
            assert_eq!(
                ui.focus(),
                Some(trigger_node),
                "expected select to restore focus to trigger on outside press ({label}, {key_label})"
            );

            ui.set_focus(Some(trigger_node));
            ui.dispatch_event(&mut app, &mut services, &key_down(open_key));
            ui.dispatch_event(&mut app, &mut services, &key_up(open_key));

            let mut reopened = false;
            for _ in 0..24 {
                run_overlay_frame(
                    &mut ui,
                    &mut app,
                    &mut services,
                    window,
                    bounds,
                    true,
                    |ui, app, services| render(ui, app, services),
                );

                let stack = OverlayController::stack_snapshot_for_window(&ui, &mut app, window);
                if stack
                    .stack
                    .iter()
                    .any(|e| e.kind == OverlayStackEntryKind::Popover && e.open)
                {
                    reopened = true;
                    break;
                }
            }
            assert!(
                reopened,
                "expected select overlay to re-open on {key_label} ({label})"
            );

            let selected_option_node: NodeId = ui
                .semantics_snapshot()
                .and_then(|snapshot| {
                    snapshot.nodes.iter().find_map(|node| {
                        (node.test_id.as_deref() == Some("select-item-beta")).then_some(node.id)
                    })
                })
                .unwrap_or_else(|| {
                    panic!("expected select-item-beta in semantics snapshot ({label}, {key_label})")
                });
            let mut focused_selected = ui.focus() == Some(selected_option_node);
            for _ in 0..12 {
                if focused_selected {
                    break;
                }
                run_overlay_frame(
                    &mut ui,
                    &mut app,
                    &mut services,
                    window,
                    bounds,
                    true,
                    |ui, app, services| render(ui, app, services),
                );
                focused_selected = ui.focus() == Some(selected_option_node);
            }
            if !focused_selected {
                let focused_test_id = ui.semantics_snapshot().and_then(|snapshot| {
                    ui.focus().and_then(|focused| {
                        snapshot
                            .nodes
                            .iter()
                            .find(|node| node.id == focused)
                            .and_then(|node| node.test_id.as_deref())
                            .map(|s| s.to_string())
                    })
                });
                panic!(
                    "expected Select to focus the selected option when reopening via keyboard ({label}, {key_label}); focus={:?}, focus_test_id={focused_test_id:?}",
                    ui.focus()
                );
            }

            ui.dispatch_event(&mut app, &mut services, &key_down(KeyCode::ArrowDown));
            ui.dispatch_event(&mut app, &mut services, &key_up(KeyCode::ArrowDown));
            run_overlay_frame(
                &mut ui,
                &mut app,
                &mut services,
                window,
                bounds,
                true,
                |ui, app, services| render(ui, app, services),
            );

            let alpha_option_node: NodeId = ui
                .semantics_snapshot()
                .and_then(|snapshot| {
                    snapshot.nodes.iter().find_map(|node| {
                        (node.test_id.as_deref() == Some("select-item-alpha")).then_some(node.id)
                    })
                })
                .unwrap_or_else(|| {
                    panic!(
                        "expected select-item-alpha in semantics snapshot ({label}, {key_label})"
                    )
                });
            assert_eq!(
                ui.focus(),
                Some(alpha_option_node),
                "expected ArrowDown to rove focus to the next enabled option (wrap + skip disabled) ({label}, {key_label})"
            );

            ui.dispatch_event(&mut app, &mut services, &key_down(KeyCode::Enter));
            ui.dispatch_event(&mut app, &mut services, &key_up(KeyCode::Enter));

            let mut closed_after_select = false;
            for _ in 0..24 {
                run_overlay_frame(
                    &mut ui,
                    &mut app,
                    &mut services,
                    window,
                    bounds,
                    false,
                    |ui, app, services| render(ui, app, services),
                );

                let stack = OverlayController::stack_snapshot_for_window(&ui, &mut app, window);
                if !stack
                    .stack
                    .iter()
                    .any(|e| e.kind == OverlayStackEntryKind::Popover && e.visible)
                {
                    closed_after_select = true;
                    break;
                }
            }

            assert!(
                closed_after_select,
                "expected select overlay to close after selecting an option ({label}, {key_label})"
            );
            assert_eq!(
                ui.focus(),
                Some(trigger_node),
                "expected select to restore focus to trigger after selecting an option ({label}, {key_label})"
            );
            assert_eq!(
                app.models().get_cloned(&selected),
                Some(Some(Arc::<str>::from("alpha"))),
                "expected Enter to select the focused option ({label}, {key_label})"
            );
        }
    }
}

#[test]
fn select_roving_scrolls_focused_option_into_view() {
    use fret_ui_kit::{OverlayController, OverlayStackEntryKind};
    use fret_ui_material3::{Select, SelectItem};

    let mut app = TestHost::default();
    app.set_global(PlatformCapabilities::default());
    apply_material_theme(&mut app, SchemeMode::Light, DynamicVariant::TonalSpot);

    let window = AppWindowId::default();
    let mut services = FakeUiServices::default();
    let mut ui: UiTree<TestHost> = UiTree::new();
    ui.set_window(window);

    let bounds = Rect::new(
        Point::new(Px(0.0), Px(0.0)),
        Size::new(Px(560.0), Px(420.0)),
    );

    let selected = app.models_mut().insert(Some(Arc::<str>::from("item-0")));
    let mut items_vec: Vec<SelectItem> = Vec::new();
    for i in 0..20 {
        let value: Arc<str> = Arc::from(format!("item-{i}"));
        let label: Arc<str> = Arc::from(format!("Item {i}"));
        items_vec.push(
            SelectItem::new(value.clone(), label).test_id(Arc::from(format!("select-item-{i}"))),
        );
    }
    let items: Arc<[SelectItem]> = items_vec.into();

    let render =
        move |ui: &mut UiTree<TestHost>, app: &mut TestHost, services: &mut dyn UiServices| {
            let selected = selected.clone();
            let items = items.clone();
            fret_ui::declarative::render_root(ui, app, services, window, bounds, "root", |cx| {
                vec![
                    Select::new(selected)
                        .a11y_label("select")
                        .placeholder("Pick one")
                        .items(items)
                        .test_id("select-trigger")
                        .into_element(cx),
                ]
            })
        };

    run_overlay_frame(
        &mut ui,
        &mut app,
        &mut services,
        window,
        bounds,
        true,
        |ui, app, services| render(ui, app, services),
    );

    let trigger_node: NodeId = ui
        .semantics_snapshot()
        .and_then(|snapshot| {
            snapshot.nodes.iter().find_map(|node| {
                (node.test_id.as_deref() == Some("select-trigger")).then_some(node.id)
            })
        })
        .expect("expected select-trigger in semantics snapshot");

    let trigger_bounds = ui
        .debug_node_visual_bounds(trigger_node)
        .expect("expected select-trigger bounds");
    let click_at = Point::new(
        Px(trigger_bounds.origin.x.0 + trigger_bounds.size.width.0 * 0.5),
        Px(trigger_bounds.origin.y.0 + trigger_bounds.size.height.0 * 0.5),
    );

    ui.set_focus(Some(trigger_node));
    ui.dispatch_event(
        &mut app,
        &mut services,
        &pointer_down(PointerId(1), click_at),
    );
    ui.dispatch_event(&mut app, &mut services, &pointer_up(PointerId(1), click_at));

    let mut opened = false;
    for _ in 0..24 {
        run_overlay_frame(
            &mut ui,
            &mut app,
            &mut services,
            window,
            bounds,
            true,
            |ui, app, services| render(ui, app, services),
        );
        let stack = OverlayController::stack_snapshot_for_window(&ui, &mut app, window);
        if stack
            .stack
            .iter()
            .any(|e| e.kind == OverlayStackEntryKind::Popover && e.open)
        {
            opened = true;
            break;
        }
    }
    assert!(opened, "expected select overlay to open");

    for _ in 0..12 {
        ui.dispatch_event(&mut app, &mut services, &key_down(KeyCode::ArrowDown));
        ui.dispatch_event(&mut app, &mut services, &key_up(KeyCode::ArrowDown));
        run_overlay_frame(
            &mut ui,
            &mut app,
            &mut services,
            window,
            bounds,
            false,
            |ui, app, services| render(ui, app, services),
        );
    }

    run_overlay_frame(
        &mut ui,
        &mut app,
        &mut services,
        window,
        bounds,
        true,
        |ui, app, services| render(ui, app, services),
    );

    let listbox_node: NodeId = ui
        .semantics_snapshot()
        .and_then(|snapshot| {
            snapshot.nodes.iter().find_map(|node| {
                (node.test_id.as_deref() == Some("select-trigger-listbox")).then_some(node.id)
            })
        })
        .expect("expected select-trigger-listbox in semantics snapshot");
    let listbox_bounds = ui
        .debug_node_visual_bounds(listbox_node)
        .expect("expected listbox bounds");

    let focused = ui.focus().expect("expected focused node after roving");
    let focused_bounds = ui
        .debug_node_visual_bounds(focused)
        .expect("expected focused bounds");

    let epsilon = 0.01;
    let listbox_top = listbox_bounds.origin.y.0;
    let listbox_bottom = listbox_bounds.origin.y.0 + listbox_bounds.size.height.0;
    let focused_top = focused_bounds.origin.y.0;
    let focused_bottom = focused_bounds.origin.y.0 + focused_bounds.size.height.0;
    assert!(
        focused_top + epsilon >= listbox_top && focused_bottom - epsilon <= listbox_bottom,
        "expected focused option to be visible within listbox viewport after roving"
    );
}

#[test]
fn select_open_scrolls_selected_option_into_view() {
    use fret_ui_kit::{OverlayController, OverlayStackEntryKind};
    use fret_ui_material3::{Select, SelectItem};

    let mut app = TestHost::default();
    app.set_global(PlatformCapabilities::default());
    apply_material_theme(&mut app, SchemeMode::Light, DynamicVariant::TonalSpot);

    let window = AppWindowId::default();
    let mut services = FakeUiServices::default();
    let mut ui: UiTree<TestHost> = UiTree::new();
    ui.set_window(window);

    let bounds = Rect::new(
        Point::new(Px(0.0), Px(0.0)),
        Size::new(Px(560.0), Px(420.0)),
    );

    let selected = app.models_mut().insert(Some(Arc::<str>::from("item-18")));
    let mut items_vec: Vec<SelectItem> = Vec::new();
    for i in 0..30 {
        let value: Arc<str> = Arc::from(format!("item-{i}"));
        let label: Arc<str> = Arc::from(format!("Item {i}"));
        items_vec.push(
            SelectItem::new(value.clone(), label).test_id(Arc::from(format!("select-item-{i}"))),
        );
    }
    let items: Arc<[SelectItem]> = items_vec.into();

    let render =
        move |ui: &mut UiTree<TestHost>, app: &mut TestHost, services: &mut dyn UiServices| {
            let selected = selected.clone();
            let items = items.clone();
            fret_ui::declarative::render_root(ui, app, services, window, bounds, "root", |cx| {
                vec![
                    Select::new(selected)
                        .a11y_label("select")
                        .placeholder("Pick one")
                        .items(items)
                        .test_id("select-trigger")
                        .into_element(cx),
                ]
            })
        };

    run_overlay_frame(
        &mut ui,
        &mut app,
        &mut services,
        window,
        bounds,
        true,
        |ui, app, services| render(ui, app, services),
    );

    let trigger_node: NodeId = ui
        .semantics_snapshot()
        .and_then(|snapshot| {
            snapshot.nodes.iter().find_map(|node| {
                (node.test_id.as_deref() == Some("select-trigger")).then_some(node.id)
            })
        })
        .expect("expected select-trigger in semantics snapshot");

    ui.set_focus(Some(trigger_node));
    ui.dispatch_event(&mut app, &mut services, &key_down(KeyCode::ArrowDown));
    ui.dispatch_event(&mut app, &mut services, &key_up(KeyCode::ArrowDown));

    let mut opened = false;
    for _ in 0..24 {
        run_overlay_frame(
            &mut ui,
            &mut app,
            &mut services,
            window,
            bounds,
            true,
            |ui, app, services| render(ui, app, services),
        );
        let stack = OverlayController::stack_snapshot_for_window(&ui, &mut app, window);
        if stack
            .stack
            .iter()
            .any(|e| e.kind == OverlayStackEntryKind::Popover && e.open)
        {
            opened = true;
            break;
        }
    }
    assert!(opened, "expected select overlay to open");

    run_overlay_frame(
        &mut ui,
        &mut app,
        &mut services,
        window,
        bounds,
        true,
        |ui, app, services| render(ui, app, services),
    );

    let listbox_node: NodeId = ui
        .semantics_snapshot()
        .and_then(|snapshot| {
            snapshot.nodes.iter().find_map(|node| {
                (node.test_id.as_deref() == Some("select-trigger-listbox")).then_some(node.id)
            })
        })
        .expect("expected select-trigger-listbox in semantics snapshot");
    let listbox_bounds = ui
        .debug_node_visual_bounds(listbox_node)
        .expect("expected listbox bounds");

    let selected_node: NodeId = ui
        .semantics_snapshot()
        .and_then(|snapshot| {
            snapshot.nodes.iter().find_map(|node| {
                (node.test_id.as_deref() == Some("select-item-18")).then_some(node.id)
            })
        })
        .expect("expected select-item-18 in semantics snapshot");
    let selected_bounds = ui
        .debug_node_visual_bounds(selected_node)
        .expect("expected selected option bounds");

    let epsilon = 0.01;
    let listbox_top = listbox_bounds.origin.y.0;
    let listbox_bottom = listbox_bounds.origin.y.0 + listbox_bounds.size.height.0;
    let selected_top = selected_bounds.origin.y.0;
    let selected_bottom = selected_bounds.origin.y.0 + selected_bounds.size.height.0;
    assert!(
        selected_top + epsilon >= listbox_top && selected_bottom - epsilon <= listbox_bottom,
        "expected the selected option to be visible within listbox viewport on open"
    );
}

fn scale_segment(scale_factor: f32) -> &'static str {
    if (scale_factor - 1.0).abs() < 1e-6 {
        "scale1_0"
    } else if (scale_factor - 1.25).abs() < 1e-6 {
        "scale1_25"
    } else if (scale_factor - 2.0).abs() < 1e-6 {
        "scale2_0"
    } else {
        panic!("unsupported scale factor: {scale_factor}");
    }
}

#[test]
fn radio_pressed_scene_structure_is_stable() {
    use fret_ui_material3::Radio;

    let cases = [
        (SchemeMode::Dark, DynamicVariant::TonalSpot, "dark/tonal"),
        (SchemeMode::Light, DynamicVariant::TonalSpot, "light/tonal"),
        (
            SchemeMode::Dark,
            DynamicVariant::Expressive,
            "dark/expressive",
        ),
        (
            SchemeMode::Light,
            DynamicVariant::Expressive,
            "light/expressive",
        ),
    ];

    for (mode, variant, label) in cases {
        let mut app = TestHost::default();
        app.set_global(PlatformCapabilities::default());
        apply_material_theme(&mut app, mode, variant);

        let window = AppWindowId::default();
        let mut services = FakeUiServices::default();
        let mut ui: UiTree<TestHost> = UiTree::new();
        ui.set_window(window);

        let bounds = Rect::new(
            Point::new(Px(0.0), Px(0.0)),
            Size::new(Px(320.0), Px(240.0)),
        );

        let selected = app.models_mut().insert(false);
        let render =
            |ui: &mut UiTree<TestHost>, app: &mut TestHost, services: &mut dyn UiServices| {
                fret_ui::declarative::render_root(ui, app, services, window, bounds, "root", |cx| {
                    let radio = Radio::new(selected.clone())
                        .a11y_label("radio")
                        .test_id("radio")
                        .into_element(cx);
                    vec![with_padding(cx, Px(32.0), radio)]
                })
            };

        let root = render(&mut ui, &mut app, &mut services);
        ui.set_root(root);
        ui.request_semantics_snapshot();
        ui.layout_all(&mut app, &mut services, bounds, 1.0);

        let radio_node: NodeId = ui
            .semantics_snapshot()
            .and_then(|snapshot| {
                snapshot.nodes.iter().find_map(|node| {
                    if node.test_id.as_deref() == Some("radio") {
                        Some(node.id)
                    } else {
                        None
                    }
                })
            })
            .expect("expected radio in semantics snapshot");
        let radio_bounds = ui
            .debug_node_visual_bounds(radio_node)
            .expect("expected radio visual bounds");
        let press_at = Point::new(
            Px(radio_bounds.origin.x.0 + radio_bounds.size.width.0 * 0.5),
            Px(radio_bounds.origin.y.0 + radio_bounds.size.height.0 * 0.5),
        );

        ui.dispatch_event(
            &mut app,
            &mut services,
            &pointer_down(PointerId(1), press_at),
        );

        let mut baseline_structure: Option<Vec<SceneSig>> = None;
        let mut baseline_quads: Option<Vec<QuadGeomSig>> = None;
        for frame in 0..24 {
            app.advance_frame();
            let root = render(&mut ui, &mut app, &mut services);
            ui.set_root(root);
            ui.layout_all(&mut app, &mut services, bounds, 1.0);

            let mut scene = Scene::default();
            ui.paint_all(&mut app, &mut services, bounds, &mut scene, 1.0);

            if frame >= 2 && frame < 7 {
                let sig = scene_signature(&scene);
                if let Some(prev) = baseline_structure.as_ref() {
                    assert_eq!(
                        sig, *prev,
                        "expected Radio to keep a stable scene structure while pressed ({label})"
                    );
                } else {
                    baseline_structure = Some(sig);
                }
            }

            if frame >= 16 {
                let sig = scene_quad_geometry_signature(&scene);
                if let Some(prev) = baseline_quads.as_ref() {
                    assert_eq!(
                        sig, *prev,
                        "expected Radio to keep stable quad geometry after animations settle ({label})"
                    );
                } else {
                    baseline_quads = Some(sig);
                }
            }
        }

        ui.dispatch_event(&mut app, &mut services, &pointer_up(PointerId(1), press_at));
    }
}
