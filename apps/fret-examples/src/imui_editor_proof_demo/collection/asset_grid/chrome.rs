use std::sync::Arc;

use fret::imui::kit;
use fret_ui_kit::recipes::imui_drag_preview::DragPreviewGhostOptions;

use super::super::ProofCollectionAsset;
use super::super::geometry::ProofCollectionLayoutMetrics;

pub(super) fn collection_asset_grid_options(
    layout: ProofCollectionLayoutMetrics,
) -> kit::GridOptions {
    kit::GridOptions {
        columns: layout.columns,
        column_gap: fret_ui_kit::MetricRef::space(fret_ui_kit::Space::N2),
        row_gap: fret_ui_kit::MetricRef::space(fret_ui_kit::Space::N2),
        row_items: fret_ui_kit::Items::Stretch,
        test_id: Some(Arc::from(
            "imui-editor-proof.authoring.imui.collection.grid",
        )),
        ..Default::default()
    }
}

pub(super) fn collection_asset_tile_options(
    asset: &ProofCollectionAsset,
    layout: ProofCollectionLayoutMetrics,
) -> kit::VerticalOptions {
    kit::VerticalOptions {
        layout: fret_ui_kit::LayoutRefinement::default()
            .flex_1()
            .min_h(layout.tile_min_height),
        gap: fret_ui_kit::MetricRef::space(fret_ui_kit::Space::N1),
        test_id: Some(Arc::from(format!(
            "imui-editor-proof.authoring.imui.collection.asset.{}",
            asset.id
        ))),
        ..Default::default()
    }
}

pub(super) fn collection_asset_selectable_options(
    asset: &ProofCollectionAsset,
) -> kit::SelectableOptions {
    kit::SelectableOptions {
        focusable: false,
        test_id: Some(Arc::from(format!(
            "imui-editor-proof.authoring.imui.collection.asset.{}.select",
            asset.id
        ))),
        ..Default::default()
    }
}

pub(super) fn collection_asset_ghost_id(asset: &ProofCollectionAsset) -> String {
    format!(
        "imui-editor-proof.authoring.imui.collection.asset.{}.ghost",
        asset.id
    )
}

pub(super) fn collection_asset_ghost_options(
    asset: &ProofCollectionAsset,
) -> DragPreviewGhostOptions {
    DragPreviewGhostOptions {
        test_id: Some(Arc::from(collection_asset_ghost_id(asset))),
        ..Default::default()
    }
}
