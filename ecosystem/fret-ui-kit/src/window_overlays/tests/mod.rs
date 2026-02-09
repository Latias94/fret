use super::state::{OVERLAY_CACHE_TTL_FRAMES, WindowOverlays};
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

mod hover;
mod pointer_capture;
mod tooltip;
mod viewport_capture;

mod cached_requests;
mod dismissible_popover;
mod dock_drag;
mod modal;
mod non_modal_overlay;
mod toast;

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

fn render_base_with_compound_trigger(
    ui: &mut UiTree<App>,
    app: &mut App,
    services: &mut dyn fret_core::UiServices,
    window: AppWindowId,
    bounds: Rect,
    open: Model<bool>,
) -> (GlobalElementId, GlobalElementId, GlobalElementId) {
    begin_frame(app, window);

    let mut field_id: Option<GlobalElementId> = None;
    let mut input_trigger_id: Option<GlobalElementId> = None;
    let mut trailing_icon_id: Option<GlobalElementId> = None;

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
                let field = cx.container(
                    ContainerProps {
                        layout: LayoutStyle {
                            position: PositionStyle::Absolute,
                            inset: InsetStyle {
                                left: Some(Px(0.0)),
                                top: Some(Px(0.0)),
                                ..Default::default()
                            },
                            size: SizeStyle {
                                width: Length::Px(Px(160.0)),
                                height: Length::Px(Px(32.0)),
                                ..Default::default()
                            },
                            ..Default::default()
                        },
                        ..Default::default()
                    },
                    |cx| {
                        vec![
                            cx.pressable_with_id(
                                PressableProps {
                                    layout: LayoutStyle {
                                        position: PositionStyle::Absolute,
                                        inset: InsetStyle {
                                            left: Some(Px(0.0)),
                                            top: Some(Px(0.0)),
                                            ..Default::default()
                                        },
                                        size: SizeStyle {
                                            width: Length::Px(Px(112.0)),
                                            height: Length::Px(Px(32.0)),
                                            ..Default::default()
                                        },
                                        ..Default::default()
                                    },
                                    enabled: true,
                                    focusable: true,
                                    ..Default::default()
                                },
                                |cx, _st, id| {
                                    cx.pressable_toggle_bool(&open);
                                    input_trigger_id = Some(id);
                                    Vec::new()
                                },
                            ),
                            cx.pressable_with_id(
                                PressableProps {
                                    layout: LayoutStyle {
                                        position: PositionStyle::Absolute,
                                        inset: InsetStyle {
                                            left: Some(Px(112.0)),
                                            top: Some(Px(0.0)),
                                            ..Default::default()
                                        },
                                        size: SizeStyle {
                                            width: Length::Px(Px(48.0)),
                                            height: Length::Px(Px(32.0)),
                                            ..Default::default()
                                        },
                                        ..Default::default()
                                    },
                                    enabled: true,
                                    focusable: true,
                                    ..Default::default()
                                },
                                |cx, _st, id| {
                                    cx.pressable_toggle_bool(&open);
                                    trailing_icon_id = Some(id);
                                    Vec::new()
                                },
                            ),
                        ]
                    },
                );
                field_id = Some(field.id);
                vec![field]
            },
        )]
    });
    ui.set_root(root);

    (
        field_id.expect("field id"),
        input_trigger_id.expect("input trigger id"),
        trailing_icon_id.expect("trailing icon id"),
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
