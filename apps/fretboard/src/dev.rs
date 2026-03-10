use std::path::Path;
use std::process::{Child, Command};
use std::time::{Duration, Instant, SystemTime};

use crate::cli::{help, workspace_root};
use crate::demos::{
    cookbook_example_feature_hint, display_path, list_cookbook_examples_from,
    list_native_demos_from, official_native_demos, prompt_choose_demo, validate_cookbook_example,
    validate_native_demo, validate_web_demo, web_demos_as_vec,
};
use crate::hotpatch::{
    HotpatchBuildIdArg, ensure_hotpatch_trigger_file_initialized, parse_hotpatch_build_id,
    resolve_workspace_relative,
};

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
enum HotpatchModeSummary {
    None,
    DxAuto,
    DxExplicit,
    Devserver,
    ReloadBoundary,
}

fn demo_id_to_native_bin(demo: &str, native_bins: &[String]) -> Option<String> {
    // `--demo` is intended to be aligned with web IDs where possible. Native demo binaries are
    // snake_case and often end in `_demo`, so we try a small set of conservative candidates.
    //
    // Some demos have intentionally different ids across platforms; handle those explicitly.
    let override_bin = match demo {
        // Web uses a wasm-only copy-path demo; native uses a different implementation id.
        "external_texture_imports_web_demo" => Some("external_texture_imports_demo"),
        _ => None,
    };
    if let Some(bin) = override_bin
        && native_bins.iter().any(|b| b == bin)
    {
        return Some(bin.to_string());
    }

    let normalized = demo.replace('-', "_");
    let mut candidates: Vec<String> = Vec::new();

    candidates.push(demo.to_string());
    if normalized != demo {
        candidates.push(normalized.clone());
    }
    if !normalized.ends_with("_demo") {
        candidates.push(format!("{normalized}_demo"));
    }

    candidates
        .into_iter()
        .find(|c| native_bins.iter().any(|b| b == c))
}

fn print_hotpatch_summary(
    mode: HotpatchModeSummary,
    bin: &str,
    demo_hotpatch_ready: bool,
    dx_available: bool,
    ws: Option<&str>,
    build_id: Option<u64>,
    trigger_path: Option<&std::path::Path>,
) {
    if mode == HotpatchModeSummary::None {
        return;
    }

    let mode_str = match mode {
        HotpatchModeSummary::None => "none",
        HotpatchModeSummary::DxAuto => "dx (auto)",
        HotpatchModeSummary::DxExplicit => "dx",
        HotpatchModeSummary::Devserver => "devserver",
        HotpatchModeSummary::ReloadBoundary => "reload-boundary",
    };

    #[derive(Debug, Clone, Copy, PartialEq, Eq)]
    enum ViewCallStrategy {
        Auto,
        HotFn,
        Direct,
    }

    fn parse_view_call_strategy(raw: &str) -> Option<ViewCallStrategy> {
        match raw.trim().to_ascii_lowercase().as_str() {
            "auto" => Some(ViewCallStrategy::Auto),
            "hotfn" => Some(ViewCallStrategy::HotFn),
            "direct" => Some(ViewCallStrategy::Direct),
            _ => None,
        }
    }

    let legacy_direct =
        std::env::var_os("FRET_HOTPATCH_VIEW_CALL_DIRECT").is_some_and(|v| !v.is_empty());
    let env_strategy = if legacy_direct {
        Some(ViewCallStrategy::Direct)
    } else if let Ok(raw) = std::env::var("FRET_HOTPATCH_VIEW_CALL_STRATEGY") {
        parse_view_call_strategy(&raw).or(Some(ViewCallStrategy::Auto))
    } else {
        Some(ViewCallStrategy::Auto)
    };

    let effective_use_direct = match env_strategy.unwrap_or(ViewCallStrategy::Auto) {
        ViewCallStrategy::Direct => true,
        ViewCallStrategy::HotFn => false,
        ViewCallStrategy::Auto => cfg!(windows),
    };

    let view_call = match (effective_use_direct, legacy_direct, env_strategy) {
        (true, true, _) => {
            "direct (view hotpatch disabled; FRET_HOTPATCH_VIEW_CALL_DIRECT=1)".to_string()
        }
        (true, false, Some(ViewCallStrategy::Direct)) => {
            "direct (view hotpatch disabled; FRET_HOTPATCH_VIEW_CALL_STRATEGY=direct)".to_string()
        }
        (true, false, Some(ViewCallStrategy::Auto)) if cfg!(windows) => {
            "direct (view hotpatch disabled; auto Windows safety default)".to_string()
        }
        (true, _, _) => "direct (view hotpatch disabled)".to_string(),
        (false, _, Some(ViewCallStrategy::HotFn)) => {
            "hotfn (FRET_HOTPATCH_VIEW_CALL_STRATEGY=hotfn)".to_string()
        }
        (false, _, Some(ViewCallStrategy::Auto)) if !cfg!(windows) => "hotfn".to_string(),
        (false, _, _) => "hotfn".to_string(),
    };

    eprintln!("Hotpatch Summary:");
    eprintln!("  bin: {bin}");
    eprintln!("  demo_hotpatch_ready: {demo_hotpatch_ready}");
    eprintln!("  mode: {mode_str}");
    eprintln!("  dx_available: {dx_available}");
    eprintln!("  view_call: {view_call}");
    if let Some(ws) = ws {
        eprintln!("  ws: {ws}");
    }
    if let Some(build_id) = build_id {
        eprintln!("  build_id: {build_id}");
    }
    if let Some(trigger_path) = trigger_path {
        eprintln!("  trigger: {}", trigger_path.display());
    }
    eprintln!("  logs:");
    eprintln!("    runner: .fret/hotpatch_runner.log");
    eprintln!("    view:   .fret/hotpatch_bootstrap.log");
    eprintln!("  status: fretboard hotpatch status --tail 40");
}

