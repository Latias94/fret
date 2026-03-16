pub const SOURCE: &str = include_str!("default_demo.rs");

// region: example
use fret::app::UiCxActionsExt as _;
use fret::{UiChild, UiCx};
use fret_core::Px;
use fret_runtime::Model;
use fret_ui_headless::table::{ColumnDef, RowKey, TableState};
use fret_ui_kit::IntoUiElement;
use fret_ui_kit::declarative::ModelWatchExt as _;
use fret_ui_kit::declarative::table::TableViewOutput;
use fret_ui_shadcn::{facade as shadcn, prelude::*};
use std::sync::Arc;

#[derive(Debug, Clone)]
struct InvoiceRow {
    key: u64,
    status: Arc<str>,
    customer_email: Arc<str>,
    amount_usd: u64,
}

fn align_end<B>(child: B) -> impl IntoUiElement<fret_app::App> + use<B>
where
    B: IntoUiElement<fret_app::App>,
{
    ui::h_flex(move |cx| ui::children![cx; child])
        .layout(LayoutRefinement::default().w_full())
        .justify_end()
}

fn footer(
    cx: &mut UiCx<'_>,
    state: Model<TableState>,
    output: Model<TableViewOutput>,
) -> impl IntoUiElement<fret_app::App> + use<> {
    let state_value = cx.watch_model(&state).layout().cloned().unwrap_or_default();
    let output_value = cx
        .watch_model(&output)
        .layout()
        .cloned()
        .unwrap_or_default();

    let page_index = state_value.pagination.page_index.saturating_add(1);
    let filtered_rows = output_value.filtered_row_count;
    let summary = Arc::<str>::from(format!(
        "{filtered_rows} filtered row(s) • Page {page_index}"
    ));

    let prev_enabled = output_value.pagination.can_prev;
    let next_enabled = output_value.pagination.can_next;

    ui::h_flex(move |cx| {
        vec![
            ui::text(summary.clone())
                .text_sm()
                .nowrap()
                .test_id("ui-gallery-data-table-default-summary")
                .into_element(cx),
            cx.spacer(fret_ui::element::SpacerProps::default()),
            shadcn::Button::new("Previous")
                .variant(shadcn::ButtonVariant::Outline)
                .size(shadcn::ButtonSize::Sm)
                .disabled(!prev_enabled)
                .on_activate(cx.actions().listen({
                    let state = state.clone();
                    move |host, _action_cx| {
                        let _ = host.models_mut().update(&state, |table_state| {
                            table_state.pagination.page_index =
                                table_state.pagination.page_index.saturating_sub(1);
                        });
                    }
                }))
                .test_id("ui-gallery-data-table-default-prev")
                .into_element(cx),
            shadcn::Button::new("Next")
                .variant(shadcn::ButtonVariant::Outline)
                .size(shadcn::ButtonSize::Sm)
                .disabled(!next_enabled)
                .on_activate(cx.actions().listen({
                    let state = state.clone();
                    move |host, _action_cx| {
                        let _ = host.models_mut().update(&state, |table_state| {
                            table_state.pagination.page_index =
                                table_state.pagination.page_index.saturating_add(1);
                        });
                    }
                }))
                .test_id("ui-gallery-data-table-default-next")
                .into_element(cx),
        ]
    })
    .layout(LayoutRefinement::default().w_full())
    .items_center()
    .gap(Space::N2)
}

pub fn render(cx: &mut UiCx<'_>) -> impl UiChild + use<> {
    let assets = cx.slot_state(
        || {
            let data: Arc<[InvoiceRow]> = Arc::from(vec![
                InvoiceRow {
                    key: 1,
                    status: Arc::from("Paid"),
                    customer_email: Arc::from("olivia@example.com"),
                    amount_usd: 320,
                },
                InvoiceRow {
                    key: 2,
                    status: Arc::from("Pending"),
                    customer_email: Arc::from("liam@example.com"),
                    amount_usd: 180,
                },
                InvoiceRow {
                    key: 3,
                    status: Arc::from("Paid"),
                    customer_email: Arc::from("ava@example.com"),
                    amount_usd: 640,
                },
                InvoiceRow {
                    key: 4,
                    status: Arc::from("Failed"),
                    customer_email: Arc::from("noah@example.com"),
                    amount_usd: 90,
                },
                InvoiceRow {
                    key: 5,
                    status: Arc::from("Refunded"),
                    customer_email: Arc::from("mia@example.com"),
                    amount_usd: 75,
                },
            ]);

            let columns: Arc<[ColumnDef<InvoiceRow>]> = Arc::from(vec![
                ColumnDef::new("status")
                    .filter_by(|row: &InvoiceRow, q| row.status.as_ref().contains(q))
                    .sort_by(|a: &InvoiceRow, b: &InvoiceRow| a.status.cmp(&b.status))
                    .size(140.0),
                ColumnDef::new("email")
                    .filter_by(|row: &InvoiceRow, q| row.customer_email.as_ref().contains(q))
                    .sort_by(|a: &InvoiceRow, b: &InvoiceRow| {
                        a.customer_email.cmp(&b.customer_email)
                    })
                    .size(280.0),
                ColumnDef::new("amount")
                    .sort_by(|a: &InvoiceRow, b: &InvoiceRow| a.amount_usd.cmp(&b.amount_usd))
                    .size(140.0),
            ]);

            (data, columns)
        },
        |state| state.clone(),
    );

    let state = cx.local_model_keyed("state", || {
        let mut table_state = TableState::default();
        table_state.pagination.page_size = 2;
        table_state
    });
    let output = cx.local_model_keyed("output", TableViewOutput::default);

    let toolbar =
        shadcn::DataTableToolbar::new(state.clone(), assets.1.clone(), |col| Arc::clone(&col.id))
            .show_global_filter(false)
            .column_filter("email")
            .column_filter_placeholder("Filter customer emails...")
            .column_filter_a11y_label("Customer email filter")
            .columns_button_label("Columns")
            .show_pinning_menu(false)
            .show_selected_text(false)
            .into_element(cx)
            .test_id("ui-gallery-data-table-default-toolbar");

    let table = shadcn::DataTable::new()
        .row_click_selection(false)
        .row_height(Px(40.0))
        .header_height(Px(40.0))
        .column_actions_menu(false)
        .output_model(output.clone())
        .refine_layout(LayoutRefinement::default().w_full().h_px(Px(280.0)))
        .into_element(
            cx,
            assets.0.clone(),
            1,
            state.clone(),
            assets.1.clone(),
            |row, _index, _parent| RowKey(row.key),
            |col| match col.id.as_ref() {
                "status" => Arc::<str>::from("Status"),
                "email" => Arc::<str>::from("Customer"),
                "amount" => Arc::<str>::from("Amount"),
                _ => col.id.clone(),
            },
            |cx, col, row| match col.id.as_ref() {
                "status" => cx.text(row.status.as_ref()),
                "email" => cx.text(row.customer_email.as_ref()),
                "amount" => {
                    let amount = Arc::<str>::from(format!("${}.00", row.amount_usd));
                    let amount_text = ui::text(amount).text_sm().tabular_nums().into_element(cx);
                    align_end(amount_text).into_element(cx)
                }
                _ => cx.text("?"),
            },
        )
        .test_id("ui-gallery-data-table-default-root");

    let footer = footer(cx, state, output)
        .into_element(cx)
        .test_id("ui-gallery-data-table-default-footer");

    ui::v_flex(move |_cx| vec![toolbar, table, footer])
        .gap(Space::N4)
        .items_start()
        .layout(LayoutRefinement::default().w_full().min_w_0())
        .into_element(cx)
}
// endregion: example
