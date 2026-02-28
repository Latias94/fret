use fret_diag_protocol::UiDiagnosticsConfigFileV1;
use std::ffi::OsString;
use std::path::{Path, PathBuf};
use std::sync::Once;

static DIAG_CFG_LOG_ONCE: Once = Once::new();

fn load_ui_diagnostics_config_file(path: &Path) -> Result<UiDiagnosticsConfigFileV1, String> {
    let bytes = std::fs::read(path).map_err(|e| e.to_string())?;
    serde_json::from_slice(&bytes).map_err(|e| e.to_string())
}

fn env_flag_override(name: &str) -> Option<bool> {
    let v = std::env::var_os(name)?;
    let v = v.to_string_lossy().trim().to_ascii_lowercase();
    if v.is_empty() {
        return Some(true);
    }
    Some(!matches!(v.as_str(), "0" | "false" | "no" | "off"))
}

fn env_usize_override(name: &str) -> Option<usize> {
    let Ok(v) = std::env::var(name) else {
        return None;
    };
    let v = v.trim();
    if v.is_empty() {
        return None;
    }
    v.parse::<usize>().ok()
}

fn resolve_config_path(out_dir: &Path, raw: &str) -> Option<PathBuf> {
    let raw = raw.trim();
    if raw.is_empty() {
        return None;
    }
    let p = PathBuf::from(raw);
    Some(if p.is_absolute() { p } else { out_dir.join(p) })
}

fn ios_home_dir() -> Option<PathBuf> {
    if !cfg!(target_os = "ios") {
        return None;
    }
    std::env::var_os("HOME")
        .filter(|v| !v.is_empty())
        .map(PathBuf::from)
}

fn ios_tmp_dir() -> PathBuf {
    ios_home_dir()
        .map(|home| home.join("tmp"))
        .unwrap_or_else(std::env::temp_dir)
}

fn resolve_ios_diag_out_dir(out_dir: PathBuf) -> PathBuf {
    if !cfg!(target_os = "ios") {
        return out_dir;
    }
    if out_dir.is_absolute() {
        return out_dir;
    }
    if let Some(home) = ios_home_dir() {
        return home.join(out_dir);
    }
    ios_tmp_dir().join(out_dir)
}

fn diag_args_override() -> (bool, Option<PathBuf>) {
    let mut enabled = false;
    let mut out_dir = None;

    // Use `args_os()` so we don't panic if the platform provides non-UTF8 argv.
    let mut args = std::env::args_os().skip(1);
    while let Some(arg) = args.next() {
        match arg.to_string_lossy().as_ref() {
            "--fret-diag" => {
                enabled = true;
            }
            "--fret-diag-dir" => {
                if let Some(dir) = args.next() {
                    if !dir.to_string_lossy().trim().is_empty() {
                        enabled = true;
                        out_dir = Some(PathBuf::from(dir));
                    }
                }
            }
            _ => {}
        }
    }

    (enabled, out_dir)
}

