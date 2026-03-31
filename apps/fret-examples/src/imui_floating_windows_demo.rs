use std::sync::Arc;

use fret::{FretApp, advanced::prelude::*};
use fret_core::{Point, Px, Rect, SemanticsRole, Size};
use fret_imui::prelude::UiWriter;
use fret_ui_kit::on_activate_notify;

struct ImUiFloatingWindowsView;

pub fn run() -> anyhow::Result<()> {
    FretApp::new("imui-floating-windows-demo")
        .window("imui_floating_windows_demo", (720.0, 480.0))
        .view::<ImUiFloatingWindowsView>()?
        .run()?;
    Ok(())
}

impl View for ImUiFloatingWindowsView {
    fn init(_app: &mut KernelApp, _window: AppWindowId) -> Self {
        Self
    }

    fn render(&mut self, cx: &mut AppUi<'_, '_>) -> Ui {
        let open_a_state = cx.state().local_init(|| true);
        let select_mode_state = cx.state().local_init(|| None::<Arc<str>>);
        let a_overlap_clicked_state = cx.state().local_init(|| false);

        fret_imui::imui_vstack(cx.elements(), |ui| {
            use fret_ui_kit::imui::UiWriterImUiFacadeExt as _;
            use fret_ui_kit::imui::UiWriterUiKitExt as _;

            let title = fret_ui_kit::ui::text("imui floating windows demo (diagnostics)")
                .text_sm()
                .font_semibold();
            ui.add_ui(title);

            let hint = fret_ui_kit::ui::text(
                "Double-click title to collapse/expand, resize from the corner, open the context menu, and test the select popup.",
            )
            .text_xs();
            ui.add_ui(hint);

            ui.separator();

            let drop_zone = ui.with_cx_mut(|cx| {
                let mut props = fret_ui::element::ContainerProps::default();
                props.layout.position = fret_ui::element::PositionStyle::Absolute;
                props.layout.inset = fret_ui::element::InsetStyle {
                    left: Some(Px(480.0)).into(),
                    top: Some(Px(220.0)).into(),
                    ..Default::default()
                };
                props.layout.size.width = fret_ui::element::Length::Px(Px(160.0));
                props.layout.size.height = fret_ui::element::Length::Px(Px(120.0));
                props.background = Some(fret_ui::Theme::global(&*cx.app).color_token("muted"));
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
                let id = "a";
                let initial_position = Point::new(Px(24.0), Px(96.0));
                let initial_size = Size::new(Px(220.0), Px(160.0));

                let resp = ui.window_with_options(
                    id,
                    "Window A",
                    initial_position,
                    fret_ui_kit::imui::WindowOptions::default()
                        .with_open(open_a_state.model())
                        .with_size(initial_size)
                        .with_resize(fret_ui_kit::imui::FloatingWindowResizeOptions::default()),
                    |ui| {
                        ui.vertical(|ui| {
                            ui.mount(|cx| {
                                let clicked_model = a_overlap_clicked_state.clone_model();

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
                                        cx.pressable_on_activate(on_activate_notify(move |host| {
                                            let _ = host
                                                .models_mut()
                                                .update(&clicked_model, |v| *v = true);
                                        }));
                                        vec![cx.text("Overlap target (A)")]
                                    },
                                );

                                let clicked = a_overlap_clicked_state.paint_value_in(cx);

                                let clicked_label = clicked.then(|| {
                                    cx.text("A overlap clicked").attach_semantics(
                                        fret_ui::element::SemanticsDecoration::default().test_id(
                                            Arc::from("imui-float-demo.a.overlap.clicked"),
                                        ),
                                    )
                                });

                                let row = cx.row(
                                    {
                                        let mut props = fret_ui::element::RowProps::default();
                                        props.layout.size.width = fret_ui::element::Length::Fill;
                                        props.gap = fret_ui::element::SpacingLength::Px(Px(8.0));
                                        props
                                    },
                                    move |_cx| vec![activate, overlap],
                                );

                                let mut out = vec![
                                    row,
                                    cx.text("Click 'Activate A' to bring Window A to front.")
                                        .attach_semantics(
                                            fret_ui::element::SemanticsDecoration::default()
                                                .test_id(Arc::from(
                                                    "imui-float-demo.a.activate.hint",
                                                )),
                                        ),
                                ];
                                if let Some(clicked_label) = clicked_label {
                                    out.push(clicked_label);
                                }
                                vec![cx.column(
                                    {
                                        let mut props = fret_ui::element::ColumnProps::default();
                                        props.layout.size.width = fret_ui::element::Length::Fill;
                                        props.gap = fret_ui::element::SpacingLength::Px(Px(8.0));
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
                                let _ = ui.menu_item_with_options(
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
                            let _ = ui.combo_model_with_options(
                                "imui-float-demo.select.popup",
                                "Mode",
                                select_mode_state.model(),
                                &select_items,
                                fret_ui_kit::imui::ComboModelOptions {
                                    test_id: Some(Arc::from("imui-float-demo.select")),
                                    ..Default::default()
                                },
                            );
                        });
                    },
                );

                let _ = ui.window_with_options(
                    "b",
                    "Window B",
                    Point::new(Px(176.0), Px(116.0)),
                    fret_ui_kit::imui::WindowOptions::default()
                        .with_size(Size::new(Px(180.0), Px(120.0)))
                        .with_resize(fret_ui_kit::imui::FloatingWindowResizeOptions::default()),
                    |ui| {
                        ui.vertical(|ui| {
                            ui.text("This window starts on top of A's overlap target.");
                            ui.separator();
                            ui.text("Then click A's 'Activate' button.");
                        });
                    },
                );

                let _ = (
                    resp.area.id,
                    resp.area.position,
                    resp.area.rect,
                    resp.size,
                    resp.resizing,
                    resp.collapsed,
                );
            });

            let _root_bounds: Rect = ui.cx_mut().bounds;
        })
    }
}
