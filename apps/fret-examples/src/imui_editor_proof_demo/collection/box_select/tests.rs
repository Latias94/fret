use super::super::authoring_parity_collection_assets;
use super::*;

fn selected_ids(selection: &ImUiMultiSelectState<Arc<str>>) -> Vec<&str> {
    selection.selected().iter().map(|id| id.as_ref()).collect()
}

fn anchor_id(selection: &ImUiMultiSelectState<Arc<str>>) -> Option<&str> {
    selection.anchor().map(|id| id.as_ref())
}

#[test]
fn proof_collection_box_select_replace_uses_visible_collection_order() {
    let assets = authoring_parity_collection_assets();
    let collection_keys = assets
        .iter()
        .map(|asset| asset.id.clone())
        .collect::<Vec<_>>();
    let rendered_items = vec![
        ProofCollectionRenderedItem {
            id: Arc::from("stone-orm"),
            local_bounds: Rect::new(
                Point::new(Px(112.0), Px(0.0)),
                Size::new(Px(96.0), Px(72.0)),
            ),
        },
        ProofCollectionRenderedItem {
            id: Arc::from("stone-albedo"),
            local_bounds: Rect::new(Point::new(Px(0.0), Px(0.0)), Size::new(Px(96.0), Px(72.0))),
        },
        ProofCollectionRenderedItem {
            id: Arc::from("stone-normal"),
            local_bounds: Rect::new(Point::new(Px(0.0), Px(84.0)), Size::new(Px(96.0), Px(72.0))),
        },
    ];
    let session = ProofCollectionBoxSelectSession {
        pointer_id: PointerId(0),
        origin_local: Point::new(Px(4.0), Px(4.0)),
        current_local: Point::new(Px(124.0), Px(152.0)),
        baseline_selected: vec![Arc::from("dust-mask")],
        append_mode: false,
        threshold_met: true,
    };

    let selection =
        proof_collection_box_select_selection(&collection_keys, &rendered_items, &session);

    assert_eq!(
        selected_ids(&selection),
        vec!["stone-albedo", "stone-normal", "stone-orm",]
    );
    assert_eq!(anchor_id(&selection), Some("stone-albedo"));
}

#[test]
fn proof_collection_box_select_append_preserves_baseline_and_adds_hits() {
    let collection_keys = authoring_parity_collection_assets()
        .iter()
        .map(|asset| asset.id.clone())
        .collect::<Vec<_>>();
    let hits = vec![Arc::from("stone-albedo"), Arc::from("stone-orm")];

    let selection = proof_collection_box_select_state_for_hits(
        &collection_keys,
        &[Arc::from("dust-mask")],
        &hits,
        true,
    );

    assert_eq!(
        selected_ids(&selection),
        vec!["stone-albedo", "stone-orm", "dust-mask",]
    );
    assert_eq!(anchor_id(&selection), Some("stone-albedo"));
}
