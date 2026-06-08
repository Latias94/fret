use std::sync::Arc;

use fret::imui::kit::ImUiMultiSelectState;

use super::ProofCollectionAsset;
use super::selection::proof_collection_selected_assets;

#[derive(Clone)]
pub(super) struct ProofCollectionDragPayload {
    lead_label: Arc<str>,
    lead_path: Arc<str>,
    asset_ids: Arc<[Arc<str>]>,
    asset_paths: Arc<[Arc<str>]>,
}

pub(super) fn proof_collection_drag_payload_for_asset(
    assets: &[ProofCollectionAsset],
    selection: &ImUiMultiSelectState<Arc<str>>,
    dragged: &ProofCollectionAsset,
) -> ProofCollectionDragPayload {
    let selected_assets = proof_collection_selected_assets(assets, selection);
    let payload_assets = if selection.is_selected(&dragged.id) && !selected_assets.is_empty() {
        selected_assets
    } else {
        vec![dragged]
    };
    let lead = payload_assets.first().copied().unwrap_or(dragged);
    let asset_ids = payload_assets
        .iter()
        .map(|asset| asset.id.clone())
        .collect::<Vec<_>>();
    let asset_paths = payload_assets
        .iter()
        .map(|asset| asset.path.clone())
        .collect::<Vec<_>>();

    ProofCollectionDragPayload {
        lead_label: lead.label.clone(),
        lead_path: lead.path.clone(),
        asset_ids: asset_ids.into(),
        asset_paths: asset_paths.into(),
    }
}

pub(super) fn proof_collection_drag_preview_title(
    payload: &ProofCollectionDragPayload,
) -> Arc<str> {
    if payload.asset_ids.len() == 1 {
        payload.lead_label.clone()
    } else {
        Arc::from(format!("{} selected assets", payload.asset_ids.len()))
    }
}

pub(super) fn proof_collection_drag_preview_subtitle(
    payload: &ProofCollectionDragPayload,
) -> Option<Arc<str>> {
    if payload.asset_paths.len() == 1 {
        Some(payload.lead_path.clone())
    } else {
        Some(Arc::from(format!(
            "{} + {} more",
            payload.lead_path,
            payload.asset_paths.len() - 1
        )))
    }
}

pub(super) fn proof_collection_drop_status(
    prefix: &str,
    payload: &ProofCollectionDragPayload,
) -> String {
    let paths = payload
        .asset_paths
        .iter()
        .map(|path| path.as_ref())
        .collect::<Vec<_>>()
        .join(", ");
    format!("{prefix} {} asset(s): {paths}", payload.asset_paths.len())
}

#[cfg(test)]
mod tests;
