use std::collections::{BTreeMap, BTreeSet, HashMap};
use std::path::{Path, PathBuf};
#[cfg(any(target_os = "windows", target_os = "macos", target_os = "linux"))]
use std::sync::{
    Arc, Mutex,
    atomic::{AtomicBool, Ordering},
};
use std::time::SystemTime;

use fret_app::App;
use fret_core::AppWindowId;
use fret_core::time::Instant;

use super::event_loop::RunnerUserEvent;
use super::window::TimerEntry;

#[cfg(any(target_os = "windows", target_os = "macos", target_os = "linux"))]
use notify::{RecommendedWatcher, RecursiveMode, Watcher};
#[cfg(any(target_os = "windows", target_os = "macos", target_os = "linux"))]
use winit::event_loop::EventLoopProxy;

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
struct FileStamp {
    len: u64,
    modified: Option<SystemTime>,
}

fn file_stamp(path: &Path) -> Option<FileStamp> {
    let metadata = std::fs::metadata(path).ok()?;
    Some(FileStamp {
        len: metadata.len(),
        modified: metadata.modified().ok(),
    })
}

#[derive(Debug, Clone, PartialEq, Eq)]
enum AssetReloadSnapshot {
    Manifest {
        manifest: Option<FileStamp>,
        entries: BTreeMap<PathBuf, Option<FileStamp>>,
    },
    Dir {
        root: Option<FileStamp>,
        files: BTreeMap<PathBuf, FileStamp>,
    },
}

#[derive(Debug, Clone)]
struct AssetReloadWatchedTarget {
    target: crate::assets::AssetReloadTarget,
    snapshot: AssetReloadSnapshot,
}

impl AssetReloadWatchedTarget {
    fn new(target: crate::assets::AssetReloadTarget) -> Self {
        let snapshot = capture_asset_reload_snapshot(&target);
        Self { target, snapshot }
    }

    fn poll_changed(&mut self) -> bool {
        let next = capture_asset_reload_snapshot(&self.target);
        if next == self.snapshot {
            return false;
        }

        self.snapshot = next;
        true
    }
}

#[derive(Debug)]
struct PollTimerState {
    interval: std::time::Duration,
    token: Option<fret_runtime::TimerToken>,
}

impl PollTimerState {
    fn new(interval: std::time::Duration) -> Self {
        Self {
            interval,
            token: None,
        }
    }

    fn ensure_scheduled(
        &mut self,
        app: &mut App,
        now: Instant,
        timers: &mut HashMap<fret_runtime::TimerToken, TimerEntry>,
    ) {
        if self.token.is_some() {
            return;
        }

        let token = app.next_timer_token();
        timers.insert(
            token,
            TimerEntry {
                window: None,
                deadline: now + self.interval,
                repeat: Some(self.interval),
            },
        );
        self.token = Some(token);
    }

    fn matches(&self, token: fret_runtime::TimerToken) -> bool {
        self.token == Some(token)
    }
}

#[derive(Debug)]
enum AssetReloadMode {
    Poll(PollTimerState),
    NativeWatch(NativeWatchState),
}

#[derive(Debug)]
struct NativeWatchState {
    fallback_poll_interval: std::time::Duration,
    fallback_poll: Option<PollTimerState>,
    #[cfg(any(target_os = "windows", target_os = "macos", target_os = "linux"))]
    pending_signal: Arc<AtomicBool>,
    #[cfg(any(target_os = "windows", target_os = "macos", target_os = "linux"))]
    watcher: Option<NativeWatchHandle>,
}

impl NativeWatchState {
    fn new(fallback_poll_interval: std::time::Duration) -> Self {
        Self {
            fallback_poll_interval,
            fallback_poll: None,
            #[cfg(any(target_os = "windows", target_os = "macos", target_os = "linux"))]
            pending_signal: Arc::new(AtomicBool::new(false)),
            #[cfg(any(target_os = "windows", target_os = "macos", target_os = "linux"))]
            watcher: None,
        }
    }

    fn ensure_fallback_poll(
        &mut self,
        app: &mut App,
        now: Instant,
        timers: &mut HashMap<fret_runtime::TimerToken, TimerEntry>,
    ) {
        let poll = self
            .fallback_poll
            .get_or_insert_with(|| PollTimerState::new(self.fallback_poll_interval));
        poll.ensure_scheduled(app, now, timers);
    }

