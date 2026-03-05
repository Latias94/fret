use std::sync::Arc;
use std::time::Duration;

use fret::prelude::*;
use fret_query::{QueryError, QueryKey, QueryPolicy, QueryState, QueryStatus, with_query_client};
use fret_ui::CommandAvailability;

mod act {
    fret::actions!([
        Invalidate = "cookbook.query_basics.invalidate.v1",
        InvalidateNamespace = "cookbook.query_basics.invalidate_namespace.v1",
        ToggleErrorMode = "cookbook.query_basics.toggle_error_mode.v1"
    ]);
}

const TEST_ID_ROOT: &str = "cookbook.query_basics.root";
const TEST_ID_STATUS_BADGE: &str = "cookbook.query_basics.status.badge";
const TEST_ID_MODE_BADGE: &str = "cookbook.query_basics.mode.badge";
const TEST_ID_DATA_LINE: &str = "cookbook.query_basics.data.line";
const TEST_ID_ERROR_LINE: &str = "cookbook.query_basics.error.line";
const TEST_ID_BTN_INVALIDATE: &str = "cookbook.query_basics.invalidate.button";
const TEST_ID_BTN_INVALIDATE_NS: &str = "cookbook.query_basics.invalidate_namespace.button";
const TEST_ID_BTN_TOGGLE_MODE: &str = "cookbook.query_basics.toggle_error_mode.button";

const QUERY_NS: &str = "fret-cookbook.query_basics.demo_data.v1";

#[derive(Debug)]
struct DemoData {
    label: Arc<str>,
}

fn demo_key() -> QueryKey<DemoData> {
    QueryKey::new_named(QUERY_NS, &0u8, "demo_data")
}

struct QueryBasicsView;

impl View for QueryBasicsView {
    fn init(_app: &mut App, _window: AppWindowId) -> Self {
        Self
    }

