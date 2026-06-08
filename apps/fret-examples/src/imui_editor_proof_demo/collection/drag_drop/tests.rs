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
