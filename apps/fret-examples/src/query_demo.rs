use std::sync::Arc;
use std::time::Duration;

use fret_kit::prelude::*;
use fret_query::ui::QueryElementContextExt as _;
use fret_query::{
    QueryError, QueryKey, QueryPolicy, QueryRetryPolicy, QueryState, QueryStatus, with_query_client,
};

const CMD_INVALIDATE: &str = "query_demo.invalidate";
const CMD_INVALIDATE_NAMESPACE: &str = "query_demo.invalidate_namespace";
const CMD_TOGGLE_FAIL_MODE: &str = "query_demo.toggle_fail_mode";

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

pub fn run() -> anyhow::Result<()> {
    fret_kit::app_with_hooks("query-demo", init_window, view, |d| {
        d.on_command(on_command)
    })?
    .with_main_window("query_demo", (560.0, 360.0))
    .init_app(|app| {
        shadcn::shadcn_themes::apply_shadcn_new_york_v4(
            app,
            shadcn::shadcn_themes::ShadcnBaseColor::Zinc,
            shadcn::shadcn_themes::ShadcnColorScheme::Dark,
        );
    })
    .run()?;
    Ok(())
}

fn init_window(app: &mut App, _window: AppWindowId) -> QueryDemoState {
    QueryDemoState {
        fail_mode: app.models_mut().insert(false),
    }
}

fn view(cx: &mut ElementContext<'_, App>, st: &mut QueryDemoState) -> ViewElements {
    let theme = Theme::global(&*cx.app).clone();

    let fail_mode = cx
        .watch_model(&st.fail_mode)
        .layout()
        .copied()
        .unwrap_or(false);

    let key = demo_key();
    let policy = QueryPolicy {
        stale_time: Duration::from_secs(2),
        cache_time: Duration::from_secs(30),
        keep_previous_data_while_loading: true,
        retry: QueryRetryPolicy::exponential(3, Duration::from_millis(250), Duration::from_secs(2)),
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
        .cloned()
        .unwrap_or_else(|| QueryState::<DemoData>::default());

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
        .on_click(CMD_TOGGLE_FAIL_MODE)
        .into_element(cx);

    let invalidate_btn = shadcn::Button::new("Invalidate")
        .variant(shadcn::ButtonVariant::Default)
        .on_click(CMD_INVALIDATE)
        .into_element(cx);

    let invalidate_ns_btn = shadcn::Button::new("Invalidate namespace")
        .variant(shadcn::ButtonVariant::Ghost)
        .on_click(CMD_INVALIDATE_NAMESPACE)
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
        if let Some(err) = error_line {
            out.push(
                ui::raw_text(cx, format!("error={err} kind={:?}", err.kind()))
                    .text_color(ColorRef::Color(theme.color_required("destructive")))
                    .into_element(cx),
            );
        }
        if let Some(dur) = duration_line {
            out.push(
                ui::raw_text(cx, dur)
                    .text_color(ColorRef::Color(theme.color_required("muted-foreground")))
                    .into_element(cx),
            );
        }
        if let Some(retry) = retry_line {
            out.push(
                ui::raw_text(cx, retry)
                    .text_color(ColorRef::Color(theme.color_required("muted-foreground")))
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
    .bg(ColorRef::Color(theme.color_required("background")))
    .p(Space::N6)
    .w_full()
    .h_full()
    .into_element(cx);

    vec![page].into()
}

fn on_command(
    app: &mut App,
    _services: &mut dyn UiServices,
    window: AppWindowId,
    _ui: &mut UiTree<App>,
    st: &mut QueryDemoState,
    cmd: &CommandId,
) {
    match cmd.as_str() {
        CMD_INVALIDATE => {
            let _ = with_query_client(app, |client, app| {
                client.invalidate(app, demo_key());
            });
            app.request_redraw(window);
        }
        CMD_INVALIDATE_NAMESPACE => {
            let _ = with_query_client(app, |client, _app| {
                client.invalidate_namespace("fret-examples.query_demo.demo_data.v1");
            });
            app.request_redraw(window);
        }
        CMD_TOGGLE_FAIL_MODE => {
            let _ = app.models_mut().update(&st.fail_mode, |v| *v = !*v);
            app.request_redraw(window);
        }
        _ => {}
    }
}
