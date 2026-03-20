use std::sync::Arc;
use std::time::Duration;

use fret::app::prelude::*;
use fret::query::{QueryError, QueryKey, QueryPolicy, QueryStatus};
use fret::style::Space;

mod act {
    fret::actions!([
        Invalidate = "cookbook.query_basics.invalidate.v1",
        InvalidateNamespace = "cookbook.query_basics.invalidate_namespace.v1",
        ToggleErrorMode = "cookbook.query_basics.toggle_error_mode.v1"
    ]);
}

const TRANSIENT_INVALIDATE_KEY: u64 = 0xC00B_1001;
const TRANSIENT_INVALIDATE_NAMESPACE: u64 = 0xC00B_1002;

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
    fn init(_app: &mut App, _window: WindowId) -> Self {
        Self
    }

    fn render(&mut self, cx: &mut AppUi<'_, '_>) -> Ui {
        let fail_mode = cx.state().local_init(|| false);

        if cx.effects().take_transient(TRANSIENT_INVALIDATE_KEY) {
            cx.data().invalidate_query(demo_key());
        }
        if cx.effects().take_transient(TRANSIENT_INVALIDATE_NAMESPACE) {
            cx.data().invalidate_query_namespace(QUERY_NS);
        }

        cx.actions()
            .local(&fail_mode)
            .toggle_bool::<act::ToggleErrorMode>();
        cx.actions()
            .transient::<act::Invalidate>(TRANSIENT_INVALIDATE_KEY);
        cx.actions()
            .transient::<act::InvalidateNamespace>(TRANSIENT_INVALIDATE_NAMESPACE);
        let fail_mode_enabled = fail_mode.layout_value(cx);

        let key = demo_key();
        let policy = QueryPolicy {
            stale_time: Duration::from_secs(2),
            cache_time: Duration::from_secs(30),
            keep_previous_data_while_loading: true,
            ..Default::default()
        };

        let handle = cx.data().query(key, policy, move |_token| {
            if fail_mode_enabled {
                return Err(QueryError::transient("simulated fetch error"));
            }
            Ok(DemoData {
                label: Arc::from("Hello from fret-query."),
            })
        });

        let state = handle.read_layout(cx);

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
        .test_id(TEST_ID_MODE_BADGE);

        let status_badge = shadcn::Badge::new(status_label)
            .variant(match state.status {
                QueryStatus::Success => shadcn::BadgeVariant::Default,
                QueryStatus::Error => shadcn::BadgeVariant::Destructive,
                QueryStatus::Idle | QueryStatus::Loading => shadcn::BadgeVariant::Secondary,
            })
            .test_id(TEST_ID_STATUS_BADGE);

        let data_line: Arc<str> = state
            .data
            .as_ref()
            .map(|d| d.label.clone())
            .unwrap_or_else(|| Arc::from("<no data>"));

        let invalidate_btn = shadcn::Button::new("Invalidate")
            .variant(shadcn::ButtonVariant::Default)
            .action(act::Invalidate)
            .test_id(TEST_ID_BTN_INVALIDATE);
        let invalidate_ns_btn = shadcn::Button::new("Invalidate namespace")
            .variant(shadcn::ButtonVariant::Ghost)
            .action(act::InvalidateNamespace)
            .test_id(TEST_ID_BTN_INVALIDATE_NS);
        let toggle_mode_btn = shadcn::Button::new("Toggle error mode")
            .variant(shadcn::ButtonVariant::Secondary)
            .action(act::ToggleErrorMode)
            .test_id(TEST_ID_BTN_TOGGLE_MODE);

        let buttons =
            ui::h_flex(|cx| ui::children![cx; invalidate_btn, invalidate_ns_btn, toggle_mode_btn])
                .gap(Space::N2)
                .items_center();

        let lines = ui::v_flex_build(|cx, out| {
            out.push(cx.text(data_line).test_id(TEST_ID_DATA_LINE));
            if let Some(err) = state.error {
                out.push(cx.text(format!("error={err}")).test_id(TEST_ID_ERROR_LINE));
            }
        })
        .gap(Space::N2);

        let card = shadcn::card(|cx| {
            ui::children![cx;
                shadcn::card_header(|cx| {
                    ui::children![cx;
                        shadcn::card_title("Query basics"),
                        shadcn::card_description(
                            "A tiny async resource example using fret-query (invalidate + error mode).",
                        ),
                        ui::h_flex(|cx| ui::children![cx; status_badge, mode_badge])
                            .gap(Space::N2)
                            .items_center(),
                    ]
                }),
                shadcn::card_content(|cx| {
                    ui::children![cx;
                        ui::v_flex(|cx| ui::children![cx; buttons, lines])
                            .gap(Space::N4)
                            .w_full()
                    ]
                }),
            ]
        })
        .ui()
        .w_full()
        .max_w(Px(560.0));

        fret_cookbook::scaffold::centered_page_muted(cx, TEST_ID_ROOT, card).into()
    }
}

fn main() -> anyhow::Result<()> {
    FretApp::new("cookbook-query-basics")
        .window("cookbook-query-basics", (640.0, 420.0))
        .config_files(false)
        .setup(fret_cookbook::install_cookbook_defaults)
        .view::<QueryBasicsView>()?
        .run()
        .map_err(anyhow::Error::from)
}
