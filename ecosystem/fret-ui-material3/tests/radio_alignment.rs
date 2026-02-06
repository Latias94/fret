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

fn apply_material_theme_rtl(app: &mut TestHost, mode: SchemeMode, variant: DynamicVariant) {
    let mut colors = ColorSchemeOptions::default();
    colors.mode = mode;
    colors.variant = variant;

    let mut cfg = theme_config_with_colors(TypographyOptions::default(), colors);
    cfg.numbers
        .insert("md.sys.fret.layout.is-rtl".to_string(), 1.0);
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

fn snapshot_material3_scene_at_frame_v1(
    app: &mut TestHost,
    ui: &mut UiTree<TestHost>,
    services: &mut dyn UiServices,
    bounds: Rect,
    scale_factor: f32,
    snapshot_frame: usize,
    render: &impl Fn(&mut UiTree<TestHost>, &mut TestHost, &mut dyn UiServices) -> NodeId,
) -> Material3HeadlessGoldenV1 {
    let mut snapshot: Option<Material3HeadlessGoldenV1> = None;
    for _frame in 0..=snapshot_frame {
        app.advance_frame();
        let root = render(ui, app, services);
        ui.set_root(root);
        ui.layout_all(app, services, bounds, scale_factor);

        let mut scene = Scene::default();
        ui.paint_all(app, services, bounds, &mut scene, scale_factor);
        snapshot = Some(material3_scene_snapshot_v1(&scene));
    }

    snapshot.unwrap_or_else(|| panic!("expected a snapshot at frame {snapshot_frame}"))
}

fn settle_material3_overlay_scene_snapshot_v1(
    app: &mut TestHost,
    ui: &mut UiTree<TestHost>,
    services: &mut dyn UiServices,
    window: AppWindowId,
    bounds: Rect,
    scale_factor: f32,
    settle_from_frame: usize,
    total_frames: usize,
    stable_message: &str,
    render: &impl Fn(&mut UiTree<TestHost>, &mut TestHost, &mut dyn UiServices) -> NodeId,
) -> Material3HeadlessGoldenV1 {
    let mut settled: Option<Material3HeadlessGoldenV1> = None;
    for frame in 0..total_frames {
        let scene = run_overlay_frame_with_scene_scaled(
            ui,
            app,
            services,
            window,
            bounds,
            scale_factor,
            false,
            |ui, app, services| render(ui, app, services),
        );

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

fn drain_zero_delay_timer_tokens(
    app: &mut TestHost,
    window: AppWindowId,
) -> Vec<fret_runtime::TimerToken> {
    let mut out: Vec<fret_runtime::TimerToken> = Vec::new();
    app.effects.retain(|effect| match effect {
        Effect::SetTimer {
            window: Some(w),
            token,
            after,
            repeat: None,
        } if *w == window && after.as_millis() == 0 => {
            out.push(*token);
            false
        }
        _ => true,
    });
    out
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
fn text_input_text_input_event_updates_model() {
    use fret_ui::element::TextInputProps;

    let mut app = TestHost::default();
    app.set_global(PlatformCapabilities::default());
    apply_material_theme(&mut app, SchemeMode::Light, DynamicVariant::TonalSpot);

    let window = AppWindowId::default();
    let mut services = FakeUiServices::default();
    let mut ui: UiTree<TestHost> = UiTree::new();
    ui.set_window(window);

    let bounds = Rect::new(
        Point::new(Px(0.0), Px(0.0)),
        Size::new(Px(520.0), Px(420.0)),
    );

    let model = app.models_mut().insert(String::new());
    let model_for_render = model.clone();

    let render =
        move |ui: &mut UiTree<TestHost>, app: &mut TestHost, services: &mut dyn UiServices| {
            fret_ui::declarative::render_root(ui, app, services, window, bounds, "root", |cx| {
                let mut props = TextInputProps::new(model_for_render.clone());
                props.layout.size.width = fret_ui::element::Length::Px(Px(200.0));
                props.layout.size.height = fret_ui::element::Length::Px(Px(40.0));
                props.a11y_label = Some(Arc::<str>::from("input"));
                props.test_id = Some(Arc::<str>::from("plain-text-input"));
                let input = cx.text_input(props);
                vec![with_padding(cx, Px(24.0), input)]
            })
        };

    let root = render(&mut ui, &mut app, &mut services);
    ui.set_root(root);
    ui.request_semantics_snapshot();
    ui.layout_all(&mut app, &mut services, bounds, 1.0);

    let input_node: NodeId = ui
        .semantics_snapshot()
        .and_then(|snapshot| {
            snapshot.nodes.iter().find_map(|node| {
                (node.test_id.as_deref() == Some("plain-text-input")).then_some(node.id)
            })
        })
        .expect("expected plain-text-input in semantics snapshot");

    ui.set_focus(Some(input_node));
    assert_eq!(
        ui.focus(),
        Some(input_node),
        "expected focus to be set to the input node",
    );

    ui.dispatch_event(&mut app, &mut services, &Event::TextInput("a".to_string()));

    let root = render(&mut ui, &mut app, &mut services);
    ui.set_root(root);
    ui.layout_all(&mut app, &mut services, bounds, 1.0);

    let value = app.models().get_cloned(&model).expect("model exists");
    assert_eq!(value, "a", "expected text input event to update the model");
}

#[test]
fn top_app_bar_exposes_toolbar_semantics_role() {
    use fret_ui_material3::{TopAppBar, TopAppBarVariant};

    let mut app = TestHost::default();
    app.set_global(PlatformCapabilities::default());
    apply_material_theme(&mut app, SchemeMode::Light, DynamicVariant::TonalSpot);

    let window = AppWindowId::default();
    let mut services = FakeUiServices::default();
    let mut ui: UiTree<TestHost> = UiTree::new();
    ui.set_window(window);

    let bounds = Rect::new(
        Point::new(Px(0.0), Px(0.0)),
        Size::new(Px(520.0), Px(220.0)),
    );

    let render =
        move |ui: &mut UiTree<TestHost>, app: &mut TestHost, services: &mut dyn UiServices| {
            fret_ui::declarative::render_root(ui, app, services, window, bounds, "root", |cx| {
                let bar = TopAppBar::new("TopAppBar")
                    .variant(TopAppBarVariant::Small)
                    .a11y_label("Material 3 Top App Bar")
                    .test_id("top-app-bar")
                    .into_element(cx);
                vec![with_padding(cx, Px(24.0), bar)]
            })
        };

    let root = render(&mut ui, &mut app, &mut services);
    ui.set_root(root);
    ui.request_semantics_snapshot();
    ui.layout_all(&mut app, &mut services, bounds, 1.0);

    let node = ui
        .semantics_snapshot()
        .and_then(|snapshot| {
            snapshot
                .nodes
                .iter()
                .find(|n| n.test_id.as_deref() == Some("top-app-bar"))
        })
        .expect("expected top-app-bar in semantics snapshot");

    assert_eq!(
        node.role,
        fret_core::SemanticsRole::Toolbar,
        "expected top app bar semantics role to be Toolbar",
    );
}

#[test]
fn snackbar_action_emits_command_and_dismisses() {
    use fret_runtime::CommandId;
    use fret_ui::action::UiActionHostAdapter;
    use fret_ui_kit::ToastStore;
    use fret_ui_material3::{Snackbar, SnackbarController, SnackbarHost};

    let mut app = TestHost::default();
    app.set_global(PlatformCapabilities::default());
    apply_material_theme(&mut app, SchemeMode::Light, DynamicVariant::TonalSpot);

    let window = AppWindowId::default();
    let mut services = FakeUiServices::default();
    let mut ui: UiTree<TestHost> = UiTree::new();
    ui.set_window(window);

    let bounds = Rect::new(
        Point::new(Px(0.0), Px(0.0)),
        Size::new(Px(860.0), Px(520.0)),
    );

    let store = app.models_mut().insert(ToastStore::default());
    let controller = SnackbarController::new(store.clone());
    let cmd = CommandId::new("material3_snackbar_action");

    {
        let mut action_host = UiActionHostAdapter { app: &mut app };
        let _id = controller.show(
            &mut action_host,
            window,
            Snackbar::new("Saved").action("Undo", cmd.clone()),
        );
    }

    let render =
        move |ui: &mut UiTree<TestHost>, app: &mut TestHost, services: &mut dyn UiServices| {
            let store = store.clone();
            fret_ui::declarative::render_root(ui, app, services, window, bounds, "root", |cx| {
                vec![SnackbarHost::new(store).max_snackbars(1).into_element(cx)]
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

    let toast_root = ui
        .semantics_snapshot()
        .and_then(|snapshot| {
            snapshot.nodes.iter().find(|node| {
                node.test_id
                    .as_deref()
                    .is_some_and(|id| id.starts_with("toast-entry-"))
            })
        })
        .expect("expected a toast-entry semantics node");

    let toast_root_id = toast_root.id;

    let snapshot = ui
        .semantics_snapshot()
        .expect("expected semantics snapshot for toast");
    let by_id: HashMap<NodeId, &fret_core::SemanticsNode> =
        snapshot.nodes.iter().map(|n| (n.id, n)).collect();

    let is_descendant_of = |mut node: NodeId, ancestor: NodeId| -> bool {
        let mut guard = 0usize;
        while guard < 256 {
            if node == ancestor {
                return true;
            }
            guard += 1;
            let Some(parent) = by_id.get(&node).and_then(|n| n.parent) else {
                return false;
            };
            node = parent;
        }
        false
    };

    let action_text = snapshot
        .nodes
        .iter()
        .find(|node| {
            node.label.as_deref() == Some("Undo") && is_descendant_of(node.id, toast_root_id)
        })
        .expect("expected the toast action text (Undo) to appear in semantics");

    let mut action_button_id = action_text.id;
    let mut guard = 0usize;
    while guard < 256 {
        guard += 1;
        let Some(node) = by_id.get(&action_button_id) else {
            break;
        };
        if node.role == fret_core::SemanticsRole::Button {
            break;
        }
        let Some(parent) = node.parent else {
            break;
        };
        action_button_id = parent;
    }

    let action_bounds = ui
        .debug_node_visual_bounds(action_button_id)
        .expect("expected toast action bounds");
    let click_at = Point::new(
        Px(action_bounds.origin.x.0 + action_bounds.size.width.0 * 0.5),
        Px(action_bounds.origin.y.0 + action_bounds.size.height.0 * 0.5),
    );

    app.effects.clear();
    ui.dispatch_event(
        &mut app,
        &mut services,
        &pointer_down(PointerId(1), click_at),
    );
    ui.dispatch_event(&mut app, &mut services, &pointer_up(PointerId(1), click_at));

    assert!(
        app.effects.iter().any(|effect| matches!(
            effect,
            Effect::Command { command, .. } if *command == cmd
        )),
        "expected clicking snackbar action to emit a command effect"
    );

    let remove_tokens: Vec<fret_runtime::TimerToken> = app
        .effects
        .iter()
        .filter_map(|effect| match effect {
            Effect::SetTimer {
                window: Some(w),
                token,
                after: _,
                repeat: None,
            } if *w == window => Some(*token),
            _ => None,
        })
        .collect();
    assert!(
        !remove_tokens.is_empty(),
        "expected snackbar dismiss to schedule a timer for removal"
    );

    for token in remove_tokens {
        ui.dispatch_event(&mut app, &mut services, &Event::Timer { token });
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

    let has_toast = ui.semantics_snapshot().is_some_and(|snapshot| {
        snapshot.nodes.iter().any(|node| {
            node.test_id
                .as_deref()
                .is_some_and(|id| id.starts_with("toast-entry-"))
        })
    });
    assert!(
        !has_toast,
        "expected snackbar to be removed after dismiss timer fires"
    );
}

#[test]
fn snackbar_dismiss_button_dismisses_without_emitting_command() {
    use fret_ui::action::UiActionHostAdapter;
    use fret_ui_kit::ToastStore;
    use fret_ui_material3::{Snackbar, SnackbarController, SnackbarHost};

    let mut app = TestHost::default();
    app.set_global(PlatformCapabilities::default());
    apply_material_theme(&mut app, SchemeMode::Light, DynamicVariant::TonalSpot);

    let window = AppWindowId::default();
    let mut services = FakeUiServices::default();
    let mut ui: UiTree<TestHost> = UiTree::new();
    ui.set_window(window);

    let bounds = Rect::new(
        Point::new(Px(0.0), Px(0.0)),
        Size::new(Px(860.0), Px(520.0)),
    );

    let store = app.models_mut().insert(ToastStore::default());
    let controller = SnackbarController::new(store.clone());
    {
        let mut action_host = UiActionHostAdapter { app: &mut app };
        let _id = controller.show(&mut action_host, window, Snackbar::new("Dismiss me"));
    }

    let render =
        move |ui: &mut UiTree<TestHost>, app: &mut TestHost, services: &mut dyn UiServices| {
            let store = store.clone();
            fret_ui::declarative::render_root(ui, app, services, window, bounds, "root", |cx| {
                vec![SnackbarHost::new(store).max_snackbars(1).into_element(cx)]
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

    let toast_root = ui
        .semantics_snapshot()
        .and_then(|snapshot| {
            snapshot.nodes.iter().find(|node| {
                node.test_id
                    .as_deref()
                    .is_some_and(|id| id.starts_with("toast-entry-"))
            })
        })
        .expect("expected a toast-entry semantics node");

    let toast_root_id = toast_root.id;

    let snapshot = ui
        .semantics_snapshot()
        .expect("expected semantics snapshot for toast");
    let by_id: HashMap<NodeId, &fret_core::SemanticsNode> =
        snapshot.nodes.iter().map(|n| (n.id, n)).collect();

    let is_descendant_of = |mut node: NodeId, ancestor: NodeId| -> bool {
        let mut guard = 0usize;
        while guard < 256 {
            if node == ancestor {
                return true;
            }
            guard += 1;
            let Some(parent) = by_id.get(&node).and_then(|n| n.parent) else {
                return false;
            };
            node = parent;
        }
        false
    };

    let close_text = snapshot
        .nodes
        .iter()
        .find(|node| {
            node.label.as_deref() == Some("\u{00D7}") && is_descendant_of(node.id, toast_root_id)
        })
        .expect("expected toast close glyph (×) to appear in semantics");

    let mut close_button_id = close_text.id;
    let mut guard = 0usize;
    while guard < 256 {
        guard += 1;
        let Some(node) = by_id.get(&close_button_id) else {
            break;
        };
        if node.role == fret_core::SemanticsRole::Button {
            break;
        }
        let Some(parent) = node.parent else {
            break;
        };
        close_button_id = parent;
    }

    let close_bounds = ui
        .debug_node_visual_bounds(close_button_id)
        .expect("expected close button bounds");
    let click_at = Point::new(
        Px(close_bounds.origin.x.0 + close_bounds.size.width.0 * 0.5),
        Px(close_bounds.origin.y.0 + close_bounds.size.height.0 * 0.5),
    );

    app.effects.clear();
    ui.dispatch_event(
        &mut app,
        &mut services,
        &pointer_down(PointerId(1), click_at),
    );
    ui.dispatch_event(&mut app, &mut services, &pointer_up(PointerId(1), click_at));

    assert!(
        !app.effects
            .iter()
            .any(|effect| matches!(effect, Effect::Command { .. })),
        "expected clicking snackbar dismiss button not to emit a command effect",
    );

    let remove_tokens: Vec<fret_runtime::TimerToken> = app
        .effects
        .iter()
        .filter_map(|effect| match effect {
            Effect::SetTimer {
                window: Some(w),
                token,
                after: _,
                repeat: None,
            } if *w == window => Some(*token),
            _ => None,
        })
        .collect();
    assert!(
        !remove_tokens.is_empty(),
        "expected snackbar dismiss to schedule a timer for removal"
    );

    for token in remove_tokens {
        ui.dispatch_event(&mut app, &mut services, &Event::Timer { token });
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

    let has_toast = ui.semantics_snapshot().is_some_and(|snapshot| {
        snapshot.nodes.iter().any(|node| {
            node.test_id
                .as_deref()
                .is_some_and(|id| id.starts_with("toast-entry-"))
        })
    });
    assert!(
        !has_toast,
        "expected snackbar to be removed after dismiss timer fires"
    );
}

#[test]
fn navigation_bar_roving_skips_disabled_and_updates_model() {
    use fret_icons::ids;
    use fret_ui_material3::{NavigationBar, NavigationBarItem};

    let mut app = TestHost::default();
    app.set_global(PlatformCapabilities::default());
    apply_material_theme(&mut app, SchemeMode::Light, DynamicVariant::TonalSpot);

    let window = AppWindowId::default();
    let mut services = FakeUiServices::default();
    let mut ui: UiTree<TestHost> = UiTree::new();
    ui.set_window(window);

    let bounds = Rect::new(
        Point::new(Px(0.0), Px(0.0)),
        Size::new(Px(560.0), Px(320.0)),
    );

    let value: Model<Arc<str>> = app.models_mut().insert(Arc::<str>::from("search"));
    let value_for_render = value.clone();

    let render =
        move |ui: &mut UiTree<TestHost>, app: &mut TestHost, services: &mut dyn UiServices| {
            let value = value_for_render.clone();
            fret_ui::declarative::render_root(ui, app, services, window, bounds, "root", |cx| {
                let bar = NavigationBar::new(value)
                    .a11y_label("Material 3 Navigation Bar")
                    .test_id("nav-bar")
                    .items(vec![
                        NavigationBarItem::new("search", "Search", ids::ui::SEARCH)
                            .a11y_label("Destination Search")
                            .test_id("nav-bar-search"),
                        NavigationBarItem::new("disabled", "Disabled", ids::ui::SLASH)
                            .disabled(true)
                            .a11y_label("Destination Disabled")
                            .test_id("nav-bar-disabled"),
                        NavigationBarItem::new("settings", "Settings", ids::ui::SETTINGS)
                            .a11y_label("Destination Settings")
                            .test_id("nav-bar-settings"),
                        NavigationBarItem::new("more", "More", ids::ui::MORE_HORIZONTAL)
                            .a11y_label("Destination More")
                            .test_id("nav-bar-more"),
                    ])
                    .into_element(cx);
                vec![with_padding(cx, Px(24.0), bar)]
            })
        };

    let root = render(&mut ui, &mut app, &mut services);
    ui.set_root(root);
    ui.request_semantics_snapshot();
    ui.layout_all(&mut app, &mut services, bounds, 1.0);

    let search_node: NodeId = ui
        .semantics_snapshot()
        .and_then(|snapshot| {
            snapshot.nodes.iter().find_map(|node| {
                (node.test_id.as_deref() == Some("nav-bar-search")).then_some(node.id)
            })
        })
        .expect("expected nav-bar-search in semantics snapshot");
    let disabled_node: NodeId = ui
        .semantics_snapshot()
        .and_then(|snapshot| {
            snapshot.nodes.iter().find_map(|node| {
                (node.test_id.as_deref() == Some("nav-bar-disabled")).then_some(node.id)
            })
        })
        .expect("expected nav-bar-disabled in semantics snapshot");
    let settings_node: NodeId = ui
        .semantics_snapshot()
        .and_then(|snapshot| {
            snapshot.nodes.iter().find_map(|node| {
                (node.test_id.as_deref() == Some("nav-bar-settings")).then_some(node.id)
            })
        })
        .expect("expected nav-bar-settings in semantics snapshot");
    let more_node: NodeId = ui
        .semantics_snapshot()
        .and_then(|snapshot| {
            snapshot.nodes.iter().find_map(|node| {
                (node.test_id.as_deref() == Some("nav-bar-more")).then_some(node.id)
            })
        })
        .expect("expected nav-bar-more in semantics snapshot");

    ui.set_focus(Some(search_node));
    ui.dispatch_event(&mut app, &mut services, &key_down(KeyCode::ArrowRight));
    ui.dispatch_event(&mut app, &mut services, &key_up(KeyCode::ArrowRight));

    let root = render(&mut ui, &mut app, &mut services);
    ui.set_root(root);
    ui.layout_all(&mut app, &mut services, bounds, 1.0);

    assert_eq!(
        ui.focus(),
        Some(settings_node),
        "expected ArrowRight to skip disabled destinations"
    );
    assert_ne!(
        ui.focus(),
        Some(disabled_node),
        "expected disabled destination to never receive focus"
    );

    let selected = app.models().get_cloned(&value).expect("value model exists");
    assert_eq!(
        selected.as_ref(),
        "settings",
        "expected selection to follow roving focus"
    );

    ui.dispatch_event(&mut app, &mut services, &key_down(KeyCode::End));
    ui.dispatch_event(&mut app, &mut services, &key_up(KeyCode::End));

    let root = render(&mut ui, &mut app, &mut services);
    ui.set_root(root);
    ui.layout_all(&mut app, &mut services, bounds, 1.0);

    assert_eq!(
        ui.focus(),
        Some(more_node),
        "expected End to rove to the last enabled destination"
    );

    ui.dispatch_event(&mut app, &mut services, &key_down(KeyCode::Home));
    ui.dispatch_event(&mut app, &mut services, &key_up(KeyCode::Home));

    let root = render(&mut ui, &mut app, &mut services);
    ui.set_root(root);
    ui.layout_all(&mut app, &mut services, bounds, 1.0);

    assert_eq!(
        ui.focus(),
        Some(search_node),
        "expected Home to rove to the first enabled destination"
    );
}

#[test]
fn navigation_bar_roving_wraps_and_skips_disabled_on_reverse() {
    use fret_icons::ids;
    use fret_ui_material3::{NavigationBar, NavigationBarItem};

    let mut app = TestHost::default();
    app.set_global(PlatformCapabilities::default());
    apply_material_theme(&mut app, SchemeMode::Light, DynamicVariant::TonalSpot);

    let window = AppWindowId::default();
    let mut services = FakeUiServices::default();
    let mut ui: UiTree<TestHost> = UiTree::new();
    ui.set_window(window);

    let bounds = Rect::new(
        Point::new(Px(0.0), Px(0.0)),
        Size::new(Px(560.0), Px(320.0)),
    );

    let value: Model<Arc<str>> = app.models_mut().insert(Arc::<str>::from("search"));
    let value_for_render = value.clone();

    let render =
        move |ui: &mut UiTree<TestHost>, app: &mut TestHost, services: &mut dyn UiServices| {
            let value = value_for_render.clone();
            fret_ui::declarative::render_root(ui, app, services, window, bounds, "root", |cx| {
                let bar = NavigationBar::new(value)
                    .a11y_label("Material 3 Navigation Bar")
                    .test_id("nav-bar")
                    .items(vec![
                        NavigationBarItem::new("search", "Search", ids::ui::SEARCH)
                            .a11y_label("Destination Search")
                            .test_id("nav-bar-search"),
                        NavigationBarItem::new("disabled", "Disabled", ids::ui::SLASH)
                            .disabled(true)
                            .a11y_label("Destination Disabled")
                            .test_id("nav-bar-disabled"),
                        NavigationBarItem::new("settings", "Settings", ids::ui::SETTINGS)
                            .a11y_label("Destination Settings")
                            .test_id("nav-bar-settings"),
                        NavigationBarItem::new("more", "More", ids::ui::MORE_HORIZONTAL)
                            .a11y_label("Destination More")
                            .test_id("nav-bar-more"),
                    ])
                    .into_element(cx);
                vec![with_padding(cx, Px(24.0), bar)]
            })
        };

    let root = render(&mut ui, &mut app, &mut services);
    ui.set_root(root);
    ui.request_semantics_snapshot();
    ui.layout_all(&mut app, &mut services, bounds, 1.0);

    let search_node: NodeId = ui
        .semantics_snapshot()
        .and_then(|snapshot| {
            snapshot.nodes.iter().find_map(|node| {
                (node.test_id.as_deref() == Some("nav-bar-search")).then_some(node.id)
            })
        })
        .expect("expected nav-bar-search in semantics snapshot");
    let disabled_node: NodeId = ui
        .semantics_snapshot()
        .and_then(|snapshot| {
            snapshot.nodes.iter().find_map(|node| {
                (node.test_id.as_deref() == Some("nav-bar-disabled")).then_some(node.id)
            })
        })
        .expect("expected nav-bar-disabled in semantics snapshot");
    let settings_node: NodeId = ui
        .semantics_snapshot()
        .and_then(|snapshot| {
            snapshot.nodes.iter().find_map(|node| {
                (node.test_id.as_deref() == Some("nav-bar-settings")).then_some(node.id)
            })
        })
        .expect("expected nav-bar-settings in semantics snapshot");
    let more_node: NodeId = ui
        .semantics_snapshot()
        .and_then(|snapshot| {
            snapshot.nodes.iter().find_map(|node| {
                (node.test_id.as_deref() == Some("nav-bar-more")).then_some(node.id)
            })
        })
        .expect("expected nav-bar-more in semantics snapshot");

    ui.set_focus(Some(search_node));
    ui.dispatch_event(&mut app, &mut services, &key_down(KeyCode::ArrowLeft));
    ui.dispatch_event(&mut app, &mut services, &key_up(KeyCode::ArrowLeft));

    let root = render(&mut ui, &mut app, &mut services);
    ui.set_root(root);
    ui.layout_all(&mut app, &mut services, bounds, 1.0);

    assert_eq!(
        ui.focus(),
        Some(more_node),
        "expected ArrowLeft to wrap to the last enabled destination when loop_navigation=true"
    );
    assert_ne!(
        ui.focus(),
        Some(disabled_node),
        "expected disabled destination to never receive focus"
    );
    let selected = app.models().get_cloned(&value).expect("value model exists");
    assert_eq!(
        selected.as_ref(),
        "more",
        "expected selection to follow roving focus after reverse wrap"
    );

    ui.dispatch_event(&mut app, &mut services, &key_down(KeyCode::ArrowLeft));
    ui.dispatch_event(&mut app, &mut services, &key_up(KeyCode::ArrowLeft));

    let root = render(&mut ui, &mut app, &mut services);
    ui.set_root(root);
    ui.layout_all(&mut app, &mut services, bounds, 1.0);

    assert_eq!(
        ui.focus(),
        Some(settings_node),
        "expected ArrowLeft to rove to the previous enabled destination"
    );

    // Now verify loop_navigation=false clamps at the first enabled item (no wrap).
    let value2: Model<Arc<str>> = app.models_mut().insert(Arc::<str>::from("search"));
    let value2_for_render = value2.clone();
    let render_no_loop =
        move |ui: &mut UiTree<TestHost>, app: &mut TestHost, services: &mut dyn UiServices| {
            let value2 = value2_for_render.clone();
            fret_ui::declarative::render_root(ui, app, services, window, bounds, "root2", |cx| {
                let bar = NavigationBar::new(value2)
                    .loop_navigation(false)
                    .a11y_label("Material 3 Navigation Bar (no loop)")
                    .test_id("nav-bar-no-loop")
                    .items(vec![
                        NavigationBarItem::new("search", "Search", ids::ui::SEARCH)
                            .a11y_label("Destination Search")
                            .test_id("nav-bar-no-loop-search"),
                        NavigationBarItem::new("more", "More", ids::ui::MORE_HORIZONTAL)
                            .a11y_label("Destination More")
                            .test_id("nav-bar-no-loop-more"),
                    ])
                    .into_element(cx);
                vec![with_padding(cx, Px(24.0), bar)]
            })
        };

    let root = render_no_loop(&mut ui, &mut app, &mut services);
    ui.set_root(root);
    ui.request_semantics_snapshot();
    ui.layout_all(&mut app, &mut services, bounds, 1.0);

    let search_node2: NodeId = ui
        .semantics_snapshot()
        .and_then(|snapshot| {
            snapshot.nodes.iter().find_map(|node| {
                (node.test_id.as_deref() == Some("nav-bar-no-loop-search")).then_some(node.id)
            })
        })
        .expect("expected nav-bar-no-loop-search in semantics snapshot");

    ui.set_focus(Some(search_node2));
    ui.dispatch_event(&mut app, &mut services, &key_down(KeyCode::ArrowLeft));
    ui.dispatch_event(&mut app, &mut services, &key_up(KeyCode::ArrowLeft));

    let root = render_no_loop(&mut ui, &mut app, &mut services);
    ui.set_root(root);
    ui.layout_all(&mut app, &mut services, bounds, 1.0);

    assert_eq!(
        ui.focus(),
        Some(search_node2),
        "expected ArrowLeft at the first item to not wrap when loop_navigation=false"
    );
}

#[test]
fn navigation_rail_roving_skips_disabled_and_updates_model() {
    use fret_icons::ids;
    use fret_ui_material3::{NavigationRail, NavigationRailItem};

    let mut app = TestHost::default();
    app.set_global(PlatformCapabilities::default());
    apply_material_theme(&mut app, SchemeMode::Light, DynamicVariant::TonalSpot);

    let window = AppWindowId::default();
    let mut services = FakeUiServices::default();
    let mut ui: UiTree<TestHost> = UiTree::new();
    ui.set_window(window);

    let bounds = Rect::new(
        Point::new(Px(0.0), Px(0.0)),
        Size::new(Px(360.0), Px(520.0)),
    );

    let value: Model<Arc<str>> = app.models_mut().insert(Arc::<str>::from("search"));
    let value_for_render = value.clone();

    let render =
        move |ui: &mut UiTree<TestHost>, app: &mut TestHost, services: &mut dyn UiServices| {
            let value = value_for_render.clone();
            fret_ui::declarative::render_root(ui, app, services, window, bounds, "root", |cx| {
                let rail = NavigationRail::new(value)
                    .a11y_label("Material 3 Navigation Rail")
                    .test_id("nav-rail")
                    .items(vec![
                        NavigationRailItem::new("search", "Search", ids::ui::SEARCH)
                            .a11y_label("Destination Search")
                            .test_id("nav-rail-search"),
                        NavigationRailItem::new("disabled", "Disabled", ids::ui::SLASH)
                            .disabled(true)
                            .a11y_label("Destination Disabled")
                            .test_id("nav-rail-disabled"),
                        NavigationRailItem::new("settings", "Settings", ids::ui::SETTINGS)
                            .a11y_label("Destination Settings")
                            .test_id("nav-rail-settings"),
                        NavigationRailItem::new("play", "Play", ids::ui::PLAY)
                            .a11y_label("Destination Play")
                            .test_id("nav-rail-play"),
                    ])
                    .into_element(cx);
                vec![with_padding(cx, Px(24.0), rail)]
            })
        };

    let root = render(&mut ui, &mut app, &mut services);
    ui.set_root(root);
    ui.request_semantics_snapshot();
    ui.layout_all(&mut app, &mut services, bounds, 1.0);

    let search_node: NodeId = ui
        .semantics_snapshot()
        .and_then(|snapshot| {
            snapshot.nodes.iter().find_map(|node| {
                (node.test_id.as_deref() == Some("nav-rail-search")).then_some(node.id)
            })
        })
        .expect("expected nav-rail-search in semantics snapshot");
    let disabled_node: NodeId = ui
        .semantics_snapshot()
        .and_then(|snapshot| {
            snapshot.nodes.iter().find_map(|node| {
                (node.test_id.as_deref() == Some("nav-rail-disabled")).then_some(node.id)
            })
        })
        .expect("expected nav-rail-disabled in semantics snapshot");
    let settings_node: NodeId = ui
        .semantics_snapshot()
        .and_then(|snapshot| {
            snapshot.nodes.iter().find_map(|node| {
                (node.test_id.as_deref() == Some("nav-rail-settings")).then_some(node.id)
            })
        })
        .expect("expected nav-rail-settings in semantics snapshot");
    let play_node: NodeId = ui
        .semantics_snapshot()
        .and_then(|snapshot| {
            snapshot.nodes.iter().find_map(|node| {
                (node.test_id.as_deref() == Some("nav-rail-play")).then_some(node.id)
            })
        })
        .expect("expected nav-rail-play in semantics snapshot");

    ui.set_focus(Some(search_node));
    ui.dispatch_event(&mut app, &mut services, &key_down(KeyCode::ArrowDown));
    ui.dispatch_event(&mut app, &mut services, &key_up(KeyCode::ArrowDown));

    let root = render(&mut ui, &mut app, &mut services);
    ui.set_root(root);
    ui.layout_all(&mut app, &mut services, bounds, 1.0);

    assert_eq!(
        ui.focus(),
        Some(settings_node),
        "expected ArrowDown to skip disabled destinations"
    );
    assert_ne!(
        ui.focus(),
        Some(disabled_node),
        "expected disabled destination to never receive focus"
    );

    let selected = app.models().get_cloned(&value).expect("value model exists");
    assert_eq!(
        selected.as_ref(),
        "settings",
        "expected selection to follow roving focus"
    );

    ui.dispatch_event(&mut app, &mut services, &key_down(KeyCode::End));
    ui.dispatch_event(&mut app, &mut services, &key_up(KeyCode::End));

    let root = render(&mut ui, &mut app, &mut services);
    ui.set_root(root);
    ui.layout_all(&mut app, &mut services, bounds, 1.0);

    assert_eq!(
        ui.focus(),
        Some(play_node),
        "expected End to rove to the last enabled destination"
    );

    ui.dispatch_event(&mut app, &mut services, &key_down(KeyCode::Home));
    ui.dispatch_event(&mut app, &mut services, &key_up(KeyCode::Home));

    let root = render(&mut ui, &mut app, &mut services);
    ui.set_root(root);
    ui.layout_all(&mut app, &mut services, bounds, 1.0);

    assert_eq!(
        ui.focus(),
        Some(search_node),
        "expected Home to rove to the first enabled destination"
    );
}

#[test]
fn navigation_rail_roving_wraps_and_skips_disabled_on_reverse() {
    use fret_icons::ids;
    use fret_ui_material3::{NavigationRail, NavigationRailItem};

    let mut app = TestHost::default();
    app.set_global(PlatformCapabilities::default());
    apply_material_theme(&mut app, SchemeMode::Light, DynamicVariant::TonalSpot);

    let window = AppWindowId::default();
    let mut services = FakeUiServices::default();
    let mut ui: UiTree<TestHost> = UiTree::new();
    ui.set_window(window);

    let bounds = Rect::new(
        Point::new(Px(0.0), Px(0.0)),
        Size::new(Px(360.0), Px(520.0)),
    );

    let value: Model<Arc<str>> = app.models_mut().insert(Arc::<str>::from("search"));
    let value_for_render = value.clone();

    let render =
        move |ui: &mut UiTree<TestHost>, app: &mut TestHost, services: &mut dyn UiServices| {
            let value = value_for_render.clone();
            fret_ui::declarative::render_root(ui, app, services, window, bounds, "root", |cx| {
                let rail = NavigationRail::new(value)
                    .a11y_label("Material 3 Navigation Rail")
                    .test_id("nav-rail")
                    .items(vec![
                        NavigationRailItem::new("search", "Search", ids::ui::SEARCH)
                            .a11y_label("Destination Search")
                            .test_id("nav-rail-search"),
                        NavigationRailItem::new("disabled", "Disabled", ids::ui::SLASH)
                            .disabled(true)
                            .a11y_label("Destination Disabled")
                            .test_id("nav-rail-disabled"),
                        NavigationRailItem::new("settings", "Settings", ids::ui::SETTINGS)
                            .a11y_label("Destination Settings")
                            .test_id("nav-rail-settings"),
                        NavigationRailItem::new("play", "Play", ids::ui::PLAY)
                            .a11y_label("Destination Play")
                            .test_id("nav-rail-play"),
                    ])
                    .into_element(cx);
                vec![with_padding(cx, Px(24.0), rail)]
            })
        };

    let root = render(&mut ui, &mut app, &mut services);
    ui.set_root(root);
    ui.request_semantics_snapshot();
    ui.layout_all(&mut app, &mut services, bounds, 1.0);

    let search_node: NodeId = ui
        .semantics_snapshot()
        .and_then(|snapshot| {
            snapshot.nodes.iter().find_map(|node| {
                (node.test_id.as_deref() == Some("nav-rail-search")).then_some(node.id)
            })
        })
        .expect("expected nav-rail-search in semantics snapshot");
    let disabled_node: NodeId = ui
        .semantics_snapshot()
        .and_then(|snapshot| {
            snapshot.nodes.iter().find_map(|node| {
                (node.test_id.as_deref() == Some("nav-rail-disabled")).then_some(node.id)
            })
        })
        .expect("expected nav-rail-disabled in semantics snapshot");
    let settings_node: NodeId = ui
        .semantics_snapshot()
        .and_then(|snapshot| {
            snapshot.nodes.iter().find_map(|node| {
                (node.test_id.as_deref() == Some("nav-rail-settings")).then_some(node.id)
            })
        })
        .expect("expected nav-rail-settings in semantics snapshot");
    let play_node: NodeId = ui
        .semantics_snapshot()
        .and_then(|snapshot| {
            snapshot.nodes.iter().find_map(|node| {
                (node.test_id.as_deref() == Some("nav-rail-play")).then_some(node.id)
            })
        })
        .expect("expected nav-rail-play in semantics snapshot");

    ui.set_focus(Some(search_node));
    ui.dispatch_event(&mut app, &mut services, &key_down(KeyCode::ArrowUp));
    ui.dispatch_event(&mut app, &mut services, &key_up(KeyCode::ArrowUp));

    let root = render(&mut ui, &mut app, &mut services);
    ui.set_root(root);
    ui.layout_all(&mut app, &mut services, bounds, 1.0);

    assert_eq!(
        ui.focus(),
        Some(play_node),
        "expected ArrowUp to wrap to the last enabled destination when loop_navigation=true"
    );
    assert_ne!(
        ui.focus(),
        Some(disabled_node),
        "expected disabled destination to never receive focus"
    );
    let selected = app.models().get_cloned(&value).expect("value model exists");
    assert_eq!(
        selected.as_ref(),
        "play",
        "expected selection to follow roving focus after reverse wrap"
    );

    ui.dispatch_event(&mut app, &mut services, &key_down(KeyCode::ArrowUp));
    ui.dispatch_event(&mut app, &mut services, &key_up(KeyCode::ArrowUp));

    let root = render(&mut ui, &mut app, &mut services);
    ui.set_root(root);
    ui.layout_all(&mut app, &mut services, bounds, 1.0);

    assert_eq!(
        ui.focus(),
        Some(settings_node),
        "expected ArrowUp to rove to the previous enabled destination"
    );
}

#[test]
fn navigation_rail_roving_does_not_wrap_when_loop_navigation_false() {
    use fret_icons::ids;
    use fret_ui_material3::{NavigationRail, NavigationRailItem};

    let mut app = TestHost::default();
    app.set_global(PlatformCapabilities::default());
    apply_material_theme(&mut app, SchemeMode::Light, DynamicVariant::TonalSpot);

    let window = AppWindowId::default();
    let mut services = FakeUiServices::default();
    let mut ui: UiTree<TestHost> = UiTree::new();
    ui.set_window(window);

    let bounds = Rect::new(
        Point::new(Px(0.0), Px(0.0)),
        Size::new(Px(360.0), Px(520.0)),
    );

    let value: Model<Arc<str>> = app.models_mut().insert(Arc::<str>::from("search"));
    let value_for_render = value.clone();

    let render =
        move |ui: &mut UiTree<TestHost>, app: &mut TestHost, services: &mut dyn UiServices| {
            let value = value_for_render.clone();
            fret_ui::declarative::render_root(ui, app, services, window, bounds, "root", |cx| {
                let rail = NavigationRail::new(value)
                    .loop_navigation(false)
                    .a11y_label("Material 3 Navigation Rail (no loop)")
                    .test_id("nav-rail-no-loop")
                    .items(vec![
                        NavigationRailItem::new("search", "Search", ids::ui::SEARCH)
                            .a11y_label("Destination Search")
                            .test_id("nav-rail-no-loop-search"),
                        NavigationRailItem::new("play", "Play", ids::ui::PLAY)
                            .a11y_label("Destination Play")
                            .test_id("nav-rail-no-loop-play"),
                    ])
                    .into_element(cx);
                vec![with_padding(cx, Px(24.0), rail)]
            })
        };

    let root = render(&mut ui, &mut app, &mut services);
    ui.set_root(root);
    ui.request_semantics_snapshot();
    ui.layout_all(&mut app, &mut services, bounds, 1.0);

    let search_node: NodeId = ui
        .semantics_snapshot()
        .and_then(|snapshot| {
            snapshot.nodes.iter().find_map(|node| {
                (node.test_id.as_deref() == Some("nav-rail-no-loop-search")).then_some(node.id)
            })
        })
        .expect("expected nav-rail-no-loop-search in semantics snapshot");
    let play_node: NodeId = ui
        .semantics_snapshot()
        .and_then(|snapshot| {
            snapshot.nodes.iter().find_map(|node| {
                (node.test_id.as_deref() == Some("nav-rail-no-loop-play")).then_some(node.id)
            })
        })
        .expect("expected nav-rail-no-loop-play in semantics snapshot");

    ui.set_focus(Some(search_node));
    ui.dispatch_event(&mut app, &mut services, &key_down(KeyCode::ArrowUp));
    ui.dispatch_event(&mut app, &mut services, &key_up(KeyCode::ArrowUp));

    let root = render(&mut ui, &mut app, &mut services);
    ui.set_root(root);
    ui.layout_all(&mut app, &mut services, bounds, 1.0);

    assert_eq!(
        ui.focus(),
        Some(search_node),
        "expected ArrowUp at the first item to not wrap when loop_navigation=false",
    );

    ui.set_focus(Some(play_node));
    ui.dispatch_event(&mut app, &mut services, &key_down(KeyCode::ArrowDown));
    ui.dispatch_event(&mut app, &mut services, &key_up(KeyCode::ArrowDown));

    let root = render(&mut ui, &mut app, &mut services);
    ui.set_root(root);
    ui.layout_all(&mut app, &mut services, bounds, 1.0);

    assert_eq!(
        ui.focus(),
        Some(play_node),
        "expected ArrowDown at the last item to not wrap when loop_navigation=false",
    );
}

#[test]
fn navigation_rail_roving_single_enabled_item_does_not_move_under_no_loop() {
    use fret_icons::ids;
    use fret_ui_material3::{NavigationRail, NavigationRailItem};

    let mut app = TestHost::default();
    app.set_global(PlatformCapabilities::default());
    apply_material_theme(&mut app, SchemeMode::Light, DynamicVariant::TonalSpot);

    let window = AppWindowId::default();
    let mut services = FakeUiServices::default();
    let mut ui: UiTree<TestHost> = UiTree::new();
    ui.set_window(window);

    let bounds = Rect::new(
        Point::new(Px(0.0), Px(0.0)),
        Size::new(Px(360.0), Px(520.0)),
    );

    let value: Model<Arc<str>> = app.models_mut().insert(Arc::<str>::from("settings"));
    let value_for_render = value.clone();

    let render =
        move |ui: &mut UiTree<TestHost>, app: &mut TestHost, services: &mut dyn UiServices| {
            let value = value_for_render.clone();
            fret_ui::declarative::render_root(ui, app, services, window, bounds, "root", |cx| {
                let rail = NavigationRail::new(value)
                    .loop_navigation(false)
                    .a11y_label("Material 3 Navigation Rail (single enabled, no loop)")
                    .test_id("nav-rail-single-enabled")
                    .items(vec![
                        NavigationRailItem::new("search", "Search", ids::ui::SEARCH)
                            .disabled(true)
                            .a11y_label("Destination Search (disabled)")
                            .test_id("nav-rail-single-enabled-search-disabled"),
                        NavigationRailItem::new("settings", "Settings", ids::ui::SETTINGS)
                            .a11y_label("Destination Settings")
                            .test_id("nav-rail-single-enabled-settings"),
                        NavigationRailItem::new("play", "Play", ids::ui::PLAY)
                            .disabled(true)
                            .a11y_label("Destination Play (disabled)")
                            .test_id("nav-rail-single-enabled-play-disabled"),
                    ])
                    .into_element(cx);
                vec![with_padding(cx, Px(24.0), rail)]
            })
        };

    let root = render(&mut ui, &mut app, &mut services);
    ui.set_root(root);
    ui.request_semantics_snapshot();
    ui.layout_all(&mut app, &mut services, bounds, 1.0);

    let settings_node: NodeId = ui
        .semantics_snapshot()
        .and_then(|snapshot| {
            snapshot.nodes.iter().find_map(|node| {
                (node.test_id.as_deref() == Some("nav-rail-single-enabled-settings"))
                    .then_some(node.id)
            })
        })
        .expect("expected nav-rail-single-enabled-settings in semantics snapshot");

    ui.set_focus(Some(settings_node));

    ui.dispatch_event(&mut app, &mut services, &key_down(KeyCode::ArrowUp));
    ui.dispatch_event(&mut app, &mut services, &key_up(KeyCode::ArrowUp));

    let root = render(&mut ui, &mut app, &mut services);
    ui.set_root(root);
    ui.layout_all(&mut app, &mut services, bounds, 1.0);

    assert_eq!(
        ui.focus(),
        Some(settings_node),
        "expected ArrowUp to keep focus when only one destination is enabled",
    );
    let selected = app.models().get_cloned(&value).expect("value model exists");
    assert_eq!(
        selected.as_ref(),
        "settings",
        "expected selection to remain on the only enabled destination",
    );

    ui.dispatch_event(&mut app, &mut services, &key_down(KeyCode::ArrowDown));
    ui.dispatch_event(&mut app, &mut services, &key_up(KeyCode::ArrowDown));

    let root = render(&mut ui, &mut app, &mut services);
    ui.set_root(root);
    ui.layout_all(&mut app, &mut services, bounds, 1.0);

    assert_eq!(
        ui.focus(),
        Some(settings_node),
        "expected ArrowDown to keep focus when only one destination is enabled",
    );
    let selected = app.models().get_cloned(&value).expect("value model exists");
    assert_eq!(
        selected.as_ref(),
        "settings",
        "expected selection to remain on the only enabled destination",
    );
}

#[test]
fn navigation_drawer_roving_skips_disabled_and_updates_model() {
    use fret_icons::ids;
    use fret_ui_material3::{NavigationDrawer, NavigationDrawerItem};

    let mut app = TestHost::default();
    app.set_global(PlatformCapabilities::default());
    apply_material_theme(&mut app, SchemeMode::Light, DynamicVariant::TonalSpot);

    let window = AppWindowId::default();
    let mut services = FakeUiServices::default();
    let mut ui: UiTree<TestHost> = UiTree::new();
    ui.set_window(window);

    let bounds = Rect::new(
        Point::new(Px(0.0), Px(0.0)),
        Size::new(Px(420.0), Px(520.0)),
    );

    let value: Model<Arc<str>> = app.models_mut().insert(Arc::<str>::from("search"));
    let value_for_render = value.clone();

    let render =
        move |ui: &mut UiTree<TestHost>, app: &mut TestHost, services: &mut dyn UiServices| {
            let value = value_for_render.clone();
            fret_ui::declarative::render_root(ui, app, services, window, bounds, "root", |cx| {
                let drawer = NavigationDrawer::new(value)
                    .a11y_label("Material 3 Navigation Drawer")
                    .test_id("nav-drawer")
                    .items(vec![
                        NavigationDrawerItem::new("search", "Search", ids::ui::SEARCH)
                            .a11y_label("Destination Search")
                            .test_id("nav-drawer-search"),
                        NavigationDrawerItem::new("disabled", "Disabled", ids::ui::SLASH)
                            .disabled(true)
                            .a11y_label("Destination Disabled")
                            .test_id("nav-drawer-disabled"),
                        NavigationDrawerItem::new("settings", "Settings", ids::ui::SETTINGS)
                            .badge_label("2")
                            .a11y_label("Destination Settings")
                            .test_id("nav-drawer-settings"),
                        NavigationDrawerItem::new("play", "Play", ids::ui::PLAY)
                            .badge_label("99+")
                            .a11y_label("Destination Play")
                            .test_id("nav-drawer-play"),
                    ])
                    .into_element(cx);
                vec![with_padding(cx, Px(24.0), drawer)]
            })
        };

    let root = render(&mut ui, &mut app, &mut services);
    ui.set_root(root);
    ui.request_semantics_snapshot();
    ui.layout_all(&mut app, &mut services, bounds, 1.0);

    let search_node: NodeId = ui
        .semantics_snapshot()
        .and_then(|snapshot| {
            snapshot.nodes.iter().find_map(|node| {
                (node.test_id.as_deref() == Some("nav-drawer-search")).then_some(node.id)
            })
        })
        .expect("expected nav-drawer-search in semantics snapshot");
    let disabled_node: NodeId = ui
        .semantics_snapshot()
        .and_then(|snapshot| {
            snapshot.nodes.iter().find_map(|node| {
                (node.test_id.as_deref() == Some("nav-drawer-disabled")).then_some(node.id)
            })
        })
        .expect("expected nav-drawer-disabled in semantics snapshot");
    let settings_node: NodeId = ui
        .semantics_snapshot()
        .and_then(|snapshot| {
            snapshot.nodes.iter().find_map(|node| {
                (node.test_id.as_deref() == Some("nav-drawer-settings")).then_some(node.id)
            })
        })
        .expect("expected nav-drawer-settings in semantics snapshot");
    let play_node: NodeId = ui
        .semantics_snapshot()
        .and_then(|snapshot| {
            snapshot.nodes.iter().find_map(|node| {
                (node.test_id.as_deref() == Some("nav-drawer-play")).then_some(node.id)
            })
        })
        .expect("expected nav-drawer-play in semantics snapshot");

    ui.set_focus(Some(search_node));
    ui.dispatch_event(&mut app, &mut services, &key_down(KeyCode::ArrowDown));
    ui.dispatch_event(&mut app, &mut services, &key_up(KeyCode::ArrowDown));

    let root = render(&mut ui, &mut app, &mut services);
    ui.set_root(root);
    ui.layout_all(&mut app, &mut services, bounds, 1.0);

    assert_eq!(
        ui.focus(),
        Some(settings_node),
        "expected ArrowDown to skip disabled destinations"
    );
    assert_ne!(
        ui.focus(),
        Some(disabled_node),
        "expected disabled destination to never receive focus"
    );

    let selected = app.models().get_cloned(&value).expect("value model exists");
    assert_eq!(
        selected.as_ref(),
        "settings",
        "expected selection to follow roving focus"
    );

    ui.dispatch_event(&mut app, &mut services, &key_down(KeyCode::End));
    ui.dispatch_event(&mut app, &mut services, &key_up(KeyCode::End));

    let root = render(&mut ui, &mut app, &mut services);
    ui.set_root(root);
    ui.layout_all(&mut app, &mut services, bounds, 1.0);

    assert_eq!(
        ui.focus(),
        Some(play_node),
        "expected End to rove to the last enabled destination"
    );

    ui.dispatch_event(&mut app, &mut services, &key_down(KeyCode::Home));
    ui.dispatch_event(&mut app, &mut services, &key_up(KeyCode::Home));

    let root = render(&mut ui, &mut app, &mut services);
    ui.set_root(root);
    ui.layout_all(&mut app, &mut services, bounds, 1.0);

    assert_eq!(
        ui.focus(),
        Some(search_node),
        "expected Home to rove to the first enabled destination"
    );
}

#[test]
fn navigation_drawer_roving_wraps_and_skips_disabled_on_reverse() {
    use fret_icons::ids;
    use fret_ui_material3::{NavigationDrawer, NavigationDrawerItem};

    let mut app = TestHost::default();
    app.set_global(PlatformCapabilities::default());
    apply_material_theme(&mut app, SchemeMode::Light, DynamicVariant::TonalSpot);

    let window = AppWindowId::default();
    let mut services = FakeUiServices::default();
    let mut ui: UiTree<TestHost> = UiTree::new();
    ui.set_window(window);

    let bounds = Rect::new(
        Point::new(Px(0.0), Px(0.0)),
        Size::new(Px(420.0), Px(520.0)),
    );

    let value: Model<Arc<str>> = app.models_mut().insert(Arc::<str>::from("search"));
    let value_for_render = value.clone();

    let render =
        move |ui: &mut UiTree<TestHost>, app: &mut TestHost, services: &mut dyn UiServices| {
            let value = value_for_render.clone();
            fret_ui::declarative::render_root(ui, app, services, window, bounds, "root", |cx| {
                let drawer = NavigationDrawer::new(value)
                    .a11y_label("Material 3 Navigation Drawer")
                    .test_id("nav-drawer")
                    .items(vec![
                        NavigationDrawerItem::new("search", "Search", ids::ui::SEARCH)
                            .a11y_label("Destination Search")
                            .test_id("nav-drawer-search"),
                        NavigationDrawerItem::new("disabled", "Disabled", ids::ui::SLASH)
                            .disabled(true)
                            .a11y_label("Destination Disabled")
                            .test_id("nav-drawer-disabled"),
                        NavigationDrawerItem::new("settings", "Settings", ids::ui::SETTINGS)
                            .badge_label("2")
                            .a11y_label("Destination Settings")
                            .test_id("nav-drawer-settings"),
                        NavigationDrawerItem::new("play", "Play", ids::ui::PLAY)
                            .badge_label("99+")
                            .a11y_label("Destination Play")
                            .test_id("nav-drawer-play"),
                    ])
                    .into_element(cx);
                vec![with_padding(cx, Px(24.0), drawer)]
            })
        };

    let root = render(&mut ui, &mut app, &mut services);
    ui.set_root(root);
    ui.request_semantics_snapshot();
    ui.layout_all(&mut app, &mut services, bounds, 1.0);

    let search_node: NodeId = ui
        .semantics_snapshot()
        .and_then(|snapshot| {
            snapshot.nodes.iter().find_map(|node| {
                (node.test_id.as_deref() == Some("nav-drawer-search")).then_some(node.id)
            })
        })
        .expect("expected nav-drawer-search in semantics snapshot");
    let disabled_node: NodeId = ui
        .semantics_snapshot()
        .and_then(|snapshot| {
            snapshot.nodes.iter().find_map(|node| {
                (node.test_id.as_deref() == Some("nav-drawer-disabled")).then_some(node.id)
            })
        })
        .expect("expected nav-drawer-disabled in semantics snapshot");
    let settings_node: NodeId = ui
        .semantics_snapshot()
        .and_then(|snapshot| {
            snapshot.nodes.iter().find_map(|node| {
                (node.test_id.as_deref() == Some("nav-drawer-settings")).then_some(node.id)
            })
        })
        .expect("expected nav-drawer-settings in semantics snapshot");
    let play_node: NodeId = ui
        .semantics_snapshot()
        .and_then(|snapshot| {
            snapshot.nodes.iter().find_map(|node| {
                (node.test_id.as_deref() == Some("nav-drawer-play")).then_some(node.id)
            })
        })
        .expect("expected nav-drawer-play in semantics snapshot");

    ui.set_focus(Some(search_node));
    ui.dispatch_event(&mut app, &mut services, &key_down(KeyCode::ArrowUp));
    ui.dispatch_event(&mut app, &mut services, &key_up(KeyCode::ArrowUp));

    let root = render(&mut ui, &mut app, &mut services);
    ui.set_root(root);
    ui.layout_all(&mut app, &mut services, bounds, 1.0);

    assert_eq!(
        ui.focus(),
        Some(play_node),
        "expected ArrowUp to wrap to the last enabled destination when loop_navigation=true"
    );
    assert_ne!(
        ui.focus(),
        Some(disabled_node),
        "expected disabled destination to never receive focus"
    );
    let selected = app.models().get_cloned(&value).expect("value model exists");
    assert_eq!(
        selected.as_ref(),
        "play",
        "expected selection to follow roving focus after reverse wrap"
    );

    ui.dispatch_event(&mut app, &mut services, &key_down(KeyCode::ArrowUp));
    ui.dispatch_event(&mut app, &mut services, &key_up(KeyCode::ArrowUp));

    let root = render(&mut ui, &mut app, &mut services);
    ui.set_root(root);
    ui.layout_all(&mut app, &mut services, bounds, 1.0);

    assert_eq!(
        ui.focus(),
        Some(settings_node),
        "expected ArrowUp to rove to the previous enabled destination"
    );
}

#[test]
fn navigation_drawer_roving_does_not_wrap_when_loop_navigation_false() {
    use fret_icons::ids;
    use fret_ui_material3::{NavigationDrawer, NavigationDrawerItem};

    let mut app = TestHost::default();
    app.set_global(PlatformCapabilities::default());
    apply_material_theme(&mut app, SchemeMode::Light, DynamicVariant::TonalSpot);

    let window = AppWindowId::default();
    let mut services = FakeUiServices::default();
    let mut ui: UiTree<TestHost> = UiTree::new();
    ui.set_window(window);

    let bounds = Rect::new(
        Point::new(Px(0.0), Px(0.0)),
        Size::new(Px(420.0), Px(520.0)),
    );

    let value: Model<Arc<str>> = app.models_mut().insert(Arc::<str>::from("search"));
    let value_for_render = value.clone();

    let render =
        move |ui: &mut UiTree<TestHost>, app: &mut TestHost, services: &mut dyn UiServices| {
            let value = value_for_render.clone();
            fret_ui::declarative::render_root(ui, app, services, window, bounds, "root", |cx| {
                let drawer = NavigationDrawer::new(value)
                    .loop_navigation(false)
                    .a11y_label("Material 3 Navigation Drawer (no loop)")
                    .test_id("nav-drawer-no-loop")
                    .items(vec![
                        NavigationDrawerItem::new("search", "Search", ids::ui::SEARCH)
                            .a11y_label("Destination Search")
                            .test_id("nav-drawer-no-loop-search"),
                        NavigationDrawerItem::new("play", "Play", ids::ui::PLAY)
                            .a11y_label("Destination Play")
                            .test_id("nav-drawer-no-loop-play"),
                    ])
                    .into_element(cx);
                vec![with_padding(cx, Px(24.0), drawer)]
            })
        };

    let root = render(&mut ui, &mut app, &mut services);
    ui.set_root(root);
    ui.request_semantics_snapshot();
    ui.layout_all(&mut app, &mut services, bounds, 1.0);

    let search_node: NodeId = ui
        .semantics_snapshot()
        .and_then(|snapshot| {
            snapshot.nodes.iter().find_map(|node| {
                (node.test_id.as_deref() == Some("nav-drawer-no-loop-search")).then_some(node.id)
            })
        })
        .expect("expected nav-drawer-no-loop-search in semantics snapshot");
    let play_node: NodeId = ui
        .semantics_snapshot()
        .and_then(|snapshot| {
            snapshot.nodes.iter().find_map(|node| {
                (node.test_id.as_deref() == Some("nav-drawer-no-loop-play")).then_some(node.id)
            })
        })
        .expect("expected nav-drawer-no-loop-play in semantics snapshot");

    ui.set_focus(Some(search_node));
    ui.dispatch_event(&mut app, &mut services, &key_down(KeyCode::ArrowUp));
    ui.dispatch_event(&mut app, &mut services, &key_up(KeyCode::ArrowUp));

    let root = render(&mut ui, &mut app, &mut services);
    ui.set_root(root);
    ui.layout_all(&mut app, &mut services, bounds, 1.0);

    assert_eq!(
        ui.focus(),
        Some(search_node),
        "expected ArrowUp at the first item to not wrap when loop_navigation=false",
    );

    ui.set_focus(Some(play_node));
    ui.dispatch_event(&mut app, &mut services, &key_down(KeyCode::ArrowDown));
    ui.dispatch_event(&mut app, &mut services, &key_up(KeyCode::ArrowDown));

    let root = render(&mut ui, &mut app, &mut services);
    ui.set_root(root);
    ui.layout_all(&mut app, &mut services, bounds, 1.0);

    assert_eq!(
        ui.focus(),
        Some(play_node),
        "expected ArrowDown at the last item to not wrap when loop_navigation=false",
    );
}

#[test]
fn navigation_drawer_roving_single_enabled_item_does_not_move_under_no_loop() {
    use fret_icons::ids;
    use fret_ui_material3::{NavigationDrawer, NavigationDrawerItem};

    let mut app = TestHost::default();
    app.set_global(PlatformCapabilities::default());
    apply_material_theme(&mut app, SchemeMode::Light, DynamicVariant::TonalSpot);

    let window = AppWindowId::default();
    let mut services = FakeUiServices::default();
    let mut ui: UiTree<TestHost> = UiTree::new();
    ui.set_window(window);

    let bounds = Rect::new(
        Point::new(Px(0.0), Px(0.0)),
        Size::new(Px(420.0), Px(520.0)),
    );

    let value: Model<Arc<str>> = app.models_mut().insert(Arc::<str>::from("settings"));
    let value_for_render = value.clone();

    let render =
        move |ui: &mut UiTree<TestHost>, app: &mut TestHost, services: &mut dyn UiServices| {
            let value = value_for_render.clone();
            fret_ui::declarative::render_root(ui, app, services, window, bounds, "root", |cx| {
                let drawer = NavigationDrawer::new(value)
                    .loop_navigation(false)
                    .a11y_label("Material 3 Navigation Drawer (single enabled, no loop)")
                    .test_id("nav-drawer-single-enabled")
                    .items(vec![
                        NavigationDrawerItem::new("search", "Search", ids::ui::SEARCH)
                            .disabled(true)
                            .a11y_label("Destination Search (disabled)")
                            .test_id("nav-drawer-single-enabled-search-disabled"),
                        NavigationDrawerItem::new("settings", "Settings", ids::ui::SETTINGS)
                            .a11y_label("Destination Settings")
                            .test_id("nav-drawer-single-enabled-settings"),
                        NavigationDrawerItem::new("play", "Play", ids::ui::PLAY)
                            .disabled(true)
                            .a11y_label("Destination Play (disabled)")
                            .test_id("nav-drawer-single-enabled-play-disabled"),
                    ])
                    .into_element(cx);
                vec![with_padding(cx, Px(24.0), drawer)]
            })
        };

    let root = render(&mut ui, &mut app, &mut services);
    ui.set_root(root);
    ui.request_semantics_snapshot();
    ui.layout_all(&mut app, &mut services, bounds, 1.0);

    let settings_node: NodeId = ui
        .semantics_snapshot()
        .and_then(|snapshot| {
            snapshot.nodes.iter().find_map(|node| {
                (node.test_id.as_deref() == Some("nav-drawer-single-enabled-settings"))
                    .then_some(node.id)
            })
        })
        .expect("expected nav-drawer-single-enabled-settings in semantics snapshot");

    ui.set_focus(Some(settings_node));

    ui.dispatch_event(&mut app, &mut services, &key_down(KeyCode::ArrowUp));
    ui.dispatch_event(&mut app, &mut services, &key_up(KeyCode::ArrowUp));

    let root = render(&mut ui, &mut app, &mut services);
    ui.set_root(root);
    ui.layout_all(&mut app, &mut services, bounds, 1.0);

    assert_eq!(
        ui.focus(),
        Some(settings_node),
        "expected ArrowUp to keep focus when only one destination is enabled",
    );
    let selected = app.models().get_cloned(&value).expect("value model exists");
    assert_eq!(
        selected.as_ref(),
        "settings",
        "expected selection to remain on the only enabled destination",
    );

    ui.dispatch_event(&mut app, &mut services, &key_down(KeyCode::ArrowDown));
    ui.dispatch_event(&mut app, &mut services, &key_up(KeyCode::ArrowDown));

    let root = render(&mut ui, &mut app, &mut services);
    ui.set_root(root);
    ui.layout_all(&mut app, &mut services, bounds, 1.0);

    assert_eq!(
        ui.focus(),
        Some(settings_node),
        "expected ArrowDown to keep focus when only one destination is enabled",
    );
    let selected = app.models().get_cloned(&value).expect("value model exists");
    assert_eq!(
        selected.as_ref(),
        "settings",
        "expected selection to remain on the only enabled destination",
    );
}

#[test]
fn time_picker_clock_dial_drag_updates_time() {
    use fret_ui_material3::{DockedTimePicker, TimePickerDisplayMode};
    use time::Time;

    let mut app = TestHost::default();
    app.set_global(PlatformCapabilities::default());
    apply_material_theme(&mut app, SchemeMode::Light, DynamicVariant::TonalSpot);

    let window = AppWindowId::default();
    let mut services = FakeUiServices::default();
    let mut ui: UiTree<TestHost> = UiTree::new();
    ui.set_window(window);

    let bounds = Rect::new(
        Point::new(Px(0.0), Px(0.0)),
        Size::new(Px(520.0), Px(420.0)),
    );

    let selected_time = Time::from_hms(9, 41, 0).expect("valid time");
    let time = app.models_mut().insert(selected_time);
    let time_for_render = time.clone();

    let render =
        move |ui: &mut UiTree<TestHost>, app: &mut TestHost, services: &mut dyn UiServices| {
            fret_ui::declarative::render_root(ui, app, services, window, bounds, "root", |cx| {
                let picker = DockedTimePicker::new(time_for_render.clone())
                    .display_mode(TimePickerDisplayMode::Dial)
                    .test_id("time-picker-docked")
                    .into_element(cx);
                vec![with_padding(cx, Px(24.0), picker)]
            })
        };

    let root = render(&mut ui, &mut app, &mut services);
    ui.set_root(root);
    ui.request_semantics_snapshot();
    ui.layout_all(&mut app, &mut services, bounds, 1.0);

    let dial: NodeId = ui
        .semantics_snapshot()
        .and_then(|snapshot| {
            snapshot.nodes.iter().find_map(|node| {
                if node.test_id.as_deref() == Some("time-picker-clock-dial") {
                    Some(node.id)
                } else {
                    None
                }
            })
        })
        .expect("expected time picker clock dial node in semantics snapshot");

    let dial_bounds = ui
        .debug_node_visual_bounds(dial)
        .expect("expected dial bounds");

    let center = Point::new(
        Px(dial_bounds.origin.x.0 + dial_bounds.size.width.0 * 0.5),
        Px(dial_bounds.origin.y.0 + dial_bounds.size.height.0 * 0.5),
    );
    let r = dial_bounds.size.width.0.min(dial_bounds.size.height.0) * 0.45;

    let start_at = Point::new(center.x, Px(center.y.0 - r));
    let drag_to = Point::new(Px(center.x.0 + r), center.y);

    ui.dispatch_event(
        &mut app,
        &mut services,
        &pointer_down(PointerId(1), start_at),
    );
    ui.dispatch_event(
        &mut app,
        &mut services,
        &pointer_move(PointerId(1), drag_to),
    );
    ui.dispatch_event(&mut app, &mut services, &pointer_up(PointerId(1), drag_to));

    let after = app
        .models()
        .get_cloned(&time)
        .unwrap_or_else(|| selected_time);
    assert_ne!(
        after, selected_time,
        "expected dial drag to update the time model"
    );
}

#[test]
fn time_picker_selector_keyboard_arrows_step_time() {
    use fret_ui_material3::{DockedTimePicker, TimePickerDisplayMode};
    use time::Time;

    let mut app = TestHost::default();
    app.set_global(PlatformCapabilities::default());
    apply_material_theme(&mut app, SchemeMode::Light, DynamicVariant::TonalSpot);

    let window = AppWindowId::default();
    let mut services = FakeUiServices::default();
    let mut ui: UiTree<TestHost> = UiTree::new();
    ui.set_window(window);

    let bounds = Rect::new(
        Point::new(Px(0.0), Px(0.0)),
        Size::new(Px(520.0), Px(420.0)),
    );

    let selected_time = Time::from_hms(9, 41, 0).expect("valid time");
    let time = app.models_mut().insert(selected_time);
    let time_for_render = time.clone();

    let render =
        move |ui: &mut UiTree<TestHost>, app: &mut TestHost, services: &mut dyn UiServices| {
            fret_ui::declarative::render_root(ui, app, services, window, bounds, "root", |cx| {
                let picker = DockedTimePicker::new(time_for_render.clone())
                    .display_mode(TimePickerDisplayMode::Dial)
                    .test_id("time-picker-docked")
                    .into_element(cx);
                vec![with_padding(cx, Px(24.0), picker)]
            })
        };

    let root = render(&mut ui, &mut app, &mut services);
    ui.set_root(root);
    ui.request_semantics_snapshot();
    ui.layout_all(&mut app, &mut services, bounds, 1.0);

    let hour_node: NodeId = ui
        .semantics_snapshot()
        .and_then(|snapshot| {
            snapshot.nodes.iter().find_map(|node| {
                if node.test_id.as_deref() == Some("time-picker-hour-selector") {
                    Some(node.id)
                } else {
                    None
                }
            })
        })
        .expect("expected hour selector node in semantics snapshot");

    ui.set_focus(Some(hour_node));
    assert_eq!(
        ui.focus(),
        Some(hour_node),
        "expected focus to be set to the hour input node"
    );
    ui.dispatch_event(&mut app, &mut services, &key_down(KeyCode::ArrowUp));
    ui.dispatch_event(&mut app, &mut services, &key_up(KeyCode::ArrowUp));

    let after_hour = app.models().get_cloned(&time).expect("time model exists");
    assert_eq!(
        after_hour,
        Time::from_hms(10, 41, 0).expect("valid time"),
        "expected ArrowUp on hour selector to step +1 hour",
    );

    let minute_node: NodeId = ui
        .semantics_snapshot()
        .and_then(|snapshot| {
            snapshot.nodes.iter().find_map(|node| {
                if node.test_id.as_deref() == Some("time-picker-minute-selector") {
                    Some(node.id)
                } else {
                    None
                }
            })
        })
        .expect("expected minute selector node in semantics snapshot");

    ui.set_focus(Some(minute_node));
    ui.dispatch_event(&mut app, &mut services, &key_down(KeyCode::ArrowDown));
    ui.dispatch_event(&mut app, &mut services, &key_up(KeyCode::ArrowDown));

    let after_minute = app.models().get_cloned(&time).expect("time model exists");
    assert_eq!(
        after_minute,
        Time::from_hms(10, 40, 0).expect("valid time"),
        "expected ArrowDown on minute selector to step -1 minute",
    );
}

#[test]
fn time_picker_time_input_replaces_and_auto_advances_hour() {
    use fret_ui_material3::{DockedTimePicker, TimePickerDisplayMode};
    use time::Time;

    let mut app = TestHost::default();
    app.set_global(PlatformCapabilities::default());
    apply_material_theme(&mut app, SchemeMode::Light, DynamicVariant::TonalSpot);

    let window = AppWindowId::default();
    let mut services = FakeUiServices::default();
    let mut ui: UiTree<TestHost> = UiTree::new();
    ui.set_window(window);

    let bounds = Rect::new(
        Point::new(Px(0.0), Px(0.0)),
        Size::new(Px(520.0), Px(420.0)),
    );

    let selected_time = Time::from_hms(9, 41, 0).expect("valid time");
    let time = app.models_mut().insert(selected_time);
    let time_for_render = time.clone();

    let render =
        move |ui: &mut UiTree<TestHost>, app: &mut TestHost, services: &mut dyn UiServices| {
            fret_ui::declarative::render_root(ui, app, services, window, bounds, "root", |cx| {
                let picker = DockedTimePicker::new(time_for_render.clone())
                    .display_mode(TimePickerDisplayMode::Input)
                    .test_id("time-picker-docked-input")
                    .into_element(cx);
                vec![with_padding(cx, Px(24.0), picker)]
            })
        };

    let root = render(&mut ui, &mut app, &mut services);
    ui.set_root(root);
    ui.request_semantics_snapshot();
    ui.layout_all(&mut app, &mut services, bounds, 1.0);

    let hour_node: NodeId = ui
        .semantics_snapshot()
        .and_then(|snapshot| {
            snapshot.nodes.iter().find_map(|node| {
                (node.test_id.as_deref() == Some("time-input-hour")).then_some(node.id)
            })
        })
        .expect("expected time-input-hour in semantics snapshot");
    let minute_node: NodeId = ui
        .semantics_snapshot()
        .and_then(|snapshot| {
            snapshot.nodes.iter().find_map(|node| {
                (node.test_id.as_deref() == Some("time-input-minute")).then_some(node.id)
            })
        })
        .expect("expected time-input-minute in semantics snapshot");

    ui.set_focus(Some(hour_node));

    ui.dispatch_event(&mut app, &mut services, &key_down(KeyCode::Digit1));
    ui.dispatch_event(&mut app, &mut services, &key_up(KeyCode::Digit1));
    ui.dispatch_event(&mut app, &mut services, &Event::TextInput("1".to_string()));

    let root = render(&mut ui, &mut app, &mut services);
    ui.set_root(root);
    ui.layout_all(&mut app, &mut services, bounds, 1.0);

    let after_first = app.models().get_cloned(&time).expect("time model exists");
    assert_eq!(
        after_first,
        Time::from_hms(1, 41, 0).expect("valid time"),
        "expected first digit to replace the existing hour",
    );

    ui.dispatch_event(&mut app, &mut services, &key_down(KeyCode::Digit2));
    ui.dispatch_event(&mut app, &mut services, &key_up(KeyCode::Digit2));
    ui.dispatch_event(&mut app, &mut services, &Event::TextInput("2".to_string()));

    for token in drain_zero_delay_timer_tokens(&mut app, window) {
        ui.dispatch_event(&mut app, &mut services, &Event::Timer { token });
    }

    let root = render(&mut ui, &mut app, &mut services);
    ui.set_root(root);
    ui.request_semantics_snapshot();
    ui.layout_all(&mut app, &mut services, bounds, 1.0);

    let after_second = app.models().get_cloned(&time).expect("time model exists");
    assert_eq!(
        after_second,
        Time::from_hms(0, 41, 0).expect("valid time"),
        "expected second digit to complete a two-digit hour (12 AM -> 00h in 24h time)",
    );
    assert_eq!(
        ui.focus(),
        Some(minute_node),
        "expected entering a two-digit hour to auto-advance focus to minutes",
    );
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
        let mut prev_quads: Option<Vec<QuadGeomSig>> = None;
        let mut stable_quads_count: usize = 0;
        let settle_probe_start = 12;
        for frame in 0..48 {
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

            if frame >= settle_probe_start {
                let sig = scene_quad_geometry_signature(&scene);
                match prev_quads.as_ref() {
                    None => {
                        stable_quads_count = 1;
                    }
                    Some(prev) if sig == *prev => {
                        stable_quads_count += 1;
                    }
                    Some(_) => {
                        stable_quads_count = 1;
                    }
                }
                prev_quads = Some(sig);
            }
        }

        assert!(
            stable_quads_count >= 6,
            "expected Tabs quad geometry to stabilize after animations settle ({label})"
        );

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
fn menu_style_overrides_apply_to_container_and_label() {
    use fret_core::Color;
    use fret_ui_kit::{ColorRef, WidgetStateProperty};
    use fret_ui_material3::menu::{Menu, MenuEntry, MenuItem, MenuStyle};

    let mut app = TestHost::default();
    app.set_global(PlatformCapabilities::default());
    apply_material_theme(&mut app, SchemeMode::Light, DynamicVariant::TonalSpot);

    let window = AppWindowId::default();
    let mut services = FakeUiServices::default();
    let mut ui: UiTree<TestHost> = UiTree::new();
    ui.set_window(window);

    let bounds = Rect::new(
        Point::new(Px(0.0), Px(0.0)),
        Size::new(Px(360.0), Px(260.0)),
    );

    let override_bg = Color {
        r: 0.9,
        g: 0.1,
        b: 0.2,
        a: 1.0,
    };
    let override_label = Color {
        r: 0.1,
        g: 0.8,
        b: 0.3,
        a: 1.0,
    };

    let style = MenuStyle::default()
        .container_background(WidgetStateProperty::new(Some(ColorRef::Color(override_bg))))
        .item_label_color(WidgetStateProperty::new(Some(ColorRef::Color(
            override_label,
        ))));

    let entries = vec![
        MenuEntry::Item(MenuItem::new("A").test_id("menu-item-a")),
        MenuEntry::Item(MenuItem::new("B").test_id("menu-item-b")),
    ];

    let root = fret_ui::declarative::render_root(
        &mut ui,
        &mut app,
        &mut services,
        window,
        bounds,
        "root",
        |cx| {
            let menu = Menu::new()
                .entries(entries.clone())
                .a11y_label("menu")
                .test_id("menu")
                .style(style.clone())
                .into_element(cx);
            vec![with_padding(cx, Px(24.0), menu)]
        },
    );
    ui.set_root(root);
    ui.layout_all(&mut app, &mut services, bounds, 1.0);

    let mut scene = Scene::default();
    ui.paint_all(&mut app, &mut services, bounds, &mut scene, 1.0);

    assert!(
        scene.ops().iter().any(|op| {
            matches!(op, SceneOp::Quad { background, .. } if *background == override_bg)
        }),
        "expected MenuStyle.container_background to affect at least one quad background"
    );
    assert!(
        scene
            .ops()
            .iter()
            .any(|op| { matches!(op, SceneOp::Text { color, .. } if *color == override_label) }),
        "expected MenuStyle.item_label_color to affect at least one text draw op"
    );
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
fn dialog_style_overrides_apply_to_container_and_text() {
    use fret_core::Color;
    use fret_ui_kit::{ColorRef, WidgetStateProperty};
    use fret_ui_material3::{Button, Dialog, DialogAction, DialogStyle};

    let mut app = TestHost::default();
    app.set_global(PlatformCapabilities::default());
    apply_material_theme(&mut app, SchemeMode::Light, DynamicVariant::TonalSpot);

    let window = AppWindowId::default();
    let mut services = FakeUiServices::default();
    let mut ui: UiTree<TestHost> = UiTree::new();
    ui.set_window(window);

    let bounds = Rect::new(
        Point::new(Px(0.0), Px(0.0)),
        Size::new(Px(420.0), Px(320.0)),
    );

    let open = app.models_mut().insert(true);

    let override_bg = Color {
        r: 0.2,
        g: 0.2,
        b: 0.9,
        a: 1.0,
    };
    let override_headline = Color {
        r: 0.9,
        g: 0.9,
        b: 0.2,
        a: 1.0,
    };
    let override_supporting = Color {
        r: 0.8,
        g: 0.2,
        b: 0.8,
        a: 1.0,
    };

    let style = DialogStyle::default()
        .container_background(WidgetStateProperty::new(Some(ColorRef::Color(override_bg))))
        .headline_color(WidgetStateProperty::new(Some(ColorRef::Color(
            override_headline,
        ))))
        .supporting_text_color(WidgetStateProperty::new(Some(ColorRef::Color(
            override_supporting,
        ))));

    let render = |ui: &mut UiTree<TestHost>, app: &mut TestHost, services: &mut dyn UiServices| {
        fret_ui::declarative::render_root(ui, app, services, window, bounds, "root", |cx| {
            let dialog = Dialog::new(open.clone())
                .headline("Dialog")
                .supporting_text("Body")
                .actions(vec![DialogAction::new("OK").test_id("dialog-ok")])
                .style(style.clone())
                .test_id("dialog")
                .into_element(
                    cx,
                    |cx| {
                        let trigger = Button::new("Underlay focus probe")
                            .test_id("dialog-trigger")
                            .into_element(cx);
                        with_padding(cx, Px(24.0), trigger)
                    },
                    |_cx| Vec::new(),
                );
            vec![dialog]
        })
    };

    let mut scene = None;
    for _ in 0..3 {
        use fret_ui_kit::OverlayController;

        app.advance_frame();
        OverlayController::begin_frame(&mut app, window);

        let root = render(&mut ui, &mut app, &mut services);
        ui.set_root(root);
        OverlayController::render(&mut ui, &mut app, &mut services, window, bounds);

        ui.layout_all(&mut app, &mut services, bounds, 1.0);
        let mut next = Scene::default();
        ui.paint_all(&mut app, &mut services, bounds, &mut next, 1.0);
        scene = Some(next);
    }

    let scene = scene.expect("expected rendered scene");

    assert!(
        scene.ops().iter().any(|op| {
            matches!(op, SceneOp::Quad { background, .. } if *background == override_bg)
        }),
        "expected DialogStyle.container_background to affect at least one quad background"
    );
    assert!(
        scene
            .ops()
            .iter()
            .any(|op| { matches!(op, SceneOp::Text { color, .. } if *color == override_headline) }),
        "expected DialogStyle.headline_color to affect at least one text draw op"
    );
    assert!(
        scene.ops().iter().any(|op| {
            matches!(op, SceneOp::Text { color, .. } if *color == override_supporting)
        }),
        "expected DialogStyle.supporting_text_color to affect at least one text draw op"
    );
}

#[test]
fn dialog_scrim_dismisses_without_activating_underlay() {
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
        let underlay_toggled = app.models_mut().insert(false);

        let open_model = open.clone();
        let underlay_model = underlay_toggled.clone();
        let render =
            move |ui: &mut UiTree<TestHost>, app: &mut TestHost, services: &mut dyn UiServices| {
                let open_model = open_model.clone();
                let underlay_model = underlay_model.clone();
                fret_ui::declarative::render_root(ui, app, services, window, bounds, "root", |cx| {
                    let underlay_model = underlay_model.clone();
                    let underlay = cx.pressable(
                        fret_ui::element::PressableProps {
                            enabled: true,
                            focusable: false,
                            a11y: fret_ui::element::PressableA11y {
                                test_id: Some(std::sync::Arc::<str>::from("underlay-fullscreen")),
                                ..Default::default()
                            },
                            layout: {
                                let mut l = fret_ui::element::LayoutStyle::default();
                                l.position = fret_ui::element::PositionStyle::Absolute;
                                l.size.width = fret_ui::element::Length::Fill;
                                l.size.height = fret_ui::element::Length::Fill;
                                l.inset = fret_ui::element::InsetStyle {
                                    top: Some(Px(0.0)),
                                    right: Some(Px(0.0)),
                                    bottom: Some(Px(0.0)),
                                    left: Some(Px(0.0)),
                                };
                                l
                            },
                            ..Default::default()
                        },
                        move |cx, _st| {
                            cx.pressable_toggle_bool(&underlay_model);
                            Vec::new()
                        },
                    );

                    let dialog = Dialog::new(open_model.clone())
                        .headline("Dialog")
                        .supporting_text("Body")
                        .actions(vec![DialogAction::new("OK").test_id("dialog-ok")])
                        .test_id("dialog")
                        .into_element(
                            cx,
                            move |cx| {
                                let trigger = Button::new("Open dialog")
                                    .test_id("dialog-trigger")
                                    .into_element(cx);
                                with_padding(cx, Px(24.0), trigger)
                            },
                            |_cx| Vec::new(),
                        );
                    vec![underlay, dialog]
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
                    (node.test_id.as_deref() == Some("dialog-trigger")).then_some(node.id)
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
        assert!(
            snapshot
                .nodes
                .iter()
                .any(|node| node.test_id.as_deref() == Some("underlay-fullscreen")),
            "expected underlay-fullscreen node while dialog is open ({label})"
        );

        let click_at = Point::new(Px(4.0), Px(4.0));

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
            "expected dialog to dismiss on scrim press ({label})"
        );
        assert_eq!(
            app.models().get_copied(&underlay_toggled),
            Some(false),
            "expected dialog scrim to prevent underlay activation ({label})"
        );
        assert_eq!(
            ui.focus(),
            Some(trigger_node),
            "expected dialog to restore focus to trigger after scrim dismissal ({label})"
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
fn rich_tooltip_opens_and_closes_on_hover_smoke() {
    use fret_ui_kit::{OverlayController, OverlayStackEntryKind};
    use fret_ui_material3::{Button, RichTooltip, TooltipProvider};

    let mut app = TestHost::default();
    app.set_global(PlatformCapabilities::default());
    apply_material_theme(&mut app, SchemeMode::Dark, DynamicVariant::TonalSpot);

    let window = AppWindowId::default();
    let mut services = FakeUiServices::default();
    let mut ui: UiTree<TestHost> = UiTree::new();
    ui.set_window(window);

    let bounds = Rect::new(
        Point::new(Px(0.0), Px(0.0)),
        Size::new(Px(420.0), Px(320.0)),
    );

    let render = |ui: &mut UiTree<TestHost>, app: &mut TestHost, services: &mut dyn UiServices| {
        fret_ui::declarative::render_root(ui, app, services, window, bounds, "root", |cx| {
            TooltipProvider::new()
                .delay_duration_frames(0)
                .skip_delay_duration_frames(0)
                .with_elements(cx, |cx| {
                    let trigger = Button::new("Trigger")
                        .test_id("tooltip-trigger")
                        .into_element(cx);
                    let tooltip = RichTooltip::new(trigger, "Supporting text")
                        .title("Title")
                        .open_delay_frames(Some(0))
                        .close_delay_frames(Some(0))
                        .into_element(cx);
                    vec![with_padding(cx, Px(24.0), tooltip)]
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
        .expect("expected tooltip-trigger in semantics snapshot");
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
    assert!(opened, "expected rich tooltip to open on hover");

    ui.dispatch_event(
        &mut app,
        &mut services,
        &pointer_move(PointerId(1), Point::new(Px(0.0), Px(0.0))),
    );

    let mut closed = false;
    for _ in 0..10 {
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
    assert!(closed, "expected rich tooltip to close after unhover");
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
    use fret_ui::element::{ContainerProps, FlexProps, Length, TextProps};
    use fret_ui_material3::{
        AssistChip, AssistChipVariant, Button, Card, CardVariant, Checkbox, FilterChip,
        FilterChipVariant, InputChip, Select, SelectItem, SuggestionChip, SuggestionChipVariant,
        Switch,
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
                Size::new(Px(420.0), Px(560.0)),
            );

            let checkbox_checked = app.models_mut().insert(true);
            let checkbox_unchecked = app.models_mut().insert(false);
            let switch_on = app.models_mut().insert(true);
            let switch_off = app.models_mut().insert(false);
            let filter_chip_selected = app.models_mut().insert(true);
            let filter_chip_unselected = app.models_mut().insert(false);
            let input_chip_selected = app.models_mut().insert(true);
            let input_chip_unselected = app.models_mut().insert(false);
            let select_empty: Model<Option<Arc<str>>> = app.models_mut().insert(None);
            let select_populated: Model<Option<Arc<str>>> =
                app.models_mut().insert(Some(Arc::<str>::from("beta")));

            let select_items: Arc<[SelectItem]> = vec![
                SelectItem::new("alpha", "Alpha"),
                SelectItem::new("beta", "Beta"),
                SelectItem::new("charlie", "Charlie (disabled)").disabled(true),
            ]
            .into();

            let render = |ui: &mut UiTree<TestHost>,
                          app: &mut TestHost,
                          services: &mut dyn UiServices| {
                fret_ui::declarative::render_root(ui, app, services, window, bounds, "root", |cx| {
                    let mut props = FlexProps::default();
                    props.direction = fret_core::Axis::Vertical;
                    props.gap = Px(16.0);

                    let content = cx.flex(props, |cx| {
                        let theme = Theme::global(&*cx.app).clone();
                        let body_style = theme
                            .text_style_by_key("md.sys.typescale.body-medium")
                            .unwrap_or_else(|| fret_core::TextStyle::default());
                        let body_color = theme.color_required("md.sys.color.on-surface");

                        let card_content =
                            |cx: &mut fret_ui::elements::ElementContext<'_, TestHost>,
                             label: &'static str| {
                                let mut container = ContainerProps::default();
                                container.layout.size.width = Length::Px(Px(360.0));
                                container.layout.size.height = Length::Px(Px(72.0));
                                container.padding = Edges::all(Px(12.0));

                                let mut text = TextProps::new(Arc::<str>::from(label));
                                text.style = Some(body_style.clone());
                                text.color = Some(body_color);

                                cx.container(container, move |cx| vec![cx.text_props(text)])
                            };

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
                            AssistChip::new("Assist chip")
                                .test_id("chip-flat")
                                .into_element(cx),
                            AssistChip::new("Assist chip (icon)")
                                .leading_icon(fret_icons::ids::ui::SETTINGS)
                                .variant(AssistChipVariant::Elevated)
                                .test_id("chip-elevated")
                                .into_element(cx),
                            SuggestionChip::new("Suggestion chip")
                                .test_id("chip-suggestion-flat")
                                .into_element(cx),
                            SuggestionChip::new("Suggestion chip (icon)")
                                .leading_icon(fret_icons::ids::ui::SEARCH)
                                .variant(SuggestionChipVariant::Elevated)
                                .test_id("chip-suggestion-elevated")
                                .into_element(cx),
                            FilterChip::new(filter_chip_selected.clone(), "Filter chip")
                                .test_id("chip-filter-selected")
                                .into_element(cx),
                            FilterChip::new(filter_chip_unselected.clone(), "Filter chip (icon)")
                                .trailing_icon(fret_icons::ids::ui::SLASH)
                                .variant(FilterChipVariant::Elevated)
                                .test_id("chip-filter-unselected-elevated")
                                .into_element(cx),
                            InputChip::new(input_chip_selected.clone(), "Input chip (icon)")
                                .leading_icon(fret_icons::ids::ui::SETTINGS)
                                .test_id("chip-input-selected")
                                .into_element(cx),
                            InputChip::new(input_chip_unselected.clone(), "Input chip")
                                .trailing_icon(fret_icons::ids::ui::SLASH)
                                .test_id("chip-input-unselected")
                                .into_element(cx),
                            Card::new()
                                .variant(CardVariant::Filled)
                                .test_id("card-filled")
                                .into_element(cx, |cx| vec![card_content(cx, "Filled card")]),
                            Card::new()
                                .variant(CardVariant::Outlined)
                                .test_id("card-outlined")
                                .into_element(cx, |cx| vec![card_content(cx, "Outlined card")]),
                            Select::new(select_empty.clone())
                                .leading_icon(fret_icons::ids::ui::SEARCH)
                                .label("Select")
                                .supporting_text("Supporting text")
                                .placeholder("Pick one")
                                .items(select_items.clone())
                                .test_id("sel-empty")
                                .into_element(cx),
                            Select::new(select_populated.clone())
                                .leading_icon(fret_icons::ids::ui::SETTINGS)
                                .label("Select")
                                .supporting_text("Supporting text")
                                .placeholder("Pick one")
                                .items(select_items.clone())
                                .test_id("sel-populated")
                                .into_element(cx),
                            Select::new(select_empty.clone())
                                .leading_icon(fret_icons::ids::ui::SEARCH)
                                .label("Select")
                                .supporting_text("Error supporting text")
                                .placeholder("Pick one")
                                .items(select_items.clone())
                                .error(true)
                                .test_id("sel-error")
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

            let select_empty_node: NodeId = ui
                .semantics_snapshot()
                .and_then(|snapshot| {
                    snapshot.nodes.iter().find_map(|node| {
                        (node.test_id.as_deref() == Some("sel-empty")).then_some(node.id)
                    })
                })
                .unwrap_or_else(|| {
                    panic!("expected sel-empty in semantics snapshot ({label}, {scale})")
                });
            let select_error_node: NodeId = ui
                .semantics_snapshot()
                .and_then(|snapshot| {
                    snapshot.nodes.iter().find_map(|node| {
                        (node.test_id.as_deref() == Some("sel-error")).then_some(node.id)
                    })
                })
                .unwrap_or_else(|| {
                    panic!("expected sel-error in semantics snapshot ({label}, {scale})")
                });
            let btn_bounds = ui
                .debug_node_visual_bounds(btn_node)
                .unwrap_or_else(|| panic!("expected btn-filled bounds ({label}, {scale})"));
            let btn_center = Point::new(
                Px(btn_bounds.origin.x.0 + btn_bounds.size.width.0 * 0.5),
                Px(btn_bounds.origin.y.0 + btn_bounds.size.height.0 * 0.5),
            );
            let select_error_bounds = ui
                .debug_node_visual_bounds(select_error_node)
                .unwrap_or_else(|| panic!("expected sel-error bounds ({label}, {scale})"));
            let select_error_center = Point::new(
                Px(select_error_bounds.origin.x.0 + select_error_bounds.size.width.0 * 0.5),
                Px(select_error_bounds.origin.y.0 + select_error_bounds.size.height.0 * 0.5),
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

            let render_select_supporting_text_insets =
                |ui: &mut UiTree<TestHost>, app: &mut TestHost, services: &mut dyn UiServices| {
                    fret_ui::declarative::render_root(
                        ui,
                        app,
                        services,
                        window,
                        bounds,
                        "select_insets_root",
                        |cx| {
                            let mut props = FlexProps::default();
                            props.direction = fret_core::Axis::Vertical;
                            props.gap = Px(16.0);

                            let content = cx.flex(props, |cx| {
                                vec![
                                    Select::new(select_empty.clone())
                                        .label("Select")
                                        .supporting_text("Supporting text")
                                        .placeholder("Pick one")
                                        .items(select_items.clone())
                                        .test_id("sel-inset-no-icon")
                                        .into_element(cx),
                                    Select::new(select_populated.clone())
                                        .leading_icon(fret_icons::ids::ui::SEARCH)
                                        .label("Select")
                                        .supporting_text("Supporting text")
                                        .placeholder("Pick one")
                                        .items(select_items.clone())
                                        .test_id("sel-inset-icon")
                                        .into_element(cx),
                                ]
                            });

                            vec![with_padding(cx, Px(24.0), content)]
                        },
                    )
                };

            let select_supporting_inset_message = format!(
                "expected the Material3 select supporting text inset scenes to be stable after animations settle ({label}, {scale})"
            );
            cases.insert(
                "idle_select_supporting_text_insets".to_string(),
                settle_material3_scene_snapshot_v1(
                    &mut app,
                    &mut ui,
                    &mut services,
                    bounds,
                    scale_factor,
                    24,
                    40,
                    &select_supporting_inset_message,
                    &render_select_supporting_text_insets,
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

            ui.dispatch_event(
                &mut app,
                &mut services,
                &pointer_move(PointerId(1), Point::new(Px(1.0), Px(1.0))),
            );
            ui.set_focus(Some(select_empty_node));
            ui.dispatch_event(&mut app, &mut services, &key_down(KeyCode::ArrowRight));
            ui.dispatch_event(&mut app, &mut services, &key_up(KeyCode::ArrowRight));

            let select_focus_visible_message = format!(
                "expected the Material3 select focus-visible scene to be stable after animations settle ({label}, {scale})"
            );
            cases.insert(
                "focus_visible_select_empty".to_string(),
                settle_material3_scene_snapshot_v1(
                    &mut app,
                    &mut ui,
                    &mut services,
                    bounds,
                    scale_factor,
                    24,
                    40,
                    &select_focus_visible_message,
                    &render,
                ),
            );

            ui.dispatch_event(
                &mut app,
                &mut services,
                &pointer_move(PointerId(1), select_error_center),
            );

            let select_hover_message = format!(
                "expected the Material3 select hover scene to be stable after animations settle ({label}, {scale})"
            );
            cases.insert(
                "hover_select_error".to_string(),
                settle_material3_scene_snapshot_v1(
                    &mut app,
                    &mut ui,
                    &mut services,
                    bounds,
                    scale_factor,
                    24,
                    40,
                    &select_hover_message,
                    &render,
                ),
            );

            ui.dispatch_event(
                &mut app,
                &mut services,
                &pointer_move(PointerId(1), Point::new(Px(1.0), Px(1.0))),
            );
            ui.set_focus(Some(select_error_node));
            ui.dispatch_event(&mut app, &mut services, &key_down(KeyCode::ArrowRight));
            ui.dispatch_event(&mut app, &mut services, &key_up(KeyCode::ArrowRight));

            let select_error_focus_visible_message = format!(
                "expected the Material3 select error focus-visible scene to be stable after animations settle ({label}, {scale})"
            );
            cases.insert(
                "focus_visible_select_error".to_string(),
                settle_material3_scene_snapshot_v1(
                    &mut app,
                    &mut ui,
                    &mut services,
                    bounds,
                    scale_factor,
                    24,
                    40,
                    &select_error_focus_visible_message,
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
fn material3_headless_fab_suite_goldens_v1() {
    use fret_ui::element::FlexProps;
    use fret_ui_material3::{Fab, FabSize, FabVariant};

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
                Size::new(Px(420.0), Px(240.0)),
            );

            let render =
                |ui: &mut UiTree<TestHost>, app: &mut TestHost, services: &mut dyn UiServices| {
                    fret_ui::declarative::render_root(
                        ui,
                        app,
                        services,
                        window,
                        bounds,
                        "fab_root",
                        |cx| {
                            let mut props = FlexProps::default();
                            props.direction = fret_core::Axis::Vertical;
                            props.gap = Px(16.0);

                            let content = cx.flex(props, |cx| {
                                let row =
                                    |cx: &mut fret_ui::elements::ElementContext<'_, TestHost>,
                                     variant: FabVariant,
                                     id_prefix: &'static str| {
                                        let mut props = FlexProps::default();
                                        props.direction = fret_core::Axis::Horizontal;
                                        props.gap = Px(16.0);
                                        cx.flex(props, move |cx| {
                                            vec![
                                                Fab::new(fret_icons::ids::ui::SEARCH)
                                                    .variant(variant)
                                                    .a11y_label("fab")
                                                    .test_id(format!("{id_prefix}-fab"))
                                                    .into_element(cx),
                                                Fab::new(fret_icons::ids::ui::SEARCH)
                                                    .variant(variant)
                                                    .size(FabSize::Small)
                                                    .a11y_label("fab small")
                                                    .test_id(format!("{id_prefix}-fab-small"))
                                                    .into_element(cx),
                                                Fab::new(fret_icons::ids::ui::SEARCH)
                                                    .variant(variant)
                                                    .size(FabSize::Large)
                                                    .a11y_label("fab large")
                                                    .test_id(format!("{id_prefix}-fab-large"))
                                                    .into_element(cx),
                                                Fab::new(fret_icons::ids::ui::SEARCH)
                                                    .variant(variant)
                                                    .label("Create")
                                                    .test_id(format!("{id_prefix}-extended-fab"))
                                                    .into_element(cx),
                                            ]
                                        })
                                    };

                                vec![
                                    row(cx, FabVariant::Surface, "fab-surface"),
                                    row(cx, FabVariant::Primary, "fab-primary"),
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

            ui.set_focus(None);
            ui.dispatch_event(
                &mut app,
                &mut services,
                &pointer_move(PointerId(1), Point::new(Px(1.0), Px(1.0))),
            );

            let fab_node: NodeId = ui
                .semantics_snapshot()
                .and_then(|snapshot| {
                    snapshot.nodes.iter().find_map(|node| {
                        (node.test_id.as_deref() == Some("fab-surface-fab")).then_some(node.id)
                    })
                })
                .unwrap_or_else(|| {
                    panic!("expected fab-surface-fab in semantics snapshot ({label}, {scale})")
                });

            let bounds_message =
                format!("expected fab-surface bounds in headless suite ({label}, {scale})");
            let fab_bounds = ui
                .debug_node_visual_bounds(fab_node)
                .unwrap_or_else(|| panic!("{bounds_message}"));
            let fab_center = Point::new(
                Px(fab_bounds.origin.x.0 + fab_bounds.size.width.0 * 0.5),
                Px(fab_bounds.origin.y.0 + fab_bounds.size.height.0 * 0.5),
            );

            let mut cases: BTreeMap<String, Material3HeadlessGoldenV1> = BTreeMap::new();

            let idle_message = format!(
                "expected the Material3 fab idle scene to be stable after animations settle ({label}, {scale})"
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
                &pointer_move(PointerId(1), fab_center),
            );
            let hover_message = format!(
                "expected the Material3 fab hover scene to be stable after animations settle ({label}, {scale})"
            );
            cases.insert(
                "hover".to_string(),
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
            ui.set_focus(Some(fab_node));
            ui.dispatch_event(&mut app, &mut services, &key_down(KeyCode::ArrowRight));
            ui.dispatch_event(&mut app, &mut services, &key_up(KeyCode::ArrowRight));

            let focus_visible_message = format!(
                "expected the Material3 fab focus-visible scene to be stable after animations settle ({label}, {scale})"
            );
            cases.insert(
                "focus_visible".to_string(),
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
            write_or_assert_material3_suite_v1(&format!("material3-fab.{scale}.{label}"), &suite);
        }
    }
}

#[test]
fn material3_headless_segmented_button_suite_goldens_v1() {
    use std::collections::BTreeSet;

    use fret_ui::element::FlexProps;
    use fret_ui_material3::{SegmentedButtonItem, SegmentedButtonSet};

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
                Size::new(Px(420.0), Px(260.0)),
            );

            let single_value: Model<Arc<str>> = app.models_mut().insert(Arc::<str>::from("alpha"));
            let multi_value: Model<BTreeSet<Arc<str>>> = app.models_mut().insert(
                [Arc::<str>::from("alpha"), Arc::<str>::from("beta")]
                    .into_iter()
                    .collect(),
            );

            let render =
                |ui: &mut UiTree<TestHost>, app: &mut TestHost, services: &mut dyn UiServices| {
                    fret_ui::declarative::render_root(
                        ui,
                        app,
                        services,
                        window,
                        bounds,
                        "segmented_root",
                        |cx| {
                            let mut props = FlexProps::default();
                            props.direction = fret_core::Axis::Vertical;
                            props.gap = Px(16.0);

                            let content = cx.flex(props, |cx| {
                                vec![
                                    SegmentedButtonSet::single(single_value.clone())
                                        .items(vec![
                                            SegmentedButtonItem::new("alpha", "Alpha")
                                                .test_id("segmented-single-alpha"),
                                            SegmentedButtonItem::new("beta", "Beta")
                                                .test_id("segmented-single-beta"),
                                            SegmentedButtonItem::new("gamma", "Gamma (disabled)")
                                                .disabled(true)
                                                .test_id("segmented-single-gamma"),
                                        ])
                                        .a11y_label("Single segmented buttons")
                                        .test_id("segmented-single")
                                        .into_element(cx),
                                    SegmentedButtonSet::multi(multi_value.clone())
                                        .items(vec![
                                            SegmentedButtonItem::new("alpha", "Alpha")
                                                .icon(fret_icons::ids::ui::SEARCH)
                                                .test_id("segmented-multi-alpha"),
                                            SegmentedButtonItem::new("beta", "Beta")
                                                .icon(fret_icons::ids::ui::SETTINGS)
                                                .test_id("segmented-multi-beta"),
                                            SegmentedButtonItem::new("gamma", "Gamma")
                                                .icon(fret_icons::ids::ui::MORE_HORIZONTAL)
                                                .test_id("segmented-multi-gamma"),
                                        ])
                                        .a11y_label("Multi segmented buttons")
                                        .test_id("segmented-multi")
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

            ui.set_focus(None);
            ui.dispatch_event(
                &mut app,
                &mut services,
                &pointer_move(PointerId(1), Point::new(Px(1.0), Px(1.0))),
            );

            let hover_node: NodeId = ui
                .semantics_snapshot()
                .and_then(|snapshot| {
                    snapshot.nodes.iter().find_map(|node| {
                        (node.test_id.as_deref() == Some("segmented-single-beta"))
                            .then_some(node.id)
                    })
                })
                .unwrap_or_else(|| {
                    panic!(
                        "expected segmented-single-beta in semantics snapshot ({label}, {scale})"
                    )
                });

            let hover_bounds = ui.debug_node_visual_bounds(hover_node).unwrap_or_else(|| {
                panic!("expected segmented-single-beta bounds in headless suite ({label}, {scale})")
            });
            let hover_center = Point::new(
                Px(hover_bounds.origin.x.0 + hover_bounds.size.width.0 * 0.5),
                Px(hover_bounds.origin.y.0 + hover_bounds.size.height.0 * 0.5),
            );

            let focus_node: NodeId = ui
                .semantics_snapshot()
                .and_then(|snapshot| {
                    snapshot.nodes.iter().find_map(|node| {
                        (node.test_id.as_deref() == Some("segmented-single-alpha"))
                            .then_some(node.id)
                    })
                })
                .unwrap_or_else(|| {
                    panic!(
                        "expected segmented-single-alpha in semantics snapshot ({label}, {scale})"
                    )
                });

            let mut cases: BTreeMap<String, Material3HeadlessGoldenV1> = BTreeMap::new();

            let idle_message = format!(
                "expected the Material3 segmented button idle scene to be stable after animations settle ({label}, {scale})"
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
                &pointer_move(PointerId(1), hover_center),
            );
            let hover_message = format!(
                "expected the Material3 segmented button hover scene to be stable after animations settle ({label}, {scale})"
            );
            cases.insert(
                "hover".to_string(),
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
            ui.set_focus(Some(focus_node));
            ui.dispatch_event(&mut app, &mut services, &key_down(KeyCode::ArrowRight));
            ui.dispatch_event(&mut app, &mut services, &key_up(KeyCode::ArrowRight));

            let focus_visible_message = format!(
                "expected the Material3 segmented button focus-visible scene to be stable after animations settle ({label}, {scale})"
            );
            cases.insert(
                "focus_visible".to_string(),
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
                &format!("material3-segmented-button.{scale}.{label}"),
                &suite,
            );
        }
    }
}

#[test]
fn material3_headless_badge_suite_goldens_v1() {
    use fret_core::Corners;
    use fret_ui::element::{ContainerProps, FlexProps, Length};
    use fret_ui_material3::{Badge, BadgePlacement};

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
                Size::new(Px(420.0), Px(200.0)),
            );

            let render = |ui: &mut UiTree<TestHost>,
                          app: &mut TestHost,
                          services: &mut dyn UiServices| {
                fret_ui::declarative::render_root(
                    ui,
                    app,
                    services,
                    window,
                    bounds,
                    "badge_root",
                    |cx| {
                        let theme = Theme::global(&*cx.app).clone();
                        let anchor_color =
                            theme.color_required("md.sys.color.surface-container-low");

                        let anchor = |cx: &mut fret_ui::elements::ElementContext<'_, TestHost>,
                                      size: Px| {
                            let mut props = ContainerProps::default();
                            props.layout.size.width = Length::Px(size);
                            props.layout.size.height = Length::Px(size);
                            props.background = Some(anchor_color);
                            props.corner_radii = Corners::all(Px(8.0));
                            cx.container(props, |_cx| Vec::<AnyElement>::new())
                        };

                        let mut props = FlexProps::default();
                        props.direction = fret_core::Axis::Horizontal;
                        props.gap = Px(24.0);
                        props.align = fret_ui::element::CrossAlign::Center;
                        props.wrap = false;

                        let content = cx.flex(props, |cx| {
                            let small = Px(24.0);
                            vec![
                                Badge::dot()
                                    .navigation_anchor_size(small)
                                    .test_id("badge-dot-nav")
                                    .into_element(cx, |cx| vec![anchor(cx, small)]),
                                Badge::text("9")
                                    .navigation_anchor_size(small)
                                    .test_id("badge-text-nav")
                                    .into_element(cx, |cx| vec![anchor(cx, small)]),
                                Badge::dot()
                                    .placement(BadgePlacement::TopRight)
                                    .test_id("badge-dot-top-right")
                                    .into_element(cx, |cx| vec![anchor(cx, Px(40.0))]),
                                Badge::text("99+")
                                    .placement(BadgePlacement::TopRight)
                                    .test_id("badge-text-top-right")
                                    .into_element(cx, |cx| vec![anchor(cx, Px(40.0))]),
                            ]
                        });

                        vec![with_padding(cx, Px(24.0), content)]
                    },
                )
            };

            let mut cases: BTreeMap<String, Material3HeadlessGoldenV1> = BTreeMap::new();
            let idle_message = format!(
                "expected the Material3 badge scene to be stable after animations settle ({label}, {scale})"
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

            let suite = Material3HeadlessSuiteV1 { cases };
            write_or_assert_material3_suite_v1(&format!("material3-badge.{scale}.{label}"), &suite);
        }
    }
}

#[test]
fn material3_headless_top_app_bar_suite_goldens_v1() {
    use fret_icons::ids;
    use fret_ui::element::ContainerProps;
    use fret_ui_material3::{TopAppBar, TopAppBarAction, TopAppBarVariant};

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
                Size::new(Px(420.0), Px(220.0)),
            );

            let make_actions = |extra: usize| -> Vec<TopAppBarAction> {
                let mut actions = vec![
                    TopAppBarAction::new(ids::ui::SEARCH)
                        .a11y_label("Search")
                        .test_id("top-app-bar-search"),
                    TopAppBarAction::new(ids::ui::MORE_HORIZONTAL)
                        .a11y_label("More actions")
                        .test_id("top-app-bar-more"),
                ];
                if extra >= 1 {
                    actions.push(
                        TopAppBarAction::new(ids::ui::SETTINGS)
                            .a11y_label("Settings")
                            .test_id("top-app-bar-settings"),
                    );
                }
                if extra >= 2 {
                    actions.push(
                        TopAppBarAction::new(ids::ui::PLAY)
                            .a11y_label("Play")
                            .test_id("top-app-bar-play"),
                    );
                }
                actions
            };

            let mut snapshot_case =
                |case_label: &'static str,
                 variant: TopAppBarVariant,
                 scrolled: bool,
                 actions: Vec<TopAppBarAction>| {
                    let render = |ui: &mut UiTree<TestHost>,
                                  app: &mut TestHost,
                                  services: &mut dyn UiServices| {
                        let actions = actions.clone();
                        fret_ui::declarative::render_root(
                            ui,
                            app,
                            services,
                            window,
                            bounds,
                            "top_app_bar_root",
                            move |cx| {
                                let theme = Theme::global(&*cx.app).clone();

                                let mut bg = ContainerProps::default();
                                bg.layout.size.width = fret_ui::element::Length::Fill;
                                bg.layout.size.height = fret_ui::element::Length::Fill;
                                bg.background =
                                    Some(theme.color_required("md.sys.color.background"));

                                let bar = TopAppBar::new(case_label)
                                    .variant(variant)
                                    .scrolled(scrolled)
                                    .navigation_icon(
                                        TopAppBarAction::new(ids::ui::CHEVRON_RIGHT)
                                            .a11y_label("Navigate")
                                            .test_id("top-app-bar-nav"),
                                    )
                                    .actions(actions)
                                    .test_id("top-app-bar");

                                vec![cx.container(bg, move |cx| vec![bar.into_element(cx)])]
                            },
                        )
                    };

                    let stable_message = format!(
                        "expected the Material3 top app bar scene to be stable after animations settle ({label}, {scale}, {case_label})"
                    );
                    settle_material3_scene_snapshot_v1(
                        &mut app,
                        &mut ui,
                        &mut services,
                        bounds,
                        scale_factor,
                        24,
                        40,
                        &stable_message,
                        &render,
                    )
                };

            let mut cases: BTreeMap<String, Material3HeadlessGoldenV1> = BTreeMap::new();
            cases.insert(
                "small.idle".to_string(),
                snapshot_case("Small", TopAppBarVariant::Small, false, make_actions(0)),
            );
            cases.insert(
                "small.scrolled".to_string(),
                snapshot_case(
                    "Small (scrolled)",
                    TopAppBarVariant::Small,
                    true,
                    make_actions(0),
                ),
            );
            cases.insert(
                "small_centered.idle".to_string(),
                snapshot_case(
                    "Small Centered",
                    TopAppBarVariant::SmallCentered,
                    false,
                    make_actions(0),
                ),
            );
            cases.insert(
                "small_centered.scrolled".to_string(),
                snapshot_case(
                    "Small Centered (scrolled)",
                    TopAppBarVariant::SmallCentered,
                    true,
                    make_actions(0),
                ),
            );
            cases.insert(
                "small_centered.wide_actions".to_string(),
                snapshot_case(
                    "Small Centered (wide actions)",
                    TopAppBarVariant::SmallCentered,
                    false,
                    make_actions(2),
                ),
            );
            cases.insert(
                "medium.idle".to_string(),
                snapshot_case("Medium", TopAppBarVariant::Medium, false, make_actions(0)),
            );
            cases.insert(
                "medium.scrolled".to_string(),
                snapshot_case(
                    "Medium (scrolled)",
                    TopAppBarVariant::Medium,
                    true,
                    make_actions(0),
                ),
            );
            cases.insert(
                "large.idle".to_string(),
                snapshot_case("Large", TopAppBarVariant::Large, false, make_actions(0)),
            );
            cases.insert(
                "large.scrolled".to_string(),
                snapshot_case(
                    "Large (scrolled)",
                    TopAppBarVariant::Large,
                    true,
                    make_actions(0),
                ),
            );

            let suite = Material3HeadlessSuiteV1 { cases };
            write_or_assert_material3_suite_v1(
                &format!("material3-top-app-bar.{scale}.{label}"),
                &suite,
            );
        }
    }
}

#[test]
fn material3_headless_navigation_suite_goldens_v1() {
    use fret_icons::ids;
    use fret_ui_material3::{
        Button, ButtonVariant, ModalNavigationDrawer, NavigationBar, NavigationBarItem,
        NavigationDrawer, NavigationDrawerItem, NavigationDrawerVariant, NavigationRail,
        NavigationRailItem,
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
            let mut cases: BTreeMap<String, Material3HeadlessGoldenV1> = BTreeMap::new();

            // NavigationBar: horizontal destinations with badges.
            {
                let mut app = TestHost::default();
                app.set_global(PlatformCapabilities::default());
                apply_material_theme(&mut app, mode, variant);

                let window = AppWindowId::default();
                let mut services = FakeUiServices::default();
                let mut ui: UiTree<TestHost> = UiTree::new();
                ui.set_window(window);

                let bounds = Rect::new(
                    Point::new(Px(0.0), Px(0.0)),
                    Size::new(Px(520.0), Px(220.0)),
                );

                let value: Model<Arc<str>> = app.models_mut().insert(Arc::<str>::from("settings"));

                let render = move |ui: &mut UiTree<TestHost>,
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
                            let bar = NavigationBar::new(value.clone())
                                .a11y_label("Material 3 Navigation Bar")
                                .test_id("headless-material3-navigation-bar")
                                .items(vec![
                                    NavigationBarItem::new("search", "Search", ids::ui::SEARCH)
                                        .badge_dot()
                                        .a11y_label("Destination Search")
                                        .test_id("nav-bar-search"),
                                    NavigationBarItem::new(
                                        "settings",
                                        "Settings",
                                        ids::ui::SETTINGS,
                                    )
                                    .a11y_label("Destination Settings")
                                    .test_id("nav-bar-settings"),
                                    NavigationBarItem::new(
                                        "more",
                                        "More",
                                        ids::ui::MORE_HORIZONTAL,
                                    )
                                    .badge_text("9")
                                    .a11y_label("Destination More")
                                    .test_id("nav-bar-more"),
                                ])
                                .into_element(cx);

                            vec![with_padding(cx, Px(24.0), bar)]
                        },
                    )
                };

                let message = format!(
                    "expected the Material3 navigation bar scene to be stable ({label}, {scale})"
                );
                cases.insert(
                    "bar.selected".to_string(),
                    settle_material3_scene_snapshot_v1(
                        &mut app,
                        &mut ui,
                        &mut services,
                        bounds,
                        scale_factor,
                        8,
                        14,
                        &message,
                        &render,
                    ),
                );
            }

            // NavigationRail: vertical destinations with disabled item.
            {
                let mut app = TestHost::default();
                app.set_global(PlatformCapabilities::default());
                apply_material_theme(&mut app, mode, variant);

                let window = AppWindowId::default();
                let mut services = FakeUiServices::default();
                let mut ui: UiTree<TestHost> = UiTree::new();
                ui.set_window(window);

                let bounds = Rect::new(
                    Point::new(Px(0.0), Px(0.0)),
                    Size::new(Px(300.0), Px(520.0)),
                );

                let value: Model<Arc<str>> = app.models_mut().insert(Arc::<str>::from("play"));

                let render = move |ui: &mut UiTree<TestHost>,
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
                            let rail = NavigationRail::new(value.clone())
                                .a11y_label("Material 3 Navigation Rail")
                                .test_id("headless-material3-navigation-rail")
                                .items(vec![
                                    NavigationRailItem::new("search", "Search", ids::ui::SEARCH)
                                        .badge_dot()
                                        .a11y_label("Destination Search")
                                        .test_id("nav-rail-search"),
                                    NavigationRailItem::new(
                                        "settings",
                                        "Settings",
                                        ids::ui::SETTINGS,
                                    )
                                    .a11y_label("Destination Settings")
                                    .test_id("nav-rail-settings"),
                                    NavigationRailItem::new("play", "Play", ids::ui::PLAY)
                                        .badge_text("99+")
                                        .a11y_label("Destination Play")
                                        .test_id("nav-rail-play"),
                                    NavigationRailItem::new("disabled", "Disabled", ids::ui::SLASH)
                                        .disabled(true)
                                        .a11y_label("Destination Disabled")
                                        .test_id("nav-rail-disabled"),
                                ])
                                .into_element(cx);

                            vec![with_padding(cx, Px(24.0), rail)]
                        },
                    )
                };

                let message = format!(
                    "expected the Material3 navigation rail scene to be stable ({label}, {scale})"
                );
                cases.insert(
                    "rail.selected".to_string(),
                    settle_material3_scene_snapshot_v1(
                        &mut app,
                        &mut ui,
                        &mut services,
                        bounds,
                        scale_factor,
                        8,
                        14,
                        &message,
                        &render,
                    ),
                );
            }

            // NavigationDrawer: pill selection + badges.
            {
                let mut app = TestHost::default();
                app.set_global(PlatformCapabilities::default());
                apply_material_theme(&mut app, mode, variant);

                let window = AppWindowId::default();
                let mut services = FakeUiServices::default();
                let mut ui: UiTree<TestHost> = UiTree::new();
                ui.set_window(window);

                let bounds = Rect::new(
                    Point::new(Px(0.0), Px(0.0)),
                    Size::new(Px(360.0), Px(520.0)),
                );

                let value: Model<Arc<str>> = app.models_mut().insert(Arc::<str>::from("settings"));

                let render = move |ui: &mut UiTree<TestHost>,
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
                            let drawer = NavigationDrawer::new(value.clone())
                                .a11y_label("Material 3 Navigation Drawer")
                                .test_id("headless-material3-navigation-drawer")
                                .items(vec![
                                    NavigationDrawerItem::new("search", "Search", ids::ui::SEARCH)
                                        .a11y_label("Destination Search")
                                        .test_id("nav-drawer-search"),
                                    NavigationDrawerItem::new(
                                        "settings",
                                        "Settings",
                                        ids::ui::SETTINGS,
                                    )
                                    .badge_label("2")
                                    .a11y_label("Destination Settings")
                                    .test_id("nav-drawer-settings"),
                                    NavigationDrawerItem::new("play", "Play", ids::ui::PLAY)
                                        .badge_label("99+")
                                        .a11y_label("Destination Play")
                                        .test_id("nav-drawer-play"),
                                    NavigationDrawerItem::new(
                                        "disabled",
                                        "Disabled",
                                        ids::ui::SLASH,
                                    )
                                    .disabled(true)
                                    .a11y_label("Destination Disabled")
                                    .test_id("nav-drawer-disabled"),
                                ])
                                .into_element(cx);

                            vec![with_padding(cx, Px(24.0), drawer)]
                        },
                    )
                };

                let message = format!(
                    "expected the Material3 navigation drawer scene to be stable ({label}, {scale})"
                );
                cases.insert(
                    "drawer.selected".to_string(),
                    settle_material3_scene_snapshot_v1(
                        &mut app,
                        &mut ui,
                        &mut services,
                        bounds,
                        scale_factor,
                        8,
                        14,
                        &message,
                        &render,
                    ),
                );
            }

            // ModalNavigationDrawer: overlay open (scrim + focus trap surface).
            {
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
                let value: Model<Arc<str>> = app.models_mut().insert(Arc::<str>::from("search"));

                let render = move |ui: &mut UiTree<TestHost>,
                                   app: &mut TestHost,
                                   services: &mut dyn UiServices| {
                    let value = value.clone();
                    fret_ui::declarative::render_root(
                        ui,
                        app,
                        services,
                        window,
                        bounds,
                        "root",
                        |cx| {
                            let panel_value = value.clone();
                            let panel = move |cx: &mut fret_ui::elements::ElementContext<
                                '_,
                                TestHost,
                            >| {
                                NavigationDrawer::new(panel_value.clone())
                                    .variant(NavigationDrawerVariant::Modal)
                                    .a11y_label("Material 3 Modal Navigation Drawer")
                                    .test_id("headless-material3-modal-navigation-drawer-panel")
                                    .items(vec![
                                        NavigationDrawerItem::new(
                                            "search",
                                            "Search",
                                            ids::ui::SEARCH,
                                        )
                                        .a11y_label("Destination Search")
                                        .test_id("nav-modal-drawer-search"),
                                        NavigationDrawerItem::new(
                                            "settings",
                                            "Settings",
                                            ids::ui::SETTINGS,
                                        )
                                        .badge_label("2")
                                        .a11y_label("Destination Settings")
                                        .test_id("nav-modal-drawer-settings"),
                                        NavigationDrawerItem::new("play", "Play", ids::ui::PLAY)
                                            .badge_label("99+")
                                            .a11y_label("Destination Play")
                                            .test_id("nav-modal-drawer-play"),
                                        NavigationDrawerItem::new(
                                            "disabled",
                                            "Disabled",
                                            ids::ui::SLASH,
                                        )
                                        .disabled(true)
                                        .a11y_label("Destination Disabled")
                                        .test_id("nav-modal-drawer-disabled"),
                                    ])
                                    .into_element(cx)
                            };

                            let underlay =
                                move |cx: &mut fret_ui::elements::ElementContext<'_, TestHost>| {
                                    Button::new("Underlay probe")
                                        .variant(ButtonVariant::Outlined)
                                        .test_id("nav-modal-drawer-underlay-probe")
                                        .into_element(cx)
                                };

                            let modal = ModalNavigationDrawer::new(open.clone())
                                .open_duration_ms(Some(1))
                                .close_duration_ms(Some(1))
                                .test_id("headless-material3-modal-navigation-drawer")
                                .into_element(cx, panel, underlay);

                            vec![with_padding(cx, Px(24.0), modal)]
                        },
                    )
                };

                let message = format!(
                    "expected the Material3 modal navigation drawer overlay scene to be stable ({label}, {scale})"
                );
                cases.insert(
                    "modal_drawer.open".to_string(),
                    settle_material3_overlay_scene_snapshot_v1(
                        &mut app,
                        &mut ui,
                        &mut services,
                        window,
                        bounds,
                        scale_factor,
                        4,
                        10,
                        &message,
                        &render,
                    ),
                );
            }

            let suite = Material3HeadlessSuiteV1 { cases };
            write_or_assert_material3_suite_v1(
                &format!("material3-navigation.{scale}.{label}"),
                &suite,
            );
        }
    }
}

#[test]
fn material3_headless_snackbar_suite_goldens_v1() {
    use fret_runtime::CommandId;
    use fret_ui_kit::ToastStore;
    use fret_ui_material3::{Snackbar, SnackbarController, SnackbarDuration, SnackbarHost};

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

            let snapshot_case = |case_label: &'static str, snackbar: Snackbar| {
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

                let store = app.models_mut().insert(ToastStore::default());
                let controller = SnackbarController::new(store.clone());
                {
                    let mut action_host = fret_ui::action::UiActionHostAdapter { app: &mut app };
                    let _id = controller.show(&mut action_host, window, snackbar);
                }

                let render = move |ui: &mut UiTree<TestHost>,
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
                            let host_layer = SnackbarHost::new(store.clone())
                                .max_snackbars(1)
                                .into_element(cx);

                            vec![with_padding(cx, Px(24.0), host_layer)]
                        },
                    )
                };

                let message = format!(
                    "expected the Material3 snackbar overlay scene to be stable ({label}, {scale}, {case_label})"
                );
                settle_material3_overlay_scene_snapshot_v1(
                    &mut app,
                    &mut ui,
                    &mut services,
                    window,
                    bounds,
                    scale_factor,
                    24,
                    40,
                    &message,
                    &render,
                )
            };

            cases.insert(
                "short.single_line".to_string(),
                snapshot_case(
                    "short.single_line",
                    Snackbar::new("Saved")
                        .action("Undo", CommandId::new("material3_snackbar_action"))
                        .duration(SnackbarDuration::Short),
                ),
            );
            cases.insert(
                "long.two_line".to_string(),
                snapshot_case(
                    "long.two_line",
                    Snackbar::new("Update available")
                        .supporting_text("Restart the app to apply the latest changes.")
                        .action("Restart", CommandId::new("material3_snackbar_action"))
                        .duration(SnackbarDuration::Long),
                ),
            );
            cases.insert(
                "indefinite.two_line".to_string(),
                snapshot_case(
                    "indefinite.two_line",
                    Snackbar::new("Connection lost")
                        .supporting_text("Trying to reconnect...")
                        .duration(SnackbarDuration::Indefinite),
                ),
            );

            let suite = Material3HeadlessSuiteV1 { cases };
            write_or_assert_material3_suite_v1(
                &format!("material3-snackbar.{scale}.{label}"),
                &suite,
            );
        }
    }
}

#[test]
fn material3_headless_divider_suite_goldens_v1() {
    use fret_ui::element::{FlexProps, SpacerProps};
    use fret_ui_material3::Divider;

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
                Size::new(Px(300.0), Px(220.0)),
            );

            let render = |ui: &mut UiTree<TestHost>,
                          app: &mut TestHost,
                          services: &mut dyn UiServices| {
                fret_ui::declarative::render_root(ui, app, services, window, bounds, "root", |cx| {
                    let mut props = FlexProps::default();
                    props.direction = fret_core::Axis::Vertical;
                    props.gap = Px(16.0);

                    let content = cx.flex(props, |cx| {
                        let mut row = FlexProps::default();
                        row.direction = fret_core::Axis::Horizontal;
                        row.gap = Px(12.0);
                        row.layout.size.width = fret_ui::element::Length::Px(Px(240.0));
                        row.layout.size.height = fret_ui::element::Length::Px(Px(32.0));

                        vec![
                            Divider::horizontal()
                                .test_id("divider-horizontal")
                                .into_element(cx),
                            cx.flex(row, |cx| {
                                vec![
                                    cx.spacer(SpacerProps::default()),
                                    Divider::vertical()
                                        .test_id("divider-vertical")
                                        .into_element(cx),
                                    cx.spacer(SpacerProps::default()),
                                ]
                            }),
                        ]
                    });

                    vec![with_padding(cx, Px(24.0), content)]
                })
            };

            let mut cases: BTreeMap<String, Material3HeadlessGoldenV1> = BTreeMap::new();
            let idle_message = format!(
                "expected the Material3 divider scene to be stable after animations settle ({label}, {scale})"
            );
            cases.insert(
                "idle".to_string(),
                settle_material3_scene_snapshot_v1(
                    &mut app,
                    &mut ui,
                    &mut services,
                    bounds,
                    scale_factor,
                    12,
                    24,
                    &idle_message,
                    &render,
                ),
            );

            let suite = Material3HeadlessSuiteV1 { cases };
            write_or_assert_material3_suite_v1(
                &format!("material3-divider.{scale}.{label}"),
                &suite,
            );
        }
    }
}

#[test]
fn material3_headless_list_suite_goldens_v1() {
    use fret_ui::element::FlexProps;
    use fret_ui_material3::{List, ListItem};

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
                ("hover_selected", Some("list-beta"), None),
                ("focus_visible_selected", None, Some("list-beta")),
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
                    Size::new(Px(520.0), Px(420.0)),
                );

                let selected = app.models_mut().insert(Arc::<str>::from("beta"));

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

                            let list = List::new(selected.clone())
                                .test_id("list")
                                .items(vec![
                                    ListItem::new("alpha", "Alpha")
                                        .leading_icon(fret_icons::ids::ui::SEARCH)
                                        .test_id("list-alpha"),
                                    ListItem::new("beta", "Beta (selected)")
                                        .leading_icon(fret_icons::ids::ui::SETTINGS)
                                        .trailing_icon(fret_icons::ids::ui::CHEVRON_RIGHT)
                                        .test_id("list-beta"),
                                    ListItem::new("charlie", "Charlie (disabled)")
                                        .leading_icon(fret_icons::ids::ui::SLASH)
                                        .disabled(true)
                                        .test_id("list-charlie"),
                                ])
                                .into_element(cx);

                            let content = cx.flex(props, |_cx| vec![list]);
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
                            "expected list scene to be stable after animations settle ({label}, {scale}, {case_name})"
                        );
                    } else {
                        settled = Some(snapshot);
                    }
                }

                let Some(snapshot) = settled else {
                    panic!("expected a settled list snapshot ({label}, {scale}, {case_name})");
                };
                cases.insert(case_name.to_string(), snapshot);
            }

            let suite = Material3HeadlessSuiteV1 { cases };
            write_or_assert_material3_suite_v1(&format!("material3-list.{scale}.{label}"), &suite);
        }
    }
}

#[test]
fn material3_headless_progress_indicator_suite_goldens_v1() {
    use fret_ui::element::FlexProps;
    use fret_ui_material3::{CircularProgressIndicator, LinearProgressIndicator};

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
                Size::new(Px(420.0), Px(260.0)),
            );

            let progress_0 = app.models_mut().insert(0.0f32);
            let progress_30 = app.models_mut().insert(0.3f32);
            let progress_100 = app.models_mut().insert(1.0f32);

            let render_determinate =
                |ui: &mut UiTree<TestHost>, app: &mut TestHost, services: &mut dyn UiServices| {
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

                            let content = cx.flex(props, |cx| {
                                vec![
                                    LinearProgressIndicator::new(progress_0.clone())
                                        .test_id("linear-0")
                                        .into_element(cx),
                                    LinearProgressIndicator::new(progress_30.clone())
                                        .test_id("linear-30")
                                        .into_element(cx),
                                    LinearProgressIndicator::new(progress_100.clone())
                                        .test_id("linear-100")
                                        .into_element(cx),
                                    {
                                        let mut row = FlexProps::default();
                                        row.direction = fret_core::Axis::Horizontal;
                                        row.gap = Px(16.0);
                                        cx.flex(row, |cx| {
                                            vec![
                                                CircularProgressIndicator::new(progress_0.clone())
                                                    .test_id("circular-0")
                                                    .into_element(cx),
                                                CircularProgressIndicator::new(progress_30.clone())
                                                    .test_id("circular-30")
                                                    .into_element(cx),
                                                CircularProgressIndicator::new(
                                                    progress_100.clone(),
                                                )
                                                .test_id("circular-100")
                                                .into_element(cx),
                                            ]
                                        })
                                    },
                                ]
                            });

                            vec![with_padding(cx, Px(24.0), content)]
                        },
                    )
                };

            let render_indeterminate =
                |ui: &mut UiTree<TestHost>, app: &mut TestHost, services: &mut dyn UiServices| {
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

                            let content = cx.flex(props, |cx| {
                                vec![
                                    LinearProgressIndicator::indeterminate()
                                        .test_id("linear-indeterminate")
                                        .into_element(cx),
                                    CircularProgressIndicator::indeterminate()
                                        .test_id("circular-indeterminate")
                                        .into_element(cx),
                                ]
                            });

                            vec![with_padding(cx, Px(24.0), content)]
                        },
                    )
                };

            let render_indeterminate_four_color =
                |ui: &mut UiTree<TestHost>, app: &mut TestHost, services: &mut dyn UiServices| {
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

                            let content = cx.flex(props, |cx| {
                                vec![
                                    LinearProgressIndicator::indeterminate()
                                        .four_color(true)
                                        .test_id("linear-indeterminate-four-color")
                                        .into_element(cx),
                                    CircularProgressIndicator::indeterminate()
                                        .four_color(true)
                                        .test_id("circular-indeterminate-four-color")
                                        .into_element(cx),
                                ]
                            });

                            vec![with_padding(cx, Px(24.0), content)]
                        },
                    )
                };

            let mut cases: BTreeMap<String, Material3HeadlessGoldenV1> = BTreeMap::new();
            let idle_message = format!(
                "expected the Material3 progress indicator scene to be stable after animations settle ({label}, {scale})"
            );
            cases.insert(
                "idle".to_string(),
                settle_material3_scene_snapshot_v1(
                    &mut app,
                    &mut ui,
                    &mut services,
                    bounds,
                    scale_factor,
                    12,
                    24,
                    &idle_message,
                    &render_determinate,
                ),
            );

            cases.insert(
                "indeterminate.f60".to_string(),
                snapshot_material3_scene_at_frame_v1(
                    &mut app,
                    &mut ui,
                    &mut services,
                    bounds,
                    scale_factor,
                    60,
                    &render_indeterminate,
                ),
            );

            cases.insert(
                "indeterminate.four_color.f60".to_string(),
                snapshot_material3_scene_at_frame_v1(
                    &mut app,
                    &mut ui,
                    &mut services,
                    bounds,
                    scale_factor,
                    60,
                    &render_indeterminate_four_color,
                ),
            );

            let suite = Material3HeadlessSuiteV1 { cases };
            write_or_assert_material3_suite_v1(
                &format!("material3-progress-indicator.{scale}.{label}"),
                &suite,
            );
        }
    }
}

#[test]
fn material3_headless_slider_suite_goldens_v1() {
    use fret_ui::element::FlexProps;
    use fret_ui_material3::{RangeSlider, Slider};

    let cases_to_render = [
        ("idle", None, None),
        ("hover", Some("slider-30"), None),
        ("focus_visible", None, Some("slider-30")),
        ("keyboard_page", None, Some("slider-30")),
        ("rtl_idle", None, None),
        ("rtl_keyboard_arrows", None, Some("slider-30")),
        ("pressed", Some("slider-30"), None),
        ("dragging", Some("slider-30"), None),
        ("with_tick_marks", None, None),
        ("tick_count", None, None),
        ("range_dragging", Some("range-slider-30-70"), None),
        (
            "range_focus_thumb_switch",
            None,
            Some("range-slider-30-70.start"),
        ),
        (
            "range_keyboard_page",
            None,
            Some("range-slider-30-70.start"),
        ),
        (
            "rtl_range_keyboard_arrows",
            None,
            Some("range-slider-30-70.start"),
        ),
    ];

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
            let bounds = Rect::new(
                Point::new(Px(0.0), Px(0.0)),
                Size::new(Px(520.0), Px(320.0)),
            );

            let mut cases: BTreeMap<String, Material3HeadlessGoldenV1> = BTreeMap::new();

            for (case_name, hover_id, focus_id) in cases_to_render {
                let window = AppWindowId::default();
                let mut app = TestHost::default();
                app.set_global(PlatformCapabilities::default());
                if case_name.starts_with("rtl_") {
                    apply_material_theme_rtl(&mut app, mode, variant);
                } else {
                    apply_material_theme(&mut app, mode, variant);
                }

                let mut services = FakeUiServices::default();
                let mut ui: UiTree<TestHost> = UiTree::new();
                ui.set_window(window);

                let value_0 = app.models_mut().insert(0.0f32);
                let value_30 = app.models_mut().insert(0.3f32);
                let value_100 = app.models_mut().insert(1.0f32);
                let range_30_70 = app.models_mut().insert([0.3f32, 0.7f32]);
                let range_10_90 = app.models_mut().insert([0.1f32, 0.9f32]);

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
                            props.gap = Px(24.0);

                            let content = cx.flex(props, |cx| {
                                let slider_step = if case_name == "with_tick_marks" {
                                    0.1
                                } else {
                                    0.01
                                };
                                let slider_with_ticks =
                                    case_name == "with_tick_marks" || case_name == "tick_count";
                                let slider_tick_count = (case_name == "tick_count").then_some(6u16);

                                let build_slider =
                                    |cx: &mut fret_ui::elements::ElementContext<'_, TestHost>,
                                     model: Model<f32>,
                                     test_id: &'static str,
                                     disabled: bool| {
                                        let slider = Slider::new(model)
                                            .range(0.0, 1.0)
                                            .step(slider_step)
                                            .with_tick_marks(slider_with_ticks);
                                        let slider = if let Some(c) = slider_tick_count {
                                            slider.tick_marks_count(c)
                                        } else {
                                            slider
                                        };
                                        let slider = if disabled {
                                            slider.disabled(true)
                                        } else {
                                            slider
                                        };
                                        slider.test_id(test_id).into_element(cx)
                                    };
                                let build_range_slider =
                                    |cx: &mut fret_ui::elements::ElementContext<'_, TestHost>,
                                     model: Model<[f32; 2]>,
                                     test_id: &'static str,
                                     disabled: bool| {
                                        let slider = RangeSlider::new(model)
                                            .range(0.0, 1.0)
                                            .step(slider_step)
                                            .with_tick_marks(slider_with_ticks);
                                        let slider = if let Some(c) = slider_tick_count {
                                            slider.tick_marks_count(c)
                                        } else {
                                            slider
                                        };
                                        let slider = if disabled {
                                            slider.disabled(true)
                                        } else {
                                            slider
                                        };
                                        slider.test_id(test_id).into_element(cx)
                                    };
                                vec![
                                    build_slider(cx, value_0.clone(), "slider-0", false),
                                    build_slider(cx, value_30.clone(), "slider-30", false),
                                    build_slider(cx, value_100.clone(), "slider-100", false),
                                    build_slider(cx, value_30.clone(), "slider-30-disabled", true),
                                    build_range_slider(
                                        cx,
                                        range_30_70.clone(),
                                        "range-slider-30-70",
                                        false,
                                    ),
                                    build_range_slider(
                                        cx,
                                        range_10_90.clone(),
                                        "range-slider-10-90-disabled",
                                        true,
                                    ),
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

                ui.dispatch_event(
                    &mut app,
                    &mut services,
                    &pointer_move(PointerId(1), Point::new(Px(1.0), Px(1.0))),
                );

                if let Some(test_id) = hover_id {
                    let node_id: NodeId = ui
                        .semantics_snapshot()
                        .and_then(|snapshot| {
                            snapshot.nodes.iter().find_map(|node| {
                                (node.test_id.as_deref() == Some(test_id)).then_some(node.id)
                            })
                        })
                        .unwrap_or_else(|| {
                            panic!("expected {test_id} in semantics snapshot ({label}, {scale}, {case_name})")
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

                    if case_name == "pressed" {
                        ui.dispatch_event(
                            &mut app,
                            &mut services,
                            &pointer_down(PointerId(1), hover_at),
                        );
                    }

                    if case_name == "dragging" {
                        ui.dispatch_event(
                            &mut app,
                            &mut services,
                            &pointer_down(PointerId(1), hover_at),
                        );
                        let drag_to = Point::new(
                            Px(node_bounds.origin.x.0 + node_bounds.size.width.0 * 0.8),
                            Px(node_bounds.origin.y.0 + node_bounds.size.height.0 * 0.5),
                        );
                        ui.dispatch_event(
                            &mut app,
                            &mut services,
                            &pointer_move(PointerId(1), drag_to),
                        );
                    }

                    if case_name == "range_dragging" {
                        let start_at = Point::new(
                            Px(node_bounds.origin.x.0 + node_bounds.size.width.0 * 0.85),
                            Px(node_bounds.origin.y.0 + node_bounds.size.height.0 * 0.5),
                        );
                        let drag_to = Point::new(
                            Px(node_bounds.origin.x.0 + node_bounds.size.width.0 * 0.95),
                            Px(node_bounds.origin.y.0 + node_bounds.size.height.0 * 0.5),
                        );
                        ui.dispatch_event(
                            &mut app,
                            &mut services,
                            &pointer_move(PointerId(1), start_at),
                        );
                        ui.dispatch_event(
                            &mut app,
                            &mut services,
                            &pointer_down(PointerId(1), start_at),
                        );
                        ui.dispatch_event(
                            &mut app,
                            &mut services,
                            &pointer_move(PointerId(1), drag_to),
                        );
                    }
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
                            panic!("expected {test_id} in semantics snapshot ({label}, {scale}, {case_name})")
                        });
                    ui.set_focus(Some(node_id));

                    if case_name == "keyboard_page" {
                        ui.dispatch_event(&mut app, &mut services, &key_down(KeyCode::PageUp));
                        ui.dispatch_event(&mut app, &mut services, &key_up(KeyCode::PageUp));
                        let after_page_up =
                            app.models_mut().read(&value_30, |v| *v).ok().unwrap_or(0.0);
                        assert!(
                            (after_page_up - 0.4).abs() <= 1e-6,
                            "expected slider PageUp to increment by a page (case={case_name}, {label}, {scale})"
                        );

                        ui.dispatch_event(&mut app, &mut services, &key_down(KeyCode::PageDown));
                        ui.dispatch_event(&mut app, &mut services, &key_up(KeyCode::PageDown));
                        let after_page_down =
                            app.models_mut().read(&value_30, |v| *v).ok().unwrap_or(0.0);
                        assert!(
                            (after_page_down - 0.3).abs() <= 1e-6,
                            "expected slider PageDown to decrement by a page (case={case_name}, {label}, {scale})"
                        );

                        ui.dispatch_event(&mut app, &mut services, &key_down(KeyCode::Home));
                        ui.dispatch_event(&mut app, &mut services, &key_up(KeyCode::Home));
                        let after_home =
                            app.models_mut().read(&value_30, |v| *v).ok().unwrap_or(0.0);
                        assert!(
                            after_home.abs() <= 1e-6,
                            "expected slider Home to snap to min (case={case_name}, {label}, {scale})"
                        );

                        ui.dispatch_event(&mut app, &mut services, &key_down(KeyCode::End));
                        ui.dispatch_event(&mut app, &mut services, &key_up(KeyCode::End));
                        let after_end =
                            app.models_mut().read(&value_30, |v| *v).ok().unwrap_or(0.0);
                        assert!(
                            (after_end - 1.0).abs() <= 1e-6,
                            "expected slider End to snap to max (case={case_name}, {label}, {scale})"
                        );
                    } else if case_name == "rtl_keyboard_arrows" {
                        ui.dispatch_event(&mut app, &mut services, &key_down(KeyCode::ArrowRight));
                        ui.dispatch_event(&mut app, &mut services, &key_up(KeyCode::ArrowRight));
                        let after_right =
                            app.models_mut().read(&value_30, |v| *v).ok().unwrap_or(0.0);
                        assert!(
                            (after_right - 0.29).abs() <= 1e-6,
                            "expected slider ArrowRight to decrement under RTL (case={case_name}, {label}, {scale})"
                        );

                        ui.dispatch_event(&mut app, &mut services, &key_down(KeyCode::ArrowLeft));
                        ui.dispatch_event(&mut app, &mut services, &key_up(KeyCode::ArrowLeft));
                        let after_left =
                            app.models_mut().read(&value_30, |v| *v).ok().unwrap_or(0.0);
                        assert!(
                            (after_left - 0.30).abs() <= 1e-6,
                            "expected slider ArrowLeft to increment under RTL (case={case_name}, {label}, {scale})"
                        );
                    } else if case_name == "range_focus_thumb_switch" {
                        ui.dispatch_event(&mut app, &mut services, &key_down(KeyCode::ArrowRight));
                        ui.dispatch_event(&mut app, &mut services, &key_up(KeyCode::ArrowRight));
                        ui.dispatch_event(&mut app, &mut services, &key_down(KeyCode::ArrowRight));
                        ui.dispatch_event(&mut app, &mut services, &key_up(KeyCode::ArrowRight));

                        let end_node_id: NodeId = ui
                            .semantics_snapshot()
                            .and_then(|snapshot| {
                                snapshot.nodes.iter().find_map(|node| {
                                    (node.test_id.as_deref()
                                        == Some("range-slider-30-70.end"))
                                    .then_some(node.id)
                                })
                            })
                            .unwrap_or_else(|| {
                                panic!("expected range-slider-30-70.end in semantics snapshot ({label}, {scale}, {case_name})")
                            });
                        ui.set_focus(Some(end_node_id));

                        ui.dispatch_event(&mut app, &mut services, &key_down(KeyCode::ArrowRight));
                        ui.dispatch_event(&mut app, &mut services, &key_up(KeyCode::ArrowRight));
                        ui.dispatch_event(&mut app, &mut services, &key_down(KeyCode::ArrowRight));
                        ui.dispatch_event(&mut app, &mut services, &key_up(KeyCode::ArrowRight));
                    } else if case_name == "range_keyboard_page" {
                        ui.dispatch_event(&mut app, &mut services, &key_down(KeyCode::PageUp));
                        ui.dispatch_event(&mut app, &mut services, &key_up(KeyCode::PageUp));
                        let after_page_up = app
                            .models_mut()
                            .read(&range_30_70, |v| *v)
                            .ok()
                            .unwrap_or([0.0, 0.0]);
                        assert!(
                            (after_page_up[0] - 0.4).abs() <= 1e-6
                                && (after_page_up[1] - 0.7).abs() <= 1e-6,
                            "expected range slider start PageUp to increment start by a page (case={case_name}, {label}, {scale})"
                        );

                        ui.dispatch_event(&mut app, &mut services, &key_down(KeyCode::PageDown));
                        ui.dispatch_event(&mut app, &mut services, &key_up(KeyCode::PageDown));
                        let after_page_down = app
                            .models_mut()
                            .read(&range_30_70, |v| *v)
                            .ok()
                            .unwrap_or([0.0, 0.0]);
                        assert!(
                            (after_page_down[0] - 0.3).abs() <= 1e-6
                                && (after_page_down[1] - 0.7).abs() <= 1e-6,
                            "expected range slider start PageDown to decrement start by a page (case={case_name}, {label}, {scale})"
                        );

                        ui.dispatch_event(&mut app, &mut services, &key_down(KeyCode::Home));
                        ui.dispatch_event(&mut app, &mut services, &key_up(KeyCode::Home));
                        let after_home = app
                            .models_mut()
                            .read(&range_30_70, |v| *v)
                            .ok()
                            .unwrap_or([0.0, 0.0]);
                        assert!(
                            after_home[0].abs() <= 1e-6 && (after_home[1] - 0.7).abs() <= 1e-6,
                            "expected range slider start Home to snap to min (case={case_name}, {label}, {scale})"
                        );

                        let end_node_id: NodeId = ui
                            .semantics_snapshot()
                            .and_then(|snapshot| {
                                snapshot.nodes.iter().find_map(|node| {
                                    (node.test_id.as_deref()
                                        == Some("range-slider-30-70.end"))
                                    .then_some(node.id)
                                })
                            })
                            .unwrap_or_else(|| {
                                panic!("expected range-slider-30-70.end in semantics snapshot ({label}, {scale}, {case_name})")
                            });
                        ui.set_focus(Some(end_node_id));

                        ui.dispatch_event(&mut app, &mut services, &key_down(KeyCode::PageDown));
                        ui.dispatch_event(&mut app, &mut services, &key_up(KeyCode::PageDown));
                        let after_end_page_down = app
                            .models_mut()
                            .read(&range_30_70, |v| *v)
                            .ok()
                            .unwrap_or([0.0, 0.0]);
                        assert!(
                            after_end_page_down[0].abs() <= 1e-6
                                && (after_end_page_down[1] - 0.6).abs() <= 1e-6,
                            "expected range slider end PageDown to decrement end by a page (case={case_name}, {label}, {scale})"
                        );

                        ui.dispatch_event(&mut app, &mut services, &key_down(KeyCode::PageUp));
                        ui.dispatch_event(&mut app, &mut services, &key_up(KeyCode::PageUp));
                        let after_end_page_up = app
                            .models_mut()
                            .read(&range_30_70, |v| *v)
                            .ok()
                            .unwrap_or([0.0, 0.0]);
                        assert!(
                            after_end_page_up[0].abs() <= 1e-6
                                && (after_end_page_up[1] - 0.7).abs() <= 1e-6,
                            "expected range slider end PageUp to increment end by a page (case={case_name}, {label}, {scale})"
                        );

                        ui.dispatch_event(&mut app, &mut services, &key_down(KeyCode::Home));
                        ui.dispatch_event(&mut app, &mut services, &key_up(KeyCode::Home));
                        let after_end_home = app
                            .models_mut()
                            .read(&range_30_70, |v| *v)
                            .ok()
                            .unwrap_or([0.0, 0.0]);
                        assert!(
                            after_end_home[0].abs() <= 1e-6 && after_end_home[1].abs() <= 1e-6,
                            "expected range slider end Home to snap to start value (case={case_name}, {label}, {scale})"
                        );

                        ui.dispatch_event(&mut app, &mut services, &key_down(KeyCode::End));
                        ui.dispatch_event(&mut app, &mut services, &key_up(KeyCode::End));
                        let after_end_end = app
                            .models_mut()
                            .read(&range_30_70, |v| *v)
                            .ok()
                            .unwrap_or([0.0, 0.0]);
                        assert!(
                            after_end_end[0].abs() <= 1e-6
                                && (after_end_end[1] - 1.0).abs() <= 1e-6,
                            "expected range slider end End to snap to max (case={case_name}, {label}, {scale})"
                        );
                    } else if case_name == "rtl_range_keyboard_arrows" {
                        ui.dispatch_event(&mut app, &mut services, &key_down(KeyCode::ArrowRight));
                        ui.dispatch_event(&mut app, &mut services, &key_up(KeyCode::ArrowRight));
                        let after_right = app
                            .models_mut()
                            .read(&range_30_70, |v| *v)
                            .ok()
                            .unwrap_or([0.0, 0.0]);
                        assert!(
                            (after_right[0] - 0.29).abs() <= 1e-6
                                && (after_right[1] - 0.7).abs() <= 1e-6,
                            "expected range slider start ArrowRight to decrement under RTL (case={case_name}, {label}, {scale})"
                        );

                        ui.dispatch_event(&mut app, &mut services, &key_down(KeyCode::ArrowLeft));
                        ui.dispatch_event(&mut app, &mut services, &key_up(KeyCode::ArrowLeft));
                        let after_left = app
                            .models_mut()
                            .read(&range_30_70, |v| *v)
                            .ok()
                            .unwrap_or([0.0, 0.0]);
                        assert!(
                            (after_left[0] - 0.30).abs() <= 1e-6
                                && (after_left[1] - 0.7).abs() <= 1e-6,
                            "expected range slider start ArrowLeft to increment under RTL (case={case_name}, {label}, {scale})"
                        );
                    } else {
                        ui.dispatch_event(&mut app, &mut services, &key_down(KeyCode::ArrowRight));
                        ui.dispatch_event(&mut app, &mut services, &key_up(KeyCode::ArrowRight));
                        ui.dispatch_event(&mut app, &mut services, &key_down(KeyCode::ArrowLeft));
                        ui.dispatch_event(&mut app, &mut services, &key_up(KeyCode::ArrowLeft));
                    }
                }

                let message = format!(
                    "expected the Material3 slider scene to be stable after animations settle ({label}, {scale}, {case_name})"
                );
                cases.insert(
                    case_name.to_string(),
                    settle_material3_scene_snapshot_v1(
                        &mut app,
                        &mut ui,
                        &mut services,
                        bounds,
                        scale_factor,
                        7,
                        9,
                        &message,
                        &render,
                    ),
                );
            }

            let suite = Material3HeadlessSuiteV1 { cases };
            write_or_assert_material3_suite_v1(
                &format!("material3-slider.{scale}.{label}"),
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
        Button, DropdownMenu, PlainTooltip, RichTooltip, Select, SelectItem, TooltipProvider,
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

            let rich_both_open_snapshot = {
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
                                    let tooltip_trigger = Button::new("Rich tooltip")
                                        .test_id("tooltip-trigger")
                                        .into_element(cx);
                                    let tooltip =
                                        RichTooltip::new(tooltip_trigger, "Supporting text")
                                            .title("Title")
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
                                                        MenuItem::new("A")
                                                            .test_id("dropdown-item-a"),
                                                    ),
                                                    MenuEntry::Item(
                                                        MenuItem::new("B")
                                                            .test_id("dropdown-item-b"),
                                                    ),
                                                    MenuEntry::Item(
                                                        MenuItem::new("C")
                                                            .test_id("dropdown-item-c"),
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
                    Px(tooltip_trigger_bounds.origin.x.0
                        + tooltip_trigger_bounds.size.width.0 * 0.5),
                    Px(tooltip_trigger_bounds.origin.y.0
                        + tooltip_trigger_bounds.size.height.0 * 0.5),
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
                    "expected both rich tooltip and menu overlays to be open ({label}, {scale})"
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
                            "expected the Material3 rich tooltip overlays scene to be stable after animations settle ({label}, {scale})"
                        );
                    } else {
                        settled = Some(snapshot);
                    }
                }

                settled.unwrap_or_else(|| {
                    panic!("expected a settled rich tooltip overlays snapshot ({label}, {scale})")
                })
            };

            let (
                select_open_snapshot,
                select_open_trigger_snapshot,
                select_open_hover_selected_snapshot,
            ) = {
                let mut app = TestHost::default();
                app.set_global(PlatformCapabilities::default());
                apply_material_theme(&mut app, mode, variant);

                let window = AppWindowId::default();
                let mut services = FakeUiServices::default();
                let mut ui: UiTree<TestHost> = UiTree::new();
                ui.set_window(window);

                let selected: Model<Option<Arc<str>>> =
                    app.models_mut().insert(Some(Arc::<str>::from("beta")));
                let error_selected: Model<Option<Arc<str>>> = app.models_mut().insert(None);

                let items: Arc<[SelectItem]> = vec![
                    SelectItem::new("alpha", "Alpha")
                        .leading_icon(fret_icons::ids::ui::SEARCH)
                        .trailing_icon(fret_icons::ids::ui::CHEVRON_RIGHT)
                        .test_id("select-item-alpha"),
                    SelectItem::new("beta", "Beta")
                        .leading_icon(fret_icons::ids::ui::SETTINGS)
                        .trailing_icon(fret_icons::ids::ui::CHEVRON_RIGHT)
                        .test_id("select-item-beta"),
                    SelectItem::new("charlie", "Charlie (disabled)")
                        .disabled(true)
                        .leading_icon(fret_icons::ids::ui::SEARCH)
                        .test_id("select-item-charlie-disabled"),
                ]
                .into();

                let render = move |ui: &mut UiTree<TestHost>,
                                   app: &mut TestHost,
                                   services: &mut dyn UiServices| {
                    let selected = selected.clone();
                    let error_selected = error_selected.clone();
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
                                .leading_icon(fret_icons::ids::ui::SEARCH)
                                .label("Label")
                                .supporting_text("Supporting text")
                                .a11y_label("select")
                                .placeholder("Pick one")
                                .items(items.clone())
                                .test_id("material3-select-trigger")
                                .into_element(cx);

                            let select_error = Select::new(error_selected)
                                .leading_icon(fret_icons::ids::ui::SETTINGS)
                                .label("Label")
                                .supporting_text("Error supporting text")
                                .a11y_label("select error")
                                .placeholder("Pick one")
                                .items(items.clone())
                                .error(true)
                                .test_id("material3-select-trigger-error")
                                .into_element(cx);

                            vec![cx.flex(props, move |_cx| vec![select, select_error])]
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

                let select_open_message = format!(
                    "expected the Material3 select overlay scene to be stable after animations settle ({label}, {scale})"
                );
                let select_open_snapshot = settle_material3_overlay_scene_snapshot_v1(
                    &mut app,
                    &mut ui,
                    &mut services,
                    window,
                    bounds,
                    scale_factor,
                    44,
                    80,
                    &select_open_message,
                    &render,
                );

                let select_open_trigger_message = format!(
                    "expected the Material3 select trigger to be stable in open state ({label}, {scale})"
                );
                let select_open_trigger_snapshot = settle_material3_scene_snapshot_v1(
                    &mut app,
                    &mut ui,
                    &mut services,
                    bounds,
                    scale_factor,
                    24,
                    40,
                    &select_open_trigger_message,
                    &render,
                );

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

                let selected_item_node: NodeId = ui
                    .semantics_snapshot()
                    .and_then(|snapshot| {
                        snapshot.nodes.iter().find_map(|node| {
                            (node.test_id.as_deref() == Some("select-item-beta")).then_some(node.id)
                        })
                    })
                    .unwrap_or_else(|| {
                        panic!("expected select-item-beta in semantics snapshot ({label}, {scale})")
                    });
                let selected_item_bounds = ui
                    .debug_node_visual_bounds(selected_item_node)
                    .unwrap_or_else(|| {
                        panic!("expected select-item-beta bounds ({label}, {scale})")
                    });
                let hover_at = Point::new(
                    Px(selected_item_bounds.origin.x.0 + selected_item_bounds.size.width.0 * 0.5),
                    Px(selected_item_bounds.origin.y.0 + selected_item_bounds.size.height.0 * 0.5),
                );

                ui.dispatch_event(
                    &mut app,
                    &mut services,
                    &pointer_move(PointerId(1), hover_at),
                );

                let select_hover_message = format!(
                    "expected the Material3 select overlay hover-selected scene to be stable after animations settle ({label}, {scale})"
                );
                let select_open_hover_selected_snapshot =
                    settle_material3_overlay_scene_snapshot_v1(
                        &mut app,
                        &mut ui,
                        &mut services,
                        window,
                        bounds,
                        scale_factor,
                        44,
                        80,
                        &select_hover_message,
                        &render,
                    );

                (
                    select_open_snapshot,
                    select_open_trigger_snapshot,
                    select_open_hover_selected_snapshot,
                )
            };

            let mut cases: BTreeMap<String, Material3HeadlessGoldenV1> = BTreeMap::new();
            cases.insert("both_open".to_string(), both_open_snapshot);
            cases.insert("rich_both_open".to_string(), rich_both_open_snapshot);
            cases.insert("select_open".to_string(), select_open_snapshot);
            cases.insert(
                "select_open_trigger".to_string(),
                select_open_trigger_snapshot,
            );
            cases.insert(
                "select_open_hover_selected".to_string(),
                select_open_hover_selected_snapshot,
            );
            let suite = Material3HeadlessSuiteV1 { cases };

            write_or_assert_material3_suite_v1(
                &format!("material3-overlays.{scale}.{label}"),
                &suite,
            );
        }
    }
}

#[test]
fn material3_autocomplete_semantics_v1() {
    use fret_core::SemanticsRole;
    use fret_ui_kit::{OverlayController, OverlayStackEntryKind};
    use fret_ui_material3::{Autocomplete, AutocompleteItem};

    let mut app = TestHost::default();
    app.set_global(PlatformCapabilities::default());
    apply_material_theme(&mut app, SchemeMode::Light, DynamicVariant::TonalSpot);

    let window = AppWindowId::default();
    let mut services = FakeUiServices::default();
    let mut ui: UiTree<TestHost> = UiTree::new();
    ui.set_window(window);

    let bounds = Rect::new(
        Point::new(Px(0.0), Px(0.0)),
        Size::new(Px(860.0), Px(520.0)),
    );

    let model = app.models_mut().insert(String::new());
    let selected_value = app
        .models_mut()
        .insert(Some(Arc::<str>::from("beta")) as Option<Arc<str>>);
    let items: Arc<[AutocompleteItem]> = Arc::from(vec![
        AutocompleteItem::new("alpha", "Alpha"),
        AutocompleteItem::new("beta", "Beta"),
        AutocompleteItem::new("gamma", "Gamma"),
    ]);

    let render =
        move |ui: &mut UiTree<TestHost>, app: &mut TestHost, services: &mut dyn UiServices| {
            fret_ui::declarative::render_root(ui, app, services, window, bounds, "root", |cx| {
                let ac = Autocomplete::new(model.clone())
                    .selected_value(selected_value.clone())
                    .items(items.clone())
                    .a11y_label("autocomplete")
                    .test_id("material3-autocomplete")
                    .into_element(cx);
                vec![with_padding(cx, Px(24.0), ac)]
            })
        };

    // Frame 1: build stable input id + bounds.
    run_overlay_frame_scaled(
        &mut ui,
        &mut app,
        &mut services,
        window,
        bounds,
        1.0,
        true,
        |ui, app, services| render(ui, app, services),
    );

    let input_node: NodeId = ui
        .semantics_snapshot()
        .and_then(|snapshot| {
            snapshot.nodes.iter().find_map(|node| {
                (node.test_id.as_deref() == Some("material3-autocomplete")).then_some(node.id)
            })
        })
        .expect("expected material3-autocomplete input node in semantics snapshot");

    ui.set_focus(Some(input_node));

    // Frame 2: focus visible to the widget.
    run_overlay_frame_scaled(
        &mut ui,
        &mut app,
        &mut services,
        window,
        bounds,
        1.0,
        false,
        |ui, app, services| render(ui, app, services),
    );

    // Open via keyboard.
    ui.dispatch_event(&mut app, &mut services, &key_down(KeyCode::ArrowDown));
    ui.dispatch_event(&mut app, &mut services, &key_up(KeyCode::ArrowDown));

    // Frame 3/4: overlay created, then relationships stabilize (controls/active-descendant).
    run_overlay_frame_scaled(
        &mut ui,
        &mut app,
        &mut services,
        window,
        bounds,
        1.0,
        false,
        |ui, app, services| render(ui, app, services),
    );
    run_overlay_frame_scaled(
        &mut ui,
        &mut app,
        &mut services,
        window,
        bounds,
        1.0,
        true,
        |ui, app, services| render(ui, app, services),
    );

    let stack = OverlayController::stack_snapshot_for_window(&ui, &mut app, window);
    assert!(
        stack.stack.iter().any(|entry| {
            entry.kind == OverlayStackEntryKind::Popover && entry.open && entry.visible
        }),
        "expected autocomplete popover overlay to be open after ArrowDown"
    );

    let snap = ui.semantics_snapshot().expect("semantics snapshot");
    let input = snap
        .nodes
        .iter()
        .find(|n| n.test_id.as_deref() == Some("material3-autocomplete"))
        .expect("combobox input node");
    assert_eq!(input.role, SemanticsRole::ComboBox);
    assert!(
        input.flags.expanded,
        "combobox input should report expanded=true while open"
    );

    let list = snap
        .nodes
        .iter()
        .find(|n| n.test_id.as_deref() == Some("material3-autocomplete-listbox"))
        .expect("listbox node");
    assert!(
        input.controls.iter().any(|id| *id == list.id),
        "combobox input should control the listbox"
    );
    assert!(
        list.labelled_by.iter().any(|id| *id == input.id),
        "listbox should be labelled by the combobox input"
    );

    let active = input
        .active_descendant
        .expect("active_descendant should be set");
    let active_node = snap
        .nodes
        .iter()
        .find(|n| n.id == active)
        .expect("active_descendant should reference a node in the snapshot");
    assert_eq!(active_node.role, SemanticsRole::ListBoxOption);

    let beta = snap
        .nodes
        .iter()
        .find(|n| n.test_id.as_deref() == Some("material3-autocomplete-option-beta"))
        .expect("expected beta option node");
    assert!(beta.flags.selected, "expected beta to be marked selected");

    // Typing still works while the overlay is open.
    ui.set_focus(Some(input.id));
    ui.dispatch_event(
        &mut app,
        &mut services,
        &fret_core::Event::TextInput("a".to_string()),
    );
    run_overlay_frame_scaled(
        &mut ui,
        &mut app,
        &mut services,
        window,
        bounds,
        1.0,
        true,
        |ui, app, services| render(ui, app, services),
    );
    let snap = ui.semantics_snapshot().expect("semantics snapshot");
    let input = snap
        .nodes
        .iter()
        .find(|n| n.test_id.as_deref() == Some("material3-autocomplete"))
        .expect("combobox input node after typing");
    assert_eq!(input.value.as_deref(), Some("a"));
}

#[test]
fn material3_autocomplete_filters_items_by_query_v1() {
    use fret_ui_kit::{OverlayController, OverlayStackEntryKind};
    use fret_ui_material3::{Autocomplete, AutocompleteItem};

    let mut app = TestHost::default();
    app.set_global(PlatformCapabilities::default());
    apply_material_theme(&mut app, SchemeMode::Light, DynamicVariant::TonalSpot);

    let window = AppWindowId::default();
    let mut services = FakeUiServices::default();
    let mut ui: UiTree<TestHost> = UiTree::new();
    ui.set_window(window);

    let bounds = Rect::new(
        Point::new(Px(0.0), Px(0.0)),
        Size::new(Px(860.0), Px(520.0)),
    );

    let model = app.models_mut().insert(String::new());
    let items: Arc<[AutocompleteItem]> = Arc::from(vec![
        AutocompleteItem::new("alpha", "Alpha"),
        AutocompleteItem::new("beta", "Beta"),
        AutocompleteItem::new("gamma", "Gamma"),
    ]);

    let render =
        move |ui: &mut UiTree<TestHost>, app: &mut TestHost, services: &mut dyn UiServices| {
            fret_ui::declarative::render_root(ui, app, services, window, bounds, "root", |cx| {
                let ac = Autocomplete::new(model.clone())
                    .items(items.clone())
                    .a11y_label("autocomplete")
                    .test_id("material3-autocomplete")
                    .into_element(cx);
                vec![with_padding(cx, Px(24.0), ac)]
            })
        };

    run_overlay_frame_scaled(
        &mut ui,
        &mut app,
        &mut services,
        window,
        bounds,
        1.0,
        true,
        |ui, app, services| render(ui, app, services),
    );

    let input_node: NodeId = ui
        .semantics_snapshot()
        .and_then(|snapshot| {
            snapshot.nodes.iter().find_map(|node| {
                (node.test_id.as_deref() == Some("material3-autocomplete")).then_some(node.id)
            })
        })
        .expect("expected material3-autocomplete input node in semantics snapshot");
    ui.set_focus(Some(input_node));

    run_overlay_frame_scaled(
        &mut ui,
        &mut app,
        &mut services,
        window,
        bounds,
        1.0,
        false,
        |ui, app, services| render(ui, app, services),
    );

    ui.dispatch_event(
        &mut app,
        &mut services,
        &fret_core::Event::TextInput("ga".to_string()),
    );
    run_overlay_frame_scaled(
        &mut ui,
        &mut app,
        &mut services,
        window,
        bounds,
        1.0,
        true,
        |ui, app, services| render(ui, app, services),
    );

    let stack = OverlayController::stack_snapshot_for_window(&ui, &mut app, window);
    assert!(
        stack.stack.iter().any(|entry| {
            entry.kind == OverlayStackEntryKind::Popover && entry.open && entry.visible
        }),
        "expected autocomplete popover overlay to be open after typing"
    );

    let snap = ui.semantics_snapshot().expect("semantics snapshot");
    assert!(
        snap.nodes
            .iter()
            .any(|n| { n.test_id.as_deref() == Some("material3-autocomplete-option-gamma") }),
        "expected gamma option after typing 'ga'"
    );
    assert!(
        !snap
            .nodes
            .iter()
            .any(|n| { n.test_id.as_deref() == Some("material3-autocomplete-option-alpha") }),
        "expected alpha option to be filtered out after typing 'ga'"
    );
}

#[test]
fn material3_autocomplete_enter_commits_and_does_not_reopen_v1() {
    use fret_ui_kit::{OverlayController, OverlayStackEntryKind};
    use fret_ui_material3::{Autocomplete, AutocompleteItem};

    let mut app = TestHost::default();
    app.set_global(PlatformCapabilities::default());
    apply_material_theme(&mut app, SchemeMode::Light, DynamicVariant::TonalSpot);

    let window = AppWindowId::default();
    let mut services = FakeUiServices::default();
    let mut ui: UiTree<TestHost> = UiTree::new();
    ui.set_window(window);

    let bounds = Rect::new(
        Point::new(Px(0.0), Px(0.0)),
        Size::new(Px(860.0), Px(520.0)),
    );

    let model = app.models_mut().insert(String::new());
    let selected_value = app.models_mut().insert(None::<Arc<str>>);
    let selected_value_for_render = selected_value.clone();
    let items: Arc<[AutocompleteItem]> = Arc::from(vec![
        AutocompleteItem::new("alpha", "Alpha"),
        AutocompleteItem::new("beta", "Beta"),
        AutocompleteItem::new("gamma", "Gamma"),
    ]);

    let render =
        move |ui: &mut UiTree<TestHost>, app: &mut TestHost, services: &mut dyn UiServices| {
            fret_ui::declarative::render_root(ui, app, services, window, bounds, "root", |cx| {
                let ac = Autocomplete::new(model.clone())
                    .selected_value(selected_value_for_render.clone())
                    .items(items.clone())
                    .a11y_label("autocomplete")
                    .test_id("material3-autocomplete")
                    .into_element(cx);
                vec![with_padding(cx, Px(24.0), ac)]
            })
        };

    run_overlay_frame_scaled(
        &mut ui,
        &mut app,
        &mut services,
        window,
        bounds,
        1.0,
        true,
        |ui, app, services| render(ui, app, services),
    );

    let input_node: NodeId = ui
        .semantics_snapshot()
        .and_then(|snapshot| {
            snapshot.nodes.iter().find_map(|node| {
                (node.test_id.as_deref() == Some("material3-autocomplete")).then_some(node.id)
            })
        })
        .expect("expected material3-autocomplete input node in semantics snapshot");
    ui.set_focus(Some(input_node));

    run_overlay_frame_scaled(
        &mut ui,
        &mut app,
        &mut services,
        window,
        bounds,
        1.0,
        false,
        |ui, app, services| render(ui, app, services),
    );

    ui.dispatch_event(&mut app, &mut services, &key_down(KeyCode::ArrowDown));
    ui.dispatch_event(&mut app, &mut services, &key_up(KeyCode::ArrowDown));
    run_overlay_frame_scaled(
        &mut ui,
        &mut app,
        &mut services,
        window,
        bounds,
        1.0,
        true,
        |ui, app, services| render(ui, app, services),
    );

    let stack = OverlayController::stack_snapshot_for_window(&ui, &mut app, window);
    assert!(
        stack.stack.iter().any(|entry| {
            entry.kind == OverlayStackEntryKind::Popover && entry.open && entry.visible
        }),
        "expected autocomplete popover overlay to be open after ArrowDown"
    );

    ui.dispatch_event(&mut app, &mut services, &key_down(KeyCode::Enter));
    ui.dispatch_event(&mut app, &mut services, &key_up(KeyCode::Enter));
    run_overlay_frame_scaled(
        &mut ui,
        &mut app,
        &mut services,
        window,
        bounds,
        1.0,
        true,
        |ui, app, services| render(ui, app, services),
    );
    run_overlay_frame_scaled(
        &mut ui,
        &mut app,
        &mut services,
        window,
        bounds,
        1.0,
        true,
        |ui, app, services| render(ui, app, services),
    );

    let stack = OverlayController::stack_snapshot_for_window(&ui, &mut app, window);
    assert!(
        !stack.stack.iter().any(|entry| {
            entry.kind == OverlayStackEntryKind::Popover && entry.open && entry.visible
        }),
        "expected autocomplete popover overlay to remain closed after Enter commit"
    );

    let snap = ui.semantics_snapshot().expect("semantics snapshot");
    let input = snap
        .nodes
        .iter()
        .find(|n| n.test_id.as_deref() == Some("material3-autocomplete"))
        .expect("combobox input node after Enter");
    assert_eq!(input.value.as_deref(), Some("Alpha"));

    let selected = app.models_mut().get_cloned(&selected_value).unwrap_or(None);
    assert_eq!(
        selected.as_deref(),
        Some("alpha"),
        "expected selected_value model to be committed on Enter"
    );
}

#[test]
fn material3_exposed_dropdown_reverts_query_to_committed_selection_on_blur_v1() {
    use fret_ui::element::{FlexProps, Length};
    use fret_ui_material3::{AutocompleteItem, ExposedDropdown, TextField, TextFieldVariant};

    let mut app = TestHost::default();
    app.set_global(PlatformCapabilities::default());
    apply_material_theme(&mut app, SchemeMode::Light, DynamicVariant::TonalSpot);

    let window = AppWindowId::default();
    let mut services = FakeUiServices::default();
    let mut ui: UiTree<TestHost> = UiTree::new();
    ui.set_window(window);

    let bounds = Rect::new(
        Point::new(Px(0.0), Px(0.0)),
        Size::new(Px(860.0), Px(520.0)),
    );

    let selected_value = app
        .models_mut()
        .insert(Some(Arc::<str>::from("beta")) as Option<Arc<str>>);
    let query = app.models_mut().insert(String::new());
    let query_for_render = query.clone();
    let other = app.models_mut().insert(String::new());

    let items: Arc<[AutocompleteItem]> = Arc::from(vec![
        AutocompleteItem::new("alpha", "Alpha"),
        AutocompleteItem::new("beta", "Beta"),
        AutocompleteItem::new("gamma", "Gamma"),
    ]);

    let render =
        move |ui: &mut UiTree<TestHost>, app: &mut TestHost, services: &mut dyn UiServices| {
            fret_ui::declarative::render_root(ui, app, services, window, bounds, "root", |cx| {
                let exposed = ExposedDropdown::new(selected_value.clone())
                    .query(query_for_render.clone())
                    .items(items.clone())
                    .a11y_label("exposed dropdown")
                    .test_id("material3-exposed-dropdown")
                    .into_element(cx);

                let other = TextField::new(other.clone())
                    .variant(TextFieldVariant::Outlined)
                    .label("Other")
                    .test_id("other-field")
                    .into_element(cx);

                let mut column = FlexProps::default();
                column.direction = fret_core::Axis::Vertical;
                column.gap = Px(24.0);
                column.layout.size.width = Length::Fill;

                let content = cx.flex(column, |_cx| vec![exposed, other]);
                vec![with_padding(cx, Px(24.0), content)]
            })
        };

    run_overlay_frame_scaled(
        &mut ui,
        &mut app,
        &mut services,
        window,
        bounds,
        1.0,
        true,
        |ui, app, services| render(ui, app, services),
    );
    run_overlay_frame_scaled(
        &mut ui,
        &mut app,
        &mut services,
        window,
        bounds,
        1.0,
        true,
        |ui, app, services| render(ui, app, services),
    );

    assert_eq!(
        app.models_mut().get_cloned(&query).unwrap_or_default(),
        "Beta",
        "expected query to synchronize from the committed selection while blurred"
    );

    let (input_node, other_node): (NodeId, NodeId) = ui
        .semantics_snapshot()
        .and_then(|snapshot| {
            let input = snapshot.nodes.iter().find_map(|node| {
                (node.test_id.as_deref() == Some("material3-exposed-dropdown")).then_some(node.id)
            })?;
            let other = snapshot.nodes.iter().find_map(|node| {
                (node.test_id.as_deref() == Some("other-field")).then_some(node.id)
            })?;
            Some((input, other))
        })
        .expect("expected input and other nodes in semantics snapshot");

    ui.set_focus(Some(input_node));
    run_overlay_frame_scaled(
        &mut ui,
        &mut app,
        &mut services,
        window,
        bounds,
        1.0,
        true,
        |ui, app, services| render(ui, app, services),
    );

    let _ = app.models_mut().update(&query, |v| *v = "ga".to_string());
    run_overlay_frame_scaled(
        &mut ui,
        &mut app,
        &mut services,
        window,
        bounds,
        1.0,
        true,
        |ui, app, services| render(ui, app, services),
    );
    assert_eq!(
        app.models_mut().get_cloned(&query).unwrap_or_default(),
        "ga",
        "expected query to remain editable while focused"
    );

    ui.set_focus(Some(other_node));
    run_overlay_frame_scaled(
        &mut ui,
        &mut app,
        &mut services,
        window,
        bounds,
        1.0,
        true,
        |ui, app, services| render(ui, app, services),
    );

    assert_eq!(
        app.models_mut().get_cloned(&query).unwrap_or_default(),
        "Beta",
        "expected query to revert to the committed selection label on blur"
    );

    let snap = ui.semantics_snapshot().expect("semantics snapshot");
    let input = snap
        .nodes
        .iter()
        .find(|n| n.test_id.as_deref() == Some("material3-exposed-dropdown"))
        .expect("combobox input node after blur");
    assert_eq!(input.value.as_deref(), Some("Beta"));
}

#[test]
fn material3_exposed_dropdown_trailing_icon_toggles_overlay_v1() {
    use fret_ui_kit::{OverlayController, OverlayStackEntryKind};
    use fret_ui_material3::{AutocompleteItem, ExposedDropdown};

    let mut app = TestHost::default();
    app.set_global(PlatformCapabilities::default());
    apply_material_theme(&mut app, SchemeMode::Light, DynamicVariant::TonalSpot);

    let window = AppWindowId::default();
    let mut services = FakeUiServices::default();
    let mut ui: UiTree<TestHost> = UiTree::new();
    ui.set_window(window);

    let bounds = Rect::new(
        Point::new(Px(0.0), Px(0.0)),
        Size::new(Px(860.0), Px(520.0)),
    );

    let selected_value = app.models_mut().insert(None::<Arc<str>>);
    let query = app.models_mut().insert(String::new());
    let query_for_render = query.clone();

    let items: Arc<[AutocompleteItem]> = Arc::from(vec![
        AutocompleteItem::new("alpha", "Alpha"),
        AutocompleteItem::new("beta", "Beta"),
        AutocompleteItem::new("gamma", "Gamma"),
    ]);

    let render =
        move |ui: &mut UiTree<TestHost>, app: &mut TestHost, services: &mut dyn UiServices| {
            fret_ui::declarative::render_root(ui, app, services, window, bounds, "root", |cx| {
                let exposed = ExposedDropdown::new(selected_value.clone())
                    .query(query_for_render.clone())
                    .items(items.clone())
                    .a11y_label("exposed dropdown")
                    .test_id("material3-exposed-dropdown")
                    .into_element(cx);
                vec![with_padding(cx, Px(24.0), exposed)]
            })
        };

    run_overlay_frame_scaled(
        &mut ui,
        &mut app,
        &mut services,
        window,
        bounds,
        1.0,
        true,
        |ui, app, services| render(ui, app, services),
    );
    run_overlay_frame_scaled(
        &mut ui,
        &mut app,
        &mut services,
        window,
        bounds,
        1.0,
        true,
        |ui, app, services| render(ui, app, services),
    );

    let icon_node: NodeId = ui
        .semantics_snapshot()
        .and_then(|snapshot| {
            snapshot.nodes.iter().find_map(|node| {
                (node.test_id.as_deref() == Some("material3-exposed-dropdown-trailing-icon"))
                    .then_some(node.id)
            })
        })
        .expect("expected trailing icon node in semantics snapshot");

    let icon_bounds = ui
        .debug_node_visual_bounds(icon_node)
        .expect("expected trailing icon bounds");
    let click_at = Point::new(
        Px(icon_bounds.origin.x.0 + icon_bounds.size.width.0 * 0.5),
        Px(icon_bounds.origin.y.0 + icon_bounds.size.height.0 * 0.5),
    );

    ui.dispatch_event(
        &mut app,
        &mut services,
        &pointer_down(PointerId(1), click_at),
    );
    ui.dispatch_event(&mut app, &mut services, &pointer_up(PointerId(1), click_at));
    run_overlay_frame_scaled(
        &mut ui,
        &mut app,
        &mut services,
        window,
        bounds,
        1.0,
        true,
        |ui, app, services| render(ui, app, services),
    );

    let stack = OverlayController::stack_snapshot_for_window(&ui, &mut app, window);
    assert!(
        stack.stack.iter().any(|entry| {
            entry.kind == OverlayStackEntryKind::Popover && entry.open && entry.visible
        }),
        "expected popover overlay to be open after clicking the trailing icon"
    );

    ui.dispatch_event(
        &mut app,
        &mut services,
        &pointer_down(PointerId(1), click_at),
    );
    ui.dispatch_event(&mut app, &mut services, &pointer_up(PointerId(1), click_at));
    run_overlay_frame_scaled(
        &mut ui,
        &mut app,
        &mut services,
        window,
        bounds,
        1.0,
        true,
        |ui, app, services| render(ui, app, services),
    );

    let stack = OverlayController::stack_snapshot_for_window(&ui, &mut app, window);
    assert!(
        !stack.stack.iter().any(|entry| {
            entry.kind == OverlayStackEntryKind::Popover && entry.open && entry.visible
        }),
        "expected popover overlay to be closed after clicking the trailing icon again"
    );
}

#[test]
fn material3_headless_autocomplete_suite_goldens_v1() {
    use fret_ui::element::{FlexProps, Length};
    use fret_ui_kit::{OverlayController, OverlayStackEntryKind};
    use fret_ui_material3::{Autocomplete, AutocompleteItem, AutocompleteVariant};

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
            let bounds = Rect::new(
                Point::new(Px(0.0), Px(0.0)),
                Size::new(Px(860.0), Px(520.0)),
            );

            let mut cases: BTreeMap<String, Material3HeadlessGoldenV1> = BTreeMap::new();

            // Closed scene: show both variants so token drift is visible.
            {
                let mut app = TestHost::default();
                app.set_global(PlatformCapabilities::default());
                apply_material_theme(&mut app, mode, variant);

                let window = AppWindowId::default();
                let mut services = FakeUiServices::default();
                let mut ui: UiTree<TestHost> = UiTree::new();
                ui.set_window(window);

                let outlined_model = app.models_mut().insert(String::new());
                let filled_model = app.models_mut().insert(String::new());
                let items: Arc<[AutocompleteItem]> = Arc::from(vec![
                    AutocompleteItem::new("alpha", "Alpha"),
                    AutocompleteItem::new("beta", "Beta"),
                    AutocompleteItem::new("gamma", "Gamma"),
                    AutocompleteItem::new("delta", "Delta"),
                    AutocompleteItem::new("epsilon", "Epsilon"),
                ]);

                let render = move |ui: &mut UiTree<TestHost>,
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
                            let mut column = FlexProps::default();
                            column.direction = fret_core::Axis::Vertical;
                            column.gap = Px(16.0);

                            let outlined = Autocomplete::new(outlined_model.clone())
                                .variant(AutocompleteVariant::Outlined)
                                .label("Outlined")
                                .placeholder("Type to search")
                                .items(items.clone())
                                .a11y_label("outlined autocomplete")
                                .test_id("material3-ac-outlined")
                                .into_element(cx);
                            let outlined = cx.container(
                                {
                                    let mut props = ContainerProps::default();
                                    props.layout.size.width = Length::Px(Px(360.0));
                                    props
                                },
                                move |_cx| vec![outlined],
                            );

                            let filled = Autocomplete::new(filled_model.clone())
                                .variant(AutocompleteVariant::Filled)
                                .label("Filled")
                                .placeholder("Type to search")
                                .items(items.clone())
                                .a11y_label("filled autocomplete")
                                .test_id("material3-ac-filled")
                                .into_element(cx);
                            let filled = cx.container(
                                {
                                    let mut props = ContainerProps::default();
                                    props.layout.size.width = Length::Px(Px(360.0));
                                    props
                                },
                                move |_cx| vec![filled],
                            );

                            let content = cx.flex(column, |_cx| vec![outlined, filled]);
                            vec![with_padding(cx, Px(24.0), content)]
                        },
                    )
                };

                let message = format!(
                    "expected the Material3 autocomplete closed scene to be stable after animations settle ({label}, {scale})"
                );
                cases.insert(
                    "both_closed".to_string(),
                    settle_material3_overlay_scene_snapshot_v1(
                        &mut app,
                        &mut ui,
                        &mut services,
                        window,
                        bounds,
                        scale_factor,
                        16,
                        32,
                        &message,
                        &render,
                    ),
                );
            }

            for (case_name, focus_test_id) in [
                ("outlined_open", "material3-ac-outlined"),
                ("filled_open", "material3-ac-filled"),
            ] {
                let mut app = TestHost::default();
                app.set_global(PlatformCapabilities::default());
                apply_material_theme(&mut app, mode, variant);

                let window = AppWindowId::default();
                let mut services = FakeUiServices::default();
                let mut ui: UiTree<TestHost> = UiTree::new();
                ui.set_window(window);

                let outlined_model = app.models_mut().insert(String::new());
                let filled_model = app.models_mut().insert(String::new());
                let items: Arc<[AutocompleteItem]> = Arc::from(vec![
                    AutocompleteItem::new("alpha", "Alpha"),
                    AutocompleteItem::new("beta", "Beta"),
                    AutocompleteItem::new("gamma", "Gamma"),
                    AutocompleteItem::new("delta", "Delta"),
                    AutocompleteItem::new("epsilon", "Epsilon"),
                ]);

                let render = move |ui: &mut UiTree<TestHost>,
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
                            let mut column = FlexProps::default();
                            column.direction = fret_core::Axis::Vertical;
                            column.gap = Px(16.0);

                            let outlined = Autocomplete::new(outlined_model.clone())
                                .variant(AutocompleteVariant::Outlined)
                                .label("Outlined")
                                .placeholder("Type to search")
                                .items(items.clone())
                                .a11y_label("outlined autocomplete")
                                .test_id("material3-ac-outlined")
                                .into_element(cx);
                            let outlined = cx.container(
                                {
                                    let mut props = ContainerProps::default();
                                    props.layout.size.width = Length::Px(Px(360.0));
                                    props
                                },
                                move |_cx| vec![outlined],
                            );

                            let filled = Autocomplete::new(filled_model.clone())
                                .variant(AutocompleteVariant::Filled)
                                .label("Filled")
                                .placeholder("Type to search")
                                .items(items.clone())
                                .a11y_label("filled autocomplete")
                                .test_id("material3-ac-filled")
                                .into_element(cx);
                            let filled = cx.container(
                                {
                                    let mut props = ContainerProps::default();
                                    props.layout.size.width = Length::Px(Px(360.0));
                                    props
                                },
                                move |_cx| vec![filled],
                            );

                            let content = cx.flex(column, |_cx| vec![outlined, filled]);
                            vec![with_padding(cx, Px(24.0), content)]
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

                let input_node: NodeId = ui
                    .semantics_snapshot()
                    .and_then(|snapshot| {
                        snapshot.nodes.iter().find_map(|node| {
                            (node.test_id.as_deref() == Some(focus_test_id)).then_some(node.id)
                        })
                    })
                    .unwrap_or_else(|| {
                        panic!(
                            "expected {focus_test_id} input node in semantics snapshot ({label}, {scale}, {case_name})"
                        )
                    });

                ui.set_focus(Some(input_node));
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

                ui.dispatch_event(&mut app, &mut services, &key_down(KeyCode::ArrowDown));
                ui.dispatch_event(&mut app, &mut services, &key_up(KeyCode::ArrowDown));

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

                let stack = OverlayController::stack_snapshot_for_window(&ui, &mut app, window);
                assert!(
                    stack.stack.iter().any(|entry| {
                        entry.kind == OverlayStackEntryKind::Popover && entry.open && entry.visible
                    }),
                    "expected autocomplete popover overlay to be open after ArrowDown ({label}, {scale}, {case_name})"
                );

                let message = format!(
                    "expected the Material3 autocomplete overlay scene to be stable after animations settle ({label}, {scale}, {case_name})"
                );
                cases.insert(
                    case_name.to_string(),
                    settle_material3_overlay_scene_snapshot_v1(
                        &mut app,
                        &mut ui,
                        &mut services,
                        window,
                        bounds,
                        scale_factor,
                        44,
                        80,
                        &message,
                        &render,
                    ),
                );
            }

            let suite = Material3HeadlessSuiteV1 { cases };
            write_or_assert_material3_suite_v1(
                &format!("material3-autocomplete.{scale}.{label}"),
                &suite,
            );
        }
    }
}

