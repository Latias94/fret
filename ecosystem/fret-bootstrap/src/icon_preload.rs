use std::collections::hash_map::DefaultHasher;
use std::hash::{Hash, Hasher};

use fret_app::App;
use fret_core::UiServices;
use fret_icons::{IconId, IconRegistry, ResolvedSvgOwned, MISSING_ICON_SVG};

use fret_canvas::cache::CacheStats;
use fret_canvas::cache::{SvgBytes, SvgCache};
use fret_canvas::diagnostics::{CanvasCacheKey, CanvasCacheStatsRegistry};

#[derive(Debug, Default)]
pub struct PreloadedIconSvgCache {
    cache: SvgCache,
}

impl PreloadedIconSvgCache {
    pub(crate) fn diagnostics_snapshot(&self) -> (usize, u64, CacheStats) {
        (
            self.cache.len(),
            self.cache.bytes_ready(),
            self.cache.stats(),
        )
    }
}

fn icon_svg_cache_key(icon: &IconId) -> u64 {
    let mut hasher = DefaultHasher::new();
    "fret.bootstrap.icon_svg".hash(&mut hasher);
    icon.hash(&mut hasher);
    hasher.finish()
}

fn missing_svg_cache_key() -> u64 {
    let mut hasher = DefaultHasher::new();
    "fret.bootstrap.icon_svg.missing".hash(&mut hasher);
    hasher.finish()
}

/// Pre-register all SVG icons in the global `IconRegistry` and store their `SvgId`s in the
/// `fret-ui-kit` `IconSvgRegistry` global.
///
/// Unlike the `fret-ui-kit` helper, this uses `fret-canvas` `SvgCache` so repeated calls replace and
/// unregister old `SvgId`s instead of leaking registered SVGs.
pub fn preload_icon_svgs(app: &mut App, services: &mut dyn UiServices) {
    let resolved: Vec<(IconId, ResolvedSvgOwned)> = app
        .with_global_mut(IconRegistry::default, |icons, _app| {
            icons.collect_resolved_owned()
        });

    app.with_global_mut(PreloadedIconSvgCache::default, |cache, app| {
        cache.cache.clear(services);
        cache.cache.begin_frame();
        let missing = cache.cache.prepare(
            services,
            missing_svg_cache_key(),
            SvgBytes::Static(MISSING_ICON_SVG),
        );

        app.with_global_mut(
            fret_ui_kit::declarative::icon::IconSvgRegistry::default,
            |registry, _app| {
                registry.clear();
                registry.set_missing(missing);

                for (icon, svg) in resolved {
                    let bytes = match svg {
                        ResolvedSvgOwned::Static(bytes) => SvgBytes::Static(bytes),
                        ResolvedSvgOwned::Bytes(bytes) => SvgBytes::Bytes(bytes),
                    };
                    let id = cache
                        .cache
                        .prepare(services, icon_svg_cache_key(&icon), bytes);
                    registry.insert(icon, id);
                }
            },
        );

        let frame_id = app.frame_id().0;
        let entries = cache.cache.len();
        let bytes_ready = cache.cache.bytes_ready();
        let stats = cache.cache.stats();
        let key = CanvasCacheKey {
            window: 0,
            node: 0,
            name: "fret-bootstrap.icon_svgs",
        };
        app.with_global_mut(CanvasCacheStatsRegistry::default, |registry, _app| {
            registry.record_svg_cache(key, frame_id, entries, bytes_ready, stats);
        });
    });
}
