//! View authoring runtime (ecosystem-level).
//!
//! This module provides a cohesive authoring loop aligned with ADR 0308:
//! - a stateful `View` object renders into the app-facing `Ui` alias (backed by the existing
//!   declarative IR),
//! - views can register typed action handlers (action-first),
//! - hook-style helpers compose existing mechanism contracts (models + observation).
//!
//! v1 notes:
//! - the explicit raw-model hook seam (`AppUiRawModelExt::raw_model<T>()`) currently returns a
//!   `Model<T>` allocated in the app-owned model store. This keeps event handlers object-safe
//!   (they only receive `UiActionHost`) while still providing view-local state ergonomics.
//! - The view runtime is intentionally additive and lives in `ecosystem/fret` (not kernel).

#[cfg(test)]
use fret_ui::action::OnActivate;

mod actions;
mod activation;
mod bridges;
mod context;
mod data;
mod effects;
mod lane_barriers;
mod layout_query;
mod local_state;
mod pointer;
mod raw;
mod runtime;
mod scheduling;
mod shell;
mod state;
#[allow(unused_imports)]
pub use actions::{
    AppRenderActionLocal, AppRenderActions, AppRenderActionsExt, AppRenderLocalsWith,
    AppUiActionLocal, AppUiActions, AppUiLocalsWith,
};
#[cfg(test)]
use activation::action_listener;
pub use activation::{AppActivateExt, AppActivateSurface};
#[cfg(test)]
use activation::{dispatch_action_listener, dispatch_payload_action_listener};
pub use context::{AppRenderContext, RenderContextAccess, View};
#[cfg(feature = "state-mutation")]
#[allow(unused_imports)]
pub use data::MutationHandleReadLayoutExt;
#[cfg(feature = "state-query")]
#[allow(unused_imports)]
pub use data::QueryHandleReadLayoutExt;
#[allow(unused_imports)]
pub use data::{AppRenderData, AppRenderDataExt, AppUiData};
#[cfg(feature = "state-selector")]
#[allow(unused_imports)]
pub use data::{LocalSelectorLayoutInputs, ModelSelectorInputs};
pub use effects::AppUiEffects;
pub use local_state::{
    AppLocalStateExt, LocalActionCapture, LocalState, LocalStateElementContextExt,
    LocalStateModelStoreExt, LocalStateRawModelExt, LocalStateTxn, TrackedStateExt, WatchedState,
};
pub use pointer::{
    AppPointerRegion, CursorIcon, MouseButton, Point, PointerActionCx, PointerCancel, PointerDown,
    PointerId, PointerMove, PointerRegion, PointerUp, Wheel,
};
pub use raw::{
    AppUiComponentLaneRequiresExplicitElementsEscapeHatch, AppUiRawActionNotifyExt,
    AppUiRawModelExt,
};
#[cfg(all(not(target_arch = "wasm32"), feature = "desktop"))]
pub use runtime::view_record_engine_frame;
pub use runtime::{
    AppUiRenderRootState, ViewWindowState, render_root_with_app_ui, view_init_window, view_view,
};
pub use shell::AppUi;
pub use state::AppUiState;

#[cfg(test)]
mod tests {
    use super::{
        AppActivateExt, AppActivateSurface, AppLocalStateExt as _, AppRenderActionsExt as _,
        AppUiRenderRootState, LocalActionCapture, LocalState, LocalStateElementContextExt as _,
        LocalStateModelStoreExt as _, LocalStateRawModelExt as _, LocalStateTxn, OnActivate, View,
        ViewWindowState, action_listener, dispatch_action_listener,
        dispatch_payload_action_listener, render_root_with_app_ui, view_init_window, view_view,
    };
    use std::any::Any;
    #[cfg(feature = "state-mutation")]
    use std::cell::RefCell;
    #[cfg(feature = "state-mutation")]
    use std::future::Future;
    #[cfg(feature = "state-mutation")]
    use std::pin::Pin;
    use std::sync::{
        Arc,
        atomic::{AtomicUsize, Ordering},
    };
    #[cfg(feature = "state-mutation")]
    use std::task::{Context, Poll, Waker};
    const ACTIVATION_RS_SOURCE: &str = include_str!("view/activation.rs");
    const ACTIONS_RS_SOURCE: &str = include_str!("view/actions.rs");
    const BRIDGES_RS_SOURCE: &str = include_str!("view/bridges.rs");
    const VIEW_RS_SOURCE: &str = include_str!("view.rs");
    const CONTEXT_RS_SOURCE: &str = include_str!("view/context.rs");
    const DATA_RS_SOURCE: &str = include_str!("view/data.rs");
    const DATA_RENDER_RS_SOURCE: &str = include_str!("view/data/render.rs");
    const EFFECTS_RS_SOURCE: &str = include_str!("view/effects.rs");
    const LANE_BARRIERS_RS_SOURCE: &str = include_str!("view/lane_barriers.rs");
    const LAYOUT_QUERY_RS_SOURCE: &str = include_str!("view/layout_query.rs");
    const LOCAL_STATE_RS_SOURCE: &str = include_str!("view/local_state.rs");
    const LOCAL_STATE_ADAPTERS_RS_SOURCE: &str = include_str!("view/local_state/adapters.rs");
    const LOCAL_STATE_BRIDGES_RS_SOURCE: &str = include_str!("view/local_state/bridges.rs");
    const POINTER_RS_SOURCE: &str = include_str!("view/pointer.rs");
    const RAW_RS_SOURCE: &str = include_str!("view/raw.rs");
    const RUNTIME_RS_SOURCE: &str = include_str!("view/runtime.rs");
    const SCHEDULING_RS_SOURCE: &str = include_str!("view/scheduling.rs");
    const SHELL_RS_SOURCE: &str = include_str!("view/shell.rs");
    const STATE_RS_SOURCE: &str = include_str!("view/state.rs");

    fn view_authoring_api_source() -> String {
        let view_api = VIEW_RS_SOURCE
            .split("\nmod tests {")
            .next()
            .expect("view.rs test module marker should exist");
        format!(
            "{view_api}\n{ACTIVATION_RS_SOURCE}\n{ACTIONS_RS_SOURCE}\n{BRIDGES_RS_SOURCE}\n{CONTEXT_RS_SOURCE}\n{DATA_RS_SOURCE}\n{DATA_RENDER_RS_SOURCE}\n{EFFECTS_RS_SOURCE}\n{LANE_BARRIERS_RS_SOURCE}\n{LAYOUT_QUERY_RS_SOURCE}\n{LOCAL_STATE_RS_SOURCE}\n{LOCAL_STATE_ADAPTERS_RS_SOURCE}\n{LOCAL_STATE_BRIDGES_RS_SOURCE}\n{POINTER_RS_SOURCE}\n{RAW_RS_SOURCE}\n{RUNTIME_RS_SOURCE}\n{SCHEDULING_RS_SOURCE}\n{SHELL_RS_SOURCE}\n{STATE_RS_SOURCE}"
        )
    }
    use fret_core::{
        AppWindowId, FrameId, Modifiers, MouseButton, NodeId, Point, PointerEvent, PointerType, Px,
        Rect, Size, TextConstraints, TextMetrics, WindowMetricsService,
    };
    use fret_runtime::{
        ActionId, CommandId, Effect, ModelStore, TickId, TimerToken,
        WindowPendingActionPayloadService,
    };
    #[cfg(feature = "state-mutation")]
    use fret_runtime::{
        DispatchPriority, Dispatcher, DispatcherHandle, ExecCapabilities, InboxDrainRegistry,
        Runnable,
    };
    use fret_ui::action::{ActionCx, ActivateReason, UiActionHost, UiFocusActionHost};
    use fret_ui::declarative::render_root;
    use fret_ui::{UiTree, element::Length};
    #[cfg(feature = "shadcn")]
    use fret_ui_kit::IntoUiElementInExt;

    #[derive(Default)]
    struct FakeUiServices;

    impl fret_core::TextService for FakeUiServices {
        fn prepare(
            &mut self,
            _input: &fret_core::TextInput,
            _constraints: TextConstraints,
        ) -> (fret_core::TextBlobId, TextMetrics) {
            (
                fret_core::TextBlobId::default(),
                TextMetrics {
                    size: Size::new(Px(10.0), Px(10.0)),
                    baseline: Px(8.0),
                },
            )
        }

        fn release(&mut self, _blob: fret_core::TextBlobId) {}
    }

    impl fret_core::PathService for FakeUiServices {
        fn prepare(
            &mut self,
            _commands: &[fret_core::PathCommand],
            _style: fret_core::PathStyle,
            _constraints: fret_core::PathConstraints,
        ) -> (fret_core::PathId, fret_core::PathMetrics) {
            (
                fret_core::PathId::default(),
                fret_core::PathMetrics::default(),
            )
        }

        fn release(&mut self, _path: fret_core::PathId) {}
    }

    impl fret_core::SvgService for FakeUiServices {
        fn register_svg(&mut self, _bytes: &[u8]) -> fret_core::SvgId {
            fret_core::SvgId::default()
        }

        fn unregister_svg(&mut self, _svg: fret_core::SvgId) -> bool {
            false
        }
    }

    impl fret_core::MaterialService for FakeUiServices {
        fn register_material(
            &mut self,
            _desc: fret_core::MaterialDescriptor,
        ) -> Result<fret_core::MaterialId, fret_core::MaterialRegistrationError> {
            Err(fret_core::MaterialRegistrationError::Unsupported)
        }

        fn unregister_material(&mut self, _id: fret_core::MaterialId) -> bool {
            false
        }
    }

    #[cfg(feature = "state-mutation")]
    #[derive(Default)]
    struct TestDispatcher;

    #[cfg(feature = "state-mutation")]
    impl Dispatcher for TestDispatcher {
        fn dispatch_on_main_thread(&self, runnable: Runnable) {
            runnable();
        }

        fn dispatch_background(&self, runnable: Runnable, _priority: DispatchPriority) {
            runnable();
        }

        fn dispatch_after(&self, _delay: std::time::Duration, runnable: Runnable) {
            runnable();
        }

        fn wake(&self, _window: Option<AppWindowId>) {}

        fn exec_capabilities(&self) -> ExecCapabilities {
            ExecCapabilities::default()
        }
    }

    #[cfg(feature = "state-mutation")]
    #[derive(Default)]
    struct ReadyOnlySpawner;

    #[cfg(feature = "state-mutation")]
    impl fret_mutation::FutureSpawner for ReadyOnlySpawner {
        fn spawn_send(&self, mut fut: Pin<Box<dyn Future<Output = ()> + Send + 'static>>) {
            let mut cx = Context::from_waker(Waker::noop());
            match fut.as_mut().poll(&mut cx) {
                Poll::Ready(()) => {}
                Poll::Pending => panic!("test mutation future should complete immediately"),
            }
        }

        fn spawn_local(&self, mut fut: Pin<Box<dyn Future<Output = ()> + 'static>>) -> bool {
            let mut cx = Context::from_waker(Waker::noop());
            match fut.as_mut().poll(&mut cx) {
                Poll::Ready(()) => true,
                Poll::Pending => panic!("test mutation future should complete immediately"),
            }
        }
    }

    #[cfg(feature = "state-mutation")]
    fn drain_inboxes(app: &mut crate::app::App, window: AppWindowId) -> bool {
        app.with_global_mut_untracked(InboxDrainRegistry::default, |registry, app| {
            registry.drain_all(app, Some(window))
        })
    }

    struct DispatchAction;
    impl fret_runtime::TypedAction for DispatchAction {
        fn action_id() -> ActionId {
            ActionId::from("test.dispatch_action.v1")
        }
    }

    struct DispatchPayloadAction;
    impl fret_runtime::TypedAction for DispatchPayloadAction {
        fn action_id() -> ActionId {
            ActionId::from("test.dispatch_payload_action.v1")
        }
    }
    impl crate::actions::TypedPayloadAction for DispatchPayloadAction {
        type Payload = u64;
    }

    struct RuntimeIncrementAction;
    impl fret_runtime::TypedAction for RuntimeIncrementAction {
        fn action_id() -> ActionId {
            ActionId::from("test.locals_with.runtime.increment.v1")
        }
    }

    #[cfg(feature = "shadcn")]
    struct RuntimeButtonIncrementAction;
    #[cfg(feature = "shadcn")]
    impl fret_runtime::TypedAction for RuntimeButtonIncrementAction {
        fn action_id() -> ActionId {
            ActionId::from("test.locals_with.runtime.button_increment.v1")
        }
    }

