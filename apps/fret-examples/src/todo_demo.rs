use std::sync::Arc;

use fret_app::{App, CommandId, Effect};
use fret_bootstrap::ui_app_with_hooks;
use fret_core::{AppWindowId, UiServices};
use fret_core::{TextOverflow, TextWrap};
use fret_launch::WinitRunnerConfig;
use fret_ui::element::{HoverRegionProps, TextProps};
use fret_ui::{Invalidation, Theme};
use fret_ui_shadcn::{self as shadcn, prelude::*};

const CMD_ADD: &str = "todo.add";
const CMD_CLEAR_DONE: &str = "todo.clear_done";
const CMD_REMOVE_PREFIX: &str = "todo.remove.";

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
    let mut config = WinitRunnerConfig::default();
    config.main_window_title = "todo_demo".to_string();
    config.main_window_size = winit::dpi::LogicalSize::new(560.0, 520.0);

    ui_app_with_hooks("todo-demo", init_window, view, |d| d.on_command(on_command))
        .configure(move |c| *c = config)
        .with_default_settings_json()?
        .with_ui_assets_budgets(64 * 1024 * 1024, 2048, 16 * 1024 * 1024, 4096)
        .init_app(|app| {
            shadcn::shadcn_themes::apply_shadcn_new_york_v4(
                app,
                shadcn::shadcn_themes::ShadcnBaseColor::Slate,
                shadcn::shadcn_themes::ShadcnColorScheme::Light,
            );
        })
        .register_icon_pack(fret_icons_lucide::register_icons)
        .preload_icon_svgs_on_gpu_ready()
        .run()
        .map_err(anyhow::Error::from)
}

fn init_window(app: &mut App, _window: AppWindowId) -> TodoState {
    let done_1 = app.models_mut().insert(false);
    let done_2 = app.models_mut().insert(true);

    let todos = app.models_mut().insert(vec![
        TodoItem {
            id: 1,
            done: done_1,
            text: Arc::from("体验 shadcn 风格组件"),
        },
        TodoItem {
            id: 2,
            done: done_2,
            text: Arc::from("把 demo 跑通并对齐黄金路径"),
        },
    ]);
    let draft = app.models_mut().insert(String::new());
    let filter = app.models_mut().insert(Some(Arc::from("all")));
    TodoState {
        todos,
        draft,
        filter,
        next_id: 3,
    }
}

