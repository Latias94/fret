use std::{
    path::PathBuf,
    sync::{
        Arc, Mutex,
        atomic::{AtomicBool, Ordering},
    },
    time::{Duration, Instant, SystemTime},
};

use winit::event_loop::EventLoopProxy;

pub(crate) fn hotpatch_diag_enabled() -> bool {
    let explicit = std::env::var_os("FRET_HOTPATCH_DIAG").is_some_and(|v| !v.is_empty());
    if explicit {
        return true;
    }

    // When hotpatching is enabled, default to emitting a minimal file log so we can diagnose
    // early crashes where stdout/stderr is swallowed by a devserver UI (e.g. `dx serve`).
    let fret_hotpatch_enabled = std::env::var_os("FRET_HOTPATCH").is_some_and(|v| !v.is_empty());
    let dioxus_cli_enabled = std::env::var_os("DIOXUS_CLI_ENABLED").is_some_and(|v| !v.is_empty());
    cfg!(debug_assertions) && (fret_hotpatch_enabled || dioxus_cli_enabled)
}

fn hotpatch_diag_paths() -> impl Iterator<Item = std::path::PathBuf> {
    let mut paths = Vec::new();

    paths.push(std::path::Path::new(".fret").join("hotpatch_runner.log"));

    let tmp = std::env::temp_dir();
    if !tmp.as_os_str().is_empty() {
        paths.push(tmp.join("fret").join("hotpatch_runner.log"));
    }

    paths.into_iter()
}

