//! Interaction regression tests for Material 3 text fields.
//!
//! These are renderer-agnostic: we assert invariants by inspecting the `SceneOp` stream.

use std::{
    any::{Any, TypeId},
    collections::{HashMap, HashSet},
};

use fret_core::{
    AppWindowId, Event, KeyCode, Modifiers, MouseButton, MouseButtons, NodeId, Point, PointerEvent,
    PointerId, PointerType, Px, Rect, Scene, Size, UiServices,
};
use fret_runtime::{
    CommandRegistry, CommandsHost, DragHost, DragKindId, DragSession, DragSessionId, Effect,
    EffectSink, FrameId, GlobalsHost, ModelHost, ModelId, ModelStore, ModelsHost,
    PlatformCapabilities, TickId, TimeHost,
};
use fret_ui::element::{ContainerProps, Length};
use fret_ui::{Theme, UiTree};
use fret_ui_material3::tokens::v30::{
    ColorSchemeOptions, DynamicVariant, SchemeMode, TypographyOptions, theme_config_with_colors,
};

mod interaction_harness;
use interaction_harness::{EdgesSig, QuadSig, RectSig, scene_quad_signature};

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

fn pointer_move(pointer_id: PointerId, position: Point) -> Event {
    Event::Pointer(PointerEvent::Move {
        pointer_id,
        position,
        buttons: MouseButtons::default(),
        modifiers: Modifiers::default(),
        pointer_type: PointerType::Mouse,
    })
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

fn find_filled_text_field_container(quads: &[QuadSig]) -> Option<QuadSig> {
    quads
        .iter()
        .copied()
        .filter(|q| {
            q.border.bottom > 0
                && q.border.top == 0
                && q.border.right == 0
                && q.border.left == 0
                && q.rect.w > 0
                && q.rect.h > 0
        })
        .max_by_key(|q| i64::from(q.rect.w) * i64::from(q.rect.h))
}

fn is_zero_edges(edges: EdgesSig) -> bool {
    edges.top == 0 && edges.right == 0 && edges.bottom == 0 && edges.left == 0
}

fn rect_inset_by_edges(rect: RectSig, border: EdgesSig) -> RectSig {
    RectSig {
        x: rect.x + border.left,
        y: rect.y + border.top,
        w: rect.w - border.left - border.right,
        h: rect.h - border.top - border.bottom,
    }
}

fn find_state_layer_overlay(quads: &[QuadSig], container: QuadSig) -> Option<QuadSig> {
    let expected = rect_inset_by_edges(container.rect, container.border);
    quads.iter().copied().find(|q| {
        q.rect == expected
            && q.corner_radii == container.corner_radii
            && is_zero_edges(q.border)
            && q.background.a > 0
    })
}

fn build_scene_for_text_field(
    ui: &mut UiTree<TestHost>,
    app: &mut TestHost,
    services: &mut dyn UiServices,
    window: AppWindowId,
    bounds: Rect,
    value: fret_runtime::Model<String>,
    variant: fret_ui_material3::TextFieldVariant,
    disabled: bool,
    error: bool,
) -> Scene {
    let root = fret_ui::declarative::render_root(ui, app, services, window, bounds, "root", |cx| {
        let field = fret_ui_material3::TextField::new(value)
            .variant(variant)
            .label("Email")
            .placeholder("name@example.com")
            .disabled(disabled)
            .error(error)
            .into_element(cx);

        let mut fixed = ContainerProps::default();
        fixed.layout.size.width = Length::Px(Px(240.0));
        fixed.layout.size.height = Length::Px(Px(56.0));
        vec![cx.container(fixed, move |_cx| vec![field])]
    });
    ui.set_root(root);
    ui.layout_all(app, services, bounds, 1.0);

    let mut scene = Scene::default();
    ui.paint_all(app, services, bounds, &mut scene, 1.0);
    scene
}

fn dispatch_hover(ui: &mut UiTree<TestHost>, app: &mut TestHost, services: &mut dyn UiServices) {
    ui.dispatch_event(
        app,
        services,
        &pointer_move(PointerId(0), Point::new(Px(10.0), Px(10.0))),
    );
}

fn dispatch_focus_click(
    ui: &mut UiTree<TestHost>,
    app: &mut TestHost,
    services: &mut dyn UiServices,
) {
    ui.dispatch_event(
        app,
        services,
        &pointer_down(PointerId(0), Point::new(Px(10.0), Px(10.0))),
    );
}

fn find_filled_hover_overlay_after_frames(
    ui: &mut UiTree<TestHost>,
    app: &mut TestHost,
    services: &mut dyn UiServices,
    window: AppWindowId,
    bounds: Rect,
    value: fret_runtime::Model<String>,
    disabled: bool,
    error: bool,
) -> Option<QuadSig> {
    for _ in 0..12 {
        app.advance_frame();
        let scene = build_scene_for_text_field(
            ui,
            app,
            services,
            window,
            bounds,
            value.clone(),
            fret_ui_material3::TextFieldVariant::Filled,
            disabled,
            error,
        );
        let quads = scene_quad_signature(&scene);
        let container = find_filled_text_field_container(&quads)?;
        if let Some(overlay) = find_state_layer_overlay(&quads, container) {
            return Some(overlay);
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

#[test]
fn filled_text_field_hover_uses_state_layer_overlay() {
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
            Size::new(Px(300.0), Px(200.0)),
        );
        let value = app.models_mut().insert(String::new());

        // Baseline: not hovered.
        let scene0 = build_scene_for_text_field(
            &mut ui,
            &mut app,
            &mut services,
            window,
            bounds,
            value.clone(),
            fret_ui_material3::TextFieldVariant::Filled,
            false,
            false,
        );
        let quads0 = scene_quad_signature(&scene0);
        let container0 = find_filled_text_field_container(&quads0)
            .expect("expected filled text field container");
        assert!(
            find_state_layer_overlay(&quads0, container0).is_none(),
            "expected no hover state layer before hover ({label})"
        );

        // Hover: move the pointer inside the fixed text field bounds.
        dispatch_hover(&mut ui, &mut app, &mut services);
        let _overlay = find_filled_hover_overlay_after_frames(
            &mut ui,
            &mut app,
            &mut services,
            window,
            bounds,
            value.clone(),
            false,
            false,
        )
        .unwrap_or_else(|| panic!("expected hover state layer overlay quad ({label})"));

        let scene1 = build_scene_for_text_field(
            &mut ui,
            &mut app,
            &mut services,
            window,
            bounds,
            value,
            fret_ui_material3::TextFieldVariant::Filled,
            false,
            false,
        );
        let quads1 = scene_quad_signature(&scene1);
        let container1 = find_filled_text_field_container(&quads1)
            .expect("expected filled text field container");
        assert_eq!(
            container0.background, container1.background,
            "expected stable container background ({label})"
        );
    }
}

#[test]
fn filled_text_field_hover_does_not_show_overlay_when_disabled() {
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
            Size::new(Px(300.0), Px(200.0)),
        );
        let value = app.models_mut().insert(String::new());

        let scene0 = build_scene_for_text_field(
            &mut ui,
            &mut app,
            &mut services,
            window,
            bounds,
            value.clone(),
            fret_ui_material3::TextFieldVariant::Filled,
            true,
            false,
        );
        let quads0 = scene_quad_signature(&scene0);
        let container0 = find_filled_text_field_container(&quads0)
            .expect("expected filled text field container");
        assert!(
            find_state_layer_overlay(&quads0, container0).is_none(),
            "expected no hover overlay baseline while disabled ({label})"
        );

        dispatch_hover(&mut ui, &mut app, &mut services);
        assert!(
            find_filled_hover_overlay_after_frames(
                &mut ui,
                &mut app,
                &mut services,
                window,
                bounds,
                value,
                true,
                false,
            )
            .is_none(),
            "expected no hover overlay while disabled ({label})"
        );
    }
}

