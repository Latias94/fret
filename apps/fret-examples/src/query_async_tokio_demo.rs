use std::future::Future;
use std::pin::Pin;
use std::sync::Arc;
use std::time::Duration;

use fret::prelude::*;
use fret_query::{
    CancellationToken, FutureSpawner, FutureSpawnerHandle, QueryError, QueryKey, QueryPolicy,
    QueryRetryPolicy, QueryState, QueryStatus, with_query_client,
};

mod act {
    fret::actions!([
        Invalidate = "query_async_tokio_demo.invalidate.v1",
        InvalidateNamespace = "query_async_tokio_demo.invalidate_namespace.v1",
        ToggleFailMode = "query_async_tokio_demo.toggle_fail_mode.v1"
    ]);
}

#[derive(Debug)]
struct DemoData {
    label: Arc<str>,
}

fn demo_key() -> QueryKey<DemoData> {
    QueryKey::new("fret-examples.query_async_tokio_demo.demo_data.v1", &0u8)
}

#[derive(Debug)]
struct TokioRuntimeGlobal {
    _rt: Arc<tokio::runtime::Runtime>,
}

#[derive(Clone)]
struct TokioHandleSpawner(tokio::runtime::Handle);

impl FutureSpawner for TokioHandleSpawner {
    fn spawn_send(&self, fut: Pin<Box<dyn Future<Output = ()> + Send + 'static>>) {
        let _ = self.0.spawn(fut);
    }
}

fn install_tokio_spawner(app: &mut App) {
    let rt = tokio::runtime::Builder::new_multi_thread()
        .enable_time()
        .build()
        .expect("failed to build tokio runtime");
    let rt = Arc::new(rt);

    let spawner: FutureSpawnerHandle = Arc::new(TokioHandleSpawner(rt.handle().clone()));
    app.set_global::<FutureSpawnerHandle>(spawner);
    app.set_global::<TokioRuntimeGlobal>(TokioRuntimeGlobal { _rt: rt });
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Default)]
enum PendingInvalidate {
    #[default]
    None,
    Key,
    Namespace,
}

struct QueryAsyncTokioDemoState {
    fail_mode: Model<bool>,
    pending_invalidate: Model<PendingInvalidate>,
}

struct QueryAsyncTokioDemoView {
    st: QueryAsyncTokioDemoState,
}

impl View for QueryAsyncTokioDemoView {
    fn init(app: &mut App, _window: AppWindowId) -> Self {
        Self {
            st: QueryAsyncTokioDemoState {
                fail_mode: app.models_mut().insert(false),
                pending_invalidate: app.models_mut().insert(PendingInvalidate::None),
            },
        }
    }

