pub const SOURCE: &str = include_str!("kanban.rs");

// region: example
use fret_ui_kit::ui;
use fret_ui_shadcn::{facade as shadcn, prelude::*};

pub fn render<H: UiHost>(cx: &mut ElementContext<'_, H>) -> AnyElement {
    cx.named("shadcn-extras-kanban-demo", |cx| {
        let items = cx.local_model_keyed("items", || {
            vec![
                shadcn::raw::extras::KanbanItem::new("card-1", "Write docs", "backlog"),
                shadcn::raw::extras::KanbanItem::new("card-2", "Port block", "backlog"),
                shadcn::raw::extras::KanbanItem::new("card-3", "Add gates", "in_progress"),
                shadcn::raw::extras::KanbanItem::new("card-4", "Fix regressions", "in_progress"),
                shadcn::raw::extras::KanbanItem::new("card-5", "Ship", "done"),
            ]
        });

        let columns = vec![
            shadcn::raw::extras::KanbanColumn::new("backlog", "Backlog"),
            shadcn::raw::extras::KanbanColumn::new("in_progress", "In Progress"),
            shadcn::raw::extras::KanbanColumn::new("done", "Done"),
        ];

        shadcn::raw::extras::Kanban::new(columns, items)
            .test_id("ui-gallery-shadcn-extras-kanban")
            .into_element_with(cx, |cx, item, ctx| {
                let title = ui::text(item.name.clone())
                    .font_medium()
                    .w_full()
                    .min_w_0()
                    .truncate()
                    .into_element(cx);

                let badge = shadcn::Badge::new(item.column.clone())
                    .variant(shadcn::BadgeVariant::Secondary)
                    .into_element(cx);

                let meta = ui::h_flex(move |_cx| vec![badge])
                    .gap(Space::N2)
                    .items_center()
                    .layout(LayoutRefinement::default().w_full())
                    .into_element(cx);

                let header = if ctx.mode == shadcn::raw::extras::KanbanCardMode::Board {
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