#[test]
fn material3_headless_menu_dialog_style_suite_goldens_v1() {
    use fret_core::{Color, Corners};
    use fret_ui::element::{ContainerProps, CrossAlign, FlexProps, Length, MainAlign};
    use fret_ui_kit::{ColorRef, WidgetStateProperty};
    use fret_ui_material3::menu::{Menu, MenuEntry, MenuItem, MenuStyle};
    use fret_ui_material3::{Button, Dialog, DialogAction, DialogStyle};

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
            let bounds = Rect::new(
                Point::new(Px(0.0), Px(0.0)),
                Size::new(Px(860.0), Px(520.0)),
            );

            let mut cases: BTreeMap<String, Material3HeadlessGoldenV1> = BTreeMap::new();

            // Menu: default vs override (in the same scene).
            {
                let mut app = TestHost::default();
                app.set_global(PlatformCapabilities::default());
                apply_material_theme(&mut app, mode, variant);

                let window = AppWindowId::default();
                let mut services = FakeUiServices::default();
                let mut ui: UiTree<TestHost> = UiTree::new();
                ui.set_window(window);

                let override_bg = Color {
                    r: 0.9,
                    g: 0.1,
                    b: 0.2,
                    a: 1.0,
                };
                let override_label = Color {
                    r: 0.1,
                    g: 0.8,
                    b: 0.3,
                    a: 1.0,
                };

                let style = MenuStyle::default()
                    .container_background(WidgetStateProperty::new(Some(ColorRef::Color(
                        override_bg,
                    ))))
                    .container_corner_radii(WidgetStateProperty::new(Some(Corners::all(Px(0.0)))))
                    .container_elevation(WidgetStateProperty::new(Some(Px(12.0))))
                    .item_label_color(WidgetStateProperty::new(Some(ColorRef::Color(
                        override_label,
                    ))));

                let entries = vec![
                    MenuEntry::Item(MenuItem::new("A").test_id("menu-item-a")),
                    MenuEntry::Item(MenuItem::new("B").test_id("menu-item-b")),
                    MenuEntry::Item(MenuItem::new("C (disabled)").disabled(true)),
                    MenuEntry::Separator,
                    MenuEntry::Item(MenuItem::new("D").test_id("menu-item-d")),
                ];

                let render = move |ui: &mut UiTree<TestHost>,
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
                            let default_menu = Menu::new()
                                .entries(entries.clone())
                                .a11y_label("default menu")
                                .test_id("menu-default")
                                .into_element(cx);

                            let override_menu = Menu::new()
                                .entries(entries.clone())
                                .a11y_label("override menu")
                                .test_id("menu-override")
                                .style(style.clone())
                                .into_element(cx);

                            let mut props = FlexProps::default();
                            props.direction = fret_core::Axis::Horizontal;
                            props.gap = Px(32.0);
                            props.align = CrossAlign::Start;
                            props.justify = MainAlign::Center;

                            let content = cx.flex(props, |cx| {
                                let mut left = ContainerProps::default();
                                left.layout.size.width = Length::Px(Px(360.0));
                                let left = cx.container(left, |_cx| vec![default_menu]);

                                let mut right = ContainerProps::default();
                                right.layout.size.width = Length::Px(Px(360.0));
                                let right = cx.container(right, |_cx| vec![override_menu]);

                                vec![left, right]
                            });

                            vec![with_padding(cx, Px(24.0), content)]
                        },
                    )
                };

                let message = format!(
                    "expected the Material3 menu style scene to be stable ({label}, {scale})"
                );
                cases.insert(
                    "menu_default_vs_override".to_string(),
                    settle_material3_scene_snapshot_v1(
                        &mut app,
                        &mut ui,
                        &mut services,
                        bounds,
                        scale_factor,
                        7,
                        9,
                        &message,
                        &render,
                    ),
                );
            }

            // Dialog: default open state (modal overlay).
            {
                let mut app = TestHost::default();
                app.set_global(PlatformCapabilities::default());
                apply_material_theme(&mut app, mode, variant);

                let window = AppWindowId::default();
                let mut services = FakeUiServices::default();
                let mut ui: UiTree<TestHost> = UiTree::new();
                ui.set_window(window);

                let open = app.models_mut().insert(true);

                let render = move |ui: &mut UiTree<TestHost>,
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
                            let dialog = Dialog::new(open.clone())
                                .headline("Dialog")
                                .supporting_text("Body")
                                .actions(vec![DialogAction::new("OK")])
                                .test_id("dialog-default")
                                .into_element(
                                    cx,
                                    |cx| {
                                        let trigger = Button::new("Underlay")
                                            .test_id("dialog-underlay")
                                            .into_element(cx);
                                        with_padding(cx, Px(24.0), trigger)
                                    },
                                    |_cx| Vec::new(),
                                );

                            vec![dialog]
                        },
                    )
                };

                let message = format!(
                    "expected the Material3 dialog default scene to be stable after animations settle ({label}, {scale})"
                );
                cases.insert(
                    "dialog_default".to_string(),
                    settle_material3_overlay_scene_snapshot_v1(
                        &mut app,
                        &mut ui,
                        &mut services,
                        window,
                        bounds,
                        scale_factor,
                        44,
                        80,
                        &message,
                        &render,
                    ),
                );
            }

            // Dialog: override surface + text colors.
            {
                let mut app = TestHost::default();
                app.set_global(PlatformCapabilities::default());
                apply_material_theme(&mut app, mode, variant);

                let window = AppWindowId::default();
                let mut services = FakeUiServices::default();
                let mut ui: UiTree<TestHost> = UiTree::new();
                ui.set_window(window);

                let open = app.models_mut().insert(true);

                let override_bg = Color {
                    r: 0.2,
                    g: 0.2,
                    b: 0.9,
                    a: 1.0,
                };
                let override_headline = Color {
                    r: 0.9,
                    g: 0.9,
                    b: 0.2,
                    a: 1.0,
                };
                let override_supporting = Color {
                    r: 0.8,
                    g: 0.2,
                    b: 0.8,
                    a: 1.0,
                };

                let style = DialogStyle::default()
                    .container_background(WidgetStateProperty::new(Some(ColorRef::Color(
                        override_bg,
                    ))))
                    .container_corner_radii(WidgetStateProperty::new(Some(Corners::all(Px(0.0)))))
                    .container_elevation(WidgetStateProperty::new(Some(Px(12.0))))
                    .headline_color(WidgetStateProperty::new(Some(ColorRef::Color(
                        override_headline,
                    ))))
                    .supporting_text_color(WidgetStateProperty::new(Some(ColorRef::Color(
                        override_supporting,
                    ))));

                let render = move |ui: &mut UiTree<TestHost>,
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
                            let dialog = Dialog::new(open.clone())
                                .headline("Dialog")
                                .supporting_text("Body")
                                .actions(vec![DialogAction::new("OK")])
                                .style(style.clone())
                                .test_id("dialog-override")
                                .into_element(
                                    cx,
                                    |cx| {
                                        let trigger = Button::new("Underlay")
                                            .test_id("dialog-underlay")
                                            .into_element(cx);
                                        with_padding(cx, Px(24.0), trigger)
                                    },
                                    |_cx| Vec::new(),
                                );

                            vec![dialog]
                        },
                    )
                };

                let message = format!(
                    "expected the Material3 dialog override scene to be stable after animations settle ({label}, {scale})"
                );
                cases.insert(
                    "dialog_override".to_string(),
                    settle_material3_overlay_scene_snapshot_v1(
                        &mut app,
                        &mut ui,
                        &mut services,
                        window,
                        bounds,
                        scale_factor,
                        44,
                        80,
                        &message,
                        &render,
                    ),
                );
            }

            let suite = Material3HeadlessSuiteV1 { cases };
            write_or_assert_material3_suite_v1(
                &format!("material3-menu-dialog-style.{scale}.{label}"),
                &suite,
            );
        }
    }
}

