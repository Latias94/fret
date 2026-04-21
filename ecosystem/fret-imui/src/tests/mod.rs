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
    DragSessionId, Effect, EffectSink, FrameId, GlobalsHost, KeyChord, ModelHost, ModelId,
    ModelStore, ModelsHost, PlatformCapabilities, ShareSheetToken, TickId, TimeHost, TimerToken,
};
use fret_ui::action::{DismissReason, DismissRequestCx, OnDismissRequest};
use fret_ui::declarative::render_root;
use fret_ui::element::Length;
use fret_ui::tree::PointerOcclusion;
use fret_ui::{ElementContext, GlobalElementId, UiTree};
use fret_ui_kit::OverlayController;
use fret_ui_kit::imui::UiWriterImUiFacadeExt as _;
use fret_ui_kit::imui::{
    CheckboxOptions, ComboModelOptions, ComboOptions, FloatingAreaOptions, FloatingWindowOptions,
    FloatingWindowResizeOptions, GridOptions, HorizontalOptions, ImUiHoveredFlags,
    InputTextOptions, MenuItemOptions, PopupMenuOptions, PopupModalOptions, ScrollOptions,
    SelectableOptions, SliderOptions, SwitchOptions, TableColumn, TableOptions, VerticalOptions,
    VirtualListMeasureMode, VirtualListOptions, VirtualListScrollHandle, WindowOptions,
};
use fret_ui_kit::{OverlayPresence, OverlayRequest};

#[derive(Default)]
struct FakeTextService {
    prepared: Vec<String>,
}