fn append_subsecond_main_export_rustflags(cmd: &mut Command) {
    // Subsecond uses `main` as an ASLR anchor on native platforms. Some toolchains don't export
    // `main` by default, which makes `subsecond::aslr_reference()` return 0 and disables hotpatch.
    //
    // Dioxus's `dx serve --hotpatch` injects equivalent linker args; in "devserver-only" mode we
    // set them explicitly so connecting to a Dioxus-style devserver can work.
    let extra: &'static str = {
        #[cfg(all(windows, not(target_arch = "wasm32")))]
        {
            "-C link-arg=/HIGHENTROPYVA:NO -C link-arg=/EXPORT:main"
        }

        #[cfg(all(target_os = "macos", not(target_arch = "wasm32"), not(windows)))]
        {
            "-C link-arg=-Wl,-exported_symbol,_main"
        }

        #[cfg(all(
            any(target_os = "linux", target_os = "android", target_os = "freebsd"),
            not(target_arch = "wasm32"),
            not(windows)
        ))]
        {
            "-C link-arg=-Wl,--export-dynamic-symbol,main"
        }

        #[cfg(any(target_arch = "wasm32", target_family = "wasm"))]
        {
            ""
        }

        #[cfg(all(
            not(windows),
            not(target_os = "macos"),
            not(any(target_os = "linux", target_os = "android", target_os = "freebsd")),
            not(any(target_arch = "wasm32", target_family = "wasm"))
        ))]
        {
            ""
        }
    };

    if extra.is_empty() {
        return;
    }

    let mut rustflags = std::env::var("RUSTFLAGS").unwrap_or_default();
    if rustflags.contains("/EXPORT:main")
        || rustflags.contains("--export-dynamic-symbol,main")
        || rustflags.contains("-exported_symbol,_main")
    {
        return;
    }

    if !rustflags.trim().is_empty() {
        rustflags.push(' ');
    }
    rustflags.push_str(extra);
    cmd.env("RUSTFLAGS", rustflags);
}

