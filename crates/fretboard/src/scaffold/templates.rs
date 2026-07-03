use super::{IconPack, ScaffoldOptions};

enum DependencySpec<'a> {
    Published { version: &'a str },
    WorkspacePath { workspace_prefix: &'a str },
}

impl DependencySpec<'_> {
    fn fret_dependency_line(&self, features: &str) -> String {
        match self {
            Self::Published { version } => format!(
                "fret = {{ version = \"{version}\", default-features = false, features = [{features}] }}\n"
            ),
            Self::WorkspacePath { workspace_prefix } => {
                let fret_path = join_workspace_path(workspace_prefix, "ecosystem/fret");
                format!(
                    "fret = {{ path = \"{fret_path}\", default-features = false, features = [{features}] }}\n"
                )
            }
        }
    }

    fn radix_dependency_line(&self) -> String {
        match self {
            Self::Published { version } => {
                format!(
                    "fret-icons-radix = {{ version = \"{version}\", features = [\"app-integration\"] }}\n"
                )
            }
            Self::WorkspacePath { workspace_prefix } => {
                let fret_icons_radix_path =
                    join_workspace_path(workspace_prefix, "ecosystem/fret-icons-radix");
                format!(
                    "fret-icons-radix = {{ path = \"{fret_icons_radix_path}\", features = [\"app-integration\"] }}\n"
                )
            }
        }
    }
}

fn join_workspace_path(workspace_prefix: &str, subpath: &str) -> String {
    if workspace_prefix == "." {
        subpath.to_string()
    } else {
        format!("{workspace_prefix}/{subpath}")
    }
}

fn generated_assets_module_decl(opts: ScaffoldOptions) -> &'static str {
    if opts.ui_assets {
        "mod generated_assets;\n\n"
    } else {
        ""
    }
}

fn generated_assets_builder_prefix(opts: ScaffoldOptions) -> &'static str {
    if opts.ui_assets {
        "    let builder = "
    } else {
        ""
    }
}

fn generated_assets_builder_suffix(opts: ScaffoldOptions) -> &'static str {
    if opts.ui_assets {
        ";\n    generated_assets::mount(builder)?\n"
    } else {
        ""
    }
}

