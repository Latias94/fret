use super::super::super::super::*;

pub(in crate::ui) fn preview_ai_queue_demo(
    cx: &mut ElementContext<'_, App>,
    _theme: &Theme,
) -> Vec<AnyElement> {
    use std::sync::Arc;

    use fret_runtime::Model;
    use fret_ui::Invalidation;
    use fret_ui_kit::declarative::stack;
    use fret_ui_kit::{LayoutRefinement, Space};

    #[derive(Default)]
    struct DemoModels {
        item_1_completed: Option<Model<bool>>,
    }

    let item_1_completed = cx.with_state(DemoModels::default, |st| st.item_1_completed.clone());
    let item_1_completed = match item_1_completed {
        Some(model) => model,
        None => {
            let model = cx.app.models_mut().insert(false);
            cx.with_state(DemoModels::default, |st| {
                st.item_1_completed = Some(model.clone())
            });
            model
        }
    };

    let item_1_is_completed = cx
        .get_model_copied(&item_1_completed, Invalidation::Layout)
        .unwrap_or(false);

    let complete_marker = item_1_is_completed
        .then(|| cx.text("").test_id("ui-ai-queue-complete-marker"))
        .unwrap_or_else(|| cx.text(""));

    let pending_items: Vec<AnyElement> = (0..40)
        .map(|i| {
            let idx = i + 1;
            let id = format!("ui-ai-queue-item-{idx}");
            let hover_marker_id = format!("ui-ai-queue-item-{idx}-hovered-marker");

            if idx == 1 {
                let on_complete: ui_ai::OnQueueItemActionActivate = Arc::new({
                    let item_1_completed = item_1_completed.clone();
                    move |host, action_cx| {
                        let _ = host.models_mut().update(&item_1_completed, |v| *v = true);
                        host.notify(action_cx);
                    }
                });

                ui_ai::QueueItem::new()
                    .test_id(id)
                    .into_element(cx, move |cx, st| {
                        let indicator = ui_ai::QueueItemIndicator::new()
                            .completed(false)
                            .into_element(cx);
                        let content = ui_ai::QueueItemContent::new(format!("Pending item {idx}"))
                            .into_element(cx);
                        let left = stack::hstack(
                            cx,
                            stack::HStackProps::default()
                                .layout(LayoutRefinement::default().w_full().min_w_0())
                                .gap(Space::N2)
                                .items_center(),
                            move |_cx| vec![indicator, content],
                        );

                        let actions_visible = st.hovered;
                        let check = cx.text("✓");
                        let actions =
                            ui_ai::QueueItemActions::new([ui_ai::QueueItemAction::new("Complete")
                                .children([check])
                                .visible(actions_visible)
                                .on_activate(on_complete)
                                .test_id("ui-ai-queue-item-1-action-complete")
                                .into_element(cx)])
                            .into_element(cx);

                        let row = stack::hstack(
                            cx,
                            stack::HStackProps::default()
                                .layout(LayoutRefinement::default().w_full().min_w_0())
                                .items_center()
                                .justify_between()
                                .gap(Space::N2),
                            move |_cx| vec![left, actions],
                        );

                        let mut out = vec![
                            row,
                            ui_ai::QueueItemDescription::new("Demo item with an action.")
                                .into_element(cx),
                        ];

                        if st.hovered {
                            out.push(cx.text("").test_id(hover_marker_id.clone()));
                        }
                        out
                    })
            } else {
                ui_ai::QueueItem::new()
                    .test_id(id)
                    .into_element(cx, move |cx, st| {
                        let indicator = ui_ai::QueueItemIndicator::new()
                            .completed(false)
                            .into_element(cx);
                        let content = ui_ai::QueueItemContent::new(format!("Pending item {idx}"))
                            .into_element(cx);
                        let left = stack::hstack(
                            cx,
                            stack::HStackProps::default()
                                .layout(LayoutRefinement::default().w_full().min_w_0())
                                .gap(Space::N2)
                                .items_center(),
                            move |_cx| vec![indicator, content],
                        );

                        let mut out = vec![
                            left,
                            ui_ai::QueueItemDescription::new("A plain pending item.")
                                .into_element(cx),
                        ];
                        if st.hovered {
                            out.push(cx.text("").test_id(hover_marker_id.clone()));
                        }
                        out
                    })
            }
        })
        .collect();

    let completed_items: Vec<AnyElement> = (0..10)
        .map(|i| {
            let idx = i + 1;
            ui_ai::QueueItem::new().into_element(cx, move |cx, _st| {
                let indicator = ui_ai::QueueItemIndicator::new()
                    .completed(true)
                    .into_element(cx);
                let content = ui_ai::QueueItemContent::new(format!("Completed item {idx}"))
                    .completed(true)
                    .into_element(cx);
                vec![
                    stack::hstack(
                        cx,
                        stack::HStackProps::default()
                            .layout(LayoutRefinement::default().w_full().min_w_0())
                            .gap(Space::N2)
                            .items_center(),
                        move |_cx| vec![indicator, content],
                    ),
                    ui_ai::QueueItemDescription::new("Completed item description.")
                        .completed(true)
                        .into_element(cx),
                ]
            })
        })
        .collect();

    let queue = ui_ai::Queue::new([
        ui_ai::QueueSection::uncontrolled(true).into_element(
            cx,
            |cx, st| {
                let label = ui_ai::QueueSectionLabel::new("Pending")
                    .count(40)
                    .into_element(cx, st.is_open);
                ui_ai::QueueSectionTrigger::new(st.open, [label]).into_element(cx, st.is_open)
            },
            move |cx| {
                ui_ai::QueueSectionContent::new([ui_ai::QueueList::new(pending_items)
                    .viewport_test_id("ui-ai-queue-pending-list-viewport")
                    .max_height_px(fret_core::Px(240.0))
                    .into_element(cx)])
                .into_element(cx)
            },
        ),
        ui_ai::QueueSection::uncontrolled(false).into_element(
            cx,
            |cx, st| {
                let label = ui_ai::QueueSectionLabel::new("Completed")
                    .count(10)
                    .into_element(cx, st.is_open);
                ui_ai::QueueSectionTrigger::new(st.open, [label])
                    .test_id("ui-ai-queue-completed-trigger")
                    .into_element(cx, st.is_open)
            },
            move |cx| {
                ui_ai::QueueSectionContent::new([ui_ai::QueueList::new(completed_items)
                    .viewport_test_id("ui-ai-queue-completed-list-viewport")
                    .max_height_px(fret_core::Px(160.0))
                    .into_element(cx)])
                .into_element(cx)
            },
        ),
    ])
    .test_id("ui-ai-queue-root")
    .into_element(cx);

    vec![stack::vstack(
        cx,
        stack::VStackProps::default()
            .layout(LayoutRefinement::default().w_full().min_w_0())
            .gap(Space::N4),
        move |cx| {
            vec![
                cx.text("Queue (AI Elements)"),
                cx.text("Hover an item to reveal actions; complete sets a demo marker."),
                queue,
                complete_marker,
            ]
        },
    )]
}
