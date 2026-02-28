use std::collections::{BTreeMap, BTreeSet};
use std::path::{Path, PathBuf};

use fret_diag_protocol::{UiDiagnosticsConfigFileV1, UiDiagnosticsConfigPathsV1};

#[derive(Debug, Clone)]
pub(crate) struct ConfigCmdContext {
    pub(crate) rest: Vec<String>,
    pub(crate) workspace_root: PathBuf,
    pub(crate) resolved_out_dir: PathBuf,
    pub(crate) resolved_ready_path: PathBuf,
    pub(crate) resolved_exit_path: PathBuf,
    pub(crate) fs_transport_cfg: crate::transport::FsDiagTransportConfig,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
enum DoctorMode {
    Manual,
    Launch,
}

impl DoctorMode {
    fn from_str(s: &str) -> Option<Self> {
        match s.trim() {
            "manual" => Some(Self::Manual),
            "launch" => Some(Self::Launch),
            _ => None,
        }
    }
}

#[derive(Debug, Clone, PartialEq, Eq)]
enum ValueSource {
    Default,
    Env(&'static str),
    ConfigFile(&'static str),
}

impl ValueSource {
    fn as_str(&self) -> &'static str {
        match self {
            Self::Default => "default",
            Self::Env(name) => name,
            Self::ConfigFile(key) => key,
        }
    }
}

#[derive(Debug, Clone)]
struct SourcedBool {
    value: bool,
    source: ValueSource,
}

#[derive(Debug, Clone)]
struct SourcedUsize {
    value: usize,
    source: ValueSource,
}

#[derive(Debug, Clone)]
struct SourcedPath {
    value: PathBuf,
    source: ValueSource,
}

#[derive(Debug, Clone)]
struct EffectiveConfig {
    enabled: SourcedBool,
    allow_script_schema_v1: SourcedBool,
    out_dir: SourcedPath,
    trigger_path: SourcedPath,
    ready_path: SourcedPath,
    exit_path: SourcedPath,
    script_path: SourcedPath,
    script_trigger_path: SourcedPath,
    script_result_path: SourcedPath,
    script_result_trigger_path: SourcedPath,
    pick_trigger_path: SourcedPath,
    pick_result_path: SourcedPath,
    pick_result_trigger_path: SourcedPath,
    inspect_path: SourcedPath,
    inspect_trigger_path: SourcedPath,
    screenshots_enabled: SourcedBool,
    screenshot_on_dump: SourcedBool,
    write_bundle_json: SourcedBool,
    write_bundle_schema2: SourcedBool,
    redact_text: SourcedBool,
    max_events: SourcedUsize,
    max_snapshots: SourcedUsize,
    script_dump_max_snapshots: SourcedUsize,
    frame_clock_fixed_delta_ms: Option<(u64, ValueSource)>,
    devtools_ws_enabled: SourcedBool,
}

fn env_flag_override(env: &BTreeMap<String, String>, name: &'static str) -> Option<bool> {
    let v = env.get(name)?;
    let raw = v.trim().to_ascii_lowercase();
    if raw.is_empty() {
        return Some(true);
    }
    Some(!matches!(raw.as_str(), "0" | "false" | "no" | "off"))
}

fn env_usize_override(env: &BTreeMap<String, String>, name: &'static str) -> Option<usize> {
    let v = env.get(name)?;
    let raw = v.trim();
    if raw.is_empty() {
        return None;
    }
    raw.parse::<usize>().ok()
}

fn resolve_config_path(out_dir: &Path, raw: &str) -> Option<PathBuf> {
    let raw = raw.trim();
    if raw.is_empty() {
        return None;
    }
    let p = PathBuf::from(raw);
    Some(if p.is_absolute() { p } else { out_dir.join(p) })
}

fn read_config_file_value(path: &Path) -> Result<serde_json::Value, String> {
    let bytes = std::fs::read(path).map_err(|e| e.to_string())?;
    serde_json::from_slice(&bytes).map_err(|e| e.to_string())
}

fn runtime_known_env_vars() -> BTreeSet<&'static str> {
    // Keep this list aligned with the runtime config loader:
    // `ecosystem/fret-bootstrap/src/ui_diagnostics/config.rs` and related diagnostics modules.
    BTreeSet::from([
        "FRET_DIAG",
        "FRET_DIAG_CONFIG_PATH",
        "FRET_DIAG_DIR",
        "FRET_DIAG_TRIGGER_PATH",
        "FRET_DIAG_READY_PATH",
        "FRET_DIAG_EXIT_PATH",
        "FRET_DIAG_SCRIPT_KEEPALIVE",
        "FRET_DIAG_SCRIPT_AUTO_DUMP",
        "FRET_DIAG_PICK_AUTO_DUMP",
        "FRET_DIAG_MAX_EVENTS",
        "FRET_DIAG_MAX_SNAPSHOTS",
        "FRET_DIAG_SCRIPT_DUMP_MAX_SNAPSHOTS",
        "FRET_DIAG_SEMANTICS",
        "FRET_DIAG_MAX_SEMANTICS_NODES",
        "FRET_DIAG_SEMANTICS_TEST_IDS_ONLY",
        "FRET_DIAG_GPU_SCREENSHOTS",
        "FRET_DIAG_SCREENSHOT_REQUEST_PATH",
        "FRET_DIAG_SCREENSHOT_TRIGGER_PATH",
        "FRET_DIAG_SCREENSHOT_RESULT_PATH",
        "FRET_DIAG_SCREENSHOT_RESULT_TRIGGER_PATH",
        "FRET_DIAG_SCRIPT_PATH",
        "FRET_DIAG_SCRIPT_TRIGGER_PATH",
        "FRET_DIAG_SCRIPT_RESULT_PATH",
        "FRET_DIAG_SCRIPT_RESULT_TRIGGER_PATH",
        "FRET_DIAG_PICK_TRIGGER_PATH",
        "FRET_DIAG_PICK_RESULT_PATH",
        "FRET_DIAG_PICK_RESULT_TRIGGER_PATH",
        "FRET_DIAG_INSPECT_PATH",
        "FRET_DIAG_INSPECT_TRIGGER_PATH",
        "FRET_DIAG_REDACT_TEXT",
        "FRET_DIAG_MAX_DEBUG_STRING_BYTES",
        "FRET_DIAG_MAX_GATING_TRACE_ENTRIES",
        "FRET_DIAG_BUNDLE_SCREENSHOT",
        "FRET_DIAG_BUNDLE_JSON_FORMAT",
        "FRET_DIAG_BUNDLE_SEMANTICS_MODE",
        "FRET_DIAG_BUNDLE_WRITE_INDEX",
        "FRET_DIAG_BUNDLE_DUMP_MAX_SEMANTICS_NODES",
        "FRET_DIAG_BUNDLE_DUMP_SEMANTICS_TEST_IDS_ONLY",
        "FRET_DEVTOOLS_WS",
        "FRET_DEVTOOLS_TOKEN",
        // Fixed frame delta override is handled by `fret-core`:
        // `crates/fret-core/src/window.rs`.
        "FRET_DIAG_FIXED_FRAME_DELTA_MS",
        "FRET_FRAME_CLOCK_FIXED_DELTA_MS",
    ])
}

fn canonical_env_vars() -> BTreeSet<&'static str> {
    // Canonical env vars for runtime diagnostics config.
    //
    // Most other `FRET_DIAG_*` env vars are compatibility/escape-hatch inputs.
    BTreeSet::from([
        "FRET_DIAG",
        "FRET_DIAG_CONFIG_PATH",
        "FRET_DIAG_GPU_SCREENSHOTS",
        "FRET_DIAG_REDACT_TEXT",
        "FRET_DIAG_FIXED_FRAME_DELTA_MS",
    ])
}