    struct RuntimePayloadAppendAction;
    impl fret_runtime::TypedAction for RuntimePayloadAppendAction {
        fn action_id() -> ActionId {
            ActionId::from("test.uicx.payload_models.append.v1")
        }
    }
    impl crate::actions::TypedPayloadAction for RuntimePayloadAppendAction {
        type Payload = u64;
    }

    #[derive(Default)]
    struct DummyActivateSurface {
        on_activate: Option<OnActivate>,
    }

    impl AppActivateSurface for DummyActivateSurface {
        fn on_activate(mut self, on_activate: OnActivate) -> Self {
            self.on_activate = Some(on_activate);
            self
        }
    }

    #[derive(Default)]
    struct FakeHost {
        models: ModelStore,
        redraws: Vec<AppWindowId>,
        notifies: Vec<ActionCx>,
        effects: Vec<Effect>,
        dispatch_sources: Vec<(ActionCx, CommandId, ActivateReason)>,
        payloads: Vec<(ActionCx, ActionId, Box<dyn Any + Send + Sync>)>,
        next_timer: u64,
    }

    impl UiActionHost for FakeHost {
        fn models_mut(&mut self) -> &mut ModelStore {
            &mut self.models
        }

        fn push_effect(&mut self, effect: Effect) {
            self.effects.push(effect);
        }

        fn request_redraw(&mut self, window: AppWindowId) {
            self.redraws.push(window);
        }

        fn next_timer_token(&mut self) -> TimerToken {
            let current = self.next_timer;
            self.next_timer = self.next_timer.saturating_add(1);
            TimerToken(current)
        }

        fn next_clipboard_token(&mut self) -> fret_runtime::ClipboardToken {
            fret_runtime::ClipboardToken::default()
        }

        fn next_share_sheet_token(&mut self) -> fret_runtime::ShareSheetToken {
            fret_runtime::ShareSheetToken::default()
        }

        fn notify(&mut self, cx: ActionCx) {
            self.notifies.push(cx);
        }

        fn record_pending_command_dispatch_source(
            &mut self,
            cx: ActionCx,
            command: &CommandId,
            reason: ActivateReason,
        ) {
            self.dispatch_sources.push((cx, command.clone(), reason));
        }

        fn record_pending_action_payload(
            &mut self,
            cx: ActionCx,
            action: &ActionId,
            payload: Box<dyn Any + Send + Sync>,
        ) {
            self.payloads.push((cx, action.clone(), payload));
        }
    }

    impl UiFocusActionHost for FakeHost {
        fn request_focus(&mut self, _target: fret_ui::GlobalElementId) {}
    }

    struct RuntimeLocalsWithView {
        count: Option<LocalState<u32>>,
        touched: Option<LocalState<bool>>,
        renders: Arc<AtomicUsize>,
        last_seen_count: Arc<AtomicUsize>,
    }

    impl View for RuntimeLocalsWithView {
        fn init(_app: &mut crate::app::App, _window: crate::WindowId) -> Self {
            Self {
                count: None,
                touched: None,
                renders: Arc::new(AtomicUsize::new(0)),
                last_seen_count: Arc::new(AtomicUsize::new(0)),
            }
        }

        fn render(&mut self, cx: &mut crate::AppUi<'_, '_>) -> crate::Ui {
            self.renders.fetch_add(1, Ordering::SeqCst);

            if self.count.is_none() {
                self.count = Some(cx.state().local_init(|| 0u32));
            }
            if self.touched.is_none() {
                self.touched = Some(cx.state().local_init(|| false));
            }

            let count = self
                .count
                .as_ref()
                .expect("count local should exist")
                .clone();
            let touched = self
                .touched
                .as_ref()
                .expect("touched local should exist")
                .clone();

            cx.actions()
                .locals_with((&count, &touched))
                .on::<RuntimeIncrementAction>(|tx, (count, touched)| {
                    let incremented = tx.update_if(&count, |value| {
                        *value += 1;
                        true
                    });
                    let flagged = tx.set(&touched, true);
                    incremented || flagged
                });

            self.last_seen_count
                .store(count.layout_value(cx) as usize, Ordering::SeqCst);

            let mut props = fret_ui::element::ContainerProps::default();
            props.layout.size.width = Length::Fill;
            props.layout.size.height = Length::Fill;

            let cx = cx.elements();
            cx.container(props, |_cx| Vec::new()).into()
        }
    }

    fn render_runtime_view(
        ui: &mut UiTree<crate::app::App>,
        app: &mut crate::app::App,
        services: &mut FakeUiServices,
        window: AppWindowId,
        bounds: Rect,
        st: &mut ViewWindowState<RuntimeLocalsWithView>,
    ) -> NodeId {
        let root = render_root(
            ui,
            app,
            services,
            window,
            bounds,
            "locals-with-runtime",
            |cx| view_view(cx, st),
        );
        ui.set_root(root);
        ui.layout_all(app, services, bounds, 1.0);
        root
    }

    #[cfg(feature = "shadcn")]
    struct RuntimeButtonActionView {
        count: Option<LocalState<u32>>,
    }

    #[cfg(feature = "shadcn")]
    impl View for RuntimeButtonActionView {
        fn init(_app: &mut crate::app::App, _window: crate::WindowId) -> Self {
            Self { count: None }
        }

        fn render(&mut self, cx: &mut crate::AppUi<'_, '_>) -> crate::Ui {
            if self.count.is_none() {
                self.count = Some(cx.state().local_init(|| 0u32));
            }

            let count = self.count.as_ref().expect("count local").clone();
            cx.actions()
                .local(&count)
                .update::<RuntimeButtonIncrementAction>(|value| {
                    *value += 1;
                });

            crate::shadcn::Button::new("Increment")
                .action(<RuntimeButtonIncrementAction as fret_runtime::TypedAction>::action_id())
                .test_id("test.runtime_button_action")
                .into_element_in(cx)
                .into()
        }
    }

    #[cfg(feature = "shadcn")]
    fn render_runtime_button_action_view(
        ui: &mut UiTree<crate::app::App>,
        app: &mut crate::app::App,
        services: &mut FakeUiServices,
        window: AppWindowId,
        bounds: Rect,
        st: &mut ViewWindowState<RuntimeButtonActionView>,
    ) -> NodeId {
        let root = render_root(
            ui,
            app,
            services,
            window,
            bounds,
            "runtime-button-action",
            |cx| view_view(cx, st),
        );
        ui.set_root(root);
        ui.request_semantics_snapshot();
        ui.layout_all(app, services, bounds, 1.0);
        root
    }

    fn seed_runtime_window_metrics(
        app: &mut crate::app::App,
        window: AppWindowId,
        bounds: Rect,
        scale_factor: f32,
    ) {
        app.with_global_mut_untracked(WindowMetricsService::default, |svc, _app| {
            svc.set_inner_size(window, bounds.size);
            svc.set_scale_factor(window, scale_factor);
            svc.set_focused(window, true);
        });
        app.with_global_mut_untracked(fret_ui::elements::ElementRuntime::new, |rt, _app| {
            rt.set_window_primary_pointer_type(window, PointerType::Unknown);
        });
    }

    fn render_runtime_view_semantics<V: View>(
        ui: &mut UiTree<crate::app::App>,
        app: &mut crate::app::App,
        services: &mut FakeUiServices,
        window: AppWindowId,
        bounds: Rect,
        scale_factor: f32,
        frame_id: u64,
        root_name: &str,
        st: &mut ViewWindowState<V>,
    ) -> fret_core::SemanticsSnapshot {
        app.set_tick_id(TickId(frame_id));
        app.set_frame_id(FrameId(frame_id));
        seed_runtime_window_metrics(app, window, bounds, scale_factor);

        let root = render_root(ui, app, services, window, bounds, root_name, |cx| {
            view_view(cx, st)
        });
        ui.set_root(root);
        ui.request_semantics_snapshot();
        ui.layout_all(app, services, bounds, scale_factor);
        ui.semantics_snapshot()
            .expect("runtime semantics snapshot")
            .clone()
    }

    fn snapshot_test_ids(snapshot: &fret_core::SemanticsSnapshot) -> Vec<String> {
        let mut ids: Vec<String> = snapshot
            .nodes
            .iter()
            .filter_map(|node| node.test_id.as_ref().map(ToString::to_string))
            .collect();
        ids.sort();
        ids
    }

    fn node_with_test_id(
        ui: &mut UiTree<crate::app::App>,
        test_id: &str,
    ) -> Option<fret_core::NodeId> {
        ui.request_semantics_snapshot();
        ui.semantics_snapshot().and_then(|snapshot| {
            snapshot
                .nodes
                .iter()
                .find(|node| node.test_id.as_deref() == Some(test_id))
                .map(|node| node.id)
        })
    }

    struct RuntimeToggleGroupFooterView {
        filter: fret_runtime::Model<Option<Arc<str>>>,
        action_flag: Option<LocalState<bool>>,
    }

    impl View for RuntimeToggleGroupFooterView {
        fn init(app: &mut crate::app::App, _window: crate::WindowId) -> Self {
            Self {
                filter: app.models_mut().insert(Some(Arc::from("all"))),
                action_flag: None,
            }
        }

        fn render(&mut self, cx: &mut crate::AppUi<'_, '_>) -> crate::Ui {
            if self.action_flag.is_none() {
                self.action_flag = Some(cx.state().local_init(|| false));
            }
            let action_flag = self
                .action_flag
                .as_ref()
                .expect("action flag should exist")
                .clone();
            cx.actions()
                .local(&action_flag)
                .update::<RuntimeIncrementAction>(|value| *value = !*value);
            let _flag = action_flag.layout_value(cx);

            let viewport = cx.environment_viewport_bounds(fret_ui::Invalidation::Layout);
            let compact = viewport.size.width.0 < 560.0;

            let cx = cx.elements();

            let filters = crate::shadcn::ToggleGroup::single(self.filter.clone())
                .deselectable(false)
                .items([
                    crate::shadcn::ToggleGroupItem::new("all", [cx.text("All")])
                        .test_id("runtime.toggle.filter.all"),
                    crate::shadcn::ToggleGroupItem::new("active", [cx.text("Active")])
                        .test_id("runtime.toggle.filter.active"),
                    crate::shadcn::ToggleGroupItem::new("completed", [cx.text("Completed")])
                        .test_id("runtime.toggle.filter.completed"),
                ])
                .into_element(cx);

            let clear = crate::shadcn::Button::new("Clear")
                .test_id("runtime.toggle.clear")
                .into_element(cx);
            let body = cx.text("Body").test_id("runtime.toggle.body");

            let footer = if compact {
                let clear_row = cx
                    .flex(
                        fret_ui::element::FlexProps {
                            direction: fret_core::Axis::Horizontal,
                            ..Default::default()
                        },
                        move |_cx| vec![clear],
                    )
                    .test_id("runtime.toggle.clear_row");
                cx.flex(
                    fret_ui::element::FlexProps {
                        direction: fret_core::Axis::Vertical,
                        ..Default::default()
                    },
                    move |_cx| vec![filters, clear_row],
                )
                .test_id("runtime.toggle.footer.compact")
            } else {
                cx.flex(
                    fret_ui::element::FlexProps {
                        direction: fret_core::Axis::Horizontal,
                        ..Default::default()
                    },
                    move |_cx| vec![filters, clear],
                )
                .test_id("runtime.toggle.footer.roomy")
            };

            let mut page_props = fret_ui::element::FlexProps::default();
            page_props.direction = fret_core::Axis::Vertical;
            page_props.layout.size.width = Length::Fill;
            page_props.layout.size.height = Length::Fill;

            cx.flex(page_props, move |_cx| vec![body, footer]).into()
        }
    }

    struct ManualRuntimeLocalsWithRoot {
        app_ui_root: AppUiRenderRootState,
        count: Option<LocalState<u32>>,
        touched: Option<LocalState<bool>>,
        renders: Arc<AtomicUsize>,
        last_seen_count: Arc<AtomicUsize>,
    }

    impl Default for ManualRuntimeLocalsWithRoot {
        fn default() -> Self {
            Self {
                app_ui_root: AppUiRenderRootState::default(),
                count: None,
                touched: None,
                renders: Arc::new(AtomicUsize::new(0)),
                last_seen_count: Arc::new(AtomicUsize::new(0)),
            }
        }
    }

