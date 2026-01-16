#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct UiGalleryWebError;

impl std::fmt::Display for UiGalleryWebError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.write_str("fret-ui-gallery-web is only available on wasm32")
    }
}

impl std::error::Error for UiGalleryWebError {}