fn runtime_known_config_keys() -> BTreeSet<&'static str> {
    // Keep this list aligned with `UiDiagnosticsConfigFileV1`.
    BTreeSet::from([
        "schema_version",
        "enabled",
        "out_dir",
        "paths",
        "allow_script_schema_v1",
        "script_keepalive",
        "script_auto_dump",
        "pick_auto_dump",
        "max_events",
        "max_snapshots",
        "script_dump_max_snapshots",
        "capture_semantics",
        "max_semantics_nodes",
        "semantics_test_ids_only",
        "screenshots_enabled",
        "screenshot_on_dump",
        "write_bundle_json",
        "write_bundle_schema2",
        "redact_text",
        "max_debug_string_bytes",
        "max_gating_trace_entries",
        "frame_clock_fixed_delta_ms",
        "devtools_embed_bundle",
    ])
}

fn runtime_known_config_paths_keys() -> BTreeSet<&'static str> {
    // Keep this list aligned with `UiDiagnosticsConfigPathsV1`.
    BTreeSet::from([
        "trigger_path",
        "ready_path",
        "exit_path",
        "screenshot_request_path",
        "screenshot_trigger_path",
        "screenshot_result_path",
        "screenshot_result_trigger_path",
        "script_path",
        "script_trigger_path",
        "script_result_path",
        "script_result_trigger_path",
        "pick_trigger_path",
        "pick_result_path",
        "pick_result_trigger_path",
        "inspect_path",
        "inspect_trigger_path",
    ])
}

fn collect_unknown_config_keys(value: &serde_json::Value) -> Vec<String> {
    let Some(obj) = value.as_object() else {
        return Vec::new();
    };
    let known = runtime_known_config_keys();
    let mut out: Vec<String> = obj
        .keys()
        .filter(|k| !known.contains(k.as_str()))
        .cloned()
        .collect();
    out.sort();
    out
}

fn collect_unknown_config_paths_keys(value: &serde_json::Value) -> Vec<String> {
    let Some(paths) = value.get("paths").and_then(|v| v.as_object()) else {
        return Vec::new();
    };
    let known = runtime_known_config_paths_keys();
    let mut out: Vec<String> = paths
        .keys()
        .filter(|k| !known.contains(k.as_str()))
        .cloned()
        .collect();
    out.sort();
    out
}

