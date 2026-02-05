//! Query-style async resource helpers (ecosystem-level).
//!
//! This crate is inspired by TanStack Query, but adapted to Fret's constraints:
//! - UI/runtime state is main-thread only.
//! - Background work must communicate across a driver boundary (`InboxDrainRegistry`, ADR 0190).
//! - Commands and effects remain data-only.
//!
//! The core contract is:
//! - query state is stored in a `Model<QueryState<T>>` so UI can observe it,
//! - background work produces typed values, marshaled back via an inbox,
//! - completion applies only if the inflight token still matches (stale results are ignored).

use std::any::{Any, TypeId};
use std::collections::HashMap;
use std::hash::{Hash, Hasher};
use std::marker::PhantomData;
use std::panic::{AssertUnwindSafe, Location, catch_unwind, resume_unwind};
use std::sync::atomic::{AtomicBool, AtomicU64, Ordering};
use std::sync::{Arc, Mutex};
use std::time::Duration;

use fret_core::AppWindowId;
use fret_core::time::Instant;
pub use fret_executor::CancellationToken;
use fret_executor::{BackgroundTask, Executors, Inbox, InboxDrainer};
use fret_runtime::{DispatcherHandle, InboxDrainHost, InboxDrainRegistry, Model, ModelId, UiHost};

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum QueryStatus {
    Idle,
    Loading,
    Success,
    Error,
}

#[derive(Debug)]
pub struct QueryState<T> {
    pub status: QueryStatus,
    pub data: Option<Arc<T>>,
    pub error: Option<Arc<str>>,
    pub inflight: Option<u64>,
    pub updated_at: Option<Instant>,
    pub last_duration: Option<Duration>,
}

impl<T> Clone for QueryState<T> {
    fn clone(&self) -> Self {
        Self {
            status: self.status,
            data: self.data.clone(),
            error: self.error.clone(),
            inflight: self.inflight,
            updated_at: self.updated_at,
            last_duration: self.last_duration,
        }
    }
}

impl<T> Default for QueryState<T> {
    fn default() -> Self {
        Self {
            status: QueryStatus::Idle,
            data: None,
            error: None,
            inflight: None,
            updated_at: None,
            last_duration: None,
        }
    }
}

#[derive(Debug, Clone)]
pub struct QueryPolicy {
    pub stale_time: Duration,
    pub cache_time: Duration,
    pub dedupe_inflight: bool,
    pub keep_previous_data_while_loading: bool,
    pub cancel_mode: QueryCancelMode,
}

impl Default for QueryPolicy {
    fn default() -> Self {
        Self {
            stale_time: Duration::from_secs(2),
            cache_time: Duration::from_secs(60),
            dedupe_inflight: true,
            keep_previous_data_while_loading: true,
            cancel_mode: QueryCancelMode::CancelInFlight,
        }
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum QueryCancelMode {
    /// Cancel the previous inflight task when starting a new request for the same key.
    CancelInFlight,
    /// Allow multiple inflight tasks; only the latest completion is applied.
    KeepInFlight,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
struct QueryKeyId {
    type_id: TypeId,
    namespace: &'static str,
    hash: u64,
}

#[derive(Debug, PartialEq, Eq, Hash)]
pub struct QueryKey<T: 'static> {
    namespace: &'static str,
    hash: u64,
    _phantom: PhantomData<*const T>,
}

impl<T: 'static> Copy for QueryKey<T> {}

impl<T: 'static> Clone for QueryKey<T> {
    fn clone(&self) -> Self {
        *self
    }
}

impl<T: 'static> QueryKey<T> {
    #[track_caller]
    pub fn new(namespace: &'static str, key: &impl Hash) -> Self {
        Self {
            namespace,
            hash: stable_hash(key),
            _phantom: PhantomData,
        }
    }

    pub fn namespace(&self) -> &'static str {
        self.namespace
    }

    pub fn hash(&self) -> u64 {
        self.hash
    }

    fn id(&self) -> QueryKeyId {
        QueryKeyId {
            type_id: TypeId::of::<T>(),
            namespace: self.namespace,
            hash: self.hash,
        }
    }
}

