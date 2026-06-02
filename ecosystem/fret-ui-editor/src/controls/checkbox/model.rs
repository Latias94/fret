use std::sync::Arc;

use fret_runtime::Model;
use fret_ui::action::{ActionCx, OnActivate};
use fret_ui::{ElementContext, Invalidation, UiHost};
use fret_ui_headless::boolean_control::{
    checkbox_checked_state_from_optional_bool, checkbox_toggle_optional_bool,
};
use fret_ui_headless::checked_state::CheckedState;

#[derive(Debug, Clone)]
pub(super) enum CheckboxModel {
    Bool(Model<bool>),
    OptionalBool(Model<Option<bool>>),
}

pub(super) fn checkbox_checked_state<H: UiHost>(
    cx: &mut ElementContext<'_, H>,
    model: &CheckboxModel,
) -> CheckedState {
    match model {
        CheckboxModel::Bool(model) => {
            let v = cx
                .get_model_copied(model, Invalidation::Paint)
                .unwrap_or(false);
            if v {
                CheckedState::Checked
            } else {
                CheckedState::Unchecked
            }
        }
        CheckboxModel::OptionalBool(model) => {
            let v = cx
                .get_model_cloned(model, Invalidation::Paint)
                .unwrap_or(None);
            checkbox_checked_state_from_optional_bool(v)
        }
    }
}

pub(super) fn checkbox_on_activate(model: CheckboxModel, enabled: bool) -> OnActivate {
    Arc::new(move |host, action_cx: ActionCx, _reason| {
        if !enabled {
            return;
        }

        match &model {
            CheckboxModel::Bool(model) => {
                let _ = host.models_mut().update(model, |v| *v = !*v);
            }
            CheckboxModel::OptionalBool(model) => {
                let _ = host
                    .models_mut()
                    .update(model, |v| *v = checkbox_toggle_optional_bool(*v));
            }
        }
        host.request_redraw(action_cx.window);
    })
}
