use std::sync::Arc;

use fret_kit::prelude::*;

const CMD_ADD: &str = "todo.add";
const CMD_REMOVE_PREFIX: &str = "todo.remove.";

const TEST_ID_INPUT: &str = "todo-input";
const TEST_ID_ADD: &str = "todo-add";

fn todo_item_test_id(id: u64, suffix: &str) -> Arc<str> {
    Arc::from(format!("todo-item-{id}-{suffix}"))
}

#[derive(Clone)]
struct TodoItem {
    id: u64,
    done: Model<bool>,
    text: Arc<str>,
}

struct TodoState {
    todos: Model<Vec<TodoItem>>,
    draft: Model<String>,
    filter: Model<Option<Arc<str>>>,
    next_id: u64,
}

#[derive(Clone, Copy)]
enum TodoFilter {
    All,
    Active,
    Completed,
}

pub fn run() -> anyhow::Result<()> {
    fret_kit::app_with_hooks("todo-demo", init_window, view, |d| d.on_command(on_command))?
        .with_main_window("todo_demo", (560.0, 520.0))
        .with_ui_assets_budgets(64 * 1024 * 1024, 2048, 16 * 1024 * 1024, 4096)
        .init_app(|app| {
            shadcn::shadcn_themes::apply_shadcn_new_york_v4(
                app,
                shadcn::shadcn_themes::ShadcnBaseColor::Slate,
                shadcn::shadcn_themes::ShadcnColorScheme::Light,
            );
        })
        .run()?;
    Ok(())
}

fn init_window(app: &mut App, _window: AppWindowId) -> TodoState {
    let done_1 = app.models_mut().insert(true);
    let done_2 = app.models_mut().insert(false);
    let done_3 = app.models_mut().insert(false);

    let todos = app.models_mut().insert(vec![
        TodoItem {
            id: 1,
            done: done_1,
            text: Arc::from("体验 Shadcn UI 设计风格"),
        },
        TodoItem {
            id: 2,
            done: done_2,
            text: Arc::from("构建一个极简待办清单"),
        },
        TodoItem {
            id: 3,
            done: done_3,
            text: Arc::from("学习布局与交互组合"),
        },
    ]);
    let draft = app.models_mut().insert(String::new());
    let filter = app.models_mut().insert(Some(Arc::from("all")));
    TodoState {
        todos,
        draft,
        filter,
        next_id: 4,
    }
}