#[derive(Debug, Clone)]
pub struct UiDiagnosticsConfig {
    pub enabled: bool,
    pub out_dir: PathBuf,
    pub trigger_path: PathBuf,
    pub ready_path: PathBuf,
    pub exit_path: PathBuf,
    /// Whether the diagnostics runtime should accept script schema v1 inputs.
    ///
    /// Tooling upgrades scripts to schema v2 on execution; tool-launched runs typically disable
    /// schema v1 parsing to keep the runtime on the v2-only path.
    pub allow_script_schema_v1: bool,
    /// When enabled, keep requesting redraws even when no script is running.
    ///
    /// This is intended for scripted diagnostics runs where the external driver triggers scripts
    /// via filesystem touch stamps, but the app might otherwise go idle between frames.
    pub script_keepalive: bool,
    pub max_events: usize,
    pub max_snapshots: usize,
    /// Maximum number of snapshots to include in script-driven bundle dumps (auto-dump and
    /// `capture_bundle` steps).
    pub script_dump_max_snapshots: usize,
    pub capture_semantics: bool,
    /// Cap the number of exported semantics nodes per snapshot (bundle size control).
    pub max_semantics_nodes: usize,
    /// Export only semantics nodes that have a `test_id` (bundle size control).
    pub semantics_test_ids_only: bool,
    pub screenshots_enabled: bool,
    pub screenshot_request_path: PathBuf,
    pub screenshot_trigger_path: PathBuf,
    pub screenshot_result_path: PathBuf,
    pub screenshot_result_trigger_path: PathBuf,
    pub script_path: PathBuf,
    pub script_trigger_path: PathBuf,
    pub script_result_path: PathBuf,
    pub script_result_trigger_path: PathBuf,
    pub script_auto_dump: bool,
    pub pick_trigger_path: PathBuf,
    pub pick_result_path: PathBuf,
    pub pick_result_trigger_path: PathBuf,
    pub pick_auto_dump: bool,
    pub inspect_path: PathBuf,
    pub inspect_trigger_path: PathBuf,
    pub redact_text: bool,
    pub max_debug_string_bytes: usize,
    pub max_gating_trace_entries: usize,
    pub screenshot_on_dump: bool,
    /// Whether the diagnostics runtime should write the large raw bundle artifact (`bundle.json`)
    /// during dumps.
    pub write_bundle_json: bool,
    /// When enabled, write a compact schema2 bundle artifact (`bundle.schema2.json`) alongside
    /// dumps (tooling can prefer this view and omit the larger raw artifact).
    ///
    /// This is intended for schema2-first + AI/sidecar-first workflows to avoid requiring
    /// tooling to parse large raw bundles just to produce a portable artifact.
    pub write_bundle_schema2: bool,
    /// Optional fixed frame delta (ms) for deterministic diagnostics/scripted tests (ADR 0240).
    ///
    /// When set, the per-window frame clock uses a synthetic monotonic time that advances by this
    /// delta each frame, rather than wall-clock `Instant::now()`.
    pub frame_clock_fixed_delta_ms: Option<u64>,

    /// Optional DevTools WebSocket endpoint for diagnostics control (script/pick/dump).
    ///
    /// When set (with `devtools_token`), diagnostics enablement is implied even if `FRET_DIAG` is
    /// not set. This is required for web runners that do not have filesystem access.
    pub devtools_ws_url: Option<String>,
    pub devtools_token: Option<String>,
    /// Whether `bundle.dumped` should embed the full `bundle.json` payload in the WS message.
    ///
    /// Web runners should enable this so tooling can materialize bundles without filesystem
    /// access inside the browser.
    pub devtools_embed_bundle: bool,
}

