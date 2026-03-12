pub const SOURCE: &str = include_str!("queue_demo.rs");

// region: example
use fret_core::{ImageColorSpace, ImageId};
use fret_runtime::Model;
use fret_ui::Invalidation;
use fret_ui_ai as ui_ai;
use fret_ui_assets::{ImageSource, ui::ImageSourceElementContextExt as _};
use fret_ui_kit::declarative::icon as decl_icon;
use fret_ui_kit::ui;
use fret_ui_kit::{LayoutRefinement, Space};
use fret_ui_shadcn::prelude::*;
use std::sync::{Arc, OnceLock};

#[derive(Default)]
struct DemoModels {
    messages: Option<Model<Vec<DemoMessage>>>,
    todos: Option<Model<Vec<DemoTodo>>>,
    action_revision: Option<Model<u64>>,
}

#[derive(Debug, Clone)]
enum DemoMessagePart {
    Text(Arc<str>),
    Image,
    File { filename: Arc<str> },
}

#[derive(Debug, Clone)]
struct DemoMessage {
    id: Arc<str>,
    parts: Arc<[DemoMessagePart]>,
}

#[derive(Debug, Clone)]
struct DemoTodo {
    id: Arc<str>,
    title: Arc<str>,
    description: Option<Arc<str>>,
    completed: bool,
}

fn demo_queue_image_id<H: UiHost>(cx: &mut ElementContext<'_, H>) -> Option<ImageId> {
    static SOURCE: OnceLock<ImageSource> = OnceLock::new();
    let source = SOURCE.get_or_init(|| {
        // Keep the snippet self-contained instead of depending on repo-relative demo assets.
        ImageSource::rgba8(
            192,
            120,
            demo_queue_preview_rgba8(192, 120),
            ImageColorSpace::Srgb,
        )
    });
    cx.use_image_source_state(source).image
}

fn demo_queue_preview_rgba8(width: u32, height: u32) -> Vec<u8> {
    let mut out = vec![0u8; (width as usize) * (height as usize) * 4];
    let width_f = (width.saturating_sub(1)).max(1) as f32;
    let height_f = (height.saturating_sub(1)).max(1) as f32;

    for y in 0..height {
        for x in 0..width {
            let idx = ((y as usize) * (width as usize) + (x as usize)) * 4;
            let fx = x as f32 / width_f;
            let fy = y as f32 / height_f;

            let mut r = (18.0 + 88.0 * fx) as u8;
            let mut g = (24.0 + 132.0 * (1.0 - fy)) as u8;
            let mut b = (46.0 + 164.0 * fy) as u8;

            let border = x < 3 || y < 3 || x + 3 >= width || y + 3 >= height;
            let card = x > width / 8 && x < width / 2 && y > height / 5 && y < (height * 4) / 5;
            let status =
                x > (width * 3) / 5 && x < (width * 7) / 8 && y > height / 3 && y < height / 2;

            if border {
                r = 245;
                g = 245;
                b = 245;
            } else if card {
                r = r.saturating_add(18);
                g = g.saturating_add(18);
                b = b.saturating_add(18);
            } else if status {
                r = 250;
                g = 250;
                b = 250;
            }

            out[idx] = r;
            out[idx + 1] = g;
            out[idx + 2] = b;
            out[idx + 3] = 255;
        }
    }

    out
}

fn default_messages() -> Vec<DemoMessage> {
    vec![
        DemoMessage {
            id: Arc::<str>::from("msg-1"),
            parts: Arc::from([DemoMessagePart::Text(Arc::<str>::from(
                "How do I set up the project?",
            ))]),
        },
        DemoMessage {
            id: Arc::<str>::from("msg-2"),
            parts: Arc::from([DemoMessagePart::Text(Arc::<str>::from(
                "What is the roadmap for Q4?",
            ))]),
        },
        DemoMessage {
            id: Arc::<str>::from("msg-3"),
            parts: Arc::from([
                DemoMessagePart::Text(Arc::<str>::from("Update the default logo to this png.")),
                DemoMessagePart::Image,
                DemoMessagePart::File {
                    filename: Arc::<str>::from("setup-guide.png"),
                },
            ]),
        },
        DemoMessage {
            id: Arc::<str>::from("msg-4"),
            parts: Arc::from([DemoMessagePart::Text(Arc::<str>::from(
                "Please generate a changelog.",
            ))]),
        },
        DemoMessage {
            id: Arc::<str>::from("msg-5"),
            parts: Arc::from([DemoMessagePart::Text(Arc::<str>::from(
                "Add dark mode support.",
            ))]),
        },
        DemoMessage {
            id: Arc::<str>::from("msg-6"),
            parts: Arc::from([DemoMessagePart::Text(Arc::<str>::from(
                "Optimize database queries.",
            ))]),
        },
        DemoMessage {
            id: Arc::<str>::from("msg-7"),
            parts: Arc::from([DemoMessagePart::Text(Arc::<str>::from(
                "Set up CI/CD pipeline.",
            ))]),
        },
    ]
}

