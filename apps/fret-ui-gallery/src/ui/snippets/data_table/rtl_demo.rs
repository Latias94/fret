pub const SOURCE: &str = include_str!("rtl_demo.rs");

// region: example
use fret::app::UiCxActionsExt as _;
use fret::{AppComponentCx, UiChild};
use fret_core::Px;
use fret_runtime::Model;
use fret_ui::Theme;
use fret_ui_headless::table::{ColumnDef, RowKey, TableState};
use fret_ui_kit::IntoUiElement;
use fret_ui_kit::declarative::ModelWatchExt as _;
use fret_ui_kit::declarative::table::TableViewOutput;
use fret_ui_shadcn::{facade as shadcn, prelude::*};
use std::sync::Arc;

mod act {
    fret::payload_actions!([
        CopyPaymentId(std::sync::Arc<str>) = "ui-gallery.data_table.rtl.copy_payment_id.v1",
        ViewCustomer(std::sync::Arc<str>) = "ui-gallery.data_table.rtl.view_customer.v1",
        ViewPaymentDetails(std::sync::Arc<str>) = "ui-gallery.data_table.rtl.view_payment_details.v1"
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

#[derive(Debug, Clone, Copy)]
struct Lang {
    dir: shadcn::LayoutDirection,
    filter_emails: &'static str,
    columns: &'static str,
    status: &'static str,
    email: &'static str,
    amount: &'static str,
    actions: &'static str,
    open_menu: &'static str,
    copy_payment_id: &'static str,
    view_customer: &'static str,
    view_payment_details: &'static str,
    rows_selected: &'static str,
    previous: &'static str,
    next: &'static str,
}

const LANG_EN: Lang = Lang {
    dir: shadcn::LayoutDirection::Ltr,
    filter_emails: "Filter emails...",
    columns: "Columns",
    status: "Status",
    email: "Email",
    amount: "Amount",
    actions: "Actions",
    open_menu: "Open menu",
    copy_payment_id: "Copy payment ID",
    view_customer: "View customer",
    view_payment_details: "View payment details",
    rows_selected: "row(s) selected.",
    previous: "Previous",
    next: "Next",
};

const LANG_AR: Lang = Lang {
    dir: shadcn::LayoutDirection::Rtl,
    filter_emails: "تصفية البريد الإلكتروني...",
    columns: "الأعمدة",
    status: "الحالة",
    email: "البريد الإلكتروني",
    amount: "المبلغ",
    actions: "الإجراءات",
    open_menu: "فتح القائمة",
    copy_payment_id: "نسخ معرف الدفع",
    view_customer: "عرض العميل",
    view_payment_details: "عرض تفاصيل الدفع",
    rows_selected: "صف(وف) محدد.",
    previous: "السابق",
    next: "التالي",
};

fn align_inline_start<B>(
    cx: &mut AppComponentCx<'_>,
    child: B,
) -> impl IntoUiElement<fret_app::App> + use<B>
where
    B: IntoUiElement<fret_app::App>,
{
    let dir = shadcn::use_direction(cx, None);
    let row =
        ui::h_flex(move |cx| ui::children![cx; child]).layout(LayoutRefinement::default().w_full());
    let row = match dir {
        shadcn::LayoutDirection::Rtl => row.justify_end(),
        shadcn::LayoutDirection::Ltr => row.justify_start(),
    };
    row
}

fn bottom_controls(
    cx: &mut AppComponentCx<'_>,
    state: Model<TableState>,
    output: Model<TableViewOutput>,
    lang: Lang,
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
        "{} of {} {}",
        selected_count, filtered_count, lang.rows_selected
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
            shadcn::Button::new(lang.previous)
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
            shadcn::Button::new(lang.next)
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
                ColumnDef::new("status").size(120.0),
                ColumnDef::new("email").size(260.0),
                ColumnDef::new("amount").size(120.0),
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

    let dir_rtl = cx.local_model_keyed("dir_rtl", || true);
    let state = cx.local_model_keyed("state", TableState::default);
    let output = cx.local_model_keyed("output", TableViewOutput::default);

    let is_rtl = cx.watch_model(&dir_rtl).layout().copied().unwrap_or(true);
    let lang = if is_rtl { LANG_AR } else { LANG_EN };

    let direction_toggle = ui::h_row(move |cx| {
        vec![
            ui::label(Arc::<str>::from("RTL"))
                .text_sm()
                .into_element(cx),
            shadcn::Switch::new(dir_rtl.clone())
                .a11y_label("Toggle RTL")
                .into_element(cx)
                .test_id("ui-gallery-data-table-rtl-toggle"),
        ]
    })
    .gap(Space::N3)
    .items_center()
    .into_element(cx);

    shadcn::DirectionProvider::new(lang.dir).into_element(cx, move |cx| {
        let toolbar = shadcn::DataTableToolbar::new(state.clone(), assets.1.clone(), |col| {
            Arc::clone(&col.id)
        })
        .show_global_filter(false)
        .column_filter("email")
        .column_filter_placeholder(lang.filter_emails)
        .column_filter_a11y_label(lang.filter_emails)
        .columns_button_label(lang.columns)
        .show_pinning_menu(false)
        .show_selected_text(false)
        .into_element(cx)
        .test_id("ui-gallery-data-table-rtl-toolbar");

        let table = shadcn::DataTable::new()
            .row_click_selection(false)
            .row_height(Px(40.0))
            .header_height(Px(40.0))
            .column_actions_menu(false)
            .output_model(output.clone())
            .refine_layout(LayoutRefinement::default().w_full())
            .into_element(
                cx,
                assets.0.clone(),
                1,
                state.clone(),
                assets.1.clone(),
                |row, _index, _parent| RowKey(row.key),
                move |col| match col.id.as_ref() {
                    "status" => Arc::<str>::from(lang.status),
                    "email" => Arc::<str>::from(lang.email),
                    "amount" => Arc::<str>::from(lang.amount),
                    "actions" => Arc::<str>::from(""),
                    _ => col.id.clone(),
                },
                move |cx, col, row| match col.id.as_ref() {
                    "status" => cx.text(row.status.as_ref()),
                    "email" => cx.text(row.email.as_ref()),
                    "amount" => {
                        let amount = Arc::<str>::from(format!("${}.00", row.amount_usd));
                        let amount_text = ui::text(amount)
                            .text_sm()
                            .tabular_nums()
                            .nowrap()
                            .into_element(cx);
                        align_inline_start(cx, amount_text).into_element(cx)
                    }
                    "actions" => {
                        cx.keyed(("ui-gallery-data-table-rtl-row-actions", row.key), |cx| {
                            let open = cx.local_model(|| false);

                            let trigger = shadcn::Button::new("")
                                .a11y_label(lang.open_menu)
                                .variant(shadcn::ButtonVariant::Ghost)
                                .size(shadcn::ButtonSize::IconXs)
                                .test_id(Arc::<str>::from(format!(
                                    "ui-gallery-data-table-rtl-row-actions-open-{}",
                                    row.key
                                )))
                                .icon(fret_icons::ids::ui::MORE_HORIZONTAL)
                                .into_element(cx);

                            let payment_id = row.id.clone();
                            shadcn::DropdownMenu::from_open(open)
                                .align(shadcn::DropdownMenuAlign::End)
                                .side(shadcn::DropdownMenuSide::Bottom)
                                .build(cx, trigger, move |_cx| {
                                    vec![
                                        shadcn::DropdownMenuEntry::Label(
                                            shadcn::DropdownMenuLabel::new(lang.actions),
                                        ),
                                        shadcn::DropdownMenuEntry::Item(
                                            shadcn::DropdownMenuItem::new(lang.copy_payment_id)
                                                .action(act::CopyPaymentId)
                                                .action_payload(payment_id.clone()),
                                        ),
                                        shadcn::DropdownMenuEntry::Separator,
                                        shadcn::DropdownMenuEntry::Item(
                                            shadcn::DropdownMenuItem::new(lang.view_customer)
                                                .action(act::ViewCustomer)
                                                .action_payload(payment_id.clone()),
                                        ),
                                        shadcn::DropdownMenuEntry::Item(
                                            shadcn::DropdownMenuItem::new(
                                                lang.view_payment_details,
                                            )
                                            .action(act::ViewPaymentDetails)
                                            .action_payload(payment_id.clone()),
                                        ),
                                    ]
                                })
                        })
                    }
                    _ => cx.text("?"),
                },
            )
            .test_id("ui-gallery-data-table-rtl-table");

        let controls = bottom_controls(cx, state, output, lang)
            .into_element(cx)
            .test_id("ui-gallery-data-table-rtl-footer");

        ui::v_flex(move |_cx| vec![direction_toggle, toolbar, table, controls])
            .gap(Space::N4)
            .items_start()
            .layout(LayoutRefinement::default().w_full().min_w_0())
            .into_element(cx)
            .test_id("ui-gallery-data-table-rtl-root")
    })
}
// endregion: example
