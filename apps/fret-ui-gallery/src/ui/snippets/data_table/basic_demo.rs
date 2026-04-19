pub const SOURCE: &str = include_str!("basic_demo.rs");

// region: example
use fret::app::AppRenderActionsExt as _;
use fret::{AppComponentCx, UiChild};
use fret_core::Px;
use fret_runtime::Model;
use fret_ui::Theme;
use fret_ui_headless::table::{ColumnDef, RowKey, Table, TableState};
use fret_ui_kit::IntoUiElement;
use fret_ui_kit::declarative::ModelWatchExt as _;
use fret_ui_kit::declarative::table::TableViewOutput;
use fret_ui_shadcn::{facade as shadcn, prelude::*};
use std::sync::Arc;

mod act {
    fret::actions!([ToggleAllPageRows = "ui-gallery.data_table.basic.toggle_all_page_rows.v1"]);
    fret::payload_actions!([
        ToggleRowSelected(u64) = "ui-gallery.data_table.basic.toggle_row_selected.v1",
        CopyPaymentId(std::sync::Arc<str>) = "ui-gallery.data_table.basic.copy_payment_id.v1",
        ViewCustomer(std::sync::Arc<str>) = "ui-gallery.data_table.basic.view_customer.v1",
        ViewPaymentDetails(std::sync::Arc<str>) = "ui-gallery.data_table.basic.view_payment_details.v1"
    ]);
}

#[derive(Debug, Clone)]
struct PaymentRow {
    key: u64,
    id: Arc<str>,
    amount_usd: u64,
    status: Arc<str>,
    email: Arc<str>,
}

fn bind_selection_actions(
    cx: &mut AppComponentCx<'_>,
    state: Model<TableState>,
    data: Arc<[PaymentRow]>,
    columns: Arc<[ColumnDef<PaymentRow>]>,
) {
    cx.actions().models::<act::ToggleAllPageRows>({
        let state = state.clone();
        let data = data.clone();
        let columns = columns.clone();
        move |models| {
            let current = models
                .read(&state, |st| st.clone())
                .ok()
                .unwrap_or_default();
            let next = Table::builder(data.as_ref())
                .columns(columns.as_ref().to_vec())
                .get_row_key(|row, _index, _parent| RowKey(row.key))
                .state(current)
                .build()
                .toggled_all_page_rows_selected(None);

            let _ = models.update(&state, |st| {
                st.row_selection = next;
            });
            true
        }
    });

    cx.actions().payload_models::<act::ToggleRowSelected>({
        move |models, row_key| {
            let current = models
                .read(&state, |st| st.clone())
                .ok()
                .unwrap_or_default();
            let next = Table::builder(data.as_ref())
                .columns(columns.as_ref().to_vec())
                .get_row_key(|row, _index, _parent| RowKey(row.key))
                .state(current)
                .build()
                .toggled_row_selected(RowKey(row_key), None, true);

            let _ = models.update(&state, |st| {
                st.row_selection = next;
            });
            true
        }
    });
}

fn align_end<B>(child: B) -> impl IntoUiElement<fret_app::App> + use<B>
where
    B: IntoUiElement<fret_app::App>,
{
    ui::h_flex(move |cx| ui::children![cx; child])
        .layout(LayoutRefinement::default().w_full())
        .justify_end()
}