#[test]
fn outlined_text_field_hover_does_not_show_overlay() {
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
    let value = app.models_mut().insert(String::new());

    let scene0 = build_scene_for_text_field(
        &mut ui,
        &mut app,
        &mut services,
        window,
        bounds,
        value.clone(),
        fret_ui_material3::TextFieldVariant::Outlined,
        false,
        false,
    );
    let quads0 = scene_quad_signature(&scene0);
    let any_overlay0 = quads0.iter().any(|q| {
        is_zero_edges(q.border) && q.background.a > 0 && (q.rect.w, q.rect.h) == (2400, 560)
    });
    assert!(!any_overlay0, "expected no overlay quads before hover");

    dispatch_hover(&mut ui, &mut app, &mut services);
    for _ in 0..12 {
        app.advance_frame();
        let scene = build_scene_for_text_field(
            &mut ui,
            &mut app,
            &mut services,
            window,
            bounds,
            value.clone(),
            fret_ui_material3::TextFieldVariant::Outlined,
            false,
            false,
        );
        let quads = scene_quad_signature(&scene);
        let any_overlay = quads.iter().any(|q| {
            is_zero_edges(q.border) && q.background.a > 0 && (q.rect.w, q.rect.h) == (2400, 560)
        });
        assert!(!any_overlay, "expected no overlay quads during hover");
    }
}

