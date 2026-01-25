use fret_core::AppWindowId;
use fret_runtime::{
    CommandId, InputContext, Platform, PlatformCapabilities, WindowCommandGatingSnapshot,
};
use fret_ui::UiHost;

pub(crate) fn default_input_context<H: UiHost>(app: &H) -> InputContext {
    let caps = app
        .global::<PlatformCapabilities>()
        .cloned()
        .unwrap_or_default();
    InputContext::fallback(Platform::current(), caps)
}

pub(crate) fn snapshot_for_window<H: UiHost>(
    app: &H,
    window: AppWindowId,
) -> WindowCommandGatingSnapshot {
    let fallback_input_ctx = default_input_context(app);
    fret_runtime::best_effort_snapshot_for_window_with_input_ctx_fallback(
        app,
        window,
        fallback_input_ctx,
    )
}

pub(crate) fn command_is_disabled_by_gating<H: UiHost>(
    app: &H,
    gating: &WindowCommandGatingSnapshot,
    command: Option<&CommandId>,
) -> bool {
    command
        .and_then(|id| app.commands().get(id.clone()).map(|meta| (id, meta)))
        .is_some_and(|(id, meta)| !gating.is_enabled_for_command(id, meta))
}
