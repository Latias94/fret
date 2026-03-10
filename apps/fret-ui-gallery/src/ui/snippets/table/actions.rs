pub const SOURCE: &str = include_str!("actions.rs");

// region: example
use fret_app::App;
use fret_ui_kit::ui::UiElementSinkExt;
use fret_ui_shadcn::{self as shadcn, prelude::*};
use std::sync::Arc;

#[derive(Default, Clone)]
struct TableModels {
    actions_open_1: Option<Model<bool>>,
    actions_open_2: Option<Model<bool>>,
    actions_open_3: Option<Model<bool>>,
}

fn ensure_models(cx: &mut ElementContext<'_, App>) -> (Model<bool>, Model<bool>, Model<bool>) {
    let state = cx.with_state(TableModels::default, |st| st.clone());
    match (
        state.actions_open_1,
        state.actions_open_2,
        state.actions_open_3,
    ) {
        (Some(open_1), Some(open_2), Some(open_3)) => (open_1, open_2, open_3),
        _ => {
            let models = cx.app.models_mut();
            let open_1 = models.insert(false);
            let open_2 = models.insert(false);
            let open_3 = models.insert(false);
            cx.with_state(TableModels::default, |st| {
                st.actions_open_1 = Some(open_1.clone());
                st.actions_open_2 = Some(open_2.clone());
                st.actions_open_3 = Some(open_3.clone());
            });
            (open_1, open_2, open_3)
        }
    }
}

fn align_end(cx: &mut ElementContext<'_, App>, child: AnyElement) -> AnyElement {
    ui::h_flex(move |_cx| [child])
        .layout(LayoutRefinement::default().w_full())
        .justify_end()
        .into_element(cx)
}

fn action_row(
    cx: &mut ElementContext<'_, App>,
    product: &'static str,
    price: &'static str,
    open_model: Model<bool>,
    key: &'static str,
) -> AnyElement {
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
                    shadcn::DropdownMenuItem::new("Delete")
                        .variant(shadcn::dropdown_menu::DropdownMenuItemVariant::Destructive),
                ),
            ]
        },
    );

    shadcn::TableRow::build(3, move |cx, out| {
        let action_cell = align_end(cx, dropdown);
        out.push_ui(cx, shadcn::TableCell::build(ui::text(product)));
        out.push_ui(cx, shadcn::TableCell::build(ui::text(price)));
        out.push_ui(cx, shadcn::TableCell::build(action_cell));
    })
    .into_element(cx)
    .test_id(row_test_id)
}

pub fn render(cx: &mut ElementContext<'_, App>) -> AnyElement {
    let (open_1, open_2, open_3) = ensure_models(cx);

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
                out.push(action_row(cx, "Gaming Mouse", "$129.99", open_1, "row-1"));
                out.push(action_row(cx, "Mechanical Keyboard", "$89.99", open_2, "row-2"));
                out.push(action_row(cx, "4K Monitor", "$299.99", open_3, "row-3"));
            }),
        );
    })
    .refine_layout(LayoutRefinement::default().w_full())
    .into_element(cx)
    .test_id("ui-gallery-table-actions")
}

// endregion: example
