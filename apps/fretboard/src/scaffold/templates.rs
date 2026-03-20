use super::{IconPack, ScaffoldOptions};

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
__ICON_IMPORT__    query::{QueryKey, QueryPolicy, QueryStatus},
    style::{ColorRef, Radius, Space, Theme, ThemeSnapshot},
};

__GENERATED_ASSET_MODULE__
mod act {
    fret::actions!([
        Add = "__PACKAGE_NAME__.todo.add.v1",
        ClearDone = "__PACKAGE_NAME__.todo.clear_done.v1",
        RefreshTip = "__PACKAGE_NAME__.todo.refresh_tip.v1",
        FilterAll = "__PACKAGE_NAME__.todo.filter_all.v1",
        FilterActive = "__PACKAGE_NAME__.todo.filter_active.v1",
        FilterCompleted = "__PACKAGE_NAME__.todo.filter_completed.v1"
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

struct TodoView;

impl View for TodoView {
    fn init(_app: &mut App, _window: WindowId) -> Self {
        Self
    }

    fn render(&mut self, cx: &mut AppUi<'_, '_>) -> Ui {
        let theme = Theme::global(&*cx.app).snapshot();
        let theme_for_rows = theme.clone();

        let draft_state = cx.state().local::<String>();
        let filter_state = cx.state().local_init(|| TodoFilter::All);
        let next_id_state = cx.state().local_init(|| 3u64);
        let tip_nonce_state = cx.state().local_init(|| 0u64);
        let todos_state = cx.state().local_init(|| {
            vec![
                TodoRow {
                    id: 1,
                    done: false,
                    text: Arc::from("Try the shadcn New York style"),
                },
                TodoRow {
                    id: 2,
                    done: true,
                    text: Arc::from("Validate selector derived state"),
                },
            ]
        });

        bind_todo_actions(
            cx,
            &draft_state,
            &filter_state,
            &next_id_state,
            &tip_nonce_state,
            &todos_state,
        );

        let draft_value = draft_state.layout_value(cx);
        let filter_value = filter_state.layout_value(cx);

        let add_enabled = !draft_value.trim().is_empty();

        let derived: TodoDerived = cx
            .data()
            .selector_layout((&todos_state, &filter_state), |(todos, filter)| {
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

        let tip_nonce_value = tip_nonce_state.paint_value(cx);
        let tip_handle = cx.data().query(tip_key(tip_nonce_value), tip_policy(), move |_token| {
                #[cfg(not(target_arch = "wasm32"))]
                std::thread::sleep(Duration::from_millis(150));

                Ok(TipData {
                    text: Arc::from(format!(
                        "Tip fetched at {:?}",
                        std::time::SystemTime::now()
                    )),
                })
            });

        let tip_state = tip_handle.read_layout(cx);

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

        let active = shadcn::Badge::new(format!("{} active", derived.active))
            .variant(shadcn::BadgeVariant::Outline);

        let tip = shadcn::Badge::new(tip_text.clone())
            .variant(shadcn::BadgeVariant::Outline)
            .ui()
            .text_color(ColorRef::Color(theme.color_token(tip_color_key)));

        let refresh_tip_btn = shadcn::Button::new("Refresh tip")
            .variant(shadcn::ButtonVariant::Secondary)
            .action(act::RefreshTip)
            .ui()
            .rounded_md();

        let stats = ui::h_flex(|cx| ui::children![cx; progress, active, tip, refresh_tip_btn])
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
            .submit_action(act::Add);

        let input_row = ui::h_flex(|cx| ui::children![cx; input, add_btn])
            .gap(Space::N2)
            .items_center()
            .w_full();

        let chip_all = filter_chip(TodoFilter::All, filter_value, act::FilterAll);
        let chip_active = filter_chip(TodoFilter::Active, filter_value, act::FilterActive);
        let chip_completed = filter_chip(
            TodoFilter::Completed,
            filter_value,
            act::FilterCompleted,
        );

        let chips = ui::h_flex(|cx| ui::children![cx;
            chip_all,
            chip_active,
            chip_completed,
        ])
        .gap(Space::N1)
        .items_center();

        let rows = ui::v_flex(|cx| {
            if derived.rows.is_empty() {
                let text = match filter_value {
                    TodoFilter::All => "No tasks yet. Add one above.",
                    TodoFilter::Active => "No active tasks.",
                    TodoFilter::Completed => "No completed tasks.",
                };

                return ui::children![
                    cx;
                    ui::text(text)
                        .text_sm()
                        .text_color(ColorRef::Color(theme_for_rows.color_token("muted-foreground")))
                ];
            }

            ui::for_each_keyed(cx, derived.rows.iter(), |row| row.id, |row| {
                let theme = theme_for_rows.clone();
                todo_row(theme, row)
            })
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

        let card = shadcn::card(|cx| {
            ui::children![cx;
                shadcn::card_header(|cx| {
                    ui::children![cx;
                        shadcn::card_title("Todo"),
                        shadcn::card_description("View runtime + typed actions + selector + query (v1)."),
                    ]
                }),
                shadcn::card_content(|cx| ui::single(cx, content)),
            ]
        })
        .ui()
        .bg(ColorRef::Color(theme.color_token("background")))
        .rounded(Radius::Lg)
        .border_1()
        .border_color(ColorRef::Color(theme.color_token("border")))
        .w_full()
        .max_w(Px(520.0))
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

fn bind_todo_actions(
    cx: &mut AppUi<'_, '_>,
    draft_state: &LocalState<String>,
    filter_state: &LocalState<TodoFilter>,
    next_id_state: &LocalState<u64>,
    tip_nonce_state: &LocalState<u64>,
    todos_state: &LocalState<Vec<TodoRow>>,
) {
    cx.actions()
        .locals_with((draft_state, next_id_state, todos_state))
        .on::<act::Add>(|tx, (draft_state, next_id_state, todos_state)| {
            let text = tx.value(&draft_state).trim().to_string();
            if text.is_empty() {
                return false;
            }

            let id = tx.value(&next_id_state);
            let _ = tx.update(&next_id_state, |v| *v = v.saturating_add(1));

            let item = TodoRow {
                id,
                done: false,
                text: Arc::from(text),
            };

            if !tx.update(&todos_state, |todos| todos.insert(0, item)) {
                return false;
            }

            tx.set(&draft_state, String::new())
        });

    cx.actions()
        .locals_with(todos_state)
        .on::<act::ClearDone>(|tx, todos_state| {
            tx.update_if(&todos_state, |rows| {
                let before = rows.len();
                rows.retain(|row| !row.done);
                rows.len() != before
            })
        });

    cx.actions()
        .local(tip_nonce_state)
        .update::<act::RefreshTip>(|v| {
            *v = v.saturating_add(1);
        });

    cx.actions()
        .local(filter_state)
        .set::<act::FilterAll>(TodoFilter::All);
    cx.actions()
        .local(filter_state)
        .set::<act::FilterActive>(TodoFilter::Active);
    cx.actions()
        .local(filter_state)
        .set::<act::FilterCompleted>(TodoFilter::Completed);

    cx.actions()
        .local(todos_state)
        .payload_update_if::<act::Toggle>(|rows, id| {
            if let Some(row) = rows.iter_mut().find(|row| row.id == id) {
                row.done = !row.done;
                true
            } else {
                false
            }
        });
}

fn filter_chip(
    filter: TodoFilter,
    current: TodoFilter,
    action: impl Into<fret::ActionId>,
) -> impl UiChild {
    let selected = filter == current;
    let action: fret::ActionId = action.into();
    shadcn::Button::new(filter.as_label())
        .variant(if selected {
            shadcn::ButtonVariant::Secondary
        } else {
            shadcn::ButtonVariant::Ghost
        })
        .size(shadcn::ButtonSize::Sm)
        .action(action)
}

fn todo_row(theme: ThemeSnapshot, row: &TodoRowSnapshot) -> impl UiChild {
    let checkbox = shadcn::Checkbox::from_checked(row.done)
        .action(act::Toggle)
        .action_payload(row.id)
        .a11y_label(row.text.clone());

    let text = ui::text(row.text.clone())
        .truncate()
        .text_sm()
        .text_color(ColorRef::Color(if row.done {
            theme.color_token("muted-foreground")
        } else {
            theme.color_token("foreground")
        }));

    ui::h_flex(|cx| ui::children![cx; checkbox, text])
        .gap(Space::N2)
        .items_center()
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

struct TodoView;

impl View for TodoView {
    fn init(_app: &mut App, _window: WindowId) -> Self {
        Self
    }

    fn render(&mut self, cx: &mut AppUi<'_, '_>) -> Ui {
        let theme = Theme::global(&*cx.app).snapshot();
        let theme_for_rows = theme.clone();

        let draft_state = cx.state().local::<String>();
        let next_id_state = cx.state().local_init(|| 3u64);
        let todos_state = cx.state().local_init(|| {
            vec![
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
            ]
        });

        bind_todo_actions(cx, &draft_state, &next_id_state, &todos_state);

        let todos = todos_state.layout_value(cx);
        let draft_value = draft_state.layout_value(cx);
        let done_count = todos.iter().filter(|row| row.done).count();
        let total_count = todos.len();
        let add_enabled = !draft_value.trim().is_empty();

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
            .placeholder("Add a task?")
            .submit_action(act::Add);

        let input_row = ui::h_flex(|cx| ui::children![cx; input, add_btn])
            .gap(Space::N2)
            .items_center()
            .w_full();

        let rows = ui::v_flex(|cx| {
            if todos.is_empty() {
                return ui::children![
                    cx;
                    ui::text("No tasks yet. Add one above.")
                        .text_sm()
                        .text_color(ColorRef::Color(
                            theme_for_rows.color_token("muted-foreground"),
                        ))
                ];
            }

            ui::for_each_keyed(cx, &todos, |row| row.id, |row| {
                let theme = theme_for_rows.clone();
                todo_row(theme, row)
            })
        })
        .gap(Space::N3)
        .w_full();

        let content = ui::v_flex(|cx| ui::children![cx; input_row, rows])
            .gap(Space::N4)
            .w_full();

        let card = shadcn::card(|cx| {
            ui::children![cx;
                shadcn::card_header(|cx| {
                    ui::children![cx;
                        shadcn::card_title("Simple Todo"),
                        shadcn::card_description(
                            "View runtime + typed actions + local-state keyed lists (no selector/query).",
                        ),
                        header_actions,
                    ]
                }),
                shadcn::card_content(|cx| ui::single(cx, content)),
            ]
        })
        .ui()
        .bg(ColorRef::Color(theme.color_token("background")))
        .rounded(Radius::Lg)
        .border_1()
        .border_color(ColorRef::Color(theme.color_token("border")))
        .w_full()
        .max_w(Px(520.0))
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

fn bind_todo_actions(
    cx: &mut AppUi<'_, '_>,
    draft_state: &LocalState<String>,
    next_id_state: &LocalState<u64>,
    todos_state: &LocalState<Vec<TodoRow>>,
) {
    cx.actions()
        .locals_with((draft_state, next_id_state, todos_state))
        .on::<act::Add>(|tx, (draft_state, next_id_state, todos_state)| {
            let text = tx.value(&draft_state).trim().to_string();
            if text.is_empty() {
                return false;
            }

            let id = tx.value(&next_id_state);
            let _ = tx.update(&next_id_state, |value| *value = value.saturating_add(1));

            if !tx.update(&todos_state, |rows| {
                rows.push(TodoRow {
                    id,
                    done: false,
                    text: Arc::from(text),
                });
            }) {
                return false;
            }

            tx.set(&draft_state, String::new())
        });

    cx.actions()
        .locals_with(todos_state)
        .on::<act::ClearDone>(|tx, todos_state| {
            tx.update_if(&todos_state, |rows| {
                let before = rows.len();
                rows.retain(|row| !row.done);
                rows.len() != before
            })
        });

    cx.actions()
        .local(todos_state)
        .payload_update_if::<act::Toggle>(|rows, id| {
            if let Some(row) = rows.iter_mut().find(|row| row.id == id) {
                row.done = !row.done;
                true
            } else {
                false
            }
        });
}

fn todo_row(theme: ThemeSnapshot, row: &TodoRow) -> impl UiChild {
    let checkbox = shadcn::Checkbox::from_checked(row.done)
        .action(act::Toggle)
        .action_payload(row.id)
        .a11y_label(row.text.clone());

    let text = ui::text(row.text.clone())
        .truncate()
        .text_sm()
        .text_color(ColorRef::Color(if row.done {
            theme.color_token("muted-foreground")
        } else {
            theme.color_token("foreground")
        }));

    ui::h_flex(|cx| ui::children![cx; checkbox, text])
        .gap(Space::N2)
        .items_center()
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

pub(super) fn generated_assets_stub_rs(package_name: &str) -> String {
    format!(
        r#"#![allow(dead_code)]

// Scaffolded by `fretboard new --ui-assets`.
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

pub(super) fn todo_template_readme_md(package_name: &str, opts: ScaffoldOptions) -> String {
    let ui_assets_line = if opts.ui_assets {
        format!(
            "- UI assets: enabled (`fret/ui-assets` + `src/generated_assets.rs` + `generated_assets::mount(builder)?`)\n- Portable asset lane: place app-owned files under `assets/`, then regenerate `src/generated_assets.rs` with `fretboard assets rust write --dir assets --out src/generated_assets.rs --app-bundle {package_name} --force`\n- Startup ownership: generated assets now publish `preferred_startup_plan()` / `preferred_startup_mode()`, so debug native uses the file-backed development lane while packaged/web/mobile stays on the compiled bundle lane\n- Resolve app-owned files via `generated_assets::locator(\"...\")` or `AssetBundleId::app(\"{package_name}\")`\n- Lower-level escape hatch: `FretApp::asset_dir(\"assets\")` still exists when you intentionally want manual startup layering\n"
        )
    } else {
        "- UI assets: disabled (use `fretboard new todo --ui-assets` if you need images/SVG caches + a default app asset bundle)\n".to_string()
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
- Ladder position: third rung of the default onboarding path (`hello` -> `simple-todo` -> `todo`)
- Authoring: view runtime + typed actions + local-state slots (action-first, v2)
- Hooks: selector + query (v1)
- State: LocalState-first (`draft`, `filter`, `todos`, id counter, query nonce). Prefer explicit `Model<T>` graphs only when shared ownership or cross-view coordination is the point.
- Default entrypoints: start with `cx.actions().locals_with((...)).on::<A>(|tx, (...)| ...)` for multi-slot `LocalState<T>` transactions, use `cx.actions().local(&local).set::<A>(...)` / `.update::<A>(...)` / `.toggle_bool::<A>()` for single-local writes, bind keyed-row payloads via `.action_payload(...)`, use `cx.actions().local(&rows_state).payload_update_if::<A>(...)` as the default row-write path, and use `cx.actions().models::<A>(...)` only when coordinating shared `Model<T>` graphs.
- Treat raw `on_action_notify` and lower-level payload helpers as cookbook/reference-only host-side glue.
- Read tracked state values near the top of `render()` before building nested card/layout sections.
- For App-only effects, prefer `cx.actions().transient::<A>(...)` in the handler and consume the transient via `cx.effects().take_transient(...)` in `render()`.
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
        format!(
            "- UI assets: enabled (`fret/ui-assets` + `src/generated_assets.rs` + `generated_assets::mount(builder)?`)\n- Portable asset lane: place app-owned files under `assets/`, then regenerate `src/generated_assets.rs` with `fretboard assets rust write --dir assets --out src/generated_assets.rs --app-bundle {package_name} --force`\n- Startup ownership: generated assets now publish `preferred_startup_plan()` / `preferred_startup_mode()`, so debug native uses the file-backed development lane while packaged/web/mobile stays on the compiled bundle lane\n- Resolve app-owned files via `generated_assets::locator(\"...\")` or `AssetBundleId::app(\"{package_name}\")`\n- Lower-level escape hatch: `FretApp::asset_dir(\"assets\")` still exists when you intentionally want manual startup layering\n"
        )
    } else {
        "- UI assets: disabled (use `fretboard new simple-todo --ui-assets` if you need images/SVG caches + a default app asset bundle)\n".to_string()
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
- Ladder position: second rung of the default onboarding path (`hello` -> `simple-todo` -> `todo`)
- Authoring: view runtime + typed actions + local-state keyed lists (action-first, v2)
- Default entrypoints: start with `cx.actions().locals_with((...)).on::<A>(|tx, (...)| ...)` for multi-slot `LocalState<T>` transactions, bind per-row payloads via `.action_payload(...)` inside `ui::for_each_keyed(...)`, and handle row writes with `cx.actions().local(&rows_state).payload_update_if::<A>(...)`.
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
- Ladder position: first rung of the default onboarding path (`hello` -> `simple-todo` -> `todo`)
- Authoring: view runtime + typed unit actions (action-first, v1)
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
        assert!(src.contains("style::{ColorRef, Radius, Space, Theme, ThemeSnapshot},"));
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
        assert!(src.contains("shadcn::card_title(\"Todo\")"));
        assert!(src.contains(
            "shadcn::card_description(\"View runtime + typed actions + selector + query (v1).\")"
        ));
        assert!(!src.contains("shadcn::Card::build(|cx, out| {"));
        assert!(!src.contains("shadcn::CardHeader::build(|cx, out| {"));
        assert!(!src.contains("shadcn::CardContent::build(|cx, out| {"));
        assert!(src.contains(".locals_with((draft_state, next_id_state, todos_state))"));
        assert!(src.contains(".on::<act::Add>(|tx, (draft_state, next_id_state, todos_state)| {"));
        assert!(src.contains(".locals_with(todos_state)"));
        assert!(src.contains(".on::<act::ClearDone>(|tx, todos_state| {"));
        assert!(src.contains("let text = tx.value(&draft_state).trim().to_string();"));
        assert!(src.contains("let id = tx.value(&next_id_state);"));
        assert!(!src.contains("tx.value_or_else(&draft_state, String::new)"));
        assert!(src.contains(".submit_action(act::Add)"));
        assert!(!src.contains(".submit_command(act::Add.into())"));
        assert!(src.contains(".local(tip_nonce_state)"));
        assert!(src.contains(".update::<act::RefreshTip>(|v| {"));
        assert!(src.contains(".local(filter_state)"));
        assert!(src.contains(".set::<act::FilterAll>(TodoFilter::All);"));
        assert!(src.contains(".local(todos_state)"));
        assert!(src.contains(".payload_update_if::<act::Toggle>(|rows, id| {"));
        assert!(src.contains(
            "let chip_all = filter_chip(TodoFilter::All, filter_value, act::FilterAll);"
        ));
        assert!(src.contains(
            "let chip_active = filter_chip(TodoFilter::Active, filter_value, act::FilterActive);"
        ));
        assert!(src.contains("let chip_completed = filter_chip("));
        assert!(!src.contains("filter_chip(cx, TodoFilter::All, filter_value)"));
        assert!(!src.contains(".action(match filter {"));
        assert!(src.contains(".payload_update_if::<act::Toggle>(|rows, id| {"));
        assert!(src.contains("cx.data()"));
        assert!(
            src.contains(".selector_layout((&todos_state, &filter_state), |(todos, filter)| {")
        );
        assert!(src.contains("cx.data().query("));
        assert!(src.contains("let tip_state = tip_handle.read_layout(cx);"));
        assert!(!src.contains("tip_handle.layout(cx).value_or_default()"));
        assert!(src.contains("query::{QueryKey, QueryPolicy, QueryStatus},"));
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
        assert!(!src.contains("deps.model_rev(&deps_todos_model);"));
        assert!(!src.contains("deps.model_rev(&deps_filter_model);"));
        assert!(!src.contains("cx.watch_model(&todos_model).layout().value_or_default();"));
        assert!(!src.contains("cx.watch_model(&filter_model).layout().value_or(TodoFilter::All);"));
        assert!(src.contains("let draft_state = cx.state().local::<String>();"));
        assert!(src.contains("let filter_state = cx.state().local_init(|| TodoFilter::All);"));
        assert!(src.contains("let todos_state = cx.state().local_init(|| {"));
        assert!(src.contains("let draft_value = draft_state.layout_value(cx);"));
        assert!(src.contains("let filter_value = filter_state.layout_value(cx);"));
        assert!(src.contains("let tip_nonce_value = tip_nonce_state.paint_value(cx);"));
        assert!(!src.contains("draft_state.layout(cx).value_or_default()"));
        assert!(!src.contains("filter_state.layout(cx).value_or(TodoFilter::All)"));
        assert!(src.contains("bind_todo_actions("));
        assert!(src.contains("fn bind_todo_actions("));
        assert!(src.contains("fn filter_chip("));
        assert!(src.contains("action: impl Into<fret::ActionId>,"));
        assert!(src.contains("ui::single(cx, todo_page(theme, card))"));
        assert!(src.contains("ui::v_flex(|cx| ui::single(cx, content))"));
        assert!(!src.contains("ui::v_flex(|cx| ui::children![cx; content])"));
        assert!(!src.contains("let card = card.into_element(cx);"));
        assert!(!src.contains("todo_page(theme, card).into_element(cx).into()"));
        assert!(src.contains("fn todo_page("));
        assert!(src.contains(") -> impl UiChild {"));
        assert!(!src.contains("Model<Vec<TodoItem>>"));
        assert!(!src.contains("Model<bool>"));
        assert!(!src.contains(".models_mut().insert("));
        assert!(!src.contains("decl_style::container_props"));
        assert!(!src.contains(".refine_style("));
        assert!(!src.contains(".refine_layout("));
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
        assert!(src.contains("shadcn::card_title(\"Simple Todo\")"));
        assert!(src.contains("shadcn::card_description("));
        assert!(!src.contains("shadcn::Card::build(|cx, out| {"));
        assert!(!src.contains("shadcn::CardHeader::build(|cx, out| {"));
        assert!(!src.contains("shadcn::CardContent::build(|cx, out| {"));
        assert!(src.contains(".locals_with((draft_state, next_id_state, todos_state))"));
        assert!(src.contains(".on::<act::Add>(|tx, (draft_state, next_id_state, todos_state)| {"));
        assert!(src.contains(".locals_with(todos_state)"));
        assert!(src.contains(".on::<act::ClearDone>(|tx, todos_state| {"));
        assert!(src.contains("let text = tx.value(&draft_state).trim().to_string();"));
        assert!(src.contains("let id = tx.value(&next_id_state);"));
        assert!(!src.contains("tx.value_or_else(&draft_state, String::new)"));
        assert!(src.contains(".submit_action(act::Add)"));
        assert!(!src.contains(".submit_command(act::Add.into())"));
        assert!(src.contains(".local(todos_state)"));
        assert!(src.contains(".payload_update_if::<act::Toggle>(|rows, id| {"));
        assert!(src.contains("fret::payload_actions!([Toggle(u64) ="));
        assert!(src.contains("let draft_state = cx.state().local::<String>();"));
        assert!(src.contains("let next_id_state = cx.state().local_init(|| 3u64);"));
        assert!(src.contains("let todos_state = cx.state().local_init(|| {"));
        assert!(src.contains("let todos = todos_state.layout_value(cx);"));
        assert!(src.contains("let draft_value = draft_state.layout_value(cx);"));
        assert!(!src.contains("todos_state.layout(cx).value_or_default()"));
        assert!(!src.contains("draft_state.layout(cx).value_or_default()"));
        assert!(src.contains("bind_todo_actions(cx, &draft_state, &next_id_state, &todos_state);"));
        assert!(src.contains("fn bind_todo_actions("));
        assert!(src.contains("ui::single(cx, todo_page(theme, content))"));
        assert!(!src.contains("let content = content.into_element(cx);"));
        assert!(!src.contains("todo_page(theme, content).into_element(cx).into()"));
        assert!(src.contains("fn todo_page("));
        assert!(!src.contains("cx: &mut UiCx<'_>,"));
        assert!(src.contains(") -> impl UiChild {"));
        assert!(src.contains("shadcn::Input::new(&draft_state)"));
        assert!(src.contains("shadcn::Checkbox::from_checked(row.done)"));
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
        let toml = simple_todo_template_cargo_toml("simple-todo-app", opts(), ".");
        assert!(!toml.contains("fret-query"));
        assert!(!toml.contains("fret-selector"));
    }

    #[test]
    fn todo_template_cargo_toml_has_no_query_selector_deps() {
        let toml = todo_template_cargo_toml("todo-app", opts(), ".");
        assert!(!toml.contains("fret-query"));
        assert!(!toml.contains("fret-selector"));
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
        let hello = hello_template_readme_md("hello-app", opts());
        assert!(hello.contains("Read local state values near the top of `render()`"));
        assert!(hello.contains("Default entrypoints"));
        assert!(hello.contains("cookbook/reference-only host-side glue"));
        assert!(hello.contains("first rung of the default onboarding path"));
        assert!(hello.contains("`cx.actions().local(&local).update::<A>(...)`"));
        assert!(hello.contains("widget-local `.action(...)` / `.listen(...)`"));
        assert!(!hello.contains("on_action_notify_models"));
        assert!(!hello.contains("use `on_activate*` only for local pressable glue"));

        let simple = simple_todo_template_readme_md("simple-todo-app", opts());
        assert!(simple.contains(
            "Use `ui::single(cx, child)` when a render root or wrapper closure only needs to late-land one typed child."
        ));
        assert!(simple.contains("prefer `LocalState<Vec<_>>` + payload actions"));
        assert!(simple.contains("Read tracked state near the top of `render()`"));
        assert!(simple.contains("`cx.actions().locals_with((...)).on::<A>(|tx, (...)| ...)`"));
        assert!(simple.contains("bind per-row payloads via `.action_payload(...)`"));
        assert!(simple.contains("`cx.actions().local(&rows_state).payload_update_if::<A>(...)`"));
        assert!(
            simple
                .contains("widget-local `.action(...)` / `.action_payload(...)` / `.listen(...)`")
        );
        assert!(simple.contains("cookbook/reference-only host-side glue"));
        assert!(simple.contains("second rung of the default onboarding path"));
        assert!(!simple.contains("on_action_notify_locals"));
        assert!(!simple.contains("`cx.actions().payload::<A>()`"));
        assert!(!simple.contains("keep `on_activate*` for local widget glue only"));

        let simple_with_assets =
            simple_todo_template_readme_md("simple-todo-app", opts_with_ui_assets());
        assert!(simple_with_assets.contains("`generated_assets::mount(builder)?`"));
        assert!(
            simple_with_assets.contains("`preferred_startup_plan()` / `preferred_startup_mode()`")
        );
        assert!(simple_with_assets.contains(
            "`fretboard assets rust write --dir assets --out src/generated_assets.rs --app-bundle simple-todo-app --force`"
        ));
        assert!(simple_with_assets.contains("`AssetBundleId::app(\"simple-todo-app\")`"));

        let todo = todo_template_readme_md("todo-app", opts());
        assert!(todo.contains("For App-only effects, prefer `cx.actions().transient::<A>(...)`"));
        assert!(todo.contains("cookbook/reference-only host-side glue"));
        assert!(todo.contains("`cx.actions().models::<A>(...)`"));
        assert!(todo.contains("`cx.effects().take_transient(...)`"));
        assert!(todo.contains("State: LocalState-first"));
        assert!(todo.contains("third rung of the default onboarding path"));
        assert!(todo.contains("bind keyed-row payloads via `.action_payload(...)`"));
        assert!(todo.contains("`cx.actions().local(&rows_state).payload_update_if::<A>(...)` as the default row-write path"));
        assert!(!todo.contains("`payload_locals::<A>(...)`"));
        assert!(!todo.contains("on_action_notify_locals"));
        assert!(!todo.contains("on_action_notify_transient"));
        assert!(!todo.contains("`cx.actions().payload::<A>()`"));

        let todo_with_assets = todo_template_readme_md("todo-app", opts_with_ui_assets());
        assert!(todo_with_assets.contains("`generated_assets::mount(builder)?`"));
        assert!(
            todo_with_assets.contains("`preferred_startup_plan()` / `preferred_startup_mode()`")
        );
        assert!(todo_with_assets.contains(
            "`fretboard assets rust write --dir assets --out src/generated_assets.rs --app-bundle todo-app --force`"
        ));
        assert!(todo_with_assets.contains("`AssetBundleId::app(\"todo-app\")`"));
    }

    #[test]
    fn generated_assets_stub_guides_regeneration_and_mounting() {
        let src = generated_assets_stub_rs("todo-app");
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
