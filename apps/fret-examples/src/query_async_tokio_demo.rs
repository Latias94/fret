use std::future::Future;
use std::pin::Pin;
use std::sync::Arc;
use std::time::Duration;

use fret::prelude::*;
use fret_query::ui::QueryElementContextExt as _;
use fret_query::{
    CancellationToken, FutureSpawner, FutureSpawnerHandle, QueryError, QueryKey, QueryPolicy,
    QueryRetryPolicy, QueryState, QueryStatus, with_query_client,
};

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

struct QueryAsyncTokioDemoState {
    fail_mode: Model<bool>,
}

#[derive(Debug, Clone)]
enum Msg {
    Invalidate,
    InvalidateNamespace,
    ToggleFailMode,
}

pub fn run() -> anyhow::Result<()> {
    fret::App::new("query-async-tokio-demo")
        .window("query_async_tokio_demo", (560.0, 360.0))
        .install_app(|app| {
            install_tokio_spawner(app);
            shadcn::shadcn_themes::apply_shadcn_new_york(
                app,
                shadcn::shadcn_themes::ShadcnBaseColor::Zinc,
                shadcn::shadcn_themes::ShadcnColorScheme::Dark,
            );
        })
        .mvu::<QueryAsyncTokioDemoProgram>()?
        .run()
        .map_err(anyhow::Error::from)
}

struct QueryAsyncTokioDemoProgram;

impl MvuProgram for QueryAsyncTokioDemoProgram {
    type State = QueryAsyncTokioDemoState;
    type Message = Msg;

    fn init(app: &mut App, _window: AppWindowId) -> Self::State {
        QueryAsyncTokioDemoState {
            fail_mode: app.models_mut().insert(false),
        }
    }

    fn update(app: &mut App, st: &mut Self::State, msg: Self::Message) {
        match msg {
            Msg::Invalidate => {
                let _ = with_query_client(app, |client, app| {
                    client.invalidate(app, demo_key());
                });
            }
            Msg::InvalidateNamespace => {
                let key = demo_key();
                let _ = with_query_client(app, |client, _app| {
                    client.invalidate_namespace(key.namespace());
                });
            }
            Msg::ToggleFailMode => {
                let _ = app.models_mut().update(&st.fail_mode, |v| *v = !*v);
            }
        }
    }

    fn view(
        cx: &mut ElementContext<'_, App>,
        st: &mut Self::State,
        msg: &mut MessageRouter<Self::Message>,
    ) -> Elements {
        view(cx, st, msg)
    }
}

fn view(
    cx: &mut ElementContext<'_, App>,
    st: &mut QueryAsyncTokioDemoState,
    msg: &mut MessageRouter<Msg>,
) -> Elements {
    let theme = Theme::global(&*cx.app).snapshot();
    let fail_mode = cx.watch_model(&st.fail_mode).layout().copied_or_default();

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

        let label: Arc<str> = Arc::from(format!(
            "async fetched at {:?}",
            fret_core::time::Instant::now()
        ));
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
        .on_click(msg.cmd(Msg::ToggleFailMode))
        .into_element(cx);

    let invalidate_btn = shadcn::Button::new("Invalidate")
        .variant(shadcn::ButtonVariant::Default)
        .on_click(msg.cmd(Msg::Invalidate))
        .into_element(cx);

    let invalidate_ns_btn = shadcn::Button::new("Invalidate namespace")
        .variant(shadcn::ButtonVariant::Ghost)
        .on_click(msg.cmd(Msg::InvalidateNamespace))
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
