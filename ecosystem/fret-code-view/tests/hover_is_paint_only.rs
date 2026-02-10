use std::{
    any::{Any, TypeId},
    collections::{HashMap, HashSet},
};

use fret_core::{
    AppWindowId, Event, Modifiers, MouseButtons, Point, PointerEvent, PointerId, PointerType, Px,
    Rect, Size, UiServices,
};
use fret_runtime::{
    CommandRegistry, CommandsHost, DragHost, DragKindId, DragSession, DragSessionId, Effect,
    EffectSink, FrameId, GlobalsHost, ModelHost, ModelId, ModelStore, ModelsHost,
    PlatformCapabilities, TickId, TimeHost,
};
use fret_ui::UiTree;
use fret_ui_shadcn::shadcn_themes::{ShadcnBaseColor, ShadcnColorScheme, apply_shadcn_new_york_v4};

#[derive(Default)]
struct TestHost {
    globals: HashMap<TypeId, Box<dyn Any>>,
    models: ModelStore,
    commands: CommandRegistry,
    redraw: HashSet<AppWindowId>,
    effects: Vec<Effect>,
    drags: HashMap<PointerId, DragSession>,
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
    fn drag(&self, pointer_id: PointerId) -> Option<&DragSession> {
        self.drags.get(&pointer_id)
    }

    fn any_drag_session(&self, mut predicate: impl FnMut(&DragSession) -> bool) -> bool {
        self.drags.values().any(|d| predicate(d))
    }

    fn find_drag_pointer_id(
        &self,
        mut predicate: impl FnMut(&DragSession) -> bool,
    ) -> Option<PointerId> {
        self.drags
            .values()
            .find(|d| predicate(d))
            .map(|d| d.pointer_id)
    }

    fn cancel_drag_sessions(
        &mut self,
        mut predicate: impl FnMut(&DragSession) -> bool,
    ) -> Vec<PointerId> {
        let to_cancel: Vec<PointerId> = self
            .drags
            .values()
            .filter(|d| predicate(d))
            .map(|d| d.pointer_id)
            .collect();
        for pointer_id in &to_cancel {
            self.cancel_drag(*pointer_id);
        }
        to_cancel
    }

    fn drag_mut(&mut self, pointer_id: PointerId) -> Option<&mut DragSession> {
        self.drags.get_mut(&pointer_id)
    }

    fn cancel_drag(&mut self, pointer_id: PointerId) {
        self.drags.remove(&pointer_id);
    }

    fn begin_drag_with_kind<T: Any>(
        &mut self,
        pointer_id: PointerId,
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
        pointer_id: PointerId,
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

fn pointer_move(pointer_id: PointerId, position: Point) -> Event {
    Event::Pointer(PointerEvent::Move {
        pointer_id,
        position,
        buttons: MouseButtons::default(),
        modifiers: Modifiers::default(),
        pointer_type: PointerType::Mouse,
    })
}

#[test]
fn code_block_hover_does_not_trigger_declarative_layout_invalidations() {
    let mut app = TestHost::default();
    app.set_global(PlatformCapabilities::default());

    apply_shadcn_new_york_v4(&mut app, ShadcnBaseColor::Neutral, ShadcnColorScheme::Light);

    let window = AppWindowId::default();
    let mut services = FakeUiServices::default();

    let bounds = Rect::new(
        Point::new(Px(0.0), Px(0.0)),
        Size::new(Px(500.0), Px(300.0)),
    );

    let mut ui: UiTree<TestHost> = UiTree::new();
    ui.set_window(window);
    ui.set_debug_enabled(true);

    let code = "fn main() {\n    println!(\"hello\");\n}\n";

    let render = |ui: &mut UiTree<TestHost>, app: &mut TestHost, services: &mut dyn UiServices| {
        fret_ui::declarative::render_root(ui, app, services, window, bounds, "root", |cx| {
            let mut options = fret_code_view::CodeBlockUiOptions::default();
            options.show_header = true;
            options.header_divider = true;
            options.header_background = fret_code_view::CodeBlockHeaderBackground::Secondary;
            options.show_copy_button = true;
            options.copy_button_on_hover = true;
            options.copy_button_placement = fret_code_view::CodeBlockCopyButtonPlacement::Header;
            options.show_scrollbar_x = true;
            options.scrollbar_x_on_hover = true;

            vec![fret_code_view::code_block_with(
                cx,
                code,
                Some("rust"),
                false,
                options,
            )]
        })
    };

    let root = render(&mut ui, &mut app, &mut services);
    ui.set_root(root);
    ui.layout_all(&mut app, &mut services, bounds, 1.0);

    let pointer_id = PointerId(1);
    ui.dispatch_event(
        &mut app,
        &mut services,
        &pointer_move(pointer_id, Point::new(Px(9999.0), Px(9999.0))),
    );

    ui.dispatch_event(
        &mut app,
        &mut services,
        &pointer_move(pointer_id, Point::new(Px(50.0), Px(50.0))),
    );

    let _root = render(&mut ui, &mut app, &mut services);

    let stats = ui.debug_stats();
    assert_eq!(stats.hover_declarative_layout_invalidations, 0);
    assert_eq!(stats.hover_declarative_hit_test_invalidations, 0);
    assert!(stats.hover_declarative_paint_invalidations > 0);
}
