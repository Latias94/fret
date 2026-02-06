use std::sync::Arc;

use fret_core::{Point, Px, Rect};
use fret_kit::prelude::*;

struct ImUiResponseSignalsState {
    left_clicks: Model<u32>,
    secondary_clicks: Model<u32>,
    double_clicks: Model<u32>,
    long_presses: Model<u32>,
    press_holding: Model<bool>,

    drag_offset: Model<Point>,
    drag_starts: Model<u32>,
    drag_stops: Model<u32>,

    last_context_menu_anchor: Model<Option<Point>>,
    menu_toggle: Model<bool>,
}

pub fn run() -> anyhow::Result<()> {
    fret_kit::app("imui-response-signals-demo", init_window, view)?
        .with_main_window("imui_response_signals_demo", (720.0, 520.0))
        .run()?;
    Ok(())
}

fn init_window(app: &mut App, _window: AppWindowId) -> ImUiResponseSignalsState {
    ImUiResponseSignalsState {
        left_clicks: app.models_mut().insert(0),
        secondary_clicks: app.models_mut().insert(0),
        double_clicks: app.models_mut().insert(0),
        long_presses: app.models_mut().insert(0),
        press_holding: app.models_mut().insert(false),

        drag_offset: app.models_mut().insert(Point::default()),
        drag_starts: app.models_mut().insert(0),
        drag_stops: app.models_mut().insert(0),

        last_context_menu_anchor: app.models_mut().insert(None::<Point>),
        menu_toggle: app.models_mut().insert(false),
    }
}

fn view(cx: &mut ElementContext<'_, App>, st: &mut ImUiResponseSignalsState) -> ViewElements {
    let left_clicks = cx
        .watch_model(&st.left_clicks)
        .layout()
        .copied()
        .unwrap_or_default();
    let secondary_clicks = cx
        .watch_model(&st.secondary_clicks)
        .layout()
        .copied()
        .unwrap_or_default();
    let double_clicks = cx
        .watch_model(&st.double_clicks)
        .layout()
        .copied()
        .unwrap_or_default();
    let long_presses = cx
        .watch_model(&st.long_presses)
        .layout()
        .copied()
        .unwrap_or_default();
    let press_holding = cx
        .watch_model(&st.press_holding)
        .layout()
        .copied()
        .unwrap_or_default();

    let drag_offset = cx
        .watch_model(&st.drag_offset)
        .layout()
        .copied()
        .unwrap_or_default();
    let drag_starts = cx
        .watch_model(&st.drag_starts)
        .layout()
        .copied()
        .unwrap_or_default();
    let drag_stops = cx
        .watch_model(&st.drag_stops)
        .layout()
        .copied()
        .unwrap_or_default();

    let last_anchor = cx
        .watch_model(&st.last_context_menu_anchor)
        .layout()
        .copied()
        .unwrap_or(None);
    let menu_toggle = cx
        .watch_model(&st.menu_toggle)
        .layout()
        .copied()
        .unwrap_or_default();

    fret_imui::imui_vstack(cx, |ui| {
        use fret_ui_kit::imui::UiWriterImUiFacadeExt as _;
        use fret_ui_kit::imui::UiWriterUiKitExt as _;

        let title = fret_ui_kit::ui::text(ui.cx_mut(), "imui response signals demo (facade)")
            .text_sm()
            .font_semibold();
        ui.add_ui(title);

        let hint = fret_ui_kit::ui::text(
            ui.cx_mut(),
            "Left click, right click, double click, drag, and open a context menu.",
        )
        .text_xs();
        ui.add_ui(hint);

        ui.separator();

        // Click variants.
        let click_report = fret_ui_kit::ui::text(
            ui.cx_mut(),
            format!(
                "clicks: left={left_clicks} secondary={secondary_clicks} double={double_clicks} long={long_presses} holding={press_holding}"
            ),
        )
        .text_sm()
        .font_medium();
        ui.add_ui(click_report);
        let click = ui.button("Click variants (left/right/double/long-press)");
        if click.clicked() {
            let _ = ui
                .cx_mut()
                .app
                .models_mut()
                .update(&st.left_clicks, |v| *v += 1);
        }
        if click.secondary_clicked() {
            let _ = ui
                .cx_mut()
                .app
                .models_mut()
                .update(&st.secondary_clicks, |v| *v += 1);
        }
        if click.double_clicked() {
            let _ = ui
                .cx_mut()
                .app
                .models_mut()
                .update(&st.double_clicks, |v| *v += 1);
        }
        if click.long_pressed() {
            let _ = ui
                .cx_mut()
                .app
                .models_mut()
                .update(&st.long_presses, |v| *v += 1);
        }
        let _ = ui
            .cx_mut()
            .app
            .models_mut()
            .update(&st.press_holding, |v| *v = click.press_holding());

        ui.separator();

        // Drag lifecycle + deltas.
        let drag_report = fret_ui_kit::ui::text(
            ui.cx_mut(),
            format!(
                "drag: starts={drag_starts} stops={drag_stops} offset=({:.1},{:.1})",
                drag_offset.x.0, drag_offset.y.0
            ),
        )
        .text_sm()
        .font_medium();
        ui.add_ui(drag_report);

        let drag = ui.button("Drag surface (hold left + move)");
        if drag.drag_started() {
            let _ = ui
                .cx_mut()
                .app
                .models_mut()
                .update(&st.drag_starts, |v| *v += 1);
        }
        if drag.dragging() {
            let delta = drag.drag_delta();
            let _ = ui.cx_mut().app.models_mut().update(&st.drag_offset, |v| {
                v.x = Px(v.x.0 + delta.x.0);
                v.y = Px(v.y.0 + delta.y.0);
            });
        }
        if drag.drag_stopped() {
            let _ = ui
                .cx_mut()
                .app
                .models_mut()
                .update(&st.drag_stops, |v| *v += 1);
        }

        let drag_delta = drag.drag_delta();
        let drag_total = drag.drag_total();
        let drag_rect: Option<Rect> = drag.core.rect;
        let drag_details = fret_ui_kit::ui::text(
            ui.cx_mut(),
            format!(
                "drag delta=({:.1},{:.1}) total=({:.1},{:.1}) rect={}",
                drag_delta.x.0,
                drag_delta.y.0,
                drag_total.x.0,
                drag_total.y.0,
                format_rect_short(drag_rect)
            ),
        )
        .text_xs();
        ui.add_ui(drag_details);

        ui.separator();

        // Context menu request + anchor.
        let ctx_report = fret_ui_kit::ui::text(
            ui.cx_mut(),
            format!(
                "context menu: last_anchor={} toggle={menu_toggle}",
                format_point_short(last_anchor)
            ),
        )
        .text_sm()
        .font_medium();
        ui.add_ui(ctx_report);

        let trigger = ui.button("Context menu (right click)");
        if trigger.context_menu_requested() {
            let _ = ui
                .cx_mut()
                .app
                .models_mut()
                .update(&st.last_context_menu_anchor, |v| {
                    *v = trigger.context_menu_anchor()
                });
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
                let _ = ui
                    .cx_mut()
                    .app
                    .models_mut()
                    .update(&st.menu_toggle, |v| *v = !*v);
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