#[test]
fn material3_headless_bottom_sheet_suite_goldens_v1() {
    use fret_ui_material3::{
        Button, ButtonVariant, DockedBottomSheet, DockedBottomSheetVariant, ModalBottomSheet,
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
            let mut cases: BTreeMap<String, Material3HeadlessGoldenV1> = BTreeMap::new();

            // Docked sheet (standard): non-overlay surface.
            {
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

                let render = move |ui: &mut UiTree<TestHost>,
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
                            let sheet = DockedBottomSheet::new()
                                .variant(DockedBottomSheetVariant::Standard)
                                .test_id("bottom-sheet-docked")
                                .into_element(cx, |cx| {
                                    vec![
                                        cx.text("Docked bottom sheet"),
                                        Button::new("Primary")
                                            .variant(ButtonVariant::Filled)
                                            .test_id("bottom-sheet-docked-primary")
                                            .into_element(cx),
                                        Button::new("Secondary")
                                            .variant(ButtonVariant::Outlined)
                                            .test_id("bottom-sheet-docked-secondary")
                                            .into_element(cx),
                                    ]
                                });

                            vec![with_padding(cx, Px(24.0), sheet)]
                        },
                    )
                };

                let message = format!(
                    "expected the Material3 docked bottom sheet scene to be stable ({label}, {scale})"
                );
                cases.insert(
                    "docked_standard".to_string(),
                    settle_material3_scene_snapshot_v1(
                        &mut app,
                        &mut ui,
                        &mut services,
                        bounds,
                        scale_factor,
                        2,
                        6,
                        &message,
                        &render,
                    ),
                );
            }

            // Modal sheet (open): overlay surface + scrim.
            {
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
                    fret_ui::declarative::render_root(
                        ui,
                        app,
                        services,
                        window,
                        bounds,
                        "root",
                        |cx| {
                            let sheet = ModalBottomSheet::new(open_model.clone())
                                .open_duration_ms(Some(1))
                                .close_duration_ms(Some(1))
                                .test_id("bottom-sheet-modal")
                                .into_element(
                                    cx,
                                    |cx| {
                                        Button::new("Underlay probe")
                                            .variant(ButtonVariant::Outlined)
                                            .test_id("bottom-sheet-underlay-probe")
                                            .into_element(cx)
                                    },
                                    |cx| {
                                        vec![
                                            cx.text("Modal bottom sheet"),
                                            Button::new("Close")
                                                .variant(ButtonVariant::Filled)
                                                .test_id("bottom-sheet-modal-close")
                                                .into_element(cx),
                                        ]
                                    },
                                );
                            vec![with_padding(cx, Px(24.0), sheet)]
                        },
                    )
                };

                let message = format!(
                    "expected the Material3 modal bottom sheet overlay scene to be stable ({label}, {scale})"
                );
                cases.insert(
                    "modal_open".to_string(),
                    settle_material3_overlay_scene_snapshot_v1(
                        &mut app,
                        &mut ui,
                        &mut services,
                        window,
                        bounds,
                        scale_factor,
                        4,
                        10,
                        &message,
                        &render,
                    ),
                );
            }

            let suite = Material3HeadlessSuiteV1 { cases };
            write_or_assert_material3_suite_v1(
                &format!("material3-bottom-sheet.{scale}.{label}"),
                &suite,
            );
        }
    }
}