#[derive(Debug, Clone)]
pub struct QueryHandle<T: 'static> {
    key: QueryKey<T>,
    model: Model<QueryState<T>>,
}

impl<T: 'static> QueryHandle<T> {
    pub fn key(&self) -> QueryKey<T> {
        self.key
    }

    pub fn model(&self) -> &Model<QueryState<T>> {
        &self.model
    }
}

type ApplyFn = fn(&mut dyn Any, QueryApplyMsg);

#[derive(Debug)]
struct QueryRuntimeEntry {
    type_id: TypeId,
    namespace: &'static str,
    hash: u64,
    model_id: ModelId,
    policy: QueryPolicy,
    last_used: Instant,
    stale: bool,
    inflight: Option<Inflight>,
    apply: ApplyFn,
}

#[derive(Debug)]
struct Inflight {
    id: u64,
    // Kept for cancellation-on-drop and future diagnostics.
    #[allow(dead_code)]
    started_at: Instant,
    #[allow(dead_code)]
    task: BackgroundTask,
}

struct QueryRuntime {
    exec: Executors,
    inbox: Inbox<QueryInboxMsg>,
    registered: AtomicBool,
    next_inflight_id: AtomicU64,
    entries: Mutex<HashMap<QueryKeyId, QueryRuntimeEntry>>,
}

impl QueryRuntime {
    fn new(dispatcher: DispatcherHandle) -> Self {
        Self {
            exec: Executors::new(dispatcher),
            inbox: Inbox::new(Default::default()),
            registered: AtomicBool::new(false),
            next_inflight_id: AtomicU64::new(1),
            entries: Mutex::new(HashMap::new()),
        }
    }

    fn next_inflight_id(&self) -> u64 {
        self.next_inflight_id.fetch_add(1, Ordering::Relaxed)
    }

    fn apply_inbox_msg(&self, host: &mut dyn InboxDrainHost, msg: QueryInboxMsg) {
        let apply = {
            let mut entries = self.entries.lock().unwrap_or_else(|p| p.into_inner());
            let Some(entry) = entries.get_mut(&msg.key) else {
                return;
            };

            if entry.model_id != msg.model_id {
                // The query was GC'd and recreated; ignore stale completions.
                return;
            }

            // Clear inflight tracking so follow-up calls can refetch if needed.
            if let Some(inflight) = entry.inflight.as_ref()
                && inflight.id == msg.inflight_id
            {
                entry.inflight = None;
            }

            entry.apply
        };

        let _ = host
            .models_mut()
            .update_any(msg.model_id, |any| apply(any, msg.apply));

        if let Some(window) = msg.window {
            host.request_redraw(window);
        }
    }
}

fn query_inbox_drainer(runtime: Arc<QueryRuntime>) -> InboxDrainer<QueryInboxMsg> {
    InboxDrainer::new(runtime.inbox.clone(), move |host, _window, msg| {
        runtime.apply_inbox_msg(host, msg);
    })
}

#[derive(Debug)]
struct QueryApplyMsg {
    inflight_id: u64,
    finished_at: Instant,
    duration: Duration,
    result: Result<Box<dyn Any + Send>, Arc<str>>,
}

#[derive(Debug)]
struct QueryInboxMsg {
    window: Option<AppWindowId>,
    key: QueryKeyId,
    model_id: ModelId,
    inflight_id: u64,
    apply: QueryApplyMsg,
}

pub struct QueryClient {
    runtime: Arc<QueryRuntime>,
    /// Main-thread-only typed model handles kept alive for cache persistence.
    handles: HashMap<QueryKeyId, Box<dyn Any>>,
    last_gc_at: Option<Instant>,
}

impl QueryClient {
    pub fn new(dispatcher: DispatcherHandle) -> Self {
        Self {
            runtime: Arc::new(QueryRuntime::new(dispatcher)),
            handles: HashMap::new(),
            last_gc_at: None,
        }
    }

