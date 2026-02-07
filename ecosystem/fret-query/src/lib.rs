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
//!
//! ## Query keys
//!
//! Keys are typed (`QueryKey<T>`) and consist of:
//! - a `'static` namespace (used for bulk invalidation),
//! - a 64-bit stable hash of a structured key value.
//!
//! Recommended conventions:
//! - Use a dot-separated namespace like `"my_crate.feature.query_name.v1"`.
//! - Ensure the hashed key value is deterministic and only contains the parameters that affect
//!   the fetch result (avoid `HashMap` iteration order, pointer addresses, random IDs, etc.).

use std::any::{Any, TypeId, type_name};
use std::collections::HashMap;
use std::fmt;
use std::future::Future;
use std::hash::{Hash, Hasher};
use std::marker::PhantomData;
use std::panic::{AssertUnwindSafe, Location, catch_unwind, resume_unwind};
use std::sync::atomic::{AtomicBool, AtomicU64, Ordering};
use std::sync::{Arc, Mutex};
use std::time::Duration;

use fret_core::time::Instant;
use fret_core::{AppWindowId, FrameId};
#[cfg(feature = "tokio")]
pub use fret_executor::TokioSpawner;
#[cfg(all(feature = "wasm", target_arch = "wasm32"))]
pub use fret_executor::WasmSpawner;
use fret_executor::{BackgroundTask, Executors, Inbox, InboxDrainer};
pub use fret_executor::{CancellationToken, FutureSpawner, FutureSpawnerHandle};
use fret_runtime::{DispatcherHandle, InboxDrainHost, InboxDrainRegistry, Model, ModelId, UiHost};

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum QueryStatus {
    Idle,
    Loading,
    Success,
    Error,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
#[non_exhaustive]
pub enum QueryErrorKind {
    /// A transient failure that may succeed if retried (network hiccup, temporary IO failure, etc).
    Transient,
    /// A permanent failure that is unlikely to succeed if retried without user action or new input.
    Permanent,
}

#[derive(Debug, Clone)]
pub struct QueryError {
    kind: QueryErrorKind,
    message: Arc<str>,
}

impl QueryError {
    pub fn new(kind: QueryErrorKind, message: impl Into<Arc<str>>) -> Self {
        Self {
            kind,
            message: message.into(),
        }
    }

    pub fn transient(message: impl Into<Arc<str>>) -> Self {
        Self::new(QueryErrorKind::Transient, message)
    }

    pub fn permanent(message: impl Into<Arc<str>>) -> Self {
        Self::new(QueryErrorKind::Permanent, message)
    }

    pub fn kind(&self) -> QueryErrorKind {
        self.kind
    }

    pub fn message(&self) -> &Arc<str> {
        &self.message
    }
}

impl fmt::Display for QueryError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        self.message.fmt(f)
    }
}

impl std::error::Error for QueryError {}

impl From<Arc<str>> for QueryError {
    fn from(value: Arc<str>) -> Self {
        Self::permanent(value)
    }
}

impl From<String> for QueryError {
    fn from(value: String) -> Self {
        Self::permanent(Arc::<str>::from(value))
    }
}

impl From<&'static str> for QueryError {
    fn from(value: &'static str) -> Self {
        Self::permanent(Arc::<str>::from(value))
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum QueryRetryOn {
    /// Only retry transient errors (recommended default).
    Transient,
    /// Retry any error kind (use with care).
    Any,
}

#[derive(Debug, Clone)]
pub enum QueryRetryPolicy {
    None,
    Fixed {
        max_retries: u32,
        delay: Duration,
        retry_on: QueryRetryOn,
    },
    Exponential {
        max_retries: u32,
        base_delay: Duration,
        max_delay: Duration,
        retry_on: QueryRetryOn,
    },
}

impl QueryRetryPolicy {
    pub fn none() -> Self {
        Self::None
    }

    pub fn fixed(max_retries: u32, delay: Duration) -> Self {
        Self::Fixed {
            max_retries,
            delay,
            retry_on: QueryRetryOn::Transient,
        }
    }

    pub fn fixed_any(max_retries: u32, delay: Duration) -> Self {
        Self::Fixed {
            max_retries,
            delay,
            retry_on: QueryRetryOn::Any,
        }
    }

    pub fn exponential(max_retries: u32, base_delay: Duration, max_delay: Duration) -> Self {
        Self::Exponential {
            max_retries,
            base_delay,
            max_delay,
            retry_on: QueryRetryOn::Transient,
        }
    }

    pub fn exponential_any(max_retries: u32, base_delay: Duration, max_delay: Duration) -> Self {
        Self::Exponential {
            max_retries,
            base_delay,
            max_delay,
            retry_on: QueryRetryOn::Any,
        }
    }

    pub fn max_retries(&self) -> u32 {
        match self {
            QueryRetryPolicy::None => 0,
            QueryRetryPolicy::Fixed { max_retries, .. } => *max_retries,
            QueryRetryPolicy::Exponential { max_retries, .. } => *max_retries,
        }
    }

    pub fn retry_on(&self) -> QueryRetryOn {
        match self {
            QueryRetryPolicy::None => QueryRetryOn::Transient,
            QueryRetryPolicy::Fixed { retry_on, .. } => *retry_on,
            QueryRetryPolicy::Exponential { retry_on, .. } => *retry_on,
        }
    }

    fn should_retry_error_kind(&self, err: &QueryError) -> bool {
        match self.retry_on() {
            QueryRetryOn::Any => true,
            QueryRetryOn::Transient => err.kind() == QueryErrorKind::Transient,
        }
    }

    pub fn next_retry_delay(&self, failures: u32, err: &QueryError) -> Option<Duration> {
        if failures == 0 {
            return None;
        }
        if failures > self.max_retries() {
            return None;
        }
        if !self.should_retry_error_kind(err) {
            return None;
        }

        match self {
            QueryRetryPolicy::None => None,
            QueryRetryPolicy::Fixed { delay, .. } => Some(*delay),
            QueryRetryPolicy::Exponential {
                base_delay,
                max_delay,
                ..
            } => {
                let pow = failures.saturating_sub(1).min(30);
                let factor = 1u32 << pow;
                let delay = base_delay
                    .checked_mul(factor)
                    .unwrap_or(*max_delay)
                    .min(*max_delay);
                Some(delay)
            }
        }
    }
}

impl Default for QueryRetryPolicy {
    fn default() -> Self {
        Self::None
    }
}

#[derive(Debug, Clone, Copy)]
pub struct QueryRetryState {
    pub failures: u32,
    pub max_retries: u32,
    pub next_retry_at: Option<Instant>,
}

impl Default for QueryRetryState {
    fn default() -> Self {
        Self {
            failures: 0,
            max_retries: 0,
            next_retry_at: None,
        }
    }
}

#[derive(Debug)]
pub struct QueryState<T> {
    pub status: QueryStatus,
    pub data: Option<Arc<T>>,
    pub error: Option<QueryError>,
    pub inflight: Option<u64>,
    pub updated_at: Option<Instant>,
    pub last_duration: Option<Duration>,
    pub retry: QueryRetryState,
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
            retry: self.retry,
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
            retry: QueryRetryState::default(),
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
    pub retry: QueryRetryPolicy,
}

impl Default for QueryPolicy {
    fn default() -> Self {
        Self {
            stale_time: Duration::from_secs(2),
            cache_time: Duration::from_secs(60),
            dedupe_inflight: true,
            keep_previous_data_while_loading: true,
            cancel_mode: QueryCancelMode::CancelInFlight,
            retry: QueryRetryPolicy::default(),
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

#[derive(Debug)]
pub struct QueryKey<T: 'static> {
    namespace: &'static str,
    hash: u64,
    debug_label: Option<&'static str>,
    _phantom: PhantomData<*const T>,
}

impl<T: 'static> Copy for QueryKey<T> {}

impl<T: 'static> Clone for QueryKey<T> {
    fn clone(&self) -> Self {
        *self
    }
}

impl<T: 'static> PartialEq for QueryKey<T> {
    fn eq(&self, other: &Self) -> bool {
        self.namespace == other.namespace && self.hash == other.hash
    }
}

impl<T: 'static> Eq for QueryKey<T> {}

impl<T: 'static> Hash for QueryKey<T> {
    fn hash<H: Hasher>(&self, state: &mut H) {
        self.namespace.hash(state);
        self.hash.hash(state);
    }
}