#[test]
fn material3_headless_date_picker_suite_goldens_v1() {
    use fret_ui_kit::headless::calendar::CalendarMonth;
    use fret_ui_material3::{
        Button, ButtonVariant, DatePickerDialog, DatePickerVariant, DockedDatePicker,
    };
    use time::{Date, Month};

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

    let today = Date::from_calendar_date(2026, Month::January, 10).expect("valid date");
    let selected_date = Date::from_calendar_date(2026, Month::January, 15).expect("valid date");

    for scale_factor in [1.0, 1.25, 2.0] {
        let scale = scale_segment(scale_factor);

        for (mode, variant, label) in schemes {
            let mut cases: BTreeMap<String, Material3HeadlessGoldenV1> = BTreeMap::new();

            // Docked picker: non-overlay surface.
            {
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

                let month = app
                    .models_mut()
                    .insert(CalendarMonth::new(2026, Month::January));
                let selected = app.models_mut().insert(Some(selected_date));

                let render = move |ui: &mut UiTree<TestHost>,
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
                            let picker = DockedDatePicker::new(month.clone(), selected.clone())
                                .variant(DatePickerVariant::Docked)
                                .today(Some(today))
                                .test_id("date-picker-docked")
                                .into_element(cx);
                            vec![with_padding(cx, Px(24.0), picker)]
                        },
                    )
                };

                let message = format!(
                    "expected the Material3 docked date picker scene to be stable ({label}, {scale})"
                );
                cases.insert(
                    "docked".to_string(),
                    settle_material3_scene_snapshot_v1(
                        &mut app,
                        &mut ui,
                        &mut services,
                        bounds,
                        scale_factor,
                        2,
                        6,
                        &message,
                        &render,
                    ),
                );
            }

            // Modal picker: overlay + scrim + focus trap.
            {
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
                let month = app
                    .models_mut()
                    .insert(CalendarMonth::new(2026, Month::January));
                let selected = app.models_mut().insert(Some(selected_date));

                let render = move |ui: &mut UiTree<TestHost>,
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
                            let dialog = DatePickerDialog::new(
                                open.clone(),
                                month.clone(),
                                selected.clone(),
                            )
                            .today(Some(today))
                            .open_duration_ms(Some(1))
                            .close_duration_ms(Some(1))
                            .test_id("date-picker-modal")
                            .into_element(cx, |cx| {
                                Button::new("Underlay probe")
                                    .variant(ButtonVariant::Outlined)
                                    .test_id("date-picker-underlay-probe")
                                    .into_element(cx)
                            });
                            vec![with_padding(cx, Px(24.0), dialog)]
                        },
                    )
                };

                let message = format!(
                    "expected the Material3 date picker modal overlay scene to be stable ({label}, {scale})"
                );
                cases.insert(
                    "modal_open".to_string(),
                    settle_material3_overlay_scene_snapshot_v1(
                        &mut app,
                        &mut ui,
                        &mut services,
                        window,
                        bounds,
                        scale_factor,
                        4,
                        10,
                        &message,
                        &render,
                    ),
                );
            }

            let suite = Material3HeadlessSuiteV1 { cases };
            write_or_assert_material3_suite_v1(
                &format!("material3-date-picker.{scale}.{label}"),
                &suite,
            );
        }
    }
}

