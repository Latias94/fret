use super::{IconPack, ScaffoldOptions};

fn join_workspace_path(workspace_prefix: &str, subpath: &str) -> String {
    if workspace_prefix == "." {
        subpath.to_string()
    } else {
        format!("{workspace_prefix}/{subpath}")
    }
}

pub(super) fn template_gitignore() -> &'static str {
    r#"/target
/.fret
**/*.rs.bk
.DS_Store
Thumbs.db
"#
}

pub(super) fn todo_template_cargo_toml(
    package_name: &str,
    opts: ScaffoldOptions,
    workspace_prefix: &str,
) -> String {
    let mut kit_features: Vec<&str> = vec!["desktop", "diagnostics", "shadcn"];
    if opts.command_palette {
        kit_features.push("command-palette");
    }
    if opts.ui_assets {
        kit_features.push("ui-assets");
    }
    match opts.icon_pack {
        IconPack::Lucide => {
            kit_features.push("icons-lucide");
            kit_features.push("preload-icon-svgs");
        }
        IconPack::Radix => {
            kit_features.push("icons-radix");
            kit_features.push("preload-icon-svgs");
        }
        IconPack::None => {}
    }

    let kit_features = kit_features
        .into_iter()
        .map(|f| format!("\"{f}\""))
        .collect::<Vec<_>>()
        .join(", ");

    let fret_path = join_workspace_path(workspace_prefix, "ecosystem/fret");
    let fret_query_path = join_workspace_path(workspace_prefix, "ecosystem/fret-query");
    let fret_selector_path = join_workspace_path(workspace_prefix, "ecosystem/fret-selector");

    format!(
        r#"[package]
name = "{package_name}"
version = "0.1.0"
edition = "2024"

[dependencies]
anyhow = "1"
fret = {{ path = "{fret_path}", default-features = false, features = [{kit_features}] }}
fret-query = {{ path = "{fret_query_path}", features = ["ui"] }}
fret-selector = {{ path = "{fret_selector_path}", features = ["ui"] }}
[workspace]
"#
    )
}

pub(super) fn simple_todo_template_cargo_toml(
    package_name: &str,
    opts: ScaffoldOptions,
    workspace_prefix: &str,
) -> String {
    let mut kit_features: Vec<&str> = vec!["desktop", "diagnostics", "shadcn"];
    if opts.command_palette {
        kit_features.push("command-palette");
    }
    if opts.ui_assets {
        kit_features.push("ui-assets");
    }
    match opts.icon_pack {
        IconPack::Lucide => {
            kit_features.push("icons-lucide");
            kit_features.push("preload-icon-svgs");
        }
        IconPack::Radix => {
            kit_features.push("icons-radix");
            kit_features.push("preload-icon-svgs");
        }
        IconPack::None => {}
    }

    let kit_features = kit_features
        .into_iter()
        .map(|f| format!("\"{f}\""))
        .collect::<Vec<_>>()
        .join(", ");

    let fret_path = join_workspace_path(workspace_prefix, "ecosystem/fret");

    format!(
        r#"[package]
name = "{package_name}"
version = "0.1.0"
edition = "2024"

[dependencies]
anyhow = "1"
fret = {{ path = "{fret_path}", default-features = false, features = [{kit_features}] }}

[workspace]
"#
    )
}

pub(super) fn empty_template_cargo_toml(package_name: &str) -> String {
    format!(
        r#"[package]
name = "{package_name}"
version = "0.1.0"
edition = "2024"

[dependencies]
anyhow = "1"

[workspace]
"#
    )
}

pub(super) fn hello_template_cargo_toml(
    package_name: &str,
    opts: ScaffoldOptions,
    workspace_prefix: &str,
) -> String {
    let mut kit_features: Vec<&str> = vec!["desktop", "diagnostics", "shadcn"];
    if opts.command_palette {
        kit_features.push("command-palette");
    }
    match opts.icon_pack {
        IconPack::Lucide => {
            kit_features.push("icons-lucide");
            kit_features.push("preload-icon-svgs");
        }
        IconPack::Radix => {
            kit_features.push("icons-radix");
            kit_features.push("preload-icon-svgs");
        }
        IconPack::None => {}
    }

    let kit_features = kit_features
        .into_iter()
        .map(|f| format!("\"{f}\""))
        .collect::<Vec<_>>()
        .join(", ");

    let fret_path = join_workspace_path(workspace_prefix, "ecosystem/fret");

    format!(
        r#"[package]
name = "{package_name}"
version = "0.1.0"
edition = "2024"

[dependencies]
anyhow = "1"
    fret = {{ path = "{fret_path}", default-features = false, features = [{kit_features}] }}

[workspace]
"#
    )
}

pub(super) fn todo_template_main_rs(_package_name: &str, opts: ScaffoldOptions) -> String {
    // Radix doesn't currently ship plus/trash icons in our curated set; keep the todo template
    // functional by falling back to text buttons when Lucide isn't selected.
    let has_action_icons = matches!(opts.icon_pack, IconPack::Lucide);

    let add_btn_def = if has_action_icons {
        r#"    let add_btn = shadcn::Button::new("")
        .size(shadcn::ButtonSize::Icon)
        .disabled(!add_enabled)
        .on_click(add_cmd.clone())
        .children([icon::icon(cx, IconId::new("lucide.plus"))])
        .into_element(cx);
"#
    } else {
        r#"    let add_btn = shadcn::Button::new("Add")
        .disabled(!add_enabled)
        .on_click(add_cmd.clone())
        .into_element(cx);
"#
    };

    let remove_btn_def = if has_action_icons {
        r#"    let remove_btn = shadcn::Button::new("")
        .size(shadcn::ButtonSize::Icon)
        .variant(shadcn::ButtonVariant::Ghost)
        .on_click(remove_cmd)
        .children([icon::icon(cx, IconId::new("lucide.trash"))])
        .into_element(cx);
"#
    } else {
        r#"    let remove_btn = shadcn::Button::new("Remove")
        .variant(shadcn::ButtonVariant::Ghost)
        .on_click(remove_cmd)
        .into_element(cx);
"#
    };

    const TEMPLATE: &str = r#"use std::sync::Arc;
use std::time::Duration;

use fret::prelude::*;
 use fret_query::ui::QueryElementContextExt as _;
 use fret_query::{QueryKey, QueryPolicy, QueryState, QueryStatus, with_query_client};
 use fret_selector::ui::SelectorElementContextExt as _;

#[derive(Clone)]
struct TodoItem {
    id: u64,
    done: Model<bool>,
    text: Arc<str>,
}

struct TodoState {
    todos: Model<Vec<TodoItem>>,
    draft: Model<String>,
    filter: Model<TodoFilter>,
    next_id: u64,
}

#[derive(Debug, Clone)]
enum Msg {
    Add,
    ClearDone,
    RefreshTip,
    SetFilter(TodoFilter),
    Remove(u64),
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
enum TodoFilter {
    All,
    Active,
    Completed,
}

impl TodoFilter {
    fn matches(self, done: bool) -> bool {
        match self {
            Self::All => true,
            Self::Active => !done,
            Self::Completed => done,
        }
    }

    fn as_label(self) -> &'static str {
        match self {
            Self::All => "All",
            Self::Active => "Active",
            Self::Completed => "Completed",
        }
    }
}

#[derive(Debug)]
struct TipData {
    text: Arc<str>,
}

fn tip_key() -> QueryKey<TipData> {
    QueryKey::new("todo.tip.v1", &0u8)
}

fn tip_policy() -> QueryPolicy {
    QueryPolicy {
        stale_time: Duration::from_secs(10),
        cache_time: Duration::from_secs(60),
        keep_previous_data_while_loading: true,
        ..Default::default()
    }
}

#[derive(Clone, PartialEq)]
struct TodoDerivedDeps {
    todos_rev: u64,
    done_revs: Vec<u64>,
    filter: TodoFilter,
}

#[derive(Clone)]
struct TodoDerived {
    rows: Arc<[TodoRowSnapshot]>,
    total: usize,
    active: usize,
    completed: usize,
}

#[derive(Clone)]
struct TodoRowSnapshot {
    id: u64,
    done: bool,
    done_model: Model<bool>,
    text: Arc<str>,
}

struct TodoProgram;

impl MvuProgram for TodoProgram {
    type State = TodoState;
    type Message = Msg;

    fn init(app: &mut App, window: AppWindowId) -> Self::State {
        init_window(app, window)
    }

    fn update(app: &mut App, state: &mut Self::State, message: Self::Message) {
        update(app, state, message);
    }

    fn view(
        cx: &mut ElementContext<'_, App>,
        state: &mut Self::State,
        msg: &mut MessageRouter<Self::Message>,
    ) -> Elements {
        view(cx, state, msg)
    }
}

fn install_app(_app: &mut App) {
    // Register app-owned globals, commands, services, etc.
}

fn main() -> anyhow::Result<()> {
    FretApp::new("todo")
        .window("todo", (560.0, 520.0))
        .install_app(install_app)
        .run_mvu::<TodoProgram>()
        .map_err(anyhow::Error::from)
}

fn init_window(app: &mut App, _window: AppWindowId) -> TodoState {
    let done_1 = app.models_mut().insert(false);
    let done_2 = app.models_mut().insert(true);
    let todos = app.models_mut().insert(vec![
        TodoItem {
            id: 1,
            done: done_1,
            text: Arc::from("Try the shadcn New York style"),
        },
        TodoItem {
            id: 2,
            done: done_2,
            text: Arc::from("Validate selector derived state"),
        },
    ]);

    TodoState {
        todos,
        draft: app.models_mut().insert(String::new()),
        filter: app.models_mut().insert(TodoFilter::All),
        next_id: 3,
    }
}

fn view(
    cx: &mut ElementContext<'_, App>,
    st: &mut TodoState,
    msg: &mut MessageRouter<Msg>,
) -> Elements {
    let draft_value = cx
        .watch_model(&st.draft)
        .layout()
        .cloned_or_default();
    let filter_value = cx
        .watch_model(&st.filter)
        .layout()
        .copied_or(TodoFilter::All);

    let theme = Theme::global(&*cx.app).snapshot();

    let derived = cx.use_selector(
        |cx| {
            let filter = cx
                .watch_model(&st.filter)
                .layout()
                .copied_or(TodoFilter::All);
            let (todos_rev, done) = cx
                .watch_model(&st.todos)
                .layout()
                .read(|host, todos| {
                    let todos_rev = st.todos.revision(host).unwrap_or(0);
                    let done = todos
                        .iter()
                        .map(|t| (t.done.id(), t.done.revision(host).unwrap_or(0)))
                        .collect::<Vec<_>>();
                    (todos_rev, done)
                })
                .ok()
                .unwrap_or((0, Vec::new()));

            for (id, _rev) in &done {
                cx.observe_model_id(*id, Invalidation::Layout);
            }

            TodoDerivedDeps {
                todos_rev,
                done_revs: done.into_iter().map(|(_, rev)| rev).collect(),
                filter,
            }
        },
        |cx| {
            cx.watch_model(&st.todos)
                .layout()
                .read(|host, todos| {
                    let filter = host.models().get_copied(&st.filter).unwrap_or(TodoFilter::All);
                    let mut rows = Vec::new();
                    let mut completed = 0usize;
                    for t in todos {
                        let done = host.models().get_copied(&t.done).unwrap_or(false);
                        if done {
                            completed += 1;
                        }
                        if filter.matches(done) {
                            rows.push(TodoRowSnapshot {
                                id: t.id,
                                done,
                                done_model: t.done.clone(),
                                text: t.text.clone(),
                            });
                        }
                    }
                    let total = todos.len();
                    TodoDerived {
                        rows: rows.into(),
                        total,
                        active: total.saturating_sub(completed),
                        completed,
                    }
                })
                .ok()
                .unwrap_or_else(|| TodoDerived {
                    rows: Arc::from([]),
                    total: 0,
                    active: 0,
                    completed: 0,
                })
        },
    );

    let tip_handle = cx.use_query(tip_key(), tip_policy(), move |_token| {
        #[cfg(not(target_arch = "wasm32"))]
        std::thread::sleep(Duration::from_millis(150));

        Ok(TipData {
            text: Arc::from(format!("Tip fetched at {:?}", std::time::SystemTime::now())),
        })
    });

    let tip_state = cx
        .watch_model(tip_handle.model())
        .layout()
        .cloned_or_else(QueryState::<TipData>::default);

    let (tip_text, tip_color_key): (Arc<str>, &'static str) = match tip_state.status {
        QueryStatus::Idle | QueryStatus::Loading => (Arc::from("Tip: loading…"), "muted-foreground"),
        QueryStatus::Error => {
            let err = tip_state
                .error
                .as_ref()
                .map(|e| e.to_string())
                .unwrap_or_else(|| String::from("unknown error"));
            (Arc::from(format!("Tip error: {err}")), "destructive")
        }
        QueryStatus::Success => {
            let text = tip_state
                .data
                .as_ref()
                .map(|d| d.text.clone())
                .unwrap_or_else(|| Arc::<str>::from("<no tip>"));
            (text, "muted-foreground")
        }
    };

    let add_enabled = !draft_value.trim().is_empty();
    let add_cmd = msg.cmd(Msg::Add);
    let clear_done_cmd = msg.cmd(Msg::ClearDone);
    let refresh_tip_cmd = msg.cmd(Msg::RefreshTip);
    let filter_all_cmd = msg.cmd(Msg::SetFilter(TodoFilter::All));
    let filter_active_cmd = msg.cmd(Msg::SetFilter(TodoFilter::Active));
    let filter_completed_cmd = msg.cmd(Msg::SetFilter(TodoFilter::Completed));

    let progress = shadcn::Badge::new(format!("{}/{} done", derived.completed, derived.total))
        .variant(shadcn::BadgeVariant::Secondary)
        .into_element(cx);

    let clear_done_btn = shadcn::Button::new("Clear completed")
        .variant(shadcn::ButtonVariant::Secondary)
        .disabled(derived.completed == 0)
        .on_click(clear_done_cmd)
        .into_element(cx);

    let refresh_tip_btn = shadcn::Button::new("Refresh tip")
        .variant(shadcn::ButtonVariant::Ghost)
        .on_click(refresh_tip_cmd)
        .into_element(cx);

    let header_actions = ui::h_flex(cx, |_cx| [progress, clear_done_btn, refresh_tip_btn])
        .gap(Space::N2)
        .items_center()
        .into_element(cx);

    let filter_all = filter_chip(cx, "All", filter_all_cmd, filter_value == TodoFilter::All);
    let filter_active =
        filter_chip(cx, "Active", filter_active_cmd, filter_value == TodoFilter::Active);
    let filter_completed = filter_chip(
        cx,
        "Completed",
        filter_completed_cmd,
        filter_value == TodoFilter::Completed,
    );

    let filter_row = ui::h_flex(cx, |_cx| [filter_all, filter_active, filter_completed])
        .gap(Space::N2)
        .items_center()
        .w_full()
        .into_element(cx);

    let tip_line = ui::raw_text(cx, tip_text)
        .text_color(ColorRef::Color(theme.color_token(tip_color_key)))
        .into_element(cx);

__ADD_BTN_DEF__

    let input = shadcn::Input::new(st.draft.clone())
        .placeholder("Add a task…")
        .submit_command(add_cmd.clone())
        .into_element(cx);

    let input_row = ui::h_flex(cx, |_cx| [input, add_btn])
        .gap(Space::N2)
        .items_center()
        .w_full()
        .into_element(cx);

    let rows = ui::v_flex_build(cx, |cx, out| {
        for row in derived.rows.iter() {
            let remove_cmd = msg.cmd(Msg::Remove(row.id));
            out.push(cx.keyed(row.id, |cx| todo_row(cx, theme, row, remove_cmd.clone())));
        }

        if derived.rows.is_empty() {
            out.push(
                ui::text(cx, format!("No {} tasks", filter_value.as_label().to_lowercase()))
                    .text_color(ColorRef::Color(theme.color_token("muted-foreground")))
                    .into_element(cx),
            );
        }
    })
        .gap(Space::N3)
        .w_full()
        .into_element(cx);

    let card = shadcn::Card::new([
        shadcn::CardHeader::new([
            shadcn::CardTitle::new("Todo").into_element(cx),
            shadcn::CardDescription::new("Best-practice baseline: model + selector + query.")
                .into_element(cx),
            header_actions,
            tip_line,
        ])
        .into_element(cx),
        shadcn::CardContent::new([
            ui::v_flex(cx, |_cx| [input_row, filter_row, rows])
                .gap(Space::N4)
                .w_full()
                .into_element(cx),
        ])
        .into_element(cx),
    ])
    .ui()
    .bg(ColorRef::Color(theme.color_token("background")))
    .rounded(Radius::Lg)
    .border_1()
    .border_color(ColorRef::Color(theme.color_token("border")))
    .w_full()
    .max_w(Px(520.0))
    .into_element(cx);

    let page = ui::container(cx, |cx| {
        [ui::v_flex(cx, |_cx| [card])
            .w_full()
            .h_full()
            .justify_center()
            .items_center()
            .into_element(cx)]
    })
    .bg(ColorRef::Color(theme.color_token("muted")))
    .p(Space::N6)
    .w_full()
    .h_full()
    .into_element(cx);

    page.into()
}

fn todo_row(
    cx: &mut ElementContext<'_, App>,
    theme: ThemeSnapshot,
    row: &TodoRowSnapshot,
    remove_cmd: CommandId,
) -> AnyElement {
    let done = row.done;
    let checkbox = shadcn::Checkbox::new(row.done_model.clone()).into_element(cx);
 __REMOVE_BTN_DEF__

    let label = cx.text_props(TextProps {
        layout: Default::default(),
        text: row.text.clone(),
        style: None,
        color: Some(theme.color_token(if done {
            "muted-foreground"
        } else {
            "foreground"
        })),
        wrap: TextWrap::None,
        overflow: TextOverflow::Ellipsis,
    });

    let left = ui::h_flex(cx, |_cx| [checkbox.clone(), label])
        .gap(Space::N3)
        .items_center()
        .flex_1()
        .min_w_0()
        .into_element(cx);

    let row = ui::h_flex(cx, |_cx| [left, remove_btn.clone()])
        .w_full()
        .justify_between()
        .items_center()
        .into_element(cx);

    ui::container(cx, |_cx| [row])
        .border_1()
        .border_color(ColorRef::Color(theme.color_token("border")))
        .rounded(Radius::Md)
        .p(Space::N3)
        .w_full()
        .into_element(cx)
}

fn filter_chip(
    cx: &mut ElementContext<'_, App>,
    label: &'static str,
    cmd: CommandId,
    selected: bool,
) -> AnyElement {
    shadcn::Button::new(label)
        .variant(if selected {
            shadcn::ButtonVariant::Secondary
        } else {
            shadcn::ButtonVariant::Ghost
        })
        .size(shadcn::ButtonSize::Sm)
        .on_click(cmd)
        .into_element(cx)
}

fn update(app: &mut App, state: &mut TodoState, msg: Msg) {
    match msg {
        Msg::Add => {
            let draft = app
                .models()
                .read(&state.draft, |s| s.clone())
                .ok()
                .unwrap_or_default();
            let text = draft.trim();
            if text.is_empty() {
                return;
            }

            let id = state.next_id;
            state.next_id += 1;
            let done = app.models_mut().insert(false);
            let item = TodoItem {
                id,
                done,
                text: Arc::from(text),
            };

            let _ = app.models_mut().update(&state.todos, |todos| {
                todos.insert(0, item);
            });
            let _ = app.models_mut().update(&state.draft, |s| {
                s.clear();
            });
        }
        Msg::ClearDone => {
            let snapshot = app
                .models()
                .read(&state.todos, |v| v.clone())
                .ok()
                .unwrap_or_default();

            let mut keep: Vec<TodoItem> = Vec::new();
            for t in snapshot {
                let done = app.models().read(&t.done, |v| *v).ok().unwrap_or(false);
                if !done {
                    keep.push(t);
                }
            }

            let _ = app.models_mut().update(&state.todos, |todos| {
                *todos = keep;
            });
        }
        Msg::RefreshTip => {
            let _ = with_query_client(app, |client, app| {
                client.invalidate(app, tip_key());
            });
        }
        Msg::SetFilter(filter) => {
            let _ = app.models_mut().update(&state.filter, |value| {
                *value = filter;
            });
        }
        Msg::Remove(id) => {
            let _ = app.models_mut().update(&state.todos, |todos| {
                todos.retain(|t| t.id != id);
            });
        }
    }
}
"#;

    TEMPLATE
        .replace("__ADD_BTN_DEF__", add_btn_def)
        .replace("__REMOVE_BTN_DEF__", remove_btn_def)
}

pub(super) fn hello_template_main_rs(package_name: &str, opts: ScaffoldOptions) -> String {
    let palette_button = if opts.command_palette {
        r#"
                shadcn::Button::new("Command palette")
                    .on_click("app.command_palette")
                    .into_element(cx),"#
    } else {
        ""
    };

    format!(
        r#"use fret::prelude::*;

#[derive(Debug, Clone)]
enum Msg {{
    Click,
}}

struct HelloProgram;

impl MvuProgram for HelloProgram {{
    type State = ();
    type Message = Msg;

    fn init(_app: &mut App, _window: AppWindowId) -> Self::State {{}}

    fn update(_app: &mut App, _state: &mut Self::State, message: Self::Message) {{
        match message {{
            Msg::Click => {{
                println!("Clicked!");
            }}
        }}
    }}

    fn view(
        cx: &mut ElementContext<'_, App>,
        _state: &mut Self::State,
        msg: &mut MessageRouter<Self::Message>,
    ) -> Elements {{
        let click_cmd = msg.cmd(Msg::Click);

        ui::v_flex(cx, |cx| {{
            [
                shadcn::Label::new("Hello, world!").into_element(cx),
                shadcn::Button::new("Click me")
                    .on_click(click_cmd)
                    .into_element(cx),
__PALETTE_BUTTON__
            ]
        }})
        .size_full()
        .gap(Space::N4)
        .items_center()
        .justify_center()
        .into_element(cx)
        .into()
    }}
}}

fn main() -> anyhow::Result<()> {{
    FretApp::new("{package_name}")
        .window("{package_name}", (560.0, 360.0))
        .run_mvu::<HelloProgram>()
        .map_err(anyhow::Error::from)
}}
"#
    )
    .replace("__PALETTE_BUTTON__", palette_button)
}

pub(super) fn simple_todo_template_main_rs(_package_name: &str, opts: ScaffoldOptions) -> String {
    // Radix doesn't currently ship plus/trash icons in our curated set; keep the template
    // functional by falling back to text buttons when Lucide isn't selected.
    let has_action_icons = matches!(opts.icon_pack, IconPack::Lucide);

    let add_btn_def = if has_action_icons {
        r#"    let add_btn = shadcn::Button::new("")
        .size(shadcn::ButtonSize::Icon)
        .disabled(!add_enabled)
        .on_click(add_cmd.clone())
        .children(ui::children![cx; icon::icon(cx, IconId::new("lucide.plus"))])
        .ui()
        .rounded_md();
"#
    } else {
        r#"    let add_btn = shadcn::Button::new("Add")
        .disabled(!add_enabled)
        .on_click(add_cmd.clone())
        .ui()
        .rounded_md();
"#
    };

    let remove_btn_def = if has_action_icons {
        r#"    let remove_btn = shadcn::Button::new("")
        .size(shadcn::ButtonSize::IconSm)
        .variant(shadcn::ButtonVariant::Ghost)
        .on_click(remove_cmd)
        .children(ui::children![cx; icon::icon(cx, IconId::new("lucide.trash"))])
        .ui()
        .rounded_md();
"#
    } else {
        r#"    let remove_btn = shadcn::Button::new("Remove")
        .variant(shadcn::ButtonVariant::Ghost)
        .on_click(remove_cmd)
        .ui()
        .rounded_md();
"#
    };

    const TEMPLATE: &str = r#"use std::sync::Arc;

use fret::prelude::*;

#[derive(Clone)]
struct TodoItem {
    id: u64,
    done: Model<bool>,
    text: Arc<str>,
}

struct TodoState {
    todos: Model<Vec<TodoItem>>,
    draft: Model<String>,
    next_id: u64,
}

#[derive(Debug, Clone)]
enum Msg {
    Add,
    ClearDone,
    Remove(u64),
}

struct TodoProgram;

impl MvuProgram for TodoProgram {
    type State = TodoState;
    type Message = Msg;

    fn init(app: &mut App, window: AppWindowId) -> Self::State {
        init_window(app, window)
    }

    fn update(app: &mut App, state: &mut Self::State, message: Self::Message) {
        update(app, state, message);
    }

    fn view(
        cx: &mut ElementContext<'_, App>,
        state: &mut Self::State,
        msg: &mut MessageRouter<Self::Message>,
    ) -> Elements {
        view(cx, state, msg)
    }
}

fn main() -> anyhow::Result<()> {
    FretApp::new("simple-todo")
        .window("simple-todo", (560.0, 520.0))
        .run_mvu::<TodoProgram>()
        .map_err(anyhow::Error::from)
}

fn init_window(app: &mut App, _window: AppWindowId) -> TodoState {
    let done_1 = app.models_mut().insert(false);
    let done_2 = app.models_mut().insert(true);
    let todos = app.models_mut().insert(vec![
        TodoItem {
            id: 1,
            done: done_1,
            text: Arc::from("Use keyed rows for dynamic lists"),
        },
        TodoItem {
            id: 2,
            done: done_2,
            text: Arc::from("Try the shadcn theme tokens"),
        },
    ]);
    let draft = app.models_mut().insert(String::new());

    TodoState {
        todos,
        draft,
        next_id: 3,
    }
}

fn update(app: &mut App, st: &mut TodoState, msg: Msg) {
    match msg {
        Msg::Add => {
            let text = app
                .models()
                .read(&st.draft, |v| v.trim().to_string())
                .ok()
                .unwrap_or_default();
            if text.is_empty() {
                return;
            }

            let done = app.models_mut().insert(false);
            let id = st.next_id;
            st.next_id = st.next_id.saturating_add(1);

            let _ = app.models_mut().update(&st.todos, |todos| {
                todos.push(TodoItem {
                    id,
                    done,
                    text: Arc::from(text),
                });
            });
            let _ = app.models_mut().update(&st.draft, |v| v.clear());
        }
        Msg::ClearDone => {
            let mut remove_ids = Vec::new();
            let todos = app
                .models()
                .read(&st.todos, |v| v.clone())
                .ok()
                .unwrap_or_default();
            for t in &todos {
                let done = app.models().read(&t.done, |v| *v).ok().unwrap_or(false);
                if done {
                    remove_ids.push(t.id);
                }
            }
            if remove_ids.is_empty() {
                return;
            }
            let _ = app.models_mut().update(&st.todos, |todos| {
                todos.retain(|t| !remove_ids.contains(&t.id));
            });
        }
        Msg::Remove(id) => {
            let _ = app.models_mut().update(&st.todos, |todos| {
                todos.retain(|t| t.id != id);
            });
        }
    }
}

fn view(cx: &mut ElementContext<'_, App>, st: &mut TodoState, msg: &mut MessageRouter<Msg>) -> Elements {
    let theme = Theme::global(&*cx.app).snapshot();

    let todos = cx.watch_model(&st.todos).layout().cloned_or_default();
    let draft_value = cx.watch_model(&st.draft).paint().cloned_or_default();

    let mut done_count = 0usize;
    for t in &todos {
        if cx.watch_model(&t.done).paint().copied_or_default() {
            done_count += 1;
        }
    }
    let total_count = todos.len();

    let add_enabled = !draft_value.trim().is_empty();
    let add_cmd = msg.cmd(Msg::Add);
    let clear_done_cmd = msg.cmd(Msg::ClearDone);

    let progress = shadcn::Badge::new(format!("{done_count}/{total_count} done"))
        .variant(shadcn::BadgeVariant::Secondary);

    let clear_done_btn = shadcn::Button::new("Clear done")
        .variant(shadcn::ButtonVariant::Secondary)
        .disabled(done_count == 0)
        .on_click(clear_done_cmd)
        .ui()
        .rounded_md();

    let header_actions = ui::h_flex(cx, |cx| ui::children![cx; progress, clear_done_btn])
        .gap(Space::N2)
        .items_center();

__ADD_BTN_DEF__

    let input = shadcn::Input::new(st.draft.clone())
        .placeholder("Add a task…")
        .submit_command(add_cmd.clone());

    let input_row = ui::h_flex(cx, |cx| ui::children![cx; input, add_btn])
        .gap(Space::N2)
        .items_center()
        .w_full();

    let rows = ui::v_flex_build(cx, |cx, out| {
        for t in &todos {
            let remove_cmd = msg.cmd(Msg::Remove(t.id));
            out.push(cx.keyed(t.id, |cx| todo_row(cx, theme, t, remove_cmd.clone())));
        }
    })
    .gap(Space::N3)
    .w_full();

    let content = ui::v_flex(cx, |cx| ui::children![cx; input_row, rows])
        .gap(Space::N4)
        .w_full();

    let card = shadcn::Card::new(ui::children![cx;
        shadcn::CardHeader::new(ui::children![cx;
            shadcn::CardTitle::new("Simple Todo"),
            shadcn::CardDescription::new("Model + MVU messages + keyed lists (no selector/query)."),
            header_actions,
        ]),
        shadcn::CardContent::new(ui::children![cx; content]),
    ])
    .ui()
    .bg(ColorRef::Color(theme.color_token("background")))
    .rounded(Radius::Lg)
    .border_1()
    .border_color(ColorRef::Color(theme.color_token("border")))
    .w_full()
    .max_w(Px(520.0))
    ;

    let page = ui::container(cx, |cx| {
        ui::children![cx;
            ui::v_flex(cx, |cx| ui::children![cx; card])
                .w_full()
                .h_full()
                .justify_center()
                .items_center()
                ,
        ]
    })
    .bg(ColorRef::Color(theme.color_token("muted")))
    .p(Space::N6)
    .w_full()
    .h_full()
    ;

    page.into_element(cx).into()
}

fn todo_row(
    cx: &mut ElementContext<'_, App>,
    theme: ThemeSnapshot,
    item: &TodoItem,
    remove_cmd: CommandId,
) -> AnyElement {
    let done = cx.watch_model(&item.done).paint().copied_or_default();

    let checkbox = shadcn::Checkbox::new(item.done.clone());

    let text = ui::raw_text(cx, item.text.clone())
        .truncate()
        .text_sm()
        .text_color(ColorRef::Color(if done {
            theme.color_token("muted-foreground")
        } else {
            theme.color_token("foreground")
        }));

__REMOVE_BTN_DEF__

    ui::h_flex(cx, |cx| ui::children![cx; checkbox, text, remove_btn])
        .gap(Space::N2)
        .items_center()
        .w_full()
        .into_element(cx)
}
"#;

    TEMPLATE
        .replace("__ADD_BTN_DEF__", add_btn_def)
        .replace("__REMOVE_BTN_DEF__", remove_btn_def)
}

pub(super) fn empty_template_main_rs() -> &'static str {
    r#"fn main() -> anyhow::Result<()> {
    println!("Hello from Fret!");
    Ok(())
}
"#
}

pub(super) fn todo_template_readme_md(package_name: &str, opts: ScaffoldOptions) -> String {
    let ui_assets_line = if opts.ui_assets {
        "- UI assets: enabled (`fret/ui-assets`)\n"
    } else {
        "- UI assets: disabled (use `fretboard new todo --ui-assets` if you need images/SVG caches)\n"
    };

    let icons_line = match opts.icon_pack {
        IconPack::Lucide => "- Icons: Lucide (`fret/icons-lucide`)\n",
        IconPack::Radix => "- Icons: Radix (`fret/icons-radix`)\n",
        IconPack::None => "- Icons: disabled\n",
    };

    let palette_line = if opts.command_palette {
        "- Command palette: enabled (Cmd/Ctrl+Shift+P)\n"
    } else {
        "- Command palette: disabled\n"
    };

    format!(
        r#"# {package_name}

Generated by `fretboard new todo`.

## Run

```bash
cargo run
```

## Common commands

```bash
cargo fmt
cargo clippy -- -D warnings
cargo run --release
```

## Hot reload (runner reload boundary)

This template supports a lightweight reload boundary when `FRET_HOTPATCH=1`.

Run with hotpatch enabled:

```bash
FRET_HOTPATCH=1 cargo run
```

PowerShell:

```powershell
$env:FRET_HOTPATCH = "1"
cargo run
```

Trigger a reload by poking the marker file (default: `.fret/hotpatch.touch`):

```bash
mkdir -p .fret && date +%s%N > .fret/hotpatch.touch
```

PowerShell:

```powershell
New-Item -ItemType Directory -Force .fret | Out-Null
Set-Content -Path .fret/hotpatch.touch -Value (Get-Date).Ticks
```

## Notes

- Theme: shadcn new-york-v4 (Slate / Light)
{icons_line}{palette_line}
{ui_assets_line}
## Next steps

- Edit UI in `src/main.rs`
- If you want hotpatch later, keep commands/IDs stable and prefer the `FretApp::new(...).run_mvu::<Program>()` golden path (ADR 0105 / 0110).
"#
    )
}

pub(super) fn empty_template_readme_md(package_name: &str) -> String {
    format!(
        r#"# {package_name}

Generated by `fretboard new`.

## Run

```bash
cargo run
```

## Common commands

```bash
cargo fmt
cargo clippy -- -D warnings
cargo run --release
```
"#
    )
}

pub(super) fn simple_todo_template_readme_md(package_name: &str, opts: ScaffoldOptions) -> String {
    let ui_assets_line = if opts.ui_assets {
        "- UI assets: enabled (`fret/ui-assets`)\n"
    } else {
        "- UI assets: disabled (use `fretboard new simple-todo --ui-assets` if you need images/SVG caches)\n"
    };

    let icons_line = match opts.icon_pack {
        IconPack::Lucide => "- Icons: Lucide (`fret/icons-lucide`)\n",
        IconPack::Radix => "- Icons: Radix (`fret/icons-radix`)\n",
        IconPack::None => "- Icons: disabled\n",
    };

    let palette_line = if opts.command_palette {
        "- Command palette: enabled (Cmd/Ctrl+Shift+P)\n"
    } else {
        "- Command palette: disabled\n"
    };

    format!(
        r#"# {package_name}

Generated by `fretboard new simple-todo`.

## Run

```bash
cargo run
```

## Common commands

```bash
cargo fmt
cargo clippy -- -D warnings
cargo run --release
```

## Notes

- Theme: shadcn new-york-v4 (Slate / Light)
{icons_line}{palette_line}
{ui_assets_line}
## Next steps

- Edit UI in `src/main.rs`
- Use `ui::children![cx; ...]` to build heterogeneous child lists without call-site `.into_element(cx)` noise.
- When rendering dynamic lists, prefer `cx.keyed(id, |cx| ...)` to keep identity stable.
"#
    )
}

pub(super) fn hello_template_readme_md(package_name: &str, opts: ScaffoldOptions) -> String {
    let icons_line = match opts.icon_pack {
        IconPack::Lucide => "- Icons: Lucide (`fret/icons-lucide`)\n",
        IconPack::Radix => "- Icons: Radix (`fret/icons-radix`)\n",
        IconPack::None => "- Icons: disabled\n",
    };

    let palette_line = if opts.command_palette {
        "- Command palette: enabled (Cmd/Ctrl+Shift+P)\n"
    } else {
        "- Command palette: disabled\n"
    };

    format!(
        r#"# {package_name}

Generated by `fretboard new hello`.

## Run

```bash
cargo run
```

## Common commands

```bash
cargo fmt
cargo clippy -- -D warnings
cargo run --release
```

## Hot reload (runner reload boundary)

This template supports a lightweight reload boundary when `FRET_HOTPATCH=1`.

Run with hotpatch enabled:

```bash
FRET_HOTPATCH=1 cargo run
```

PowerShell:

```powershell
$env:FRET_HOTPATCH = "1"
cargo run
```

Trigger a reload by poking the marker file (default: `.fret/hotpatch.touch`):

```bash
mkdir -p .fret && date +%s%N > .fret/hotpatch.touch
```

PowerShell:

```powershell
New-Item -ItemType Directory -Force .fret | Out-Null
Set-Content -Path .fret/hotpatch.touch -Value (Get-Date).Ticks
```

## Notes

- Theme: shadcn new-york-v4 (default via `fret-ui-shadcn/app-integration`)
{icons_line}{palette_line}
- Next: edit `src/main.rs` and replace the view tree
"#
    )
}

#[cfg(test)]
mod tests {
    use super::*;

    fn opts() -> ScaffoldOptions {
        ScaffoldOptions {
            icon_pack: IconPack::Lucide,
            command_palette: true,
            ui_assets: false,
        }
    }

    #[test]
    fn todo_template_uses_default_authoring_dialect() {
        let src = todo_template_main_rs("todo-app", opts());
        assert!(src.contains("ui::container("));
        assert!(src.contains("ui::h_flex("));
        assert!(src.contains(".ui()"));
        assert!(!src.contains("decl_style::container_props"));
    }

    #[test]
    fn hello_template_uses_default_authoring_dialect() {
        let src = hello_template_main_rs("hello-app", opts());
        assert!(src.contains("ui::v_flex("));
        assert!(src.contains(".into_element(cx)"));
        assert!(!src.contains("decl_style::container_props"));
    }

    #[test]
    fn simple_todo_template_has_low_adapter_noise_and_no_query_selector() {
        let src = simple_todo_template_main_rs("simple-todo-app", opts());
        assert!(src.contains("ui::children!["));
        assert!(src.contains("cx.keyed("));
        assert!(!src.contains("fret_query"));
        assert!(!src.contains("fret_selector"));

        let into_element_count = src.matches(".into_element(cx)").count();
        assert!(
            into_element_count <= 10,
            "expected <= 10 explicit `.into_element(cx)` calls, got {into_element_count}"
        );
    }

    #[test]
    fn simple_todo_template_cargo_toml_has_no_query_selector_deps() {
        let toml = simple_todo_template_cargo_toml("simple-todo-app", opts(), ".");
        assert!(!toml.contains("fret-query"));
        assert!(!toml.contains("fret-selector"));
    }
}