    fn matches_fallback_poll(&self, token: fret_runtime::TimerToken) -> bool {
        self.fallback_poll
            .as_ref()
            .is_some_and(|poll| poll.matches(token))
    }

    #[cfg(any(target_os = "windows", target_os = "macos", target_os = "linux"))]
    fn set_event_loop_proxy(
        &mut self,
        targets: &[AssetReloadWatchedTarget],
        proxy: EventLoopProxy,
        proxy_events: Arc<Mutex<Vec<RunnerUserEvent>>>,
    ) -> Result<(), notify::Error> {
        if self.watcher.is_some() {
            return Ok(());
        }

        let pending_signal = self.pending_signal.clone();
        let callback_proxy = proxy.clone();
        let callback_events = proxy_events.clone();

        let mut watcher =
            notify::recommended_watcher(move |result: Result<notify::Event, notify::Error>| {
                if let Err(error) = result {
                    tracing::warn!(?error, "asset reload watcher event error");
                }
                if pending_signal.swap(true, Ordering::SeqCst) {
                    return;
                }
                if let Ok(mut queue) = callback_events.lock() {
                    queue.push(RunnerUserEvent::AssetReloadWake);
                }
                callback_proxy.wake_up();
            })?;

        let roots = collect_watch_roots(targets);
        watch_roots(&mut watcher, &roots)?;
        self.watcher = Some(NativeWatchHandle { watcher, roots });
        self.fallback_poll = None;
        Ok(())
    }

    #[cfg(not(any(target_os = "windows", target_os = "macos", target_os = "linux")))]
    fn set_event_loop_proxy(
        &mut self,
        _targets: &[AssetReloadWatchedTarget],
        _proxy: (),
        _proxy_events: (),
    ) -> Result<(), std::convert::Infallible> {
        Ok(())
    }

    #[cfg(any(target_os = "windows", target_os = "macos", target_os = "linux"))]
    fn take_pending_signal(&self) -> bool {
        self.pending_signal.swap(false, Ordering::SeqCst)
    }

    #[cfg(not(any(target_os = "windows", target_os = "macos", target_os = "linux")))]
    fn take_pending_signal(&self) -> bool {
        false
    }

    #[cfg(any(target_os = "windows", target_os = "macos", target_os = "linux"))]
    fn refresh_watch_roots(&mut self, targets: &[AssetReloadWatchedTarget]) {
        let Some(handle) = self.watcher.as_mut() else {
            return;
        };

        let next_roots = collect_watch_roots(targets);
        if next_roots == handle.roots {
            return;
        }

        if let Err(error) = rewatch_roots(&mut handle.watcher, &handle.roots, &next_roots) {
            tracing::warn!(?error, "asset reload watcher root refresh failed");
            return;
        }

        handle.roots = next_roots;
    }

    #[cfg(not(any(target_os = "windows", target_os = "macos", target_os = "linux")))]
    fn refresh_watch_roots(&mut self, _targets: &[AssetReloadWatchedTarget]) {}

    #[cfg(test)]
    fn note_watch_signal_for_tests(&self) {
        #[cfg(any(target_os = "windows", target_os = "macos", target_os = "linux"))]
        self.pending_signal.store(true, Ordering::SeqCst);
    }
}

#[cfg(any(target_os = "windows", target_os = "macos", target_os = "linux"))]
#[derive(Debug)]
struct NativeWatchHandle {
    watcher: RecommendedWatcher,
    roots: BTreeSet<PathBuf>,
}

pub(super) struct AssetReloadController {
    targets: Vec<AssetReloadWatchedTarget>,
    mode: AssetReloadMode,
}

impl AssetReloadController {
    pub(super) fn new(
        policy: Option<crate::assets::AssetReloadPolicy>,
        targets: Vec<crate::assets::AssetReloadTarget>,
    ) -> Option<Self> {
        let policy = policy?;
        if targets.is_empty() {
            return None;
        }

        let targets = targets
            .into_iter()
            .map(AssetReloadWatchedTarget::new)
            .collect();
        let mode = match policy {
            crate::assets::AssetReloadPolicy::PollMetadata { interval } => {
                AssetReloadMode::Poll(PollTimerState::new(interval))
            }
            crate::assets::AssetReloadPolicy::NativeWatcher {
                fallback_poll_interval,
            } => AssetReloadMode::NativeWatch(NativeWatchState::new(fallback_poll_interval)),
        };

        Some(Self { targets, mode })
    }

