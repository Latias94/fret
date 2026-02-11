use std::sync::Arc;

use fret_core::{Point, Px, Rect, SemanticsRole, Size};
use fret_imui::prelude::UiWriter;
use fret_kit::prelude::*;

struct ImUiFloatingWindowsState {
    open_a: Model<bool>,
    select_mode: Model<Option<Arc<str>>>,
    a_overlap_clicked: Model<bool>,
}

pub fn run() -> anyhow::Result<()> {
    fret_kit::app("imui-floating-windows-demo", init_window, view)?
        .with_main_window("imui_floating_windows_demo", (720.0, 480.0))
        .run()?;
    Ok(())
}

fn init_window(app: &mut App, _window: AppWindowId) -> ImUiFloatingWindowsState {
    ImUiFloatingWindowsState {
        open_a: app.models_mut().insert(true),
        select_mode: app.models_mut().insert(None::<Arc<str>>),
        a_overlap_clicked: app.models_mut().insert(false),
    }
}

fn view(cx: &mut ElementContext<'_, App>, st: &mut ImUiFloatingWindowsState) -> ViewElements {
    // Use a column root to avoid the "all children overlap at (0,0)" footgun when emitting multiple
    // immediate-mode elements at the top level.
    fret_imui::imui_vstack(cx, |ui| {
        use fret_ui_kit::imui::UiWriterImUiFacadeExt as _;
        use fret_ui_kit::imui::UiWriterUiKitExt as _;

        let title = fret_ui_kit::ui::text(ui.cx_mut(), "imui floating windows demo (diagnostics)")
            .text_sm()
            .font_semibold();
        ui.add_ui(title);

        let hint = fret_ui_kit::ui::text(
            ui.cx_mut(),
            "Drag the title bar onto the drop zone, double-click title to collapse/expand, resize from the corner, open the context menu, and test the select popup.",
        )
        .text_xs();
        ui.add_ui(hint);

        ui.separator();

        let drop_zone = ui.with_cx_mut(|cx| {
            let mut props = fret_ui::element::ContainerProps::default();
            props.layout.position = fret_ui::element::PositionStyle::Absolute;
            props.layout.inset = fret_ui::element::InsetStyle {
                left: Some(Px(480.0)),
                top: Some(Px(220.0)),
                ..Default::default()
            };
            props.layout.size.width = fret_ui::element::Length::Px(Px(160.0));
            props.layout.size.height = fret_ui::element::Length::Px(Px(120.0));
            props.background = Some(fret_ui::Theme::global(&*cx.app).color_required("muted"));
            let zone = cx.container(props, |cx| {
                vec![
                    cx.text("Drop zone").attach_semantics(
                        fret_ui::element::SemanticsDecoration::default()
                            .test_id(Arc::from("imui-float-demo.drop-zone.label")),
                    ),
                ]
            });
            zone.attach_semantics(
                fret_ui::element::SemanticsDecoration::default()
                    .test_id(Arc::from("imui-float-demo.drop-zone")),
            )
        });
        ui.add(drop_zone);

        ui.floating_layer("imui-float-demo.floating-layer", |ui| {
            // Window A: used by diag scripts for drag/resize/context-menu overlay coexistence.
            let id = "a";
            let initial_position = Point::new(Px(24.0), Px(96.0));
            let initial_size = Size::new(Px(220.0), Px(160.0));

            let resp = ui.window_open_resizable(
                id,
                "Window A",
                &st.open_a,
                initial_position,
                initial_size,
                |ui| {
                    ui.vertical(|ui| {
                        ui.mount(|cx| {
                            let clicked_model = st.a_overlap_clicked.clone();

                            let activate = cx.pressable(
                                {
                                    let mut props = fret_ui::element::PressableProps::default();
                                    props.a11y = fret_ui::element::PressableA11y {
                                        role: Some(SemanticsRole::Button),
                                        label: Some(Arc::from("Activate A (content)")),
                                        test_id: Some(Arc::from("imui-float-demo.a.activate")),
                                        ..Default::default()
                                    };
                                    props
                                },
                                |cx, _state| vec![cx.text("Activate A (content)")],
                            );

                            let overlap = cx.pressable(
                                {
                                    let mut props = fret_ui::element::PressableProps::default();
                                    props.a11y = fret_ui::element::PressableA11y {
                                        role: Some(SemanticsRole::Button),
                                        label: Some(Arc::from("Overlap target (A)")),
                                        test_id: Some(Arc::from("imui-float-demo.a.overlap")),
                                        ..Default::default()
                                    };
                                    props
                                },
                                move |cx, _state| {
                                    cx.pressable_on_activate(Arc::new(
                                        move |host, acx, _reason| {
                                            let _ = host
                                                .models_mut()
                                                .update(&clicked_model, |v| *v = true);
                                            host.notify(acx);
                                        },
                                    ));
                                    vec![cx.text("Overlap target (A)")]
                                },
                            );

                            let clicked = cx
                                .read_model(
                                    &st.a_overlap_clicked,
                                    fret_ui::Invalidation::Paint,
                                    |_app, v| *v,
                                )
                                .unwrap_or(false);

                            let clicked_label = clicked.then(|| {
                                cx.text("A overlap clicked").attach_semantics(
                                    fret_ui::element::SemanticsDecoration::default()
                                        .test_id(Arc::from("imui-float-demo.a.overlap.clicked")),
                                )
                            });

                            let row = cx.row(
                                {
                                    let mut props = fret_ui::element::RowProps::default();
                                    props.layout.size.width = fret_ui::element::Length::Fill;
                                    props.gap = Px(8.0);
                                    props
                                },
                                move |_cx| vec![activate, overlap],
                            );

                            let mut out = vec![
                                row,
                                cx.text("Click 'Activate A' to bring Window A to front.")
                                    .attach_semantics(
                                        fret_ui::element::SemanticsDecoration::default()
                                            .test_id(Arc::from("imui-float-demo.a.activate.hint")),
                                    ),
                            ];
                            if let Some(clicked_label) = clicked_label {
                                out.push(clicked_label);
                            }
                            vec![cx.column(
                                {
                                    let mut props = fret_ui::element::ColumnProps::default();
                                    props.layout.size.width = fret_ui::element::Length::Fill;
                                    props.gap = Px(8.0);
                                    props
                                },
                                move |_cx| out,
                            )]
                        });

                        ui.separator();
                        ui.text("Right click the button to open a context menu.");
                        let trigger = ui.button("Context menu (right click)");
                        ui.begin_popup_context_menu("ctx", trigger, |ui| {
                            let open = ui.popup_open_model("ctx");
                            let _ = ui.menu_item_ex(
                                "Close menu",
                                fret_ui_kit::imui::MenuItemOptions {
                                    close_popup: Some(open),
                                    test_id: Some(Arc::from("imui-float-demo.ctx.close")),
                                    ..Default::default()
                                },
                            );
                        });

                        ui.separator();
                        let select_items = [
                            Arc::<str>::from("Alpha"),
                            Arc::<str>::from("Beta"),
                            Arc::<str>::from("Gamma"),
                        ];
                        let _ = ui.select_model_ex(
                            "Mode",
                            &st.select_mode,
                            &select_items,
                            fret_ui_kit::imui::SelectOptions {
                                test_id: Some(Arc::from("imui-float-demo.select")),
                                ..Default::default()
                            },
                        );
                    });
                },
            );

            // Window B: used by v3 diag scripts to assert content-click activation/z-order.
            let _ = ui.window_resizable(
                "b",
                "Window B",
                Point::new(Px(176.0), Px(116.0)),
                Size::new(Px(180.0), Px(120.0)),
                |ui| {
                    ui.vertical(|ui| {
                        ui.text("This window starts on top of A's overlap target.");
                        ui.separator();
                        ui.text("Then click A's 'Activate' button.");
                    });
                },
            );

            // Expose a stable rect anchor used by scripts for overlap assertions.
            let _ = (
                resp.area.id,
                resp.area.position,
                resp.area.rect,
                resp.size,
                resp.resizing,
                resp.collapsed,
            );
        });

        // Keep the root scoped rect available for future scripted selection by bounds.
        let _root_bounds: Rect = ui.cx_mut().bounds;
    })
}
