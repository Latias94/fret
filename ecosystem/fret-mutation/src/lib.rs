//! Explicit async mutation/submission helpers (ecosystem-level).
//!
//! This crate complements `fret-query`:
//! - `fret-query` owns observed async reads with cache/freshness/invalidation semantics,
//! - `fret-mutation` owns explicit submit-like async work that must not start from render
//!   observation.
//!
//! The core contract is:
//! - mutation state is stored in a `Model<MutationState<TIn, TOut>>`,
//! - creating/reading a handle does not start work,
//! - only `submit(...)` starts work,
//! - background completion crosses the driver boundary via `InboxDrainRegistry`,
//! - and terminal state remains visible until explicit reset or a newer submit.

use std::any::Any;
use std::collections::HashMap;
use std::fmt;
use std::future::Future;
use std::marker::PhantomData;
use std::num::NonZeroU64;
use std::pin::Pin;
use std::rc::Rc;
use std::sync::atomic::{AtomicU64, Ordering};
use std::sync::{Arc, Mutex, Weak};
use std::time::Duration;

use fret_core::AppWindowId;
use fret_core::time::Instant;
#[cfg(feature = "tokio")]
pub use fret_executor::TokioSpawner;
#[cfg(all(feature = "wasm", target_arch = "wasm32"))]
pub use fret_executor::WasmSpawner;
use fret_executor::{BackgroundTask, Executors, Inbox, InboxDrainer};
pub use fret_executor::{CancellationToken, FutureSpawner, FutureSpawnerHandle};
use fret_runtime::{
    DispatcherHandle, InboxDrainHost, InboxDrainRegistry, Model, ModelId, ModelStore, UiHost,
    WeakModel,
};

type SendMutationFuture<T> =
    Pin<Box<dyn Future<Output = Result<T, MutationError>> + Send + 'static>>;
type LocalMutationFuture<T> = Pin<Box<dyn Future<Output = Result<T, MutationError>> + 'static>>;
type SendSubmitFn<TIn, TOut> =
    dyn Fn(CancellationToken, Arc<TIn>) -> SendMutationFuture<TOut> + Send + Sync;
type LocalSubmitFn<TIn, TOut> =
    dyn Fn(CancellationToken, Arc<TIn>) -> LocalMutationFuture<TOut> + 'static;

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum MutationStatus {
    Idle,
    Running,
    Success,
    Error,
}

impl MutationStatus {
    pub const fn as_str(self) -> &'static str {
        match self {
            Self::Idle => "Idle",
            Self::Running => "Running",
            Self::Success => "Success",
            Self::Error => "Error",
        }
    }

    pub const fn is_idle(self) -> bool {
        matches!(self, Self::Idle)
    }

    pub const fn is_running(self) -> bool {
        matches!(self, Self::Running)
    }

    pub const fn is_success(self) -> bool {
        matches!(self, Self::Success)
    }

    pub const fn is_error(self) -> bool {
        matches!(self, Self::Error)
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
#[non_exhaustive]
pub enum MutationErrorKind {
    Transient,
    Permanent,
}

#[derive(Debug, Clone)]
pub struct MutationError {
    kind: MutationErrorKind,
    message: Arc<str>,
}

impl MutationError {
    pub fn new(kind: MutationErrorKind, message: impl Into<Arc<str>>) -> Self {
        Self {
            kind,
            message: message.into(),
        }
    }

    pub fn transient(message: impl Into<Arc<str>>) -> Self {
        Self::new(MutationErrorKind::Transient, message)
    }

    pub fn permanent(message: impl Into<Arc<str>>) -> Self {
        Self::new(MutationErrorKind::Permanent, message)
    }

    pub fn kind(&self) -> MutationErrorKind {
        self.kind
    }

    pub fn message(&self) -> &Arc<str> {
        &self.message
    }
}

impl fmt::Display for MutationError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        self.message.fmt(f)
    }
}

impl std::error::Error for MutationError {}

impl From<Arc<str>> for MutationError {
    fn from(value: Arc<str>) -> Self {
        Self::permanent(value)
    }
}

impl From<String> for MutationError {
    fn from(value: String) -> Self {
        Self::permanent(Arc::<str>::from(value))
    }
}

