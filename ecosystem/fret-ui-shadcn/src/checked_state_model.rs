use fret_runtime::Model;
use fret_ui_kit::primitives::checkbox::CheckedState;

/// Narrow interop bridge for tri-state widgets that still store their value in a
/// `Model<CheckedState>`.
pub trait IntoCheckedStateModel {
    fn into_checked_state_model(self) -> Model<CheckedState>;
}

impl IntoCheckedStateModel for Model<CheckedState> {
    fn into_checked_state_model(self) -> Model<CheckedState> {
        self
    }
}

impl IntoCheckedStateModel for &Model<CheckedState> {
    fn into_checked_state_model(self) -> Model<CheckedState> {
        self.clone()
    }
}