fn bottom_controls(
    cx: &mut AppComponentCx<'_>,
    state: Model<TableState>,
    output: Model<TableViewOutput>,
) -> impl IntoUiElement<fret_app::App> + use<> {
    let state_value = cx.watch_model(&state).layout().cloned().unwrap_or_default();
    let output_value = cx
        .watch_model(&output)
        .layout()
        .cloned()
        .unwrap_or_default();

    let selected_count = state_value.row_selection.len();
    let filtered_count = output_value.filtered_row_count;

    let label: Arc<str> = Arc::from(format!(
        "{selected_count} of {filtered_count} row(s) selected."
    ));

    let prev_enabled = output_value.pagination.can_prev;
    let next_enabled = output_value.pagination.can_next;

    let theme = Theme::global(&*cx.app);
    let muted_fg = theme.color_by_key("muted-foreground");
    let mut text = ui::text(label).text_sm().nowrap();
    if let Some(color) = muted_fg {
        text = text.text_color(ColorRef::Color(color));
    }

    ui::h_flex(move |cx| {
        vec![
            text.into_element(cx),
            cx.spacer(fret_ui::element::SpacerProps::default()),
            shadcn::Button::new("Previous")
                .variant(shadcn::ButtonVariant::Outline)
                .size(shadcn::ButtonSize::Sm)
                .disabled(!prev_enabled)
                .on_activate(cx.actions().listen({
                    let state = state.clone();
                    move |host, _action_cx| {
                        let _ = host.models_mut().update(&state, |st| {
                            st.pagination.page_index = st.pagination.page_index.saturating_sub(1);
                        });
                    }
                }))
                .into_element(cx),
            shadcn::Button::new("Next")
                .variant(shadcn::ButtonVariant::Outline)
                .size(shadcn::ButtonSize::Sm)
                .disabled(!next_enabled)
                .on_activate(cx.actions().listen({
                    let state = state.clone();
                    move |host, _action_cx| {
                        let _ = host.models_mut().update(&state, |st| {
                            st.pagination.page_index = st.pagination.page_index.saturating_add(1);
                        });
                    }
                }))
                .into_element(cx),
        ]
    })
    .layout(LayoutRefinement::default().w_full())
    .items_center()
    .gap(Space::N2)
}

