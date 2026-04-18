pub const SOURCE: &str = include_str!("actions.rs");

// region: example
use std::sync::Arc;

use fret::{AppComponentCx, UiChild};
use fret_core::FontWeight;
use fret_ui_kit::IntoUiElement;
use fret_ui_shadcn::{facade as shadcn, prelude::*};

fn action_row(
    product: &'static str,
    price: &'static str,
    open_model: Model<bool>,
    slug: &'static str,
) -> impl IntoUiElement<fret_app::App> + use<> {
    let row_test_id = format!("ui-gallery-table-actions-row-{slug}");
    let trigger_id = format!("ui-gallery-table-actions-trigger-{slug}");

    shadcn::table_row(3, move |cx| {
        let dropdown = shadcn::DropdownMenu::from_open(open_model.clone())
            .align(shadcn::DropdownMenuAlign::End)
            .build(
                cx,
                shadcn::Button::new("")
                    .a11y_label("Open menu")
                    .variant(shadcn::ButtonVariant::Ghost)
                    .size(shadcn::ButtonSize::Icon)
                    .icon(fret_icons::ids::ui::MORE_HORIZONTAL)
                    .toggle_model(open_model.clone())
                    .test_id(Arc::<str>::from(trigger_id.clone())),
                |_cx| {
                    vec![
                        shadcn::DropdownMenuEntry::Item(shadcn::DropdownMenuItem::new("Edit")),
                        shadcn::DropdownMenuEntry::Item(shadcn::DropdownMenuItem::new("Duplicate")),
                        shadcn::DropdownMenuEntry::Separator,
                        shadcn::DropdownMenuEntry::Item(
                            shadcn::DropdownMenuItem::new("Delete").variant(
                                shadcn::raw::dropdown_menu::DropdownMenuItemVariant::Destructive,
                            ),
                        ),
                    ]
                },
            );

        ui::children![
            cx;
            shadcn::table_cell(ui::text(product).font_weight(FontWeight::MEDIUM)),
            shadcn::table_cell(ui::text(price)),
            shadcn::table_cell(dropdown).text_align_end(),
        ]
    })
    .test_id(row_test_id)
}

pub fn render(cx: &mut AppComponentCx<'_>) -> impl UiChild + use<> {
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
                                .refine_layout(LayoutRefinement::default().w_px(Px(140.0))),
                            shadcn::table_head("Actions")
                                .text_align_end()
                                .refine_layout(LayoutRefinement::default().w_px(Px(96.0))),
                        ]
                    })
                    .border_bottom(true),
                ]
            }),
            shadcn::table_body(|_cx| {
                vec![
                    action_row("Wireless Mouse", "$29.99", open_1, "wireless-mouse"),
                    action_row(
                        "Mechanical Keyboard",
                        "$129.99",
                        open_2,
                        "mechanical-keyboard",
                    ),
                    action_row("USB-C Hub", "$49.99", open_3, "usb-c-hub"),
                ]
            }),
        ]
    })
    .refine_layout(LayoutRefinement::default().w_full())
    .into_element(cx)
    .test_id("ui-gallery-table-actions")
}

// endregion: example
