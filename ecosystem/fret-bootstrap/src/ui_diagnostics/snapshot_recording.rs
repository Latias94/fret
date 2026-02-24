use super::*;

pub(super) fn changed_model_sources_top(
    app: &App,
    changed_models: &[u64],
) -> Vec<UiChangedModelSourceHotspotV1> {
    if !cfg!(debug_assertions) || changed_models.is_empty() {
        return Vec::new();
    }

    let mut counts: HashMap<(String, String, u32, u32), u32> = HashMap::new();
    for &model in changed_models {
        let id = ModelId::from(KeyData::from_ffi(model));
        let Some(info) = app.models().debug_last_changed_info_for_id(id) else {
            continue;
        };
        let ty = info.type_name.to_string();
        *counts
            .entry((ty, info.file.to_string(), info.line, info.column))
            .or_insert(0) += 1;
    }

    let mut out: Vec<UiChangedModelSourceHotspotV1> = counts
        .into_iter()
        .map(
            |((type_name, file, line, column), count)| UiChangedModelSourceHotspotV1 {
                type_name,
                changed_at: UiSourceLocationV1 { file, line, column },
                count,
            },
        )
        .collect();
    out.sort_by(|a, b| {
        b.count
            .cmp(&a.count)
            .then_with(|| a.type_name.cmp(&b.type_name))
            .then_with(|| a.changed_at.file.cmp(&b.changed_at.file))
            .then_with(|| a.changed_at.line.cmp(&b.changed_at.line))
            .then_with(|| a.changed_at.column.cmp(&b.changed_at.column))
    });
    out.truncate(8);
    out
}

#[cfg(feature = "preload-icon-svgs")]
pub(super) fn icon_svg_cache_stats(app: &App) -> Option<UiRetainedSvgCacheStatsV1> {
    let stats = app.global::<fret_ui_kit::declarative::icon::IconSvgPreloadDiagnostics>()?;
    let entries = stats.entries;
    let bytes_ready = stats.bytes_ready;
    let register_calls = stats.register_calls;
    Some(UiRetainedSvgCacheStatsV1 {
        entries,
        bytes_ready,
        stats: UiCacheStatsV1 {
            prepare_calls: register_calls,
            ..Default::default()
        },
    })
}

#[cfg(not(feature = "preload-icon-svgs"))]
pub(super) fn icon_svg_cache_stats(_app: &App) -> Option<UiRetainedSvgCacheStatsV1> {
    None
}

