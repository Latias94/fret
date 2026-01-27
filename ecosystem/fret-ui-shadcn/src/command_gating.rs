use fret_core::AppWindowId;
use fret_runtime::{
    CommandId, InputContext, InputDispatchPhase, Platform, PlatformCapabilities,
    WindowCommandGatingService, WindowCommandGatingSnapshot,
};
use fret_ui::UiHost;

pub(crate) fn default_input_context<H: UiHost>(app: &H) -> InputContext {
    let caps = app
        .global::<PlatformCapabilities>()
        .cloned()
        .unwrap_or_default();
    InputContext {
        platform: Platform::current(),
        caps,
        ui_has_modal: false,
        window_arbitration: None,
        focus_is_text_input: false,
        edit_can_undo: true,
        edit_can_redo: true,
        dispatch_phase: InputDispatchPhase::Bubble,
    }
}

pub(crate) fn snapshot_for_window<H: UiHost>(
    app: &H,
    window: AppWindowId,
) -> WindowCommandGatingSnapshot {
    let fallback_input_ctx = default_input_context(app);
    app.global::<WindowCommandGatingService>()
        .and_then(|svc| svc.snapshot(window))
        .cloned()
        .unwrap_or_else(|| {
            fret_runtime::snapshot_for_window_with_input_ctx_fallback(
                app,
                window,
                fallback_input_ctx,
            )
        })
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
