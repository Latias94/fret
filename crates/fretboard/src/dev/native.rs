use std::collections::VecDeque;
use std::path::Path;
use std::process::{Child, Command};

use fret_core::time::{Duration, Instant, SystemTime};

use super::contracts::DevNativeCommandArgs;
use super::project::{NativeTargetKind, SelectedNativeTarget, resolve_native_target};
use super::resolve_bool_override;

pub(crate) fn run_native_contract(args: DevNativeCommandArgs) -> Result<(), String> {
    let selected = resolve_native_target(
        args.manifest_path.as_deref(),
        args.package.as_deref(),
        args.bin.as_deref(),
        args.example.as_deref(),
    )?;
    let strict_runtime =
        resolve_bool_override(args.strict_runtime, args.no_strict_runtime).unwrap_or(true);
    let watch = resolve_bool_override(args.watch, args.no_watch).unwrap_or(false);
    let supervise = resolve_bool_override(args.supervise, args.no_supervise).unwrap_or(watch);
    let watch_poll_ms = Duration::from_millis(args.watch_poll_ms.unwrap_or(800));
    let target_label = format!("{}::{}", selected.package_name, selected.target_name);

    eprintln!(
        "Starting native target `{}` ({})",
        target_label,
        selected.kind.cargo_flag().trim_start_matches('-')
    );

    let mut cmd = Command::new("cargo");
    cmd.current_dir(&selected.invocation_root)
        .arg(if watch { "build" } else { "run" })
        .arg("--manifest-path")
        .arg(&selected.manifest_path);

    if selected.needs_package_flag {
        cmd.args(["-p", &selected.package_name]);
    }

    if let Some(profile) = args.profile.as_deref() {
        cmd.args(["--profile", profile]);
    }

    cmd.args([selected.kind.cargo_flag(), &selected.target_name]);
    configure_strict_runtime_env(&mut cmd, strict_runtime);
    configure_dev_state_env(&mut cmd, watch, args.dev_state_reset);

    if !watch {
        if !args.passthrough.is_empty() {
            cmd.arg("--").args(args.passthrough.iter());
        }

        if supervise {
            run_with_restart_supervisor(
                cmd,
                RestartSupervisorOptions {
                    target_label,
                    max_restarts: 5,
                    crash_window: Duration::from_secs(60),
                    crash_threshold: Duration::from_secs(10),
                },
            )
        } else {
            let status = cmd.status().map_err(|err| err.to_string())?;
            if !status.success() {
                return Err(format!("cargo exited with status: {status}"));
            }
            Ok(())
        }
    } else {
        let build_env = capture_command_env(&cmd);
        dev_native_watch_build_and_run(
            &selected,
            args.profile.as_deref(),
            cmd,
            build_env,
            args.passthrough,
            WorkspaceWatchOptions {
                poll_interval: watch_poll_ms,
                supervise,
            },
            RestartSupervisorOptions {
                target_label,
                max_restarts: 20,
                crash_window: Duration::from_secs(60),
                crash_threshold: Duration::from_secs(10),
            },
        )
    }
}

fn configure_dev_state_env(cmd: &mut Command, watch: bool, dev_state_reset: bool) {
    if watch || dev_state_reset {
        cmd.env("FRET_DEV_STATE", "1");
        cmd.env("FRET_DEV_STATE_DEBOUNCE_MS", "0");
    }
    if dev_state_reset {
        cmd.env("FRET_DEV_STATE_RESET", "1");
    }
}

fn configure_strict_runtime_env(cmd: &mut Command, strict_runtime: bool) {
    cmd.env(
        "FRET_STRICT_RUNTIME",
        if strict_runtime { "1" } else { "0" },
    );
}

#[derive(Debug, Clone, Copy)]
struct WorkspaceWatchOptions {
    poll_interval: Duration,
    supervise: bool,
}

#[derive(Debug, Clone)]
struct CapturedEnv(Vec<(std::ffi::OsString, std::ffi::OsString)>);

fn capture_command_env(cmd: &Command) -> CapturedEnv {
    let pairs = cmd
        .get_envs()
        .filter_map(|(key, value)| Some((key.to_os_string(), value?.to_os_string())));
    CapturedEnv(pairs.collect())
}

