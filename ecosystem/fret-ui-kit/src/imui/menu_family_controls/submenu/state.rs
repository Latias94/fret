use fret_runtime::Model;
use fret_ui::UiHost;

use crate::imui::UiWriterImUiFacadeExt;

pub(super) struct SubmenuOpenSnapshot {
    pub(super) popup_open: Model<bool>,
    pub(super) was_open_model: Model<bool>,
    pub(super) open_before: bool,
    pub(super) was_open_before_render: bool,
}

pub(super) fn submenu_open_snapshot<H: UiHost, W: UiWriterImUiFacadeExt<H> + ?Sized>(
    ui: &mut W,
    id: &str,
) -> SubmenuOpenSnapshot {
    let popup_open = ui.popup_open_model(id);
    let was_open_model =
        ui.with_cx_mut(|cx| cx.local_model_keyed(format!("was_open.{id}"), || false));
    let open_before = read_submenu_open_after(ui, &popup_open);
    let was_open_before_render = ui.with_cx_mut(|cx| {
        cx.read_model(
            &was_open_model,
            fret_ui::Invalidation::Paint,
            |_app, value| *value,
        )
        .unwrap_or(false)
    });

    SubmenuOpenSnapshot {
        popup_open,
        was_open_model,
        open_before,
        was_open_before_render,
    }
}

pub(super) fn read_submenu_open_after<H: UiHost, W: UiWriterImUiFacadeExt<H> + ?Sized>(
    ui: &mut W,
    popup_open: &Model<bool>,
) -> bool {
    ui.with_cx_mut(|cx| {
        cx.read_model(popup_open, fret_ui::Invalidation::Paint, |_app, value| {
            *value
        })
        .unwrap_or(false)
    })
}

pub(super) fn record_submenu_open_after<H: UiHost, W: UiWriterImUiFacadeExt<H> + ?Sized>(
    ui: &mut W,
    was_open_model: &Model<bool>,
    open_after: bool,
) {
    ui.with_cx_mut(|cx| {
        let _ = cx
            .app
            .models_mut()
            .update(was_open_model, |value| *value = open_after);
    });
}