    fn ensure_registered<H: UiHost>(&mut self, app: &mut H) {
        if self.runtime.registered.swap(true, Ordering::SeqCst) {
            return;
        }

        let runtime = self.runtime.clone();
        app.with_global_mut_untracked(InboxDrainRegistry::default, |registry, _app| {
            registry.register(Arc::new(query_inbox_drainer(runtime)));
        });
    }

    pub fn use_query<H, T>(
        &mut self,
        app: &mut H,
        window: AppWindowId,
        key: QueryKey<T>,
        policy: QueryPolicy,
        fetch: impl FnOnce(CancellationToken) -> Result<T, Arc<str>> + Send + 'static,
    ) -> QueryHandle<T>
    where
        H: UiHost,
        T: Any + Send + Sync + 'static,
    {
        self.ensure_registered(app);

        let now = Instant::now();
        self.gc(now);

        let key_id = key.id();
        let model = self
            .handles
            .get(&key_id)
            .and_then(|any| any.downcast_ref::<Model<QueryState<T>>>().cloned())
            .unwrap_or_else(|| {
                let model = app.models_mut().insert(QueryState::<T>::default());
                self.handles.insert(key_id, Box::new(model.clone()));
                model
            });

        self.touch_entry::<T>(key, model.id(), policy.clone(), now);

        let should_fetch = self.should_fetch(app, &model, key_id, now);
        if should_fetch {
            self.start_fetch(app, window, key, policy, model.clone(), fetch, now);
        }

        QueryHandle { key, model }
    }

    pub fn invalidate<H, T>(&mut self, _app: &mut H, key: QueryKey<T>)
    where
        H: UiHost,
        T: Any + Send + Sync + 'static,
    {
        let key_id = key.id();
        let mut entries = self
            .runtime
            .entries
            .lock()
            .unwrap_or_else(|p| p.into_inner());
        let Some(entry) = entries.get_mut(&key_id) else {
            return;
        };
        entry.stale = true;
        if entry.policy.cancel_mode == QueryCancelMode::CancelInFlight {
            entry.inflight = None;
        }
    }

    pub fn invalidate_namespace(&mut self, namespace: &'static str) {
        let mut entries = self
            .runtime
            .entries
            .lock()
            .unwrap_or_else(|p| p.into_inner());
        for entry in entries.values_mut() {
            if entry.namespace == namespace {
                entry.stale = true;
                if entry.policy.cancel_mode == QueryCancelMode::CancelInFlight {
                    entry.inflight = None;
                }
            }
        }
    }