fn apply_captured_env(cmd: &mut Command, env: &CapturedEnv) {
    for (key, value) in env.0.iter() {
        cmd.env(key, value);
    }
}

fn dev_native_watch_build_and_run(
    selected: &SelectedNativeTarget,
    profile: Option<&str>,
    mut build_cmd: Command,
    build_env: CapturedEnv,
    passthrough: Vec<String>,
    watch: WorkspaceWatchOptions,
    supervisor_opts: RestartSupervisorOptions,
) -> Result<(), String> {
    eprintln!(
        "Watch: enabled (poll_ms={}, supervise={})",
        watch.poll_interval.as_millis(),
        watch.supervise
    );

    let mut watcher = WorkspaceWatch::new(&selected.workspace_root, watch.poll_interval);
    watcher.baseline()?;

    let restart_trigger_path = selected
        .workspace_root
        .join(".fret")
        .join("watch_restart.touch");
    ensure_restart_trigger_file_initialized(&restart_trigger_path)?;

    loop {
        let status = build_cmd.status().map_err(|err| err.to_string())?;
        if !status.success() {
            eprintln!("error: build failed (status: {status})");
            eprintln!("  waiting for changes...");
            watcher.wait_for_change()?;
            continue;
        }

        let exe = dev_native_exe_path(
            &selected.target_directory,
            selected.kind,
            &selected.target_name,
            profile,
        );
        if !exe.is_file() {
            return Err(format!(
                "expected built executable at `{}` but it was not found",
                exe.display()
            ));
        }

        let mut crash_times: VecDeque<Instant> = VecDeque::new();
        let mut restarts = 0usize;
        let mut child = spawn_native_run_process(
            &exe,
            &selected.invocation_root,
            &build_env,
            &restart_trigger_path,
            &passthrough,
        )?;
        let mut start = Instant::now();

        loop {
            if watcher.poll_changed()? {
                eprintln!("Watch: change detected, restarting...");
                request_graceful_watch_restart(&mut child, &restart_trigger_path)?;
                build_cmd =
                    recreate_build_command(&build_cmd, &selected.invocation_root, &build_env);
                break;
            }

            if let Some(status) = child.try_wait().map_err(|err| err.to_string())? {
                if status.success() || status.code() == Some(130) {
                    return Ok(());
                }

                let elapsed = Instant::now().duration_since(start);
                if !watch.supervise {
                    if elapsed <= supervisor_opts.crash_threshold {
                        print_repeated_crash_guidance(&supervisor_opts, "fast exit detected");
                    } else {
                        eprintln!("warning: process exited with status: {status}");
                    }
                    eprintln!("  waiting for changes (or press Ctrl+C to stop)...");
                    watcher.wait_for_change()?;
                    build_cmd =
                        recreate_build_command(&build_cmd, &selected.invocation_root, &build_env);
                    break;
                }

                record_crash(
                    &mut crash_times,
                    &mut restarts,
                    elapsed,
                    &supervisor_opts,
                    status.to_string(),
                )?;
                std::thread::sleep(Duration::from_millis(200));
                child = spawn_native_run_process(
                    &exe,
                    &selected.invocation_root,
                    &build_env,
                    &restart_trigger_path,
                    &passthrough,
                )?;
                start = Instant::now();
            }

            std::thread::sleep(watch.poll_interval);
        }
    }
}

fn recreate_build_command(
    previous: &Command,
    invocation_root: &Path,
    build_env: &CapturedEnv,
) -> Command {
    let mut next = Command::new("cargo");
    next.current_dir(invocation_root);
    next.args(previous.get_args());
    apply_captured_env(&mut next, build_env);
    next
}

fn spawn_native_run_process(
    exe: &Path,
    invocation_root: &Path,
    build_env: &CapturedEnv,
    restart_trigger_path: &Path,
    passthrough: &[String],
) -> Result<Child, String> {
    let mut cmd = Command::new(exe);
    cmd.current_dir(invocation_root);
    apply_captured_env(&mut cmd, build_env);
    cmd.env("FRET_WATCH_RESTART_TRIGGER_PATH", restart_trigger_path);
    if !passthrough.is_empty() {
        cmd.args(passthrough.iter());
    }
    cmd.spawn().map_err(|err| err.to_string())
}

