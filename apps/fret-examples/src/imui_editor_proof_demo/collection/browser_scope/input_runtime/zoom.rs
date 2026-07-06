use std::sync::Arc;

use fret_core::Px;
use fret_runtime::Model;
use fret_ui::ElementContext;
use fret_ui::scroll::ScrollHandle;

use super::super::super::KernelApp;
use super::super::super::geometry::{ProofCollectionLayoutMetrics, proof_collection_zoom_request};
use super::super::super::model_owner::ProofCollectionModelOwner;

pub(super) fn install_collection_browser_scope_zoom_runtime(
    cx: &mut ElementContext<'_, KernelApp>,
    collection_layout: ProofCollectionLayoutMetrics,
    collection_scroll_handle: ScrollHandle,
    collection_zoom_model: Model<Px>,
    collection_asset_count: usize,
) {
    cx.pointer_region_on_wheel(Arc::new(move |host, acx, wheel| {
        let Some(update) = proof_collection_zoom_request(
            collection_layout,
            collection_scroll_handle.offset(),
            wheel.position_local,
            wheel.delta,
            wheel.modifiers,
            collection_asset_count,
        ) else {
            return false;
        };

        ProofCollectionModelOwner::new(host.models_mut())
            .set_zoom_extent(&collection_zoom_model, update.next_tile_extent);
        collection_scroll_handle.set_offset(update.next_scroll_offset);
        host.notify(acx);
        true
    }));
}