pub(crate) fn dev_native(args: Vec<String>) -> Result<(), String> {
    let root = workspace_root()?;
    let demos = list_native_demos_from(&root)?;
    let cookbook_examples = list_cookbook_examples_from(&root)?;

    let mut bin: Option<String> = None;
    let mut demo: Option<String> = None;
    let mut example: Option<String> = None;
    let mut cargo_profile: Option<String> = None;
    let mut choose = false;
    let mut include_maintainer = false;
    let mut hotpatch = false;
    let mut hotpatch_reload_only = false;
    let mut hotpatch_trigger_path: Option<String> = None;
    let mut hotpatch_poll_ms: Option<u64> = None;
    let mut hotpatch_devserver_ws: Option<String> = None;
    let mut hotpatch_build_id: Option<HotpatchBuildIdArg> = None;
    let mut hotpatch_dx = false;
    let mut hotpatch_dx_ws: Option<String> = None;
    let mut supervise: Option<bool> = None;
    let mut watch: Option<bool> = None;
    let mut watch_poll_ms: Option<u64> = None;
    let mut dev_state_reset = false;
    let mut passthrough: Vec<String> = Vec::new();

    let mut it = args.into_iter();
    while let Some(a) = it.next() {
        match a.as_str() {
            "--bin" => {
                bin = Some(
                    it.next()
                        .ok_or_else(|| "--bin requires a value".to_string())?,
                );
            }
            "--demo" => {
                demo = Some(
                    it.next()
                        .ok_or_else(|| "--demo requires a value".to_string())?,
                );
            }
            "--example" => {
                example = Some(
                    it.next()
                        .ok_or_else(|| "--example requires a value".to_string())?,
                );
            }
            "--cargo-profile" | "--profile" => {
                cargo_profile = Some(
                    it.next()
                        .ok_or_else(|| "--profile requires a value".to_string())?,
                );
            }
            "--choose" => choose = true,
            "--all" => include_maintainer = true,
            "--hotpatch" => hotpatch = true,
            "--hotpatch-reload" => {
                hotpatch = true;
                hotpatch_reload_only = true;
            }
            "--hotpatch-trigger-path" => {
                hotpatch_trigger_path = Some(
                    it.next()
                        .ok_or_else(|| "--hotpatch-trigger-path requires a value".to_string())?,
                );
            }
            "--hotpatch-poll-ms" => {
                let raw = it
                    .next()
                    .ok_or_else(|| "--hotpatch-poll-ms requires a value".to_string())?;
                hotpatch_poll_ms = Some(raw.parse::<u64>().map_err(|e| e.to_string())?);
            }
            "--hotpatch-devserver" => {
                hotpatch_devserver_ws = Some(
                    it.next()
                        .ok_or_else(|| "--hotpatch-devserver requires a value".to_string())?,
                );
            }
            "--hotpatch-dx" => hotpatch_dx = true,
            "--hotpatch-dx-ws" => {
                hotpatch_dx_ws = Some(
                    it.next()
                        .ok_or_else(|| "--hotpatch-dx-ws requires a value".to_string())?,
                );
            }
            "--supervise" => supervise = Some(true),
            "--no-supervise" => supervise = Some(false),
            "--watch" => watch = Some(true),
            "--no-watch" => watch = Some(false),
            "--dev-state-reset" => dev_state_reset = true,
            "--watch-poll-ms" => {
                let raw = it
                    .next()
                    .ok_or_else(|| "--watch-poll-ms requires a value".to_string())?;
                watch_poll_ms = Some(raw.parse::<u64>().map_err(|e| e.to_string())?);
            }
            "--hotpatch-build-id" => {
                let raw = it
                    .next()
                    .ok_or_else(|| "--hotpatch-build-id requires a value".to_string())?;
                hotpatch_build_id = Some(parse_hotpatch_build_id(&raw)?);
            }
            "--" => {
                passthrough.extend(it);
                break;
            }
            "--help" | "-h" => return help(),
            other => return Err(format!("unknown argument for dev native: {other}")),
        }
    }

    let selection_count = demo.is_some() as u32 + bin.is_some() as u32 + example.is_some() as u32;
    if selection_count > 1 {
        return Err("cannot combine --demo/--bin/--example (choose exactly one)".to_string());
    }

    if (demo.is_some() || example.is_some()) && choose {
        return Err("cannot combine --choose with --demo/--example".to_string());
    }

    if example.is_some()
        && (hotpatch
            || hotpatch_devserver_ws.is_some()
            || hotpatch_dx
            || watch.unwrap_or(false)
            || hotpatch_trigger_path.is_some()
            || hotpatch_poll_ms.is_some()
            || hotpatch_build_id.is_some())
    {
        return Err(
            "cookbook examples do not support --hotpatch/--watch yet (use `--bin` demos for now)"
                .to_string(),
        );
    }

    if let Some(example) = example.as_deref() {
        validate_cookbook_example(&cookbook_examples, example)?;

        let mut cmd = Command::new("cargo");
        cmd.current_dir(&root).args(["run"]);

        let default_profile = cfg!(windows).then_some("dev-fast");
        if let Some(profile) = cargo_profile.as_deref().or(default_profile) {
            #[cfg(windows)]
            if cargo_profile.is_none() {
                eprintln!(
                    "Note: Windows default uses `--profile dev-fast` for faster builds (override with: --profile dev)."
                );
            }
            cmd.args(["--profile", profile]);
        }

        if let Some(hint) = cookbook_example_feature_hint(example) {
            let Some(features) = hint.trim().strip_prefix("--features ") else {
                return Err(format!(
                    "internal error: unexpected cookbook feature hint format for `{example}`: `{hint}`"
                ));
            };
            eprintln!("note: auto-enabled for `{example}`: --features {features}");
            cmd.args(["--features", features]);
        }

        cmd.args(["-p", "fret-cookbook", "--example", example]);

        if dev_state_reset {
            cmd.env("FRET_DEV_STATE", "1");
            cmd.env("FRET_DEV_STATE_DEBOUNCE_MS", "0");
            cmd.env("FRET_DEV_STATE_RESET", "1");
        }

        if !passthrough.is_empty() {
            cmd.arg("--").args(passthrough);
        }

        let status = cmd.status().map_err(|e| e.to_string())?;
        if !status.success() {
            return Err(format!("cargo exited with status: {status}"));
        }
        return Ok(());
    }

    // Prefer running native demos by `--bin` even when the user passed a web-style `--demo` id.
    // This keeps behavior consistent (feature flags, hotpatch/watch support, etc.), and avoids
    // relying on the `apps/fret-demo` selection shell having perfect parity with web ids.
    if let Some(demo_id) = demo.as_deref()
        && let Some(mapped) = demo_id_to_native_bin(demo_id, &demos)
    {
        bin = Some(mapped);
        demo = None;
    }

    let demo_needs_bin = demo.is_some()
        && (hotpatch
            || hotpatch_devserver_ws.is_some()
            || hotpatch_dx
            || watch.unwrap_or(false)
            || hotpatch_trigger_path.is_some()
            || hotpatch_poll_ms.is_some()
            || hotpatch_build_id.is_some());

    if demo_needs_bin {
        let demo_id = demo.as_deref().unwrap_or_default();
        return Err(format!(
            "cannot combine `--demo {demo_id}` with --hotpatch/--watch because it does not map to a native demo binary.\n  hint: try `fretboard list native-demos` and use `--bin <name>`.\n  hint: if you only want to run the demo shell, omit --hotpatch/--watch."
        ));
    }

    // `--demo` runs the `apps/fret-demo` demo-selection shell (mirrors web's `?demo=...`).
    // Keep this path conservative: hotpatch and watch are currently bin-centric (`--bin`).
    if let Some(demo) = demo.as_deref() {
        // Keep `--demo` aligned with web demo IDs when possible.
        if demo != "todo_demo" {
            validate_web_demo(demo)?;
        }

        let mut cmd = Command::new("cargo");
        cmd.current_dir(&root).args(["run"]);

        let default_profile = cfg!(windows).then_some("dev-fast");
        if let Some(profile) = cargo_profile.as_deref().or(default_profile) {
            #[cfg(windows)]
            if cargo_profile.is_none() {
                eprintln!(
                    "Note: Windows default uses `--profile dev-fast` for faster builds (override with: --profile dev)."
                );
            }
            cmd.args(["--profile", profile]);
        }

        cmd.args(["-p", "fret-demo"]);

        if dev_state_reset {
            cmd.env("FRET_DEV_STATE", "1");
            cmd.env("FRET_DEV_STATE_DEBOUNCE_MS", "0");
            cmd.env("FRET_DEV_STATE_RESET", "1");
        }

        cmd.arg("--").arg(demo);
        if !passthrough.is_empty() {
            cmd.args(passthrough);
        }

        let status = cmd.status().map_err(|e| e.to_string())?;
        if !status.success() {
            return Err(format!("cargo exited with status: {status}"));
        }
        return Ok(());
    }

    if hotpatch && hotpatch_devserver_ws.is_some() {
        return Err("cannot combine --hotpatch and --hotpatch-devserver".to_string());
    }
    if hotpatch_dx && (hotpatch || hotpatch_devserver_ws.is_some()) {
        return Err(
            "cannot combine --hotpatch-dx with --hotpatch/--hotpatch-devserver".to_string(),
        );
    }

    let choose_demos = if include_maintainer {
        demos.clone()
    } else {
        official_native_demos(&demos)
    };

    let bin = match (bin.as_deref(), choose) {
        (Some(name), _) => {
            validate_native_demo(&demos, name)?;
            name.to_string()
        }
        (None, true) => prompt_choose_demo(
            "Select a native demo",
            &choose_demos,
            Some("todo_demo"),
            |name| validate_native_demo(&demos, name),
        )?,
        (None, false) => "todo_demo".to_string(),
    };

    let dx_available = dx_available();
    let hotpatch_auto_uses_dx = hotpatch
        && !hotpatch_reload_only
        && hotpatch_devserver_ws.is_none()
        && dx_available
        && is_hotpatch_ready_native_demo(&bin);

    let mode_summary = if hotpatch_dx {
        HotpatchModeSummary::DxExplicit
    } else if hotpatch_auto_uses_dx {
        HotpatchModeSummary::DxAuto
    } else if hotpatch_devserver_ws.is_some() {
        HotpatchModeSummary::Devserver
    } else if hotpatch {
        HotpatchModeSummary::ReloadBoundary
    } else {
        HotpatchModeSummary::None
    };

    if (hotpatch || hotpatch_devserver_ws.is_some()) && !is_hotpatch_ready_native_demo(&bin) {
        eprintln!(
            "warning: `{bin}` is not a hotpatch-ready demo. Hotpatch will only trigger a safe runner reload boundary.\n  try: `--bin todo_demo` or `--bin assets_demo` for the FnDriver/UiAppDriver hotpatch path"
        );
    }

    if hotpatch_dx || hotpatch_auto_uses_dx {
        let ws = hotpatch_dx_ws.as_deref().unwrap_or("<dx-managed>");
        print_hotpatch_summary(
            mode_summary,
            &bin,
            is_hotpatch_ready_native_demo(&bin),
            dx_available,
            Some(ws),
            None,
            None,
        );
        return dev_native_hotpatch_dx(
            &root,
            &bin,
            hotpatch_dx_ws.as_deref(),
            hotpatch_build_id,
            passthrough,
        );
    }

    let effective_watch =
        watch.unwrap_or(hotpatch && !dx_available && hotpatch_devserver_ws.is_none());
    let effective_watch_poll = Duration::from_millis(watch_poll_ms.unwrap_or(800));

    // In reload-boundary/devserver mode, `cargo run` is the supervisor. If the app crashes,
    // it's easy to end up in a frustrating "rerun command manually" loop.
    //
    // For hotpatch-focused workflows, default to a lightweight restart supervisor that prints
    // actionable guidance on repeated failures. This keeps the inner loop closer to `dx serve`
    // without embedding a full file-watcher build system in `fretboard` (L1 scope).
    let effective_supervise =
        supervise.unwrap_or(hotpatch || hotpatch_devserver_ws.is_some() || effective_watch);

    let mut cmd = Command::new("cargo");
    cmd.current_dir(&root);

    if effective_watch || hotpatch || hotpatch_devserver_ws.is_some() || dev_state_reset {
        cmd.env("FRET_DEV_STATE", "1");
    }
    if effective_watch || hotpatch || hotpatch_devserver_ws.is_some() {
        // In watch/hotpatch workflows, favor a tighter "state is always there after restart"
        // loop over minimizing JSON writes.
        cmd.env("FRET_DEV_STATE_DEBOUNCE_MS", "0");
    }
    if dev_state_reset {
        cmd.env("FRET_DEV_STATE_RESET", "1");
    }

    // When `--watch` is enabled, prefer `cargo build` + running the produced binary directly so we can
    // reliably terminate and restart the app process on Windows.
    let cargo_subcommand = if effective_watch { "build" } else { "run" };
    cmd.arg(cargo_subcommand);

    let default_profile = cfg!(windows).then_some("dev-fast");
    if let Some(profile) = cargo_profile.as_deref().or(default_profile) {
        #[cfg(windows)]
        if cargo_profile.is_none() {
            eprintln!(
                "Note: Windows default uses `--profile dev-fast` for faster builds (override with: --profile dev)."
            );
        }
        cmd.args(["--profile", profile]);
    }

    cmd.args(["-p", "fret-demo"]);
    let mut cargo_features: Vec<&str> = Vec::new();
    if hotpatch || hotpatch_devserver_ws.is_some() {
        cargo_features.push("hotpatch");
        cmd.env("FRET_HOTPATCH", "1");
    }
    if matches!(bin.as_str(), "node_graph_demo") {
        cargo_features.push("node-graph-demos");
    } else if matches!(
        bin.as_str(),
        "node_graph_domain_demo" | "node_graph_legacy_demo" | "imui_node_graph_demo"
    ) {
        cargo_features.push("node-graph-demos-legacy");
    }
    let cargo_features = cargo_features.join(",");
    if !cargo_features.is_empty() {
        cmd.args(["--features", &cargo_features]);
    }
    if hotpatch {
        let trigger_path = hotpatch_trigger_path
            .as_deref()
            .unwrap_or(".fret/hotpatch.touch");
        let trigger_path = resolve_workspace_relative(&root, trigger_path);

        print_hotpatch_summary(
            mode_summary,
            &bin,
            is_hotpatch_ready_native_demo(&bin),
            dx_available,
            None,
            None,
            Some(&trigger_path),
        );
        if hotpatch_reload_only {
            eprintln!("  note: forced reload-boundary mode (--hotpatch-reload)");
        } else if dx_available {
            eprintln!(
                "  note: dx (dioxus-cli) is available, but this run is using reload-boundary mode"
            );
            eprintln!(
                "    tip: omit --hotpatch-reload or use --hotpatch to run in dx hotpatch mode"
            );
        }

        // Ensure the trigger file exists before the app starts so the runner can capture the
        // initial marker without forcing an immediate hot reload.
        ensure_hotpatch_trigger_file_initialized(&trigger_path)?;

        eprintln!(
            "Hotpatch(file-trigger): enabled (note: this only triggers a runner reload boundary; it does not rebuild/apply patches)"
        );
        eprintln!("  trigger: {}", trigger_path.display());
        eprintln!("  poke:    fretboard hotpatch poke");

        cmd.env("FRET_HOTPATCH_TRIGGER_PATH", &trigger_path);
        if let Some(ms) = hotpatch_poll_ms {
            cmd.env("FRET_HOTPATCH_POLL_MS", ms.to_string());
        }
    }
    if let Some(ws) = hotpatch_devserver_ws.as_deref() {
        cmd.env("FRET_HOTPATCH_DEVSERVER_WS", ws);

        let build_id = match hotpatch_build_id.unwrap_or(HotpatchBuildIdArg::Auto) {
            HotpatchBuildIdArg::None => None,
            // Default to not forcing a build id. Dioxus devservers often assign their own build id,
            // and filtering can accidentally ignore valid patches ("no ASLR reference"/no match).
            HotpatchBuildIdArg::Auto => None,
            HotpatchBuildIdArg::Value(v) => Some(v),
        };

        print_hotpatch_summary(
            mode_summary,
            &bin,
            is_hotpatch_ready_native_demo(&bin),
            dx_available,
            Some(ws),
            build_id,
            None,
        );
        eprintln!(
            "  note: this expects an external devserver that delivers Subsecond JumpTables (e.g. dioxus-cli)"
        );

        if let Some(build_id) = build_id {
            cmd.env("FRET_HOTPATCH_BUILD_ID", build_id.to_string());
        }

        // Ensure `main` is exported so `subsecond::aslr_reference()` can succeed.
        append_subsecond_main_export_rustflags(&mut cmd);

        #[cfg(windows)]
        {
            eprintln!(
                "  windows note: default view_call strategy is `direct` (safe; view hotpatch disabled); see docs/adr/0105-dev-hotpatch-subsecond-and-hot-reload-safety.md"
            );
            eprintln!(
                "    to force view-level hotpatching: set FRET_HOTPATCH_VIEW_CALL_STRATEGY=hotfn (may crash)"
            );
        }
    }
    cmd.args(["--bin", &bin]);

    if !effective_watch {
        if !passthrough.is_empty() {
            cmd.arg("--").args(passthrough);
        }

        if effective_supervise {
            run_with_restart_supervisor(
                cmd,
                RestartSupervisorOptions {
                    bin: bin.clone(),
                    dx_available,
                    demo_hotpatch_ready: is_hotpatch_ready_native_demo(&bin),
                    hotpatch_enabled: hotpatch || hotpatch_devserver_ws.is_some(),
                    max_restarts: 5,
                    crash_window: Duration::from_secs(60),
                    crash_threshold: Duration::from_secs(10),
                },
            )?;
        } else {
            let status = cmd.status().map_err(|e| e.to_string())?;
            if !status.success() {
                return Err(format!("cargo exited with status: {status}"));
            }
        }

        return Ok(());
    }

    let build_env = capture_command_env(&cmd);
    dev_native_watch_build_and_run(
        &root,
        &bin,
        cmd,
        build_env,
        passthrough,
        WorkspaceWatchOptions {
            poll_interval: effective_watch_poll,
        },
        RestartSupervisorOptions {
            bin: bin.clone(),
            dx_available,
            demo_hotpatch_ready: is_hotpatch_ready_native_demo(&bin),
            hotpatch_enabled: hotpatch || hotpatch_devserver_ws.is_some(),
            max_restarts: 20,
            crash_window: Duration::from_secs(60),
            crash_threshold: Duration::from_secs(10),
        },
    )
}

