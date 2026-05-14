use super::*;

use std::sync::atomic::{AtomicUsize, Ordering};

use fret_mechanism_harness::{
    MechanismCase, MechanismHarness, MechanismSuite, ObservedTree, ScenarioObserveError,
};
use serde::Deserialize;

const RETAINED_VIRTUAL_LIST_RECONCILE: &str = include_str!(concat!(
    env!("CARGO_MANIFEST_DIR"),
    "/src/declarative/tests/fixtures/retained_virtual_list_reconcile_v1.json"
));

#[derive(Debug, Clone, Deserialize)]
#[serde(tag = "kind", rename_all = "snake_case")]
enum RetainedVirtualListScenario {
    RetainedVirtualList {
        len: usize,
        viewport_width: f32,
        viewport_height: f32,
        row_height: f32,
        overscan: usize,
        keep_alive: usize,
        warmup_frames: usize,
        steps: Vec<RetainedVirtualListStep>,
    },
}

#[derive(Debug, Clone, Deserialize)]
struct RetainedVirtualListStep {
    label: String,
    offset_y: f32,
    frames: usize,
}

#[derive(Debug, Default, Clone, Copy)]
struct ReconcileAccumulator {
    records: u32,
    escape_records: u32,
    prefetch_records: u32,
    attached_items: u32,
    detached_items: u32,
    kept_alive_items: u32,
    reused_from_keep_alive_items: u32,
    evicted_keep_alive_items: u32,
    keep_alive_pool_len_after_max: u32,
}

#[test]
fn mechanism_harness_retained_virtual_list_reconcile_matches_oracles() {
    let suite: MechanismSuite<RetainedVirtualListScenario> =
        MechanismSuite::from_json_str(RETAINED_VIRTUAL_LIST_RECONCILE)
            .expect("retained virtual-list reconcile fixture suite");

    let mut observer: fn(
        &MechanismCase<RetainedVirtualListScenario>,
    ) -> Result<ObservedTree, ScenarioObserveError> = observe_case;
    MechanismHarness::new().assert_suite_passes(&suite, &mut observer);
}

fn observe_case(
    case: &MechanismCase<RetainedVirtualListScenario>,
) -> Result<ObservedTree, ScenarioObserveError> {
    match &case.scenario {
        RetainedVirtualListScenario::RetainedVirtualList {
            len,
            viewport_width,
            viewport_height,
            row_height,
            overscan,
            keep_alive,
            warmup_frames,
            steps,
        } => observe_retained_virtual_list_case(
            *len,
            *viewport_width,
            *viewport_height,
            *row_height,
            *overscan,
            *keep_alive,
            *warmup_frames,
            steps,
        ),
    }
}

#[allow(clippy::too_many_arguments)]
fn observe_retained_virtual_list_case(
    len: usize,
    viewport_width: f32,
    viewport_height: f32,
    row_height: f32,
    overscan: usize,
    keep_alive: usize,
    warmup_frames: usize,
    steps: &[RetainedVirtualListStep],
) -> Result<ObservedTree, ScenarioObserveError> {
    let window = AppWindowId::default();
    let mut app = TestHost::new();
    let mut ui: UiTree<TestHost> = UiTree::new();
    ui.set_window(window);
    ui.set_view_cache_enabled(true);
    ui.set_debug_enabled(true);

    let scroll_handle = crate::scroll::VirtualListScrollHandle::new();
    let bounds = Rect::new(
        Point::new(Px(0.0), Px(0.0)),
        Size::new(Px(viewport_width), Px(viewport_height)),
    );
    let mut services = FakeTextService::default();
    let render_calls = Arc::new(AtomicUsize::new(0));

    for _frame in 0..warmup_frames {
        render_retained_list_frame(
            &mut ui,
            &mut app,
            &mut services,
            window,
            bounds,
            len,
            row_height,
            overscan,
            keep_alive,
            &scroll_handle,
            Arc::clone(&render_calls),
        );
    }
    let render_calls_after_warmup = render_calls.load(Ordering::SeqCst);

    let mut observed = ObservedTree::new(bounds);
    for step in steps {
        scroll_handle.set_offset(Point::new(Px(0.0), Px(step.offset_y)));
        let mut acc = ReconcileAccumulator::default();

        for _frame in 0..step.frames {
            let before = ui.debug_retained_virtual_list_reconciles().len();
            render_retained_list_frame(
                &mut ui,
                &mut app,
                &mut services,
                window,
                bounds,
                len,
                row_height,
                overscan,
                keep_alive,
                &scroll_handle,
                Arc::clone(&render_calls),
            );

            let records = ui
                .debug_retained_virtual_list_reconciles()
                .get(before..)
                .unwrap_or_else(|| ui.debug_retained_virtual_list_reconciles());
            acc.add_records(records);
        }

        set_reconcile_metrics(&mut observed, &step.label, acc);
    }

    observed.set_metric(
        "render.cache_root_calls_after_warmup",
        render_calls
            .load(Ordering::SeqCst)
            .saturating_sub(render_calls_after_warmup) as f32,
    );
    Ok(observed)
}