    #[track_caller]
    fn touch_entry<T: Any + Send + Sync + 'static>(
        &mut self,
        key: QueryKey<T>,
        model_id: ModelId,
        policy: QueryPolicy,
        now: Instant,
    ) {
        let key_id = key.id();
        let mut entries = self
            .runtime
            .entries
            .lock()
            .unwrap_or_else(|p| p.into_inner());
        let entry = entries.entry(key_id).or_insert_with(|| QueryRuntimeEntry {
            type_id: TypeId::of::<T>(),
            namespace: key.namespace,
            hash: key.hash,
            model_id,
            policy: policy.clone(),
            last_used: now,
            stale: true,
            inflight: None,
            apply: apply_query_result::<T>,
        });

        if entry.type_id != TypeId::of::<T>() {
            tracing::error!(
                namespace = entry.namespace,
                hash = entry.hash,
                stored = ?entry.type_id,
                requested = ?TypeId::of::<T>(),
                "query key type mismatch"
            );
            return;
        }

        entry.model_id = model_id;
        entry.policy = policy;
        entry.last_used = now;
    }

    fn should_fetch<H: UiHost, T: Any + Send + Sync + 'static>(
        &self,
        app: &mut H,
        model: &Model<QueryState<T>>,
        key: QueryKeyId,
        now: Instant,
    ) -> bool {
        let (dedupe_inflight, stale, stale_time) = {
            let entries = self
                .runtime
                .entries
                .lock()
                .unwrap_or_else(|p| p.into_inner());
            let Some(entry) = entries.get(&key) else {
                return true;
            };
            (
                entry.policy.dedupe_inflight,
                entry.stale,
                entry.policy.stale_time,
            )
        };

        let state = model.read_ref(app, |s| (*s).clone()).ok();
        let Some(state) = state else {
            return true;
        };

        if dedupe_inflight && state.inflight.is_some() {
            return false;
        }

        if stale {
            return true;
        }

        if state.status == QueryStatus::Idle {
            return true;
        }

        let Some(updated_at) = state.updated_at else {
            return true;
        };
        now.duration_since(updated_at) >= stale_time
    }

    fn start_fetch<H, T>(
        &mut self,
        app: &mut H,
        window: AppWindowId,
        key: QueryKey<T>,
        policy: QueryPolicy,
        model: Model<QueryState<T>>,
        fetch: impl FnOnce(CancellationToken) -> Result<T, Arc<str>> + Send + 'static,
        now: Instant,
    ) where
        H: UiHost,
        T: Any + Send + Sync + 'static,
    {
        let key_id = key.id();
        let model_id = model.id();
        let inflight_id = self.runtime.next_inflight_id();
        let sender = self.runtime.inbox.sender();
        let started_at = now;

        let task =
            self.runtime
                .exec
                .spawn_background_to_inbox(Some(window), sender, move |token| {
                    let result = if cfg!(panic = "unwind") {
                        catch_unwind(AssertUnwindSafe(|| fetch(token)))
                    } else {
                        Ok(fetch(token))
                    };

                    let finished_at = Instant::now();
                    let duration = finished_at.duration_since(started_at);

                    let result = match result {
                        Ok(Ok(value)) => Ok(Box::new(value) as Box<dyn Any + Send>),
                        Ok(Err(err)) => Err(err),
                        Err(panic) => {
                            let loc = Location::caller();
                            let _ = loc;
                            resume_unwind(panic)
                        }
                    };

                    QueryInboxMsg {
                        window: Some(window),
                        key: key_id,
                        model_id,
                        inflight_id,
                        apply: QueryApplyMsg {
                            inflight_id,
                            finished_at,
                            duration,
                            result,
                        },
                    }
                });

        let mut entries = self
            .runtime
            .entries
            .lock()
            .unwrap_or_else(|p| p.into_inner());
        let Some(entry) = entries.get_mut(&key_id) else {
            return;
        };

        if let Some(prev) = entry.inflight.take()
            && policy.cancel_mode == QueryCancelMode::CancelInFlight
        {
            drop(prev);
        }

        entry.stale = false;
        entry.inflight = Some(Inflight {
            id: inflight_id,
            started_at,
            task,
        });

        let _ = app.models_mut().update(&model, |st| {
            st.status = QueryStatus::Loading;
            st.inflight = Some(inflight_id);
            st.error = None;
            if !policy.keep_previous_data_while_loading {
                st.data = None;
            }
        });
    }

    pub fn gc(&mut self, now: Instant) {
        let last = self.last_gc_at;
        if last.is_some_and(|t| now.duration_since(t) < Duration::from_millis(250)) {
            return;
        }
        self.last_gc_at = Some(now);

        let evict: Vec<QueryKeyId> = {
            let entries = self
                .runtime
                .entries
                .lock()
                .unwrap_or_else(|p| p.into_inner());
            entries
                .iter()
                .filter_map(|(key, entry)| {
                    let idle = entry.inflight.is_none();
                    let expired = now.duration_since(entry.last_used) >= entry.policy.cache_time;
                    (idle && expired).then_some(*key)
                })
                .collect()
        };

        if evict.is_empty() {
            return;
        }

        {
            let mut entries = self
                .runtime
                .entries
                .lock()
                .unwrap_or_else(|p| p.into_inner());
            for key in &evict {
                entries.remove(key);
            }
        }

        for key in evict {
            self.handles.remove(&key);
        }
    }
}