pub(super) fn canvas_cache_stats_for_window(app: &App, window: u64) -> Vec<UiCanvasCacheEntryV1> {
    let Some(registry) = app.global::<fret_canvas::diagnostics::CanvasCacheStatsRegistry>() else {
        return Vec::new();
    };

    registry
        .iter()
        .filter_map(|(key, snap)| {
            ((key.window == window) || (key.window == 0)).then_some((key, snap))
        })
        .map(|(key, snap)| UiCanvasCacheEntryV1 {
            node: key.node,
            name: key.name.to_string(),
            path: snap.path.map(|s| UiCacheKindSnapshotV1 {
                entries: s.entries,
                bytes_ready: s.bytes_ready,
                stats: UiCacheStatsV1 {
                    get_calls: s.stats.get_calls,
                    get_hits: s.stats.get_hits,
                    get_misses: s.stats.get_misses,
                    prepare_calls: s.stats.prepare_calls,
                    prepare_hits: s.stats.prepare_hits,
                    prepare_misses: s.stats.prepare_misses,
                    prune_calls: s.stats.prune_calls,
                    clear_calls: s.stats.clear_calls,
                    evict_calls: s.stats.evict_calls,
                    release_replaced: s.stats.release_replaced,
                    release_prune_age: s.stats.release_prune_age,
                    release_prune_budget: s.stats.release_prune_budget,
                    release_clear: s.stats.release_clear,
                    release_evict: s.stats.release_evict,
                },
            }),
            svg: snap.svg.map(|s| UiCacheKindSnapshotV1 {
                entries: s.entries,
                bytes_ready: s.bytes_ready,
                stats: UiCacheStatsV1 {
                    get_calls: s.stats.get_calls,
                    get_hits: s.stats.get_hits,
                    get_misses: s.stats.get_misses,
                    prepare_calls: s.stats.prepare_calls,
                    prepare_hits: s.stats.prepare_hits,
                    prepare_misses: s.stats.prepare_misses,
                    prune_calls: s.stats.prune_calls,
                    clear_calls: s.stats.clear_calls,
                    evict_calls: s.stats.evict_calls,
                    release_replaced: s.stats.release_replaced,
                    release_prune_age: s.stats.release_prune_age,
                    release_prune_budget: s.stats.release_prune_budget,
                    release_clear: s.stats.release_clear,
                    release_evict: s.stats.release_evict,
                },
            }),
            text: snap.text.map(|s| UiCacheKindSnapshotV1 {
                entries: s.entries,
                bytes_ready: s.bytes_ready,
                stats: UiCacheStatsV1 {
                    get_calls: s.stats.get_calls,
                    get_hits: s.stats.get_hits,
                    get_misses: s.stats.get_misses,
                    prepare_calls: s.stats.prepare_calls,
                    prepare_hits: s.stats.prepare_hits,
                    prepare_misses: s.stats.prepare_misses,
                    prune_calls: s.stats.prune_calls,
                    clear_calls: s.stats.clear_calls,
                    evict_calls: s.stats.evict_calls,
                    release_replaced: s.stats.release_replaced,
                    release_prune_age: s.stats.release_prune_age,
                    release_prune_budget: s.stats.release_prune_budget,
                    release_clear: s.stats.release_clear,
                    release_evict: s.stats.release_evict,
                },
            }),
            scene_op_tiles: snap.scene_op_tiles.map(|s| UiSceneOpTileCacheSnapshotV1 {
                entries: s.entries,
                requested_tiles: s.requested_tiles,
                budget_limit: s.budget_limit,
                budget_used: s.budget_used,
                skipped_tiles: s.skipped_tiles,
                stats: UiSceneOpTileCacheStatsV1 {
                    calls: s.stats.calls,
                    hits: s.stats.hits,
                    misses: s.stats.misses,
                    stored_tiles: s.stats.stored_tiles,
                    recorded_ops: s.stats.recorded_ops,
                    replayed_ops: s.stats.replayed_ops,
                    clear_calls: s.stats.clear_calls,
                    prune_calls: s.stats.prune_calls,
                    evict_calls: s.stats.evict_calls,
                    evict_prune_age: s.stats.evict_prune_age,
                    evict_prune_budget: s.stats.evict_prune_budget,
                },
            }),
            work_budget: snap.work_budget.map(|b| UiWorkBudgetSnapshotV1 {
                requested_units: b.requested_units,
                limit: b.limit,
                used: b.used,
                skipped_units: b.skipped_units,
            }),
        })
        .collect()
}

pub(super) fn resource_caches_for_window(
    app: &App,
    window: u64,
    redact_text: bool,
    max_debug_string_bytes: usize,
) -> Option<UiResourceCachesV1> {
    let icon_svg_cache = icon_svg_cache_stats(app);
    let canvas = canvas_cache_stats_for_window(app, window);
    let render_text = app
        .global::<fret_core::RendererTextPerfSnapshot>()
        .copied()
        .map(UiRendererTextPerfSnapshotV1::from_core);
    let render_text_font_trace = app
        .global::<fret_core::RendererTextFontTraceSnapshot>()
        .cloned()
        .map(|s| {
            UiRendererTextFontTraceSnapshotV1::from_core(s, redact_text, max_debug_string_bytes)
        });
    let render_text_fallback_policy = app
        .global::<fret_core::RendererTextFallbackPolicySnapshot>()
        .cloned()
        .map(|s| UiRendererTextFallbackPolicySnapshotV1::from_core(s, max_debug_string_bytes));

    (icon_svg_cache.is_some()
        || !canvas.is_empty()
        || render_text.is_some()
        || render_text_font_trace.is_some()
        || render_text_fallback_policy.is_some())
    .then_some(UiResourceCachesV1 {
        icon_svg_cache,
        canvas,
        render_text,
        render_text_font_trace,
        render_text_fallback_policy,
    })
}
