use fret_assets::AssetRequest;
use fret_runtime::{FontFamilyDefaultsPolicy, RendererFontSourceLane};

use super::{WinitAppDriver, WinitRunner};

impl<D: WinitAppDriver> WinitRunner<D> {
    pub(super) fn handle_text_add_font_assets(&mut self, requests: Vec<AssetRequest>) {
        let Some(renderer) = self.renderer.as_mut() else {
            return;
        };

        let added = crate::runner::font_catalog::inject_font_asset_requests_and_refresh_catalog(
            &mut self.app,
            renderer,
            requests,
            RendererFontSourceLane::AssetRequest,
            FontFamilyDefaultsPolicy::None,
        );
        if added == 0 {
            return;
        }

        self.request_redraw_all_windows();
    }

    pub(super) fn handle_text_rescan_system_fonts(&mut self) {
        self.request_system_font_rescan();
    }
}
