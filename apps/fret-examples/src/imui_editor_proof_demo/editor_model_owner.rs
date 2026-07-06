use std::any::Any;

use fret_runtime::{Model, ModelStore};
use fret_ui_editor::controls::TextFieldOutcome;

use super::proof_helpers::edit_session_outcome_label;

pub(super) struct EditorProofModelOwner<'a> {
    models: &'a mut ModelStore,
}

impl<'a> EditorProofModelOwner<'a> {
    pub(super) fn new(models: &'a mut ModelStore) -> Self {
        Self { models }
    }

    fn update<T: Any, R>(&mut self, model: &Model<T>, f: impl FnOnce(&mut T) -> R) -> Option<R> {
        self.models.update(model, f).ok()
    }

    fn replace_string(&mut self, model: &Model<String>, next_value: &str) -> bool {
        self.update(model, |value| {
            value.clear();
            value.push_str(next_value);
        })
        .is_some()
    }

    pub(super) fn record_asset_ref_action(
        &mut self,
        asset_slot_model: &Model<String>,
        action_model: &Model<String>,
        action_label: &'static str,
        next_asset: Option<&'static str>,
    ) {
        if let Some(next_asset) = next_asset {
            let _ = self.replace_string(asset_slot_model, next_asset);
        }
        let _ = self.replace_string(action_model, action_label);
    }

    pub(super) fn record_text_assist_accept(
        &mut self,
        accepted_label_model: &Model<String>,
        accepted_label: &str,
    ) {
        let _ = self.replace_string(accepted_label_model, accepted_label);
    }

    pub(super) fn record_text_field_outcome(
        &mut self,
        outcome_model: &Model<String>,
        outcome: TextFieldOutcome,
    ) {
        let _ = self.replace_string(outcome_model, edit_session_outcome_label(outcome));
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use fret_runtime::ModelStore;
    use fret_ui_editor::primitives::EditSessionOutcome;

    #[test]
    fn editor_proof_model_owner_records_asset_ref_actions() {
        let mut models = ModelStore::default();
        let asset_slot = models.insert(String::from("textures/default.ktx2"));
        let action = models.insert(String::new());

        EditorProofModelOwner::new(&mut models).record_asset_ref_action(
            &asset_slot,
            &action,
            "Chose alternate base texture",
            Some("textures/alternate.ktx2"),
        );

        assert_eq!(
            models.read(&asset_slot, Clone::clone).unwrap(),
            "textures/alternate.ktx2"
        );
        assert_eq!(
            models.read(&action, Clone::clone).unwrap(),
            "Chose alternate base texture"
        );

        EditorProofModelOwner::new(&mut models).record_asset_ref_action(
            &asset_slot,
            &action,
            "Reveal requested",
            None,
        );

        assert_eq!(
            models.read(&asset_slot, Clone::clone).unwrap(),
            "textures/alternate.ktx2"
        );
        assert_eq!(
            models.read(&action, Clone::clone).unwrap(),
            "Reveal requested"
        );
    }

    #[test]
    fn editor_proof_model_owner_records_text_assist_and_outcomes() {
        let mut models = ModelStore::default();
        let accepted = models.insert(String::new());
        let outcome = models.insert(String::new());

        EditorProofModelOwner::new(&mut models)
            .record_text_assist_accept(&accepted, "Directional Light");
        assert_eq!(
            models.read(&accepted, Clone::clone).unwrap(),
            "Directional Light"
        );

        EditorProofModelOwner::new(&mut models)
            .record_text_field_outcome(&outcome, EditSessionOutcome::Committed);
        assert_eq!(models.read(&outcome, Clone::clone).unwrap(), "Committed");

        EditorProofModelOwner::new(&mut models)
            .record_text_field_outcome(&outcome, EditSessionOutcome::Canceled);
        assert_eq!(models.read(&outcome, Clone::clone).unwrap(), "Canceled");
    }
}
