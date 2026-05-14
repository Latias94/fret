use super::*;

#[derive(Debug, Clone, Copy)]
pub(crate) enum FloatingLayerOverlayVariant {
    Menu,
    Popover,
}

pub(crate) fn window_behavior_options(behavior: FloatingWindowOptions) -> WindowOptions {
    WindowOptions::default().with_behavior(behavior)
}

pub(crate) fn resizable_window_options(size: Size) -> WindowOptions {
    WindowOptions::default()
        .with_size(size)
        .with_resize(FloatingWindowResizeOptions::default())
}

pub(crate) fn resizable_window_options_with_behavior(
    size: Size,
    behavior: FloatingWindowOptions,
) -> WindowOptions {
    resizable_window_options(size).with_behavior(behavior)
}

pub(crate) fn open_window_options(open: &fret_runtime::Model<bool>) -> WindowOptions {
    WindowOptions::default().with_open(open)
}

pub(crate) fn open_window_options_with_behavior(
    open: &fret_runtime::Model<bool>,
    behavior: FloatingWindowOptions,
) -> WindowOptions {
    WindowOptions::default()
        .with_open(open)
        .with_behavior(behavior)
}

pub(crate) fn render_floating_layer_with_overlay(
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