fn view(cx: &mut ElementContext<'_, App>, st: &mut TodoState) -> Vec<AnyElement> {
    cx.observe_model(&st.todos, Invalidation::Layout);
    cx.observe_model(&st.draft, Invalidation::Layout);
    cx.observe_model(&st.filter, Invalidation::Layout);

    let theme = Theme::global(&*cx.app).clone();

    let draft_value = cx
        .app
        .models()
        .read(&st.draft, |s| s.clone())
        .ok()
        .unwrap_or_default();

    let add_enabled = !draft_value.trim().is_empty();

    let add_btn = shadcn::Button::new("")
        .size(shadcn::ButtonSize::Icon)
        .variant(shadcn::ButtonVariant::Default)
        .disabled(!add_enabled)
        .on_click(CMD_ADD)
        .children(vec![icon::icon(cx, IconId::new("lucide.plus"))])
        .into_element(cx);

    let input = shadcn::Input::new(st.draft.clone())
        .a11y_label("Todo")
        .placeholder("添加新任务...")
        .submit_command(CommandId::new(CMD_ADD))
        .into_element(cx);

    let todos = cx
        .app
        .models()
        .read(&st.todos, |v| v.clone())
        .ok()
        .unwrap_or_default();

    for it in &todos {
        cx.observe_model(&it.done, Invalidation::Layout);
    }

    let completed = todos
        .iter()
        .filter(|t| cx.app.models().read(&t.done, |v| *v).ok().unwrap_or(false))
        .count();
    let active = todos.len().saturating_sub(completed);

    let header = shadcn::CardHeader::new(vec![stack::hstack(
        cx,
        stack::HStackProps::default()
            .layout(LayoutRefinement::default().w_full())
            .justify_between()
            .items_start(),
        |cx| {
            let left = stack::vstack(
                cx,
                stack::VStackProps::default().gap(Space::N1).items_start(),
                |cx| {
                    vec![
                        shadcn::CardTitle::new("待办事项").into_element(cx),
                        shadcn::CardDescription::new("管理你的日常任务。").into_element(cx),
                    ]
                },
            );

            let icon_bg = {
                let props = decl_style::container_props(
                    &theme,
                    ChromeRefinement::default()
                        .bg(ColorRef::Color(theme.color_required("muted")))
                        .rounded(Radius::Full),
                    LayoutRefinement::default()
                        .w_px(MetricRef::Px(fret_core::Px(32.0)))
                        .h_px(MetricRef::Px(fret_core::Px(32.0))),
                );
                cx.container(props, |cx| {
                    vec![stack::hstack(
                        cx,
                        stack::HStackProps::default()
                            .layout(LayoutRefinement::default().w_full().h_full())
                            .justify_center()
                            .items_center(),
                        |cx| {
                            vec![icon::icon_with(
                                cx,
                                IconId::new("lucide.calendar"),
                                Some(fret_core::Px(16.0)),
                                Some(ColorRef::Color(theme.color_required("muted-foreground"))),
                            )]
                        },
                    )]
                })
            };

            vec![left, icon_bg]
        },
    )])
    .into_element(cx);

    let input_row = stack::hstack(
        cx,
        stack::HStackProps::default()
            .layout(LayoutRefinement::default().w_full())
            .gap(Space::N2)
            .items_center(),
        |_cx| vec![input, add_btn],
    );

    let list_max_h = MetricRef::Px(fret_core::Px(260.0));

    let tabs = shadcn::Tabs::new(st.filter.clone())
        .refine_layout(LayoutRefinement::default().w_full())
        .list_full_width(true)
        .items([
            shadcn::TabsItem::new(
                "all",
                "全部",
                vec![todo_list_panel(
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
                vec![todo_list_panel(
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
                vec![todo_list_panel(
                    cx,
                    &theme,
                    &todos,
                    TodoFilter::Completed,
                    list_max_h.clone(),
                )],
            ),
        ])
        .into_element(cx);

    let content = shadcn::CardContent::new(vec![stack::vstack(
        cx,
        stack::VStackProps::default()
            .layout(LayoutRefinement::default().w_full())
            .gap(Space::N4),
        |_cx| vec![input_row, tabs],
    )])
    .into_element(cx);

    let footer = shadcn::CardFooter::new(vec![stack::hstack(
        cx,
        stack::HStackProps::default()
            .layout(LayoutRefinement::default().w_full())
            .justify_between()
            .items_center(),
        |cx| {
            let left = cx.text(format!("{active} 个进行中"));

            let mut right: Vec<AnyElement> = Vec::new();
            if completed > 0 {
                let clear_done = shadcn::Button::new("清除已完成")
                    .variant(shadcn::ButtonVariant::Ghost)
                    .size(shadcn::ButtonSize::Sm)
                    .on_click(CMD_CLEAR_DONE)
                    .into_element(cx);
                right.push(clear_done);

                right.push(
                    shadcn::Badge::new(format!("已完成 {completed}"))
                        .variant(shadcn::BadgeVariant::Secondary)
                        .into_element(cx),
                );
            }

            let right = stack::hstack(
                cx,
                stack::HStackProps::default().gap(Space::N2).items_center(),
                |_cx| right,
            );

            vec![left, right]
        },
    )])
    .into_element(cx);

    let card = shadcn::Card::new(vec![header, content, footer])
        .refine_layout(
            LayoutRefinement::default()
                .w_full()
                .max_w(MetricRef::Px(fret_core::Px(460.0))),
        )
        .into_element(cx);

    let page = {
        let props = decl_style::container_props(
            &theme,
            ChromeRefinement::default()
                .bg(ColorRef::Color(theme.color_required("muted")))
                .p(Space::N4),
            LayoutRefinement::default().w_full().h_full(),
        );
        cx.container(props, |cx| {
            vec![stack::vstack(
                cx,
                stack::VStackProps::default()
                    .layout(LayoutRefinement::default().w_full().h_full())
                    .justify_center()
                    .items_center()
                    .gap(Space::N6),
                |cx| {
                    vec![
                        card,
                        cx.text_props(TextProps {
                            layout: Default::default(),
                            text: Arc::from("Shadcn 风格 · Fret UiAppDriver demo"),
                            style: None,
                            color: Some(theme.color_required("muted-foreground")),
                            wrap: TextWrap::None,
                            overflow: TextOverflow::Clip,
                        }),
                    ]
                },
            )]
        })
    };

    vec![page]
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
            .p(Space::N6);
        let props =
            decl_style::container_props(theme, chrome, LayoutRefinement::default().w_full());
        return cx.container(props, |cx| {
            vec![stack::vstack(
                cx,
                stack::VStackProps::default()
                    .layout(LayoutRefinement::default().w_full())
                    .items_center(),
                |cx| vec![cx.text(label)],
            )]
        });
    }

    let rows = stack::vstack(
        cx,
        stack::VStackProps::default()
            .layout(LayoutRefinement::default().w_full())
            .gap(Space::N3),
        |cx| {
            filtered
                .iter()
                .map(|(t, done)| cx.keyed(t.id, |cx| todo_row(cx, theme, t, *done)))
                .collect()
        },
    );

    shadcn::ScrollArea::new(vec![rows])
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

    let remove_btn = shadcn::Button::new("")
        .size(shadcn::ButtonSize::Icon)
        .variant(shadcn::ButtonVariant::Ghost)
        .on_click(remove_cmd(it.id))
        .children(vec![icon::icon(cx, IconId::new("lucide.trash-2"))])
        .into_element(cx);

    cx.hover_region(HoverRegionProps::default(), move |cx, hovered| {
        let bg = hovered.then(|| theme.color_required("accent"));

        let mut chrome = row_chrome.clone();
        if let Some(bg) = bg {
            chrome = chrome.bg(ColorRef::Color(bg));
        }

        let props =
            decl_style::container_props(theme, chrome, LayoutRefinement::default().w_full());

        vec![cx.container(props, |cx| {
            vec![stack::hstack(
                cx,
                stack::HStackProps::default()
                    .layout(LayoutRefinement::default().w_full())
                    .justify_between()
                    .items_center(),
                |cx| {
                    let left = stack::hstack(
                        cx,
                        stack::HStackProps::default()
                            .layout(LayoutRefinement::default().flex_1().min_w_0())
                            .gap(Space::N3)
                            .items_center(),
                        |cx| {
                            vec![
                                checkbox.clone(),
                                todo_label_simple(cx, theme, &it.text, done),
                            ]
                        },
                    );

                    let right = cx.interactivity_gate(true, hovered, |cx| {
                        vec![cx.opacity(if hovered { 1.0 } else { 0.0 }, |_cx| {
                            vec![remove_btn.clone()]
                        })]
                    });

                    vec![left, right]
                },
            )]
        })]
    })
}

fn todo_label_simple(
    cx: &mut ElementContext<'_, App>,
    theme: &Theme,
    text: &Arc<str>,
    done: bool,
) -> AnyElement {
    let fg = if done {
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
    _ui: &mut fret_ui::UiTree<App>,
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
        CMD_CLEAR_DONE => {
            let snapshot = app
                .models()
                .read(&state.todos, |v| v.clone())
                .ok()
                .unwrap_or_default();

            let keep: Vec<TodoItem> = snapshot
                .into_iter()
                .filter(|t| !app.models().read(&t.done, |v| *v).ok().unwrap_or(false))
                .collect();

            let _ = app.models_mut().update(&state.todos, move |todos| {
                *todos = keep;
            });
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