    fn render(&mut self, cx: &mut ViewCx<'_, '_, App>) -> Elements {
        let theme = Theme::global(&*cx.app).snapshot();

        let pending = cx
            .watch_model(&self.st.pending_invalidate)
            .layout()
            .copied_or_default();
        if pending != PendingInvalidate::None {
            let key = demo_key();
            let _ = with_query_client(cx.app, |client, app| match pending {
                PendingInvalidate::None => {}
                PendingInvalidate::Key => {
                    client.invalidate(app, key);
                }
                PendingInvalidate::Namespace => {
                    client.invalidate_namespace(key.namespace());
                }
            });

            let _ = cx
                .app
                .models_mut()
                .update(&self.st.pending_invalidate, |v| {
                    *v = PendingInvalidate::None
                });
        }

        let fail_mode = cx.watch_model(&self.st.fail_mode).layout().copied_or_default();

        let key = demo_key();
        let policy = QueryPolicy {
            stale_time: Duration::from_secs(2),
            cache_time: Duration::from_secs(30),
            keep_previous_data_while_loading: true,
            retry: QueryRetryPolicy::exponential(3, Duration::from_millis(250), Duration::from_secs(2)),
            ..Default::default()
        };

        let handle = cx.use_query_async(key, policy, move |token: CancellationToken| async move {
            tokio::time::sleep(Duration::from_millis(250)).await;

            if token.is_cancelled() {
                return Err(QueryError::transient("cancelled"));
            }

            if fail_mode {
                return Err(QueryError::transient("simulated async fetch error"));
            }

            let label: Arc<str> =
                Arc::from(format!("async fetched at {:?}", fret_core::time::Instant::now()));
            Ok(DemoData { label })
        });

        let state = cx
            .watch_model(handle.model())
            .layout()
            .cloned_or_else(QueryState::<DemoData>::default);

        let status_label = match state.status {
            QueryStatus::Idle => "Idle",
            QueryStatus::Loading => "Loading",
            QueryStatus::Success => "Success",
            QueryStatus::Error => "Error",
        };

        let mode_badge = shadcn::Badge::new(if fail_mode { "Mode: Error" } else { "Mode: Ok" })
            .variant(shadcn::BadgeVariant::Secondary)
            .into_element(cx);

        let status_badge = shadcn::Badge::new(status_label)
            .variant(match state.status {
                QueryStatus::Success => shadcn::BadgeVariant::Default,
                QueryStatus::Error => shadcn::BadgeVariant::Destructive,
                QueryStatus::Idle | QueryStatus::Loading => shadcn::BadgeVariant::Secondary,
            })
            .into_element(cx);

        let info_line = match state.status {
            QueryStatus::Loading if state.data.is_some() => "Refreshing (kept previous data)…",
            QueryStatus::Loading => "Loading…",
            QueryStatus::Success => "Ready.",
            QueryStatus::Error => "Fetch failed.",
            QueryStatus::Idle => "Idle.",
        };

        let data_line: Arc<str> = state
            .data
            .as_ref()
            .map(|d| d.label.clone())
            .unwrap_or_else(|| Arc::from("<no data>"));

        let toggle_mode_btn = shadcn::Button::new("Toggle error mode")
            .variant(shadcn::ButtonVariant::Secondary)
            .action(act::ToggleFailMode)
            .into_element(cx);

        let invalidate_btn = shadcn::Button::new("Invalidate")
            .variant(shadcn::ButtonVariant::Default)
            .action(act::Invalidate)
            .into_element(cx);

        let invalidate_ns_btn = shadcn::Button::new("Invalidate namespace")
            .variant(shadcn::ButtonVariant::Ghost)
            .action(act::InvalidateNamespace)
            .into_element(cx);

        let buttons = ui::h_flex(cx, |_cx| {
            [invalidate_btn, invalidate_ns_btn, toggle_mode_btn]
        })
        .gap(Space::N2)
        .items_center()
        .into_element(cx);

        let lines = ui::v_flex(cx, |cx| {
            let mut out: Vec<AnyElement> = Vec::new();
            out.push(ui::raw_text(cx, info_line).into_element(cx));
            out.push(ui::raw_text(cx, data_line).into_element(cx));
            out.push(
                ui::raw_text(
                    cx,
                    "Async fetch uses a runtime-provided FutureSpawnerHandle (Tokio in this demo).",
                )
                .text_color(ColorRef::Color(theme.color_token("muted-foreground")))
                .into_element(cx),
            );
            if let Some(err) = state.error.as_ref() {
                out.push(
                    ui::raw_text(cx, format!("error={err} kind={:?}", err.kind()))
                        .text_color(ColorRef::Color(theme.color_token("destructive")))
                        .into_element(cx),
                );
            }
            out
        })
        .gap(Space::N2)
        .into_element(cx);

        let header = shadcn::CardHeader::new([
            shadcn::CardTitle::new("Async query demo (Tokio)").into_element(cx),
            shadcn::CardDescription::new("use_query_async + FutureSpawnerHandle").into_element(cx),
            ui::h_flex(cx, |_cx| [status_badge, mode_badge])
                .gap(Space::N2)
                .items_center()
                .into_element(cx),
        ])
        .into_element(cx);

        let content = shadcn::CardContent::new([ui::v_flex(cx, |_cx| [buttons, lines])
            .gap(Space::N4)
            .w_full()
            .into_element(cx)])
        .into_element(cx);

        let card = shadcn::Card::new([header, content])
            .ui()
            .w_full()
            .max_w(Px(520.0))
            .into_element(cx);

        cx.on_action_notify::<act::ToggleFailMode>({
            let fail_mode = self.st.fail_mode.clone();
            move |host, _acx| {
                let _ = host.models_mut().update(&fail_mode, |v| *v = !*v);
                true
            }
        });

        cx.on_action_notify::<act::Invalidate>({
            let pending = self.st.pending_invalidate.clone();
            move |host, _acx| {
                let _ = host.models_mut().update(&pending, |v| *v = PendingInvalidate::Key);
                true
            }
        });

        cx.on_action_notify::<act::InvalidateNamespace>({
            let pending = self.st.pending_invalidate.clone();
            move |host, _acx| {
                let _ = host
                    .models_mut()
                    .update(&pending, |v| *v = PendingInvalidate::Namespace);
                true
            }
        });

        let page = ui::container(cx, |cx| {
            [ui::v_flex(cx, |_cx| [card])
                .w_full()
                .h_full()
                .justify_center()
                .items_center()
                .into_element(cx)]
        })
        .bg(ColorRef::Color(theme.color_token("background")))
        .p(Space::N6)
        .w_full()
        .h_full()
        .into_element(cx);

        page.into()
    }
}

pub fn run() -> anyhow::Result<()> {
    FretApp::new("query-async-tokio-demo")
        .window("query-async-tokio-demo", (560.0, 360.0))
        .config_files(false)
        .install_app(|app| {
            install_tokio_spawner(app);
            shadcn::shadcn_themes::apply_shadcn_new_york(
                app,
                shadcn::shadcn_themes::ShadcnBaseColor::Zinc,
                shadcn::shadcn_themes::ShadcnColorScheme::Dark,
            );
        })
        .run_view::<QueryAsyncTokioDemoView>()
        .map_err(anyhow::Error::from)
}

