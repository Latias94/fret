use fret_authoring::mark_immediate_render_frame;
use fret_ui::{ElementContext, UiHost};

pub(in crate::imui) fn prepare_imui_runtime_for_frame<H: UiHost>(cx: &mut ElementContext<'_, H>) {
    let _ = mark_immediate_render_frame(cx);
}