    fn render_manual_runtime_view(
        ui: &mut UiTree<crate::app::App>,
        app: &mut crate::app::App,
        services: &mut FakeUiServices,
        window: AppWindowId,
        bounds: Rect,
        st: &mut ManualRuntimeLocalsWithRoot,
    ) -> NodeId {
        let ManualRuntimeLocalsWithRoot {
            app_ui_root,
            count,
            touched,
            renders,
            last_seen_count,
        } = st;
        let root = render_root_with_app_ui(
            fret_ui::declarative::RenderRootContext::new(ui, app, services, window, bounds),
            "manual-locals-with-runtime",
            app_ui_root,
            |cx| {
                renders.fetch_add(1, Ordering::SeqCst);

                if count.is_none() {
                    *count = Some(cx.state().local_init(|| 0u32));
                }
                if touched.is_none() {
                    *touched = Some(cx.state().local_init(|| false));
                }

                let count = count.as_ref().expect("count local should exist").clone();
                let touched = touched
                    .as_ref()
                    .expect("touched local should exist")
                    .clone();

                cx.actions()
                    .locals_with((&count, &touched))
                    .on::<RuntimeIncrementAction>(|tx, (count, touched)| {
                        let incremented = tx.update_if(&count, |value| {
                            *value += 1;
                            true
                        });
                        let flagged = tx.set(&touched, true);
                        incremented || flagged
                    });

                last_seen_count.store(count.layout_value(cx) as usize, Ordering::SeqCst);

                let mut props = fret_ui::element::ContainerProps::default();
                props.layout.size.width = Length::Fill;
                props.layout.size.height = Length::Fill;

                let cx = cx.elements();
                cx.container(props, |_cx| Vec::new()).into()
            },
        );
        ui.set_root(root);
        ui.layout_all(app, services, bounds, 1.0);
        root
    }

    fn first_leaf(ui: &UiTree<crate::app::App>, mut node: NodeId) -> NodeId {
        loop {
            let children = ui.children(node);
            if children.is_empty() {
                return node;
            }
            node = children[0];
        }
    }

    #[test]
    fn local_state_value_in_helpers_clone_store_values() {
        let mut host = FakeHost::default();
        let local = LocalState {
            model: host.models.insert(String::from("hello")),
        };

        assert_eq!(local.value_in(&host.models), Some(String::from("hello")));
        assert_eq!(
            local.value_in_or(&host.models, String::from("fallback")),
            String::from("hello")
        );
        assert_eq!(
            LocalState {
                model: host.models.insert(String::new()),
            }
            .value_in_or_default(&host.models),
            String::new()
        );
    }

    #[test]
    fn local_state_from_model_wraps_existing_raw_handle() {
        let mut host = FakeHost::default();
        let model = host.models.insert(String::from("hello"));
        let local = LocalState::from_model(model.clone());

        assert_eq!(local.model(), &model);
        assert_eq!(local.value_in(&host.models), Some(String::from("hello")));
    }

    #[test]
    fn local_state_new_in_allocates_without_exposing_raw_model_handle() {
        let mut host = FakeHost::default();
        let local = LocalState::new_in(&mut host.models, String::from("hello"));

        assert_eq!(local.value_in(&host.models), Some(String::from("hello")));
    }

    #[test]
    fn app_local_state_constructor_allocates_without_exposing_model_store_callsite() {
        let mut app = crate::app::App::new();
        let local = app.local_state(String::from("hello"));

        assert_eq!(local.value_in(app.models()), Some(String::from("hello")));
    }

    #[test]
    fn local_state_borrowed_read_helpers_project_without_clone_noise() {
        let mut app = crate::app::App::new();
        let window = AppWindowId::default();
        let bounds = Rect::new(
            Point::new(Px(0.0), Px(0.0)),
            Size::new(Px(100.0), Px(100.0)),
        );
        let mut ui = UiTree::<crate::app::App>::new();
        ui.set_window(window);
        ui.set_view_cache_enabled(true);
        let mut services = FakeUiServices;
        let local = LocalState::from_model(app.models_mut().insert(vec![1u32, 2, 3]));

        let root = render_root_with_app_ui(
            fret_ui::declarative::RenderRootContext::new(
                &mut ui,
                &mut app,
                &mut services,
                window,
                bounds,
            ),
            "local-state-borrowed-read",
            &mut AppUiRenderRootState::default(),
            |cx| {
                let layout_len = local.layout_read_ref(cx, |values| values.len());
                let paint_len = local.paint_read_ref(cx, |values| values.len());
                assert_eq!(layout_len, 3);
                assert_eq!(paint_len, 3);
                let cx = cx.elements();
                cx.container(fret_ui::element::ContainerProps::default(), |_cx| {
                    Vec::new()
                })
                .into()
            },
        );
        ui.set_root(root);
        ui.layout_all(&mut app, &mut services, bounds, 1.0);
    }

    #[test]
    fn local_state_bridge_read_helpers_project_without_clone_noise() {
        let mut app = crate::app::App::new();
        let window = AppWindowId::default();
        let bounds = Rect::new(
            Point::new(Px(0.0), Px(0.0)),
            Size::new(Px(100.0), Px(100.0)),
        );
        let mut ui = UiTree::<crate::app::App>::new();
        ui.set_window(window);
        ui.set_view_cache_enabled(true);
        let mut services = FakeUiServices;
        let local = LocalState::from_model(app.models_mut().insert(vec![1u32, 2, 3]));

        let root = render_root(
            &mut ui,
            &mut app,
            &mut services,
            window,
            bounds,
            "local-state-bridge-read",
            |cx| {
                let layout_values = local.layout_value_in(cx);
                let layout_len = local.layout_read_ref_in(cx, |values| values.len());
                let paint_values = local.paint_value_in(cx);
                let paint_len = local.paint_read_ref_in(cx, |values| values.len());
                assert_eq!(layout_values, vec![1u32, 2, 3]);
                assert_eq!(layout_len, 3);
                assert_eq!(paint_values, vec![1u32, 2, 3]);
                assert_eq!(paint_len, 3);
                vec![
                    cx.container(fret_ui::element::ContainerProps::default(), |_cx| {
                        Vec::new()
                    })
                    .into(),
                ]
            },
        );
        ui.set_root(root);
        ui.layout_all(&mut app, &mut services, bounds, 1.0);
    }

    #[test]
    fn local_state_txn_value_reads_initialized_locals_without_fallback_noise() {
        let mut host = FakeHost::default();
        let draft = LocalState {
            model: host.models.insert(String::from("draft")),
        };
        let next_id = LocalState {
            model: host.models.insert(7u64),
        };

        let tx = LocalStateTxn {
            models: &mut host.models,
        };

        assert_eq!(tx.value(&draft), String::from("draft"));
        assert_eq!(tx.value(&next_id), 7u64);
    }

    #[test]
    fn local_action_capture_clones_local_state_handles_from_refs() {
        let mut host = FakeHost::default();
        let draft = LocalState {
            model: host.models.insert(String::from("draft")),
        };
        let next_id = LocalState {
            model: host.models.insert(7u64),
        };

        let (draft_capture, next_id_capture) = (&draft, &next_id).capture_owned();

        assert_eq!(
            draft_capture.value_in(&host.models),
            Some(String::from("draft"))
        );
        assert_eq!(next_id_capture.value_in(&host.models), Some(7u64));
    }

    #[test]
    fn local_action_capture_supports_wide_local_tuples() {
        let mut host = FakeHost::default();
        let a = LocalState {
            model: host.models.insert(1u64),
        };
        let b = LocalState {
            model: host.models.insert(2u64),
        };
        let c = LocalState {
            model: host.models.insert(3u64),
        };
        let d = LocalState {
            model: host.models.insert(4u64),
        };
        let e = LocalState {
            model: host.models.insert(5u64),
        };
        let f = LocalState {
            model: host.models.insert(6u64),
        };
        let g = LocalState {
            model: host.models.insert(7u64),
        };
        let h = LocalState {
            model: host.models.insert(8u64),
        };

        let captures = (&a, &b, &c, &d, &e, &f, &g, &h).capture_owned();

        assert_eq!(captures.0.value_in(&host.models), Some(1u64));
        assert_eq!(captures.1.value_in(&host.models), Some(2u64));
        assert_eq!(captures.2.value_in(&host.models), Some(3u64));
        assert_eq!(captures.3.value_in(&host.models), Some(4u64));
        assert_eq!(captures.4.value_in(&host.models), Some(5u64));
        assert_eq!(captures.5.value_in(&host.models), Some(6u64));
        assert_eq!(captures.6.value_in(&host.models), Some(7u64));
        assert_eq!(captures.7.value_in(&host.models), Some(8u64));
    }

    #[test]
    fn local_state_update_in_if_returns_closure_handled_state() {
        let mut host = FakeHost::default();
        let local = LocalState {
            model: host.models.insert(vec![1u64, 2, 3]),
        };

        assert!(local.update_in_if(&mut host.models, |values| {
            let before = values.len();
            values.retain(|value| *value != 2);
            values.len() != before
        }));
        assert_eq!(
            host.models
                .read(local.model(), |values| values.clone())
                .unwrap(),
            vec![1, 3]
        );
        assert!(!local.update_in_if(&mut host.models, |values| {
            let before = values.len();
            values.retain(|value| *value != 99);
            values.len() != before
        }));
    }

    #[test]
    fn local_state_update_action_requests_redraw_and_notify() {
        let mut host = FakeHost::default();
        let model = host.models.insert(1i32);
        let local = LocalState {
            model: model.clone(),
        };
        let action_cx = ActionCx {
            window: AppWindowId::default(),
            target: fret_ui::GlobalElementId(42),
        };

        assert!(local.update_action(&mut host, action_cx, |value| *value += 1));
        assert_eq!(host.models.read(&model, |value| *value).unwrap(), 2);
        assert_eq!(host.redraws, vec![action_cx.window]);
        assert_eq!(host.notifies, vec![action_cx]);
    }

    #[test]
    fn local_state_update_action_if_only_notifies_when_handled() {
        let mut host = FakeHost::default();
        let local = LocalState {
            model: host.models.insert(vec![1u64, 2, 3]),
        };
        let action_cx = ActionCx {
            window: AppWindowId::default(),
            target: fret_ui::GlobalElementId(7),
        };

        assert!(local.update_action_if(&mut host, action_cx, |values| {
            let before = values.len();
            values.retain(|value| *value != 2);
            values.len() != before
        }));
        assert_eq!(host.redraws, vec![action_cx.window]);
        assert_eq!(host.notifies, vec![action_cx]);

        host.redraws.clear();
        host.notifies.clear();
        assert!(!local.update_action_if(&mut host, action_cx, |values| {
            let before = values.len();
            values.retain(|value| *value != 99);
            values.len() != before
        }));
        assert!(host.redraws.is_empty());
        assert!(host.notifies.is_empty());
    }

    #[test]
    fn local_state_update_action_if_can_use_payload_from_closure() {
        let mut host = FakeHost::default();
        let local = LocalState {
            model: host.models.insert(vec![1u64, 2, 3]),
        };
        let action_cx = ActionCx {
            window: AppWindowId::default(),
            target: fret_ui::GlobalElementId(9),
        };

        assert!(local.update_action_if(&mut host, action_cx, |values| {
            let remove_id = 2u64;
            let before = values.len();
            values.retain(|value| *value != remove_id);
            values.len() != before
        }));
        assert_eq!(host.redraws, vec![action_cx.window]);
        assert_eq!(host.notifies, vec![action_cx]);
    }

