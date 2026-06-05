use super::*;

use fret_ui_kit::imui::ImUiMultiSelectState;

#[derive(Clone, PartialEq, Eq)]
pub(super) struct TestCollectionAsset {
    pub(super) id: Arc<str>,
    pub(super) label: Arc<str>,
    pub(super) path: Arc<str>,
}

#[derive(Clone)]
pub(super) struct TestCollectionDragPayload {
    pub(super) ids: Arc<[Arc<str>]>,
    pub(super) paths: Arc<[Arc<str>]>,
}

pub(super) fn test_collection_assets() -> Arc<[TestCollectionAsset]> {
    vec![
        TestCollectionAsset {
            id: Arc::from("alpha"),
            label: Arc::from("Alpha"),
            path: Arc::from("textures/alpha.ktx2"),
        },
        TestCollectionAsset {
            id: Arc::from("beta"),
            label: Arc::from("Beta"),
            path: Arc::from("textures/beta.ktx2"),
        },
        TestCollectionAsset {
            id: Arc::from("gamma"),
            label: Arc::from("Gamma"),
            path: Arc::from("textures/gamma.ktx2"),
        },
        TestCollectionAsset {
            id: Arc::from("delta"),
            label: Arc::from("Delta"),
            path: Arc::from("textures/delta.ktx2"),
        },
    ]
    .into()
}

fn selected_test_collection_assets<'a>(
    assets: &'a [TestCollectionAsset],
    selection: &ImUiMultiSelectState<Arc<str>>,
) -> Vec<&'a TestCollectionAsset> {
    selection
        .selected()
        .iter()
        .filter_map(|id| assets.iter().find(|asset| asset.id == *id))
        .collect()
}

pub(super) fn test_collection_drag_payload_for_asset(
    assets: &[TestCollectionAsset],
    selection: &ImUiMultiSelectState<Arc<str>>,
    dragged: &TestCollectionAsset,
) -> TestCollectionDragPayload {
    let selected_assets = selected_test_collection_assets(assets, selection);
    let payload_assets = if selection.is_selected(&dragged.id) && !selected_assets.is_empty() {
        selected_assets
    } else {
        vec![dragged]
    };

    let ids = payload_assets
        .iter()
        .map(|asset| asset.id.clone())
        .collect::<Vec<_>>();
    let paths = payload_assets
        .iter()
        .map(|asset| asset.path.clone())
        .collect::<Vec<_>>();

    TestCollectionDragPayload {
        ids: ids.into(),
        paths: paths.into(),
    }
}

#[test]
fn collection_drag_payload_for_selected_asset_carries_selected_set() {
    let assets = test_collection_assets();
    let all_keys = assets
        .iter()
        .map(|asset| asset.id.clone())
        .collect::<Vec<_>>();
    let selection = ImUiMultiSelectState::from_ordered_selection(
        &all_keys,
        vec![Arc::from("beta"), Arc::from("delta")],
        Some(Arc::from("delta")),
    );

    let payload = test_collection_drag_payload_for_asset(assets.as_ref(), &selection, &assets[3]);

    assert_eq!(
        payload.ids.as_ref(),
        &[Arc::<str>::from("beta"), Arc::<str>::from("delta")]
    );
    assert_eq!(
        payload.paths.as_ref(),
        &[
            Arc::<str>::from("textures/beta.ktx2"),
            Arc::<str>::from("textures/delta.ktx2"),
        ]
    );
}

#[test]
fn collection_drag_payload_for_unselected_asset_carries_dragged_asset_only() {
    let assets = test_collection_assets();
    let all_keys = assets
        .iter()
        .map(|asset| asset.id.clone())
        .collect::<Vec<_>>();
    let selection = ImUiMultiSelectState::from_ordered_selection(
        &all_keys,
        vec![Arc::from("beta"), Arc::from("delta")],
        Some(Arc::from("delta")),
    );

    let payload = test_collection_drag_payload_for_asset(assets.as_ref(), &selection, &assets[0]);

    assert_eq!(payload.ids.as_ref(), &[Arc::<str>::from("alpha")]);
    assert_eq!(
        payload.paths.as_ref(),
        &[Arc::<str>::from("textures/alpha.ktx2")]
    );
}