#[derive(Debug, Clone, Copy)]
struct WorkspaceWatchOptions {
    poll_interval: Duration,
}

#[derive(Debug, Clone)]
struct CapturedEnv(Vec<(std::ffi::OsString, std::ffi::OsString)>);

fn capture_command_env(cmd: &Command) -> CapturedEnv {
    let pairs = cmd
        .get_envs()
        .filter_map(|(k, v)| Some((k.to_os_string(), v?.to_os_string())));
    CapturedEnv(pairs.collect())
}

fn apply_captured_env(cmd: &mut Command, env: &CapturedEnv) {
    for (k, v) in env.0.iter() {
        cmd.env(k, v);
    }
}

fn dev_native_watch_build_and_run(
    workspace_root: &Path,
    bin: &str,
    mut build_cmd: Command,
    build_env: CapturedEnv,
    passthrough: Vec<String>,
    watch: WorkspaceWatchOptions,
    supervisor_opts: RestartSupervisorOptions,
) -> Result<(), String> {
    eprintln!(
        "Watch: enabled (poll_ms={})",
        watch.poll_interval.as_millis()
    );
    eprintln!("  note: this is a rebuild+restart loop (not Subsecond patch building)");

    let mut watcher = WorkspaceWatch::new(workspace_root, watch.poll_interval);
    watcher.baseline()?;

    let restart_trigger_path = workspace_root.join(".fret").join("watch_restart.touch");
    ensure_restart_trigger_file_initialized(&restart_trigger_path)?;

    loop {
        // Build step
        let status = build_cmd.status().map_err(|e| e.to_string())?;
        if !status.success() {
            eprintln!("error: build failed (status: {status})");
            eprintln!("  waiting for changes...");
            watcher.wait_for_change()?;
            continue;
        }

        // Run step
        let exe = dev_native_exe_path(workspace_root, bin);
        if !exe.is_file() {
            return Err(format!(
                "expected built binary at `{}` but it was not found",
                exe.display()
            ));
        }

        let mut run_cmd = Command::new(&exe);
        run_cmd.current_dir(workspace_root);
        apply_captured_env(&mut run_cmd, &build_env);
        run_cmd.env("FRET_WATCH_RESTART_TRIGGER_PATH", &restart_trigger_path);
        if !passthrough.is_empty() {
            run_cmd.args(passthrough.iter());
        }

        let mut child = run_cmd.spawn().map_err(|e| e.to_string())?;
        let start = Instant::now();

        loop {
            if watcher.poll_changed()? {
                eprintln!("Watch: change detected, restarting...");
                request_graceful_watch_restart(&mut child, &restart_trigger_path)?;

                // Re-create the build command to ensure any transient state is cleared.
                let mut next_build = Command::new("cargo");
                next_build.current_dir(workspace_root);
                next_build.args(build_cmd.get_args());
                apply_captured_env(&mut next_build, &build_env);
                build_cmd = next_build;
                break;
            }

            if let Some(status) = child.try_wait().map_err(|e| e.to_string())? {
                if status.success() {
                    return Ok(());
                }

                let elapsed = Instant::now().duration_since(start);
                if elapsed <= supervisor_opts.crash_threshold {
                    print_repeated_crash_guidance(&supervisor_opts, "fast exit detected");
                } else {
                    eprintln!("warning: process exited with status: {status}");
                    eprintln!("  waiting for changes (or press Ctrl+C to stop)...");
                }

                watcher.wait_for_change()?;

                // Ensure we rebuild after a crash once a change occurs.
                break;
            }

            std::thread::sleep(watch.poll_interval);
        }
    }
}

