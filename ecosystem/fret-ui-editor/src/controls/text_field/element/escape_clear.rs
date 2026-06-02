use std::sync::Arc;

use fret_core::KeyCode;
use fret_runtime::Model;
use fret_ui::ElementContext;
use fret_ui::UiHost;
use fret_ui::elements::GlobalElementId;

#[cfg(test)]
mod tests;

pub(super) fn install_text_field_escape_clear_handler<H: UiHost>(
    cx: &mut ElementContext<'_, H>,
    entry_id: GlobalElementId,
    model: Model<String>,
) {
    cx.key_add_on_key_down_capture_for(
        entry_id,
        Arc::new(move |host, action_cx, down| {
            if !text_field_escape_clear_should_handle_key(down.key) {
                return false;
            }

            let _ = host.models_mut().update(&model, |s| s.clear());
            host.request_redraw(action_cx.window);
            true
        }),
    );
}

fn text_field_escape_clear_should_handle_key(key: KeyCode) -> bool {
    key == KeyCode::Escape
}