    pub(super) fn publish_support(&self, app: &mut App) {
        fret_runtime::set_asset_reload_support(
            app,
            fret_runtime::AssetReloadSupport { file_watch: true },
        );
        if matches!(self.mode, AssetReloadMode::Poll(_)) {
            Self::publish_runtime_status(
                app,
                fret_runtime::AssetReloadBackendKind::PollMetadata,
                fret_runtime::AssetReloadBackendKind::PollMetadata,
                None,
                None,
            );
        }
    }

    pub(super) fn arm_for_startup(
        &mut self,
        app: &mut App,
        now: Instant,
        timers: &mut HashMap<fret_runtime::TimerToken, TimerEntry>,
    ) {
        if let AssetReloadMode::Poll(poll) = &mut self.mode {
            poll.ensure_scheduled(app, now, timers);
        }
    }

    #[cfg(any(target_os = "windows", target_os = "macos", target_os = "linux"))]
    pub(super) fn set_event_loop_proxy(
        &mut self,
        app: &mut App,
        now: Instant,
        timers: &mut HashMap<fret_runtime::TimerToken, TimerEntry>,
        proxy: EventLoopProxy,
        proxy_events: Arc<Mutex<Vec<RunnerUserEvent>>>,
    ) {
        match &mut self.mode {
            AssetReloadMode::Poll(poll) => {
                poll.ensure_scheduled(app, now, timers);
                Self::publish_runtime_status(
                    app,
                    fret_runtime::AssetReloadBackendKind::PollMetadata,
                    fret_runtime::AssetReloadBackendKind::PollMetadata,
                    None,
                    None,
                );
            }
            AssetReloadMode::NativeWatch(watch) => {
                if let Err(error) = watch.set_event_loop_proxy(&self.targets, proxy, proxy_events) {
                    let fallback_message = error.to_string();
                    tracing::warn!(
                        ?error,
                        "asset reload watcher unavailable; falling back to metadata polling"
                    );
                    watch.ensure_fallback_poll(app, now, timers);
                    Self::publish_runtime_status(
                        app,
                        fret_runtime::AssetReloadBackendKind::NativeWatcher,
                        fret_runtime::AssetReloadBackendKind::PollMetadata,
                        Some(fret_runtime::AssetReloadFallbackReason::WatcherInstallFailed),
                        Some(fallback_message),
                    );
                } else {
                    Self::publish_runtime_status(
                        app,
                        fret_runtime::AssetReloadBackendKind::NativeWatcher,
                        fret_runtime::AssetReloadBackendKind::NativeWatcher,
                        None,
                        None,
                    );
                }
            }
        }
    }

    pub(super) fn handle_timer(
        &mut self,
        app: &mut App,
        token: fret_runtime::TimerToken,
        windows: &[AppWindowId],
    ) -> bool {
        let matches = match &self.mode {
            AssetReloadMode::Poll(poll) => poll.matches(token),
            AssetReloadMode::NativeWatch(watch) => watch.matches_fallback_poll(token),
        };
        if !matches {
            return false;
        }

        self.apply_reload_if_changed(app, windows)
    }

    pub(super) fn handle_proxy_wake(&mut self, app: &mut App, windows: &[AppWindowId]) -> bool {
        let has_pending_signal = match &self.mode {
            AssetReloadMode::NativeWatch(watch) => watch.take_pending_signal(),
            AssetReloadMode::Poll(_) => false,
        };
        if !has_pending_signal {
            return false;
        }

        self.apply_reload_if_changed(app, windows);
        if let AssetReloadMode::NativeWatch(watch) = &mut self.mode {
            watch.refresh_watch_roots(&self.targets);
        }
        true
    }

    fn apply_reload_if_changed(&mut self, app: &mut App, windows: &[AppWindowId]) -> bool {
        if !self
            .targets
            .iter_mut()
            .any(AssetReloadWatchedTarget::poll_changed)
        {
            return false;
        }

        fret_runtime::bump_asset_reload_epoch(app);
        for &window in windows {
            app.request_redraw(window);
        }
        true
    }

