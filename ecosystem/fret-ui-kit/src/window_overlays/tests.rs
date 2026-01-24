use super::state::WindowOverlays;
use super::*;

use crate::declarative::action_hooks::ActionHooksExt;
use std::sync::Arc;

use fret_app::App;
use fret_core::AppWindowId;
use fret_core::Event;
use fret_core::PointerId;
use fret_core::{PathCommand, SvgId, SvgService};
use fret_core::{PathConstraints, PathId, PathMetrics, PathService, PathStyle};
use fret_core::{
    Point, Px, Rect, TextBlobId, TextConstraints, TextInput, TextMetrics, TextService,
};
use fret_runtime::{FrameId, Model};
use fret_ui::UiTree;
use fret_ui::action::{DismissReason, UiActionHostAdapter};
use fret_ui::element::{
    ContainerProps, InsetStyle, LayoutStyle, Length, PointerRegionProps, PositionStyle,
    PressableProps, ScrollAxis, SizeStyle, WheelRegionProps,
};
use fret_ui::elements::GlobalElementId;
use fret_ui::scroll::ScrollHandle;

#[derive(Default)]
struct FakeServices;

fn begin_frame(app: &mut App, window: AppWindowId) {
    let next = FrameId(app.frame_id().0.saturating_add(1));
    app.set_frame_id(next);
    super::begin_frame(app, window);
}

