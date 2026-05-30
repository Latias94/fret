use fret_runtime::Model;
use fret_ui::{ElementContext, UiHost};

pub(super) fn popup_modal_open_model<H: UiHost>(
    cx: &mut ElementContext<'_, H>,
    id: &str,
) -> Model<bool> {
    super::super::super::with_popup_store_for_id(cx, id, |st, _app| st.open.clone())
}

pub(super) fn popup_modal_is_open<H: UiHost>(
    cx: &mut ElementContext<'_, H>,
    open: &Model<bool>,
) -> bool {
    cx.read_model(open, fret_ui::Invalidation::Paint, |_app, v| *v)
        .unwrap_or(false)
}

pub(super) fn refresh_popup_modal_keep_alive<H: UiHost>(cx: &mut ElementContext<'_, H>, id: &str) {
    let keep_alive_generation = super::super::super::popup_render_generation_for_window(cx);
    super::super::super::with_popup_store_for_id(cx, id, move |st, _app| {
        st.keep_alive_generation = Some(keep_alive_generation);
    });
}
