use fret_core::ImageId;
use fret_ui::{ElementContext, UiHost};
use fret_ui_assets::{ImageSource, ui::ImageSourceElementContextExt as _};
use std::path::PathBuf;
use std::sync::{Arc, OnceLock};

pub(crate) fn landscape_image_id<H: UiHost>(cx: &mut ElementContext<'_, H>) -> Option<ImageId> {
    #[cfg(not(target_arch = "wasm32"))]
    {
        static SOURCE: OnceLock<Option<ImageSource>> = OnceLock::new();
        let source = SOURCE.get_or_init(|| {
            let path = PathBuf::from(env!("CARGO_MANIFEST_DIR"))
                .join("../../assets/textures/aspect-ratio-landscape.jpg");
            path.exists()
                .then(|| ImageSource::from_path(Arc::new(path)))
        });
        let state = source
            .as_ref()
            .map(|source| cx.use_image_source_state(source));
        return state.and_then(|state| state.image);
    }

    #[cfg(target_arch = "wasm32")]
    {
        static SOURCE: OnceLock<ImageSource> = OnceLock::new();
        let source = SOURCE.get_or_init(|| {
            ImageSource::from_url(Arc::<str>::from("textures/aspect-ratio-landscape.jpg"))
        });
        return cx.use_image_source_state(source).image;
    }
}

pub(crate) fn portrait_image_id<H: UiHost>(cx: &mut ElementContext<'_, H>) -> Option<ImageId> {
    #[cfg(not(target_arch = "wasm32"))]
    {
        static SOURCE: OnceLock<Option<ImageSource>> = OnceLock::new();
        let source = SOURCE.get_or_init(|| {
            let path = PathBuf::from(env!("CARGO_MANIFEST_DIR"))
                .join("../../assets/textures/aspect-ratio-portrait.jpg");
            path.exists()
                .then(|| ImageSource::from_path(Arc::new(path)))
        });
        let state = source
            .as_ref()
            .map(|source| cx.use_image_source_state(source));
        return state.and_then(|state| state.image);
    }

    #[cfg(target_arch = "wasm32")]
    {
        static SOURCE: OnceLock<ImageSource> = OnceLock::new();
        let source = SOURCE.get_or_init(|| {
            ImageSource::from_url(Arc::<str>::from("textures/aspect-ratio-portrait.jpg"))
        });
        return cx.use_image_source_state(source).image;
    }
}
