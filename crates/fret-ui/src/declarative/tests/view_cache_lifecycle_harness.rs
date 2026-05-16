use std::collections::HashSet;
use std::sync::atomic::{AtomicUsize, Ordering};
use std::sync::{Arc, Mutex};

use fret_mechanism_harness::{
    MechanismCase, MechanismHarness, MechanismSuite, ObservedTree, ScenarioObserveError,
};
use serde::Deserialize;

use super::*;

const VIEW_CACHE_LIFECYCLE: &str = include_str!(concat!(
    env!("CARGO_MANIFEST_DIR"),
    "/src/declarative/tests/fixtures/view_cache_lifecycle_v1.json"
));

#[derive(Debug, Clone, Deserialize)]
#[serde(tag = "kind", rename_all = "snake_case")]
enum ViewCacheLifecycleScenario {
    ViewCacheLifecycle(ViewCacheLifecycleFixture),
    LayoutQueryViewCache(LayoutQueryViewCacheFixture),
}

#[derive(Debug, Clone, Deserialize)]
struct ViewCacheLifecycleFixture {
    frames: usize,
    #[serde(default)]
    cache_keys: Vec<u64>,
    #[serde(default)]
    dependency: CacheDependency,
    #[serde(default)]
    update_after_frame: Option<usize>,
    #[serde(default)]
    update_model: ModelUpdateTarget,
    #[serde(default)]
    request_animation_frame: bool,
    #[serde(default)]
    inspection_active: bool,
    #[serde(default)]
    preserve_state: bool,
    #[serde(default)]
    contain_layout_when_bounds_known: bool,
}

#[derive(Debug, Clone, Copy, Default, Deserialize)]
#[serde(rename_all = "snake_case")]
enum CacheDependency {
    #[default]
    None,
    ObservedModel,
}

#[derive(Debug, Clone, Copy, Default, Deserialize)]
#[serde(rename_all = "snake_case")]
enum ModelUpdateTarget {
    #[default]
    None,
    Observed,
    Unrelated,
}

#[derive(Debug, Clone, Deserialize)]
struct LayoutQueryViewCacheFixture {
    frames: usize,
    initial_width: f32,
    changed_width: f32,
    change_before_frame: usize,
    viewport_width: f32,
    viewport_height: f32,
}

#[derive(Debug, Default)]
struct CacheStatsAccumulator {
    total: u32,
    reused: u32,
    first_mount: u32,
    node_recreated: u32,
    marked_reuse_root: u32,
    view_cache_disabled: u32,
    inspection_active: u32,
    not_marked_reuse_root: u32,
    cache_key_mismatch: u32,
    needs_rerender: u32,
    layout_invalidated: u32,
    manual_cache_root: u32,
}

#[test]
fn mechanism_harness_view_cache_lifecycle_matches_oracles() {
    let suite: MechanismSuite<ViewCacheLifecycleScenario> =
        MechanismSuite::from_json_str(VIEW_CACHE_LIFECYCLE)
            .expect("view-cache lifecycle fixture suite");

    let mut observer: fn(
        &MechanismCase<ViewCacheLifecycleScenario>,
    ) -> Result<ObservedTree, ScenarioObserveError> = observe_case;
    MechanismHarness::new().assert_suite_passes(&suite, &mut observer);
}

fn observe_case(
    case: &MechanismCase<ViewCacheLifecycleScenario>,
) -> Result<ObservedTree, ScenarioObserveError> {
    match &case.scenario {
        ViewCacheLifecycleScenario::ViewCacheLifecycle(scenario) => {
            observe_view_cache_lifecycle(scenario)
        }
        ViewCacheLifecycleScenario::LayoutQueryViewCache(scenario) => {
            observe_layout_query_view_cache(scenario)
        }
    }
}