fn ensure_restart_trigger_file_initialized(path: &Path) -> Result<(), String> {
    if path.is_file() {
        return Ok(());
    }
    poke_restart_trigger_file(path)
}

fn poke_restart_trigger_file(path: &Path) -> Result<(), String> {
    if let Some(parent) = path.parent() {
        std::fs::create_dir_all(parent).map_err(|err| err.to_string())?;
    }
    let marker = SystemTime::now()
        .duration_since(SystemTime::UNIX_EPOCH)
        .map(|duration| format!("restart-{}", duration.as_millis()))
        .unwrap_or_else(|_| "restart".to_string());
    std::fs::write(path, marker).map_err(|err| err.to_string())
}

fn request_graceful_watch_restart(child: &mut Child, trigger_path: &Path) -> Result<(), String> {
    if child.try_wait().map_err(|err| err.to_string())?.is_some() {
        return Ok(());
    }

    let _ = poke_restart_trigger_file(trigger_path);
    let start = Instant::now();
    let timeout = Duration::from_millis(1500);
    loop {
        if child.try_wait().map_err(|err| err.to_string())?.is_some() {
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

fn dev_native_exe_path(
    target_directory: &Path,
    kind: NativeTargetKind,
    target_name: &str,
    profile: Option<&str>,
) -> std::path::PathBuf {
    let mut path = target_directory.join(profile_output_dir(profile));
    if matches!(kind, NativeTargetKind::Example) {
        path = path.join("examples");
    }
    path = path.join(target_name);
    if cfg!(windows) {
        path.set_extension("exe");
    }
    path
}

fn profile_output_dir(profile: Option<&str>) -> String {
    match profile.unwrap_or("dev") {
        "dev" => "debug".to_string(),
        "release" => "release".to_string(),
        other => other.to_string(),
    }
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
        let changed = self.last_sig.is_some_and(|previous| previous != sig);
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
        self.walk_and_hash(&self.root, &mut hasher)?;
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
                Ok(read_dir) => read_dir,
                Err(_) => continue,
            };

            for entry in read_dir.flatten() {
                let path = entry.path();
                let file_name = entry.file_name();
                let file_name = file_name.to_string_lossy();

                if entry.file_type().map(|kind| kind.is_dir()).unwrap_or(false) {
                    if should_skip_dir(&file_name) {
                        continue;
                    }
                    stack.push(path);
                    continue;
                }

                if !should_watch_file(&path) {
                    continue;
                }

                self.hash_file_stamp(&path, hasher)?;
            }
        }
        Ok(())
    }

    fn hash_file_stamp(
        &self,
        path: &Path,
        hasher: &mut std::collections::hash_map::DefaultHasher,
    ) -> Result<(), String> {
        use std::hash::Hash as _;

        let meta = match std::fs::metadata(path) {
            Ok(meta) => meta,
            Err(_) => {
                path.to_string_lossy().hash(hasher);
                return Ok(());
            }
        };

        let modified = meta.modified().ok();
        let nanos = modified
            .and_then(|time| time.duration_since(std::time::UNIX_EPOCH).ok())
            .map(|duration| duration.as_nanos())
            .unwrap_or(0);

        path.to_string_lossy().hash(hasher);
        meta.len().hash(hasher);
        nanos.hash(hasher);
        Ok(())
    }
}

fn should_skip_dir(file_name: &str) -> bool {
    matches!(
        file_name,
        "target" | ".git" | ".fret" | "repo-ref" | "dist" | "node_modules"
    )
}

fn should_watch_file(path: &Path) -> bool {
    let Some(ext) = path.extension().and_then(|value| value.to_str()) else {
        return false;
    };

    matches!(
        ext,
        "rs" | "toml" | "lock" | "wgsl" | "ron" | "json" | "yaml" | "yml" | "html" | "css" | "svg"
    )
}

