use std::collections::HashMap;
use std::sync::Arc;

use fret::imui::kit::ImUiMultiSelectState;
use fret_core::{Point, PointerId, Px, Rect, Size};

use super::geometry::{proof_collection_drag_rect, proof_collection_rects_intersect};

#[derive(Debug, Clone, PartialEq)]
pub(super) struct ProofCollectionRenderedItem {
    pub(super) id: Arc<str>,
    pub(super) local_bounds: Rect,
}

#[derive(Debug, Clone, PartialEq)]
pub(super) struct ProofCollectionBoxSelectSession {
    pub(super) pointer_id: PointerId,
    pub(super) origin_local: Point,
    pub(super) current_local: Point,
    pub(super) baseline_selected: Vec<Arc<str>>,
    pub(super) append_mode: bool,
    pub(super) threshold_met: bool,
}

#[derive(Debug, Clone, Default, PartialEq)]
pub(super) struct ProofCollectionBoxSelectState {
    pub(super) session: Option<ProofCollectionBoxSelectSession>,
}

fn proof_collection_box_select_hits(
    collection_keys: &[Arc<str>],
    rendered_items: &[ProofCollectionRenderedItem],
    drag_rect: Rect,
) -> Vec<Arc<str>> {
    let bounds_by_id = rendered_items
        .iter()
        .map(|item| (item.id.as_ref(), item.local_bounds))
        .collect::<HashMap<_, _>>();

    collection_keys
        .iter()
        .filter(|key| {
            bounds_by_id
                .get(key.as_ref())
                .is_some_and(|bounds| proof_collection_rects_intersect(*bounds, drag_rect))
        })
        .cloned()
        .collect()
}

fn proof_collection_box_select_state_for_hits(
    collection_keys: &[Arc<str>],
    baseline_selected: &[Arc<str>],
    hits: &[Arc<str>],
    append_mode: bool,
) -> ImUiMultiSelectState<Arc<str>> {
    let selected = if append_mode {
        let mut merged = baseline_selected.to_vec();
        for hit in hits {
            if !merged.iter().any(|item| item == hit) {
                merged.push(hit.clone());
            }
        }
        merged
    } else {
        hits.to_vec()
    };

    ImUiMultiSelectState::from_ordered_selection(collection_keys, selected, None)
}

pub(super) fn proof_collection_box_select_selection(
    collection_keys: &[Arc<str>],
    rendered_items: &[ProofCollectionRenderedItem],
    session: &ProofCollectionBoxSelectSession,
) -> ImUiMultiSelectState<Arc<str>> {
    let drag_rect = proof_collection_drag_rect(session.origin_local, session.current_local);
    let hits = proof_collection_box_select_hits(collection_keys, rendered_items, drag_rect);
    proof_collection_box_select_state_for_hits(
        collection_keys,
        &session.baseline_selected,
        &hits,
        session.append_mode,
    )
}

pub(super) fn proof_collection_box_select_active_rect(
    state: &ProofCollectionBoxSelectState,
) -> Option<Rect> {
    let session = state.session.as_ref()?;
    session
        .threshold_met
        .then(|| proof_collection_drag_rect(session.origin_local, session.current_local))
}

#[cfg(test)]
mod tests {
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
                local_bounds: Rect::new(
                    Point::new(Px(0.0), Px(0.0)),
                    Size::new(Px(96.0), Px(72.0)),
                ),
            },
            ProofCollectionRenderedItem {
                id: Arc::from("stone-normal"),
                local_bounds: Rect::new(
                    Point::new(Px(0.0), Px(84.0)),
                    Size::new(Px(96.0), Px(72.0)),
                ),
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
}