fn observe_view_cache_lifecycle(
    scenario: &ViewCacheLifecycleFixture,
) -> Result<ObservedTree, ScenarioObserveError> {
    let window = AppWindowId::default();
    let mut app = TestHost::new();
    let observed_model = app.models_mut().insert(0u32);
    let unrelated_model = app.models_mut().insert(0u32);

    let mut ui: UiTree<TestHost> = UiTree::new();
    ui.set_window(window);
    ui.set_debug_enabled(true);
    ui.set_view_cache_enabled(true);
    ui.set_inspection_active(scenario.inspection_active);

    let bounds = Rect::new(
        Point::new(Px(0.0), Px(0.0)),
        Size::new(Px(240.0), Px(120.0)),
    );
    let mut services = FakeTextService::default();
    let renders = Arc::new(AtomicUsize::new(0));
    let leaf_id = Arc::new(Mutex::new(None::<crate::elements::GlobalElementId>));
    let observed_values = Arc::new(Mutex::new(Vec::<i64>::new()));
    let mut stats = CacheStatsAccumulator::default();

    for frame in 0..scenario.frames {
        let cache_key = scenario.cache_keys.get(frame).copied().unwrap_or(1);
        let renders_for_frame = Arc::clone(&renders);
        let leaf_id_for_frame = Arc::clone(&leaf_id);
        let observed_values_for_frame = Arc::clone(&observed_values);
        let observed_model_for_frame = observed_model.clone();
        let dependency = scenario.dependency;
        let request_animation_frame = scenario.request_animation_frame;
        let preserve_state = scenario.preserve_state;
        let contain_layout_when_bounds_known = scenario.contain_layout_when_bounds_known;

        render_root_for_frame(
            &mut ui,
            &mut app,
            &mut services,
            window,
            bounds,
            "mechanism-harness-view-cache-lifecycle",
            move |cx| {
                let mut props = crate::element::ViewCacheProps {
                    cache_key,
                    ..Default::default()
                };
                props.layout.size.width = Length::Px(Px(120.0));
                props.layout.size.height = Length::Px(Px(40.0));
                if contain_layout_when_bounds_known {
                    props = props.contain_layout_when_bounds_known(true);
                }

                vec![cx.view_cache(props, move |cx| {
                    renders_for_frame.fetch_add(1, Ordering::SeqCst);
                    if request_animation_frame {
                        cx.request_animation_frame();
                    }

                    let label = match dependency {
                        CacheDependency::None => "leaf".to_string(),
                        CacheDependency::ObservedModel => {
                            cx.observe_model(&observed_model_for_frame, Invalidation::Layout);
                            let value = cx
                                .app
                                .models()
                                .get_copied(&observed_model_for_frame)
                                .unwrap_or_default();
                            observed_values_for_frame
                                .lock()
                                .expect("observed model values")
                                .push(value as i64);
                            format!("Value {value}")
                        }
                    };

                    let leaf = cx.text(label);
                    if preserve_state {
                        *leaf_id_for_frame.lock().expect("leaf id") = Some(leaf.id);
                        cx.state_for(
                            leaf.id,
                            || 0u32,
                            |value| {
                                if *value == 0 {
                                    *value = 123;
                                }
                            },
                        );
                    }

                    vec![leaf]
                })]
            },
        );

        ui.layout_all(&mut app, &mut services, bounds, 1.0);
        let mut scene = Scene::default();
        ui.paint_all(&mut app, &mut services, bounds, &mut scene, 1.0);
        stats.add(ui.debug_cache_root_stats().as_slice());

        app.advance_frame();

        if scenario.update_after_frame == Some(frame) {
            match scenario.update_model {
                ModelUpdateTarget::None => {}
                ModelUpdateTarget::Observed => {
                    let _ = observed_model.update(&mut app, |value, _cx| {
                        *value = value.saturating_add(1);
                    });
                }
                ModelUpdateTarget::Unrelated => {
                    let _ = unrelated_model.update(&mut app, |value, _cx| {
                        *value = value.saturating_add(1);
                    });
                }
            }
        }
    }

    let mut observed = ObservedTree::new(bounds);
    observed.set_metric("renders.cache_root", renders.load(Ordering::SeqCst) as f32);
    set_cache_stats_metrics(&mut observed, &stats);

    let values = observed_values.lock().expect("observed model values");
    observed.set_metric("observed.model_values", values.len() as f32);
    observed.set_metric(
        "observed.model_unique_values",
        unique_i64_count(&values) as f32,
    );

    let state_value = match *leaf_id.lock().expect("leaf id") {
        Some(leaf) => crate::elements::with_element_state(&mut app, window, leaf, || 0u32, |v| *v),
        None => 0,
    };
    observed.set_metric("state.leaf_value", state_value as f32);

    Ok(observed)
}

