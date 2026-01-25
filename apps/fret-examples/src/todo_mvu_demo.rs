use std::sync::Arc;

use fret_kit::prelude::*;

const TEST_ID_INPUT: &str = "todo-mvu-input";
const TEST_ID_ADD: &str = "todo-mvu-add";

fn todo_item_test_id(id: u64, suffix: &str) -> Arc<str> {
    Arc::from(format!("todo-mvu-item-{id}-{suffix}"))
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

#[derive(Debug, Clone)]
enum Msg {
    Add,
    Remove(u64),
    ClearDone,
}

pub fn run() -> anyhow::Result<()> {
    fret_kit::mvu::app::<TodoMvuProgram>("todo-mvu-demo")?
        .with_main_window("todo_mvu_demo", (560.0, 520.0))
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

struct TodoMvuProgram;

impl MvuProgram for TodoMvuProgram {
    type State = TodoState;
    type Message = Msg;

    fn init(app: &mut App, _window: AppWindowId) -> Self::State {
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
                text: Arc::from("减少 CommandId 样板"),
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

    fn update(app: &mut App, st: &mut Self::State, message: Self::Message) {
        match message {
            Msg::Add => {
                let draft = app
                    .models()
                    .read(&st.draft, |s| s.trim().to_string())
                    .unwrap_or_default();
                if draft.is_empty() {
                    return;
                }

                let id = st.next_id;
                st.next_id += 1;
                let done = app.models_mut().insert(false);
                let item = TodoItem {
                    id,
                    done,
                    text: Arc::from(draft),
                };
                let _ = app.models_mut().update(&st.todos, |v| v.push(item));
                let _ = app.models_mut().update(&st.draft, |s| s.clear());
            }
            Msg::Remove(id) => {
                let _ = app
                    .models_mut()
                    .update(&st.todos, |v| v.retain(|t| t.id != id));
            }
            Msg::ClearDone => {
                let done_ids: Vec<u64> = app
                    .models()
                    .read(&st.todos, |items| {
                        items
                            .iter()
                            .filter_map(|t| {
                                app.models()
                                    .read(&t.done, |v| *v)
                                    .ok()
                                    .unwrap_or(false)
                                    .then_some(t.id)
                            })
                            .collect()
                    })
                    .unwrap_or_default();
                if done_ids.is_empty() {
                    return;
                }
                let _ = app.models_mut().update(&st.todos, |v| {
                    v.retain(|t| !done_ids.iter().any(|id| *id == t.id))
                });
            }
        }
    }

    fn view(
        cx: &mut ElementContext<'_, App>,
        st: &mut Self::State,
        msg: &mut MessageRouter<Self::Message>,
    ) -> Vec<AnyElement> {
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
            .on_click(msg.cmd(Msg::Add))
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
            .submit_command(msg.cmd(Msg::Add))
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
        let todos_with_done: Vec<(TodoItem, bool)> = todos
            .into_iter()
            .map(|t| {
                let done = cx
                    .watch_model(&t.done)
                    .invalidation(done_invalidation)
                    .copied()
                    .unwrap_or(false);
                (t, done)
            })
            .collect();

        let completed = todos_with_done.iter().filter(|(_, done)| *done).count();
        let active = todos_with_done.len().saturating_sub(completed);

        let header = shadcn::CardHeader::new([ui::h_flex(cx, |cx| {
            let left = ui::v_flex(cx, |cx| {
                [
                    shadcn::CardTitle::new("待办事项（MVU）").into_element(cx),
                    shadcn::CardDescription::new("Typed messages via MessageRouter.")
                        .into_element(cx),
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
            .w_px(MetricRef::Px(Px(32.0)))
            .h_px(MetricRef::Px(Px(32.0)))
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

        let list_max_h = MetricRef::Px(Px(260.0));

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
                        &todos_with_done,
                        None,
                        msg,
                        list_max_h.clone(),
                    )],
                ),
                shadcn::TabsItem::new(
                    "active",
                    format!("进行中 ({active})"),
                    [todo_list_panel(
                        cx,
                        &theme,
                        &todos_with_done,
                        Some(false),
                        msg,
                        list_max_h.clone(),
                    )],
                ),
                shadcn::TabsItem::new(
                    "done",
                    format!("已完成 ({completed})"),
                    [todo_list_panel(
                        cx,
                        &theme,
                        &todos_with_done,
                        Some(true),
                        msg,
                        list_max_h,
                    )],
                ),
            ])
            .into_element(cx);

        let clear_done = shadcn::Button::new("清除已完成")
            .variant(shadcn::ButtonVariant::Secondary)
            .disabled(completed == 0)
            .on_click(msg.cmd(Msg::ClearDone))
            .into_element(cx);

        let footer = shadcn::CardFooter::new([ui::h_flex(cx, |cx| {
            let left = cx.text(format!("{active} 个进行中"));

            let completed_badge = (completed > 0).then(|| {
                shadcn::Badge::new(format!("已完成 {completed}"))
                    .variant(shadcn::BadgeVariant::Secondary)
                    .into_element(cx)
            });

            let right = ui::h_flex(cx, move |_cx| {
                completed_badge
                    .into_iter()
                    .chain(std::iter::once(clear_done))
            })
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

        let content = shadcn::CardContent::new([ui::v_flex(cx, |_cx| [input_row, tabs])
            .gap(Space::N4)
            .w_full()
            .into_element(cx)])
        .into_element(cx);

        let card = shadcn::Card::new([header, content, footer])
            .ui()
            .w_full()
            .max_w(MetricRef::Px(Px(460.0)))
            .into_element(cx);

        let page = ui::container(cx, |cx| {
            [ui::v_flex(cx, |cx| {
                [
                    card,
                    ui::raw_text(cx, "MVU demo · typed messages via MessageRouter")
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

        vec![cx.semantics(
            SemanticsProps {
                role: SemanticsRole::Panel,
                label: Some(Arc::from("Debug:todo-mvu-demo:page")),
                ..Default::default()
            },
            move |_cx| [page],
        )]
    }
}

fn todo_list_panel(
    cx: &mut ElementContext<'_, App>,
    theme: &Theme,
    todos: &[(TodoItem, bool)],
    show_done: Option<bool>,
    msg: &mut MessageRouter<Msg>,
    max_h: MetricRef,
) -> AnyElement {
    let filtered: Vec<(&TodoItem, bool, CommandId)> = todos
        .iter()
        .filter_map(|t| {
            let (t, done) = t;
            if show_done.is_some_and(|v| v != *done) {
                return None;
            }
            let remove_cmd = msg.cmd(Msg::Remove(t.id));
            Some((t, *done, remove_cmd))
        })
        .collect();

    if filtered.is_empty() {
        let label = match show_done {
            None => "暂无任务，开始添加吧",
            Some(false) => "太棒了！所有任务已完成",
            Some(true) => "没有已完成的任务",
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

    let rows = ui::v_flex(cx, |cx| {
        filtered
            .iter()
            .map(|(t, done, remove_cmd)| {
                cx.keyed(t.id, |cx| todo_row(cx, theme, t, *done, remove_cmd.clone()))
            })
            .elements()
    })
    .w_full()
    .gap(Space::N3)
    .into_element(cx);

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
    item: &TodoItem,
    done: bool,
    remove_cmd: CommandId,
) -> AnyElement {
    let row_chrome = ChromeRefinement::default()
        .border_1()
        .border_color(ColorRef::Color(theme.color_required("border")))
        .rounded(Radius::Lg)
        .p(Space::N3);

    let checkbox = shadcn::Checkbox::new(item.done.clone()).into_element(cx);
    let checkbox = cx.semantics(
        SemanticsProps {
            role: SemanticsRole::Checkbox,
            test_id: Some(todo_item_test_id(item.id, "done")),
            ..Default::default()
        },
        move |_cx| [checkbox],
    );

    let fg = if done {
        theme.color_required("muted-foreground")
    } else {
        theme.color_required("foreground")
    };
    let text = cx.text_props(TextProps {
        layout: Default::default(),
        text: item.text.clone(),
        style: None,
        color: Some(fg),
        wrap: fret_core::TextWrap::None,
        overflow: fret_core::TextOverflow::Clip,
    });

    let remove_btn = shadcn::Button::new("")
        .size(shadcn::ButtonSize::IconSm)
        .variant(shadcn::ButtonVariant::Ghost)
        .on_click(remove_cmd)
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
            test_id: Some(todo_item_test_id(item.id, "remove")),
            ..Default::default()
        },
        move |_cx| [remove_btn],
    );

    let done_badge = done.then(|| {
        shadcn::Badge::new("Done")
            .variant(shadcn::BadgeVariant::Secondary)
            .into_element(cx)
    });
    let row = ui::h_flex(cx, move |_cx| {
        std::iter::once(checkbox)
            .chain(std::iter::once(text))
            .chain(std::iter::once(remove_btn))
            .chain(done_badge.into_iter())
    })
    .items_center()
    .into_element(cx);

    let row = cx.semantics(
        SemanticsProps {
            role: SemanticsRole::ListItem,
            test_id: Some(todo_item_test_id(item.id, "row")),
            ..Default::default()
        },
        move |_cx| [row],
    );

    ui::container(cx, move |_cx| [row])
        .style(row_chrome)
        .w_full()
        .into_element(cx)
}
