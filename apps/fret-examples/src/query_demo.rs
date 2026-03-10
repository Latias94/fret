use std::sync::Arc;
use std::time::Duration;

use fret::{FretApp, advanced::prelude::*, shadcn};
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

fn query_policy() -> QueryPolicy {
    QueryPolicy {
        stale_time: Duration::from_secs(2),
        cache_time: Duration::from_secs(30),
        keep_previous_data_while_loading: true,
        retry: QueryRetryPolicy::exponential(3, Duration::from_millis(250), Duration::from_secs(2)),
        ..Default::default()
    }
}

struct QueryDemoView;

impl View for QueryDemoView {
    fn init(_app: &mut KernelApp, _window: AppWindowId) -> Self {
        Self
    }

    fn render(&mut self, cx: &mut AppUi<'_, '_>) -> Ui {
        let theme = Theme::global(&*cx.app).snapshot();
        let fail_mode_state = cx.use_local_with(|| false);

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

        cx.on_action_notify_toggle_local_bool::<act::ToggleFailMode>(&fail_mode_state);
        cx.on_action_notify_transient::<act::Invalidate>(TRANSIENT_INVALIDATE_KEY);
        cx.on_action_notify_transient::<act::InvalidateNamespace>(TRANSIENT_INVALIDATE_NAMESPACE);

        let fail_mode = fail_mode_state.watch(cx).layout().value_or_default();

        let query_handle = cx.data().query(demo_key(), query_policy(), move |_token| {
            if fail_mode {
                return Err(QueryError::transient("simulated fetch error"));
            }
            let label: Arc<str> =
                Arc::from(format!("fetched at {:?}", fret_core::time::Instant::now()));
            Ok(DemoData { label })
        });

        let query_state = query_handle
            .layout(cx)
            .value_or_else(QueryState::<DemoData>::default);

        let status_label = match query_state.status {
            QueryStatus::Idle => "Idle",
            QueryStatus::Loading => "Loading",
            QueryStatus::Success => "Success",
            QueryStatus::Error => "Error",
        };
        let status_variant = match query_state.status {
            QueryStatus::Success => shadcn::BadgeVariant::Default,
            QueryStatus::Error => shadcn::BadgeVariant::Destructive,
            QueryStatus::Idle | QueryStatus::Loading => shadcn::BadgeVariant::Secondary,
        };
        let info_line = match query_state.status {
            QueryStatus::Loading if query_state.data.is_some() => {
                "Refreshing (kept previous data)?"
            }
            QueryStatus::Loading => "Loading?",
            QueryStatus::Success => "Ready.",
            QueryStatus::Error => "Fetch failed.",
            QueryStatus::Idle => "Idle.",
        };
        let data_line: Arc<str> = query_state
            .data
            .as_ref()
            .map(|data| data.label.clone())
            .unwrap_or_else(|| Arc::from("<no data>"));
        let error_line = query_state
            .error
            .as_ref()
            .map(|err| format!("Error: {:?}", err.kind()))
            .unwrap_or_else(|| "Error: <none>".to_string());
        let error_color = if query_state.error.is_some() {
            theme.color_token("destructive")
        } else {
            theme.color_token("muted-foreground")
        };
        let duration_line = query_state
            .last_duration
            .map(|duration| Arc::from(format!("last_duration={duration:?}")));
        let retry_line = query_state.retry.next_retry_at.map(|next_retry_at| {
            Arc::from(format!(
                "retry: failures={} next_at={next_retry_at:?}",
                query_state.retry.failures
            ))
        });

        let status_row = ui::h_row_build(|cx, out| {
            out.push_ui(cx, shadcn::Badge::new(status_label).variant(status_variant));
            out.push_ui(
                cx,
                shadcn::Badge::new(if fail_mode { "Mode: Error" } else { "Mode: Ok" })
                    .variant(shadcn::BadgeVariant::Secondary),
            );
        })
        .gap(Space::N2)
        .items_center()
        .into_element(cx);

        let buttons = ui::h_row_build(|cx, out| {
            out.push_ui(
                cx,
                shadcn::Button::new("Invalidate")
                    .variant(shadcn::ButtonVariant::Default)
                    .action(act::Invalidate),
            );
            out.push_ui(
                cx,
                shadcn::Button::new("Invalidate namespace")
                    .variant(shadcn::ButtonVariant::Ghost)
                    .action(act::InvalidateNamespace),
            );
            out.push_ui(
                cx,
                shadcn::Button::new("Toggle error mode")
                    .variant(shadcn::ButtonVariant::Secondary)
                    .action(act::ToggleFailMode),
            );
        })
        .gap(Space::N2)
        .items_center()
        .into_element(cx);

        let detail_body = ui::v_flex_build(|cx, out| {
            out.push_ui(cx, ui::raw_text(info_line));
            out.push_ui(cx, ui::raw_text(data_line));
            out.push_ui(
                cx,
                ui::raw_text(error_line).text_color(ColorRef::Color(error_color)),
            );
            if let Some(duration_line) = duration_line {
                out.push_ui(
                    cx,
                    ui::raw_text(duration_line)
                        .text_color(ColorRef::Color(theme.color_token("muted-foreground"))),
                );
            }
            if let Some(retry_line) = retry_line {
                out.push_ui(
                    cx,
                    ui::raw_text(retry_line)
                        .text_color(ColorRef::Color(theme.color_token("muted-foreground"))),
                );
            }
        })
        .gap(Space::N2)
        .into_element(cx);

        let card = shadcn::Card::build(|cx, out| {
            out.push_ui(
                cx,
                shadcn::CardHeader::build(|cx, out| {
                    out.push_ui(cx, shadcn::CardTitle::new("Query demo"));
                    out.push_ui(
                        cx,
                        shadcn::CardDescription::new("Async resource state via fret-query."),
                    );
                    out.push(status_row);
                }),
            );
            out.push_ui(
                cx,
                shadcn::CardContent::build(|cx, out| {
                    out.push_ui(
                        cx,
                        ui::v_flex_build(|_cx, out| {
                            out.extend([buttons, detail_body]);
                        })
                        .gap(Space::N4)
                        .w_full(),
                    );
                }),
            );
        })
        .ui()
        .w_full()
        .max_w(Px(520.0))
        .into_element(cx);

        ui::v_flex_build(|_cx, out| {
            out.push(card);
        })
        .bg(ColorRef::Color(theme.color_token("background")))
        .p(Space::N6)
        .w_full()
        .h_full()
        .justify_center()
        .items_center()
        .into_element(cx)
        .into()
    }
}

pub fn run() -> anyhow::Result<()> {
    FretApp::new("query-demo")
        .window("query-demo", (560.0, 360.0))
        .config_files(false)
        .view::<QueryDemoView>()?
        .run()
        .map_err(anyhow::Error::from)
}
