pub const SOURCE: &str = include_str!("actions.rs");

// region: example
use fret::UiCx;
use fret_ui_kit::IntoUiElement;
use fret_ui_kit::ui::UiElementSinkExt;
use fret_ui_shadcn::{facade as shadcn, prelude::*};
use std::sync::Arc;

fn align_end<B>(child: B) -> impl IntoUiElement<fret_app::App> + use<B>
where
    B: IntoUiElement<fret_app::App>,
{
    ui::h_flex(move |cx| ui::children![cx; child])
        .layout(LayoutRefinement::default().w_full())
        .justify_end()
}

fn action_row(
    cx: &mut UiCx<'_>,
    product: &'static str,
    price: &'static str,
    open_model: Model<bool>,
    key: &'static str,
) -> impl IntoUiElement<fret_app::App> + use<> {
    let row_test_id = format!("ui-gallery-table-actions-row-{key}");
    let trigger_id = format!("ui-gallery-table-actions-trigger-{key}");

    let dropdown = shadcn::DropdownMenu::new(open_model.clone()).into_element(
        cx,
        |cx| {
            shadcn::Button::new("?")
                .variant(shadcn::ButtonVariant::Ghost)
                .size(shadcn::ButtonSize::Icon)
                .toggle_model(open_model.clone())
                .test_id(Arc::<str>::from(trigger_id.clone()))
                .into_element(cx)
        },
        |_cx| {
            vec![
                shadcn::DropdownMenuEntry::Item(shadcn::DropdownMenuItem::new("Edit")),
                shadcn::DropdownMenuEntry::Item(shadcn::DropdownMenuItem::new("Duplicate")),
                shadcn::DropdownMenuEntry::Separator,
                shadcn::DropdownMenuEntry::Item(
                    shadcn::DropdownMenuItem::new("Delete").variant(
                        fret_ui_shadcn::dropdown_menu::DropdownMenuItemVariant::Destructive,
                    ),
                ),
            ]
        },
    );

    shadcn::TableRow::build(3, move |cx, out| {
        let action_cell = align_end(dropdown).into_element(cx);
        out.push_ui(cx, shadcn::TableCell::build(ui::text(product)));
        out.push_ui(cx, shadcn::TableCell::build(ui::text(price)));
        out.push_ui(cx, shadcn::TableCell::build(action_cell));
    })
    .into_element(cx)
    .test_id(row_test_id)
}

pub fn render(cx: &mut UiCx<'_>) -> AnyElement {
    let open_1 = cx.local_model_keyed("actions_open_1", || false);
    let open_2 = cx.local_model_keyed("actions_open_2", || false);
    let open_3 = cx.local_model_keyed("actions_open_3", || false);

    shadcn::Table::build(|cx, out| {
        out.push_ui(
            cx,
            shadcn::TableHeader::build(|cx, out| {
                out.push(
                    shadcn::TableRow::build(3, |cx, out| {
                        out.push(
                            shadcn::TableHead::new("Product")
                                .refine_layout(LayoutRefinement::default().w_px(Px(280.0)))
                                .into_element(cx),
                        );
                        out.push(
                            shadcn::TableHead::new("Price")
                                .refine_layout(LayoutRefinement::default().w_px(Px(180.0)))
                                .into_element(cx),
                        );
                        out.push(
                            shadcn::TableHead::new("Actions")
                                .refine_layout(LayoutRefinement::default().w_px(Px(120.0)))
                                .into_element(cx),
                        );
                    })
                    .border_bottom(true)
                    .into_element(cx),
                );
            }),
        );
        out.push_ui(
            cx,
            shadcn::TableBody::build(|cx, out| {
                out.push(
                    action_row(cx, "Gaming Mouse", "$129.99", open_1, "row-1").into_element(cx),
                );
                out.push(
                    action_row(cx, "Mechanical Keyboard", "$89.99", open_2, "row-2")
                        .into_element(cx),
                );
                out.push(action_row(cx, "4K Monitor", "$299.99", open_3, "row-3").into_element(cx));
            }),
        );
    })
    .refine_layout(LayoutRefinement::default().w_full())
    .into_element(cx)
    .test_id("ui-gallery-table-actions")
}

// endregion: example
