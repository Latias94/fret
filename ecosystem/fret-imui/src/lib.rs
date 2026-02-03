//! Immediate-mode authoring facade for Fret.
//!
//! This crate provides a small, policy-light API that feels closer to `egui` / Dear ImGui while
//! compiling down to Fret's declarative element tree (`AnyElement` via `ElementContext`).
//!
//! Notes:
//! - This crate intentionally does not depend on platform or renderer crates.
//! - Styling/recipes should live in separate ecosystem crates (e.g. shadcn/material adapters).

use std::hash::Hash;
use std::sync::Arc;

use fret_core::{Px, Rect, SemanticsRole};
use fret_runtime::Model;
use fret_ui::action::UiActionHostExt as _;
use fret_ui::element::{
    AnyElement, ColumnProps, ContainerProps, Elements, Length, PressableA11y, PressableProps,
    RowProps,
};
use fret_ui::{ElementContext, Theme, UiHost};

pub mod prelude {
    pub use crate::{ImUi, Response, imui, imui_build, imui_vstack};
    pub use fret_authoring::UiWriter;
}

#[derive(Debug, Clone, Copy, Default)]
pub struct Response {
    pub hovered: bool,
    pub pressed: bool,
    pub focused: bool,
    pub clicked: bool,
    pub changed: bool,
    pub rect: Option<Rect>,
}

impl Response {
    pub fn clicked(self) -> bool {
        self.clicked
    }

    pub fn changed(self) -> bool {
        self.changed
    }
}

const fn fnv1a64(bytes: &[u8]) -> u64 {
    let mut hash = 0xcbf2_9ce4_8422_2325u64;
    let mut i = 0usize;
    while i < bytes.len() {
        hash ^= bytes[i] as u64;
        hash = hash.wrapping_mul(0x0000_0100_0000_01b3u64);
        i += 1;
    }
    hash
}

const KEY_CLICKED: u64 = fnv1a64(b"fret-imui.clicked.v1");
const KEY_CHANGED: u64 = fnv1a64(b"fret-imui.changed.v1");

pub fn imui<'a, H: UiHost>(
    cx: &mut ElementContext<'a, H>,
    f: impl for<'cx> FnOnce(&mut ImUi<'cx, 'a, H>),
) -> Elements {
    let mut out = Vec::new();
    imui_build(cx, &mut out, f);
    out.into()
}

/// Convenience entry point that wraps the produced elements in a `Column` so siblings are laid out.
///
/// This avoids the common "all children overlap at (0,0)" footgun when embedding multiple imui
/// children under a non-layout parent (e.g. `Container`) or when returning multiple root children.
pub fn imui_vstack<'a, H: UiHost>(
    cx: &mut ElementContext<'a, H>,
    f: impl for<'cx> FnOnce(&mut ImUi<'cx, 'a, H>),
) -> Elements {
    let mut props = ColumnProps::default();
    props.layout.size.width = Length::Fill;
    props.layout.size.height = Length::Fill;

    let element = cx.column(props, |cx| imui(cx, f));
    vec![element].into()
}

pub fn imui_build<'a, H: UiHost>(
    cx: &mut ElementContext<'a, H>,
    out: &mut Vec<AnyElement>,
    f: impl for<'cx> FnOnce(&mut ImUi<'cx, 'a, H>),
) {
    let mut ui = ImUi { cx, out };
    f(&mut ui);
}

pub struct ImUi<'cx, 'a, H: UiHost> {
    cx: &'cx mut ElementContext<'a, H>,
    out: &'cx mut Vec<AnyElement>,
}

