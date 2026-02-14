use std::path::Path;
use std::process::Command;

use crate::cli::{help, workspace_root};
use crate::demos::{
    display_path, list_native_demos_from, prompt_choose_demo, validate_native_demo,
    validate_web_demo, web_demos_as_vec,
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

    let view_call =
        if std::env::var_os("FRET_HOTPATCH_VIEW_CALL_DIRECT").is_some_and(|v| !v.is_empty()) {
            "direct (view hotpatch disabled)"
        } else {
            "hotfn"
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

    let mut bin: Option<String> = None;
    let mut choose = false;
    let mut hotpatch = false;
    let mut hotpatch_reload_only = false;
    let mut hotpatch_trigger_path: Option<String> = None;
    let mut hotpatch_poll_ms: Option<u64> = None;
    let mut hotpatch_devserver_ws: Option<String> = None;
    let mut hotpatch_build_id: Option<HotpatchBuildIdArg> = None;
    let mut hotpatch_dx = false;
    let mut hotpatch_dx_ws: Option<String> = None;
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
            "--choose" => choose = true,
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

    if hotpatch && hotpatch_devserver_ws.is_some() {
        return Err("cannot combine --hotpatch and --hotpatch-devserver".to_string());
    }
    if hotpatch_dx && (hotpatch || hotpatch_devserver_ws.is_some()) {
        return Err(
            "cannot combine --hotpatch-dx with --hotpatch/--hotpatch-devserver".to_string(),
        );
    }

    let bin = match (bin.as_deref(), choose) {
        (Some(name), _) => {
            validate_native_demo(&demos, name)?;
            name.to_string()
        }
        (None, true) => prompt_choose_demo(
            "Select a native demo",
            &demos,
            Some("components_gallery"),
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

    let mut cmd = Command::new("cargo");
    cmd.current_dir(&root).args(["run", "-p", "fret-demo"]);
    let mut cargo_features: Vec<&str> = Vec::new();
    if hotpatch || hotpatch_devserver_ws.is_some() {
        cargo_features.push("hotpatch");
        cmd.env("FRET_HOTPATCH", "1");
    }
    if matches!(bin.as_str(), "node_graph_demo" | "node_graph_domain_demo") {
        cargo_features.push("node-graph-demos");
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
                "  windows note: view-level hotpatch may crash on the first patched view call; see docs/adr/0105-dev-hotpatch-subsecond-and-hot-reload-safety.md"
            );
            eprintln!(
                "    workaround: set FRET_HOTPATCH_VIEW_CALL_DIRECT=1 (disables view-level hotpatching)"
            );
        }
    }
    cmd.args(["--bin", &bin]);
    if !passthrough.is_empty() {
        cmd.arg("--").args(passthrough);
    }

    let status = cmd.status().map_err(|e| e.to_string())?;
    if !status.success() {
        return Err(format!("cargo exited with status: {status}"));
    }
    Ok(())
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
    matches!(
        name,
        "todo_demo" | "assets_demo" | "hotpatch_smoke_demo" | "hello_counter_demo"
    )
}

pub(crate) fn dev_web(args: Vec<String>) -> Result<(), String> {
    let mut port: Option<u16> = None;
    let mut demo: Option<String> = None;
    let mut choose = false;

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

    eprintln!("Starting Trunk dev server in `{}`", display_path(&web_dir));
    eprintln!("Open: {url}");

    let mut cmd = Command::new("trunk");
    cmd.current_dir(web_dir).args(["serve"]);
    if let Some(port) = port {
        cmd.args(["--port", &port.to_string()]);
    }

    let status = cmd.status().map_err(|e| {
        if e.kind() == std::io::ErrorKind::NotFound {
            "failed to run `trunk` (not found). Install it with: `cargo install trunk`".to_string()
        } else {
            e.to_string()
        }
    })?;
    if !status.success() {
        return Err(format!("trunk exited with status: {status}"));
    }
    Ok(())
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
            "Hotpatch(dx): windows note: view-level hotpatch may crash on the first patched view call; see docs/adr/0105-dev-hotpatch-subsecond-and-hot-reload-safety.md"
        );
        eprintln!(
            "  workaround: set FRET_HOTPATCH_VIEW_CALL_DIRECT=1 (disables view-level hotpatching)"
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
    if matches!(bin, "node_graph_demo" | "node_graph_domain_demo") {
        cargo_features.push("node-graph-demos");
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
