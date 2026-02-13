use fret_core::{AppWindowId, Event};
use fret_runtime::{EffectSink, GlobalsHost, ModelHost, TimeHost};

use crate::image_asset_cache::{ImageAssetCacheHostExt, ImageAssetStats};
use crate::svg_asset_cache::{SvgAssetCacheHostExt, SvgAssetStats};

#[derive(Debug, Clone, Copy)]
pub struct UiAssetsBudgets {
    pub image_budget_bytes: u64,
    pub image_max_ready_entries: usize,
    pub svg_budget_bytes: u64,
    pub svg_max_ready_entries: usize,
}

impl Default for UiAssetsBudgets {
    fn default() -> Self {
        Self {
            image_budget_bytes: 64 * 1024 * 1024,
            image_max_ready_entries: 4096,
            svg_budget_bytes: 16 * 1024 * 1024,
            svg_max_ready_entries: 4096,
        }
    }
}

/// A tiny facade over the UI render-asset caches (images + SVGs).
///
/// This stays aligned with ADR 0004: assets are registered via effects at flush points and
/// referenced by stable IDs. This type only provides ergonomic wiring helpers.
pub struct UiAssets;

impl UiAssets {
    /// Ensure the global caches exist and apply budgets.
    pub fn configure<H: GlobalsHost>(host: &mut H, budgets: UiAssetsBudgets) {
        host.with_image_asset_cache(|cache, _host| {
            cache.set_budget_bytes(budgets.image_budget_bytes);
            cache.set_max_ready_entries(budgets.image_max_ready_entries);
        });

        host.with_svg_asset_cache(|cache, _host| {
            cache.set_budget_bytes(budgets.svg_budget_bytes);
            cache.set_max_ready_entries(budgets.svg_max_ready_entries);
        });
    }

    /// Drive the image cache state machine using the event pipeline.
    ///
    /// Notes:
    /// - SVG assets are registered directly through `UiServices::svg()` (no event driving needed).
    /// - This call is cheap for unrelated events.
    pub fn handle_event<H: GlobalsHost + TimeHost + EffectSink + ModelHost>(
        host: &mut H,
        window: AppWindowId,
        event: &Event,
    ) -> bool {
        host.with_image_asset_cache(|cache, host| {
            #[cfg(feature = "ui")]
            let key = match event {
                Event::ImageRegistered { token, .. } | Event::ImageRegisterFailed { token, .. } => {
                    cache.key_for_token(*token)
                }
                _ => None,
            };

            let changed = cache.handle_event(host, window, event);

            #[cfg(feature = "ui")]
            if let Some(key) = key {
                crate::image_source::notify_image_asset_key(host, key);
            }

            changed
        })
    }

    pub fn image_stats<H: GlobalsHost>(host: &mut H) -> ImageAssetStats {
        host.with_image_asset_cache(|cache, _host| cache.stats())
    }

    pub fn svg_stats<H: GlobalsHost>(host: &mut H) -> SvgAssetStats {
        host.with_svg_asset_cache(|cache, _host| cache.stats())
    }
}