#[test]
fn filled_text_field_focus_uses_focus_indicator_thickness() {
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
            Size::new(Px(300.0), Px(200.0)),
        );
        let value = app.models_mut().insert(String::new());

        let value_model = value.clone();
        let render =
            |ui: &mut UiTree<TestHost>, app: &mut TestHost, services: &mut dyn UiServices| {
                fret_ui::declarative::render_root(ui, app, services, window, bounds, "root", |cx| {
                    let field = fret_ui_material3::TextField::new(value_model.clone())
                        .variant(fret_ui_material3::TextFieldVariant::Filled)
                        .label("Email")
                        .placeholder("name@example.com")
                        .test_id("tf")
                        .into_element(cx);

                    let mut fixed = ContainerProps::default();
                    fixed.layout.size.width = Length::Px(Px(240.0));
                    fixed.layout.size.height = Length::Px(Px(56.0));
                    vec![cx.container(fixed, move |_cx| vec![field])]
                })
            };

        let root0 = render(&mut ui, &mut app, &mut services);
        ui.set_root(root0);
        ui.request_semantics_snapshot();
        ui.layout_all(&mut app, &mut services, bounds, 1.0);

        let mut scene0 = Scene::default();
        ui.paint_all(&mut app, &mut services, bounds, &mut scene0, 1.0);
        let quads0 = scene_quad_signature(&scene0);
        let container0 = find_filled_text_field_container(&quads0)
            .expect("expected filled text field container");
        assert_eq!(
            container0.border.bottom, 10,
            "expected idle indicator thickness ({label})"
        );

        let text_field_node: NodeId = ui
            .semantics_snapshot()
            .and_then(|snapshot| {
                snapshot
                    .nodes
                    .iter()
                    .find_map(|node| (node.test_id.as_deref() == Some("tf")).then_some(node.id))
            })
            .unwrap_or_else(|| panic!("expected tf in semantics snapshot ({label})"));
        ui.set_focus(Some(text_field_node));
        ui.dispatch_event(&mut app, &mut services, &key_down(KeyCode::ArrowRight));
        ui.dispatch_event(&mut app, &mut services, &key_up(KeyCode::ArrowRight));

        let mut settled: Option<i32> = None;
        for frame in 0..64 {
            app.advance_frame();

            let root1 = render(&mut ui, &mut app, &mut services);
            ui.set_root(root1);
            ui.layout_all(&mut app, &mut services, bounds, 1.0);

            let mut scene1 = Scene::default();
            ui.paint_all(&mut app, &mut services, bounds, &mut scene1, 1.0);

            let quads1 = scene_quad_signature(&scene1);
            let container1 = find_filled_text_field_container(&quads1)
                .expect("expected filled text field container");

            if frame < 28 {
                continue;
            }

            if let Some(prev) = settled {
                assert_eq!(
                    container1.border.bottom, prev,
                    "expected focused indicator thickness to be stable after animations settle ({label})"
                );
            } else {
                settled = Some(container1.border.bottom);
            }
        }

        assert_eq!(
            settled.expect("expected focused indicator thickness after animations settle"),
            20,
            "expected focused indicator thickness ({label})"
        );
    }
}

