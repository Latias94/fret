//! Advanced/reference demo: IMUI response signals proof surface.
//!
//! Why advanced: it keeps the outward response/query contract log explicit for click, drag,
//! context-menu, menu, submenu, tab, and combo helper behavior.
//! Not a first-contact teaching surface: prefer `imui_action_basics` and
//! `imui_editor_proof_demo` for onboarding, then use this file when you need proof-first contract
//! evidence rather than a polished showcase.
//!
//! This is a reference/product-validation surface, but it intentionally stays proof-first instead
//! of showcase-first. Use `imui_interaction_showcase_demo` when you need the presentable shell.

use std::sync::Arc;

use fret::{FretApp, advanced::prelude::*};
use fret_core::{Point, Px, Rect};

struct ImUiResponseSignalsView;

pub fn run() -> anyhow::Result<()> {
    FretApp::new("imui-response-signals-demo")
        .window("imui_response_signals_demo", (720.0, 520.0))
        .view::<ImUiResponseSignalsView>()?
        .run()?;
    Ok(())
}

impl View for ImUiResponseSignalsView {
    fn init(_app: &mut KernelApp, _window: AppWindowId) -> Self {
        Self
    }

    fn render(&mut self, cx: &mut AppUi<'_, '_>) -> Ui {
        let left_clicks = cx.state().local_init(|| 0u32);
        let secondary_clicks = cx.state().local_init(|| 0u32);
        let double_clicks = cx.state().local_init(|| 0u32);
        let long_presses = cx.state().local_init(|| 0u32);
        let press_holding = cx.state().local_init(|| false);

        let drag_offset = cx.state().local_init(Point::default);
        let drag_starts = cx.state().local_init(|| 0u32);
        let drag_stops = cx.state().local_init(|| 0u32);

        let last_context_menu_anchor = cx.state().local_init(|| None::<Point>);
        let menu_toggle = cx.state().local_init(|| false);
        let lifecycle_button_activations = cx.state().local_init(|| 0u32);
        let lifecycle_button_deactivations = cx.state().local_init(|| 0u32);
        let lifecycle_checkbox_edits = cx.state().local_init(|| 0u32);
        let lifecycle_checkbox_after_edit = cx.state().local_init(|| 0u32);
        let lifecycle_slider_edits = cx.state().local_init(|| 0u32);
        let lifecycle_slider_after_edit = cx.state().local_init(|| 0u32);
        let lifecycle_text_activations = cx.state().local_init(|| 0u32);
        let lifecycle_text_deactivations = cx.state().local_init(|| 0u32);
        let lifecycle_text_after_edit = cx.state().local_init(|| 0u32);
        let lifecycle_menu_activations = cx.state().local_init(|| 0u32);
        let lifecycle_menu_deactivations = cx.state().local_init(|| 0u32);
        let lifecycle_combo_activations = cx.state().local_init(|| 0u32);
        let lifecycle_combo_deactivations = cx.state().local_init(|| 0u32);
        let lifecycle_combo_model_edits = cx.state().local_init(|| 0u32);
        let lifecycle_combo_model_after_edit = cx.state().local_init(|| 0u32);
        let trigger_menu_opened = cx.state().local_init(|| 0u32);
        let trigger_menu_closed = cx.state().local_init(|| 0u32);
        let trigger_submenu_toggled = cx.state().local_init(|| 0u32);
        let trigger_tab_switched = cx.state().local_init(|| 0u32);
        let trigger_tab_scene_clicks = cx.state().local_init(|| 0u32);
        let trigger_tab_scene_activations = cx.state().local_init(|| 0u32);
        let trigger_tab_scene_deactivations = cx.state().local_init(|| 0u32);
        let trigger_tab_selected = cx
            .state()
            .local_init(|| Some(Arc::<str>::from("inspector")));
        let lifecycle_checkbox_value = cx.state().local_init(|| false);
        let lifecycle_slider_value = cx.state().local_init(|| 24.0f32);
        let lifecycle_text_value = cx.state().local_init(String::new);
        let lifecycle_combo_model_value = cx.state().local_init(|| None::<Arc<str>>);

        let left_clicks_value = left_clicks.layout_value(cx);
        let secondary_clicks_value = secondary_clicks.layout_value(cx);
        let double_clicks_value = double_clicks.layout_value(cx);
        let long_presses_value = long_presses.layout_value(cx);
        let press_holding_value = press_holding.layout_value(cx);

        let drag_offset_value = drag_offset.layout_value(cx);
        let drag_starts_value = drag_starts.layout_value(cx);
        let drag_stops_value = drag_stops.layout_value(cx);

        let last_anchor_value = last_context_menu_anchor.layout_value(cx);
        let menu_toggle_value = menu_toggle.layout_value(cx);
        let lifecycle_button_activations_value = lifecycle_button_activations.layout_value(cx);
        let lifecycle_button_deactivations_value = lifecycle_button_deactivations.layout_value(cx);
        let lifecycle_checkbox_edits_value = lifecycle_checkbox_edits.layout_value(cx);
        let lifecycle_checkbox_after_edit_value = lifecycle_checkbox_after_edit.layout_value(cx);
        let lifecycle_slider_edits_value = lifecycle_slider_edits.layout_value(cx);
        let lifecycle_slider_after_edit_value = lifecycle_slider_after_edit.layout_value(cx);
        let lifecycle_text_activations_value = lifecycle_text_activations.layout_value(cx);
        let lifecycle_text_deactivations_value = lifecycle_text_deactivations.layout_value(cx);
        let lifecycle_text_after_edit_value = lifecycle_text_after_edit.layout_value(cx);
        let lifecycle_menu_activations_value = lifecycle_menu_activations.layout_value(cx);
        let lifecycle_menu_deactivations_value = lifecycle_menu_deactivations.layout_value(cx);
        let lifecycle_combo_activations_value = lifecycle_combo_activations.layout_value(cx);
        let lifecycle_combo_deactivations_value = lifecycle_combo_deactivations.layout_value(cx);
        let lifecycle_combo_model_edits_value = lifecycle_combo_model_edits.layout_value(cx);
        let lifecycle_combo_model_after_edit_value =
            lifecycle_combo_model_after_edit.layout_value(cx);
        let trigger_menu_opened_value = trigger_menu_opened.layout_value(cx);
        let trigger_menu_closed_value = trigger_menu_closed.layout_value(cx);
        let trigger_submenu_toggled_value = trigger_submenu_toggled.layout_value(cx);
        let trigger_tab_switched_value = trigger_tab_switched.layout_value(cx);
        let trigger_tab_scene_clicks_value = trigger_tab_scene_clicks.layout_value(cx);
        let trigger_tab_scene_activations_value = trigger_tab_scene_activations.layout_value(cx);
        let trigger_tab_scene_deactivations_value =
            trigger_tab_scene_deactivations.layout_value(cx);
        let trigger_tab_selected_value = trigger_tab_selected.layout_value(cx);
        let lifecycle_checkbox_value_value = lifecycle_checkbox_value.layout_value(cx);
        let lifecycle_slider_value_value = lifecycle_slider_value.layout_value(cx);
        let lifecycle_text_value_value = lifecycle_text_value.layout_value(cx);
        let lifecycle_combo_model_value_value = lifecycle_combo_model_value.layout_value(cx);

        fret_imui::imui_vstack(cx.elements(), |ui| {
            use fret_ui_kit::imui::UiWriterImUiFacadeExt as _;
            use fret_ui_kit::imui::UiWriterUiKitExt as _;

            let title = fret_ui_kit::ui::text("imui response signals proof (facade)")
                .text_sm()
                .font_semibold();
            ui.add_ui(title);

            let hint = fret_ui_kit::ui::text(
                "Proof surface: right click, double click, drag, and open menus. For the polished shell, run imui_interaction_showcase_demo.",
            )
            .text_xs();
            ui.add_ui(hint);

            ui.separator();

            let click_report = fret_ui_kit::ui::text(format!(
                "clicks: left={left_clicks_value} secondary={secondary_clicks_value} double={double_clicks_value} long={long_presses_value} holding={press_holding_value}"
            ))
            .text_sm()
            .font_medium();
            ui.add_ui(click_report);

            let click = ui.button("Click variants (left/right/double/long-press)");
            if click.clicked() {
                let _ = left_clicks.update_in(ui.cx_mut().app.models_mut(), |value| *value += 1);
            }
            if click.secondary_clicked() {
                let _ =
                    secondary_clicks.update_in(ui.cx_mut().app.models_mut(), |value| *value += 1);
            }
            if click.double_clicked() {
                let _ = double_clicks.update_in(ui.cx_mut().app.models_mut(), |value| *value += 1);
            }
            if click.long_pressed() {
                let _ = long_presses.update_in(ui.cx_mut().app.models_mut(), |value| *value += 1);
            }
            let _ = press_holding.set_in(ui.cx_mut().app.models_mut(), click.press_holding());

            ui.separator();

            let drag_report = fret_ui_kit::ui::text(format!(
                "drag: starts={drag_starts_value} stops={drag_stops_value} offset=({:.1},{:.1})",
                drag_offset_value.x.0, drag_offset_value.y.0
            ))
            .text_sm()
            .font_medium();
            ui.add_ui(drag_report);

            let drag = ui.button("Drag surface (hold left + move)");
            if drag.drag_started() {
                let _ = drag_starts.update_in(ui.cx_mut().app.models_mut(), |value| *value += 1);
            }
            if drag.dragging() {
                let delta = drag.drag_delta();
                let _ = drag_offset.update_in(ui.cx_mut().app.models_mut(), |value| {
                    value.x = Px(value.x.0 + delta.x.0);
                    value.y = Px(value.y.0 + delta.y.0);
                });
            }
            if drag.drag_stopped() {
                let _ = drag_stops.update_in(ui.cx_mut().app.models_mut(), |value| *value += 1);
            }

            let drag_delta = drag.drag_delta();
            let drag_total = drag.drag_total();
            let drag_rect: Option<Rect> = drag.core.rect;
            let drag_details = fret_ui_kit::ui::text(format!(
                "drag delta=({:.1},{:.1}) total=({:.1},{:.1}) rect={}",
                drag_delta.x.0,
                drag_delta.y.0,
                drag_total.x.0,
                drag_total.y.0,
                format_rect_short(drag_rect)
            ))
            .text_xs();
            ui.add_ui(drag_details);

            ui.separator();

            let lifecycle_report = fret_ui_kit::ui::text(format!(
                "lifecycle: button a/d={lifecycle_button_activations_value}/{lifecycle_button_deactivations_value} checkbox edit/after={lifecycle_checkbox_edits_value}/{lifecycle_checkbox_after_edit_value} slider edit/after={lifecycle_slider_edits_value}/{lifecycle_slider_after_edit_value} text a/d/after={lifecycle_text_activations_value}/{lifecycle_text_deactivations_value}/{lifecycle_text_after_edit_value}"
            ))
            .text_sm()
            .font_medium();
            ui.add_ui(lifecycle_report);

            let lifecycle_more_report = fret_ui_kit::ui::text(format!(
                "more lifecycle: menu a/d={lifecycle_menu_activations_value}/{lifecycle_menu_deactivations_value} combo a/d={lifecycle_combo_activations_value}/{lifecycle_combo_deactivations_value} combo_model edit/after={lifecycle_combo_model_edits_value}/{lifecycle_combo_model_after_edit_value}"
            ))
            .text_sm()
            .font_medium();
            ui.add_ui(lifecycle_more_report);

            let lifecycle_button = ui.button("Lifecycle button (hold and release)");
            if lifecycle_button.activated() {
                let _ = lifecycle_button_activations
                    .update_in(ui.cx_mut().app.models_mut(), |value| *value += 1);
            }
            if lifecycle_button.deactivated() {
                let _ = lifecycle_button_deactivations
                    .update_in(ui.cx_mut().app.models_mut(), |value| *value += 1);
            }

            let lifecycle_checkbox_resp =
                ui.checkbox_model("Edited checkbox", lifecycle_checkbox_value.model());
            if lifecycle_checkbox_resp.edited() {
                let _ = lifecycle_checkbox_edits
                    .update_in(ui.cx_mut().app.models_mut(), |value| *value += 1);
            }
            if lifecycle_checkbox_resp.deactivated_after_edit() {
                let _ = lifecycle_checkbox_after_edit
                    .update_in(ui.cx_mut().app.models_mut(), |value| *value += 1);
            }

            let lifecycle_slider_resp = ui.slider_f32_model_with_options(
                "Edited slider",
                lifecycle_slider_value.model(),
                fret_ui_kit::imui::SliderOptions {
                    min: 0.0,
                    max: 100.0,
                    step: 1.0,
                    ..Default::default()
                },
            );
            if lifecycle_slider_resp.edited() {
                let _ = lifecycle_slider_edits
                    .update_in(ui.cx_mut().app.models_mut(), |value| *value += 1);
            }
            if lifecycle_slider_resp.deactivated_after_edit() {
                let _ = lifecycle_slider_after_edit
                    .update_in(ui.cx_mut().app.models_mut(), |value| *value += 1);
            }

            let lifecycle_text_resp = ui.input_text_model_with_options(
                lifecycle_text_value.model(),
                fret_ui_kit::imui::InputTextOptions {
                    placeholder: Some(Arc::from("Focus, type, and blur")),
                    ..Default::default()
                },
            );
            if lifecycle_text_resp.activated() {
                let _ = lifecycle_text_activations
                    .update_in(ui.cx_mut().app.models_mut(), |value| *value += 1);
            }
            if lifecycle_text_resp.deactivated() {
                let _ = lifecycle_text_deactivations
                    .update_in(ui.cx_mut().app.models_mut(), |value| *value += 1);
            }
            if lifecycle_text_resp.deactivated_after_edit() {
                let _ = lifecycle_text_after_edit
                    .update_in(ui.cx_mut().app.models_mut(), |value| *value += 1);
            }

            let menu_lifecycle = ui.menu_item_with_options(
                "Lifecycle menu item (press and release)",
                fret_ui_kit::imui::MenuItemOptions {
                    test_id: Some(Arc::from("imui-resp-demo.lifecycle-menu")),
                    ..Default::default()
                },
            );
            if menu_lifecycle.activated() {
                let _ = lifecycle_menu_activations
                    .update_in(ui.cx_mut().app.models_mut(), |value| *value += 1);
            }
            if menu_lifecycle.deactivated() {
                let _ = lifecycle_menu_deactivations
                    .update_in(ui.cx_mut().app.models_mut(), |value| *value += 1);
            }

            let combo_resp = ui.combo_with_options(
                "imui-resp-demo.lifecycle-combo",
                "Lifecycle combo",
                "Preview only",
                fret_ui_kit::imui::ComboOptions {
                    test_id: Some(Arc::from("imui-resp-demo.lifecycle-combo")),
                    ..Default::default()
                },
                |ui| {
                    let _ = ui.selectable_with_options(
                        "Preview only",
                        fret_ui_kit::imui::SelectableOptions {
                            test_id: Some(Arc::from("imui-resp-demo.lifecycle-combo.option.0")),
                            ..Default::default()
                        },
                    );
                },
            );
            if combo_resp.trigger.activated() {
                let _ = lifecycle_combo_activations
                    .update_in(ui.cx_mut().app.models_mut(), |value| *value += 1);
            }
            if combo_resp.trigger.deactivated() {
                let _ = lifecycle_combo_deactivations
                    .update_in(ui.cx_mut().app.models_mut(), |value| *value += 1);
            }

            let lifecycle_combo_items = vec![
                Arc::<str>::from("Alpha"),
                Arc::<str>::from("Beta"),
                Arc::<str>::from("Gamma"),
            ];
            let combo_model_resp = ui.combo_model_with_options(
                "imui-resp-demo.lifecycle-combo-model",
                "Lifecycle combo model",
                lifecycle_combo_model_value.model(),
                &lifecycle_combo_items,
                fret_ui_kit::imui::ComboModelOptions {
                    test_id: Some(Arc::from("imui-resp-demo.lifecycle-combo-model")),
                    placeholder: Some(Arc::from("Pick a mode")),
                    ..Default::default()
                },
            );
            if combo_model_resp.edited() {
                let _ = lifecycle_combo_model_edits
                    .update_in(ui.cx_mut().app.models_mut(), |value| *value += 1);
            }
            if combo_model_resp.deactivated_after_edit() {
                let _ = lifecycle_combo_model_after_edit
                    .update_in(ui.cx_mut().app.models_mut(), |value| *value += 1);
            }

            let lifecycle_details = fret_ui_kit::ui::text(format!(
                "current values: checkbox={} slider={:.0} text_len={} combo_model={}",
                lifecycle_checkbox_value_value,
                lifecycle_slider_value_value,
                lifecycle_text_value_value.len(),
                lifecycle_combo_model_value_value
                    .as_deref()
                    .unwrap_or("none")
            ))
            .text_xs();
            ui.add_ui(lifecycle_details);

            ui.separator();

            let trigger_surface_report = fret_ui_kit::ui::text(format!(
                "trigger surfaces: menu open/close={trigger_menu_opened_value}/{trigger_menu_closed_value} submenu toggles={trigger_submenu_toggled_value} tabs selected={} switches={} scene click/a/d={}/{}/{}",
                trigger_tab_selected_value.as_deref().unwrap_or("none"),
                trigger_tab_switched_value,
                trigger_tab_scene_clicks_value,
                trigger_tab_scene_activations_value,
                trigger_tab_scene_deactivations_value
            ))
            .text_sm()
            .font_medium();
            ui.add_ui(trigger_surface_report);

            ui.menu_bar_with_options(
                fret_ui_kit::imui::MenuBarOptions {
                    test_id: Some(Arc::from("imui-resp-demo.trigger-menu.root")),
                    ..Default::default()
                },
                |ui| {
                    let file_menu = ui.begin_menu_with_options(
                        "imui-resp-demo.trigger-menu.file",
                        "Trigger surface menu",
                        fret_ui_kit::imui::BeginMenuOptions {
                            test_id: Some(Arc::from("imui-resp-demo.trigger-menu.file")),
                            ..Default::default()
                        },
                        |ui| {
                            let recent_menu = ui.begin_submenu_with_options(
                                "imui-resp-demo.trigger-menu.recent",
                                "Recent",
                                fret_ui_kit::imui::BeginSubmenuOptions {
                                    test_id: Some(Arc::from(
                                        "imui-resp-demo.trigger-menu.file.recent",
                                    )),
                                    ..Default::default()
                                },
                                |ui| {
                                    let _ = ui.menu_item_with_options(
                                        "Project Alpha",
                                        fret_ui_kit::imui::MenuItemOptions {
                                            test_id: Some(Arc::from(
                                                "imui-resp-demo.trigger-menu.file.recent.alpha",
                                            )),
                                            ..Default::default()
                                        },
                                    );
                                },
                            );
                            if recent_menu.toggled() {
                                let _ = trigger_submenu_toggled
                                    .update_in(ui.cx_mut().app.models_mut(), |value| *value += 1);
                            }
                        },
                    );
                    if file_menu.opened() {
                        let _ = trigger_menu_opened
                            .update_in(ui.cx_mut().app.models_mut(), |value| *value += 1);
                    }
                    if file_menu.closed() {
                        let _ = trigger_menu_closed
                            .update_in(ui.cx_mut().app.models_mut(), |value| *value += 1);
                    }
                },
            );

            let tab_response = ui.tab_bar_with_options(
                "imui-resp-demo.trigger-tabs",
                fret_ui_kit::imui::TabBarOptions {
                    selected: Some(trigger_tab_selected.model().clone()),
                    test_id: Some(Arc::from("imui-resp-demo.trigger-tabs.root")),
                    ..Default::default()
                },
                |tabs| {
                    tabs.begin_tab_item_with_options(
                        "scene",
                        "Scene",
                        fret_ui_kit::imui::TabItemOptions {
                            test_id: Some(Arc::from("imui-resp-demo.trigger-tabs.scene")),
                            panel_test_id: Some(Arc::from(
                                "imui-resp-demo.trigger-tabs.scene.panel",
                            )),
                            ..Default::default()
                        },
                        |ui| {
                            ui.text("Scene trigger panel");
                        },
                    );
                    tabs.begin_tab_item_with_options(
                        "inspector",
                        "Inspector",
                        fret_ui_kit::imui::TabItemOptions {
                            test_id: Some(Arc::from("imui-resp-demo.trigger-tabs.inspector")),
                            panel_test_id: Some(Arc::from(
                                "imui-resp-demo.trigger-tabs.inspector.panel",
                            )),
                            ..Default::default()
                        },
                        |ui| {
                            ui.text("Inspector trigger panel");
                        },
                    );
                },
            );
            if tab_response.selected_changed() {
                let _ = trigger_tab_switched
                    .update_in(ui.cx_mut().app.models_mut(), |value| *value += 1);
            }
            if let Some(scene_tab) = tab_response.trigger("scene") {
                if scene_tab.clicked() {
                    let _ = trigger_tab_scene_clicks
                        .update_in(ui.cx_mut().app.models_mut(), |value| *value += 1);
                }
                if scene_tab.activated() {
                    let _ = trigger_tab_scene_activations
                        .update_in(ui.cx_mut().app.models_mut(), |value| *value += 1);
                }
                if scene_tab.deactivated() {
                    let _ = trigger_tab_scene_deactivations
                        .update_in(ui.cx_mut().app.models_mut(), |value| *value += 1);
                }
            }

            ui.separator();

            let ctx_report = fret_ui_kit::ui::text(format!(
                "context menu: last_anchor={} toggle={menu_toggle_value}",
                format_point_short(last_anchor_value)
            ))
            .text_sm()
            .font_medium();
            ui.add_ui(ctx_report);

            let trigger = ui.button("Context menu (right click)");
            if trigger.context_menu_requested() {
                let _ = last_context_menu_anchor
                    .set_in(ui.cx_mut().app.models_mut(), trigger.context_menu_anchor());
            }

            ui.begin_popup_context_menu("ctx", trigger, |ui| {
                let toggle = ui.menu_item_with_options(
                    "Toggle flag",
                    fret_ui_kit::imui::MenuItemOptions {
                        test_id: Some(Arc::from("imui-resp-demo.ctx.toggle")),
                        ..Default::default()
                    },
                );
                if toggle.clicked() {
                    let _ = menu_toggle
                        .update_in(ui.cx_mut().app.models_mut(), |value| *value = !*value);
                }

                let open = ui.popup_open_model("ctx");
                let _ = ui.menu_item_with_options(
                    "Close menu",
                    fret_ui_kit::imui::MenuItemOptions {
                        close_popup: Some(open),
                        test_id: Some(Arc::from("imui-resp-demo.ctx.close")),
                        ..Default::default()
                    },
                );
            });
        })
    }
}

fn format_point_short(p: Option<Point>) -> String {
    match p {
        Some(p) => format!("({:.1},{:.1})", p.x.0, p.y.0),
        None => "none".to_string(),
    }
}

fn format_rect_short(r: Option<Rect>) -> String {
    match r {
        Some(r) => format!(
            "({:.1},{:.1} {:.1}x{:.1})",
            r.origin.x.0, r.origin.y.0, r.size.width.0, r.size.height.0
        ),
        None => "none".to_string(),
    }
}
