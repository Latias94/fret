use fret::imui::ImUi;

use super::runtime_state::{ProofCollectionRuntimeModels, ProofCollectionRuntimeSnapshot};
use super::{KernelApp, ProofCollectionAsset};

pub(super) fn clear_stale_collection_rename_session(
    ui: &mut ImUi<'_, '_, KernelApp>,
    models: &ProofCollectionRuntimeModels,
    snapshot: &ProofCollectionRuntimeSnapshot,
    assets: &[ProofCollectionAsset],
) {
    if let Some(session) = snapshot.rename_session.as_ref()
        && !assets.iter().any(|asset| asset.id == session.target_id)
    {
        let _ = ui
            .cx_mut()
            .app
            .models_mut()
            .update(&models.rename_session, |state| *state = None);
        let _ = ui
            .cx_mut()
            .app
            .models_mut()
            .update(&models.rename_focus_pending, |state| *state = false);
    }
}
