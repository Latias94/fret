use fret_app::App;
use fret_core::{
    AppWindowId, Event, FrameId, Modifiers, PathCommand, PathConstraints, PathId, PathMetrics,
    PathService, PathStyle, Point, PointerEvent, PointerId, PointerType, Px, Rect, SemanticsRole,
    Size as CoreSize, SvgId, SvgService, TextBlobId, TextConstraints, TextMetrics, TextService,
};
use fret_runtime::Model;
use fret_ui::ElementContext;
use fret_ui::element::{ContainerProps, LayoutStyle, Length};
use fret_ui::scroll::ScrollHandle;
use fret_ui::tree::UiTree;
use fret_ui_kit::OverlayController;
use fret_ui_shadcn::facade::{self as shadcn, Combobox, ComboboxItem, themes as shadcn_themes};
use std::sync::Arc;

struct FakeServices;

impl TextService for FakeServices {
    fn prepare(
        &mut self,
        _input: &fret_core::TextInput,
        _constraints: TextConstraints,
    ) -> (TextBlobId, TextMetrics) {
        (
            TextBlobId::default(),
            TextMetrics {
                size: CoreSize::new(Px(0.0), Px(0.0)),
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

impl fret_core::MaterialService for FakeServices {
    fn register_material(
        &mut self,
        _desc: fret_core::MaterialDescriptor,
    ) -> Result<fret_core::MaterialId, fret_core::MaterialRegistrationError> {
        Ok(fret_core::MaterialId::default())
    }

    fn unregister_material(&mut self, _id: fret_core::MaterialId) -> bool {
        true
    }
}

fn render_frame(
    ui: &mut UiTree<App>,
    app: &mut App,
    services: &mut dyn fret_core::UiServices,
    window: AppWindowId,
    bounds: Rect,
    open: Model<bool>,
    value: Model<Option<Arc<str>>>,
    scroll_handle: ScrollHandle,
    request_semantics: bool,
) {
    let next_frame = FrameId(app.frame_id().0.saturating_add(1));
    app.set_frame_id(next_frame);

    OverlayController::begin_frame(app, window);
    let root = fret_ui::declarative::render_root(
        ui,
        app,
        services,
        window,
        bounds,
        "combobox-responsive-breakpoint",
        move |cx: &mut ElementContext<'_, App>| {
            let open = open.clone();
            let value = value.clone();
            let scroll_handle = scroll_handle.clone();

            let mut scroll_layout = LayoutStyle::default();
            scroll_layout.size.width = Length::Fill;
            scroll_layout.size.height = Length::Fill;
            scroll_layout.overflow = fret_ui::element::Overflow::Clip;

            vec![cx.scroll(
                fret_ui::element::ScrollProps {
                    layout: scroll_layout,
                    axis: fret_ui::element::ScrollAxis::Y,
                    scroll_handle: Some(scroll_handle),
                    ..Default::default()
                },
                move |cx| {
                    vec![cx.container(
                        ContainerProps {
                            layout: {
                                let mut layout = LayoutStyle::default();
                                layout.size.width = Length::Fill;
                                layout.size.height = Length::Px(Px(1200.0));
                                layout
                            },
                            ..Default::default()
                        },
                        move |cx| {
                            let items = (0..40).map(|idx| {
                                let value = Arc::from(format!("value-{idx}"));
                                let label = Arc::from(format!("Label {idx}"));
                                ComboboxItem::new(value, label)
                            });

                            vec![cx.container(
                                ContainerProps {
                                    layout: {
                                        let mut layout = LayoutStyle::default();
                                        layout.position = fret_ui::element::PositionStyle::Absolute;
                                        layout.inset.left = Some(Px(16.0)).into();
                                        layout.inset.top = Some(Px(160.0)).into();
                                        layout.size.width = Length::Px(Px(280.0));
                                        layout
                                    },
                                    ..Default::default()
                                },
                                move |cx| {
                                    vec![
                                        Combobox::new(value, open)
                                            .device_shell_responsive(true)
                                            .a11y_label("Combobox")
                                            .test_id_prefix("combobox-responsive")
                                            .items(items)
                                            .into_element_parts(cx, |_cx| {
                                                vec![shadcn::ComboboxPart::from(
                                                    shadcn::ComboboxInput::new()
                                                        .placeholder("Select an option"),
                                                )]
                                            }),
                                    ]
                                },
                            )]
                        },
                    )]
                },
            )]
        },
    );

    ui.set_root(root);
    OverlayController::render(ui, app, services, window, bounds);
    if request_semantics {
        ui.request_semantics_snapshot();
    }
    ui.layout_all(app, services, bounds, 1.0);
}

fn assert_underlay_scroll_behavior(bounds: Rect, expect_scroll_blocked: bool) {
    let window = AppWindowId::default();
    let mut app = App::new();
    let mut ui: UiTree<App> = UiTree::new();
    ui.set_window(window);
    let mut services = FakeServices;

    shadcn_themes::apply_shadcn_new_york(
        &mut app,
        shadcn_themes::ShadcnBaseColor::Neutral,
        shadcn_themes::ShadcnColorScheme::Light,
    );

    let value: Model<Option<Arc<str>>> = app.models_mut().insert(None);
    let open: Model<bool> = app.models_mut().insert(false);
    let scroll_handle = ScrollHandle::default();

    // Frame 1: mount closed so element ids are stable before opening overlays.
    render_frame(
        &mut ui,
        &mut app,
        &mut services,
        window,
        bounds,
        open.clone(),
        value.clone(),
        scroll_handle.clone(),
        false,
    );

    let _ = app.models_mut().update(&open, |v| *v = true);

    // Frame 2+: open and render twice so any first-frame size estimation settles.
    render_frame(
        &mut ui,
        &mut app,
        &mut services,
        window,
        bounds,
        open.clone(),
        value.clone(),
        scroll_handle.clone(),
        false,
    );
    render_frame(
        &mut ui,
        &mut app,
        &mut services,
        window,
        bounds,
        open.clone(),
        value.clone(),
        scroll_handle.clone(),
        true,
    );

    let snap = ui.semantics_snapshot().expect("semantics snapshot").clone();
    let _ = snap
        .nodes
        .iter()
        .find(|n| n.role == SemanticsRole::ListBox)
        .expect("combobox listbox semantics (open)");

    ui.dispatch_event(
        &mut app,
        &mut services,
        &Event::Pointer(PointerEvent::Wheel {
            pointer_id: PointerId::default(),
            position: Point::new(Px(10.0), Px(10.0)),
            delta: Point::new(Px(0.0), Px(-80.0)),
            modifiers: Modifiers::default(),
            pointer_type: PointerType::Mouse,
        }),
    );

    // Apply the wheel event (if any) to the scroll state.
    render_frame(
        &mut ui,
        &mut app,
        &mut services,
        window,
        bounds,
        open,
        value,
        scroll_handle.clone(),
        false,
    );

    let y = scroll_handle.offset().y.0.abs();
    if expect_scroll_blocked {
        assert!(
            y < 0.01,
            "expected responsive combobox Drawer to block underlay scroll; y={y}",
        );
    } else {
        assert!(
            y > 0.1,
            "expected responsive combobox Popover to allow underlay scroll; y={y}",
        );
    }
}

#[test]
fn combobox_responsive_switches_between_drawer_and_popover_at_md_breakpoint() {
    // Mobile width: Drawer-backed combobox is expected to behave modally.
    assert_underlay_scroll_behavior(
        Rect::new(
            Point::new(Px(0.0), Px(0.0)),
            CoreSize::new(Px(480.0), Px(400.0)),
        ),
        true,
    );

    // Desktop width: Popover-backed combobox is click-through by default (ADR 0069).
    assert_underlay_scroll_behavior(
        Rect::new(
            Point::new(Px(0.0), Px(0.0)),
            CoreSize::new(Px(1200.0), Px(400.0)),
        ),
        false,
    );
}
