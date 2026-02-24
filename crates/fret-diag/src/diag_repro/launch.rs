use super::super::*;

pub(super) struct PreparedReproLaunch {
    pub(super) launch: Option<Vec<String>>,
    pub(super) launch_env: Vec<(String, String)>,
    pub(super) tracy_feature_injected: bool,
    pub(super) tracy_env_enabled: bool,
    pub(super) renderdoc_capture_dir: Option<PathBuf>,
    pub(super) renderdoc_autocapture_after_frames: Option<u32>,
}

#[allow(clippy::too_many_arguments)]
pub(super) fn prepare_repro_launch(
    resolved_out_dir: &Path,
    launch: Option<Vec<String>>,
    launch_env: Vec<(String, String)>,
    check_redraw_hitches_max_total_ms_threshold: Option<u64>,
    with_tracy: bool,
    with_renderdoc: bool,
    renderdoc_after_frames: Option<u32>,
) -> PreparedReproLaunch {
    let mut repro_launch = launch;
    let mut repro_launch_env = launch_env;
    let _ = ensure_env_var(&mut repro_launch_env, "FRET_DIAG_RENDERER_PERF", "1");

    if check_redraw_hitches_max_total_ms_threshold.is_some() {
        let _ = ensure_env_var(&mut repro_launch_env, "FRET_REDRAW_HITCH_LOG", "1");
        let _ = ensure_env_var(
            &mut repro_launch_env,
            "FRET_REDRAW_HITCH_LOG_PATH",
            "redraw_hitches.log",
        );
    }

    let mut tracy_feature_injected: bool = false;
    let mut tracy_env_enabled: bool = false;
    if with_tracy {
        tracy_env_enabled = ensure_env_var(&mut repro_launch_env, "FRET_TRACY", "1");
        if let Some(cmd) = repro_launch.as_mut() {
            tracy_feature_injected = cargo_run_inject_feature(cmd, "fret-bootstrap/tracy");
        }

        let note = "\
# Tracy capture (best-effort)\n\
\n\
This repro was run with `FRET_TRACY=1` (and may have auto-injected `--features fret-bootstrap/tracy` when the launch command was `cargo run`).\n\
\n\
Notes:\n\
- Tracy requires running the target with the `fret-bootstrap/tracy` feature enabled.\n\
- The capture file is not recorded automatically by `fretboard` yet. Use the Tracy UI to connect and save a capture.\n\
\n\
See: `docs/tracy.md`.\n";
        let _ = std::fs::write(resolved_out_dir.join("tracy.note.md"), note);
    }

    let mut renderdoc_capture_dir: Option<PathBuf> = None;
    let mut renderdoc_autocapture_after_frames: Option<u32> = None;
    if with_renderdoc {
        let after = renderdoc_after_frames.unwrap_or(60);
        let capture_dir = resolved_out_dir.join("renderdoc");
        let _ = std::fs::create_dir_all(&capture_dir);

        let _ = ensure_env_var(&mut repro_launch_env, "FRET_RENDERDOC", "1");
        let _ = ensure_env_var(
            &mut repro_launch_env,
            "FRET_RENDERDOC_CAPTURE_DIR",
            capture_dir.to_string_lossy().as_ref(),
        );
        let _ = ensure_env_var(
            &mut repro_launch_env,
            "FRET_RENDERDOC_AUTOCAPTURE_AFTER_FRAMES",
            &after.to_string(),
        );

        renderdoc_capture_dir = Some(capture_dir);
        renderdoc_autocapture_after_frames = Some(after);
    }

    PreparedReproLaunch {
        launch: repro_launch,
        launch_env: repro_launch_env,
        tracy_feature_injected,
        tracy_env_enabled,
        renderdoc_capture_dir,
        renderdoc_autocapture_after_frames,
    }
}