    #[test]
    fn locals_with_runtime_dispatch_updates_locals_and_rerenders_cached_view() {
        let mut app = crate::app::App::new();
        let window = AppWindowId::default();
        let bounds = Rect::new(Point::new(Px(0.0), Px(0.0)), Size::new(Px(160.0), Px(80.0)));
        let mut ui = UiTree::<crate::app::App>::new();
        ui.set_window(window);
        ui.set_view_cache_enabled(true);

        let mut services = FakeUiServices;
        let mut st = view_init_window::<RuntimeLocalsWithView>(&mut app, window);

        app.set_frame_id(FrameId(1));
        let root = render_runtime_view(&mut ui, &mut app, &mut services, window, bounds, &mut st);
        ui.set_focus(Some(first_leaf(&ui, root)));
        assert!(
            st.cached_handlers.is_some(),
            "view render should install cached action handlers before command dispatch"
        );
        assert!(
            st.cached_action_root.is_some(),
            "view render should cache the concrete action root used for runtime dispatch"
        );
        assert_eq!(st.view.renders.load(Ordering::SeqCst), 1);
        assert_eq!(st.view.last_seen_count.load(Ordering::SeqCst), 0);

        app.set_frame_id(FrameId(2));
        render_runtime_view(&mut ui, &mut app, &mut services, window, bounds, &mut st);
        assert_eq!(
            st.view.renders.load(Ordering::SeqCst),
            1,
            "expected the view-cache root to reuse the previous frame before any notify-driven invalidation"
        );

        let command = <RuntimeIncrementAction as fret_runtime::TypedAction>::action_id();
        assert!(ui.dispatch_command(&mut app, &mut services, &command));
        assert_eq!(
            st.view
                .count
                .as_ref()
                .and_then(|local| local.value_in(app.models())),
            Some(1)
        );
        assert_eq!(
            st.view
                .touched
                .as_ref()
                .and_then(|local| local.value_in(app.models())),
            Some(true)
        );
        assert!(
            app.flush_effects()
                .iter()
                .any(|effect| matches!(effect, Effect::Redraw(redraw) if *redraw == window)),
            "locals_with action dispatch should request a redraw through the runtime host"
        );

        app.set_frame_id(FrameId(3));
        render_runtime_view(&mut ui, &mut app, &mut services, window, bounds, &mut st);
        assert_eq!(
            st.view.renders.load(Ordering::SeqCst),
            2,
            "notify should force the cached view root to rerender on the next frame"
        );
        assert_eq!(st.view.last_seen_count.load(Ordering::SeqCst), 1);
    }

    #[cfg(feature = "shadcn")]
    #[test]
    fn shadcn_button_action_keyboard_activation_dispatches_app_ui_action() {
        let mut app = crate::app::App::new();
        let window = AppWindowId::default();
        let bounds = Rect::new(Point::new(Px(0.0), Px(0.0)), Size::new(Px(240.0), Px(80.0)));
        let mut ui = UiTree::<crate::app::App>::new();
        ui.set_window(window);
        ui.set_view_cache_enabled(true);

        let mut services = FakeUiServices;
        let mut st = view_init_window::<RuntimeButtonActionView>(&mut app, window);

        app.set_frame_id(FrameId(1));
        let _root = render_runtime_button_action_view(
            &mut ui,
            &mut app,
            &mut services,
            window,
            bounds,
            &mut st,
        );
        let button = node_with_test_id(&mut ui, "test.runtime_button_action")
            .expect("button semantics node");
        ui.set_focus(Some(button));

        ui.dispatch_event(
            &mut app,
            &mut services,
            &fret_core::Event::KeyDown {
                key: fret_core::KeyCode::Enter,
                modifiers: Modifiers::default(),
                repeat: false,
            },
        );
        ui.dispatch_event(
            &mut app,
            &mut services,
            &fret_core::Event::KeyUp {
                key: fret_core::KeyCode::Enter,
                modifiers: Modifiers::default(),
            },
        );

        let command = <RuntimeButtonIncrementAction as fret_runtime::TypedAction>::action_id();
        let effects = app.flush_effects();
        assert!(
            effects.iter().any(
                |effect| matches!(effect, Effect::Command { command: seen, .. } if *seen == command)
            ),
            "keyboard activation should emit the button action command"
        );
        assert!(
            ui.dispatch_command(&mut app, &mut services, &command),
            "the emitted button action command should route to the AppUi action root"
        );
        assert_eq!(
            st.view
                .count
                .as_ref()
                .and_then(|local| local.value_in(app.models())),
            Some(1)
        );
    }

    #[test]
    fn app_ui_unit_action_handler_publishes_available_command_snapshot_by_default() {
        let mut app = crate::app::App::new();
        let window = AppWindowId::default();
        let bounds = Rect::new(Point::new(Px(0.0), Px(0.0)), Size::new(Px(160.0), Px(80.0)));
        let mut ui = UiTree::<crate::app::App>::new();
        ui.set_window(window);
        ui.set_view_cache_enabled(true);

        let command = <RuntimeIncrementAction as fret_runtime::TypedAction>::action_id();
        app.commands_mut().register(
            command.clone(),
            fret_runtime::CommandMeta::new("Runtime Increment")
                .with_scope(fret_runtime::CommandScope::Widget),
        );

        let mut services = FakeUiServices;
        let mut st = view_init_window::<RuntimeLocalsWithView>(&mut app, window);
        app.set_frame_id(FrameId(1));
        let _root = render_runtime_view(&mut ui, &mut app, &mut services, window, bounds, &mut st);
        assert_eq!(
            ui.focus(),
            None,
            "this regression covers command-palette/menu discovery before the app has a focused leaf"
        );
        ui.publish_window_runtime_snapshots(&mut app);

        let availability = app
            .global::<fret_runtime::WindowCommandActionAvailabilityService>()
            .and_then(|svc| svc.available(window, &command));
        assert_eq!(
            availability,
            Some(true),
            "registered unit action handlers should be command-palette/menu available by default"
        );

        assert!(
            ui.dispatch_command(&mut app, &mut services, &command),
            "registered unit action handlers should dispatch through the no-focus command route"
        );
        assert_eq!(
            st.view
                .count
                .as_ref()
                .and_then(|local| local.value_in(app.models())),
            Some(1)
        );
    }

    #[test]
    fn app_ui_unit_action_handler_publishes_available_snapshot_when_focus_exists() {
        let mut app = crate::app::App::new();
        let window = AppWindowId::default();
        let bounds = Rect::new(Point::new(Px(0.0), Px(0.0)), Size::new(Px(160.0), Px(80.0)));
        let mut ui = UiTree::<crate::app::App>::new();
        ui.set_window(window);
        ui.set_view_cache_enabled(true);

        let command = <RuntimeIncrementAction as fret_runtime::TypedAction>::action_id();
        app.commands_mut().register(
            command.clone(),
            fret_runtime::CommandMeta::new("Runtime Increment")
                .with_scope(fret_runtime::CommandScope::Widget),
        );

        let mut services = FakeUiServices;
        let mut st = view_init_window::<RuntimeLocalsWithView>(&mut app, window);
        app.set_frame_id(FrameId(1));
        let root = render_runtime_view(&mut ui, &mut app, &mut services, window, bounds, &mut st);
        ui.set_focus(Some(first_leaf(&ui, root)));
        ui.publish_window_runtime_snapshots(&mut app);

        let availability = app
            .global::<fret_runtime::WindowCommandActionAvailabilityService>()
            .and_then(|svc| svc.available(window, &command));
        assert_eq!(
            availability,
            Some(true),
            "view-level AppUi action handlers should stay available after focus moves into normal content"
        );

        assert!(
            ui.dispatch_command(&mut app, &mut services, &command),
            "focused-content command dispatch should still reach the AppUi action route fallback"
        );
        assert_eq!(
            st.view
                .count
                .as_ref()
                .and_then(|local| local.value_in(app.models())),
            Some(1)
        );
    }

    #[test]
    fn view_runtime_cache_enable_transition_keeps_toggle_group_footer_semantics_after_compact_resize()
     {
        let mut app = crate::app::App::new();
        let window = AppWindowId::default();
        let roomy_bounds = Rect::new(
            Point::new(Px(0.0), Px(0.0)),
            Size::new(Px(560.0), Px(660.0)),
        );
        let compact_bounds = Rect::new(
            Point::new(Px(0.0), Px(0.0)),
            Size::new(Px(420.0), Px(560.0)),
        );
        let mut ui = UiTree::<crate::app::App>::new();
        ui.set_window(window);
        ui.set_debug_enabled(true);

        let mut services = FakeUiServices;
        let mut st = view_init_window::<RuntimeToggleGroupFooterView>(&mut app, window);

        let _frame1 = render_runtime_view_semantics(
            &mut ui,
            &mut app,
            &mut services,
            window,
            roomy_bounds,
            2.0,
            1,
            "runtime-toggle-group-footer",
            &mut st,
        );

        ui.set_view_cache_enabled(true);

        let mut failures: Vec<String> = Vec::new();
        for frame_id in 2..=8 {
            let snapshot = render_runtime_view_semantics(
                &mut ui,
                &mut app,
                &mut services,
                window,
                compact_bounds,
                2.0,
                frame_id,
                "runtime-toggle-group-footer",
                &mut st,
            );
            let ids = snapshot_test_ids(&snapshot);
            for expected in [
                "runtime.toggle.body",
                "runtime.toggle.footer.compact",
                "runtime.toggle.clear_row",
                "runtime.toggle.clear",
                "runtime.toggle.filter.all",
                "runtime.toggle.filter.active",
                "runtime.toggle.filter.completed",
            ] {
                if !ids.iter().any(|id| id == expected) {
                    let cache_roots = ui.debug_cache_root_stats();
                    let removed = ui.debug_removed_subtrees();
                    failures.push(format!(
                        "frame{frame_id} should keep {expected} after runtime cache-enable transition; ids={ids:?}; cache_roots={cache_roots:?}; removed={removed:?}"
                    ));
                }
            }
        }

        if !failures.is_empty() {
            panic!("{}", failures.join("\n"));
        }
    }

    #[test]
    fn manual_render_root_with_app_ui_keeps_handlers_and_local_state_alive() {
        let mut app = crate::app::App::new();
        let window = AppWindowId::default();
        let bounds = Rect::new(Point::new(Px(0.0), Px(0.0)), Size::new(Px(160.0), Px(80.0)));
        let mut ui = UiTree::<crate::app::App>::new();
        ui.set_window(window);
        ui.set_view_cache_enabled(true);

        let mut services = FakeUiServices;
        let mut st = ManualRuntimeLocalsWithRoot::default();

        app.set_frame_id(FrameId(1));
        let root =
            render_manual_runtime_view(&mut ui, &mut app, &mut services, window, bounds, &mut st);
        ui.set_focus(Some(first_leaf(&ui, root)));
        assert!(
            st.app_ui_root.cached_handlers.is_some(),
            "manual AppUi root should install cached action handlers before command dispatch"
        );
        assert!(
            st.app_ui_root.cached_action_root.is_some(),
            "manual AppUi root should cache the concrete action root used for runtime dispatch"
        );
        assert_eq!(st.renders.load(Ordering::SeqCst), 1);
        assert_eq!(st.last_seen_count.load(Ordering::SeqCst), 0);

        app.set_frame_id(FrameId(2));
        render_manual_runtime_view(&mut ui, &mut app, &mut services, window, bounds, &mut st);
        assert_eq!(
            st.renders.load(Ordering::SeqCst),
            1,
            "expected the manual AppUi root to reuse the previous frame before any notify-driven invalidation"
        );

        let command = <RuntimeIncrementAction as fret_runtime::TypedAction>::action_id();
        assert!(ui.dispatch_command(&mut app, &mut services, &command));
        assert_eq!(
            st.count
                .as_ref()
                .and_then(|local| local.value_in(app.models())),
            Some(1)
        );
        assert_eq!(
            st.touched
                .as_ref()
                .and_then(|local| local.value_in(app.models())),
            Some(true)
        );
        assert!(
            app.flush_effects()
                .iter()
                .any(|effect| matches!(effect, Effect::Redraw(redraw) if *redraw == window)),
            "manual AppUi root dispatch should request a redraw through the runtime host"
        );

        app.set_frame_id(FrameId(3));
        render_manual_runtime_view(&mut ui, &mut app, &mut services, window, bounds, &mut st);
        assert_eq!(
            st.renders.load(Ordering::SeqCst),
            2,
            "notify should force the cached manual AppUi root to rerender on the next frame"
        );
        assert_eq!(st.last_seen_count.load(Ordering::SeqCst), 1);
    }

