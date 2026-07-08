pub const SOURCE: &str = include_str!("kanban.rs");

// region: example
use fret::{AppComponentCx, UiChild};
use fret_ui_kit::{declarative::text as decl_text, ui};
use fret_ui_shadcn::{facade as shadcn, prelude::*};

pub fn render(cx: &mut AppComponentCx<'_>) -> impl UiChild + use<> {
    cx.named("shadcn-extras-kanban-demo", |cx| {
        let items = cx.local_model_keyed("items", || {
            vec![
                shadcn::extras::KanbanItem::new("card-1", "Write docs", "backlog"),
                shadcn::extras::KanbanItem::new("card-2", "Port block", "backlog"),
                shadcn::extras::KanbanItem::new("card-3", "Add gates", "in_progress"),
                shadcn::extras::KanbanItem::new("card-4", "Fix regressions", "in_progress"),
                shadcn::extras::KanbanItem::new("card-5", "Ship", "done"),
            ]
        });

        let columns = vec![
            shadcn::extras::KanbanColumn::new("backlog", "Backlog"),
            shadcn::extras::KanbanColumn::new("in_progress", "In Progress"),
            shadcn::extras::KanbanColumn::new("done", "Done"),
        ];

        shadcn::extras::Kanban::new(columns, items)
            .test_id("ui-gallery-shadcn-extras-kanban")
            .into_element_with(cx, |cx, item, ctx| {
                let title = decl_text::text_button_label(cx, item.name.clone());

                let badge = shadcn::Badge::new(item.column.clone())
                    .variant(shadcn::BadgeVariant::Secondary)
                    .into_element(cx);

                let meta = ui::h_flex(move |_cx| vec![badge])
                    .gap(Space::N2)
                    .items_center()
                    .layout(LayoutRefinement::default().w_full())
                    .into_element(cx);

                let header = if ctx.mode == shadcn::extras::KanbanCardMode::Board {
                    let checkbox =
                        shadcn::Checkbox::new_controllable(cx, None, false).into_element(cx);
                    ui::h_flex(move |_cx| vec![checkbox, title])
                        .gap(Space::N2)
                        .items_center()
                        .layout(LayoutRefinement::default().w_full())
                        .into_element(cx)
                } else {
                    title
                };

                ui::v_flex(move |_cx| vec![header, meta])
                    .gap(Space::N1)
                    .items_stretch()
                    .layout(LayoutRefinement::default().w_full())
                    .into_element(cx)
            })
    })
}
// endregion: example