#[allow(clippy::too_many_arguments)]
fn render_retained_list_frame(
    ui: &mut UiTree<TestHost>,
    app: &mut TestHost,
    services: &mut FakeTextService,
    window: AppWindowId,
    bounds: Rect,
    len: usize,
    row_height: f32,
    overscan: usize,
    keep_alive: usize,
    scroll_handle: &crate::scroll::VirtualListScrollHandle,
    render_calls: Arc<AtomicUsize>,
) {
    render_root_for_frame(
        ui,
        app,
        services,
        window,
        bounds,
        "mechanism-harness-retained-virtual-list-reconcile",
        |cx| {
            let mut cache = crate::element::ViewCacheProps::default();
            cache.layout.size.width = crate::element::Length::Fill;
            cache.layout.size.height = crate::element::Length::Fill;
            cache.cache_key = 1;

            vec![cx.view_cache(cache, move |cx| {
                render_calls.fetch_add(1, Ordering::SeqCst);

                let list_layout = crate::element::LayoutStyle {
                    size: crate::element::SizeStyle {
                        width: crate::element::Length::Fill,
                        height: crate::element::Length::Fill,
                        ..Default::default()
                    },
                    overflow: crate::element::Overflow::Clip,
                    ..Default::default()
                };

                let key_at: crate::windowed_surface_host::RetainedVirtualListKeyAtFn =
                    Arc::new(|i| i as crate::ItemKey);
                let row: crate::windowed_surface_host::RetainedVirtualListRowFn<TestHost> =
                    Arc::new(|cx, i| cx.text(format!("row {i}")));
                let options = crate::element::VirtualListOptions::new(Px(row_height), overscan)
                    .keep_alive(keep_alive);

                vec![cx.virtual_list_keyed_retained_with_layout(
                    list_layout,
                    len,
                    options,
                    scroll_handle,
                    key_at,
                    row,
                )]
            })]
        },
    );
    ui.layout_all(app, services, bounds, 1.0);
    app.advance_frame();
}

impl ReconcileAccumulator {
    fn add_records(&mut self, records: &[crate::tree::UiDebugRetainedVirtualListReconcile]) {
        for record in records {
            self.records = self.records.saturating_add(1);
            match record.reconcile_kind {
                crate::tree::UiDebugRetainedVirtualListReconcileKind::Escape => {
                    self.escape_records = self.escape_records.saturating_add(1);
                }
                crate::tree::UiDebugRetainedVirtualListReconcileKind::Prefetch => {
                    self.prefetch_records = self.prefetch_records.saturating_add(1);
                }
            }
            self.attached_items = self.attached_items.saturating_add(record.attached_items);
            self.detached_items = self.detached_items.saturating_add(record.detached_items);
            self.kept_alive_items = self
                .kept_alive_items
                .saturating_add(record.kept_alive_items);
            self.reused_from_keep_alive_items = self
                .reused_from_keep_alive_items
                .saturating_add(record.reused_from_keep_alive_items);
            self.evicted_keep_alive_items = self
                .evicted_keep_alive_items
                .saturating_add(record.evicted_keep_alive_items);
            self.keep_alive_pool_len_after_max = self
                .keep_alive_pool_len_after_max
                .max(record.keep_alive_pool_len_after);
        }
    }
}

fn set_reconcile_metrics(observed: &mut ObservedTree, label: &str, acc: ReconcileAccumulator) {
    let prefix = format!("capture.{label}.retained_reconcile");
    observed.set_metric(format!("{prefix}.records"), acc.records as f32);
    observed.set_metric(
        format!("{prefix}.escape_records"),
        acc.escape_records as f32,
    );
    observed.set_metric(
        format!("{prefix}.prefetch_records"),
        acc.prefetch_records as f32,
    );
    observed.set_metric(
        format!("{prefix}.attached_items_total"),
        acc.attached_items as f32,
    );
    observed.set_metric(
        format!("{prefix}.detached_items_total"),
        acc.detached_items as f32,
    );
    observed.set_metric(
        format!("{prefix}.kept_alive_items_total"),
        acc.kept_alive_items as f32,
    );
    observed.set_metric(
        format!("{prefix}.reused_from_keep_alive_items_total"),
        acc.reused_from_keep_alive_items as f32,
    );
    observed.set_metric(
        format!("{prefix}.evicted_keep_alive_items_total"),
        acc.evicted_keep_alive_items as f32,
    );
    observed.set_metric(
        format!("{prefix}.keep_alive_pool_len_after_max"),
        acc.keep_alive_pool_len_after_max as f32,
    );
}