    fn render(&mut self, cx: &mut ViewCx<'_, '_, App>) -> Elements {
        let fail_mode = cx.use_state::<bool>();
        let invalidate_requested = cx.use_state::<bool>();
        let invalidate_namespace_requested = cx.use_state::<bool>();

        cx.on_action::<act::ToggleErrorMode>({
            let fail_mode = fail_mode.clone();
            move |host, acx| {
                let _ = host.models_mut().update(&fail_mode, |v| *v = !*v);
                host.request_redraw(acx.window);
                host.notify(acx);
                true
            }
        });
        cx.on_action::<act::Invalidate>({
            let invalidate_requested = invalidate_requested.clone();
            move |host, acx| {
                let _ = host
                    .models_mut()
                    .update(&invalidate_requested, |v| *v = true);
                host.request_redraw(acx.window);
                host.notify(acx);
                true
            }
        });
        cx.on_action::<act::InvalidateNamespace>({
            let invalidate_namespace_requested = invalidate_namespace_requested.clone();
            move |host, acx| {
                let _ = host
                    .models_mut()
                    .update(&invalidate_namespace_requested, |v| *v = true);
                host.request_redraw(acx.window);
                host.notify(acx);
                true
            }
        });
        cx.on_action_availability::<act::ToggleErrorMode>(|_host, _acx| {
            CommandAvailability::Available
        });
        cx.on_action_availability::<act::Invalidate>(|_host, _acx| CommandAvailability::Available);
        cx.on_action_availability::<act::InvalidateNamespace>(|_host, _acx| {
            CommandAvailability::Available
        });

        let fail_mode_enabled = cx.watch_model(&fail_mode).layout().copied_or(false);
        let do_invalidate = cx
            .watch_model(&invalidate_requested)
            .layout()
            .copied_or(false);
        let do_invalidate_namespace = cx
            .watch_model(&invalidate_namespace_requested)
            .layout()
            .copied_or(false);
        let window = cx.window;

        if do_invalidate {
            let key = demo_key();
            let _ = with_query_client(cx.app, |client, app| {
                client.invalidate(app, key);
            });
            let _ = cx
                .app
                .models_mut()
                .update(&invalidate_requested, |v| *v = false);
            cx.app.request_redraw(window);
        }
        if do_invalidate_namespace {
            let _ = with_query_client(cx.app, |client, _app| {
                client.invalidate_namespace(QUERY_NS);
            });
            let _ = cx
                .app
                .models_mut()
                .update(&invalidate_namespace_requested, |v| *v = false);
            cx.app.request_redraw(window);
        }

        let key = demo_key();
        let policy = QueryPolicy {
            stale_time: Duration::from_secs(2),
            cache_time: Duration::from_secs(30),
            keep_previous_data_while_loading: true,
            ..Default::default()
        };

        let handle = cx.use_query(key, policy, move |_token| {
            if fail_mode_enabled {
                return Err(QueryError::transient("simulated fetch error"));
            }
            Ok(DemoData {
                label: Arc::from("Hello from fret-query."),
            })
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

        let mode_badge = shadcn::Badge::new(if fail_mode_enabled {
            "Mode: Error"
        } else {
            "Mode: Ok"
        })
        .variant(shadcn::BadgeVariant::Secondary)
        .into_element(cx)
        .test_id(TEST_ID_MODE_BADGE);

        let status_badge = shadcn::Badge::new(status_label)
            .variant(match state.status {
                QueryStatus::Success => shadcn::BadgeVariant::Default,
                QueryStatus::Error => shadcn::BadgeVariant::Destructive,
                QueryStatus::Idle | QueryStatus::Loading => shadcn::BadgeVariant::Secondary,
            })
            .into_element(cx)
            .test_id(TEST_ID_STATUS_BADGE);

        let data_line: Arc<str> = state
            .data
            .as_ref()
            .map(|d| d.label.clone())
            .unwrap_or_else(|| Arc::from("<no data>"));

        let invalidate_btn = shadcn::Button::new("Invalidate")
            .variant(shadcn::ButtonVariant::Default)
            .action(act::Invalidate)
            .into_element(cx)
            .test_id(TEST_ID_BTN_INVALIDATE);
        let invalidate_ns_btn = shadcn::Button::new("Invalidate namespace")
            .variant(shadcn::ButtonVariant::Ghost)
            .action(act::InvalidateNamespace)
            .into_element(cx)
            .test_id(TEST_ID_BTN_INVALIDATE_NS);
        let toggle_mode_btn = shadcn::Button::new("Toggle error mode")
            .variant(shadcn::ButtonVariant::Secondary)
            .action(act::ToggleErrorMode)
            .into_element(cx)
            .test_id(TEST_ID_BTN_TOGGLE_MODE);

        let buttons = ui::h_flex(|_cx| [invalidate_btn, invalidate_ns_btn, toggle_mode_btn])
            .gap(Space::N2)
            .items_center()
            .into_element(cx);

        let lines = ui::v_flex(|cx| {
            let data = cx.text(data_line).test_id(TEST_ID_DATA_LINE);
            let mut out: Vec<AnyElement> = vec![data];
            if let Some(err) = state.error {
                out.push(cx.text(format!("error={err}")).test_id(TEST_ID_ERROR_LINE));
            }
            out
        })
        .gap(Space::N2)
        .into_element(cx);

        let header = shadcn::CardHeader::new([
            shadcn::CardTitle::new("Query basics").into_element(cx),
            shadcn::CardDescription::new(
                "A tiny async resource example using fret-query (invalidate + error mode).",
            )
            .into_element(cx),
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
            .max_w(Px(560.0))
            .into_element(cx);

        fret_cookbook::scaffold::centered_page_muted(cx, TEST_ID_ROOT, card).into()
    }
}

fn main() -> anyhow::Result<()> {
    FretApp::new("cookbook-query-basics")
        .window("cookbook-query-basics", (640.0, 420.0))
        .config_files(false)
        .install_app(fret_cookbook::install_cookbook_defaults)
        .run_view::<QueryBasicsView>()
        .map_err(anyhow::Error::from)
}
