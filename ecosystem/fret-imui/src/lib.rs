//! Immediate-mode authoring facade for Fret.
//!
//! This crate provides a small, policy-light *authoring frontend* that compiles down to Fret's
//! declarative element tree (`AnyElement` via `ElementContext`).
//!
//! The "egui/imgui-like experience" (richer response signals, widget helpers, floating areas,
//! overlays, etc.) is intentionally hosted in ecosystem facade crates (e.g. `fret-ui-kit` behind its
//! `imui` feature) to keep this crate minimal and third-party-friendly.
//!
//! Notes:
//! - This crate intentionally does not depend on platform or renderer crates.
//! - Styling/recipes should live in separate ecosystem crates (e.g. shadcn/material adapters).

use std::hash::Hash;

pub use fret_authoring::Response;
use fret_authoring::UiWriter;
use fret_ui::element::{AnyElement, ColumnProps, Elements, Length, RowProps};
use fret_ui::{ElementContext, UiHost};

#[cfg(feature = "query")]
pub mod query;
#[cfg(feature = "selector")]
pub mod selector;

pub mod prelude {
    #[cfg(feature = "query")]
    pub use crate::query::UiWriterQueryExt as _;
    #[cfg(feature = "selector")]
    pub use crate::selector::UiWriterSelectorExt as _;
    pub use crate::{ImUi, Response, imui, imui_build, imui_vstack};
    pub use fret_authoring::UiWriter;
}

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

    pub fn push_id<K: Hash>(
        &mut self,
        key: K,
        f: impl for<'cx2, 'a2> FnOnce(&mut ImUi<'cx2, 'a2, H>),
    ) {
        self.id(key, f);
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

    pub fn row(&mut self, f: impl for<'cx2, 'a2> FnOnce(&mut ImUi<'cx2, 'a2, H>)) {
        let element = self.cx.row(RowProps::default(), |cx| imui(cx, f));
        self.out.push(element);
    }

    pub fn column(&mut self, f: impl for<'cx2, 'a2> FnOnce(&mut ImUi<'cx2, 'a2, H>)) {
        let element = self.cx.column(ColumnProps::default(), |cx| imui(cx, f));
        self.out.push(element);
    }
}