impl TextService for FakeTextService {
    fn prepare(
        &mut self,
        input: &fret_core::TextInput,
        _constraints: TextConstraints,
    ) -> (fret_core::TextBlobId, TextMetrics) {
        self.prepared.push(input.text().to_string());
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

impl fret_core::MaterialService for FakeTextService {
    fn register_material(
        &mut self,
        _desc: fret_core::MaterialDescriptor,
    ) -> Result<fret_core::MaterialId, fret_core::MaterialRegistrationError> {
        Err(fret_core::MaterialRegistrationError::Unsupported)
    }

    fn unregister_material(&mut self, _id: fret_core::MaterialId) -> bool {
        true
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
    next_share_sheet_token: u64,
    next_image_upload_token: u64,
}

impl TestHost {
    fn new() -> Self {
        Self::default()
    }

    fn commands_mut(&mut self) -> &mut CommandRegistry {
        &mut self.commands
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

    fn next_share_sheet_token(&mut self) -> ShareSheetToken {
        let token = ShareSheetToken(self.next_share_sheet_token);
        self.next_share_sheet_token = self.next_share_sheet_token.saturating_add(1);
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

fn advance_and_run_frame<R>(
    ui: &mut UiTree<TestHost>,
    app: &mut TestHost,
    services: &mut FakeTextService,
    window: AppWindowId,
    bounds: Rect,
    root_name: &str,
    render: &R,
) -> fret_core::NodeId
where
    R: for<'a> Fn(&mut ElementContext<'a, TestHost>) -> crate::Elements,
{
    app.advance_frame();
    run_frame(ui, app, services, window, bounds, root_name, |cx| {
        render(cx)
    })
}

fn render_imui_disabled_scope_overlay_scene(
    cx: &mut ElementContext<'_, TestHost>,
    under_clicked: Rc<Cell<bool>>,
    over_clicked: Rc<Cell<bool>>,
    over_hovered: Rc<Cell<bool>>,
    over_hovered_like_imgui: Rc<Cell<bool>>,
    over_hovered_allow_when_disabled: Rc<Cell<bool>>,
    over_id: Rc<Cell<Option<fret_ui::elements::GlobalElementId>>>,
) -> crate::Elements {
    let mut stack = fret_ui::element::StackProps::default();
    stack.layout.size.width = Length::Fill;
    let element = cx.stack_props(stack, |cx| {
        crate::imui_raw(cx, |ui| {
            let under = ui.menu_item_with_options(
                "Underlay",
                MenuItemOptions {
                    test_id: Some(Arc::from("imui-underlay-item")),
                    ..Default::default()
                },
            );
            under_clicked.set(under.clicked());

            ui.disabled_scope(true, |ui| {
                let over = ui.menu_item_with_options(
                    "Overlay",
                    MenuItemOptions {
                        test_id: Some(Arc::from("imui-overlay-item")),
                        ..Default::default()
                    },
                );
                over_id.set(over.id);
                over_clicked.set(over.clicked());
                over_hovered.set(over.core.hovered);
                over_hovered_like_imgui.set(over.hovered_like_imgui());
                over_hovered_allow_when_disabled
                    .set(over.is_hovered(ImUiHoveredFlags::ALLOW_WHEN_DISABLED));
            });
        })
    });
    vec![element].into()
}

fn render_imui_disabled_scope_tooltip_hover_scene(
    cx: &mut ElementContext<'_, TestHost>,
    hovered_for_tooltip: Rc<Cell<bool>>,
    hovered_raw: Rc<Cell<bool>>,
    stationary_met: Rc<Cell<bool>>,
    delay_short_met: Rc<Cell<bool>>,
    delay_normal_met: Rc<Cell<bool>>,
) -> crate::Elements {
    let mut stack = fret_ui::element::StackProps::default();
    stack.layout.size.width = Length::Fill;
    let element = cx.stack_props(stack, |cx| {
        crate::imui_raw(cx, |ui| {
            ui.disabled_scope(true, |ui| {
                let resp = ui.menu_item_with_options(
                    "Tooltip target",
                    MenuItemOptions {
                        test_id: Some(Arc::from("imui-tooltip-target")),
                        ..Default::default()
                    },
                );
                hovered_for_tooltip.set(resp.is_hovered(ImUiHoveredFlags::FOR_TOOLTIP));
                hovered_raw.set(resp.pointer_hovered_raw);
                stationary_met.set(resp.hover_stationary_met);
                delay_short_met.set(resp.hover_delay_short_met);
                delay_normal_met.set(resp.hover_delay_normal_met);
            });
        })
    });
    vec![element].into()
}

fn render_imui_popup_modal_barrier_hover_scene(
    cx: &mut ElementContext<'_, TestHost>,
    popup_id: &'static str,
    open_popup: bool,
    popup_opened: Rc<Cell<bool>>,
    under_hovered_default: Rc<Cell<bool>>,
    under_hovered_allow_when_blocked: Rc<Cell<bool>>,
    under_hovered_raw: Rc<Cell<bool>>,
    under_hovered_raw_below_barrier: Rc<Cell<bool>>,
) -> crate::Elements {
    let anchor = Rect::new(
        Point::new(Px(280.0), Px(160.0)),
        Size::new(Px(1.0), Px(1.0)),
    );
    let mut stack = fret_ui::element::StackProps::default();
    stack.layout.size.width = Length::Fill;
    let element = cx.stack_props(stack, |cx| {
        crate::imui_raw(cx, |ui| {
            let under = ui.menu_item_with_options(
                "Underlay",
                MenuItemOptions {
                    test_id: Some(Arc::from("imui-underlay-item")),
                    ..Default::default()
                },
            );
            under_hovered_default.set(under.core.hovered);
            under_hovered_allow_when_blocked
                .set(under.is_hovered(ImUiHoveredFlags::ALLOW_WHEN_BLOCKED_BY_POPUP));
            under_hovered_raw.set(under.pointer_hovered_raw);
            under_hovered_raw_below_barrier.set(under.pointer_hovered_raw_below_barrier);

            if open_popup {
                ui.open_popup_at(popup_id, anchor);
            }
            popup_opened.set(ui.begin_popup_menu(popup_id, None, |ui| {
                ui.menu_item_with_options(
                    "Popup item",
                    MenuItemOptions {
                        test_id: Some(Arc::from("imui-popup-item")),
                        ..Default::default()
                    },
                );
            }));
        })
    });
    vec![element].into()
}

fn render_imui_shared_hover_delay_scene(
    cx: &mut ElementContext<'_, TestHost>,
    id_a: Rc<Cell<Option<fret_ui::elements::GlobalElementId>>>,
    hovered_b_shared: Rc<Cell<bool>>,
    hovered_b_no_shared: Rc<Cell<bool>>,
    b_stationary_met: Rc<Cell<bool>>,
    b_delay_short_met: Rc<Cell<bool>>,
    b_delay_short_shared_met: Rc<Cell<bool>>,
    id_b: Rc<Cell<Option<fret_ui::elements::GlobalElementId>>>,
) -> crate::Elements {
    let mut stack = fret_ui::element::StackProps::default();
    stack.layout.size.width = Length::Fill;
    let element = cx.stack_props(stack, |cx| {
        crate::imui_raw(cx, |ui| {
            let a = ui.menu_item_with_options(
                "A",
                MenuItemOptions {
                    test_id: Some(Arc::from("imui-shared-delay-a")),
                    ..Default::default()
                },
            );
            id_a.set(a.id);

            let b = ui.menu_item_with_options(
                "B",
                MenuItemOptions {
                    test_id: Some(Arc::from("imui-shared-delay-b")),
                    ..Default::default()
                },
            );
            id_b.set(b.id);
            b_stationary_met.set(b.hover_stationary_met);
            b_delay_short_met.set(b.hover_delay_short_met);
            b_delay_short_shared_met.set(b.hover_delay_short_shared_met);
            let flags = ImUiHoveredFlags::DELAY_SHORT | ImUiHoveredFlags::NO_NAV_OVERRIDE;
            hovered_b_shared.set(b.is_hovered(flags));
            hovered_b_no_shared.set(b.is_hovered(flags | ImUiHoveredFlags::NO_SHARED_DELAY));
        })
    });
    vec![element].into()
}

fn render_imui_active_item_blocks_hover_scene(
    cx: &mut ElementContext<'_, TestHost>,
    a_hovered: Rc<Cell<bool>>,
    a_focused: Rc<Cell<bool>>,
    b_core_hovered: Rc<Cell<bool>>,
    b_blocked_by_active_item: Rc<Cell<bool>>,
    b_hovered_default: Rc<Cell<bool>>,
    b_hovered_allow_when_blocked: Rc<Cell<bool>>,
) -> crate::Elements {
    let mut stack = fret_ui::element::StackProps::default();
    stack.layout.size.width = Length::Fill;
    let element = cx.stack_props(stack, |cx| {
        crate::imui_raw(cx, |ui| {
            ui.vertical(|ui| {
                let a = ui.menu_item_with_options(
                    "A",
                    MenuItemOptions {
                        test_id: Some(Arc::from("imui-active-item-a")),
                        ..Default::default()
                    },
                );
                a_hovered.set(a.core.hovered);
                a_focused.set(a.core.focused);

                let b = ui.menu_item_with_options(
                    "B",
                    MenuItemOptions {
                        test_id: Some(Arc::from("imui-active-item-b")),
                        ..Default::default()
                    },
                );

                b_core_hovered.set(b.core.hovered);
                b_blocked_by_active_item.set(b.hover_blocked_by_active_item);
                let flags = ImUiHoveredFlags::NO_NAV_OVERRIDE;
                b_hovered_default.set(b.is_hovered(flags));
                b_hovered_allow_when_blocked
                    .set(b.is_hovered(flags | ImUiHoveredFlags::ALLOW_WHEN_BLOCKED_BY_ACTIVE_ITEM));
            });
        })
    });
    vec![element].into()
}

fn click_at(
    ui: &mut UiTree<TestHost>,
    app: &mut TestHost,
    services: &mut FakeTextService,
    at: Point,
) {
    click_at_with_modifiers(ui, app, services, at, Modifiers::default());
}

fn click_at_with_modifiers(
    ui: &mut UiTree<TestHost>,
    app: &mut TestHost,
    services: &mut FakeTextService,
    at: Point,
    modifiers: Modifiers,
) {
    ui.dispatch_event(
        app,
        services,
        &Event::Pointer(fret_core::PointerEvent::Down {
            pointer_id: PointerId(0),
            position: at,
            button: MouseButton::Left,
            modifiers,
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
            modifiers,
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
    key_down_with_repeat(ui, app, services, key, modifiers, false);
}

fn key_down_with_repeat(
    ui: &mut UiTree<TestHost>,
    app: &mut TestHost,
    services: &mut FakeTextService,
    key: KeyCode,
    modifiers: Modifiers,
    repeat: bool,
) {
    ui.dispatch_event(
        app,
        services,
        &Event::KeyDown {
            key,
            modifiers,
            repeat,
        },
    );
}

fn ctrl_modifiers() -> Modifiers {
    Modifiers {
        ctrl: true,
        ..Default::default()
    }
}

fn ctrl_shortcut(key: KeyCode) -> KeyChord {
    KeyChord::new(key, ctrl_modifiers())
}

fn key_down_ctrl(
    ui: &mut UiTree<TestHost>,
    app: &mut TestHost,
    services: &mut FakeTextService,
    key: KeyCode,
) {
    key_down(ui, app, services, key, ctrl_modifiers());
}

fn key_down_ctrl_repeat(
    ui: &mut UiTree<TestHost>,
    app: &mut TestHost,
    services: &mut FakeTextService,
    key: KeyCode,
) {
    key_down_with_repeat(ui, app, services, key, ctrl_modifiers(), true);
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
    app.effects
        .retain(|effect| !matches!(effect, Effect::SetTimer { repeat, .. } if repeat.is_none()));

    let dispatched = pending.len();
    for token in pending {
        ui.dispatch_event(app, services, &Event::Timer { token });
    }
    dispatched
}

fn pending_nonrepeating_timer_tokens(app: &TestHost) -> Vec<TimerToken> {
    let mut pending: Vec<TimerToken> = Vec::new();
    for effect in &app.effects {
        if let Effect::SetTimer { token, repeat, .. } = effect
            && repeat.is_none()
        {
            pending.push(*token);
        }
    }
    pending
}

fn dispatch_timer_tokens(
    ui: &mut UiTree<TestHost>,
    app: &mut TestHost,
    services: &mut FakeTextService,
    tokens: &[TimerToken],
) -> usize {
    let mut dispatched = 0usize;
    for token in tokens {
        let token = *token;
        let mut removed = false;
        app.effects.retain(|effect| {
            let is_match = matches!(
                effect,
                Effect::SetTimer { token: t, repeat, .. } if *t == token && repeat.is_none()
            );
            if is_match {
                removed = true;
            }
            !is_match
        });
        if removed {
            dispatched += 1;
            ui.dispatch_event(app, services, &Event::Timer { token });
        }
    }
    dispatched
}

fn fnv1a64(bytes: &[u8]) -> u64 {
    let mut hash = 0xcbf2_9ce4_8422_2325u64;
    for b in bytes {
        hash ^= *b as u64;
        hash = hash.wrapping_mul(0x0000_0100_0000_01b3u64);
    }
    hash
}

fn hover_timer_token_for(kind: u64, element: fret_ui::elements::GlobalElementId) -> TimerToken {
    let mut hash = 0xcbf2_9ce4_8422_2325u64;
    for b in kind.to_le_bytes() {
        hash ^= b as u64;
        hash = hash.wrapping_mul(0x0000_0100_0000_01b3u64);
    }
    for b in element.0.to_le_bytes() {
        hash ^= b as u64;
        hash = hash.wrapping_mul(0x0000_0100_0000_01b3u64);
    }
    TimerToken(hash)
}

fn first_child_point(ui: &UiTree<TestHost>, root: fret_core::NodeId) -> Point {
    let child = ui.children(root)[0];
    let bounds = ui.debug_node_bounds(child).expect("child bounds");
    Point::new(Px(bounds.origin.x.0 + 1.0), Px(bounds.origin.y.0 + 1.0))
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

fn ui_writer_imui_facade_ext_smoke<H: fret_ui::UiHost>(ui: &mut impl fret_authoring::UiWriter<H>) {
    use fret_ui_kit::imui::UiWriterImUiFacadeExt as _;

    ui.text("Hello");
    ui.separator();
    let _ = ui.button("OK");
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

fn focus_test_id(
    ui: &mut UiTree<TestHost>,
    app: &mut TestHost,
    services: &mut FakeTextService,
    bounds: Rect,
    test_id: &str,
) -> fret_core::NodeId {
    let node = node_for_test_id(ui, app, services, bounds, test_id);
    ui.set_focus(Some(node));
    assert_eq!(ui.focus(), Some(node));
    node
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

#[derive(Debug, Clone, Copy)]
enum FloatingLayerOverlayVariant {
    Menu,
    Popover,
}

fn window_behavior_options(behavior: FloatingWindowOptions) -> WindowOptions {
    WindowOptions::default().with_behavior(behavior)
}

fn resizable_window_options(size: Size) -> WindowOptions {
    WindowOptions::default()
        .with_size(size)
        .with_resize(FloatingWindowResizeOptions::default())
}

fn resizable_window_options_with_behavior(
    size: Size,
    behavior: FloatingWindowOptions,
) -> WindowOptions {
    resizable_window_options(size).with_behavior(behavior)
}

fn open_window_options(open: &fret_runtime::Model<bool>) -> WindowOptions {
    WindowOptions::default().with_open(open)
}

fn open_window_options_with_behavior(
    open: &fret_runtime::Model<bool>,
    behavior: FloatingWindowOptions,
) -> WindowOptions {
    WindowOptions::default()
        .with_open(open)
        .with_behavior(behavior)
}

fn render_floating_layer_with_overlay(
    cx: &mut ElementContext<'_, TestHost>,
    open: fret_runtime::Model<bool>,
    variant: FloatingLayerOverlayVariant,
    overlay_id_out: Rc<Cell<Option<GlobalElementId>>>,
) -> crate::Elements {
    overlay_id_out.set(None);

    crate::imui_raw(cx, |ui| {
        ui.floating_layer("layer", |ui| {
            let open_for_request = open.clone();
            let overlay_id_out = overlay_id_out.clone();

            let _ = ui.window_with_options(
                "a",
                "A",
                Point::new(Px(10.0), Px(10.0)),
                window_behavior_options(FloatingWindowOptions::default()),
                move |ui| {
                    let is_open = ui
                        .cx_mut()
                        .read_model(
                            &open_for_request,
                            fret_ui::Invalidation::Paint,
                            |_app, v| *v,
                        )
                        .unwrap_or(false);

                    ui.vertical(|ui| {
                        let anchor = ui.cx_mut().named("overlay-anchor", |cx| {
                            cx.container(
                                {
                                    let mut props = fret_ui::element::ContainerProps::default();
                                    props.layout.size.width = fret_ui::element::Length::Px(Px(1.0));
                                    props.layout.size.height =
                                        fret_ui::element::Length::Px(Px(1.0));
                                    props
                                },
                                |_cx| Vec::new(),
                            )
                        });
                        let trigger_id = anchor.id;
                        ui.add(anchor);

                        // Ensure stable bounds for overlap hit tests.
                        let body = ui.cx_mut().container(
                            {
                                let mut props = fret_ui::element::ContainerProps::default();
                                props.layout.size.width = fret_ui::element::Length::Px(Px(220.0));
                                props.layout.size.height = fret_ui::element::Length::Px(Px(140.0));
                                props
                            },
                            |_cx| Vec::new(),
                        );
                        ui.add(body);

                        if !is_open {
                            return;
                        }

                        let overlay_key = match variant {
                            FloatingLayerOverlayVariant::Menu => "menu",
                            FloatingLayerOverlayVariant::Popover => "popover",
                        };
                        let overlay_id = ui.cx_mut().named(overlay_key, |cx| cx.root_id());
                        overlay_id_out.set(Some(overlay_id));

                        let content = ui.cx_mut().container(
                            {
                                let mut props = fret_ui::element::ContainerProps::default();
                                props.layout.size.width = fret_ui::element::Length::Px(Px(140.0));
                                props.layout.size.height = fret_ui::element::Length::Px(Px(80.0));
                                props
                            },
                            |cx| vec![cx.text("Overlay")],
                        );

                        let open_for_dismiss = open_for_request.clone();
                        let on_dismiss_request: OnDismissRequest =
                            Arc::new(move |host, acx, req: &mut DismissRequestCx| {
                                match req.reason {
                                    DismissReason::Escape | DismissReason::OutsidePress { .. } => {
                                        let _ = host
                                            .models_mut()
                                            .update(&open_for_dismiss, |v| *v = false);
                                        host.notify(acx);
                                    }
                                    _ => {}
                                }
                            });

                        let mut req = match variant {
                            FloatingLayerOverlayVariant::Menu => OverlayRequest::dismissible_menu(
                                overlay_id,
                                trigger_id,
                                open_for_request.clone(),
                                OverlayPresence::instant(true),
                                vec![content],
                            ),
                            FloatingLayerOverlayVariant::Popover => {
                                OverlayRequest::dismissible_popover(
                                    overlay_id,
                                    trigger_id,
                                    open_for_request.clone(),
                                    OverlayPresence::instant(true),
                                    vec![content],
                                )
                            }
                        };
                        req.dismissible_on_dismiss_request = Some(on_dismiss_request);
                        OverlayController::request(ui.cx_mut(), req);
                    });
                },
            );

            let _ = ui.window_with_options(
                "b",
                "B",
                Point::new(Px(90.0), Px(10.0)),
                window_behavior_options(FloatingWindowOptions::default()),
                |ui| {
                    let body = ui.cx_mut().container(
                        {
                            let mut props = fret_ui::element::ContainerProps::default();
                            props.layout.size.width = fret_ui::element::Length::Px(Px(240.0));
                            props.layout.size.height = fret_ui::element::Length::Px(Px(140.0));
                            props
                        },
                        |_cx| Vec::new(),
                    );
                    ui.add(body);
                },
            );
        });
    })
}

mod composition;
mod floating;
mod interaction;
mod models;
mod popup_hover;
