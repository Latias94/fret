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
mod tests {
    use super::super::authoring_parity_collection_assets;
    use super::*;

    fn selection_state(selected: &[&str], anchor: Option<&str>) -> ImUiMultiSelectState<Arc<str>> {
        ImUiMultiSelectState::new(
            selected.iter().map(|id| Arc::from(*id)).collect(),
            anchor.map(Arc::from),
        )
    }

    #[test]
    fn proof_collection_drag_payload_for_selected_asset_carries_selected_set() {
        let assets = authoring_parity_collection_assets()
            .iter()
            .cloned()
            .collect::<Vec<_>>();
        let selection = selection_state(&["stone-albedo", "dust-mask"], Some("stone-albedo"));
        let dragged = assets
            .iter()
            .find(|asset| asset.id.as_ref() == "stone-albedo")
            .expect("fixture should include stone albedo");

        let payload = proof_collection_drag_payload_for_asset(&assets, &selection, dragged);

        assert_eq!(
            payload.asset_ids.as_ref(),
            [Arc::from("stone-albedo"), Arc::from("dust-mask")]
        );
        assert_eq!(
            proof_collection_drag_preview_title(&payload),
            Arc::from("2 selected assets")
        );
        assert_eq!(
            proof_collection_drag_preview_subtitle(&payload),
            Some(Arc::from("textures/stone/albedo.ktx2 + 1 more"))
        );
        assert_eq!(
            proof_collection_drop_status("Delivered", &payload),
            "Delivered 2 asset(s): textures/stone/albedo.ktx2, textures/shared/dust-mask.ktx2"
        );
    }

    #[test]
    fn proof_collection_drag_payload_for_unselected_asset_carries_dragged_asset_only() {
        let assets = authoring_parity_collection_assets()
            .iter()
            .cloned()
            .collect::<Vec<_>>();
        let selection = selection_state(&["stone-albedo", "dust-mask"], Some("stone-albedo"));
        let dragged = assets
            .iter()
            .find(|asset| asset.id.as_ref() == "stone-normal")
            .expect("fixture should include stone normal");

        let payload = proof_collection_drag_payload_for_asset(&assets, &selection, dragged);

        assert_eq!(payload.asset_ids.as_ref(), [Arc::from("stone-normal")]);
        assert_eq!(
            proof_collection_drag_preview_title(&payload),
            Arc::from("Stone Normal")
        );
        assert_eq!(
            proof_collection_drag_preview_subtitle(&payload),
            Some(Arc::from("textures/stone/normal.ktx2"))
        );
        assert_eq!(
            proof_collection_drop_status("Preview", &payload),
            "Preview 1 asset(s): textures/stone/normal.ktx2"
        );
    }
}