fn apply_query_result<T: Any + Send + Sync + 'static>(state_any: &mut dyn Any, msg: QueryApplyMsg) {
    let state = state_any
        .downcast_mut::<QueryState<T>>()
        .expect("query model type mismatch");

    if state.inflight != Some(msg.inflight_id) {
        return;
    }

    state.inflight = None;
    state.last_duration = Some(msg.duration);

    match msg.result {
        Ok(value_any) => {
            let Ok(value) = value_any.downcast::<T>() else {
                state.status = QueryStatus::Error;
                state.error = Some(Arc::<str>::from("query result type mismatch"));
                return;
            };

            state.status = QueryStatus::Success;
            state.data = Some(Arc::new(*value));
            state.error = None;
            state.updated_at = Some(msg.finished_at);
        }
        Err(err) => {
            state.status = QueryStatus::Error;
            state.error = Some(err);
            state.updated_at = Some(msg.finished_at);
        }
    }
}

pub fn with_query_client<H: UiHost, R>(
    app: &mut H,
    f: impl FnOnce(&mut QueryClient, &mut H) -> R,
) -> Option<R> {
    let dispatcher = app.global::<DispatcherHandle>()?.clone();
    Some(app.with_global_mut_untracked(
        || QueryClient::new(dispatcher),
        |client, app| f(client, app),
    ))
}

#[derive(Default)]
struct Fnv1a64(u64);

impl Hasher for Fnv1a64 {
    fn finish(&self) -> u64 {
        self.0
    }

    fn write(&mut self, bytes: &[u8]) {
        let mut hash = if self.0 == 0 {
            0xcbf29ce484222325
        } else {
            self.0
        };
        for b in bytes {
            hash ^= *b as u64;
            hash = hash.wrapping_mul(0x100000001b3);
        }
        self.0 = hash;
    }
}

fn stable_hash<T: Hash>(value: &T) -> u64 {
    let mut hasher = Fnv1a64::default();
    value.hash(&mut hasher);
    hasher.finish()
}

#[cfg(feature = "ui")]
pub mod ui {
    use super::*;
    use fret_ui::{ElementContext, UiHost};