fn compute_effective_runtime_config(
    env: &BTreeMap<String, String>,
    config_file: Option<&UiDiagnosticsConfigFileV1>,
) -> EffectiveConfig {
    let config_enabled = config_file
        .map(|c| c.enabled.unwrap_or(true))
        .unwrap_or(false);
    let config_out_dir = config_file
        .and_then(|c| c.out_dir.as_deref())
        .map(|s| s.trim())
        .filter(|s| !s.is_empty())
        .map(PathBuf::from);

    let raw_diag = env
        .get("FRET_DIAG")
        .map(|v| v.trim())
        .filter(|v| !v.is_empty());
    let raw_out_dir = env
        .get("FRET_DIAG_DIR")
        .map(|v| v.trim())
        .filter(|v| !v.is_empty())
        .map(PathBuf::from);

    let diag_enabled = raw_diag.is_some() || raw_out_dir.is_some() || config_enabled;

    let devtools_ws_url = env
        .get("FRET_DEVTOOLS_WS")
        .map(|v| v.trim())
        .filter(|v| !v.is_empty());
    let devtools_token = env
        .get("FRET_DEVTOOLS_TOKEN")
        .map(|v| v.trim())
        .filter(|v| !v.is_empty());
    let devtools_ws_enabled_bool = devtools_ws_url.is_some() && devtools_token.is_some();

    let enabled = if devtools_ws_enabled_bool {
        SourcedBool {
            value: true,
            source: ValueSource::Env("env:FRET_DEVTOOLS_WS+FRET_DEVTOOLS_TOKEN"),
        }
    } else if diag_enabled {
        let source = if raw_diag.is_some() {
            ValueSource::Env("env:FRET_DIAG")
        } else if raw_out_dir.is_some() {
            ValueSource::Env("env:FRET_DIAG_DIR")
        } else if config_file.is_some() {
            ValueSource::ConfigFile("config:enabled")
        } else {
            ValueSource::Default
        };
        SourcedBool {
            value: true,
            source,
        }
    } else {
        SourcedBool {
            value: false,
            source: ValueSource::Default,
        }
    };

    let allow_script_schema_v1 = if let Some(v) = config_file.and_then(|c| c.allow_script_schema_v1)
    {
        SourcedBool {
            value: v,
            source: ValueSource::ConfigFile("config:allow_script_schema_v1"),
        }
    } else {
        SourcedBool {
            value: true,
            source: ValueSource::Default,
        }
    };

    let out_dir = if let Some(p) = raw_out_dir.clone() {
        SourcedPath {
            value: p,
            source: ValueSource::Env("env:FRET_DIAG_DIR"),
        }
    } else if let Some(p) = config_out_dir.clone() {
        SourcedPath {
            value: p,
            source: ValueSource::ConfigFile("config:out_dir"),
        }
    } else {
        SourcedPath {
            value: PathBuf::from("target").join("fret-diag"),
            source: ValueSource::Default,
        }
    };
    let out_dir_for_config_paths = out_dir.value.clone();

    let config_paths: Option<&UiDiagnosticsConfigPathsV1> =
        config_file.and_then(|c| c.paths.as_ref());

    let choose_path = |env_key: &'static str,
                       config_key: &'static str,
                       default_name: &'static str| {
        if let Some(v) = env.get(env_key).map(|v| v.trim()).filter(|v| !v.is_empty()) {
            return SourcedPath {
                value: PathBuf::from(v),
                source: ValueSource::Env(env_key),
            };
        }

        let from_config: Option<&str> = config_paths.and_then(|p| match config_key {
            "trigger_path" => p.trigger_path.as_deref(),
            "ready_path" => p.ready_path.as_deref(),
            "exit_path" => p.exit_path.as_deref(),
            "screenshot_request_path" => p.screenshot_request_path.as_deref(),
            "screenshot_trigger_path" => p.screenshot_trigger_path.as_deref(),
            "screenshot_result_path" => p.screenshot_result_path.as_deref(),
            "screenshot_result_trigger_path" => p.screenshot_result_trigger_path.as_deref(),
            "script_path" => p.script_path.as_deref(),
            "script_trigger_path" => p.script_trigger_path.as_deref(),
            "script_result_path" => p.script_result_path.as_deref(),
            "script_result_trigger_path" => p.script_result_trigger_path.as_deref(),
            "pick_trigger_path" => p.pick_trigger_path.as_deref(),
            "pick_result_path" => p.pick_result_path.as_deref(),
            "pick_result_trigger_path" => p.pick_result_trigger_path.as_deref(),
            "inspect_path" => p.inspect_path.as_deref(),
            "inspect_trigger_path" => p.inspect_trigger_path.as_deref(),
            _ => None,
        });

        if let Some(v) = from_config.and_then(|s| resolve_config_path(&out_dir_for_config_paths, s))
        {
            return SourcedPath {
                value: v,
                source: ValueSource::ConfigFile(config_key),
            };
        }

        SourcedPath {
            value: out_dir_for_config_paths.join(default_name),
            source: ValueSource::Default,
        }
    };

    let trigger_path = choose_path("FRET_DIAG_TRIGGER_PATH", "trigger_path", "trigger.touch");
    let ready_path = choose_path("FRET_DIAG_READY_PATH", "ready_path", "ready.touch");
    let exit_path = choose_path("FRET_DIAG_EXIT_PATH", "exit_path", "exit.touch");

    let script_path = choose_path("FRET_DIAG_SCRIPT_PATH", "script_path", "script.json");
    let script_trigger_path = choose_path(
        "FRET_DIAG_SCRIPT_TRIGGER_PATH",
        "script_trigger_path",
        "script.touch",
    );
    let script_result_path = choose_path(
        "FRET_DIAG_SCRIPT_RESULT_PATH",
        "script_result_path",
        "script.result.json",
    );
    let script_result_trigger_path = choose_path(
        "FRET_DIAG_SCRIPT_RESULT_TRIGGER_PATH",
        "script_result_trigger_path",
        "script.result.touch",
    );

    let pick_trigger_path = choose_path(
        "FRET_DIAG_PICK_TRIGGER_PATH",
        "pick_trigger_path",
        "pick.touch",
    );
    let pick_result_path = choose_path(
        "FRET_DIAG_PICK_RESULT_PATH",
        "pick_result_path",
        "pick.result.json",
    );
    let pick_result_trigger_path = choose_path(
        "FRET_DIAG_PICK_RESULT_TRIGGER_PATH",
        "pick_result_trigger_path",
        "pick.result.touch",
    );

    let inspect_path = choose_path("FRET_DIAG_INSPECT_PATH", "inspect_path", "inspect.json");
    let inspect_trigger_path = choose_path(
        "FRET_DIAG_INSPECT_TRIGGER_PATH",
        "inspect_trigger_path",
        "inspect.touch",
    );

    let screenshots_enabled = if let Some(v) = env_flag_override(env, "FRET_DIAG_GPU_SCREENSHOTS") {
        SourcedBool {
            value: v,
            source: ValueSource::Env("env:FRET_DIAG_GPU_SCREENSHOTS"),
        }
    } else if let Some(v) = config_file.and_then(|c| c.screenshots_enabled) {
        SourcedBool {
            value: v,
            source: ValueSource::ConfigFile("config:screenshots_enabled"),
        }
    } else {
        SourcedBool {
            value: false,
            source: ValueSource::Default,
        }
    };

    let screenshot_on_dump = if let Some(v) = env_flag_override(env, "FRET_DIAG_BUNDLE_SCREENSHOT")
    {
        SourcedBool {
            value: v,
            source: ValueSource::Env("env:FRET_DIAG_BUNDLE_SCREENSHOT"),
        }
    } else if let Some(v) = config_file.and_then(|c| c.screenshot_on_dump) {
        SourcedBool {
            value: v,
            source: ValueSource::ConfigFile("config:screenshot_on_dump"),
        }
    } else {
        SourcedBool {
            value: false,
            source: ValueSource::Default,
        }
    };

    let write_bundle_json = if let Some(v) = config_file.and_then(|c| c.write_bundle_json) {
        SourcedBool {
            value: v,
            source: ValueSource::ConfigFile("config:write_bundle_json"),
        }
    } else {
        SourcedBool {
            value: true,
            source: ValueSource::Default,
        }
    };

    let write_bundle_schema2 = if let Some(v) = config_file.and_then(|c| c.write_bundle_schema2) {
        SourcedBool {
            value: v,
            source: ValueSource::ConfigFile("config:write_bundle_schema2"),
        }
    } else {
        SourcedBool {
            value: false,
            source: ValueSource::Default,
        }
    };

    let redact_text = if let Some(v) = env_flag_override(env, "FRET_DIAG_REDACT_TEXT") {
        SourcedBool {
            value: v,
            source: ValueSource::Env("env:FRET_DIAG_REDACT_TEXT"),
        }
    } else if let Some(v) = config_file.and_then(|c| c.redact_text) {
        SourcedBool {
            value: v,
            source: ValueSource::ConfigFile("config:redact_text"),
        }
    } else {
        SourcedBool {
            value: true,
            source: ValueSource::Default,
        }
    };

    let max_events = if let Some(v) = env_usize_override(env, "FRET_DIAG_MAX_EVENTS") {
        SourcedUsize {
            value: v,
            source: ValueSource::Env("env:FRET_DIAG_MAX_EVENTS"),
        }
    } else if let Some(v) = config_file.and_then(|c| c.max_events).map(|v| v as usize) {
        SourcedUsize {
            value: v,
            source: ValueSource::ConfigFile("config:max_events"),
        }
    } else {
        SourcedUsize {
            value: 2000,
            source: ValueSource::Default,
        }
    };

    let max_snapshots = if let Some(v) = env_usize_override(env, "FRET_DIAG_MAX_SNAPSHOTS") {
        SourcedUsize {
            value: v,
            source: ValueSource::Env("env:FRET_DIAG_MAX_SNAPSHOTS"),
        }
    } else if let Some(v) = config_file
        .and_then(|c| c.max_snapshots)
        .map(|v| v as usize)
    {
        SourcedUsize {
            value: v,
            source: ValueSource::ConfigFile("config:max_snapshots"),
        }
    } else {
        SourcedUsize {
            value: 300,
            source: ValueSource::Default,
        }
    };

    let raw_script_dump_max =
        if let Some(v) = env_usize_override(env, "FRET_DIAG_SCRIPT_DUMP_MAX_SNAPSHOTS") {
            Some((
                v,
                ValueSource::Env("env:FRET_DIAG_SCRIPT_DUMP_MAX_SNAPSHOTS"),
            ))
        } else if let Some(v) = config_file
            .and_then(|c| c.script_dump_max_snapshots)
            .map(|v| v as usize)
        {
            Some((
                v,
                ValueSource::ConfigFile("config:script_dump_max_snapshots"),
            ))
        } else {
            None
        };

    let (script_dump_max_snapshots, script_dump_source) =
        if let Some((v, src)) = raw_script_dump_max {
            (v, src)
        } else {
            (30, ValueSource::Default)
        };
    let script_dump_max_snapshots = if max_snapshots.value == 0 {
        0
    } else {
        script_dump_max_snapshots.clamp(1, max_snapshots.value)
    };
    let script_dump_max_snapshots = SourcedUsize {
        value: script_dump_max_snapshots,
        source: script_dump_source,
    };

    let frame_clock_fixed_delta_ms = {
        let mut out: Option<(u64, ValueSource)> = None;

        if let Some(v) = env
            .get("FRET_DIAG_FIXED_FRAME_DELTA_MS")
            .map(|v| v.trim())
            .filter(|v| !v.is_empty())
        {
            if let Ok(parsed) = v.parse::<u64>() {
                if parsed > 0 {
                    out = Some((
                        parsed,
                        ValueSource::Env("env:FRET_DIAG_FIXED_FRAME_DELTA_MS"),
                    ));
                }
            }
        }

        if out.is_none() {
            if let Some(v) = env
                .get("FRET_FRAME_CLOCK_FIXED_DELTA_MS")
                .map(|v| v.trim())
                .filter(|v| !v.is_empty())
            {
                if let Ok(parsed) = v.parse::<u64>() {
                    if parsed > 0 {
                        out = Some((
                            parsed,
                            ValueSource::Env("env:FRET_FRAME_CLOCK_FIXED_DELTA_MS"),
                        ));
                    }
                }
            }
        }

        if out.is_none() {
            if let Some(v) = config_file.and_then(|c| c.frame_clock_fixed_delta_ms) {
                if v > 0 {
                    out = Some((
                        v,
                        ValueSource::ConfigFile("config:frame_clock_fixed_delta_ms"),
                    ));
                }
            }
        }

        out
    };

    let devtools_ws_enabled = SourcedBool {
        value: devtools_ws_enabled_bool,
        source: if devtools_ws_enabled_bool {
            ValueSource::Env("env:FRET_DEVTOOLS_WS+FRET_DEVTOOLS_TOKEN")
        } else {
            ValueSource::Default
        },
    };

    EffectiveConfig {
        enabled,
        allow_script_schema_v1,
        out_dir,
        trigger_path,
        ready_path,
        exit_path,
        script_path,
        script_trigger_path,
        script_result_path,
        script_result_trigger_path,
        pick_trigger_path,
        pick_result_path,
        pick_result_trigger_path,
        inspect_path,
        inspect_trigger_path,
        screenshots_enabled,
        screenshot_on_dump,
        write_bundle_json,
        write_bundle_schema2,
        redact_text,
        max_events,
        max_snapshots,
        script_dump_max_snapshots,
        frame_clock_fixed_delta_ms,
        devtools_ws_enabled,
    }
}

