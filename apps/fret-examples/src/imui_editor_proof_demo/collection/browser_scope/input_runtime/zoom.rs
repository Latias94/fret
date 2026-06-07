use std::sync::Arc;

use fret_core::Px;
use fret_runtime::Model;
use fret_ui::ElementContext;
use fret_ui::action::UiActionHostExt as _;
use fret_ui::scroll::ScrollHandle;

use super::super::super::KernelApp;
use super::super::super::geometry::{ProofCollectionLayoutMetrics, proof_collection_zoom_request};

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

        let _ = host.update_model(&collection_zoom_model, |state| {
            *state = update.next_tile_extent;
        });
        collection_scroll_handle.set_offset(update.next_scroll_offset);
        host.notify(acx);
        true
    }));
}