impl<T: 'static> QueryKey<T> {
    #[track_caller]
    pub fn new(namespace: &'static str, key: &impl Hash) -> Self {
        #[cfg(debug_assertions)]
        debug_validate_query_namespace(namespace, Location::caller());

        Self {
            namespace,
            hash: stable_hash(key),
            debug_label: None,
            _phantom: PhantomData,
        }
    }

    #[track_caller]
    pub fn new_named(namespace: &'static str, key: &impl Hash, debug_label: &'static str) -> Self {
        #[cfg(debug_assertions)]
        debug_validate_query_namespace(namespace, Location::caller());

        Self {
            namespace,
            hash: stable_hash(key),
            debug_label: Some(debug_label),
            _phantom: PhantomData,
        }
    }

    pub fn namespace(&self) -> &'static str {
        self.namespace
    }

    pub fn hash(&self) -> u64 {
        self.hash
    }

    pub fn debug_label(&self) -> Option<&'static str> {
        self.debug_label
    }

    fn id(&self) -> QueryKeyId {
        QueryKeyId {
            type_id: TypeId::of::<T>(),
            namespace: self.namespace,
            hash: self.hash,
        }
    }
}

#[cfg(debug_assertions)]
fn debug_validate_query_namespace(namespace: &'static str, loc: &'static Location<'static>) {
    let has_scope = namespace.contains('.') || namespace.contains("::");
    let has_ws = namespace.chars().any(|c| c.is_whitespace());
    let has_upper = namespace.chars().any(|c| c.is_ascii_uppercase());
    let last = namespace.rsplit('.').next().unwrap_or(namespace);
    let has_version_suffix = last.len() >= 2
        && last.as_bytes()[0] == b'v'
        && last.as_bytes()[1..].iter().all(|b| b.is_ascii_digit());

    let suspicious = !has_scope || has_ws || has_upper || !has_version_suffix;
    if !suspicious {
        return;
    }

    use std::collections::HashSet;
    use std::sync::OnceLock;

    static WARNED: OnceLock<Mutex<HashSet<&'static str>>> = OnceLock::new();
    let warned = WARNED.get_or_init(|| Mutex::new(HashSet::new()));
    let should_warn = warned
        .lock()
        .unwrap_or_else(|p| p.into_inner())
        .insert(namespace);
    if !should_warn {
        return;
    }

    tracing::warn!(
        namespace,
        file = loc.file(),
        line = loc.line(),
        "suspicious query namespace; recommended format is 'crate.subsystem.query_name.v1'"
    );
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

#[derive(Debug, Clone)]
pub struct QueryClientSnapshot {
    pub captured_at: Instant,
    pub entries: Vec<QuerySnapshotEntry>,
}

#[derive(Debug, Clone)]
pub struct QuerySnapshotEntry {
    pub namespace: &'static str,
    pub hash: u64,
    pub debug_label: Option<&'static str>,
    pub type_name: &'static str,
    pub model_id: ModelId,
    pub policy: QueryPolicy,
    pub stale: bool,
    pub status: QueryStatus,
    pub inflight: Option<u64>,
    pub last_used: Instant,
    pub last_observed_frame: Option<FrameId>,
    pub updated_at: Option<Instant>,
    pub last_duration: Option<Duration>,
    pub retry: QueryRetryState,
    pub last_error_kind: Option<QueryErrorKind>,
    pub last_error_message: Option<Arc<str>>,
}

type ApplyFn = fn(&mut dyn Any, QueryApplyMsg) -> bool;
type ApplyRetryFn = fn(&mut dyn Any, QueryRetryState);

#[derive(Debug)]
struct QueryRuntimeRetry {
    failures: u32,
    next_retry_at: Option<Instant>,
    scheduled_wake: Option<CancellationToken>,
}

impl Default for QueryRuntimeRetry {
    fn default() -> Self {
        Self {
            failures: 0,
            next_retry_at: None,
            scheduled_wake: None,
        }
    }
}

#[derive(Debug)]
struct QueryRuntimeEntry {
    type_id: TypeId,
    type_name: &'static str,
    namespace: &'static str,
    hash: u64,
    debug_label: Option<&'static str>,
    model_id: ModelId,
    policy: QueryPolicy,
    last_used: Instant,
    last_observed_frame: Option<FrameId>,
    stale: bool,
    status: QueryStatus,
    updated_at: Option<Instant>,
    last_duration: Option<Duration>,
    last_error_kind: Option<QueryErrorKind>,
    last_error_message: Option<Arc<str>>,
    inflight: Option<Inflight>,
    retry: QueryRuntimeRetry,
    apply: ApplyFn,
    apply_retry: ApplyRetryFn,
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
        let window = msg.window;
        let key = msg.key;
        let model_id = msg.model_id;

        let Some(apply_msg) = msg.apply else {
            if let Some(window) = window {
                let exists = {
                    let entries = self.entries.lock().unwrap_or_else(|p| p.into_inner());
                    entries
                        .get(&key)
                        .is_some_and(|entry| entry.model_id == model_id)
                };
                if exists {
                    tracing::debug!(
                        target: "fret_query::diag",
                        namespace = key.namespace,
                        hash = key.hash,
                        model_id = ?model_id,
                        "query retry wake triggered redraw"
                    );
                    host.request_redraw(window);
                }
            }
            return;
        };

        let Some(inflight_id) = msg.inflight_id else {
            return;
        };

        let finished_at = apply_msg.finished_at;
        let duration = apply_msg.duration;
        let outcome_err = apply_msg.result.as_ref().err().cloned();

        let (apply, apply_retry, namespace, hash, debug_label, type_name) = {
            let mut entries = self.entries.lock().unwrap_or_else(|p| p.into_inner());
            let Some(entry) = entries.get_mut(&key) else {
                tracing::debug!(
                    target: "fret_query::diag",
                    namespace = key.namespace,
                    hash = key.hash,
                    model_id = ?model_id,
                    inflight_id,
                    "query completion dropped because entry is missing"
                );
                return;
            };

            if entry.model_id != model_id {
                // The query was GC'd and recreated; ignore stale completions.
                tracing::debug!(
                    target: "fret_query::diag",
                    namespace = entry.namespace,
                    hash = entry.hash,
                    debug_label = entry.debug_label,
                    type_name = entry.type_name,
                    model_id = ?model_id,
                    entry_model_id = ?entry.model_id,
                    inflight_id,
                    "query completion dropped due to model mismatch"
                );
                return;
            }

            // Clear inflight tracking so follow-up calls can refetch if needed.
            if let Some(inflight) = entry.inflight.as_ref()
                && inflight.id == inflight_id
            {
                entry.inflight = None;
            }

            (
                entry.apply,
                entry.apply_retry,
                entry.namespace,
                entry.hash,
                entry.debug_label,
                entry.type_name,
            )
        };

        let applied = host
            .models_mut()
            .update_any(model_id, |any| apply(any, apply_msg))
            .ok()
            .unwrap_or(false);
        if !applied {
            tracing::debug!(
                target: "fret_query::diag",
                namespace,
                hash,
                debug_label,
                type_name,
                model_id = ?model_id,
                inflight_id,
                "query completion ignored because inflight token no longer matches"
            );
            return;
        }

        let (retry_state, retry_delay) = {
            let mut entries = self.entries.lock().unwrap_or_else(|p| p.into_inner());
            let Some(entry) = entries.get_mut(&key) else {
                return;
            };
            if entry.model_id != model_id {
                return;
            }

            let max_retries = entry.policy.retry.max_retries();
            let mut retry_delay = None;

            entry.updated_at = Some(finished_at);
            entry.last_duration = Some(duration);

            match outcome_err.as_ref() {
                None => {
                    entry.status = QueryStatus::Success;
                    entry.last_error_kind = None;
                    entry.last_error_message = None;
                    entry.retry.failures = 0;
                    entry.retry.next_retry_at = None;
                    if let Some(token) = entry.retry.scheduled_wake.take() {
                        token.cancel();
                    }
                }
                Some(err) => {
                    entry.status = QueryStatus::Error;
                    entry.last_error_kind = Some(err.kind());
                    entry.last_error_message = Some(err.message().clone());
                    entry.retry.failures = entry.retry.failures.saturating_add(1);
                    let failures = entry.retry.failures;
                    let delay = entry.policy.retry.next_retry_delay(failures, &err);
                    retry_delay = delay;

                    entry.retry.next_retry_at = delay.map(|d| Instant::now() + d);

                    if let Some(token) = entry.retry.scheduled_wake.take() {
                        token.cancel();
                    }

                    if let (Some(delay), Some(window)) = (delay, window) {
                        let sender = self.inbox.sender();
                        let wake_msg = QueryInboxMsg {
                            window: Some(window),
                            key,
                            model_id,
                            inflight_id: None,
                            apply: None,
                        };
                        let task = self.exec.spawn_after(delay, move |token| {
                            if token.is_cancelled() {
                                return;
                            }
                            let _ = sender.send(wake_msg);
                        });
                        let token = task.token();
                        task.detach();
                        entry.retry.scheduled_wake = Some(token);

                        tracing::debug!(
                            target: "fret_query::diag",
                            namespace = entry.namespace,
                            hash = entry.hash,
                            debug_label = entry.debug_label,
                            type_name = entry.type_name,
                            model_id = ?entry.model_id,
                            failures,
                            max_retries,
                            delay_ms = delay.as_millis() as u64,
                            "query retry scheduled"
                        );
                    }
                }
            }

            (
                QueryRetryState {
                    failures: entry.retry.failures,
                    max_retries,
                    next_retry_at: entry.retry.next_retry_at,
                },
                retry_delay,
            )
        };

        let _ = host
            .models_mut()
            .update_any(model_id, |any| apply_retry(any, retry_state));

        match outcome_err {
            None => {
                tracing::debug!(
                    target: "fret_query::diag",
                    namespace,
                    hash,
                    debug_label,
                    type_name,
                    model_id = ?model_id,
                    inflight_id,
                    duration_ms = duration.as_millis() as u64,
                    "query fetch finished (success)"
                );
            }
            Some(err) => {
                tracing::debug!(
                    target: "fret_query::diag",
                    namespace,
                    hash,
                    debug_label,
                    type_name,
                    model_id = ?model_id,
                    inflight_id,
                    duration_ms = duration.as_millis() as u64,
                    error_kind = ?err.kind(),
                    error = %err,
                    retry_scheduled = retry_delay.is_some(),
                    "query fetch finished (error)"
                );
            }
        }

        if let Some(window) = window {
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
    result: Result<Box<dyn Any + Send>, QueryError>,
}

#[derive(Debug)]
struct QueryInboxMsg {
    window: Option<AppWindowId>,
    key: QueryKeyId,
    model_id: ModelId,
    inflight_id: Option<u64>,
    apply: Option<QueryApplyMsg>,
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
        fetch: impl FnOnce(CancellationToken) -> Result<T, QueryError> + Send + 'static,
    ) -> QueryHandle<T>
    where
        H: UiHost,
        T: Any + Send + Sync + 'static,
    {
        self.ensure_registered(app);

        let frame_id = app.frame_id();
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

        let remounted = self.touch_entry::<T>(key, model.id(), policy.clone(), now, frame_id);

        let should_fetch = self.should_fetch(app, &model, key_id, now, remounted);
        if should_fetch {
            self.start_fetch(app, window, key, policy, model.clone(), fetch, now);
        }

        QueryHandle { key, model }
    }

    /// Use a query whose fetch function is an async future.
    ///
    /// This requires installing a `FutureSpawnerHandle` as a global (e.g. a tokio-backed spawner),
    /// so `fret-query` can integrate with any async runtime without forcing one in the kernel.
    pub fn use_query_async<H, T, Fut>(
        &mut self,
        app: &mut H,
        window: AppWindowId,
        key: QueryKey<T>,
        policy: QueryPolicy,
        fetch: impl FnOnce(CancellationToken) -> Fut + Send + 'static,
    ) -> QueryHandle<T>
    where
        H: UiHost,
        T: Any + Send + Sync + 'static,
        Fut: Future<Output = Result<T, QueryError>> + Send + 'static,
    {
        self.ensure_registered(app);

        let frame_id = app.frame_id();
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

        let remounted = self.touch_entry::<T>(key, model.id(), policy.clone(), now, frame_id);

        let should_fetch = self.should_fetch(app, &model, key_id, now, remounted);
        if should_fetch {
            let spawner = app.global::<FutureSpawnerHandle>().cloned();
            if let Some(spawner) = spawner {
                self.start_fetch_async(
                    app,
                    window,
                    key,
                    policy,
                    model.clone(),
                    &*spawner,
                    fetch,
                    now,
                );
            } else {
                let _ = app.models_mut().update(&model, |st| {
                    st.status = QueryStatus::Error;
                    st.error = Some(QueryError::permanent(
                        "missing FutureSpawnerHandle global (install an async spawner to use use_query_async)",
                    ));
                });
            }
        }

        QueryHandle { key, model }
    }

    /// Use a query whose fetch function is an async `!Send` future (typically wasm).
    pub fn use_query_async_local<H, T, Fut>(
        &mut self,
        app: &mut H,
        window: AppWindowId,
        key: QueryKey<T>,
        policy: QueryPolicy,
        fetch: impl FnOnce(CancellationToken) -> Fut + 'static,
    ) -> QueryHandle<T>
    where
        H: UiHost,
        T: Any + Send + Sync + 'static,
        Fut: Future<Output = Result<T, QueryError>> + 'static,
    {
        self.ensure_registered(app);

        let frame_id = app.frame_id();
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

        let remounted = self.touch_entry::<T>(key, model.id(), policy.clone(), now, frame_id);

        let should_fetch = self.should_fetch(app, &model, key_id, now, remounted);
        if should_fetch {
            let spawner = app.global::<FutureSpawnerHandle>().cloned();
            if let Some(spawner) = spawner {
                self.start_fetch_async_local(
                    app,
                    window,
                    key,
                    policy,
                    model.clone(),
                    &*spawner,
                    fetch,
                    now,
                );
            } else {
                let _ = app.models_mut().update(&model, |st| {
                    st.status = QueryStatus::Error;
                    st.error = Some(QueryError::permanent(
                        "missing FutureSpawnerHandle global (install an async spawner to use use_query_async_local)",
                    ));
                });
            }
        }

        QueryHandle { key, model }
    }

    /// Prefetch a query into the cache.
    ///
    /// This is semantically equivalent to calling [`QueryClient::use_query`] at least once, but is
    /// intended for non-render paths (e.g. hover prefetch, navigation intent, service warmup).
    pub fn prefetch<H, T>(
        &mut self,
        app: &mut H,
        window: AppWindowId,
        key: QueryKey<T>,
        policy: QueryPolicy,
        fetch: impl FnOnce(CancellationToken) -> Result<T, QueryError> + Send + 'static,
    ) -> QueryHandle<T>
    where
        H: UiHost,
        T: Any + Send + Sync + 'static,
    {
        self.use_query(app, window, key, policy, fetch)
    }

    /// Force a refetch for the given key (even if the cached value is still fresh).
    ///
    /// Note: inflight behavior is still controlled by [`QueryPolicy`]:
    /// - if `dedupe_inflight=true`, an inflight request will not be duplicated.
    /// - if `cancel_mode=CancelInFlight`, the previous inflight task is canceled before starting.
    pub fn refetch<H, T>(
        &mut self,
        app: &mut H,
        window: AppWindowId,
        key: QueryKey<T>,
        policy: QueryPolicy,
        fetch: impl FnOnce(CancellationToken) -> Result<T, QueryError> + Send + 'static,
    ) -> QueryHandle<T>
    where
        H: UiHost,
        T: Any + Send + Sync + 'static,
    {
        self.invalidate(app, key);
        self.use_query(app, window, key, policy, fetch)
    }

    pub fn prefetch_async<H, T, Fut>(
        &mut self,
        app: &mut H,
        window: AppWindowId,
        key: QueryKey<T>,
        policy: QueryPolicy,
        fetch: impl FnOnce(CancellationToken) -> Fut + Send + 'static,
    ) -> QueryHandle<T>
    where
        H: UiHost,
        T: Any + Send + Sync + 'static,
        Fut: Future<Output = Result<T, QueryError>> + Send + 'static,
    {
        self.use_query_async(app, window, key, policy, fetch)
    }

    pub fn refetch_async<H, T, Fut>(
        &mut self,
        app: &mut H,
        window: AppWindowId,
        key: QueryKey<T>,
        policy: QueryPolicy,
        fetch: impl FnOnce(CancellationToken) -> Fut + Send + 'static,
    ) -> QueryHandle<T>
    where
        H: UiHost,
        T: Any + Send + Sync + 'static,
        Fut: Future<Output = Result<T, QueryError>> + Send + 'static,
    {
        self.invalidate(app, key);
        self.use_query_async(app, window, key, policy, fetch)
    }

    pub fn prefetch_async_local<H, T, Fut>(
        &mut self,
        app: &mut H,
        window: AppWindowId,
        key: QueryKey<T>,
        policy: QueryPolicy,
        fetch: impl FnOnce(CancellationToken) -> Fut + 'static,
    ) -> QueryHandle<T>
    where
        H: UiHost,
        T: Any + Send + Sync + 'static,
        Fut: Future<Output = Result<T, QueryError>> + 'static,
    {
        self.use_query_async_local(app, window, key, policy, fetch)
    }

    pub fn refetch_async_local<H, T, Fut>(
        &mut self,
        app: &mut H,
        window: AppWindowId,
        key: QueryKey<T>,
        policy: QueryPolicy,
        fetch: impl FnOnce(CancellationToken) -> Fut + 'static,
    ) -> QueryHandle<T>
    where
        H: UiHost,
        T: Any + Send + Sync + 'static,
        Fut: Future<Output = Result<T, QueryError>> + 'static,
    {
        self.invalidate(app, key);
        self.use_query_async_local(app, window, key, policy, fetch)
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
        let had_inflight = entry.inflight.is_some();
        entry.stale = true;
        entry.retry.next_retry_at = None;
        if let Some(token) = entry.retry.scheduled_wake.take() {
            token.cancel();
        }
        if entry.policy.cancel_mode == QueryCancelMode::CancelInFlight {
            entry.inflight = None;
            if had_inflight {
                tracing::debug!(
                    target: "fret_query::diag",
                    namespace = entry.namespace,
                    hash = entry.hash,
                    debug_label = entry.debug_label,
                    type_name = entry.type_name,
                    model_id = ?entry.model_id,
                    cancel_mode = ?entry.policy.cancel_mode,
                    "query inflight cancelled by invalidate"
                );
            }
        }

        tracing::debug!(
            target: "fret_query::diag",
            namespace = entry.namespace,
            hash = entry.hash,
            debug_label = entry.debug_label,
            type_name = entry.type_name,
            model_id = ?entry.model_id,
            "query invalidated"
        );
    }

    pub fn invalidate_namespace(&mut self, namespace: &'static str) {
        let mut entries = self
            .runtime
            .entries
            .lock()
            .unwrap_or_else(|p| p.into_inner());
        let mut affected = 0usize;
        for entry in entries.values_mut() {
            if entry.namespace == namespace {
                let had_inflight = entry.inflight.is_some();
                entry.stale = true;
                entry.retry.next_retry_at = None;
                if let Some(token) = entry.retry.scheduled_wake.take() {
                    token.cancel();
                }
                if entry.policy.cancel_mode == QueryCancelMode::CancelInFlight {
                    entry.inflight = None;
                    if had_inflight {
                        tracing::debug!(
                            target: "fret_query::diag",
                            namespace = entry.namespace,
                            hash = entry.hash,
                            debug_label = entry.debug_label,
                            type_name = entry.type_name,
                            model_id = ?entry.model_id,
                            cancel_mode = ?entry.policy.cancel_mode,
                            "query inflight cancelled by namespace invalidation"
                        );
                    }
                }
                affected = affected.saturating_add(1);
            }
        }

        tracing::debug!(
            target: "fret_query::diag",
            namespace,
            affected,
            "query namespace invalidated"
        );
    }

    pub fn snapshot(&self) -> QueryClientSnapshot {
        let captured_at = Instant::now();

        let mut entries = {
            let runtime_entries = self
                .runtime
                .entries
                .lock()
                .unwrap_or_else(|p| p.into_inner());

            runtime_entries
                .values()
                .map(|entry| QuerySnapshotEntry {
                    namespace: entry.namespace,
                    hash: entry.hash,
                    debug_label: entry.debug_label,
                    type_name: entry.type_name,
                    model_id: entry.model_id,
                    policy: entry.policy.clone(),
                    stale: entry.stale,
                    status: entry.status,
                    inflight: entry.inflight.as_ref().map(|inflight| inflight.id),
                    last_used: entry.last_used,
                    last_observed_frame: entry.last_observed_frame,
                    updated_at: entry.updated_at,
                    last_duration: entry.last_duration,
                    retry: QueryRetryState {
                        failures: entry.retry.failures,
                        max_retries: entry.policy.retry.max_retries(),
                        next_retry_at: entry.retry.next_retry_at,
                    },
                    last_error_kind: entry.last_error_kind,
                    last_error_message: entry.last_error_message.clone(),
                })
                .collect::<Vec<_>>()
        };

        entries.sort_by_key(|entry| (entry.namespace, entry.hash, entry.type_name));

        QueryClientSnapshot {
            captured_at,
            entries,
        }
    }

    #[track_caller]
    fn touch_entry<T: Any + Send + Sync + 'static>(
        &mut self,
        key: QueryKey<T>,
        model_id: ModelId,
        policy: QueryPolicy,
        now: Instant,
        frame_id: FrameId,
    ) -> bool {
        let key_id = key.id();
        let mut entries = self
            .runtime
            .entries
            .lock()
            .unwrap_or_else(|p| p.into_inner());
        let entry = entries.entry(key_id).or_insert_with(|| QueryRuntimeEntry {
            type_id: TypeId::of::<T>(),
            type_name: type_name::<T>(),
            namespace: key.namespace,
            hash: key.hash,
            debug_label: key.debug_label,
            model_id,
            policy: policy.clone(),
            last_used: now,
            last_observed_frame: None,
            stale: true,
            status: QueryStatus::Idle,
            updated_at: None,
            last_duration: None,
            last_error_kind: None,
            last_error_message: None,
            inflight: None,
            retry: QueryRuntimeRetry::default(),
            apply: apply_query_result::<T>,
            apply_retry: apply_query_retry_state::<T>,
        });

        let remounted = match entry.last_observed_frame {
            None => true,
            Some(prev) => frame_id.0.saturating_sub(prev.0) > 1,
        };
        entry.last_observed_frame = Some(frame_id);

        if entry.type_id != TypeId::of::<T>() {
            tracing::error!(
                namespace = entry.namespace,
                hash = entry.hash,
                debug_label = entry.debug_label,
                stored = ?entry.type_id,
                requested = ?TypeId::of::<T>(),
                "query key type mismatch"
            );
            return remounted;
        }

        entry.model_id = model_id;
        entry.policy = policy;
        entry.last_used = now;
        entry.debug_label = key.debug_label.or(entry.debug_label);

        remounted
    }

    fn should_fetch<H: UiHost, T: Any + Send + Sync + 'static>(
        &self,
        app: &mut H,
        model: &Model<QueryState<T>>,
        key: QueryKeyId,
        now: Instant,
        remounted: bool,
    ) -> bool {
        let (
            policy,
            has_inflight,
            stale,
            stale_time,
            next_retry_at,
            namespace,
            hash,
            debug_label,
            type_name,
            model_id,
        ) = {
            let entries = self
                .runtime
                .entries
                .lock()
                .unwrap_or_else(|p| p.into_inner());
            let Some(entry) = entries.get(&key) else {
                return true;
            };
            (
                entry.policy.clone(),
                entry.inflight.is_some(),
                entry.stale,
                entry.policy.stale_time,
                entry.retry.next_retry_at,
                entry.namespace,
                entry.hash,
                entry.debug_label,
                entry.type_name,
                entry.model_id,
            )
        };

        let state = model.read_ref(app, |s| (*s).clone()).ok();
        let Some(state) = state else {
            return true;
        };

        if stale {
            if has_inflight {
                if policy.dedupe_inflight {
                    tracing::debug!(
                        target: "fret_query::diag",
                        namespace,
                        hash,
                        debug_label,
                        type_name,
                        model_id = ?model_id,
                        stale,
                        has_inflight,
                        dedupe_inflight = policy.dedupe_inflight,
                        cancel_mode = ?policy.cancel_mode,
                        decision = "cancel_or_wait_existing_inflight",
                        "query should_fetch decision"
                    );
                    return policy.cancel_mode == QueryCancelMode::CancelInFlight;
                }
                tracing::debug!(
                    target: "fret_query::diag",
                    namespace,
                    hash,
                    debug_label,
                    type_name,
                    model_id = ?model_id,
                    stale,
                    has_inflight,
                    dedupe_inflight = policy.dedupe_inflight,
                    decision = "start_new_fetch_with_parallel_inflight",
                    "query should_fetch decision"
                );
                return true;
            }
            tracing::debug!(
                target: "fret_query::diag",
                namespace,
                hash,
                debug_label,
                type_name,
                model_id = ?model_id,
                stale,
                has_inflight,
                decision = "start_fetch_stale",
                "query should_fetch decision"
            );
            return true;
        }

        if policy.dedupe_inflight && has_inflight {
            tracing::debug!(
                target: "fret_query::diag",
                namespace,
                hash,
                debug_label,
                type_name,
                model_id = ?model_id,
                stale,
                has_inflight,
                dedupe_inflight = policy.dedupe_inflight,
                decision = "skip_deduped_inflight",
                "query should_fetch decision"
            );
            return false;
        }

        if state.status == QueryStatus::Idle {
            tracing::debug!(
                target: "fret_query::diag",
                namespace,
                hash,
                debug_label,
                type_name,
                model_id = ?model_id,
                decision = "start_fetch_idle",
                "query should_fetch decision"
            );
            return true;
        }

        if state.status == QueryStatus::Error {
            let Some(next_retry_at) = next_retry_at else {
                tracing::debug!(
                    target: "fret_query::diag",
                    namespace,
                    hash,
                    debug_label,
                    type_name,
                    model_id = ?model_id,
                    status = ?state.status,
                    decision = "skip_error_no_retry",
                    "query should_fetch decision"
                );
                return false;
            };
            let should_retry = now >= next_retry_at;
            tracing::debug!(
                target: "fret_query::diag",
                namespace,
                hash,
                debug_label,
                type_name,
                model_id = ?model_id,
                status = ?state.status,
                next_retry_at = ?next_retry_at,
                should_retry,
                decision = if should_retry { "start_fetch_retry_due" } else { "skip_retry_not_due" },
                "query should_fetch decision"
            );
            return should_retry;
        }

        if state.status != QueryStatus::Success {
            tracing::debug!(
                target: "fret_query::diag",
                namespace,
                hash,
                debug_label,
                type_name,
                model_id = ?model_id,
                status = ?state.status,
                decision = "skip_non_terminal_status",
                "query should_fetch decision"
            );
            return false;
        }

        let Some(updated_at) = state.updated_at else {
            tracing::debug!(
                target: "fret_query::diag",
                namespace,
                hash,
                debug_label,
                type_name,
                model_id = ?model_id,
                status = ?state.status,
                decision = "start_missing_updated_at",
                "query should_fetch decision"
            );
            return true;
        };

        let stale_by_time = now.duration_since(updated_at) >= stale_time;
        if !stale_by_time {
            tracing::debug!(
                target: "fret_query::diag",
                namespace,
                hash,
                debug_label,
                type_name,
                model_id = ?model_id,
                status = ?state.status,
                remounted,
                stale_by_time,
                decision = "skip_fresh",
                "query should_fetch decision"
            );
            return false;
        }

        // Becoming stale by time does not automatically refetch: only refetch when an observer
        // is (re)attached after a gap (TanStack-like "mount + stale" behavior).
        tracing::debug!(
            target: "fret_query::diag",
            namespace,
            hash,
            debug_label,
            type_name,
            model_id = ?model_id,
            status = ?state.status,
            remounted,
            stale_by_time,
            decision = if remounted { "start_remount_stale" } else { "skip_stale_while_observed" },
            "query should_fetch decision"
        );

        remounted
    }

    fn start_fetch<H, T>(
        &mut self,
        app: &mut H,
        window: AppWindowId,
        key: QueryKey<T>,
        policy: QueryPolicy,
        model: Model<QueryState<T>>,
        fetch: impl FnOnce(CancellationToken) -> Result<T, QueryError> + Send + 'static,
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
                        inflight_id: Some(inflight_id),
                        apply: Some(QueryApplyMsg {
                            inflight_id,
                            finished_at,
                            duration,
                            result,
                        }),
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

        entry.retry.next_retry_at = None;
        if let Some(token) = entry.retry.scheduled_wake.take() {
            token.cancel();
        }

        if let Some(prev) = entry.inflight.take() {
            let Inflight { task, .. } = prev;
            match policy.cancel_mode {
                QueryCancelMode::CancelInFlight => {
                    tracing::debug!(
                        target: "fret_query::diag",
                        namespace = entry.namespace,
                        hash = entry.hash,
                        debug_label = entry.debug_label,
                        type_name = entry.type_name,
                        model_id = ?entry.model_id,
                        cancel_mode = ?policy.cancel_mode,
                        "query inflight cancelled by new fetch"
                    );
                    drop(task)
                }
                QueryCancelMode::KeepInFlight => task.detach(),
            }
        }

        entry.stale = false;
        entry.status = QueryStatus::Loading;
        entry.inflight = Some(Inflight {
            id: inflight_id,
            started_at,
            task,
        });

        tracing::debug!(
            target: "fret_query::diag",
            namespace = entry.namespace,
            hash = entry.hash,
            debug_label = entry.debug_label,
            type_name = entry.type_name,
            model_id = ?entry.model_id,
            inflight_id,
            "query fetch started"
        );

        let _ = app.models_mut().update(&model, |st| {
            st.status = QueryStatus::Loading;
            st.inflight = Some(inflight_id);
            st.error = None;
            st.retry.max_retries = policy.retry.max_retries();
            st.retry.next_retry_at = None;
            if !policy.keep_previous_data_while_loading {
                st.data = None;
            }
        });
    }

    fn start_fetch_async<H, T, Fut>(
        &mut self,
        app: &mut H,
        window: AppWindowId,
        key: QueryKey<T>,
        policy: QueryPolicy,
        model: Model<QueryState<T>>,
        spawner: &dyn FutureSpawner,
        fetch: impl FnOnce(CancellationToken) -> Fut + Send + 'static,
        now: Instant,
    ) where
        H: UiHost,
        T: Any + Send + Sync + 'static,
        Fut: Future<Output = Result<T, QueryError>> + Send + 'static,
    {
        let key_id = key.id();
        let model_id = model.id();
        let inflight_id = self.runtime.next_inflight_id();
        let sender = self.runtime.inbox.sender();
        let started_at = now;

        let task = self.runtime.exec.spawn_future_to_inbox(
            spawner,
            Some(window),
            sender,
            move |token| async move {
                let result = fetch(token).await;

                let finished_at = Instant::now();
                let duration = finished_at.duration_since(started_at);
                let result = match result {
                    Ok(value) => Ok(Box::new(value) as Box<dyn Any + Send>),
                    Err(err) => Err(err),
                };

                QueryInboxMsg {
                    window: Some(window),
                    key: key_id,
                    model_id,
                    inflight_id: Some(inflight_id),
                    apply: Some(QueryApplyMsg {
                        inflight_id,
                        finished_at,
                        duration,
                        result,
                    }),
                }
            },
        );

        let mut entries = self
            .runtime
            .entries
            .lock()
            .unwrap_or_else(|p| p.into_inner());
        let Some(entry) = entries.get_mut(&key_id) else {
            return;
        };

        entry.retry.next_retry_at = None;
        if let Some(token) = entry.retry.scheduled_wake.take() {
            token.cancel();
        }

        if let Some(prev) = entry.inflight.take() {
            let Inflight { task, .. } = prev;
            match policy.cancel_mode {
                QueryCancelMode::CancelInFlight => {
                    tracing::debug!(
                        target: "fret_query::diag",
                        namespace = entry.namespace,
                        hash = entry.hash,
                        debug_label = entry.debug_label,
                        type_name = entry.type_name,
                        model_id = ?entry.model_id,
                        cancel_mode = ?policy.cancel_mode,
                        "query inflight cancelled by new fetch"
                    );
                    drop(task)
                }
                QueryCancelMode::KeepInFlight => task.detach(),
            }
        }

        entry.stale = false;
        entry.status = QueryStatus::Loading;
        entry.inflight = Some(Inflight {
            id: inflight_id,
            started_at,
            task,
        });

        tracing::debug!(
            target: "fret_query::diag",
            namespace = entry.namespace,
            hash = entry.hash,
            debug_label = entry.debug_label,
            type_name = entry.type_name,
            model_id = ?entry.model_id,
            inflight_id,
            "query fetch started"
        );

        let _ = app.models_mut().update(&model, |st| {
            st.status = QueryStatus::Loading;
            st.inflight = Some(inflight_id);
            st.error = None;
            st.retry.max_retries = policy.retry.max_retries();
            st.retry.next_retry_at = None;
            if !policy.keep_previous_data_while_loading {
                st.data = None;
            }
        });
    }

    fn start_fetch_async_local<H, T, Fut>(
        &mut self,
        app: &mut H,
        window: AppWindowId,
        key: QueryKey<T>,
        policy: QueryPolicy,
        model: Model<QueryState<T>>,
        spawner: &dyn FutureSpawner,
        fetch: impl FnOnce(CancellationToken) -> Fut + 'static,
        now: Instant,
    ) where
        H: UiHost,
        T: Any + Send + Sync + 'static,
        Fut: Future<Output = Result<T, QueryError>> + 'static,
    {
        let key_id = key.id();
        let model_id = model.id();
        let inflight_id = self.runtime.next_inflight_id();
        let sender = self.runtime.inbox.sender();
        let started_at = now;

        let Some(task) = self.runtime.exec.spawn_local_future_to_inbox(
            spawner,
            Some(window),
            sender,
            move |token| async move {
                let result = fetch(token).await;

                let finished_at = Instant::now();
                let duration = finished_at.duration_since(started_at);
                let result = match result {
                    Ok(value) => Ok(Box::new(value) as Box<dyn Any + Send>),
                    Err(err) => Err(err),
                };

                QueryInboxMsg {
                    window: Some(window),
                    key: key_id,
                    model_id,
                    inflight_id: Some(inflight_id),
                    apply: Some(QueryApplyMsg {
                        inflight_id,
                        finished_at,
                        duration,
                        result,
                    }),
                }
            },
        ) else {
            let _ = app.models_mut().update(&model, |st| {
                st.status = QueryStatus::Error;
                st.error = Some(QueryError::permanent(
                    "FutureSpawner does not support local futures (use a wasm/local spawner or use_query_async)",
                ));
            });

            let mut entries = self
                .runtime
                .entries
                .lock()
                .unwrap_or_else(|p| p.into_inner());
            if let Some(entry) = entries.get_mut(&key_id) {
                entry.status = QueryStatus::Error;
                entry.last_error_kind = Some(QueryErrorKind::Permanent);
                entry.last_error_message = Some(Arc::from(
                    "FutureSpawner does not support local futures (use a wasm/local spawner or use_query_async)",
                ));
                entry.updated_at = Some(now);
                entry.last_duration = None;
            }

            tracing::debug!(
                target: "fret_query::diag",
                namespace = key.namespace(),
                hash = key.hash(),
                debug_label = key.debug_label(),
                type_name = type_name::<T>(),
                model_id = ?model_id,
                "query fetch failed to start: local future spawner unsupported"
            );
            return;
        };

        let mut entries = self
            .runtime
            .entries
            .lock()
            .unwrap_or_else(|p| p.into_inner());
        let Some(entry) = entries.get_mut(&key_id) else {
            return;
        };

        entry.retry.next_retry_at = None;
        if let Some(token) = entry.retry.scheduled_wake.take() {
            token.cancel();
        }

        if let Some(prev) = entry.inflight.take() {
            let Inflight { task, .. } = prev;
            match policy.cancel_mode {
                QueryCancelMode::CancelInFlight => {
                    tracing::debug!(
                        target: "fret_query::diag",
                        namespace = entry.namespace,
                        hash = entry.hash,
                        debug_label = entry.debug_label,
                        type_name = entry.type_name,
                        model_id = ?entry.model_id,
                        cancel_mode = ?policy.cancel_mode,
                        "query inflight cancelled by new fetch"
                    );
                    drop(task)
                }
                QueryCancelMode::KeepInFlight => task.detach(),
            }
        }

        entry.stale = false;
        entry.status = QueryStatus::Loading;
        entry.inflight = Some(Inflight {
            id: inflight_id,
            started_at,
            task,
        });

        tracing::debug!(
            target: "fret_query::diag",
            namespace = entry.namespace,
            hash = entry.hash,
            debug_label = entry.debug_label,
            type_name = entry.type_name,
            model_id = ?entry.model_id,
            inflight_id,
            "query fetch started"
        );

        let _ = app.models_mut().update(&model, |st| {
            st.status = QueryStatus::Loading;
            st.inflight = Some(inflight_id);
            st.error = None;
            st.retry.max_retries = policy.retry.max_retries();
            st.retry.next_retry_at = None;
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
                if let Some(entry) = entries.get(key) {
                    tracing::debug!(
                        target: "fret_query::diag",
                        namespace = entry.namespace,
                        hash = entry.hash,
                        debug_label = entry.debug_label,
                        type_name = entry.type_name,
                        model_id = ?entry.model_id,
                        "query cache evicted by gc"
                    );
                }
                entries.remove(key);
            }
        }

        for key in evict {
            self.handles.remove(&key);
        }
    }
}