    pub trait QueryElementContextExt {
        fn use_query<T: Any + Send + Sync + 'static>(
            &mut self,
            key: QueryKey<T>,
            policy: QueryPolicy,
            fetch: impl FnOnce(CancellationToken) -> Result<T, Arc<str>> + Send + 'static,
        ) -> QueryHandle<T>;
    }

    impl<'a, H: UiHost> QueryElementContextExt for ElementContext<'a, H> {
        fn use_query<T: Any + Send + Sync + 'static>(
            &mut self,
            key: QueryKey<T>,
            policy: QueryPolicy,
            fetch: impl FnOnce(CancellationToken) -> Result<T, Arc<str>> + Send + 'static,
        ) -> QueryHandle<T> {
            let window = self.window;
            with_query_client(self.app, |client, app| {
                client.use_query(app, window, key, policy, fetch)
            })
            .unwrap_or_else(|| {
                let model = self.app.models_mut().insert(QueryState::<T> {
                    status: QueryStatus::Error,
                    data: None,
                    error: Some(Arc::<str>::from("missing DispatcherHandle global")),
                    inflight: None,
                    updated_at: None,
                    last_duration: None,
                });
                QueryHandle { key, model }
            })
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use fret_app::App;
    use fret_runtime::{DispatchPriority, Dispatcher, ExecCapabilities, Runnable};

    #[derive(Default)]
    struct TestDispatcher {
        background: Mutex<Vec<Runnable>>,
        main: Mutex<Vec<Runnable>>,
    }

    impl TestDispatcher {
        fn take_background(&self) -> Vec<Runnable> {
            let mut guard = self.background.lock().unwrap();
            std::mem::take(&mut *guard)
        }
    }

    impl Dispatcher for TestDispatcher {
        fn dispatch_on_main_thread(&self, task: Runnable) {
            self.main.lock().unwrap().push(task);
        }

        fn dispatch_background(&self, task: Runnable, _priority: DispatchPriority) {
            self.background.lock().unwrap().push(task);
        }

        fn dispatch_after(&self, _delay: Duration, task: Runnable) {
            self.main.lock().unwrap().push(task);
        }

        fn wake(&self, _window: Option<AppWindowId>) {}

        fn exec_capabilities(&self) -> ExecCapabilities {
            ExecCapabilities::default()
        }
    }

    fn drain_inboxes(app: &mut App, window: Option<AppWindowId>) -> bool {
        app.with_global_mut_untracked(InboxDrainRegistry::default, |registry, app| {
            registry.drain_all(app, window)
        })
    }

    #[test]
    fn dedupes_inflight_by_default() {
        let mut app = App::new();
        let dispatcher = Arc::new(TestDispatcher::default());
        let dispatcher_handle: DispatcherHandle = dispatcher.clone();
        app.set_global::<DispatcherHandle>(dispatcher_handle);

        let window = AppWindowId::default();
        let key = QueryKey::<u32>::new("test", &0u32);

        with_query_client(&mut app, |client, app| {
            let _ = client.use_query(app, window, key, QueryPolicy::default(), |_token| Ok(1u32));
            let _ = client.use_query(app, window, key, QueryPolicy::default(), |_token| Ok(2u32));
        })
        .unwrap();

        assert_eq!(dispatcher.take_background().len(), 1);
    }

    #[test]
    fn invalidate_triggers_refetch() {
        let mut app = App::new();
        let dispatcher = Arc::new(TestDispatcher::default());
        let dispatcher_handle: DispatcherHandle = dispatcher.clone();
        app.set_global::<DispatcherHandle>(dispatcher_handle);

        let window = AppWindowId::default();
        let key = QueryKey::<u32>::new("test", &123u32);

        let handle = with_query_client(&mut app, |client, app| {
            client.use_query(app, window, key, QueryPolicy::default(), |_token| Ok(1u32))
        })
        .unwrap();

        let tasks = dispatcher.take_background();
        assert_eq!(tasks.len(), 1);
        for task in tasks {
            task();
        }

        assert!(drain_inboxes(&mut app, Some(window)));

        let state = handle.model.read_ref(&app, |s| s.clone()).unwrap();
        assert_eq!(state.status, QueryStatus::Success);

        with_query_client(&mut app, |client, app| {
            client.invalidate(app, key);
            let _ = client.use_query(app, window, key, QueryPolicy::default(), |_token| Ok(2u32));
        })
        .unwrap();

        let tasks = dispatcher.take_background();
        assert_eq!(tasks.len(), 1);
    }

    #[test]
    fn gc_drops_cached_models() {
        let mut app = App::new();
        let dispatcher = Arc::new(TestDispatcher::default());
        let dispatcher_handle: DispatcherHandle = dispatcher.clone();
        app.set_global::<DispatcherHandle>(dispatcher_handle);

        let window = AppWindowId::default();
        let key = QueryKey::<u32>::new("test", &42u32);

        let handle = with_query_client(&mut app, |client, app| {
            let policy = QueryPolicy {
                cache_time: Duration::ZERO,
                ..Default::default()
            };
            client.use_query(app, window, key, policy, |_token| Ok(1u32))
        })
        .unwrap();

        let tasks = dispatcher.take_background();
        assert_eq!(tasks.len(), 1);
        for task in tasks {
            task();
        }
        assert!(drain_inboxes(&mut app, Some(window)));

        let first_id = handle.model.id();
        with_query_client(&mut app, |client, _app| {
            client.last_gc_at = None;
            client.gc(Instant::now());
        })
        .unwrap();

        let second_id = with_query_client(&mut app, |client, app| {
            let policy = QueryPolicy {
                cache_time: Duration::ZERO,
                ..Default::default()
            };
            let handle = client.use_query(app, window, key, policy, |_token| Ok(1u32));
            handle.model.id()
        })
        .unwrap();

        assert_ne!(first_id, second_id);
    }
}
