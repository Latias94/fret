use std::sync::Arc;

use fret_app::App;
use fret_core::AppWindowId;
use fret_runner_winit::{ImeSurroundingTextUpdate, WinitPlatform};
use winit::window::Window;

pub(super) fn apply_window_redraw_text_input_snapshot(
    app: &App,
    app_window: AppWindowId,
    platform: &mut WinitPlatform,
    window: &dyn Window,
    #[cfg(target_os = "android")] android_soft_input_request: &mut Option<bool>,
) {
    // Consume the window-scoped text-input snapshot after render so the runner can position the
    // IME candidate window based on the final painted caret rect.
    //
    // Note: v1 still emits `Effect::ImeSetCursorArea` from widgets; this snapshot path is a
    // runner-level fallback and an integration surface for future macOS (NSTextInputClient) interop.
    let Some(snapshot) = app
        .global::<fret_runtime::WindowTextInputSnapshotService>()
        .and_then(|svc| svc.snapshot(app_window))
    else {
        return;
    };

    let mut dirty = false;
    let ime_changed = platform.set_ime_allowed(snapshot.focus_is_text_input);
    dirty |= ime_changed;
    #[cfg(target_os = "android")]
    if ime_changed {
        *android_soft_input_request = Some(snapshot.focus_is_text_input);
    }
    if snapshot.focus_is_text_input
        && let Some(rect) = snapshot.ime_cursor_area
    {
        dirty |= platform.set_ime_cursor_area(rect);
    }
    if snapshot.focus_is_text_input {
        let surrounding = snapshot
            .surrounding_text
            .as_ref()
            .map(|s| ImeSurroundingTextUpdate {
                text: Arc::clone(&s.text),
                cursor: s.cursor,
                anchor: s.anchor,
            });
        dirty |= platform.set_ime_surrounding_text(surrounding);
    } else {
        dirty |= platform.set_ime_surrounding_text(None);
    }

    if dirty {
        if std::env::var_os("FRET_IME_DEBUG").is_some_and(|v| !v.is_empty()) {
            tracing::info!(
                "IME_DEBUG snapshot: window={:?} focus={} cursor_area={:?}",
                app_window,
                snapshot.focus_is_text_input,
                snapshot.ime_cursor_area
            );
        }
        platform.prepare_frame(window);
    }
}