fn ensure_restart_trigger_file_initialized(path: &Path) -> Result<(), String> {
    if path.is_file() {
        return Ok(());
    }
    poke_restart_trigger_file(path)
}

fn poke_restart_trigger_file(path: &Path) -> Result<(), String> {
    if let Some(parent) = path.parent() {
        std::fs::create_dir_all(parent).map_err(|e| e.to_string())?;
    }
    let marker = SystemTime::now()
        .duration_since(SystemTime::UNIX_EPOCH)
        .map(|d| format!("restart-{}", d.as_millis()))
        .unwrap_or_else(|_| "restart".to_string());
    std::fs::write(path, marker).map_err(|e| e.to_string())
}

fn request_graceful_watch_restart(child: &mut Child, trigger_path: &Path) -> Result<(), String> {
    // Best-effort: ask the app process to exit cleanly so it can flush dev-state, then fall back
    // to killing it if it doesn't exit in time.
    if child.try_wait().map_err(|e| e.to_string())?.is_some() {
        return Ok(());
    }

    let _ = poke_restart_trigger_file(trigger_path);
    let start = Instant::now();
    let timeout = Duration::from_millis(1500);
    loop {
        if child.try_wait().map_err(|e| e.to_string())?.is_some() {
            return Ok(());
        }
        if start.elapsed() >= timeout {
            break;
        }
        std::thread::sleep(Duration::from_millis(50));
    }

    let _ = child.kill();
    let _ = child.wait();
    Ok(())
}

fn dev_native_exe_path(workspace_root: &Path, bin: &str) -> std::path::PathBuf {
    let mut p = workspace_root.join("target").join("debug").join(bin);
    if cfg!(windows) {
        p.set_extension("exe");
    }
    p
}

struct WorkspaceWatch {
    root: std::path::PathBuf,
    poll_interval: Duration,
    last_sig: Option<u64>,
}

impl WorkspaceWatch {
    fn new(root: &Path, poll_interval: Duration) -> Self {
        Self {
            root: root.to_path_buf(),
            poll_interval,
            last_sig: None,
        }
    }

    fn baseline(&mut self) -> Result<(), String> {
        self.last_sig = Some(self.scan_signature()?);
        Ok(())
    }

    fn poll_changed(&mut self) -> Result<bool, String> {
        let sig = self.scan_signature()?;
        let changed = self.last_sig.is_some_and(|prev| prev != sig);
        self.last_sig = Some(sig);
        Ok(changed)
    }

    fn wait_for_change(&mut self) -> Result<(), String> {
        loop {
            if self.poll_changed()? {
                return Ok(());
            }
            std::thread::sleep(self.poll_interval);
        }
    }

