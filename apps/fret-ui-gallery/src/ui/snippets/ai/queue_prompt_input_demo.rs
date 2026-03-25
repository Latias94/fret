pub const SOURCE: &str = include_str!("queue_prompt_input_demo.rs");

// region: example
use fret::{UiChild, UiCx};
use fret_ui::Invalidation;
use fret_ui::Theme;
use fret_ui::element::AnyElement;
use fret_ui_ai as ui_ai;
use fret_ui_kit::declarative::icon as decl_icon;
use fret_ui_kit::ui;
use fret_ui_kit::{ChromeRefinement, Corners4, LayoutRefinement, Space};
use std::sync::Arc;

#[derive(Clone, Copy)]
struct DemoModel {
    chef: &'static str,
    chef_slug: &'static str,
    id: &'static str,
    name: &'static str,
    providers: &'static [&'static str],
}

const MODELS: &[DemoModel] = &[
    DemoModel {
        chef: "OpenAI",
        chef_slug: "openai",
        id: "gpt-4o",
        name: "GPT-4o",
        providers: &["openai", "azure"],
    },
    DemoModel {
        chef: "OpenAI",
        chef_slug: "openai",
        id: "gpt-4o-mini",
        name: "GPT-4o Mini",
        providers: &["openai", "azure"],
    },
    DemoModel {
        chef: "Anthropic",
        chef_slug: "anthropic",
        id: "claude-opus-4-20250514",
        name: "Claude 4 Opus",
        providers: &["anthropic", "azure", "google", "amazon-bedrock"],
    },
    DemoModel {
        chef: "Anthropic",
        chef_slug: "anthropic",
        id: "claude-sonnet-4-20250514",
        name: "Claude 4 Sonnet",
        providers: &["anthropic", "azure", "google", "amazon-bedrock"],
    },
    DemoModel {
        chef: "Google",
        chef_slug: "google",
        id: "gemini-2.0-flash-exp",
        name: "Gemini 2.0 Flash",
        providers: &["google"],
    },
];

fn sample_todos() -> Vec<ui_ai::QueueTodo> {
    vec![
        ui_ai::QueueTodo::new("todo-1", "Write project documentation")
            .description("Complete the README and API docs")
            .status(ui_ai::QueueTodoStatus::Completed),
        ui_ai::QueueTodo::new("todo-2", "Implement authentication")
            .status(ui_ai::QueueTodoStatus::Pending),
        ui_ai::QueueTodo::new("todo-3", "Fix bug #42")
            .description("Resolve crash on settings page")
            .status(ui_ai::QueueTodoStatus::Pending),
        ui_ai::QueueTodo::new("todo-4", "Refactor queue logic")
            .description("Unify queue and todo state management")
            .status(ui_ai::QueueTodoStatus::Pending),
        ui_ai::QueueTodo::new("todo-5", "Add unit tests")
            .description("Increase test coverage for hooks")
            .status(ui_ai::QueueTodoStatus::Pending),
    ]
}

