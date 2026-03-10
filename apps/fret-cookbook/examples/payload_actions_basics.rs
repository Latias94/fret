use std::sync::Arc;

use fret::app::prelude::*;

mod act {
    fret::payload_actions!([Remove(u64) = "cookbook.payload_actions.remove.v2"]);
}

const TEST_ID_ROOT: &str = "cookbook.payload_actions.root";
const TEST_ID_ROWS: &str = "cookbook.payload_actions.rows";

fn row_test_id(id: u64) -> Arc<str> {
    Arc::from(format!("cookbook.payload_actions.row.{id}"))
}

fn remove_test_id(id: u64) -> Arc<str> {
    Arc::from(format!("cookbook.payload_actions.remove.{id}"))
}

#[derive(Debug, Clone)]
struct Row {
    id: u64,
    label: Arc<str>,
}

struct PayloadActionsView;

impl View for PayloadActionsView {
    fn init(_app: &mut KernelApp, _window: AppWindowId) -> Self {
        Self
    }

    fn render(&mut self, cx: &mut AppUi<'_, '_>) -> Ui {
        let rows_state = cx.state().local_init(|| {
            vec![
                Row {
                    id: 1,
                    label: Arc::from("Parameterize actions without routers"),
                },
                Row {
                    id: 2,
                    label: Arc::from("Keep IR data-first (payload is transient)"),
                },
                Row {
                    id: 3,
                    label: Arc::from("Prepare for future DSL/frontends"),
                },
            ]
        });
        let rows_snapshot = cx.state().watch(&rows_state).layout().value_or_default();

        let rows_el = ui::v_flex_build(|cx, out| {
            for row in &rows_snapshot {
                let row_id = row.id;
                let row_label = row.label.clone();

                out.push_ui(
                    cx,
                    ui::keyed(row_id, |_cx| {
                        let label = ui::text(row_label).text_sm();

                        let remove = shadcn::Button::new("Remove")
                            .variant(shadcn::ButtonVariant::Secondary)
                            .action(act::Remove)
                            .action_payload(row_id)
                            .test_id(remove_test_id(row_id));

                        ui::h_flex(|cx| ui::children![cx; label, remove])
                            .gap(Space::N2)
                            .items_center()
                            .justify_between()
                            .w_full()
                            .test_id(row_test_id(row_id))
                    }),
                );
            }
        })
        .gap(Space::N2)
        .w_full()
        .test_id(TEST_ID_ROWS);

        cx.actions()
            .payload::<act::Remove>()
            .local_update_if::<Vec<Row>>(&rows_state, |rows, id| {
                let before = rows.len();
                rows.retain(|row| row.id != id);
                rows.len() != before
            });

        let card = shadcn::Card::build(|cx, out| {
            out.push_ui(
                cx,
                shadcn::CardContent::build(|cx, out| {
                    out.push_ui(cx, rows_el);
                }),
            );
        })
        .ui()
        .w_full()
        .max_w(Px(560.0));

        fret_cookbook::scaffold::centered_page_muted_ui(cx, TEST_ID_ROOT, card).into()
    }
}

fn main() -> anyhow::Result<()> {
    FretApp::new("cookbook-payload-actions-basics")
        .window("cookbook-payload-actions-basics", (640.0, 360.0))
        .install_app(fret_cookbook::install_cookbook_defaults)
        .run_view::<PayloadActionsView>()
        .map_err(anyhow::Error::from)
}