impl From<&'static str> for MutationError {
    fn from(value: &'static str) -> Self {
        Self::permanent(Arc::<str>::from(value))
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum MutationConcurrencyPolicy {
    CancelInFlight,
    DropNewWhileRunning,
    AllowParallelLatestWins,
}

#[derive(Debug, Clone)]
pub struct MutationPolicy {
    pub concurrency: MutationConcurrencyPolicy,
    pub keep_previous_data_while_running: bool,
}

impl Default for MutationPolicy {
    fn default() -> Self {
        Self {
            concurrency: MutationConcurrencyPolicy::CancelInFlight,
            keep_previous_data_while_running: true,
        }
    }
}

#[derive(Debug)]
pub struct MutationState<TIn, TOut> {
    pub status: MutationStatus,
    pub input: Option<Arc<TIn>>,
    pub data: Option<Arc<TOut>>,
    pub error: Option<MutationError>,
    pub inflight: Option<u64>,
    pub updated_at: Option<Instant>,
    pub last_duration: Option<Duration>,
}

impl<TIn, TOut> Clone for MutationState<TIn, TOut> {
    fn clone(&self) -> Self {
        Self {
            status: self.status,
            input: self.input.clone(),
            data: self.data.clone(),
            error: self.error.clone(),
            inflight: self.inflight,
            updated_at: self.updated_at,
            last_duration: self.last_duration,
        }
    }
}

impl<TIn, TOut> Default for MutationState<TIn, TOut> {
    fn default() -> Self {
        Self {
            status: MutationStatus::Idle,
            input: None,
            data: None,
            error: None,
            inflight: None,
            updated_at: None,
            last_duration: None,
        }
    }
}

impl<TIn, TOut> MutationState<TIn, TOut> {
    pub fn is_idle(&self) -> bool {
        self.status.is_idle()
    }

    pub fn is_running(&self) -> bool {
        self.status.is_running()
    }

    pub fn is_success(&self) -> bool {
        self.status.is_success()
    }

    pub fn is_error(&self) -> bool {
        self.status.is_error()
    }

    pub fn has_data(&self) -> bool {
        self.data.is_some()
    }

    pub fn has_error(&self) -> bool {
        self.error.is_some()
    }
}

#[derive(Clone)]
enum MutationRuntimeHandle<TIn: 'static, TOut: 'static> {
    Ready(Arc<MutationRuntime<TIn, TOut>>),
    MissingDispatcher(Arc<str>),
}

enum MutationSubmitter<TIn: 'static, TOut: 'static> {
    Send(Arc<SendSubmitFn<TIn, TOut>>),
    Local(Rc<LocalSubmitFn<TIn, TOut>>),
}

impl<TIn: 'static, TOut: 'static> Clone for MutationSubmitter<TIn, TOut> {
    fn clone(&self) -> Self {
        match self {
            Self::Send(f) => Self::Send(f.clone()),
            Self::Local(f) => Self::Local(f.clone()),
        }
    }
}

#[derive(Clone)]
pub struct MutationHandle<TIn: 'static, TOut: 'static> {
    model: Model<MutationState<TIn, TOut>>,
    runtime: MutationRuntimeHandle<TIn, TOut>,
    spawner: Option<FutureSpawnerHandle>,
    policy: MutationPolicy,
    submitter: MutationSubmitter<TIn, TOut>,
}

impl<TIn: 'static, TOut: 'static> MutationHandle<TIn, TOut> {
    pub fn model(&self) -> &Model<MutationState<TIn, TOut>> {
        &self.model
    }

    /// Returns an opaque token for the current successful completion, if the handle is presently
    /// showing a success terminal state.
    ///
    /// Treat this as a compare-only token for once-per-success follow-up work. It is cleared when
    /// a new submit starts or when the mutation leaves the success state.
    pub fn success_token(&self) -> Option<NonZeroU64> {
        match &self.runtime {
            MutationRuntimeHandle::Ready(runtime) => {
                NonZeroU64::new(runtime.success_token.load(Ordering::SeqCst))
            }
            MutationRuntimeHandle::MissingDispatcher(_) => None,
        }
    }

    pub fn submit(&self, models: &mut ModelStore, window: AppWindowId, input: TIn) -> bool
    where
        TIn: Any + Send + Sync + 'static,
        TOut: Any + Send + Sync + 'static,
    {
        let input = Arc::new(input);
        let Some(spawner) = self.spawner.as_deref() else {
            self.set_submit_error(
                models,
                Some(input),
                MutationError::permanent(
                    "missing FutureSpawnerHandle global (install an async spawner to use mutation_async)",
                ),
            );
            return true;
        };

        match (&self.runtime, &self.submitter) {
            (MutationRuntimeHandle::Ready(runtime), MutationSubmitter::Send(submitter)) => runtime
                .submit_send(
                    models,
                    window,
                    input,
                    &self.policy,
                    spawner,
                    submitter.clone(),
                )
                .unwrap_or_else(|err| {
                    self.set_submit_error(models, None, err);
                    true
                }),
            (MutationRuntimeHandle::Ready(runtime), MutationSubmitter::Local(submitter)) => runtime
                .submit_local(
                    models,
                    window,
                    input,
                    &self.policy,
                    spawner,
                    submitter.clone(),
                )
                .unwrap_or_else(|err| {
                    self.set_submit_error(models, None, err);
                    true
                }),
            (MutationRuntimeHandle::MissingDispatcher(message), _) => {
                self.set_submit_error(
                    models,
                    Some(input),
                    MutationError::permanent(message.clone()),
                );
                true
            }
        }
    }

    pub fn cancel(&self, models: &mut ModelStore) -> bool
    where
        TIn: Any + Send + Sync + 'static,
        TOut: Any + Send + Sync + 'static,
    {
        match &self.runtime {
            MutationRuntimeHandle::Ready(runtime) => runtime.cancel(models),
            MutationRuntimeHandle::MissingDispatcher(_) => false,
        }
    }

    pub fn reset(&self, models: &mut ModelStore) -> bool
    where
        TIn: Any + Send + Sync + 'static,
        TOut: Any + Send + Sync + 'static,
    {
        let cancelled = self.cancel(models);
        if let MutationRuntimeHandle::Ready(runtime) = &self.runtime {
            runtime.success_token.store(0, Ordering::SeqCst);
        }
        let reset = models
            .update(&self.model, |st| *st = MutationState::default())
            .is_ok();
        cancelled || reset
    }

    fn send(
        model: Model<MutationState<TIn, TOut>>,
        runtime: MutationRuntimeHandle<TIn, TOut>,
        spawner: Option<FutureSpawnerHandle>,
        policy: MutationPolicy,
        submit: impl Fn(CancellationToken, Arc<TIn>) -> SendMutationFuture<TOut> + Send + Sync + 'static,
    ) -> Self {
        Self {
            model,
            runtime,
            spawner,
            policy,
            submitter: MutationSubmitter::Send(Arc::new(submit)),
        }
    }

    fn local(
        model: Model<MutationState<TIn, TOut>>,
        runtime: MutationRuntimeHandle<TIn, TOut>,
        spawner: Option<FutureSpawnerHandle>,
        policy: MutationPolicy,
        submit: impl Fn(CancellationToken, Arc<TIn>) -> LocalMutationFuture<TOut> + 'static,
    ) -> Self {
        Self {
            model,
            runtime,
            spawner,
            policy,
            submitter: MutationSubmitter::Local(Rc::new(submit)),
        }
    }

    fn set_submit_error(
        &self,
        models: &mut ModelStore,
        input: Option<Arc<TIn>>,
        err: MutationError,
    ) {
        let _ = models.update(&self.model, |st| {
            st.status = MutationStatus::Error;
            if let Some(input) = input {
                st.input = Some(input);
            }
            st.error = Some(err);
            st.inflight = None;
            st.updated_at = Some(Instant::now());
            st.last_duration = None;
        });
        if let MutationRuntimeHandle::Ready(runtime) = &self.runtime {
            runtime.success_token.store(0, Ordering::SeqCst);
        }
    }
}

struct MutationRuntime<TIn: 'static, TOut: 'static> {
    model_id: ModelId,
    exec: Executors,
    inbox: Inbox<MutationInboxMsg<TOut>>,
    tasks: Arc<Mutex<HashMap<u64, BackgroundTask>>>,
    latest_submission_id: AtomicU64,
    success_token: AtomicU64,
    _phantom: PhantomData<fn() -> TIn>,
}

impl<TIn: Any + Send + Sync + 'static, TOut: Any + Send + Sync + 'static>
    MutationRuntime<TIn, TOut>
{
    fn new(model_id: ModelId, dispatcher: DispatcherHandle) -> Self {
        Self {
            model_id,
            exec: Executors::new(dispatcher),
            inbox: Inbox::new(Default::default()),
            tasks: Arc::new(Mutex::new(HashMap::new())),
            latest_submission_id: AtomicU64::new(0),
            success_token: AtomicU64::new(0),
            _phantom: PhantomData,
        }
    }

    fn submit_send(
        &self,
        models: &mut ModelStore,
        window: AppWindowId,
        input: Arc<TIn>,
        policy: &MutationPolicy,
        spawner: &dyn FutureSpawner,
        submit: Arc<SendSubmitFn<TIn, TOut>>,
    ) -> Result<bool, MutationError> {
        let Some(submission_id) = self.prepare_submit(models, input.clone(), policy)? else {
            return Ok(false);
        };

        let started_at = Instant::now();
        let sender = self.inbox.sender();
        let task = self
            .exec
            .spawn_future_to_inbox(spawner, Some(window), sender, {
                let input = input.clone();
                let submit = submit.clone();
                move |token| {
                    let input = input.clone();
                    let submit = submit.clone();
                    async move {
                        let result = submit(token, input).await;
                        MutationInboxMsg {
                            window,
                            submission_id,
                            finished_at: Instant::now(),
                            duration: Instant::now().duration_since(started_at),
                            result,
                        }
                    }
                }
            });
        self.tasks
            .lock()
            .unwrap_or_else(|p| p.into_inner())
            .insert(submission_id, task);
        Ok(true)
    }

    fn submit_local(
        &self,
        models: &mut ModelStore,
        window: AppWindowId,
        input: Arc<TIn>,
        policy: &MutationPolicy,
        spawner: &dyn FutureSpawner,
        submit: Rc<LocalSubmitFn<TIn, TOut>>,
    ) -> Result<bool, MutationError> {
        let Some(submission_id) = self.prepare_submit(models, input.clone(), policy)? else {
            return Ok(false);
        };

        let started_at = Instant::now();
        let sender = self.inbox.sender();
        let Some(task) = self
            .exec
            .spawn_local_future_to_inbox(spawner, Some(window), sender, {
                let input = input.clone();
                let submit = submit.clone();
                move |token| {
                    let input = input.clone();
                    let submit = submit.clone();
                    async move {
                        let result = submit(token, input).await;
                        MutationInboxMsg {
                            window,
                            submission_id,
                            finished_at: Instant::now(),
                            duration: Instant::now().duration_since(started_at),
                            result,
                        }
                    }
                }
            })
        else {
            return Err(MutationError::permanent(
                "FutureSpawner does not support local futures (use a wasm/local spawner or mutation_async)",
            ));
        };

        self.tasks
            .lock()
            .unwrap_or_else(|p| p.into_inner())
            .insert(submission_id, task);
        Ok(true)
    }

    fn prepare_submit(
        &self,
        models: &mut ModelStore,
        input: Arc<TIn>,
        policy: &MutationPolicy,
    ) -> Result<Option<u64>, MutationError> {
        let mut tasks = self.tasks.lock().unwrap_or_else(|p| p.into_inner());
        match policy.concurrency {
            MutationConcurrencyPolicy::CancelInFlight => {
                for (_, task) in tasks.drain() {
                    drop(task);
                }
            }
            MutationConcurrencyPolicy::DropNewWhileRunning => {
                if !tasks.is_empty() {
                    return Ok(None);
                }
            }
            MutationConcurrencyPolicy::AllowParallelLatestWins => {}
        }

        let submission_id = self.latest_submission_id.fetch_add(1, Ordering::SeqCst) + 1;
        self.success_token.store(0, Ordering::SeqCst);
        let applied = self
            .update_state(models, |st| {
                st.status = MutationStatus::Running;
                st.input = Some(input);
                st.error = None;
                st.inflight = Some(submission_id);
                st.updated_at = Some(Instant::now());
                st.last_duration = None;
                if !policy.keep_previous_data_while_running {
                    st.data = None;
                }
            })
            .is_some();
        if !applied {
            for (_, task) in tasks.drain() {
                drop(task);
            }
            return Ok(None);
        }

        Ok(Some(submission_id))
    }

    fn update_state<R>(
        &self,
        models: &mut ModelStore,
        f: impl FnOnce(&mut MutationState<TIn, TOut>) -> R,
    ) -> Option<R> {
        models
            .update_any(self.model_id, |any| {
                let state = any.downcast_mut::<MutationState<TIn, TOut>>()?;
                Some(f(state))
            })
            .ok()
            .flatten()
    }

    fn cancel(&self, models: &mut ModelStore) -> bool {
        let mut tasks = self.tasks.lock().unwrap_or_else(|p| p.into_inner());
        if tasks.is_empty() {
            return false;
        }
        for (_, task) in tasks.drain() {
            drop(task);
        }
        drop(tasks);

        self.update_state(models, |st| {
            st.status = MutationStatus::Idle;
            st.inflight = None;
            st.updated_at = Some(Instant::now());
            st.last_duration = None;
        })
        .is_some_and(|_| {
            self.success_token.store(0, Ordering::SeqCst);
            true
        })
    }

    fn apply(&self, host: &mut dyn InboxDrainHost, msg: MutationInboxMsg<TOut>) {
        self.tasks
            .lock()
            .unwrap_or_else(|p| p.into_inner())
            .remove(&msg.submission_id);

        if self.latest_submission_id.load(Ordering::SeqCst) != msg.submission_id {
            return;
        }

        let mut next_success_token = 0;
        let applied = self.update_state(host.models_mut(), |st| {
            st.inflight = None;
            st.updated_at = Some(msg.finished_at);
            st.last_duration = Some(msg.duration);
            match msg.result {
                Ok(value) => {
                    st.status = MutationStatus::Success;
                    st.data = Some(Arc::new(value));
                    st.error = None;
                    next_success_token = msg.submission_id;
                }
                Err(err) => {
                    st.status = MutationStatus::Error;
                    st.error = Some(err);
                    next_success_token = 0;
                }
            }
        });
        if applied.is_some() {
            self.success_token
                .store(next_success_token, Ordering::SeqCst);
            host.request_redraw(msg.window);
        }
    }
}

struct MutationInboxMsg<TOut> {
    window: AppWindowId,
    submission_id: u64,
    finished_at: Instant,
    duration: Duration,
    result: Result<TOut, MutationError>,
}

trait ErasedMutationRuntimeEntry: Any {
    fn as_any(&self) -> &dyn Any;
    fn is_live(&self) -> bool;
}

struct TypedMutationRuntimeEntry<TIn: 'static, TOut: 'static> {
    runtime: Arc<MutationRuntime<TIn, TOut>>,
    state: WeakModel<MutationState<TIn, TOut>>,
}

impl<TIn: Any + Send + Sync + 'static, TOut: Any + Send + Sync + 'static> ErasedMutationRuntimeEntry
    for TypedMutationRuntimeEntry<TIn, TOut>
{
    fn as_any(&self) -> &dyn Any {
        self
    }

    fn is_live(&self) -> bool {
        self.state.upgrade().is_some()
    }
}

#[derive(Default)]
struct MutationRuntimeRegistry {
    entries: HashMap<ModelId, Box<dyn ErasedMutationRuntimeEntry>>,
}

impl MutationRuntimeRegistry {
    fn runtime_for<H, TIn, TOut>(
        &mut self,
        app: &mut H,
        state: &Model<MutationState<TIn, TOut>>,
        dispatcher: DispatcherHandle,
    ) -> Arc<MutationRuntime<TIn, TOut>>
    where
        H: UiHost,
        TIn: Any + Send + Sync + 'static,
        TOut: Any + Send + Sync + 'static,
    {
        self.entries.retain(|_, entry| entry.is_live());
        if let Some(existing) = self.entries.get(&state.id()).and_then(|entry| {
            entry
                .as_any()
                .downcast_ref::<TypedMutationRuntimeEntry<TIn, TOut>>()
        }) {
            return existing.runtime.clone();
        }

        let runtime = Arc::new(MutationRuntime::<TIn, TOut>::new(state.id(), dispatcher));
        let drainer = mutation_inbox_drainer(runtime.inbox.clone(), Arc::downgrade(&runtime));
        app.with_global_mut_untracked(InboxDrainRegistry::default, |registry, _app| {
            registry.register(Arc::new(drainer));
        });
        self.entries.insert(
            state.id(),
            Box::new(TypedMutationRuntimeEntry {
                runtime: runtime.clone(),
                state: state.downgrade(),
            }),
        );
        runtime
    }
}

fn ensure_runtime<H, TIn, TOut>(
    app: &mut H,
    state: &Model<MutationState<TIn, TOut>>,
) -> MutationRuntimeHandle<TIn, TOut>
where
    H: UiHost,
    TIn: Any + Send + Sync + 'static,
    TOut: Any + Send + Sync + 'static,
{
    let Some(dispatcher) = app.global::<DispatcherHandle>().cloned() else {
        return MutationRuntimeHandle::<TIn, TOut>::MissingDispatcher(Arc::from(
            "missing DispatcherHandle global (install a dispatcher to use mutation_async)",
        ));
    };

    app.with_global_mut_untracked(MutationRuntimeRegistry::default, |registry, app| {
        MutationRuntimeHandle::Ready(registry.runtime_for(app, state, dispatcher))
    })
}

fn mutation_inbox_drainer<TIn, TOut>(
    inbox: Inbox<MutationInboxMsg<TOut>>,
    runtime: Weak<MutationRuntime<TIn, TOut>>,
) -> InboxDrainer<MutationInboxMsg<TOut>>
where
    TIn: Any + Send + Sync + 'static,
    TOut: Any + Send + Sync + 'static,
{
    InboxDrainer::new(inbox, move |host, _window, msg| {
        let Some(runtime) = runtime.upgrade() else {
            return;
        };
        runtime.apply(host, msg);
    })
}

#[cfg(feature = "ui")]
pub mod ui {
    use super::*;
    use fret_ui::action::{ActionCx, UiActionHost};
    use fret_ui::{ElementContext, UiHost};

    pub trait MutationElementContextExt {
        fn use_mutation_async<TIn, TOut, Fut>(
            &mut self,
            policy: MutationPolicy,
            submit: impl Fn(CancellationToken, Arc<TIn>) -> Fut + Send + Sync + 'static,
        ) -> MutationHandle<TIn, TOut>
        where
            TIn: Any + Send + Sync + 'static,
            TOut: Any + Send + Sync + 'static,
            Fut: Future<Output = Result<TOut, MutationError>> + Send + 'static;

        fn use_mutation_async_local<TIn, TOut, Fut>(
            &mut self,
            policy: MutationPolicy,
            submit: impl Fn(CancellationToken, Arc<TIn>) -> Fut + 'static,
        ) -> MutationHandle<TIn, TOut>
        where
            TIn: Any + Send + Sync + 'static,
            TOut: Any + Send + Sync + 'static,
            Fut: Future<Output = Result<TOut, MutationError>> + 'static;
    }

    impl<'a, H: UiHost> MutationElementContextExt for ElementContext<'a, H> {
        fn use_mutation_async<TIn, TOut, Fut>(
            &mut self,
            policy: MutationPolicy,
            submit: impl Fn(CancellationToken, Arc<TIn>) -> Fut + Send + Sync + 'static,
        ) -> MutationHandle<TIn, TOut>
        where
            TIn: Any + Send + Sync + 'static,
            TOut: Any + Send + Sync + 'static,
            Fut: Future<Output = Result<TOut, MutationError>> + Send + 'static,
        {
            let model = self.local_model(MutationState::<TIn, TOut>::default);
            let runtime = ensure_runtime(self.app, &model);
            let spawner = self.app.global::<FutureSpawnerHandle>().cloned();
            MutationHandle::send(model, runtime, spawner, policy, move |token, input| {
                Box::pin(submit(token, input))
            })
        }

        fn use_mutation_async_local<TIn, TOut, Fut>(
            &mut self,
            policy: MutationPolicy,
            submit: impl Fn(CancellationToken, Arc<TIn>) -> Fut + 'static,
        ) -> MutationHandle<TIn, TOut>
        where
            TIn: Any + Send + Sync + 'static,
            TOut: Any + Send + Sync + 'static,
            Fut: Future<Output = Result<TOut, MutationError>> + 'static,
        {
            let model = self.local_model(MutationState::<TIn, TOut>::default);
            let runtime = ensure_runtime(self.app, &model);
            let spawner = self.app.global::<FutureSpawnerHandle>().cloned();
            MutationHandle::local(model, runtime, spawner, policy, move |token, input| {
                Box::pin(submit(token, input))
            })
        }
    }

    pub trait MutationHandleActionExt<TIn: 'static, TOut: 'static> {
        fn submit_action(
            &self,
            host: &mut dyn UiActionHost,
            action_cx: ActionCx,
            input: TIn,
        ) -> bool
        where
            TIn: Any + Send + Sync + 'static,
            TOut: Any + Send + Sync + 'static;

        fn cancel_action(&self, host: &mut dyn UiActionHost, action_cx: ActionCx) -> bool
        where
            TIn: Any + Send + Sync + 'static,
            TOut: Any + Send + Sync + 'static;

        fn reset_action(&self, host: &mut dyn UiActionHost, action_cx: ActionCx) -> bool
        where
            TIn: Any + Send + Sync + 'static,
            TOut: Any + Send + Sync + 'static;
    }

    impl<TIn: 'static, TOut: 'static> MutationHandleActionExt<TIn, TOut> for MutationHandle<TIn, TOut> {
        fn submit_action(
            &self,
            host: &mut dyn UiActionHost,
            action_cx: ActionCx,
            input: TIn,
        ) -> bool
        where
            TIn: Any + Send + Sync + 'static,
            TOut: Any + Send + Sync + 'static,
        {
            let changed = self.submit(host.models_mut(), action_cx.window, input);
            if changed {
                host.request_redraw(action_cx.window);
                host.notify(action_cx);
            }
            changed
        }

        fn cancel_action(&self, host: &mut dyn UiActionHost, action_cx: ActionCx) -> bool
        where
            TIn: Any + Send + Sync + 'static,
            TOut: Any + Send + Sync + 'static,
        {
            let changed = self.cancel(host.models_mut());
            if changed {
                host.request_redraw(action_cx.window);
                host.notify(action_cx);
            }
            changed
        }

        fn reset_action(&self, host: &mut dyn UiActionHost, action_cx: ActionCx) -> bool
        where
            TIn: Any + Send + Sync + 'static,
            TOut: Any + Send + Sync + 'static,
        {
            let changed = self.reset(host.models_mut());
            if changed {
                host.request_redraw(action_cx.window);
                host.notify(action_cx);
            }
            changed
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use fret_app::App;
    use fret_executor::FutureSpawner;
    use fret_runtime::{DispatchPriority, Dispatcher, ExecCapabilities, Runnable};
    use pollster::block_on;

    fn drain_inboxes(app: &mut App, window: Option<AppWindowId>) -> bool {
        app.with_global_mut_untracked(InboxDrainRegistry::default, |registry, app| {
            registry.drain_all(app, window)
        })
    }

    #[derive(Default)]
    struct TestDispatcher {
        background: Mutex<Vec<Runnable>>,
    }

    impl Dispatcher for TestDispatcher {
        fn dispatch_on_main_thread(&self, runnable: Runnable) {
            runnable();
        }

        fn dispatch_background(&self, runnable: Runnable, _priority: DispatchPriority) {
            self.background.lock().unwrap().push(runnable);
        }

        fn dispatch_after(&self, _delay: Duration, runnable: Runnable) {
            runnable();
        }

        fn wake(&self, _window: Option<AppWindowId>) {}

        fn exec_capabilities(&self) -> ExecCapabilities {
            ExecCapabilities::default()
        }
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
    fn submit_send_updates_state_to_success() {
        let mut app = App::new();
        let dispatcher = Arc::new(TestDispatcher::default());
        let dispatcher_handle: DispatcherHandle = dispatcher.clone();
        app.set_global::<DispatcherHandle>(dispatcher_handle);

        let spawner = Arc::new(TestSpawner::default());
        let spawner_handle: FutureSpawnerHandle = spawner.clone();
        app.set_global::<FutureSpawnerHandle>(spawner_handle.clone());

        let state = app
            .models_mut()
            .insert(MutationState::<u32, u32>::default());
        let runtime = MutationRuntimeHandle::Ready(
            app.with_global_mut_untracked(MutationRuntimeRegistry::default, |registry, app| {
                registry.runtime_for(app, &state, dispatcher.clone())
            }),
        );
        let handle = MutationHandle::send(
            state.clone(),
            runtime,
            Some(spawner_handle),
            MutationPolicy::default(),
            |_token, input| Box::pin(async move { Ok(*input + 1) }),
        );

        assert!(handle.submit(app.models_mut(), AppWindowId::default(), 41));
        assert_eq!(handle.success_token(), None);
        let running = state.read_ref(&app, |s| s.clone()).unwrap();
        assert_eq!(running.status, MutationStatus::Running);

        for fut in spawner.drain_send() {
            block_on(fut);
        }
        assert!(drain_inboxes(&mut app, Some(AppWindowId::default())));

        let state = state.read_ref(&app, |s| s.clone()).unwrap();
        assert_eq!(state.status, MutationStatus::Success);
        assert_eq!(state.data.as_deref().copied(), Some(42));
        assert!(handle.success_token().is_some());
    }

    #[test]
    fn success_token_tracks_only_current_success_state() {
        let mut app = App::new();
        let dispatcher = Arc::new(TestDispatcher::default());
        let dispatcher_handle: DispatcherHandle = dispatcher.clone();
        app.set_global::<DispatcherHandle>(dispatcher_handle);

        let spawner = Arc::new(TestSpawner::default());
        let spawner_handle: FutureSpawnerHandle = spawner.clone();
        app.set_global::<FutureSpawnerHandle>(spawner_handle.clone());

        let state = app
            .models_mut()
            .insert(MutationState::<u32, u32>::default());
        let runtime = MutationRuntimeHandle::Ready(
            app.with_global_mut_untracked(MutationRuntimeRegistry::default, |registry, app| {
                registry.runtime_for(app, &state, dispatcher.clone())
            }),
        );
        let handle = MutationHandle::send(
            state,
            runtime,
            Some(spawner_handle),
            MutationPolicy::default(),
            |_token, input| {
                Box::pin(async move {
                    if *input == 0 {
                        Err(MutationError::transient("boom"))
                    } else {
                        Ok(*input)
                    }
                })
            },
        );

        assert_eq!(handle.success_token(), None);

        assert!(handle.submit(app.models_mut(), AppWindowId::default(), 1));
        assert_eq!(handle.success_token(), None);
        for fut in spawner.drain_send() {
            block_on(fut);
        }
        assert!(drain_inboxes(&mut app, Some(AppWindowId::default())));
        let first_token = handle
            .success_token()
            .expect("success token should exist after success");

        assert!(handle.submit(app.models_mut(), AppWindowId::default(), 0));
        assert_eq!(handle.success_token(), None);
        for fut in spawner.drain_send() {
            block_on(fut);
        }
        assert!(drain_inboxes(&mut app, Some(AppWindowId::default())));
        assert_eq!(
            handle.success_token(),
            None,
            "error terminal state should not keep a success token"
        );

        assert!(handle.submit(app.models_mut(), AppWindowId::default(), 2));
        for fut in spawner.drain_send() {
            block_on(fut);
        }
        assert!(drain_inboxes(&mut app, Some(AppWindowId::default())));
        let second_token = handle
            .success_token()
            .expect("new success should publish a fresh token");
        assert_ne!(first_token, second_token);

        assert!(handle.reset(app.models_mut()));
        assert_eq!(handle.success_token(), None);
    }

    #[test]
    fn cancel_resets_running_state_to_idle() {
        let mut app = App::new();
        let dispatcher = Arc::new(TestDispatcher::default());
        let dispatcher_handle: DispatcherHandle = dispatcher.clone();
        app.set_global::<DispatcherHandle>(dispatcher_handle);

        let state = app
            .models_mut()
            .insert(MutationState::<u32, u32>::default());
        let runtime = MutationRuntimeHandle::Ready(
            app.with_global_mut_untracked(MutationRuntimeRegistry::default, |registry, app| {
                registry.runtime_for(app, &state, dispatcher.clone())
            }),
        );
        let handle = MutationHandle::send(
            state.clone(),
            runtime,
            None,
            MutationPolicy::default(),
            |_token, input| Box::pin(async move { Ok(*input) }),
        );

        handle.set_submit_error(
            app.models_mut(),
            Some(Arc::new(7)),
            MutationError::transient("running"),
        );
        let _ = app.models_mut().update(&state, |st| {
            st.status = MutationStatus::Running;
            st.inflight = Some(1);
        });

        let runtime = ensure_runtime(&mut app, &state);
        let handle = MutationHandle::send(
            state.clone(),
            runtime,
            None,
            MutationPolicy::default(),
            |_token, input| Box::pin(async move { Ok(*input) }),
        );
        assert!(!handle.cancel(app.models_mut()));
    }
}