#[test]
fn filled_text_field_error_hover_uses_state_layer_overlay() {
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
    let value = app.models_mut().insert(String::new());

    let scene0 = build_scene_for_text_field(
        &mut ui,
        &mut app,
        &mut services,
        window,
        bounds,
        value.clone(),
        fret_ui_material3::TextFieldVariant::Filled,
        false,
        true,
    );
    let quads0 = scene_quad_signature(&scene0);
    let container0 =
        find_filled_text_field_container(&quads0).expect("expected filled text field container");
    assert!(find_state_layer_overlay(&quads0, container0).is_none());

    dispatch_hover(&mut ui, &mut app, &mut services);
    let _overlay = find_filled_hover_overlay_after_frames(
        &mut ui,
        &mut app,
        &mut services,
        window,
        bounds,
        value.clone(),
        false,
        true,
    )
    .expect("expected hover overlay in error state");

    let scene1 = build_scene_for_text_field(
        &mut ui,
        &mut app,
        &mut services,
        window,
        bounds,
        value,
        fret_ui_material3::TextFieldVariant::Filled,
        false,
        true,
    );
    let quads1 = scene_quad_signature(&scene1);
    let container1 =
        find_filled_text_field_container(&quads1).expect("expected filled text field container");
    assert_eq!(container0.background, container1.background);
}

#[test]
fn filled_text_field_hover_overlay_survives_focus_transition() {
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
    let value = app.models_mut().insert(String::new());

    let scene0 = build_scene_for_text_field(
        &mut ui,
        &mut app,
        &mut services,
        window,
        bounds,
        value.clone(),
        fret_ui_material3::TextFieldVariant::Filled,
        false,
        false,
    );
    let quads0 = scene_quad_signature(&scene0);
    let container0 =
        find_filled_text_field_container(&quads0).expect("expected filled text field container");

    dispatch_hover(&mut ui, &mut app, &mut services);
    let _overlay0 = find_filled_hover_overlay_after_frames(
        &mut ui,
        &mut app,
        &mut services,
        window,
        bounds,
        value.clone(),
        false,
        false,
    )
    .expect("expected hover overlay");

    dispatch_focus_click(&mut ui, &mut app, &mut services);
    let mut settled_container: Option<QuadSig> = None;
    let mut settled_overlay_count: Option<usize> = None;

    for frame in 0..64 {
        app.advance_frame();
        let scene1 = build_scene_for_text_field(
            &mut ui,
            &mut app,
            &mut services,
            window,
            bounds,
            value.clone(),
            fret_ui_material3::TextFieldVariant::Filled,
            false,
            false,
        );
        let quads1 = scene_quad_signature(&scene1);
        let container1 = find_filled_text_field_container(&quads1)
            .expect("expected filled text field container");

        let overlays: Vec<_> = quads1
            .iter()
            .copied()
            .filter(|q| {
                is_zero_edges(q.border)
                    && q.background.a > 0
                    && q.corner_radii == container1.corner_radii
                    && q.rect == rect_inset_by_edges(container1.rect, container1.border)
            })
            .collect();

        if frame < 28 {
            continue;
        }

        if let (Some(prev_container), Some(prev_overlays)) =
            (settled_container.as_ref(), settled_overlay_count.as_ref())
        {
            assert_eq!(
                container1, *prev_container,
                "expected focused container to be stable after animations settle"
            );
            assert_eq!(
                overlays.len(),
                *prev_overlays,
                "expected hover overlay count to be stable after animations settle"
            );
        } else {
            settled_container = Some(container1);
            settled_overlay_count = Some(overlays.len());
        }
    }

    let container1 = settled_container.expect("expected a settled focused container");
    assert_eq!(container0.background, container1.background);
    assert_eq!(
        settled_overlay_count.expect("expected settled hover overlay count"),
        1,
        "expected a single hover overlay after focus"
    );
}
