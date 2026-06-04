use fret_app::App;
use fret_core::FrameId;

#[derive(Clone, Copy)]
pub(super) struct WindowRedrawTextDiagnosticsMode {
    debug_enabled: bool,
    diagnostics_enabled: bool,
}

impl WindowRedrawTextDiagnosticsMode {
    fn diagnostics_enabled(self) -> bool {
        self.diagnostics_enabled
    }

    fn debug_enabled(self) -> bool {
        self.debug_enabled
    }
}

pub(super) fn window_redraw_text_diagnostics_mode_from_env() -> WindowRedrawTextDiagnosticsMode {
    let debug_enabled = std::env::var_os("FRET_RENDER_TEXT_DEBUG").is_some_and(|v| !v.is_empty());
    let diagnostics_enabled =
        std::env::var_os("FRET_DIAG_DIR").is_some_and(|v| !v.is_empty()) || debug_enabled;
    WindowRedrawTextDiagnosticsMode {
        debug_enabled,
        diagnostics_enabled,
    }
}

pub(super) fn begin_window_redraw_text_diagnostics_frame(
    renderer: &mut fret_render::Renderer,
    mode: WindowRedrawTextDiagnosticsMode,
) {
    if mode.diagnostics_enabled() {
        renderer.begin_text_diagnostics_frame();
    }
}

pub(super) fn publish_window_redraw_text_diagnostics(
    app: &mut App,
    renderer: &fret_render::Renderer,
    frame_id: FrameId,
    mode: WindowRedrawTextDiagnosticsMode,
) {
    crate::runner::font_catalog::publish_renderer_svg_text_bridge_diagnostics(app, renderer);

    if !mode.diagnostics_enabled() {
        return;
    }

    let diagnostics = renderer.text_diagnostics_snapshot(frame_id);
    let trace = renderer.text_font_trace_snapshot(frame_id);
    let policy = renderer.text_fallback_policy_snapshot(frame_id);

    if mode.debug_enabled() {
        app.set_global(diagnostics);
        app.set_global(trace);
        app.set_global(policy);
    } else {
        // Avoid turning per-frame diagnostics snapshots into global-change propagation /
        // invalidation work during perf-sensitive runs.
        app.with_global_mut_untracked(
            fret_core::RendererTextPerfSnapshot::default,
            |slot, _app| {
                *slot = diagnostics;
            },
        );
        app.with_global_mut_untracked(
            fret_core::RendererTextFontTraceSnapshot::default,
            |slot, _app| {
                *slot = trace;
            },
        );
        app.with_global_mut_untracked(
            fret_core::RendererTextFallbackPolicySnapshot::default,
            |slot, _app| {
                *slot = policy;
            },
        );
    }
}