    #[cfg(feature = "state-mutation")]
    #[test]
    fn app_ui_data_update_after_mutation_completion_projects_terminal_state_once() {
        fn render_frame(
            app: &mut crate::app::App,
            ui: &mut UiTree<crate::app::App>,
            services: &mut FakeUiServices,
            window: AppWindowId,
            bounds: Rect,
            st: &mut AppUiRenderRootState,
            handle_cell: &RefCell<Option<fret_mutation::MutationHandle<u8, u8>>>,
            count_cell: &RefCell<Option<LocalState<u32>>>,
            status_cell: &RefCell<Option<LocalState<String>>>,
            frame_id: u64,
        ) {
            app.set_frame_id(FrameId(frame_id));
            let root = render_root_with_app_ui(
                fret_ui::declarative::RenderRootContext::new(ui, app, services, window, bounds),
                "mutation-completion-update",
                st,
                |cx| {
                    let applied_count = cx.state().local_init(|| 0u32);
                    let applied_status = cx.state().local_init(|| "Idle".to_string());
                    let handle = cx.data().mutation_async(
                        fret_mutation::MutationPolicy::default(),
                        |_token, input: Arc<u8>| async move {
                            if *input == 0 {
                                Err(fret_mutation::MutationError::transient("boom"))
                            } else {
                                Ok(*input)
                            }
                        },
                    );
                    if handle_cell.borrow().is_none() {
                        *handle_cell.borrow_mut() = Some(handle.clone());
                    }
                    if count_cell.borrow().is_none() {
                        *count_cell.borrow_mut() = Some(applied_count.clone());
                    }
                    if status_cell.borrow().is_none() {
                        *status_cell.borrow_mut() = Some(applied_status.clone());
                    }
                    let _ = cx.data().update_after_mutation_completion(
                        0xF123_2002,
                        &handle,
                        |models, st| {
                            let mut changed = false;
                            changed = applied_count
                                .update_in(models, |value| *value = value.saturating_add(1))
                                || changed;
                            let next_status = if st.is_success() {
                                "Success".to_string()
                            } else {
                                "Error".to_string()
                            };
                            changed = applied_status.set_in(models, next_status) || changed;
                            changed
                        },
                    );

                    let cx = cx.elements();
                    cx.container(fret_ui::element::ContainerProps::default(), |_cx| {
                        Vec::new()
                    })
                    .into()
                },
            );
            ui.set_root(root);
            ui.layout_all(app, services, bounds, 1.0);
        }

        let mut app = crate::app::App::new();
        let window = AppWindowId::default();
        let bounds = Rect::new(Point::new(Px(0.0), Px(0.0)), Size::new(Px(160.0), Px(80.0)));
        let mut ui = UiTree::<crate::app::App>::new();
        ui.set_window(window);
        let dispatcher: DispatcherHandle = Arc::new(TestDispatcher);
        app.set_global::<DispatcherHandle>(dispatcher);
        let spawner: fret_mutation::FutureSpawnerHandle = Arc::new(ReadyOnlySpawner);
        app.set_global::<fret_mutation::FutureSpawnerHandle>(spawner);

        let mut services = FakeUiServices;
        let mut st = AppUiRenderRootState::default();
        let handle_cell = RefCell::new(None::<fret_mutation::MutationHandle<u8, u8>>);
        let count_cell = RefCell::new(None::<LocalState<u32>>);
        let status_cell = RefCell::new(None::<LocalState<String>>);

        render_frame(
            &mut app,
            &mut ui,
            &mut services,
            window,
            bounds,
            &mut st,
            &handle_cell,
            &count_cell,
            &status_cell,
            1,
        );

        let handle = handle_cell
            .borrow()
            .as_ref()
            .expect("mutation handle should be captured")
            .clone();
        let applied_count = count_cell
            .borrow()
            .as_ref()
            .expect("applied_count local should be captured")
            .clone();
        let applied_status = status_cell
            .borrow()
            .as_ref()
            .expect("applied_status local should be captured")
            .clone();

        assert_eq!(applied_count.value_in_or_default(app.models_mut()), 0);
        assert_eq!(applied_status.value_in_or_default(app.models_mut()), "Idle");

        assert!(handle.submit(app.models_mut(), window, 0));
        assert!(drain_inboxes(&mut app, window));

        render_frame(
            &mut app,
            &mut ui,
            &mut services,
            window,
            bounds,
            &mut st,
            &handle_cell,
            &count_cell,
            &status_cell,
            2,
        );
        assert_eq!(applied_count.value_in_or_default(app.models_mut()), 1);
        assert_eq!(
            applied_status.value_in_or_default(app.models_mut()),
            "Error"
        );

        render_frame(
            &mut app,
            &mut ui,
            &mut services,
            window,
            bounds,
            &mut st,
            &handle_cell,
            &count_cell,
            &status_cell,
            3,
        );
        assert_eq!(
            applied_count.value_in_or_default(app.models_mut()),
            1,
            "same terminal completion should not reapply the projection"
        );

        assert!(handle.submit(app.models_mut(), window, 1));
        assert!(drain_inboxes(&mut app, window));

        render_frame(
            &mut app,
            &mut ui,
            &mut services,
            window,
            bounds,
            &mut st,
            &handle_cell,
            &count_cell,
            &status_cell,
            4,
        );
        assert_eq!(applied_count.value_in_or_default(app.models_mut()), 2);
        assert_eq!(
            applied_status.value_in_or_default(app.models_mut()),
            "Success"
        );
    }

    #[cfg(feature = "state-mutation")]
    #[test]
    fn app_ui_data_take_mutation_completion_only_fires_once_per_terminal_state() {
        fn render_frame(
            app: &mut crate::app::App,
            ui: &mut UiTree<crate::app::App>,
            services: &mut FakeUiServices,
            window: AppWindowId,
            bounds: Rect,
            st: &mut AppUiRenderRootState,
            handle_cell: &RefCell<Option<fret_mutation::MutationHandle<u8, u8>>>,
            completions_seen: &Arc<AtomicUsize>,
            frame_id: u64,
        ) {
            app.set_frame_id(FrameId(frame_id));
            let root = render_root_with_app_ui(
                fret_ui::declarative::RenderRootContext::new(ui, app, services, window, bounds),
                "mutation-completion-once",
                st,
                |cx| {
                    let handle = cx.data().mutation_async(
                        fret_mutation::MutationPolicy::default(),
                        |_token, input: Arc<u8>| async move {
                            if *input == 0 {
                                Err(fret_mutation::MutationError::transient("boom"))
                            } else {
                                Ok(*input)
                            }
                        },
                    );
                    if handle_cell.borrow().is_none() {
                        *handle_cell.borrow_mut() = Some(handle.clone());
                    }
                    if cx.data().take_mutation_completion(0xF123_2000, &handle) {
                        completions_seen.fetch_add(1, Ordering::SeqCst);
                    }

                    let cx = cx.elements();
                    cx.container(fret_ui::element::ContainerProps::default(), |_cx| {
                        Vec::new()
                    })
                    .into()
                },
            );
            ui.set_root(root);
            ui.layout_all(app, services, bounds, 1.0);
        }

        let mut app = crate::app::App::new();
        let window = AppWindowId::default();
        let bounds = Rect::new(Point::new(Px(0.0), Px(0.0)), Size::new(Px(160.0), Px(80.0)));
        let mut ui = UiTree::<crate::app::App>::new();
        ui.set_window(window);
        let dispatcher: DispatcherHandle = Arc::new(TestDispatcher);
        app.set_global::<DispatcherHandle>(dispatcher);
        let spawner: fret_mutation::FutureSpawnerHandle = Arc::new(ReadyOnlySpawner);
        app.set_global::<fret_mutation::FutureSpawnerHandle>(spawner);

        let mut services = FakeUiServices;
        let mut st = AppUiRenderRootState::default();
        let completions_seen = Arc::new(AtomicUsize::new(0));
        let handle_cell = RefCell::new(None::<fret_mutation::MutationHandle<u8, u8>>);

        render_frame(
            &mut app,
            &mut ui,
            &mut services,
            window,
            bounds,
            &mut st,
            &handle_cell,
            &completions_seen,
            1,
        );
        assert_eq!(completions_seen.load(Ordering::SeqCst), 0);

        let handle = handle_cell
            .borrow()
            .as_ref()
            .expect("mutation handle should be captured")
            .clone();

        assert!(handle.submit(app.models_mut(), window, 0));
        assert!(drain_inboxes(&mut app, window));

        render_frame(
            &mut app,
            &mut ui,
            &mut services,
            window,
            bounds,
            &mut st,
            &handle_cell,
            &completions_seen,
            2,
        );
        assert_eq!(completions_seen.load(Ordering::SeqCst), 1);

        render_frame(
            &mut app,
            &mut ui,
            &mut services,
            window,
            bounds,
            &mut st,
            &handle_cell,
            &completions_seen,
            3,
        );
        assert_eq!(
            completions_seen.load(Ordering::SeqCst),
            1,
            "same terminal completion should not retrigger on later renders"
        );

        assert!(handle.retry_last(app.models_mut(), window));
        assert!(drain_inboxes(&mut app, window));

        render_frame(
            &mut app,
            &mut ui,
            &mut services,
            window,
            bounds,
            &mut st,
            &handle_cell,
            &completions_seen,
            4,
        );
        assert_eq!(
            completions_seen.load(Ordering::SeqCst),
            2,
            "retrying the same stored input should still surface a fresh completion"
        );
    }

    #[cfg(feature = "state-mutation")]
    #[test]
    fn app_ui_data_take_mutation_success_only_fires_once_per_completion() {
        fn render_frame(
            app: &mut crate::app::App,
            ui: &mut UiTree<crate::app::App>,
            services: &mut FakeUiServices,
            window: AppWindowId,
            bounds: Rect,
            st: &mut AppUiRenderRootState,
            handle_cell: &RefCell<Option<fret_mutation::MutationHandle<(), ()>>>,
            completions_seen: &Arc<AtomicUsize>,
            frame_id: u64,
        ) {
            app.set_frame_id(FrameId(frame_id));
            let root = render_root_with_app_ui(
                fret_ui::declarative::RenderRootContext::new(ui, app, services, window, bounds),
                "mutation-success-once",
                st,
                |cx| {
                    let handle = cx.data().mutation_async(
                        fret_mutation::MutationPolicy::default(),
                        |_token, _input: Arc<()>| async { Ok(()) },
                    );
                    if handle_cell.borrow().is_none() {
                        *handle_cell.borrow_mut() = Some(handle.clone());
                    }
                    if cx.data().take_mutation_success(0xF123_2001, &handle) {
                        completions_seen.fetch_add(1, Ordering::SeqCst);
                    }

                    let cx = cx.elements();
                    cx.container(fret_ui::element::ContainerProps::default(), |_cx| {
                        Vec::new()
                    })
                    .into()
                },
            );
            ui.set_root(root);
            ui.layout_all(app, services, bounds, 1.0);
        }

        let mut app = crate::app::App::new();
        let window = AppWindowId::default();
        let bounds = Rect::new(Point::new(Px(0.0), Px(0.0)), Size::new(Px(160.0), Px(80.0)));
        let mut ui = UiTree::<crate::app::App>::new();
        ui.set_window(window);
        let dispatcher: DispatcherHandle = Arc::new(TestDispatcher);
        app.set_global::<DispatcherHandle>(dispatcher);
        let spawner: fret_mutation::FutureSpawnerHandle = Arc::new(ReadyOnlySpawner);
        app.set_global::<fret_mutation::FutureSpawnerHandle>(spawner);

        let mut services = FakeUiServices;
        let mut st = AppUiRenderRootState::default();
        let completions_seen = Arc::new(AtomicUsize::new(0));
        let handle_cell = RefCell::new(None::<fret_mutation::MutationHandle<(), ()>>);

        render_frame(
            &mut app,
            &mut ui,
            &mut services,
            window,
            bounds,
            &mut st,
            &handle_cell,
            &completions_seen,
            1,
        );
        assert_eq!(completions_seen.load(Ordering::SeqCst), 0);

        let handle = handle_cell
            .borrow()
            .as_ref()
            .expect("mutation handle should be captured")
            .clone();

        assert!(handle.submit(app.models_mut(), window, ()));
        assert!(drain_inboxes(&mut app, window));

        render_frame(
            &mut app,
            &mut ui,
            &mut services,
            window,
            bounds,
            &mut st,
            &handle_cell,
            &completions_seen,
            2,
        );
        assert_eq!(completions_seen.load(Ordering::SeqCst), 1);

        render_frame(
            &mut app,
            &mut ui,
            &mut services,
            window,
            bounds,
            &mut st,
            &handle_cell,
            &completions_seen,
            3,
        );
        assert_eq!(
            completions_seen.load(Ordering::SeqCst),
            1,
            "same mutation completion should not retrigger on later renders"
        );

        assert!(handle.submit(app.models_mut(), window, ()));
        assert!(drain_inboxes(&mut app, window));

        render_frame(
            &mut app,
            &mut ui,
            &mut services,
            window,
            bounds,
            &mut st,
            &handle_cell,
            &completions_seen,
            4,
        );
        assert_eq!(
            completions_seen.load(Ordering::SeqCst),
            2,
            "a new successful completion should retrigger exactly once"
        );
    }