#[derive(Debug, Clone)]
struct RestartSupervisorOptions {
    target_label: String,
    max_restarts: usize,
    crash_window: Duration,
    crash_threshold: Duration,
}

fn run_with_restart_supervisor(
    mut cmd: Command,
    opts: RestartSupervisorOptions,
) -> Result<(), String> {
    let mut crash_times: VecDeque<Instant> = VecDeque::new();
    let mut restarts = 0usize;

    loop {
        let start = Instant::now();
        let status = cmd.status().map_err(|err| err.to_string())?;
        if status.success() || status.code() == Some(130) {
            return Ok(());
        }

        record_crash(
            &mut crash_times,
            &mut restarts,
            Instant::now().duration_since(start),
            &opts,
            status.to_string(),
        )?;

        std::thread::sleep(Duration::from_millis(200));
    }
}

fn record_crash(
    crash_times: &mut VecDeque<Instant>,
    restarts: &mut usize,
    elapsed: Duration,
    opts: &RestartSupervisorOptions,
    status: String,
) -> Result<(), String> {
    if elapsed <= opts.crash_threshold {
        crash_times.push_back(Instant::now());
    }
    while crash_times
        .front()
        .is_some_and(|time| Instant::now().duration_since(*time) > opts.crash_window)
    {
        crash_times.pop_front();
    }

    *restarts += 1;
    if *restarts > opts.max_restarts {
        eprintln!(
            "error: dev supervisor exceeded max restarts ({}) for `{}`.",
            opts.max_restarts, opts.target_label
        );
        print_repeated_crash_guidance(opts, "too many restarts");
        return Err(format!("process exited with status: {status}"));
    }

    if crash_times.len() >= 3 {
        print_repeated_crash_guidance(opts, "repeated fast exits detected");
    } else {
        eprintln!(
            "warning: process exited with status: {status} (restart {}/{})",
            restarts, opts.max_restarts
        );
    }

    Ok(())
}

fn print_repeated_crash_guidance(opts: &RestartSupervisorOptions, reason: &str) {
    eprintln!("warning: {reason} ({})", opts.target_label);
    eprintln!("  hint: rerun without --supervise if you want the first crash to stop immediately");
    eprintln!("  hint: rerun with --watch if you want rebuilds after source changes");
    eprintln!("  hint: inspect the app stderr/stdout around the first failing launch");
}

#[cfg(test)]
mod tests {
    use std::ffi::OsStr;
    use std::path::Path;
    use std::process::Command;

    use super::{
        NativeTargetKind, configure_strict_runtime_env, dev_native_exe_path, profile_output_dir,
    };

    #[test]
    fn profile_output_dir_maps_dev_to_debug() {
        assert_eq!(profile_output_dir(None), "debug");
        assert_eq!(profile_output_dir(Some("dev")), "debug");
        assert_eq!(profile_output_dir(Some("release")), "release");
        assert_eq!(profile_output_dir(Some("bench-fast")), "bench-fast");
    }

    #[test]
    fn native_exe_path_places_examples_under_examples_dir() {
        let path = dev_native_exe_path(
            Path::new("/tmp/project/target"),
            NativeTargetKind::Example,
            "simple_todo",
            None,
        );
        assert!(path.ends_with("debug/examples/simple_todo"));
    }

    #[test]
    fn configure_strict_runtime_env_sets_explicit_value() {
        let mut cmd = Command::new("cargo");
        configure_strict_runtime_env(&mut cmd, true);
        let strict = cmd
            .get_envs()
            .find(|(key, _)| *key == OsStr::new("FRET_STRICT_RUNTIME"))
            .and_then(|(_, value)| value)
            .expect("strict runtime env should be set");
        assert_eq!(strict, "1");

        let mut cmd = Command::new("cargo");
        configure_strict_runtime_env(&mut cmd, false);
        let strict = cmd
            .get_envs()
            .find(|(key, _)| *key == OsStr::new("FRET_STRICT_RUNTIME"))
            .and_then(|(_, value)| value)
            .expect("strict runtime env should be set");
        assert_eq!(strict, "0");
    }
}
