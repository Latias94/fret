use fret::prelude::*;
use fret_ui_kit::headless::table::{ColumnDef, RowKey, TableState, create_column_helper};
use std::sync::Arc;

const TEST_ID_ROOT: &str = "cookbook.data_table_basics.root";
const TEST_ID_TABLE: &str = "cookbook.data_table_basics.table";

#[derive(Debug, Clone)]
struct DemoRow {
    id: u64,
    name: Arc<str>,
    role: Arc<str>,
    score: i32,
}

struct DataTableBasicsView {
    table_state: Model<TableState>,
    table_output: Model<shadcn::DataTableViewOutput>,
    rows: Arc<[DemoRow]>,
    columns: Arc<[ColumnDef<DemoRow>]>,
}

impl View for DataTableBasicsView {
    fn init(app: &mut App, _window: AppWindowId) -> Self {
        let rows: Arc<[DemoRow]> = (0..2_000)
            .map(|i| DemoRow {
                id: i as u64,
                name: Arc::from(format!("User {i}")),
                role: Arc::from(if i % 7 == 0 { "Admin" } else { "Member" }),
                score: ((i * 31) % 997) as i32,
            })
            .collect::<Vec<_>>()
            .into();

        let mut state = TableState::default();
        state.pagination.page_size = 50;

        let helper = create_column_helper::<DemoRow>();
        let columns: Arc<[ColumnDef<DemoRow>]> = vec![
            helper.clone().accessor("id", |r| r.id),
            helper.clone().accessor_str("name", |r| r.name.as_ref()),
            helper.clone().accessor_str("role", |r| r.role.as_ref()),
            helper.accessor("score", |r| r.score),
        ]
        .into();

        Self {
            table_state: app.models_mut().insert(state),
            table_output: app
                .models_mut()
                .insert(shadcn::DataTableViewOutput::default()),
            rows,
            columns,
        }
    }

    fn render(&mut self, cx: &mut ViewCx<'_, '_, App>) -> Elements {
        let theme = Theme::global(&*cx.app).snapshot();

        let header = shadcn::CardHeader::build(|cx, out| {
            out.push_ui(cx, shadcn::CardTitle::new("Data table basics"));
            out.push_ui(
                cx,
                shadcn::CardDescription::new(
                    "A shadcn-style DataTable backed by the TanStack-aligned headless engine: sorting, filtering, pagination, and virtualized rows.",
                ),
            );
        });

        let toolbar = shadcn::DataTableToolbar::new(
            self.table_state.clone(),
            Arc::clone(&self.columns),
            |col| match col.id.as_ref() {
                "id" => Arc::from("ID"),
                "name" => Arc::from("Name"),
                "role" => Arc::from("Role"),
                "score" => Arc::from("Score"),
                _ => Arc::clone(&col.id),
            },
        );

        let pagination =
            shadcn::DataTablePagination::new(self.table_state.clone(), self.table_output.clone());

        let data_table = shadcn::DataTable::new()
            .output_model(self.table_output.clone())
            .into_element(
                cx,
                Arc::clone(&self.rows),
                1,
                self.table_state.clone(),
                Arc::clone(&self.columns),
                |row, _i, _parent| RowKey(row.id),
                |col| match col.id.as_ref() {
                    "id" => Arc::from("ID"),
                    "name" => Arc::from("Name"),
                    "role" => Arc::from("Role"),
                    "score" => Arc::from("Score"),
                    _ => Arc::clone(&col.id),
                },
                |cx, col, row| match col.id.as_ref() {
                    "id" => cx.text(Arc::from(row.id.to_string())),
                    "name" => cx.text(Arc::clone(&row.name)),
                    "role" => cx.text(Arc::clone(&row.role)),
                    "score" => cx.text(Arc::from(row.score.to_string())),
                    _ => cx.text(Arc::from("")),
                },
            );

        let table_slot = ui::container(|_cx| [data_table])
            .bg(ColorRef::Color(theme.color_token("card")))
            .border_1()
            .border_color(ColorRef::Color(theme.color_token("border")))
            .rounded_md()
            .overflow_hidden()
            .size_full()
            .test_id(TEST_ID_TABLE);

        let content = shadcn::CardContent::build(|cx, out| {
            out.push_ui(
                cx,
                ui::v_flex(|cx| ui::children![cx; toolbar, table_slot, pagination])
                    .gap(Space::N3)
                    .h_full(),
            );
        });

        let card = shadcn::Card::build(|cx, out| {
            out.push_ui(cx, header);
            out.push_ui(cx, content);
        })
        .ui()
        .w_full()
        .h_full()
        .max_w(Px(1180.0));

        let root = fret_cookbook::scaffold::centered_page_background_ui(cx, TEST_ID_ROOT, card);
        vec![root].into()
    }
}

fn main() -> anyhow::Result<()> {
    FretApp::new("cookbook-data-table-basics")
        .window("cookbook-data-table-basics", (1180.0, 820.0))
        .config_files(false)
        .install_app(fret_cookbook::install_cookbook_defaults)
        .run_view::<DataTableBasicsView>()
        .map_err(anyhow::Error::from)
}
