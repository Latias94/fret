use std::sync::Arc;

use fret_runtime::{
    CommandId, InputContext, InputDispatchPhase, KeymapService, Platform, PlatformCapabilities,
    WindowInputContextService, format_sequence,
};
use fret_ui::{ElementContext, UiHost};

pub(crate) fn command_shortcut_label<H: UiHost>(
    cx: &mut ElementContext<'_, H>,
    command: &CommandId,
    platform: Platform,
) -> Option<Arc<str>> {
    let caps = cx
        .app
        .global::<PlatformCapabilities>()
        .cloned()
        .unwrap_or_default();

    let snapshot = cx
        .app
        .global::<WindowInputContextService>()
        .and_then(|svc| svc.snapshot(cx.window))
        .cloned();

    let mut base_ctx = snapshot.unwrap_or(InputContext {
        platform,
        caps: caps.clone(),
        ui_has_modal: false,
        window_arbitration: None,
        focus_is_text_input: false,
        text_boundary_mode: fret_runtime::TextBoundaryMode::UnicodeWord,
        edit_can_undo: true,
        edit_can_redo: true,
        router_can_back: false,
        router_can_forward: false,
        dispatch_phase: InputDispatchPhase::Bubble,
    });
    base_ctx.platform = platform;
    base_ctx.caps = caps;
    base_ctx.dispatch_phase = InputDispatchPhase::Bubble;

    let seq = cx
        .app
        .global::<KeymapService>()
        .and_then(|svc| {
            svc.keymap
                .display_shortcut_for_command_sequence(&base_ctx, command)
        })
        .or_else(|| {
            cx.app.commands().get(command.clone()).and_then(|meta| {
                meta.default_keybindings
                    .iter()
                    .find(|kb| kb.platform.matches(platform))
                    .map(|kb| kb.sequence.clone())
            })
        })?;

    Some(Arc::from(format_sequence(platform, &seq)))
}
