use std::sync::OnceLock;

#[doc(hidden)]
pub fn released_blob_cache_entries() -> usize {
    static ENTRIES: OnceLock<usize> = OnceLock::new();
    *ENTRIES.get_or_init(|| {
        // Default: bounded on native builds. Retain recently released text blobs to reduce
        // `Text::prepare` thrash when wrap widths oscillate (e.g. interactive resize jitter).
        std::env::var("FRET_TEXT_RELEASED_BLOB_CACHE_ENTRIES")
            .ok()
            .and_then(|v| v.parse::<usize>().ok())
            // Allow disabling via env var (`0`).
            .unwrap_or(default_released_blob_cache_entries())
            .min(2048)
    })
}

fn default_released_blob_cache_entries() -> usize {
    // Keep wasm builds conservative; native builds get a bounded default to improve interactive
    // resize and other width-jitter workloads.
    #[cfg(target_arch = "wasm32")]
    {
        0
    }
    #[cfg(not(target_arch = "wasm32"))]
    {
        256
    }
}

#[doc(hidden)]
pub fn measure_shaping_cache_entries() -> usize {
    static ENTRIES: OnceLock<usize> = OnceLock::new();
    *ENTRIES.get_or_init(|| {
        // Default: 4096 entries. This cache is the main defense against `TextService::measure`
        // reshaping thrash when a layout pass touches many unique strings (common in editor
        // surfaces) and when wrap widths churn during interactive resize.
        std::env::var("FRET_TEXT_MEASURE_SHAPING_CACHE_ENTRIES")
            .ok()
            .and_then(|v| v.parse::<usize>().ok())
            .unwrap_or(4096)
            .clamp(64, 65_536)
    })
}

#[doc(hidden)]
pub fn prepared_shape_cache_entries() -> usize {
    static ENTRIES: OnceLock<usize> = OnceLock::new();
    *ENTRIES.get_or_init(|| {
        // Bound renderer-owned prepared layout/glyph shapes separately from measure-only shaping.
        // Native editor surfaces benefit from a larger window; wasm keeps the CPU heap footprint
        // tighter by default.
        std::env::var("FRET_TEXT_SHAPE_CACHE_ENTRIES")
            .ok()
            .and_then(|v| v.parse::<usize>().ok())
            .unwrap_or(default_prepared_shape_cache_entries())
            .clamp(64, 65_536)
    })
}

fn default_prepared_shape_cache_entries() -> usize {
    #[cfg(target_arch = "wasm32")]
    {
        1024
    }
    #[cfg(not(target_arch = "wasm32"))]
    {
        4096
    }
}

#[doc(hidden)]
pub fn glyph_atlas_max_pages() -> usize {
    static PAGES: OnceLock<usize> = OnceLock::new();
    *PAGES.get_or_init(|| {
        // Keep glyph residency as an explicit page budget instead of a renderer-local constant.
        // Each page is a large GPU texture, so clamp env overrides to a conservative range.
        std::env::var("FRET_TEXT_GLYPH_ATLAS_MAX_PAGES")
            .ok()
            .and_then(|v| v.parse::<usize>().ok())
            .unwrap_or(default_glyph_atlas_max_pages())
            .clamp(1, 16)
    })
}

fn default_glyph_atlas_max_pages() -> usize {
    #[cfg(target_arch = "wasm32")]
    {
        1
    }
    #[cfg(not(target_arch = "wasm32"))]
    {
        2
    }
}

#[doc(hidden)]
pub fn measure_shaping_cache_min_text_len_bytes() -> usize {
    static MIN_BYTES: OnceLock<usize> = OnceLock::new();
    *MIN_BYTES.get_or_init(|| {
        // Default: cache only "meaningfully expensive" paragraphs (e.g. long editor lines).
        //
        // Short UI labels (menus/tabs/buttons) are typically cheap to shape, and caching every
        // distinct label can bloat the cache and degrade cache locality across long-lived
        // reuse-launch perf suites.
        std::env::var("FRET_TEXT_MEASURE_SHAPING_CACHE_MIN_TEXT_LEN_BYTES")
            .ok()
            .and_then(|v| v.parse::<usize>().ok())
            .unwrap_or(128)
            .min(1_048_576)
    })
}