pub fn render(cx: &mut AppComponentCx<'_>) -> impl UiChild + use<> {
    ui::v_flex(move |cx| {
        let assets = cx.slot_state(
            || {
                let data: Arc<[PaymentRow]> = Arc::from(vec![
                    PaymentRow {
                        key: 1,
                        id: Arc::from("m5gr84i9"),
                        amount_usd: 316,
                        status: Arc::from("success"),
                        email: Arc::from("ken99@example.com"),
                    },
                    PaymentRow {
                        key: 2,
                        id: Arc::from("3u1reuv4"),
                        amount_usd: 242,
                        status: Arc::from("success"),
                        email: Arc::from("Abe45@example.com"),
                    },
                    PaymentRow {
                        key: 3,
                        id: Arc::from("derv1ws0"),
                        amount_usd: 837,
                        status: Arc::from("processing"),
                        email: Arc::from("Monserrat44@example.com"),
                    },
                    PaymentRow {
                        key: 4,
                        id: Arc::from("5kma53ae"),
                        amount_usd: 874,
                        status: Arc::from("success"),
                        email: Arc::from("Silas22@example.com"),
                    },
                    PaymentRow {
                        key: 5,
                        id: Arc::from("bhqecj4p"),
                        amount_usd: 721,
                        status: Arc::from("failed"),
                        email: Arc::from("carmella@example.com"),
                    },
                ]);

                let columns: Arc<[ColumnDef<PaymentRow>]> = Arc::from(vec![
                    ColumnDef::new("select")
                        .enable_sorting(false)
                        .enable_multi_sort(false)
                        .enable_column_filter(false)
                        .enable_global_filter(false)
                        .enable_hiding(false)
                        .enable_ordering(false)
                        .enable_pinning(false)
                        .enable_resizing(false)
                        .size(44.0)
                        .min_size(44.0)
                        .max_size(44.0),
                    ColumnDef::new("status")
                        .filter_by(|row: &PaymentRow, q| row.status.as_ref().contains(q))
                        .sort_by(|a: &PaymentRow, b: &PaymentRow| a.status.cmp(&b.status))
                        .size(120.0),
                    ColumnDef::new("email")
                        .filter_by(|row: &PaymentRow, q| row.email.as_ref().contains(q))
                        .sort_by(|a: &PaymentRow, b: &PaymentRow| a.email.cmp(&b.email))
                        .size(260.0),
                    ColumnDef::new("amount")
                        .sort_by(|a: &PaymentRow, b: &PaymentRow| a.amount_usd.cmp(&b.amount_usd))
                        .size(120.0),
                    ColumnDef::new("actions")
                        .enable_sorting(false)
                        .enable_multi_sort(false)
                        .enable_column_filter(false)
                        .enable_global_filter(false)
                        .enable_hiding(false)
                        .enable_ordering(false)
                        .enable_pinning(false)
                        .enable_resizing(false)
                        .size(60.0)
                        .min_size(60.0)
                        .max_size(60.0),
                ]);

                (data, columns)
            },
            |st| st.clone(),
        );

        let state = cx.local_model_keyed("state", TableState::default);
        let output = cx.local_model_keyed("output", TableViewOutput::default);

        bind_selection_actions(cx, state.clone(), assets.0.clone(), assets.1.clone());

        let state_value = cx.watch_model(&state).layout().cloned().unwrap_or_default();

        let toolbar = shadcn::DataTableToolbar::new(state.clone(), assets.1.clone(), |col| {
            Arc::clone(&col.id)
        })
        .show_global_filter(false)
        .column_filter("email")
        .column_filter_placeholder("Filter emails...")
        .column_filter_a11y_label("Email filter")
        .faceted_selected_badges_query(shadcn::DataTableToolbarResponsiveQuery::Viewport)
        .columns_button_label("Columns")
        .show_pinning_menu(false)
        .show_selected_text(false)
        .into_element(cx)
        .test_id("ui-gallery-data-table-basic-toolbar");

        let state_for_header_checkbox = state.clone();
        let assets_for_header_checkbox = assets.clone();
        let table = shadcn::DataTable::new()
            .row_click_selection(false)
            .row_height(Px(40.0))
            .header_height(Px(40.0))
            .column_actions_menu(false)
            .output_model(output.clone())
            .refine_layout(LayoutRefinement::default().w_full())
            .into_element_with_header_cell(
                cx,
                assets.0.clone(),
                1,
                state.clone(),
                assets.1.clone(),
                |row, _index, _parent| RowKey(row.key),
                |col| match col.id.as_ref() {
                    // shadcn docs: title-cased headers.
                    "status" => Arc::<str>::from("Status"),
                    "email" => Arc::<str>::from("Email"),
                    "amount" => Arc::<str>::from("Amount"),
                    // shadcn docs: the row-actions column uses an icon-only trigger and no header label.
                    "actions" => Arc::<str>::from(""),
                    // The select column header is overridden by a checkbox header cell.
                    "select" => Arc::<str>::from(""),
                    _ => col.id.clone(),
                },
                move |cx, col, sort_state| {
                    if col.id.as_ref() != "select" {
                        if col.id.as_ref() != "amount" {
                            return None;
                        }

                        let theme = Theme::global(&*cx.app).snapshot();
                        let sort_fg = theme.color_token("muted-foreground");
                        let icon_id = match sort_state {
                            Some(true) => "lucide.arrow-down",
                            Some(false) => "lucide.arrow-up",
                            None => "lucide.chevrons-up-down",
                        };

                        let header = ui::h_flex(move |cx| {
                            vec![
                                ui::label(Arc::<str>::from("Amount"))
                                    .text_sm()
                                    .nowrap()
                                    .into_element(cx),
                                fret_ui_kit::declarative::icon::icon_with(
                                    cx,
                                    fret_icons::IconId::new_static(icon_id),
                                    Some(Px(16.0)),
                                    Some(ColorRef::Color(sort_fg)),
                                ),
                            ]
                        })
                        .layout(LayoutRefinement::default().w_full().h_full())
                        .justify_end()
                        .items_center()
                        .gap(Space::N2)
                        .into_element(cx);

                        return Some(vec![header]);
                    }

                    let state_value = cx
                        .app
                        .models()
                        .read(&state_for_header_checkbox, |st| st.clone())
                        .ok()
                        .unwrap_or_default();
                    let table = Table::builder(assets_for_header_checkbox.0.as_ref())
                        .columns(assets_for_header_checkbox.1.as_ref().to_vec())
                        .get_row_key(|row, _index, _parent| RowKey(row.key))
                        .state(state_value)
                        .build();

                    let checked = if table.is_all_page_rows_selected() {
                        Some(true)
                    } else if table.is_some_page_rows_selected() {
                        None
                    } else {
                        Some(false)
                    };

                    let model = cx.local_model_keyed("select_all_checked", || checked);
                    let _ = cx.app.models_mut().update(&model, |v| *v = checked);

                    Some(vec![
                        shadcn::Checkbox::new_optional(model)
                            .a11y_label("Select all")
                            .test_id("ui-gallery-data-table-basic-select-all")
                            .action(act::ToggleAllPageRows)
                            .into_element(cx),
                    ])
                },
                move |cx, col, row| match col.id.as_ref() {
                    "select" => {
                        let row_key = RowKey(row.key);
                        let checked = state_value.row_selection.contains(&row_key);
                        cx.keyed(
                            ("ui-gallery-data-table-basic-select-row", row_key.0),
                            |cx| {
                                let model = cx.local_model(|| checked);
                                let _ = cx.app.models_mut().update(&model, |v| *v = checked);

                                shadcn::Checkbox::new(model)
                                    .a11y_label("Select row")
                                    .test_id(Arc::<str>::from(format!(
                                        "ui-gallery-data-table-basic-select-row-{}",
                                        row_key.0
                                    )))
                                    .action(act::ToggleRowSelected)
                                    .action_payload(row_key.0)
                                    .into_element(cx)
                            },
                        )
                    }
                    "status" => cx.text(row.status.as_ref()),
                    "email" => cx.text(row.email.as_ref()),
                    "amount" => {
                        let amount = Arc::<str>::from(format!("${}.00", row.amount_usd));
                        let amount_text = ui::text(amount)
                            .text_sm()
                            .tabular_nums()
                            .nowrap()
                            .into_element(cx);
                        align_end(amount_text).into_element(cx)
                    }
                    "actions" => {
                        cx.keyed(("ui-gallery-data-table-basic-row-actions", row.key), |cx| {
                            let open = cx.local_model(|| false);

                            let trigger = shadcn::Button::new("")
                                .a11y_label("Open menu")
                                .variant(shadcn::ButtonVariant::Ghost)
                                .size(shadcn::ButtonSize::IconXs)
                                .test_id(Arc::<str>::from(format!(
                                    "ui-gallery-data-table-basic-row-actions-open-{}",
                                    row.key
                                )))
                                .icon(fret_icons::ids::ui::MORE_HORIZONTAL)
                                .into_element(cx);

                            let payment_id = row.id.clone();
                            let menu = shadcn::DropdownMenu::from_open(open)
                                .align(shadcn::DropdownMenuAlign::End)
                                .side(shadcn::DropdownMenuSide::Bottom)
                                .build(cx, trigger, move |_cx| {
                                    vec![
                                        shadcn::DropdownMenuEntry::Label(
                                            shadcn::DropdownMenuLabel::new("Actions"),
                                        ),
                                        shadcn::DropdownMenuEntry::Item(
                                            shadcn::DropdownMenuItem::new("Copy payment ID")
                                                .action(act::CopyPaymentId)
                                                .action_payload(payment_id.clone()),
                                        ),
                                        shadcn::DropdownMenuEntry::Separator,
                                        shadcn::DropdownMenuEntry::Item(
                                            shadcn::DropdownMenuItem::new("View customer")
                                                .action(act::ViewCustomer)
                                                .action_payload(payment_id.clone()),
                                        ),
                                        shadcn::DropdownMenuEntry::Item(
                                            shadcn::DropdownMenuItem::new("View payment details")
                                                .action(act::ViewPaymentDetails)
                                                .action_payload(payment_id.clone()),
                                        ),
                                    ]
                                });

                            align_end(menu).into_element(cx)
                        })
                    }
                    _ => cx.text("?"),
                },
            )
            .test_id("ui-gallery-data-table-basic-root");

        let controls = bottom_controls(cx, state, output)
            .into_element(cx)
            .test_id("ui-gallery-data-table-basic-footer");

        vec![toolbar, table, controls]
    })
    .gap(Space::N4)
    .items_start()
    .layout(LayoutRefinement::default().w_full().min_w_0())
    .into_element(cx)
}
// endregion: example