fn view(cx: &mut ElementContext<'_, App>, st: &mut TodoState) -> fret_kit::ViewElements {
    let theme = Theme::global(&*cx.app).clone();

    let filter_value = cx
        .watch_model(&st.filter)
        .layout()
        .cloned()
        .unwrap_or_default();

    let done_invalidation = if filter_value.as_deref() == Some("all") {
        Invalidation::Paint
    } else {
        Invalidation::Layout
    };

    let draft_value = cx
        .watch_model(&st.draft)
        .layout()
        .cloned()
        .unwrap_or_default();

    let add_enabled = !draft_value.trim().is_empty();

    let add_btn = shadcn::Button::new("")
        .size(shadcn::ButtonSize::Icon)
        .variant(shadcn::ButtonVariant::Default)
        .disabled(!add_enabled)
        .on_click(CMD_ADD)
        .children([icon::icon(cx, IconId::new("lucide.plus"))])
        .into_element(cx);
    let add_btn = cx.semantics(
        SemanticsProps {
            role: SemanticsRole::Button,
            test_id: Some(Arc::from(TEST_ID_ADD)),
            ..Default::default()
        },
        move |_cx| [add_btn],
    );

    let input = shadcn::Input::new(st.draft.clone())
        .a11y_label("Todo")
        .placeholder("添加新任务...")
        .submit_command(CommandId::new(CMD_ADD))
        .into_element(cx);
    let input = cx.semantics(
        SemanticsProps {
            role: SemanticsRole::TextField,
            test_id: Some(Arc::from(TEST_ID_INPUT)),
            ..Default::default()
        },
        move |_cx| [input],
    );

    let todos = cx
        .watch_model(&st.todos)
        .layout()
        .cloned()
        .unwrap_or_default();
    let mut completed = 0usize;
    for it in &todos {
        let is_done = cx
            .watch_model(&it.done)
            .invalidation(done_invalidation)
            .copied()
            .unwrap_or(false);
        if is_done {
            completed += 1;
        }
    }
    let active = todos.len().saturating_sub(completed);

    let header = shadcn::CardHeader::new([ui::h_flex(cx, |cx| {
        let left = ui::v_flex(cx, |cx| {
            [
                shadcn::CardTitle::new("待办事项").into_element(cx),
                shadcn::CardDescription::new("管理你的日常任务。").into_element(cx),
            ]
        })
        .gap(Space::N1)
        .items_start()
        .into_element(cx);

        let icon_bg = ui::container(cx, |cx| {
            [ui::h_flex(cx, |cx| {
                [icon::icon_with(
                    cx,
                    IconId::new("lucide.calendar"),
                    Some(Px(16.0)),
                    Some(ColorRef::Color(theme.color_required("muted-foreground"))),
                )]
            })
            .w_full()
            .h_full()
            .justify_center()
            .items_center()
            .into_element(cx)]
        })
        .bg(ColorRef::Color(theme.color_required("muted")))
        .rounded(Radius::Full)
        .w_px(Px(32.0))
        .h_px(Px(32.0))
        .into_element(cx);

        [left, icon_bg]
    })
    .w_full()
    .justify_between()
    .items_start()
    .into_element(cx)])
    .into_element(cx);

    let input_row = ui::h_flex(cx, |_cx| [input, add_btn])
        .w_full()
        .gap(Space::N2)
        .items_center()
        .into_element(cx);

    let list_max_h: MetricRef = Px(260.0).into();

    let tabs = shadcn::Tabs::new(st.filter.clone())
        .refine_layout(LayoutRefinement::default().w_full())
        .list_full_width(true)
        .items([
            shadcn::TabsItem::new(
                "all",
                "全部",
                [todo_list_panel(
                    cx,
                    &theme,
                    &todos,
                    TodoFilter::All,
                    list_max_h.clone(),
                )],
            ),
            shadcn::TabsItem::new(
                "active",
                "进行中",
                [todo_list_panel(
                    cx,
                    &theme,
                    &todos,
                    TodoFilter::Active,
                    list_max_h.clone(),
                )],
            ),
            shadcn::TabsItem::new(
                "completed",
                "已完成",
                [todo_list_panel(
                    cx,
                    &theme,
                    &todos,
                    TodoFilter::Completed,
                    list_max_h.clone(),
                )],
            ),
        ])
        .into_element(cx);

    let content = shadcn::CardContent::new([ui::v_flex(cx, |_cx| [input_row, tabs])
        .gap(Space::N4)
        .w_full()
        .into_element(cx)])
    .into_element(cx);

    let footer = shadcn::CardFooter::new([ui::h_flex(cx, |cx| {
        let left = cx.text(format!("{active} 个进行中"));

        let completed_badge = (completed > 0).then(|| {
            shadcn::Badge::new(format!("已完成 {completed}"))
                .variant(shadcn::BadgeVariant::Secondary)
                .into_element(cx)
        });

        let right = ui::h_flex(cx, |_cx| completed_badge.into_iter())
            .gap(Space::N2)
            .items_center()
            .into_element(cx);

        [left, right]
    })
    .w_full()
    .justify_between()
    .items_center()
    .into_element(cx)])
    .into_element(cx);

    let card = shadcn::Card::new([header, content, footer])
        .ui()
        .w_full()
        .max_w(Px(460.0))
        .into_element(cx);

    let page = ui::container(cx, |cx| {
        [ui::v_flex(cx, |cx| {
            [
                card,
                ui::raw_text(cx, "Shadcn 风格 · Fret UiAppDriver demo")
                    .text_color(ColorRef::Color(theme.color_required("muted-foreground")))
                    .nowrap()
                    .into_element(cx),
            ]
        })
        .w_full()
        .h_full()
        .justify_center()
        .items_center()
        .gap(Space::N6)
        .into_element(cx)]
    })
    .bg(ColorRef::Color(theme.color_required("muted")))
    .p(Space::N4)
    .w_full()
    .h_full()
    .into_element(cx);

    ViewElements::from([cx.semantics(
        SemanticsProps {
            role: SemanticsRole::Panel,
            label: Some(Arc::from("Debug:todo-demo:page")),
            ..Default::default()
        },
        move |_cx| [page],
    )])
}

fn todo_list_panel(
    cx: &mut ElementContext<'_, App>,
    theme: &Theme,
    todos: &[TodoItem],
    filter: TodoFilter,
    max_h: MetricRef,
) -> AnyElement {
    let filtered: Vec<(TodoItem, bool)> = todos
        .iter()
        .cloned()
        .filter_map(|t| {
            let done = cx.app.models().read(&t.done, |v| *v).ok().unwrap_or(false);
            let include = match filter {
                TodoFilter::All => true,
                TodoFilter::Active => !done,
                TodoFilter::Completed => done,
            };
            include.then_some((t, done))
        })
        .collect();

    if filtered.is_empty() {
        let label = match filter {
            TodoFilter::All => "暂无任务，开始添加吧",
            TodoFilter::Active => "太棒了！所有任务已完成",
            TodoFilter::Completed => "没有已完成的任务",
        };

        let chrome = ChromeRefinement::default()
            .border_1()
            .border_color(ColorRef::Color(theme.color_required("border")))
            .rounded(Radius::Lg)
            .px(Space::N4)
            .py(Space::N10);
        return ui::container(cx, |cx| {
            [ui::v_flex(cx, |cx| [cx.text(label)])
                .w_full()
                .items_center()
                .into_element(cx)]
        })
        .style(chrome)
        .w_full()
        .into_element(cx);
    }

    let rows = stack::vstack_build(
        cx,
        stack::VStackProps::default()
            .layout(LayoutRefinement::default().w_full())
            .gap(Space::N3),
        |cx, out| {
            out.extend(
                filtered
                    .iter()
                    .map(|(t, done)| cx.keyed(t.id, |cx| todo_row(cx, theme, t, *done))),
            );
        },
    );

    shadcn::ScrollArea::new([rows])
        .refine_layout(
            LayoutRefinement::default()
                .w_full()
                .max_h(max_h)
                .overflow_hidden(),
        )
        .into_element(cx)
}