fn apply_query_result<T: Any + Send + Sync + 'static>(
    state_any: &mut dyn Any,
    msg: QueryApplyMsg,
) -> bool {
    let state = state_any
        .downcast_mut::<QueryState<T>>()
        .expect("query model type mismatch");

    if state.inflight != Some(msg.inflight_id) {
        return false;
    }

    state.inflight = None;
    state.last_duration = Some(msg.duration);

    match msg.result {
        Ok(value_any) => {
            let Ok(value) = value_any.downcast::<T>() else {
                state.status = QueryStatus::Error;
                state.error = Some(QueryError::permanent("query result type mismatch"));
                return true;
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

    true
}

fn apply_query_retry_state<T: Any + Send + Sync + 'static>(
    state_any: &mut dyn Any,
    retry: QueryRetryState,
) {
    let state = state_any
        .downcast_mut::<QueryState<T>>()
        .expect("query model type mismatch");
    state.retry = retry;
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
    use std::future::Future;

    use fret_ui::{ElementContext, UiHost};

    pub trait QueryElementContextExt {
        fn use_query<T: Any + Send + Sync + 'static>(
            &mut self,
            key: QueryKey<T>,
            policy: QueryPolicy,
            fetch: impl FnOnce(CancellationToken) -> Result<T, QueryError> + Send + 'static,
        ) -> QueryHandle<T>;

        fn use_query_async<T, Fut>(
            &mut self,
            key: QueryKey<T>,
            policy: QueryPolicy,
            fetch: impl FnOnce(CancellationToken) -> Fut + Send + 'static,
        ) -> QueryHandle<T>
        where
            T: Any + Send + Sync + 'static,
            Fut: Future<Output = Result<T, QueryError>> + Send + 'static;

        fn use_query_async_local<T, Fut>(
            &mut self,
            key: QueryKey<T>,
            policy: QueryPolicy,
            fetch: impl FnOnce(CancellationToken) -> Fut + 'static,
        ) -> QueryHandle<T>
        where
            T: Any + Send + Sync + 'static,
            Fut: Future<Output = Result<T, QueryError>> + 'static;
    }

    impl<'a, H: UiHost> QueryElementContextExt for ElementContext<'a, H> {
        fn use_query<T: Any + Send + Sync + 'static>(
            &mut self,
            key: QueryKey<T>,
            policy: QueryPolicy,
            fetch: impl FnOnce(CancellationToken) -> Result<T, QueryError> + Send + 'static,
        ) -> QueryHandle<T> {
            let window = self.window;
            with_query_client(self.app, |client, app| {
                client.use_query(app, window, key, policy, fetch)
            })
            .unwrap_or_else(|| {
                let model = self.app.models_mut().insert(QueryState::<T> {
                    status: QueryStatus::Error,
                    data: None,
                    error: Some(QueryError::permanent("missing DispatcherHandle global")),
                    inflight: None,
                    updated_at: None,
                    last_duration: None,
                    retry: QueryRetryState::default(),
                });
                QueryHandle { key, model }
            })
        }

        fn use_query_async<T, Fut>(
            &mut self,
            key: QueryKey<T>,
            policy: QueryPolicy,
            fetch: impl FnOnce(CancellationToken) -> Fut + Send + 'static,
        ) -> QueryHandle<T>
        where
            T: Any + Send + Sync + 'static,
            Fut: Future<Output = Result<T, QueryError>> + Send + 'static,
        {
            let window = self.window;
            with_query_client(self.app, |client, app| {
                client.use_query_async(app, window, key, policy, fetch)
            })
            .unwrap_or_else(|| {
                let model = self.app.models_mut().insert(QueryState::<T> {
                    status: QueryStatus::Error,
                    data: None,
                    error: Some(QueryError::permanent("missing DispatcherHandle global")),
                    inflight: None,
                    updated_at: None,
                    last_duration: None,
                    retry: QueryRetryState::default(),
                });
                QueryHandle { key, model }
            })
        }

        fn use_query_async_local<T, Fut>(
            &mut self,
            key: QueryKey<T>,
            policy: QueryPolicy,
            fetch: impl FnOnce(CancellationToken) -> Fut + 'static,
        ) -> QueryHandle<T>
        where
            T: Any + Send + Sync + 'static,
            Fut: Future<Output = Result<T, QueryError>> + 'static,
        {
            let window = self.window;
            with_query_client(self.app, |client, app| {
                client.use_query_async_local(app, window, key, policy, fetch)
            })
            .unwrap_or_else(|| {
                let model = self.app.models_mut().insert(QueryState::<T> {
                    status: QueryStatus::Error,
                    data: None,
                    error: Some(QueryError::permanent("missing DispatcherHandle global")),
                    inflight: None,
                    updated_at: None,
                    last_duration: None,
                    retry: QueryRetryState::default(),
                });
                QueryHandle { key, model }
            })
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::future::Future;
    use std::pin::Pin;

    use fret_app::App;
    use fret_runtime::{DispatchPriority, Dispatcher, ExecCapabilities, Runnable};
    use pollster::block_on;

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

        fn take_main(&self) -> Vec<Runnable> {
            let mut guard = self.main.lock().unwrap();
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

    #[derive(Default)]
    struct TestSpawner {
        send: Mutex<Vec<Pin<Box<dyn Future<Output = ()> + Send + 'static>>>>,
    }

    impl TestSpawner {
        fn drain_send(&self) -> Vec<Pin<Box<dyn Future<Output = ()> + Send + 'static>>> {
            let mut guard = self.send.lock().unwrap();
            std::mem::take(&mut *guard)
        }
    }

    impl FutureSpawner for TestSpawner {
        fn spawn_send(&self, fut: Pin<Box<dyn Future<Output = ()> + Send + 'static>>) {
            self.send.lock().unwrap().push(fut);
        }

        fn spawn_local(&self, fut: Pin<Box<dyn Future<Output = ()> + 'static>>) -> bool {
            block_on(fut);
            true
        }
    }

    #[test]
    fn dedupes_inflight_by_default() {
        let mut app = App::new();
        let dispatcher = Arc::new(TestDispatcher::default());
        let dispatcher_handle: DispatcherHandle = dispatcher.clone();
        app.set_global::<DispatcherHandle>(dispatcher_handle);

        let window = AppWindowId::default();
        let key = QueryKey::<u32>::new("test.query.v1", &0u32);

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
        let key = QueryKey::<u32>::new("test.query.v1", &123u32);

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
    fn use_query_async_fetches_and_applies() {
        let mut app = App::new();
        let dispatcher = Arc::new(TestDispatcher::default());
        let dispatcher_handle: DispatcherHandle = dispatcher.clone();
        app.set_global::<DispatcherHandle>(dispatcher_handle);

        let spawner = Arc::new(TestSpawner::default());
        let spawner_handle: FutureSpawnerHandle = spawner.clone();
        app.set_global::<FutureSpawnerHandle>(spawner_handle);

        let window = AppWindowId::default();
        let key = QueryKey::<u32>::new("test.query.v1", &321u32);

        let handle = with_query_client(&mut app, |client, app| {
            client.use_query_async(
                app,
                window,
                key,
                QueryPolicy::default(),
                |_token| async move { Ok(123u32) },
            )
        })
        .unwrap();

        for fut in spawner.drain_send() {
            block_on(fut);
        }
        assert!(drain_inboxes(&mut app, Some(window)));

        let state = handle.model.read_ref(&app, |s| s.clone()).unwrap();
        assert_eq!(state.status, QueryStatus::Success);
        assert_eq!(state.data.as_deref().copied(), Some(123));
    }

    #[test]
    fn use_query_async_local_fetches_and_applies() {
        let mut app = App::new();
        let dispatcher = Arc::new(TestDispatcher::default());
        let dispatcher_handle: DispatcherHandle = dispatcher.clone();
        app.set_global::<DispatcherHandle>(dispatcher_handle);

        let spawner = Arc::new(TestSpawner::default());
        let spawner_handle: FutureSpawnerHandle = spawner.clone();
        app.set_global::<FutureSpawnerHandle>(spawner_handle);

        let window = AppWindowId::default();
        let key = QueryKey::<u32>::new("test.query.v1", &654u32);

        let handle = with_query_client(&mut app, |client, app| {
            client.use_query_async_local(app, window, key, QueryPolicy::default(), |_token| async {
                Ok(456u32)
            })
        })
        .unwrap();

        assert!(drain_inboxes(&mut app, Some(window)));

        let state = handle.model.read_ref(&app, |s| s.clone()).unwrap();
        assert_eq!(state.status, QueryStatus::Success);
        assert_eq!(state.data.as_deref().copied(), Some(456));
    }

    #[test]
    fn refetch_forces_fetch_when_fresh() {
        let mut app = App::new();
        let dispatcher = Arc::new(TestDispatcher::default());
        let dispatcher_handle: DispatcherHandle = dispatcher.clone();
        app.set_global::<DispatcherHandle>(dispatcher_handle);

        let window = AppWindowId::default();
        let key = QueryKey::<u32>::new("test.query.v1", &999u32);
        let policy = QueryPolicy {
            stale_time: Duration::from_secs(60),
            ..Default::default()
        };

        let handle = with_query_client(&mut app, |client, app| {
            client.use_query(app, window, key, policy.clone(), |_token| Ok(1u32))
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
        assert_eq!(state.data.as_deref().copied(), Some(1));

        with_query_client(&mut app, |client, app| {
            let _ = client.refetch(app, window, key, policy.clone(), |_token| Ok(2u32));
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
        assert_eq!(state.data.as_deref().copied(), Some(2));
    }

    #[test]
    fn stale_time_does_not_refetch_while_continuously_observed() {
        let mut app = App::new();
        let dispatcher = Arc::new(TestDispatcher::default());
        let dispatcher_handle: DispatcherHandle = dispatcher.clone();
        app.set_global::<DispatcherHandle>(dispatcher_handle);

        let window = AppWindowId::default();
        let key = QueryKey::<u32>::new("test.query.v1", &111u32);
        let policy = QueryPolicy {
            stale_time: Duration::ZERO,
            ..Default::default()
        };

        app.set_frame_id(FrameId(1));
        let handle = with_query_client(&mut app, |client, app| {
            client.use_query(app, window, key, policy.clone(), |_token| Ok(1u32))
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

        app.set_frame_id(FrameId(2));
        with_query_client(&mut app, |client, app| {
            let _ = client.use_query(app, window, key, policy.clone(), |_token| Ok(2u32));
        })
        .unwrap();

        assert_eq!(dispatcher.take_background().len(), 0);
    }

    #[test]
    fn stale_time_refetches_on_remount_after_frame_gap() {
        let mut app = App::new();
        let dispatcher = Arc::new(TestDispatcher::default());
        let dispatcher_handle: DispatcherHandle = dispatcher.clone();
        app.set_global::<DispatcherHandle>(dispatcher_handle);

        let window = AppWindowId::default();
        let key = QueryKey::<u32>::new("test.query.v1", &222u32);
        let policy = QueryPolicy {
            stale_time: Duration::ZERO,
            ..Default::default()
        };

        app.set_frame_id(FrameId(1));
        let _handle = with_query_client(&mut app, |client, app| {
            client.use_query(app, window, key, policy.clone(), |_token| Ok(1u32))
        })
        .unwrap();

        let tasks = dispatcher.take_background();
        assert_eq!(tasks.len(), 1);
        for task in tasks {
            task();
        }
        assert!(drain_inboxes(&mut app, Some(window)));

        app.set_frame_id(FrameId(3));
        with_query_client(&mut app, |client, app| {
            let _ = client.use_query(app, window, key, policy.clone(), |_token| Ok(2u32));
        })
        .unwrap();

        assert_eq!(dispatcher.take_background().len(), 1);
    }

    #[test]
    fn remount_does_not_refetch_when_data_is_still_fresh() {
        let mut app = App::new();
        let dispatcher = Arc::new(TestDispatcher::default());
        let dispatcher_handle: DispatcherHandle = dispatcher.clone();
        app.set_global::<DispatcherHandle>(dispatcher_handle);

        let window = AppWindowId::default();
        let key = QueryKey::<u32>::new("test.query.v1", &333u32);
        let policy = QueryPolicy {
            stale_time: Duration::from_secs(60),
            ..Default::default()
        };

        app.set_frame_id(FrameId(1));
        let _handle = with_query_client(&mut app, |client, app| {
            client.use_query(app, window, key, policy.clone(), |_token| Ok(1u32))
        })
        .unwrap();

        let tasks = dispatcher.take_background();
        assert_eq!(tasks.len(), 1);
        for task in tasks {
            task();
        }
        assert!(drain_inboxes(&mut app, Some(window)));

        app.set_frame_id(FrameId(3));
        with_query_client(&mut app, |client, app| {
            let _ = client.use_query(app, window, key, policy.clone(), |_token| Ok(2u32));
        })
        .unwrap();

        assert_eq!(dispatcher.take_background().len(), 0);
    }

    #[test]
    fn remount_does_not_refetch_error_without_retry_policy() {
        let mut app = App::new();
        let dispatcher = Arc::new(TestDispatcher::default());
        let dispatcher_handle: DispatcherHandle = dispatcher.clone();
        app.set_global::<DispatcherHandle>(dispatcher_handle);

        let window = AppWindowId::default();
        let key = QueryKey::<u32>::new("test.query.v1", &444u32);
        let policy = QueryPolicy {
            stale_time: Duration::from_secs(60),
            retry: QueryRetryPolicy::none(),
            ..Default::default()
        };

        app.set_frame_id(FrameId(1));
        let handle = with_query_client(&mut app, |client, app| {
            client.use_query(app, window, key, policy.clone(), |_token| {
                Err(QueryError::permanent("boom"))
            })
        })
        .unwrap();

        let tasks = dispatcher.take_background();
        assert_eq!(tasks.len(), 1);
        for task in tasks {
            task();
        }
        assert!(drain_inboxes(&mut app, Some(window)));

        let state = handle.model.read_ref(&app, |s| s.clone()).unwrap();
        assert_eq!(state.status, QueryStatus::Error);
        assert!(state.retry.next_retry_at.is_none());

        app.set_frame_id(FrameId(3));
        with_query_client(&mut app, |client, app| {
            let _ = client.use_query(app, window, key, policy.clone(), |_token| Ok(123u32));
        })
        .unwrap();

        assert_eq!(dispatcher.take_background().len(), 0);
    }

    #[test]
    fn gc_drops_cached_models() {
        let mut app = App::new();
        let dispatcher = Arc::new(TestDispatcher::default());
        let dispatcher_handle: DispatcherHandle = dispatcher.clone();
        app.set_global::<DispatcherHandle>(dispatcher_handle);

        let window = AppWindowId::default();
        let key = QueryKey::<u32>::new("test.query.v1", &42u32);

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

    #[test]
    fn query_key_debug_label_does_not_affect_identity() {
        let a = QueryKey::<u32>::new("test.query.v1", &123u32);
        let b = QueryKey::<u32>::new_named("test.query.v1", &123u32, "debug label");
        assert_eq!(a, b);
        assert_eq!(a.id(), b.id());
    }

    #[test]
    fn snapshot_reports_query_runtime_metadata() {
        let mut app = App::new();
        let dispatcher = Arc::new(TestDispatcher::default());
        let dispatcher_handle: DispatcherHandle = dispatcher.clone();
        app.set_global::<DispatcherHandle>(dispatcher_handle);

        let window = AppWindowId::default();
        let key = QueryKey::<u32>::new_named("test.query.v1", &900u32, "snapshot_meta");
        let policy = QueryPolicy {
            stale_time: Duration::from_secs(60),
            retry: QueryRetryPolicy::fixed(1, Duration::from_millis(10)),
            ..Default::default()
        };

        with_query_client(&mut app, |client, app| {
            let _ = client.use_query(app, window, key, policy.clone(), |_token| {
                Err(QueryError::transient("boom"))
            });
        })
        .unwrap();

        let tasks = dispatcher.take_background();
        assert_eq!(tasks.len(), 1);
        for task in tasks {
            task();
        }
        assert!(drain_inboxes(&mut app, Some(window)));

        let snapshot = with_query_client(&mut app, |client, _app| client.snapshot()).unwrap();
        let entry = snapshot
            .entries
            .iter()
            .find(|entry| entry.namespace == "test.query.v1" && entry.hash == key.hash())
            .expect("snapshot should contain test query entry");

        assert_eq!(entry.status, QueryStatus::Error);
        assert_eq!(entry.last_error_kind, Some(QueryErrorKind::Transient));
        assert_eq!(entry.last_error_message.as_deref(), Some("boom"));
        assert!(entry.last_duration.is_some());
        assert_eq!(entry.retry.failures, 1);
        assert_eq!(entry.retry.max_retries, 1);
        assert_eq!(entry.debug_label, Some("snapshot_meta"));
        assert!(entry.type_name.contains("u32"));
    }

    #[test]
    fn snapshot_excludes_evicted_entries_after_gc() {
        let mut app = App::new();
        let dispatcher = Arc::new(TestDispatcher::default());
        let dispatcher_handle: DispatcherHandle = dispatcher.clone();
        app.set_global::<DispatcherHandle>(dispatcher_handle);

        let window = AppWindowId::default();
        let key = QueryKey::<u32>::new("test.query.v1", &901u32);

        with_query_client(&mut app, |client, app| {
            let policy = QueryPolicy {
                cache_time: Duration::ZERO,
                ..Default::default()
            };
            let _ = client.use_query(app, window, key, policy, |_token| Ok(1u32));
        })
        .unwrap();

        let tasks = dispatcher.take_background();
        assert_eq!(tasks.len(), 1);
        for task in tasks {
            task();
        }
        assert!(drain_inboxes(&mut app, Some(window)));

        let snapshot_before =
            with_query_client(&mut app, |client, _app| client.snapshot()).unwrap();
        assert!(
            snapshot_before
                .entries
                .iter()
                .any(|entry| entry.namespace == "test.query.v1" && entry.hash == key.hash())
        );

        with_query_client(&mut app, |client, _app| {
            client.last_gc_at = None;
            client.gc(Instant::now());
        })
        .unwrap();

        let snapshot_after = with_query_client(&mut app, |client, _app| client.snapshot()).unwrap();
        assert!(
            !snapshot_after
                .entries
                .iter()
                .any(|entry| entry.namespace == "test.query.v1" && entry.hash == key.hash())
        );
    }

    #[test]
    fn retry_fixed_zero_delay_allows_followup_fetch_after_error() {
        let mut app = App::new();
        let dispatcher = Arc::new(TestDispatcher::default());
        let dispatcher_handle: DispatcherHandle = dispatcher.clone();
        app.set_global::<DispatcherHandle>(dispatcher_handle);

        let window = AppWindowId::default();
        let key = QueryKey::<u32>::new("test.query.v1", &777u32);
        let policy = QueryPolicy {
            stale_time: Duration::from_secs(60),
            retry: QueryRetryPolicy::fixed(1, Duration::ZERO),
            ..Default::default()
        };

        let handle = with_query_client(&mut app, |client, app| {
            client.use_query(app, window, key, policy.clone(), |_token| {
                Err(QueryError::transient("boom"))
            })
        })
        .unwrap();

        let tasks = dispatcher.take_background();
        assert_eq!(tasks.len(), 1);
        for task in tasks {
            task();
        }
        assert!(drain_inboxes(&mut app, Some(window)));

        let main = dispatcher.take_main();
        assert_eq!(main.len(), 1);
        for task in main {
            task();
        }
        assert!(drain_inboxes(&mut app, Some(window)));

        let state = handle.model.read_ref(&app, |s| s.clone()).unwrap();
        assert_eq!(state.status, QueryStatus::Error);
        assert_eq!(state.retry.failures, 1);
        assert_eq!(state.retry.max_retries, 1);
        assert!(state.retry.next_retry_at.is_some());

        with_query_client(&mut app, |client, app| {
            let _ = client.use_query(app, window, key, policy.clone(), |_token| Ok(123u32));
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
        assert_eq!(state.data.as_deref().copied(), Some(123));
        assert_eq!(state.retry.failures, 0);
        assert!(state.retry.next_retry_at.is_none());
    }
}