impl<'cx, 'a, H: UiHost> UiWriter<H> for ImUi<'cx, 'a, H> {
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
        cell::{Cell, RefCell},
        collections::{HashMap, HashSet},
        rc::Rc,
        sync::Arc,
    };

    use fret_core::{
        AppWindowId, CaretAffinity, Event, KeyCode, Modifiers, MouseButton, MouseButtons, Point,
        PointerId, PointerType, Px, Rect, SemanticsRole, Size, TextConstraints, TextMetrics,
        TextService,
    };
    use fret_runtime::{
        ClipboardToken, CommandRegistry, CommandsHost, DragHost, DragKindId, DragSession,
        DragSessionId, Effect, EffectSink, FrameId, GlobalsHost, ModelHost, ModelId, ModelStore,
        ModelsHost, PlatformCapabilities, TickId, TimeHost, TimerToken,
    };
    use fret_ui::declarative::render_root;
    use fret_ui::element::Length;
    use fret_ui::{ElementContext, UiTree};
    use fret_ui_kit::OverlayController;
    use fret_ui_kit::imui::UiWriterImUiFacadeExt as _;
    use fret_ui_kit::imui::{
        GridOptions, HorizontalOptions, InputTextOptions, MenuItemOptions, PopupMenuOptions,
        ScrollOptions, SelectOptions, SliderOptions, SwitchOptions, ToggleOptions, VerticalOptions,
    };

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
        OverlayController::begin_frame(app, window);
        let root = render_root(ui, app, services, window, bounds, root_name, render);
        OverlayController::render(ui, app, services, window, bounds);
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

    fn double_click_at(
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
                click_count: 2,
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
                click_count: 2,
                pointer_type: PointerType::Mouse,
            }),
        );
    }

    fn right_click_at(
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
                button: MouseButton::Right,
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
                button: MouseButton::Right,
                modifiers: Modifiers::default(),
                is_click: true,
                click_count: 1,
                pointer_type: PointerType::Mouse,
            }),
        );
    }

    fn pointer_move_at(
        ui: &mut UiTree<TestHost>,
        app: &mut TestHost,
        services: &mut FakeTextService,
        at: Point,
        buttons: MouseButtons,
    ) {
        ui.dispatch_event(
            app,
            services,
            &Event::Pointer(fret_core::PointerEvent::Move {
                pointer_id: PointerId(0),
                position: at,
                buttons,
                modifiers: Modifiers::default(),
                pointer_type: PointerType::Mouse,
            }),
        );
    }

    fn key_down(
        ui: &mut UiTree<TestHost>,
        app: &mut TestHost,
        services: &mut FakeTextService,
        key: KeyCode,
        modifiers: Modifiers,
    ) {
        ui.dispatch_event(
            app,
            services,
            &Event::KeyDown {
                key,
                modifiers,
                repeat: false,
            },
        );
    }

    fn text_input_event(
        ui: &mut UiTree<TestHost>,
        app: &mut TestHost,
        services: &mut FakeTextService,
        text: &str,
    ) {
        ui.dispatch_event(app, services, &Event::TextInput(text.to_string()));
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
        pointer_up_at_with_is_click(ui, app, services, at, true);
    }

    fn pointer_up_at_with_is_click(
        ui: &mut UiTree<TestHost>,
        app: &mut TestHost,
        services: &mut FakeTextService,
        at: Point,
        is_click: bool,
    ) {
        ui.dispatch_event(
            app,
            services,
            &Event::Pointer(fret_core::PointerEvent::Up {
                pointer_id: PointerId(0),
                position: at,
                button: MouseButton::Left,
                modifiers: Modifiers::default(),
                is_click,
                click_count: 1,
                pointer_type: PointerType::Mouse,
            }),
        );
    }

    fn dispatch_all_timers(
        ui: &mut UiTree<TestHost>,
        app: &mut TestHost,
        services: &mut FakeTextService,
    ) -> usize {
        let mut pending: Vec<TimerToken> = Vec::new();
        for effect in &app.effects {
            if let Effect::SetTimer { token, repeat, .. } = effect
                && repeat.is_none()
            {
                pending.push(*token);
            }
        }
        app.effects.retain(
            |effect| !matches!(effect, Effect::SetTimer { repeat, .. } if repeat.is_none()),
        );

        let dispatched = pending.len();
        for token in pending {
            ui.dispatch_event(app, services, &Event::Timer { token });
        }
        dispatched
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
    fn right_click_sets_context_menu_requested_true_once() {
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

        let requested = Rc::new(Cell::new(false));
        let secondary_clicked = Rc::new(Cell::new(false));
        let requested_out = requested.clone();
        let secondary_clicked_out = secondary_clicked.clone();
        let root = run_frame(
            &mut ui,
            &mut app,
            &mut services,
            window,
            bounds,
            "imui-context-menu-right-click",
            |cx| {
                crate::imui(cx, |ui| {
                    let resp = ui.button("OK");
                    requested_out.set(resp.context_menu_requested());
                    secondary_clicked_out.set(resp.secondary_clicked());
                })
            },
        );
        assert!(!requested.get());
        assert!(!secondary_clicked.get());

        let at = first_child_point(&ui, root);
        right_click_at(&mut ui, &mut app, &mut services, at);

        app.advance_frame();
        let requested_out = requested.clone();
        let secondary_clicked_out = secondary_clicked.clone();
        let _root = run_frame(
            &mut ui,
            &mut app,
            &mut services,
            window,
            bounds,
            "imui-context-menu-right-click",
            |cx| {
                crate::imui(cx, |ui| {
                    let resp = ui.button("OK");
                    requested_out.set(resp.context_menu_requested());
                    secondary_clicked_out.set(resp.secondary_clicked());
                })
            },
        );
        assert!(requested.get());
        assert!(secondary_clicked.get());

        app.advance_frame();
        let requested_out = requested.clone();
        let secondary_clicked_out = secondary_clicked.clone();
        let _root = run_frame(
            &mut ui,
            &mut app,
            &mut services,
            window,
            bounds,
            "imui-context-menu-right-click",
            |cx| {
                crate::imui(cx, |ui| {
                    let resp = ui.button("OK");
                    requested_out.set(resp.context_menu_requested());
                    secondary_clicked_out.set(resp.secondary_clicked());
                })
            },
        );
        assert!(!requested.get());
        assert!(!secondary_clicked.get());
    }

    #[test]
    fn context_menu_popup_opens_on_right_click_and_closes_on_outside_click() {
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

        let open = Rc::new(Cell::new(false));
        let open_out = open.clone();
        let root = run_frame(
            &mut ui,
            &mut app,
            &mut services,
            window,
            bounds,
            "imui-popup-context-menu",
            |cx| {
                crate::imui(cx, |ui| {
                    let resp = ui.button("OK");
                    open_out.set(ui.begin_popup_context_menu("ctx", resp, |ui| {
                        ui.text("Menu");
                    }));
                })
            },
        );
        assert!(!open.get());

        let at = first_child_point(&ui, root);
        right_click_at(&mut ui, &mut app, &mut services, at);

        app.advance_frame();
        let open_out = open.clone();
        let _root = run_frame(
            &mut ui,
            &mut app,
            &mut services,
            window,
            bounds,
            "imui-popup-context-menu",
            |cx| {
                crate::imui(cx, |ui| {
                    let resp = ui.button("OK");
                    open_out.set(ui.begin_popup_context_menu("ctx", resp, |ui| {
                        ui.text("Menu");
                    }));
                })
            },
        );
        assert!(open.get());

        click_at(
            &mut ui,
            &mut app,
            &mut services,
            Point::new(Px(230.0), Px(110.0)),
        );

        app.advance_frame();
        let open_out = open.clone();
        let _root = run_frame(
            &mut ui,
            &mut app,
            &mut services,
            window,
            bounds,
            "imui-popup-context-menu",
            |cx| {
                crate::imui(cx, |ui| {
                    let resp = ui.button("OK");
                    open_out.set(ui.begin_popup_context_menu("ctx", resp, |ui| {
                        ui.text("Menu");
                    }));
                })
            },
        );
        assert!(!open.get());
    }

    fn bounds_for_test_id(ui: &UiTree<TestHost>, test_id: &str) -> Rect {
        let snap = ui.semantics_snapshot().expect("semantics snapshot");
        let node = snap
            .nodes
            .iter()
            .find(|n| n.test_id.as_deref() == Some(test_id))
            .unwrap_or_else(|| panic!("expected node with test_id={test_id}"));
        node.bounds
    }

    #[test]
    fn context_menu_popup_item_click_closes_popup() {
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

        let open = Rc::new(Cell::new(false));
        let open_out = open.clone();
        let root = run_frame(
            &mut ui,
            &mut app,
            &mut services,
            window,
            bounds,
            "imui-popup-context-menu-item-close",
            |cx| {
                crate::imui(cx, |ui| {
                    let resp = ui.button("OK");
                    open_out.set(ui.begin_popup_context_menu_ex(
                        "ctx",
                        resp,
                        PopupMenuOptions {
                            estimated_size: Size::new(Px(120.0), Px(60.0)),
                            ..Default::default()
                        },
                        |ui| {
                            let open_model = ui.popup_open_model("ctx");
                            ui.menu_item_ex(
                                "Close",
                                MenuItemOptions {
                                    close_popup: Some(open_model),
                                    test_id: Some(Arc::from("imui-popup-ctx-item-close")),
                                    ..Default::default()
                                },
                            );
                        },
                    ));
                })
            },
        );
        assert!(!open.get());

        let at = first_child_point(&ui, root);
        right_click_at(&mut ui, &mut app, &mut services, at);

        app.advance_frame();
        ui.request_semantics_snapshot();
        let open_out = open.clone();
        let _root = run_frame(
            &mut ui,
            &mut app,
            &mut services,
            window,
            bounds,
            "imui-popup-context-menu-item-close",
            |cx| {
                crate::imui(cx, |ui| {
                    let resp = ui.button("OK");
                    open_out.set(ui.begin_popup_context_menu_ex(
                        "ctx",
                        resp,
                        PopupMenuOptions {
                            estimated_size: Size::new(Px(120.0), Px(60.0)),
                            ..Default::default()
                        },
                        |ui| {
                            let open_model = ui.popup_open_model("ctx");
                            ui.menu_item_ex(
                                "Close",
                                MenuItemOptions {
                                    close_popup: Some(open_model),
                                    test_id: Some(Arc::from("imui-popup-ctx-item-close")),
                                    ..Default::default()
                                },
                            );
                        },
                    ));
                })
            },
        );
        assert!(open.get());

        let item_bounds = bounds_for_test_id(&ui, "imui-popup-ctx-item-close");
        let click_point = Point::new(
            Px(item_bounds.origin.x.0 + item_bounds.size.width.0 * 0.5),
            Px(item_bounds.origin.y.0 + item_bounds.size.height.0 * 0.5),
        );
        let hit = ui.debug_hit_test(click_point).hit.expect("hit node");
        let snap = ui.semantics_snapshot().expect("semantics snapshot");
        let hit_test_id = snap
            .nodes
            .iter()
            .find(|n| n.id == hit)
            .and_then(|n| n.test_id.as_deref());
        assert_eq!(
            hit_test_id,
            Some("imui-popup-ctx-item-close"),
            "expected click to hit the menu item pressable"
        );

        click_at(&mut ui, &mut app, &mut services, click_point);

        app.advance_frame();
        let open_out = open.clone();
        let _root = run_frame(
            &mut ui,
            &mut app,
            &mut services,
            window,
            bounds,
            "imui-popup-context-menu-item-close",
            |cx| {
                crate::imui(cx, |ui| {
                    let resp = ui.button("OK");
                    open_out.set(ui.begin_popup_context_menu("ctx", resp, |_ui| {}));
                })
            },
        );
        assert!(!open.get());
    }

    #[test]
    fn context_menu_popup_keyboard_open_focuses_first_item_and_escape_restores_trigger_focus() {
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

        let open = Rc::new(Cell::new(false));
        let open_out = open.clone();
        let root = run_frame(
            &mut ui,
            &mut app,
            &mut services,
            window,
            bounds,
            "imui-popup-context-menu-keyboard-open",
            |cx| {
                crate::imui(cx, |ui| {
                    let resp = ui.button("OK");
                    open_out.set(ui.begin_popup_context_menu_ex(
                        "ctx",
                        resp,
                        PopupMenuOptions {
                            estimated_size: Size::new(Px(160.0), Px(90.0)),
                            ..Default::default()
                        },
                        |ui| {
                            ui.menu_item_ex(
                                "Item A",
                                MenuItemOptions {
                                    test_id: Some(Arc::from("imui-popup-ctx-item-a")),
                                    ..Default::default()
                                },
                            );
                            ui.menu_item_ex(
                                "Item B",
                                MenuItemOptions {
                                    test_id: Some(Arc::from("imui-popup-ctx-item-b")),
                                    ..Default::default()
                                },
                            );
                        },
                    ));
                })
            },
        );
        assert!(!open.get());

        let at = first_child_point(&ui, root);
        click_at(&mut ui, &mut app, &mut services, at);
        let focus_before_open = ui.focus();
        assert!(
            focus_before_open.is_some(),
            "expected trigger to take focus"
        );

        key_down(
            &mut ui,
            &mut app,
            &mut services,
            KeyCode::ContextMenu,
            Modifiers::default(),
        );

        app.advance_frame();
        ui.request_semantics_snapshot();
        let open_out = open.clone();
        let _root = run_frame(
            &mut ui,
            &mut app,
            &mut services,
            window,
            bounds,
            "imui-popup-context-menu-keyboard-open",
            |cx| {
                crate::imui(cx, |ui| {
                    let resp = ui.button("OK");
                    open_out.set(ui.begin_popup_context_menu_ex(
                        "ctx",
                        resp,
                        PopupMenuOptions {
                            estimated_size: Size::new(Px(160.0), Px(90.0)),
                            ..Default::default()
                        },
                        |ui| {
                            ui.menu_item_ex(
                                "Item A",
                                MenuItemOptions {
                                    test_id: Some(Arc::from("imui-popup-ctx-item-a")),
                                    ..Default::default()
                                },
                            );
                            ui.menu_item_ex(
                                "Item B",
                                MenuItemOptions {
                                    test_id: Some(Arc::from("imui-popup-ctx-item-b")),
                                    ..Default::default()
                                },
                            );
                        },
                    ));
                })
            },
        );
        assert!(open.get());

        let focus = ui.focus().expect("focus");
        let snap = ui.semantics_snapshot().expect("semantics snapshot");
        let focused_test_id = snap
            .nodes
            .iter()
            .find(|n| n.id == focus)
            .and_then(|n| n.test_id.as_deref());
        assert_eq!(focused_test_id, Some("imui-popup-ctx-item-a"));

        key_down(
            &mut ui,
            &mut app,
            &mut services,
            KeyCode::Escape,
            Modifiers::default(),
        );

        app.advance_frame();
        let open_out = open.clone();
        let _root = run_frame(
            &mut ui,
            &mut app,
            &mut services,
            window,
            bounds,
            "imui-popup-context-menu-keyboard-open",
            |cx| {
                crate::imui(cx, |ui| {
                    let resp = ui.button("OK");
                    open_out.set(ui.begin_popup_context_menu("ctx", resp, |_ui| {}));
                })
            },
        );
        assert!(!open.get());
        assert_eq!(ui.focus(), focus_before_open);
    }

    #[test]
    fn context_menu_popup_arrow_keys_move_focus_between_items() {
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

        let open = Rc::new(Cell::new(false));

        let root = run_frame(
            &mut ui,
            &mut app,
            &mut services,
            window,
            bounds,
            "imui-popup-context-menu-arrow-nav",
            |cx| {
                crate::imui(cx, |ui| {
                    let resp = ui.button("OK");
                    let open_out = open.clone();
                    open_out.set(ui.begin_popup_context_menu_ex(
                        "ctx",
                        resp,
                        PopupMenuOptions {
                            estimated_size: Size::new(Px(160.0), Px(90.0)),
                            ..Default::default()
                        },
                        |ui| {
                            ui.menu_item_ex(
                                "Item A",
                                MenuItemOptions {
                                    test_id: Some(Arc::from("imui-popup-ctx-item-a")),
                                    ..Default::default()
                                },
                            );
                            ui.menu_item_ex(
                                "Item B",
                                MenuItemOptions {
                                    test_id: Some(Arc::from("imui-popup-ctx-item-b")),
                                    ..Default::default()
                                },
                            );
                        },
                    ));
                })
            },
        );
        assert!(!open.get());

        let at = first_child_point(&ui, root);
        click_at(&mut ui, &mut app, &mut services, at);
        key_down(
            &mut ui,
            &mut app,
            &mut services,
            KeyCode::ContextMenu,
            Modifiers::default(),
        );

        app.advance_frame();
        ui.request_semantics_snapshot();
        let _root = run_frame(
            &mut ui,
            &mut app,
            &mut services,
            window,
            bounds,
            "imui-popup-context-menu-arrow-nav",
            |cx| {
                crate::imui(cx, |ui| {
                    let resp = ui.button("OK");
                    let open_out = open.clone();
                    open_out.set(ui.begin_popup_context_menu_ex(
                        "ctx",
                        resp,
                        PopupMenuOptions {
                            estimated_size: Size::new(Px(160.0), Px(90.0)),
                            ..Default::default()
                        },
                        |ui| {
                            ui.menu_item_ex(
                                "Item A",
                                MenuItemOptions {
                                    test_id: Some(Arc::from("imui-popup-ctx-item-a")),
                                    ..Default::default()
                                },
                            );
                            ui.menu_item_ex(
                                "Item B",
                                MenuItemOptions {
                                    test_id: Some(Arc::from("imui-popup-ctx-item-b")),
                                    ..Default::default()
                                },
                            );
                        },
                    ));
                })
            },
        );
        assert!(open.get());

        let focus = ui.focus().expect("focus");
        let snap = ui.semantics_snapshot().expect("semantics snapshot");
        let focused_test_id = snap
            .nodes
            .iter()
            .find(|n| n.id == focus)
            .and_then(|n| n.test_id.as_deref());
        assert_eq!(focused_test_id, Some("imui-popup-ctx-item-a"));

        key_down(
            &mut ui,
            &mut app,
            &mut services,
            KeyCode::ArrowDown,
            Modifiers::default(),
        );

        app.advance_frame();
        ui.request_semantics_snapshot();
        let _root = run_frame(
            &mut ui,
            &mut app,
            &mut services,
            window,
            bounds,
            "imui-popup-context-menu-arrow-nav",
            |cx| {
                crate::imui(cx, |ui| {
                    let resp = ui.button("OK");
                    let open_out = open.clone();
                    open_out.set(ui.begin_popup_context_menu_ex(
                        "ctx",
                        resp,
                        PopupMenuOptions {
                            estimated_size: Size::new(Px(160.0), Px(90.0)),
                            ..Default::default()
                        },
                        |ui| {
                            ui.menu_item_ex(
                                "Item A",
                                MenuItemOptions {
                                    test_id: Some(Arc::from("imui-popup-ctx-item-a")),
                                    ..Default::default()
                                },
                            );
                            ui.menu_item_ex(
                                "Item B",
                                MenuItemOptions {
                                    test_id: Some(Arc::from("imui-popup-ctx-item-b")),
                                    ..Default::default()
                                },
                            );
                        },
                    ));
                })
            },
        );

        let focus = ui.focus().expect("focus");
        let snap = ui.semantics_snapshot().expect("semantics snapshot");
        let focused_test_id = snap
            .nodes
            .iter()
            .find(|n| n.id == focus)
            .and_then(|n| n.test_id.as_deref());
        assert_eq!(focused_test_id, Some("imui-popup-ctx-item-b"));

        key_down(
            &mut ui,
            &mut app,
            &mut services,
            KeyCode::ArrowUp,
            Modifiers::default(),
        );

        app.advance_frame();
        ui.request_semantics_snapshot();
        let _root = run_frame(
            &mut ui,
            &mut app,
            &mut services,
            window,
            bounds,
            "imui-popup-context-menu-arrow-nav",
            |cx| {
                crate::imui(cx, |ui| {
                    let resp = ui.button("OK");
                    let open_out = open.clone();
                    open_out.set(ui.begin_popup_context_menu_ex(
                        "ctx",
                        resp,
                        PopupMenuOptions {
                            estimated_size: Size::new(Px(160.0), Px(90.0)),
                            ..Default::default()
                        },
                        |ui| {
                            ui.menu_item_ex(
                                "Item A",
                                MenuItemOptions {
                                    test_id: Some(Arc::from("imui-popup-ctx-item-a")),
                                    ..Default::default()
                                },
                            );
                            ui.menu_item_ex(
                                "Item B",
                                MenuItemOptions {
                                    test_id: Some(Arc::from("imui-popup-ctx-item-b")),
                                    ..Default::default()
                                },
                            );
                        },
                    ));
                })
            },
        );

        let focus = ui.focus().expect("focus");
        let snap = ui.semantics_snapshot().expect("semantics snapshot");
        let focused_test_id = snap
            .nodes
            .iter()
            .find(|n| n.id == focus)
            .and_then(|n| n.test_id.as_deref());
        assert_eq!(focused_test_id, Some("imui-popup-ctx-item-a"));
    }

    #[test]
    fn menu_item_checkbox_stamps_semantics_checked_state() {
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

        let open = Rc::new(Cell::new(false));

        let root = run_frame(
            &mut ui,
            &mut app,
            &mut services,
            window,
            bounds,
            "imui-menu-item-checkbox-semantics",
            |cx| {
                crate::imui(cx, |ui| {
                    let resp = ui.button("OK");
                    let open_out = open.clone();
                    open_out.set(ui.begin_popup_context_menu_ex(
                        "ctx",
                        resp,
                        PopupMenuOptions {
                            estimated_size: Size::new(Px(160.0), Px(90.0)),
                            ..Default::default()
                        },
                        |ui| {
                            ui.menu_item_checkbox_ex(
                                "Flag",
                                true,
                                MenuItemOptions {
                                    test_id: Some(Arc::from("imui-popup-ctx-item-flag")),
                                    ..Default::default()
                                },
                            );
                        },
                    ));
                })
            },
        );
        assert!(!open.get());

        let at = first_child_point(&ui, root);
        click_at(&mut ui, &mut app, &mut services, at);
        key_down(
            &mut ui,
            &mut app,
            &mut services,
            KeyCode::ContextMenu,
            Modifiers::default(),
        );

        app.advance_frame();
        ui.request_semantics_snapshot();
        let _root = run_frame(
            &mut ui,
            &mut app,
            &mut services,
            window,
            bounds,
            "imui-menu-item-checkbox-semantics",
            |cx| {
                crate::imui(cx, |ui| {
                    let resp = ui.button("OK");
                    let open_out = open.clone();
                    open_out.set(ui.begin_popup_context_menu_ex(
                        "ctx",
                        resp,
                        PopupMenuOptions {
                            estimated_size: Size::new(Px(160.0), Px(90.0)),
                            ..Default::default()
                        },
                        |ui| {
                            ui.menu_item_checkbox_ex(
                                "Flag",
                                true,
                                MenuItemOptions {
                                    test_id: Some(Arc::from("imui-popup-ctx-item-flag")),
                                    ..Default::default()
                                },
                            );
                        },
                    ));
                })
            },
        );
        assert!(open.get());

        let snap = ui.semantics_snapshot().expect("semantics snapshot");
        let node = snap
            .nodes
            .iter()
            .find(|n| n.test_id.as_deref() == Some("imui-popup-ctx-item-flag"))
            .expect("checkbox node");
        assert_eq!(node.role, SemanticsRole::MenuItemCheckbox);
        assert_eq!(node.flags.checked, Some(true));
    }

    #[test]
    fn double_click_sets_double_clicked_true_once() {
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

        let double_clicked = Rc::new(Cell::new(false));
        let double_clicked_out = double_clicked.clone();
        let root = run_frame(
            &mut ui,
            &mut app,
            &mut services,
            window,
            bounds,
            "imui-double-click",
            |cx| {
                crate::imui(cx, |ui| {
                    double_clicked_out.set(ui.button("OK").double_clicked());
                })
            },
        );
        assert!(!double_clicked.get());

        let at = first_child_point(&ui, root);
        double_click_at(&mut ui, &mut app, &mut services, at);

        app.advance_frame();
        let double_clicked_out = double_clicked.clone();
        let _root = run_frame(
            &mut ui,
            &mut app,
            &mut services,
            window,
            bounds,
            "imui-double-click",
            |cx| {
                crate::imui(cx, |ui| {
                    double_clicked_out.set(ui.button("OK").double_clicked());
                })
            },
        );
        assert!(double_clicked.get());

        app.advance_frame();
        let double_clicked_out = double_clicked.clone();
        let _root = run_frame(
            &mut ui,
            &mut app,
            &mut services,
            window,
            bounds,
            "imui-double-click",
            |cx| {
                crate::imui(cx, |ui| {
                    double_clicked_out.set(ui.button("OK").double_clicked());
                })
            },
        );
        assert!(!double_clicked.get());
    }

    #[test]
    fn shift_f10_sets_context_menu_requested_true_once() {
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

        let requested = Rc::new(Cell::new(false));
        let requested_out = requested.clone();
        let root = run_frame(
            &mut ui,
            &mut app,
            &mut services,
            window,
            bounds,
            "imui-context-menu-shift-f10",
            |cx| {
                crate::imui(cx, |ui| {
                    requested_out.set(ui.button("OK").context_menu_requested());
                })
            },
        );
        assert!(!requested.get());

        let at = first_child_point(&ui, root);
        click_at(&mut ui, &mut app, &mut services, at);

        app.advance_frame();
        let requested_out = requested.clone();
        let _root = run_frame(
            &mut ui,
            &mut app,
            &mut services,
            window,
            bounds,
            "imui-context-menu-shift-f10",
            |cx| {
                crate::imui(cx, |ui| {
                    requested_out.set(ui.button("OK").context_menu_requested());
                })
            },
        );
        assert!(!requested.get());

        key_down(
            &mut ui,
            &mut app,
            &mut services,
            KeyCode::F10,
            Modifiers {
                shift: true,
                ..Modifiers::default()
            },
        );

        app.advance_frame();
        let requested_out = requested.clone();
        let _root = run_frame(
            &mut ui,
            &mut app,
            &mut services,
            window,
            bounds,
            "imui-context-menu-shift-f10",
            |cx| {
                crate::imui(cx, |ui| {
                    requested_out.set(ui.button("OK").context_menu_requested());
                })
            },
        );
        assert!(requested.get());

        app.advance_frame();
        let requested_out = requested.clone();
        let _root = run_frame(
            &mut ui,
            &mut app,
            &mut services,
            window,
            bounds,
            "imui-context-menu-shift-f10",
            |cx| {
                crate::imui(cx, |ui| {
                    requested_out.set(ui.button("OK").context_menu_requested());
                })
            },
        );
        assert!(!requested.get());
    }

    #[allow(dead_code)]
    fn ui_writer_imui_facade_ext_smoke<H: fret_ui::UiHost>(
        ui: &mut impl fret_authoring::UiWriter<H>,
    ) {
        use fret_ui_kit::imui::UiWriterImUiFacadeExt as _;

        ui.text("Hello");
        ui.separator();
        let _ = ui.button("OK");
    }

    #[test]
    fn ui_writer_imui_facade_ext_compiles() {}

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
    fn drag_started_stopped_and_delta_are_consistent() {
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

        let started = Rc::new(Cell::new(false));
        let dragging = Rc::new(Cell::new(false));
        let stopped = Rc::new(Cell::new(false));
        let delta = Rc::new(Cell::new(Point::default()));

        let started_out = started.clone();
        let dragging_out = dragging.clone();
        let stopped_out = stopped.clone();
        let delta_out = delta.clone();
        let root = run_frame(
            &mut ui,
            &mut app,
            &mut services,
            window,
            bounds,
            "imui-drag-signals",
            |cx| {
                crate::imui(cx, |ui| {
                    let resp = ui.button("OK");
                    started_out.set(resp.drag_started());
                    dragging_out.set(resp.dragging());
                    stopped_out.set(resp.drag_stopped());
                    delta_out.set(resp.drag_delta());
                })
            },
        );
        assert!(!started.get());
        assert!(!dragging.get());
        assert!(!stopped.get());

        let start = first_child_point(&ui, root);
        pointer_down_at(&mut ui, &mut app, &mut services, start);

        app.advance_frame();
        let started_out = started.clone();
        let dragging_out = dragging.clone();
        let stopped_out = stopped.clone();
        let delta_out = delta.clone();
        let _root = run_frame(
            &mut ui,
            &mut app,
            &mut services,
            window,
            bounds,
            "imui-drag-signals",
            |cx| {
                crate::imui(cx, |ui| {
                    let resp = ui.button("OK");
                    started_out.set(resp.drag_started());
                    dragging_out.set(resp.dragging());
                    stopped_out.set(resp.drag_stopped());
                    delta_out.set(resp.drag_delta());
                })
            },
        );
        assert!(!started.get());
        assert!(!dragging.get());
        assert!(!stopped.get());

        // Move below the threshold.
        let p1 = Point::new(Px(start.x.0 + 2.0), Px(start.y.0));
        pointer_move_at(
            &mut ui,
            &mut app,
            &mut services,
            p1,
            MouseButtons {
                left: true,
                ..MouseButtons::default()
            },
        );

        app.advance_frame();
        let started_out = started.clone();
        let dragging_out = dragging.clone();
        let stopped_out = stopped.clone();
        let delta_out = delta.clone();
        let _root = run_frame(
            &mut ui,
            &mut app,
            &mut services,
            window,
            bounds,
            "imui-drag-signals",
            |cx| {
                crate::imui(cx, |ui| {
                    let resp = ui.button("OK");
                    started_out.set(resp.drag_started());
                    dragging_out.set(resp.dragging());
                    stopped_out.set(resp.drag_stopped());
                    delta_out.set(resp.drag_delta());
                })
            },
        );
        assert!(!started.get());
        assert!(!dragging.get());
        assert!(!stopped.get());

        // Move past the threshold to start dragging (delta should be the frame delta, not the total).
        let p2 = Point::new(Px(start.x.0 + 6.0), Px(start.y.0));
        pointer_move_at(
            &mut ui,
            &mut app,
            &mut services,
            p2,
            MouseButtons {
                left: true,
                ..MouseButtons::default()
            },
        );

        app.advance_frame();
        let started_out = started.clone();
        let dragging_out = dragging.clone();
        let stopped_out = stopped.clone();
        let delta_out = delta.clone();
        let _root = run_frame(
            &mut ui,
            &mut app,
            &mut services,
            window,
            bounds,
            "imui-drag-signals",
            |cx| {
                crate::imui(cx, |ui| {
                    let resp = ui.button("OK");
                    started_out.set(resp.drag_started());
                    dragging_out.set(resp.dragging());
                    stopped_out.set(resp.drag_stopped());
                    delta_out.set(resp.drag_delta());
                })
            },
        );
        assert!(started.get());
        assert!(dragging.get());
        assert!(!stopped.get());
        assert_eq!(delta.get(), Point::new(Px(4.0), Px(0.0)));

        pointer_up_at_with_is_click(&mut ui, &mut app, &mut services, p2, false);

        app.advance_frame();
        let started_out = started.clone();
        let dragging_out = dragging.clone();
        let stopped_out = stopped.clone();
        let delta_out = delta.clone();
        let _root = run_frame(
            &mut ui,
            &mut app,
            &mut services,
            window,
            bounds,
            "imui-drag-signals",
            |cx| {
                crate::imui(cx, |ui| {
                    let resp = ui.button("OK");
                    started_out.set(resp.drag_started());
                    dragging_out.set(resp.dragging());
                    stopped_out.set(resp.drag_stopped());
                    delta_out.set(resp.drag_delta());
                })
            },
        );
        assert!(!started.get());
        assert!(!dragging.get());
        assert!(stopped.get());
    }

    #[test]
    fn long_press_sets_long_pressed_true_once_and_reports_holding() {
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

        let long_pressed = Rc::new(Cell::new(false));
        let holding = Rc::new(Cell::new(false));

        let long_pressed_out = long_pressed.clone();
        let holding_out = holding.clone();
        let root = run_frame(
            &mut ui,
            &mut app,
            &mut services,
            window,
            bounds,
            "imui-long-press-signals",
            |cx| {
                crate::imui(cx, |ui| {
                    let resp = ui.button("OK");
                    long_pressed_out.set(resp.long_pressed());
                    holding_out.set(resp.press_holding());
                })
            },
        );
        assert!(!long_pressed.get());
        assert!(!holding.get());

        let at = first_child_point(&ui, root);
        pointer_down_at(&mut ui, &mut app, &mut services, at);
        let dispatched = dispatch_all_timers(&mut ui, &mut app, &mut services);
        assert!(dispatched > 0);

        app.advance_frame();
        let long_pressed_out = long_pressed.clone();
        let holding_out = holding.clone();
        let _root = run_frame(
            &mut ui,
            &mut app,
            &mut services,
            window,
            bounds,
            "imui-long-press-signals",
            |cx| {
                crate::imui(cx, |ui| {
                    let resp = ui.button("OK");
                    long_pressed_out.set(resp.long_pressed());
                    holding_out.set(resp.press_holding());
                })
            },
        );

        assert!(long_pressed.get());
        assert!(holding.get());

        app.advance_frame();
        let long_pressed_out = long_pressed.clone();
        let holding_out = holding.clone();
        let _root = run_frame(
            &mut ui,
            &mut app,
            &mut services,
            window,
            bounds,
            "imui-long-press-signals",
            |cx| {
                crate::imui(cx, |ui| {
                    let resp = ui.button("OK");
                    long_pressed_out.set(resp.long_pressed());
                    holding_out.set(resp.press_holding());
                })
            },
        );
        assert!(!long_pressed.get());
        assert!(holding.get());

        pointer_up_at(&mut ui, &mut app, &mut services, at);

        app.advance_frame();
        let long_pressed_out = long_pressed.clone();
        let holding_out = holding.clone();
        let _root = run_frame(
            &mut ui,
            &mut app,
            &mut services,
            window,
            bounds,
            "imui-long-press-signals",
            |cx| {
                crate::imui(cx, |ui| {
                    let resp = ui.button("OK");
                    long_pressed_out.set(resp.long_pressed());
                    holding_out.set(resp.press_holding());
                })
            },
        );
        assert!(!long_pressed.get());
        assert!(!holding.get());
    }

    fn floating_window_nodes(
        ui: &UiTree<TestHost>,
        root: fret_core::NodeId,
    ) -> (fret_core::NodeId, fret_core::NodeId) {
        let window = ui.children(root)[0];
        let chrome = ui.children(window)[0];
        let col = ui.children(chrome)[0];
        let title_bar = ui.children(col)[0];
        (window, title_bar)
    }

    fn point_for_test_id(
        ui: &mut UiTree<TestHost>,
        app: &mut TestHost,
        services: &mut FakeTextService,
        bounds: Rect,
        test_id: &str,
    ) -> Point {
        let node = node_for_test_id(ui, app, services, bounds, test_id);
        let bounds = ui.debug_node_bounds(node).expect("node bounds");
        Point::new(Px(bounds.origin.x.0 + 1.0), Px(bounds.origin.y.0 + 1.0))
    }

    fn node_for_test_id(
        ui: &mut UiTree<TestHost>,
        app: &mut TestHost,
        services: &mut FakeTextService,
        bounds: Rect,
        test_id: &str,
    ) -> fret_core::NodeId {
        ui.request_semantics_snapshot();
        ui.layout_all(app, services, bounds, 1.0);
        let snap = ui.semantics_snapshot().expect("semantics snapshot");
        snap.nodes
            .iter()
            .find(|n| n.test_id.as_deref() == Some(test_id))
            .unwrap_or_else(|| panic!("expected semantics node with test_id {test_id:?}"))
            .id
    }

    fn has_test_id(
        ui: &mut UiTree<TestHost>,
        app: &mut TestHost,
        services: &mut FakeTextService,
        bounds: Rect,
        test_id: &str,
    ) -> bool {
        ui.request_semantics_snapshot();
        ui.layout_all(app, services, bounds, 1.0);
        ui.semantics_snapshot()
            .expect("semantics snapshot")
            .nodes
            .iter()
            .any(|n| n.test_id.as_deref() == Some(test_id))
    }

    #[test]
    fn floating_window_moves_when_dragging_title_bar() {
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
            "imui-floating-window-drag",
            |cx| {
                crate::imui(cx, |ui| {
                    ui.floating_window("demo", "Demo", Point::new(Px(10.0), Px(10.0)), |ui| {
                        ui.text("Hello");
                    });
                })
            },
        );

        let (window_node, _title_bar_node) = floating_window_nodes(&ui, root);
        let before = ui.debug_node_bounds(window_node).expect("window bounds");
        let start = point_for_test_id(
            &mut ui,
            &mut app,
            &mut services,
            bounds,
            "imui.float_window.title_bar:demo",
        );

        pointer_down_at(&mut ui, &mut app, &mut services, start);
        let moved = Point::new(Px(start.x.0 + 6.0), start.y);
        pointer_move_at(
            &mut ui,
            &mut app,
            &mut services,
            moved,
            MouseButtons {
                left: true,
                ..MouseButtons::default()
            },
        );

        app.advance_frame();
        let root = run_frame(
            &mut ui,
            &mut app,
            &mut services,
            window,
            bounds,
            "imui-floating-window-drag",
            |cx| {
                crate::imui(cx, |ui| {
                    ui.floating_window("demo", "Demo", Point::new(Px(10.0), Px(10.0)), |ui| {
                        ui.text("Hello");
                    });
                })
            },
        );

        let (window_node, _title_bar_node) = floating_window_nodes(&ui, root);
        let after = ui.debug_node_bounds(window_node).expect("window bounds");
        assert!(
            after.origin.x.0 > before.origin.x.0,
            "expected floating window to move right"
        );

        pointer_up_at_with_is_click(&mut ui, &mut app, &mut services, moved, false);
    }

    #[test]
    fn floating_area_moves_when_dragging_drag_surface() {
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
            "imui-floating-area-drag",
            |cx| {
                crate::imui(cx, |ui| {
                    ui.floating_area("demo", Point::new(Px(10.0), Px(10.0)), |ui, area| {
                        let mut props = fret_ui::element::PointerRegionProps::default();
                        props.layout.size.width = Length::Px(Px(140.0));
                        props.layout.size.height = Length::Px(Px(24.0));
                        let drag = ui
                            .floating_area_drag_surface_ex(area, props, |_cx, _id| {}, |_ui| {})
                            .attach_semantics(
                                fret_ui::element::SemanticsDecoration::default()
                                    .test_id(Arc::from("imui.float_area.drag:demo")),
                            );
                        ui.add(drag);
                    });
                })
            },
        );

        let area_node = node_for_test_id(
            &mut ui,
            &mut app,
            &mut services,
            bounds,
            "imui.float_area.area:demo",
        );
        let before = ui.debug_node_bounds(area_node).expect("area bounds");
        let start = point_for_test_id(
            &mut ui,
            &mut app,
            &mut services,
            bounds,
            "imui.float_area.drag:demo",
        );

        pointer_down_at(&mut ui, &mut app, &mut services, start);
        let moved = Point::new(Px(start.x.0 + 6.0), start.y);
        pointer_move_at(
            &mut ui,
            &mut app,
            &mut services,
            moved,
            MouseButtons {
                left: true,
                ..MouseButtons::default()
            },
        );

        app.advance_frame();
        let _root = run_frame(
            &mut ui,
            &mut app,
            &mut services,
            window,
            bounds,
            "imui-floating-area-drag",
            |cx| {
                crate::imui(cx, |ui| {
                    ui.floating_area("demo", Point::new(Px(10.0), Px(10.0)), |ui, area| {
                        let mut props = fret_ui::element::PointerRegionProps::default();
                        props.layout.size.width = Length::Px(Px(140.0));
                        props.layout.size.height = Length::Px(Px(24.0));
                        let drag = ui
                            .floating_area_drag_surface_ex(area, props, |_cx, _id| {}, |_ui| {})
                            .attach_semantics(
                                fret_ui::element::SemanticsDecoration::default()
                                    .test_id(Arc::from("imui.float_area.drag:demo")),
                            );
                        ui.add(drag);
                    });
                })
            },
        );

        let area_node = node_for_test_id(
            &mut ui,
            &mut app,
            &mut services,
            bounds,
            "imui.float_area.area:demo",
        );
        let after = ui.debug_node_bounds(area_node).expect("area bounds");
        assert!(
            after.origin.x.0 > before.origin.x.0,
            "expected floating area to move right"
        );

        pointer_up_at_with_is_click(&mut ui, &mut app, &mut services, moved, false);
        let _ = ui.children(root);
    }

    #[test]
    fn floating_area_bring_to_front_updates_hit_test_order() {
        let window = AppWindowId::default();
        let bounds = Rect::new(
            Point::new(Px(0.0), Px(0.0)),
            Size::new(Px(320.0), Px(200.0)),
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
            "imui-floating-area-z-order",
            |cx| {
                crate::imui(cx, |ui| {
                    ui.floating_layer("layer", |ui| {
                        ui.floating_area("a", Point::new(Px(10.0), Px(10.0)), |ui, area| {
                            let mut props = fret_ui::element::PointerRegionProps::default();
                            props.layout.size.width = Length::Px(Px(140.0));
                            props.layout.size.height = Length::Px(Px(80.0));
                            let drag = ui
                                .floating_area_drag_surface_ex(area, props, |_cx, _id| {}, |_ui| {})
                                .attach_semantics(
                                    fret_ui::element::SemanticsDecoration::default()
                                        .test_id(Arc::from("imui.float_area.drag:a")),
                                );
                            ui.add(drag);
                        });
                        ui.floating_area("b", Point::new(Px(60.0), Px(10.0)), |ui, area| {
                            let mut props = fret_ui::element::PointerRegionProps::default();
                            props.layout.size.width = Length::Px(Px(140.0));
                            props.layout.size.height = Length::Px(Px(80.0));
                            let drag = ui
                                .floating_area_drag_surface_ex(area, props, |_cx, _id| {}, |_ui| {})
                                .attach_semantics(
                                    fret_ui::element::SemanticsDecoration::default()
                                        .test_id(Arc::from("imui.float_area.drag:b")),
                                );
                            ui.add(drag);
                        });
                    });
                })
            },
        );

        let _ = ui.children(root);
        let area_a = node_for_test_id(
            &mut ui,
            &mut app,
            &mut services,
            bounds,
            "imui.float_area.area:a",
        );
        let area_b = node_for_test_id(
            &mut ui,
            &mut app,
            &mut services,
            bounds,
            "imui.float_area.area:b",
        );

        let a_bounds = ui.debug_node_bounds(area_a).expect("area a bounds");
        let b_bounds = ui.debug_node_bounds(area_b).expect("area b bounds");

        let overlap_left = a_bounds.origin.x.0.max(b_bounds.origin.x.0);
        let overlap_top = a_bounds.origin.y.0.max(b_bounds.origin.y.0);
        let overlap_right = (a_bounds.origin.x.0 + a_bounds.size.width.0)
            .min(b_bounds.origin.x.0 + b_bounds.size.width.0);
        let overlap_bottom = (a_bounds.origin.y.0 + a_bounds.size.height.0)
            .min(b_bounds.origin.y.0 + b_bounds.size.height.0);
        assert!(
            overlap_right > overlap_left + 4.0 && overlap_bottom > overlap_top + 4.0,
            "expected areas to overlap for z-order hit testing"
        );
        let overlap = Point::new(Px(overlap_left + 2.0), Px(overlap_top + 2.0));

        let layer_stack = ui.children(root)[0];
        let stack_children = ui.children(layer_stack);
        let stack_idx_a = stack_children
            .iter()
            .position(|n| *n == area_a)
            .expect("expected area A to be a stack child");
        let stack_idx_b = stack_children
            .iter()
            .position(|n| *n == area_b)
            .expect("expected area B to be a stack child");
        assert!(
            stack_idx_b > stack_idx_a,
            "expected area B to be after A initially"
        );

        let hit = ui
            .debug_hit_test(overlap)
            .hit
            .expect("expected overlap point to hit a node");
        let path = ui.debug_node_path(hit);
        assert!(
            path.contains(&area_b),
            "expected area B to be top initially"
        );
        assert!(
            !path.contains(&area_a),
            "expected area A not to be hit initially"
        );

        let handle_a = point_for_test_id(
            &mut ui,
            &mut app,
            &mut services,
            bounds,
            "imui.float_area.drag:a",
        );
        click_at(&mut ui, &mut app, &mut services, handle_a);

        app.advance_frame();
        let _root = run_frame(
            &mut ui,
            &mut app,
            &mut services,
            window,
            bounds,
            "imui-floating-area-z-order",
            |cx| {
                crate::imui(cx, |ui| {
                    ui.floating_layer("layer", |ui| {
                        ui.floating_area("a", Point::new(Px(10.0), Px(10.0)), |ui, area| {
                            let mut props = fret_ui::element::PointerRegionProps::default();
                            props.layout.size.width = Length::Px(Px(140.0));
                            props.layout.size.height = Length::Px(Px(80.0));
                            let drag = ui
                                .floating_area_drag_surface_ex(area, props, |_cx, _id| {}, |_ui| {})
                                .attach_semantics(
                                    fret_ui::element::SemanticsDecoration::default()
                                        .test_id(Arc::from("imui.float_area.drag:a")),
                                );
                            ui.add(drag);
                        });
                        ui.floating_area("b", Point::new(Px(60.0), Px(10.0)), |ui, area| {
                            let mut props = fret_ui::element::PointerRegionProps::default();
                            props.layout.size.width = Length::Px(Px(140.0));
                            props.layout.size.height = Length::Px(Px(80.0));
                            let drag = ui
                                .floating_area_drag_surface_ex(area, props, |_cx, _id| {}, |_ui| {})
                                .attach_semantics(
                                    fret_ui::element::SemanticsDecoration::default()
                                        .test_id(Arc::from("imui.float_area.drag:b")),
                                );
                            ui.add(drag);
                        });
                    });
                })
            },
        );

        let area_a = node_for_test_id(
            &mut ui,
            &mut app,
            &mut services,
            bounds,
            "imui.float_area.area:a",
        );
        let area_b = node_for_test_id(
            &mut ui,
            &mut app,
            &mut services,
            bounds,
            "imui.float_area.area:b",
        );

        let layer_stack = ui.children(root)[0];
        let stack_children = ui.children(layer_stack);
        let stack_idx_a = stack_children
            .iter()
            .position(|n| *n == area_a)
            .expect("expected area A to be a stack child");
        let stack_idx_b = stack_children
            .iter()
            .position(|n| *n == area_b)
            .expect("expected area B to be a stack child");
        assert!(
            stack_idx_a > stack_idx_b,
            "expected area A to be after B after activation"
        );

        let hit = ui
            .debug_hit_test(overlap)
            .hit
            .expect("expected overlap point to hit a node");
        let path = ui.debug_node_path(hit);
        assert!(
            path.contains(&area_a),
            "expected area A to be top after activating it"
        );
        assert!(
            !path.contains(&area_b),
            "expected area B not to be hit after activation"
        );
    }

    #[test]
    fn window_wrapper_reports_position_and_size() {
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

        let reported_pos = Rc::new(Cell::new(Point::new(Px(0.0), Px(0.0))));
        let reported_size = Rc::new(Cell::new(None::<Size>));

        let reported_pos_out = reported_pos.clone();
        let reported_size_out = reported_size.clone();

        let initial_position = Point::new(Px(10.0), Px(10.0));
        let initial_size = Size::new(Px(140.0), Px(80.0));

        let _root = run_frame(
            &mut ui,
            &mut app,
            &mut services,
            window,
            bounds,
            "imui-window-wrapper-reports-position-and-size",
            |cx| {
                crate::imui(cx, |ui| {
                    let resp =
                        ui.window_resizable("demo", "Demo", initial_position, initial_size, |ui| {
                            ui.text("Hello")
                        });
                    reported_pos_out.set(resp.position());
                    reported_size_out.set(resp.size());
                })
            },
        );

        assert_eq!(reported_pos.get(), initial_position);
        assert_eq!(reported_size.get(), Some(initial_size));
    }

    #[test]
    fn floating_window_close_button_sets_open_false() {
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

        let open = app.models_mut().insert(true);

        let root = run_frame(
            &mut ui,
            &mut app,
            &mut services,
            window,
            bounds,
            "imui-floating-window-close",
            |cx| {
                crate::imui(cx, |ui| {
                    ui.floating_window_open(
                        "demo",
                        "Demo",
                        &open,
                        Point::new(Px(10.0), Px(10.0)),
                        |ui| {
                            ui.text("Hello");
                        },
                    );
                })
            },
        );

        let _ = floating_window_nodes(&ui, root);
        let close = point_for_test_id(
            &mut ui,
            &mut app,
            &mut services,
            bounds,
            "imui.float_window.close:demo",
        );
        click_at(&mut ui, &mut app, &mut services, close);
        assert!(!app.models().get_copied(&open).unwrap_or(true));
    }

    #[test]
    fn floating_window_escape_sets_open_false_after_focusing_title_bar() {
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

        let open = app.models_mut().insert(true);

        let root = run_frame(
            &mut ui,
            &mut app,
            &mut services,
            window,
            bounds,
            "imui-floating-window-escape",
            |cx| {
                crate::imui(cx, |ui| {
                    ui.floating_window_open(
                        "demo",
                        "Demo",
                        &open,
                        Point::new(Px(10.0), Px(10.0)),
                        |ui| {
                            ui.text("Hello");
                        },
                    );
                })
            },
        );

        let _ = floating_window_nodes(&ui, root);
        let title_bar_node = node_for_test_id(
            &mut ui,
            &mut app,
            &mut services,
            bounds,
            "imui.float_window.title_bar:demo",
        );
        let title_bar_bounds = ui
            .debug_node_bounds(title_bar_node)
            .expect("title bar bounds");
        let title_bar = Point::new(
            Px(title_bar_bounds.origin.x.0 + title_bar_bounds.size.width.0 * 0.5),
            Px(title_bar_bounds.origin.y.0 + title_bar_bounds.size.height.0 * 0.5),
        );
        click_at(&mut ui, &mut app, &mut services, title_bar);
        assert!(ui.focus().is_some(), "expected title bar to take focus");

        key_down(
            &mut ui,
            &mut app,
            &mut services,
            KeyCode::Escape,
            Modifiers::default(),
        );
        assert!(!app.models().get_copied(&open).unwrap_or(true));
    }

    #[test]
    fn floating_layer_bring_to_front_updates_hit_test_order() {
        let window = AppWindowId::default();
        let bounds = Rect::new(
            Point::new(Px(0.0), Px(0.0)),
            Size::new(Px(320.0), Px(200.0)),
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
            "imui-floating-layer-z-order",
            |cx| {
                crate::imui(cx, |ui| {
                    ui.floating_layer("layer", |ui| {
                        ui.floating_window("a", "A", Point::new(Px(10.0), Px(10.0)), |ui| {
                            let element = ui.cx_mut().container(
                                {
                                    let mut props = fret_ui::element::ContainerProps::default();
                                    props.layout.size.width =
                                        fret_ui::element::Length::Px(Px(140.0));
                                    props.layout.size.height =
                                        fret_ui::element::Length::Px(Px(80.0));
                                    props
                                },
                                |_cx| Vec::new(),
                            );
                            ui.add(element);
                        });
                        ui.floating_window("b", "B", Point::new(Px(60.0), Px(10.0)), |ui| {
                            let element = ui.cx_mut().container(
                                {
                                    let mut props = fret_ui::element::ContainerProps::default();
                                    props.layout.size.width =
                                        fret_ui::element::Length::Px(Px(140.0));
                                    props.layout.size.height =
                                        fret_ui::element::Length::Px(Px(80.0));
                                    props
                                },
                                |_cx| Vec::new(),
                            );
                            ui.add(element);
                        });
                    });
                })
            },
        );

        let _ = ui.children(root);
        let window_a = node_for_test_id(
            &mut ui,
            &mut app,
            &mut services,
            bounds,
            "imui.float_window.window:a",
        );
        let window_b = node_for_test_id(
            &mut ui,
            &mut app,
            &mut services,
            bounds,
            "imui.float_window.window:b",
        );

        let a_bounds = ui.debug_node_bounds(window_a).expect("window a bounds");
        let b_bounds = ui.debug_node_bounds(window_b).expect("window b bounds");

        let overlap_left = a_bounds.origin.x.0.max(b_bounds.origin.x.0);
        let overlap_top = a_bounds.origin.y.0.max(b_bounds.origin.y.0);
        let overlap_right = (a_bounds.origin.x.0 + a_bounds.size.width.0)
            .min(b_bounds.origin.x.0 + b_bounds.size.width.0);
        let overlap_bottom = (a_bounds.origin.y.0 + a_bounds.size.height.0)
            .min(b_bounds.origin.y.0 + b_bounds.size.height.0);
        assert!(
            overlap_right > overlap_left + 4.0 && overlap_bottom > overlap_top + 4.0,
            "expected windows to overlap for z-order hit testing"
        );
        let overlap = Point::new(Px(overlap_left + 2.0), Px(overlap_top + 2.0));

        let layer_stack = ui.children(root)[0];
        let stack_children = ui.children(layer_stack);
        let stack_idx_a = stack_children
            .iter()
            .position(|n| *n == window_a)
            .expect("expected window A to be a stack child");
        let stack_idx_b = stack_children
            .iter()
            .position(|n| *n == window_b)
            .expect("expected window B to be a stack child");
        assert!(
            stack_idx_b > stack_idx_a,
            "expected window B to be after A initially"
        );

        let hit = ui
            .debug_hit_test(overlap)
            .hit
            .expect("expected overlap point to hit a node");
        let path = ui.debug_node_path(hit);
        assert!(
            path.contains(&window_b),
            "expected window B to be top initially"
        );
        assert!(
            !path.contains(&window_a),
            "expected window A not to be hit initially"
        );

        let title_a = point_for_test_id(
            &mut ui,
            &mut app,
            &mut services,
            bounds,
            "imui.float_window.title_bar:a",
        );
        click_at(&mut ui, &mut app, &mut services, title_a);

        app.advance_frame();
        let _root = run_frame(
            &mut ui,
            &mut app,
            &mut services,
            window,
            bounds,
            "imui-floating-layer-z-order",
            |cx| {
                crate::imui(cx, |ui| {
                    ui.floating_layer("layer", |ui| {
                        ui.floating_window("a", "A", Point::new(Px(10.0), Px(10.0)), |ui| {
                            let element = ui.cx_mut().container(
                                {
                                    let mut props = fret_ui::element::ContainerProps::default();
                                    props.layout.size.width =
                                        fret_ui::element::Length::Px(Px(140.0));
                                    props.layout.size.height =
                                        fret_ui::element::Length::Px(Px(80.0));
                                    props
                                },
                                |_cx| Vec::new(),
                            );
                            ui.add(element);
                        });
                        ui.floating_window("b", "B", Point::new(Px(60.0), Px(10.0)), |ui| {
                            let element = ui.cx_mut().container(
                                {
                                    let mut props = fret_ui::element::ContainerProps::default();
                                    props.layout.size.width =
                                        fret_ui::element::Length::Px(Px(140.0));
                                    props.layout.size.height =
                                        fret_ui::element::Length::Px(Px(80.0));
                                    props
                                },
                                |_cx| Vec::new(),
                            );
                            ui.add(element);
                        });
                    });
                })
            },
        );

        let window_a = node_for_test_id(
            &mut ui,
            &mut app,
            &mut services,
            bounds,
            "imui.float_window.window:a",
        );
        let window_b = node_for_test_id(
            &mut ui,
            &mut app,
            &mut services,
            bounds,
            "imui.float_window.window:b",
        );

        let layer_stack = ui.children(root)[0];
        let stack_children = ui.children(layer_stack);
        let stack_idx_a = stack_children
            .iter()
            .position(|n| *n == window_a)
            .expect("expected window A to be a stack child");
        let stack_idx_b = stack_children
            .iter()
            .position(|n| *n == window_b)
            .expect("expected window B to be a stack child");
        assert!(
            stack_idx_a > stack_idx_b,
            "expected window A to be after B after activation"
        );

        let hit = ui
            .debug_hit_test(overlap)
            .hit
            .expect("expected overlap point to hit a node");
        let path = ui.debug_node_path(hit);
        assert!(
            path.contains(&window_a),
            "expected window A to be top after activating it"
        );
        assert!(
            !path.contains(&window_b),
            "expected window B not to be hit after activation"
        );
    }

    #[test]
    fn floating_window_resizes_when_dragging_corner_handle() {
        let window = AppWindowId::default();
        let bounds = Rect::new(
            Point::new(Px(0.0), Px(0.0)),
            Size::new(Px(320.0), Px(200.0)),
        );

        let mut ui = UiTree::new();
        ui.set_window(window);

        let mut app = TestHost::new();
        app.set_global(PlatformCapabilities::default());
        let mut services = FakeTextService::default();

        let _root = run_frame(
            &mut ui,
            &mut app,
            &mut services,
            window,
            bounds,
            "imui-floating-window-resize",
            |cx| {
                crate::imui(cx, |ui| {
                    ui.floating_window_resizable(
                        "demo",
                        "Demo",
                        Point::new(Px(10.0), Px(10.0)),
                        Size::new(Px(140.0), Px(80.0)),
                        |ui| {
                            ui.text("Hello");
                        },
                    );
                })
            },
        );

        let window_node = node_for_test_id(
            &mut ui,
            &mut app,
            &mut services,
            bounds,
            "imui.float_window.window:demo",
        );
        let before = ui.debug_node_bounds(window_node).expect("window bounds");

        let corner = point_for_test_id(
            &mut ui,
            &mut app,
            &mut services,
            bounds,
            "imui.float_window.resize.corner:demo",
        );
        pointer_down_at(&mut ui, &mut app, &mut services, corner);
        let moved = Point::new(Px(corner.x.0 + 20.0), Px(corner.y.0 + 10.0));
        pointer_move_at(
            &mut ui,
            &mut app,
            &mut services,
            moved,
            MouseButtons {
                left: true,
                ..MouseButtons::default()
            },
        );

        app.advance_frame();
        let root = run_frame(
            &mut ui,
            &mut app,
            &mut services,
            window,
            bounds,
            "imui-floating-window-resize",
            |cx| {
                crate::imui(cx, |ui| {
                    ui.floating_window_resizable(
                        "demo",
                        "Demo",
                        Point::new(Px(10.0), Px(10.0)),
                        Size::new(Px(140.0), Px(80.0)),
                        |ui| {
                            ui.text("Hello");
                        },
                    );
                })
            },
        );
        let _ = ui.children(root);

        let window_node = node_for_test_id(
            &mut ui,
            &mut app,
            &mut services,
            bounds,
            "imui.float_window.window:demo",
        );
        let after = ui.debug_node_bounds(window_node).expect("window bounds");
        assert!(
            after.size.width.0 > before.size.width.0,
            "expected window to grow wider"
        );
        assert!(
            after.size.height.0 > before.size.height.0,
            "expected window to grow taller"
        );

        pointer_up_at_with_is_click(&mut ui, &mut app, &mut services, moved, false);
    }

    #[test]
    fn floating_window_resizes_from_left_updates_origin_and_width() {
        let window = AppWindowId::default();
        let bounds = Rect::new(
            Point::new(Px(0.0), Px(0.0)),
            Size::new(Px(360.0), Px(240.0)),
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
            "imui-floating-window-resize-left",
            |cx| {
                crate::imui(cx, |ui| {
                    ui.floating_window_resizable(
                        "demo",
                        "Demo",
                        Point::new(Px(80.0), Px(40.0)),
                        Size::new(Px(140.0), Px(80.0)),
                        |ui| ui.text("Hello"),
                    );
                })
            },
        );

        let _ = ui.children(root);
        let window_node = node_for_test_id(
            &mut ui,
            &mut app,
            &mut services,
            bounds,
            "imui.float_window.window:demo",
        );
        let before = ui.debug_node_bounds(window_node).expect("window bounds");

        let left = point_for_test_id(
            &mut ui,
            &mut app,
            &mut services,
            bounds,
            "imui.float_window.resize.left:demo",
        );
        pointer_down_at(&mut ui, &mut app, &mut services, left);
        let moved = Point::new(Px(left.x.0 - 18.0), left.y);
        pointer_move_at(
            &mut ui,
            &mut app,
            &mut services,
            moved,
            MouseButtons {
                left: true,
                ..MouseButtons::default()
            },
        );

        app.advance_frame();
        let root = run_frame(
            &mut ui,
            &mut app,
            &mut services,
            window,
            bounds,
            "imui-floating-window-resize-left",
            |cx| {
                crate::imui(cx, |ui| {
                    ui.floating_window_resizable(
                        "demo",
                        "Demo",
                        Point::new(Px(80.0), Px(40.0)),
                        Size::new(Px(140.0), Px(80.0)),
                        |ui| ui.text("Hello"),
                    );
                })
            },
        );
        let _ = ui.children(root);

        let window_node = node_for_test_id(
            &mut ui,
            &mut app,
            &mut services,
            bounds,
            "imui.float_window.window:demo",
        );
        let after = ui.debug_node_bounds(window_node).expect("window bounds");
        assert!(
            after.origin.x.0 < before.origin.x.0,
            "expected origin.x to move left when resizing from left"
        );
        assert!(
            after.size.width.0 > before.size.width.0,
            "expected width to grow when resizing from left"
        );

        pointer_up_at_with_is_click(&mut ui, &mut app, &mut services, moved, false);
    }

    #[test]
    fn floating_window_title_bar_double_click_toggles_collapsed() {
        let window = AppWindowId::default();
        let bounds = Rect::new(
            Point::new(Px(0.0), Px(0.0)),
            Size::new(Px(360.0), Px(240.0)),
        );

        let mut ui = UiTree::new();
        ui.set_window(window);

        let mut app = TestHost::new();
        app.set_global(PlatformCapabilities::default());
        let mut services = FakeTextService::default();

        let collapsed = Rc::new(Cell::new(false));
        let resizing = Rc::new(Cell::new(false));
        let area_id = Rc::new(Cell::new(0u64));

        let collapsed_out = collapsed.clone();
        let resizing_out = resizing.clone();
        let area_id_out = area_id.clone();
        let root = run_frame(
            &mut ui,
            &mut app,
            &mut services,
            window,
            bounds,
            "imui-floating-window-collapse",
            |cx| {
                crate::imui(cx, |ui| {
                    let resp = ui.window_resizable(
                        "demo",
                        "Demo",
                        Point::new(Px(60.0), Px(36.0)),
                        Size::new(Px(180.0), Px(120.0)),
                        |ui| ui.text("Hello"),
                    );
                    collapsed_out.set(resp.collapsed());
                    resizing_out.set(resp.resizing());
                    area_id_out.set(resp.area.id.0);
                })
            },
        );
        let _ = ui.children(root);
        assert!(!collapsed.get());
        assert!(!resizing.get());
        let area_id_before = area_id.get();
        assert_ne!(area_id_before, 0, "expected non-zero floating area id");

        let window_node = node_for_test_id(
            &mut ui,
            &mut app,
            &mut services,
            bounds,
            "imui.float_window.window:demo",
        );
        let before = ui.debug_node_bounds(window_node).expect("window bounds");

        let title_bar_node = node_for_test_id(
            &mut ui,
            &mut app,
            &mut services,
            bounds,
            "imui.float_window.title_bar:demo",
        );
        let title_bar_bounds = ui
            .debug_node_bounds(title_bar_node)
            .expect("title bar bounds");
        let title_bar = Point::new(
            Px(title_bar_bounds.origin.x.0 + title_bar_bounds.size.width.0 * 0.5),
            Px(title_bar_bounds.origin.y.0 + title_bar_bounds.size.height.0 * 0.5),
        );
        double_click_at(&mut ui, &mut app, &mut services, title_bar);

        app.advance_frame();
        let collapsed_out = collapsed.clone();
        let resizing_out = resizing.clone();
        let area_id_out = area_id.clone();
        let _root = run_frame(
            &mut ui,
            &mut app,
            &mut services,
            window,
            bounds,
            "imui-floating-window-collapse",
            |cx| {
                crate::imui(cx, |ui| {
                    let resp = ui.window_resizable(
                        "demo",
                        "Demo",
                        Point::new(Px(60.0), Px(36.0)),
                        Size::new(Px(180.0), Px(120.0)),
                        |ui| ui.text("Hello"),
                    );
                    collapsed_out.set(resp.collapsed());
                    resizing_out.set(resp.resizing());
                    area_id_out.set(resp.area.id.0);
                })
            },
        );
        assert!(collapsed.get());
        assert!(!resizing.get());
        let area_id_collapsed = area_id.get();
        assert_eq!(
            area_id_collapsed, area_id_before,
            "expected floating area id stable across collapse"
        );

        let window_node = node_for_test_id(
            &mut ui,
            &mut app,
            &mut services,
            bounds,
            "imui.float_window.window:demo",
        );
        let collapsed_bounds = ui.debug_node_bounds(window_node).expect("window bounds");
        assert!(
            collapsed_bounds.size.height.0 < before.size.height.0,
            "expected collapsed window to be shorter"
        );
        assert!(
            !has_test_id(
                &mut ui,
                &mut app,
                &mut services,
                bounds,
                "imui.float_window.resize.corner:demo",
            ),
            "expected resize handles hidden while collapsed"
        );

        let title_bar_after_collapse_node = node_for_test_id(
            &mut ui,
            &mut app,
            &mut services,
            bounds,
            "imui.float_window.title_bar:demo",
        );
        let title_bar_after_collapse_bounds = ui
            .debug_node_bounds(title_bar_after_collapse_node)
            .expect("title bar bounds");
        let title_bar_after_collapse = Point::new(
            Px(title_bar_after_collapse_bounds.origin.x.0
                + title_bar_after_collapse_bounds.size.width.0 * 0.5),
            Px(title_bar_after_collapse_bounds.origin.y.0
                + title_bar_after_collapse_bounds.size.height.0 * 0.5),
        );
        double_click_at(&mut ui, &mut app, &mut services, title_bar_after_collapse);

        app.advance_frame();
        let collapsed_out = collapsed.clone();
        let resizing_out = resizing.clone();
        let area_id_out = area_id.clone();
        let _root = run_frame(
            &mut ui,
            &mut app,
            &mut services,
            window,
            bounds,
            "imui-floating-window-collapse",
            |cx| {
                crate::imui(cx, |ui| {
                    let resp = ui.window_resizable(
                        "demo",
                        "Demo",
                        Point::new(Px(60.0), Px(36.0)),
                        Size::new(Px(180.0), Px(120.0)),
                        |ui| ui.text("Hello"),
                    );
                    collapsed_out.set(resp.collapsed());
                    resizing_out.set(resp.resizing());
                    area_id_out.set(resp.area.id.0);
                })
            },
        );
        assert!(!collapsed.get());
        assert!(!resizing.get());
        assert_eq!(
            area_id.get(),
            area_id_before,
            "expected floating area id stable across expand"
        );
        assert!(
            has_test_id(
                &mut ui,
                &mut app,
                &mut services,
                bounds,
                "imui.float_window.resize.corner:demo",
            ),
            "expected resize handles restored after expanding"
        );
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

    #[test]
    fn input_text_model_reports_changed_once_after_text_input() {
        let window = AppWindowId::default();
        let bounds = Rect::new(
            Point::new(Px(0.0), Px(0.0)),
            Size::new(Px(320.0), Px(140.0)),
        );

        let mut ui = UiTree::new();
        ui.set_window(window);

        let mut app = TestHost::new();
        app.set_global(PlatformCapabilities::default());
        let mut services = FakeTextService::default();

        let model = app.models_mut().insert(String::new());

        let changed = Rc::new(Cell::new(false));
        let text = Rc::new(RefCell::new(String::new()));

        let changed_out = changed.clone();
        let text_out = text.clone();
        let root = run_frame(
            &mut ui,
            &mut app,
            &mut services,
            window,
            bounds,
            "imui-input-text",
            |cx| {
                crate::imui(cx, |ui| {
                    changed_out.set(ui.input_text_model(&model).changed());
                    let current = ui
                        .cx_mut()
                        .app
                        .models()
                        .get_cloned(&model)
                        .unwrap_or_default();
                    text_out.replace(current);
                })
            },
        );
        assert!(!changed.get());
        assert!(text.borrow().is_empty());

        let at = first_child_point(&ui, root);
        click_at(&mut ui, &mut app, &mut services, at);
        text_input_event(&mut ui, &mut app, &mut services, "hello");

        app.advance_frame();
        let changed_out = changed.clone();
        let text_out = text.clone();
        let _root = run_frame(
            &mut ui,
            &mut app,
            &mut services,
            window,
            bounds,
            "imui-input-text",
            |cx| {
                crate::imui(cx, |ui| {
                    changed_out.set(ui.input_text_model(&model).changed());
                    let current = ui
                        .cx_mut()
                        .app
                        .models()
                        .get_cloned(&model)
                        .unwrap_or_default();
                    text_out.replace(current);
                })
            },
        );
        assert!(changed.get());
        assert_eq!(text.borrow().as_str(), "hello");

        app.advance_frame();
        let changed_out = changed.clone();
        let text_out = text.clone();
        let _root = run_frame(
            &mut ui,
            &mut app,
            &mut services,
            window,
            bounds,
            "imui-input-text",
            |cx| {
                crate::imui(cx, |ui| {
                    changed_out.set(ui.input_text_model(&model).changed());
                    let current = ui
                        .cx_mut()
                        .app
                        .models()
                        .get_cloned(&model)
                        .unwrap_or_default();
                    text_out.replace(current);
                })
            },
        );
        assert!(!changed.get());
        assert_eq!(text.borrow().as_str(), "hello");
    }

    #[test]
    fn textarea_model_reports_changed_once_after_text_input() {
        let window = AppWindowId::default();
        let bounds = Rect::new(
            Point::new(Px(0.0), Px(0.0)),
            Size::new(Px(360.0), Px(220.0)),
        );

        let mut ui = UiTree::new();
        ui.set_window(window);

        let mut app = TestHost::new();
        app.set_global(PlatformCapabilities::default());
        let mut services = FakeTextService::default();

        let model = app.models_mut().insert(String::new());

        let changed = Rc::new(Cell::new(false));
        let text = Rc::new(RefCell::new(String::new()));

        let changed_out = changed.clone();
        let text_out = text.clone();
        let root = run_frame(
            &mut ui,
            &mut app,
            &mut services,
            window,
            bounds,
            "imui-textarea",
            |cx| {
                crate::imui(cx, |ui| {
                    changed_out.set(ui.textarea_model(&model).changed());
                    let current = ui
                        .cx_mut()
                        .app
                        .models()
                        .get_cloned(&model)
                        .unwrap_or_default();
                    text_out.replace(current);
                })
            },
        );
        assert!(!changed.get());
        assert!(text.borrow().is_empty());

        let at = first_child_point(&ui, root);
        click_at(&mut ui, &mut app, &mut services, at);
        text_input_event(&mut ui, &mut app, &mut services, "line-1\nline-2");

        app.advance_frame();
        let changed_out = changed.clone();
        let text_out = text.clone();
        let _root = run_frame(
            &mut ui,
            &mut app,
            &mut services,
            window,
            bounds,
            "imui-textarea",
            |cx| {
                crate::imui(cx, |ui| {
                    changed_out.set(ui.textarea_model(&model).changed());
                    let current = ui
                        .cx_mut()
                        .app
                        .models()
                        .get_cloned(&model)
                        .unwrap_or_default();
                    text_out.replace(current);
                })
            },
        );
        assert!(changed.get());
        assert_eq!(text.borrow().as_str(), "line-1\nline-2");

        app.advance_frame();
        let changed_out = changed.clone();
        let text_out = text.clone();
        let _root = run_frame(
            &mut ui,
            &mut app,
            &mut services,
            window,
            bounds,
            "imui-textarea",
            |cx| {
                crate::imui(cx, |ui| {
                    changed_out.set(ui.textarea_model(&model).changed());
                    let current = ui
                        .cx_mut()
                        .app
                        .models()
                        .get_cloned(&model)
                        .unwrap_or_default();
                    text_out.replace(current);
                })
            },
        );
        assert!(!changed.get());
        assert_eq!(text.borrow().as_str(), "line-1\nline-2");
    }

    #[test]
    fn push_id_keeps_changed_signal_stable_after_reorder() {
        let window = AppWindowId::default();
        let bounds = Rect::new(
            Point::new(Px(0.0), Px(0.0)),
            Size::new(Px(360.0), Px(220.0)),
        );

        let mut ui = UiTree::new();
        ui.set_window(window);

        let mut app = TestHost::new();
        app.set_global(PlatformCapabilities::default());
        let mut services = FakeTextService::default();

        let model_a = app.models_mut().insert(String::new());
        let model_b = app.models_mut().insert(String::new());

        let order = Rc::new(RefCell::new(vec![1_u8, 2_u8]));
        let changed = Rc::new(RefCell::new(HashMap::<u8, bool>::new()));

        let order_out = order.clone();
        let changed_out = changed.clone();
        let _root = run_frame(
            &mut ui,
            &mut app,
            &mut services,
            window,
            bounds,
            "imui-push-id-reorder",
            |cx| {
                crate::imui(cx, |ui| {
                    changed_out.borrow_mut().clear();
                    let order_now = order_out.borrow().clone();
                    let changed_map = changed_out.clone();
                    ui.column(|ui| {
                        for key in order_now {
                            let model = if key == 1 {
                                model_a.clone()
                            } else {
                                model_b.clone()
                            };
                            let test_id: Arc<str> = Arc::from(format!("imui-input-{key}"));
                            let changed_map = changed_map.clone();
                            ui.push_id(key, |ui| {
                                let resp = ui.input_text_model_ex(
                                    &model,
                                    InputTextOptions {
                                        test_id: Some(test_id),
                                        ..Default::default()
                                    },
                                );
                                changed_map.borrow_mut().insert(key, resp.changed());
                            });
                        }
                    });
                })
            },
        );
        assert_eq!(changed.borrow().get(&1).copied().unwrap_or(false), false);
        assert_eq!(changed.borrow().get(&2).copied().unwrap_or(false), false);

        let at = point_for_test_id(&mut ui, &mut app, &mut services, bounds, "imui-input-1");
        click_at(&mut ui, &mut app, &mut services, at);
        text_input_event(&mut ui, &mut app, &mut services, "hello");

        app.advance_frame();
        let order_out = order.clone();
        let changed_out = changed.clone();
        let _root = run_frame(
            &mut ui,
            &mut app,
            &mut services,
            window,
            bounds,
            "imui-push-id-reorder",
            |cx| {
                crate::imui(cx, |ui| {
                    changed_out.borrow_mut().clear();
                    let order_now = order_out.borrow().clone();
                    let changed_map = changed_out.clone();
                    ui.column(|ui| {
                        for key in order_now {
                            let model = if key == 1 {
                                model_a.clone()
                            } else {
                                model_b.clone()
                            };
                            let test_id: Arc<str> = Arc::from(format!("imui-input-{key}"));
                            let changed_map = changed_map.clone();
                            ui.push_id(key, |ui| {
                                let resp = ui.input_text_model_ex(
                                    &model,
                                    InputTextOptions {
                                        test_id: Some(test_id),
                                        ..Default::default()
                                    },
                                );
                                changed_map.borrow_mut().insert(key, resp.changed());
                            });
                        }
                    });
                })
            },
        );
        assert_eq!(changed.borrow().get(&1).copied().unwrap_or(false), true);
        assert_eq!(changed.borrow().get(&2).copied().unwrap_or(false), false);

        order.borrow_mut().swap(0, 1);
        app.advance_frame();
        let order_out = order.clone();
        let changed_out = changed.clone();
        let _root = run_frame(
            &mut ui,
            &mut app,
            &mut services,
            window,
            bounds,
            "imui-push-id-reorder",
            |cx| {
                crate::imui(cx, |ui| {
                    changed_out.borrow_mut().clear();
                    let order_now = order_out.borrow().clone();
                    let changed_map = changed_out.clone();
                    ui.column(|ui| {
                        for key in order_now {
                            let model = if key == 1 {
                                model_a.clone()
                            } else {
                                model_b.clone()
                            };
                            let test_id: Arc<str> = Arc::from(format!("imui-input-{key}"));
                            let changed_map = changed_map.clone();
                            ui.push_id(key, |ui| {
                                let resp = ui.input_text_model_ex(
                                    &model,
                                    InputTextOptions {
                                        test_id: Some(test_id),
                                        ..Default::default()
                                    },
                                );
                                changed_map.borrow_mut().insert(key, resp.changed());
                            });
                        }
                    });
                })
            },
        );
        assert_eq!(changed.borrow().get(&1).copied().unwrap_or(false), false);
        assert_eq!(changed.borrow().get(&2).copied().unwrap_or(false), false);
    }

    #[test]
    fn toggle_model_reports_changed_once_after_click() {
        let window = AppWindowId::default();
        let bounds = Rect::new(
            Point::new(Px(0.0), Px(0.0)),
            Size::new(Px(320.0), Px(140.0)),
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
        let _root = run_frame(
            &mut ui,
            &mut app,
            &mut services,
            window,
            bounds,
            "imui-toggle",
            |cx| {
                crate::imui(cx, |ui| {
                    changed_out.set(
                        ui.toggle_model_ex(
                            "Flag",
                            &model,
                            ToggleOptions {
                                test_id: Some(Arc::from("imui-toggle")),
                                ..Default::default()
                            },
                        )
                        .changed(),
                    );
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

        let at = point_for_test_id(&mut ui, &mut app, &mut services, bounds, "imui-toggle");
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
            "imui-toggle",
            |cx| {
                crate::imui(cx, |ui| {
                    changed_out.set(
                        ui.toggle_model_ex(
                            "Flag",
                            &model,
                            ToggleOptions {
                                test_id: Some(Arc::from("imui-toggle")),
                                ..Default::default()
                            },
                        )
                        .changed(),
                    );
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
            "imui-toggle",
            |cx| {
                crate::imui(cx, |ui| {
                    changed_out.set(
                        ui.toggle_model_ex(
                            "Flag",
                            &model,
                            ToggleOptions {
                                test_id: Some(Arc::from("imui-toggle")),
                                ..Default::default()
                            },
                        )
                        .changed(),
                    );
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

    #[test]
    fn switch_model_reports_changed_once_after_click() {
        let window = AppWindowId::default();
        let bounds = Rect::new(
            Point::new(Px(0.0), Px(0.0)),
            Size::new(Px(320.0), Px(140.0)),
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
            "imui-switch",
            |cx| {
                crate::imui(cx, |ui| {
                    changed_out.set(
                        ui.switch_model_ex(
                            "Power",
                            &model,
                            SwitchOptions {
                                test_id: Some(Arc::from("imui-switch")),
                                ..Default::default()
                            },
                        )
                        .changed(),
                    );
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
            "imui-switch",
            |cx| {
                crate::imui(cx, |ui| {
                    changed_out.set(
                        ui.switch_model_ex(
                            "Power",
                            &model,
                            SwitchOptions {
                                test_id: Some(Arc::from("imui-switch")),
                                ..Default::default()
                            },
                        )
                        .changed(),
                    );
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
            "imui-switch",
            |cx| {
                crate::imui(cx, |ui| {
                    changed_out.set(
                        ui.switch_model_ex(
                            "Power",
                            &model,
                            SwitchOptions {
                                test_id: Some(Arc::from("imui-switch")),
                                ..Default::default()
                            },
                        )
                        .changed(),
                    );
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

    #[test]
    fn slider_f32_model_reports_changed_once_after_pointer_input() {
        let window = AppWindowId::default();
        let bounds = Rect::new(
            Point::new(Px(0.0), Px(0.0)),
            Size::new(Px(360.0), Px(140.0)),
        );

        let mut ui = UiTree::new();
        ui.set_window(window);

        let mut app = TestHost::new();
        app.set_global(PlatformCapabilities::default());
        let mut services = FakeTextService::default();

        let model = app.models_mut().insert(0.0_f32);

        let changed = Rc::new(Cell::new(false));
        let value = Rc::new(Cell::new(0.0_f32));

        let changed_out = changed.clone();
        let value_out = value.clone();
        let root = run_frame(
            &mut ui,
            &mut app,
            &mut services,
            window,
            bounds,
            "imui-slider",
            |cx| {
                crate::imui(cx, |ui| {
                    changed_out.set(
                        ui.slider_f32_model_ex(
                            "Volume",
                            &model,
                            SliderOptions {
                                min: 0.0,
                                max: 100.0,
                                step: 1.0,
                                test_id: Some(Arc::from("imui-slider")),
                                ..Default::default()
                            },
                        )
                        .changed(),
                    );
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
        assert!((value.get() - 0.0).abs() <= f32::EPSILON);

        let slider_node = ui.children(root)[0];
        let slider_bounds = ui.debug_node_bounds(slider_node).expect("slider bounds");
        let at = Point::new(
            Px(slider_bounds.origin.x.0 + slider_bounds.size.width.0 * 0.9),
            Px(slider_bounds.origin.y.0 + slider_bounds.size.height.0 * 0.5),
        );
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
            "imui-slider",
            |cx| {
                crate::imui(cx, |ui| {
                    changed_out.set(
                        ui.slider_f32_model_ex(
                            "Volume",
                            &model,
                            SliderOptions {
                                min: 0.0,
                                max: 100.0,
                                step: 1.0,
                                test_id: Some(Arc::from("imui-slider")),
                                ..Default::default()
                            },
                        )
                        .changed(),
                    );
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
        assert!(value.get() >= 70.0);

        app.advance_frame();
        let changed_out = changed.clone();
        let value_out = value.clone();
        let _root = run_frame(
            &mut ui,
            &mut app,
            &mut services,
            window,
            bounds,
            "imui-slider",
            |cx| {
                crate::imui(cx, |ui| {
                    changed_out.set(
                        ui.slider_f32_model_ex(
                            "Volume",
                            &model,
                            SliderOptions {
                                min: 0.0,
                                max: 100.0,
                                step: 1.0,
                                test_id: Some(Arc::from("imui-slider")),
                                ..Default::default()
                            },
                        )
                        .changed(),
                    );
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
        assert!(value.get() >= 70.0);
    }

    #[test]
    fn select_model_reports_changed_once_after_option_pick() {
        let window = AppWindowId::default();
        let bounds = Rect::new(
            Point::new(Px(0.0), Px(0.0)),
            Size::new(Px(360.0), Px(220.0)),
        );

        let mut ui = UiTree::new();
        ui.set_window(window);

        let mut app = TestHost::new();
        app.set_global(PlatformCapabilities::default());
        let mut services = FakeTextService::default();

        let model = app.models_mut().insert(None::<Arc<str>>);
        let items = vec![Arc::<str>::from("Alpha"), Arc::<str>::from("Beta")];

        let changed = Rc::new(Cell::new(false));
        let selected = Rc::new(RefCell::new(None::<Arc<str>>));

        let changed_out = changed.clone();
        let selected_out = selected.clone();
        let _root = run_frame(
            &mut ui,
            &mut app,
            &mut services,
            window,
            bounds,
            "imui-select",
            |cx| {
                crate::imui(cx, |ui| {
                    changed_out.set(
                        ui.select_model_ex(
                            "Mode",
                            &model,
                            &items,
                            SelectOptions {
                                test_id: Some(Arc::from("imui-select")),
                                ..Default::default()
                            },
                        )
                        .changed(),
                    );
                    let now = ui.cx_mut().app.models().get_cloned(&model).unwrap_or(None);
                    selected_out.replace(now);
                })
            },
        );
        assert!(!changed.get());
        assert!(selected.borrow().is_none());

        let trigger = point_for_test_id(&mut ui, &mut app, &mut services, bounds, "imui-select");
        click_at(&mut ui, &mut app, &mut services, trigger);

        app.advance_frame();
        let changed_out = changed.clone();
        let selected_out = selected.clone();
        let _root = run_frame(
            &mut ui,
            &mut app,
            &mut services,
            window,
            bounds,
            "imui-select",
            |cx| {
                crate::imui(cx, |ui| {
                    changed_out.set(
                        ui.select_model_ex(
                            "Mode",
                            &model,
                            &items,
                            SelectOptions {
                                test_id: Some(Arc::from("imui-select")),
                                ..Default::default()
                            },
                        )
                        .changed(),
                    );
                    let now = ui.cx_mut().app.models().get_cloned(&model).unwrap_or(None);
                    selected_out.replace(now);
                })
            },
        );
        assert!(changed.get());
        assert_eq!(selected.borrow().as_deref(), Some("Alpha"));

        app.advance_frame();
        let changed_out = changed.clone();
        let selected_out = selected.clone();
        let _root = run_frame(
            &mut ui,
            &mut app,
            &mut services,
            window,
            bounds,
            "imui-select",
            |cx| {
                crate::imui(cx, |ui| {
                    changed_out.set(
                        ui.select_model_ex(
                            "Mode",
                            &model,
                            &items,
                            SelectOptions {
                                test_id: Some(Arc::from("imui-select")),
                                ..Default::default()
                            },
                        )
                        .changed(),
                    );
                    let now = ui.cx_mut().app.models().get_cloned(&model).unwrap_or(None);
                    selected_out.replace(now);
                })
            },
        );
        assert!(!changed.get());
        assert_eq!(selected.borrow().as_deref(), Some("Alpha"));
    }

    #[test]
    fn container_helpers_layout_horizontal_vertical_grid_and_scroll() {
        let window = AppWindowId::default();
        let bounds = Rect::new(
            Point::new(Px(0.0), Px(0.0)),
            Size::new(Px(420.0), Px(320.0)),
        );

        let mut ui = UiTree::new();
        ui.set_window(window);

        let mut app = TestHost::new();
        app.set_global(PlatformCapabilities::default());
        fret_ui::Theme::with_global_mut(&mut app, |theme| {
            let mut cfg = fret_ui::theme::ThemeConfig {
                name: "Test".to_string(),
                ..fret_ui::theme::ThemeConfig::default()
            };
            cfg.colors.insert(
                "scrollbar.track.background".to_string(),
                "#1f1f1f".to_string(),
            );
            cfg.colors.insert(
                "scrollbar.thumb.background".to_string(),
                "#5f5f5f".to_string(),
            );
            cfg.colors.insert(
                "scrollbar.thumb.hover.background".to_string(),
                "#7f7f7f".to_string(),
            );
            cfg.metrics
                .insert("metric.scrollbar.width".to_string(), 8.0);
            theme.apply_config_patch(&cfg);
        });
        let mut services = FakeTextService::default();

        let _root = run_frame(
            &mut ui,
            &mut app,
            &mut services,
            window,
            bounds,
            "imui-container-helpers-layout",
            |cx| {
                crate::imui(cx, |ui| {
                    ui.vertical_ex(
                        VerticalOptions {
                            gap: Px(8.0).into(),
                            ..Default::default()
                        },
                        |ui| {
                            ui.horizontal_ex(
                                HorizontalOptions {
                                    gap: Px(10.0).into(),
                                    ..Default::default()
                                },
                                |ui| {
                                    ui.menu_item_ex(
                                        "Left",
                                        MenuItemOptions {
                                            test_id: Some(Arc::from("imui-container-left")),
                                            ..Default::default()
                                        },
                                    );
                                    ui.menu_item_ex(
                                        "Right",
                                        MenuItemOptions {
                                            test_id: Some(Arc::from("imui-container-right")),
                                            ..Default::default()
                                        },
                                    );
                                },
                            );

                            ui.grid_ex(
                                GridOptions {
                                    columns: 2,
                                    column_gap: Px(6.0).into(),
                                    row_gap: Px(6.0).into(),
                                    ..Default::default()
                                },
                                |ui| {
                                    ui.menu_item_ex(
                                        "A",
                                        MenuItemOptions {
                                            test_id: Some(Arc::from("imui-grid-a")),
                                            ..Default::default()
                                        },
                                    );
                                    ui.menu_item_ex(
                                        "B",
                                        MenuItemOptions {
                                            test_id: Some(Arc::from("imui-grid-b")),
                                            ..Default::default()
                                        },
                                    );
                                    ui.menu_item_ex(
                                        "C",
                                        MenuItemOptions {
                                            test_id: Some(Arc::from("imui-grid-c")),
                                            ..Default::default()
                                        },
                                    );
                                },
                            );

                            ui.scroll_ex(
                                ScrollOptions {
                                    axis: fret_ui::element::ScrollAxis::X,
                                    show_scrollbar_x: true,
                                    show_scrollbar_y: false,
                                    ..Default::default()
                                },
                                |ui| {
                                    ui.menu_item_ex(
                                        "Scroll Child",
                                        MenuItemOptions {
                                            test_id: Some(Arc::from("imui-scroll-child")),
                                            ..Default::default()
                                        },
                                    );
                                },
                            );
                        },
                    );
                })
            },
        );

        let left = point_for_test_id(
            &mut ui,
            &mut app,
            &mut services,
            bounds,
            "imui-container-left",
        );
        let right = point_for_test_id(
            &mut ui,
            &mut app,
            &mut services,
            bounds,
            "imui-container-right",
        );
        assert!(right.x.0 > left.x.0);

        let grid_a = point_for_test_id(&mut ui, &mut app, &mut services, bounds, "imui-grid-a");
        let grid_b = point_for_test_id(&mut ui, &mut app, &mut services, bounds, "imui-grid-b");
        let grid_c = point_for_test_id(&mut ui, &mut app, &mut services, bounds, "imui-grid-c");
        assert!(grid_b.x.0 > grid_a.x.0);
        assert!(grid_c.y.0 > grid_a.y.0);

        let scroll_child = point_for_test_id(
            &mut ui,
            &mut app,
            &mut services,
            bounds,
            "imui-scroll-child",
        );
        assert!(scroll_child.y.0 > grid_c.y.0);
    }
    // Note: `for_each_keyed` is exercised indirectly by downstream ecosystem crates. The core
    // smoke tests above focus on interaction correctness (`clicked` / `changed`).
}