fn todo_row(
    cx: &mut ElementContext<'_, App>,
    theme: &Theme,
    it: &TodoItem,
    done: bool,
) -> AnyElement {
    let row_chrome = ChromeRefinement::default()
        .border_1()
        .border_color(ColorRef::Color(theme.color_required("border")))
        .rounded(Radius::Lg)
        .p(Space::N3);

    let checkbox = shadcn::Checkbox::new(it.done.clone())
        .a11y_label("Done")
        .into_element(cx);
    let checkbox = cx.semantics(
        SemanticsProps {
            role: SemanticsRole::Checkbox,
            test_id: Some(todo_item_test_id(it.id, "done")),
            ..Default::default()
        },
        move |_cx| [checkbox],
    );

    let remove_btn = shadcn::Button::new("")
        .size(shadcn::ButtonSize::IconSm)
        .variant(shadcn::ButtonVariant::Ghost)
        .on_click(remove_cmd(it.id))
        .children([icon::icon_with(
            cx,
            IconId::new("lucide.trash-2"),
            Some(Px(16.0)),
            Some(ColorRef::Color(theme.color_required("muted-foreground"))),
        )])
        .into_element(cx);
    let remove_btn = cx.semantics(
        SemanticsProps {
            role: SemanticsRole::Button,
            test_id: Some(todo_item_test_id(it.id, "remove")),
            ..Default::default()
        },
        move |_cx| [remove_btn],
    );

    let mut hover = HoverRegionProps::default();
    hover.layout.size.width = Length::Fill;
    cx.hover_region(hover, move |cx, hovered| {
        let bg = hovered.then(|| theme.color_required("accent"));

        let mut chrome = row_chrome.clone();
        if let Some(bg) = bg {
            chrome = chrome.bg(ColorRef::Color(bg));
        }

        let label = todo_label_simple(cx, theme, &it.text, done, hovered);

        let left = ui::h_flex(cx, |_cx| [checkbox.clone(), label])
            .flex_1()
            .min_w_0()
            .gap(Space::N3)
            .items_center()
            .into_element(cx);

        let right = cx.interactivity_gate(true, hovered, |cx| {
            [cx.opacity(if hovered { 1.0 } else { 0.0 }, |_cx| [remove_btn.clone()])]
        });

        let row = ui::h_flex(cx, |_cx| [left, right])
            .w_full()
            .justify_between()
            .items_center()
            .into_element(cx);

        [ui::container(cx, |_cx| [row])
            .style(chrome)
            .w_full()
            .into_element(cx)]
    })
}

fn todo_label_simple(
    cx: &mut ElementContext<'_, App>,
    theme: &Theme,
    text: &Arc<str>,
    done: bool,
    hovered: bool,
) -> AnyElement {
    let fg = if hovered {
        theme.color_required("accent-foreground")
    } else if done {
        theme.color_required("muted-foreground")
    } else {
        theme.color_required("foreground")
    };

    cx.text_props(TextProps {
        layout: Default::default(),
        text: text.clone(),
        style: None,
        color: Some(fg),
        wrap: TextWrap::None,
        overflow: TextOverflow::Clip,
    })
}

fn on_command(
    app: &mut App,
    services: &mut dyn UiServices,
    window: AppWindowId,
    _ui: &mut UiTree<App>,
    state: &mut TodoState,
    cmd: &CommandId,
) {
    let _ = services;

    match cmd.as_str() {
        CMD_ADD => {
            let text = app
                .models()
                .read(&state.draft, |s| s.trim().to_string())
                .ok()
                .unwrap_or_default();
            if text.is_empty() {
                return;
            }

            let id = state.next_id;
            state.next_id = state.next_id.saturating_add(1);

            let done = app.models_mut().insert(false);

            let _ = app.models_mut().update(&state.todos, |todos| {
                todos.insert(
                    0,
                    TodoItem {
                        id,
                        done,
                        text: Arc::from(text),
                    },
                );
            });
            let _ = app.models_mut().update(&state.draft, |s| s.clear());
            app.push_effect(Effect::Redraw(window));
        }
        other => {
            if let Some(id) = other.strip_prefix(CMD_REMOVE_PREFIX) {
                let Ok(id) = id.parse::<u64>() else {
                    return;
                };
                let _ = app.models_mut().update(&state.todos, |todos| {
                    todos.retain(|t| t.id != id);
                });
                app.push_effect(Effect::Redraw(window));
            }
        }
    }
}

fn remove_cmd(id: u64) -> CommandId {
    CommandId::new(format!("{CMD_REMOVE_PREFIX}{id}"))
}
