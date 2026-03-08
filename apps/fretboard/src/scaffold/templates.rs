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
    let mut kit_features: Vec<&str> = vec!["desktop", "shadcn", "state"];
    if opts.command_palette {
        kit_features.push("command-palette");
    }
    if opts.ui_assets {
        kit_features.push("ui-assets");
    }
    match opts.icon_pack {
        IconPack::Lucide => {
            kit_features.push("icons");
            kit_features.push("preload-icon-svgs");
        }
        IconPack::Radix => {
            // Radix icons are installed via an explicit dependency + install hook (no `fret` feature).
        }
        IconPack::None => {}
    }

    let kit_features = kit_features
        .into_iter()
        .map(|f| format!("\"{f}\""))
        .collect::<Vec<_>>()
        .join(", ");

    let fret_path = join_workspace_path(workspace_prefix, "ecosystem/fret");
    let fret_icons_radix_path = join_workspace_path(workspace_prefix, "ecosystem/fret-icons-radix");
    let fret_query_path = join_workspace_path(workspace_prefix, "ecosystem/fret-query");
    let fret_selector_path = join_workspace_path(workspace_prefix, "ecosystem/fret-selector");

    let radix_dep = if matches!(opts.icon_pack, IconPack::Radix) {
        format!(
            "fret-icons-radix = {{ path = \"{fret_icons_radix_path}\", features = [\"app-integration\"] }}\n"
        )
    } else {
        String::new()
    };

    format!(
        r#"[package]
name = "{package_name}"
version = "0.1.0"
edition = "2024"

[dependencies]
anyhow = "1"
fret = {{ path = "{fret_path}", default-features = false, features = [{kit_features}] }}
{radix_dep}fret-query = {{ path = "{fret_query_path}", features = ["ui"] }}
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
    let mut kit_features: Vec<&str> = vec!["desktop", "shadcn"];
    if opts.command_palette {
        kit_features.push("command-palette");
    }
    if opts.ui_assets {
        kit_features.push("ui-assets");
    }
    match opts.icon_pack {
        IconPack::Lucide => {
            kit_features.push("icons");
            kit_features.push("preload-icon-svgs");
        }
        IconPack::Radix => {
            // Radix icons are installed via an explicit dependency + install hook (no `fret` feature).
        }
        IconPack::None => {}
    }

    let kit_features = kit_features
        .into_iter()
        .map(|f| format!("\"{f}\""))
        .collect::<Vec<_>>()
        .join(", ");

    let fret_path = join_workspace_path(workspace_prefix, "ecosystem/fret");
    let fret_icons_radix_path = join_workspace_path(workspace_prefix, "ecosystem/fret-icons-radix");

    let radix_dep = if matches!(opts.icon_pack, IconPack::Radix) {
        format!(
            "fret-icons-radix = {{ path = \"{fret_icons_radix_path}\", features = [\"app-integration\"] }}\n"
        )
    } else {
        String::new()
    };

    format!(
        r#"[package]
name = "{package_name}"
version = "0.1.0"
edition = "2024"

[dependencies]
anyhow = "1"
fret = {{ path = "{fret_path}", default-features = false, features = [{kit_features}] }}
{radix_dep}

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
    let mut kit_features: Vec<&str> = vec!["desktop", "shadcn"];
    if opts.command_palette {
        kit_features.push("command-palette");
    }
    match opts.icon_pack {
        IconPack::Lucide => {
            kit_features.push("icons");
            kit_features.push("preload-icon-svgs");
        }
        IconPack::Radix => {
            // Radix icons are installed via an explicit dependency + install hook (no `fret` feature).
        }
        IconPack::None => {}
    }

    let kit_features = kit_features
        .into_iter()
        .map(|f| format!("\"{f}\""))
        .collect::<Vec<_>>()
        .join(", ");

    let fret_path = join_workspace_path(workspace_prefix, "ecosystem/fret");
    let fret_icons_radix_path = join_workspace_path(workspace_prefix, "ecosystem/fret-icons-radix");

    let radix_dep = if matches!(opts.icon_pack, IconPack::Radix) {
        format!(
            "fret-icons-radix = {{ path = \"{fret_icons_radix_path}\", features = [\"app-integration\"] }}\n"
        )
    } else {
        String::new()
    };

    format!(
        r#"[package]
name = "{package_name}"
version = "0.1.0"
edition = "2024"

[dependencies]
anyhow = "1"
    fret = {{ path = "{fret_path}", default-features = false, features = [{kit_features}] }}
{radix_dep}

[workspace]
"#
    )
}

pub(super) fn todo_template_main_rs(package_name: &str, opts: ScaffoldOptions) -> String {
    // Radix doesn't currently ship the Lucide plus icon in our curated set; keep the template
    // functional by falling back to a text button when Lucide isn't selected.
    let has_action_icons = matches!(opts.icon_pack, IconPack::Lucide);

    let add_btn_def = if has_action_icons {
        r#"    let add_btn = shadcn::Button::new("")
        .size(shadcn::ButtonSize::Icon)
        .disabled(!add_enabled)
        .action(act::Add)
        .children(ui::children![cx; icon::icon(cx, IconId::new("lucide.plus"))])
        .ui()
        .rounded_md();
"#
    } else {
        r#"    let add_btn = shadcn::Button::new("Add")
        .disabled(!add_enabled)
        .action(act::Add)
        .ui()
        .rounded_md();
"#
    };

    let palette_button = if opts.command_palette {
        r#"
                shadcn::Button::new("Command palette")
                    .action("app.command_palette"),"#
    } else {
        ""
    };

    let install_icons = match opts.icon_pack {
        IconPack::Radix => {
            r#"    fret_icons_radix::install_app(app);
"#
        }
        IconPack::Lucide | IconPack::None => "",
    };

    const TEMPLATE: &str = r#"use std::sync::Arc;
use std::time::Duration;

use fret::prelude::*;
use fret_query::{QueryKey, QueryPolicy, QueryState, QueryStatus};
use fret_selector::ui::DepsBuilder;

mod act {
    fret::actions!([
        Add = "__PACKAGE_NAME__.todo.add.v1",
        ClearDone = "__PACKAGE_NAME__.todo.clear_done.v1",
        RefreshTip = "__PACKAGE_NAME__.todo.refresh_tip.v1",
        FilterAll = "__PACKAGE_NAME__.todo.filter_all.v1",
        FilterActive = "__PACKAGE_NAME__.todo.filter_active.v1",
        FilterCompleted = "__PACKAGE_NAME__.todo.filter_completed.v1"
    ]);
}

#[derive(Clone)]
struct TodoItem {
    id: u64,
    done: Model<bool>,
    text: Arc<str>,
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

fn tip_key(nonce: u64) -> QueryKey<TipData> {
    QueryKey::new("__PACKAGE_NAME__.todo.tip.v1", &nonce)
}

fn tip_policy() -> QueryPolicy {
    QueryPolicy {
        stale_time: Duration::from_secs(10),
        cache_time: Duration::from_secs(60),
        keep_previous_data_while_loading: true,
        ..Default::default()
    }
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

struct TodoView {
    todos: Model<Vec<TodoItem>>,
    draft: Model<String>,
    filter: Model<TodoFilter>,
    next_id: Model<u64>,
    tip_nonce: Model<u64>,
}

impl View for TodoView {
    fn init(app: &mut App, _window: AppWindowId) -> Self {
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

        Self {
            todos,
            draft: app.models_mut().insert(String::new()),
            filter: app.models_mut().insert(TodoFilter::All),
            next_id: app.models_mut().insert(3u64),
            tip_nonce: app.models_mut().insert(0u64),
        }
    }

    fn render(&mut self, cx: &mut ViewCx<'_, '_, App>) -> Elements {
        let theme = Theme::global(&*cx.app).snapshot();

        let draft_value = cx
            .watch_model(&self.draft)
            .layout()
            .value_or_default();
        let filter_value = cx
            .watch_model(&self.filter)
            .layout()
            .value_or(TodoFilter::All);

        let add_enabled = !draft_value.trim().is_empty();

        cx.on_action_notify_models::<act::Add>({
            let todos = self.todos.clone();
            let draft = self.draft.clone();
            let next_id = self.next_id.clone();
            move |models| {
                let text = models
                    .read(&draft, |s| s.trim().to_string())
                    .ok()
                    .unwrap_or_default();
                if text.is_empty() {
                    return false;
                }

                let id = models
                    .read(&next_id, |v| *v)
                    .ok()
                    .unwrap_or(1);
                let _ = models.update(&next_id, |v| *v = v.saturating_add(1));

                let done = models.insert(false);
                let item = TodoItem {
                    id,
                    done,
                    text: Arc::from(text),
                };

                let _ = models.update(&todos, |todos| {
                    todos.insert(0, item);
                });
                let _ = models.update(&draft, |s| s.clear());
                true
            }
        });

        cx.on_action_notify_models::<act::ClearDone>({
            let todos = self.todos.clone();
            move |models| {
                let snapshot = models
                    .read(&todos, |v| v.clone())
                    .ok()
                    .unwrap_or_default();

                let mut keep: Vec<TodoItem> = Vec::new();
                for t in snapshot {
                    let done = models
                        .read(&t.done, |v| *v)
                        .ok()
                        .unwrap_or(false);
                    if !done {
                        keep.push(t);
                    }
                }

                let _ = models.update(&todos, |todos| {
                    *todos = keep;
                });

                true
            }
        });

        cx.on_action_notify_models::<act::RefreshTip>({{
            let tip_nonce = self.tip_nonce.clone();
            move |models| models.update(&tip_nonce, |v| *v = v.saturating_add(1)).is_ok()
        }});

        cx.on_action_notify_models::<act::FilterAll>({{
            let filter = self.filter.clone();
            move |models| models.update(&filter, |v| *v = TodoFilter::All).is_ok()
        }});
        cx.on_action_notify_models::<act::FilterActive>({{
            let filter = self.filter.clone();
            move |models| models.update(&filter, |v| *v = TodoFilter::Active).is_ok()
        }});
        cx.on_action_notify_models::<act::FilterCompleted>({{
            let filter = self.filter.clone();
            move |models| models.update(&filter, |v| *v = TodoFilter::Completed).is_ok()
        }});

        let derived: TodoDerived = cx.use_selector(
            |cx| {
                let todos = cx
                    .watch_model(&self.todos)
                    .layout()
                    .value_or_default();
                let mut deps = DepsBuilder::new(cx);
                deps.model_rev(&self.todos);
                deps.model_rev(&self.filter);
                for t in &todos {
                    deps.model_rev(&t.done);
                }
                deps.finish()
            },
            |cx| {
                let todos = cx
                    .watch_model(&self.todos)
                    .layout()
                    .value_or_default();
                let filter = cx
                    .watch_model(&self.filter)
                    .layout()
                    .value_or(TodoFilter::All);

                let mut rows = Vec::new();
                let mut completed = 0usize;
                for t in &todos {
                    let done = cx.watch_model(&t.done).paint().value_or_default();
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
            },
        );

        let tip_nonce_value = cx.watch_model(&self.tip_nonce).paint().value_or(0);
        let tip_handle =
            cx.use_query(tip_key(tip_nonce_value), tip_policy(), move |_token| {
                #[cfg(not(target_arch = "wasm32"))]
                std::thread::sleep(Duration::from_millis(150));

                Ok(TipData {
                    text: Arc::from(format!(
                        "Tip fetched at {:?}",
                        std::time::SystemTime::now()
                    )),
                })
            });

        let tip_state = tip_handle
            .layout(cx)
            .value_or_else(QueryState::<TipData>::default);

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

        let progress = shadcn::Badge::new(format!("{}/{} done", derived.completed, derived.total))
            .variant(shadcn::BadgeVariant::Secondary);

        let tip = shadcn::Badge::new(tip_text.clone())
            .variant(shadcn::BadgeVariant::Outline)
            .ui()
            .text_color(ColorRef::Color(theme.color_token(tip_color_key)));

        let refresh_tip_btn = shadcn::Button::new("Refresh tip")
            .variant(shadcn::ButtonVariant::Secondary)
            .action(act::RefreshTip)
            .ui()
            .rounded_md();

        let stats = ui::h_flex(|cx| ui::children![cx; progress, tip, refresh_tip_btn])
            .gap(Space::N2)
            .items_center();

        let clear_done_btn = shadcn::Button::new("Clear done")
            .variant(shadcn::ButtonVariant::Secondary)
            .disabled(derived.completed == 0)
            .action(act::ClearDone)
            .ui()
            .rounded_md();

__ADD_BTN_DEF__

        let input = shadcn::Input::new(&draft_state)
            .placeholder("Add a task…")
            .submit_command(act::Add.into());

        let input_row = ui::h_flex(|cx| ui::children![cx; input, add_btn])
            .gap(Space::N2)
            .items_center()
            .w_full();

        let chips = ui::h_flex(|cx| ui::children![cx;
            filter_chip(cx, TodoFilter::All, filter_value),
            filter_chip(cx, TodoFilter::Active, filter_value),
            filter_chip(cx, TodoFilter::Completed, filter_value),
        ])
        .gap(Space::N1)
        .items_center();

        let rows = ui::v_flex_build(|cx, out| {
            for row in derived.rows.iter() {
                out.push_ui(cx, ui::keyed(row.id, |_cx| todo_row(theme, row)));
            }
        })
        .gap(Space::N3)
        .w_full();

        let content = ui::v_flex(|cx| ui::children![cx;
            stats,
            chips,
            input_row,
            rows,
            clear_done_btn,
__PALETTE_BUTTON__
        ])
        .gap(Space::N4)
        .w_full();

        let card = shadcn::Card::build(|cx, out| {
            out.push_ui(
                cx,
                shadcn::CardHeader::build(|cx, out| {
                    out.push_ui(cx, shadcn::CardTitle::new("Todo"));
                    out.push_ui(
                        cx,
                        shadcn::CardDescription::new("View runtime + typed actions + selector + query (v1)."),
                    );
                }),
            );
            out.push_ui(
                cx,
                shadcn::CardContent::build(|cx, out| {
                    out.push_ui(cx, content);
                }),
            );
        })
        .ui()
        .bg(ColorRef::Color(theme.color_token("background")))
        .rounded(Radius::Lg)
        .border_1()
        .border_color(ColorRef::Color(theme.color_token("border")))
        .w_full()
        .max_w(Px(520.0))
        ;

        let page = ui::container(|cx| ui::children![cx;
            ui::v_flex(|cx| ui::children![cx; card])
                .w_full()
                .h_full()
                .justify_center()
                .items_center()
                ,
        ])
        .bg(ColorRef::Color(theme.color_token("muted")))
        .p(Space::N6)
        .w_full()
        .h_full()
        ;

        page.into_element(cx).into()
    }
}

fn filter_chip(
    cx: &mut ElementContext<'_, App>,
    filter: TodoFilter,
    current: TodoFilter,
) -> impl UiChildIntoElement<App> {
    let selected = filter == current;
    shadcn::Button::new(filter.as_label())
        .variant(if selected {
            shadcn::ButtonVariant::Secondary
        } else {
            shadcn::ButtonVariant::Ghost
        })
        .size(shadcn::ButtonSize::Sm)
        .action(match filter {
            TodoFilter::All => act::FilterAll,
            TodoFilter::Active => act::FilterActive,
            TodoFilter::Completed => act::FilterCompleted,
        })
}

fn todo_row(theme: ThemeSnapshot, row: &TodoRowSnapshot) -> impl UiChildIntoElement<App> {
    let checkbox = shadcn::Checkbox::new(row.done_model.clone());

    let label = ui::raw_text(row.text.clone())
        .text_color(ColorRef::Color(theme.color_token(if row.done {
            "muted-foreground"
        } else {
            "foreground"
        })))
        .truncate();

    let row = ui::h_flex(|cx| ui::children![cx; checkbox, label])
        .gap(Space::N3)
        .items_center()
        .w_full();

    ui::container(|cx| ui::children![cx; row])
        .border_1()
        .border_color(ColorRef::Color(theme.color_token("border")))
        .rounded(Radius::Md)
        .p(Space::N3)
        .w_full()
}

fn install_app(app: &mut App) {
__INSTALL_ICONS__
    // Register app-owned globals, commands, services, etc.
}

fn main() -> anyhow::Result<()> {
    FretApp::new("__PACKAGE_NAME__")
        .window("__PACKAGE_NAME__", (560.0, 520.0))
        .install_app(install_app)
        .run_view::<TodoView>()
        .map_err(anyhow::Error::from)
}
"#;

    TEMPLATE
        .replace("__ADD_BTN_DEF__", add_btn_def)
        .replace("__INSTALL_ICONS__", install_icons)
        .replace("__PALETTE_BUTTON__", palette_button)
        .replace("__PACKAGE_NAME__", package_name)
}

pub(super) fn hello_template_main_rs(package_name: &str, opts: ScaffoldOptions) -> String {
    let palette_button = if opts.command_palette {
        r#"
                shadcn::Button::new("Command palette")
                    .action("app.command_palette")
                    ,"#
    } else {
        ""
    };

    let install_icons = match opts.icon_pack {
        IconPack::Radix => {
            r#"    fret_icons_radix::install_app(app);
"#
        }
        IconPack::Lucide | IconPack::None => "",
    };

    format!(
        r#"use fret::prelude::*;

mod act {{
    fret::actions!([Click = "{package_name}.hello.click.v1"]);
}}

struct HelloView;

impl View for HelloView {{
    fn init(_app: &mut App, _window: AppWindowId) -> Self {{
        Self
    }}

    fn render(&mut self, cx: &mut ViewCx<'_, '_, App>) -> Elements {{
        let click_count = cx.use_state::<u32>();
        let click_count_value = cx.watch_model(&click_count).layout().value_or(0);

        cx.on_action_notify_models::<act::Click>({{
            let click_count = click_count.clone();
            move |models| models.update(&click_count, |v| *v = v.saturating_add(1)).is_ok()
        }});

        ui::v_flex(|cx| {{
            ui::children![cx;
                shadcn::Label::new("Hello, world!"),
                cx.text(format!("Clicks: {{click_count_value}}")),
                shadcn::Button::new("Click me").action(act::Click),
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

fn install_app(app: &mut App) {{
__INSTALL_ICONS__
    // Register app-owned globals, commands, services, etc.
}}

fn main() -> anyhow::Result<()> {{
    FretApp::new("{package_name}")
        .window("{package_name}", (560.0, 360.0))
        .install_app(install_app)
        .run_view::<HelloView>()
        .map_err(anyhow::Error::from)
}}
"#
    )
    .replace("__PALETTE_BUTTON__", palette_button)
    .replace("__INSTALL_ICONS__", install_icons)
}

pub(super) fn simple_todo_template_main_rs(package_name: &str, opts: ScaffoldOptions) -> String {
    // Radix doesn't currently ship the Lucide plus icon in our curated set; keep the template
    // functional by falling back to text buttons when Lucide isn't selected.
    let has_action_icons = matches!(opts.icon_pack, IconPack::Lucide);

    let add_btn_def = if has_action_icons {
        r#"    let add_btn = shadcn::Button::new("")
        .size(shadcn::ButtonSize::Icon)
        .disabled(!add_enabled)
        .action(act::Add)
        .children(ui::children![cx; icon::icon(cx, IconId::new("lucide.plus"))])
        .ui()
        .rounded_md();
"#
    } else {
        r#"    let add_btn = shadcn::Button::new("Add")
        .disabled(!add_enabled)
        .action(act::Add)
        .ui()
        .rounded_md();
"#
    };

    let palette_button = if opts.command_palette {
        r#"
            shadcn::Button::new("Command palette")
                .action("app.command_palette")
                ,"#
    } else {
        ""
    };

    let install_icons = match opts.icon_pack {
        IconPack::Radix => {
            r#"    fret_icons_radix::install_app(app);
"#
        }
        IconPack::Lucide | IconPack::None => "",
    };

    const TEMPLATE: &str = r#"use std::sync::Arc;

use fret::prelude::*;

mod act {
    fret::actions!([
        Add = "__PACKAGE_NAME__.simple_todo.add.v1",
        ClearDone = "__PACKAGE_NAME__.simple_todo.clear_done.v1"
    ]);
}

#[derive(Clone)]
struct TodoItem {
    id: u64,
    done: Model<bool>,
    text: Arc<str>,
}

struct TodoView {
    todos: Model<Vec<TodoItem>>,
}

impl View for TodoView {
    fn init(app: &mut App, _window: AppWindowId) -> Self {
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

        Self { todos }
    }

    fn render(&mut self, cx: &mut ViewCx<'_, '_, App>) -> Elements {
        let theme = Theme::global(&*cx.app).snapshot();

        let draft_state = cx.use_local::<String>();
        let next_id_state = cx.use_local_with(|| 3u64);

        let todos = cx.watch_model(&self.todos).layout().value_or_default();
        let draft_value = draft_state.watch(cx).layout().value_or_default();

        let mut row_done = Vec::with_capacity(todos.len());
        let mut done_count = 0usize;
        for t in &todos {
            let done = cx.watch_model(&t.done).paint().value_or_default();
            row_done.push(done);
            if done {
                done_count += 1;
            }
        }
        let total_count = todos.len();

        let add_enabled = !draft_value.trim().is_empty();

        cx.on_action_notify_models::<act::Add>({
            let todos = self.todos.clone();
            let draft_state = draft_state.clone();
            let next_id_state = next_id_state.clone();
            move |models| {
                let text = draft_state.value_in_or_else(models, String::new).trim().to_string();
                if text.is_empty() {
                    return false;
                }

                let done = models.insert(false);
                let id = next_id_state.value_in_or(models, 1);
                let _ = next_id_state.update_in(models, |value| *value = value.saturating_add(1));

                let _ = models.update(&todos, |todos| {
                    todos.push(TodoItem {
                        id,
                        done,
                        text: Arc::from(text),
                    });
                });
                draft_state.set_in(models, String::new())
            }
        });

        cx.on_action_notify_models::<act::ClearDone>({
            let todos = self.todos.clone();
            move |models| {
                let snapshot = models
                    .read(&todos, |v| v.clone())
                    .ok()
                    .unwrap_or_default();

                let mut keep = Vec::new();
                for t in snapshot {
                    let done = models
                        .read(&t.done, |v| *v)
                        .ok()
                        .unwrap_or(false);
                    if !done {
                        keep.push(t);
                    }
                }

                let _ = models.update(&todos, |todos| *todos = keep);
                true
            }
        });

        let progress = shadcn::Badge::new(format!("{done_count}/{total_count} done"))
            .variant(shadcn::BadgeVariant::Secondary);

        let clear_done_btn = shadcn::Button::new("Clear done")
            .variant(shadcn::ButtonVariant::Secondary)
            .disabled(done_count == 0)
            .action(act::ClearDone)
            .ui()
            .rounded_md();

        let header_actions = ui::h_flex(|cx| ui::children![cx; progress, clear_done_btn])
            .gap(Space::N2)
            .items_center();

__ADD_BTN_DEF__

        let input = shadcn::Input::new(&draft_state)
            .placeholder("Add a task…")
            .submit_command(act::Add.into());

        let input_row = ui::h_flex(|cx| ui::children![cx; input, add_btn])
            .gap(Space::N2)
            .items_center()
            .w_full();

        let rows = ui::v_flex_build(|cx, out| {
            for (t, done) in todos.iter().zip(row_done.iter().copied()) {
                out.push_ui(cx, ui::keyed(t.id, |_cx| todo_row(theme, t, done)));
            }
        })
        .gap(Space::N3)
        .w_full();

        let content = ui::v_flex(|cx| ui::children![cx; input_row, rows])
            .gap(Space::N4)
            .w_full();

        let card = shadcn::Card::build(|cx, out| {
            out.push_ui(
                cx,
                shadcn::CardHeader::build(|cx, out| {
                    out.push_ui(cx, shadcn::CardTitle::new("Simple Todo"));
                    out.push_ui(
                        cx,
                        shadcn::CardDescription::new("View runtime + typed actions + keyed lists (no selector/query)."),
                    );
                    out.push(header_actions);
                }),
            );
            out.push_ui(
                cx,
                shadcn::CardContent::build(|cx, out| {
                    out.push_ui(cx, content);
                }),
            );
        })
        .ui()
        .bg(ColorRef::Color(theme.color_token("background")))
        .rounded(Radius::Lg)
        .border_1()
        .border_color(ColorRef::Color(theme.color_token("border")))
        .w_full()
        .max_w(Px(520.0))
        ;

        let page = ui::container(|cx| {
            ui::children![cx;
                ui::v_flex(|cx| ui::children![cx;
                    card,
__PALETTE_BUTTON__
                ])
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
}

fn todo_row(theme: ThemeSnapshot, item: &TodoItem, done: bool) -> impl UiChildIntoElement<App> {
    let checkbox = shadcn::Checkbox::new(item.done.clone());

    let text = ui::raw_text(item.text.clone())
        .truncate()
        .text_sm()
        .text_color(ColorRef::Color(if done {
            theme.color_token("muted-foreground")
        } else {
            theme.color_token("foreground")
        }));

    ui::h_flex(|cx| ui::children![cx; checkbox, text])
        .gap(Space::N2)
        .items_center()
        .w_full()
}

fn install_app(app: &mut App) {
__INSTALL_ICONS__
    // Register app-owned globals, commands, services, etc.
}

fn main() -> anyhow::Result<()> {
    FretApp::new("__PACKAGE_NAME__")
        .window("__PACKAGE_NAME__", (560.0, 520.0))
        .install_app(install_app)
        .run_view::<TodoView>()
        .map_err(anyhow::Error::from)
}
"#;

    TEMPLATE
        .replace("__ADD_BTN_DEF__", add_btn_def)
        .replace("__INSTALL_ICONS__", install_icons)
        .replace("__PALETTE_BUTTON__", palette_button)
        .replace("__PACKAGE_NAME__", package_name)
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
        IconPack::Lucide => "- Icons: enabled (default Lucide pack)\n",
        IconPack::Radix => "- Icons: Radix (via `fret-icons-radix` dependency)\n",
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

## Notes

- Theme: shadcn new-york-v4 (Slate / Light)
{icons_line}{palette_line}
{ui_assets_line}
- Authoring: view runtime + typed unit actions (action-first, v1)
- Hooks: selector + query (v1)
- Default entrypoints: `on_action_notify_models`, `on_action_notify_transient`, and local `on_activate*` only when you truly need widget-local pressable glue.
- Treat raw `on_action_notify` as cookbook/reference-only host-side glue.
- Read model values near the top of `render()` before building nested card/layout sections.
- For App-only effects, prefer `on_action_notify_transient` in the handler and consume the transient in `render()`.
## Next steps

- Edit UI in `src/main.rs`
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
        IconPack::Lucide => "- Icons: enabled (default Lucide pack)\n",
        IconPack::Radix => "- Icons: Radix (via `fret-icons-radix` dependency)\n",
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
- Authoring: view runtime + typed unit actions (action-first, v1)
- Default entrypoints: start with `on_action_notify_models`; keep `on_activate*` for local widget glue only.
- Treat raw `on_action_notify` as cookbook/reference-only host-side glue.
- For keyed dynamic lists, keep the collection in an explicit `Model<Vec<_>>` when row items still own nested models, and move adjacent draft/counter state to `use_local*`.
- Read tracked state near the top of `render()` and keep row rendering driven by locals.
## Next steps

- Edit UI in `src/main.rs`
- Use `ui::children![cx; ...]` to build heterogeneous child lists without call-site `.into_element(cx)` noise.
- When rendering dynamic lists, prefer `out.push_ui(cx, ui::keyed(id, |cx| ...))` to keep identity stable without an eager landing cliff.
"#
    )
}

pub(super) fn hello_template_readme_md(package_name: &str, opts: ScaffoldOptions) -> String {
    let icons_line = match opts.icon_pack {
        IconPack::Lucide => "- Icons: enabled (default Lucide pack)\n",
        IconPack::Radix => "- Icons: Radix (via `fret-icons-radix` dependency)\n",
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

## Notes

- Theme: shadcn new-york-v4 (default via `fret-ui-shadcn/app-integration`)
{icons_line}{palette_line}
- Authoring: view runtime + typed unit actions (action-first, v1)
- Default entrypoints: start with `on_action_notify_models`; use `on_activate*` only for local pressable glue.
- Treat raw `on_action_notify` as cookbook/reference-only host-side glue.
- Read model values near the top of `render()` and keep action handlers on `on_action_notify*` when possible.
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
        assert!(src.contains("ui::children!["));
        assert!(!src.contains("ui::container( |"));
        assert!(!src.contains("ui::h_flex( |"));
        assert!(!src.contains("ui::v_flex( |"));
        assert!(!src.contains("ui::raw_text( "));
        assert!(src.contains("impl View for TodoView"));
        assert!(src.contains(".run_view::<TodoView>()"));
        assert!(src.contains("fret::actions!(["));
        assert!(src.contains("shadcn::Card::build(|cx, out| {"));
        assert!(src.contains("shadcn::CardHeader::build(|cx, out| {"));
        assert!(src.contains("shadcn::CardContent::build(|cx, out| {"));
        assert!(src.contains("out.push_ui(\n                cx,\n                shadcn::CardHeader::build(|cx, out| {"));
        assert!(src.contains("out.push_ui(\n                cx,\n                shadcn::CardContent::build(|cx, out| {"));
        assert!(src.contains("cx.on_action_notify_models::<act::Add>"));
        assert!(src.contains("cx.on_action_notify_models::<act::ClearDone>"));
        assert!(src.contains("cx.on_action_notify_models::<act::RefreshTip>"));
        assert!(!src.contains("cx.on_action_notify_model_update::<act::RefreshTip"));
        assert!(!src.contains("cx.on_action_notify_model_set::<act::Filter"));
        assert!(!src.contains("decl_style::container_props"));
        assert!(!src.contains(".refine_style("));
        assert!(!src.contains(".refine_layout("));

        let into_element_count = src.matches(".into_element(cx)").count();
        assert!(
            into_element_count <= 18,
            "expected <= 18 explicit `.into_element(cx)` calls, got {into_element_count}"
        );
    }

    #[test]
    fn hello_template_uses_default_authoring_dialect() {
        let src = hello_template_main_rs("hello-app", opts());
        assert!(src.contains("ui::v_flex("));
        assert!(!src.contains("ui::v_flex( |"));
        assert!(src.contains("impl View for HelloView"));
        assert!(src.contains(".run_view::<HelloView>()"));
        assert!(src.contains("cx.on_action_notify_models::<act::Click>"));
        assert!(!src.contains("cx.on_action_notify_model_update::<act::Click"));
        assert!(src.contains(".into_element(cx)"));
        assert!(!src.contains("decl_style::container_props"));
    }

    #[test]
    fn simple_todo_template_has_low_adapter_noise_and_no_query_selector() {
        let src = simple_todo_template_main_rs("simple-todo-app", opts());
        assert!(src.contains("ui::children!["));
        assert!(src.contains("ui::keyed("));
        assert!(!src.contains("ui::container( |"));
        assert!(!src.contains("ui::h_flex( |"));
        assert!(!src.contains("ui::v_flex( |"));
        assert!(!src.contains("ui::raw_text( "));
        assert!(src.contains("impl View for TodoView"));
        assert!(src.contains(".run_view::<TodoView>()"));
        assert!(src.contains("fret::actions!(["));
        assert!(src.contains("shadcn::Card::build(|cx, out| {"));
        assert!(src.contains("shadcn::CardHeader::build(|cx, out| {"));
        assert!(src.contains("shadcn::CardContent::build(|cx, out| {"));
        assert!(src.contains("out.push_ui(\n                cx,\n                shadcn::CardHeader::build(|cx, out| {"));
        assert!(src.contains("out.push_ui(\n                cx,\n                shadcn::CardContent::build(|cx, out| {"));
        assert!(src.contains("cx.on_action_notify_models::<act::Add>"));
        assert!(src.contains("cx.on_action_notify_models::<act::ClearDone>"));
        assert!(src.contains("let draft_state = cx.use_local::<String>();"));
        assert!(src.contains("let next_id_state = cx.use_local_with(|| 3u64);"));
        assert!(src.contains("shadcn::Input::new(&draft_state)"));
        assert!(!src.contains("fret_query"));
        assert!(!src.contains("fret_selector"));
        assert!(!src.contains(".refine_style("));
        assert!(!src.contains(".refine_layout("));

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

    #[test]
    fn template_readmes_capture_authoring_guidance() {
        let hello = hello_template_readme_md("hello-app", opts());
        assert!(hello.contains("Read model values near the top of `render()`"));
        assert!(hello.contains("Default entrypoints"));
        assert!(hello.contains("cookbook/reference-only host-side glue"));

        let simple = simple_todo_template_readme_md("simple-todo-app", opts());
        assert!(simple.contains(
            "When rendering dynamic lists, prefer `out.push_ui(cx, ui::keyed(id, |cx| ...))`"
        ));
        assert!(simple.contains("keep the collection in an explicit `Model<Vec<_>>`"));
        assert!(simple.contains("Read tracked state near the top of `render()`"));
        assert!(simple.contains("start with `on_action_notify_models`"));
        assert!(simple.contains("cookbook/reference-only host-side glue"));

        let todo = todo_template_readme_md("todo-app", opts());
        assert!(todo.contains("For App-only effects, prefer `on_action_notify_transient`"));
        assert!(todo.contains("cookbook/reference-only host-side glue"));
        assert!(todo.contains(
            "Default entrypoints: `on_action_notify_models`, `on_action_notify_transient`"
        ));
    }
}