impl TextService for FakeServices {
    fn prepare(
        &mut self,
        _input: &TextInput,
        _constraints: TextConstraints,
    ) -> (TextBlobId, TextMetrics) {
        (
            TextBlobId::default(),
            TextMetrics {
                size: fret_core::Size::new(Px(0.0), Px(0.0)),
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

fn render_base_with_trigger(
    ui: &mut UiTree<App>,
    app: &mut App,
    services: &mut dyn fret_core::UiServices,
    window: AppWindowId,
    bounds: Rect,
    open: Model<bool>,
) -> GlobalElementId {
    begin_frame(app, window);

    let mut trigger_id: Option<GlobalElementId> = None;
    let root = fret_ui::declarative::render_root(ui, app, services, window, bounds, "test", |cx| {
        vec![cx.pressable_with_id(
            PressableProps {
                layout: {
                    let mut layout = LayoutStyle::default();
                    layout.size.width = Length::Px(Px(80.0));
                    layout.size.height = Length::Px(Px(32.0));
                    layout
                },
                ..Default::default()
            },
            |cx, _st, id| {
                cx.pressable_toggle_bool(&open);
                trigger_id = Some(id);
                vec![cx.container(ContainerProps::default(), |_| Vec::new())]
            },
        )]
    });
    ui.set_root(root);
    trigger_id.expect("trigger id")
}

#[test]
fn window_resize_closes_modal_overlays_that_opt_in() {
    let mut app = App::new();
    let mut ui = UiTree::new();
    let mut services = FakeServices::default();
    let window = AppWindowId::default();

    let open = app.models_mut().insert(true);
    fret_runtime::apply_window_metrics_event(&mut app, window, &Event::WindowFocusChanged(true));

    let bounds_a = Rect::new(
        Point::new(Px(0.0), Px(0.0)),
        fret_core::Size::new(Px(200.0), Px(120.0)),
    );
    let bounds_b = Rect::new(
        Point::new(Px(0.0), Px(0.0)),
        fret_core::Size::new(Px(300.0), Px(120.0)),
    );

    let _trigger = render_base_with_trigger(
        &mut ui,
        &mut app,
        &mut services,
        window,
        bounds_a,
        open.clone(),
    );

    request_modal_for_window(
        &mut app,
        window,
        ModalRequest {
            id: GlobalElementId(0x1),
            root_name: "modal".into(),
            trigger: None,
            close_on_window_focus_lost: false,
            close_on_window_resize: true,
            open: open.clone(),
            present: true,
            initial_focus: None,
            on_open_auto_focus: None,
            on_close_auto_focus: None,
            on_dismiss_request: None,
            children: Vec::new(),
        },
    );
    render(&mut ui, &mut app, &mut services, window, bounds_a);
    assert_eq!(app.models().get_copied(&open), Some(true));

    begin_frame(&mut app, window);
    request_modal_for_window(
        &mut app,
        window,
        ModalRequest {
            id: GlobalElementId(0x1),
            root_name: "modal".into(),
            trigger: None,
            close_on_window_focus_lost: false,
            close_on_window_resize: true,
            open: open.clone(),
            present: true,
            initial_focus: None,
            on_open_auto_focus: None,
            on_close_auto_focus: None,
            on_dismiss_request: None,
            children: Vec::new(),
        },
    );
    render(&mut ui, &mut app, &mut services, window, bounds_b);
    assert_eq!(app.models().get_copied(&open), Some(false));
}

#[test]
fn window_focus_lost_closes_modal_overlays_that_opt_in() {
    let mut app = App::new();
    let mut ui = UiTree::new();
    let mut services = FakeServices::default();
    let window = AppWindowId::default();

    let open = app.models_mut().insert(true);

    let bounds = Rect::new(
        Point::new(Px(0.0), Px(0.0)),
        fret_core::Size::new(Px(200.0), Px(120.0)),
    );

    let _trigger = render_base_with_trigger(
        &mut ui,
        &mut app,
        &mut services,
        window,
        bounds,
        open.clone(),
    );

    fret_runtime::apply_window_metrics_event(&mut app, window, &Event::WindowFocusChanged(true));
    request_modal_for_window(
        &mut app,
        window,
        ModalRequest {
            id: GlobalElementId(0x1),
            root_name: "modal".into(),
            trigger: None,
            close_on_window_focus_lost: true,
            close_on_window_resize: false,
            open: open.clone(),
            present: true,
            initial_focus: None,
            on_open_auto_focus: None,
            on_close_auto_focus: None,
            on_dismiss_request: None,
            children: Vec::new(),
        },
    );
    render(&mut ui, &mut app, &mut services, window, bounds);
    assert_eq!(app.models().get_copied(&open), Some(true));

    begin_frame(&mut app, window);
    fret_runtime::apply_window_metrics_event(&mut app, window, &Event::WindowFocusChanged(false));
    request_modal_for_window(
        &mut app,
        window,
        ModalRequest {
            id: GlobalElementId(0x1),
            root_name: "modal".into(),
            trigger: None,
            close_on_window_focus_lost: true,
            close_on_window_resize: false,
            open: open.clone(),
            present: true,
            initial_focus: None,
            on_open_auto_focus: None,
            on_close_auto_focus: None,
            on_dismiss_request: None,
            children: Vec::new(),
        },
    );
    render(&mut ui, &mut app, &mut services, window, bounds);
    assert_eq!(app.models().get_copied(&open), Some(false));
}

#[test]
fn cached_modal_request_is_synthesized_when_open_without_rerender() {
    let mut app = App::new();
    let mut ui = UiTree::new();
    let mut services = FakeServices::default();
    let window = AppWindowId::default();

    let open = app.models_mut().insert(false);
    let bounds = Rect::new(
        Point::new(Px(0.0), Px(0.0)),
        fret_core::Size::new(Px(200.0), Px(120.0)),
    );

    app.set_frame_id(FrameId(1));
    render_base_with_trigger(
        &mut ui,
        &mut app,
        &mut services,
        window,
        bounds,
        open.clone(),
    );

    request_modal_for_window(
        &mut app,
        window,
        ModalRequest {
            id: GlobalElementId(0x1),
            root_name: "modal".into(),
            trigger: None,
            close_on_window_focus_lost: false,
            close_on_window_resize: false,
            open: open.clone(),
            present: false,
            initial_focus: None,
            on_open_auto_focus: None,
            on_close_auto_focus: None,
            on_dismiss_request: None,
            children: Vec::new(),
        },
    );
    render(&mut ui, &mut app, &mut services, window, bounds);

    app.set_frame_id(FrameId(2));
    begin_frame(&mut app, window);
    let _ = app.models_mut().update(&open, |v| *v = true);
    render(&mut ui, &mut app, &mut services, window, bounds);

    let layer = app.with_global_mut_untracked(WindowOverlays::default, |overlays, _app| {
        overlays
            .modals
            .get(&(window, GlobalElementId(0x1)))
            .map(|m| m.layer)
    });
    let layer = layer.expect("modal layer");
    assert!(ui.is_layer_visible(layer));
}

#[test]
fn cached_popover_request_is_synthesized_when_open_without_rerender() {
    let mut app = App::new();
    let mut ui = UiTree::new();
    let mut services = FakeServices::default();
    let window = AppWindowId::default();

    let open = app.models_mut().insert(false);
    let bounds = Rect::new(
        Point::new(Px(0.0), Px(0.0)),
        fret_core::Size::new(Px(200.0), Px(120.0)),
    );

    app.set_frame_id(FrameId(1));
    let trigger = render_base_with_trigger(
        &mut ui,
        &mut app,
        &mut services,
        window,
        bounds,
        open.clone(),
    );

    request_dismissible_popover_for_window(
        &mut app,
        window,
        DismissiblePopoverRequest {
            id: GlobalElementId(0x2),
            root_name: "popover".into(),
            trigger,
            dismissable_branches: Vec::new(),
            consume_outside_pointer_events: false,
            disable_outside_pointer_events: false,
            close_on_window_focus_lost: false,
            close_on_window_resize: false,
            open: open.clone(),
            present: false,
            initial_focus: None,
            on_open_auto_focus: None,
            on_close_auto_focus: None,
            on_dismiss_request: None,
            on_pointer_move: None,
            children: Vec::new(),
        },
    );
    render(&mut ui, &mut app, &mut services, window, bounds);

    app.set_frame_id(FrameId(2));
    begin_frame(&mut app, window);
    let _ = app.models_mut().update(&open, |v| *v = true);
    render(&mut ui, &mut app, &mut services, window, bounds);

    let layer = app.with_global_mut_untracked(WindowOverlays::default, |overlays, _app| {
        overlays
            .popovers
            .get(&(window, GlobalElementId(0x2)))
            .map(|p| p.layer)
    });
    let layer = layer.expect("popover layer");
    assert!(ui.is_layer_visible(layer));
}

fn render_base_with_trigger_and_underlay(
    ui: &mut UiTree<App>,
    app: &mut App,
    services: &mut dyn fret_core::UiServices,
    window: AppWindowId,
    bounds: Rect,
    open: Model<bool>,
    underlay_clicked: Model<bool>,
) -> (GlobalElementId, GlobalElementId) {
    begin_frame(app, window);

    let mut trigger_id: Option<GlobalElementId> = None;
    let mut underlay_id: Option<GlobalElementId> = None;

    let root = fret_ui::declarative::render_root(ui, app, services, window, bounds, "test", |cx| {
        vec![cx.container(
            ContainerProps {
                layout: {
                    let mut layout = LayoutStyle::default();
                    layout.size.width = Length::Fill;
                    layout.size.height = Length::Fill;
                    layout
                },
                ..Default::default()
            },
            |cx| {
                vec![
                    cx.pressable_with_id(
                        PressableProps {
                            layout: {
                                LayoutStyle {
                                    position: PositionStyle::Absolute,
                                    inset: InsetStyle {
                                        left: Some(Px(0.0)),
                                        top: Some(Px(0.0)),
                                        ..Default::default()
                                    },
                                    size: SizeStyle {
                                        width: Length::Px(Px(80.0)),
                                        height: Length::Px(Px(32.0)),
                                        ..Default::default()
                                    },
                                    ..Default::default()
                                }
                            },
                            enabled: true,
                            focusable: true,
                            ..Default::default()
                        },
                        |cx, _st, id| {
                            cx.pressable_toggle_bool(&open);
                            trigger_id = Some(id);
                            Vec::new()
                        },
                    ),
                    cx.pressable_with_id(
                        PressableProps {
                            layout: {
                                LayoutStyle {
                                    position: PositionStyle::Absolute,
                                    inset: InsetStyle {
                                        left: Some(Px(0.0)),
                                        top: Some(Px(120.0)),
                                        ..Default::default()
                                    },
                                    size: SizeStyle {
                                        width: Length::Px(Px(160.0)),
                                        height: Length::Px(Px(32.0)),
                                        ..Default::default()
                                    },
                                    ..Default::default()
                                }
                            },
                            enabled: true,
                            focusable: true,
                            ..Default::default()
                        },
                        |cx, _st, id| {
                            cx.pressable_toggle_bool(&underlay_clicked);
                            underlay_id = Some(id);
                            Vec::new()
                        },
                    ),
                ]
            },
        )]
    });
    ui.set_root(root);

    (
        trigger_id.expect("trigger id"),
        underlay_id.expect("underlay id"),
    )
}

fn render_base_with_trigger_and_underlay_pointer_move(
    ui: &mut UiTree<App>,
    app: &mut App,
    services: &mut dyn fret_core::UiServices,
    window: AppWindowId,
    bounds: Rect,
    open: Model<bool>,
    underlay_moved: Model<bool>,
    underlay_scroll: ScrollHandle,
) -> (GlobalElementId, GlobalElementId) {
    begin_frame(app, window);

    let mut trigger_id: Option<GlobalElementId> = None;
    let mut underlay_id: Option<GlobalElementId> = None;

    let root = fret_ui::declarative::render_root(ui, app, services, window, bounds, "test", |cx| {
        vec![cx.container(
            ContainerProps {
                layout: {
                    let mut layout = LayoutStyle::default();
                    layout.size.width = Length::Fill;
                    layout.size.height = Length::Fill;
                    layout
                },
                ..Default::default()
            },
            |cx| {
                let trigger = cx.pressable_with_id(
                    PressableProps {
                        layout: {
                            LayoutStyle {
                                position: PositionStyle::Absolute,
                                inset: InsetStyle {
                                    left: Some(Px(0.0)),
                                    top: Some(Px(0.0)),
                                    ..Default::default()
                                },
                                size: SizeStyle {
                                    width: Length::Px(Px(80.0)),
                                    height: Length::Px(Px(32.0)),
                                    ..Default::default()
                                },
                                ..Default::default()
                            }
                        },
                        enabled: true,
                        focusable: true,
                        ..Default::default()
                    },
                    |cx, _st, id| {
                        cx.pressable_toggle_bool(&open);
                        trigger_id = Some(id);
                        Vec::new()
                    },
                );

                let underlay = cx.wheel_region(
                    WheelRegionProps {
                        layout: {
                            LayoutStyle {
                                position: PositionStyle::Absolute,
                                inset: InsetStyle {
                                    left: Some(Px(0.0)),
                                    top: Some(Px(120.0)),
                                    ..Default::default()
                                },
                                size: SizeStyle {
                                    width: Length::Px(Px(160.0)),
                                    height: Length::Px(Px(32.0)),
                                    ..Default::default()
                                },
                                ..Default::default()
                            }
                        },
                        axis: ScrollAxis::Y,
                        scroll_target: None,
                        scroll_handle: underlay_scroll,
                    },
                    |cx| {
                        let underlay = cx.pointer_region(
                            PointerRegionProps {
                                layout: {
                                    let mut layout = LayoutStyle::default();
                                    layout.size.width = Length::Fill;
                                    layout.size.height = Length::Fill;
                                    layout
                                },
                                enabled: true,
                            },
                            |cx| {
                                let underlay_moved = underlay_moved.clone();
                                cx.pointer_region_on_pointer_move(Arc::new(
                                    move |host, _cx, _mv| {
                                        let _ = host
                                            .models_mut()
                                            .update(&underlay_moved, |v| *v = true);
                                        false
                                    },
                                ));
                                Vec::new()
                            },
                        );
                        underlay_id = Some(underlay.id);
                        vec![underlay]
                    },
                );

                vec![trigger, underlay]
            },
        )]
    });
    ui.set_root(root);

    (
        trigger_id.expect("trigger id"),
        underlay_id.expect("underlay id"),
    )
}

fn render_base_with_trigger_and_underlay_pressable_wheel(
    ui: &mut UiTree<App>,
    app: &mut App,
    services: &mut dyn fret_core::UiServices,
    window: AppWindowId,
    bounds: Rect,
    open: Model<bool>,
    underlay_clicked: Model<bool>,
    underlay_scroll: ScrollHandle,
) -> (GlobalElementId, GlobalElementId) {
    begin_frame(app, window);

    let mut trigger_id: Option<GlobalElementId> = None;
    let mut underlay_id: Option<GlobalElementId> = None;

    let root = fret_ui::declarative::render_root(ui, app, services, window, bounds, "test", |cx| {
        vec![cx.container(
            ContainerProps {
                layout: {
                    let mut layout = LayoutStyle::default();
                    layout.size.width = Length::Fill;
                    layout.size.height = Length::Fill;
                    layout
                },
                ..Default::default()
            },
            |cx| {
                let trigger = cx.pressable_with_id(
                    PressableProps {
                        layout: {
                            LayoutStyle {
                                position: PositionStyle::Absolute,
                                inset: InsetStyle {
                                    left: Some(Px(0.0)),
                                    top: Some(Px(0.0)),
                                    ..Default::default()
                                },
                                size: SizeStyle {
                                    width: Length::Px(Px(80.0)),
                                    height: Length::Px(Px(32.0)),
                                    ..Default::default()
                                },
                                ..Default::default()
                            }
                        },
                        enabled: true,
                        focusable: true,
                        ..Default::default()
                    },
                    |cx, _st, id| {
                        cx.pressable_toggle_bool(&open);
                        trigger_id = Some(id);
                        Vec::new()
                    },
                );

                let underlay = cx.wheel_region(
                    WheelRegionProps {
                        layout: {
                            LayoutStyle {
                                position: PositionStyle::Absolute,
                                inset: InsetStyle {
                                    left: Some(Px(0.0)),
                                    top: Some(Px(120.0)),
                                    ..Default::default()
                                },
                                size: SizeStyle {
                                    width: Length::Px(Px(160.0)),
                                    height: Length::Px(Px(32.0)),
                                    ..Default::default()
                                },
                                ..Default::default()
                            }
                        },
                        axis: ScrollAxis::Y,
                        scroll_target: None,
                        scroll_handle: underlay_scroll,
                    },
                    |cx| {
                        let underlay_clicked = underlay_clicked.clone();
                        vec![cx.pressable_with_id(
                            PressableProps {
                                layout: {
                                    let mut layout = LayoutStyle::default();
                                    layout.size.width = Length::Fill;
                                    layout.size.height = Length::Fill;
                                    layout
                                },
                                enabled: true,
                                focusable: true,
                                ..Default::default()
                            },
                            |cx, _st, id| {
                                cx.pressable_toggle_bool(&underlay_clicked);
                                underlay_id = Some(id);
                                Vec::new()
                            },
                        )]
                    },
                );

                vec![trigger, underlay]
            },
        )]
    });
    ui.set_root(root);

    (
        trigger_id.expect("trigger id"),
        underlay_id.expect("underlay id"),
    )
}

#[test]
fn dismissible_popover_closes_on_outside_press() {
    let window = AppWindowId::default();
    let mut app = App::new();
    let mut ui: UiTree<App> = UiTree::new();
    ui.set_window(window);

    let open = app.models_mut().insert(false);

    let mut services = FakeServices;
    let bounds = Rect::new(
        Point::new(Px(0.0), Px(0.0)),
        fret_core::Size::new(Px(300.0), Px(200.0)),
    );

    // First frame: render base to establish stable bounds for the trigger element.
    let trigger = render_base_with_trigger(
        &mut ui,
        &mut app,
        &mut services,
        window,
        bounds,
        open.clone(),
    );
    ui.layout_all(&mut app, &mut services, bounds, 1.0);

    // Open via click.
    ui.dispatch_event(
        &mut app,
        &mut services,
        &fret_core::Event::Pointer(fret_core::PointerEvent::Down {
            position: Point::new(Px(10.0), Px(10.0)),
            button: fret_core::MouseButton::Left,
            modifiers: fret_core::Modifiers::default(),
            click_count: 1,
            pointer_id: PointerId(0),
            pointer_type: Default::default(),
        }),
    );
    ui.dispatch_event(
        &mut app,
        &mut services,
        &fret_core::Event::Pointer(fret_core::PointerEvent::Up {
            position: Point::new(Px(10.0), Px(10.0)),
            button: fret_core::MouseButton::Left,
            modifiers: fret_core::Modifiers::default(),
            is_click: true,
            click_count: 1,
            pointer_id: PointerId(0),
            pointer_type: Default::default(),
        }),
    );
    assert_eq!(app.models().get_copied(&open), Some(true));

    // Second frame: request and render a dismissible popover.
    begin_frame(&mut app, window);
    let _ = render_base_with_trigger(
        &mut ui,
        &mut app,
        &mut services,
        window,
        bounds,
        open.clone(),
    );

    request_dismissible_popover_for_window(
        &mut app,
        window,
        DismissiblePopoverRequest {
            id: trigger,
            root_name: popover_root_name(trigger),
            trigger,
            dismissable_branches: Vec::new(),
            consume_outside_pointer_events: false,
            disable_outside_pointer_events: false,
            close_on_window_focus_lost: false,
            close_on_window_resize: false,
            open: open.clone(),
            present: true,
            initial_focus: None,
            on_open_auto_focus: None,
            on_close_auto_focus: None,
            on_dismiss_request: None,
            on_pointer_move: None,
            children: vec![],
        },
    );

    render(&mut ui, &mut app, &mut services, window, bounds);
    ui.layout_all(&mut app, &mut services, bounds, 1.0);

    assert_eq!(ui.captured(), None);

    // Pointer down outside should close (observer pass).
    ui.dispatch_event(
        &mut app,
        &mut services,
        &fret_core::Event::Pointer(fret_core::PointerEvent::Down {
            position: Point::new(Px(250.0), Px(180.0)),
            button: fret_core::MouseButton::Left,
            modifiers: fret_core::Modifiers::default(),
            click_count: 1,
            pointer_id: PointerId(0),
            pointer_type: Default::default(),
        }),
    );
    assert_eq!(app.models().get_copied(&open), Some(false));
}

#[test]
fn non_modal_overlay_open_auto_focus_handler_can_prevent_default_focus() {
    let window = AppWindowId::default();
    let mut app = App::new();
    let mut ui: UiTree<App> = UiTree::new();
    ui.set_window(window);

    let open = app.models_mut().insert(true);

    let mut services = FakeServices;
    let bounds = Rect::new(
        Point::new(Px(0.0), Px(0.0)),
        fret_core::Size::new(Px(300.0), Px(200.0)),
    );

    // First frame: base root to establish stable element mappings for the trigger.
    let trigger = render_base_with_trigger(
        &mut ui,
        &mut app,
        &mut services,
        window,
        bounds,
        open.clone(),
    );
    ui.layout_all(&mut app, &mut services, bounds, 1.0);

    let trigger_node =
        fret_ui::elements::node_for_element(&mut app, window, trigger).expect("trigger node");
    ui.set_focus(Some(trigger_node));

    let overlay_children =
        fret_ui::elements::with_element_cx(&mut app, window, bounds, "popover-child", |cx| {
            vec![cx.pressable(
                PressableProps {
                    layout: {
                        let mut layout = LayoutStyle::default();
                        layout.size.width = Length::Px(Px(80.0));
                        layout.size.height = Length::Px(Px(32.0));
                        layout
                    },
                    enabled: true,
                    focusable: true,
                    ..Default::default()
                },
                |_cx, _st| Vec::new(),
            )]
        });

    let on_open_auto_focus: fret_ui::action::OnOpenAutoFocus =
        Arc::new(|_host, _cx, req| req.prevent_default());

    // Second frame: mount a non-modal overlay and suppress default initial focus.
    begin_frame(&mut app, window);
    let _ = render_base_with_trigger(
        &mut ui,
        &mut app,
        &mut services,
        window,
        bounds,
        open.clone(),
    );
    request_dismissible_popover_for_window(
        &mut app,
        window,
        DismissiblePopoverRequest {
            id: trigger,
            root_name: popover_root_name(trigger),
            trigger,
            dismissable_branches: Vec::new(),
            consume_outside_pointer_events: false,
            disable_outside_pointer_events: false,
            close_on_window_focus_lost: false,
            close_on_window_resize: false,
            open: open.clone(),
            present: true,
            initial_focus: None,
            on_open_auto_focus: Some(on_open_auto_focus),
            on_close_auto_focus: None,
            on_dismiss_request: None,
            on_pointer_move: None,
            children: overlay_children,
        },
    );
    render(&mut ui, &mut app, &mut services, window, bounds);
    ui.layout_all(&mut app, &mut services, bounds, 1.0);

    assert_eq!(ui.focus(), Some(trigger_node));
}

#[test]
fn dismissible_popover_does_not_close_on_inside_press() {
    let window = AppWindowId::default();
    let mut app = App::new();
    let mut ui: UiTree<App> = UiTree::new();
    ui.set_window(window);

    let open = app.models_mut().insert(false);

    let mut services = FakeServices;
    let bounds = Rect::new(
        Point::new(Px(0.0), Px(0.0)),
        fret_core::Size::new(Px(300.0), Px(200.0)),
    );

    // First frame: render base to establish stable bounds for the trigger element.
    let trigger = render_base_with_trigger(
        &mut ui,
        &mut app,
        &mut services,
        window,
        bounds,
        open.clone(),
    );
    ui.layout_all(&mut app, &mut services, bounds, 1.0);

    // Open via click.
    ui.dispatch_event(
        &mut app,
        &mut services,
        &fret_core::Event::Pointer(fret_core::PointerEvent::Down {
            position: Point::new(Px(10.0), Px(10.0)),
            button: fret_core::MouseButton::Left,
            modifiers: fret_core::Modifiers::default(),
            click_count: 1,
            pointer_id: PointerId(0),
            pointer_type: Default::default(),
        }),
    );
    ui.dispatch_event(
        &mut app,
        &mut services,
        &fret_core::Event::Pointer(fret_core::PointerEvent::Up {
            position: Point::new(Px(10.0), Px(10.0)),
            button: fret_core::MouseButton::Left,
            modifiers: fret_core::Modifiers::default(),
            is_click: true,
            click_count: 1,
            pointer_id: PointerId(0),
            pointer_type: Default::default(),
        }),
    );
    assert_eq!(app.models().get_copied(&open), Some(true));

    // Second frame: request and render a dismissible popover with a non-pressable child so
    // the pointer-down bubbles to the root in the normal dispatch path.
    begin_frame(&mut app, window);
    let _ = render_base_with_trigger(
        &mut ui,
        &mut app,
        &mut services,
        window,
        bounds,
        open.clone(),
    );

    let root_name = popover_root_name(trigger);
    let children = fret_ui::elements::with_element_cx(&mut app, window, bounds, &root_name, |cx| {
        vec![cx.container(
            ContainerProps {
                layout: LayoutStyle {
                    position: PositionStyle::Absolute,
                    inset: fret_ui::element::InsetStyle {
                        top: Some(Px(40.0)),
                        left: Some(Px(40.0)),
                        ..Default::default()
                    },
                    size: fret_ui::element::SizeStyle {
                        width: Length::Px(Px(120.0)),
                        height: Length::Px(Px(80.0)),
                        ..Default::default()
                    },
                    ..Default::default()
                },
                ..Default::default()
            },
            |_| Vec::new(),
        )]
    });

    request_dismissible_popover_for_window(
        &mut app,
        window,
        DismissiblePopoverRequest {
            id: trigger,
            root_name,
            trigger,
            dismissable_branches: Vec::new(),
            consume_outside_pointer_events: false,
            disable_outside_pointer_events: false,
            close_on_window_focus_lost: false,
            close_on_window_resize: false,
            open: open.clone(),
            present: true,
            initial_focus: None,
            on_open_auto_focus: None,
            on_close_auto_focus: None,
            on_dismiss_request: None,
            on_pointer_move: None,
            children,
        },
    );

    render(&mut ui, &mut app, &mut services, window, bounds);
    ui.layout_all(&mut app, &mut services, bounds, 1.0);

    assert_eq!(ui.captured(), None);

    // Pointer down inside the popover content should not close it.
    ui.dispatch_event(
        &mut app,
        &mut services,
        &fret_core::Event::Pointer(fret_core::PointerEvent::Down {
            position: Point::new(Px(50.0), Px(50.0)),
            button: fret_core::MouseButton::Left,
            modifiers: fret_core::Modifiers::default(),
            click_count: 1,
            pointer_id: PointerId(0),
            pointer_type: Default::default(),
        }),
    );
    assert_eq!(app.models().get_copied(&open), Some(true));
}

#[test]
fn dismissible_popover_does_not_close_on_outside_press_in_branch_subtree() {
    let window = AppWindowId::default();
    let mut app = App::new();
    let mut ui: UiTree<App> = UiTree::new();
    ui.set_window(window);

    let open = app.models_mut().insert(false);
    let underlay_clicked = app.models_mut().insert(false);

    let mut services = FakeServices;
    let bounds = Rect::new(
        Point::new(Px(0.0), Px(0.0)),
        fret_core::Size::new(Px(300.0), Px(200.0)),
    );

    // First frame: base layer with trigger + underlay pressable.
    let (_trigger, underlay) = render_base_with_trigger_and_underlay(
        &mut ui,
        &mut app,
        &mut services,
        window,
        bounds,
        open.clone(),
        underlay_clicked.clone(),
    );
    ui.layout_all(&mut app, &mut services, bounds, 1.0);

    // Open via click on the trigger.
    ui.dispatch_event(
        &mut app,
        &mut services,
        &fret_core::Event::Pointer(fret_core::PointerEvent::Down {
            position: Point::new(Px(10.0), Px(10.0)),
            button: fret_core::MouseButton::Left,
            modifiers: fret_core::Modifiers::default(),
            click_count: 1,
            pointer_id: PointerId(0),
            pointer_type: Default::default(),
        }),
    );
    ui.dispatch_event(
        &mut app,
        &mut services,
        &fret_core::Event::Pointer(fret_core::PointerEvent::Up {
            position: Point::new(Px(10.0), Px(10.0)),
            button: fret_core::MouseButton::Left,
            modifiers: fret_core::Modifiers::default(),
            is_click: true,
            click_count: 1,
            pointer_id: PointerId(0),
            pointer_type: Default::default(),
        }),
    );
    assert_eq!(app.models().get_copied(&open), Some(true));

    // Second frame: request a dismissible popover, with a branch pointing at the underlay subtree.
    begin_frame(&mut app, window);
    let (trigger, _underlay2) = render_base_with_trigger_and_underlay(
        &mut ui,
        &mut app,
        &mut services,
        window,
        bounds,
        open.clone(),
        underlay_clicked.clone(),
    );

    request_dismissible_popover_for_window(
        &mut app,
        window,
        DismissiblePopoverRequest {
            id: trigger,
            root_name: popover_root_name(trigger),
            trigger,
            dismissable_branches: vec![underlay],
            consume_outside_pointer_events: false,
            disable_outside_pointer_events: false,
            close_on_window_focus_lost: false,
            close_on_window_resize: false,
            open: open.clone(),
            present: true,
            initial_focus: None,
            on_open_auto_focus: None,
            on_close_auto_focus: None,
            on_dismiss_request: None,
            on_pointer_move: None,
            children: Vec::new(),
        },
    );

    render(&mut ui, &mut app, &mut services, window, bounds);
    ui.layout_all(&mut app, &mut services, bounds, 1.0);

    // Clicking the underlay subtree should remain click-through and should NOT dismiss.
    ui.dispatch_event(
        &mut app,
        &mut services,
        &fret_core::Event::Pointer(fret_core::PointerEvent::Down {
            position: Point::new(Px(10.0), Px(130.0)),
            button: fret_core::MouseButton::Left,
            modifiers: fret_core::Modifiers::default(),
            click_count: 1,
            pointer_id: PointerId(0),
            pointer_type: Default::default(),
        }),
    );
    ui.dispatch_event(
        &mut app,
        &mut services,
        &fret_core::Event::Pointer(fret_core::PointerEvent::Up {
            position: Point::new(Px(10.0), Px(130.0)),
            button: fret_core::MouseButton::Left,
            modifiers: fret_core::Modifiers::default(),
            is_click: true,
            click_count: 1,
            pointer_id: PointerId(0),
            pointer_type: Default::default(),
        }),
    );

    assert_eq!(app.models().get_copied(&open), Some(true));
    assert_eq!(app.models().get_copied(&underlay_clicked), Some(true));

    // Third frame: keep requesting the overlay; it should still be open.
    begin_frame(&mut app, window);
    let (trigger, _underlay3) = render_base_with_trigger_and_underlay(
        &mut ui,
        &mut app,
        &mut services,
        window,
        bounds,
        open.clone(),
        underlay_clicked.clone(),
    );
    request_dismissible_popover_for_window(
        &mut app,
        window,
        DismissiblePopoverRequest {
            id: trigger,
            root_name: popover_root_name(trigger),
            trigger,
            dismissable_branches: vec![underlay],
            consume_outside_pointer_events: false,
            disable_outside_pointer_events: false,
            close_on_window_focus_lost: false,
            close_on_window_resize: false,
            open: open.clone(),
            present: true,
            initial_focus: None,
            on_open_auto_focus: None,
            on_close_auto_focus: None,
            on_dismiss_request: None,
            on_pointer_move: None,
            children: Vec::new(),
        },
    );
    render(&mut ui, &mut app, &mut services, window, bounds);
    ui.layout_all(&mut app, &mut services, bounds, 1.0);

    assert_eq!(app.models().get_copied(&open), Some(true));
}

#[test]
fn dismissible_popover_treats_trigger_as_implicit_branch() {
    let window = AppWindowId::default();
    let mut app = App::new();
    let mut ui: UiTree<App> = UiTree::new();
    ui.set_window(window);

    let open = app.models_mut().insert(false);
    let underlay_clicked = app.models_mut().insert(false);

    let mut services = FakeServices;
    let bounds = Rect::new(
        Point::new(Px(0.0), Px(0.0)),
        fret_core::Size::new(Px(300.0), Px(200.0)),
    );

    // First frame: base layer with trigger + underlay pressable.
    let (_trigger, _underlay) = render_base_with_trigger_and_underlay(
        &mut ui,
        &mut app,
        &mut services,
        window,
        bounds,
        open.clone(),
        underlay_clicked.clone(),
    );
    ui.layout_all(&mut app, &mut services, bounds, 1.0);

    // Open via click on the trigger.
    ui.dispatch_event(
        &mut app,
        &mut services,
        &fret_core::Event::Pointer(fret_core::PointerEvent::Down {
            position: Point::new(Px(10.0), Px(10.0)),
            button: fret_core::MouseButton::Left,
            modifiers: fret_core::Modifiers::default(),
            click_count: 1,
            pointer_id: PointerId(0),
            pointer_type: Default::default(),
        }),
    );
    ui.dispatch_event(
        &mut app,
        &mut services,
        &fret_core::Event::Pointer(fret_core::PointerEvent::Up {
            position: Point::new(Px(10.0), Px(10.0)),
            button: fret_core::MouseButton::Left,
            modifiers: fret_core::Modifiers::default(),
            is_click: true,
            click_count: 1,
            pointer_id: PointerId(0),
            pointer_type: Default::default(),
        }),
    );
    assert_eq!(app.models().get_copied(&open), Some(true));

    // Second frame: request a dismissible popover with no explicit branches.
    begin_frame(&mut app, window);
    let (trigger, _underlay2) = render_base_with_trigger_and_underlay(
        &mut ui,
        &mut app,
        &mut services,
        window,
        bounds,
        open.clone(),
        underlay_clicked.clone(),
    );

    request_dismissible_popover_for_window(
        &mut app,
        window,
        DismissiblePopoverRequest {
            id: trigger,
            root_name: popover_root_name(trigger),
            trigger,
            dismissable_branches: Vec::new(),
            consume_outside_pointer_events: false,
            disable_outside_pointer_events: false,
            close_on_window_focus_lost: false,
            close_on_window_resize: false,
            open: open.clone(),
            present: true,
            initial_focus: None,
            on_open_auto_focus: None,
            on_close_auto_focus: None,
            on_dismiss_request: None,
            on_pointer_move: None,
            children: Vec::new(),
        },
    );

    render(&mut ui, &mut app, &mut services, window, bounds);
    ui.layout_all(&mut app, &mut services, bounds, 1.0);

    // Clicking the trigger should close via toggle, and should NOT re-open due to outside-press
    // observer dismissal running before the trigger activation.
    ui.dispatch_event(
        &mut app,
        &mut services,
        &fret_core::Event::Pointer(fret_core::PointerEvent::Down {
            position: Point::new(Px(10.0), Px(10.0)),
            button: fret_core::MouseButton::Left,
            modifiers: fret_core::Modifiers::default(),
            click_count: 1,
            pointer_id: PointerId(0),
            pointer_type: Default::default(),
        }),
    );
    ui.dispatch_event(
        &mut app,
        &mut services,
        &fret_core::Event::Pointer(fret_core::PointerEvent::Up {
            position: Point::new(Px(10.0), Px(10.0)),
            button: fret_core::MouseButton::Left,
            modifiers: fret_core::Modifiers::default(),
            is_click: true,
            click_count: 1,
            pointer_id: PointerId(0),
            pointer_type: Default::default(),
        }),
    );

    assert_eq!(app.models().get_copied(&open), Some(false));
}

#[test]
fn dismissible_popover_closes_on_focus_change_outside() {
    let window = AppWindowId::default();
    let mut app = App::new();
    let mut ui: UiTree<App> = UiTree::new();
    ui.set_window(window);

    let open = app.models_mut().insert(false);
    let underlay_clicked = app.models_mut().insert(false);

    let mut services = FakeServices;
    let bounds = Rect::new(
        Point::new(Px(0.0), Px(0.0)),
        fret_core::Size::new(Px(300.0), Px(200.0)),
    );

    // First frame: base layer, open via trigger click.
    let (_trigger, underlay) = render_base_with_trigger_and_underlay(
        &mut ui,
        &mut app,
        &mut services,
        window,
        bounds,
        open.clone(),
        underlay_clicked.clone(),
    );
    ui.layout_all(&mut app, &mut services, bounds, 1.0);

    ui.dispatch_event(
        &mut app,
        &mut services,
        &fret_core::Event::Pointer(fret_core::PointerEvent::Down {
            position: Point::new(Px(10.0), Px(10.0)),
            button: fret_core::MouseButton::Left,
            modifiers: fret_core::Modifiers::default(),
            click_count: 1,
            pointer_id: PointerId(0),
            pointer_type: Default::default(),
        }),
    );
    ui.dispatch_event(
        &mut app,
        &mut services,
        &fret_core::Event::Pointer(fret_core::PointerEvent::Up {
            position: Point::new(Px(10.0), Px(10.0)),
            button: fret_core::MouseButton::Left,
            modifiers: fret_core::Modifiers::default(),
            is_click: true,
            click_count: 1,
            pointer_id: PointerId(0),
            pointer_type: Default::default(),
        }),
    );
    assert_eq!(app.models().get_copied(&open), Some(true));

    // Second frame: request the popover and render it.
    begin_frame(&mut app, window);
    let (trigger, _underlay2) = render_base_with_trigger_and_underlay(
        &mut ui,
        &mut app,
        &mut services,
        window,
        bounds,
        open.clone(),
        underlay_clicked.clone(),
    );
    request_dismissible_popover_for_window(
        &mut app,
        window,
        DismissiblePopoverRequest {
            id: trigger,
            root_name: popover_root_name(trigger),
            trigger,
            dismissable_branches: Vec::new(),
            consume_outside_pointer_events: false,
            disable_outside_pointer_events: false,
            close_on_window_focus_lost: false,
            close_on_window_resize: false,
            open: open.clone(),
            present: true,
            initial_focus: None,
            on_open_auto_focus: None,
            on_close_auto_focus: None,
            on_dismiss_request: None,
            on_pointer_move: None,
            children: Vec::new(),
        },
    );
    render(&mut ui, &mut app, &mut services, window, bounds);
    ui.layout_all(&mut app, &mut services, bounds, 1.0);

    // Simulate a focus change outside of the overlay subtree (e.g. Tab navigation).
    let underlay_node =
        fret_ui::elements::node_for_element(&mut app, window, underlay).expect("underlay node");
    ui.set_focus(Some(underlay_node));

    // Third frame: focus-outside should dismiss.
    begin_frame(&mut app, window);
    let (trigger, _underlay3) = render_base_with_trigger_and_underlay(
        &mut ui,
        &mut app,
        &mut services,
        window,
        bounds,
        open.clone(),
        underlay_clicked.clone(),
    );
    request_dismissible_popover_for_window(
        &mut app,
        window,
        DismissiblePopoverRequest {
            id: trigger,
            root_name: popover_root_name(trigger),
            trigger,
            dismissable_branches: Vec::new(),
            consume_outside_pointer_events: false,
            disable_outside_pointer_events: false,
            close_on_window_focus_lost: false,
            close_on_window_resize: false,
            open: open.clone(),
            present: true,
            initial_focus: None,
            on_open_auto_focus: None,
            on_close_auto_focus: None,
            on_dismiss_request: None,
            on_pointer_move: None,
            children: Vec::new(),
        },
    );
    render(&mut ui, &mut app, &mut services, window, bounds);
    ui.layout_all(&mut app, &mut services, bounds, 1.0);

    assert_eq!(app.models().get_copied(&open), Some(false));
}

#[test]
fn dismissible_popover_focus_outside_routes_through_dismiss_handler() {
    let window = AppWindowId::default();
    let mut app = App::new();
    let mut ui: UiTree<App> = UiTree::new();
    ui.set_window(window);

    let open = app.models_mut().insert(false);
    let underlay_clicked = app.models_mut().insert(false);

    let mut services = FakeServices;
    let bounds = Rect::new(
        Point::new(Px(0.0), Px(0.0)),
        fret_core::Size::new(Px(300.0), Px(200.0)),
    );

    // First frame: base layer, open via trigger click.
    let (_trigger, underlay) = render_base_with_trigger_and_underlay(
        &mut ui,
        &mut app,
        &mut services,
        window,
        bounds,
        open.clone(),
        underlay_clicked.clone(),
    );
    ui.layout_all(&mut app, &mut services, bounds, 1.0);

    ui.dispatch_event(
        &mut app,
        &mut services,
        &fret_core::Event::Pointer(fret_core::PointerEvent::Down {
            position: Point::new(Px(10.0), Px(10.0)),
            button: fret_core::MouseButton::Left,
            modifiers: fret_core::Modifiers::default(),
            click_count: 1,
            pointer_id: PointerId(0),
            pointer_type: Default::default(),
        }),
    );
    ui.dispatch_event(
        &mut app,
        &mut services,
        &fret_core::Event::Pointer(fret_core::PointerEvent::Up {
            position: Point::new(Px(10.0), Px(10.0)),
            button: fret_core::MouseButton::Left,
            modifiers: fret_core::Modifiers::default(),
            is_click: true,
            click_count: 1,
            pointer_id: PointerId(0),
            pointer_type: Default::default(),
        }),
    );
    assert_eq!(app.models().get_copied(&open), Some(true));

    // Second frame: request the popover and render it.
    begin_frame(&mut app, window);
    let (trigger, _underlay2) = render_base_with_trigger_and_underlay(
        &mut ui,
        &mut app,
        &mut services,
        window,
        bounds,
        open.clone(),
        underlay_clicked.clone(),
    );
    request_dismissible_popover_for_window(
        &mut app,
        window,
        DismissiblePopoverRequest {
            id: trigger,
            root_name: popover_root_name(trigger),
            trigger,
            dismissable_branches: Vec::new(),
            consume_outside_pointer_events: false,
            disable_outside_pointer_events: false,
            close_on_window_focus_lost: false,
            close_on_window_resize: false,
            open: open.clone(),
            present: true,
            initial_focus: None,
            on_open_auto_focus: None,
            on_close_auto_focus: None,
            on_dismiss_request: None,
            on_pointer_move: None,
            children: Vec::new(),
        },
    );
    render(&mut ui, &mut app, &mut services, window, bounds);
    ui.layout_all(&mut app, &mut services, bounds, 1.0);

    // Simulate a focus change outside of the overlay subtree (e.g. Tab navigation).
    let underlay_node =
        fret_ui::elements::node_for_element(&mut app, window, underlay).expect("underlay node");
    ui.set_focus(Some(underlay_node));

    let reason_cell: Arc<std::sync::Mutex<Option<DismissReason>>> =
        Arc::new(std::sync::Mutex::new(None));
    let reason_cell_for_handler = reason_cell.clone();
    let handler: fret_ui::action::OnDismissRequest = Arc::new(move |_host, _cx, req| {
        let mut lock = reason_cell_for_handler.lock().unwrap();
        *lock = Some(req.reason);
        req.prevent_default();
    });

    // Third frame: focus-outside should route through the dismiss handler. The handler chooses not
    // to close `open`, mirroring Radix `preventDefault` behavior.
    begin_frame(&mut app, window);
    let (trigger, _underlay3) = render_base_with_trigger_and_underlay(
        &mut ui,
        &mut app,
        &mut services,
        window,
        bounds,
        open.clone(),
        underlay_clicked.clone(),
    );
    request_dismissible_popover_for_window(
        &mut app,
        window,
        DismissiblePopoverRequest {
            id: trigger,
            root_name: popover_root_name(trigger),
            trigger,
            dismissable_branches: Vec::new(),
            consume_outside_pointer_events: false,
            disable_outside_pointer_events: false,
            close_on_window_focus_lost: false,
            close_on_window_resize: false,
            open: open.clone(),
            present: true,
            initial_focus: None,
            on_open_auto_focus: None,
            on_close_auto_focus: None,
            on_dismiss_request: Some(handler),
            on_pointer_move: None,
            children: Vec::new(),
        },
    );
    render(&mut ui, &mut app, &mut services, window, bounds);
    ui.layout_all(&mut app, &mut services, bounds, 1.0);

    assert_eq!(app.models().get_copied(&open), Some(true));
    assert_eq!(
        *reason_cell.lock().unwrap(),
        Some(DismissReason::FocusOutside)
    );
}

#[test]
fn dismissible_popover_does_not_close_on_focus_change_to_trigger() {
    let window = AppWindowId::default();
    let mut app = App::new();
    let mut ui: UiTree<App> = UiTree::new();
    ui.set_window(window);

    let open = app.models_mut().insert(false);
    let underlay_clicked = app.models_mut().insert(false);

    let mut services = FakeServices;
    let bounds = Rect::new(
        Point::new(Px(0.0), Px(0.0)),
        fret_core::Size::new(Px(300.0), Px(200.0)),
    );

    // First frame: base layer, open via trigger click.
    let (_trigger, _underlay) = render_base_with_trigger_and_underlay(
        &mut ui,
        &mut app,
        &mut services,
        window,
        bounds,
        open.clone(),
        underlay_clicked.clone(),
    );
    ui.layout_all(&mut app, &mut services, bounds, 1.0);

    ui.dispatch_event(
        &mut app,
        &mut services,
        &fret_core::Event::Pointer(fret_core::PointerEvent::Down {
            position: Point::new(Px(10.0), Px(10.0)),
            button: fret_core::MouseButton::Left,
            modifiers: fret_core::Modifiers::default(),
            click_count: 1,
            pointer_id: PointerId(0),
            pointer_type: Default::default(),
        }),
    );
    ui.dispatch_event(
        &mut app,
        &mut services,
        &fret_core::Event::Pointer(fret_core::PointerEvent::Up {
            position: Point::new(Px(10.0), Px(10.0)),
            button: fret_core::MouseButton::Left,
            modifiers: fret_core::Modifiers::default(),
            is_click: true,
            click_count: 1,
            pointer_id: PointerId(0),
            pointer_type: Default::default(),
        }),
    );
    assert_eq!(app.models().get_copied(&open), Some(true));

    // Second frame: request the popover and render it.
    begin_frame(&mut app, window);
    let (trigger, _underlay2) = render_base_with_trigger_and_underlay(
        &mut ui,
        &mut app,
        &mut services,
        window,
        bounds,
        open.clone(),
        underlay_clicked.clone(),
    );
    request_dismissible_popover_for_window(
        &mut app,
        window,
        DismissiblePopoverRequest {
            id: trigger,
            root_name: popover_root_name(trigger),
            trigger,
            dismissable_branches: Vec::new(),
            consume_outside_pointer_events: false,
            disable_outside_pointer_events: false,
            close_on_window_focus_lost: false,
            close_on_window_resize: false,
            open: open.clone(),
            present: true,
            initial_focus: None,
            on_open_auto_focus: None,
            on_close_auto_focus: None,
            on_dismiss_request: None,
            on_pointer_move: None,
            children: Vec::new(),
        },
    );
    render(&mut ui, &mut app, &mut services, window, bounds);
    ui.layout_all(&mut app, &mut services, bounds, 1.0);

    // Simulate a focus change to the trigger element (branch subtree).
    let trigger_node =
        fret_ui::elements::node_for_element(&mut app, window, trigger).expect("trigger node");
    ui.set_focus(Some(trigger_node));

    // Third frame: focus is outside the overlay root but inside the trigger branch, so it should
    // remain open.
    begin_frame(&mut app, window);
    let (trigger, _underlay3) = render_base_with_trigger_and_underlay(
        &mut ui,
        &mut app,
        &mut services,
        window,
        bounds,
        open.clone(),
        underlay_clicked.clone(),
    );
    request_dismissible_popover_for_window(
        &mut app,
        window,
        DismissiblePopoverRequest {
            id: trigger,
            root_name: popover_root_name(trigger),
            trigger,
            dismissable_branches: Vec::new(),
            consume_outside_pointer_events: false,
            disable_outside_pointer_events: false,
            close_on_window_focus_lost: false,
            close_on_window_resize: false,
            open: open.clone(),
            present: true,
            initial_focus: None,
            on_open_auto_focus: None,
            on_close_auto_focus: None,
            on_dismiss_request: None,
            on_pointer_move: None,
            children: Vec::new(),
        },
    );
    render(&mut ui, &mut app, &mut services, window, bounds);
    ui.layout_all(&mut app, &mut services, bounds, 1.0);

    assert_eq!(app.models().get_copied(&open), Some(true));
}

#[test]
fn toast_viewport_focus_command_focuses_active_toast_layer() {
    let window = AppWindowId::default();
    let mut app = App::new();
    let mut ui: UiTree<App> = UiTree::new();
    ui.set_window(window);
    let mut services = FakeServices::default();

    let bounds = Rect::new(
        Point::new(Px(0.0), Px(0.0)),
        fret_core::Size::new(Px(300.0), Px(200.0)),
    );

    begin_frame(&mut app, window);
    let base = fret_ui::declarative::render_root(
        &mut ui,
        &mut app,
        &mut services,
        window,
        bounds,
        "base",
        |_cx| Vec::new(),
    );
    ui.set_root(base);

    let store = toast_store(&mut app);
    let _ = toast_action(
        &mut UiActionHostAdapter { app: &mut app },
        store.clone(),
        window,
        ToastRequest::new("Hello"),
    );

    begin_frame(&mut app, window);
    let viewport_id = GlobalElementId(0xbeef);
    request_toast_layer_for_window(
        &mut app,
        window,
        ToastLayerRequest::new(viewport_id, store.clone()),
    );
    render(&mut ui, &mut app, &mut services, window, bounds);
    ui.layout_all(&mut app, &mut services, bounds, 1.0);

    let layer = app.with_global_mut(WindowOverlays::default, |overlays, _app| {
        overlays
            .toast_layers
            .get(&(window, viewport_id))
            .map(|l| l.layer)
            .expect("toast layer installed")
    });
    let expected = ui.layer_root(layer).expect("layer root");

    assert!(try_handle_window_command(
        &mut ui,
        &mut app,
        window,
        &fret_runtime::CommandId::from(TOAST_VIEWPORT_FOCUS_COMMAND),
    ));
    assert_eq!(ui.focus(), Some(expected));
}

#[test]
fn toast_hit_testing_tracks_render_transform() {
    let window = AppWindowId::default();
    let mut app = App::new();
    let mut ui: UiTree<App> = UiTree::new();
    ui.set_window(window);
    let mut services = FakeServices::default();

    let bounds = Rect::new(
        Point::new(Px(0.0), Px(0.0)),
        fret_core::Size::new(Px(600.0), Px(400.0)),
    );

    begin_frame(&mut app, window);
    let base = fret_ui::declarative::render_root(
        &mut ui,
        &mut app,
        &mut services,
        window,
        bounds,
        "base",
        |_cx| Vec::new(),
    );
    ui.set_root(base);

    let store = toast_store(&mut app);
    let toast_id = toast_action(
        &mut UiActionHostAdapter { app: &mut app },
        store.clone(),
        window,
        ToastRequest::new("Hello").duration(None),
    );

    // Simulate a completed drag to install a non-zero settle offset (render translation) for the
    // first frame.
    let (started, moved, ended) = app
        .models_mut()
        .update(&store, |st| {
            let _ = st.set_window_swipe_config(window, ToastSwipeDirection::Right, Px(1000.0));
            let started = st.begin_drag(window, toast_id, Point::new(Px(0.0), Px(0.0)));
            let moved = st
                .drag_move(window, toast_id, Point::new(Px(200.0), Px(0.0)))
                .is_some();
            let ended = st.end_drag(window, toast_id).is_some();
            (started, moved, ended)
        })
        .expect("toast drag state update");
    assert!(started && moved && ended);

    let (settle_from, drag_offset, dragging) = app
        .models()
        .read(&store, |st| {
            let toast = st
                .toasts_for_window(window)
                .iter()
                .find(|t| t.id == toast_id)
                .expect("toast present");
            (toast.settle_from, toast.drag_offset, toast.dragging)
        })
        .expect("toast state snapshot");
    assert!(settle_from.is_some());
    assert_eq!(drag_offset, Point::new(Px(0.0), Px(0.0)));
    assert!(!dragging);

    begin_frame(&mut app, window);
    let viewport_id = GlobalElementId(0xbeef);
    request_toast_layer_for_window(
        &mut app,
        window,
        ToastLayerRequest::new(viewport_id, store.clone())
            .position(ToastPosition::TopLeft)
            .margin(Px(0.0))
            .gap(Px(0.0)),
    );
    render(&mut ui, &mut app, &mut services, window, bounds);
    ui.layout_all(&mut app, &mut services, bounds, 1.0);

    let toast_count = app
        .models()
        .read(&store, |st| st.toasts_for_window(window).len())
        .unwrap_or_default();
    assert!(toast_count > 0);

    let toast_layer = app.with_global_mut(WindowOverlays::default, |overlays, _app| {
        overlays
            .toast_layers
            .get(&(window, viewport_id))
            .map(|l| l.layer)
            .expect("toast layer installed")
    });
    let layer_info = ui
        .debug_layers_in_paint_order()
        .into_iter()
        .find(|l| l.id == toast_layer)
        .expect("toast layer info");
    assert!(layer_info.visible);
    assert!(layer_info.hit_testable);

    // Find a point that's inside the visually translated toast (settle offset), but outside the
    // untransformed layout bounds. This should still hit the toast and install a drag start.
    let mut hit = false;
    for y in (8..=360).step_by(16) {
        for x in (300..=580).step_by(10) {
            let pos = Point::new(Px(x as f32), Px(y as f32));
            ui.dispatch_event(
                &mut app,
                &mut services,
                &fret_core::Event::Pointer(fret_core::PointerEvent::Down {
                    position: pos,
                    button: fret_core::MouseButton::Left,
                    modifiers: fret_core::Modifiers::default(),
                    click_count: 1,
                    pointer_id: PointerId(0),
                    pointer_type: fret_core::PointerType::Mouse,
                }),
            );

            hit = app
                .models_mut()
                .update(&store, |st| {
                    st.drag_move(
                        window,
                        toast_id,
                        Point::new(Px(x as f32 + 1.0), Px(y as f32)),
                    )
                    .is_some()
                })
                .expect("toast drag follow-up update");

            ui.dispatch_event(
                &mut app,
                &mut services,
                &fret_core::Event::Pointer(fret_core::PointerEvent::Up {
                    position: pos,
                    button: fret_core::MouseButton::Left,
                    modifiers: fret_core::Modifiers::default(),
                    is_click: true,
                    click_count: 1,
                    pointer_id: PointerId(0),
                    pointer_type: fret_core::PointerType::Mouse,
                }),
            );

            if hit {
                break;
            }
        }
        if hit {
            break;
        }
    }
    assert!(
        hit,
        "toast did not respond to pointer down; hit@10,10={:?} hit@350,10={:?} hit@500,200={:?}",
        ui.debug_hit_test(Point::new(Px(10.0), Px(10.0))),
        ui.debug_hit_test(Point::new(Px(350.0), Px(10.0))),
        ui.debug_hit_test(Point::new(Px(500.0), Px(200.0))),
    );
}

#[test]
fn modal_blocks_underlay_click_and_closes_on_escape() {
    let window = AppWindowId::default();
    let mut app = App::new();
    let mut ui: UiTree<App> = UiTree::new();
    ui.set_window(window);

    let open = app.models_mut().insert(true);
    let underlay_clicked = app.models_mut().insert(false);

    let mut services = FakeServices;
    let bounds = Rect::new(
        Point::new(Px(0.0), Px(0.0)),
        fret_core::Size::new(Px(300.0), Px(200.0)),
    );

    // Base layer contains a pressable that increments underlay_clicks.
    begin_frame(&mut app, window);
    let base = fret_ui::declarative::render_root(
        &mut ui,
        &mut app,
        &mut services,
        window,
        bounds,
        "base",
        |cx| {
            vec![cx.pressable(
                PressableProps {
                    layout: {
                        let mut layout = LayoutStyle::default();
                        layout.size.width = Length::Fill;
                        layout.size.height = Length::Fill;
                        layout
                    },
                    enabled: true,
                    focusable: true,
                    ..Default::default()
                },
                |cx, _st| {
                    cx.pressable_toggle_bool(&underlay_clicked);
                    vec![]
                },
            )]
        },
    );
    ui.set_root(base);

    // Install modal layer.
    begin_frame(&mut app, window);
    let modal_children =
        fret_ui::elements::with_element_cx(&mut app, window, bounds, "modal-child", |cx| {
            vec![cx.pressable(
                PressableProps {
                    layout: {
                        let mut layout = LayoutStyle::default();
                        layout.size.width = Length::Px(Px(80.0));
                        layout.size.height = Length::Px(Px(32.0));
                        layout
                    },
                    enabled: true,
                    focusable: true,
                    ..Default::default()
                },
                |_cx, _st| vec![],
            )]
        });
    request_modal_for_window(
        &mut app,
        window,
        ModalRequest {
            id: GlobalElementId(0xabc),
            root_name: modal_root_name(GlobalElementId(0xabc)),
            trigger: None,
            close_on_window_focus_lost: false,
            close_on_window_resize: false,
            open: open.clone(),
            present: true,
            initial_focus: None,
            on_open_auto_focus: None,
            on_close_auto_focus: None,
            on_dismiss_request: None,
            children: modal_children,
        },
    );

    render(&mut ui, &mut app, &mut services, window, bounds);
    ui.layout_all(&mut app, &mut services, bounds, 1.0);

    // Clicking underlay area should not reach base (modal barrier blocks underlay input).
    ui.dispatch_event(
        &mut app,
        &mut services,
        &fret_core::Event::Pointer(fret_core::PointerEvent::Down {
            position: Point::new(Px(10.0), Px(10.0)),
            button: fret_core::MouseButton::Left,
            modifiers: fret_core::Modifiers::default(),
            click_count: 1,
            pointer_id: PointerId(0),
            pointer_type: Default::default(),
        }),
    );
    ui.dispatch_event(
        &mut app,
        &mut services,
        &fret_core::Event::Pointer(fret_core::PointerEvent::Up {
            position: Point::new(Px(10.0), Px(10.0)),
            button: fret_core::MouseButton::Left,
            modifiers: fret_core::Modifiers::default(),
            is_click: true,
            click_count: 1,
            pointer_id: PointerId(0),
            pointer_type: Default::default(),
        }),
    );

    assert_eq!(app.models().get_copied(&underlay_clicked), Some(false));

    // Escape should close via DismissibleLayer.
    ui.dispatch_event(
        &mut app,
        &mut services,
        &fret_core::Event::KeyDown {
            key: fret_core::KeyCode::Escape,
            modifiers: fret_core::Modifiers::default(),
            repeat: false,
        },
    );
    assert_eq!(app.models().get_copied(&open), Some(false));
}

#[test]
fn modal_dismiss_handler_can_prevent_default_close() {
    let window = AppWindowId::default();
    let mut app = App::new();
    let mut ui: UiTree<App> = UiTree::new();
    ui.set_window(window);

    let open = app.models_mut().insert(true);
    let dismiss_called = app.models_mut().insert(false);

    let mut services = FakeServices;
    let bounds = Rect::new(
        Point::new(Px(0.0), Px(0.0)),
        fret_core::Size::new(Px(300.0), Px(200.0)),
    );

    begin_frame(&mut app, window);
    let base = fret_ui::declarative::render_root(
        &mut ui,
        &mut app,
        &mut services,
        window,
        bounds,
        "base",
        |_cx| Vec::new(),
    );
    ui.set_root(base);

    begin_frame(&mut app, window);
    let dismiss_called_for_handler = dismiss_called.clone();
    request_modal_for_window(
        &mut app,
        window,
        ModalRequest {
            id: GlobalElementId(0x111),
            root_name: modal_root_name(GlobalElementId(0x111)),
            trigger: None,
            close_on_window_focus_lost: false,
            close_on_window_resize: false,
            open: open.clone(),
            present: true,
            initial_focus: None,
            on_open_auto_focus: None,
            on_close_auto_focus: None,
            on_dismiss_request: Some(Arc::new(move |host, _cx, req| {
                let _ = host
                    .models_mut()
                    .update(&dismiss_called_for_handler, |v| *v = true);
                req.prevent_default();
            })),
            children: Vec::new(),
        },
    );

    render(&mut ui, &mut app, &mut services, window, bounds);
    ui.layout_all(&mut app, &mut services, bounds, 1.0);

    ui.dispatch_event(
        &mut app,
        &mut services,
        &fret_core::Event::KeyDown {
            key: fret_core::KeyCode::Escape,
            modifiers: fret_core::Modifiers::default(),
            repeat: false,
        },
    );

    assert_eq!(app.models().get_copied(&dismiss_called), Some(true));
    assert_eq!(app.models().get_copied(&open), Some(true));
}

#[test]
fn modal_can_remain_present_while_still_blocking_underlay_during_close_animation() {
    let window = AppWindowId::default();
    let mut app = App::new();
    let mut ui: UiTree<App> = UiTree::new();
    ui.set_window(window);

    let open = app.models_mut().insert(false);
    let underlay_clicked = app.models_mut().insert(false);
    let overlay_clicked = app.models_mut().insert(false);

    let mut services = FakeServices;
    let bounds = Rect::new(
        Point::new(Px(0.0), Px(0.0)),
        fret_core::Size::new(Px(300.0), Px(200.0)),
    );

    // Base layer contains a full-size pressable we expect NOT to receive the click.
    begin_frame(&mut app, window);
    let base = fret_ui::declarative::render_root(
        &mut ui,
        &mut app,
        &mut services,
        window,
        bounds,
        "base",
        |cx| {
            vec![cx.pressable(
                PressableProps {
                    layout: {
                        let mut layout = LayoutStyle::default();
                        layout.size.width = Length::Fill;
                        layout.size.height = Length::Fill;
                        layout
                    },
                    enabled: true,
                    focusable: true,
                    ..Default::default()
                },
                |cx, _st| {
                    cx.pressable_toggle_bool(&underlay_clicked);
                    vec![]
                },
            )]
        },
    );
    ui.set_root(base);

    // Install a modal layer that is still `present` but `open=false` (closing animation).
    begin_frame(&mut app, window);
    let modal_id = GlobalElementId(0xbeef);
    let modal_children =
        fret_ui::elements::with_element_cx(&mut app, window, bounds, "modal-child", |cx| {
            vec![cx.pressable(
                PressableProps {
                    layout: {
                        let mut layout = LayoutStyle::default();
                        layout.size.width = Length::Fill;
                        layout.size.height = Length::Fill;
                        layout
                    },
                    enabled: true,
                    focusable: false,
                    ..Default::default()
                },
                |cx, _st| {
                    cx.pressable_toggle_bool(&overlay_clicked);
                    vec![]
                },
            )]
        });
    request_modal_for_window(
        &mut app,
        window,
        ModalRequest {
            id: modal_id,
            root_name: modal_root_name(modal_id),
            trigger: None,
            close_on_window_focus_lost: false,
            close_on_window_resize: false,
            open,
            present: true,
            initial_focus: None,
            on_open_auto_focus: None,
            on_close_auto_focus: None,
            on_dismiss_request: None,
            children: modal_children,
        },
    );

    render(&mut ui, &mut app, &mut services, window, bounds);
    ui.layout_all(&mut app, &mut services, bounds, 1.0);

    ui.dispatch_event(
        &mut app,
        &mut services,
        &fret_core::Event::Pointer(fret_core::PointerEvent::Down {
            position: Point::new(Px(10.0), Px(10.0)),
            button: fret_core::MouseButton::Left,
            modifiers: fret_core::Modifiers::default(),
            click_count: 1,
            pointer_id: PointerId(0),
            pointer_type: Default::default(),
        }),
    );
    ui.dispatch_event(
        &mut app,
        &mut services,
        &fret_core::Event::Pointer(fret_core::PointerEvent::Up {
            position: Point::new(Px(10.0), Px(10.0)),
            button: fret_core::MouseButton::Left,
            modifiers: fret_core::Modifiers::default(),
            is_click: true,
            click_count: 1,
            pointer_id: PointerId(0),
            pointer_type: Default::default(),
        }),
    );

    assert_eq!(app.models().get_copied(&underlay_clicked), Some(false));
    assert_eq!(app.models().get_copied(&overlay_clicked), Some(true));
}

#[test]
fn modal_restores_focus_to_trigger_while_closing_but_still_present() {
    let window = AppWindowId::default();
    let mut app = App::new();
    let mut ui: UiTree<App> = UiTree::new();
    ui.set_window(window);

    let open = app.models_mut().insert(true);

    let mut services = FakeServices;
    let bounds = Rect::new(
        Point::new(Px(0.0), Px(0.0)),
        fret_core::Size::new(Px(300.0), Px(200.0)),
    );

    // First frame: render base to establish stable element mappings for the trigger.
    let trigger = render_base_with_trigger(
        &mut ui,
        &mut app,
        &mut services,
        window,
        bounds,
        open.clone(),
    );
    ui.layout_all(&mut app, &mut services, bounds, 1.0);

    let trigger_node =
        fret_ui::elements::node_for_element(&mut app, window, trigger).expect("trigger node");
    ui.set_focus(Some(trigger_node));

    let modal_id = GlobalElementId(0xabc);
    let mut modal_focusable: Option<GlobalElementId> = None;
    let modal_children =
        fret_ui::elements::with_element_cx(&mut app, window, bounds, "modal-child", |cx| {
            vec![cx.pressable_with_id(
                PressableProps {
                    layout: {
                        let mut layout = LayoutStyle::default();
                        layout.size.width = Length::Px(Px(80.0));
                        layout.size.height = Length::Px(Px(32.0));
                        layout
                    },
                    enabled: true,
                    focusable: true,
                    ..Default::default()
                },
                |_cx, _st, id| {
                    modal_focusable = Some(id);
                    Vec::new()
                },
            )]
        });
    let modal_focusable = modal_focusable.expect("modal focusable element id");

    // Second frame: install a modal overlay and ensure focus is inside the modal layer.
    begin_frame(&mut app, window);
    let _ = render_base_with_trigger(
        &mut ui,
        &mut app,
        &mut services,
        window,
        bounds,
        open.clone(),
    );

    request_modal_for_window(
        &mut app,
        window,
        ModalRequest {
            id: modal_id,
            root_name: modal_root_name(modal_id),
            trigger: Some(trigger),
            close_on_window_focus_lost: false,
            close_on_window_resize: false,
            open: open.clone(),
            present: true,
            initial_focus: Some(modal_focusable),
            on_open_auto_focus: None,
            on_close_auto_focus: None,
            on_dismiss_request: None,
            children: modal_children.clone(),
        },
    );

    render(&mut ui, &mut app, &mut services, window, bounds);
    ui.layout_all(&mut app, &mut services, bounds, 1.0);

    let modal_focus_node =
        fret_ui::elements::node_for_element(&mut app, window, modal_focusable).expect("modal node");
    assert_eq!(ui.focus(), Some(modal_focus_node));

    // Third frame: close (`open=false`) but keep `present=true` to simulate an exit transition.
    let _ = app.models_mut().update(&open, |v| *v = false);

    begin_frame(&mut app, window);
    let _ = render_base_with_trigger(
        &mut ui,
        &mut app,
        &mut services,
        window,
        bounds,
        open.clone(),
    );
    request_modal_for_window(
        &mut app,
        window,
        ModalRequest {
            id: modal_id,
            root_name: modal_root_name(modal_id),
            trigger: Some(trigger),
            close_on_window_focus_lost: false,
            close_on_window_resize: false,
            open: open.clone(),
            present: true,
            initial_focus: Some(modal_focusable),
            on_open_auto_focus: None,
            on_close_auto_focus: None,
            on_dismiss_request: None,
            children: modal_children,
        },
    );

    render(&mut ui, &mut app, &mut services, window, bounds);
    ui.layout_all(&mut app, &mut services, bounds, 1.0);

    let trigger_node =
        fret_ui::elements::node_for_element(&mut app, window, trigger).expect("trigger node");
    assert_eq!(ui.focus(), Some(trigger_node));
}

#[test]
fn modal_close_auto_focus_handler_can_prevent_default_restore() {
    let window = AppWindowId::default();
    let mut app = App::new();
    let mut ui: UiTree<App> = UiTree::new();
    ui.set_window(window);

    let open = app.models_mut().insert(true);

    let mut services = FakeServices;
    let bounds = Rect::new(
        Point::new(Px(0.0), Px(0.0)),
        fret_core::Size::new(Px(300.0), Px(200.0)),
    );

    let trigger = render_base_with_trigger(
        &mut ui,
        &mut app,
        &mut services,
        window,
        bounds,
        open.clone(),
    );
    ui.layout_all(&mut app, &mut services, bounds, 1.0);

    let modal_id = GlobalElementId(0xabc);
    let mut modal_focusable: Option<GlobalElementId> = None;
    let modal_children =
        fret_ui::elements::with_element_cx(&mut app, window, bounds, "modal-child", |cx| {
            vec![cx.pressable_with_id(
                PressableProps {
                    layout: {
                        let mut layout = LayoutStyle::default();
                        layout.size.width = Length::Px(Px(80.0));
                        layout.size.height = Length::Px(Px(32.0));
                        layout
                    },
                    enabled: true,
                    focusable: true,
                    ..Default::default()
                },
                |_cx, _st, id| {
                    modal_focusable = Some(id);
                    Vec::new()
                },
            )]
        });
    let modal_focusable = modal_focusable.expect("modal focusable element id");

    let on_close_auto_focus: fret_ui::action::OnCloseAutoFocus =
        Arc::new(|_host, _cx, req| req.prevent_default());

    // Second frame: mount modal and focus inside.
    begin_frame(&mut app, window);
    let _ = render_base_with_trigger(
        &mut ui,
        &mut app,
        &mut services,
        window,
        bounds,
        open.clone(),
    );
    request_modal_for_window(
        &mut app,
        window,
        ModalRequest {
            id: modal_id,
            root_name: modal_root_name(modal_id),
            trigger: Some(trigger),
            close_on_window_focus_lost: false,
            close_on_window_resize: false,
            open: open.clone(),
            present: true,
            initial_focus: Some(modal_focusable),
            on_open_auto_focus: None,
            on_close_auto_focus: Some(on_close_auto_focus.clone()),
            on_dismiss_request: None,
            children: modal_children.clone(),
        },
    );
    render(&mut ui, &mut app, &mut services, window, bounds);
    ui.layout_all(&mut app, &mut services, bounds, 1.0);

    let modal_focus_node =
        fret_ui::elements::node_for_element(&mut app, window, modal_focusable).expect("modal node");
    assert_eq!(ui.focus(), Some(modal_focus_node));

    // Third frame: close while still present; prevent restoring focus to the trigger.
    let _ = app.models_mut().update(&open, |v| *v = false);

    begin_frame(&mut app, window);
    let _ = render_base_with_trigger(
        &mut ui,
        &mut app,
        &mut services,
        window,
        bounds,
        open.clone(),
    );
    request_modal_for_window(
        &mut app,
        window,
        ModalRequest {
            id: modal_id,
            root_name: modal_root_name(modal_id),
            trigger: Some(trigger),
            close_on_window_focus_lost: false,
            close_on_window_resize: false,
            open: open.clone(),
            present: true,
            initial_focus: Some(modal_focusable),
            on_open_auto_focus: None,
            on_close_auto_focus: Some(on_close_auto_focus),
            on_dismiss_request: None,
            children: modal_children,
        },
    );
    render(&mut ui, &mut app, &mut services, window, bounds);
    ui.layout_all(&mut app, &mut services, bounds, 1.0);

    assert_eq!(ui.focus(), Some(modal_focus_node));
}

#[test]
fn modal_initial_focus_is_only_applied_on_opening_edge() {
    let window = AppWindowId::default();
    let mut app = App::new();
    let mut ui: UiTree<App> = UiTree::new();
    ui.set_window(window);

    let open = app.models_mut().insert(true);

    let mut services = FakeServices;
    let bounds = Rect::new(
        Point::new(Px(0.0), Px(0.0)),
        fret_core::Size::new(Px(300.0), Px(200.0)),
    );

    // First frame: render base to establish stable element mappings for the trigger.
    let trigger = render_base_with_trigger(
        &mut ui,
        &mut app,
        &mut services,
        window,
        bounds,
        open.clone(),
    );
    ui.layout_all(&mut app, &mut services, bounds, 1.0);

    let modal_id = GlobalElementId(0xabc);
    let mut a: Option<GlobalElementId> = None;
    let mut b: Option<GlobalElementId> = None;

    let modal_children =
        fret_ui::elements::with_element_cx(&mut app, window, bounds, "modal-child", |cx| {
            let props = PressableProps {
                layout: {
                    let mut layout = LayoutStyle::default();
                    layout.size.width = Length::Px(Px(80.0));
                    layout.size.height = Length::Px(Px(32.0));
                    layout
                },
                enabled: true,
                focusable: true,
                ..Default::default()
            };

            vec![
                cx.pressable_with_id(props.clone(), |_cx, _st, id| {
                    a = Some(id);
                    Vec::new()
                }),
                cx.pressable_with_id(props, |_cx, _st, id| {
                    b = Some(id);
                    Vec::new()
                }),
            ]
        });
    let a = a.expect("focusable a element id");
    let b = b.expect("focusable b element id");

    // Second frame: mount a modal overlay with two focusable elements, using `initial_focus=A`.
    begin_frame(&mut app, window);
    let _ = render_base_with_trigger(
        &mut ui,
        &mut app,
        &mut services,
        window,
        bounds,
        open.clone(),
    );

    request_modal_for_window(
        &mut app,
        window,
        ModalRequest {
            id: modal_id,
            root_name: modal_root_name(modal_id),
            trigger: Some(trigger),
            close_on_window_focus_lost: false,
            close_on_window_resize: false,
            open: open.clone(),
            present: true,
            initial_focus: Some(a),
            on_open_auto_focus: None,
            on_close_auto_focus: None,
            on_dismiss_request: None,
            children: modal_children.clone(),
        },
    );
    render(&mut ui, &mut app, &mut services, window, bounds);
    ui.layout_all(&mut app, &mut services, bounds, 1.0);

    let node_a = fret_ui::elements::node_for_element(&mut app, window, a).expect("node a");
    let node_b = fret_ui::elements::node_for_element(&mut app, window, b).expect("node b");
    assert_eq!(ui.focus(), Some(node_a));

    // Simulate user moving focus within the modal.
    ui.set_focus(Some(node_b));
    assert_eq!(ui.focus(), Some(node_b));

    // Third frame: keep the modal open and re-request it. Initial focus should not be re-applied.
    begin_frame(&mut app, window);
    let _ = render_base_with_trigger(
        &mut ui,
        &mut app,
        &mut services,
        window,
        bounds,
        open.clone(),
    );
    request_modal_for_window(
        &mut app,
        window,
        ModalRequest {
            id: modal_id,
            root_name: modal_root_name(modal_id),
            trigger: Some(trigger),
            close_on_window_focus_lost: false,
            close_on_window_resize: false,
            open: open.clone(),
            present: true,
            initial_focus: Some(a),
            on_open_auto_focus: None,
            on_close_auto_focus: None,
            on_dismiss_request: None,
            children: modal_children,
        },
    );
    render(&mut ui, &mut app, &mut services, window, bounds);
    ui.layout_all(&mut app, &mut services, bounds, 1.0);

    assert_eq!(ui.focus(), Some(node_b));
}

#[test]
fn modal_reasserts_focus_when_focus_leaves_modal_layer_while_open() {
    let window = AppWindowId::default();
    let mut app = App::new();
    let mut ui: UiTree<App> = UiTree::new();
    ui.set_window(window);

    let open = app.models_mut().insert(true);

    let mut services = FakeServices;
    let bounds = Rect::new(
        Point::new(Px(0.0), Px(0.0)),
        fret_core::Size::new(Px(300.0), Px(200.0)),
    );

    // First frame: render base to establish stable element mappings for the trigger.
    let trigger = render_base_with_trigger(
        &mut ui,
        &mut app,
        &mut services,
        window,
        bounds,
        open.clone(),
    );
    ui.layout_all(&mut app, &mut services, bounds, 1.0);

    let trigger_node =
        fret_ui::elements::node_for_element(&mut app, window, trigger).expect("trigger node");
    ui.set_focus(Some(trigger_node));

    let modal_id = GlobalElementId(0xabc);
    let mut modal_focusable: Option<GlobalElementId> = None;
    let modal_children =
        fret_ui::elements::with_element_cx(&mut app, window, bounds, "modal-child", |cx| {
            vec![cx.pressable_with_id(
                PressableProps {
                    layout: {
                        let mut layout = LayoutStyle::default();
                        layout.size.width = Length::Px(Px(80.0));
                        layout.size.height = Length::Px(Px(32.0));
                        layout
                    },
                    enabled: true,
                    focusable: true,
                    ..Default::default()
                },
                |_cx, _st, id| {
                    modal_focusable = Some(id);
                    Vec::new()
                },
            )]
        });
    let modal_focusable = modal_focusable.expect("modal focusable element id");

    // Second frame: install a modal overlay. Focus should move inside the modal layer.
    begin_frame(&mut app, window);
    let _ = render_base_with_trigger(
        &mut ui,
        &mut app,
        &mut services,
        window,
        bounds,
        open.clone(),
    );
    request_modal_for_window(
        &mut app,
        window,
        ModalRequest {
            id: modal_id,
            root_name: modal_root_name(modal_id),
            trigger: Some(trigger),
            close_on_window_focus_lost: false,
            close_on_window_resize: false,
            open: open.clone(),
            present: true,
            initial_focus: Some(modal_focusable),
            on_open_auto_focus: None,
            on_close_auto_focus: None,
            on_dismiss_request: None,
            children: modal_children.clone(),
        },
    );
    render(&mut ui, &mut app, &mut services, window, bounds);
    ui.layout_all(&mut app, &mut services, bounds, 1.0);

    let modal_focus_node =
        fret_ui::elements::node_for_element(&mut app, window, modal_focusable).expect("modal node");
    assert_eq!(ui.focus(), Some(modal_focus_node));

    // Simulate a bug where focus is programmatically moved back to the underlay while the modal is
    // still open. The overlay policy should reassert focus containment on the next frame.
    ui.set_focus(Some(trigger_node));

    begin_frame(&mut app, window);
    let _ = render_base_with_trigger(
        &mut ui,
        &mut app,
        &mut services,
        window,
        bounds,
        open.clone(),
    );
    request_modal_for_window(
        &mut app,
        window,
        ModalRequest {
            id: modal_id,
            root_name: modal_root_name(modal_id),
            trigger: Some(trigger),
            close_on_window_focus_lost: false,
            close_on_window_resize: false,
            open: open.clone(),
            present: true,
            initial_focus: Some(modal_focusable),
            on_open_auto_focus: None,
            on_close_auto_focus: None,
            on_dismiss_request: None,
            children: modal_children,
        },
    );
    render(&mut ui, &mut app, &mut services, window, bounds);
    ui.layout_all(&mut app, &mut services, bounds, 1.0);

    assert_eq!(ui.focus(), Some(modal_focus_node));
}

#[test]
fn non_modal_overlay_can_remain_present_while_pointer_transparent_during_close_animation() {
    let window = AppWindowId::default();
    let mut app = App::new();
    let mut ui: UiTree<App> = UiTree::new();
    ui.set_window(window);

    let open = app.models_mut().insert(false);
    let underlay_clicked = app.models_mut().insert(false);
    let overlay_clicked = app.models_mut().insert(false);

    let mut services = FakeServices;
    let bounds = Rect::new(
        Point::new(Px(0.0), Px(0.0)),
        fret_core::Size::new(Px(300.0), Px(200.0)),
    );

    // Base layer contains a full-size pressable we expect to receive the click.
    begin_frame(&mut app, window);
    let base = fret_ui::declarative::render_root(
        &mut ui,
        &mut app,
        &mut services,
        window,
        bounds,
        "base",
        |cx| {
            vec![cx.pressable(
                PressableProps {
                    layout: {
                        let mut layout = LayoutStyle::default();
                        layout.size.width = Length::Fill;
                        layout.size.height = Length::Fill;
                        layout
                    },
                    enabled: true,
                    focusable: true,
                    ..Default::default()
                },
                |cx, _st| {
                    cx.pressable_toggle_bool(&underlay_clicked);
                    vec![]
                },
            )]
        },
    );
    ui.set_root(base);

    // Install a non-modal layer that is still `present` but `open=false` (closing animation).
    begin_frame(&mut app, window);
    let trigger = GlobalElementId(0xdead);
    let overlay_children =
        fret_ui::elements::with_element_cx(&mut app, window, bounds, "popover-child", |cx| {
            vec![cx.pressable(
                PressableProps {
                    layout: {
                        let mut layout = LayoutStyle::default();
                        layout.size.width = Length::Fill;
                        layout.size.height = Length::Fill;
                        layout
                    },
                    enabled: true,
                    focusable: false,
                    ..Default::default()
                },
                |cx, _st| {
                    cx.pressable_toggle_bool(&overlay_clicked);
                    vec![]
                },
            )]
        });

    request_dismissible_popover_for_window(
        &mut app,
        window,
        DismissiblePopoverRequest {
            id: trigger,
            root_name: popover_root_name(trigger),
            trigger,
            dismissable_branches: Vec::new(),
            consume_outside_pointer_events: false,
            disable_outside_pointer_events: false,
            close_on_window_focus_lost: false,
            close_on_window_resize: false,
            open,
            present: true,
            initial_focus: None,
            on_open_auto_focus: None,
            on_close_auto_focus: None,
            on_dismiss_request: None,
            on_pointer_move: None,
            children: overlay_children,
        },
    );

    render(&mut ui, &mut app, &mut services, window, bounds);
    ui.layout_all(&mut app, &mut services, bounds, 1.0);

    ui.dispatch_event(
        &mut app,
        &mut services,
        &fret_core::Event::Pointer(fret_core::PointerEvent::Down {
            position: Point::new(Px(10.0), Px(10.0)),
            button: fret_core::MouseButton::Left,
            modifiers: fret_core::Modifiers::default(),
            click_count: 1,
            pointer_id: PointerId(0),
            pointer_type: Default::default(),
        }),
    );
    ui.dispatch_event(
        &mut app,
        &mut services,
        &fret_core::Event::Pointer(fret_core::PointerEvent::Up {
            position: Point::new(Px(10.0), Px(10.0)),
            button: fret_core::MouseButton::Left,
            modifiers: fret_core::Modifiers::default(),
            is_click: true,
            click_count: 1,
            pointer_id: PointerId(0),
            pointer_type: Default::default(),
        }),
    );

    assert_eq!(app.models().get_copied(&underlay_clicked), Some(true));
    assert_eq!(app.models().get_copied(&overlay_clicked), Some(false));
}

#[test]
fn non_modal_overlay_disable_outside_pointer_events_does_not_block_underlay_while_closing() {
    let window = AppWindowId::default();
    let mut app = App::new();
    let mut ui: UiTree<App> = UiTree::new();
    ui.set_window(window);

    let open = app.models_mut().insert(false);
    let underlay_clicked = app.models_mut().insert(false);
    let overlay_clicked = app.models_mut().insert(false);

    let mut services = FakeServices;
    let bounds = Rect::new(
        Point::new(Px(0.0), Px(0.0)),
        fret_core::Size::new(Px(300.0), Px(200.0)),
    );

    begin_frame(&mut app, window);
    let base = fret_ui::declarative::render_root(
        &mut ui,
        &mut app,
        &mut services,
        window,
        bounds,
        "base",
        |cx| {
            vec![cx.pressable(
                PressableProps {
                    layout: {
                        let mut layout = LayoutStyle::default();
                        layout.size.width = Length::Fill;
                        layout.size.height = Length::Fill;
                        layout
                    },
                    enabled: true,
                    focusable: true,
                    ..Default::default()
                },
                |cx, _st| {
                    cx.pressable_toggle_bool(&underlay_clicked);
                    vec![]
                },
            )]
        },
    );
    ui.set_root(base);

    begin_frame(&mut app, window);
    let trigger = GlobalElementId(0xdead);
    let overlay_children =
        fret_ui::elements::with_element_cx(&mut app, window, bounds, "popover-child", |cx| {
            vec![cx.pressable(
                PressableProps {
                    layout: {
                        let mut layout = LayoutStyle::default();
                        layout.size.width = Length::Fill;
                        layout.size.height = Length::Fill;
                        layout
                    },
                    enabled: true,
                    focusable: false,
                    ..Default::default()
                },
                |cx, _st| {
                    cx.pressable_toggle_bool(&overlay_clicked);
                    vec![]
                },
            )]
        });

    request_dismissible_popover_for_window(
        &mut app,
        window,
        DismissiblePopoverRequest {
            id: trigger,
            root_name: popover_root_name(trigger),
            trigger,
            dismissable_branches: Vec::new(),
            consume_outside_pointer_events: false,
            disable_outside_pointer_events: true,
            close_on_window_focus_lost: false,
            close_on_window_resize: false,
            open,
            present: true,
            initial_focus: None,
            on_open_auto_focus: None,
            on_close_auto_focus: None,
            on_dismiss_request: None,
            on_pointer_move: None,
            children: overlay_children,
        },
    );

    render(&mut ui, &mut app, &mut services, window, bounds);
    ui.layout_all(&mut app, &mut services, bounds, 1.0);

    ui.dispatch_event(
        &mut app,
        &mut services,
        &fret_core::Event::Pointer(fret_core::PointerEvent::Down {
            position: Point::new(Px(10.0), Px(10.0)),
            button: fret_core::MouseButton::Left,
            modifiers: fret_core::Modifiers::default(),
            click_count: 1,
            pointer_id: PointerId(0),
            pointer_type: Default::default(),
        }),
    );
    ui.dispatch_event(
        &mut app,
        &mut services,
        &fret_core::Event::Pointer(fret_core::PointerEvent::Up {
            position: Point::new(Px(10.0), Px(10.0)),
            button: fret_core::MouseButton::Left,
            modifiers: fret_core::Modifiers::default(),
            is_click: true,
            click_count: 1,
            pointer_id: PointerId(0),
            pointer_type: Default::default(),
        }),
    );

    assert_eq!(app.models().get_copied(&underlay_clicked), Some(true));
    assert_eq!(app.models().get_copied(&overlay_clicked), Some(false));
}

#[test]
fn non_modal_overlay_does_not_request_outside_press_observer_while_closing() {
    let window = AppWindowId::default();
    let mut app = App::new();
    let mut ui: UiTree<App> = UiTree::new();
    ui.set_window(window);

    let open = app.models_mut().insert(false);

    let mut services = FakeServices;
    let bounds = Rect::new(
        Point::new(Px(0.0), Px(0.0)),
        fret_core::Size::new(Px(300.0), Px(200.0)),
    );

    // Base root (required so the window exists and rendering can proceed).
    begin_frame(&mut app, window);
    let base = fret_ui::declarative::render_root(
        &mut ui,
        &mut app,
        &mut services,
        window,
        bounds,
        "base",
        |_| Vec::new(),
    );
    ui.set_root(base);

    // Install a non-modal layer that is still `present` but `open=false` (closing animation).
    begin_frame(&mut app, window);
    let trigger = GlobalElementId(0xdead);
    request_dismissible_popover_for_window(
        &mut app,
        window,
        DismissiblePopoverRequest {
            id: trigger,
            root_name: popover_root_name(trigger),
            trigger,
            dismissable_branches: Vec::new(),
            consume_outside_pointer_events: false,
            disable_outside_pointer_events: false,
            close_on_window_focus_lost: false,
            close_on_window_resize: false,
            open,
            present: true,
            initial_focus: None,
            on_open_auto_focus: None,
            on_close_auto_focus: None,
            on_dismiss_request: None,
            on_pointer_move: None,
            children: Vec::new(),
        },
    );

    render(&mut ui, &mut app, &mut services, window, bounds);
    ui.layout_all(&mut app, &mut services, bounds, 1.0);

    let layer = app
        .with_global_mut(WindowOverlays::default, |overlays, _app| {
            overlays.popovers.get(&(window, trigger)).map(|p| p.layer)
        })
        .expect("popover layer");

    let info = ui
        .debug_layers_in_paint_order()
        .into_iter()
        .find(|l| l.id == layer)
        .expect("popover debug layer info");

    assert!(info.visible);
    assert!(!info.blocks_underlay_input);
    assert!(!info.hit_testable);
    assert!(!info.wants_pointer_down_outside_events);
}

#[test]
fn tooltip_does_not_request_observers_by_default() {
    let window = AppWindowId::default();
    let mut app = App::new();
    let mut ui: UiTree<App> = UiTree::new();
    ui.set_window(window);

    let mut services = FakeServices;
    let bounds = Rect::new(
        Point::new(Px(0.0), Px(0.0)),
        fret_core::Size::new(Px(300.0), Px(200.0)),
    );

    // Base root (required so the window exists and rendering can proceed).
    begin_frame(&mut app, window);
    let base = fret_ui::declarative::render_root(
        &mut ui,
        &mut app,
        &mut services,
        window,
        bounds,
        "base",
        |_| Vec::new(),
    );
    ui.set_root(base);

    // Tooltips are click-through and should not install outside-press / pointer-move observers
    // unless the request explicitly opts into them.
    begin_frame(&mut app, window);
    let id = GlobalElementId(0xdead);
    request_tooltip_for_window(
        &mut app,
        window,
        TooltipRequest {
            id,
            root_name: tooltip_root_name(id),
            interactive: true,
            trigger: Some(id),
            on_dismiss_request: None,
            on_pointer_move: None,
            children: Vec::new(),
        },
    );

    render(&mut ui, &mut app, &mut services, window, bounds);
    ui.layout_all(&mut app, &mut services, bounds, 1.0);

    let layer = app
        .with_global_mut(WindowOverlays::default, |overlays, _app| {
            overlays.tooltips.get(&(window, id)).map(|t| t.layer)
        })
        .expect("tooltip layer");

    let info = ui
        .debug_layers_in_paint_order()
        .into_iter()
        .find(|l| l.id == layer)
        .expect("tooltip debug layer info");

    assert!(info.visible);
    assert!(!info.blocks_underlay_input);
    assert!(!info.hit_testable);
    assert!(!info.wants_pointer_down_outside_events);
    assert!(!info.wants_pointer_move_events);
}

#[test]
fn tooltip_does_not_request_observers_while_closing() {
    let window = AppWindowId::default();
    let mut app = App::new();
    let mut ui: UiTree<App> = UiTree::new();
    ui.set_window(window);

    let mut services = FakeServices;
    let bounds = Rect::new(
        Point::new(Px(0.0), Px(0.0)),
        fret_core::Size::new(Px(300.0), Px(200.0)),
    );

    // Base root (required so the window exists and rendering can proceed).
    begin_frame(&mut app, window);
    let base = fret_ui::declarative::render_root(
        &mut ui,
        &mut app,
        &mut services,
        window,
        bounds,
        "base",
        |_| Vec::new(),
    );
    ui.set_root(base);

    // Install a tooltip layer that is still present but non-interactive (closing animation).
    begin_frame(&mut app, window);
    let id = GlobalElementId(0xdead);
    let handler: fret_ui::action::OnDismissRequest = Arc::new(|_host, _cx, _req| {});
    let on_pointer_move: fret_ui::action::OnDismissiblePointerMove =
        Arc::new(|_host, _cx, _move| false);
    request_tooltip_for_window(
        &mut app,
        window,
        TooltipRequest {
            id,
            root_name: tooltip_root_name(id),
            interactive: false,
            trigger: Some(id),
            on_dismiss_request: Some(handler),
            on_pointer_move: Some(on_pointer_move),
            children: Vec::new(),
        },
    );

    render(&mut ui, &mut app, &mut services, window, bounds);
    ui.layout_all(&mut app, &mut services, bounds, 1.0);

    let layer = app
        .with_global_mut(WindowOverlays::default, |overlays, _app| {
            overlays.tooltips.get(&(window, id)).map(|t| t.layer)
        })
        .expect("tooltip layer");

    let info = ui
        .debug_layers_in_paint_order()
        .into_iter()
        .find(|l| l.id == layer)
        .expect("tooltip debug layer info");

    assert!(info.visible);
    assert!(!info.blocks_underlay_input);
    assert!(!info.hit_testable);
    assert!(!info.wants_pointer_down_outside_events);
    assert!(!info.wants_pointer_move_events);
}

#[test]
fn hover_overlay_is_click_through_while_closing() {
    let window = AppWindowId::default();
    let mut app = App::new();
    let mut ui: UiTree<App> = UiTree::new();
    ui.set_window(window);

    let mut services = FakeServices;
    let bounds = Rect::new(
        Point::new(Px(0.0), Px(0.0)),
        fret_core::Size::new(Px(300.0), Px(200.0)),
    );

    // Base root (required so the window exists and rendering can proceed).
    begin_frame(&mut app, window);
    let base = fret_ui::declarative::render_root(
        &mut ui,
        &mut app,
        &mut services,
        window,
        bounds,
        "base",
        |_| Vec::new(),
    );
    ui.set_root(base);

    // Install a hover overlay that is still present but non-interactive (closing animation).
    begin_frame(&mut app, window);
    let id = GlobalElementId(0xdead);
    request_hover_overlay_for_window(
        &mut app,
        window,
        HoverOverlayRequest {
            id,
            root_name: hover_overlay_root_name(id),
            interactive: false,
            trigger: id,
            children: Vec::new(),
        },
    );

    render(&mut ui, &mut app, &mut services, window, bounds);
    ui.layout_all(&mut app, &mut services, bounds, 1.0);

    let layer = app
        .with_global_mut(WindowOverlays::default, |overlays, _app| {
            overlays.hover_overlays.get(&(window, id)).map(|h| h.layer)
        })
        .expect("hover overlay layer");

    let info = ui
        .debug_layers_in_paint_order()
        .into_iter()
        .find(|l| l.id == layer)
        .expect("hover overlay debug layer info");

    assert!(info.visible);
    assert!(!info.blocks_underlay_input);
    assert!(!info.hit_testable);
    assert!(!info.wants_pointer_down_outside_events);
    assert!(!info.wants_pointer_move_events);

    let arbitration = crate::OverlayController::arbitration_snapshot(&ui);
    assert_eq!(
        arbitration.pointer_occlusion,
        fret_ui::tree::PointerOcclusion::None,
        "expected hover overlay close transition to be click-through"
    );
}

#[test]
fn non_modal_overlay_restores_focus_when_focus_is_missing_on_unmount() {
    let window = AppWindowId::default();
    let mut app = App::new();
    let mut ui: UiTree<App> = UiTree::new();
    ui.set_window(window);

    let open = app.models_mut().insert(true);

    let mut services = FakeServices;
    let bounds = Rect::new(
        Point::new(Px(0.0), Px(0.0)),
        fret_core::Size::new(Px(300.0), Px(200.0)),
    );

    // First frame: render base to establish stable bounds for the trigger element.
    let trigger = render_base_with_trigger(
        &mut ui,
        &mut app,
        &mut services,
        window,
        bounds,
        open.clone(),
    );
    ui.layout_all(&mut app, &mut services, bounds, 1.0);

    // Second frame: request and render a dismissible popover (open=true).
    begin_frame(&mut app, window);
    let _ = render_base_with_trigger(
        &mut ui,
        &mut app,
        &mut services,
        window,
        bounds,
        open.clone(),
    );
    request_dismissible_popover_for_window(
        &mut app,
        window,
        DismissiblePopoverRequest {
            id: trigger,
            root_name: popover_root_name(trigger),
            trigger,
            dismissable_branches: Vec::new(),
            consume_outside_pointer_events: false,
            disable_outside_pointer_events: false,
            close_on_window_focus_lost: false,
            close_on_window_resize: false,
            open: open.clone(),
            present: true,
            initial_focus: None,
            on_open_auto_focus: None,
            on_close_auto_focus: None,
            on_dismiss_request: None,
            on_pointer_move: None,
            children: Vec::new(),
        },
    );
    render(&mut ui, &mut app, &mut services, window, bounds);
    ui.layout_all(&mut app, &mut services, bounds, 1.0);

    ui.set_focus(None);

    // Close the overlay so the cached request is no longer synthesized.
    //
    // With cached request declarations (for view caching), an open overlay can be synthesized even
    // if the subtree that emits requests is skipped for a frame. To model a true unmount, ensure
    // `open=false` before the frame where the overlay is no longer requested.
    let _ = app.models_mut().update(&open, |v| *v = false);

    // Third frame: do not request the popover (unmounted), and expect focus to be restored to
    // the trigger since focus is missing.
    begin_frame(&mut app, window);
    let _ = render_base_with_trigger(
        &mut ui,
        &mut app,
        &mut services,
        window,
        bounds,
        open.clone(),
    );
    render(&mut ui, &mut app, &mut services, window, bounds);
    ui.layout_all(&mut app, &mut services, bounds, 1.0);

    let trigger_node =
        fret_ui::elements::node_for_element(&mut app, window, trigger).expect("trigger node");
    assert_eq!(ui.focus(), Some(trigger_node));
}

#[test]
fn non_modal_overlay_does_not_restore_focus_when_focus_moves_to_underlay_on_unmount() {
    let window = AppWindowId::default();
    let mut app = App::new();
    let mut ui: UiTree<App> = UiTree::new();
    ui.set_window(window);

    let open = app.models_mut().insert(false);
    let underlay_clicked = app.models_mut().insert(false);

    let mut services = FakeServices;
    let bounds = Rect::new(
        Point::new(Px(0.0), Px(0.0)),
        fret_core::Size::new(Px(300.0), Px(200.0)),
    );

    // First frame: render base to establish stable bounds for the trigger element.
    let (trigger, underlay) = render_base_with_trigger_and_underlay(
        &mut ui,
        &mut app,
        &mut services,
        window,
        bounds,
        open.clone(),
        underlay_clicked.clone(),
    );
    ui.layout_all(&mut app, &mut services, bounds, 1.0);

    // Open via click on trigger.
    ui.dispatch_event(
        &mut app,
        &mut services,
        &fret_core::Event::Pointer(fret_core::PointerEvent::Down {
            position: Point::new(Px(10.0), Px(10.0)),
            button: fret_core::MouseButton::Left,
            modifiers: fret_core::Modifiers::default(),
            click_count: 1,
            pointer_id: PointerId(0),
            pointer_type: Default::default(),
        }),
    );
    ui.dispatch_event(
        &mut app,
        &mut services,
        &fret_core::Event::Pointer(fret_core::PointerEvent::Up {
            position: Point::new(Px(10.0), Px(10.0)),
            button: fret_core::MouseButton::Left,
            modifiers: fret_core::Modifiers::default(),
            is_click: true,
            click_count: 1,
            pointer_id: PointerId(0),
            pointer_type: Default::default(),
        }),
    );
    assert_eq!(app.models().get_copied(&open), Some(true));

    // Second frame: request and render a dismissible popover.
    begin_frame(&mut app, window);
    let _ = render_base_with_trigger_and_underlay(
        &mut ui,
        &mut app,
        &mut services,
        window,
        bounds,
        open.clone(),
        underlay_clicked.clone(),
    );

    let overlay_children =
        fret_ui::elements::with_element_cx(&mut app, window, bounds, "overlay-children", |cx| {
            vec![cx.container(
                ContainerProps {
                    layout: {
                        LayoutStyle {
                            position: PositionStyle::Absolute,
                            inset: InsetStyle {
                                left: Some(Px(0.0)),
                                top: Some(Px(40.0)),
                                ..Default::default()
                            },
                            size: SizeStyle {
                                width: Length::Px(Px(200.0)),
                                height: Length::Px(Px(40.0)),
                                ..Default::default()
                            },
                            ..Default::default()
                        }
                    },
                    ..Default::default()
                },
                |_| Vec::new(),
            )]
        });
    request_dismissible_popover_for_window(
        &mut app,
        window,
        DismissiblePopoverRequest {
            id: trigger,
            root_name: popover_root_name(trigger),
            trigger,
            dismissable_branches: Vec::new(),
            consume_outside_pointer_events: false,
            disable_outside_pointer_events: false,
            close_on_window_focus_lost: false,
            close_on_window_resize: false,
            open: open.clone(),
            present: true,
            initial_focus: None,
            on_open_auto_focus: None,
            on_close_auto_focus: None,
            on_dismiss_request: None,
            on_pointer_move: None,
            children: overlay_children,
        },
    );

    render(&mut ui, &mut app, &mut services, window, bounds);
    ui.layout_all(&mut app, &mut services, bounds, 1.0);

    // Click underlay while popover is open: outside press is click-through, so focus should move
    // to the underlay target (and the popover should close).
    ui.dispatch_event(
        &mut app,
        &mut services,
        &fret_core::Event::Pointer(fret_core::PointerEvent::Down {
            position: Point::new(Px(10.0), Px(130.0)),
            button: fret_core::MouseButton::Left,
            modifiers: fret_core::Modifiers::default(),
            click_count: 1,
            pointer_id: PointerId(0),
            pointer_type: Default::default(),
        }),
    );
    ui.dispatch_event(
        &mut app,
        &mut services,
        &fret_core::Event::Pointer(fret_core::PointerEvent::Up {
            position: Point::new(Px(10.0), Px(130.0)),
            button: fret_core::MouseButton::Left,
            modifiers: fret_core::Modifiers::default(),
            is_click: true,
            click_count: 1,
            pointer_id: PointerId(0),
            pointer_type: Default::default(),
        }),
    );

    assert_eq!(app.models().get_copied(&open), Some(false));
    assert_eq!(app.models().get_copied(&underlay_clicked), Some(true));

    let underlay_node =
        fret_ui::elements::node_for_element(&mut app, window, underlay).expect("underlay node");
    assert_eq!(ui.focus(), Some(underlay_node));

    // Third frame: unmount the popover. Focus restoration must not override the new underlay focus.
    begin_frame(&mut app, window);
    let _ = render_base_with_trigger_and_underlay(
        &mut ui,
        &mut app,
        &mut services,
        window,
        bounds,
        open.clone(),
        underlay_clicked.clone(),
    );

    render(&mut ui, &mut app, &mut services, window, bounds);
    ui.layout_all(&mut app, &mut services, bounds, 1.0);

    assert_eq!(ui.focus(), Some(underlay_node));
}

#[test]
fn non_modal_overlay_can_consume_outside_press_to_block_underlay_activation() {
    let window = AppWindowId::default();
    let mut app = App::new();
    let mut ui: UiTree<App> = UiTree::new();
    ui.set_window(window);

    let open = app.models_mut().insert(false);
    let underlay_clicked = app.models_mut().insert(false);

    let mut services = FakeServices;
    let bounds = Rect::new(
        Point::new(Px(0.0), Px(0.0)),
        fret_core::Size::new(Px(300.0), Px(200.0)),
    );

    // First frame: render base to establish stable bounds for the trigger element.
    let (trigger, _underlay) = render_base_with_trigger_and_underlay(
        &mut ui,
        &mut app,
        &mut services,
        window,
        bounds,
        open.clone(),
        underlay_clicked.clone(),
    );
    ui.layout_all(&mut app, &mut services, bounds, 1.0);

    // Open via click on trigger.
    ui.dispatch_event(
        &mut app,
        &mut services,
        &fret_core::Event::Pointer(fret_core::PointerEvent::Down {
            position: Point::new(Px(10.0), Px(10.0)),
            button: fret_core::MouseButton::Left,
            modifiers: fret_core::Modifiers::default(),
            click_count: 1,
            pointer_id: PointerId(0),
            pointer_type: fret_core::PointerType::Mouse,
        }),
    );
    ui.dispatch_event(
        &mut app,
        &mut services,
        &fret_core::Event::Pointer(fret_core::PointerEvent::Up {
            position: Point::new(Px(10.0), Px(10.0)),
            button: fret_core::MouseButton::Left,
            modifiers: fret_core::Modifiers::default(),
            is_click: true,
            click_count: 1,
            pointer_id: PointerId(0),
            pointer_type: fret_core::PointerType::Mouse,
        }),
    );
    assert_eq!(app.models().get_copied(&open), Some(true));

    // Second frame: request and render a dismissible popover that consumes outside presses.
    begin_frame(&mut app, window);
    let (_trigger2, _underlay2) = render_base_with_trigger_and_underlay(
        &mut ui,
        &mut app,
        &mut services,
        window,
        bounds,
        open.clone(),
        underlay_clicked.clone(),
    );

    let overlay_children =
        fret_ui::elements::with_element_cx(&mut app, window, bounds, "overlay-children", |cx| {
            vec![cx.container(
                ContainerProps {
                    layout: {
                        LayoutStyle {
                            position: PositionStyle::Absolute,
                            inset: InsetStyle {
                                left: Some(Px(0.0)),
                                top: Some(Px(40.0)),
                                ..Default::default()
                            },
                            size: SizeStyle {
                                width: Length::Px(Px(200.0)),
                                height: Length::Px(Px(40.0)),
                                ..Default::default()
                            },
                            ..Default::default()
                        }
                    },
                    ..Default::default()
                },
                |_| Vec::new(),
            )]
        });
    request_dismissible_popover_for_window(
        &mut app,
        window,
        DismissiblePopoverRequest {
            id: trigger,
            root_name: popover_root_name(trigger),
            trigger,
            dismissable_branches: Vec::new(),
            consume_outside_pointer_events: true,
            disable_outside_pointer_events: false,
            close_on_window_focus_lost: false,
            close_on_window_resize: false,
            open: open.clone(),
            present: true,
            initial_focus: None,
            on_open_auto_focus: None,
            on_close_auto_focus: None,
            on_dismiss_request: None,
            on_pointer_move: None,
            children: overlay_children,
        },
    );

    render(&mut ui, &mut app, &mut services, window, bounds);
    ui.layout_all(&mut app, &mut services, bounds, 1.0);

    // Click underlay while popover is open: outside press closes, but must not activate the underlay.
    ui.dispatch_event(
        &mut app,
        &mut services,
        &fret_core::Event::Pointer(fret_core::PointerEvent::Down {
            position: Point::new(Px(10.0), Px(130.0)),
            button: fret_core::MouseButton::Left,
            modifiers: fret_core::Modifiers::default(),
            click_count: 1,
            pointer_id: PointerId(0),
            pointer_type: fret_core::PointerType::Mouse,
        }),
    );
    ui.dispatch_event(
        &mut app,
        &mut services,
        &fret_core::Event::Pointer(fret_core::PointerEvent::Up {
            position: Point::new(Px(10.0), Px(130.0)),
            button: fret_core::MouseButton::Left,
            modifiers: fret_core::Modifiers::default(),
            is_click: true,
            click_count: 1,
            pointer_id: PointerId(0),
            pointer_type: fret_core::PointerType::Mouse,
        }),
    );

    assert_eq!(app.models().get_copied(&open), Some(false));
    assert_eq!(app.models().get_copied(&underlay_clicked), Some(false));

    // Third frame: unmount the popover. Focus should restore to the trigger (since focus stayed inside the overlay).
    begin_frame(&mut app, window);
    let _ = render_base_with_trigger_and_underlay(
        &mut ui,
        &mut app,
        &mut services,
        window,
        bounds,
        open.clone(),
        underlay_clicked.clone(),
    );

    render(&mut ui, &mut app, &mut services, window, bounds);
    ui.layout_all(&mut app, &mut services, bounds, 1.0);

    let trigger_node =
        fret_ui::elements::node_for_element(&mut app, window, trigger).expect("trigger node");
    assert_eq!(ui.focus(), Some(trigger_node));
}

#[test]
fn non_modal_overlay_dismiss_handler_can_prevent_default_close() {
    let window = AppWindowId::default();
    let mut app = App::new();
    let mut ui: UiTree<App> = UiTree::new();
    ui.set_window(window);

    let open = app.models_mut().insert(true);
    let underlay_clicked = app.models_mut().insert(false);
    let dismiss_called = app.models_mut().insert(false);

    let mut services = FakeServices;
    let bounds = Rect::new(
        Point::new(Px(0.0), Px(0.0)),
        fret_core::Size::new(Px(300.0), Px(200.0)),
    );

    begin_frame(&mut app, window);
    let (trigger, _underlay) = render_base_with_trigger_and_underlay(
        &mut ui,
        &mut app,
        &mut services,
        window,
        bounds,
        open.clone(),
        underlay_clicked.clone(),
    );
    ui.layout_all(&mut app, &mut services, bounds, 1.0);

    begin_frame(&mut app, window);
    let dismiss_called_for_handler = dismiss_called.clone();
    request_dismissible_popover_for_window(
        &mut app,
        window,
        DismissiblePopoverRequest {
            id: trigger,
            root_name: popover_root_name(trigger),
            trigger,
            dismissable_branches: Vec::new(),
            consume_outside_pointer_events: true,
            disable_outside_pointer_events: false,
            close_on_window_focus_lost: false,
            close_on_window_resize: false,
            open: open.clone(),
            present: true,
            initial_focus: None,
            on_open_auto_focus: None,
            on_close_auto_focus: None,
            on_dismiss_request: Some(Arc::new(move |host, _cx, req| {
                let _ = host
                    .models_mut()
                    .update(&dismiss_called_for_handler, |v| *v = true);
                req.prevent_default();
            })),
            on_pointer_move: None,
            children: Vec::new(),
        },
    );

    render(&mut ui, &mut app, &mut services, window, bounds);
    ui.layout_all(&mut app, &mut services, bounds, 1.0);

    // Outside press should invoke the dismiss handler, but not close nor activate the underlay.
    ui.dispatch_event(
        &mut app,
        &mut services,
        &fret_core::Event::Pointer(fret_core::PointerEvent::Down {
            position: Point::new(Px(10.0), Px(130.0)),
            button: fret_core::MouseButton::Left,
            modifiers: fret_core::Modifiers::default(),
            click_count: 1,
            pointer_id: PointerId(0),
            pointer_type: fret_core::PointerType::Mouse,
        }),
    );
    ui.dispatch_event(
        &mut app,
        &mut services,
        &fret_core::Event::Pointer(fret_core::PointerEvent::Up {
            position: Point::new(Px(10.0), Px(130.0)),
            button: fret_core::MouseButton::Left,
            modifiers: fret_core::Modifiers::default(),
            is_click: true,
            click_count: 1,
            pointer_id: PointerId(0),
            pointer_type: fret_core::PointerType::Mouse,
        }),
    );

    assert_eq!(app.models().get_copied(&dismiss_called), Some(true));
    assert_eq!(app.models().get_copied(&open), Some(true));
    assert_eq!(app.models().get_copied(&underlay_clicked), Some(false));
}

#[test]
fn non_modal_overlay_can_disable_outside_pointer_events_while_open() {
    let window = AppWindowId::default();
    let mut app = App::new();
    let mut ui: UiTree<App> = UiTree::new();
    ui.set_window(window);

    let open = app.models_mut().insert(false);
    let underlay_moved = app.models_mut().insert(false);
    let underlay_scroll = ScrollHandle::default();
    underlay_scroll.set_viewport_size(fret_core::Size::new(Px(160.0), Px(32.0)));
    underlay_scroll.set_content_size(fret_core::Size::new(Px(160.0), Px(200.0)));

    let mut services = FakeServices;
    let bounds = Rect::new(
        Point::new(Px(0.0), Px(0.0)),
        fret_core::Size::new(Px(300.0), Px(200.0)),
    );

    // Base layer with a pointer region that flips `underlay_moved` on pointer move.
    let (trigger, _underlay) = render_base_with_trigger_and_underlay_pointer_move(
        &mut ui,
        &mut app,
        &mut services,
        window,
        bounds,
        open.clone(),
        underlay_moved.clone(),
        underlay_scroll.clone(),
    );
    ui.layout_all(&mut app, &mut services, bounds, 1.0);

    ui.dispatch_event(
        &mut app,
        &mut services,
        &fret_core::Event::Pointer(fret_core::PointerEvent::Move {
            position: Point::new(Px(10.0), Px(130.0)),
            buttons: fret_core::MouseButtons::default(),
            modifiers: fret_core::Modifiers::default(),
            pointer_id: PointerId(0),
            pointer_type: fret_core::PointerType::Mouse,
        }),
    );
    assert_eq!(app.models().get_copied(&underlay_moved), Some(true));
    let _ = app.models_mut().update(&underlay_moved, |v| *v = false);

    // Second frame: request and render a dismissible popover that disables outside pointer events
    // while open (Radix `disableOutsidePointerEvents` outcome).
    let _ = app.models_mut().update(&open, |v| *v = true);
    begin_frame(&mut app, window);
    let (_trigger2, _underlay2) = render_base_with_trigger_and_underlay_pointer_move(
        &mut ui,
        &mut app,
        &mut services,
        window,
        bounds,
        open.clone(),
        underlay_moved.clone(),
        underlay_scroll.clone(),
    );

    request_dismissible_popover_for_window(
        &mut app,
        window,
        DismissiblePopoverRequest {
            id: trigger,
            root_name: popover_root_name(trigger),
            trigger,
            dismissable_branches: Vec::new(),
            consume_outside_pointer_events: true,
            disable_outside_pointer_events: true,
            close_on_window_focus_lost: false,
            close_on_window_resize: false,
            open: open.clone(),
            present: true,
            initial_focus: None,
            on_open_auto_focus: None,
            on_close_auto_focus: None,
            on_dismiss_request: None,
            on_pointer_move: None,
            children: Vec::new(),
        },
    );

    render(&mut ui, &mut app, &mut services, window, bounds);
    ui.layout_all(&mut app, &mut services, bounds, 1.0);

    ui.dispatch_event(
        &mut app,
        &mut services,
        &fret_core::Event::Pointer(fret_core::PointerEvent::Move {
            position: Point::new(Px(10.0), Px(130.0)),
            buttons: fret_core::MouseButtons::default(),
            modifiers: fret_core::Modifiers::default(),
            pointer_id: PointerId(0),
            pointer_type: fret_core::PointerType::Mouse,
        }),
    );
    assert_eq!(app.models().get_copied(&underlay_moved), Some(false));

    // Underlay scroll should still be reachable while outside pointer events are disabled:
    // the default policy uses an "except scroll" occlusion mode.
    let prev_scroll_y = underlay_scroll.offset().y;
    ui.dispatch_event(
        &mut app,
        &mut services,
        &fret_core::Event::Pointer(fret_core::PointerEvent::Wheel {
            position: Point::new(Px(10.0), Px(130.0)),
            delta: Point::new(Px(0.0), Px(-10.0)),
            modifiers: fret_core::Modifiers::default(),
            pointer_id: PointerId(0),
            pointer_type: fret_core::PointerType::Mouse,
        }),
    );
    assert!(underlay_scroll.offset().y.0 > prev_scroll_y.0);
}

#[test]
fn non_modal_menu_trigger_press_closes_without_reopening_under_occlusion() {
    let window = AppWindowId::default();
    let mut app = App::new();
    let mut ui: UiTree<App> = UiTree::new();
    ui.set_window(window);

    let open = app.models_mut().insert(false);

    let mut services = FakeServices;
    let bounds = Rect::new(
        Point::new(Px(0.0), Px(0.0)),
        fret_core::Size::new(Px(300.0), Px(200.0)),
    );

    // First frame: render base to establish stable bounds for the trigger element.
    let trigger = render_base_with_trigger(
        &mut ui,
        &mut app,
        &mut services,
        window,
        bounds,
        open.clone(),
    );
    ui.layout_all(&mut app, &mut services, bounds, 1.0);

    // Open via click.
    ui.dispatch_event(
        &mut app,
        &mut services,
        &fret_core::Event::Pointer(fret_core::PointerEvent::Down {
            position: Point::new(Px(10.0), Px(10.0)),
            button: fret_core::MouseButton::Left,
            modifiers: fret_core::Modifiers::default(),
            click_count: 1,
            pointer_id: PointerId(0),
            pointer_type: fret_core::PointerType::Mouse,
        }),
    );
    ui.dispatch_event(
        &mut app,
        &mut services,
        &fret_core::Event::Pointer(fret_core::PointerEvent::Up {
            position: Point::new(Px(10.0), Px(10.0)),
            button: fret_core::MouseButton::Left,
            modifiers: fret_core::Modifiers::default(),
            is_click: true,
            click_count: 1,
            pointer_id: PointerId(0),
            pointer_type: fret_core::PointerType::Mouse,
        }),
    );
    assert_eq!(app.models().get_copied(&open), Some(true));

    // Second frame: render a menu-like dismissible popover that disables outside pointer events.
    begin_frame(&mut app, window);
    let _trigger2 = render_base_with_trigger(
        &mut ui,
        &mut app,
        &mut services,
        window,
        bounds,
        open.clone(),
    );

    request_dismissible_popover_for_window(
        &mut app,
        window,
        DismissiblePopoverRequest {
            id: trigger,
            root_name: popover_root_name(trigger),
            trigger,
            dismissable_branches: Vec::new(),
            consume_outside_pointer_events: true,
            disable_outside_pointer_events: true,
            close_on_window_focus_lost: false,
            close_on_window_resize: false,
            open: open.clone(),
            present: true,
            initial_focus: None,
            on_open_auto_focus: None,
            on_close_auto_focus: None,
            on_dismiss_request: None,
            on_pointer_move: None,
            children: Vec::new(),
        },
    );

    render(&mut ui, &mut app, &mut services, window, bounds);
    ui.layout_all(&mut app, &mut services, bounds, 1.0);

    // Pressing the trigger while open should close the menu-like overlay without immediately
    // re-opening it (a common edge when outside-press dismissal runs before trigger activation).
    ui.dispatch_event(
        &mut app,
        &mut services,
        &fret_core::Event::Pointer(fret_core::PointerEvent::Down {
            position: Point::new(Px(10.0), Px(10.0)),
            button: fret_core::MouseButton::Left,
            modifiers: fret_core::Modifiers::default(),
            click_count: 1,
            pointer_id: PointerId(0),
            pointer_type: fret_core::PointerType::Mouse,
        }),
    );
    ui.dispatch_event(
        &mut app,
        &mut services,
        &fret_core::Event::Pointer(fret_core::PointerEvent::Up {
            position: Point::new(Px(10.0), Px(10.0)),
            button: fret_core::MouseButton::Left,
            modifiers: fret_core::Modifiers::default(),
            is_click: true,
            click_count: 1,
            pointer_id: PointerId(0),
            pointer_type: fret_core::PointerType::Mouse,
        }),
    );
    assert_eq!(app.models().get_copied(&open), Some(false));
}

#[test]
fn non_modal_menu_blocks_underlay_click_but_allows_wheel() {
    let window = AppWindowId::default();
    let mut app = App::new();
    let mut ui: UiTree<App> = UiTree::new();
    ui.set_window(window);

    let open = app.models_mut().insert(false);
    let underlay_clicked = app.models_mut().insert(false);

    let underlay_scroll = ScrollHandle::default();
    underlay_scroll.set_viewport_size(fret_core::Size::new(Px(160.0), Px(32.0)));
    underlay_scroll.set_content_size(fret_core::Size::new(Px(160.0), Px(200.0)));

    let mut services = FakeServices;
    let bounds = Rect::new(
        Point::new(Px(0.0), Px(0.0)),
        fret_core::Size::new(Px(300.0), Px(200.0)),
    );

    // First frame: base layer with trigger + underlay pressable + wheel region.
    let (trigger, _underlay) = render_base_with_trigger_and_underlay_pressable_wheel(
        &mut ui,
        &mut app,
        &mut services,
        window,
        bounds,
        open.clone(),
        underlay_clicked.clone(),
        underlay_scroll.clone(),
    );
    ui.layout_all(&mut app, &mut services, bounds, 1.0);

    // Open via click on the trigger.
    ui.dispatch_event(
        &mut app,
        &mut services,
        &fret_core::Event::Pointer(fret_core::PointerEvent::Down {
            position: Point::new(Px(10.0), Px(10.0)),
            button: fret_core::MouseButton::Left,
            modifiers: fret_core::Modifiers::default(),
            click_count: 1,
            pointer_id: PointerId(0),
            pointer_type: fret_core::PointerType::Mouse,
        }),
    );
    ui.dispatch_event(
        &mut app,
        &mut services,
        &fret_core::Event::Pointer(fret_core::PointerEvent::Up {
            position: Point::new(Px(10.0), Px(10.0)),
            button: fret_core::MouseButton::Left,
            modifiers: fret_core::Modifiers::default(),
            is_click: true,
            click_count: 1,
            pointer_id: PointerId(0),
            pointer_type: fret_core::PointerType::Mouse,
        }),
    );
    assert_eq!(app.models().get_copied(&open), Some(true));

    // Second frame: request a menu-like dismissible popover (consume outside presses + occlude mouse).
    begin_frame(&mut app, window);
    let (_trigger2, _underlay2) = render_base_with_trigger_and_underlay_pressable_wheel(
        &mut ui,
        &mut app,
        &mut services,
        window,
        bounds,
        open.clone(),
        underlay_clicked.clone(),
        underlay_scroll.clone(),
    );

    request_dismissible_popover_for_window(
        &mut app,
        window,
        DismissiblePopoverRequest {
            id: trigger,
            root_name: popover_root_name(trigger),
            trigger,
            dismissable_branches: Vec::new(),
            consume_outside_pointer_events: true,
            disable_outside_pointer_events: true,
            close_on_window_focus_lost: false,
            close_on_window_resize: false,
            open: open.clone(),
            present: true,
            initial_focus: None,
            on_open_auto_focus: None,
            on_close_auto_focus: None,
            on_dismiss_request: None,
            on_pointer_move: None,
            children: Vec::new(),
        },
    );

    render(&mut ui, &mut app, &mut services, window, bounds);
    ui.layout_all(&mut app, &mut services, bounds, 1.0);

    // Wheel should still reach the underlay scroll target even while mouse interactions are blocked.
    let prev_scroll_y = underlay_scroll.offset().y;
    ui.dispatch_event(
        &mut app,
        &mut services,
        &fret_core::Event::Pointer(fret_core::PointerEvent::Wheel {
            position: Point::new(Px(10.0), Px(130.0)),
            delta: Point::new(Px(0.0), Px(-10.0)),
            modifiers: fret_core::Modifiers::default(),
            pointer_id: PointerId(0),
            pointer_type: fret_core::PointerType::Mouse,
        }),
    );
    assert!(underlay_scroll.offset().y.0 > prev_scroll_y.0);

    // Clicking the underlay should dismiss without activating the underlay pressable.
    ui.dispatch_event(
        &mut app,
        &mut services,
        &fret_core::Event::Pointer(fret_core::PointerEvent::Down {
            position: Point::new(Px(10.0), Px(130.0)),
            button: fret_core::MouseButton::Left,
            modifiers: fret_core::Modifiers::default(),
            click_count: 1,
            pointer_id: PointerId(0),
            pointer_type: fret_core::PointerType::Mouse,
        }),
    );
    ui.dispatch_event(
        &mut app,
        &mut services,
        &fret_core::Event::Pointer(fret_core::PointerEvent::Up {
            position: Point::new(Px(10.0), Px(130.0)),
            button: fret_core::MouseButton::Left,
            modifiers: fret_core::Modifiers::default(),
            is_click: true,
            click_count: 1,
            pointer_id: PointerId(0),
            pointer_type: fret_core::PointerType::Mouse,
        }),
    );
    assert_eq!(app.models().get_copied(&open), Some(false));
    assert_eq!(app.models().get_copied(&underlay_clicked), Some(false));
}

#[test]
fn dock_drag_closes_non_modal_overlays_for_entire_window() {
    let window = AppWindowId::default();
    let mut app = App::new();
    let mut ui: UiTree<App> = UiTree::new();
    ui.set_window(window);

    let open = app.models_mut().insert(false);

    let mut services = FakeServices;
    let bounds = Rect::new(
        Point::new(Px(0.0), Px(0.0)),
        fret_core::Size::new(Px(300.0), Px(200.0)),
    );

    // First frame: render base to establish stable bounds for the trigger element.
    let trigger = render_base_with_trigger(
        &mut ui,
        &mut app,
        &mut services,
        window,
        bounds,
        open.clone(),
    );
    ui.layout_all(&mut app, &mut services, bounds, 1.0);

    // Second frame: open a non-modal popover overlay.
    let _ = app.models_mut().update(&open, |v| *v = true);
    begin_frame(&mut app, window);
    let _trigger2 = render_base_with_trigger(
        &mut ui,
        &mut app,
        &mut services,
        window,
        bounds,
        open.clone(),
    );
    request_dismissible_popover_for_window(
        &mut app,
        window,
        DismissiblePopoverRequest {
            id: trigger,
            root_name: popover_root_name(trigger),
            trigger,
            dismissable_branches: Vec::new(),
            consume_outside_pointer_events: false,
            disable_outside_pointer_events: false,
            close_on_window_focus_lost: false,
            close_on_window_resize: false,
            open: open.clone(),
            present: true,
            initial_focus: None,
            on_open_auto_focus: None,
            on_close_auto_focus: None,
            on_dismiss_request: None,
            on_pointer_move: None,
            children: Vec::new(),
        },
    );
    render(&mut ui, &mut app, &mut services, window, bounds);
    ui.layout_all(&mut app, &mut services, bounds, 1.0);
    assert_eq!(app.models().get_copied(&open), Some(true));

    // Start a dock drag session for a *different* pointer id (window-global suppression).
    app.begin_drag_with_kind(
        PointerId(7),
        fret_runtime::DRAG_KIND_DOCK_PANEL,
        window,
        Point::new(Px(10.0), Px(10.0)),
        (),
    );

    // Third frame: re-request the overlay; window_overlays policy should force it closed.
    begin_frame(&mut app, window);
    let _trigger3 = render_base_with_trigger(
        &mut ui,
        &mut app,
        &mut services,
        window,
        bounds,
        open.clone(),
    );
    request_dismissible_popover_for_window(
        &mut app,
        window,
        DismissiblePopoverRequest {
            id: trigger,
            root_name: popover_root_name(trigger),
            trigger,
            dismissable_branches: Vec::new(),
            consume_outside_pointer_events: false,
            disable_outside_pointer_events: false,
            close_on_window_focus_lost: false,
            close_on_window_resize: false,
            open: open.clone(),
            present: true,
            initial_focus: None,
            on_open_auto_focus: None,
            on_close_auto_focus: None,
            on_dismiss_request: None,
            on_pointer_move: None,
            children: Vec::new(),
        },
    );
    render(&mut ui, &mut app, &mut services, window, bounds);
    ui.layout_all(&mut app, &mut services, bounds, 1.0);
    assert_eq!(app.models().get_copied(&open), Some(false));
}

#[test]
fn dock_drag_closes_dismissible_popovers_only_in_affected_window() {
    use slotmap::KeyData;

    let window_a = AppWindowId::from(KeyData::from_ffi(1));
    let window_b = AppWindowId::from(KeyData::from_ffi(2));

    let mut app = App::new();

    let mut ui_a: UiTree<App> = UiTree::new();
    ui_a.set_window(window_a);
    let mut ui_b: UiTree<App> = UiTree::new();
    ui_b.set_window(window_b);

    let open_a = app.models_mut().insert(false);
    let open_b = app.models_mut().insert(false);

    let mut services = FakeServices;
    let bounds = Rect::new(
        Point::new(Px(0.0), Px(0.0)),
        fret_core::Size::new(Px(300.0), Px(200.0)),
    );

    // Frame 1: render base to establish stable bounds for the trigger element in each window.
    let trigger_a = render_base_with_trigger(
        &mut ui_a,
        &mut app,
        &mut services,
        window_a,
        bounds,
        open_a.clone(),
    );
    ui_a.layout_all(&mut app, &mut services, bounds, 1.0);

    let trigger_b = render_base_with_trigger(
        &mut ui_b,
        &mut app,
        &mut services,
        window_b,
        bounds,
        open_b.clone(),
    );
    ui_b.layout_all(&mut app, &mut services, bounds, 1.0);

    // Frame 2: open a non-modal popover overlay in both windows.
    let _ = app.models_mut().update(&open_a, |v| *v = true);
    let _ = app.models_mut().update(&open_b, |v| *v = true);

    begin_frame(&mut app, window_a);
    let _ = render_base_with_trigger(
        &mut ui_a,
        &mut app,
        &mut services,
        window_a,
        bounds,
        open_a.clone(),
    );
    request_dismissible_popover_for_window(
        &mut app,
        window_a,
        DismissiblePopoverRequest {
            id: trigger_a,
            root_name: popover_root_name(trigger_a),
            trigger: trigger_a,
            dismissable_branches: Vec::new(),
            consume_outside_pointer_events: false,
            disable_outside_pointer_events: false,
            close_on_window_focus_lost: false,
            close_on_window_resize: false,
            open: open_a.clone(),
            present: true,
            initial_focus: None,
            on_open_auto_focus: None,
            on_close_auto_focus: None,
            on_dismiss_request: None,
            on_pointer_move: None,
            children: Vec::new(),
        },
    );
    render(&mut ui_a, &mut app, &mut services, window_a, bounds);
    ui_a.layout_all(&mut app, &mut services, bounds, 1.0);

    begin_frame(&mut app, window_b);
    let _ = render_base_with_trigger(
        &mut ui_b,
        &mut app,
        &mut services,
        window_b,
        bounds,
        open_b.clone(),
    );
    request_dismissible_popover_for_window(
        &mut app,
        window_b,
        DismissiblePopoverRequest {
            id: trigger_b,
            root_name: popover_root_name(trigger_b),
            trigger: trigger_b,
            dismissable_branches: Vec::new(),
            consume_outside_pointer_events: false,
            disable_outside_pointer_events: false,
            close_on_window_focus_lost: false,
            close_on_window_resize: false,
            open: open_b.clone(),
            present: true,
            initial_focus: None,
            on_open_auto_focus: None,
            on_close_auto_focus: None,
            on_dismiss_request: None,
            on_pointer_move: None,
            children: Vec::new(),
        },
    );
    render(&mut ui_b, &mut app, &mut services, window_b, bounds);
    ui_b.layout_all(&mut app, &mut services, bounds, 1.0);

    assert_eq!(app.models().get_copied(&open_a), Some(true));
    assert_eq!(app.models().get_copied(&open_b), Some(true));

    // Start a dock drag session for window A only.
    app.begin_drag_with_kind(
        PointerId(7),
        fret_runtime::DRAG_KIND_DOCK_PANEL,
        window_a,
        Point::new(Px(10.0), Px(10.0)),
        (),
    );

    // Frame 3: window A popover should be force-closed; window B popover should remain open.
    begin_frame(&mut app, window_a);
    let _ = render_base_with_trigger(
        &mut ui_a,
        &mut app,
        &mut services,
        window_a,
        bounds,
        open_a.clone(),
    );
    request_dismissible_popover_for_window(
        &mut app,
        window_a,
        DismissiblePopoverRequest {
            id: trigger_a,
            root_name: popover_root_name(trigger_a),
            trigger: trigger_a,
            dismissable_branches: Vec::new(),
            consume_outside_pointer_events: false,
            disable_outside_pointer_events: false,
            close_on_window_focus_lost: false,
            close_on_window_resize: false,
            open: open_a.clone(),
            present: true,
            initial_focus: None,
            on_open_auto_focus: None,
            on_close_auto_focus: None,
            on_dismiss_request: None,
            on_pointer_move: None,
            children: Vec::new(),
        },
    );
    render(&mut ui_a, &mut app, &mut services, window_a, bounds);
    ui_a.layout_all(&mut app, &mut services, bounds, 1.0);

    begin_frame(&mut app, window_b);
    let _ = render_base_with_trigger(
        &mut ui_b,
        &mut app,
        &mut services,
        window_b,
        bounds,
        open_b.clone(),
    );
    request_dismissible_popover_for_window(
        &mut app,
        window_b,
        DismissiblePopoverRequest {
            id: trigger_b,
            root_name: popover_root_name(trigger_b),
            trigger: trigger_b,
            dismissable_branches: Vec::new(),
            consume_outside_pointer_events: false,
            disable_outside_pointer_events: false,
            close_on_window_focus_lost: false,
            close_on_window_resize: false,
            open: open_b.clone(),
            present: true,
            initial_focus: None,
            on_open_auto_focus: None,
            on_close_auto_focus: None,
            on_dismiss_request: None,
            on_pointer_move: None,
            children: Vec::new(),
        },
    );
    render(&mut ui_b, &mut app, &mut services, window_b, bounds);
    ui_b.layout_all(&mut app, &mut services, bounds, 1.0);

    assert_eq!(app.models().get_copied(&open_a), Some(false));
    assert_eq!(
        app.models().get_copied(&open_b),
        Some(true),
        "expected dock drag to only affect overlays in windows participating in the drag session"
    );
}

#[test]
fn dock_drag_hides_hover_overlays_in_affected_window() {
    let window = AppWindowId::default();
    let mut app = App::new();
    let mut ui: UiTree<App> = UiTree::new();
    ui.set_window(window);

    let open = app.models_mut().insert(false);
    let underlay_clicked = app.models_mut().insert(false);

    let mut services = FakeServices;
    let bounds = Rect::new(
        Point::new(Px(0.0), Px(0.0)),
        fret_core::Size::new(Px(300.0), Px(200.0)),
    );

    let (trigger, _underlay) = render_base_with_trigger_and_underlay(
        &mut ui,
        &mut app,
        &mut services,
        window,
        bounds,
        open.clone(),
        underlay_clicked.clone(),
    );
    render(&mut ui, &mut app, &mut services, window, bounds);
    ui.layout_all(&mut app, &mut services, bounds, 1.0);

    request_hover_overlay_for_window(
        &mut app,
        window,
        HoverOverlayRequest {
            id: trigger,
            root_name: hover_overlay_root_name(trigger),
            interactive: true,
            trigger,
            children: Vec::new(),
        },
    );
    render(&mut ui, &mut app, &mut services, window, bounds);

    let layer = app.with_global_mut_untracked(WindowOverlays::default, |overlays, _app| {
        overlays
            .hover_overlays
            .get(&(window, trigger))
            .map(|h| h.layer)
    });
    let layer = layer.expect("hover overlay layer");
    assert!(ui.is_layer_visible(layer));

    app.begin_drag_with_kind(
        PointerId(7),
        fret_runtime::DRAG_KIND_DOCK_PANEL,
        window,
        Point::new(Px(10.0), Px(10.0)),
        (),
    );

    begin_frame(&mut app, window);
    let _ = render_base_with_trigger_and_underlay(
        &mut ui,
        &mut app,
        &mut services,
        window,
        bounds,
        open.clone(),
        underlay_clicked.clone(),
    );
    request_hover_overlay_for_window(
        &mut app,
        window,
        HoverOverlayRequest {
            id: trigger,
            root_name: hover_overlay_root_name(trigger),
            interactive: true,
            trigger,
            children: Vec::new(),
        },
    );
    render(&mut ui, &mut app, &mut services, window, bounds);

    assert!(
        !ui.is_layer_visible(layer),
        "expected dock drag to hide hover overlays in the affected window"
    );

    app.cancel_drag(PointerId(7));
    begin_frame(&mut app, window);
    let _ = render_base_with_trigger_and_underlay(
        &mut ui,
        &mut app,
        &mut services,
        window,
        bounds,
        open.clone(),
        underlay_clicked,
    );
    request_hover_overlay_for_window(
        &mut app,
        window,
        HoverOverlayRequest {
            id: trigger,
            root_name: hover_overlay_root_name(trigger),
            interactive: true,
            trigger,
            children: Vec::new(),
        },
    );
    render(&mut ui, &mut app, &mut services, window, bounds);

    assert!(
        ui.is_layer_visible(layer),
        "expected hover overlays to become visible again after dock drag ends"
    );
}

#[test]
fn dock_drag_hides_tooltips_in_affected_window() {
    let window = AppWindowId::default();
    let mut app = App::new();
    let mut ui: UiTree<App> = UiTree::new();
    ui.set_window(window);

    let open = app.models_mut().insert(false);
    let underlay_clicked = app.models_mut().insert(false);

    let mut services = FakeServices;
    let bounds = Rect::new(
        Point::new(Px(0.0), Px(0.0)),
        fret_core::Size::new(Px(300.0), Px(200.0)),
    );

    let (trigger, _underlay) = render_base_with_trigger_and_underlay(
        &mut ui,
        &mut app,
        &mut services,
        window,
        bounds,
        open.clone(),
        underlay_clicked.clone(),
    );
    render(&mut ui, &mut app, &mut services, window, bounds);
    ui.layout_all(&mut app, &mut services, bounds, 1.0);

    request_tooltip_for_window(
        &mut app,
        window,
        TooltipRequest {
            id: trigger,
            root_name: tooltip_root_name(trigger),
            interactive: true,
            trigger: Some(trigger),
            on_dismiss_request: None,
            on_pointer_move: None,
            children: Vec::new(),
        },
    );
    render(&mut ui, &mut app, &mut services, window, bounds);

    let layer = app.with_global_mut_untracked(WindowOverlays::default, |overlays, _app| {
        overlays.tooltips.get(&(window, trigger)).map(|t| t.layer)
    });
    let layer = layer.expect("tooltip layer");
    assert!(ui.is_layer_visible(layer));

    app.begin_drag_with_kind(
        PointerId(7),
        fret_runtime::DRAG_KIND_DOCK_PANEL,
        window,
        Point::new(Px(10.0), Px(10.0)),
        (),
    );

    begin_frame(&mut app, window);
    let _ = render_base_with_trigger_and_underlay(
        &mut ui,
        &mut app,
        &mut services,
        window,
        bounds,
        open.clone(),
        underlay_clicked.clone(),
    );
    request_tooltip_for_window(
        &mut app,
        window,
        TooltipRequest {
            id: trigger,
            root_name: tooltip_root_name(trigger),
            interactive: true,
            trigger: Some(trigger),
            on_dismiss_request: None,
            on_pointer_move: None,
            children: Vec::new(),
        },
    );
    render(&mut ui, &mut app, &mut services, window, bounds);

    assert!(
        !ui.is_layer_visible(layer),
        "expected dock drag to hide tooltips in the affected window"
    );

    app.cancel_drag(PointerId(7));
    begin_frame(&mut app, window);
    let _ = render_base_with_trigger_and_underlay(
        &mut ui,
        &mut app,
        &mut services,
        window,
        bounds,
        open.clone(),
        underlay_clicked,
    );
    request_tooltip_for_window(
        &mut app,
        window,
        TooltipRequest {
            id: trigger,
            root_name: tooltip_root_name(trigger),
            interactive: true,
            trigger: Some(trigger),
            on_dismiss_request: None,
            on_pointer_move: None,
            children: Vec::new(),
        },
    );
    render(&mut ui, &mut app, &mut services, window, bounds);

    assert!(
        ui.is_layer_visible(layer),
        "expected tooltips to become visible again after dock drag ends"
    );
}

#[test]
fn dock_drag_forces_menu_like_overlay_to_drop_pointer_occlusion_while_closing() {
    let window = AppWindowId::default();
    let mut app = App::new();
    let mut ui: UiTree<App> = UiTree::new();
    ui.set_window(window);

    let open = app.models_mut().insert(false);

    let mut services = FakeServices;
    let bounds = Rect::new(
        Point::new(Px(0.0), Px(0.0)),
        fret_core::Size::new(Px(300.0), Px(200.0)),
    );

    // First frame: render base to establish stable bounds for the trigger element.
    let trigger = render_base_with_trigger(
        &mut ui,
        &mut app,
        &mut services,
        window,
        bounds,
        open.clone(),
    );
    ui.layout_all(&mut app, &mut services, bounds, 1.0);

    // Second frame: open a menu-like overlay that enables pointer occlusion.
    let _ = app.models_mut().update(&open, |v| *v = true);
    begin_frame(&mut app, window);
    let _trigger2 = render_base_with_trigger(
        &mut ui,
        &mut app,
        &mut services,
        window,
        bounds,
        open.clone(),
    );
    request_dismissible_popover_for_window(
        &mut app,
        window,
        DismissiblePopoverRequest {
            id: trigger,
            root_name: popover_root_name(trigger),
            trigger,
            dismissable_branches: Vec::new(),
            consume_outside_pointer_events: true,
            disable_outside_pointer_events: true,
            close_on_window_focus_lost: false,
            close_on_window_resize: false,
            open: open.clone(),
            present: true,
            initial_focus: None,
            on_open_auto_focus: None,
            on_close_auto_focus: None,
            on_dismiss_request: None,
            on_pointer_move: None,
            children: Vec::new(),
        },
    );
    render(&mut ui, &mut app, &mut services, window, bounds);
    ui.layout_all(&mut app, &mut services, bounds, 1.0);

    let snap = crate::overlay_controller::OverlayController::stack_snapshot_for_window(
        &ui, &mut app, window,
    );
    assert_eq!(app.models().get_copied(&open), Some(true));
    assert_eq!(
        snap.arbitration.pointer_occlusion,
        fret_ui::tree::PointerOcclusion::BlockMouseExceptScroll,
        "expected menu-like overlay to enable pointer occlusion while open"
    );
    assert_eq!(snap.topmost_pointer_occluding_overlay, Some(trigger));

    // Start a dock drag session for a *different* pointer id (window-global suppression).
    app.begin_drag_with_kind(
        PointerId(7),
        fret_runtime::DRAG_KIND_DOCK_PANEL,
        window,
        Point::new(Px(10.0), Px(10.0)),
        (),
    );

    // Third frame: re-request the overlay; window_overlays policy should force it closed and
    // drop pointer occlusion even if the layer remains present.
    begin_frame(&mut app, window);
    let _trigger3 = render_base_with_trigger(
        &mut ui,
        &mut app,
        &mut services,
        window,
        bounds,
        open.clone(),
    );
    request_dismissible_popover_for_window(
        &mut app,
        window,
        DismissiblePopoverRequest {
            id: trigger,
            root_name: popover_root_name(trigger),
            trigger,
            dismissable_branches: Vec::new(),
            consume_outside_pointer_events: true,
            disable_outside_pointer_events: true,
            close_on_window_focus_lost: false,
            close_on_window_resize: false,
            open: open.clone(),
            present: true,
            initial_focus: None,
            on_open_auto_focus: None,
            on_close_auto_focus: None,
            on_dismiss_request: None,
            on_pointer_move: None,
            children: Vec::new(),
        },
    );
    render(&mut ui, &mut app, &mut services, window, bounds);
    ui.layout_all(&mut app, &mut services, bounds, 1.0);

    let snap = crate::overlay_controller::OverlayController::stack_snapshot_for_window(
        &ui, &mut app, window,
    );
    assert_eq!(app.models().get_copied(&open), Some(false));
    assert_eq!(
        snap.arbitration.pointer_occlusion,
        fret_ui::tree::PointerOcclusion::None,
        "expected dock drag to force menu-like overlay to drop pointer occlusion"
    );
    assert_eq!(snap.topmost_pointer_occluding_overlay, None);
}

fn render_base_with_trigger_and_capture_underlay(
    ui: &mut UiTree<App>,
    app: &mut App,
    services: &mut dyn fret_core::UiServices,
    window: AppWindowId,
    bounds: Rect,
    open: Model<bool>,
) -> (GlobalElementId, GlobalElementId) {
    begin_frame(app, window);

    let mut trigger_id: Option<GlobalElementId> = None;
    let mut underlay_id: Option<GlobalElementId> = None;

    let root = fret_ui::declarative::render_root(ui, app, services, window, bounds, "test", |cx| {
        vec![cx.container(
            ContainerProps {
                layout: {
                    let mut layout = LayoutStyle::default();
                    layout.size.width = Length::Fill;
                    layout.size.height = Length::Fill;
                    layout
                },
                ..Default::default()
            },
            |cx| {
                let trigger = cx.pressable_with_id(
                    PressableProps {
                        layout: {
                            LayoutStyle {
                                position: PositionStyle::Absolute,
                                inset: InsetStyle {
                                    left: Some(Px(0.0)),
                                    top: Some(Px(0.0)),
                                    ..Default::default()
                                },
                                size: SizeStyle {
                                    width: Length::Px(Px(80.0)),
                                    height: Length::Px(Px(32.0)),
                                    ..Default::default()
                                },
                                ..Default::default()
                            }
                        },
                        enabled: true,
                        focusable: true,
                        ..Default::default()
                    },
                    |cx, _st, id| {
                        cx.pressable_toggle_bool(&open);
                        trigger_id = Some(id);
                        Vec::new()
                    },
                );

                let underlay = cx.pointer_region(
                    PointerRegionProps {
                        layout: {
                            LayoutStyle {
                                position: PositionStyle::Absolute,
                                inset: InsetStyle {
                                    left: Some(Px(0.0)),
                                    top: Some(Px(120.0)),
                                    ..Default::default()
                                },
                                size: SizeStyle {
                                    width: Length::Px(Px(160.0)),
                                    height: Length::Px(Px(32.0)),
                                    ..Default::default()
                                },
                                ..Default::default()
                            }
                        },
                        enabled: true,
                    },
                    |cx| {
                        cx.pointer_region_on_pointer_down(Arc::new(move |host, _cx, _down| {
                            host.capture_pointer();
                            true
                        }));
                        Vec::new()
                    },
                );
                underlay_id = Some(underlay.id);

                vec![trigger, underlay]
            },
        )]
    });
    ui.set_root(root);

    (
        trigger_id.expect("trigger id"),
        underlay_id.expect("underlay id"),
    )
}

#[test]
fn dock_drag_does_not_restore_closed_non_modal_overlays_on_drag_end() {
    let window = AppWindowId::default();
    let mut app = App::new();
    let mut ui: UiTree<App> = UiTree::new();
    ui.set_window(window);

    let open = app.models_mut().insert(false);

    let mut services = FakeServices;
    let bounds = Rect::new(
        Point::new(Px(0.0), Px(0.0)),
        fret_core::Size::new(Px(300.0), Px(200.0)),
    );

    // First frame: render base to establish stable bounds for the trigger element.
    let trigger = render_base_with_trigger(
        &mut ui,
        &mut app,
        &mut services,
        window,
        bounds,
        open.clone(),
    );
    ui.layout_all(&mut app, &mut services, bounds, 1.0);

    // Second frame: open a non-modal popover overlay.
    let _ = app.models_mut().update(&open, |v| *v = true);
    begin_frame(&mut app, window);
    let _ = render_base_with_trigger(
        &mut ui,
        &mut app,
        &mut services,
        window,
        bounds,
        open.clone(),
    );
    request_dismissible_popover_for_window(
        &mut app,
        window,
        DismissiblePopoverRequest {
            id: trigger,
            root_name: popover_root_name(trigger),
            trigger,
            dismissable_branches: Vec::new(),
            consume_outside_pointer_events: false,
            disable_outside_pointer_events: false,
            close_on_window_focus_lost: false,
            close_on_window_resize: false,
            open: open.clone(),
            present: true,
            initial_focus: None,
            on_open_auto_focus: None,
            on_close_auto_focus: None,
            on_dismiss_request: None,
            on_pointer_move: None,
            children: Vec::new(),
        },
    );
    render(&mut ui, &mut app, &mut services, window, bounds);
    ui.layout_all(&mut app, &mut services, bounds, 1.0);
    assert_eq!(app.models().get_copied(&open), Some(true));

    // Start a dock drag session; policy should force-close the non-modal overlay.
    app.begin_drag_with_kind(
        PointerId(7),
        fret_runtime::DRAG_KIND_DOCK_PANEL,
        window,
        Point::new(Px(10.0), Px(10.0)),
        (),
    );

    // Third frame: re-request the overlay; window_overlays policy should force it closed.
    begin_frame(&mut app, window);
    let _ = render_base_with_trigger(
        &mut ui,
        &mut app,
        &mut services,
        window,
        bounds,
        open.clone(),
    );
    request_dismissible_popover_for_window(
        &mut app,
        window,
        DismissiblePopoverRequest {
            id: trigger,
            root_name: popover_root_name(trigger),
            trigger,
            dismissable_branches: Vec::new(),
            consume_outside_pointer_events: false,
            disable_outside_pointer_events: false,
            close_on_window_focus_lost: false,
            close_on_window_resize: false,
            open: open.clone(),
            present: true,
            initial_focus: None,
            on_open_auto_focus: None,
            on_close_auto_focus: None,
            on_dismiss_request: None,
            on_pointer_move: None,
            children: Vec::new(),
        },
    );
    render(&mut ui, &mut app, &mut services, window, bounds);
    ui.layout_all(&mut app, &mut services, bounds, 1.0);
    assert_eq!(app.models().get_copied(&open), Some(false));

    // End the drag and render another frame; overlays should stay closed unless the user reopens.
    app.cancel_drag(PointerId(7));
    begin_frame(&mut app, window);
    let _ = render_base_with_trigger(
        &mut ui,
        &mut app,
        &mut services,
        window,
        bounds,
        open.clone(),
    );
    request_dismissible_popover_for_window(
        &mut app,
        window,
        DismissiblePopoverRequest {
            id: trigger,
            root_name: popover_root_name(trigger),
            trigger,
            dismissable_branches: Vec::new(),
            consume_outside_pointer_events: false,
            disable_outside_pointer_events: false,
            close_on_window_focus_lost: false,
            close_on_window_resize: false,
            open: open.clone(),
            present: true,
            initial_focus: None,
            on_open_auto_focus: None,
            on_close_auto_focus: None,
            on_dismiss_request: None,
            on_pointer_move: None,
            children: Vec::new(),
        },
    );
    render(&mut ui, &mut app, &mut services, window, bounds);
    ui.layout_all(&mut app, &mut services, bounds, 1.0);
    assert_eq!(app.models().get_copied(&open), Some(false));
}

#[test]
fn pointer_capture_forces_menu_like_overlay_to_close_and_drop_occlusion() {
    let window = AppWindowId::default();
    let mut app = App::new();
    let mut ui: UiTree<App> = UiTree::new();
    ui.set_window(window);

    let open = app.models_mut().insert(false);

    let mut services = FakeServices;
    let bounds = Rect::new(
        Point::new(Px(0.0), Px(0.0)),
        fret_core::Size::new(Px(300.0), Px(200.0)),
    );

    // First frame: render base and capture pointer 0 in the underlay (viewport-like capture).
    let (trigger, _underlay) = render_base_with_trigger_and_capture_underlay(
        &mut ui,
        &mut app,
        &mut services,
        window,
        bounds,
        open.clone(),
    );
    ui.layout_all(&mut app, &mut services, bounds, 1.0);
    ui.dispatch_event(
        &mut app,
        &mut services,
        &Event::Pointer(fret_core::PointerEvent::Down {
            position: Point::new(Px(10.0), Px(130.0)),
            button: fret_core::MouseButton::Left,
            modifiers: fret_core::Modifiers::default(),
            click_count: 1,
            pointer_id: PointerId(0),
            pointer_type: fret_core::PointerType::Mouse,
        }),
    );
    assert!(
        ui.any_captured_node().is_some(),
        "expected pointer capture to be active before opening the menu-like overlay"
    );

    // Second frame: attempt to open a menu-like overlay that would normally enable pointer occlusion.
    let _ = app.models_mut().update(&open, |v| *v = true);
    begin_frame(&mut app, window);
    let _ = render_base_with_trigger_and_capture_underlay(
        &mut ui,
        &mut app,
        &mut services,
        window,
        bounds,
        open.clone(),
    );
    request_dismissible_popover_for_window(
        &mut app,
        window,
        DismissiblePopoverRequest {
            id: trigger,
            root_name: popover_root_name(trigger),
            trigger,
            dismissable_branches: Vec::new(),
            consume_outside_pointer_events: true,
            disable_outside_pointer_events: true,
            close_on_window_focus_lost: false,
            close_on_window_resize: false,
            open: open.clone(),
            present: true,
            initial_focus: None,
            on_open_auto_focus: None,
            on_close_auto_focus: None,
            on_dismiss_request: None,
            on_pointer_move: None,
            children: Vec::new(),
        },
    );
    render(&mut ui, &mut app, &mut services, window, bounds);
    ui.layout_all(&mut app, &mut services, bounds, 1.0);

    assert_eq!(
        app.models().get_copied(&open),
        Some(true),
        "expected capture to suspend pointer gating (not force-close) for menu-like overlays"
    );

    let snap = crate::overlay_controller::OverlayController::stack_snapshot_for_window(
        &ui, &mut app, window,
    );
    assert_eq!(
        snap.arbitration.pointer_occlusion,
        fret_ui::tree::PointerOcclusion::None,
        "expected pointer occlusion to be suppressed while capture is active"
    );
    assert_eq!(snap.topmost_pointer_occluding_overlay, None);

    let base_root = ui.base_root().expect("base root");
    let popover_layer = ui
        .debug_layers_in_paint_order()
        .into_iter()
        .find(|l| l.root != base_root)
        .expect("popover layer");
    assert!(
        !popover_layer.hit_testable,
        "expected capture to suspend popover pointer hit-testing"
    );
}

#[test]
fn pointer_capture_forces_consuming_popover_to_close() {
    let window = AppWindowId::default();
    let mut app = App::new();
    let mut ui: UiTree<App> = UiTree::new();
    ui.set_window(window);

    let open = app.models_mut().insert(false);

    let mut services = FakeServices;
    let bounds = Rect::new(
        Point::new(Px(0.0), Px(0.0)),
        fret_core::Size::new(Px(300.0), Px(200.0)),
    );

    // First frame: render base and capture pointer 0 in the underlay (viewport-like capture).
    let (trigger, _underlay) = render_base_with_trigger_and_capture_underlay(
        &mut ui,
        &mut app,
        &mut services,
        window,
        bounds,
        open.clone(),
    );
    ui.layout_all(&mut app, &mut services, bounds, 1.0);
    ui.dispatch_event(
        &mut app,
        &mut services,
        &Event::Pointer(fret_core::PointerEvent::Down {
            position: Point::new(Px(10.0), Px(130.0)),
            button: fret_core::MouseButton::Left,
            modifiers: fret_core::Modifiers::default(),
            click_count: 1,
            pointer_id: PointerId(0),
            pointer_type: fret_core::PointerType::Mouse,
        }),
    );
    assert!(
        ui.any_captured_node().is_some(),
        "expected pointer capture to be active before opening the consuming popover"
    );

    // Second frame: attempt to open a consuming non-modal overlay. Even without pointer occlusion,
    // we must not introduce non-click-through dismissal semantics while another layer owns capture.
    let _ = app.models_mut().update(&open, |v| *v = true);
    begin_frame(&mut app, window);
    let _ = render_base_with_trigger_and_capture_underlay(
        &mut ui,
        &mut app,
        &mut services,
        window,
        bounds,
        open.clone(),
    );
    request_dismissible_popover_for_window(
        &mut app,
        window,
        DismissiblePopoverRequest {
            id: trigger,
            root_name: popover_root_name(trigger),
            trigger,
            dismissable_branches: Vec::new(),
            consume_outside_pointer_events: true,
            disable_outside_pointer_events: false,
            close_on_window_focus_lost: false,
            close_on_window_resize: false,
            open: open.clone(),
            present: true,
            initial_focus: None,
            on_open_auto_focus: None,
            on_close_auto_focus: None,
            on_dismiss_request: None,
            on_pointer_move: None,
            children: Vec::new(),
        },
    );
    render(&mut ui, &mut app, &mut services, window, bounds);
    ui.layout_all(&mut app, &mut services, bounds, 1.0);

    assert_eq!(
        app.models().get_copied(&open),
        Some(true),
        "expected pointer capture to suspend pointer gating (not force-close) for consuming popovers"
    );

    let base_root = ui.base_root().expect("base root");
    let popover_layer = ui
        .debug_layers_in_paint_order()
        .into_iter()
        .find(|l| l.root != base_root)
        .expect("popover layer");
    assert!(
        !popover_layer.hit_testable,
        "expected capture to suspend popover pointer hit-testing"
    );
}

#[test]
fn dock_drag_restores_focus_when_focus_is_missing_on_drag_end() {
    let window = AppWindowId::default();
    let mut app = App::new();
    let mut ui: UiTree<App> = UiTree::new();
    ui.set_window(window);

    let open = app.models_mut().insert(false);

    let mut services = FakeServices;
    let bounds = Rect::new(
        Point::new(Px(0.0), Px(0.0)),
        fret_core::Size::new(Px(300.0), Px(200.0)),
    );

    // First frame: render base and focus the trigger.
    let trigger = render_base_with_trigger(
        &mut ui,
        &mut app,
        &mut services,
        window,
        bounds,
        open.clone(),
    );
    render(&mut ui, &mut app, &mut services, window, bounds);
    ui.layout_all(&mut app, &mut services, bounds, 1.0);
    let trigger_node =
        fret_ui::elements::node_for_element(&mut app, window, trigger).expect("trigger node");
    ui.set_focus(Some(trigger_node));

    // Start a dock drag session and render a frame so the overlay policy can record the focus snapshot.
    app.begin_cross_window_drag_with_kind(
        PointerId(7),
        fret_runtime::DRAG_KIND_DOCK_PANEL,
        window,
        Point::new(Px(10.0), Px(10.0)),
        (),
    );
    begin_frame(&mut app, window);
    let _ = render_base_with_trigger(
        &mut ui,
        &mut app,
        &mut services,
        window,
        bounds,
        open.clone(),
    );
    render(&mut ui, &mut app, &mut services, window, bounds);
    ui.layout_all(&mut app, &mut services, bounds, 1.0);

    // Simulate focus being cleared during the drag (platform/runner behavior).
    ui.set_focus(None);

    // End the drag and render another frame; focus should restore to the pre-drag focus node.
    app.cancel_drag(PointerId(7));
    begin_frame(&mut app, window);
    let _ = render_base_with_trigger(
        &mut ui,
        &mut app,
        &mut services,
        window,
        bounds,
        open.clone(),
    );
    render(&mut ui, &mut app, &mut services, window, bounds);
    ui.layout_all(&mut app, &mut services, bounds, 1.0);

    assert_eq!(ui.focus(), Some(trigger_node));
}

#[test]
fn pointer_capture_hides_hover_overlays_in_same_window() {
    let window = AppWindowId::default();
    let mut app = App::new();
    let mut ui: UiTree<App> = UiTree::new();
    ui.set_window(window);

    let open = app.models_mut().insert(false);
    let underlay_clicked = app.models_mut().insert(false);

    let mut services = FakeServices;
    let bounds = Rect::new(
        Point::new(Px(0.0), Px(0.0)),
        fret_core::Size::new(Px(300.0), Px(200.0)),
    );

    let (trigger, _underlay) = render_base_with_trigger_and_underlay(
        &mut ui,
        &mut app,
        &mut services,
        window,
        bounds,
        open.clone(),
        underlay_clicked.clone(),
    );
    ui.layout_all(&mut app, &mut services, bounds, 1.0);

    request_hover_overlay_for_window(
        &mut app,
        window,
        HoverOverlayRequest {
            id: trigger,
            root_name: hover_overlay_root_name(trigger),
            interactive: true,
            trigger,
            children: Vec::new(),
        },
    );
    render(&mut ui, &mut app, &mut services, window, bounds);

    let layer = app.with_global_mut_untracked(WindowOverlays::default, |overlays, _app| {
        overlays
            .hover_overlays
            .get(&(window, trigger))
            .map(|h| h.layer)
    });
    let layer = layer.expect("hover overlay layer");
    assert!(ui.is_layer_visible(layer));

    // Start a pointer-capture session by pressing (without releasing) a `Pressable`.
    ui.dispatch_event(
        &mut app,
        &mut services,
        &fret_core::Event::Pointer(fret_core::PointerEvent::Down {
            position: Point::new(Px(10.0), Px(130.0)),
            button: fret_core::MouseButton::Left,
            modifiers: fret_core::Modifiers::default(),
            click_count: 1,
            pointer_id: PointerId(0),
            pointer_type: fret_core::PointerType::Mouse,
        }),
    );
    assert!(
        ui.captured().is_some(),
        "expected pressable pointer down to capture"
    );

    begin_frame(&mut app, window);
    let (_trigger2, _underlay2) = render_base_with_trigger_and_underlay(
        &mut ui,
        &mut app,
        &mut services,
        window,
        bounds,
        open.clone(),
        underlay_clicked.clone(),
    );
    request_hover_overlay_for_window(
        &mut app,
        window,
        HoverOverlayRequest {
            id: trigger,
            root_name: hover_overlay_root_name(trigger),
            interactive: true,
            trigger,
            children: Vec::new(),
        },
    );
    render(&mut ui, &mut app, &mut services, window, bounds);

    assert!(
        !ui.is_layer_visible(layer),
        "expected hover overlay to be hidden during pointer capture"
    );
}

#[test]
fn pointer_capture_restores_hover_overlays_after_release() {
    let window = AppWindowId::default();
    let mut app = App::new();
    let mut ui: UiTree<App> = UiTree::new();
    ui.set_window(window);

    let open = app.models_mut().insert(false);
    let underlay_clicked = app.models_mut().insert(false);

    let mut services = FakeServices;
    let bounds = Rect::new(
        Point::new(Px(0.0), Px(0.0)),
        fret_core::Size::new(Px(300.0), Px(200.0)),
    );

    let (trigger, _underlay) = render_base_with_trigger_and_underlay(
        &mut ui,
        &mut app,
        &mut services,
        window,
        bounds,
        open.clone(),
        underlay_clicked.clone(),
    );
    ui.layout_all(&mut app, &mut services, bounds, 1.0);

    request_hover_overlay_for_window(
        &mut app,
        window,
        HoverOverlayRequest {
            id: trigger,
            root_name: hover_overlay_root_name(trigger),
            interactive: true,
            trigger,
            children: Vec::new(),
        },
    );
    render(&mut ui, &mut app, &mut services, window, bounds);

    let layer = app.with_global_mut_untracked(WindowOverlays::default, |overlays, _app| {
        overlays
            .hover_overlays
            .get(&(window, trigger))
            .map(|h| h.layer)
    });
    let layer = layer.expect("hover overlay layer");
    assert!(ui.is_layer_visible(layer));

    ui.dispatch_event(
        &mut app,
        &mut services,
        &fret_core::Event::Pointer(fret_core::PointerEvent::Down {
            position: Point::new(Px(10.0), Px(130.0)),
            button: fret_core::MouseButton::Left,
            modifiers: fret_core::Modifiers::default(),
            click_count: 1,
            pointer_id: PointerId(0),
            pointer_type: fret_core::PointerType::Mouse,
        }),
    );
    assert!(
        ui.captured().is_some(),
        "expected pressable pointer down to capture"
    );

    begin_frame(&mut app, window);
    let (_trigger2, _underlay2) = render_base_with_trigger_and_underlay(
        &mut ui,
        &mut app,
        &mut services,
        window,
        bounds,
        open.clone(),
        underlay_clicked.clone(),
    );
    request_hover_overlay_for_window(
        &mut app,
        window,
        HoverOverlayRequest {
            id: trigger,
            root_name: hover_overlay_root_name(trigger),
            interactive: true,
            trigger,
            children: Vec::new(),
        },
    );
    render(&mut ui, &mut app, &mut services, window, bounds);
    assert!(
        !ui.is_layer_visible(layer),
        "expected hover overlay to be hidden during pointer capture"
    );

    ui.dispatch_event(
        &mut app,
        &mut services,
        &fret_core::Event::Pointer(fret_core::PointerEvent::Up {
            pointer_id: fret_core::PointerId(0),
            position: Point::new(Px(10.0), Px(130.0)),
            button: fret_core::MouseButton::Left,
            modifiers: fret_core::Modifiers::default(),
            is_click: false,
            pointer_type: fret_core::PointerType::Mouse,
            click_count: 1,
        }),
    );
    assert!(
        ui.captured().is_none(),
        "expected pointer capture to be released on pointer up"
    );

    begin_frame(&mut app, window);
    let (_trigger3, _underlay3) = render_base_with_trigger_and_underlay(
        &mut ui,
        &mut app,
        &mut services,
        window,
        bounds,
        open,
        underlay_clicked,
    );
    request_hover_overlay_for_window(
        &mut app,
        window,
        HoverOverlayRequest {
            id: trigger,
            root_name: hover_overlay_root_name(trigger),
            interactive: true,
            trigger,
            children: Vec::new(),
        },
    );
    render(&mut ui, &mut app, &mut services, window, bounds);
    assert!(
        ui.is_layer_visible(layer),
        "expected hover overlay to become visible again after capture release"
    );
}

#[test]
fn pointer_capture_hides_tooltips_in_same_window() {
    let window = AppWindowId::default();
    let mut app = App::new();
    let mut ui: UiTree<App> = UiTree::new();
    ui.set_window(window);

    let open = app.models_mut().insert(false);
    let underlay_clicked = app.models_mut().insert(false);

    let mut services = FakeServices;
    let bounds = Rect::new(
        Point::new(Px(0.0), Px(0.0)),
        fret_core::Size::new(Px(300.0), Px(200.0)),
    );

    let (trigger, _underlay) = render_base_with_trigger_and_underlay(
        &mut ui,
        &mut app,
        &mut services,
        window,
        bounds,
        open.clone(),
        underlay_clicked.clone(),
    );
    ui.layout_all(&mut app, &mut services, bounds, 1.0);

    request_tooltip_for_window(
        &mut app,
        window,
        TooltipRequest {
            id: trigger,
            root_name: tooltip_root_name(trigger),
            interactive: true,
            trigger: Some(trigger),
            on_dismiss_request: None,
            on_pointer_move: None,
            children: Vec::new(),
        },
    );
    render(&mut ui, &mut app, &mut services, window, bounds);

    let layer = app.with_global_mut_untracked(WindowOverlays::default, |overlays, _app| {
        overlays.tooltips.get(&(window, trigger)).map(|t| t.layer)
    });
    let layer = layer.expect("tooltip layer");
    assert!(ui.is_layer_visible(layer));

    ui.dispatch_event(
        &mut app,
        &mut services,
        &fret_core::Event::Pointer(fret_core::PointerEvent::Down {
            position: Point::new(Px(10.0), Px(130.0)),
            button: fret_core::MouseButton::Left,
            modifiers: fret_core::Modifiers::default(),
            click_count: 1,
            pointer_id: PointerId(0),
            pointer_type: fret_core::PointerType::Mouse,
        }),
    );
    assert!(
        ui.captured().is_some(),
        "expected pressable pointer down to capture"
    );

    begin_frame(&mut app, window);
    let (_trigger2, _underlay2) = render_base_with_trigger_and_underlay(
        &mut ui,
        &mut app,
        &mut services,
        window,
        bounds,
        open.clone(),
        underlay_clicked.clone(),
    );
    request_tooltip_for_window(
        &mut app,
        window,
        TooltipRequest {
            id: trigger,
            root_name: tooltip_root_name(trigger),
            interactive: true,
            trigger: Some(trigger),
            on_dismiss_request: None,
            on_pointer_move: None,
            children: Vec::new(),
        },
    );
    render(&mut ui, &mut app, &mut services, window, bounds);

    assert!(
        !ui.is_layer_visible(layer),
        "expected tooltip to be hidden during pointer capture"
    );
}

#[test]
fn pointer_capture_restores_tooltips_after_release() {
    let window = AppWindowId::default();
    let mut app = App::new();
    let mut ui: UiTree<App> = UiTree::new();
    ui.set_window(window);

    let open = app.models_mut().insert(false);
    let underlay_clicked = app.models_mut().insert(false);

    let mut services = FakeServices;
    let bounds = Rect::new(
        Point::new(Px(0.0), Px(0.0)),
        fret_core::Size::new(Px(300.0), Px(200.0)),
    );

    let (trigger, _underlay) = render_base_with_trigger_and_underlay(
        &mut ui,
        &mut app,
        &mut services,
        window,
        bounds,
        open.clone(),
        underlay_clicked.clone(),
    );
    ui.layout_all(&mut app, &mut services, bounds, 1.0);

    request_tooltip_for_window(
        &mut app,
        window,
        TooltipRequest {
            id: trigger,
            root_name: tooltip_root_name(trigger),
            interactive: true,
            trigger: Some(trigger),
            on_dismiss_request: None,
            on_pointer_move: None,
            children: Vec::new(),
        },
    );
    render(&mut ui, &mut app, &mut services, window, bounds);

    let layer = app.with_global_mut_untracked(WindowOverlays::default, |overlays, _app| {
        overlays.tooltips.get(&(window, trigger)).map(|t| t.layer)
    });
    let layer = layer.expect("tooltip layer");
    assert!(ui.is_layer_visible(layer));

    ui.dispatch_event(
        &mut app,
        &mut services,
        &fret_core::Event::Pointer(fret_core::PointerEvent::Down {
            position: Point::new(Px(10.0), Px(130.0)),
            button: fret_core::MouseButton::Left,
            modifiers: fret_core::Modifiers::default(),
            click_count: 1,
            pointer_id: PointerId(0),
            pointer_type: fret_core::PointerType::Mouse,
        }),
    );
    assert!(
        ui.captured().is_some(),
        "expected pressable pointer down to capture"
    );

    begin_frame(&mut app, window);
    let (_trigger2, _underlay2) = render_base_with_trigger_and_underlay(
        &mut ui,
        &mut app,
        &mut services,
        window,
        bounds,
        open.clone(),
        underlay_clicked.clone(),
    );
    request_tooltip_for_window(
        &mut app,
        window,
        TooltipRequest {
            id: trigger,
            root_name: tooltip_root_name(trigger),
            interactive: true,
            trigger: Some(trigger),
            on_dismiss_request: None,
            on_pointer_move: None,
            children: Vec::new(),
        },
    );
    render(&mut ui, &mut app, &mut services, window, bounds);
    assert!(
        !ui.is_layer_visible(layer),
        "expected tooltip to be hidden during pointer capture"
    );

    ui.dispatch_event(
        &mut app,
        &mut services,
        &fret_core::Event::Pointer(fret_core::PointerEvent::Up {
            pointer_id: fret_core::PointerId(0),
            position: Point::new(Px(10.0), Px(130.0)),
            button: fret_core::MouseButton::Left,
            modifiers: fret_core::Modifiers::default(),
            is_click: false,
            pointer_type: fret_core::PointerType::Mouse,
            click_count: 1,
        }),
    );
    assert!(
        ui.captured().is_none(),
        "expected pointer capture to be released on pointer up"
    );

    begin_frame(&mut app, window);
    let (_trigger3, _underlay3) = render_base_with_trigger_and_underlay(
        &mut ui,
        &mut app,
        &mut services,
        window,
        bounds,
        open,
        underlay_clicked,
    );
    request_tooltip_for_window(
        &mut app,
        window,
        TooltipRequest {
            id: trigger,
            root_name: tooltip_root_name(trigger),
            interactive: true,
            trigger: Some(trigger),
            on_dismiss_request: None,
            on_pointer_move: None,
            children: Vec::new(),
        },
    );
    render(&mut ui, &mut app, &mut services, window, bounds);
    assert!(
        ui.is_layer_visible(layer),
        "expected tooltip to become visible again after capture release"
    );
}

#[test]
fn viewport_capture_hides_hover_overlays_and_restores_after_release() {
    let window = AppWindowId::default();
    let mut app = App::new();
    let mut ui: UiTree<App> = UiTree::new();
    ui.set_window(window);

    let open = app.models_mut().insert(false);

    let mut services = FakeServices;
    let bounds = Rect::new(
        Point::new(Px(0.0), Px(0.0)),
        fret_core::Size::new(Px(300.0), Px(200.0)),
    );

    // First frame: render base so we can request a hover overlay above it.
    let (trigger, _underlay) = render_base_with_trigger_and_capture_underlay(
        &mut ui,
        &mut app,
        &mut services,
        window,
        bounds,
        open.clone(),
    );
    render(&mut ui, &mut app, &mut services, window, bounds);
    ui.layout_all(&mut app, &mut services, bounds, 1.0);

    request_hover_overlay_for_window(
        &mut app,
        window,
        HoverOverlayRequest {
            id: trigger,
            root_name: hover_overlay_root_name(trigger),
            interactive: true,
            trigger,
            children: Vec::new(),
        },
    );
    render(&mut ui, &mut app, &mut services, window, bounds);

    let layer = app.with_global_mut_untracked(WindowOverlays::default, |overlays, _app| {
        overlays
            .hover_overlays
            .get(&(window, trigger))
            .map(|h| h.layer)
    });
    let layer = layer.expect("hover overlay layer");
    assert!(ui.is_layer_visible(layer));

    // Start a viewport-like pointer capture by pressing the underlay pointer region.
    ui.dispatch_event(
        &mut app,
        &mut services,
        &Event::Pointer(fret_core::PointerEvent::Down {
            pointer_id: fret_core::PointerId(0),
            position: Point::new(Px(10.0), Px(130.0)),
            button: fret_core::MouseButton::Left,
            modifiers: fret_core::Modifiers::default(),
            pointer_type: fret_core::PointerType::Mouse,
            click_count: 1,
        }),
    );
    assert!(
        ui.captured().is_some(),
        "expected underlay to capture the pointer"
    );

    begin_frame(&mut app, window);
    let _ = render_base_with_trigger_and_capture_underlay(
        &mut ui,
        &mut app,
        &mut services,
        window,
        bounds,
        open.clone(),
    );
    request_hover_overlay_for_window(
        &mut app,
        window,
        HoverOverlayRequest {
            id: trigger,
            root_name: hover_overlay_root_name(trigger),
            interactive: true,
            trigger,
            children: Vec::new(),
        },
    );
    render(&mut ui, &mut app, &mut services, window, bounds);

    assert!(
        !ui.is_layer_visible(layer),
        "expected hover overlay to be hidden during viewport pointer capture"
    );

    ui.dispatch_event(
        &mut app,
        &mut services,
        &Event::Pointer(fret_core::PointerEvent::Up {
            pointer_id: fret_core::PointerId(0),
            position: Point::new(Px(10.0), Px(130.0)),
            button: fret_core::MouseButton::Left,
            modifiers: fret_core::Modifiers::default(),
            is_click: false,
            pointer_type: fret_core::PointerType::Mouse,
            click_count: 1,
        }),
    );
    assert!(
        ui.captured().is_none(),
        "expected capture to be released on pointer up"
    );

    begin_frame(&mut app, window);
    let _ = render_base_with_trigger_and_capture_underlay(
        &mut ui,
        &mut app,
        &mut services,
        window,
        bounds,
        open,
    );
    request_hover_overlay_for_window(
        &mut app,
        window,
        HoverOverlayRequest {
            id: trigger,
            root_name: hover_overlay_root_name(trigger),
            interactive: true,
            trigger,
            children: Vec::new(),
        },
    );
    render(&mut ui, &mut app, &mut services, window, bounds);

    assert!(
        ui.is_layer_visible(layer),
        "expected hover overlay to become visible again after capture release"
    );
}

#[test]
fn viewport_capture_hides_tooltips_and_restores_after_release() {
    let window = AppWindowId::default();
    let mut app = App::new();
    let mut ui: UiTree<App> = UiTree::new();
    ui.set_window(window);

    let open = app.models_mut().insert(false);

    let mut services = FakeServices;
    let bounds = Rect::new(
        Point::new(Px(0.0), Px(0.0)),
        fret_core::Size::new(Px(300.0), Px(200.0)),
    );

    let (trigger, _underlay) = render_base_with_trigger_and_capture_underlay(
        &mut ui,
        &mut app,
        &mut services,
        window,
        bounds,
        open.clone(),
    );
    render(&mut ui, &mut app, &mut services, window, bounds);
    ui.layout_all(&mut app, &mut services, bounds, 1.0);

    request_tooltip_for_window(
        &mut app,
        window,
        TooltipRequest {
            id: trigger,
            root_name: tooltip_root_name(trigger),
            interactive: true,
            trigger: Some(trigger),
            on_dismiss_request: None,
            on_pointer_move: None,
            children: Vec::new(),
        },
    );
    render(&mut ui, &mut app, &mut services, window, bounds);

    let layer = app.with_global_mut_untracked(WindowOverlays::default, |overlays, _app| {
        overlays.tooltips.get(&(window, trigger)).map(|t| t.layer)
    });
    let layer = layer.expect("tooltip layer");
    assert!(ui.is_layer_visible(layer));

    // Start a viewport-like pointer capture by pressing the underlay.
    ui.dispatch_event(
        &mut app,
        &mut services,
        &Event::Pointer(fret_core::PointerEvent::Down {
            pointer_id: fret_core::PointerId(0),
            position: Point::new(Px(10.0), Px(130.0)),
            button: fret_core::MouseButton::Left,
            modifiers: fret_core::Modifiers::default(),
            pointer_type: fret_core::PointerType::Mouse,
            click_count: 1,
        }),
    );
    assert!(
        ui.captured().is_some(),
        "expected underlay to capture the pointer"
    );

    begin_frame(&mut app, window);
    let _ = render_base_with_trigger_and_capture_underlay(
        &mut ui,
        &mut app,
        &mut services,
        window,
        bounds,
        open.clone(),
    );
    request_tooltip_for_window(
        &mut app,
        window,
        TooltipRequest {
            id: trigger,
            root_name: tooltip_root_name(trigger),
            interactive: true,
            trigger: Some(trigger),
            on_dismiss_request: None,
            on_pointer_move: None,
            children: Vec::new(),
        },
    );
    render(&mut ui, &mut app, &mut services, window, bounds);
    assert!(
        !ui.is_layer_visible(layer),
        "expected tooltip to be hidden during viewport pointer capture"
    );

    ui.dispatch_event(
        &mut app,
        &mut services,
        &Event::Pointer(fret_core::PointerEvent::Up {
            pointer_id: fret_core::PointerId(0),
            position: Point::new(Px(10.0), Px(130.0)),
            button: fret_core::MouseButton::Left,
            modifiers: fret_core::Modifiers::default(),
            is_click: false,
            pointer_type: fret_core::PointerType::Mouse,
            click_count: 1,
        }),
    );
    assert!(
        ui.captured().is_none(),
        "expected capture to be released on pointer up"
    );

    begin_frame(&mut app, window);
    let _ = render_base_with_trigger_and_capture_underlay(
        &mut ui,
        &mut app,
        &mut services,
        window,
        bounds,
        open,
    );
    request_tooltip_for_window(
        &mut app,
        window,
        TooltipRequest {
            id: trigger,
            root_name: tooltip_root_name(trigger),
            interactive: true,
            trigger: Some(trigger),
            on_dismiss_request: None,
            on_pointer_move: None,
            children: Vec::new(),
        },
    );
    render(&mut ui, &mut app, &mut services, window, bounds);
    assert!(
        ui.is_layer_visible(layer),
        "expected tooltip to become visible again after capture release"
    );
}
