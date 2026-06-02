use fret_core::{Point, Px};
use fret_runtime::Model;
use fret_ui::{ElementContext, GlobalElementId, UiHost};

use crate::primitives::tooltip as radix_tooltip;

use super::super::trigger::install_pointer_move_open_gate_for;

pub(super) struct TooltipRuntimeModels {
    pub(super) open: Model<bool>,
    pub(super) panel_id: Model<Option<GlobalElementId>>,
    pub(super) event_models: radix_tooltip::TooltipTriggerEventModels,
    pub(super) last_pointer: Model<Option<Point>>,
}

pub(super) fn prepare_tooltip_runtime_models<H: UiHost>(
    cx: &mut ElementContext<'_, H>,
    trigger_id: GlobalElementId,
) -> TooltipRuntimeModels {
    let open = cx.local_model_keyed("open", || false);
    let panel_id = cx.local_model_keyed("panel_id", || None::<GlobalElementId>);
    let event_models = radix_tooltip::tooltip_trigger_event_models(cx);
    let last_pointer = radix_tooltip::tooltip_last_pointer_model(cx);

    radix_tooltip::tooltip_install_default_trigger_dismiss_handlers(
        cx,
        trigger_id,
        event_models.clone(),
    );
    install_pointer_move_open_gate_for(
        cx,
        trigger_id,
        event_models.clone(),
        Px(5.0),
        last_pointer.clone(),
    );

    TooltipRuntimeModels {
        open,
        panel_id,
        event_models,
        last_pointer,
    }
}