    fn publish_runtime_status(
        app: &mut App,
        configured_backend: fret_runtime::AssetReloadBackendKind,
        active_backend: fret_runtime::AssetReloadBackendKind,
        fallback_reason: Option<fret_runtime::AssetReloadFallbackReason>,
        fallback_message: Option<String>,
    ) {
        fret_runtime::set_asset_reload_status(
            app,
            fret_runtime::AssetReloadStatus {
                configured_backend,
                active_backend,
                fallback_reason,
                fallback_message,
            },
        );
    }

    #[cfg(test)]
    fn poll_timer_token(&self) -> Option<fret_runtime::TimerToken> {
        match &self.mode {
            AssetReloadMode::Poll(poll) => poll.token,
            AssetReloadMode::NativeWatch(watch) => {
                watch.fallback_poll.as_ref().and_then(|p| p.token)
            }
        }
    }

    #[cfg(test)]
    fn note_watch_signal_for_tests(&self) {
        if let AssetReloadMode::NativeWatch(watch) = &self.mode {
            watch.note_watch_signal_for_tests();
        }
    }
}

fn capture_asset_reload_snapshot(target: &crate::assets::AssetReloadTarget) -> AssetReloadSnapshot {
    match target {
        crate::assets::AssetReloadTarget::Manifest { path } => {
            capture_manifest_reload_snapshot(path.as_path())
        }
        crate::assets::AssetReloadTarget::Dir { path } => {
            capture_dir_reload_snapshot(path.as_path())
        }
    }
}

fn capture_manifest_reload_snapshot(path: &Path) -> AssetReloadSnapshot {
    let manifest = file_stamp(path);
    let mut entries = BTreeMap::new();

    if let Ok(manifest_file) = crate::assets::FileAssetManifestV1::load_json_path(path) {
        let base_dir = path.parent().unwrap_or_else(|| Path::new("."));
        for bundle in manifest_file.bundles {
            let bundle_root = bundle.root.unwrap_or_default();
            for entry in bundle.entries {
                let entry_path = entry
                    .path
                    .unwrap_or_else(|| PathBuf::from(entry.key.as_str()));
                let resolved = resolve_manifest_entry_path(base_dir, &bundle_root, &entry_path);
                entries.insert(resolved.clone(), file_stamp(&resolved));
            }
        }
    }

    AssetReloadSnapshot::Manifest { manifest, entries }
}

fn capture_dir_reload_snapshot(path: &Path) -> AssetReloadSnapshot {
    let root = file_stamp(path);
    let mut files = Vec::new();
    let _ = collect_reload_dir_files(path, &mut files);

    let files = files
        .into_iter()
        .filter_map(|path| file_stamp(&path).map(|stamp| (path, stamp)))
        .collect();

    AssetReloadSnapshot::Dir { root, files }
}

fn resolve_manifest_entry_path(base_dir: &Path, bundle_root: &Path, entry_path: &Path) -> PathBuf {
    if entry_path.is_absolute() {
        return entry_path.to_path_buf();
    }

    let joined_root = if bundle_root.is_absolute() {
        bundle_root.to_path_buf()
    } else {
        base_dir.join(bundle_root)
    };
    joined_root.join(entry_path)
}

fn collect_reload_dir_files(dir: &Path, out: &mut Vec<PathBuf>) -> std::io::Result<()> {
    let mut entries = std::fs::read_dir(dir)?;
    let mut paths = Vec::new();
    while let Some(entry) = entries.next() {
        paths.push(entry?.path());
    }
    paths.sort();

    for path in paths {
        let metadata = std::fs::metadata(&path)?;
        if metadata.is_dir() {
            collect_reload_dir_files(&path, out)?;
        } else if metadata.is_file() {
            out.push(path);
        }
    }

    Ok(())
}

