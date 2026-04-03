use fret_runtime::Model;
use fret_ui_kit::headless::form_state::FormState;

/// Narrow interop bridge for form helpers that watch a shared `FormState`.
pub trait IntoFormStateModel {
    fn into_form_state_model(self) -> Model<FormState>;
}

impl IntoFormStateModel for Model<FormState> {
    fn into_form_state_model(self) -> Model<FormState> {
        self
    }
}

impl IntoFormStateModel for &Model<FormState> {
    fn into_form_state_model(self) -> Model<FormState> {
        self.clone()
    }
}
