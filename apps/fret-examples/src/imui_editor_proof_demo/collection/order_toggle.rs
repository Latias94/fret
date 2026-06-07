use std::sync::Arc;

use fret::imui::{kit, prelude::*};
use fret_runtime::Model;

use super::KernelApp;

pub(super) fn render_collection_order_toggle(
    ui: &mut ImUi<'_, '_, KernelApp>,
    reverse_order_model: &Model<bool>,
    reverse_order: bool,
) -> bool {
    let order_toggle = ui.button_with_options(
        if reverse_order {
            "Show folder order"
        } else {
            "Reverse visible order"
        },
        kit::ButtonOptions {
            test_id: Some(Arc::from(
                "imui-editor-proof.authoring.imui.collection.order-toggle",
            )),
            ..Default::default()
        },
    );
    if !order_toggle.clicked() {
        return reverse_order;
    }

    let _ = ui
        .cx_mut()
        .app
        .models_mut()
        .update(reverse_order_model, |value| *value = !*value);
    !reverse_order
}
