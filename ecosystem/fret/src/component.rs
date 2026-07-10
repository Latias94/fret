/// Component-facing UI asset helpers for generic `ElementContext<H>` snippets.
///
/// Use this lane when code is reusable component/snippet authoring rather than default app
/// render code. The default app lane remains `fret::app::ui_assets`.
#[cfg(feature = "ui-assets")]
pub mod ui_assets {
    pub use fret_core::{ImageColorSpace, ImageId};
    pub use fret_ui_assets::image_asset_cache::{ImageAssetKey, ImageAssetStats};
    pub use fret_ui_assets::image_asset_state::ImageLoadingStatus;
    pub use fret_ui_assets::image_source::{ImageSource, ImageSourceOptions, ImageSourceState};
    pub use fret_ui_assets::svg_asset_cache::SvgAssetStats;
    pub use fret_ui_assets::ui::SvgAssetSourceState;

    pub fn rgba8_image_state<H: fret_ui::UiHost>(
        cx: &mut fret_ui::ElementContext<'_, H>,
        width: u32,
        height: u32,
        rgba: &[u8],
        color_space: fret_core::ImageColorSpace,
    ) -> (
        ImageAssetKey,
        Option<fret_core::ImageId>,
        ImageLoadingStatus,
    ) {
        fret_ui_assets::ui::use_rgba8_image_state_in(cx, width, height, rgba, color_space)
    }

    pub fn image_source_state<H: fret_ui::UiHost>(
        cx: &mut fret_ui::ElementContext<'_, H>,
        source: &ImageSource,
    ) -> ImageSourceState {
        fret_ui_assets::ui::use_image_source_state_in(cx, source)
    }

    pub fn image_source_state_from_asset_request<H: fret_ui::UiHost>(
        cx: &mut fret_ui::ElementContext<'_, H>,
        request: &crate::assets::AssetRequest,
    ) -> ImageSourceState {
        fret_ui_assets::ui::use_image_source_state_from_asset_request_in(cx, request)
    }

    pub fn image_source_state_from_asset_locator<H: fret_ui::UiHost>(
        cx: &mut fret_ui::ElementContext<'_, H>,
        locator: crate::assets::AssetLocator,
    ) -> ImageSourceState {
        fret_ui_assets::ui::use_image_source_state_from_asset_locator_in(cx, locator)
    }

    pub fn svg_source_state_from_asset_request<H: fret_ui::UiHost>(
        cx: &mut fret_ui::ElementContext<'_, H>,
        request: &crate::assets::AssetRequest,
    ) -> SvgAssetSourceState {
        fret_ui_assets::ui::svg_source_state_from_asset_request_in(cx, request)
    }

    pub fn svg_source_state_from_asset_locator<H: fret_ui::UiHost>(
        cx: &mut fret_ui::ElementContext<'_, H>,
        locator: crate::assets::AssetLocator,
    ) -> SvgAssetSourceState {
        fret_ui_assets::ui::svg_source_state_from_asset_locator_in(cx, locator)
    }

    pub fn image_stats<H: fret_ui::UiHost>(
        cx: &mut fret_ui::ElementContext<'_, H>,
    ) -> ImageAssetStats {
        fret_ui_assets::ui::image_stats_in(cx)
    }

    pub fn svg_stats<H: fret_ui::UiHost>(cx: &mut fret_ui::ElementContext<'_, H>) -> SvgAssetStats {
        fret_ui_assets::ui::svg_stats_in(cx)
    }
}

/// Common imports for reusable component crates built on Fret.
pub mod prelude;