    fn scan_signature(&self) -> Result<u64, String> {
        use std::hash::Hasher as _;

        let mut hasher = std::collections::hash_map::DefaultHasher::new();

        // Always include root-level build configuration files.
        for path in [
            self.root.join("Cargo.toml"),
            self.root.join("Cargo.lock"),
            self.root.join("rust-toolchain.toml"),
        ] {
            self.hash_file_stamp(&path, &mut hasher)?;
        }

        // Watch the Rust workspace surface but exclude extremely large areas (`repo-ref`, `target`, `.git`).
        for dir in ["apps", "crates", "ecosystem"] {
            let p = self.root.join(dir);
            self.walk_and_hash(&p, &mut hasher)?;
        }

        Ok(hasher.finish())
    }

    fn walk_and_hash(
        &self,
        root: &Path,
        hasher: &mut std::collections::hash_map::DefaultHasher,
    ) -> Result<(), String> {
        let mut stack = vec![root.to_path_buf()];
        while let Some(dir) = stack.pop() {
            let read_dir = match std::fs::read_dir(&dir) {
                Ok(rd) => rd,
                Err(_) => continue,
            };

            for entry in read_dir.flatten() {
                let path = entry.path();
                let file_name = entry.file_name();
                let file_name = file_name.to_string_lossy();

                if entry.file_type().map(|t| t.is_dir()).unwrap_or(false) {
                    if file_name == "target"
                        || file_name == ".git"
                        || file_name == ".fret"
                        || file_name == "repo-ref"
                        || file_name == "docs"
                    {
                        continue;
                    }
                    stack.push(path);
                    continue;
                }

                if !self.should_watch_file(&path) {
                    continue;
                }

                self.hash_file_stamp(&path, hasher)?;
            }
        }
        Ok(())
    }

    fn should_watch_file(&self, path: &Path) -> bool {
        let Some(ext) = path.extension().and_then(|s| s.to_str()) else {
            return false;
        };
        matches!(ext, "rs" | "toml" | "lock" | "wgsl" | "ron")
    }

    fn hash_file_stamp(
        &self,
        path: &Path,
        hasher: &mut std::collections::hash_map::DefaultHasher,
    ) -> Result<(), String> {
        use std::hash::Hash as _;

        let meta = match std::fs::metadata(path) {
            Ok(m) => m,
            Err(_) => {
                // Missing files affect the signature; include the path only.
                path.to_string_lossy().hash(hasher);
                return Ok(());
            }
        };

        let modified = meta.modified().ok();
        let nanos = modified
            .and_then(|t| t.duration_since(std::time::UNIX_EPOCH).ok())
            .map(|d| d.as_nanos())
            .unwrap_or(0);

        path.to_string_lossy().hash(hasher);
        meta.len().hash(hasher);
        nanos.hash(hasher);
        Ok(())
    }
}

#[derive(Debug, Clone)]
struct RestartSupervisorOptions {
    bin: String,
    dx_available: bool,
    demo_hotpatch_ready: bool,
    hotpatch_enabled: bool,
    max_restarts: usize,
    crash_window: Duration,
    crash_threshold: Duration,
}

fn run_with_restart_supervisor(
    mut cmd: Command,
    opts: RestartSupervisorOptions,
) -> Result<(), String> {
    let mut crash_times: std::collections::VecDeque<Instant> = std::collections::VecDeque::new();
    let mut restarts = 0usize;

    loop {
        let start = Instant::now();
        let status = cmd.status().map_err(|e| e.to_string())?;
        if status.success() {
            return Ok(());
        }

        // Ctrl+C or similar user interrupt: exit cleanly without "crash" guidance.
        if status.code() == Some(130) {
            return Ok(());
        }

        // Cargo uses 101 for "compilation failed". Restarting doesn't help without a rebuild trigger.
        if status.code() == Some(101) {
            return Err(format!("cargo exited with status: {status}"));
        }

        let elapsed = Instant::now().duration_since(start);
        if elapsed <= opts.crash_threshold {
            crash_times.push_back(Instant::now());
        }
        while crash_times
            .front()
            .is_some_and(|t| Instant::now().duration_since(*t) > opts.crash_window)
        {
            crash_times.pop_front();
        }

        restarts += 1;
        if restarts > opts.max_restarts {
            eprintln!(
                "error: dev supervisor exceeded max restarts ({}).",
                opts.max_restarts
            );
            print_repeated_crash_guidance(&opts, "too many restarts");
            return Err(format!("cargo exited with status: {status}"));
        }

        if crash_times.len() >= 3 {
            print_repeated_crash_guidance(&opts, "repeated fast exits detected");
        } else {
            eprintln!(
                "warning: process exited with status: {status} (restart {restarts}/{})",
                opts.max_restarts
            );
        }

        std::thread::sleep(Duration::from_millis(200));
    }
}

fn print_repeated_crash_guidance(opts: &RestartSupervisorOptions, reason: &str) {
    eprintln!("warning: {reason} (bin={})", opts.bin);
    eprintln!("  status: fretboard hotpatch status --tail 80");
    eprintln!("  logs:");
    eprintln!("    runner: .fret/hotpatch_runner.log");
    eprintln!("    view:   .fret/hotpatch_bootstrap.log");

    if opts.dx_available && opts.demo_hotpatch_ready {
        eprintln!("  try: fretboard dev native --bin {} --hotpatch", opts.bin);
    }

    if opts.hotpatch_enabled {
        eprintln!(
            "  try: fretboard dev native --bin {} --hotpatch-reload (disable Subsecond; reload boundary only)",
            opts.bin
        );
    }

    if opts.hotpatch_enabled && !cfg!(windows) {
        eprintln!(
            "  try: set FRET_HOTPATCH_VIEW_CALL_STRATEGY=direct (disables view-level hotpatching)"
        );
    }

    eprintln!(
        "  fallback: do a full rebuild/restart (hotpatch is best-effort; structural changes require rebuild)"
    );
}

fn dx_available() -> bool {
    let out = Command::new("dx")
        .arg("--version")
        .stdout(std::process::Stdio::null())
        .stderr(std::process::Stdio::null())
        .status();
    out.is_ok_and(|s| s.success())
}

fn is_hotpatch_ready_native_demo(name: &str) -> bool {
    matches!(name, "todo_demo" | "assets_demo" | "hotpatch_smoke_demo")
}

