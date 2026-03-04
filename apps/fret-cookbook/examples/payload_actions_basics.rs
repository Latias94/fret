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
        let rows_snapshot = cx.watch_model(&self.rows).layout().cloned_or_default();

        let rows_el = ui::v_flex_build(cx, |cx, out| {
            for row in &rows_snapshot {
                let row_id = row.id;
                let row_label = row.label.clone();

                out.push(cx.keyed(row_id, |cx| {
                    let label = ui::text(cx, row_label).text_sm().into_element(cx);

                    let remove = shadcn::Button::new("Remove")
                        .variant(shadcn::ButtonVariant::Secondary)
                        .action(act::Remove)
                        .action_payload(row_id)
                        .test_id(remove_test_id(row_id))
                        .into_element(cx);

                    ui::h_flex(cx, |_cx| [label, remove])
                        .gap(Space::N2)
                        .items_center()
                        .justify_between()
                        .w_full()
                        .test_id(row_test_id(row_id))
                        .into_element(cx)
                }));
            }
        })
        .gap(Space::N2)
        .w_full()
        .into_element(cx)
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

        let card = shadcn::Card::new([shadcn::CardContent::new([rows_el]).into_element(cx)])
            .ui()
            .w_full()
            .max_w(Px(560.0))
            .into_element(cx);

        fret_cookbook::scaffold::centered_page_muted(cx, TEST_ID_ROOT, card).into()
    }
}

fn main() -> anyhow::Result<()> {
    FretApp::new("cookbook-payload-actions-basics")
        .window("cookbook-payload-actions-basics", (640.0, 360.0))
        .install_app(fret_cookbook::install_cookbook_defaults)
        .run_view::<PayloadActionsView>()
        .map_err(anyhow::Error::from)
}