fn env_snapshot_from_process() -> BTreeMap<String, String> {
    std::env::vars().collect()
}

fn overlay_launch_reserved_env(
    base: &BTreeMap<String, String>,
    ready_path: &Path,
    exit_path: &Path,
    fs_transport_cfg: &crate::transport::FsDiagTransportConfig,
) -> (BTreeMap<String, String>, Vec<String>) {
    let mut env = base.clone();
    let mut overridden: Vec<String> = Vec::new();

    let out_dir = &fs_transport_cfg.out_dir;
    let config_path = out_dir.join("diag.config.json");
    let reserved: [(&'static str, String); 19] = [
        ("FRET_DIAG", "1".to_string()),
        ("FRET_DIAG_DIR", out_dir.display().to_string()),
        (
            "FRET_DIAG_TRIGGER_PATH",
            fs_transport_cfg.trigger_path.display().to_string(),
        ),
        ("FRET_DIAG_READY_PATH", ready_path.display().to_string()),
        ("FRET_DIAG_EXIT_PATH", exit_path.display().to_string()),
        ("FRET_DIAG_CONFIG_PATH", config_path.display().to_string()),
        (
            "FRET_DIAG_SCRIPT_PATH",
            fs_transport_cfg.script_path.display().to_string(),
        ),
        (
            "FRET_DIAG_SCRIPT_TRIGGER_PATH",
            fs_transport_cfg.script_trigger_path.display().to_string(),
        ),
        (
            "FRET_DIAG_SCRIPT_RESULT_PATH",
            fs_transport_cfg.script_result_path.display().to_string(),
        ),
        (
            "FRET_DIAG_SCRIPT_RESULT_TRIGGER_PATH",
            fs_transport_cfg
                .script_result_trigger_path
                .display()
                .to_string(),
        ),
        (
            "FRET_DIAG_PICK_TRIGGER_PATH",
            fs_transport_cfg.pick_trigger_path.display().to_string(),
        ),
        (
            "FRET_DIAG_PICK_RESULT_PATH",
            fs_transport_cfg.pick_result_path.display().to_string(),
        ),
        (
            "FRET_DIAG_PICK_RESULT_TRIGGER_PATH",
            fs_transport_cfg
                .pick_result_trigger_path
                .display()
                .to_string(),
        ),
        (
            "FRET_DIAG_INSPECT_PATH",
            fs_transport_cfg.inspect_path.display().to_string(),
        ),
        (
            "FRET_DIAG_INSPECT_TRIGGER_PATH",
            fs_transport_cfg.inspect_trigger_path.display().to_string(),
        ),
        (
            "FRET_DIAG_SCREENSHOT_REQUEST_PATH",
            fs_transport_cfg
                .screenshots_request_path
                .display()
                .to_string(),
        ),
        (
            "FRET_DIAG_SCREENSHOT_TRIGGER_PATH",
            fs_transport_cfg
                .screenshots_trigger_path
                .display()
                .to_string(),
        ),
        (
            "FRET_DIAG_SCREENSHOT_RESULT_PATH",
            fs_transport_cfg
                .screenshots_result_path
                .display()
                .to_string(),
        ),
        (
            "FRET_DIAG_SCREENSHOT_RESULT_TRIGGER_PATH",
            fs_transport_cfg
                .screenshots_result_trigger_path
                .display()
                .to_string(),
        ),
    ];

    for (k, v) in reserved {
        if env.get(k).is_some_and(|prev| prev != &v) {
            overridden.push(k.to_string());
        }
        env.insert(k.to_string(), v);
    }

    (env, overridden)
}

fn config_doctor_report_json(
    effective_env: &BTreeMap<String, String>,
    config_path: Option<&Path>,
    config_value: Option<&serde_json::Value>,
    config_file: Option<&UiDiagnosticsConfigFileV1>,
    effective: &EffectiveConfig,
    warnings: &[serde_json::Value],
) -> serde_json::Value {
    let mut env_set: Vec<serde_json::Value> = effective_env
        .iter()
        .filter(|(k, v)| k.starts_with("FRET_") && !v.trim().is_empty())
        .map(|(k, v)| serde_json::json!({ "key": k, "value": v }))
        .collect();
    env_set.sort_by(|a, b| {
        a.get("key")
            .and_then(|v| v.as_str())
            .cmp(&b.get("key").and_then(|v| v.as_str()))
    });

    let config_unknown_keys = config_value
        .map(collect_unknown_config_keys)
        .unwrap_or_default();
    let config_unknown_paths_keys = config_value
        .map(collect_unknown_config_paths_keys)
        .unwrap_or_default();

    let frame_delta = effective
        .frame_clock_fixed_delta_ms
        .as_ref()
        .map(|(v, src)| {
            serde_json::json!({
                "value_ms": v,
                "source": src.as_str(),
            })
        });

    serde_json::json!({
        "schema_version": 1,
        "kind": "diag_config_doctor_report",
        "inputs": {
            "config_path": config_path.map(|p| p.display().to_string()),
            "config_loaded": config_file.is_some(),
            "env_set": env_set,
        },
        "config_file": {
            "schema_version": config_file.map(|c| c.schema_version),
            "unknown_keys": config_unknown_keys,
            "unknown_paths_keys": config_unknown_paths_keys,
        },
        "effective": {
            "enabled": { "value": effective.enabled.value, "source": effective.enabled.source.as_str() },
            "allow_script_schema_v1": { "value": effective.allow_script_schema_v1.value, "source": effective.allow_script_schema_v1.source.as_str() },
            "out_dir": { "value": effective.out_dir.value.display().to_string(), "source": effective.out_dir.source.as_str() },
            "trigger_path": { "value": effective.trigger_path.value.display().to_string(), "source": effective.trigger_path.source.as_str() },
            "ready_path": { "value": effective.ready_path.value.display().to_string(), "source": effective.ready_path.source.as_str() },
            "exit_path": { "value": effective.exit_path.value.display().to_string(), "source": effective.exit_path.source.as_str() },
            "script_path": { "value": effective.script_path.value.display().to_string(), "source": effective.script_path.source.as_str() },
            "script_trigger_path": { "value": effective.script_trigger_path.value.display().to_string(), "source": effective.script_trigger_path.source.as_str() },
            "script_result_path": { "value": effective.script_result_path.value.display().to_string(), "source": effective.script_result_path.source.as_str() },
            "script_result_trigger_path": { "value": effective.script_result_trigger_path.value.display().to_string(), "source": effective.script_result_trigger_path.source.as_str() },
            "pick_trigger_path": { "value": effective.pick_trigger_path.value.display().to_string(), "source": effective.pick_trigger_path.source.as_str() },
            "pick_result_path": { "value": effective.pick_result_path.value.display().to_string(), "source": effective.pick_result_path.source.as_str() },
            "pick_result_trigger_path": { "value": effective.pick_result_trigger_path.value.display().to_string(), "source": effective.pick_result_trigger_path.source.as_str() },
            "inspect_path": { "value": effective.inspect_path.value.display().to_string(), "source": effective.inspect_path.source.as_str() },
            "inspect_trigger_path": { "value": effective.inspect_trigger_path.value.display().to_string(), "source": effective.inspect_trigger_path.source.as_str() },
            "screenshots_enabled": { "value": effective.screenshots_enabled.value, "source": effective.screenshots_enabled.source.as_str() },
            "redact_text": { "value": effective.redact_text.value, "source": effective.redact_text.source.as_str() },
            "max_events": { "value": effective.max_events.value, "source": effective.max_events.source.as_str() },
            "max_snapshots": { "value": effective.max_snapshots.value, "source": effective.max_snapshots.source.as_str() },
            "script_dump_max_snapshots": { "value": effective.script_dump_max_snapshots.value, "source": effective.script_dump_max_snapshots.source.as_str() },
            "frame_clock_fixed_delta_ms": frame_delta,
            "devtools_ws_enabled": { "value": effective.devtools_ws_enabled.value, "source": effective.devtools_ws_enabled.source.as_str() },
        },
        "warnings": warnings,
    })
}

fn print_sourced_path(name: &str, v: &SourcedPath) {
    println!(
        "{name}: {}  (source={})",
        v.value.display(),
        v.source.as_str()
    );
}

fn print_sourced_bool(name: &str, v: &SourcedBool) {
    println!("{name}: {}  (source={})", v.value, v.source.as_str());
}

fn print_sourced_usize(name: &str, v: &SourcedUsize) {
    println!("{name}: {}  (source={})", v.value, v.source.as_str());
}

fn cmd_config_doctor(ctx: ConfigCmdContext, rest: &[String]) -> Result<(), String> {
    let mut mode: DoctorMode = DoctorMode::Launch;
    let mut report_json: bool = false;
    let mut config_path_override: Option<PathBuf> = None;
    let mut show_env_all: bool = false;

    let mut i: usize = 0;
    while i < rest.len() {
        let arg = rest[i].as_str();
        match arg {
            "--report-json" => {
                report_json = true;
                i += 1;
            }
            "--show-env" => {
                i += 1;
                let Some(v) = rest.get(i).map(|s| s.as_str()) else {
                    return Err("missing value for --show-env (expected: set|all)".to_string());
                };
                show_env_all = match v {
                    "set" => false,
                    "all" => true,
                    _ => {
                        return Err("invalid value for --show-env (expected: set|all)".to_string());
                    }
                };
                i += 1;
            }
            "--mode" => {
                i += 1;
                let Some(v) = rest.get(i).map(|s| s.as_str()) else {
                    return Err("missing value for --mode (expected: launch|manual)".to_string());
                };
                mode = DoctorMode::from_str(v).ok_or_else(|| {
                    "invalid value for --mode (expected: launch|manual)".to_string()
                })?;
                i += 1;
            }
            "--config-path" => {
                i += 1;
                let Some(v) = rest.get(i).cloned() else {
                    return Err("missing value for --config-path".to_string());
                };
                config_path_override =
                    Some(crate::resolve_path(&ctx.workspace_root, PathBuf::from(v)));
                i += 1;
            }
            "--help" | "-h" => {
                println!(
                    "Usage: fretboard diag config doctor [--mode launch|manual] [--config-path <path>] [--show-env set|all] [--report-json]\n\n\
Default mode is `launch`, which overlays the tooling-reserved env vars (FRET_DIAG*, ready/exit paths) as if you were using `--launch`.\n\
Use `--mode manual` to report what the runtime would see from the current process env only."
                );
                return Ok(());
            }
            other if other.starts_with('-') => {
                return Err(format!("unknown config doctor flag: {other}"));
            }
            other => {
                return Err(format!(
                    "unexpected positional arg for config doctor: {other}"
                ));
            }
        }
    }

    let env_base = env_snapshot_from_process();
    let (effective_env, overridden_reserved) = match mode {
        DoctorMode::Manual => (env_base, Vec::new()),
        DoctorMode::Launch => overlay_launch_reserved_env(
            &env_base,
            &ctx.resolved_ready_path,
            &ctx.resolved_exit_path,
            &ctx.fs_transport_cfg,
        ),
    };

    let default_launch_cfg_path = ctx.fs_transport_cfg.out_dir.join("diag.config.json");
    let config_path = config_path_override
        .or_else(|| {
            effective_env
                .get("FRET_DIAG_CONFIG_PATH")
                .map(|v| v.trim().to_string())
                .filter(|v| !v.is_empty())
                .map(PathBuf::from)
        })
        .or_else(|| {
            (mode == DoctorMode::Launch && default_launch_cfg_path.is_file())
                .then(|| default_launch_cfg_path.clone())
        });

    let mut warnings: Vec<serde_json::Value> = Vec::new();

    if mode == DoctorMode::Launch && !overridden_reserved.is_empty() {
        warnings.push(serde_json::json!({
            "severity": "warning",
            "code": "diag.config.tooling_reserved_env_overrides",
            "message": "tooling-reserved env vars would override values from the current shell env in --launch mode",
            "keys": overridden_reserved,
        }));
    }

    let mut config_value: Option<serde_json::Value> = None;
    let mut config_file: Option<UiDiagnosticsConfigFileV1> = None;
    if let Some(path) = config_path.as_ref() {
        if path.is_file() {
            match read_config_file_value(path) {
                Ok(v) => {
                    config_value = Some(v.clone());
                    match serde_json::from_value::<UiDiagnosticsConfigFileV1>(v) {
                        Ok(cfg) => {
                            if cfg.schema_version != 1 {
                                warnings.push(serde_json::json!({
                                    "severity": "warning",
                                    "code": "diag.config.schema_version_mismatch",
                                    "message": "config file schema_version is not 1; the runtime may ignore/partially apply fields",
                                    "schema_version": cfg.schema_version,
                                }));
                            }
                            config_file = Some(cfg);
                        }
                        Err(err) => {
                            warnings.push(serde_json::json!({
                                "severity": "warning",
                                "code": "diag.config.parse_failed",
                                "message": "failed to parse config file as UiDiagnosticsConfigFileV1",
                                "error": err.to_string(),
                            }));
                        }
                    }
                }
                Err(err) => {
                    warnings.push(serde_json::json!({
                        "severity": "warning",
                        "code": "diag.config.read_failed",
                        "message": "failed to read config file",
                        "error": err,
                    }));
                }
            }
        } else {
            warnings.push(serde_json::json!({
                "severity": "info",
                "code": "diag.config.missing",
                "message": "config file path is set but the file does not exist",
                "path": path.display().to_string(),
            }));
        }
    } else if mode == DoctorMode::Launch && !default_launch_cfg_path.is_file() {
        warnings.push(serde_json::json!({
            "severity": "info",
            "code": "diag.config.launch_default_missing",
            "message": "no config file was found at the default launch path; tooling would normally write this file when launching",
            "path": default_launch_cfg_path.display().to_string(),
        }));
    }

    if let Some(v) = config_value.as_ref() {
        let unknown = collect_unknown_config_keys(v);
        if !unknown.is_empty() {
            warnings.push(serde_json::json!({
                "severity": "warning",
                "code": "diag.config.unknown_keys",
                "message": "config file contains unknown keys (likely ignored by runtime)",
                "keys": unknown,
            }));
        }
        let unknown_paths = collect_unknown_config_paths_keys(v);
        if !unknown_paths.is_empty() {
            warnings.push(serde_json::json!({
                "severity": "warning",
                "code": "diag.config.unknown_paths_keys",
                "message": "config file contains unknown paths.* keys (likely ignored by runtime)",
                "keys": unknown_paths,
            }));
        }
    }

    let known_env = runtime_known_env_vars();
    let mut unknown_env: Vec<String> = effective_env
        .iter()
        .filter(|(k, v)| k.starts_with("FRET_DIAG_") && !v.trim().is_empty())
        .map(|(k, _)| k.clone())
        .filter(|k| !known_env.contains(k.as_str()))
        .collect();
    unknown_env.sort();
    if !unknown_env.is_empty() {
        warnings.push(serde_json::json!({
            "severity": "warning",
            "code": "diag.config.unknown_env_vars",
            "message": "unknown FRET_DIAG_* env vars are set (possibly deprecated or ignored by the runtime)",
            "keys": unknown_env,
        }));
    }

    if effective_env
        .get("FRET_FRAME_CLOCK_FIXED_DELTA_MS")
        .map(|v| !v.trim().is_empty())
        .unwrap_or(false)
    {
        warnings.push(serde_json::json!({
            "severity": "info",
            "code": "diag.config.fixed_frame_delta_generic_alias",
            "message": "FRET_FRAME_CLOCK_FIXED_DELTA_MS is set; prefer FRET_DIAG_FIXED_FRAME_DELTA_MS for diagnostics",
        }));
    }

    if mode == DoctorMode::Manual {
        let canonical = canonical_env_vars();
        let mut compat_env: Vec<String> = effective_env
            .iter()
            .filter(|(k, v)| k.starts_with("FRET_DIAG_") && !v.trim().is_empty())
            .map(|(k, _)| k.clone())
            .filter(|k| !canonical.contains(k.as_str()))
            .collect();
        compat_env.sort();
        if !compat_env.is_empty() && config_path.is_some() && config_file.is_some() {
            warnings.push(serde_json::json!({
                "severity": "info",
                "code": "diag.config.compat_env_vars_present",
                "message": "compatibility env vars are set; prefer FRET_DIAG_CONFIG_PATH + the canonical overrides",
                "canonical": canonical.into_iter().collect::<Vec<_>>(),
                "compat": compat_env,
            }));
        }
    }

    let effective = compute_effective_runtime_config(&effective_env, config_file.as_ref());
    push_output_risk_warnings(
        mode,
        &effective_env,
        config_file.as_ref(),
        &effective,
        &mut warnings,
    );

    if report_json {
        let report = config_doctor_report_json(
            &effective_env,
            config_path.as_deref(),
            config_value.as_ref(),
            config_file.as_ref(),
            &effective,
            &warnings,
        );
        println!(
            "{}",
            serde_json::to_string_pretty(&report).map_err(|e| e.to_string())?
        );
        return Ok(());
    }

    println!("diag config doctor");
    println!(
        "mode: {}",
        match mode {
            DoctorMode::Launch => "launch",
            DoctorMode::Manual => "manual",
        }
    );
    println!("workspace_root: {}", ctx.workspace_root.display());
    println!("tooling_out_dir: {}", ctx.resolved_out_dir.display());
    if let Some(p) = config_path.as_deref() {
        println!("config_path: {}", p.display());
    } else {
        println!("config_path: (none)");
    }

    if show_env_all {
        println!("env: (all FRET_* set)");
        let mut keys: Vec<&String> = effective_env
            .keys()
            .filter(|k| k.starts_with("FRET_"))
            .collect();
        keys.sort();
        for k in keys {
            let v = effective_env.get(k).map(|v| v.as_str()).unwrap_or("");
            println!("  {k}={v}");
        }
    } else {
        println!("env: (set FRET_* only)");
        let mut keys: Vec<&String> = effective_env
            .iter()
            .filter(|(k, v)| k.starts_with("FRET_") && !v.trim().is_empty())
            .map(|(k, _)| k)
            .collect();
        keys.sort();
        for k in keys {
            let v = effective_env.get(k).map(|v| v.as_str()).unwrap_or("");
            println!("  {k}={v}");
        }
    }

    if !warnings.is_empty() {
        println!("warnings:");
        for w in warnings.iter() {
            let sev = w.get("severity").and_then(|v| v.as_str()).unwrap_or("info");
            let code = w.get("code").and_then(|v| v.as_str()).unwrap_or("unknown");
            let msg = w.get("message").and_then(|v| v.as_str()).unwrap_or("");
            println!("  - {sev} {code}: {msg}");
        }
    }

    println!("effective:");
    print_sourced_bool("enabled", &effective.enabled);
    print_sourced_bool("allow_script_schema_v1", &effective.allow_script_schema_v1);
    print_sourced_path("out_dir", &effective.out_dir);
    print_sourced_path("trigger_path", &effective.trigger_path);
    print_sourced_path("ready_path", &effective.ready_path);
    print_sourced_path("exit_path", &effective.exit_path);
    print_sourced_path("script_path", &effective.script_path);
    print_sourced_path("script_trigger_path", &effective.script_trigger_path);
    print_sourced_path("script_result_path", &effective.script_result_path);
    print_sourced_path(
        "script_result_trigger_path",
        &effective.script_result_trigger_path,
    );
    print_sourced_path("pick_trigger_path", &effective.pick_trigger_path);
    print_sourced_path("pick_result_path", &effective.pick_result_path);
    print_sourced_path(
        "pick_result_trigger_path",
        &effective.pick_result_trigger_path,
    );
    print_sourced_path("inspect_path", &effective.inspect_path);
    print_sourced_path("inspect_trigger_path", &effective.inspect_trigger_path);
    print_sourced_bool("screenshots_enabled", &effective.screenshots_enabled);
    print_sourced_bool("screenshot_on_dump", &effective.screenshot_on_dump);
    print_sourced_bool("write_bundle_json", &effective.write_bundle_json);
    print_sourced_bool("write_bundle_schema2", &effective.write_bundle_schema2);
    print_sourced_bool("redact_text", &effective.redact_text);
    print_sourced_usize("max_events", &effective.max_events);
    print_sourced_usize("max_snapshots", &effective.max_snapshots);
    print_sourced_usize(
        "script_dump_max_snapshots",
        &effective.script_dump_max_snapshots,
    );
    if let Some((ms, src)) = effective.frame_clock_fixed_delta_ms.as_ref() {
        println!(
            "frame_clock_fixed_delta_ms: {ms}  (source={})",
            src.as_str()
        );
    } else {
        println!("frame_clock_fixed_delta_ms: (none)");
    }
    print_sourced_bool("devtools_ws_enabled", &effective.devtools_ws_enabled);

    Ok(())
}

fn parse_env_u64(env: &BTreeMap<String, String>, key: &'static str) -> Option<u64> {
    let raw = env.get(key)?.trim();
    if raw.is_empty() {
        return None;
    }
    raw.parse::<u64>().ok()
}

fn push_output_risk_warnings(
    mode: DoctorMode,
    effective_env: &BTreeMap<String, String>,
    config_file: Option<&UiDiagnosticsConfigFileV1>,
    effective: &EffectiveConfig,
    warnings: &mut Vec<serde_json::Value>,
) {
    let launch_like = mode == DoctorMode::Launch;

    if launch_like && config_file.is_none() {
        warnings.push(serde_json::json!({
            "severity": "warning",
            "code": "diag.config.launch_missing_config_file",
            "message": "launch-mode config file is missing/unreadable; runtime defaults may write large bundle.json artifacts",
            "suggest": "ensure tooling can write <dir>/diag.config.json (or pass --config-path to config doctor)",
        }));
    }

    if launch_like && effective.write_bundle_json.value {
        warnings.push(serde_json::json!({
            "severity": "warning",
            "code": "diag.config.large_bundle_json_enabled",
            "message": "write_bundle_json=true can produce very large bundle.json artifacts; prefer small-by-default settings for tool-launched runs",
            "suggest": "set write_bundle_json=false and write_bundle_schema2=true in diag.config.json",
            "source": effective.write_bundle_json.source.as_str(),
        }));
    }

    if launch_like && !effective.write_bundle_schema2.value {
        warnings.push(serde_json::json!({
            "severity": "info",
            "code": "diag.config.schema2_companion_disabled",
            "message": "write_bundle_schema2=false disables the compact schema2 companion artifact; tooling prefers bundle.schema2.json when present",
            "suggest": "set write_bundle_schema2=true in diag.config.json for launched runs",
            "source": effective.write_bundle_schema2.source.as_str(),
        }));
    }

    if launch_like
        && effective.write_bundle_json.value
        && effective_env
            .get("FRET_DIAG_BUNDLE_JSON_FORMAT")
            .is_some_and(|v| v.trim() == "pretty")
    {
        warnings.push(serde_json::json!({
            "severity": "warning",
            "code": "diag.config.bundle_json_pretty_print",
            "message": "FRET_DIAG_BUNDLE_JSON_FORMAT=pretty can significantly increase bundle.json size",
            "suggest": "use compact JSON (unset the env var) or disable bundle.json via write_bundle_json=false",
        }));
    }

    if launch_like && effective.max_snapshots.value > 600 {
        warnings.push(serde_json::json!({
            "severity": "warning",
            "code": "diag.config.max_snapshots_high",
            "message": "max_snapshots is high; large rings increase memory use and can inflate bundle exports",
            "max_snapshots": effective.max_snapshots.value,
            "source": effective.max_snapshots.source.as_str(),
            "suggest": "keep max_snapshots near the default unless you have a specific need",
        }));
    }

    if launch_like && effective.script_dump_max_snapshots.value > 30 {
        warnings.push(serde_json::json!({
            "severity": "warning",
            "code": "diag.config.script_dump_max_snapshots_high",
            "message": "script_dump_max_snapshots is high; scripted dumps can easily produce very large bundles",
            "script_dump_max_snapshots": effective.script_dump_max_snapshots.value,
            "source": effective.script_dump_max_snapshots.source.as_str(),
            "suggest": "for shareable repros, prefer <= 10 (tool-launched default)",
        }));
    }

    if launch_like
        && effective_env
            .get("FRET_DIAG_BUNDLE_SEMANTICS_MODE")
            .is_some_and(|v| v.trim() == "all")
    {
        warnings.push(serde_json::json!({
            "severity": "warning",
            "code": "diag.config.semantics_mode_all",
            "message": "FRET_DIAG_BUNDLE_SEMANTICS_MODE=all exports semantics on every snapshot and can massively increase bundle size",
            "suggest": "prefer mode=last for script-driven dumps unless you need per-frame semantics",
        }));
    }

    let max_semantics_nodes = parse_env_u64(effective_env, "FRET_DIAG_MAX_SEMANTICS_NODES");
    if launch_like && max_semantics_nodes.is_some_and(|v| v > 100_000) {
        warnings.push(serde_json::json!({
            "severity": "warning",
            "code": "diag.config.max_semantics_nodes_high",
            "message": "FRET_DIAG_MAX_SEMANTICS_NODES is high; semantics exports can dominate bundle size for large UIs",
            "max_semantics_nodes": max_semantics_nodes,
            "suggest": "consider a lower cap or enable FRET_DIAG_SEMANTICS_TEST_IDS_ONLY=1 for test_id-focused triage",
        }));
    }

    let dump_max_semantics_nodes =
        parse_env_u64(effective_env, "FRET_DIAG_BUNDLE_DUMP_MAX_SEMANTICS_NODES");
    if launch_like && dump_max_semantics_nodes.is_some_and(|v| v > 100_000) {
        warnings.push(serde_json::json!({
            "severity": "warning",
            "code": "diag.config.bundle_dump_max_semantics_nodes_high",
            "message": "FRET_DIAG_BUNDLE_DUMP_MAX_SEMANTICS_NODES is high; bundle dumps can become huge for large UIs",
            "max_semantics_nodes": dump_max_semantics_nodes,
            "suggest": "prefer a smaller cap for shareable scripted repros",
        }));
    }

    let max_debug_string_bytes = parse_env_u64(effective_env, "FRET_DIAG_MAX_DEBUG_STRING_BYTES");
    if launch_like && max_debug_string_bytes.is_some_and(|v| v > 16 * 1024) {
        warnings.push(serde_json::json!({
            "severity": "info",
            "code": "diag.config.max_debug_string_bytes_high",
            "message": "FRET_DIAG_MAX_DEBUG_STRING_BYTES is high; large debug strings can inflate exports",
            "max_debug_string_bytes": max_debug_string_bytes,
            "suggest": "prefer 2048 (tool-launched default) for shareable artifacts",
        }));
    }

    let max_gating_trace_entries =
        parse_env_u64(effective_env, "FRET_DIAG_MAX_GATING_TRACE_ENTRIES");
    if launch_like && max_gating_trace_entries.is_some_and(|v| v > 500) {
        warnings.push(serde_json::json!({
            "severity": "info",
            "code": "diag.config.max_gating_trace_entries_high",
            "message": "FRET_DIAG_MAX_GATING_TRACE_ENTRIES is high; large traces can inflate exports",
            "max_gating_trace_entries": max_gating_trace_entries,
            "suggest": "prefer 200 (default) unless you need deeper command gating traces",
        }));
    }
}

pub(crate) fn cmd_config(ctx: ConfigCmdContext) -> Result<(), String> {
    let Some(sub) = ctx.rest.first().map(|s| s.as_str()) else {
        return Err(
            "missing diag config subcommand (try: fretboard diag config doctor)".to_string(),
        );
    };
    let rest: Vec<String> = ctx.rest.iter().skip(1).cloned().collect();
    match sub {
        "doctor" => cmd_config_doctor(ctx, &rest),
        other => Err(format!(
            "unknown diag config subcommand: {other} (expected: doctor)"
        )),
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    fn repo_root_for_tests() -> PathBuf {
        // `CARGO_MANIFEST_DIR` is `<repo>/crates/fret-diag`.
        let dir = PathBuf::from(env!("CARGO_MANIFEST_DIR"));
        dir.parent()
            .and_then(|p| p.parent())
            .map(|p| p.to_path_buf())
            .expect("repo root")
    }

    #[test]
    fn unknown_env_vars_are_reported_by_list_membership() {
        let mut env = BTreeMap::new();
        env.insert("FRET_DIAG".to_string(), "1".to_string());
        env.insert("FRET_DIAG_SOMETHING_UNKNOWN".to_string(), "1".to_string());
        let known = runtime_known_env_vars();
        assert!(known.contains("FRET_DIAG"));
        assert!(!known.contains("FRET_DIAG_SOMETHING_UNKNOWN"));
        let mut unknown: Vec<String> = env
            .iter()
            .filter(|(k, v)| k.starts_with("FRET_DIAG_") && !v.trim().is_empty())
            .map(|(k, _)| k.clone())
            .filter(|k| !known.contains(k.as_str()))
            .collect();
        unknown.sort();
        assert_eq!(unknown, vec!["FRET_DIAG_SOMETHING_UNKNOWN".to_string()]);
    }

    #[test]
    fn script_dump_max_snapshots_clamps_to_max_snapshots() {
        let mut env = BTreeMap::new();
        env.insert("FRET_DIAG".to_string(), "1".to_string());
        env.insert("FRET_DIAG_MAX_SNAPSHOTS".to_string(), "10".to_string());
        env.insert(
            "FRET_DIAG_SCRIPT_DUMP_MAX_SNAPSHOTS".to_string(),
            "999".to_string(),
        );
        let cfg = compute_effective_runtime_config(&env, None);
        assert_eq!(cfg.max_snapshots.value, 10);
        assert_eq!(cfg.script_dump_max_snapshots.value, 10);
    }

    #[test]
    fn example_config_file_has_no_unknown_keys() {
        let root = repo_root_for_tests();
        let path = root.join("tools/diag-configs/diag.config.example.json");
        assert!(path.is_file(), "missing example config: {}", path.display());

        let value = read_config_file_value(&path).expect("read example config");
        let unknown = collect_unknown_config_keys(&value);
        assert!(
            unknown.is_empty(),
            "example config has unknown keys: {:?}",
            unknown
        );
        let unknown_paths = collect_unknown_config_paths_keys(&value);
        assert!(
            unknown_paths.is_empty(),
            "example config has unknown paths keys: {:?}",
            unknown_paths
        );
        let cfg: UiDiagnosticsConfigFileV1 =
            serde_json::from_value(value).expect("parse example config");
        assert_eq!(cfg.schema_version, 1);
    }
}
