use std::io::{IsTerminal as _, Write as _};
use std::path::{Path, PathBuf};

use crate::cli::{help, workspace_root};

pub(crate) fn init_cmd(args: Vec<String>) -> Result<(), String> {
    new_template_cmd("init", args)
}

pub(crate) fn new_cmd(args: Vec<String>) -> Result<(), String> {
    if args.len() == 1 && matches!(args[0].as_str(), "--help" | "-h") {
        return help();
    }
    if args.is_empty() {
        return new_wizard();
    }
    new_template_cmd("new", args)
}

fn new_template_cmd(invoked_as: &str, args: Vec<String>) -> Result<(), String> {
    let mut it = args.into_iter();
    let Some(template) = it.next() else {
        return Err(format!("missing template (try: {invoked_as} todo)"));
    };
    if matches!(template.as_str(), "--help" | "-h") {
        return help();
    }

    match template.as_str() {
        "empty" => init_empty(it.collect()),
        "todo" => init_todo(it.collect()),
        "hello" | "hello-world" => init_hello(it.collect()),
        other => Err(format!("unknown template for {invoked_as}: {other}")),
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
enum NewTemplate {
    Empty,
    Hello,
    Todo,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
enum IconPack {
    None,
    Lucide,
    Radix,
}

impl IconPack {
    fn parse(raw: &str) -> Result<Self, String> {
        match raw.trim().to_ascii_lowercase().as_str() {
            "none" | "no" | "off" | "false" => Ok(Self::None),
            "lucide" => Ok(Self::Lucide),
            "radix" => Ok(Self::Radix),
            other => Err(format!(
                "unknown icon pack: {other} (expected: lucide|radix|none)"
            )),
        }
    }

    fn as_str(self) -> &'static str {
        match self {
            IconPack::None => "none",
            IconPack::Lucide => "lucide",
            IconPack::Radix => "radix",
        }
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
struct ScaffoldOptions {
    icon_pack: IconPack,
    command_palette: bool,
    ui_assets: bool,
}

fn new_wizard() -> Result<(), String> {
    if !std::io::stdin().is_terminal() {
        return Err(
            "interactive wizard requires a TTY (try: `fretboard new todo --name my-todo`)"
                .to_string(),
        );
    }

    let root = workspace_root()?;

    println!("Fretboard new (interactive)");
    println!();

    let template = prompt_choice(
        "Template",
        &[
            ("empty", NewTemplate::Empty),
            ("hello", NewTemplate::Hello),
            ("todo", NewTemplate::Todo),
        ],
        1,
    )?;

    let default_name = match template {
        NewTemplate::Empty => "my-app",
        NewTemplate::Hello => "hello-world",
        NewTemplate::Todo => "todo-app",
    };

    let name_raw = prompt_line("Package name", Some(default_name))?;
    let package_name = sanitize_package_name(&name_raw)?;

    let default_out = root.join("local").join(&package_name);
    let out_raw = prompt_line(
        "Output path (blank = default)",
        Some(default_out.to_string_lossy().as_ref()),
    )?;
    let out_dir = PathBuf::from(out_raw);

    let icon_pack = match template {
        NewTemplate::Empty => IconPack::None,
        _ => prompt_choice(
            "Icons",
            &[
                ("lucide", IconPack::Lucide),
                ("radix", IconPack::Radix),
                ("none", IconPack::None),
            ],
            0,
        )?,
    };

    let command_palette = match template {
        NewTemplate::Empty => false,
        _ => prompt_yes_no("Enable command palette? (--command-palette)", false)?,
    };

    let ui_assets = match template {
        NewTemplate::Todo => prompt_yes_no("Enable UI assets cache? (--ui-assets)", false)?,
        _ => false,
    };

    let opts = ScaffoldOptions {
        icon_pack,
        command_palette,
        ui_assets,
    };

    println!();
    println!("Summary:");
    println!("  template: {:?}", template);
    println!("  name:     {package_name}");
    println!("  path:     {}", out_dir.display());
    if !matches!(template, NewTemplate::Empty) {
        println!("  icons:    {}", opts.icon_pack.as_str());
        println!("  palette:  {}", opts.command_palette);
    }
    if matches!(template, NewTemplate::Todo) {
        println!("  ui-assets: {}", opts.ui_assets);
    }
    println!();

    if !prompt_yes_no("Proceed?", true)? {
        return Err("aborted".to_string());
    }

    match template {
        NewTemplate::Empty => init_empty_at(&out_dir, &package_name),
        NewTemplate::Hello => init_hello_at(&out_dir, &package_name, opts),
        NewTemplate::Todo => init_todo_at(&out_dir, &package_name, opts),
    }
}

fn init_empty(args: Vec<String>) -> Result<(), String> {
    let root = workspace_root()?;

    let mut out_path: Option<PathBuf> = None;
    let mut name: Option<String> = None;

    let mut it = args.into_iter();
    while let Some(a) = it.next() {
        match a.as_str() {
            "--path" => {
                let raw = it
                    .next()
                    .ok_or_else(|| "--path requires a value".to_string())?;
                out_path = Some(PathBuf::from(raw));
            }
            "--name" => {
                name = Some(
                    it.next()
                        .ok_or_else(|| "--name requires a value".to_string())?,
                );
            }
            "--help" | "-h" => return help(),
            other => return Err(format!("unknown argument for init empty: {other}")),
        }
    }

    let package_name = sanitize_package_name(name.as_deref().unwrap_or("my-app"))?;

    let out_dir = out_path.unwrap_or_else(|| root.join("local").join(&package_name));
    init_empty_at(&out_dir, &package_name)
}

fn init_todo(args: Vec<String>) -> Result<(), String> {
    let root = workspace_root()?;

    let mut out_path: Option<PathBuf> = None;
    let mut name: Option<String> = None;
    let mut ui_assets = false;
    let mut icon_pack = IconPack::Lucide;
    let mut command_palette = false;

    let mut it = args.into_iter();
    while let Some(a) = it.next() {
        match a.as_str() {
            "--path" => {
                let raw = it
                    .next()
                    .ok_or_else(|| "--path requires a value".to_string())?;
                out_path = Some(PathBuf::from(raw));
            }
            "--name" => {
                name = Some(
                    it.next()
                        .ok_or_else(|| "--name requires a value".to_string())?,
                );
            }
            "--ui-assets" => ui_assets = true,
            "--icons" => {
                let raw = it
                    .next()
                    .ok_or_else(|| "--icons requires a value".to_string())?;
                icon_pack = IconPack::parse(&raw)?;
            }
            "--no-icons" => icon_pack = IconPack::None,
            "--command-palette" => command_palette = true,
            "--help" | "-h" => return help(),
            other => return Err(format!("unknown argument for init todo: {other}")),
        }
    }

    let package_name = sanitize_package_name(name.as_deref().unwrap_or("todo-app"))?;

    let out_dir = out_path.unwrap_or_else(|| root.join("local").join(&package_name));
    init_todo_at(
        &out_dir,
        &package_name,
        ScaffoldOptions {
            icon_pack,
            command_palette,
            ui_assets,
        },
    )
}

fn init_todo_at(out_dir: &Path, package_name: &str, opts: ScaffoldOptions) -> Result<(), String> {
    ensure_dir_is_new_or_empty(out_dir)?;

    let cargo_toml = todo_template_cargo_toml(package_name, opts);
    write_new_file(&out_dir.join("Cargo.toml"), &cargo_toml)?;

    let src_dir = out_dir.join("src");
    std::fs::create_dir_all(&src_dir).map_err(|e| e.to_string())?;
    write_new_file(
        &src_dir.join("main.rs"),
        &todo_template_main_rs(package_name, opts),
    )?;
    write_new_file(
        &out_dir.join("README.md"),
        &todo_template_readme_md(package_name, opts),
    )?;

    println!("Initialized todo template at: {}", out_dir.display());
    println!("Next:");
    println!(
        "  cargo run --manifest-path {}",
        out_dir.join("Cargo.toml").display()
    );
    Ok(())
}

fn init_hello(args: Vec<String>) -> Result<(), String> {
    let root = workspace_root()?;

    let mut out_path: Option<PathBuf> = None;
    let mut name: Option<String> = None;
    let mut icon_pack = IconPack::Lucide;
    let mut command_palette = false;

    let mut it = args.into_iter();
    while let Some(a) = it.next() {
        match a.as_str() {
            "--path" => {
                let raw = it
                    .next()
                    .ok_or_else(|| "--path requires a value".to_string())?;
                out_path = Some(PathBuf::from(raw));
            }
            "--name" => {
                name = Some(
                    it.next()
                        .ok_or_else(|| "--name requires a value".to_string())?,
                );
            }
            "--icons" => {
                let raw = it
                    .next()
                    .ok_or_else(|| "--icons requires a value".to_string())?;
                icon_pack = IconPack::parse(&raw)?;
            }
            "--no-icons" => icon_pack = IconPack::None,
            "--command-palette" => command_palette = true,
            "--help" | "-h" => return help(),
            other => return Err(format!("unknown argument for init hello: {other}")),
        }
    }

    let package_name = sanitize_package_name(name.as_deref().unwrap_or("hello-world"))?;

    let out_dir = out_path.unwrap_or_else(|| root.join("local").join(&package_name));
    init_hello_at(
        &out_dir,
        &package_name,
        ScaffoldOptions {
            icon_pack,
            command_palette,
            ui_assets: false,
        },
    )
}

fn init_hello_at(out_dir: &Path, package_name: &str, opts: ScaffoldOptions) -> Result<(), String> {
    ensure_dir_is_new_or_empty(out_dir)?;

    let cargo_toml = hello_template_cargo_toml(package_name, opts);
    write_new_file(&out_dir.join("Cargo.toml"), &cargo_toml)?;

    let src_dir = out_dir.join("src");
    std::fs::create_dir_all(&src_dir).map_err(|e| e.to_string())?;
    write_new_file(
        &src_dir.join("main.rs"),
        &hello_template_main_rs(package_name, opts),
    )?;
    write_new_file(
        &out_dir.join("README.md"),
        &hello_template_readme_md(package_name, opts),
    )?;

    println!("Initialized hello template at: {}", out_dir.display());
    println!("Next:");
    println!(
        "  cargo run --manifest-path {}",
        out_dir.join("Cargo.toml").display()
    );
    Ok(())
}

fn sanitize_package_name(raw: &str) -> Result<String, String> {
    let raw = raw.trim();
    if raw.is_empty() {
        return Err("package name cannot be empty".to_string());
    }

    let mut out = String::with_capacity(raw.len());
    for c in raw.chars() {
        let c = if c.is_ascii_whitespace() { '-' } else { c };
        if c.is_ascii_alphanumeric() || c == '-' || c == '_' {
            out.push(c.to_ascii_lowercase());
        } else {
            return Err(format!(
                "invalid package name: `{raw}` (unsupported character: `{c}`)"
            ));
        }
    }

    Ok(out)
}

fn init_empty_at(out_dir: &Path, package_name: &str) -> Result<(), String> {
    ensure_dir_is_new_or_empty(out_dir)?;

    let cargo_toml = empty_template_cargo_toml(package_name);
    write_new_file(&out_dir.join("Cargo.toml"), &cargo_toml)?;

    let src_dir = out_dir.join("src");
    std::fs::create_dir_all(&src_dir).map_err(|e| e.to_string())?;
    write_new_file(&src_dir.join("main.rs"), empty_template_main_rs())?;
    write_new_file(
        &out_dir.join("README.md"),
        &empty_template_readme_md(package_name),
    )?;

    println!("Initialized empty template at: {}", out_dir.display());
    println!("Next:");
    println!(
        "  cargo run --manifest-path {}",
        out_dir.join("Cargo.toml").display()
    );
    Ok(())
}

fn ensure_dir_is_new_or_empty(path: &Path) -> Result<(), String> {
    if path.exists() {
        let mut entries = std::fs::read_dir(path).map_err(|e| e.to_string())?;
        if entries.next().is_some() {
            return Err(format!(
                "output directory is not empty: {} (choose another --path)",
                path.display()
            ));
        }
        return Ok(());
    }

    std::fs::create_dir_all(path).map_err(|e| e.to_string())
}

fn write_new_file(path: &Path, contents: &str) -> Result<(), String> {
    if path.exists() {
        return Err(format!(
            "refusing to overwrite existing file: {}",
            path.display()
        ));
    }
    std::fs::write(path, contents).map_err(|e| e.to_string())
}

fn todo_template_cargo_toml(package_name: &str, opts: ScaffoldOptions) -> String {
    let mut bootstrap_features: Vec<&str> = vec!["ui-app-driver", "diagnostics"];
    if opts.command_palette {
        bootstrap_features.push("ui-app-command-palette");
    }
    if opts.ui_assets {
        bootstrap_features.push("ui-assets");
    }
    match opts.icon_pack {
        IconPack::Lucide => {
            bootstrap_features.push("icons-lucide");
            bootstrap_features.push("preload-icon-svgs");
        }
        IconPack::Radix => {
            bootstrap_features.push("icons-radix");
            bootstrap_features.push("preload-icon-svgs");
        }
        IconPack::None => {}
    }

    let bootstrap_features = bootstrap_features
        .into_iter()
        .map(|f| format!("\"{f}\""))
        .collect::<Vec<_>>()
        .join(", ");

    format!(
        r#"[package]
name = "{package_name}"
version = "0.1.0"
edition = "2024"

[dependencies]
anyhow = "1"
fret-app = {{ path = "../../crates/fret-app" }}
fret-bootstrap = {{ path = "../../ecosystem/fret-bootstrap", features = [{bootstrap_features}] }}
fret-ui-shadcn = {{ path = "../../ecosystem/fret-ui-shadcn", features = ["app-integration"] }}
[workspace]
"#
    )
}

fn empty_template_cargo_toml(package_name: &str) -> String {
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

fn hello_template_cargo_toml(package_name: &str, opts: ScaffoldOptions) -> String {
    let mut bootstrap_features: Vec<&str> = vec!["ui-app-driver", "diagnostics"];
    if opts.command_palette {
        bootstrap_features.push("ui-app-command-palette");
    }
    match opts.icon_pack {
        IconPack::Lucide => bootstrap_features.push("icons-lucide"),
        IconPack::Radix => bootstrap_features.push("icons-radix"),
        IconPack::None => {}
    }

    let bootstrap_features = bootstrap_features
        .into_iter()
        .map(|f| format!("\"{f}\""))
        .collect::<Vec<_>>()
        .join(", ");

    format!(
        r#"[package]
name = "{package_name}"
version = "0.1.0"
edition = "2024"

[dependencies]
anyhow = "1"
fret-app = {{ path = "../../crates/fret-app" }}
fret-bootstrap = {{ path = "../../ecosystem/fret-bootstrap", features = [{bootstrap_features}] }}
fret-ui-shadcn = {{ path = "../../ecosystem/fret-ui-shadcn", features = ["app-integration"] }}

[workspace]
"#
    )
}

fn todo_template_main_rs(_package_name: &str, opts: ScaffoldOptions) -> String {
    let ui_assets_builder = if opts.ui_assets {
        "\n        .with_ui_assets_budgets(64 * 1024 * 1024, 4096, 16 * 1024 * 1024, 4096)"
    } else {
        ""
    };

    let icons_builder = match opts.icon_pack {
        IconPack::Lucide => {
            "\n        .with_lucide_icons()\n        .preload_icon_svgs_on_gpu_ready()"
        }
        IconPack::Radix => {
            "\n        .with_radix_icons()\n        .preload_icon_svgs_on_gpu_ready()"
        }
        IconPack::None => "",
    };

    // Radix doesn't currently ship plus/trash icons in our curated set; keep the todo template
    // functional by falling back to text buttons when Lucide isn't selected.
    let has_action_icons = matches!(opts.icon_pack, IconPack::Lucide);

    let add_btn_def = if has_action_icons {
        r#"    let add_btn = shadcn::Button::new("")
        .size(shadcn::ButtonSize::Icon)
        .disabled(!add_enabled)
        .on_click(CMD_ADD)
        .children(vec![icon::icon(cx, IconId::new("lucide.plus"))])
        .into_element(cx);
"#
    } else {
        r#"    let add_btn = shadcn::Button::new("Add")
        .disabled(!add_enabled)
        .on_click(CMD_ADD)
        .into_element(cx);
"#
    };

    let remove_btn_def = if has_action_icons {
        r#"    let remove_btn = shadcn::Button::new("")
        .size(shadcn::ButtonSize::Icon)
        .variant(shadcn::ButtonVariant::Ghost)
        .on_click(remove_cmd)
        .children(vec![icon::icon(cx, IconId::new("lucide.trash"))])
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

use fret_app::{App, CommandId};
use fret_bootstrap::ui_app_with_hooks;
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
    next_id: u64,
}

fn main() -> anyhow::Result<()> {
    ui_app_with_hooks("todo", init_window, view, |d| d.on_command(on_command))
        .with_default_diagnostics()
        .with_default_config_files()?__UI_ASSETS_BUILDER__
        .with_main_window("todo", (560.0, 520.0))
        .init_app(|app| {
            shadcn::install_app(app);
        })__ICONS_BUILDER__
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
            text: Arc::from("Try the shadcn theme"),
        },
        TodoItem {
            id: 2,
            done: done_2,
            text: Arc::from("Build a tiny todo app"),
        },
    ]);

    TodoState {
        todos,
        draft: app.models_mut().insert(String::new()),
        next_id: 3,
    }
}

fn view(cx: &mut ElementContext<'_, App>, st: &mut TodoState) -> Vec<AnyElement> {
    cx.observe_model(&st.todos, Invalidation::Layout);
    cx.observe_model(&st.draft, Invalidation::Layout);

    let theme = Theme::global(&*cx.app).clone();
    let draft_value = cx
        .app
        .models()
        .read(&st.draft, |s| s.clone())
        .ok()
        .unwrap_or_default();

    let add_enabled = !draft_value.trim().is_empty();
__ADD_BTN_DEF__

    let input = shadcn::Input::new(st.draft.clone())
        .placeholder("Add a task…")
        .submit_command(CommandId::new(CMD_ADD))
        .into_element(cx);

    let input_row = stack::hstack(
        cx,
        stack::HStackProps::default()
            .layout(LayoutRefinement::default().w_full())
            .gap(Space::N2)
            .items_center(),
        |_cx| vec![input, add_btn],
    );

    let todos = cx
        .app
        .models()
        .read(&st.todos, |v| v.clone())
        .ok()
        .unwrap_or_default();

    let rows = stack::vstack(
        cx,
        stack::VStackProps::default()
            .layout(LayoutRefinement::default().w_full())
            .gap(Space::N3),
        |cx| {
            todos
                .iter()
                .map(|t| cx.keyed(t.id, |cx| todo_row(cx, &theme, t)))
                .collect()
        },
    );

    let chrome = ChromeRefinement::default()
        .bg(ColorRef::Color(theme.color_required("background")))
        .rounded(Radius::Lg)
        .border_1()
        .border_color(ColorRef::Color(theme.color_required("border")));

    let card = shadcn::Card::new(vec![
        shadcn::CardHeader::new(vec![
            shadcn::CardTitle::new("Todo").into_element(cx),
            shadcn::CardDescription::new("A minimal Fret + shadcn template.").into_element(cx),
        ])
        .into_element(cx),
        shadcn::CardContent::new(vec![
            stack::vstack(
                cx,
                stack::VStackProps::default()
                    .layout(LayoutRefinement::default().w_full())
                    .gap(Space::N4),
                |_cx| vec![input_row, rows],
            ),
        ])
        .into_element(cx),
    ])
    .refine_style(chrome)
    .refine_layout(LayoutRefinement::default().w_full().max_w(MetricRef::Px(Px(520.0))))
    .into_element(cx);

    let page = cx.container(
        decl_style::container_props(
            &theme,
            ChromeRefinement::default()
                .bg(ColorRef::Color(theme.color_required("muted")))
                .p(Space::N6),
            LayoutRefinement::default().w_full().h_full(),
        ),
        |cx| {
            vec![stack::vstack(
                cx,
                stack::VStackProps::default()
                    .layout(LayoutRefinement::default().w_full().h_full())
                    .justify_center()
                    .items_center(),
                |_cx| vec![card],
            )]
        },
    );

    vec![page]
}

fn todo_row(cx: &mut ElementContext<'_, App>, theme: &Theme, item: &TodoItem) -> AnyElement {
    cx.observe_model(&item.done, Invalidation::Layout);
    let done = cx
        .app
        .models()
        .read(&item.done, |v| *v)
        .ok()
        .unwrap_or(false);

    let checkbox = shadcn::Checkbox::new(item.done.clone()).into_element(cx);
    let remove_cmd = CommandId::new(format!("{}{}", CMD_REMOVE_PREFIX, item.id));
__REMOVE_BTN_DEF__

    let props = decl_style::container_props(
        theme,
        ChromeRefinement::default()
            .border_1()
            .border_color(ColorRef::Color(theme.color_required("border")))
            .rounded(Radius::Md)
            .p(Space::N3),
        LayoutRefinement::default().w_full(),
    );

    cx.container(props, |cx| {
        vec![stack::hstack(
            cx,
            stack::HStackProps::default()
                .layout(LayoutRefinement::default().w_full())
                .justify_between()
                .items_center(),
            |cx| {
                let label = cx.text_props(TextProps {
                    layout: Default::default(),
                    text: item.text.clone(),
                    style: None,
                    color: Some(theme.color_required(if done {
                        "muted-foreground"
                    } else {
                        "foreground"
                    })),
                    wrap: TextWrap::None,
                    overflow: TextOverflow::Ellipsis,
                });

                vec![
                    stack::hstack(
                        cx,
                        stack::HStackProps::default()
                            .layout(LayoutRefinement::default().flex_1().min_w_0())
                            .gap(Space::N3)
                            .items_center(),
                        |_cx| vec![checkbox.clone(), label],
                    ),
                    remove_btn.clone(),
                ]
            },
        )]
    })
}

fn on_command(
    app: &mut App,
    _services: &mut dyn UiServices,
    _window: AppWindowId,
    _ui: &mut UiTree<App>,
    state: &mut TodoState,
    cmd: &CommandId,
) {
    match cmd.as_str() {
        CMD_ADD => {
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
        CMD_CLEAR_DONE => {
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
        other => {
            if let Some(id) = other.strip_prefix(CMD_REMOVE_PREFIX) {
                if let Ok(id) = id.parse::<u64>() {
                    let _ = app.models_mut().update(&state.todos, |todos| {
                        todos.retain(|t| t.id != id);
                    });
                }
            }
        }
    }
}
"#;

    TEMPLATE
        .replace("__UI_ASSETS_BUILDER__", ui_assets_builder)
        .replace("__ICONS_BUILDER__", icons_builder)
        .replace("__ADD_BTN_DEF__", add_btn_def)
        .replace("__REMOVE_BTN_DEF__", remove_btn_def)
}

fn hello_template_main_rs(package_name: &str, opts: ScaffoldOptions) -> String {
    let icons_builder = match opts.icon_pack {
        IconPack::Lucide => "\n        .with_lucide_icons()",
        IconPack::Radix => "\n        .with_radix_icons()",
        IconPack::None => "",
    };

    let palette_button = if opts.command_palette {
        r#"
                shadcn::Button::new("Command palette")
                    .on_click("app.command_palette")
                    .into_element(cx),"#
    } else {
        ""
    };

    format!(
        r#"use fret_app::{{App, CommandId}};
use fret_bootstrap::ui_app_with_hooks;
use fret_ui_shadcn::{{self as shadcn, prelude::*}};

const CMD_CLICK: &str = "hello.click";

fn main() -> anyhow::Result<()> {{
    ui_app_with_hooks("{package_name}", init_window, view, |d| d.on_command(on_command))
        .with_default_diagnostics()
        .with_default_config_files()?
        .with_main_window("{package_name}", (560.0, 360.0))
        .init_app(|app| {{
            shadcn::install_app(app);
        }})__ICONS_BUILDER__
        .run()
        .map_err(anyhow::Error::from)
}}

fn init_window(_app: &mut App, _window: AppWindowId) {{}}

fn view(cx: &mut ElementContext<'_, App>, _st: &mut ()) -> Vec<AnyElement> {{
    vec![stack::vstack(
        cx,
        stack::VStackProps::default()
            .layout(LayoutRefinement::default().size_full())
            .gap(Space::N4)
            .items_center()
            .justify_center(),
        |cx| {{
            vec![
                shadcn::Label::new("Hello, world!").into_element(cx),
                shadcn::Button::new("Click me")
                    .on_click(CMD_CLICK)
                    .into_element(cx),
__PALETTE_BUTTON__
            ]
        }},
    )]
}}

fn on_command(
    _app: &mut App,
    _services: &mut dyn UiServices,
    _window: AppWindowId,
    _ui: &mut UiTree<App>,
    _st: &mut (),
    cmd: &CommandId,
) {{
    if cmd.as_str() == CMD_CLICK {{
        println!("Clicked!");
    }}
}}
"#
    )
    .replace("__ICONS_BUILDER__", icons_builder)
    .replace("__PALETTE_BUTTON__", palette_button)
}

fn empty_template_main_rs() -> &'static str {
    r#"fn main() -> anyhow::Result<()> {
    println!("Hello from Fret!");
    Ok(())
}
"#
}

fn todo_template_readme_md(package_name: &str, opts: ScaffoldOptions) -> String {
    let ui_assets_line = if opts.ui_assets {
        "- UI assets: enabled (`fret-bootstrap/ui-assets`)\n"
    } else {
        "- UI assets: disabled (use `fretboard new todo --ui-assets` if you need images/SVG caches)\n"
    };

    let icons_line = match opts.icon_pack {
        IconPack::Lucide => "- Icons: Lucide (`fret-bootstrap/icons-lucide`)\n",
        IconPack::Radix => "- Icons: Radix (`fret-bootstrap/icons-radix`)\n",
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

## Notes

- Theme: shadcn new-york-v4 (Slate / Light)
{icons_line}{palette_line}
{ui_assets_line}
## Next steps

- Edit UI in `src/main.rs`
- If you want hotpatch later, keep commands/IDs stable and prefer the `ui_app_with_hooks` golden path (ADR 0107 / 0112).
"#
    )
}

fn empty_template_readme_md(package_name: &str) -> String {
    format!(
        r#"# {package_name}

Generated by `fretboard new`.

## Run

```bash
cargo run
```
"#
    )
}

fn hello_template_readme_md(package_name: &str, opts: ScaffoldOptions) -> String {
    let icons_line = match opts.icon_pack {
        IconPack::Lucide => "- Icons: Lucide (`fret-bootstrap/icons-lucide`)\n",
        IconPack::Radix => "- Icons: Radix (`fret-bootstrap/icons-radix`)\n",
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

## Notes

- Theme: shadcn new-york-v4 (default via `fret-ui-shadcn/app-integration`)
{icons_line}{palette_line}
- Next: edit `src/main.rs` and replace the view tree
"#
    )
}

fn prompt_line(prompt: &str, default: Option<&str>) -> Result<String, String> {
    let mut stdout = std::io::stdout();
    match default {
        Some(default) => {
            write!(&mut stdout, "{prompt} [{default}]: ").map_err(|e| e.to_string())?;
        }
        None => {
            write!(&mut stdout, "{prompt}: ").map_err(|e| e.to_string())?;
        }
    }
    stdout.flush().map_err(|e| e.to_string())?;

    let mut line = String::new();
    std::io::stdin()
        .read_line(&mut line)
        .map_err(|e| e.to_string())?;
    let line = line.trim().to_string();
    if line.is_empty() {
        Ok(default.unwrap_or_default().to_string())
    } else {
        Ok(line)
    }
}

fn prompt_yes_no(prompt: &str, default: bool) -> Result<bool, String> {
    let hint = if default { "Y/n" } else { "y/N" };
    loop {
        let v = prompt_line(prompt, Some(hint))?;
        let v = v.trim().to_ascii_lowercase();
        match v.as_str() {
            "" => return Ok(default),
            "y" | "yes" | "true" | "1" => return Ok(true),
            "n" | "no" | "false" | "0" => return Ok(false),
            _ => {
                println!("Please enter y/n.");
            }
        }
    }
}

fn prompt_choice<T: Copy>(
    prompt: &str,
    items: &[(&str, T)],
    default_index: usize,
) -> Result<T, String> {
    if items.is_empty() {
        return Err("prompt_choice requires at least one item".to_string());
    }
    let default_index = default_index.min(items.len().saturating_sub(1));

    println!("{prompt}:");
    for (i, (label, _)) in items.iter().enumerate() {
        if i == default_index {
            println!("  {}) {} (default)", i + 1, label);
        } else {
            println!("  {}) {}", i + 1, label);
        }
    }

    loop {
        let raw = prompt_line("Select", Some(&(default_index + 1).to_string()))?;
        let raw = raw.trim();
        if raw.is_empty() {
            return Ok(items[default_index].1);
        }
        let Ok(n) = raw.parse::<usize>() else {
            println!("Please enter a number.");
            continue;
        };
        if (1..=items.len()).contains(&n) {
            return Ok(items[n - 1].1);
        }
        println!("Please enter a number between 1 and {}.", items.len());
    }
}
