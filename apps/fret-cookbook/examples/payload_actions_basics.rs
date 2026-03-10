use std::sync::Arc;

use fret::prelude::*;

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

struct PayloadActionsView {
    rows: Model<Vec<Row>>,
}

impl View for PayloadActionsView {
    fn init(app: &mut App, _window: AppWindowId) -> Self {
        let rows = app.models_mut().insert(vec![
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
        ]);
        Self { rows }
    }

    fn render(&mut self, cx: &mut ViewCx<'_, '_, App>) -> Elements {
        let rows_snapshot = cx.watch_model(&self.rows).layout().value_or_default();

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

        cx.on_payload_action::<act::Remove>({
            let rows = self.rows.clone();
            move |host, acx, id| {
                let _ = host.models_mut().update(&rows, |rows| {
                    rows.retain(|r| r.id != id);
                });
                host.request_redraw(acx.window);
                host.notify(acx);
                true
            }
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
