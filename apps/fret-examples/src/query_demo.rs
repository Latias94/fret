use std::sync::Arc;
use std::time::Duration;

use fret::prelude::*;
use fret_query::{
    QueryError, QueryKey, QueryPolicy, QueryRetryPolicy, QueryState, QueryStatus, with_query_client,
};

mod act {
    fret::actions!([
        Invalidate = "query_demo.invalidate.v1",
        InvalidateNamespace = "query_demo.invalidate_namespace.v1",
        ToggleFailMode = "query_demo.toggle_fail_mode.v1"
    ]);
}

const TRANSIENT_INVALIDATE_KEY: u64 = 0xAFA0_0001;
const TRANSIENT_INVALIDATE_NAMESPACE: u64 = 0xAFA0_0002;

#[derive(Debug)]
struct DemoData {
    label: Arc<str>,
}

fn demo_key() -> QueryKey<DemoData> {
    QueryKey::new("fret-examples.query_demo.demo_data.v1", &0u8)
}

struct QueryDemoState {
    fail_mode: Model<bool>,
}

struct QueryDemoView {
    st: QueryDemoState,
}

impl View for QueryDemoView {
    fn init(app: &mut App, _window: AppWindowId) -> Self {
        Self {
            st: QueryDemoState {
                fail_mode: app.models_mut().insert(false),
            },
        }
    }

    fn render(&mut self, cx: &mut ViewCx<'_, '_, App>) -> Elements {
        let theme = Theme::global(&*cx.app).snapshot();

        if cx.take_transient_on_action_root(TRANSIENT_INVALIDATE_KEY) {
            let _ = with_query_client(cx.app, |client, app| {
                client.invalidate(app, demo_key());
            });
        }
        if cx.take_transient_on_action_root(TRANSIENT_INVALIDATE_NAMESPACE) {
            let key = demo_key();
            let _ = with_query_client(cx.app, |client, _app| {
                client.invalidate_namespace(key.namespace());
            });
        }

        let fail_mode = cx
            .watch_model(&self.st.fail_mode)
            .layout()
            .copied_or_default();

        let key = demo_key();
        let policy = QueryPolicy {
            stale_time: Duration::from_secs(2),
            cache_time: Duration::from_secs(30),
            keep_previous_data_while_loading: true,
            retry: QueryRetryPolicy::exponential(
                3,
                Duration::from_millis(250),
                Duration::from_secs(2),
            ),
            ..Default::default()
        };

        let handle = cx.use_query(key, policy, move |_token| {
            if fail_mode {
                return Err(QueryError::transient("simulated fetch error"));
            }
            let label: Arc<str> =
                Arc::from(format!("fetched at {:?}", fret_core::time::Instant::now()));
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

        let error_line = state.error.clone();

        let duration_line = state
            .last_duration
            .map(|d| Arc::from(format!("last_duration={d:?}")));

        let retry_line = state.retry.next_retry_at.map(|at| {
            Arc::from(format!(
                "retry: failures={} next_at={at:?}",
                state.retry.failures
            ))
        });

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

        let buttons = ui::h_flex(|_cx| [invalidate_btn, invalidate_ns_btn, toggle_mode_btn])
            .gap(Space::N2)
            .items_center()
            .into_element(cx);

        let lines = ui::v_flex(|cx| {
            let mut out: Vec<AnyElement> = Vec::new();
            out.push(ui::raw_text(info_line).into_element(cx));
            out.push(ui::raw_text(data_line).into_element(cx));

            if let Some(err) = error_line {
                out.push(
                    ui::raw_text(format!("Error: {:?}", err.kind()))
                        .text_color(ColorRef::Color(theme.color_token("destructive")))
                        .into_element(cx),
                );
            } else {
                out.push(
                    ui::raw_text("Error: <none>")
                        .text_color(ColorRef::Color(theme.color_token("muted-foreground")))
                        .into_element(cx),
                );
            }

            if let Some(dur) = duration_line {
                out.push(
                    ui::raw_text(dur)
                        .text_color(ColorRef::Color(theme.color_token("muted-foreground")))
                        .into_element(cx),
                );
            }
            if let Some(retry) = retry_line {
                out.push(
                    ui::raw_text(retry)
                        .text_color(ColorRef::Color(theme.color_token("muted-foreground")))
                        .into_element(cx),
                );
            }
            out
        })
        .gap(Space::N2)
        .into_element(cx);

        let header = shadcn::CardHeader::new([
            shadcn::CardTitle::new("Query demo").into_element(cx),
            shadcn::CardDescription::new("Async resource state via fret-query.").into_element(cx),
            ui::h_flex(|_cx| [status_badge, mode_badge])
                .gap(Space::N2)
                .items_center()
                .into_element(cx),
        ])
        .into_element(cx);

        let content = shadcn::CardContent::new([ui::v_flex(|_cx| [buttons, lines])
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

        cx.on_action_notify_transient::<act::Invalidate>(TRANSIENT_INVALIDATE_KEY);
        cx.on_action_notify_transient::<act::InvalidateNamespace>(TRANSIENT_INVALIDATE_NAMESPACE);

        let page = ui::container(|cx| {
            [ui::v_flex(|_cx| [card])
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
    FretApp::new("query-demo")
        .window("query-demo", (560.0, 360.0))
        .config_files(false)
        .run_view::<QueryDemoView>()
        .map_err(anyhow::Error::from)
}