fn default_todos() -> Vec<DemoTodo> {
    vec![
        DemoTodo {
            id: Arc::<str>::from("todo-1"),
            title: Arc::<str>::from("Write project documentation"),
            description: Some(Arc::<str>::from("Complete the README and API docs")),
            completed: true,
        },
        DemoTodo {
            id: Arc::<str>::from("todo-2"),
            title: Arc::<str>::from("Implement authentication"),
            description: None,
            completed: false,
        },
        DemoTodo {
            id: Arc::<str>::from("todo-3"),
            title: Arc::<str>::from("Fix bug #42"),
            description: Some(Arc::<str>::from("Resolve crash on settings page")),
            completed: false,
        },
        DemoTodo {
            id: Arc::<str>::from("todo-4"),
            title: Arc::<str>::from("Refactor queue logic"),
            description: Some(Arc::<str>::from("Unify queue and todo state management")),
            completed: false,
        },
        DemoTodo {
            id: Arc::<str>::from("todo-5"),
            title: Arc::<str>::from("Add unit tests"),
            description: Some(Arc::<str>::from("Increase test coverage for hooks")),
            completed: false,
        },
    ]
}

pub fn render<H: UiHost + 'static>(cx: &mut ElementContext<'_, H>) -> AnyElement {
    let messages = cx.with_state(DemoModels::default, |st| st.messages.clone());
    let messages = match messages {
        Some(model) => model,
        None => {
            let model = cx.app.models_mut().insert(default_messages());
            cx.with_state(DemoModels::default, |st| st.messages = Some(model.clone()));
            model
        }
    };

    let todos = cx.with_state(DemoModels::default, |st| st.todos.clone());
    let todos = match todos {
        Some(model) => model,
        None => {
            let model = cx.app.models_mut().insert(default_todos());
            cx.with_state(DemoModels::default, |st| st.todos = Some(model.clone()));
            model
        }
    };

    let action_revision = cx.with_state(DemoModels::default, |st| st.action_revision.clone());
    let action_revision = match action_revision {
        Some(model) => model,
        None => {
            let model = cx.app.models_mut().insert(0_u64);
            cx.with_state(DemoModels::default, |st| {
                st.action_revision = Some(model.clone())
            });
            model
        }
    };

    let messages_snapshot = cx
        .get_model_cloned(&messages, Invalidation::Layout)
        .unwrap_or_default();
    let todos_snapshot = cx
        .get_model_cloned(&todos, Invalidation::Layout)
        .unwrap_or_default();
    let action_rev = cx
        .get_model_copied(&action_revision, Invalidation::Layout)
        .unwrap_or(0);

    let action_marker = (action_rev > 0)
        .then(|| {
            cx.container(fret_ui::element::ContainerProps::default(), |_cx| {
                Vec::<AnyElement>::new()
            })
            .test_id("ui-ai-queue-action-marker")
        })
        .unwrap_or_else(|| cx.text(""));

    let image_id = demo_queue_image_id(cx);

    let queued_items = {
        let mut out = Vec::new();
        for (idx, message) in messages_snapshot.iter().enumerate() {
            let idx = idx + 1;
            let msg = message.clone();
            let keyed_id = msg.id.clone();
            let hover_marker_id =
                Arc::<str>::from(format!("ui-ai-queue-item-{idx}-hovered-marker"));

            let on_remove: ui_ai::OnQueueItemActionActivate = Arc::new({
                let messages = messages.clone();
                let action_revision = action_revision.clone();
                let msg_id = msg.id.clone();
                move |host, action_cx| {
                    let _ = host.models_mut().update(&messages, |v| {
                        v.retain(|m| m.id != msg_id);
                    });
                    let _ = host
                        .models_mut()
                        .update(&action_revision, |v| *v = v.saturating_add(1));
                    host.notify(action_cx);
                }
            });

            let on_send: ui_ai::OnQueueItemActionActivate = Arc::new({
                let messages = messages.clone();
                let action_revision = action_revision.clone();
                let msg_id = msg.id.clone();
                move |host, action_cx| {
                    let _ = host.models_mut().update(&messages, |v| {
                        v.retain(|m| m.id != msg_id);
                    });
                    let _ = host
                        .models_mut()
                        .update(&action_revision, |v| *v = v.saturating_add(1));
                    host.notify(action_cx);
                }
            });

            out.push(cx.keyed(keyed_id, move |cx| {
                ui_ai::QueueItem::new()
                    .test_id(format!("ui-ai-queue-item-{idx}"))
                    .into_element(cx, move |cx, st| {
                        let summary: Arc<str> = {
                            let mut s = String::new();
                            for part in msg.parts.iter() {
                                if let DemoMessagePart::Text(text) = part {
                                    if !s.is_empty() {
                                        s.push(' ');
                                    }
                                    s.push_str(text.as_ref());
                                }
                            }
                            if s.trim().is_empty() {
                                Arc::<str>::from("(queued message)")
                            } else {
                                Arc::<str>::from(s.trim().to_string())
                            }
                        };

                        let indicator = ui_ai::QueueItemIndicator::new().into_element(cx);
                        let content = ui_ai::QueueItemContent::new(summary).into_element(cx);
                        let left = ui::h_row(move |_cx| vec![indicator, content])
                            .layout(LayoutRefinement::default().flex_1().min_w_0())
                            .gap(Space::N2)
                            .items_center()
                            .into_element(cx);

                        let actions_visible = st.hovered;
                        let remove_icon = decl_icon::icon_with(
                            cx,
                            fret_icons::IconId::new_static("lucide.trash-2"),
                            Some(fret_core::Px(12.0)),
                            None,
                        );
                        let send_icon = decl_icon::icon_with(
                            cx,
                            fret_icons::IconId::new_static("lucide.arrow-up"),
                            Some(fret_core::Px(14.0)),
                            None,
                        );
                        let mut remove_action = ui_ai::QueueItemAction::new("Remove from queue")
                            .children([remove_icon])
                            .visible(actions_visible)
                            .on_activate(on_remove.clone());
                        if idx == 1 {
                            remove_action =
                                remove_action.test_id("ui-ai-queue-item-1-action-remove");
                        }
                        let remove_action = remove_action.into_element(cx);

                        let send_action = ui_ai::QueueItemAction::new("Send now")
                            .children([send_icon])
                            .visible(actions_visible)
                            .on_activate(on_send.clone())
                            .into_element(cx);

                        let mut actions =
                            ui_ai::QueueItemActions::new([remove_action, send_action])
                                .refine_layout(LayoutRefinement::default().flex_none())
                                .into_element(cx);
                        if idx == 1 && st.hovered {
                            actions = actions.test_id(hover_marker_id.clone());
                        }

                        let row = ui::h_flex(move |_cx| vec![left, actions])
                            .layout(LayoutRefinement::default().w_full().min_w_0())
                            .items_center()
                            .justify_between()
                            .gap(Space::N2)
                            .into_element(cx);

                        let mut children = vec![row];

                        let mut attachments: Vec<AnyElement> = Vec::new();
                        for part in msg.parts.iter() {
                            match part {
                                DemoMessagePart::Text(_) => {}
                                DemoMessagePart::Image => {
                                    if let Some(id) = image_id {
                                        attachments
                                            .push(ui_ai::QueueItemImage::new(id).into_element(cx));
                                    }
                                }
                                DemoMessagePart::File { filename } => {
                                    attachments.push(
                                        ui_ai::QueueItemFile::new(filename.clone())
                                            .into_element(cx),
                                    );
                                }
                            }
                        }

                        if !attachments.is_empty() {
                            children.push(
                                ui_ai::QueueItemAttachment::new(attachments).into_element(cx),
                            );
                        }

                        children
                    })
            }));
        }
        out
    };

    let todo_items = {
        let mut out = Vec::new();
        for todo in todos_snapshot.iter() {
            let todo = todo.clone();
            let keyed_id = todo.id.clone();
            let on_remove: ui_ai::OnQueueItemActionActivate = Arc::new({
                let todos = todos.clone();
                let action_revision = action_revision.clone();
                let todo_id = todo.id.clone();
                move |host, action_cx| {
                    let _ = host.models_mut().update(&todos, |v| {
                        v.retain(|t| t.id != todo_id);
                    });
                    let _ = host
                        .models_mut()
                        .update(&action_revision, |v| *v = v.saturating_add(1));
                    host.notify(action_cx);
                }
            });

            out.push(cx.keyed(keyed_id, move |cx| {
                ui_ai::QueueItem::new().into_element(cx, move |cx, st| {
                    let indicator = ui_ai::QueueItemIndicator::new()
                        .completed(todo.completed)
                        .into_element(cx);
                    let content = ui_ai::QueueItemContent::new(todo.title.clone())
                        .completed(todo.completed)
                        .into_element(cx);
                    let left = ui::h_row(move |_cx| vec![indicator, content])
                        .layout(LayoutRefinement::default().flex_1().min_w_0())
                        .gap(Space::N2)
                        .items_center()
                        .into_element(cx);

                    let actions_visible = st.hovered;
                    let remove_icon = decl_icon::icon_with(
                        cx,
                        fret_icons::IconId::new_static("lucide.trash-2"),
                        Some(fret_core::Px(12.0)),
                        None,
                    );
                    let actions =
                        ui_ai::QueueItemActions::new([ui_ai::QueueItemAction::new("Remove todo")
                            .children([remove_icon])
                            .visible(actions_visible)
                            .on_activate(on_remove.clone())
                            .into_element(cx)])
                        .refine_layout(LayoutRefinement::default().flex_none())
                        .into_element(cx);

                    let row = ui::h_flex(move |_cx| vec![left, actions])
                        .layout(LayoutRefinement::default().w_full().min_w_0())
                        .items_center()
                        .justify_between()
                        .gap(Space::N2)
                        .into_element(cx);

                    let mut out = vec![row];
                    if let Some(desc) = todo.description.as_ref() {
                        out.push(
                            ui_ai::QueueItemDescription::new(desc.clone())
                                .completed(todo.completed)
                                .into_element(cx),
                        );
                    }
                    out
                })
            }));
        }
        out
    };

    let queue = ui_ai::Queue::new([
        ui_ai::QueueSection::uncontrolled(true).into_element(
            cx,
            |cx, st| {
                let label = ui_ai::QueueSectionLabel::new("Queued")
                    .count(messages_snapshot.len() as u32)
                    .into_element(cx, st.is_open);
                ui_ai::QueueSectionTrigger::new(st.open, [label])
                    .test_id("ui-ai-queue-queued-trigger")
                    .into_element(cx, st.is_open)
            },
            move |cx| {
                ui_ai::QueueSectionContent::new([ui_ai::QueueList::new(queued_items)
                    .viewport_test_id("ui-ai-queue-queued-list-viewport")
                    .max_height_px(fret_core::Px(160.0))
                    .into_element(cx)])
                .into_element(cx)
            },
        ),
        ui_ai::QueueSection::uncontrolled(true).into_element(
            cx,
            |cx, st| {
                let label = ui_ai::QueueSectionLabel::new("Todo")
                    .count(todos_snapshot.len() as u32)
                    .into_element(cx, st.is_open);
                ui_ai::QueueSectionTrigger::new(st.open, [label])
                    .test_id("ui-ai-queue-todo-trigger")
                    .into_element(cx, st.is_open)
            },
            move |cx| {
                ui_ai::QueueSectionContent::new([ui_ai::QueueList::new(todo_items)
                    .viewport_test_id("ui-ai-queue-todo-list-viewport")
                    .max_height_px(fret_core::Px(160.0))
                    .into_element(cx)])
                .into_element(cx)
            },
        ),
    ])
    .test_id("ui-ai-queue-root")
    .into_element(cx);

    ui::v_flex(move |cx| {
        vec![
            cx.text("Queue (AI Elements)"),
            cx.text("Hover an item to reveal actions; actions increment a demo marker."),
            queue,
            action_marker,
        ]
    })
    .layout(LayoutRefinement::default().w_full().min_w_0())
    .gap(Space::N4)
    .into_element(cx)
}
// endregion: example
