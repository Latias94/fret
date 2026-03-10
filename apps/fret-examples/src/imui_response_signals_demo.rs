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
        let left_clicks = cx.use_local_with(|| 0u32);
        let secondary_clicks = cx.use_local_with(|| 0u32);
        let double_clicks = cx.use_local_with(|| 0u32);
        let long_presses = cx.use_local_with(|| 0u32);
        let press_holding = cx.use_local_with(|| false);

        let drag_offset = cx.use_local_with(Point::default);
        let drag_starts = cx.use_local_with(|| 0u32);
        let drag_stops = cx.use_local_with(|| 0u32);

        let last_context_menu_anchor = cx.use_local_with(|| None::<Point>);
        let menu_toggle = cx.use_local_with(|| false);

        let left_clicks_value = left_clicks.layout(cx).value_or_default();
        let secondary_clicks_value = secondary_clicks.layout(cx).value_or_default();
        let double_clicks_value = double_clicks.layout(cx).value_or_default();
        let long_presses_value = long_presses.layout(cx).value_or_default();
        let press_holding_value = press_holding.layout(cx).value_or_default();

        let drag_offset_value = drag_offset.layout(cx).value_or_default();
        let drag_starts_value = drag_starts.layout(cx).value_or_default();
        let drag_stops_value = drag_stops.layout(cx).value_or_default();

        let last_anchor_value = last_context_menu_anchor.layout(cx).value_or_default();
        let menu_toggle_value = menu_toggle.layout(cx).value_or_default();

        fret_imui::imui_vstack(cx.elements(), |ui| {
            use fret_ui_kit::imui::UiWriterImUiFacadeExt as _;
            use fret_ui_kit::imui::UiWriterUiKitExt as _;

            let title = fret_ui_kit::ui::text("imui response signals demo (facade)")
                .text_sm()
                .font_semibold();
            ui.add_ui(title);

            let hint =
                fret_ui_kit::ui::text("Right click, double click, drag, and open a context menu.")
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
                let toggle = ui.menu_item_ex(
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
                let _ = ui.menu_item_ex(
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