#[test]
fn material3_headless_time_picker_suite_goldens_v1() {
    use fret_ui_material3::{
        Button, ButtonVariant, DockedTimePicker, TimePickerDialog, TimePickerDisplayMode,
    };
    use time::Time;

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

    let selected_time = Time::from_hms(9, 41, 0).expect("valid time");

    for scale_factor in [1.0, 1.25, 2.0] {
        let scale = scale_segment(scale_factor);

        for (mode, variant, label) in schemes {
            let mut cases: BTreeMap<String, Material3HeadlessGoldenV1> = BTreeMap::new();

            // Docked picker: non-overlay surface.
            {
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

                let time = app.models_mut().insert(selected_time);

                let render = move |ui: &mut UiTree<TestHost>,
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
                            let picker = DockedTimePicker::new(time.clone())
                                .test_id("time-picker-docked")
                                .into_element(cx);
                            vec![with_padding(cx, Px(24.0), picker)]
                        },
                    )
                };

                let message = format!(
                    "expected the Material3 docked time picker scene to be stable ({label}, {scale})"
                );
                cases.insert(
                    "docked".to_string(),
                    settle_material3_scene_snapshot_v1(
                        &mut app,
                        &mut ui,
                        &mut services,
                        bounds,
                        scale_factor,
                        2,
                        6,
                        &message,
                        &render,
                    ),
                );
            }

            // Docked picker: input mode.
            {
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

                let time = app.models_mut().insert(selected_time);

                let render = move |ui: &mut UiTree<TestHost>,
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
                            let picker = DockedTimePicker::new(time.clone())
                                .display_mode(TimePickerDisplayMode::Input)
                                .test_id("time-picker-docked-input")
                                .into_element(cx);
                            vec![with_padding(cx, Px(24.0), picker)]
                        },
                    )
                };

                let message = format!(
                    "expected the Material3 docked time picker input scene to be stable ({label}, {scale})"
                );
                cases.insert(
                    "docked_input".to_string(),
                    settle_material3_scene_snapshot_v1(
                        &mut app,
                        &mut ui,
                        &mut services,
                        bounds,
                        scale_factor,
                        2,
                        6,
                        &message,
                        &render,
                    ),
                );
            }

            // Modal picker: overlay + scrim + focus trap.
            {
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
                let time = app.models_mut().insert(selected_time);

                let render = move |ui: &mut UiTree<TestHost>,
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
                            let dialog = TimePickerDialog::new(open.clone(), time.clone())
                                .open_duration_ms(Some(1))
                                .close_duration_ms(Some(1))
                                .test_id("time-picker-modal")
                                .into_element(cx, |cx| {
                                    Button::new("Underlay probe")
                                        .variant(ButtonVariant::Outlined)
                                        .test_id("time-picker-underlay-probe")
                                        .into_element(cx)
                                });
                            vec![with_padding(cx, Px(24.0), dialog)]
                        },
                    )
                };

                let message = format!(
                    "expected the Material3 time picker modal overlay scene to be stable ({label}, {scale})"
                );
                cases.insert(
                    "modal_open".to_string(),
                    settle_material3_overlay_scene_snapshot_v1(
                        &mut app,
                        &mut ui,
                        &mut services,
                        window,
                        bounds,
                        scale_factor,
                        4,
                        10,
                        &message,
                        &render,
                    ),
                );
            }

            // Modal picker: input mode.
            {
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
                let time = app.models_mut().insert(selected_time);

                let render = move |ui: &mut UiTree<TestHost>,
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
                            let dialog = TimePickerDialog::new(open.clone(), time.clone())
                                .initial_display_mode(TimePickerDisplayMode::Input)
                                .open_duration_ms(Some(1))
                                .close_duration_ms(Some(1))
                                .test_id("time-picker-modal-input")
                                .into_element(cx, |cx| {
                                    Button::new("Underlay probe")
                                        .variant(ButtonVariant::Outlined)
                                        .test_id("time-picker-underlay-probe")
                                        .into_element(cx)
                                });
                            vec![with_padding(cx, Px(24.0), dialog)]
                        },
                    )
                };

                let message = format!(
                    "expected the Material3 time picker modal overlay input scene to be stable ({label}, {scale})"
                );
                cases.insert(
                    "modal_open_input".to_string(),
                    settle_material3_overlay_scene_snapshot_v1(
                        &mut app,
                        &mut ui,
                        &mut services,
                        window,
                        bounds,
                        scale_factor,
                        4,
                        10,
                        &message,
                        &render,
                    ),
                );
            }

            let suite = Material3HeadlessSuiteV1 { cases };
            write_or_assert_material3_suite_v1(
                &format!("material3-time-picker.{scale}.{label}"),
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
        (KeyCode::Enter, "enter"),
        (KeyCode::Space, "space"),
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
fn chip_set_roving_treats_trailing_action_focus_as_active_chip() {
    use fret_ui_material3::{ChipSet, ChipSetItem, InputChip, SuggestionChip};

    let mut app = TestHost::default();
    app.set_global(PlatformCapabilities::default());
    apply_material_theme(&mut app, SchemeMode::Light, DynamicVariant::TonalSpot);

    let window = AppWindowId::default();
    let mut services = FakeUiServices::default();
    let mut ui: UiTree<TestHost> = UiTree::new();
    ui.set_window(window);

    let bounds = Rect::new(
        Point::new(Px(0.0), Px(0.0)),
        Size::new(Px(420.0), Px(220.0)),
    );

    let chip_a_selected = app.models_mut().insert(false);

    let render = |ui: &mut UiTree<TestHost>, app: &mut TestHost, services: &mut dyn UiServices| {
        fret_ui::declarative::render_root(ui, app, services, window, bounds, "root", |cx| {
            let chip_a = InputChip::new(chip_a_selected.clone(), "Alpha")
                .trailing_icon(fret_icons::ids::ui::CLOSE)
                .on_trailing_icon_activate(Arc::new(|_host, _acx, _reason| {}))
                .test_id("chip-a");

            let chip_b = SuggestionChip::new("Beta").test_id("chip-b");

            let set = ChipSet::new(vec![ChipSetItem::from(chip_a), ChipSetItem::from(chip_b)])
                .a11y_label("chips")
                .test_id("chip-set")
                .into_element(cx);

            vec![with_padding(cx, Px(24.0), set)]
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

    let chip_a_node: NodeId = ui
        .semantics_snapshot()
        .and_then(|snapshot| {
            snapshot
                .nodes
                .iter()
                .find_map(|node| (node.test_id.as_deref() == Some("chip-a")).then_some(node.id))
        })
        .expect("expected chip-a in semantics snapshot");

    let chip_a_trailing_node: NodeId = ui
        .semantics_snapshot()
        .and_then(|snapshot| {
            snapshot.nodes.iter().find_map(|node| {
                (node.test_id.as_deref() == Some("chip-a.trailing-icon")).then_some(node.id)
            })
        })
        .expect("expected chip-a.trailing-icon in semantics snapshot");

    let chip_b_node: NodeId = ui
        .semantics_snapshot()
        .and_then(|snapshot| {
            snapshot
                .nodes
                .iter()
                .find_map(|node| (node.test_id.as_deref() == Some("chip-b")).then_some(node.id))
        })
        .expect("expected chip-b in semantics snapshot");

    ui.set_focus(Some(chip_a_node));
    assert_eq!(ui.focus(), Some(chip_a_node));

    // ArrowRight should move focus to the trailing action inside the chip (handled by the chip),
    // not rove to the next chip.
    ui.dispatch_event(
        &mut app,
        &mut services,
        &Event::KeyDown {
            key: KeyCode::ArrowRight,
            modifiers: Modifiers::default(),
            repeat: false,
        },
    );
    assert_eq!(
        ui.focus(),
        Some(chip_a_trailing_node),
        "expected ArrowRight to focus trailing action (chip-internal navigation)",
    );

    // ArrowRight again should bubble to ChipSet roving (chip-internal handler does not consume),
    // and move focus to the next chip.
    ui.dispatch_event(
        &mut app,
        &mut services,
        &Event::KeyDown {
            key: KeyCode::ArrowRight,
            modifiers: Modifiers::default(),
            repeat: false,
        },
    );
    assert_eq!(
        ui.focus(),
        Some(chip_b_node),
        "expected ChipSet roving to treat trailing-focus as within the active chip",
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

#[test]
fn select_menu_matches_anchor_width_and_clamps_height_to_available_space() {
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
        Size::new(Px(420.0), Px(320.0)),
    );

    let selected = app.models_mut().insert(Some(Arc::<str>::from("v0")));
    let items: Arc<[SelectItem]> = (0..40)
        .map(|i| SelectItem::new(Arc::<str>::from(format!("v{i}")), format!("Item {i}")))
        .collect::<Vec<_>>()
        .into();

    let selected_model = selected.clone();
    let render =
        move |ui: &mut UiTree<TestHost>, app: &mut TestHost, services: &mut dyn UiServices| {
            let selected_model = selected_model.clone();
            let items = items.clone();
            fret_ui::declarative::render_root(ui, app, services, window, bounds, "root", |cx| {
                let mut l = fret_ui::element::LayoutStyle::default();
                l.position = fret_ui::element::PositionStyle::Absolute;
                l.inset = fret_ui::element::InsetStyle {
                    top: Some(Px(200.0)),
                    left: Some(Px(24.0)),
                    right: None,
                    bottom: None,
                };
                l.size.width = fret_ui::element::Length::Px(Px(240.0));
                l.size.height = fret_ui::element::Length::Auto;
                l.overflow = fret_ui::element::Overflow::Visible;

                vec![cx.container(
                    fret_ui::element::ContainerProps {
                        layout: l,
                        ..Default::default()
                    },
                    move |cx| {
                        vec![
                            Select::new(selected_model)
                                .a11y_label("select")
                                .placeholder("Pick one")
                                .items(items)
                                .test_id("select-trigger")
                                .into_element(cx),
                        ]
                    },
                )]
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

    for _ in 0..20 {
        run_overlay_frame(
            &mut ui,
            &mut app,
            &mut services,
            window,
            bounds,
            true,
            |ui, app, services| render(ui, app, services),
        );
    }

    let snapshot = ui
        .semantics_snapshot()
        .expect("expected semantics snapshot");

    let listbox_node: NodeId = snapshot
        .nodes
        .iter()
        .find_map(|node| {
            (node.test_id.as_deref() == Some("select-trigger-listbox")).then_some(node.id)
        })
        .expect("expected select-trigger-listbox in semantics snapshot");
    let listbox_bounds = ui
        .debug_node_visual_bounds(listbox_node)
        .expect("expected listbox bounds");

    let epsilon = 0.01;
    assert!(
        (listbox_bounds.size.width.0 - trigger_bounds.size.width.0).abs() <= epsilon,
        "expected listbox width to match trigger width"
    );

    let collision_top = 48.0;
    let collision_bottom = 48.0;
    let gap = 4.0;

    let outer_top = bounds.origin.y.0 + collision_top;
    let outer_bottom = bounds.origin.y.0 + bounds.size.height.0 - collision_bottom;
    let anchor_top = trigger_bounds.origin.y.0;
    let anchor_bottom = trigger_bounds.origin.y.0 + trigger_bounds.size.height.0;

    let available_above = anchor_top - (outer_top + gap);
    let available_below = outer_bottom - (anchor_bottom + gap);
    let available = available_above.max(available_below).max(0.0);

    assert!(
        listbox_bounds.size.height.0 <= available + epsilon,
        "expected listbox height to clamp to available space (got {}, want <= {})",
        listbox_bounds.size.height.0,
        available
    );
    assert!(
        (listbox_bounds.size.height.0 - available).abs() <= 0.5,
        "expected listbox height to match available space when content overflows (got {}, want ~ {})",
        listbox_bounds.size.height.0,
        available
    );
}

#[test]
fn select_exposes_combobox_controls_and_listbox_labelled_by_relations() {
    use fret_ui_kit::{OverlayController, OverlayStackEntryKind};
    use fret_ui_material3::{Select, SelectItem};

    let mut app = TestHost::default();
    app.set_global(PlatformCapabilities::default());
    apply_material_theme(&mut app, SchemeMode::Dark, DynamicVariant::TonalSpot);

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
        SelectItem::new("alpha", "Alpha"),
        SelectItem::new("beta", "Beta"),
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
    assert!(opened, "expected select overlay to open on click");

    // One extra frame: the trigger's `controls_element` is resolved via last-frame element IDs.
    run_overlay_frame(
        &mut ui,
        &mut app,
        &mut services,
        window,
        bounds,
        false,
        |ui, app, services| render(ui, app, services),
    );
    run_overlay_frame(
        &mut ui,
        &mut app,
        &mut services,
        window,
        bounds,
        true,
        |ui, app, services| render(ui, app, services),
    );

    let snap = ui.semantics_snapshot().expect("semantics snapshot");
    let trigger = snap
        .nodes
        .iter()
        .find(|n| n.test_id.as_deref() == Some("select-trigger"))
        .expect("select trigger semantics node");
    assert!(
        trigger.flags.expanded,
        "select trigger should report expanded=true while open"
    );

    let listbox = snap
        .nodes
        .iter()
        .find(|n| n.test_id.as_deref() == Some("select-trigger-listbox"))
        .expect("select listbox semantics node");

    assert!(
        trigger.controls.iter().any(|id| *id == listbox.id),
        "select trigger should control the listbox"
    );
    assert!(
        listbox.labelled_by.iter().any(|id| *id == trigger.id),
        "select listbox should be labelled by the trigger"
    );
}

#[test]
fn select_listbox_typeahead_moves_focus_skipping_disabled_options() {
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

    let selected = app.models_mut().insert(Some(Arc::<str>::from("beta")));
    let items: Arc<[SelectItem]> = vec![
        SelectItem::new("alpha", "Alpha").test_id("select-item-alpha"),
        SelectItem::new("beta", "Beta").test_id("select-item-beta"),
        SelectItem::new("charlie", "Charlie (disabled)")
            .disabled(true)
            .test_id("select-item-charlie-disabled"),
        SelectItem::new("delta", "Delta").test_id("select-item-delta"),
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

    let beta_option_node: NodeId = ui
        .semantics_snapshot()
        .and_then(|snapshot| {
            snapshot.nodes.iter().find_map(|node| {
                (node.test_id.as_deref() == Some("select-item-beta")).then_some(node.id)
            })
        })
        .expect("expected select-item-beta in semantics snapshot");
    assert_eq!(
        ui.focus(),
        Some(beta_option_node),
        "expected select to focus the selected option when opening via keyboard"
    );

    ui.dispatch_event(&mut app, &mut services, &key_down(KeyCode::KeyC));
    ui.dispatch_event(&mut app, &mut services, &key_up(KeyCode::KeyC));
    run_overlay_frame(
        &mut ui,
        &mut app,
        &mut services,
        window,
        bounds,
        true,
        |ui, app, services| render(ui, app, services),
    );
    let focused_test_id = ui.semantics_snapshot().and_then(|snapshot| {
        ui.focus().and_then(|focused| {
            snapshot
                .nodes
                .iter()
                .find(|node| node.id == focused)
                .and_then(|node| node.test_id.as_deref())
        })
    });
    assert_eq!(
        focused_test_id,
        Some("select-item-beta"),
        "expected typeahead to ignore disabled matches (KeyC)"
    );

    // Wait for the typeahead buffer to expire (select installs a prefix-buffer typeahead policy).
    for _ in 0..40 {
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

    ui.dispatch_event(&mut app, &mut services, &key_down(KeyCode::KeyD));
    ui.dispatch_event(&mut app, &mut services, &key_up(KeyCode::KeyD));
    run_overlay_frame(
        &mut ui,
        &mut app,
        &mut services,
        window,
        bounds,
        true,
        |ui, app, services| render(ui, app, services),
    );

    let focused_test_id = ui.semantics_snapshot().and_then(|snapshot| {
        ui.focus().and_then(|focused| {
            snapshot
                .nodes
                .iter()
                .find(|node| node.id == focused)
                .and_then(|node| node.test_id.as_deref())
        })
    });
    assert_eq!(
        focused_test_id,
        Some("select-item-delta"),
        "expected typeahead to rove focus to the matching option (KeyD)"
    );
}

#[test]
fn select_typeahead_delay_controls_buffer_expiration() {
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

    let selected = app.models_mut().insert(Some(Arc::<str>::from("beta")));
    let items: Arc<[SelectItem]> = vec![
        SelectItem::new("beta", "Beta").test_id("select-item-beta"),
        SelectItem::new("delta", "Delta").test_id("select-item-delta"),
        SelectItem::new("echo", "Echo").test_id("select-item-echo"),
    ]
    .into();

    let delay_ms = 1000;
    let timeout_ticks = fret_ui_material3::motion::ms_to_frames(delay_ms);

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
                        .typeahead_delay_ms(delay_ms)
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

    let beta_option_node: NodeId = ui
        .semantics_snapshot()
        .and_then(|snapshot| {
            snapshot.nodes.iter().find_map(|node| {
                (node.test_id.as_deref() == Some("select-item-beta")).then_some(node.id)
            })
        })
        .expect("expected select-item-beta in semantics snapshot");
    assert_eq!(
        ui.focus(),
        Some(beta_option_node),
        "expected select to focus the selected option when opening via keyboard"
    );

    ui.dispatch_event(&mut app, &mut services, &key_down(KeyCode::KeyD));
    ui.dispatch_event(&mut app, &mut services, &key_up(KeyCode::KeyD));
    run_overlay_frame(
        &mut ui,
        &mut app,
        &mut services,
        window,
        bounds,
        true,
        |ui, app, services| render(ui, app, services),
    );

    let focused_test_id = ui.semantics_snapshot().and_then(|snapshot| {
        ui.focus().and_then(|focused| {
            snapshot
                .nodes
                .iter()
                .find(|node| node.id == focused)
                .and_then(|node| node.test_id.as_deref())
        })
    });
    assert_eq!(
        focused_test_id,
        Some("select-item-delta"),
        "expected typeahead (KeyD) to focus Delta"
    );

    // The buffer should still be active: `d` + `e` => "de" matches Delta, not Echo.
    for _ in 0..timeout_ticks.saturating_sub(1) {
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

    ui.dispatch_event(&mut app, &mut services, &key_down(KeyCode::KeyE));
    ui.dispatch_event(&mut app, &mut services, &key_up(KeyCode::KeyE));
    run_overlay_frame(
        &mut ui,
        &mut app,
        &mut services,
        window,
        bounds,
        true,
        |ui, app, services| render(ui, app, services),
    );

    let focused_test_id = ui.semantics_snapshot().and_then(|snapshot| {
        ui.focus().and_then(|focused| {
            snapshot
                .nodes
                .iter()
                .find(|node| node.id == focused)
                .and_then(|node| node.test_id.as_deref())
        })
    });
    assert_eq!(
        focused_test_id,
        Some("select-item-delta"),
        "expected typeahead buffer to keep 'de' and stay on Delta before timeout"
    );

    // Now let the buffer expire, then 'e' should match Echo.
    for _ in 0..(timeout_ticks + 2) {
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

    ui.dispatch_event(&mut app, &mut services, &key_down(KeyCode::KeyE));
    ui.dispatch_event(&mut app, &mut services, &key_up(KeyCode::KeyE));
    run_overlay_frame(
        &mut ui,
        &mut app,
        &mut services,
        window,
        bounds,
        true,
        |ui, app, services| render(ui, app, services),
    );

    let focused_test_id = ui.semantics_snapshot().and_then(|snapshot| {
        ui.focus().and_then(|focused| {
            snapshot
                .nodes
                .iter()
                .find(|node| node.id == focused)
                .and_then(|node| node.test_id.as_deref())
        })
    });
    assert_eq!(
        focused_test_id,
        Some("select-item-echo"),
        "expected typeahead buffer to expire and 'e' to match Echo after timeout"
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
