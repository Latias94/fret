//! UI sugar for `fret-ui-assets`.
//!
//! This module is intentionally optional (feature-flagged) to keep `fret-ui-assets` usable in
//! non-`fret-ui` contexts while still providing ViewCache-safe ergonomics for UI authors.

use fret_ui::{ElementContext, Invalidation, UiHost};

use crate::image_source::{
    register_asset_key_for_source, ImageSource, ImageSourceOptions, ImageSourceState,
    with_image_source_loader,
};
use crate::image_asset_cache::ImageAssetKey;

pub trait ImageSourceElementContextExt {
    fn use_image_source_state(&mut self, source: &ImageSource) -> ImageSourceState;

    fn use_image_source_state_with_options(
        &mut self,
        source: &ImageSource,
        options: ImageSourceOptions,
    ) -> ImageSourceState;
}

impl<H: UiHost> ImageSourceElementContextExt for ElementContext<'_, H> {
    fn use_image_source_state(&mut self, source: &ImageSource) -> ImageSourceState {
        self.use_image_source_state_with_options(source, ImageSourceOptions::default())
    }

    fn use_image_source_state_with_options(
        &mut self,
        source: &ImageSource,
        options: ImageSourceOptions,
    ) -> ImageSourceState {
        // ViewCache correctness:
        //
        // - observe a per-request model that is updated when async decode completes (inbox drainer),
        // - bump the same model on GPU-ready events (via `UiAssets::handle_event` integration).
        //
        // Without these observations, a view-cached subtree may never re-render even though the
        // app requests redraws.
        if let Some(model) = with_image_source_loader(self.app, |loader, app| {
            loader.use_signal_model(app, source, options)
        }) {
            self.observe_model(&model, Invalidation::Paint);
        }

        // Fast path: RGBA8 sources skip async decode, so we register the key→signal mapping here.
        if let Some((width, height, rgba, color_space)) = source.rgba8_meta() {
            let key = ImageAssetKey::from_rgba8(width, height, color_space, rgba.as_ref());
            register_asset_key_for_source(self.app, source, options, key);
        }

        crate::use_image_source_state_with_options(self.app, self.window, source, options)
    }
}