    #[test]
    fn raw_model_with_reuses_element_context_local_model_substrate() {
        let api_source = view_authoring_api_source();

        assert!(api_source.contains("self.cx.local_model_at(callsite, init)"));
        assert!(
            api_source.contains("self.cx.note_repeated_call_in_render_evaluation_at(callsite)")
        );
        assert!(!api_source.contains("struct RawModelSlot<T>"));
        assert!(!api_source.contains("struct RawModelRenderPassDiagnostics"));
        assert!(!api_source.contains("fn note_raw_model_call_in_render_pass("));
    }

    #[test]
    fn local_state_owner_module_stays_private_with_view_reexports() {
        let view_api = VIEW_RS_SOURCE
            .split("\nmod tests {")
            .next()
            .expect("view.rs test module marker should exist");
        assert!(view_api.contains("mod local_state;"));
        assert!(view_api.contains("pub use local_state::{"));
        assert!(LOCAL_STATE_RS_SOURCE.contains("mod adapters;"));
        assert!(LOCAL_STATE_RS_SOURCE.contains("mod bridges;"));
        assert!(LOCAL_STATE_RS_SOURCE.contains("pub use bridges::{"));
        assert!(view_api.contains("LocalActionCapture"));
        assert!(view_api.contains("LocalStateTxn"));
        assert!(view_api.contains("LocalStateRawModelExt"));
        assert!(view_api.contains("LocalStateModelStoreExt"));
        assert!(view_api.contains("LocalStateElementContextExt"));
        assert!(view_api.contains("TrackedStateExt"));
        assert!(view_api.contains("WatchedState"));
        assert!(
            LOCAL_STATE_RS_SOURCE
                .contains("Local view-owned state for the app-facing `View` authoring lane.")
        );
        assert!(!view_api.contains("pub struct LocalState<T>"));
    }

    #[test]
    fn data_and_local_state_modules_stay_split_instead_of_regrowing_aggregators() {
        assert!(DATA_RS_SOURCE.contains("mod render;"));
        assert!(DATA_RS_SOURCE.contains("pub use render::{AppRenderData, AppRenderDataExt};"));
        assert!(DATA_RENDER_RS_SOURCE.contains("pub struct AppRenderData"));
        assert!(LOCAL_STATE_ADAPTERS_RS_SOURCE.contains("IntoBoolModel for LocalState<bool>"));
        assert!(LOCAL_STATE_BRIDGES_RS_SOURCE.contains("pub trait LocalStateRawModelExt<T>"));
        assert!(LOCAL_STATE_BRIDGES_RS_SOURCE.contains("pub trait LocalStateModelStoreExt<T>"));
        assert!(
            LOCAL_STATE_BRIDGES_RS_SOURCE.contains("pub trait LocalStateElementContextExt<T: Any>")
        );

        let data_lines = DATA_RS_SOURCE.lines().count();
        let local_state_lines = LOCAL_STATE_RS_SOURCE.lines().count();
        assert!(
            data_lines <= 760,
            "view/data.rs regrew to {data_lines} lines; split selector/query/mutation modules instead"
        );
        assert!(
            local_state_lines <= 780,
            "view/local_state.rs regrew to {local_state_lines} lines; keep adapters and advanced bridges split"
        );
    }

    #[test]
    fn payload_models_runtime_dispatch_updates_shared_models_and_requests_redraw() {
        let mut app = crate::app::App::new();
        let window = AppWindowId::default();
        let bounds = Rect::new(Point::new(Px(0.0), Px(0.0)), Size::new(Px(160.0), Px(80.0)));
        let mut ui = UiTree::<crate::app::App>::new();
        ui.set_window(window);

        let mut services = FakeUiServices;
        let selected_rows = app.models_mut().insert(Vec::<u64>::new());

        let root = render_root(
            &mut ui,
            &mut app,
            &mut services,
            window,
            bounds,
            "app-render-payload-models-runtime",
            |cx| {
                cx.actions().payload_models::<RuntimePayloadAppendAction>({
                    let selected_rows = selected_rows.clone();
                    move |models, row_id| {
                        models
                            .update(&selected_rows, |rows| rows.push(row_id))
                            .is_ok()
                    }
                });

                vec![
                    cx.container(fret_ui::element::ContainerProps::default(), |_cx| {
                        Vec::new()
                    })
                    .into(),
                ]
            },
        );
        ui.set_root(root);
        ui.layout_all(&mut app, &mut services, bounds, 1.0);

        let command = <RuntimePayloadAppendAction as fret_runtime::TypedAction>::action_id();
        app.with_global_mut(WindowPendingActionPayloadService::default, |svc, app| {
            svc.record(window, app.tick_id(), command.clone(), Box::new(41u64));
        });

        assert!(
            ui.dispatch_command(&mut app, &mut services, &command),
            "payload_models dispatch should be handled when a pending payload is present"
        );
        assert_eq!(
            app.models()
                .read(&selected_rows, |rows| rows.clone())
                .ok()
                .unwrap_or_default(),
            vec![41u64]
        );
        assert!(
            app.flush_effects()
                .iter()
                .any(|effect| matches!(effect, Effect::Redraw(redraw) if *redraw == window)),
            "handled payload_models dispatch should request redraw"
        );
    }

    #[cfg(feature = "shadcn")]
    #[test]
    fn checkbox_action_payload_round_trips_through_payload_models() {
        let mut app = crate::app::App::new();
        let window = AppWindowId::default();
        let bounds = Rect::new(Point::new(Px(0.0), Px(0.0)), Size::new(Px(160.0), Px(80.0)));
        let mut ui = UiTree::<crate::app::App>::new();
        ui.set_window(window);

        let mut services = FakeUiServices;
        let checkbox_checked = app.models_mut().insert(false);
        let selected_rows = app.models_mut().insert(Vec::<u64>::new());

        let root = render_root(
            &mut ui,
            &mut app,
            &mut services,
            window,
            bounds,
            "app-render-payload-models-checkbox",
            |cx| {
                cx.actions().payload_models::<RuntimePayloadAppendAction>({
                    let selected_rows = selected_rows.clone();
                    move |models, row_id| {
                        models
                            .update(&selected_rows, |rows| rows.push(row_id))
                            .is_ok()
                    }
                });

                vec![
                    fret_ui_shadcn::facade::Checkbox::new(checkbox_checked.clone())
                        .test_id("payload-checkbox")
                        .action(
                            <RuntimePayloadAppendAction as fret_runtime::TypedAction>::action_id(),
                        )
                        .action_payload(41u64)
                        .into_element(cx),
                ]
            },
        );
        ui.set_root(root);
        ui.request_semantics_snapshot();
        ui.layout_all(&mut app, &mut services, bounds, 1.0);

        let snap = ui.semantics_snapshot().expect("semantics snapshot");
        let checkbox = snap
            .nodes
            .iter()
            .find(|node| node.test_id.as_deref() == Some("payload-checkbox"))
            .expect("checkbox semantics node");
        let position = Point::new(
            Px(checkbox.bounds.origin.x.0 + checkbox.bounds.size.width.0 * 0.5),
            Px(checkbox.bounds.origin.y.0 + checkbox.bounds.size.height.0 * 0.5),
        );

        ui.dispatch_event(
            &mut app,
            &mut services,
            &fret_core::Event::Pointer(PointerEvent::Down {
                pointer_id: fret_core::PointerId(0),
                position,
                button: MouseButton::Left,
                modifiers: Modifiers::default(),
                pointer_type: PointerType::Mouse,
                click_count: 1,
            }),
        );
        ui.dispatch_event(
            &mut app,
            &mut services,
            &fret_core::Event::Pointer(PointerEvent::Up {
                pointer_id: fret_core::PointerId(0),
                position,
                button: MouseButton::Left,
                modifiers: Modifiers::default(),
                is_click: true,
                pointer_type: PointerType::Mouse,
                click_count: 1,
            }),
        );

        let mut saw_command = false;
        for effect in app.flush_effects() {
            match effect {
                Effect::Command {
                    window: Some(target_window),
                    command,
                } if target_window == window => {
                    saw_command = true;
                    assert!(
                        ui.dispatch_command(&mut app, &mut services, &command),
                        "checkbox payload command should be handled by app render payload_models"
                    );
                }
                other => app.push_effect(other),
            }
        }

        assert!(saw_command, "checkbox click should emit an Effect::Command");
        assert_eq!(
            app.models()
                .read(&selected_rows, |rows| rows.clone())
                .ok()
                .unwrap_or_default(),
            vec![41u64]
        );
    }

    #[test]
    fn dispatch_listener_queues_a_command_effect() {
        let mut host = FakeHost::default();
        let action_cx = ActionCx {
            window: AppWindowId::default(),
            target: fret_ui::GlobalElementId(17),
        };

        let dispatch = dispatch_action_listener::<DispatchAction>();
        dispatch(&mut host, action_cx, ActivateReason::Pointer);

        assert_eq!(
            host.effects,
            vec![Effect::Command {
                window: Some(action_cx.window),
                command: <DispatchAction as fret_runtime::TypedAction>::action_id(),
            }]
        );
        assert_eq!(
            host.dispatch_sources,
            vec![(
                action_cx,
                <DispatchAction as fret_runtime::TypedAction>::action_id(),
                ActivateReason::Pointer
            )]
        );
    }

    #[test]
    fn dispatch_payload_listener_records_payload_before_dispatch() {
        let mut host = FakeHost::default();
        let action_cx = ActionCx {
            window: AppWindowId::default(),
            target: fret_ui::GlobalElementId(23),
        };

        let dispatch = dispatch_payload_action_listener::<DispatchPayloadAction>(42);
        dispatch(&mut host, action_cx, ActivateReason::Keyboard);

        assert_eq!(
            host.effects,
            vec![Effect::Command {
                window: Some(action_cx.window),
                command: <DispatchPayloadAction as fret_runtime::TypedAction>::action_id(),
            }]
        );
        assert_eq!(host.payloads.len(), 1);
        assert_eq!(host.payloads[0].0, action_cx);
        assert_eq!(
            host.payloads[0].1,
            <DispatchPayloadAction as fret_runtime::TypedAction>::action_id()
        );
        assert_eq!(host.payloads[0].2.downcast_ref::<u64>().copied(), Some(42));
    }

    #[test]
    fn action_listener_hides_activate_reason_for_simple_widget_glue() {
        let mut host = FakeHost::default();
        let action_cx = ActionCx {
            window: AppWindowId::default(),
            target: fret_ui::GlobalElementId(31),
        };

        let listener = action_listener(move |host, cx| {
            host.request_redraw(cx.window);
            host.notify(cx);
        });
        listener(&mut host, action_cx, ActivateReason::Keyboard);

        assert_eq!(host.redraws, vec![action_cx.window]);
        assert_eq!(host.notifies, vec![action_cx]);
    }

    #[test]
    fn app_activate_surface_contract_can_store_activation_handlers() {
        let widget = DummyActivateSurface::default().on_activate(action_listener(|host, cx| {
            host.request_redraw(cx.window);
        }));
        assert!(widget.on_activate.is_some());
    }

    #[test]
    fn app_activate_ext_action_alias_dispatches_without_turbofish() {
        let widget = DummyActivateSurface::default().action(DispatchAction);
        let dispatch = widget
            .on_activate
            .expect("action alias should store an activation handler");
        let mut host = FakeHost::default();
        let action_cx = ActionCx {
            window: AppWindowId::default(),
            target: fret_ui::GlobalElementId(77),
        };

        dispatch(&mut host, action_cx, ActivateReason::Pointer);

        assert_eq!(
            host.effects,
            vec![Effect::Command {
                window: Some(action_cx.window),
                command: <DispatchAction as fret_runtime::TypedAction>::action_id(),
            }]
        );
    }

    #[test]
    fn app_activate_ext_action_payload_alias_records_payload_without_turbofish() {
        let widget = DummyActivateSurface::default().action_payload(DispatchPayloadAction, 9);
        let dispatch = widget
            .on_activate
            .expect("action_payload alias should store an activation handler");
        let mut host = FakeHost::default();
        let action_cx = ActionCx {
            window: AppWindowId::default(),
            target: fret_ui::GlobalElementId(88),
        };

        dispatch(&mut host, action_cx, ActivateReason::Keyboard);

        assert_eq!(host.payloads.len(), 1);
        assert_eq!(host.payloads[0].0, action_cx);
        assert_eq!(
            host.payloads[0].1,
            <DispatchPayloadAction as fret_runtime::TypedAction>::action_id()
        );
        assert_eq!(host.payloads[0].2.downcast_ref::<u64>().copied(), Some(9));
    }