#[cfg(any(target_os = "windows", target_os = "macos", target_os = "linux"))]
fn collect_watch_roots(targets: &[AssetReloadWatchedTarget]) -> BTreeSet<PathBuf> {
    let mut roots = BTreeSet::new();
    for target in targets {
        match &target.target {
            crate::assets::AssetReloadTarget::Dir { path } => {
                roots.insert(existing_watch_root(path));
            }
            crate::assets::AssetReloadTarget::Manifest { path } => {
                roots.insert(existing_watch_root(path));
                if let Ok(manifest_file) = crate::assets::FileAssetManifestV1::load_json_path(path)
                {
                    let base_dir = path.parent().unwrap_or_else(|| Path::new("."));
                    for bundle in manifest_file.bundles {
                        let bundle_root = bundle.root.unwrap_or_default();
                        if !bundle_root.as_os_str().is_empty() {
                            let resolved_root = if bundle_root.is_absolute() {
                                bundle_root.clone()
                            } else {
                                base_dir.join(&bundle_root)
                            };
                            roots.insert(existing_watch_root(&resolved_root));
                        }

                        for entry in bundle.entries {
                            let entry_path = entry
                                .path
                                .unwrap_or_else(|| PathBuf::from(entry.key.as_str()));
                            let resolved =
                                resolve_manifest_entry_path(base_dir, &bundle_root, &entry_path);
                            roots.insert(existing_watch_root(&resolved));
                        }
                    }
                }
            }
        }
    }
    roots
}

#[cfg(any(target_os = "windows", target_os = "macos", target_os = "linux"))]
fn existing_watch_root(path: &Path) -> PathBuf {
    let mut candidate = if path.is_dir() {
        path.to_path_buf()
    } else {
        path.parent().unwrap_or(path).to_path_buf()
    };

    loop {
        if candidate.is_dir() {
            return candidate;
        }
        if !candidate.pop() {
            return std::env::current_dir().unwrap_or_else(|_| PathBuf::from("."));
        }
    }
}

#[cfg(any(target_os = "windows", target_os = "macos", target_os = "linux"))]
fn watch_roots(
    watcher: &mut RecommendedWatcher,
    roots: &BTreeSet<PathBuf>,
) -> Result<(), notify::Error> {
    for root in roots {
        watcher.watch(root, RecursiveMode::Recursive)?;
    }
    Ok(())
}

#[cfg(any(target_os = "windows", target_os = "macos", target_os = "linux"))]
fn rewatch_roots(
    watcher: &mut RecommendedWatcher,
    current: &BTreeSet<PathBuf>,
    next: &BTreeSet<PathBuf>,
) -> Result<(), notify::Error> {
    for root in current {
        if !next.contains(root) {
            let _ = watcher.unwatch(root);
        }
    }
    for root in next {
        if !current.contains(root) {
            watcher.watch(root, RecursiveMode::Recursive)?;
        }
    }
    Ok(())
}

#[cfg(test)]
mod tests {
    use std::sync::atomic::{AtomicU64, Ordering};

    use fret_app::Effect;
    use slotmap::KeyData;

    use super::*;

    static NEXT_TEMP_ID: AtomicU64 = AtomicU64::new(0);

    fn make_temp_dir(tag: &str) -> PathBuf {
        let id = NEXT_TEMP_ID.fetch_add(1, Ordering::Relaxed);
        let dir = std::env::temp_dir().join(format!(
            "fret-asset-reload-{tag}-{}-{id}",
            std::process::id()
        ));
        let _ = std::fs::remove_dir_all(&dir);
        std::fs::create_dir_all(&dir).expect("create temp dir");
        dir
    }

    fn redraw_windows(app: &mut App) -> Vec<AppWindowId> {
        app.flush_effects()
            .into_iter()
            .filter_map(|effect| match effect {
                Effect::Redraw(window) => Some(window),
                _ => None,
            })
            .collect()
    }