fn lucide_action_icon_import(opts: ScaffoldOptions) -> &'static str {
    if matches!(opts.icon_pack, IconPack::Lucide) {
        "    icons::{icon, IconId},\n"
    } else {
        ""
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

pub(super) fn todo_template_cargo_toml_public(
    package_name: &str,
    opts: ScaffoldOptions,
    version: &str,
) -> String {
    todo_template_cargo_toml_with(package_name, opts, DependencySpec::Published { version })
}

pub(super) fn todo_template_cargo_toml_repo(
    package_name: &str,
    opts: ScaffoldOptions,
    workspace_prefix: &str,
) -> String {
    todo_template_cargo_toml_with(
        package_name,
        opts,
        DependencySpec::WorkspacePath { workspace_prefix },
    )
}

fn todo_template_cargo_toml_with(
    package_name: &str,
    opts: ScaffoldOptions,
    deps: DependencySpec<'_>,
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

    let radix_dep = if matches!(opts.icon_pack, IconPack::Radix) {
        deps.radix_dependency_line()
    } else {
        String::new()
    };
    let fret_dep = deps.fret_dependency_line(&kit_features);

    format!(
        r#"[package]
name = "{package_name}"
version = "0.1.0"
edition = "2024"

[dependencies]
anyhow = "1"
{fret_dep}
{radix_dep}
[workspace]
"#
    )
}

pub(super) fn workbench_lite_template_cargo_toml_public(
    package_name: &str,
    opts: ScaffoldOptions,
    version: &str,
) -> String {
    workbench_lite_template_cargo_toml_with(
        package_name,
        opts,
        DependencySpec::Published { version },
    )
}

pub(super) fn workbench_lite_template_cargo_toml_repo(
    package_name: &str,
    opts: ScaffoldOptions,
    workspace_prefix: &str,
) -> String {
    workbench_lite_template_cargo_toml_with(
        package_name,
        opts,
        DependencySpec::WorkspacePath { workspace_prefix },
    )
}

pub(super) fn mutation_workbench_template_cargo_toml_public(
    package_name: &str,
    opts: ScaffoldOptions,
    version: &str,
) -> String {
    mutation_workbench_template_cargo_toml_with(
        package_name,
        opts,
        DependencySpec::Published { version },
    )
}

pub(super) fn mutation_workbench_template_cargo_toml_repo(
    package_name: &str,
    opts: ScaffoldOptions,
    workspace_prefix: &str,
) -> String {
    mutation_workbench_template_cargo_toml_with(
        package_name,
        opts,
        DependencySpec::WorkspacePath { workspace_prefix },
    )
}

fn mutation_workbench_template_cargo_toml_with(
    package_name: &str,
    opts: ScaffoldOptions,
    deps: DependencySpec<'_>,
) -> String {
    let mut kit_features: Vec<&str> = vec![
        "desktop",
        "shadcn",
        "state-query",
        "state-mutation",
        "command-palette",
        "diagnostics",
    ];
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

    let radix_dep = if matches!(opts.icon_pack, IconPack::Radix) {
        deps.radix_dependency_line()
    } else {
        String::new()
    };
    let fret_dep = deps.fret_dependency_line(&kit_features);

    format!(
        r#"[package]
name = "{package_name}"
version = "0.1.0"
edition = "2024"

[dependencies]
anyhow = "1"
{fret_dep}
{radix_dep}
tokio = {{ version = "1", default-features = false, features = ["rt-multi-thread", "time"] }}

[workspace]
"#
    )
}

fn workbench_lite_template_cargo_toml_with(
    package_name: &str,
    opts: ScaffoldOptions,
    deps: DependencySpec<'_>,
) -> String {
    let mut kit_features: Vec<&str> = vec!["desktop", "shadcn", "command-palette", "diagnostics"];
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

    let radix_dep = if matches!(opts.icon_pack, IconPack::Radix) {
        deps.radix_dependency_line()
    } else {
        String::new()
    };
    let fret_dep = deps.fret_dependency_line(&kit_features);

    format!(
        r#"[package]
name = "{package_name}"
version = "0.1.0"
edition = "2024"

[dependencies]
anyhow = "1"
{fret_dep}
{radix_dep}

[workspace]
"#
    )
}

pub(super) fn simple_todo_template_cargo_toml_public(
    package_name: &str,
    opts: ScaffoldOptions,
    version: &str,
) -> String {
    simple_todo_template_cargo_toml_with(package_name, opts, DependencySpec::Published { version })
}

pub(super) fn simple_todo_template_cargo_toml_repo(
    package_name: &str,
    opts: ScaffoldOptions,
    workspace_prefix: &str,
) -> String {
    simple_todo_template_cargo_toml_with(
        package_name,
        opts,
        DependencySpec::WorkspacePath { workspace_prefix },
    )
}

fn simple_todo_template_cargo_toml_with(
    package_name: &str,
    opts: ScaffoldOptions,
    deps: DependencySpec<'_>,
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

    let radix_dep = if matches!(opts.icon_pack, IconPack::Radix) {
        deps.radix_dependency_line()
    } else {
        String::new()
    };
    let fret_dep = deps.fret_dependency_line(&kit_features);

    format!(
        r#"[package]
name = "{package_name}"
version = "0.1.0"
edition = "2024"

[dependencies]
anyhow = "1"
{fret_dep}
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

pub(super) fn hello_template_cargo_toml_public(
    package_name: &str,
    opts: ScaffoldOptions,
    version: &str,
) -> String {
    hello_template_cargo_toml_with(package_name, opts, DependencySpec::Published { version })
}

pub(super) fn hello_template_cargo_toml_repo(
    package_name: &str,
    opts: ScaffoldOptions,
    workspace_prefix: &str,
) -> String {
    hello_template_cargo_toml_with(
        package_name,
        opts,
        DependencySpec::WorkspacePath { workspace_prefix },
    )
}

fn hello_template_cargo_toml_with(
    package_name: &str,
    opts: ScaffoldOptions,
    deps: DependencySpec<'_>,
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

    let radix_dep = if matches!(opts.icon_pack, IconPack::Radix) {
        deps.radix_dependency_line()
    } else {
        String::new()
    };
    let fret_dep = deps.fret_dependency_line(&kit_features);

    format!(
        r#"[package]
name = "{package_name}"
version = "0.1.0"
edition = "2024"

[dependencies]
anyhow = "1"
{fret_dep}
{radix_dep}

[workspace]
"#
    )
}

pub(super) fn todo_template_main_rs(package_name: &str, opts: ScaffoldOptions) -> String {
    // Radix doesn't currently ship the Lucide plus icon in our curated set; keep the template
    // functional by falling back to a text button when Lucide isn't selected.
    let has_action_icons = matches!(opts.icon_pack, IconPack::Lucide);
    let install_app_binding = if matches!(opts.icon_pack, IconPack::Radix) {
        "app"
    } else {
        "_app"
    };

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
        r#"    let add_btn = shadcn::Button::new("Add task")
        .disabled(!add_enabled)
        .action(act::Add)
        .ui()
        .rounded_md();
"#
    };

    let install_icons = match opts.icon_pack {
        IconPack::Radix => {
            r#"    fret_icons_radix::app::install(app);
"#
        }
        IconPack::Lucide | IconPack::None => "",
    };
    let generated_assets_module = generated_assets_module_decl(opts);
    let builder_prefix = generated_assets_builder_prefix(opts);
    let builder_suffix = generated_assets_builder_suffix(opts);
    let icon_import = lucide_action_icon_import(opts);

    const TEMPLATE: &str = r#"use std::sync::Arc;
use std::time::Duration;

use fret::app::LocalState;
use fret::app::prelude::*;
use fret::{
__ICON_IMPORT__    query::{QueryKey, QueryPolicy},
    style::{ChromeRefinement, ColorRef, LayoutRefinement, Radius, Space, Theme, ThemeSnapshot},
};

__GENERATED_ASSET_MODULE__
mod act {
    fret::actions!([
        Add = "__PACKAGE_NAME__.todo.add.v1",
        ClearDone = "__PACKAGE_NAME__.todo.clear_done.v1",
        RefreshTip = "__PACKAGE_NAME__.todo.refresh_tip.v1"
    ]);

    fret::payload_actions!([Toggle(u64) = "__PACKAGE_NAME__.todo.toggle.v1"]);
}

#[derive(Clone)]
struct TodoRow {
    id: u64,
    done: bool,
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

    fn value(self) -> &'static str {
        match self {
            Self::All => "all",
            Self::Active => "active",
            Self::Completed => "completed",
        }
    }

    fn from_value(value: Option<&str>) -> Self {
        match value {
            Some("active") => Self::Active,
            Some("completed") => Self::Completed,
            _ => Self::All,
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
    text: Arc<str>,
}

struct TodoLocals {
    draft: LocalState<String>,
    filter: LocalState<Option<Arc<str>>>,
    next_id: LocalState<u64>,
    tip_nonce: LocalState<u64>,
    todos: LocalState<Vec<TodoRow>>,
}

impl TodoLocals {
    fn new(cx: &mut AppUi<'_, '_>) -> Self {
        Self {
            draft: cx.state().local::<String>(),
            filter: cx
                .state()
                .local_init(|| Some(Arc::from(TodoFilter::All.value()))),
            next_id: cx.state().local_init(|| 3u64),
            tip_nonce: cx.state().local_init(|| 0u64),
            todos: cx.state().local_init(|| vec![
                    TodoRow {
                        id: 1,
                        done: false,
                        text: Arc::from("Draft the Friday release checklist"),
                    },
                    TodoRow {
                        id: 2,
                        done: true,
                        text: Arc::from("Reply to the design review notes"),
                    },
                ]),
        }
    }

    fn bind_actions(&self, cx: &mut AppUi<'_, '_>) {
        cx.actions()
            .locals_with((&self.draft, &self.next_id, &self.todos))
            .on::<act::Add>(|tx, (draft, next_id, todos)| {
                let text = tx.value(&draft).trim().to_string();
                if text.is_empty() {
                    return false;
                }

                let id = tx.value(&next_id);
                let _ = tx.update(&next_id, |v| *v = v.saturating_add(1));

                let item = TodoRow {
                    id,
                    done: false,
                    text: Arc::from(text),
                };

                if !tx.update(&todos, |rows| rows.insert(0, item)) {
                    return false;
                }

                tx.set(&draft, String::new())
            });

        cx.actions()
            .locals_with(&self.todos)
            .on::<act::ClearDone>(|tx, todos| {
                tx.update_if(&todos, |rows| {
                    let before = rows.len();
                    rows.retain(|row| !row.done);
                    rows.len() != before
                })
            });

        cx.actions()
            .local(&self.tip_nonce)
            .update::<act::RefreshTip>(|v| {
                *v = v.saturating_add(1);
            });

        cx.actions()
            .local(&self.todos)
            .payload_update_if::<act::Toggle>(|rows, id| {
                if let Some(row) = rows.iter_mut().find(|row| row.id == id) {
                    row.done = !row.done;
                    true
                } else {
                    false
                }
            });
    }
}

struct TodoView;

impl View for TodoView {
    fn init(_app: &mut App, _window: WindowId) -> Self {
        Self
    }

    fn render(&mut self, cx: &mut AppUi<'_, '_>) -> Ui {
        let theme = Theme::global(cx.app()).snapshot();
        let theme_for_rows = theme.clone();
        let locals = TodoLocals::new(cx);
        locals.bind_actions(cx);

        let draft_value = locals.draft.layout_value(cx);
        let filter_value = TodoFilter::from_value(locals.filter.layout_value(cx).as_deref());

        let add_enabled = !draft_value.trim().is_empty();
        let muted_foreground = theme.color_token("muted-foreground");

        let derived: TodoDerived = cx
            .data()
            .selector_layout((&locals.todos, &locals.filter), |(todos, filter)| {
                let filter = TodoFilter::from_value(filter.as_deref());
                let mut rows = Vec::new();
                let mut completed = 0usize;
                for t in todos.iter() {
                    let done = t.done;
                    if done {
                        completed += 1;
                    }
                    if filter.matches(done) {
                        rows.push(TodoRowSnapshot {
                            id: t.id,
                            done,
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
            });

        let status_text = if derived.total == 0 {
            "Capture the next thing worth doing.".to_string()
        } else if derived.active == 0 {
            "Everything is done. Clear completed or add the next task.".to_string()
        } else {
            format!(
                "{} task{} still in progress.",
                derived.active,
                if derived.active == 1 { "" } else { "s" }
            )
        };

        let tip_nonce_value = locals.tip_nonce.paint_value(cx);
        let tip_handle = cx.data().query(tip_key(tip_nonce_value), tip_policy(), move |_token| {
                const TIPS: &[&str] = &[
                    "Finish one active task before adding another.",
                    "Break the next large item into a single concrete step.",
                    "Delete stale tasks when they stop being real work.",
                ];

                Ok(TipData {
                    text: Arc::from(TIPS[(tip_nonce_value as usize) % TIPS.len()]),
                })
            });

        let tip_state = tip_handle.read_layout(cx);

        let (tip_text, tip_color_key): (Arc<str>, &'static str) = if tip_state.is_loading()
            || tip_state.is_idle()
        {
            (Arc::from("Loading a focus note..."), "muted-foreground")
        } else if tip_state.is_error() {
            let err = tip_state
                .error
                .as_ref()
                .map(|e| e.to_string())
                .unwrap_or_else(|| String::from("unknown error"));
            (Arc::from(format!("Could not load a focus note: {err}")), "destructive")
        } else {
            let text = tip_state
                .data
                .as_ref()
                .map(|d| d.text.clone())
                .unwrap_or_else(|| Arc::<str>::from("<no note>"));
            (text, "muted-foreground")
        };
        let tip_color = theme.color_token(tip_color_key);

        let progress_label = if derived.total == 0 {
            "No tasks yet".to_string()
        } else {
            format!("{} of {} done", derived.completed, derived.total)
        };

        let progress_badge = shadcn::Badge::new(progress_label)
            .variant(shadcn::BadgeVariant::Secondary);

        let summary = ui::text(status_text)
            .text_sm()
            .text_color(ColorRef::Color(muted_foreground));

        let title_block = ui::v_flex(|cx| {
            ui::children![
                cx;
                shadcn::card_title("My tasks"),
                summary,
            ]
        })
        .gap(Space::N1)
        .flex_1()
        .min_w_0();

        let header = ui::h_flex(|cx| ui::children![cx; title_block, progress_badge])
            .gap(Space::N3)
            .items_center()
            .justify_between()
            .w_full();

        let clear_done_btn = shadcn::Button::new("Clear done")
            .variant(shadcn::ButtonVariant::Ghost)
            .size(shadcn::ButtonSize::Sm)
            .disabled(derived.completed == 0)
            .action(act::ClearDone)
            .ui()
            .rounded_md();

__ADD_BTN_DEF__

        let input = shadcn::Input::new(&locals.draft)
            .placeholder("Write a task and press Enter")
            .submit_action(act::Add)
            .ui()
            .flex_1()
            .min_w_0();

        let input_row = ui::h_flex(|cx| ui::children![cx; input, add_btn])
            .gap(Space::N3)
            .items_center()
            .w_full();

        let chips = shadcn::ToggleGroup::single(&locals.filter)
            .deselectable(false)
            .spacing(Space::N1)
            .refine_layout(LayoutRefinement::default().flex_none())
            .items([
                filter_group_item(cx, TodoFilter::All),
                filter_group_item(cx, TodoFilter::Active),
                filter_group_item(cx, TodoFilter::Completed),
            ]);

        let tip_callout = ui::container(|cx| {
            ui::single(
                cx,
                ui::h_flex(|cx| {
                    ui::children![
                        cx;
                        ui::v_flex(|cx| {
                            ui::children![
                                cx;
                                shadcn::Label::new("Focus note"),
                                ui::text(tip_text.clone())
                                    .text_sm()
                                    .text_color(ColorRef::Color(tip_color)),
                            ]
                        })
                        .gap(Space::N1)
                        .flex_1()
                        .min_w_0(),
                        shadcn::Button::new("Another note")
                            .variant(shadcn::ButtonVariant::Ghost)
                            .size(shadcn::ButtonSize::Sm)
                            .action(act::RefreshTip),
                    ]
                })
                .gap(Space::N3)
                .items_center()
                .w_full(),
            )
        })
        .rounded(Radius::Md)
        .border_1()
        .border_color(ColorRef::Color(theme.color_token("border")))
        .bg(ColorRef::Color(theme.color_token("muted")))
        .p(Space::N3)
        .w_full();

        let rows_body = ui::v_flex(|cx| {
            if derived.rows.is_empty() {
                let text = match filter_value {
                    TodoFilter::All => "No tasks yet. Add one above.",
                    TodoFilter::Active => "No active tasks.",
                    TodoFilter::Completed => "No completed tasks.",
                };

                return ui::children![
                    cx;
                    ui::container(|cx| {
                        ui::single(
                            cx,
                            ui::text(text)
                                .text_sm()
                                .text_color(ColorRef::Color(
                                    theme_for_rows.color_token("muted-foreground"),
                                )),
                        )
                    })
                    .rounded(Radius::Md)
                    .border_1()
                    .border_color(ColorRef::Color(theme_for_rows.color_token("border")))
                    .bg(ColorRef::Color(theme_for_rows.color_token("muted")))
                    .p(Space::N5)
                    .w_full()
                    .into_element(cx)
                ];
            }

            ui::for_each_keyed(cx, derived.rows.iter(), |row| row.id, |row| {
                let theme = theme_for_rows.clone();
                todo_row(theme, row)
            })
        })
        .gap(Space::N3)
        .w_full()
        .items_stretch();

        let rows = ui::container(|cx| ui::single(cx, rows_body))
            .rounded(Radius::Lg)
            .border_1()
            .border_color(ColorRef::Color(theme.color_token("border")))
            .bg(ColorRef::Color(theme.color_token("background")))
            .p(Space::N3)
            .w_full();

        let content = ui::v_flex(|cx| ui::children![cx;
            input_row,
            rows,
            tip_callout,
        ])
        .gap(Space::N4)
        .w_full();

        let footer_summary = if derived.total == 0 {
            "No tasks yet".to_string()
        } else {
            format!("{} active / {} completed", derived.active, derived.completed)
        };

        let footer_right = ui::h_flex(|cx| ui::children![
            cx;
            ui::text(footer_summary)
                .text_sm()
                .text_color(ColorRef::Color(muted_foreground)),
            clear_done_btn
        ])
            .gap(Space::N2)
            .items_center();

        let footer = ui::h_flex(|cx| ui::children![cx; chips, footer_right])
            .gap(Space::N3)
            .items_center()
            .justify_between()
            .w_full();

        let card = shadcn::card(|cx| {
            ui::children![cx;
                shadcn::card_header(|cx| {
                    ui::children![cx; header]
                }),
                shadcn::card_content(|cx| ui::single(cx, content)),
                shadcn::card_footer(|cx| ui::children![cx; footer]),
            ]
        })
        .ui()
        .bg(ColorRef::Color(theme.color_token("background")))
        .rounded(Radius::Lg)
        .border_1()
        .border_color(ColorRef::Color(theme.color_token("border")))
        .shadow_lg()
        .w_full()
        .max_w(Px(620.0))
        ;

        ui::single(cx, todo_page(theme, card))
    }
}

fn todo_page(
    theme: ThemeSnapshot,
    content: impl UiChild,
) -> impl UiChild {
    ui::container(move |cx| ui::single(
        cx,
        ui::v_flex(|cx| ui::single(cx, content))
            .w_full()
            .h_full()
            .justify_center()
            .items_center(),
    ))
    .bg(ColorRef::Color(theme.color_token("muted")))
    .p(Space::N6)
    .w_full()
    .h_full()
}

fn filter_group_item(cx: &mut AppUi<'_, '_>, filter: TodoFilter) -> shadcn::ToggleGroupItem {
    shadcn::ToggleGroupItem::new(filter.value(), [ui::text(filter.as_label()).into_element_in(cx)])
        .a11y_label(format!("Show {} tasks", filter.as_label().to_lowercase()))
        .refine_style(ChromeRefinement::default().rounded(Radius::Full))
        .refine_layout(LayoutRefinement::default().h_px(Px(28.0)).min_h(Px(28.0)))
}

fn todo_row(theme: ThemeSnapshot, row: &TodoRowSnapshot) -> impl UiChild {
    let checkbox = shadcn::Checkbox::from_checked(row.done)
        .action(act::Toggle)
        .action_payload(row.id)
        .a11y_label(row.text.clone());

    let text = ui::text(row.text.clone())
        .truncate()
        .text_sm()
        .flex_1()
        .min_w_0()
        .text_color(ColorRef::Color(if row.done {
            theme.color_token("muted-foreground")
        } else {
            theme.color_token("foreground")
        }));

    ui::h_flex(|cx| ui::children![cx; checkbox, text])
        .gap(Space::N3)
        .items_center()
        .bg(ColorRef::Color(if row.done {
            theme.color_token("muted")
        } else {
            theme.color_token("background")
        }))
        .border_1()
        .border_color(ColorRef::Color(theme.color_token("border")))
        .rounded(Radius::Md)
        .p(Space::N3)
        .shadow_sm()
        .w_full()
}

fn install_app(__INSTALL_APP_BINDING__: &mut App) {
__INSTALL_ICONS__
    // Register app-owned globals, commands, services, etc.
}

fn main() -> anyhow::Result<()> {
__BUILDER_PREFIX__FretApp::new("__PACKAGE_NAME__")
        .window("__PACKAGE_NAME__", (560.0, 520.0))
        .setup(install_app)
        .view::<TodoView>()?
__BUILDER_SUFFIX__        .run()
        .map_err(anyhow::Error::from)
}
"#;

    TEMPLATE
        .replace("__ADD_BTN_DEF__", add_btn_def)
        .replace("__GENERATED_ASSET_MODULE__", generated_assets_module)
        .replace("__BUILDER_PREFIX__", builder_prefix)
        .replace("__BUILDER_SUFFIX__", builder_suffix)
        .replace("__ICON_IMPORT__", icon_import)
        .replace("__INSTALL_APP_BINDING__", install_app_binding)
        .replace("__INSTALL_ICONS__", install_icons)
        .replace("__PACKAGE_NAME__", package_name)
}

pub(super) fn workbench_lite_template_main_rs(package_name: &str, opts: ScaffoldOptions) -> String {
    let install_app_binding = if matches!(opts.icon_pack, IconPack::Radix) {
        "app"
    } else {
        "_app"
    };
    let install_icons = match opts.icon_pack {
        IconPack::Radix => {
            r#"    fret_icons_radix::app::install(app);
"#
        }
        IconPack::Lucide | IconPack::None => "",
    };

    const TEMPLATE: &str = r#"use std::sync::Arc;

use fret::app::LocalState;
use fret::app::prelude::*;
use fret::style::{ColorRef, Radius, Space, Theme, ThemeSnapshot};

mod act {
    fret::actions!([
        OpenSettings = "__PACKAGE_NAME__.workbench.settings.open.v1",
        SaveSettings = "__PACKAGE_NAME__.workbench.settings.save.v1",
        CancelSettings = "__PACKAGE_NAME__.workbench.settings.cancel.v1",
        SubmitJob = "__PACKAGE_NAME__.workbench.job.submit.v1",
        ResetJob = "__PACKAGE_NAME__.workbench.job.reset.v1"
    ]);
}

const TEST_ID_ROOT: &str = "workbench_lite.root";
const TEST_ID_SIDEBAR: &str = "workbench_lite.sidebar";
const TEST_ID_COMMAND: &str = "workbench_lite.command";
const TEST_ID_SETTINGS: &str = "workbench_lite.settings";
const TEST_ID_DIALOG: &str = "workbench_lite.settings.dialog";
const TEST_ID_PROJECT_LABEL: &str = "workbench_lite.settings.project_label";
const TEST_ID_OWNER_LABEL: &str = "workbench_lite.settings.owner_label";
const TEST_ID_PROJECT_INPUT: &str = "workbench_lite.settings.project";
const TEST_ID_OWNER_INPUT: &str = "workbench_lite.settings.owner";
const TEST_ID_SAVE_SETTINGS: &str = "workbench_lite.settings.save";
const TEST_ID_CANCEL_SETTINGS: &str = "workbench_lite.settings.cancel";
const TEST_ID_CLOSE_SETTINGS: &str = "workbench_lite.settings.close";
const TEST_ID_PROMPT_INPUT: &str = "workbench_lite.prompt";
const TEST_ID_SUBMIT: &str = "workbench_lite.submit";
const TEST_ID_STATUS: &str = "workbench_lite.status";
const TEST_ID_CONTENT: &str = "workbench_lite.content";

#[derive(Clone)]
struct WorkItem {
    id: u64,
    title: Arc<str>,
    state: Arc<str>,
}

struct WorkbenchLocals {
    settings_open: LocalState<bool>,
    project_name: LocalState<String>,
    owner_name: LocalState<String>,
    draft_project_name: LocalState<String>,
    draft_owner_name: LocalState<String>,
    prompt: LocalState<String>,
    submitted: LocalState<u32>,
    jobs: LocalState<Vec<WorkItem>>,
}

impl WorkbenchLocals {
    fn new(cx: &mut AppUi<'_, '_>) -> Self {
        Self {
            settings_open: cx.state().local_init(|| false),
            project_name: cx.state().local_init(|| "Fret Studio".to_string()),
            owner_name: cx.state().local_init(|| "UI Platform".to_string()),
            draft_project_name: cx.state().local_init(|| "Fret Studio".to_string()),
            draft_owner_name: cx.state().local_init(|| "UI Platform".to_string()),
            prompt: cx.state().local::<String>(),
            submitted: cx.state().local_init(|| 0u32),
            jobs: cx.state().local_init(|| vec![
                WorkItem {
                    id: 1,
                    title: Arc::from("Audit command routing"),
                    state: Arc::from("Ready"),
                },
                WorkItem {
                    id: 2,
                    title: Arc::from("Review settings surface"),
                    state: Arc::from("Draft"),
                },
                WorkItem {
                    id: 3,
                    title: Arc::from("Prepare release notes"),
                    state: Arc::from("Queued"),
                },
            ]),
        }
    }

    fn bind_actions(&self, cx: &mut AppUi<'_, '_>) {
        cx.actions()
            .locals_with((
                &self.settings_open,
                &self.project_name,
                &self.owner_name,
                &self.draft_project_name,
                &self.draft_owner_name,
            ))
            .on::<act::OpenSettings>(
                |tx, (settings_open, project_name, owner_name, draft_project_name, draft_owner_name)| {
                    tx.set(&draft_project_name, tx.value(&project_name))
                        && tx.set(&draft_owner_name, tx.value(&owner_name))
                        && tx.set(&settings_open, true)
                },
            );

        cx.actions()
            .locals_with((
                &self.settings_open,
                &self.project_name,
                &self.owner_name,
                &self.draft_project_name,
                &self.draft_owner_name,
            ))
            .on::<act::SaveSettings>(
                |tx, (settings_open, project_name, owner_name, draft_project_name, draft_owner_name)| {
                let project = tx.value(&draft_project_name).trim().to_string();
                let owner = tx.value(&draft_owner_name).trim().to_string();
                if project.is_empty() || owner.is_empty() {
                    return false;
                }
                    tx.set(&project_name, project.clone())
                        && tx.set(&owner_name, owner.clone())
                        && tx.set(&draft_project_name, project)
                        && tx.set(&draft_owner_name, owner)
                        && tx.set(&settings_open, false)
                },
            );

        cx.actions()
            .local(&self.settings_open)
            .set::<act::CancelSettings>(false);

        cx.actions()
            .locals_with((&self.prompt, &self.submitted, &self.jobs))
            .on::<act::SubmitJob>(|tx, (prompt, submitted, jobs)| {
                let text = tx.value(&prompt).trim().to_string();
                if text.is_empty() {
                    return false;
                }

                let submitted_count = tx.value(&submitted).saturating_add(1);
                let ok = tx.set(&submitted, submitted_count);
                let ok = tx.update(&jobs, |items| {
                    items.insert(
                        0,
                        WorkItem {
                            id: 100 + submitted_count as u64,
                            title: Arc::from(text),
                            state: Arc::from("Submitted"),
                        },
                    );
                }) && ok;
                tx.set(&prompt, String::new()) && ok
            });

        cx.actions()
            .locals_with((&self.prompt, &self.submitted))
            .on::<act::ResetJob>(|tx, (prompt, submitted)| {
                tx.set(&prompt, String::new()) && tx.set(&submitted, 0)
            });
    }
}

struct WorkbenchView;

impl View for WorkbenchView {
    fn init(_app: &mut App, _window: WindowId) -> Self {
        Self
    }

    fn render(&mut self, cx: &mut AppUi<'_, '_>) -> Ui {
        let theme = Theme::global(cx.app()).snapshot();
        let locals = WorkbenchLocals::new(cx);
        locals.bind_actions(cx);

        let project_name = locals.project_name.layout_value(cx);
        let owner_name = locals.owner_name.layout_value(cx);
        let prompt = locals.prompt.layout_value(cx);
        let submitted = locals.submitted.layout_value(cx);
        let jobs = locals.jobs.layout_value(cx);
        let can_submit = !prompt.trim().is_empty();

        let sidebar = sidebar_panel(&theme, &project_name, &owner_name);
        let item_count = jobs.len();
        let content = content_panel(&theme, &locals, jobs, submitted, can_submit);
        let settings = settings_dialog(cx, &locals);

        let shell = ui::h_flex(|cx| {
            ui::children![
                cx;
                sidebar,
                ui::v_flex(|cx| {
                    ui::children![cx; content, status_bar(&theme, submitted, item_count)]
                })
                .gap(Space::N3)
                .flex_1()
                .min_w_0()
                .h_full(),
                settings,
            ]
        })
        .gap(Space::N4)
        .w_full()
        .h_full();

        ui::single(
            cx,
            ui::container(|cx| ui::single(cx, shell))
                .bg(ColorRef::Color(theme.color_token("muted")))
                .p(Space::N4)
                .w_full()
                .h_full()
                .test_id(TEST_ID_ROOT),
        )
    }
}

fn sidebar_panel(theme: &ThemeSnapshot, project_name: &str, owner_name: &str) -> impl UiChild {
    let project_name = project_name.to_string();
    let owner_name = owner_name.to_string();
    let muted = theme.color_token("muted-foreground");

    ui::v_flex(move |cx| {
        ui::children![
            cx;
            ui::v_flex(move |cx| {
                ui::children![
                    cx;
                    ui::text(project_name.clone())
                        .font_semibold()
                        .test_id(TEST_ID_PROJECT_LABEL),
                    ui::text(owner_name.clone())
                        .text_sm()
                        .text_color(ColorRef::Color(muted))
                        .test_id(TEST_ID_OWNER_LABEL),
                ]
            })
            .gap(Space::N1),
            shadcn::Separator::new(),
            shadcn::Button::new("Command palette")
                .variant(shadcn::ButtonVariant::Outline)
                .action("app.command_palette")
                .test_id(TEST_ID_COMMAND),
            shadcn::Button::new("Settings")
                .variant(shadcn::ButtonVariant::Ghost)
                .action(act::OpenSettings)
                .test_id(TEST_ID_SETTINGS),
            ui::v_flex(|cx| {
                ui::children![
                    cx;
                    shadcn::Badge::new("Workbench")
                        .variant(shadcn::BadgeVariant::Secondary),
                    shadcn::Badge::new("Public app facade")
                        .variant(shadcn::BadgeVariant::Outline),
                ]
            })
            .gap(Space::N2),
        ]
    })
    .gap(Space::N4)
    .w_px(Px(220.0))
    .h_full()
    .rounded(Radius::Lg)
    .border_1()
    .border_color(ColorRef::Color(theme.color_token("border")))
    .bg(ColorRef::Color(theme.color_token("background")))
    .p(Space::N4)
    .test_id(TEST_ID_SIDEBAR)
}

fn content_panel(
    theme: &ThemeSnapshot,
    locals: &WorkbenchLocals,
    jobs: Vec<WorkItem>,
    submitted: u32,
    can_submit: bool,
) -> impl UiChild {
    let muted = theme.color_token("muted-foreground");
    let rows_theme = theme.clone();
    let item_count = jobs.len();

    let prompt_input = shadcn::Input::new(&locals.prompt)
        .a11y_label("Workbench prompt")
        .placeholder("Describe the next job")
        .submit_action(act::SubmitJob)
        .test_id(TEST_ID_PROMPT_INPUT);

    let submit_row = ui::h_flex(move |cx| {
        ui::children![
            cx;
            prompt_input,
            shadcn::Button::new("Submit")
                .action(act::SubmitJob)
                .disabled(!can_submit)
                .test_id(TEST_ID_SUBMIT),
            shadcn::Button::new("Reset")
                .variant(shadcn::ButtonVariant::Outline)
                .action(act::ResetJob),
        ]
    })
    .gap(Space::N2)
    .items_center()
    .w_full();

    let rows = ui::v_flex(move |cx| {
        let rows_theme = rows_theme.clone();
        ui::for_each_keyed(cx, jobs.iter(), |job| job.id, move |job| {
            work_item_row(rows_theme.clone(), job.clone())
        })
    })
    .gap(Space::N2)
    .w_full();

    let summary = if submitted == 0 {
        "No simulated submissions yet.".to_string()
    } else {
        format!("{submitted} simulated submission{}.", if submitted == 1 { "" } else { "s" })
    };

    let body = ui::v_flex(move |cx| {
        ui::children![
            cx;
            ui::h_flex(move |cx| {
                ui::children![
                    cx;
                    ui::v_flex(move |cx| {
                        ui::children![
                            cx;
                            shadcn::card_title("Operations queue"),
                            ui::text(summary)
                                .text_sm()
                                .text_color(ColorRef::Color(muted)),
                        ]
                    })
                    .gap(Space::N1)
                    .flex_1()
                    .min_w_0(),
                    shadcn::Badge::new(format!("{} items", item_count))
                        .variant(shadcn::BadgeVariant::Secondary),
                ]
            })
            .gap(Space::N3)
            .items_center()
            .justify_between()
            .w_full(),
            submit_row,
            rows,
        ]
    })
    .gap(Space::N4)
    .w_full();

    shadcn::card(move |cx| {
        ui::children![
            cx;
            shadcn::card_header(|cx| {
                ui::children![
                    cx;
                    shadcn::card_title("Workbench Lite"),
                    shadcn::card_description(
                        "A second-hour app slice with commands, settings, content, and status.",
                    ),
                ]
            }),
            shadcn::card_content(|cx| ui::single(cx, body)),
        ]
    })
    .ui()
    .w_full()
    .h_full()
    .test_id(TEST_ID_CONTENT)
}

fn work_item_row(theme: ThemeSnapshot, job: WorkItem) -> impl UiChild {
    let title = job.title.clone();
    let state = job.state.clone();
    let border = theme.color_token("border");
    let background = theme.color_token("background");

    ui::h_flex(move |cx| {
        ui::children![
            cx;
            ui::text(title.clone())
                .text_sm()
                .flex_1()
                .min_w_0(),
            shadcn::Badge::new(state.clone())
                .variant(shadcn::BadgeVariant::Outline),
        ]
    })
    .gap(Space::N2)
    .items_center()
    .w_full()
    .rounded(Radius::Md)
    .border_1()
    .border_color(ColorRef::Color(border))
    .bg(ColorRef::Color(background))
    .p(Space::N3)
}

fn status_bar(theme: &ThemeSnapshot, submitted: u32, item_count: usize) -> impl UiChild {
    let muted = theme.color_token("muted-foreground");
    let border = theme.color_token("border");
    let background = theme.color_token("background");
    let summary = format!("{item_count} queued / {submitted} submitted");

    ui::h_flex(move |cx| {
        ui::children![
            cx;
            ui::text("Ready")
                .text_sm()
                .text_color(ColorRef::Color(muted)),
            ui::text(summary.clone())
                .text_sm()
                .text_color(ColorRef::Color(muted)),
        ]
    })
    .gap(Space::N3)
    .items_center()
    .justify_between()
    .w_full()
    .rounded(Radius::Md)
    .border_1()
    .border_color(ColorRef::Color(border))
    .bg(ColorRef::Color(background))
    .px_3()
    .py_2()
    .test_id(TEST_ID_STATUS)
}

fn settings_dialog(cx: &mut AppUi<'_, '_>, locals: &WorkbenchLocals) -> impl UiChild + use<> {
    let open_for_cancel = locals.settings_open.clone();
    let draft_project_name = locals.draft_project_name.clone();
    let draft_owner_name = locals.draft_owner_name.clone();

    shadcn::Dialog::new(&locals.settings_open).into_element_in(
        cx,
        move |cx| {
            shadcn::Button::new("Settings")
                .variant(shadcn::ButtonVariant::Ghost)
                .action(act::OpenSettings)
                .into_element(cx)
        },
        move |cx| {
            let fields = ui::v_flex(|cx| {
                ui::children![
                    cx;
                    ui::v_flex(|cx| {
                        ui::children![
                            cx;
                            shadcn::Label::new("Project"),
                            shadcn::Input::new(&draft_project_name)
                                .a11y_label("Project")
                                .placeholder("Project name")
                                .test_id(TEST_ID_PROJECT_INPUT),
                        ]
                    })
                    .gap(Space::N1),
                    ui::v_flex(|cx| {
                        ui::children![
                            cx;
                            shadcn::Label::new("Owner"),
                            shadcn::Input::new(&draft_owner_name)
                                .a11y_label("Owner")
                                .placeholder("Team or person")
                                .submit_action(act::SaveSettings)
                                .test_id(TEST_ID_OWNER_INPUT),
                        ]
                    })
                    .gap(Space::N1),
                ]
            })
            .gap(Space::N3)
            .w_full();

            shadcn::DialogContent::new([
                shadcn::DialogHeader::new([
                    shadcn::DialogTitle::new("Workbench settings").into_element_in(cx),
                    shadcn::DialogDescription::new("Edit the visible project label and owner.")
                        .into_element_in(cx),
                ])
                .into_element_in(cx),
                fields.into_element(cx),
                shadcn::DialogFooter::new([
                    shadcn::Button::new("Cancel")
                        .variant(shadcn::ButtonVariant::Outline)
                        .action(act::CancelSettings)
                        .test_id(TEST_ID_CANCEL_SETTINGS)
                        .into_element_in(cx),
                    shadcn::Button::new("Save")
                        .action(act::SaveSettings)
                        .test_id(TEST_ID_SAVE_SETTINGS)
                        .into_element_in(cx),
                ])
                .into_element_in(cx),
                shadcn::DialogClose::new(open_for_cancel)
                    .into_element_in(cx)
                    .test_id(TEST_ID_CLOSE_SETTINGS),
            ])
            .show_close_button(false)
            .into_element_in(cx)
            .test_id(TEST_ID_DIALOG)
        },
    )
}

fn install_app(__INSTALL_APP_BINDING__: &mut App) {
__INSTALL_ICONS__
    // Register app-owned globals, commands, services, etc.
}

fn main() -> anyhow::Result<()> {
    FretApp::new("__PACKAGE_NAME__")
        .window("__PACKAGE_NAME__", (980.0, 620.0))
        .setup(install_app)
        .view::<WorkbenchView>()?
        .run()
        .map_err(anyhow::Error::from)
}
"#;

    TEMPLATE
        .replace("__INSTALL_APP_BINDING__", install_app_binding)
        .replace("__INSTALL_ICONS__", install_icons)
        .replace("__PACKAGE_NAME__", package_name)
}

pub(super) fn mutation_workbench_template_main_rs(
    package_name: &str,
    opts: ScaffoldOptions,
) -> String {
    let install_app_binding = if matches!(opts.icon_pack, IconPack::Radix) {
        "app"
    } else {
        "_app"
    };
    let install_icons = match opts.icon_pack {
        IconPack::Radix => {
            r#"    fret_icons_radix::app::install(app);
"#
        }
        IconPack::Lucide | IconPack::None => "",
    };

    const TEMPLATE: &str = r#"use std::future::Future;
use std::pin::Pin;
use std::sync::{
    Arc, Mutex,
    atomic::{AtomicBool, Ordering},
};
use std::time::Duration;

use fret::app::LocalState;
use fret::app::prelude::*;
use fret::mutation::{
    CancellationToken, FutureSpawner, FutureSpawnerHandle, MutationError, MutationHandle,
    MutationPolicy, MutationState,
};
use fret::query::{QueryError, QueryKey, QueryPolicy, QueryState};
use fret::style::{ColorRef, Radius, Space, Theme, ThemeSnapshot};
use fret::{FretApp, shadcn};

mod act {
    fret::actions!([
        SavePreset = "__PACKAGE_NAME__.mutation_workbench.save_preset.v1",
        RetrySave = "__PACKAGE_NAME__.mutation_workbench.retry_save.v1",
        FailNextSave = "__PACKAGE_NAME__.mutation_workbench.fail_next_save.v1",
        ClearCatalog = "__PACKAGE_NAME__.mutation_workbench.clear_catalog.v1"
    ]);
}

const PRESET_QUERY_NS: &str = "__PACKAGE_NAME__.mutation_workbench.presets.v1";
const EFFECT_APPLY_COMPLETION: u64 = 0xAFA0_3001;
const EFFECT_INVALIDATE_QUERY: u64 = 0xAFA0_3002;
const EFFECT_SUCCESS_TOAST: u64 = 0xAFA0_3003;
const EFFECT_ERROR_TOAST: u64 = 0xAFA0_3004;

const TEST_ID_ROOT: &str = "mutation_workbench.root";
const TEST_ID_NAME: &str = "mutation_workbench.name";
const TEST_ID_ENDPOINT: &str = "mutation_workbench.endpoint";
const TEST_ID_SAVE: &str = "mutation_workbench.save";
const TEST_ID_RETRY: &str = "mutation_workbench.retry";
const TEST_ID_FAIL_NEXT: &str = "mutation_workbench.fail_next";
const TEST_ID_CLEAR: &str = "mutation_workbench.clear";
const TEST_ID_MUTATION_STATUS: &str = "mutation_workbench.mutation.status";
const TEST_ID_QUERY_STATUS: &str = "mutation_workbench.query.status";
const TEST_ID_ERROR: &str = "mutation_workbench.error";
const TEST_ID_LAST_SAVED: &str = "mutation_workbench.last_saved";
const TEST_ID_CATALOG_COUNT: &str = "mutation_workbench.catalog.count";
const TEST_ID_CATALOG_EMPTY: &str = "mutation_workbench.catalog.empty";

#[derive(Debug, Clone)]
struct TokioRuntimeGlobal {
    _rt: Arc<tokio::runtime::Runtime>,
}

#[derive(Clone)]
struct TokioHandleSpawner(tokio::runtime::Handle);

impl FutureSpawner for TokioHandleSpawner {
    fn spawn_send(&self, fut: Pin<Box<dyn Future<Output = ()> + Send + 'static>>) {
        let _ = self.0.spawn(fut);
    }
}

fn install_async_runtime(app: &mut App) {
    let rt = tokio::runtime::Builder::new_multi_thread()
        .enable_time()
        .build()
        .expect("failed to build tokio runtime");
    let rt = Arc::new(rt);
    let spawner: FutureSpawnerHandle = Arc::new(TokioHandleSpawner(rt.handle().clone()));
    app.set_global::<FutureSpawnerHandle>(spawner);
    app.set_global::<TokioRuntimeGlobal>(TokioRuntimeGlobal { _rt: rt });
}

#[derive(Debug, Clone, Default)]
struct PresetCatalog {
    saved: Arc<Mutex<Vec<SavedPreset>>>,
}

#[derive(Debug, Clone)]
struct PresetDraft {
    name: Arc<str>,
    endpoint: Arc<str>,
    force_error_once: Arc<AtomicBool>,
}

#[derive(Debug, Clone)]
struct SavedPreset {
    name: Arc<str>,
    endpoint: Arc<str>,
    summary: Arc<str>,
}

#[derive(Clone)]
struct MutationWorkbenchLocals {
    name: LocalState<String>,
    endpoint: LocalState<String>,
    fail_next: LocalState<bool>,
    note: LocalState<String>,
    last_saved: LocalState<String>,
}

impl MutationWorkbenchLocals {
    fn new(cx: &mut AppUi<'_, '_>) -> Self {
        Self {
            name: cx.state().local_init(|| "Create issue".to_string()),
            endpoint: cx.state().local_init(|| "/api/issues".to_string()),
            fail_next: cx.state().local_init(|| false),
            note: cx
                .state()
                .local_init(|| "Submit a preset to exercise mutation + query refresh.".to_string()),
            last_saved: cx
                .state()
                .local_init(|| "No preset has been saved yet.".to_string()),
        }
    }
}

struct MutationWorkbenchView;

impl View for MutationWorkbenchView {
    fn init(_app: &mut App, _window: WindowId) -> Self {
        Self
    }

    fn render(&mut self, cx: &mut AppUi<'_, '_>) -> Ui {
        let theme = Theme::global(cx.app()).snapshot();
        let catalog = cx
            .app()
            .global::<PresetCatalog>()
            .cloned()
            .unwrap_or_default();
        let locals = MutationWorkbenchLocals::new(cx);

        let catalog_for_query = catalog.clone();
        let presets = cx.data().query_async(
            QueryKey::<Vec<SavedPreset>>::new(PRESET_QUERY_NS, &()),
            QueryPolicy::default(),
            move |_token| {
                let catalog = catalog_for_query.clone();
                async move { load_presets(catalog).await }
            },
        );

        let catalog_for_save = catalog.clone();
        let save = cx.data().mutation_async(MutationPolicy::default(), move |token, draft| {
            let catalog = catalog_for_save.clone();
            save_preset(catalog, token, draft)
        });

        bind_actions(cx, &locals, &save, &catalog);

        let mutation = save.read_layout(cx);
        let query = presets.read_layout(cx);
        let _ = cx.data().invalidate_query_namespace_after_mutation_success(
            EFFECT_INVALIDATE_QUERY,
            &save,
            PRESET_QUERY_NS,
        );
        let _ = cx
            .data()
            .update_locals_after_mutation_completion(EFFECT_APPLY_COMPLETION, &save, {
                let locals = locals.clone();
                move |tx, state| {
                    if let Some(saved) = state.data.as_ref() {
                        tx.set(&locals.last_saved, saved.summary.to_string())
                            || tx.set(&locals.note, format!("Saved \"{}\".", saved.name))
                    } else {
                        let message = state
                            .error
                            .as_ref()
                            .map(ToString::to_string)
                            .unwrap_or_else(|| "Unknown save failure".to_string());
                        tx.set(&locals.note, format!("Save failed: {message}"))
                    }
                }
            });
        emit_completion_feedback(cx, &save, &mutation);

        let name = locals.name.layout_value(cx);
        let endpoint = locals.endpoint.layout_value(cx);
        let note = locals.note.layout_value(cx);
        let last_saved = locals.last_saved.layout_value(cx);
        let fail_next = locals.fail_next.layout_value(cx);
        let can_submit = !name.trim().is_empty() && !endpoint.trim().is_empty() && !mutation.is_running();
        let can_retry = mutation.input.is_some() && !mutation.is_running();

        let content = content_panel(
            &theme,
            &locals,
            &mutation,
            &query,
            note,
            last_saved,
            fail_next,
            can_submit,
            can_retry,
        );

        ui::single(
            cx,
            ui::container(|cx| {
                ui::children![
                    cx;
                    content,
                    shadcn::Toaster::new(),
                ]
            })
                .bg(ColorRef::Color(theme.color_token("muted")))
                .p(Space::N4)
                .w_full()
                .h_full()
                .test_id(TEST_ID_ROOT),
        )
    }
}

fn bind_actions(
    cx: &mut AppUi<'_, '_>,
    locals: &MutationWorkbenchLocals,
    save: &MutationHandle<PresetDraft, SavedPreset>,
    catalog: &PresetCatalog,
) {
    cx.actions().mutation_submit::<act::SavePreset, _, _>(save, {
        let locals = locals.clone();
        move |tx| {
            let name = tx.value_or(&locals.name, String::new());
            let endpoint = tx.value_or(&locals.endpoint, String::new());
            let force_error = tx.value_or(&locals.fail_next, false);
            let _ = tx.set(&locals.fail_next, false);
            let _ = tx.set(&locals.note, "Saving preset...".to_string());
            Some(PresetDraft {
                name: Arc::from(name),
                endpoint: Arc::from(endpoint),
                force_error_once: Arc::new(AtomicBool::new(force_error)),
            })
        }
    });

    cx.actions().mutation_retry_last::<act::RetrySave, _, _>(save, {
        let locals = locals.clone();
        move |tx| tx.set(&locals.note, "Retrying the last preset...".to_string())
    });

    cx.actions()
        .local(&locals.fail_next)
        .set::<act::FailNextSave>(true);

    cx.actions().locals_with((&locals.note, &locals.last_saved)).on::<act::ClearCatalog>({
        let catalog = catalog.clone();
        move |tx, (note, last_saved)| {
            if let Ok(mut saved) = catalog.saved.lock() {
                saved.clear();
            }
            tx.set(&note, "Catalog cleared. Save again to repopulate the query.".to_string())
                || tx.set(&last_saved, "No preset has been saved yet.".to_string())
        }
    });
}

fn content_panel(
    theme: &ThemeSnapshot,
    locals: &MutationWorkbenchLocals,
    mutation: &MutationState<PresetDraft, SavedPreset>,
    query: &QueryState<Vec<SavedPreset>>,
    note: String,
    last_saved: String,
    fail_next: bool,
    can_submit: bool,
    can_retry: bool,
) -> impl UiChild {
    let muted = theme.color_token("muted-foreground");
    let border = theme.color_token("border");
    let background = theme.color_token("background");
    let rows = query
        .data
        .as_ref()
        .map(|rows| rows.as_ref().clone())
        .unwrap_or_default();
    let catalog_count = rows.len();
    let mutation_status_text = format!("Mutation: {}", mutation.status.as_str());
    let query_status_text = format!("Query: {}", query.status.as_str());
    let catalog_count_text = format!("{catalog_count} saved");

    let inputs = ui::v_flex(|cx| {
        ui::children![
            cx;
            ui::v_flex(|cx| {
                ui::children![
                    cx;
                    shadcn::Label::new("Preset name"),
                    shadcn::Input::new(&locals.name)
                        .a11y_label("Preset name")
                        .placeholder("Create issue")
                        .test_id(TEST_ID_NAME),
                ]
            })
            .gap(Space::N1),
            ui::v_flex(|cx| {
                ui::children![
                    cx;
                    shadcn::Label::new("Endpoint"),
                    shadcn::Input::new(&locals.endpoint)
                        .a11y_label("Endpoint")
                        .placeholder("/api/issues")
                        .submit_action(act::SavePreset)
                        .test_id(TEST_ID_ENDPOINT),
                ]
            })
            .gap(Space::N1),
        ]
    })
    .gap(Space::N3)
    .w_full();

    let actions = ui::h_flex(move |cx| {
        ui::children![
            cx;
            shadcn::Button::new("Save preset")
                .action(act::SavePreset)
                .disabled(!can_submit)
                .test_id(TEST_ID_SAVE),
            shadcn::Button::new("Retry")
                .variant(shadcn::ButtonVariant::Outline)
                .action(act::RetrySave)
                .disabled(!can_retry)
                .test_id(TEST_ID_RETRY),
            shadcn::Button::new(if fail_next { "Next save will fail" } else { "Fail next" })
                .variant(shadcn::ButtonVariant::Ghost)
                .action(act::FailNextSave)
                .test_id(TEST_ID_FAIL_NEXT),
            shadcn::Button::new("Clear")
                .variant(shadcn::ButtonVariant::Outline)
                .action(act::ClearCatalog)
                .test_id(TEST_ID_CLEAR),
        ]
    })
    .gap(Space::N2)
    .items_center()
    .w_full();

    let status = ui::v_flex(move |cx| {
        ui::children![
            cx;
            ui::h_flex(|cx| {
                ui::children![
                    cx;
                    status_pill(
                        mutation_status_text.clone(),
                        TEST_ID_MUTATION_STATUS,
                        ColorRef::Color(border),
                        ColorRef::Color(background),
                    ),
                    status_pill(
                        query_status_text.clone(),
                        TEST_ID_QUERY_STATUS,
                        ColorRef::Color(border),
                        ColorRef::Color(background),
                    ),
                    status_pill(
                        catalog_count_text.clone(),
                        TEST_ID_CATALOG_COUNT,
                        ColorRef::Color(border),
                        ColorRef::Color(background),
                    ),
                ]
            })
            .gap(Space::N2)
            .items_center(),
            ui::text(note.clone())
                .text_sm()
                .text_color(ColorRef::Color(muted)),
            ui::text(last_saved.clone())
                .text_sm()
                .test_id(TEST_ID_LAST_SAVED),
            error_text(theme, mutation, query),
        ]
    })
    .gap(Space::N2)
    .w_full();

    let catalog = catalog_panel(theme.clone(), rows);

    shadcn::card(move |cx| {
        ui::children![
            cx;
            shadcn::card_header(|cx| {
                ui::children![
                    cx;
                    shadcn::card_title("Mutation Workbench"),
                    shadcn::card_description(
                        "Submit, retry, toast feedback, and query invalidation through the public AppUi facade.",
                    ),
                ]
            }),
            shadcn::card_content(|cx| {
                ui::children![cx; inputs, actions, status, catalog]
            }),
        ]
    })
    .ui()
    .w_full()
    .max_w(Px(820.0))
}

fn status_pill(
    label: String,
    test_id: &'static str,
    border: ColorRef,
    background: ColorRef,
) -> impl UiChild {
    ui::container(move |cx| {
        ui::single(
            cx,
            ui::text(label.clone())
                .text_sm()
                .font_medium()
                .test_id(test_id),
        )
    })
    .rounded(Radius::Full)
    .border_1()
    .border_color(border)
    .bg(background)
    .px_3()
    .py_1()
}

fn error_text(
    theme: &ThemeSnapshot,
    mutation: &MutationState<PresetDraft, SavedPreset>,
    query: &QueryState<Vec<SavedPreset>>,
) -> impl UiChild {
    let destructive = theme.color_token("destructive");
    let message = mutation
        .error
        .as_ref()
        .map(ToString::to_string)
        .or_else(|| query.error.as_ref().map(ToString::to_string))
        .unwrap_or_else(|| "No errors.".to_string());

    ui::text(message)
        .text_sm()
        .text_color(ColorRef::Color(destructive))
        .test_id(TEST_ID_ERROR)
}

fn catalog_panel(theme: ThemeSnapshot, rows: Vec<SavedPreset>) -> impl UiChild {
    let border = theme.color_token("border");
    let background = theme.color_token("background");

    ui::v_flex(move |cx| {
        if rows.is_empty() {
            return ui::children![
                cx;
                ui::container(|cx| {
                    ui::single(
                        cx,
                        ui::text("No saved presets yet.")
                            .text_sm()
                            .test_id(TEST_ID_CATALOG_EMPTY),
                    )
                })
                .rounded(Radius::Md)
                .border_1()
                .border_color(ColorRef::Color(border))
                .bg(ColorRef::Color(background))
                .p(Space::N3)
                .w_full()
                .into_element(cx)
            ];
        }

        ui::for_each_keyed(cx, rows.iter(), |row| row.summary.clone(), move |row| {
            ui::h_flex(|cx| {
                ui::children![
                    cx;
                    ui::text(row.name.clone()).text_sm().font_medium().flex_1().min_w_0(),
                    ui::text(row.endpoint.clone()).text_sm(),
                ]
            })
            .gap(Space::N2)
            .items_center()
            .w_full()
            .rounded(Radius::Md)
            .border_1()
            .border_color(ColorRef::Color(border))
            .bg(ColorRef::Color(background))
            .p(Space::N3)
        })
    })
    .gap(Space::N2)
    .w_full()
}

fn emit_completion_feedback(
    cx: &mut AppUi<'_, '_>,
    save: &MutationHandle<PresetDraft, SavedPreset>,
    state: &MutationState<PresetDraft, SavedPreset>,
) {
    if state.is_success() && cx.data().take_mutation_success(EFFECT_SUCCESS_TOAST, save) {
        let Some(saved) = state.data.as_ref() else {
            return;
        };
        cx.effects().toast_success(
            "Preset saved",
            shadcn::ToastMessageOptions::new().description(saved.summary.to_string()),
        );
        return;
    }

    if state.is_error() && cx.data().take_mutation_completion(EFFECT_ERROR_TOAST, save) {
        let description = state
            .error
            .as_ref()
            .map(ToString::to_string)
            .unwrap_or_else(|| "Unknown save failure".to_string());
        cx.effects().toast_error(
            "Save failed",
            shadcn::ToastMessageOptions::new().description(description),
        );
    }
}

async fn load_presets(catalog: PresetCatalog) -> Result<Vec<SavedPreset>, QueryError> {
    tokio::time::sleep(Duration::from_millis(60)).await;
    catalog
        .saved
        .lock()
        .map(|saved| saved.clone())
        .map_err(|_| QueryError::transient("catalog lock poisoned"))
}

async fn save_preset(
    catalog: PresetCatalog,
    token: CancellationToken,
    input: Arc<PresetDraft>,
) -> Result<SavedPreset, MutationError> {
    tokio::time::sleep(Duration::from_millis(120)).await;
    if token.is_cancelled() {
        return Err(MutationError::transient("save cancelled"));
    }
    if input.force_error_once.swap(false, Ordering::SeqCst) {
        return Err(MutationError::transient("forced save failure"));
    }

    let name = input.name.trim();
    if name.is_empty() {
        return Err(MutationError::permanent("Preset name is required"));
    }
    let endpoint = input.endpoint.trim();
    if endpoint.is_empty() || !endpoint.starts_with('/') {
        return Err(MutationError::permanent("Endpoint must start with `/`"));
    }

    let saved = SavedPreset {
        name: Arc::from(name),
        endpoint: Arc::from(endpoint),
        summary: Arc::from(format!("{name} -> {endpoint}")),
    };
    let mut rows = catalog
        .saved
        .lock()
        .map_err(|_| MutationError::transient("catalog lock poisoned"))?;
    rows.insert(0, saved.clone());
    Ok(saved)
}

fn install_app(__INSTALL_APP_BINDING__: &mut App) {
__INSTALL_ICONS__
    install_async_runtime(__INSTALL_APP_BINDING__);
    __INSTALL_APP_BINDING__.set_global::<PresetCatalog>(PresetCatalog::default());
}

fn main() -> anyhow::Result<()> {
    FretApp::new("__PACKAGE_NAME__")
        .window("__PACKAGE_NAME__", (920.0, 680.0))
        .setup(install_app)
        .view::<MutationWorkbenchView>()?
        .run()
        .map_err(anyhow::Error::from)
}
"#;

    TEMPLATE
        .replace("__INSTALL_APP_BINDING__", install_app_binding)
        .replace("__INSTALL_ICONS__", install_icons)
        .replace("__PACKAGE_NAME__", package_name)
}

pub(super) fn hello_template_main_rs(package_name: &str, opts: ScaffoldOptions) -> String {
    let install_app_binding = if matches!(opts.icon_pack, IconPack::Radix) {
        "app"
    } else {
        "_app"
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
            r#"    fret_icons_radix::app::install(app);
"#
        }
        IconPack::Lucide | IconPack::None => "",
    };

    format!(
        r#"use fret::app::prelude::*;
use fret::style::Space;

mod act {{
    fret::actions!([Click = "{package_name}.hello.click.v1"]);
}}

struct HelloView;

impl View for HelloView {{
    fn init(_app: &mut App, _window: WindowId) -> Self {{
        Self
    }}

    fn render(&mut self, cx: &mut AppUi<'_, '_>) -> Ui {{
        let click_count_state = cx.state().local::<u32>();
        let click_count_value = click_count_state.layout_value(cx);

        cx.actions().local(&click_count_state).update::<act::Click>(|v| {{
            *v = v.saturating_add(1);
        }});

        ui::single(
            cx,
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
            .justify_center(),
        )
    }}
}}

fn install_app({install_app_binding}: &mut App) {{
__INSTALL_ICONS__
    // Register app-owned globals, commands, services, etc.
}}

fn main() -> anyhow::Result<()> {{
    FretApp::new("{package_name}")
        .window("{package_name}", (560.0, 360.0))
        .setup(install_app)
        .view::<HelloView>()?
        .run()
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
    let install_app_binding = if matches!(opts.icon_pack, IconPack::Radix) {
        "app"
    } else {
        "_app"
    };

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
        r#"    let add_btn = shadcn::Button::new("Add task")
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
            r#"    fret_icons_radix::app::install(app);
"#
        }
        IconPack::Lucide | IconPack::None => "",
    };
    let generated_assets_module = generated_assets_module_decl(opts);
    let builder_prefix = generated_assets_builder_prefix(opts);
    let builder_suffix = generated_assets_builder_suffix(opts);
    let icon_import = lucide_action_icon_import(opts);

    const TEMPLATE: &str = r#"use std::sync::Arc;

use fret::app::LocalState;
use fret::app::prelude::*;
use fret::{
__ICON_IMPORT__
    style::{ColorRef, Radius, Space, Theme, ThemeSnapshot},
};

__GENERATED_ASSET_MODULE__
mod act {
    fret::actions!([
        Add = "__PACKAGE_NAME__.simple_todo.add.v1",
        ClearDone = "__PACKAGE_NAME__.simple_todo.clear_done.v1"
    ]);

    fret::payload_actions!([Toggle(u64) = "__PACKAGE_NAME__.simple_todo.toggle.v1"]);
}

#[derive(Clone)]
struct TodoRow {
    id: u64,
    done: bool,
    text: Arc<str>,
}

struct TodoLocals {
    draft: LocalState<String>,
    next_id: LocalState<u64>,
    todos: LocalState<Vec<TodoRow>>,
}

impl TodoLocals {
    fn new(cx: &mut AppUi<'_, '_>) -> Self {
        Self {
            draft: cx.state().local::<String>(),
            next_id: cx.state().local_init(|| 3u64),
            todos: cx.state().local_init(|| vec![
                    TodoRow {
                        id: 1,
                        done: false,
                        text: Arc::from("Keep the keyed list in LocalState<Vec<_>>"),
                    },
                    TodoRow {
                        id: 2,
                        done: true,
                        text: Arc::from("Use payload actions for row toggles"),
                    },
                ]),
        }
    }

    fn bind_actions(&self, cx: &mut AppUi<'_, '_>) {
        cx.actions()
            .locals_with((&self.draft, &self.next_id, &self.todos))
            .on::<act::Add>(|tx, (draft, next_id, todos)| {
                let text = tx.value(&draft).trim().to_string();
                if text.is_empty() {
                    return false;
                }

                let id = tx.value(&next_id);
                let _ = tx.update(&next_id, |value| *value = value.saturating_add(1));

                if !tx.update(&todos, |rows| {
                    rows.push(TodoRow {
                        id,
                        done: false,
                        text: Arc::from(text),
                    });
                }) {
                    return false;
                }

                tx.set(&draft, String::new())
            });

        cx.actions()
            .locals_with(&self.todos)
            .on::<act::ClearDone>(|tx, todos| {
                tx.update_if(&todos, |rows| {
                    let before = rows.len();
                    rows.retain(|row| !row.done);
                    rows.len() != before
                })
            });

        cx.actions()
            .local(&self.todos)
            .payload_update_if::<act::Toggle>(|rows, id| {
                if let Some(row) = rows.iter_mut().find(|row| row.id == id) {
                    row.done = !row.done;
                    true
                } else {
                    false
                }
            });
    }
}

struct TodoView;

impl View for TodoView {
    fn init(_app: &mut App, _window: WindowId) -> Self {
        Self
    }

    fn render(&mut self, cx: &mut AppUi<'_, '_>) -> Ui {
        let theme = Theme::global(cx.app()).snapshot();
        let theme_for_rows = theme.clone();
        let locals = TodoLocals::new(cx);
        locals.bind_actions(cx);

        let todos = locals.todos.layout_value(cx);
        let draft_value = locals.draft.layout_value(cx);
        let done_count = todos.iter().filter(|row| row.done).count();
        let total_count = todos.len();
        let active_count = total_count.saturating_sub(done_count);
        let add_enabled = !draft_value.trim().is_empty();
        let muted_foreground = theme.color_token("muted-foreground");
        let status_text = if total_count == 0 {
            "Capture the next thing you need to do.".to_string()
        } else if active_count == 0 {
            "Everything is done. Clear completed or add something new.".to_string()
        } else {
            format!(
                "{active_count} task{} left for today.",
                if active_count == 1 { "" } else { "s" }
            )
        };

        let progress = shadcn::Badge::new(format!("{done_count}/{total_count} done"))
            .variant(shadcn::BadgeVariant::Secondary);

        let summary = ui::text(status_text)
            .text_sm()
            .text_color(ColorRef::Color(muted_foreground));

        let title_block = ui::v_flex(|cx| {
            ui::children![
                cx;
                shadcn::card_title("My tasks"),
                summary,
            ]
        })
        .gap(Space::N1)
        .flex_1()
        .min_w_0();

        let clear_done_btn = shadcn::Button::new("Clear done")
            .variant(shadcn::ButtonVariant::Ghost)
            .size(shadcn::ButtonSize::Sm)
            .disabled(done_count == 0)
            .action(act::ClearDone)
            .ui()
            .rounded_md();

__ADD_BTN_DEF__

        let input = shadcn::Input::new(&locals.draft)
            .placeholder("Write a task and press Enter")
            .submit_action(act::Add)
            .ui()
            .flex_1()
            .min_w_0();

        let input_row = ui::h_flex(|cx| ui::children![cx; input, add_btn])
            .gap(Space::N3)
            .items_center()
            .w_full();

        let rows_body = ui::v_flex(|cx| {
            if todos.is_empty() {
                return ui::children![
                    cx;
                    ui::container(|cx| {
                        ui::single(
                            cx,
                            ui::text("No tasks yet. Add one above.")
                                .text_sm()
                                .text_color(ColorRef::Color(
                                    theme_for_rows.color_token("muted-foreground"),
                                )),
                        )
                    })
                    .rounded(Radius::Md)
                    .border_1()
                    .border_color(ColorRef::Color(theme_for_rows.color_token("border")))
                    .bg(ColorRef::Color(theme_for_rows.color_token("muted")))
                    .p(Space::N5)
                    .w_full()
                    .into_element(cx)
                ];
            }

            ui::for_each_keyed(cx, &todos, |row| row.id, |row| {
                let theme = theme_for_rows.clone();
                todo_row(theme, row)
            })
        })
        .gap(Space::N3)
        .w_full()
        .items_stretch();

        let rows = ui::container(|cx| ui::single(cx, rows_body))
            .rounded(Radius::Lg)
            .border_1()
            .border_color(ColorRef::Color(theme.color_token("border")))
            .bg(ColorRef::Color(theme.color_token("background")))
            .p(Space::N3)
            .w_full();

        let content = ui::v_flex(|cx| ui::children![cx; input_row, rows])
            .gap(Space::N4)
            .w_full();

        let footer_summary = ui::h_flex(|cx| {
            ui::children![
                cx;
                progress,
                ui::text(format!("{active_count} left"))
                    .text_sm()
                    .text_color(ColorRef::Color(muted_foreground)),
            ]
        })
        .gap(Space::N2)
        .items_center();

        let footer = ui::h_flex(|cx| ui::children![cx; footer_summary, clear_done_btn])
            .gap(Space::N3)
            .items_center()
            .justify_between()
            .w_full();

        let card = shadcn::card(|cx| {
            ui::children![cx;
                shadcn::card_header(|cx| {
                    ui::children![cx; title_block]
                }),
                shadcn::card_content(|cx| ui::single(cx, content)),
                shadcn::card_footer(|cx| ui::children![cx; footer]),
            ]
        })
        .ui()
        .bg(ColorRef::Color(theme.color_token("background")))
        .rounded(Radius::Lg)
        .border_1()
        .border_color(ColorRef::Color(theme.color_token("border")))
        .shadow_lg()
        .w_full()
        .max_w(Px(620.0))
        ;

        let content = ui::v_flex(|cx| ui::children![cx;
            card,
__PALETTE_BUTTON__
        ])
        .w_full()
        .h_full()
        .justify_center()
        .items_center();

        ui::single(cx, todo_page(theme, content))
    }
}

fn todo_page(
    theme: ThemeSnapshot,
    content: impl UiChild,
) -> impl UiChild {
    ui::container(|cx| ui::single(cx, content))
        .bg(ColorRef::Color(theme.color_token("muted")))
        .p(Space::N6)
    .w_full()
    .h_full()
}

fn todo_row(theme: ThemeSnapshot, row: &TodoRow) -> impl UiChild {
    let checkbox = shadcn::Checkbox::from_checked(row.done)
        .action(act::Toggle)
        .action_payload(row.id)
        .a11y_label(row.text.clone());

    let text = ui::text(row.text.clone())
        .truncate()
        .text_sm()
        .flex_1()
        .min_w_0()
        .text_color(ColorRef::Color(if row.done {
            theme.color_token("muted-foreground")
        } else {
            theme.color_token("foreground")
        }));

    ui::h_flex(|cx| ui::children![cx; checkbox, text])
        .gap(Space::N3)
        .items_center()
        .bg(ColorRef::Color(if row.done {
            theme.color_token("muted")
        } else {
            theme.color_token("background")
        }))
        .border_1()
        .border_color(ColorRef::Color(theme.color_token("border")))
        .rounded(Radius::Md)
        .p(Space::N3)
        .shadow_sm()
        .w_full()
}

fn install_app(__INSTALL_APP_BINDING__: &mut App) {
__INSTALL_ICONS__
    // Register app-owned globals, commands, services, etc.
}

fn main() -> anyhow::Result<()> {
__BUILDER_PREFIX__FretApp::new("__PACKAGE_NAME__")
        .window("__PACKAGE_NAME__", (560.0, 520.0))
        .setup(install_app)
        .view::<TodoView>()?
__BUILDER_SUFFIX__        .run()
        .map_err(anyhow::Error::from)
}
"#;

    TEMPLATE
        .replace("__ADD_BTN_DEF__", add_btn_def)
        .replace("__GENERATED_ASSET_MODULE__", generated_assets_module)
        .replace("__BUILDER_PREFIX__", builder_prefix)
        .replace("__BUILDER_SUFFIX__", builder_suffix)
        .replace("__ICON_IMPORT__", icon_import)
        .replace("__INSTALL_APP_BINDING__", install_app_binding)
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

pub(super) fn generated_assets_stub_rs(package_name: &str, new_bin_name: &str) -> String {
    format!(
        r#"#![allow(dead_code)]

// Scaffolded by `{new_bin_name} new --ui-assets`.
// Regenerate this file after editing `assets/`:
//   fretboard assets rust write --dir assets --out src/generated_assets.rs --app-bundle {package_name} --force
// Ecosystem/package crates can use `Bundle` or `install(app)` on the app setup surface; apps on
// the builder lane can use `mount(builder)?`.

use fret::assets::{{
    self, AssetBundleId, AssetKey, AssetLocator, AssetStartupMode, AssetStartupPlan,
    StaticAssetEntry,
}};

pub fn bundle_id() -> AssetBundleId {{
    AssetBundleId::app("{package_name}")
}}

pub fn locator(key: impl Into<AssetKey>) -> AssetLocator {{
    AssetLocator::bundle(bundle_id(), key)
}}

pub const DEVELOPMENT_SOURCE_DIR: &str = concat!(env!("CARGO_MANIFEST_DIR"), "/assets");

pub const ENTRIES: &[StaticAssetEntry] = &[];

pub fn packaged_startup_plan() -> AssetStartupPlan {{
    AssetStartupPlan::new().packaged_bundle_entries(bundle_id(), ENTRIES.iter().copied())
}}

pub fn preferred_startup_plan() -> AssetStartupPlan {{
    packaged_startup_plan().development_bundle_dir_if_native(bundle_id(), DEVELOPMENT_SOURCE_DIR)
}}

pub const fn preferred_startup_mode() -> AssetStartupMode {{
    AssetStartupMode::preferred()
}}

pub fn register(app: &mut fret::app::App) {{
    assets::register_bundle_entries(app, bundle_id(), ENTRIES.iter().copied());
}}

pub fn install(app: &mut fret::app::App) {{
    register(app);
}}

pub struct Bundle;

impl fret::integration::InstallIntoApp for Bundle {{
    fn install_into_app(self, app: &mut fret::app::App) {{
        register(app);
    }}
}}

pub fn mount<S: 'static>(builder: fret::UiAppBuilder<S>) -> fret::Result<fret::UiAppBuilder<S>> {{
    builder.with_asset_startup(bundle_id(), preferred_startup_mode(), preferred_startup_plan())
}}
"#
    )
}

pub(super) fn todo_template_readme_md(
    package_name: &str,
    opts: ScaffoldOptions,
    new_bin_name: &str,
) -> String {
    let ui_assets_line = if opts.ui_assets {
        format!(
            "- UI assets: enabled (`fret/ui-assets` + `src/generated_assets.rs` + `generated_assets::mount(builder)?`)\n- Portable asset lane: place app-owned files under `assets/`, then regenerate `src/generated_assets.rs` with `fretboard assets rust write --dir assets --out src/generated_assets.rs --app-bundle {package_name} --force`\n- Startup ownership: generated assets now publish `preferred_startup_plan()` / `preferred_startup_mode()`, so `generated_assets::mount(builder)?` applies the file-backed development lane on native while packaged/web/mobile stays on the compiled bundle lane\n- Resolve app-owned files via `generated_assets::locator(\"...\")` or `AssetBundleId::app(\"{package_name}\")`\n- File-backed development escape hatch: record manual app intent with `FretApp::asset_startup(...)` or apply it on the desktop builder with `UiAppBuilder::with_asset_startup(...)` + `AssetStartupPlan::development_dir(...)`\n"
        )
    } else {
        format!(
            "- UI assets: disabled (use `{new_bin_name} new todo --ui-assets` if you need images/SVG caches + a default app asset bundle)\n"
        )
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

Generated by `{new_bin_name} new todo`.

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
- Ladder position: third rung of the default onboarding path (`hello` -> `simple-todo` -> `todo`)
- Product posture: a deletable product baseline first, with selector/query slices kept visible but secondary
- Authoring: view runtime + typed actions + grouped view locals (action-first, v2)
- Hooks: one selector projection + one query-backed focus note
- State: LocalState-first (`draft`, `filter`, `todos`, id counter, query nonce). Prefer explicit `Model<T>` graphs only when shared ownership or cross-view coordination is the point.
- State adapter policy: selector/query helpers live at the recipe or app-facade seam (`fret/state-selector`, `fret/state-query`, or `fret/state`). Keep primitive/base component APIs value-first, route dynamic row/item mutations through typed payload actions, and follow `docs/workstreams/component-ecosystem-state-integration-v1/component-ecosystem-state-integration-v1.md`.
- Default entrypoints: keep one or two trivial local slots inline; when a view owns several related `LocalState<T>` slots, prefer a small `*Locals` bundle with `new(cx)` and optional `bind_actions(&self, cx)`. Inside that bundle, use `cx.actions().locals_with((...)).on::<A>(|tx, (...)| ...)` for grouped LocalState transactions, use `cx.actions().local(&local).set::<A>(...)` / `.update::<A>(...)` / `.toggle_bool::<A>()` for single-local writes, bind keyed-row payloads via `.action_payload(...)`, use `cx.actions().local(&rows_state).payload_update_if::<A>(...)` as the default row-write path, and use `cx.actions().models::<A>(...)` only when coordinating shared `Model<T>` graphs.
- Treat raw `on_action_notify` and lower-level payload helpers as cookbook/reference-only host-side glue.
- Read tracked state values near the top of `render()` before building nested card/layout sections.
- For App-only effects, prefer `cx.actions().transient::<A>(...)` in the handler and consume the transient via `cx.effects().take_transient(...)` in `render()`.

## First cuts if you want a smaller app

- Delete the query-backed focus note first if you do not need async state yet (`tip_nonce`, `tip_key`, `tip_policy`, `RefreshTip`, `tip_handle`, `tip_callout`).
- Delete filters next if your first version only needs one task list (`TodoFilter`, filter actions, filter chips, selector dependency on `filter`).
- If you remove both slices, replace `TodoDerived` with direct reads from `locals.todos` and drop the `state` feature from `Cargo.toml`.

## Next steps

- Edit UI in `src/main.rs`
"#
    )
}

pub(super) fn workbench_lite_template_readme_md(
    package_name: &str,
    opts: ScaffoldOptions,
    new_bin_name: &str,
) -> String {
    let icons_line = match opts.icon_pack {
        IconPack::Lucide => "- Icons: enabled (default Lucide pack)\n",
        IconPack::Radix => "- Icons: Radix (via `fret-icons-radix` dependency)\n",
        IconPack::None => "- Icons: disabled\n",
    };

    format!(
        r#"# {package_name}

Generated by `{new_bin_name} new workbench-lite`.

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

## Diagnostics

From the Fret repository, run the public settings-dialog script against this app:

```bash
cargo run -p fretboard-dev -- diag run tools/diag-scripts/public-app/workbench-lite-settings-dialog.json --launch -- cargo run --manifest-path path/to/{package_name}/Cargo.toml
```

The script covers settings open/close, focus containment, save, cancel, Escape, and focus restore
through stable `workbench_lite.*` selectors.

## Notes

- Theme: shadcn new-york-v4 (Slate / Light)
{icons_line}- Command palette: enabled (Cmd/Ctrl+Shift+P)
- Diagnostics: enabled for scripted native/web public app checks.
- Ladder position: second-hour starter after `hello` -> `simple-todo` -> `todo`.
- Authoring surface: `use fret::app::prelude::*;` plus explicit `fret::style` imports for styling nouns.
- App slices: command palette button, settings dialog, content pane, status bar, and simulated submit flow.
- State: view-owned `LocalState<T>` only. The settings dialog uses committed + draft local state so Cancel/Escape discard edits and Save commits trimmed values. The submit flow is intentionally synchronous so the starter does not require mutation runtime setup.
- Dialog policy stays in the shadcn recipe layer; the template keeps raw runtime and manual assembly imports out of generated app code.
- Stable diagnostics selectors are in `src/main.rs` as `TEST_ID_*` constants.

## Next steps

- Replace the simulated submit flow with a cookbook mutation recipe when you need real async work.
- Split the side bar and content pane into app-local modules once the file stops fitting on one screen.
- Move to an explicit workspace/docking starter only when editor-grade shell ownership is the point.
"#
    )
}

pub(super) fn mutation_workbench_template_readme_md(
    package_name: &str,
    opts: ScaffoldOptions,
    new_bin_name: &str,
) -> String {
    let icons_line = match opts.icon_pack {
        IconPack::Lucide => "- Icons: enabled (default Lucide pack)\n",
        IconPack::Radix => "- Icons: Radix (via `fret-icons-radix` dependency)\n",
        IconPack::None => "- Icons: disabled\n",
    };

    format!(
        r#"# {package_name}

Generated by `{new_bin_name} new mutation-workbench`.

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

## Diagnostics

From the Fret repository, run the public mutation script against this app:

```bash
cargo run -p fretboard-dev -- diag run tools/diag-scripts/public-app/mutation-workbench-flow.json --launch -- cargo run --manifest-path path/to/{package_name}/Cargo.toml
```

The script covers submit, async success, query refresh, forced error, editable input preservation,
retry, and toast feedback through stable `mutation_workbench.*` selectors.

## Notes

- Theme: shadcn new-york-v4 (Slate / Light)
{icons_line}- Command palette: enabled (Cmd/Ctrl+Shift+P)
- Diagnostics: enabled for scripted native/web public app checks.
- Ladder position: async second-hour starter after `workbench-lite`.
- Authoring surface: `use fret::app::prelude::*;` plus explicit `fret::mutation`, `fret::query`, and `fret::style` imports for advanced nouns.
- Mutation path: `cx.actions().mutation_submit(...)` and `cx.actions().mutation_retry_last(...)` keep submit/retry on the public AppUi action facade.
- Feedback path: `cx.data().update_locals_after_mutation_completion(...)` projects mutation completion into view-owned local state, while `cx.effects().toast_success(...)` / `toast_error(...)` keep Sonner feedback effect-only.
- Query path: `cx.data().invalidate_query_namespace_after_mutation_success(...)` refreshes the saved preset list after a successful save.
- Raw runtime policy: generated source intentionally avoids retained runtime trees, raw element erasure, host adapters, model-store plumbing, and framework-internal crates.
- Stable diagnostics selectors are in `src/main.rs` as `TEST_ID_*` constants.

## First cuts if you want a smaller app

- Delete the forced-error button and retry action if your first mutation is fire-and-forget.
- Delete the query list if completion feedback alone is enough.
- Delete the app-owned Tokio runtime setup when your production shell installs a shared executor.

## Next steps

- Replace the in-memory `PresetCatalog` with your app service boundary.
- Keep server payloads data-only and project completion into `LocalState<T>` or queries at the app facade.
"#
    )
}

pub(super) fn empty_template_readme_md(package_name: &str, new_bin_name: &str) -> String {
    format!(
        r#"# {package_name}

Generated by `{new_bin_name} new`.

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

pub(super) fn simple_todo_template_readme_md(
    package_name: &str,
    opts: ScaffoldOptions,
    new_bin_name: &str,
) -> String {
    let ui_assets_line = if opts.ui_assets {
        format!(
            "- UI assets: enabled (`fret/ui-assets` + `src/generated_assets.rs` + `generated_assets::mount(builder)?`)\n- Portable asset lane: place app-owned files under `assets/`, then regenerate `src/generated_assets.rs` with `fretboard assets rust write --dir assets --out src/generated_assets.rs --app-bundle {package_name} --force`\n- Startup ownership: generated assets now publish `preferred_startup_plan()` / `preferred_startup_mode()`, so `generated_assets::mount(builder)?` applies the file-backed development lane on native while packaged/web/mobile stays on the compiled bundle lane\n- Resolve app-owned files via `generated_assets::locator(\"...\")` or `AssetBundleId::app(\"{package_name}\")`\n- File-backed development escape hatch: record manual app intent with `FretApp::asset_startup(...)` or apply it on the desktop builder with `UiAppBuilder::with_asset_startup(...)` + `AssetStartupPlan::development_dir(...)`\n"
        )
    } else {
        format!(
            "- UI assets: disabled (use `{new_bin_name} new simple-todo --ui-assets` if you need images/SVG caches + a default app asset bundle)\n"
        )
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

Generated by `{new_bin_name} new simple-todo`.

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
- Ladder position: second rung of the default onboarding path (`hello` -> `simple-todo` -> `todo`)
- Authoring: view runtime + typed actions + grouped view locals (action-first, v2)
- State adapters: this rung stays LocalState/payload-action only. Upgrade to `todo` or enable `fret/state-selector`, `fret/state-query`, or `fret/state` when selector/query integration is the point; keep base components value-first and put adapters at the recipe or app-facade seam per `docs/workstreams/component-ecosystem-state-integration-v1/component-ecosystem-state-integration-v1.md`.
- Default entrypoints: keep one or two trivial local slots inline; when a view owns several related `LocalState<T>` slots, prefer a small `*Locals` bundle with `new(cx)` and optional `bind_actions(&self, cx)`. Bind per-row payloads via `.action_payload(...)` inside `ui::for_each_keyed(...)`, and handle row writes with `cx.actions().local(&rows_state).payload_update_if::<A>(...)`.
- Keep widget-local `.action(...)` / `.action_payload(...)` / `.listen(...)` for activation-only glue instead of reopening raw `on_activate*` on the default path.
- Treat raw `on_action_notify` as cookbook/reference-only host-side glue.
- For keyed dynamic lists, prefer `LocalState<Vec<_>>` + payload actions when the rows are view-owned; keep explicit `Model<Vec<_>>` only when shared ownership or runtime coordination is the point.
- Read tracked state near the top of `render()` and keep row rendering driven by local snapshots.
## Next steps

- Edit UI in `src/main.rs`
- Use `ui::children![cx; ...]` to build heterogeneous child lists without call-site `.into_element(cx)` noise.
- Use `ui::single(cx, child)` when a render root or wrapper closure only needs to late-land one typed child.
- When rendering dynamic lists, prefer `ui::for_each_keyed(cx, items, |item| id, |item| ...)` to keep identity stable without dropping back to `v_flex_build(...)`.
"#
    )
}

pub(super) fn hello_template_readme_md(
    package_name: &str,
    opts: ScaffoldOptions,
    new_bin_name: &str,
) -> String {
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

Generated by `{new_bin_name} new hello`.

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
- Ladder position: first rung of the default onboarding path (`hello` -> `simple-todo` -> `todo`)
- Authoring: view runtime + typed unit actions (action-first, v1)
- State adapters: intentionally out of scope on this first rung. When selector/query integration becomes useful, keep primitive/base component APIs value-first and add optional recipe or app-facade adapters per `docs/workstreams/component-ecosystem-state-integration-v1/component-ecosystem-state-integration-v1.md`.
- Default entrypoints: start with `cx.actions().local(&local).update::<A>(...)`; if a control only exposes activation glue, prefer widget-local `.action(...)` / `.listen(...)` instead of teaching raw `on_activate*` first.
- Treat raw `on_action_notify` as cookbook/reference-only host-side glue.
- Read local state values near the top of `render()` and keep action handlers on `cx.actions()` when possible.
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

    fn opts_with_ui_assets() -> ScaffoldOptions {
        ScaffoldOptions {
            ui_assets: true,
            ..opts()
        }
    }

    #[test]
    fn todo_template_uses_default_authoring_dialect() {
        let src = todo_template_main_rs("todo-app", opts());
        assert!(src.contains("use fret::app::prelude::*;"));
        assert!(src.contains("icons::{icon, IconId},"));
        assert!(src.contains(
            "style::{ChromeRefinement, ColorRef, LayoutRefinement, Radius, Space, Theme, ThemeSnapshot},"
        ));
        assert!(src.contains("fn init(_app: &mut App, _window: WindowId) -> Self"));
        assert!(src.contains("fn render(&mut self, cx: &mut AppUi<'_, '_>) -> Ui"));
        assert!(!src.contains("cx: &mut UiCx<'_>,"));
        assert!(src.contains("impl UiChild"));
        assert!(src.contains("ui::container("));
        assert!(src.contains("ui::h_flex("));
        assert!(src.contains("ui::children!["));
        assert!(src.contains("ui::for_each_keyed("));
        assert!(!src.contains("ui::container( |"));
        assert!(!src.contains("ui::h_flex( |"));
        assert!(!src.contains("ui::v_flex( |"));
        assert!(!src.contains("ui::raw_text( "));
        assert!(src.contains("impl View for TodoView"));
        assert!(src.contains(".view::<TodoView>()?"));
        assert!(src.contains(".run()"));
        assert!(!src.contains(".run_view::<TodoView>()"));
        assert!(src.contains("fret::actions!(["));
        assert!(src.contains("fret::payload_actions!([Toggle(u64) ="));
        assert!(src.contains("shadcn::card(|cx| {"));
        assert!(src.contains("shadcn::card_header(|cx| {"));
        assert!(src.contains("shadcn::card_content(|cx| ui::single(cx, content))"));
        assert!(src.contains("shadcn::card_title(\"My tasks\")"));
        assert!(src.contains("let summary = ui::text(status_text)"));
        assert!(src.contains("let progress_label = if derived.total == 0 {"));
        assert!(src.contains("let progress_badge = shadcn::Badge::new(progress_label)"));
        assert!(!src.contains("shadcn::Card::build(|cx, out| {"));
        assert!(!src.contains("shadcn::CardHeader::build(|cx, out| {"));
        assert!(!src.contains("shadcn::CardContent::build(|cx, out| {"));
        assert!(src.contains("struct TodoLocals {"));
        assert!(src.contains("fn new(cx: &mut AppUi<'_, '_>) -> Self {"));
        assert!(src.contains("struct TodoView;"));
        assert!(src.contains("let locals = TodoLocals::new(cx);"));
        assert!(src.contains("locals.bind_actions(cx);"));
        assert!(src.contains("draft: cx.state().local::<String>(),"));
        assert!(src.contains("filter: LocalState<Option<Arc<str>>>,"));
        assert!(src.contains("local_init(|| Some(Arc::from(TodoFilter::All.value())))"));
        assert!(src.contains("next_id: cx.state().local_init(|| 3u64),"));
        assert!(src.contains("tip_nonce: cx.state().local_init(|| 0u64),"));
        assert!(src.contains("todos: cx.state().local_init(|| vec!["));
        assert!(src.contains(".locals_with((&self.draft, &self.next_id, &self.todos))"));
        assert!(src.contains(".on::<act::Add>(|tx, (draft, next_id, todos)| {"));
        assert!(src.contains(".locals_with(&self.todos)"));
        assert!(src.contains(".on::<act::ClearDone>(|tx, todos| {"));
        assert!(src.contains("let text = tx.value(&draft).trim().to_string();"));
        assert!(src.contains("let id = tx.value(&next_id);"));
        assert!(!src.contains("tx.value_or_else(&draft, String::new)"));
        assert!(src.contains(".submit_action(act::Add)"));
        assert!(!src.contains(".submit_command(act::Add.into())"));
        assert!(src.contains(".local(&self.tip_nonce)"));
        assert!(src.contains(".update::<act::RefreshTip>(|v| {"));
        assert!(src.contains(".local(&self.todos)"));
        assert!(src.contains(".payload_update_if::<act::Toggle>(|rows, id| {"));
        assert!(src.contains("let chips = shadcn::ToggleGroup::single(&locals.filter)"));
        assert!(src.contains(".deselectable(false)"));
        assert!(src.contains(
            "shadcn::ToggleGroupItem::new(filter.value(), [ui::text(filter.as_label()).into_element_in(cx)])"
        ));
        assert!(!src.contains("filter_chip(cx, TodoFilter::All, filter_value)"));
        assert!(!src.contains("filter_chip("));
        assert!(!src.contains("FilterAll = \""));
        assert!(!src.contains("FilterActive = \""));
        assert!(!src.contains("FilterCompleted = \""));
        assert!(!src.contains(".set::<act::FilterAll>(TodoFilter::All);"));
        assert!(!src.contains(".action(match filter {"));
        assert!(src.contains(".payload_update_if::<act::Toggle>(|rows, id| {"));
        assert!(src.contains("cx.data()"));
        assert!(
            src.contains(".selector_layout((&locals.todos, &locals.filter), |(todos, filter)| {")
        );
        assert!(src.contains("cx.data().query("));
        assert!(src.contains("let tip_state = tip_handle.read_layout(cx);"));
        assert!(src.contains("shadcn::Label::new(\"Focus note\")"));
        assert!(src.contains("shadcn::Button::new(\"Another note\")"));
        assert!(!src.contains("tip_handle.layout(cx).value_or_default()"));
        assert!(src.contains("query::{QueryKey, QueryPolicy},"));
        assert!(src.contains("if tip_state.is_loading()"));
        assert!(src.contains("|| tip_state.is_idle()"));
        assert!(src.contains("} else if tip_state.is_error() {"));
        assert!(!src.contains("selector::{DepsBuilder, LocalDepsBuilderExt as _},"));
        assert!(!src.contains("deps.local_layout_rev(&todos_state);"));
        assert!(!src.contains("deps.local_layout_rev(&filter_state);"));
        assert!(!src.contains("let todos = todos_state.layout_in(cx).value_or_default();"));
        assert!(
            !src.contains("let filter = filter_state.layout_in(cx).value_or(TodoFilter::All);")
        );
        assert!(!src.contains("use fret_query::{QueryKey, QueryPolicy, QueryState, QueryStatus};"));
        assert!(!src.contains("use fret_query::{QueryKey, QueryPolicy, QueryStatus};"));
        assert!(!src.contains("use fret_selector::ui::DepsBuilder;"));
        assert!(!src.contains("clone_model()"));
        assert!(!src.contains("TodoLocals::new(app)"));
        assert!(!src.contains("LocalState::from_model(app.models_mut().insert("));
        assert!(!src.contains("shadcn::Button::new(\"Command palette\")"));
        assert!(!src.contains("deps.model_rev(&deps_todos_model);"));
        assert!(!src.contains("deps.model_rev(&deps_filter_model);"));
        assert!(!src.contains("cx.watch_model(&todos_model).layout().value_or_default();"));
        assert!(!src.contains("cx.watch_model(&filter_model).layout().value_or(TodoFilter::All);"));
        assert!(src.contains("let draft_value = locals.draft.layout_value(cx);"));
        assert!(src.contains(
            "let filter_value = TodoFilter::from_value(locals.filter.layout_value(cx).as_deref());"
        ));
        assert!(src.contains("let filter = TodoFilter::from_value(filter.as_deref());"));
        assert!(src.contains("let tip_nonce_value = locals.tip_nonce.paint_value(cx);"));
        assert!(src.contains("let footer_summary = if derived.total == 0 {"));
        assert!(!src.contains("draft_state.layout(cx).value_or_default()"));
        assert!(!src.contains("filter_state.layout(cx).value_or(TodoFilter::All)"));
        assert!(!src.contains("bind_todo_actions("));
        assert!(src.contains("fn bind_actions(&self, cx: &mut AppUi<'_, '_>) {"));
        assert!(src.contains("fn filter_group_item("));
        assert!(src.contains("ui::single(cx, todo_page(theme, card))"));
        assert!(src.contains("ui::v_flex(|cx| ui::single(cx, content))"));
        assert!(!src.contains("ui::v_flex(|cx| ui::children![cx; content])"));
        assert!(!src.contains("let card = card.into_element(cx);"));
        assert!(!src.contains("todo_page(theme, card).into_element(cx).into()"));
        assert!(src.contains("fn todo_page("));
        assert!(src.contains(") -> impl UiChild {"));
        assert!(!src.contains("fret_cookbook::scaffold::"));
        assert!(!src.contains("centered_page_muted("));
        assert!(!src.contains("centered_page_background("));
        assert!(!src.contains("Model<Vec<TodoItem>>"));
        assert!(!src.contains("Model<bool>"));
        assert!(!src.contains(".models_mut().insert("));
        assert!(!src.contains("decl_style::container_props"));
        assert!(src.contains(".refine_style(ChromeRefinement::default().rounded(Radius::Full))"));
        assert!(src.contains(
            ".refine_layout(LayoutRefinement::default().h_px(Px(28.0)).min_h(Px(28.0)))"
        ));
        assert!(!src.contains("UiIntoElement"));
        assert!(!src.contains("UiHostBoundIntoElement"));
        assert!(!src.contains("UiChildIntoElement"));
        assert!(!src.contains("UiBuilderHostBoundIntoElementExt"));

        let into_element_count = src.matches(".into_element(cx)").count();
        assert!(
            into_element_count <= 18,
            "expected <= 18 explicit `.into_element(cx)` calls, got {into_element_count}"
        );
    }

    #[test]
    fn todo_template_mounts_generated_assets_when_ui_assets_are_enabled() {
        let src = todo_template_main_rs("todo-app", opts_with_ui_assets());
        assert!(src.contains("mod generated_assets;"));
        assert!(src.contains("let builder = FretApp::new(\"todo-app\")"));
        assert!(src.contains("generated_assets::mount(builder)?"));
        assert!(!src.contains(".asset_dir(\"assets\")"));
    }

    #[test]
    fn workbench_lite_template_uses_public_app_facade_only() {
        let src = workbench_lite_template_main_rs("workbench-lite-app", opts());
        assert!(src.contains("use fret::app::prelude::*;"));
        assert!(src.contains("use fret::app::LocalState;"));
        assert!(src.contains("use fret::style::{ColorRef, Radius, Space, Theme, ThemeSnapshot};"));
        assert!(src.contains("struct WorkbenchLocals {"));
        assert!(src.contains("struct WorkbenchView;"));
        assert!(src.contains("impl View for WorkbenchView"));
        assert!(src.contains(".view::<WorkbenchView>()?"));
        assert!(src.contains(".action(\"app.command_palette\")"));
        assert!(src.contains("shadcn::Dialog::new(&locals.settings_open).into_element_in("));
        assert!(src.contains("shadcn::DialogContent::new(["));
        assert!(src.contains("TEST_ID_DIALOG"));
        assert!(src.contains("TEST_ID_CLOSE_SETTINGS"));
        assert!(src.contains("draft_project_name: LocalState<String>"));
        assert!(src.contains("draft_owner_name: LocalState<String>"));
        assert!(src.contains("TEST_ID_PROMPT_INPUT"));
        assert!(src.contains("TEST_ID_SUBMIT"));
        assert!(src.contains("TEST_ID_STATUS"));
        assert!(src.contains("TEST_ID_CONTENT"));
        assert!(src.contains("&self.draft_project_name"));
        assert!(src.contains("&self.draft_owner_name"));
        assert!(src.contains("tx.set(&draft_project_name, tx.value(&project_name))"));
        assert!(src.contains("tx.set(&draft_owner_name, tx.value(&owner_name))"));
        assert!(src.contains("tx.value(&draft_project_name).trim().to_string()"));
        assert!(src.contains("tx.value(&draft_owner_name).trim().to_string()"));
        assert!(src.contains("shadcn::Input::new(&draft_project_name)"));
        assert!(src.contains("shadcn::Input::new(&draft_owner_name)"));
        assert!(src.contains("test_id(TEST_ID_PROJECT_LABEL)"));
        assert!(src.contains("test_id(TEST_ID_OWNER_LABEL)"));
        assert!(src.contains(".locals_with((&self.prompt, &self.submitted, &self.jobs))"));
        assert!(src.contains(".on::<act::SubmitJob>(|tx, (prompt, submitted, jobs)| {"));
        assert!(src.contains("ui::for_each_keyed(cx, jobs.iter(), |job| job.id, move |job| {"));
        assert!(!src.contains(&format!("use {}::", "fret_ui")));
        assert!(!src.contains(&format!("use {}::", "fret_core")));
        assert!(!src.contains(&format!("{}Driver", "Fn")));
        assert!(!src.contains(&format!("{}Tree", "Ui")));
        assert!(!src.contains(&format!("{}Context", "Element")));
        assert!(!src.contains(&format!("fret::{}::prelude::*", "advanced")));
        assert!(!src.contains("fret_cookbook::scaffold::"));
        assert!(!src.contains("fret_mutation"));

        let into_element_count = src.matches(".into_element(cx)").count();
        assert!(
            into_element_count <= 3,
            "expected <= 3 explicit `.into_element(cx)` calls, got {into_element_count}"
        );
    }

    #[test]
    fn workbench_lite_template_cargo_toml_enables_command_palette_without_state() {
        let toml = workbench_lite_template_cargo_toml_repo("workbench-lite-app", opts(), ".");
        assert!(toml.contains("\"command-palette\""));
        assert!(toml.contains("\"diagnostics\""));
        assert!(toml.contains("\"desktop\""));
        assert!(toml.contains("\"shadcn\""));
        assert!(!toml.contains("\"state\""));
        assert!(!toml.contains("fret-query"));
        assert!(!toml.contains("fret-selector"));
        assert!(!toml.contains("fret-mutation"));
    }

    #[test]
    fn workbench_lite_readme_documents_second_hour_position() {
        let readme = workbench_lite_template_readme_md("workbench-lite-app", opts(), "fretboard");
        assert!(readme.contains("Generated by `fretboard new workbench-lite`."));
        assert!(readme.contains("Ladder position: second-hour starter"));
        assert!(readme.contains("command palette button, settings dialog, content pane"));
        assert!(readme.contains("keeps raw runtime and manual assembly imports out"));
        assert!(
            readme.contains("tools/diag-scripts/public-app/workbench-lite-settings-dialog.json")
        );
    }

    #[test]
    fn mutation_workbench_template_uses_public_app_facade_only() {
        let src = mutation_workbench_template_main_rs("mutation-workbench-app", opts());
        assert!(src.contains("use fret::app::prelude::*;"));
        assert!(src.contains("use fret::app::LocalState;"));
        assert!(src.contains("use fret::mutation::{"));
        assert!(src.contains("use fret::query::{QueryError, QueryKey, QueryPolicy, QueryState};"));
        assert!(src.contains("use fret::style::{ColorRef, Radius, Space, Theme, ThemeSnapshot};"));
        assert!(src.contains("struct MutationWorkbenchLocals {"));
        assert!(src.contains("struct MutationWorkbenchView;"));
        assert!(src.contains("impl View for MutationWorkbenchView"));
        assert!(src.contains(".view::<MutationWorkbenchView>()?"));
        assert!(src.contains("cx.data().query_async("));
        assert!(src.contains("cx.data().mutation_async("));
        assert!(src.contains("cx.actions().mutation_submit::<act::SavePreset"));
        assert!(src.contains("cx.actions().mutation_retry_last::<act::RetrySave"));
        assert!(src.contains(".update_locals_after_mutation_completion"));
        assert!(src.contains("cx.data().invalidate_query_namespace_after_mutation_success"));
        assert!(src.contains("cx.effects().toast_success("));
        assert!(src.contains("cx.effects().toast_error("));
        assert!(src.contains("shadcn::Toaster::new()"));
        assert!(src.contains("TEST_ID_ROOT"));
        assert!(src.contains("TEST_ID_NAME"));
        assert!(src.contains("TEST_ID_ENDPOINT"));
        assert!(src.contains("TEST_ID_SAVE"));
        assert!(src.contains("TEST_ID_RETRY"));
        assert!(src.contains("TEST_ID_FAIL_NEXT"));
        assert!(src.contains("TEST_ID_MUTATION_STATUS"));
        assert!(src.contains("TEST_ID_QUERY_STATUS"));
        assert!(src.contains("TEST_ID_ERROR"));
        assert!(src.contains("TEST_ID_LAST_SAVED"));
        assert!(src.contains("TEST_ID_CATALOG_COUNT"));
        assert!(!src.contains(&format!("use {}::", "fret_ui")));
        assert!(!src.contains(&format!("use {}::", "fret_core")));
        assert!(!src.contains(&format!("use {}::", "fret_app")));
        assert!(!src.contains(&format!("{}Element", "Any")));
        assert!(!src.contains(&format!("{}Tree", "Ui")));
        assert!(!src.contains("UiActionHostAdapter"));
        assert!(!src.contains(&format!("fret_runtime::{}Store", "Model")));
        assert!(!src.contains(&format!("fret::{}::prelude::*", "advanced")));
        assert!(!src.contains("handle.submit("));
        assert!(!src.contains("handle.retry_last("));
        assert!(!src.contains("LocalStateTxn"));

        let into_element_count = src.matches(".into_element(cx)").count();
        assert!(
            into_element_count <= 1,
            "expected <= 1 explicit `.into_element(cx)` calls, got {into_element_count}"
        );
    }

    #[test]
    fn mutation_workbench_template_cargo_toml_enables_async_state_features() {
        let toml =
            mutation_workbench_template_cargo_toml_repo("mutation-workbench-app", opts(), ".");
        assert!(toml.contains("\"command-palette\""));
        assert!(toml.contains("\"diagnostics\""));
        assert!(toml.contains("\"desktop\""));
        assert!(toml.contains("\"shadcn\""));
        assert!(toml.contains("\"state-query\""));
        assert!(toml.contains("\"state-mutation\""));
        assert!(toml.contains("tokio = { version = \"1\""));
        assert!(toml.contains("\"rt-multi-thread\""));
        assert!(toml.contains("\"time\""));
    }

    #[test]
    fn mutation_workbench_readme_documents_public_async_gate() {
        let readme =
            mutation_workbench_template_readme_md("mutation-workbench-app", opts(), "fretboard");
        assert!(readme.contains("Generated by `fretboard new mutation-workbench`."));
        assert!(readme.contains("Ladder position: async second-hour starter"));
        assert!(readme.contains("cx.actions().mutation_submit(...)"));
        assert!(readme.contains("cx.actions().mutation_retry_last(...)"));
        assert!(readme.contains("cx.effects().toast_success(...)"));
        assert!(readme.contains("model-store plumbing"));
        assert!(readme.contains("tools/diag-scripts/public-app/mutation-workbench-flow.json"));
    }

    #[test]
    fn hello_template_uses_default_authoring_dialect() {
        let src = hello_template_main_rs("hello-app", opts());
        assert!(src.contains("use fret::app::prelude::*;"));
        assert!(src.contains("use fret::style::Space;"));
        assert!(src.contains("fn init(_app: &mut App, _window: WindowId) -> Self"));
        assert!(src.contains("fn render(&mut self, cx: &mut AppUi<'_, '_>) -> Ui"));
        assert!(src.contains("ui::v_flex("));
        assert!(!src.contains("ui::v_flex( |"));
        assert!(src.contains("impl View for HelloView"));
        assert!(src.contains(".view::<HelloView>()?"));
        assert!(src.contains(".run()"));
        assert!(!src.contains(".run_view::<HelloView>()"));
        assert!(src.contains("let click_count_state = cx.state().local::<u32>();"));
        assert!(src.contains("let click_count_value = click_count_state.layout_value(cx);"));
        assert!(!src.contains("click_count_state.layout(cx).value_or(0)"));
        assert!(src.contains("cx.actions().local(&click_count_state).update::<act::Click>"));
        assert!(!src.contains("cx.on_action_notify_models::<act::Click>"));
        assert!(!src.contains("cx.use_state::<u32>()"));
        assert!(src.contains("ui::single("));
        assert!(!src.contains("decl_style::container_props"));
    }

    #[test]
    fn simple_todo_template_has_low_adapter_noise_and_no_query_selector() {
        let src = simple_todo_template_main_rs("simple-todo-app", opts());
        assert!(src.contains("use fret::app::prelude::*;"));
        assert!(src.contains("icons::{icon, IconId},"));
        assert!(src.contains("style::{ColorRef, Radius, Space, Theme, ThemeSnapshot},"));
        assert!(src.contains("fn init(_app: &mut App, _window: WindowId) -> Self"));
        assert!(src.contains("fn render(&mut self, cx: &mut AppUi<'_, '_>) -> Ui"));
        assert!(src.contains("let theme = Theme::global(cx.app()).snapshot();"));
        assert!(!src.contains("Theme::global(&*cx.app).snapshot()"));
        assert!(src.contains("impl UiChild"));
        assert!(src.contains("ui::children!["));
        assert!(src.contains("ui::for_each_keyed("));
        assert!(!src.contains("ui::container( |"));
        assert!(!src.contains("ui::h_flex( |"));
        assert!(!src.contains("ui::v_flex( |"));
        assert!(!src.contains("ui::raw_text( "));
        assert!(src.contains("impl View for TodoView"));
        assert!(src.contains(".view::<TodoView>()?"));
        assert!(src.contains(".run()"));
        assert!(!src.contains(".run_view::<TodoView>()"));
        assert!(src.contains("fret::actions!(["));
        assert!(src.contains("shadcn::card(|cx| {"));
        assert!(src.contains("shadcn::card_header(|cx| {"));
        assert!(src.contains("shadcn::card_content(|cx| ui::single(cx, content))"));
        assert!(src.contains("shadcn::card_title(\"My tasks\")"));
        assert!(src.contains("let summary = ui::text(status_text)"));
        assert!(!src.contains("shadcn::Card::build(|cx, out| {"));
        assert!(!src.contains("shadcn::CardHeader::build(|cx, out| {"));
        assert!(!src.contains("shadcn::CardContent::build(|cx, out| {"));
        assert!(src.contains("struct TodoLocals {"));
        assert!(src.contains("fn new(cx: &mut AppUi<'_, '_>) -> Self {"));
        assert!(src.contains("struct TodoView;"));
        assert!(src.contains("let locals = TodoLocals::new(cx);"));
        assert!(src.contains("locals.bind_actions(cx);"));
        assert!(src.contains("draft: cx.state().local::<String>(),"));
        assert!(src.contains("next_id: cx.state().local_init(|| 3u64),"));
        assert!(src.contains("todos: cx.state().local_init(|| vec!["));
        assert!(src.contains(".locals_with((&self.draft, &self.next_id, &self.todos))"));
        assert!(src.contains(".on::<act::Add>(|tx, (draft, next_id, todos)| {"));
        assert!(src.contains(".locals_with(&self.todos)"));
        assert!(src.contains(".on::<act::ClearDone>(|tx, todos| {"));
        assert!(src.contains("let text = tx.value(&draft).trim().to_string();"));
        assert!(src.contains("let id = tx.value(&next_id);"));
        assert!(!src.contains("tx.value_or_else(&draft, String::new)"));
        assert!(src.contains(".submit_action(act::Add)"));
        assert!(!src.contains(".submit_command(act::Add.into())"));
        assert!(src.contains(".local(&self.todos)"));
        assert!(src.contains(".payload_update_if::<act::Toggle>(|rows, id| {"));
        assert!(src.contains("fret::payload_actions!([Toggle(u64) ="));
        assert!(src.contains("let todos = locals.todos.layout_value(cx);"));
        assert!(src.contains("let draft_value = locals.draft.layout_value(cx);"));
        assert!(!src.contains("todos_state.layout(cx).value_or_default()"));
        assert!(!src.contains("draft_state.layout(cx).value_or_default()"));
        assert!(
            !src.contains("bind_todo_actions(cx, &draft_state, &next_id_state, &todos_state);")
        );
        assert!(src.contains("fn bind_actions(&self, cx: &mut AppUi<'_, '_>) {"));
        assert!(src.contains("ui::single(cx, todo_page(theme, content))"));
        assert!(!src.contains("let content = content.into_element(cx);"));
        assert!(!src.contains("todo_page(theme, content).into_element(cx).into()"));
        assert!(src.contains("fn todo_page("));
        assert!(!src.contains("cx: &mut UiCx<'_>,"));
        assert!(src.contains(") -> impl UiChild {"));
        assert!(!src.contains("fret_cookbook::scaffold::"));
        assert!(!src.contains("centered_page_muted("));
        assert!(!src.contains("centered_page_background("));
        assert!(src.contains("shadcn::Input::new(&locals.draft)"));
        assert!(src.contains("shadcn::Checkbox::from_checked(row.done)"));
        assert!(!src.contains("TodoLocals::new(app)"));
        assert!(!src.contains("LocalState::from_model(app.models_mut().insert("));
        assert!(!src.contains("Model<Vec<TodoItem>>"));
        assert!(!src.contains("Model<bool>"));
        assert!(!src.contains("fret_query"));
        assert!(!src.contains("fret_selector"));
        assert!(!src.contains(".refine_style("));
        assert!(!src.contains(".refine_layout("));
        assert!(!src.contains("UiIntoElement"));
        assert!(!src.contains("UiHostBoundIntoElement"));
        assert!(!src.contains("UiChildIntoElement"));
        assert!(!src.contains("UiBuilderHostBoundIntoElementExt"));

        let into_element_count = src.matches(".into_element(cx)").count();
        assert!(
            into_element_count <= 10,
            "expected <= 10 explicit `.into_element(cx)` calls, got {into_element_count}"
        );
    }

    #[test]
    fn simple_todo_template_mounts_generated_assets_when_ui_assets_are_enabled() {
        let src = simple_todo_template_main_rs("simple-todo-app", opts_with_ui_assets());
        assert!(src.contains("mod generated_assets;"));
        assert!(src.contains("let builder = FretApp::new(\"simple-todo-app\")"));
        assert!(src.contains("generated_assets::mount(builder)?"));
        assert!(!src.contains(".asset_dir(\"assets\")"));
    }

    #[test]
    fn simple_todo_template_cargo_toml_has_no_query_selector_deps() {
        let toml = simple_todo_template_cargo_toml_repo("simple-todo-app", opts(), ".");
        assert!(!toml.contains("fret-query"));
        assert!(!toml.contains("fret-selector"));
    }

    #[test]
    fn todo_template_cargo_toml_has_no_query_selector_deps() {
        let toml = todo_template_cargo_toml_repo("todo-app", opts(), ".");
        assert!(!toml.contains("fret-query"));
        assert!(!toml.contains("fret-selector"));
    }

    #[test]
    fn public_template_cargo_toml_uses_versioned_deps() {
        let hello = hello_template_cargo_toml_public("hello-app", opts(), "0.1.0");
        assert!(hello.contains("fret = { version = \"0.1.0\""));
        assert!(!hello.contains("path = "));

        let todo = todo_template_cargo_toml_public("todo-app", opts(), "0.1.0");
        assert!(todo.contains("fret = { version = \"0.1.0\""));
        assert!(!todo.contains("path = "));
    }

    #[test]
    fn radix_icon_pack_templates_use_explicit_app_install_surface() {
        let mut options = opts();
        options.icon_pack = IconPack::Radix;

        let todo = todo_template_main_rs("todo-app", options);
        assert!(todo.contains("fret_icons_radix::app::install(app);"));
        assert!(!todo.contains("fret_icons_radix::install_app(app);"));
        assert!(!todo.contains("icons::{icon, IconId},"));

        let simple = simple_todo_template_main_rs("simple-todo-app", options);
        assert!(!simple.contains("icons::{icon, IconId},"));
    }

    #[test]
    fn non_lucide_templates_omit_action_icon_imports() {
        for icon_pack in [IconPack::Radix, IconPack::None] {
            let mut options = opts();
            options.icon_pack = icon_pack;

            let todo = todo_template_main_rs("todo-app", options);
            assert!(!todo.contains("icons::{icon, IconId},"));

            let simple = simple_todo_template_main_rs("simple-todo-app", options);
            assert!(!simple.contains("icons::{icon, IconId},"));
        }
    }

    #[test]
    fn template_readmes_capture_authoring_guidance() {
        let hello = hello_template_readme_md("hello-app", opts(), "fretboard");
        assert!(hello.contains("Read local state values near the top of `render()`"));
        assert!(hello.contains("Default entrypoints"));
        assert!(hello.contains("cookbook/reference-only host-side glue"));
        assert!(hello.contains("first rung of the default onboarding path"));
        assert!(hello.contains("State adapters: intentionally out of scope on this first rung"));
        assert!(hello.contains(
            "keep primitive/base component APIs value-first and add optional recipe or app-facade adapters"
        ));
        assert!(hello.contains("`cx.actions().local(&local).update::<A>(...)`"));
        assert!(hello.contains("widget-local `.action(...)` / `.listen(...)`"));
        assert!(!hello.contains("on_action_notify_models"));
        assert!(!hello.contains("use `on_activate*` only for local pressable glue"));
        assert!(!hello.contains("AppUiRawModelExt"));
        assert!(!hello.contains("cx.raw_model::<"));
        assert!(!hello.contains("use_state"));

        let simple = simple_todo_template_readme_md("simple-todo-app", opts(), "fretboard");
        assert!(simple.contains(
            "Use `ui::single(cx, child)` when a render root or wrapper closure only needs to late-land one typed child."
        ));
        assert!(simple.contains("prefer `LocalState<Vec<_>>` + payload actions"));
        assert!(simple.contains("Read tracked state near the top of `render()`"));
        assert!(simple.contains("prefer a small `*Locals` bundle with `new(cx)`"));
        assert!(simple.contains("optional `bind_actions(&self, cx)`"));
        assert!(simple.contains("Bind per-row payloads via `.action_payload(...)`"));
        assert!(simple.contains("`cx.actions().local(&rows_state).payload_update_if::<A>(...)`"));
        assert!(
            simple
                .contains("widget-local `.action(...)` / `.action_payload(...)` / `.listen(...)`")
        );
        assert!(simple.contains("cookbook/reference-only host-side glue"));
        assert!(simple.contains("second rung of the default onboarding path"));
        assert!(simple.contains("State adapters: this rung stays LocalState/payload-action only"));
        assert!(simple.contains(
            "Upgrade to `todo` or enable `fret/state-selector`, `fret/state-query`, or `fret/state`"
        ));
        assert!(simple.contains(
            "keep base components value-first and put adapters at the recipe or app-facade seam"
        ));
        assert!(!simple.contains("on_action_notify_locals"));
        assert!(!simple.contains("`cx.actions().payload::<A>()`"));
        assert!(!simple.contains("keep `on_activate*` for local widget glue only"));
        assert!(!simple.contains("AppUiRawModelExt"));
        assert!(!simple.contains("cx.raw_model::<"));
        assert!(!simple.contains("use_state"));

        let simple_with_assets =
            simple_todo_template_readme_md("simple-todo-app", opts_with_ui_assets(), "fretboard");
        assert!(simple_with_assets.contains("`generated_assets::mount(builder)?`"));
        assert!(
            simple_with_assets.contains("`preferred_startup_plan()` / `preferred_startup_mode()`")
        );
        assert!(simple_with_assets.contains(
            "`generated_assets::mount(builder)?` applies the file-backed development lane"
        ));
        assert!(
            simple_with_assets
                .contains("record manual app intent with `FretApp::asset_startup(...)`")
        );
        assert!(simple_with_assets.contains(
            "apply it on the desktop builder with `UiAppBuilder::with_asset_startup(...)`"
        ));
        assert!(simple_with_assets.contains(
            "`fretboard assets rust write --dir assets --out src/generated_assets.rs --app-bundle simple-todo-app --force`"
        ));
        assert!(simple_with_assets.contains("`AssetBundleId::app(\"simple-todo-app\")`"));

        let todo = todo_template_readme_md("todo-app", opts(), "fretboard");
        assert!(todo.contains("For App-only effects, prefer `cx.actions().transient::<A>(...)`"));
        assert!(todo.contains("cookbook/reference-only host-side glue"));
        assert!(todo.contains("`cx.actions().models::<A>(...)`"));
        assert!(todo.contains("`cx.effects().take_transient(...)`"));
        assert!(todo.contains("State: LocalState-first"));
        assert!(todo.contains(
            "State adapter policy: selector/query helpers live at the recipe or app-facade seam"
        ));
        assert!(todo.contains("route dynamic row/item mutations through typed payload actions"));
        assert!(todo.contains(
            "docs/workstreams/component-ecosystem-state-integration-v1/component-ecosystem-state-integration-v1.md"
        ));
        assert!(todo.contains("third rung of the default onboarding path"));
        assert!(todo.contains("Product posture: a deletable product baseline first"));
        assert!(todo.contains("Delete the query-backed focus note first"));
        assert!(todo.contains("Delete filters next"));
        assert!(todo.contains("replace `TodoDerived` with direct reads from `locals.todos`"));
        assert!(todo.contains("bind keyed-row payloads via `.action_payload(...)`"));
        assert!(todo.contains("`cx.actions().local(&rows_state).payload_update_if::<A>(...)` as the default row-write path"));
        assert!(!todo.contains("`payload_locals::<A>(...)`"));
        assert!(!todo.contains("on_action_notify_locals"));
        assert!(!todo.contains("on_action_notify_transient"));
        assert!(!todo.contains("`cx.actions().payload::<A>()`"));
        assert!(!todo.contains("AppUiRawModelExt"));
        assert!(!todo.contains("cx.raw_model::<"));
        assert!(!todo.contains("use_state"));

        let todo_with_assets =
            todo_template_readme_md("todo-app", opts_with_ui_assets(), "fretboard");
        assert!(todo_with_assets.contains("`generated_assets::mount(builder)?`"));
        assert!(
            todo_with_assets.contains("`preferred_startup_plan()` / `preferred_startup_mode()`")
        );
        assert!(todo_with_assets.contains(
            "`generated_assets::mount(builder)?` applies the file-backed development lane"
        ));
        assert!(
            todo_with_assets
                .contains("record manual app intent with `FretApp::asset_startup(...)`")
        );
        assert!(todo_with_assets.contains(
            "apply it on the desktop builder with `UiAppBuilder::with_asset_startup(...)`"
        ));
        assert!(todo_with_assets.contains(
            "`fretboard assets rust write --dir assets --out src/generated_assets.rs --app-bundle todo-app --force`"
        ));
        assert!(todo_with_assets.contains("`AssetBundleId::app(\"todo-app\")`"));
    }

    #[test]
    fn generated_assets_stub_guides_regeneration_and_mounting() {
        let src = generated_assets_stub_rs("todo-app", "fretboard");
        assert!(src.contains("AssetBundleId::app(\"todo-app\")"));
        assert!(src.contains("pub fn locator(key: impl Into<AssetKey>) -> AssetLocator"));
        assert!(src.contains(
            "pub const DEVELOPMENT_SOURCE_DIR: &str = concat!(env!(\"CARGO_MANIFEST_DIR\"), \"/assets\");"
        ));
        assert!(src.contains("pub fn packaged_startup_plan() -> AssetStartupPlan"));
        assert!(src.contains("pub fn preferred_startup_plan() -> AssetStartupPlan"));
        assert!(src.contains("pub const fn preferred_startup_mode() -> AssetStartupMode"));
        assert!(src.contains(
            "packaged_startup_plan().development_bundle_dir_if_native(bundle_id(), DEVELOPMENT_SOURCE_DIR)"
        ));
        assert!(src.contains("AssetStartupMode::preferred()"));
        assert!(src.contains("pub fn register(app: &mut fret::app::App)"));
        assert!(src.contains("pub fn install(app: &mut fret::app::App)"));
        assert!(src.contains("pub struct Bundle;"));
        assert!(src.contains("impl fret::integration::InstallIntoApp for Bundle"));
        assert!(src.contains(
            "pub fn mount<S: 'static>(builder: fret::UiAppBuilder<S>) -> fret::Result<fret::UiAppBuilder<S>>"
        ));
        assert!(src.contains(
            "builder.with_asset_startup(bundle_id(), preferred_startup_mode(), preferred_startup_plan())"
        ));
        assert!(src.contains("register(app);"));
        assert!(src.contains(
            "fretboard assets rust write --dir assets --out src/generated_assets.rs --app-bundle todo-app --force"
        ));
    }
}