    #[cfg(feature = "shadcn")]
    #[test]
    fn local_state_supports_text_value_widgets() {
        let mut host = FakeHost::default();
        let local = LocalState {
            model: host.models.insert(String::from("hello")),
        };

        let _input = fret_ui_shadcn::facade::Input::new(&local);
        let _textarea = fret_ui_shadcn::facade::Textarea::new(&local);
    }

    #[test]
    fn grouped_authoring_surfaces_replace_flat_app_ui_helpers() {
        let api_source = view_authoring_api_source();
        assert!(!api_source.contains("pub fn use_state<"));
        assert!(!api_source.contains("pub fn use_state_keyed<"));
        assert!(!api_source.contains("fn use_state_keyed<"));
        assert!(!api_source.contains("pub fn raw_model<"));
        assert!(!api_source.contains("pub fn use_local<"));
        assert!(!api_source.contains("pub fn use_local_keyed<"));
        assert!(!api_source.contains("pub fn use_local_with<"));
        assert!(!api_source.contains("pub fn on_action_notify_local_update<"));
        assert!(!api_source.contains("pub fn on_action_notify_local_set<"));
        assert!(!api_source.contains("pub fn on_action_notify_toggle_local_bool<"));
        assert!(!api_source.contains("pub fn on_action_notify_models<"));
        assert!(!api_source.contains("pub fn on_action_notify_locals<"));
        assert!(!api_source.contains("pub fn on_action_notify_transient<"));
        assert!(!api_source.contains("fn on_action<A: crate::TypedAction>("));
        assert!(
            !api_source.contains("fn on_payload_action<A: crate::actions::TypedPayloadAction>(")
        );
        assert!(!api_source.contains("fn on_action_availability<A: crate::TypedAction>("));
        assert!(!api_source.contains("fn on_action_notify_model_update<"));
        assert!(!api_source.contains("fn on_action_notify_model_set<"));
        assert!(!api_source.contains("fn on_action_notify_toggle_bool<"));
        assert!(!api_source.contains("pub fn on_payload_action_notify_local_update_if<"));
        assert!(!api_source.contains("pub fn on_payload_action_notify_locals<"));
        assert!(!api_source.contains("pub struct AppUiPayloadActions<"));
        assert!(!api_source.contains("pub struct UiCxPayloadActions<"));
        assert!(!api_source.contains("pub fn payload_locals<"));
        assert!(!api_source.contains("pub fn payload<A>(self) -> AppUiPayloadActions"));
        assert!(!api_source.contains("pub fn payload<A>(self) -> UiCxPayloadActions"));
        assert!(!api_source.contains("pub fn local_update_if<T>("));
        assert!(!api_source.contains(
            "pub fn locals(self, f: impl for<'m> Fn(&mut LocalStateTxn<'m>, A::Payload) -> bool + 'static)"
        ));
        assert!(
            api_source.contains("pub fn value<T: Any + Clone>(&self, local: &LocalState<T>) -> T")
        );
        assert!(
            api_source
                .contains("pub fn layout_value<'a, H: UiHost + 'a, Cx>(&self, cx: &mut Cx) -> T")
        );
        assert!(
            api_source
                .contains("pub fn paint_value<'a, H: UiHost + 'a, Cx>(&self, cx: &mut Cx) -> T")
        );
        assert!(api_source.contains("pub fn layout_read_ref<'a, H: UiHost + 'a, Cx, R>("));
        assert!(api_source.contains("pub fn paint_read_ref<'a, H: UiHost + 'a, Cx, R>("));
        assert!(!LOCAL_STATE_RS_SOURCE.contains("pub trait LocalStateRawModelExt"));
        assert!(!LOCAL_STATE_RS_SOURCE.contains("pub trait LocalStateModelStoreExt"));
        assert!(!LOCAL_STATE_RS_SOURCE.contains("pub trait LocalStateElementContextExt"));
        assert!(!LOCAL_STATE_RS_SOURCE.contains("pub fn from_model("));
        assert!(!LOCAL_STATE_RS_SOURCE.contains("pub fn model(&self) -> &Model<T>"));
        assert!(!LOCAL_STATE_RS_SOURCE.contains("pub fn clone_model(&self) -> Model<T>"));
        assert!(!LOCAL_STATE_RS_SOURCE.contains("pub fn read_in<R>("));
        assert!(!LOCAL_STATE_RS_SOURCE.contains("pub fn update_in("));
        assert!(!LOCAL_STATE_RS_SOURCE.contains("pub fn set_in("));
        assert!(!LOCAL_STATE_RS_SOURCE.contains("pub fn watch_in<"));
        assert!(!LOCAL_STATE_RS_SOURCE.contains("pub fn layout_value_in<"));
        assert!(!LOCAL_STATE_RS_SOURCE.contains("pub fn paint_value_in<"));
        assert!(api_source.contains("pub trait LocalStateRawModelExt<T>"));
        assert!(api_source.contains("pub trait LocalStateModelStoreExt<T>"));
        assert!(api_source.contains("pub trait LocalStateElementContextExt<T: Any>"));
        assert!(api_source.contains("fn layout_value_in<'cx, 'm, 'a, H: UiHost>("));
        assert!(api_source.contains("fn layout_read_ref_in<'cx, 'm, 'a, H: UiHost, R>("));
        assert!(api_source.contains("fn paint_value_in<'cx, 'm, 'a, H: UiHost>("));
        assert!(api_source.contains("fn paint_read_ref_in<'cx, 'm, 'a, H: UiHost, R>("));
        assert!(!api_source.contains("pub fn use_selector<"));
        assert!(!api_source.contains("pub fn use_selector_keyed<"));
        assert!(!api_source.contains("pub fn use_query<"));
        assert!(!api_source.contains("pub fn use_query_async<"));
        assert!(!api_source.contains("pub fn use_query_async_local<"));
        assert!(!api_source.contains("pub fn take_transient_on_action_root("));
        assert!(!api_source.contains("pub type WatchedModel"));
        assert!(!api_source.contains("pub type WatchedLocal"));
        assert!(!api_source.contains("pub fn update_action("));
        assert!(!api_source.contains("pub fn update_action_if("));
        assert!(!api_source.contains("pub fn set_action("));
        assert!(!api_source.contains("pub trait LocalSelectorDepsBuilderExt"));
        assert!(api_source.contains("pub(crate) trait LocalSelectorDepsBuilderExt"));
        assert!(api_source.contains("pub trait LocalSelectorLayoutInputs"));
        assert!(api_source.contains("pub trait ModelSelectorInputs"));
        assert!(api_source.contains("pub trait QueryHandleReadLayoutExt<T: 'static>"));
        assert!(!api_source.contains("pub trait AppUiRawStateExt"));
        assert!(api_source.contains("pub trait AppUiRawModelExt"));
        assert!(api_source.contains("pub trait AppUiRawActionNotifyExt"));
        assert!(
            api_source.contains("pub trait AppUiComponentLaneRequiresExplicitElementsEscapeHatch")
        );
        assert!(api_source.contains("pub trait RenderContextAccess<'a, H: UiHost + 'a>"));
        assert!(api_source.contains("pub trait AppRenderDataExt"));
        assert!(api_source.contains("pub trait AppRenderActionsExt"));
        assert!(!api_source.contains("pub fn watch_local<'m, T: Any>("));
        assert!(api_source.contains("pub(crate) fn watch_local<'m, T: Any>("));
        assert!(!api_source.contains("pub fn action_root(&self) -> fret_ui::GlobalElementId"));
        assert!(!api_source.contains("pub fn new(cx: &'cx mut ElementContext<'a, H>, action_root: fret_ui::GlobalElementId) -> Self"));
        assert!(api_source.contains("pub(crate) fn new("));
        assert!(api_source.contains("pub fn actions(&mut self) -> AppUiActions"));
        assert!(api_source.contains(
            "impl<'cx, 'a, H: UiHost> fret_ui_kit::command::ElementCommandGatingExt for AppUi<'cx, 'a, H> {"
        ));
        assert!(api_source.contains(
            "pub fn request_animation_frame(&mut self) {\n        self.cx.request_animation_frame();\n    }"
        ));
        assert!(api_source.contains("pub fn layout_query_bounds("));
        assert!(api_source.contains("pub fn layout_query_region_with_id<I>("));
        assert!(api_source.contains("pub fn layout_query_region<I>("));
        assert!(api_source.contains(
            "let mut carried_action_handlers = Some(std::mem::take(&mut self.action_handlers));"
        ));
        assert!(api_source.contains("self.cx.layout_query_region_with_id(props, |cx, id| {"));
        assert!(api_source.contains("pub fn scope<R>(&mut self, _f: impl FnOnce(&mut Self) -> R)"));
        assert!(
            api_source.contains(
                "pub fn named<R>(&mut self, _name: &str, _f: impl FnOnce(&mut Self) -> R)"
            )
        );
        assert!(api_source.contains(
            "pub fn slot_state<S: Any, R>(&mut self, _init: impl FnOnce() -> S, _f: impl FnOnce(&mut S) -> R)"
        ));
        assert!(api_source.contains(
            "pub fn local_model<T: Any>(&mut self, _init: impl FnOnce() -> T)\n    where"
        ));
        assert!(api_source.contains(
            "pub fn local_model_keyed<K: Hash, T: Any>(&mut self, _key: K, _init: impl FnOnce() -> T)"
        ));
        assert!(api_source.contains("pub fn state_for<S: Any, R>("));
        assert!(
            api_source.contains("Self: AppUiComponentLaneRequiresExplicitElementsEscapeHatch,")
        );
        assert!(api_source.contains(
            "pub fn local<T>(self, local: &LocalState<T>) -> AppUiActionLocal<'view, 'cx, 'a, H, T>"
        ));
        assert!(!api_source.contains("pub fn local_update<A, T>("));
        assert!(!api_source.contains("pub fn local_set<A, T>("));
        assert!(!api_source.contains("pub fn toggle_local_bool<A>("));
        assert!(!api_source.contains("pub fn payload_local_update_if<A, T>("));
        assert!(api_source.contains("fn read_layout<'a, H: UiHost + 'a, Cx>("));
        assert!(api_source.contains("pub fn selector_layout<Inputs, TValue>("));
        assert!(api_source.contains("pub fn selector_model_layout<Inputs, TValue>("));
        assert!(api_source.contains("pub fn selector_model_paint<Inputs, TValue>("));
        assert!(!api_source.contains("pub fn selector_layout_keyed<K: Hash, Inputs, TValue>("));
        assert!(!api_source.contains("pub fn selector_keyed<K: Hash, Deps, TValue>("));
        assert!(
            api_source
                .contains("pub fn query_snapshot(self) -> Option<fret_query::QueryClientSnapshot>")
        );
        assert!(
            api_source.contains("pub fn query_snapshot_entry<T: Any + Send + Sync + 'static>(")
        );
        assert!(api_source.contains("pub fn cancel_query<T: Any + Send + Sync + 'static>(self, key: fret_query::QueryKey<T>)"));
        assert!(api_source.contains("pub fn invalidate_query<T: Any + Send + Sync + 'static>("));
        assert!(
            api_source.contains("pub fn invalidate_query_namespace(self, namespace: &'static str)")
        );
        assert!(
            api_source.contains("pub fn take_mutation_completion<TIn: 'static, TOut: 'static>(")
        );
        assert!(
            api_source
                .contains("pub fn update_after_mutation_completion<TIn: 'static, TOut: 'static>(")
        );
        assert!(api_source.contains(
            "pub fn update_locals_after_mutation_completion<TIn: 'static, TOut: 'static>("
        ));
        assert!(api_source.contains("pub fn take_mutation_success<TIn: 'static, TOut: 'static>("));
        assert!(api_source.contains("pub fn invalidate_query_after_mutation_success<"));
        assert!(api_source.contains(
            "pub fn invalidate_query_namespace_after_mutation_success<TIn: 'static, TOut: 'static>("
        ));
        assert!(api_source.contains("pub fn mutation_submit<A, TIn, TOut>("));
        assert!(api_source.contains("pub fn mutation_retry_last<A, TIn, TOut>("));
        assert!(api_source.contains("pub fn toast_message("));
        assert!(api_source.contains("pub fn toast_success("));
        assert!(api_source.contains("pub fn toast_error("));
        assert!(api_source.contains("pub fn toast_dismiss_all("));
        assert!(api_source.contains("pub trait AppActivateSurface"));
        assert!(api_source.contains("pub trait AppActivateExt"));
        assert!(!api_source.contains("pub trait AppActivateCxMarker"));
        assert!(!api_source.contains("AppActivateCxMarker for AppUi"));
        assert!(!api_source.contains("AppActivateCxMarker for ElementContext"));
        assert!(api_source.contains("fn action<A>(self, _action: A) -> Self"));
        assert!(
            api_source
                .contains("fn action_payload<A>(self, _action: A, payload: A::Payload) -> Self")
        );
        assert!(api_source.contains(
            "fn listen(self, f: impl Fn(&mut dyn UiActionHost, ActionCx) + 'static) -> Self"
        ));
        assert!(!api_source.contains("pub fn action<A>(self, _action: A) -> OnActivate"));
        assert!(!api_source.contains(
            "pub fn action_payload<A>(self, _action: A, payload: A::Payload) -> OnActivate"
        ));
        assert!(!api_source.contains("fn dispatch<A>(self) -> Self"));
        assert!(!api_source.contains("fn dispatch_payload<A>(self, payload: A::Payload) -> Self"));
        assert!(!api_source.contains("pub fn dispatch<A>(self) -> OnActivate"));
        assert!(
            !api_source
                .contains("pub fn dispatch_payload<A>(self, payload: A::Payload) -> OnActivate")
        );
        assert!(api_source.contains("pub fn listen("));
        assert!(api_source.contains(
            "pub fn payload_models<A>(\n        self,\n        f: impl Fn(&mut fret_runtime::ModelStore, A::Payload) -> bool + 'static,"
        ));
        assert!(
            api_source.contains(
                "#[doc(hidden)]\npub struct AppUiActionLocal<'view, 'cx, 'a, H: UiHost, T>"
            )
        );
        assert!(api_source.contains("#[doc(hidden)]\npub struct AppRenderActionLocal<'cx, 'a, T>"));
        assert!(api_source.contains("pub fn update<A>(self, update: impl Fn(&mut T) + 'static)"));
        assert!(api_source.contains("pub fn set<A>(self, value: T)"));
        assert!(api_source.contains("pub fn toggle_bool<A>(self)"));
        assert!(api_source.contains("pub fn payload_update_if<A>("));
        assert!(api_source.contains("pub fn locals_with<C>("));
        assert!(api_source.contains(
            "pub fn on<A>(self, f: impl for<'m> Fn(&mut LocalStateTxn<'m>, C) -> bool + 'static)"
        ));
        assert!(api_source.contains(
            "pub fn availability<A>(\n        self,\n        f: impl for<'m> Fn(&mut LocalStateTxn<'m>, C) -> fret_ui::CommandAvailability"
        ));
        assert!(!api_source.contains(
            "pub fn locals<A>(self, f: impl for<'m> Fn(&mut LocalStateTxn<'m>) -> bool + 'static)"
        ));
        assert!(!api_source.contains("pub fn listener("));
        assert!(!api_source.contains("impl AppActivateSurface for fret_ui_shadcn::facade::Button"));
        assert!(
            !api_source
                .contains("impl AppActivateSurface for fret_ui_shadcn::facade::SidebarMenuButton")
        );
        assert!(!api_source.contains("impl AppActivateSurface for fret_ui_shadcn::facade::Badge"));
        assert!(
            !api_source
                .contains("impl AppActivateSurface for fret_ui_shadcn::raw::extras::BannerAction")
        );
        assert!(
            !api_source
                .contains("impl AppActivateSurface for fret_ui_shadcn::raw::extras::BannerClose")
        );
        assert!(
            !api_source.contains("impl AppActivateSurface for fret_ui_shadcn::raw::extras::Ticker")
        );
        assert!(!api_source.contains("impl AppActivateSurface for fret_ui_material3::Card"));
        assert!(
            !api_source.contains("impl AppActivateSurface for fret_ui_material3::DialogAction")
        );
        assert!(
            !api_source.contains("impl AppActivateSurface for fret_ui_material3::TopAppBarAction")
        );
        assert!(api_source.contains("pub fn data(&mut self) -> AppUiData"));
        assert!(api_source.contains("pub fn effects(&mut self) -> AppUiEffects"));
        assert!(!api_source.contains("pub trait AppActionCxSurface"));
        assert!(!api_source.contains("pub trait AppActionCxExt"));
        assert!(
            !api_source.contains("impl AppActivateSurface for fret_ui_ai::WorkflowControlsButton")
        );
        assert!(!api_source.contains("impl AppActivateSurface for fret_ui_ai::MessageAction"));
        assert!(!api_source.contains("impl AppActivateSurface for fret_ui_ai::ArtifactClose"));
        assert!(!api_source.contains("impl AppActivateSurface for fret_ui_ai::CheckpointTrigger"));
        assert!(!api_source.contains("impl AppActivateSurface for fret_ui_ai::ArtifactAction"));
        assert!(!api_source.contains("impl AppActivateSurface for fret_ui_ai::ConfirmationAction"));
        assert!(
            !api_source.contains("impl AppActivateSurface for fret_ui_ai::ConversationDownload")
        );
        assert!(!api_source.contains("impl AppActivateSurface for fret_ui_ai::PromptInputButton"));
        assert!(
            !api_source
                .contains("impl AppActivateSurface for fret_ui_ai::WebPreviewNavigationButton")
        );
        assert!(!api_source.contains("impl AppActivateSurface for fret_ui_ai::Attachment"));
        assert!(!api_source.contains("impl AppActivateSurface for fret_ui_ai::QueueItemAction"));
        assert!(!api_source.contains("impl AppActivateSurface for fret_ui_ai::Test"));
        assert!(!api_source.contains("impl AppActivateSurface for fret_ui_ai::FileTreeAction"));
        assert!(!api_source.contains("impl AppActivateSurface for fret_ui_ai::Suggestion"));
        assert!(!api_source.contains("impl AppActivateSurface for fret_ui_ai::MessageBranch"));
        assert!(
            !api_source.contains("impl AppActivateSurface for fret_ui_ai::TerminalClearButton")
        );
    }

