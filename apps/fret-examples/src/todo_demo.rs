use std::sync::Arc;

use fret_core::{Axis, Edges, KeyCode};
use fret_kit::prelude::*;
use fret_ui::ThemeConfig;
use fret_ui::action::PressablePointerDownResult;
use fret_ui::element::{
    CrossAlign, FlexProps, LayoutStyle, MainAlign, PressableProps, RovingFlexProps,
    RovingFocusProps, SemanticsDecoration,
};
use fret_ui_kit::declarative::action_hooks::ActionHooksExt as _;
use fret_ui_kit::primitives::roving_focus_group;

const CMD_ADD: &str = "todo.add";
const CMD_CLEAR_COMPLETED: &str = "todo.clear_completed";
const CMD_REMOVE_PREFIX: &str = "todo.remove.";
const CMD_TOGGLE_PREFIX: &str = "todo.toggle.";

const TEST_ID_INPUT: &str = "todo-input";
const TEST_ID_ADD: &str = "todo-add";
const TEST_ID_VISUAL_TOGGLE: &str = "todo-visual-toggle";
const TEST_ID_VISUAL_PANEL: &str = "todo-visual-panel";

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
                shadcn::shadcn_themes::ShadcnBaseColor::Zinc,
                shadcn::shadcn_themes::ShadcnColorScheme::Dark,
            );

            let cfg = ThemeConfig::from_slice(include_bytes!("todo_theme_overrides.json")).expect(
                "todo_demo theme overrides must be valid ThemeConfig JSON (see ThemeConfig)",
            );
            // IMPORTANT: this is a *token patch* (metrics-only today). Using `apply_config` would
            // reset the shadcn palette we just applied, which makes `primary-foreground` wrong
            // (e.g. checkmarks become invisible on a light `primary` background).
            Theme::with_global_mut(app, |theme| theme.extend_tokens_from_config(&cfg));
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
        .children([icon::icon_with(
            cx,
            IconId::new("lucide.plus"),
            Some(Px(16.0)),
            Some(ColorRef::Color(theme.color_required("primary-foreground"))),
        )])
        .into_element(cx)
        .attach_semantics(
            SemanticsDecoration::default()
                .role(SemanticsRole::Button)
                .test_id(TEST_ID_ADD),
        );

    let input = shadcn::Input::new(st.draft.clone())
        .a11y_label("Todo")
        .placeholder("添加新任务...")
        .submit_command(CommandId::new(CMD_ADD))
        .into_element(cx)
        .attach_semantics(
            SemanticsDecoration::default()
                .role(SemanticsRole::TextField)
                .test_id(TEST_ID_INPUT),
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

    let clear_completed_enabled = completed > 0;

    let header = shadcn::CardHeader::new([ui::h_flex(cx, |cx| {
        let left = ui::v_flex(cx, |cx| {
            [
                shadcn::CardTitle::new("待办事项").into_element(cx),
                shadcn::CardDescription::new("用键盘更高效地管理任务。").into_element(cx),
            ]
        })
        .gap(Space::N1)
        .items_start()
        .into_element(cx);

        let right = ui::h_flex(cx, |cx| {
            let clear_completed_btn = shadcn::Button::new("清除已完成")
                .size(shadcn::ButtonSize::Sm)
                .variant(shadcn::ButtonVariant::Secondary)
                .disabled(!clear_completed_enabled)
                .on_click(CMD_CLEAR_COMPLETED)
                .into_element(cx)
                .attach_semantics(
                    SemanticsDecoration::default()
                        .role(SemanticsRole::Button)
                        .test_id("todo-clear-completed"),
                );

            let progress = shadcn::Badge::new(format!("{completed}/{total}", total = todos.len()))
                .variant(shadcn::BadgeVariant::Secondary)
                .into_element(cx);

            [progress, clear_completed_btn]
        })
        .gap(Space::N2)
        .items_center()
        .into_element(cx);

        [left, right]
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
                    done_invalidation,
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
                    done_invalidation,
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
                    done_invalidation,
                )],
            ),
        ])
        .into_element(cx);

    let debug_panel = todo_visual_debug_panel(cx, &theme, &draft_value, add_enabled);

    let content = shadcn::CardContent::new([ui::v_flex(cx, |_cx| [input_row, tabs, debug_panel])
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

        let right = ui::v_flex(cx, |cx| {
            let row = ui::h_flex(cx, |_cx| completed_badge.into_iter())
                .gap(Space::N2)
                .items_center()
                .into_element(cx);

            let help = shadcn::KbdGroup::new([
                shadcn::Kbd::new("Enter").into_element(cx),
                ui::raw_text(cx, "添加")
                    .text_color(ColorRef::Color(theme.color_required("muted-foreground")))
                    .into_element(cx),
                shadcn::Kbd::new("↑/↓").into_element(cx),
                ui::raw_text(cx, "移动")
                    .text_color(ColorRef::Color(theme.color_required("muted-foreground")))
                    .into_element(cx),
                shadcn::Kbd::new("Space").into_element(cx),
                ui::raw_text(cx, "切换完成")
                    .text_color(ColorRef::Color(theme.color_required("muted-foreground")))
                    .into_element(cx),
                shadcn::Kbd::new("Del").into_element(cx),
                ui::raw_text(cx, "删除")
                    .text_color(ColorRef::Color(theme.color_required("muted-foreground")))
                    .into_element(cx),
            ])
            .into_element(cx);

            [row, help]
        })
        .gap(Space::N2)
        .items_end()
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
                ui::raw_text(cx, "Shadcn 风格 · Fret UiAppDriver demo · keyboard-first")
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
    .bg(ColorRef::Color(theme.color_required("background")))
    .p(Space::N4)
    .w_full()
    .h_full()
    .into_element(cx);

    ViewElements::from([page.attach_semantics(
        SemanticsDecoration::default()
            .role(SemanticsRole::Panel)
            .label("Debug:todo-demo:page"),
    )])
}

fn todo_list_panel(
    cx: &mut ElementContext<'_, App>,
    theme: &Theme,
    todos: &[TodoItem],
    filter: TodoFilter,
    max_h: MetricRef,
    done_invalidation: Invalidation,
) -> AnyElement {
    let filtered: Vec<(TodoItem, bool)> = todos
        .iter()
        .cloned()
        .filter_map(|t| {
            let done = cx
                .watch_model(&t.done)
                .invalidation(done_invalidation)
                .copied()
                .unwrap_or(false);
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

    let disabled: Arc<[bool]> = Arc::from(vec![false; filtered.len()].into_boxed_slice());
    let labels: Arc<[Arc<str>]> = Arc::from(
        filtered
            .iter()
            .map(|(t, _)| t.text.clone())
            .collect::<Vec<_>>()
            .into_boxed_slice(),
    );

    let rows = roving_focus_group::roving_focus_group_apg(
        cx,
        RovingFlexProps {
            flex: FlexProps {
                layout: LayoutStyle::default(),
                direction: Axis::Vertical,
                gap: MetricRef::space(Space::N2).resolve(theme),
                padding: Edges::all(Px(0.0)),
                justify: MainAlign::Start,
                align: CrossAlign::Start,
                wrap: false,
            },
            roving: RovingFocusProps {
                enabled: true,
                wrap: true,
                disabled,
            },
        },
        roving_focus_group::TypeaheadPolicy::Prefix {
            labels,
            timeout_ticks: 30,
        },
        move |cx| {
            filtered
                .iter()
                .map(|(t, done)| cx.keyed(t.id, |cx| todo_row(cx, theme, t, *done)))
                .collect::<Vec<_>>()
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
    let id = it.id;
    let text = it.text.clone();

    let radius = MetricRef::radius(Radius::Lg).resolve(theme);
    let focus_ring = shadcn::decl_style::focus_ring(theme, radius);

    let border = theme.color_required("border");
    let bg = theme.color_required("background");
    let accent = theme.color_required("accent");
    let bg_hover = {
        let mut c = accent;
        c.a = (c.a * 0.45).clamp(0.0, 1.0);
        c
    };
    let bg_pressed = {
        let mut c = accent;
        c.a = (c.a * 0.7).clamp(0.0, 1.0);
        c
    };

    let delete_zone_w = theme
        .metric_by_key("component.size.md.icon_button.size")
        .unwrap_or(Px(32.0));

    let row = fret_ui_kit::declarative::chrome::control_chrome_pressable_with_id_props(
        cx,
        move |cx, st, element_id| {
            cx.key_prepend_on_key_down_for(
                element_id,
                Arc::new(move |host, acx, down| {
                    if matches!(down.key, KeyCode::Delete | KeyCode::Backspace) {
                        host.dispatch_command(Some(acx.window), remove_cmd(id));
                        return true;
                    }
                    false
                }),
            );

            cx.pressable_dispatch_command_if_enabled(toggle_cmd(id));

            cx.pressable_on_pointer_down(Arc::new(move |host, acx, down| {
                let bounds = host.bounds();
                let x1 = bounds.origin.x.0 + bounds.size.width.0;
                if down.position.x.0 >= x1 - delete_zone_w.0 {
                    host.dispatch_command(Some(acx.window), remove_cmd(id));
                    return PressablePointerDownResult::SkipDefaultAndStopPropagation;
                }
                PressablePointerDownResult::Continue
            }));

            let mut a11y = fret_ui::element::PressableA11y::default();
            a11y.role = Some(SemanticsRole::Checkbox);
            a11y.label = Some(text.clone());
            a11y.checked = Some(done);
            a11y.test_id = Some(todo_item_test_id(id, "done"));

            let mut pressable_layout = LayoutStyle::default();
            pressable_layout.size.width = Length::Fill;
            pressable_layout.size.min_height = Some(
                theme
                    .metric_by_key("component.size.md.button.h")
                    .unwrap_or(Px(36.0)),
            );

            let pressable_props = PressableProps {
                layout: pressable_layout,
                enabled: true,
                focus_ring: Some(focus_ring),
                a11y,
                ..Default::default()
            };

            let row_bg = if st.pressed {
                bg_pressed
            } else if st.hovered || st.focused {
                bg_hover
            } else {
                bg
            };

            let chrome_props = shadcn::decl_style::container_props(
                theme,
                ChromeRefinement::default()
                    .rounded(Radius::Lg)
                    .border_1()
                    .border_color(ColorRef::Color(border))
                    .bg(ColorRef::Color(row_bg))
                    .px(Space::N3)
                    .py(Space::N2),
                LayoutRefinement::default(),
            );

            let children = move |cx: &mut ElementContext<'_, App>| {
                let fg = if done {
                    theme.color_required("muted-foreground")
                } else {
                    theme.color_required("foreground")
                };

                let checkbox_size = theme
                    .metric_by_key("component.checkbox.size")
                    .unwrap_or(Px(16.0));
                let checkbox_bg = if done {
                    theme.color_required("primary")
                } else {
                    theme.color_required("background")
                };
                let checkbox_border = theme
                    .color_by_key("input")
                    .or_else(|| theme.color_by_key("border"))
                    .unwrap_or(border);
                let checkbox_fg = theme.color_required("primary-foreground");
                let checkbox_icon_size = Px((checkbox_size.0 - 2.0).max(10.0));

                let indicator = ui::container(cx, |cx| {
                    let icon = done.then(|| {
                        icon::icon_with(
                            cx,
                            IconId::new("lucide.check"),
                            Some(checkbox_icon_size),
                            Some(ColorRef::Color(checkbox_fg)),
                        )
                    });
                    [ui::h_flex(cx, |_cx| icon.into_iter())
                        .w_full()
                        .h_full()
                        .justify_center()
                        .items_center()
                        .into_element(cx)]
                })
                .w_px(checkbox_size)
                .h_px(checkbox_size)
                .flex_shrink_0()
                .rounded(Radius::Sm)
                .border_1()
                .border_color(ColorRef::Color(checkbox_border))
                .bg(ColorRef::Color(checkbox_bg))
                .into_element(cx);

                let indicator = indicator.attach_semantics(
                    SemanticsDecoration::default()
                        .role(SemanticsRole::Panel)
                        .test_id(todo_item_test_id(id, "indicator")),
                );

                let label = ui::text(cx, text.clone())
                    .flex_1()
                    .min_w_0()
                    .nowrap()
                    .truncate()
                    .text_color(ColorRef::Color(fg))
                    .into_element(cx);

                let left = ui::h_flex(cx, |_cx| [indicator, label])
                    .flex_1()
                    .min_w_0()
                    .gap(Space::N3)
                    .items_center()
                    .into_element(cx);

                let remove_zone = {
                    let remove_icon = icon::icon_with(
                        cx,
                        IconId::new("lucide.trash"),
                        Some(Px(16.0)),
                        Some(ColorRef::Color(theme.color_required("muted-foreground"))),
                    );

                    ui::container(cx, |cx| {
                        [ui::h_flex(cx, |_cx| [remove_icon])
                            .w_full()
                            .h_full()
                            .justify_center()
                            .items_center()
                            .into_element(cx)]
                    })
                    .w_px(delete_zone_w)
                    .h_full()
                    .into_element(cx)
                    .attach_semantics(
                        SemanticsDecoration::default()
                            .role(SemanticsRole::Button)
                            .test_id(todo_item_test_id(id, "remove")),
                    )
                };

                [ui::h_flex(cx, |_cx| [left, remove_zone.into()])
                    .w_full()
                    .items_center()
                    .into_element(cx)]
            };

            (pressable_props, chrome_props, children)
        },
    );

    row
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
        CMD_CLEAR_COMPLETED => {
            let snapshot = app
                .models()
                .read(&state.todos, |v| v.clone())
                .ok()
                .unwrap_or_default();
            let keep: Vec<TodoItem> = snapshot
                .into_iter()
                .filter(|t| !app.models().read(&t.done, |v| *v).ok().unwrap_or(false))
                .collect();
            let _ = app.models_mut().update(&state.todos, |todos| *todos = keep);
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
                return;
            }
            if let Some(id) = other.strip_prefix(CMD_TOGGLE_PREFIX) {
                let Ok(id) = id.parse::<u64>() else {
                    return;
                };
                let done_model = app
                    .models()
                    .read(&state.todos, |todos| {
                        todos.iter().find(|t| t.id == id).map(|t| t.done.clone())
                    })
                    .ok()
                    .flatten();
                if let Some(done_model) = done_model {
                    let _ = app.models_mut().update(&done_model, |v| *v = !*v);
                    app.push_effect(Effect::Redraw(window));
                }
            }
        }
    }
}

fn remove_cmd(id: u64) -> CommandId {
    CommandId::new(format!("{CMD_REMOVE_PREFIX}{id}"))
}

fn toggle_cmd(id: u64) -> CommandId {
    CommandId::new(format!("{CMD_TOGGLE_PREFIX}{id}"))
}

fn todo_visual_debug_panel(
    cx: &mut ElementContext<'_, App>,
    theme: &Theme,
    draft_value: &str,
    add_enabled: bool,
) -> AnyElement {
    let border = theme.color_required("border");
    let muted_fg = theme.color_required("muted-foreground");

    let primary = theme.color_required("primary");
    let primary_fg = theme.color_required("primary-foreground");
    let foreground = theme.color_required("foreground");
    let background = theme.color_required("background");

    let token_line: Arc<str> = Arc::from(format!(
        "tokens(primary={primary:?} primary-foreground={primary_fg:?} foreground={foreground:?} background={background:?})",
    ));
    let draft_line: Arc<str> = Arc::from(format!(
        "draft(len={} enabled={}) value={:?}",
        draft_value.len(),
        add_enabled,
        draft_value
    ));

    shadcn::Collapsible::uncontrolled(false)
        .refine_layout(LayoutRefinement::default().w_full())
        .into_element_with_open_model(
            cx,
            move |cx, open, is_open| {
                let caret = if is_open {
                    IconId::new("lucide.chevron-up")
                } else {
                    IconId::new("lucide.chevron-down")
                };

                shadcn::Button::new("视觉自检")
                    .variant(shadcn::ButtonVariant::Ghost)
                    .size(shadcn::ButtonSize::Sm)
                    .toggle_model(open)
                    .test_id(TEST_ID_VISUAL_TOGGLE)
                    .children([icon::icon_with(
                        cx,
                        caret,
                        Some(Px(16.0)),
                        Some(ColorRef::Color(muted_fg)),
                    )])
                    .into_element(cx)
            },
            move |cx| {
                let token_dbg = ui::raw_text(cx, token_line.clone())
                    .text_color(ColorRef::Color(muted_fg))
                    .into_element(cx);
                let draft_dbg = ui::raw_text(cx, draft_line.clone())
                    .text_color(ColorRef::Color(muted_fg))
                    .into_element(cx);

                let swatch = |cx: &mut ElementContext<'_, App>,
                              bg: fret_core::Color,
                              fg: fret_core::Color,
                              label: &'static str| {
                    ui::container(cx, |cx| {
                        [ui::raw_text(cx, label)
                            .text_color(ColorRef::Color(fg))
                            .into_element(cx)]
                    })
                    .bg(ColorRef::Color(bg))
                    .border_1()
                    .border_color(ColorRef::Color(border))
                    .rounded(Radius::Md)
                    .px(Space::N2)
                    .py(Space::N1)
                    .into_element(cx)
                };

                let primary_swatch =
                    swatch(cx, primary, primary_fg, "primary on primary-foreground");
                let fg_swatch = swatch(cx, primary, foreground, "primary on foreground");

                let icon_sample = |cx: &mut ElementContext<'_, App>,
                                   icon_id: IconId,
                                   color: fret_core::Color,
                                   size: Px| {
                    ui::container(cx, |cx| {
                        let icon =
                            icon::icon_with(cx, icon_id, Some(size), Some(ColorRef::Color(color)));
                        [ui::h_flex(cx, |_cx| [icon])
                            .w_full()
                            .h_full()
                            .justify_center()
                            .items_center()
                            .into_element(cx)]
                    })
                    .w_px(Px(28.0))
                    .h_px(Px(28.0))
                    .border_1()
                    .border_color(ColorRef::Color(border))
                    .rounded(Radius::Md)
                    .into_element(cx)
                };

                let checkbox_probe =
                    |cx: &mut ElementContext<'_, App>, done: bool, label: &'static str| {
                        let checkbox_size = Px(16.0);
                        let checkbox_bg = if done { primary } else { background };
                        let checkbox_border = theme
                            .color_by_key("input")
                            .or_else(|| theme.color_by_key("border"))
                            .unwrap_or(border);
                        let checkbox_fg = primary_fg;

                        let indicator = ui::container(cx, |cx| {
                            let icon = done.then(|| {
                                icon::icon_with(
                                    cx,
                                    IconId::new("lucide.check"),
                                    Some(checkbox_size),
                                    Some(ColorRef::Color(checkbox_fg)),
                                )
                            });
                            [ui::h_flex(cx, |_cx| icon.into_iter())
                                .w_full()
                                .h_full()
                                .justify_center()
                                .items_center()
                                .into_element(cx)]
                        })
                        .w_px(checkbox_size)
                        .h_px(checkbox_size)
                        .flex_shrink_0()
                        .rounded(Radius::Sm)
                        .border_1()
                        .border_color(ColorRef::Color(checkbox_border))
                        .bg(ColorRef::Color(checkbox_bg))
                        .into_element(cx);

                        let label = ui::text(cx, label)
                            .nowrap()
                            .truncate()
                            .text_color(ColorRef::Color(foreground))
                            .into_element(cx);

                        ui::h_flex(cx, |_cx| [indicator, label])
                            .gap(Space::N2)
                            .items_center()
                            .into_element(cx)
                    };

                let content = ui::v_flex(cx, |cx| {
                    let row1 = ui::h_flex(cx, |_cx| [token_dbg, draft_dbg])
                        .gap(Space::N2)
                        .w_full()
                        .into_element(cx);

                    let row2 = ui::h_flex(cx, |_cx| [primary_swatch, fg_swatch])
                        .gap(Space::N2)
                        .w_full()
                        .items_center()
                        .into_element(cx);

                    let row3 = ui::h_flex(cx, |cx| {
                        [
                            icon_sample(cx, IconId::new("lucide.plus"), primary_fg, Px(16.0)),
                            icon_sample(cx, IconId::new("lucide.plus"), foreground, Px(16.0)),
                            icon_sample(cx, IconId::new("lucide.check"), primary_fg, Px(16.0)),
                            icon_sample(cx, IconId::new("lucide.check"), foreground, Px(16.0)),
                        ]
                    })
                    .gap(Space::N2)
                    .w_full()
                    .items_center()
                    .into_element(cx);

                    let row4 = ui::h_flex(cx, |cx| {
                        [
                            icon_sample(cx, IconId::new("lucide.plus"), foreground, Px(12.0)),
                            icon_sample(cx, IconId::new("lucide.plus"), foreground, Px(16.0)),
                            icon_sample(cx, IconId::new("lucide.plus"), foreground, Px(20.0)),
                        ]
                    })
                    .gap(Space::N2)
                    .w_full()
                    .items_center()
                    .into_element(cx);

                    let row5 = ui::h_flex(cx, |cx| {
                        [
                            checkbox_probe(cx, false, "align probe"),
                            checkbox_probe(cx, true, "align probe (checked)"),
                        ]
                    })
                    .gap(Space::N3)
                    .w_full()
                    .items_center()
                    .into_element(cx);

                    [row1, row2, row3, row4, row5]
                })
                .gap(Space::N2)
                .w_full()
                .into_element(cx);

                let panel = ui::container(cx, |_cx| [content])
                    .border_1()
                    .border_color(ColorRef::Color(border))
                    .rounded(Radius::Lg)
                    .p(Space::N3)
                    .w_full()
                    .into_element(cx);

                panel.attach_semantics(
                    SemanticsDecoration::default()
                        .role(SemanticsRole::Panel)
                        .test_id(TEST_ID_VISUAL_PANEL),
                )
            },
        )
}