fn observe_layout_query_view_cache(
    scenario: &LayoutQueryViewCacheFixture,
) -> Result<ObservedTree, ScenarioObserveError> {
    let window = AppWindowId::default();
    let mut app = TestHost::new();
    let mut ui: UiTree<TestHost> = UiTree::new();
    ui.set_window(window);
    ui.set_debug_enabled(true);
    ui.set_view_cache_enabled(true);

    let bounds = Rect::new(
        Point::new(Px(0.0), Px(0.0)),
        Size::new(Px(scenario.viewport_width), Px(scenario.viewport_height)),
    );
    let mut services = FakeTextService::default();

    let width = Arc::new(Mutex::new(Px(scenario.initial_width)));
    let reads = Arc::new(Mutex::new(Vec::<Option<Px>>::new()));
    let cached_reads = Arc::new(Mutex::new(Vec::<Option<Px>>::new()));
    let renders = Arc::new(AtomicUsize::new(0));
    let mut stats = CacheStatsAccumulator::default();
    let mut renders_after_settle = 0usize;
    let mut renders_after_same_frame_change = 0usize;

    for frame in 0..scenario.frames {
        if frame == scenario.change_before_frame {
            *width.lock().expect("layout query width") = Px(scenario.changed_width);
        }

        let width_for_frame = Arc::clone(&width);
        let reads_for_frame = Arc::clone(&reads);
        let cached_reads_for_frame = Arc::clone(&cached_reads);
        let renders_for_frame = Arc::clone(&renders);

        render_root_for_frame(
            &mut ui,
            &mut app,
            &mut services,
            window,
            bounds,
            "mechanism-harness-layout-query-view-cache",
            move |cx| {
                let w = *width_for_frame.lock().expect("layout query width");
                let mut region_id: Option<crate::elements::GlobalElementId> = None;
                let region = cx.layout_query_region_with_id(
                    crate::element::LayoutQueryRegionProps::default(),
                    |cx, id| {
                        region_id = Some(id);
                        let mut container = crate::element::ContainerProps::default();
                        container.layout.size.width = Length::Px(w);
                        container.layout.size.height = Length::Px(Px(20.0));
                        vec![cx.container(container, |cx| vec![cx.text("region")])]
                    },
                );
                let region_id = region_id.expect("layout query region id should be recorded");

                let snapshot = cx
                    .layout_query_bounds(region_id, Invalidation::Layout)
                    .map(|rect| rect.size.width);
                reads_for_frame
                    .lock()
                    .expect("layout query reads")
                    .push(snapshot);

                let cached = cx.view_cache(crate::element::ViewCacheProps::default(), move |cx| {
                    renders_for_frame.fetch_add(1, Ordering::SeqCst);
                    let snapshot = cx
                        .layout_query_bounds(region_id, Invalidation::Layout)
                        .map(|rect| rect.size.width);
                    cached_reads_for_frame
                        .lock()
                        .expect("cached layout query reads")
                        .push(snapshot);
                    vec![cx.text("cached")]
                });

                vec![region, cached]
            },
        );

        ui.layout_all(&mut app, &mut services, bounds, 1.0);
        let mut scene = Scene::default();
        ui.paint_all(&mut app, &mut services, bounds, &mut scene, 1.0);
        stats.add(ui.debug_cache_root_stats().as_slice());

        app.advance_frame();

        if frame == 1 {
            renders_after_settle = renders.load(Ordering::SeqCst);
        }
        if frame == scenario.change_before_frame {
            renders_after_same_frame_change = renders.load(Ordering::SeqCst);
        }
    }

    let mut observed = ObservedTree::new(bounds);
    observed.set_metric("renders.cache_root", renders.load(Ordering::SeqCst) as f32);
    observed.set_metric(
        "layout_query.render_delta_same_frame",
        renders_after_same_frame_change.saturating_sub(renders_after_settle) as f32,
    );
    observed.set_metric(
        "layout_query.render_delta_final",
        renders
            .load(Ordering::SeqCst)
            .saturating_sub(renders_after_settle) as f32,
    );

    let reads = reads.lock().expect("layout query reads");
    for (index, read) in reads.iter().enumerate() {
        observed.set_metric(
            format!("layout_query.root_read.frame{index}_width"),
            read.map(|value| value.0).unwrap_or(-1.0),
        );
    }

    let cached_reads = cached_reads.lock().expect("cached layout query reads");
    let cached_last = cached_reads
        .last()
        .copied()
        .flatten()
        .map(|value| value.0)
        .unwrap_or(-1.0);
    observed.set_metric("layout_query.cached_last_width", cached_last);
    set_cache_stats_metrics(&mut observed, &stats);

    Ok(observed)
}

