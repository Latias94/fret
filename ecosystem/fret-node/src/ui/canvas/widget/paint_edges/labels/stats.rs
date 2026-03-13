use crate::ui::canvas::widget::*;

pub(super) fn record_edge_budget_stat<H: UiHost>(
    cx: &mut PaintCx<'_, H>,
    name: &'static str,
    limit: u32,
    used: u32,
    skipped: u32,
) {
    let Some(window) = cx.window else {
        return;
    };
    let frame_id = cx.app.frame_id().0;
    let key = CanvasCacheKey {
        window: window.data().as_ffi(),
        node: cx.node.data().as_ffi(),
        name,
    };
    cx.app
        .with_global_mut(CanvasCacheStatsRegistry::default, |registry, _app| {
            registry.record_work_budget(
                key,
                frame_id,
                used.saturating_add(skipped),
                limit,
                used,
                skipped,
            );
        });
}