impl Default for UiDiagnosticsConfig {
    fn default() -> Self {
        let (diag_arg_enabled, diag_arg_dir) = diag_args_override();

        let config_path = std::env::var_os("FRET_DIAG_CONFIG_PATH")
            .filter(|v| !v.is_empty())
            .map(PathBuf::from);
        let config_file =
            config_path
                .as_ref()
                .and_then(|p| match load_ui_diagnostics_config_file(p) {
                    Ok(v) => Some(v),
                    Err(err) => {
                        tracing::warn!(
                            target: "fret",
                            config_path = ?p,
                            error = %err,
                            "failed to load ui diagnostics config file"
                        );
                        None
                    }
                });
        let config_enabled = config_file
            .as_ref()
            .map(|c| c.enabled.unwrap_or(true))
            .unwrap_or(false);
        let config_out_dir = config_file
            .as_ref()
            .and_then(|c| c.out_dir.as_deref())
            .map(|s| s.trim())
            .filter(|s| !s.is_empty())
            .map(PathBuf::from)
            .map(resolve_ios_diag_out_dir);

        let raw_diag = std::env::var_os("FRET_DIAG")
            .filter(|v| !v.is_empty())
            .or_else(|| diag_arg_enabled.then(|| OsString::from("1")));
        let raw_out_dir = std::env::var_os("FRET_DIAG_DIR")
            .filter(|v| !v.is_empty())
            .or_else(|| diag_arg_dir.as_ref().map(|p| p.clone().into_os_string()))
            .or_else(|| config_out_dir.as_ref().map(|p| p.clone().into_os_string()));

        let out_dir_env = raw_out_dir.as_ref();
        let diag_enabled = raw_diag.is_some() || out_dir_env.is_some() || config_enabled;

        let (devtools_ws_url, devtools_token) = {
            #[cfg(all(feature = "diagnostics-ws", target_arch = "wasm32"))]
            {
                fret_diag_ws::client::devtools_ws_config_from_window_query()
            }
            #[cfg(all(feature = "diagnostics-ws", not(target_arch = "wasm32")))]
            {
                let ws_url = std::env::var("FRET_DEVTOOLS_WS")
                    .ok()
                    .filter(|v| !v.trim().is_empty());
                let token = std::env::var("FRET_DEVTOOLS_TOKEN")
                    .ok()
                    .filter(|v| !v.trim().is_empty());
                (ws_url, token)
            }
            #[cfg(not(feature = "diagnostics-ws"))]
            {
                (None, None)
            }
        };

        let enabled = diag_enabled || (devtools_ws_url.is_some() && devtools_token.is_some());
        let out_dir = out_dir_env.map(PathBuf::from).unwrap_or_else(|| {
            if cfg!(target_os = "ios") {
                ios_tmp_dir().join("fret-diag")
            } else {
                PathBuf::from("target").join("fret-diag")
            }
        });
        let out_dir = resolve_ios_diag_out_dir(out_dir);
        let trigger_path = std::env::var_os("FRET_DIAG_TRIGGER_PATH")
            .filter(|v| !v.is_empty())
            .map(PathBuf::from)
            .or_else(|| {
                config_file
                    .as_ref()
                    .and_then(|c| c.paths.as_ref())
                    .and_then(|p| p.trigger_path.as_deref())
                    .and_then(|s| resolve_config_path(&out_dir, s))
            })
            .unwrap_or_else(|| out_dir.join("trigger.touch"));
        let ready_path = std::env::var_os("FRET_DIAG_READY_PATH")
            .filter(|v| !v.is_empty())
            .map(PathBuf::from)
            .or_else(|| {
                config_file
                    .as_ref()
                    .and_then(|c| c.paths.as_ref())
                    .and_then(|p| p.ready_path.as_deref())
                    .and_then(|s| resolve_config_path(&out_dir, s))
            })
            .unwrap_or_else(|| out_dir.join("ready.touch"));
        let exit_path = std::env::var_os("FRET_DIAG_EXIT_PATH")
            .filter(|v| !v.is_empty())
            .map(PathBuf::from)
            .or_else(|| {
                config_file
                    .as_ref()
                    .and_then(|c| c.paths.as_ref())
                    .and_then(|p| p.exit_path.as_deref())
                    .and_then(|s| resolve_config_path(&out_dir, s))
            })
            .unwrap_or_else(|| out_dir.join("exit.touch"));

        let allow_script_schema_v1 = config_file
            .as_ref()
            .and_then(|c| c.allow_script_schema_v1)
            .unwrap_or(true);

        let script_keepalive = enabled
            && env_flag_override("FRET_DIAG_SCRIPT_KEEPALIVE")
                .or_else(|| config_file.as_ref().and_then(|c| c.script_keepalive))
                .unwrap_or(true);
        if enabled
            || raw_diag.as_ref().is_some_and(|v| !v.is_empty())
            || raw_out_dir.as_ref().is_some_and(|v| !v.is_empty())
            || diag_arg_enabled
            || diag_arg_dir.is_some()
            || config_file.is_some()
        {
            let diag_val = raw_diag.as_ref().map(|v| v.to_string_lossy().to_string());
            let dir_val = raw_out_dir
                .as_ref()
                .map(|v| v.to_string_lossy().to_string());
            DIAG_CFG_LOG_ONCE.call_once(|| {
                tracing::info!(
                    target: "fret",
                    enabled,
                    diag = diag_val.as_deref().unwrap_or(""),
                    diag_dir = dir_val.as_deref().unwrap_or(""),
                    diag_arg_enabled,
                    diag_arg_dir = ?diag_arg_dir,
                    config_path = ?config_path,
                    out_dir = ?out_dir,
                    trigger_path = ?trigger_path,
                    "ui diagnostics config",
                );
            });
        }

        let max_events = env_usize_override("FRET_DIAG_MAX_EVENTS")
            .or_else(|| {
                config_file
                    .as_ref()
                    .and_then(|c| c.max_events)
                    .map(|v| v as usize)
            })
            .unwrap_or(2000);
        let max_snapshots = env_usize_override("FRET_DIAG_MAX_SNAPSHOTS")
            .or_else(|| {
                config_file
                    .as_ref()
                    .and_then(|c| c.max_snapshots)
                    .map(|v| v as usize)
            })
            .unwrap_or(300);
        let script_dump_max_snapshots = env_usize_override("FRET_DIAG_SCRIPT_DUMP_MAX_SNAPSHOTS")
            .or_else(|| {
                config_file
                    .as_ref()
                    .and_then(|c| c.script_dump_max_snapshots)
                    .map(|v| v as usize)
            })
            .unwrap_or(30);
        let script_dump_max_snapshots = if max_snapshots == 0 {
            0
        } else {
            script_dump_max_snapshots.clamp(1, max_snapshots)
        };
        let capture_semantics = env_flag_override("FRET_DIAG_SEMANTICS")
            .or_else(|| config_file.as_ref().and_then(|c| c.capture_semantics))
            .unwrap_or(true);
        let max_semantics_nodes = env_usize_override("FRET_DIAG_MAX_SEMANTICS_NODES")
            .or_else(|| {
                config_file
                    .as_ref()
                    .and_then(|c| c.max_semantics_nodes)
                    .map(|v| v as usize)
            })
            .unwrap_or(50_000)
            .clamp(0, 500_000);
        let semantics_test_ids_only = env_flag_override("FRET_DIAG_SEMANTICS_TEST_IDS_ONLY")
            .or_else(|| config_file.as_ref().and_then(|c| c.semantics_test_ids_only))
            .unwrap_or(false);
        let screenshots_enabled = env_flag_override("FRET_DIAG_GPU_SCREENSHOTS")
            .or_else(|| config_file.as_ref().and_then(|c| c.screenshots_enabled))
            .unwrap_or(false);
        let screenshot_request_path = std::env::var_os("FRET_DIAG_SCREENSHOT_REQUEST_PATH")
            .filter(|v| !v.is_empty())
            .map(PathBuf::from)
            .or_else(|| {
                config_file
                    .as_ref()
                    .and_then(|c| c.paths.as_ref())
                    .and_then(|p| p.screenshot_request_path.as_deref())
                    .and_then(|s| resolve_config_path(&out_dir, s))
            })
            .unwrap_or_else(|| out_dir.join("screenshots.request.json"));
        let screenshot_trigger_path = std::env::var_os("FRET_DIAG_SCREENSHOT_TRIGGER_PATH")
            .filter(|v| !v.is_empty())
            .map(PathBuf::from)
            .or_else(|| {
                config_file
                    .as_ref()
                    .and_then(|c| c.paths.as_ref())
                    .and_then(|p| p.screenshot_trigger_path.as_deref())
                    .and_then(|s| resolve_config_path(&out_dir, s))
            })
            .unwrap_or_else(|| out_dir.join("screenshots.touch"));
        let screenshot_result_path = std::env::var_os("FRET_DIAG_SCREENSHOT_RESULT_PATH")
            .filter(|v| !v.is_empty())
            .map(PathBuf::from)
            .or_else(|| {
                config_file
                    .as_ref()
                    .and_then(|c| c.paths.as_ref())
                    .and_then(|p| p.screenshot_result_path.as_deref())
                    .and_then(|s| resolve_config_path(&out_dir, s))
            })
            .unwrap_or_else(|| out_dir.join("screenshots.result.json"));
        let screenshot_result_trigger_path =
            std::env::var_os("FRET_DIAG_SCREENSHOT_RESULT_TRIGGER_PATH")
                .filter(|v| !v.is_empty())
                .map(PathBuf::from)
                .or_else(|| {
                    config_file
                        .as_ref()
                        .and_then(|c| c.paths.as_ref())
                        .and_then(|p| p.screenshot_result_trigger_path.as_deref())
                        .and_then(|s| resolve_config_path(&out_dir, s))
                })
                .unwrap_or_else(|| out_dir.join("screenshots.result.touch"));
        let script_path = std::env::var_os("FRET_DIAG_SCRIPT_PATH")
            .filter(|v| !v.is_empty())
            .map(PathBuf::from)
            .or_else(|| {
                config_file
                    .as_ref()
                    .and_then(|c| c.paths.as_ref())
                    .and_then(|p| p.script_path.as_deref())
                    .and_then(|s| resolve_config_path(&out_dir, s))
            })
            .unwrap_or_else(|| out_dir.join("script.json"));
        let script_trigger_path = std::env::var_os("FRET_DIAG_SCRIPT_TRIGGER_PATH")
            .filter(|v| !v.is_empty())
            .map(PathBuf::from)
            .or_else(|| {
                config_file
                    .as_ref()
                    .and_then(|c| c.paths.as_ref())
                    .and_then(|p| p.script_trigger_path.as_deref())
                    .and_then(|s| resolve_config_path(&out_dir, s))
            })
            .unwrap_or_else(|| out_dir.join("script.touch"));
        let script_result_path = std::env::var_os("FRET_DIAG_SCRIPT_RESULT_PATH")
            .filter(|v| !v.is_empty())
            .map(PathBuf::from)
            .or_else(|| {
                config_file
                    .as_ref()
                    .and_then(|c| c.paths.as_ref())
                    .and_then(|p| p.script_result_path.as_deref())
                    .and_then(|s| resolve_config_path(&out_dir, s))
            })
            .unwrap_or_else(|| out_dir.join("script.result.json"));
        let script_result_trigger_path = std::env::var_os("FRET_DIAG_SCRIPT_RESULT_TRIGGER_PATH")
            .filter(|v| !v.is_empty())
            .map(PathBuf::from)
            .or_else(|| {
                config_file
                    .as_ref()
                    .and_then(|c| c.paths.as_ref())
                    .and_then(|p| p.script_result_trigger_path.as_deref())
                    .and_then(|s| resolve_config_path(&out_dir, s))
            })
            .unwrap_or_else(|| out_dir.join("script.result.touch"));
        let script_auto_dump = env_flag_override("FRET_DIAG_SCRIPT_AUTO_DUMP")
            .or_else(|| config_file.as_ref().and_then(|c| c.script_auto_dump))
            .unwrap_or(true);
        let pick_trigger_path = std::env::var_os("FRET_DIAG_PICK_TRIGGER_PATH")
            .filter(|v| !v.is_empty())
            .map(PathBuf::from)
            .or_else(|| {
                config_file
                    .as_ref()
                    .and_then(|c| c.paths.as_ref())
                    .and_then(|p| p.pick_trigger_path.as_deref())
                    .and_then(|s| resolve_config_path(&out_dir, s))
            })
            .unwrap_or_else(|| out_dir.join("pick.touch"));
        let pick_result_path = std::env::var_os("FRET_DIAG_PICK_RESULT_PATH")
            .filter(|v| !v.is_empty())
            .map(PathBuf::from)
            .or_else(|| {
                config_file
                    .as_ref()
                    .and_then(|c| c.paths.as_ref())
                    .and_then(|p| p.pick_result_path.as_deref())
                    .and_then(|s| resolve_config_path(&out_dir, s))
            })
            .unwrap_or_else(|| out_dir.join("pick.result.json"));
        let pick_result_trigger_path = std::env::var_os("FRET_DIAG_PICK_RESULT_TRIGGER_PATH")
            .filter(|v| !v.is_empty())
            .map(PathBuf::from)
            .or_else(|| {
                config_file
                    .as_ref()
                    .and_then(|c| c.paths.as_ref())
                    .and_then(|p| p.pick_result_trigger_path.as_deref())
                    .and_then(|s| resolve_config_path(&out_dir, s))
            })
            .unwrap_or_else(|| out_dir.join("pick.result.touch"));
        let pick_auto_dump = env_flag_override("FRET_DIAG_PICK_AUTO_DUMP")
            .or_else(|| config_file.as_ref().and_then(|c| c.pick_auto_dump))
            .unwrap_or(true);
        let inspect_path = std::env::var_os("FRET_DIAG_INSPECT_PATH")
            .filter(|v| !v.is_empty())
            .map(PathBuf::from)
            .or_else(|| {
                config_file
                    .as_ref()
                    .and_then(|c| c.paths.as_ref())
                    .and_then(|p| p.inspect_path.as_deref())
                    .and_then(|s| resolve_config_path(&out_dir, s))
            })
            .unwrap_or_else(|| out_dir.join("inspect.json"));
        let inspect_trigger_path = std::env::var_os("FRET_DIAG_INSPECT_TRIGGER_PATH")
            .filter(|v| !v.is_empty())
            .map(PathBuf::from)
            .or_else(|| {
                config_file
                    .as_ref()
                    .and_then(|c| c.paths.as_ref())
                    .and_then(|p| p.inspect_trigger_path.as_deref())
                    .and_then(|s| resolve_config_path(&out_dir, s))
            })
            .unwrap_or_else(|| out_dir.join("inspect.touch"));
        let redact_text = env_flag_override("FRET_DIAG_REDACT_TEXT")
            .or_else(|| config_file.as_ref().and_then(|c| c.redact_text))
            .unwrap_or(true);
        let max_debug_string_bytes = env_usize_override("FRET_DIAG_MAX_DEBUG_STRING_BYTES")
            .or_else(|| {
                config_file
                    .as_ref()
                    .and_then(|c| c.max_debug_string_bytes)
                    .map(|v| v as usize)
            })
            .unwrap_or(4096)
            .clamp(0, 256 * 1024);
        let max_gating_trace_entries = env_usize_override("FRET_DIAG_MAX_GATING_TRACE_ENTRIES")
            .or_else(|| {
                config_file
                    .as_ref()
                    .and_then(|c| c.max_gating_trace_entries)
                    .map(|v| v as usize)
            })
            .unwrap_or(200)
            .clamp(0, 2000);
        let screenshot_on_dump = env_flag_override("FRET_DIAG_BUNDLE_SCREENSHOT")
            .or_else(|| config_file.as_ref().and_then(|c| c.screenshot_on_dump))
            .unwrap_or(false);
        let write_bundle_json = config_file
            .as_ref()
            .and_then(|c| c.write_bundle_json)
            .unwrap_or(true);
        let write_bundle_schema2 = config_file
            .as_ref()
            .and_then(|c| c.write_bundle_schema2)
            .unwrap_or(false);
        let frame_clock_fixed_delta_ms = fret_core::WindowFrameClockService::fixed_delta_from_env()
            .map(|d| d.as_millis())
            .and_then(|ms| u64::try_from(ms).ok())
            .filter(|v| *v > 0)
            .or_else(|| {
                config_file
                    .as_ref()
                    .and_then(|c| c.frame_clock_fixed_delta_ms)
            });

        Self {
            enabled,
            out_dir,
            trigger_path,
            ready_path,
            exit_path,
            allow_script_schema_v1,
            script_keepalive,
            max_events,
            max_snapshots,
            script_dump_max_snapshots,
            capture_semantics,
            max_semantics_nodes,
            semantics_test_ids_only,
            screenshots_enabled,
            screenshot_request_path,
            screenshot_trigger_path,
            screenshot_result_path,
            screenshot_result_trigger_path,
            script_path,
            script_trigger_path,
            script_result_path,
            script_result_trigger_path,
            script_auto_dump,
            pick_trigger_path,
            pick_result_path,
            pick_result_trigger_path,
            pick_auto_dump,
            inspect_path,
            inspect_trigger_path,
            redact_text,
            max_debug_string_bytes,
            max_gating_trace_entries,
            screenshot_on_dump,
            write_bundle_json,
            write_bundle_schema2,
            frame_clock_fixed_delta_ms,
            devtools_ws_url,
            devtools_token,
            devtools_embed_bundle: config_file
                .as_ref()
                .and_then(|c| c.devtools_embed_bundle)
                .unwrap_or(cfg!(target_arch = "wasm32")),
        }
    }
}