impl CacheStatsAccumulator {
    fn add(&mut self, stats: &[crate::tree::UiDebugCacheRootStats]) {
        for stat in stats {
            self.total = self.total.saturating_add(1);
            self.reused = self.reused.saturating_add(stat.reused as u32);
            match stat.reuse_reason {
                crate::tree::UiDebugCacheRootReuseReason::FirstMount => {
                    self.first_mount = self.first_mount.saturating_add(1);
                }
                crate::tree::UiDebugCacheRootReuseReason::NodeRecreated => {
                    self.node_recreated = self.node_recreated.saturating_add(1);
                }
                crate::tree::UiDebugCacheRootReuseReason::MarkedReuseRoot => {
                    self.marked_reuse_root = self.marked_reuse_root.saturating_add(1);
                }
                crate::tree::UiDebugCacheRootReuseReason::ViewCacheDisabled => {
                    self.view_cache_disabled = self.view_cache_disabled.saturating_add(1);
                }
                crate::tree::UiDebugCacheRootReuseReason::InspectionActive => {
                    self.inspection_active = self.inspection_active.saturating_add(1);
                }
                crate::tree::UiDebugCacheRootReuseReason::NotMarkedReuseRoot => {
                    self.not_marked_reuse_root = self.not_marked_reuse_root.saturating_add(1);
                }
                crate::tree::UiDebugCacheRootReuseReason::CacheKeyMismatch => {
                    self.cache_key_mismatch = self.cache_key_mismatch.saturating_add(1);
                }
                crate::tree::UiDebugCacheRootReuseReason::NeedsRerender => {
                    self.needs_rerender = self.needs_rerender.saturating_add(1);
                }
                crate::tree::UiDebugCacheRootReuseReason::LayoutInvalidated => {
                    self.layout_invalidated = self.layout_invalidated.saturating_add(1);
                }
                crate::tree::UiDebugCacheRootReuseReason::ManualCacheRoot => {
                    self.manual_cache_root = self.manual_cache_root.saturating_add(1);
                }
            }
        }
    }
}

fn set_cache_stats_metrics(observed: &mut ObservedTree, stats: &CacheStatsAccumulator) {
    observed.set_metric("cache.roots.total", stats.total as f32);
    observed.set_metric("cache.roots.reused", stats.reused as f32);
    observed.set_metric("cache.reason.first_mount", stats.first_mount as f32);
    observed.set_metric("cache.reason.node_recreated", stats.node_recreated as f32);
    observed.set_metric(
        "cache.reason.marked_reuse_root",
        stats.marked_reuse_root as f32,
    );
    observed.set_metric(
        "cache.reason.view_cache_disabled",
        stats.view_cache_disabled as f32,
    );
    observed.set_metric(
        "cache.reason.inspection_active",
        stats.inspection_active as f32,
    );
    observed.set_metric(
        "cache.reason.not_marked_reuse_root",
        stats.not_marked_reuse_root as f32,
    );
    observed.set_metric(
        "cache.reason.cache_key_mismatch",
        stats.cache_key_mismatch as f32,
    );
    observed.set_metric("cache.reason.needs_rerender", stats.needs_rerender as f32);
    observed.set_metric(
        "cache.reason.layout_invalidated",
        stats.layout_invalidated as f32,
    );
    observed.set_metric(
        "cache.reason.manual_cache_root",
        stats.manual_cache_root as f32,
    );
}

fn unique_i64_count(values: &[i64]) -> usize {
    values.iter().copied().collect::<HashSet<_>>().len()
}