pub(crate) fn dev_web(args: Vec<String>) -> Result<(), String> {
    let mut port: Option<u16> = None;
    let mut demo: Option<String> = None;
    let mut choose = false;
    // Dev web is primarily an interactive workflow; default to opening the browser
    // once the server is reachable. Use `--no-open` for CI or when you explicitly
    // do not want the auto-open behavior.
    let mut open = true;
    let mut devtools_ws_url: Option<String> = None;
    let mut devtools_token: Option<String> = None;

    let mut it = args.into_iter();
    while let Some(a) = it.next() {
        match a.as_str() {
            "--port" => {
                let raw = it
                    .next()
                    .ok_or_else(|| "--port requires a value".to_string())?;
                port = Some(raw.parse::<u16>().map_err(|e| e.to_string())?);
            }
            "--demo" => {
                demo = Some(
                    it.next()
                        .ok_or_else(|| "--demo requires a value".to_string())?,
                );
            }
            "--choose" => choose = true,
            "--open" => open = true,
            "--no-open" => open = false,
            "--devtools-ws-url" => {
                devtools_ws_url = Some(
                    it.next()
                        .ok_or_else(|| "--devtools-ws-url requires a value".to_string())?,
                );
            }
            "--devtools-token" => {
                devtools_token = Some(
                    it.next()
                        .ok_or_else(|| "--devtools-token requires a value".to_string())?,
                );
            }
            "--help" | "-h" => return help(),
            other => return Err(format!("unknown argument for dev web: {other}")),
        }
    }

    let root = workspace_root()?;
    let web_dir = root.join("apps").join("fret-demo-web");

    let effective_port = port.unwrap_or(8080);
    let mut url = format!("http://127.0.0.1:{effective_port}");
    let demo = match (demo.as_deref(), choose) {
        (Some(name), _) => {
            validate_web_demo(name)?;
            Some(name.to_string())
        }
        (None, true) => {
            let demos = web_demos_as_vec();
            let default = demos.first().map(|d| d.as_str());
            Some(prompt_choose_demo(
                "Select a web demo",
                &demos,
                default,
                validate_web_demo,
            )?)
        }
        (None, false) => None,
    };
    if let Some(demo) = demo.as_deref() {
        url.push_str(&format!("/?demo={demo}"));
    }

    if let Some(ws_url) = devtools_ws_url.as_deref() {
        if ws_url.trim().is_empty() {
            return Err("--devtools-ws-url must not be empty".to_string());
        }
        let sep = if url.contains('?') { '&' } else { '?' };
        url.push(sep);
        url.push_str("fret_devtools_ws=");
        url.push_str(ws_url.trim());
    }

    if let Some(token) = devtools_token.as_deref() {
        if token.trim().is_empty() {
            return Err("--devtools-token must not be empty".to_string());
        }
        let sep = if url.contains('?') { '&' } else { '?' };
        url.push(sep);
        url.push_str("fret_devtools_token=");
        url.push_str(token.trim());
    }

    eprintln!("Starting Trunk dev server in `{}`", display_path(&web_dir));

    let mut cmd = Command::new("trunk");
    cmd.current_dir(&web_dir).args(["serve"]);
    if let Some(port) = port {
        cmd.args(["--port", &port.to_string()]);
    }

    let mut child = cmd.spawn().map_err(|e| {
        if e.kind() == std::io::ErrorKind::NotFound {
            "failed to run `trunk` (not found). Install it with: `cargo install trunk`".to_string()
        } else {
            e.to_string()
        }
    })?;

    std::thread::spawn({
        let url = url.clone();
        let web_dir = web_dir.clone();
        move || {
            use std::net::{SocketAddr, TcpStream, ToSocketAddrs as _};
            use std::time::{Duration, Instant};

            let start = Instant::now();
            let deadline = Duration::from_secs(90);

            let Ok(mut addrs) = format!("127.0.0.1:{effective_port}").to_socket_addrs() else {
                return;
            };
            let Some(addr) = addrs.find(SocketAddr::is_ipv4) else {
                return;
            };

            while start.elapsed() < deadline {
                if TcpStream::connect_timeout(&addr, Duration::from_millis(150)).is_ok() {
                    let remaining = deadline.saturating_sub(start.elapsed());
                    let assets_ready = wait_for_trunk_web_assets_ready(&web_dir, remaining);
                    if assets_ready {
                        eprintln!("\nFret web demo ready: {url}\n");
                    } else {
                        eprintln!(
                            "\nFret web dev server is reachable but assets may still be building: {url}\n"
                        );
                    }
                    if open {
                        if let Err(err) = open_url(&url) {
                            eprintln!("warning: failed to open browser: {err}");
                        }
                    }
                    return;
                }
                std::thread::sleep(Duration::from_millis(200));
            }

            eprintln!("\nFret web demo (may still be building): {url}\n");
            if open {
                if let Err(err) = open_url(&url) {
                    eprintln!("warning: failed to open browser: {err}");
                }
            }
        }
    });

    let status = child.wait().map_err(|e| e.to_string())?;
    if !status.success() {
        return Err(format!("trunk exited with status: {status}"));
    }
    Ok(())
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
struct TrunkWebAssetSignature {
    index_len: u64,
    index_mtime_ms: u128,
    js_len: u64,
    js_mtime_ms: u128,
    wasm_len: u64,
    wasm_mtime_ms: u128,
}

fn wait_for_trunk_web_assets_ready(web_dir: &Path, timeout: Duration) -> bool {
    let start = Instant::now();
    let mut last_sig: Option<TrunkWebAssetSignature> = None;
    let mut stable_since: Option<Instant> = None;

    while start.elapsed() < timeout {
        if let Some(sig) = trunk_web_asset_signature(web_dir) {
            match last_sig {
                Some(prev) if prev == sig => {
                    let stable = stable_since.get_or_insert_with(Instant::now);
                    if stable.elapsed() >= Duration::from_millis(1200) {
                        return true;
                    }
                }
                _ => {
                    last_sig = Some(sig);
                    stable_since = Some(Instant::now());
                }
            }
        } else {
            last_sig = None;
            stable_since = None;
        }
        std::thread::sleep(Duration::from_millis(200));
    }

    false
}

fn trunk_web_asset_signature(web_dir: &Path) -> Option<TrunkWebAssetSignature> {
    let dist_dir = web_dir.join("dist");
    let index = dist_dir.join("index.html");
    let js = latest_dist_asset(&dist_dir, |name| {
        name.starts_with("fret-demo-web-") && name.ends_with(".js")
    })?;
    let wasm = latest_dist_asset(&dist_dir, |name| {
        name.starts_with("fret-demo-web-") && name.ends_with("_bg.wasm")
    })?;

    let (index_len, index_mtime_ms) = file_len_and_mtime_ms(&index)?;
    let (js_len, js_mtime_ms) = file_len_and_mtime_ms(&js)?;
    let (wasm_len, wasm_mtime_ms) = file_len_and_mtime_ms(&wasm)?;

    if index_len == 0 || js_len == 0 || wasm_len == 0 {
        return None;
    }

    Some(TrunkWebAssetSignature {
        index_len,
        index_mtime_ms,
        js_len,
        js_mtime_ms,
        wasm_len,
        wasm_mtime_ms,
    })
}

fn latest_dist_asset(
    dist_dir: &Path,
    mut predicate: impl FnMut(&str) -> bool,
) -> Option<std::path::PathBuf> {
    let mut newest: Option<(SystemTime, std::path::PathBuf)> = None;
    for entry in std::fs::read_dir(dist_dir).ok()? {
        let entry = entry.ok()?;
        let path = entry.path();
        if !entry.file_type().ok()?.is_file() {
            continue;
        }
        let name = path.file_name()?.to_str()?;
        if !predicate(name) {
            continue;
        }
        let modified = entry
            .metadata()
            .ok()?
            .modified()
            .ok()
            .unwrap_or(SystemTime::UNIX_EPOCH);
        match &newest {
            Some((best_modified, _)) if modified <= *best_modified => {}
            _ => newest = Some((modified, path)),
        }
    }
    newest.map(|(_, path)| path)
}

fn file_len_and_mtime_ms(path: &Path) -> Option<(u64, u128)> {
    let meta = std::fs::metadata(path).ok()?;
    let modified_ms = meta
        .modified()
        .ok()?
        .duration_since(SystemTime::UNIX_EPOCH)
        .ok()?
        .as_millis();
    Some((meta.len(), modified_ms))
}

fn open_url(url: &str) -> Result<(), String> {
    #[cfg(windows)]
    {
        let status = Command::new("rundll32.exe")
            .args(["url.dll,FileProtocolHandler", url])
            .status()
            .map_err(|e| e.to_string())?;
        if !status.success() {
            return Err(format!(
                "rundll32 FileProtocolHandler exited with status: {status}"
            ));
        }
        return Ok(());
    }

    #[cfg(target_os = "macos")]
    {
        let status = Command::new("open")
            .arg(url)
            .status()
            .map_err(|e| e.to_string())?;
        if !status.success() {
            return Err(format!("open exited with status: {status}"));
        }
        return Ok(());
    }

    #[cfg(all(not(windows), not(target_os = "macos")))]
    {
        let status = Command::new("xdg-open")
            .arg(url)
            .status()
            .map_err(|e| e.to_string())?;
        if !status.success() {
            return Err(format!("xdg-open exited with status: {status}"));
        }
        return Ok(());
    }
}

fn dev_native_hotpatch_dx(
    workspace_root: &Path,
    bin: &str,
    hotpatch_dx_ws: Option<&str>,
    hotpatch_build_id: Option<HotpatchBuildIdArg>,
    passthrough: Vec<String>,
) -> Result<(), String> {
    let mut cmd = Command::new("dx");
    cmd.current_dir(workspace_root)
        .args(["serve", "--hotpatch", "--open", "false"]);

    if let Some(ws) = hotpatch_dx_ws {
        let (addr, port) = parse_ws_endpoint_addr(ws)?;
        cmd.args(["--addr", &addr, "--port", &port.to_string()]);
        cmd.env("FRET_HOTPATCH_DEVSERVER_WS", ws);
    }

    #[cfg(windows)]
    {
        eprintln!(
            "Hotpatch(dx): windows note: default view_call strategy is `direct` (safe; view hotpatch disabled); see docs/adr/0105-dev-hotpatch-subsecond-and-hot-reload-safety.md"
        );
        eprintln!(
            "  to force view-level hotpatching: set FRET_HOTPATCH_VIEW_CALL_STRATEGY=hotfn (may crash)"
        );
    }

    let resolved_build_id = match hotpatch_build_id.unwrap_or(HotpatchBuildIdArg::Auto) {
        HotpatchBuildIdArg::None => None,
        // In `dx serve` mode the devserver assigns its own build id (typically `0`).
        // Forcing a random build id breaks client matching ("no ASLR reference").
        HotpatchBuildIdArg::Auto => None,
        HotpatchBuildIdArg::Value(v) => Some(v),
    };

    cmd.env("FRET_HOTPATCH", "1");
    if let Some(build_id) = resolved_build_id {
        cmd.env("FRET_HOTPATCH_BUILD_ID", build_id.to_string());
    }

    let mut cargo_features: Vec<&str> = vec!["hotpatch"];
    if matches!(bin, "node_graph_demo") {
        cargo_features.push("node-graph-demos");
    } else if matches!(
        bin,
        "node_graph_domain_demo" | "node_graph_legacy_demo" | "imui_node_graph_demo"
    ) {
        cargo_features.push("node-graph-demos-legacy");
    }
    let cargo_features = cargo_features.join(",");

    cmd.args([
        "--package",
        "fret-demo",
        "--features",
        &cargo_features,
        "--bin",
        bin,
    ]);

    if !passthrough.is_empty() {
        cmd.args(["--args", &passthrough.join(" ")]);
    }

    let status = cmd.status().map_err(|e| {
        if e.kind() == std::io::ErrorKind::NotFound {
            "failed to run `dx` (not found). Install it with: `cargo install dioxus-cli`"
                .to_string()
        } else {
            e.to_string()
        }
    })?;
    if !status.success() {
        return Err(format!("dx exited with status: {status}"));
    }
    Ok(())
}

fn parse_ws_endpoint_addr(ws: &str) -> Result<(String, u16), String> {
    let ws = ws.trim();
    let without_scheme = ws
        .strip_prefix("ws://")
        .or_else(|| ws.strip_prefix("wss://"))
        .ok_or_else(|| format!("invalid ws endpoint `{ws}` (expected ws://... or wss://...)"))?;

    let host_port = without_scheme.split('/').next().unwrap_or_default().trim();
    if host_port.is_empty() {
        return Err(format!(
            "invalid ws endpoint `{ws}` (expected ws://<host>:<port>/...)"
        ));
    }

    let (host, port_raw) = if let Some(rest) = host_port.strip_prefix('[') {
        let end = rest
            .find(']')
            .ok_or_else(|| format!("invalid ws endpoint `{ws}` (malformed IPv6 host)"))?;
        let host = &rest[..end];
        let port = rest[end + 1..]
            .strip_prefix(':')
            .ok_or_else(|| format!("invalid ws endpoint `{ws}` (missing port)"))?;
        (host, port)
    } else {
        host_port
            .rsplit_once(':')
            .ok_or_else(|| format!("invalid ws endpoint `{ws}` (missing port)"))?
    };

    let host = match host {
        "localhost" => "127.0.0.1",
        other => other,
    };
    let port = port_raw
        .parse::<u16>()
        .map_err(|e| format!("invalid ws endpoint `{ws}` (invalid port `{port_raw}`): {e}"))?;

    Ok((host.to_string(), port))
}
