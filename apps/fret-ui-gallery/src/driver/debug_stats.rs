use fret_app::{App, Model};
use fret_core::AppWindowId;
use fret_ui::UiTree;
use std::sync::Arc;

use super::{DebugHudState, inspector};

pub(crate) struct DebugHudBundle {
    pub(crate) show: bool,
    pub(crate) lines: Vec<Arc<str>>,
    pub(crate) inspector_status: Option<super::status_bar::InspectorStatus>,
}

pub(crate) fn compute_debug_hud_bundle(
    app: &mut App,
    ui: &UiTree<App>,
    window: AppWindowId,
    debug_hud: &mut DebugHudState,
    inspector_enabled: &Model<bool>,
    inspector_last_pointer: &Model<Option<fret_core::Point>>,
    debug_on: bool,
) -> DebugHudBundle {
    let last_debug_stats = ui.debug_stats();
    let frame_dt = if debug_on {
        debug_hud.tick(fret_core::time::Instant::now())
    } else {
        None
    };
    let fps = debug_hud.ema_fps();

    let last_cache_roots = ui.debug_cache_root_stats();
    let cache_root_breakdown: Option<Vec<Arc<str>>> = if !last_cache_roots.is_empty() {
        let total = last_cache_roots.len();
        let hits = last_cache_roots.iter().filter(|r| r.reused).count();
        let replayed_ops: u32 = last_cache_roots.iter().map(|r| r.paint_replayed_ops).sum();

        let mut lines: Vec<Arc<str>> = vec![Arc::from(format!(
            "cache_roots total={total} hits={hits} replayed_ops={replayed_ops}"
        ))];

        let max_items = 3usize;
        for (index, root) in last_cache_roots.iter().take(max_items).enumerate() {
            let element_path = root.element.and_then(|element| {
                app.with_global_mut_untracked(fret_ui::ElementRuntime::new, |runtime, _| {
                    runtime.debug_path_for_element(window, element)
                })
            });

            lines.push(Arc::from(format!(
                "cache_root[{index}] node={:?} reused={} contained_layout={} replayed_ops={} el={} {}",
                root.root,
                root.reused as u8,
                root.contained_layout as u8,
                root.paint_replayed_ops,
                root.element
                    .map(|id| format!("{:#x}", id.0))
                    .unwrap_or_else(|| "<none>".to_string()),
                element_path.as_deref().unwrap_or(""),
            )));
        }

        Some(lines)
    } else {
        None
    };
    let hot_model_breakdown: Option<Arc<str>> = {
        let hotspots = ui.debug_model_change_hotspots();
        if hotspots.is_empty() {
            None
        } else {
            let mut line = String::from("hot_models");
            for hs in hotspots.iter().take(3) {
                line.push(' ');
                line.push_str(&format!("{:?}={}", hs.model, hs.observation_edges));
            }
            Some(Arc::from(line))
        }
    };
    let unobserved_model_breakdown: Option<Arc<str>> = {
        let unobserved = ui.debug_model_change_unobserved();
        if unobserved.is_empty() {
            None
        } else {
            let mut line = format!(
                "unobs_models={}",
                ui.debug_stats().model_change_unobserved_models
            );
            for entry in unobserved.iter().take(3) {
                let type_name = entry.created.map(|c| c.type_name).unwrap_or("<unknown>");
                let type_name = type_name.rsplit("::").next().unwrap_or(type_name);
                line.push(' ');
                line.push_str(&format!("{:?}={}", entry.model, type_name));
            }
            Some(Arc::from(line))
        }
    };

    let show = debug_on;
    let mut lines: Vec<Arc<str>> = if show {
        let mut lines: Vec<Arc<str>> = Vec::new();

        lines.push(Arc::from(format!(
            "fps={:.1} frame_dt_ms={:.2} solve_us={}",
            fps.unwrap_or(0.0),
            frame_dt.map(|dt| dt.as_secs_f64() * 1000.0).unwrap_or(0.0),
            last_debug_stats.layout_engine_solve_time.as_micros()
        )));
        lines.push(Arc::from(format!(
            "frame={:?} layout_us={} paint_us={} layout_nodes={}/{} paint_nodes={}/{}",
            last_debug_stats.frame_id,
            last_debug_stats.layout_time.as_micros(),
            last_debug_stats.paint_time.as_micros(),
            last_debug_stats.layout_nodes_performed,
            last_debug_stats.layout_nodes_visited,
            last_debug_stats.paint_nodes_performed,
            last_debug_stats.paint_nodes,
        )));
        lines.push(Arc::from(format!(
            "paint_cache hits={} misses={} replayed_ops={}",
            last_debug_stats.paint_cache_hits,
            last_debug_stats.paint_cache_misses,
            last_debug_stats.paint_cache_replayed_ops
        )));
        lines.push(Arc::from(format!(
            "view_cache active={} trunc={} relayouts={}",
            last_debug_stats.view_cache_active as u8,
            last_debug_stats.view_cache_invalidation_truncations,
            last_debug_stats.view_cache_contained_relayouts
        )));
        lines.push(Arc::from(format!(
            "changes models={} edges={} roots={} walks={} nodes={}",
            last_debug_stats.model_change_models,
            last_debug_stats.model_change_observation_edges,
            last_debug_stats.model_change_invalidation_roots,
            last_debug_stats.invalidation_walk_calls_model_change,
            last_debug_stats.invalidation_walk_nodes_model_change
        )));
        lines.push(Arc::from(format!(
            "globals count={} edges={} roots={} walks={} nodes={}",
            last_debug_stats.global_change_globals,
            last_debug_stats.global_change_observation_edges,
            last_debug_stats.global_change_invalidation_roots,
            last_debug_stats.invalidation_walk_calls_global_change,
            last_debug_stats.invalidation_walk_nodes_global_change
        )));
        lines.push(Arc::from(format!(
            "hover edges pressable={} region={} decl inst={} hit={} layout={} paint={}",
            last_debug_stats.hover_pressable_target_changes,
            last_debug_stats.hover_hover_region_target_changes,
            last_debug_stats.hover_declarative_instance_changes,
            last_debug_stats.hover_declarative_hit_test_invalidations,
            last_debug_stats.hover_declarative_layout_invalidations,
            last_debug_stats.hover_declarative_paint_invalidations,
        )));

        let hover_hotspots = ui.debug_hover_declarative_invalidation_hotspots(3);
        for (index, hs) in hover_hotspots.iter().enumerate() {
            let element_path = hs.element.and_then(|element| {
                app.with_global_mut_untracked(fret_ui::ElementRuntime::new, |runtime, _| {
                    runtime.debug_path_for_element(window, element)
                })
            });

            lines.push(Arc::from(format!(
                "hover_decl[{index}] node={:?} hit={} layout={} paint={} el={} {}",
                hs.node,
                hs.hit_test,
                hs.layout,
                hs.paint,
                hs.element
                    .map(|id| format!("{:#x}", id.0))
                    .unwrap_or_else(|| "<none>".to_string()),
                element_path.as_deref().unwrap_or(""),
            )));
        }

        if let Some(extra) = cache_root_breakdown.as_ref() {
            lines.extend(extra.iter().cloned());
        }
        if let Some(line) = hot_model_breakdown.as_ref() {
            lines.push(line.clone());
        }
        if let Some(line) = unobserved_model_breakdown.as_ref() {
            lines.push(line.clone());
        }

        lines
    } else {
        Vec::new()
    };

    let inspector_status = if app.models().get_copied(inspector_enabled).unwrap_or(false) {
        let pointer = app
            .models()
            .get_copied(inspector_last_pointer)
            .unwrap_or(None);
        Some(inspector::compute_inspector_status(
            app, ui, window, pointer,
        ))
    } else {
        None
    };

    if show && let Some((cursor, hit, focus, text)) = inspector_status.as_ref() {
        lines.push(Arc::from("--- inspector ---"));
        lines.push(cursor.clone());
        lines.push(hit.clone());
        lines.push(focus.clone());
        lines.push(text.clone());
    }

    DebugHudBundle {
        show,
        lines,
        inspector_status,
    }
}