pub fn render(cx: &mut UiCx<'_>) -> impl UiChild + use<> {
    let todos = cx.local_model_keyed("todos", sample_todos);

    let text = cx.local_model_keyed("text", String::new);
    let attachments = cx.local_model_keyed("attachments", Vec::<ui_ai::AttachmentData>::new);
    let sent_count = cx.local_model_keyed("sent_count", || 0u32);

    let model_open = cx.local_model_keyed("model_open", || false);
    let model_query = cx.local_model_keyed("model_query", String::new);
    let model_selected = cx.local_model_keyed("model_selected", || Arc::<str>::from(MODELS[0].id));

    let todos_now = cx
        .get_model_cloned(&todos, Invalidation::Layout)
        .unwrap_or_default();

    let selected_now = cx
        .get_model_cloned(&model_selected, Invalidation::Paint)
        .unwrap_or_else(|| Arc::<str>::from(MODELS[0].id));

    let selected_model = MODELS
        .iter()
        .find(|m| m.id == selected_now.as_ref())
        .copied();

    let on_send: fret_ui::action::OnActivate = Arc::new({
        let sent_count = sent_count.clone();
        move |host, action_cx, _reason| {
            let _ = host
                .models_mut()
                .update(&sent_count, |v| *v = v.saturating_add(1));
            host.notify(action_cx);
            host.request_redraw(action_cx.window);
        }
    });

    let on_add_attachments: fret_ui::action::OnActivate = Arc::new({
        let attachments = attachments.clone();
        move |host, action_cx, _reason| {
            let file = ui_ai::AttachmentFileData::new("att-0")
                .filename("design.png")
                .media_type("image/png")
                .size_bytes(42_000);
            let item = ui_ai::AttachmentData::File(file);
            let _ = host.models_mut().update(&attachments, |v| {
                if v.iter().any(|x| x.id().as_ref() == "att-0") {
                    return;
                }
                v.push(item);
            });
            host.notify(action_cx);
            host.request_redraw(action_cx.window);
        }
    });

    let todo_items = {
        let mut out = Vec::new();
        for todo in todos_now.iter() {
            let todo = todo.clone();
            let keyed_id = todo.id.clone();

            let on_remove: ui_ai::OnQueueItemActionActivate = Arc::new({
                let todos = todos.clone();
                let todo_id = todo.id.clone();
                move |host, action_cx| {
                    let _ = host.models_mut().update(&todos, |v| {
                        v.retain(|t| t.id != todo_id);
                    });
                    host.notify(action_cx);
                    host.request_redraw(action_cx.window);
                }
            });

            out.push(cx.keyed(keyed_id, move |cx| {
                let mut item = ui_ai::QueueItem::new();
                if todo.id.as_ref() == "todo-1" {
                    item = item.test_id("ui-ai-queue-prompt-input-todo-1");
                }
                item.into_element(cx, move |cx, _st| {
                    let is_completed = todo.is_completed();

                    let indicator = ui_ai::QueueItemIndicator::new()
                        .completed(is_completed)
                        .into_element(cx);

                    let content = ui_ai::QueueItemContent::new(todo.title.clone())
                        .completed(is_completed)
                        .into_element(cx);

                    let remove_icon = decl_icon::icon_with(
                        cx,
                        fret_icons::IconId::new_static("lucide.trash-2"),
                        Some(fret_core::Px(12.0)),
                        None,
                    );
                    let mut remove_action = ui_ai::QueueItemAction::new("Remove todo")
                        .children([remove_icon])
                        .on_activate(on_remove.clone());
                    if todo.id.as_ref() == "todo-1" {
                        remove_action =
                            remove_action.test_id("ui-ai-queue-prompt-input-todo-1-remove");
                    }
                    let actions = ui_ai::QueueItemActions::new([remove_action.into_element(cx)])
                        .refine_layout(LayoutRefinement::default().flex_none())
                        .into_element(cx);

                    let row = ui::h_row(move |_cx| vec![indicator, content, actions])
                        .layout(LayoutRefinement::default().w_full().min_w_0())
                        .gap(Space::N2)
                        .items_center()
                        .into_element(cx);

                    let mut children = vec![row];
                    if let Some(desc) = todo.description.as_ref() {
                        children.push(
                            ui_ai::QueueItemDescription::new(desc.clone())
                                .completed(is_completed)
                                .into_element(cx),
                        );
                    }
                    children
                })
            }));
        }
        out
    };

    let queue = {
        let theme = Theme::global(&*cx.app).clone();
        let radius = {
            let base = theme.metric_token("metric.radius.lg");
            fret_core::Px(base.0 + 4.0)
        };

        let mut children: Vec<AnyElement> = Vec::new();
        if !todos_now.is_empty() {
            children.push(
                ui_ai::QueueSection::uncontrolled(true).into_content_only_element(cx, move |cx| {
                    ui_ai::QueueSectionContent::new([ui_ai::QueueList::new(todo_items)
                        .max_height_px(fret_core::Px(150.0))
                        .viewport_test_id("ui-ai-queue-prompt-input-todo-viewport")
                        .refine_layout(LayoutRefinement::default().mt(Space::N0))
                        .into_element(cx)])
                    .into_element(cx)
                }),
            );
        }

        ui_ai::Queue::new(children)
            .test_id("ui-ai-queue-prompt-input-queue")
            .refine_layout(LayoutRefinement::default().w_percent(95.0).min_w_0())
            .refine_style(ChromeRefinement::default().corner_radii(Corners4::tltrbrbl(
                radius,
                radius,
                fret_core::Px(0.0),
                fret_core::Px(0.0),
            )))
            .into_element(cx)
    };

    let prompt = ui_ai::PromptInputProvider::new()
        .text_model(text)
        .attachments_model(attachments)
        .into_element_with_children(cx, move |cx, controller| {
            let menu = ui_ai::PromptInputActionMenu::new(
                ui_ai::PromptInputActionMenuContent::new([]).item(
                    ui_ai::PromptInputActionMenuItem::new("Add photos or files")
                        .leading_icon(fret_icons::IconId::new("lucide.image"))
                        .on_activate(on_add_attachments.clone())
                        .test_id("ui-ai-queue-prompt-input-add-attachments-item"),
                ),
            )
            .trigger(
                ui_ai::PromptInputActionMenuTrigger::new()
                    .test_id("ui-ai-queue-prompt-input-action-menu-trigger"),
            );

            let search = ui_ai::PromptInputButton::new("Search")
                .children([
                    decl_icon::icon(cx, fret_icons::IconId::new("lucide.globe")),
                    ui::text("Search").into_element(cx),
                ])
                .test_id("ui-ai-queue-prompt-input-search")
                .into_element(cx);

            let trigger_children = {
                let mut children: Vec<AnyElement> = Vec::new();
                if let Some(model) = selected_model {
                    children.push(ui_ai::ModelSelectorLogo::new(model.chef_slug).into_element(cx));
                    children.push(ui_ai::ModelSelectorName::new(model.name).into_element(cx));
                }
                children
            };

            let on_select = |value: &'static str| -> fret_ui::action::OnActivate {
                let model_selected = model_selected.clone();
                let model_query = model_query.clone();
                let model_open = model_open.clone();
                Arc::new(move |host, action_cx, _reason| {
                    let _ = host
                        .models_mut()
                        .update(&model_selected, |v| *v = Arc::<str>::from(value));
                    let _ = host.models_mut().update(&model_query, |v| v.clear());
                    let _ = host.models_mut().update(&model_open, |v| *v = false);
                    host.notify(action_cx);
                    host.request_redraw(action_cx.window);
                })
            };

            let make_item = |cx: &mut UiCx<'_>, m: DemoModel| {
                let is_selected = m.id == selected_now.as_ref();

                let check = decl_icon::icon_with(
                    cx,
                    fret_icons::ids::ui::CHECK,
                    Some(fret_core::Px(16.0)),
                    None,
                );
                let check = cx.opacity(if is_selected { 1.0 } else { 0.0 }, move |_cx| vec![check]);
                let check = ui::h_row(move |_cx| vec![check])
                    .layout(
                        LayoutRefinement::default()
                            .ml_auto()
                            .w_px(fret_core::Px(16.0))
                            .h_px(fret_core::Px(16.0)),
                    )
                    .justify_center()
                    .items_center()
                    .into_element(cx);

                let mut item = ui_ai::ModelSelectorItem::new(m.name)
                    .value(m.id)
                    .child(ui_ai::ModelSelectorLogo::new(m.chef_slug))
                    .child(ui_ai::ModelSelectorName::new(m.name))
                    .child(ui_ai::ModelSelectorLogoGroup::new(
                        m.providers
                            .iter()
                            .copied()
                            .map(ui_ai::ModelSelectorLogo::new),
                    ))
                    .child(check)
                    .on_select_action(on_select(m.id));
                if m.id == "claude-opus-4-20250514" {
                    item = item.test_id("ui-ai-queue-prompt-input-model-item-claude-opus-4");
                }
                item
            };

            let mut chefs: Vec<&'static str> = Vec::new();
            for m in MODELS {
                if !chefs.contains(&m.chef) {
                    chefs.push(m.chef);
                }
            }

            let mut groups = Vec::new();
            for chef in chefs {
                let mut items = Vec::new();
                for m in MODELS.iter().copied().filter(|m| m.chef == chef) {
                    items.push(make_item(cx, m));
                }
                groups.push(ui_ai::ModelSelectorGroup::new(items).heading(chef));
            }

            let trigger_btn = ui_ai::PromptInputButton::new("Model")
                .children(trigger_children)
                .test_id("ui-ai-queue-prompt-input-model-trigger-button")
                .into_element(cx);

            let model_selector = ui_ai::ModelSelector::new()
                .open_model(model_open.clone())
                .children([
                    ui_ai::ModelSelectorChild::Trigger(
                        ui_ai::ModelSelectorTrigger::new(trigger_btn)
                            .test_id("ui-ai-queue-prompt-input-model-trigger"),
                    ),
                    ui_ai::ModelSelectorChild::Content(
                        ui_ai::ModelSelectorContent::new([])
                            .input(
                                ui_ai::ModelSelectorInput::new(model_query.clone())
                                    .placeholder("Search models...")
                                    .input_test_id("ui-ai-queue-prompt-input-model-input"),
                            )
                            .list(
                                ui_ai::ModelSelectorList::new_entries(groups)
                                    .empty_text("No models found.")
                                    .query_model(model_query.clone()),
                            )
                            .test_id_root("ui-ai-queue-prompt-input-model-content"),
                    ),
                ])
                .into_element(cx);

            let input = ui_ai::PromptInputRoot::new(controller.text)
                .attachments(controller.attachments.expect("provider sets attachments"))
                .on_send(on_send)
                .on_add_attachments(on_add_attachments)
                .multiple(true)
                .placeholder("Write a message...")
                .test_id_root("ui-ai-queue-prompt-input")
                .test_id_textarea("ui-ai-queue-prompt-input-textarea")
                .test_id_send("ui-ai-queue-prompt-input-send")
                .test_id_stop("ui-ai-queue-prompt-input-stop")
                .test_id_attachments("ui-ai-queue-prompt-input-attachments")
                .into_element_with_slots(cx, move |cx| ui_ai::PromptInputSlots {
                    block_start: vec![
                        ui_ai::PromptInputHeader::new([
                            ui_ai::PromptInputAttachmentsRow::new().into_element(cx)
                        ])
                        .into_element(cx),
                    ],
                    block_end: vec![
                        ui_ai::PromptInputFooter::new(
                            [ui_ai::PromptInputTools::empty()
                                .child(menu)
                                .child(search)
                                .child(model_selector)],
                            [ui_ai::PromptInputSubmit::new()],
                        )
                        .into_element(cx),
                    ],
                });

            vec![input]
        });

    let sent_marker = (cx
        .get_model_copied(&sent_count, Invalidation::Paint)
        .unwrap_or(0)
        == 1)
        .then(|| cx.text("").test_id("ui-ai-queue-prompt-input-sent-count-1"));

    let mut stack_children: Vec<AnyElement> = Vec::new();
    stack_children.push(queue);
    stack_children.push(prompt);
    if let Some(marker) = sent_marker {
        stack_children.push(marker);
    }

    ui::v_flex(move |cx| {
        vec![
            cx.text("Queue + PromptInput (AI Elements)"),
            cx.text("Docs-aligned composition: content-only QueueSection above PromptInput tools."),
            ui::v_flex(move |_cx| stack_children)
                .layout(
                    LayoutRefinement::default()
                        .w_full()
                        .min_w_0()
                        .h_px(fret_core::Px(400.0))
                        .min_h_0(),
                )
                .justify_end()
                .gap(Space::N0)
                .into_element(cx),
        ]
    })
    .layout(LayoutRefinement::default().w_full().min_w_0())
    .gap(Space::N4)
    .into_element(cx)
}
// endregion: example
