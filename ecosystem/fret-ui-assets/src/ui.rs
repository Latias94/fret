//! UI sugar for `fret-ui-assets`.
//!
//! This module is intentionally optional (feature-flagged) to keep `fret-ui-assets` usable in
//! non-`fret-ui` contexts while still providing ViewCache-safe ergonomics for UI authors.

use fret_ui::{ElementContext, Invalidation, UiHost};

use crate::UiAssetsReloadEpoch;
use crate::image_asset_cache::ImageAssetKey;
use crate::image_source::{
    ImageSource, ImageSourceOptions, ImageSourceState, register_asset_key_for_source,
    with_image_source_loader,
};
#[cfg(not(target_arch = "wasm32"))]
use crate::svg_file::{SvgFileSource, svg_source_from_file_cached};

pub trait ImageSourceElementContextExt {
    fn use_image_source_state(&mut self, source: &ImageSource) -> ImageSourceState;

    fn use_image_source_state_with_options(
        &mut self,
        source: &ImageSource,
        options: ImageSourceOptions,
    ) -> ImageSourceState;

    /// Like [`Self::use_image_source_state_with_options`], but allows callers to choose the
    /// invalidation kind used for the underlying async decode/GPU-ready signal.
    ///
    /// Use `Invalidation::Layout` when the returned [`ImageSourceState`] influences layout (for
    /// example, when you explicitly size to `intrinsic_size_px`).
    fn use_image_source_state_with_options_and_invalidation(
        &mut self,
        source: &ImageSource,
        options: ImageSourceOptions,
        invalidation: Invalidation,
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
        self.use_image_source_state_with_options_and_invalidation(
            source,
            options,
            Invalidation::Paint,
        )
    }

    fn use_image_source_state_with_options_and_invalidation(
        &mut self,
        source: &ImageSource,
        options: ImageSourceOptions,
        invalidation: Invalidation,
    ) -> ImageSourceState {
        // ViewCache correctness for dev reloads: observe a global epoch that can be bumped when
        // path-based assets should be reloaded (without restarting the app).
        self.observe_global::<UiAssetsReloadEpoch>(invalidation);

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
            self.observe_model(&model, invalidation);
        }

        // Fast path: RGBA8 sources skip async decode, so we register the key→signal mapping here.
        if let Some((width, height, rgba, color_space)) = source.rgba8_meta() {
            let key = ImageAssetKey::from_rgba8(width, height, color_space, rgba.as_ref());
            register_asset_key_for_source(self.app, source, options, key);
        }

        crate::use_image_source_state_with_options(self.app, self.window, source, options)
    }
}

#[cfg(not(target_arch = "wasm32"))]
pub trait SvgFileElementContextExt {
    fn svg_source_from_file(&mut self, source: &SvgFileSource) -> Option<fret_ui::SvgSource>;

    /// Like [`Self::svg_source_from_file`], but allows callers to choose the invalidation kind
    /// used for the dev reload epoch signal.
    fn svg_source_from_file_with_invalidation(
        &mut self,
        source: &SvgFileSource,
        invalidation: Invalidation,
    ) -> Option<fret_ui::SvgSource>;
}

#[cfg(not(target_arch = "wasm32"))]
impl<H: UiHost> SvgFileElementContextExt for ElementContext<'_, H> {
    fn svg_source_from_file(&mut self, source: &SvgFileSource) -> Option<fret_ui::SvgSource> {
        self.svg_source_from_file_with_invalidation(source, Invalidation::Paint)
    }

    fn svg_source_from_file_with_invalidation(
        &mut self,
        source: &SvgFileSource,
        invalidation: Invalidation,
    ) -> Option<fret_ui::SvgSource> {
        self.observe_global::<UiAssetsReloadEpoch>(invalidation);
        svg_source_from_file_cached(self.app, source).ok()
    }
}
