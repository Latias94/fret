//! Snippet-backed Aspect Ratio examples (shadcn/ui v4).
//!
//! Each snippet is real, compiled Rust code:
//! - Preview renders by calling `render(cx, ...)`.
//! - Code tab shows the same file via `include_str!` (optionally region-sliced).
//!
//! Keep `ui-gallery-aspect-ratio-*` `test_id`s stable: diag scripts depend on them.

mod images;

pub mod composable_children;
pub mod demo;
pub mod portrait;
pub mod rtl;
pub mod square;
pub mod usage;

#[cfg(test)]
mod tests {
    const IMAGES_SOURCE: &str = include_str!("images.rs");
    const DEMO_SOURCE: &str = include_str!("demo.rs");
    const USAGE_SOURCE: &str = include_str!("usage.rs");
    const COMPOSABLE_SOURCE: &str = include_str!("composable_children.rs");
    const PORTRAIT_SOURCE: &str = include_str!("portrait.rs");
    const RTL_SOURCE: &str = include_str!("rtl.rs");
    const SQUARE_SOURCE: &str = include_str!("square.rs");

    #[test]
    fn aspect_ratio_gallery_previews_resolve_bundle_assets() {
        assert!(IMAGES_SOURCE.contains("use_image_source_state_from_asset_request"));
        assert!(IMAGES_SOURCE.contains("ui_gallery_aspect_ratio_landscape_request"));
        assert!(IMAGES_SOURCE.contains("ui_gallery_aspect_ratio_portrait_request"));
        assert!(IMAGES_SOURCE.contains("ui_gallery_profile_square_request"));
        assert!(!IMAGES_SOURCE.contains("ImageSource::rgba8("));

        for source in [
            DEMO_SOURCE,
            USAGE_SOURCE,
            COMPOSABLE_SOURCE,
            PORTRAIT_SOURCE,
            RTL_SOURCE,
            SQUARE_SOURCE,
        ] {
            assert!(!source.contains("ImageSource::rgba8("));
        }
    }
}