impl<'cx, 'a, H: UiHost> ImUi<'cx, 'a, H> {
    pub fn cx_mut(&mut self) -> &mut ElementContext<'a, H> {
        self.cx
    }

    pub fn add(&mut self, element: AnyElement) {
        self.out.push(element);
    }

    pub fn mount<I>(&mut self, f: impl FnOnce(&mut ElementContext<'_, H>) -> I)
    where
        I: IntoIterator<Item = AnyElement>,
    {
        self.out.extend(f(self.cx).into_iter());
    }

    pub fn id<K: Hash>(&mut self, key: K, f: impl for<'cx2, 'a2> FnOnce(&mut ImUi<'cx2, 'a2, H>)) {
        let out = &mut *self.out;
        self.cx.keyed(key, |cx| {
            let mut ui = ImUi { cx, out };
            f(&mut ui);
        });
    }

    pub fn for_each_keyed<I, K, T>(
        &mut self,
        items: I,
        mut f: impl FnMut(&mut ImUi<'_, '_, H>, &K, T),
    ) where
        I: IntoIterator<Item = (K, T)>,
        K: Hash,
    {
        let f = &mut f;
        for (key, item) in items {
            self.id(&key, |ui| f(ui, &key, item));
        }
    }

    /// Iterates over a slice using callsite-based (unkeyed) identity.
    ///
    /// This is convenient for static lists where order never changes. For dynamic collections
    /// (insert/remove/reorder), prefer `for_each_keyed(...)` or wrap each item in `ui.id(key, ...)`
    /// to preserve per-element state.
    ///
    /// In debug builds, the underlying runtime emits a warning if the list order changes between
    /// frames (aligning with the existing `ElementContext::for_each_unkeyed` diagnostics).
    pub fn for_each_unkeyed<T: Hash>(
        &mut self,
        items: &[T],
        mut f: impl FnMut(&mut ImUi<'_, '_, H>, usize, &T),
    ) {
        let f = &mut f;
        let out = &mut *self.out;
        self.cx.for_each_unkeyed(items, |cx, index, item| {
            let mut ui = ImUi { cx, out };
            f(&mut ui, index, item);
        });
    }

    pub fn text(&mut self, text: impl Into<Arc<str>>) {
        self.out.push(self.cx.text(text));
    }

    pub fn separator(&mut self) {
        let mut props = ContainerProps::default();
        let theme = Theme::global(&*self.cx.app);
        props.background = Some(theme.color_required("border"));
        props.layout.size.width = Length::Fill;
        props.layout.size.height = Length::Px(Px(1.0));
        self.out.push(self.cx.container(props, |_| Vec::new()));
    }

    pub fn row(&mut self, f: impl for<'cx2, 'a2> FnOnce(&mut ImUi<'cx2, 'a2, H>)) {
        let element = self.cx.row(RowProps::default(), |cx| imui(cx, f));
        self.out.push(element);
    }

    pub fn column(&mut self, f: impl for<'cx2, 'a2> FnOnce(&mut ImUi<'cx2, 'a2, H>)) {
        let element = self.cx.column(ColumnProps::default(), |cx| imui(cx, f));
        self.out.push(element);
    }

    pub fn button(&mut self, label: impl Into<Arc<str>>) -> Response {
        let label = label.into();
        let mut response = Response::default();

        let mut props = PressableProps::default();
        props.a11y = PressableA11y {
            role: Some(SemanticsRole::Button),
            label: Some(label.clone()),
            ..Default::default()
        };

        let element = self.cx.pressable_with_id(props, |cx, state, id| {
            cx.pressable_on_activate(Arc::new(move |host, acx, _reason| {
                host.record_transient_event(acx, KEY_CLICKED);
                host.notify(acx);
            }));

            response.hovered = state.hovered;
            response.pressed = state.pressed;
            response.focused = state.focused;
            response.clicked = cx.take_transient_for(id, KEY_CLICKED);
            response.rect = cx.last_bounds_for_element(id);

            vec![cx.text(label.clone())]
        });

        self.out.push(element);
        response
    }

    pub fn checkbox_model(&mut self, label: impl Into<Arc<str>>, model: &Model<bool>) -> Response {
        let label = label.into();
        let value = self
            .cx
            .read_model(model, fret_ui::Invalidation::Paint, |_app, v| *v)
            .unwrap_or(false);

        let mut response = Response::default();

        let mut props = PressableProps::default();
        props.a11y = PressableA11y {
            role: Some(SemanticsRole::Checkbox),
            label: Some(label.clone()),
            checked: Some(value),
            ..Default::default()
        };

        let model = model.clone();
        let element = self.cx.pressable_with_id(props, |cx, state, id| {
            let model = model.clone();
            cx.pressable_on_activate(Arc::new(move |host, acx, _reason| {
                let _ = host.update_model(&model, |v: &mut bool| *v = !*v);
                host.record_transient_event(acx, KEY_CHANGED);
                host.notify(acx);
            }));

            response.hovered = state.hovered;
            response.pressed = state.pressed;
            response.focused = state.focused;
            response.changed = cx.take_transient_for(id, KEY_CHANGED);
            response.rect = cx.last_bounds_for_element(id);

            let prefix: Arc<str> = if value {
                Arc::from("[x] ")
            } else {
                Arc::from("[ ] ")
            };
            vec![cx.text(Arc::from(format!("{prefix}{label}")))]
        });

        self.out.push(element);
        response
    }
}

impl<'cx, 'a, H: UiHost> fret_authoring::UiWriter<H> for ImUi<'cx, 'a, H> {
    fn with_cx_mut<R>(&mut self, f: impl FnOnce(&mut ElementContext<'_, H>) -> R) -> R {
        f(self.cx)
    }

    fn add(&mut self, element: AnyElement) {
        self.out.push(element);
    }
}

#[cfg(test)]
mod tests {
    use std::{
        any::{Any, TypeId},
        cell::Cell,
        collections::{HashMap, HashSet},
        rc::Rc,
    };

    use fret_core::{
        AppWindowId, CaretAffinity, Event, Modifiers, MouseButton, Point, PointerId, PointerType,
        Px, Rect, Size, TextConstraints, TextMetrics, TextService,
    };
    use fret_runtime::{
        ClipboardToken, CommandRegistry, CommandsHost, DragHost, DragKindId, DragSession,
        DragSessionId, Effect, EffectSink, FrameId, GlobalsHost, ModelHost, ModelId, ModelStore,
        ModelsHost, PlatformCapabilities, TickId, TimeHost, TimerToken,
    };
    use fret_ui::declarative::render_root;
    use fret_ui::{ElementContext, UiTree};

    #[derive(Default)]
    struct FakeTextService;

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
        fn new() -> Self {
            Self::default()
        }

        fn advance_frame(&mut self) {
            self.tick_id.0 = self.tick_id.0.saturating_add(1);
            self.frame_id.0 = self.frame_id.0.saturating_add(1);
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

    impl ModelHost for TestHost {
        fn models(&self) -> &ModelStore {
            &self.models
        }

        fn models_mut(&mut self) -> &mut ModelStore {
            &mut self.models
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

        fn next_timer_token(&mut self) -> TimerToken {
            let token = TimerToken(self.next_timer_token);
            self.next_timer_token = self.next_timer_token.saturating_add(1);
            token
        }

        fn next_clipboard_token(&mut self) -> ClipboardToken {
            let token = ClipboardToken(self.next_clipboard_token);
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

        fn drag_mut(&mut self, pointer_id: PointerId) -> Option<&mut DragSession> {
            self.drags.get_mut(&pointer_id)
        }

        fn cancel_drag(&mut self, pointer_id: PointerId) {
            self.drags.remove(&pointer_id);
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

    fn run_frame(
        ui: &mut UiTree<TestHost>,
        app: &mut TestHost,
        services: &mut FakeTextService,
        window: AppWindowId,
        bounds: Rect,
        root_name: &str,
        render: impl FnOnce(&mut ElementContext<'_, TestHost>) -> crate::Elements,
    ) -> fret_core::NodeId {
        let root = render_root(ui, app, services, window, bounds, root_name, render);
        ui.layout_all(app, services, bounds, 1.0);
        root
    }

    fn click_at(
        ui: &mut UiTree<TestHost>,
        app: &mut TestHost,
        services: &mut FakeTextService,
        at: Point,
    ) {
        ui.dispatch_event(
            app,
            services,
            &Event::Pointer(fret_core::PointerEvent::Down {
                pointer_id: PointerId(0),
                position: at,
                button: MouseButton::Left,
                modifiers: Modifiers::default(),
                click_count: 1,
                pointer_type: PointerType::Mouse,
            }),
        );
        ui.dispatch_event(
            app,
            services,
            &Event::Pointer(fret_core::PointerEvent::Up {
                pointer_id: PointerId(0),
                position: at,
                button: MouseButton::Left,
                modifiers: Modifiers::default(),
                is_click: true,
                click_count: 1,
                pointer_type: PointerType::Mouse,
            }),
        );
    }

    fn pointer_down_at(
        ui: &mut UiTree<TestHost>,
        app: &mut TestHost,
        services: &mut FakeTextService,
        at: Point,
    ) {
        ui.dispatch_event(
            app,
            services,
            &Event::Pointer(fret_core::PointerEvent::Down {
                pointer_id: PointerId(0),
                position: at,
                button: MouseButton::Left,
                modifiers: Modifiers::default(),
                click_count: 1,
                pointer_type: PointerType::Mouse,
            }),
        );
    }

    fn pointer_up_at(
        ui: &mut UiTree<TestHost>,
        app: &mut TestHost,
        services: &mut FakeTextService,
        at: Point,
    ) {
        ui.dispatch_event(
            app,
            services,
            &Event::Pointer(fret_core::PointerEvent::Up {
                pointer_id: PointerId(0),
                position: at,
                button: MouseButton::Left,
                modifiers: Modifiers::default(),
                is_click: true,
                click_count: 1,
                pointer_type: PointerType::Mouse,
            }),
        );
    }

    fn first_child_point(ui: &UiTree<TestHost>, root: fret_core::NodeId) -> Point {
        let child = ui.children(root)[0];
        let bounds = ui.debug_node_bounds(child).expect("child bounds");
        Point::new(Px(bounds.origin.x.0 + 1.0), Px(bounds.origin.y.0 + 1.0))
    }

    #[test]
    fn click_sets_clicked_true_once() {
        let window = AppWindowId::default();
        let bounds = Rect::new(
            Point::new(Px(0.0), Px(0.0)),
            Size::new(Px(240.0), Px(120.0)),
        );

        let mut ui = UiTree::new();
        ui.set_window(window);

        let mut app = TestHost::new();
        app.set_global(PlatformCapabilities::default());
        let mut services = FakeTextService::default();

        let clicked = Rc::new(Cell::new(false));
        let clicked_out = clicked.clone();
        let root = run_frame(
            &mut ui,
            &mut app,
            &mut services,
            window,
            bounds,
            "imui-click-once",
            |cx| {
                crate::imui(cx, |ui| {
                    clicked_out.set(ui.button("OK").clicked());
                })
            },
        );
        assert!(!clicked.get());

        let at = first_child_point(&ui, root);
        click_at(&mut ui, &mut app, &mut services, at);

        app.advance_frame();
        let clicked_out = clicked.clone();
        let _root = run_frame(
            &mut ui,
            &mut app,
            &mut services,
            window,
            bounds,
            "imui-click-once",
            |cx| {
                crate::imui(cx, |ui| {
                    clicked_out.set(ui.button("OK").clicked());
                })
            },
        );
        assert!(clicked.get());

        app.advance_frame();
        let clicked_out = clicked.clone();
        let _root = run_frame(
            &mut ui,
            &mut app,
            &mut services,
            window,
            bounds,
            "imui-click-once",
            |cx| {
                crate::imui(cx, |ui| {
                    clicked_out.set(ui.button("OK").clicked());
                })
            },
        );
        assert!(!clicked.get());
    }

    #[test]
    fn ui_kit_builder_can_be_rendered_from_imui() {
        let window = AppWindowId::default();
        let bounds = Rect::new(
            Point::new(Px(0.0), Px(0.0)),
            Size::new(Px(240.0), Px(120.0)),
        );

        let mut ui = UiTree::new();
        ui.set_window(window);

        let mut app = TestHost::new();
        app.set_global(PlatformCapabilities::default());
        let mut services = FakeTextService::default();

        let root = run_frame(
            &mut ui,
            &mut app,
            &mut services,
            window,
            bounds,
            "imui-ui-kit-bridge",
            |cx| {
                crate::imui(cx, |ui| {
                    use fret_ui_kit::imui::UiWriterUiKitExt as _;

                    let builder = fret_ui_kit::ui::text(ui.cx_mut(), "Hello").text_sm();
                    ui.add_ui(builder);
                })
            },
        );

        assert_eq!(ui.children(root).len(), 1);
    }

    #[test]
    fn holding_press_does_not_repeat_clicked() {
        let window = AppWindowId::default();
        let bounds = Rect::new(
            Point::new(Px(0.0), Px(0.0)),
            Size::new(Px(240.0), Px(120.0)),
        );

        let mut ui = UiTree::new();
        ui.set_window(window);

        let mut app = TestHost::new();
        app.set_global(PlatformCapabilities::default());
        let mut services = FakeTextService::default();

        let clicked = Rc::new(Cell::new(false));
        let clicked_out = clicked.clone();
        let root = run_frame(
            &mut ui,
            &mut app,
            &mut services,
            window,
            bounds,
            "imui-hold-press",
            |cx| {
                crate::imui(cx, |ui| {
                    clicked_out.set(ui.button("OK").clicked());
                })
            },
        );
        assert!(!clicked.get());

        let at = first_child_point(&ui, root);
        pointer_down_at(&mut ui, &mut app, &mut services, at);

        app.advance_frame();
        let clicked_out = clicked.clone();
        let _root = run_frame(
            &mut ui,
            &mut app,
            &mut services,
            window,
            bounds,
            "imui-hold-press",
            |cx| {
                crate::imui(cx, |ui| {
                    clicked_out.set(ui.button("OK").clicked());
                })
            },
        );
        assert!(!clicked.get());

        pointer_up_at(&mut ui, &mut app, &mut services, at);

        app.advance_frame();
        let clicked_out = clicked.clone();
        let _root = run_frame(
            &mut ui,
            &mut app,
            &mut services,
            window,
            bounds,
            "imui-hold-press",
            |cx| {
                crate::imui(cx, |ui| {
                    clicked_out.set(ui.button("OK").clicked());
                })
            },
        );
        assert!(clicked.get());

        app.advance_frame();
        let clicked_out = clicked.clone();
        let _root = run_frame(
            &mut ui,
            &mut app,
            &mut services,
            window,
            bounds,
            "imui-hold-press",
            |cx| {
                crate::imui(cx, |ui| {
                    clicked_out.set(ui.button("OK").clicked());
                })
            },
        );
        assert!(!clicked.get());
    }

    #[test]
    fn checkbox_changed_is_delivered_once_and_updates_model() {
        let window = AppWindowId::default();
        let bounds = Rect::new(
            Point::new(Px(0.0), Px(0.0)),
            Size::new(Px(240.0), Px(120.0)),
        );

        let mut ui = UiTree::new();
        ui.set_window(window);

        let mut app = TestHost::new();
        app.set_global(PlatformCapabilities::default());
        let mut services = FakeTextService::default();

        let model = app.models_mut().insert(false);

        let changed = Rc::new(Cell::new(false));
        let value = Rc::new(Cell::new(false));

        let changed_out = changed.clone();
        let value_out = value.clone();
        let root = run_frame(
            &mut ui,
            &mut app,
            &mut services,
            window,
            bounds,
            "imui-checkbox",
            |cx| {
                crate::imui(cx, |ui| {
                    changed_out.set(ui.checkbox_model("Enabled", &model).changed());
                    let now = ui
                        .cx_mut()
                        .app
                        .models()
                        .get_copied(&model)
                        .unwrap_or_default();
                    value_out.set(now);
                })
            },
        );
        assert!(!changed.get());
        assert!(!value.get());

        let at = first_child_point(&ui, root);
        click_at(&mut ui, &mut app, &mut services, at);

        app.advance_frame();
        let changed_out = changed.clone();
        let value_out = value.clone();
        let _root = run_frame(
            &mut ui,
            &mut app,
            &mut services,
            window,
            bounds,
            "imui-checkbox",
            |cx| {
                crate::imui(cx, |ui| {
                    changed_out.set(ui.checkbox_model("Enabled", &model).changed());
                    let now = ui
                        .cx_mut()
                        .app
                        .models()
                        .get_copied(&model)
                        .unwrap_or_default();
                    value_out.set(now);
                })
            },
        );
        assert!(changed.get());
        assert!(value.get());

        app.advance_frame();
        let changed_out = changed.clone();
        let value_out = value.clone();
        let _root = run_frame(
            &mut ui,
            &mut app,
            &mut services,
            window,
            bounds,
            "imui-checkbox",
            |cx| {
                crate::imui(cx, |ui| {
                    changed_out.set(ui.checkbox_model("Enabled", &model).changed());
                    let now = ui
                        .cx_mut()
                        .app
                        .models()
                        .get_copied(&model)
                        .unwrap_or_default();
                    value_out.set(now);
                })
            },
        );
        assert!(!changed.get());
        assert!(value.get());
    }

    // Note: `for_each_keyed` is exercised indirectly by downstream ecosystem crates. The core
    // smoke tests above focus on interaction correctness (`clicked` / `changed`).
}
