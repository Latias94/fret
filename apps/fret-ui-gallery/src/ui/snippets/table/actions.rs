pub const SOURCE: &str = include_str!("actions.rs");

// region: example
use std::sync::Arc;

use fret::{UiChild, UiCx};
use fret_ui_kit::IntoUiElement;
use fret_ui_shadcn::{facade as shadcn, prelude::*};

fn align_end<B>(child: B) -> impl IntoUiElement<fret_app::App> + use<B>
where
    B: IntoUiElement<fret_app::App>,
{
    ui::h_flex(move |cx| ui::children![cx; child])
        .layout(LayoutRefinement::default().w_full())
        .justify_end()
}

fn action_row(
    product: &'static str,
    price: &'static str,
    open_model: Model<bool>,
    key: &'static str,
) -> impl IntoUiElement<fret_app::App> + use<> {
    let row_test_id = format!("ui-gallery-table-actions-row-{key}");
    let trigger_id = format!("ui-gallery-table-actions-trigger-{key}");

    shadcn::table_row(3, move |cx| {
        let dropdown = shadcn::DropdownMenu::from_open(open_model.clone()).build(
            cx,
            shadcn::Button::new("?")
                .variant(shadcn::ButtonVariant::Ghost)
                .size(shadcn::ButtonSize::Icon)
                .toggle_model(open_model.clone())
                .test_id(Arc::<str>::from(trigger_id.clone())),
            |_cx| {
                vec![
                    shadcn::DropdownMenuEntry::Item(shadcn::DropdownMenuItem::new("Edit")),
                    shadcn::DropdownMenuEntry::Item(shadcn::DropdownMenuItem::new("Duplicate")),
                    shadcn::DropdownMenuEntry::Separator,
                    shadcn::DropdownMenuEntry::Item(
                        shadcn::DropdownMenuItem::new("Delete")
                            .variant(shadcn::raw::dropdown_menu::DropdownMenuItemVariant::Destructive),
                    ),
                ]
            },
        );

        ui::children![
            cx;
            shadcn::table_cell(ui::text(product)),
            shadcn::table_cell(ui::text(price)),
            shadcn::table_cell(align_end(dropdown)),
        ]
    })
    .test_id(row_test_id)
}

pub fn render(cx: &mut UiCx<'_>) -> impl UiChild + use<> {
    let open_1 = cx.local_model_keyed("actions_open_1", || false);
    let open_2 = cx.local_model_keyed("actions_open_2", || false);
    let open_3 = cx.local_model_keyed("actions_open_3", || false);

    shadcn::table(|cx| {
        ui::children![
            cx;
            shadcn::table_header(|cx| {
                ui::children![
                    cx;
                    shadcn::table_row(3, |cx| {
                        ui::children![
                            cx;
                            shadcn::table_head("Product")
                                .refine_layout(LayoutRefinement::default().w_px(Px(280.0))),
                            shadcn::table_head("Price")
                                .refine_layout(LayoutRefinement::default().w_px(Px(180.0))),
                            shadcn::table_head("Actions")
                                .refine_layout(LayoutRefinement::default().w_px(Px(120.0))),
                        ]
                    })
                    .border_bottom(true),
                ]
            }),
            shadcn::table_body(|_cx| {
                vec![
                    action_row("Gaming Mouse", "$129.99", open_1, "row-1"),
                    action_row("Mechanical Keyboard", "$89.99", open_2, "row-2"),
                    action_row("4K Monitor", "$299.99", open_3, "row-3"),
                ]
            }),
        ]
    })
    .refine_layout(LayoutRefinement::default().w_full())
    .into_element(cx)
    .test_id("ui-gallery-table-actions")
}

// endregion: example