pub(crate) fn hotpatch_diag_log(line: &str) {
    if !hotpatch_diag_enabled() {
        return;
    }

    use std::io::Write as _;
    let ts = SystemTime::now()
        .duration_since(SystemTime::UNIX_EPOCH)
        .map(|d| d.as_millis())
        .unwrap_or_default();
    let thread_id = format!("{:?}", std::thread::current().id());
    let msg = format!("[{ts}] [thread={thread_id}] {line}\n");

    for path in hotpatch_diag_paths() {
        if let Some(dir) = path.parent() {
            let _ = std::fs::create_dir_all(dir);
        }
        if let Ok(mut file) = std::fs::OpenOptions::new()
            .create(true)
            .append(true)
            .open(&path)
        {
            let _ = file.write_all(msg.as_bytes());
            let _ = file.flush();
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    use std::sync::{Mutex, OnceLock};

    static ENV_LOCK: OnceLock<Mutex<()>> = OnceLock::new();

    struct EnvVarGuard {
        key: &'static str,
        prev: Option<std::ffi::OsString>,
    }

    impl EnvVarGuard {
        fn set(key: &'static str, value: &str) -> Self {
            let prev = std::env::var_os(key);
            unsafe { std::env::set_var(key, value) };
            Self { key, prev }
        }
    }

    impl Drop for EnvVarGuard {
        fn drop(&mut self) {
            match self.prev.take() {
                Some(v) => unsafe { std::env::set_var(self.key, v) },
                None => unsafe { std::env::remove_var(self.key) },
            }
        }
    }

    #[test]
    fn file_trigger_fires_on_marker_change() {
        let _guard = ENV_LOCK.get_or_init(|| Mutex::new(())).lock().unwrap();

        let tmp = std::env::temp_dir().join(format!(
            "fret_hotpatch_file_trigger_{}_{}.touch",
            std::process::id(),
            std::time::SystemTime::now()
                .duration_since(std::time::UNIX_EPOCH)
                .unwrap_or_default()
                .as_nanos()
        ));
        let _ = std::fs::write(&tmp, "a");

        let _env_path =
            EnvVarGuard::set("FRET_HOTPATCH_TRIGGER_PATH", tmp.to_string_lossy().as_ref());
        let _env_poll = EnvVarGuard::set("FRET_HOTPATCH_POLL_MS", "0");

        let now = Instant::now();
        let mut trigger = FileTrigger::from_env(now, false).expect("FileTrigger::from_env");

        assert!(trigger.poll(now).is_none(), "no change should not fire");

        let _ = std::fs::write(&tmp, "b");
        let req = trigger
            .poll(Instant::now())
            .expect("marker change should fire");
        assert!(matches!(req.kind, HotpatchRequestKind::TriggerFileChanged));
        assert_eq!(req.trigger_path.as_deref(), Some(tmp.as_path()));

        let _ = std::fs::remove_file(&tmp);
    }
}

#[derive(Debug, Clone, Copy)]
pub(crate) enum HotpatchRequestKind {
    SubsecondPatchApplied,
    TriggerFileChanged,
}

#[derive(Debug, Clone)]
pub(crate) struct HotpatchRequest {
    pub(crate) kind: HotpatchRequestKind,
    pub(crate) trigger_path: Option<PathBuf>,
}

#[derive(Debug)]
pub(crate) struct HotpatchTrigger {
    file: Option<FileTrigger>,
    subsecond: SubsecondTrigger,
}

pub(crate) fn hotpatch_trigger_from_env(now: Instant) -> Option<HotpatchTrigger> {
    let fret_hotpatch_enabled = std::env::var_os("FRET_HOTPATCH").is_some_and(|v| !v.is_empty());
    let dioxus_cli_enabled = std::env::var_os("DIOXUS_CLI_ENABLED").is_some_and(|v| !v.is_empty());

    if !fret_hotpatch_enabled && !dioxus_cli_enabled {
        return None;
    }

    let subsecond = SubsecondTrigger::new();

    let devserver_ws = std::env::var("FRET_HOTPATCH_DEVSERVER_WS")
        .ok()
        .filter(|s| !s.trim().is_empty())
        .or_else(dioxus_devserver_ws_from_env);
    // When running under `dx serve`, prefer the CLI-assigned build id (`DIOXUS_BUILD_ID`), since the
    // devserver uses it to group clients. A mismatched build id can make the devserver treat the
    // client as "not connected" for hotpatching purposes (it won't see an ASLR reference).
    let build_id = if dioxus_cli_enabled {
        std::env::var("DIOXUS_BUILD_ID")
            .ok()
            .and_then(|s| s.parse::<u64>().ok())
    } else {
        std::env::var("FRET_HOTPATCH_BUILD_ID")
            .ok()
            .and_then(|s| s.parse::<u64>().ok())
            .or_else(|| {
                std::env::var("DIOXUS_BUILD_ID")
                    .ok()
                    .and_then(|s| s.parse::<u64>().ok())
            })
    };
    if let Some(endpoint) = devserver_ws.as_deref() {
        hotpatch_diag_log(&format!(
            "hotpatch(subsecond): enabled endpoint={endpoint} build_id={build_id:?}"
        ));
        subsecond.spawn_devserver_listener(endpoint.to_string(), build_id);
    }

    let file = FileTrigger::from_env(now, devserver_ws.is_none() && fret_hotpatch_enabled);

    Some(HotpatchTrigger { file, subsecond })
}

fn dioxus_devserver_ws_from_env() -> Option<String> {
    let ip = std::env::var("DIOXUS_DEVSERVER_IP")
        .ok()
        .filter(|s| !s.trim().is_empty())?;
    let port = std::env::var("DIOXUS_DEVSERVER_PORT")
        .ok()
        .and_then(|s| s.parse::<u16>().ok())?;
    Some(format!("ws://{ip}:{port}/_dioxus"))
}

impl HotpatchTrigger {
    pub(crate) fn set_event_loop_proxy(&self, proxy: EventLoopProxy) {
        self.subsecond.set_event_loop_proxy(proxy);
    }

    pub(crate) fn next_poll_at(&self) -> Option<Instant> {
        self.file.as_ref().map(|t| t.next_poll_at)
    }

    pub(crate) fn poll(&mut self, now: Instant) -> Option<HotpatchRequest> {
        if self.subsecond.take_pending() {
            return Some(HotpatchRequest {
                kind: HotpatchRequestKind::SubsecondPatchApplied,
                trigger_path: None,
            });
        }

        let Some(file) = self.file.as_mut() else {
            return None;
        };
        file.poll(now)
    }
}

#[derive(Debug)]
struct FileTrigger {
    path: PathBuf,
    poll_interval: Duration,
    next_poll_at: Instant,
    last_marker: Option<String>,
}

impl FileTrigger {
    fn from_env(now: Instant, enable_default: bool) -> Option<Self> {
        let path = std::env::var_os("FRET_HOTPATCH_TRIGGER_PATH")
            .map(PathBuf::from)
            .or_else(|| enable_default.then(|| PathBuf::from(".fret/hotpatch.touch")))?;

        let poll_interval = std::env::var("FRET_HOTPATCH_POLL_MS")
            .ok()
            .and_then(|s| s.parse::<u64>().ok())
            .map(Duration::from_millis)
            .unwrap_or_else(|| Duration::from_millis(200));

        let last_marker = std::fs::read_to_string(&path)
            .ok()
            .map(|s| s.trim().to_string())
            .filter(|s| !s.is_empty())
            .or_else(|| {
                let mtime: SystemTime = std::fs::metadata(&path).ok()?.modified().ok()?;
                Some(format!("{:?}", mtime))
            });

        Some(Self {
            path,
            poll_interval,
            next_poll_at: now + poll_interval,
            last_marker,
        })
    }

    fn poll(&mut self, now: Instant) -> Option<HotpatchRequest> {
        if now < self.next_poll_at {
            return None;
        }
        self.next_poll_at = now + self.poll_interval;

        let marker = std::fs::read_to_string(&self.path)
            .ok()
            .map(|s| s.trim().to_string())
            .filter(|s| !s.is_empty())
            .or_else(|| {
                let mtime = std::fs::metadata(&self.path)
                    .ok()
                    .and_then(|m| m.modified().ok())?;
                Some(format!("{:?}", mtime))
            });

        let Some(marker) = marker else {
            return None;
        };

        if self
            .last_marker
            .as_ref()
            .is_some_and(|prev| prev == &marker)
        {
            return None;
        }
        self.last_marker = Some(marker);

        Some(HotpatchRequest {
            kind: HotpatchRequestKind::TriggerFileChanged,
            trigger_path: Some(self.path.clone()),
        })
    }
}

#[derive(Debug, Clone)]
struct SubsecondTrigger {
    pending: Arc<AtomicBool>,
    proxy: Arc<Mutex<Option<EventLoopProxy>>>,
}

impl SubsecondTrigger {
    fn new() -> Self {
        let trigger = Self {
            pending: Arc::new(AtomicBool::new(false)),
            proxy: Arc::new(Mutex::new(None)),
        };

        // Register a handler that runs immediately after a patch is committed (JumpTable swapped).
        // This handler must remain very lightweight and thread-safe.
        let pending = Arc::clone(&trigger.pending);
        let proxy = Arc::clone(&trigger.proxy);
        subsecond::register_handler(Arc::new(move || {
            hotpatch_diag_log("hotpatch(subsecond): register_handler fired (patch committed)");
            pending.store(true, Ordering::Release);
            if let Ok(guard) = proxy.lock() {
                if let Some(proxy) = guard.as_ref() {
                    proxy.wake_up();
                }
            }
        }));

        trigger
    }

    fn set_event_loop_proxy(&self, proxy: EventLoopProxy) {
        if let Ok(mut guard) = self.proxy.lock() {
            *guard = Some(proxy);
        }
    }

    fn take_pending(&self) -> bool {
        let was_pending = self.pending.swap(false, Ordering::AcqRel);
        if was_pending {
            hotpatch_diag_log("hotpatch(subsecond): pending=true (runner should hot reload)");
        }
        was_pending
    }

    fn spawn_devserver_listener(&self, endpoint: String, build_id: Option<u64>) {
        if !cfg!(debug_assertions) {
            return;
        }

        let pid = std::process::id();
        let aslr_reference = subsecond::aslr_reference();
        if aslr_reference == 0 {
            tracing::warn!(
                %endpoint,
                pid,
                build_id,
                "hotpatch(subsecond): aslr_reference=0 (cannot hotpatch; ensure `main` is exported)"
            );
            hotpatch_diag_log(&format!(
                "hotpatch(subsecond): aslr_reference=0 endpoint={endpoint} pid={pid} build_id={build_id:?} (cannot hotpatch; ensure `main` is exported)"
            ));
            return;
        }
        let proxy = Arc::clone(&self.proxy);

        std::thread::spawn(move || {
            let mut backoff = Duration::from_millis(200);

            loop {
                let sep = if endpoint.contains('?') { '&' } else { '?' };
                let mut uri = format!("{endpoint}{sep}aslr_reference={aslr_reference}&pid={pid}");
                if let Some(build_id) = build_id {
                    uri.push_str(&format!("&build_id={build_id}"));
                }

                tracing::info!(%endpoint, %uri, aslr_reference, pid, build_id, "hotpatch(subsecond): connecting to devserver");
                hotpatch_diag_log(&format!(
                    "hotpatch(subsecond): connecting uri={uri} aslr_reference={aslr_reference} pid={pid} build_id={build_id:?}"
                ));

                let (mut websocket, _req) = match tungstenite::connect(uri) {
                    Ok((websocket, req)) => (websocket, req),
                    Err(err) => {
                        tracing::debug!(%err, "hotpatch(subsecond): devserver connect failed");
                        hotpatch_diag_log(&format!(
                            "hotpatch(subsecond): connect failed err={err} backoff_ms={}",
                            backoff.as_millis()
                        ));
                        std::thread::sleep(backoff);
                        backoff = (backoff * 2).min(Duration::from_secs(5));
                        continue;
                    }
                };

                backoff = Duration::from_millis(200);
                tracing::debug!("hotpatch(subsecond): connected to devserver");
                hotpatch_diag_log("hotpatch(subsecond): connected");

                while let Ok(msg) = websocket.read() {
                    let tungstenite::Message::Text(text) = msg else {
                        continue;
                    };

                    let Ok(value) = serde_json::from_str::<serde_json::Value>(&text) else {
                        continue;
                    };

                    let Some(hotreload) = value.get("HotReload") else {
                        continue;
                    };

                    let for_pid = hotreload
                        .get("for_pid")
                        .and_then(|v| v.as_u64())
                        .and_then(|v| u32::try_from(v).ok());
                    if for_pid != Some(pid) {
                        continue;
                    }

                    if let Some(build_id) = build_id {
                        let for_build_id = hotreload.get("for_build_id").and_then(|v| v.as_u64());
                        if for_build_id != Some(build_id) {
                            continue;
                        }
                    }

                    let Some(jump_table_value) = hotreload.get("jump_table") else {
                        continue;
                    };
                    if jump_table_value.is_null() {
                        continue;
                    }

                    let Ok(jump_table) =
                        serde_json::from_value::<subsecond::JumpTable>(jump_table_value.clone())
                    else {
                        continue;
                    };

                    hotpatch_diag_log(&format!(
                        "hotpatch(subsecond): received jumptable lib={} map_len={} aslr_reference={} new_base_address={} ifunc_count={}",
                        jump_table.lib.display(),
                        jump_table.map.len(),
                        jump_table.aslr_reference,
                        jump_table.new_base_address,
                        jump_table.ifunc_count
                    ));

                    if let Err(err) = unsafe { subsecond::apply_patch(jump_table) } {
                        tracing::warn!(%err, "hotpatch(subsecond): apply_patch failed");
                        hotpatch_diag_log(&format!(
                            "hotpatch(subsecond): apply_patch failed err={err}"
                        ));
                        continue;
                    }

                    hotpatch_diag_log("hotpatch(subsecond): apply_patch ok");
                    if let Ok(guard) = proxy.lock() {
                        if let Some(proxy) = guard.as_ref() {
                            proxy.wake_up();
                        }
                    }
                }

                tracing::debug!("hotpatch(subsecond): devserver disconnected");
                hotpatch_diag_log("hotpatch(subsecond): devserver disconnected");
                std::thread::sleep(Duration::from_millis(200));
            }
        });
    }
}
