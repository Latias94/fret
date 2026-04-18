pub const SOURCE: &str = include_str!("code_outline.rs");

// region: example
use fret::{AppComponentCx, UiChild};
use fret_core::Px;
use fret_runtime::Model;
use fret_ui::element::AnyElement;
use fret_ui_headless::table::{ColumnDef, RowKey, TableState};
use fret_ui_kit::IntoUiElement;
use fret_ui_kit::declarative::table::TableViewOutput;
use fret_ui_shadcn::{facade as shadcn, prelude::*};
use std::sync::Arc;

#[derive(Debug, Clone)]
struct PaymentRow {
    key: u64,
    status: Arc<str>,
    email: Arc<str>,
    amount_usd: u64,
}

#[derive(Debug, Clone)]
struct ReusablePartsAssets {
    rows: Arc<[PaymentRow]>,
    columns: Arc<[ColumnDef<PaymentRow>]>,
}

fn align_end<B>(child: B) -> impl IntoUiElement<fret_app::App> + use<B>
where
    B: IntoUiElement<fret_app::App>,
{
    ui::h_flex(move |cx| ui::children![cx; child])
        .layout(LayoutRefinement::default().w_full())
        .justify_end()
}

fn reusable_cell_test_id(row: &PaymentRow, column_id: &str) -> Arc<str> {
    Arc::<str>::from(format!(
        "ui-gallery-data-table-reusable-cell-{}-{column_id}",
        row.key
    ))
}

fn reusable_view_options(
    cx: &mut AppComponentCx<'_>,
    open: Model<bool>,
    state: Model<TableState>,
    columns: Arc<[ColumnDef<PaymentRow>]>,
) -> AnyElement {
    ui::h_flex(move |cx| {
        vec![
            shadcn::DataTableViewOptions::from_table_state(
                open.clone(),
                state.clone(),
                columns,
                |col| match col.id.as_ref() {
                    "status" => Arc::<str>::from("Status"),
                    "email" => Arc::<str>::from("Email"),
                    "amount" => Arc::<str>::from("Amount"),
                    _ => col.id.clone(),
                },
            )
            .into_element(cx)
            .test_id("ui-gallery-data-table-reusable-view-options"),
        ]
    })
    .layout(LayoutRefinement::default().w_full())
    .justify_end()
    .into_element(cx)
}

fn reusable_table(
    cx: &mut AppComponentCx<'_>,
    assets: &ReusablePartsAssets,
    state: Model<TableState>,
    output: Model<TableViewOutput>,
) -> AnyElement {
    shadcn::DataTable::new()
        .row_click_selection(false)
        .column_actions_menu(true)
        .row_height(Px(40.0))
        .header_height(Px(40.0))
        .output_model(output)
        .refine_layout(LayoutRefinement::default().w_full().h_px(Px(240.0)))
        .into_element(
            cx,
            assets.rows.clone(),
            1,
            state,
            assets.columns.clone(),
            |row, _index, _parent| RowKey(row.key),
            |col| match col.id.as_ref() {
                "status" => Arc::<str>::from("Status"),
                "email" => Arc::<str>::from("Email"),
                "amount" => Arc::<str>::from("Amount"),
                _ => col.id.clone(),
            },
            |cx, col, row| match col.id.as_ref() {
                "status" => cx
                    .text(row.status.as_ref())
                    .test_id(reusable_cell_test_id(row, "status")),
                "email" => cx
                    .text(row.email.as_ref())
                    .test_id(reusable_cell_test_id(row, "email")),
                "amount" => {
                    let amount = Arc::<str>::from(format!("${}.00", row.amount_usd));
                    let amount_text = ui::text(amount)
                        .text_sm()
                        .tabular_nums()
                        .nowrap()
                        .into_element(cx);
                    align_end(amount_text).into_element(cx)
                }
                _ => cx.text("?"),
            },
        )
        .test_id("ui-gallery-data-table-reusable-root")
}

fn reusable_pagination(
    cx: &mut AppComponentCx<'_>,
    state: Model<TableState>,
    output: Model<TableViewOutput>,
) -> AnyElement {
    shadcn::DataTablePagination::new(state, output)
        .page_sizes(Arc::from([2usize, 4, 8]))
        .into_element(cx)
        .test_id("ui-gallery-data-table-reusable-pagination")
}

pub fn render(cx: &mut AppComponentCx<'_>) -> impl UiChild + use<> {
    let assets = cx.slot_state(
        || {
            let rows: Arc<[PaymentRow]> = Arc::from(vec![
                PaymentRow {
                    key: 1,
                    status: Arc::from("success"),
                    email: Arc::from("ken99@example.com"),
                    amount_usd: 316,
                },
                PaymentRow {
                    key: 2,
                    status: Arc::from("success"),
                    email: Arc::from("abe45@example.com"),
                    amount_usd: 242,
                },
                PaymentRow {
                    key: 3,
                    status: Arc::from("processing"),
                    email: Arc::from("monserrat44@example.com"),
                    amount_usd: 837,
                },
                PaymentRow {
                    key: 4,
                    status: Arc::from("failed"),
                    email: Arc::from("carmella@example.com"),
                    amount_usd: 721,
                },
            ]);

            let columns: Arc<[ColumnDef<PaymentRow>]> = Arc::from(vec![
                ColumnDef::new("status")
                    .filter_by(|row: &PaymentRow, q| row.status.as_ref().contains(q))
                    .sort_by(|a: &PaymentRow, b: &PaymentRow| a.status.cmp(&b.status))
                    .enable_pinning(false)
                    .size(140.0),
                ColumnDef::new("email")
                    .filter_by(|row: &PaymentRow, q| row.email.as_ref().contains(q))
                    .sort_by(|a: &PaymentRow, b: &PaymentRow| a.email.cmp(&b.email))
                    .enable_pinning(false)
                    .size(280.0),
                ColumnDef::new("amount")
                    .sort_by(|a: &PaymentRow, b: &PaymentRow| a.amount_usd.cmp(&b.amount_usd))
                    .enable_pinning(false)
                    .size(140.0),
            ]);

            ReusablePartsAssets { rows, columns }
        },
        |st| st.clone(),
    );

    let state = cx.local_model_keyed("state", || {
        let mut state = TableState::default();
        state.pagination.page_size = 2;
        state
    });
    let output = cx.local_model_keyed("output", TableViewOutput::default);
    let view_options_open = cx.local_model_keyed("view_options_open", || false);

    let view_options =
        reusable_view_options(cx, view_options_open, state.clone(), assets.columns.clone());
    let table = reusable_table(cx, &assets, state.clone(), output.clone());
    let pagination = reusable_pagination(cx, state, output);

    ui::v_flex(move |_cx| vec![view_options, table, pagination])
        .gap(Space::N3)
        .items_start()
        .layout(LayoutRefinement::default().w_full().min_w_0())
        .into_element(cx)
}
// endregion: example