    #[test]
    fn polling_reload_controller_bumps_epoch_and_redraws_all_windows() {
        let asset_dir = make_temp_dir("poll").join("assets");
        std::fs::create_dir_all(asset_dir.join("images")).expect("create images dir");
        std::fs::write(asset_dir.join("images/logo.png"), b"first").expect("write initial asset");

        let mut app = App::new();
        let mut timers = HashMap::new();
        let mut controller = AssetReloadController::new(
            Some(crate::assets::AssetReloadPolicy::poll_metadata(
                std::time::Duration::from_millis(16),
            )),
            vec![crate::assets::AssetReloadTarget::Dir {
                path: asset_dir.clone(),
            }],
        )
        .expect("polling policy should install controller");

        controller.publish_support(&mut app);
        assert_eq!(
            fret_runtime::asset_reload_status(&app),
            Some(fret_runtime::AssetReloadStatus {
                configured_backend: fret_runtime::AssetReloadBackendKind::PollMetadata,
                active_backend: fret_runtime::AssetReloadBackendKind::PollMetadata,
                fallback_reason: None,
                fallback_message: None,
            })
        );
        controller.arm_for_startup(&mut app, Instant::now(), &mut timers);
        let token = controller.poll_timer_token().expect("poll timer installed");

        let window_a = AppWindowId::default();
        let window_b = AppWindowId::from(KeyData::from_ffi(1));

        assert!(!controller.handle_timer(&mut app, token, &[window_a, window_b]));
        assert_eq!(fret_runtime::asset_reload_epoch(&app), None);
        assert!(redraw_windows(&mut app).is_empty());

        std::fs::write(asset_dir.join("images/logo.png"), b"second-version")
            .expect("rewrite asset with different length");

        assert!(controller.handle_timer(&mut app, token, &[window_a, window_b]));
        assert_eq!(
            fret_runtime::asset_reload_epoch(&app),
            Some(fret_runtime::AssetReloadEpoch(1))
        );

        let windows = redraw_windows(&mut app);
        assert_eq!(windows.len(), 2);
        assert!(windows.contains(&window_a));
        assert!(windows.contains(&window_b));
    }

    #[test]
    fn native_watch_reload_controller_bumps_epoch_and_redraws_all_windows() {
        let asset_dir = make_temp_dir("watch").join("assets");
        std::fs::create_dir_all(asset_dir.join("images")).expect("create images dir");
        std::fs::write(asset_dir.join("images/logo.png"), b"first").expect("write initial asset");

        let mut app = App::new();
        let mut controller = AssetReloadController::new(
            Some(crate::assets::AssetReloadPolicy::native_watcher(
                std::time::Duration::from_millis(250),
            )),
            vec![crate::assets::AssetReloadTarget::Dir {
                path: asset_dir.clone(),
            }],
        )
        .expect("native watcher policy should install controller");

        let window_a = AppWindowId::default();
        let window_b = AppWindowId::from(KeyData::from_ffi(1));

        controller.note_watch_signal_for_tests();
        assert!(controller.handle_proxy_wake(&mut app, &[window_a, window_b]));
        assert_eq!(fret_runtime::asset_reload_epoch(&app), None);
        assert!(redraw_windows(&mut app).is_empty());

        std::fs::write(asset_dir.join("images/logo.png"), b"second-version")
            .expect("rewrite asset with different length");

        controller.note_watch_signal_for_tests();
        assert!(controller.handle_proxy_wake(&mut app, &[window_a, window_b]));
        assert_eq!(
            fret_runtime::asset_reload_epoch(&app),
            Some(fret_runtime::AssetReloadEpoch(1))
        );

        let windows = redraw_windows(&mut app);
        assert_eq!(windows.len(), 2);
        assert!(windows.contains(&window_a));
        assert!(windows.contains(&window_b));
    }

    #[cfg(any(target_os = "windows", target_os = "macos", target_os = "linux"))]
    #[test]
    fn manifest_watch_roots_include_manifest_parent_and_bundle_roots() {
        let root = make_temp_dir("manifest");
        let assets = root.join("assets");
        std::fs::create_dir_all(assets.join("images")).expect("create images dir");
        let manifest = root.join("assets.manifest.json");
        std::fs::write(assets.join("images/logo.png"), b"logo").expect("write asset");
        crate::assets::FileAssetManifestV1::new([crate::assets::FileAssetManifestBundleV1::new(
            "app://demo",
            [crate::assets::FileAssetManifestEntryV1::new(
                "images/logo.png",
            )],
        )
        .with_root("assets")])
        .write_json_path(&manifest)
        .expect("write manifest");

        let targets = vec![AssetReloadWatchedTarget::new(
            crate::assets::AssetReloadTarget::Manifest {
                path: manifest.clone(),
            },
        )];
        let roots = collect_watch_roots(&targets);

        assert!(roots.contains(&root));
        assert!(roots.contains(&assets));
    }
}