    #[test]
    fn structural_grouped_carriers_stay_hidden_from_first_contact_rustdoc() {
        let api_source = view_authoring_api_source();

        assert!(api_source.contains("#[doc(hidden)]\npub struct LocalStateTxn<'a>"));
        assert!(api_source.contains("#[doc(hidden)]\npub trait LocalActionCapture"));
        assert!(
            api_source.contains(
                "#[doc(hidden)]\npub struct AppUiLocalsWith<'view, 'cx, 'a, H: UiHost, C>"
            )
        );
        assert!(api_source.contains("#[doc(hidden)]\npub struct AppRenderLocalsWith<'cx, 'a, C>"));
        assert!(
            api_source.contains("#[doc(hidden)]\npub struct AppUiState<'view, 'cx, 'a, H: UiHost>")
        );
        assert!(
            api_source
                .contains("#[doc(hidden)]\npub struct AppUiActions<'view, 'cx, 'a, H: UiHost>")
        );
        assert!(api_source.contains("#[doc(hidden)]\npub struct AppRenderActions<'cx, 'a>"));
        assert!(
            api_source.contains("#[doc(hidden)]\npub struct AppUiData<'view, 'cx, 'a, H: UiHost>")
        );
        assert!(api_source.contains("#[doc(hidden)]\npub struct AppRenderData<'cx, 'a>"));
        assert!(
            api_source
                .contains("#[doc(hidden)]\npub struct AppUiEffects<'view, 'cx, 'a, H: UiHost>")
        );
        assert!(!api_source.contains("#[doc(hidden)]\npub struct LocalState<T>"));
        assert!(!api_source.contains("#[doc(hidden)]\npub trait TrackedStateExt<T: Any>"));
        assert!(!api_source.contains("#[doc(hidden)]\npub trait AppActivateExt"));
    }

    #[test]
    fn tracked_read_builder_stays_visible_while_structural_carriers_hide() {
        let api_source = view_authoring_api_source();

        assert!(
            api_source
                .contains("#[must_use]\npub struct WatchedState<'cx, 'm, 'a, H: UiHost, T: Any>")
        );
        assert!(!api_source.contains(
            "#[doc(hidden)]\n#[must_use]\npub struct WatchedState<'cx, 'm, 'a, H: UiHost, T: Any>"
        ));
        assert!(api_source.contains(
            "Prefer `LocalState::layout_value(...)` / `paint_value(...)` for ordinary initialized app-lane"
        ));
    }

    #[test]
    fn local_state_docs_classify_default_and_bridge_surfaces() {
        let api_source = view_authoring_api_source();
        assert!(api_source.contains("Default app-facing handle for view-owned local state."));
        assert!(api_source.contains("Insert a new view-owned local slot into an existing"));
        assert!(
            api_source
                .contains("Explicit raw `Model<T>` bridge for advanced/component/hybrid surfaces.")
        );
        assert!(api_source.contains(
            "Explicit `ModelStore` bridge for advanced transactions and manual/hybrid surfaces."
        ));
        assert!(api_source.contains(
            "Explicit `ElementContext` bridge for helper-heavy component or advanced surfaces."
        ));
        assert!(api_source.contains(
            "Default app code should prefer `state.layout_value(cx)` / `state.paint_value(cx)`"
        ));
        assert!(api_source.contains(
            "Read the current local value through a layout invalidation tracked read on the default app"
        ));
        assert!(api_source.contains(
            "Read a derived value from this local through a layout invalidation tracked borrow on the"
        ));
        assert!(api_source.contains(
            "Read the current local value through a paint invalidation tracked read on the default app"
        ));
        assert!(api_source.contains(
            "Read a derived value from this local through a paint invalidation tracked borrow on the"
        ));
        assert!(api_source.contains(
            "This trait is intentionally omitted from `fret::app::prelude::*` and reexported from"
        ));
        assert!(api_source.contains("`fret::advanced::prelude::*`."));
    }

    #[test]
    fn app_ui_keeps_raw_element_lane_explicit() {
        let api_source = view_authoring_api_source();
        assert!(api_source.contains(
            "`AppUi` intentionally does not implement `Deref<Target = ElementContext<...>>`."
        ));
        assert!(api_source.contains("app-facing render-authoring"));
        assert!(api_source.contains("raw `ElementContext`"));
        assert!(!api_source.contains("std::ops::Deref for AppUi"));
        assert!(!api_source.contains("std::ops::DerefMut for AppUi"));
    }

    #[test]
    fn app_ui_keeps_command_gating_and_animation_frame_surface_without_deref() {
        fn assert_command_gating_impl<T: fret_ui_kit::command::ElementCommandGatingExt>() {}

        assert_command_gating_impl::<crate::AppUi<'static, 'static>>();

        let _request_animation_frame: fn(&mut crate::AppUi<'static, 'static>) =
            crate::AppUi::request_animation_frame;
        let _set_continuous_frames: fn(&mut crate::AppUi<'static, 'static>, bool) =
            crate::AppUi::set_continuous_frames;
        let _layout_query_bounds: fn(
            &mut crate::AppUi<'static, 'static>,
            fret_ui::GlobalElementId,
            fret_ui::Invalidation,
        ) -> Option<fret_core::Rect> = crate::AppUi::layout_query_bounds;
        let _layout_query_region_with_id: fn(
            &mut crate::AppUi<'static, 'static>,
            fret_ui::element::LayoutQueryRegionProps,
            for<'b> fn(
                &mut crate::AppUi<'b, 'static>,
                fret_ui::GlobalElementId,
            ) -> std::vec::Vec<fret_ui::element::AnyElement>,
        ) -> fret_ui::element::AnyElement = crate::AppUi::layout_query_region_with_id::<
            std::vec::Vec<fret_ui::element::AnyElement>,
        >;
        let _layout_query_region: fn(
            &mut crate::AppUi<'static, 'static>,
            fret_ui::element::LayoutQueryRegionProps,
            for<'b> fn(
                &mut crate::AppUi<'b, 'static>,
            ) -> std::vec::Vec<fret_ui::element::AnyElement>,
        ) -> fret_ui::element::AnyElement =
            crate::AppUi::layout_query_region::<std::vec::Vec<fret_ui::element::AnyElement>>;
    }
}
