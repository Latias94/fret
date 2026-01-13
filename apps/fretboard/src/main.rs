use std::path::{Path, PathBuf};
use std::process::{Command, ExitCode};

use std::io::{IsTerminal as _, Write as _};

fn main() -> ExitCode {
    match run() {
        Ok(()) => ExitCode::SUCCESS,
        Err(err) => {
            eprintln!("error: {err}");
            ExitCode::from(2)
        }
    }
}

fn run() -> Result<(), String> {
    let mut args = std::env::args().skip(1);
    let Some(cmd) = args.next() else {
        return help();
    };

    match cmd.as_str() {
        "help" | "-h" | "--help" => help(),
        "init" => init_cmd(args.collect()),
        "new" => new_cmd(args.collect()),
        "hotpatch" => hotpatch_cmd(args.collect()),
        "list" => match args.next().as_deref() {
            Some("native-demos") => list_native_demos(),
            Some("web-demos") => list_web_demos(),
            Some(other) => Err(format!("unknown list target: {other}")),
            None => Err("missing list target (try: list native-demos)".to_string()),
        },
        "dev" => match args.next().as_deref() {
            Some("native") => dev_native(args.collect()),
            Some("web") => dev_web(args.collect()),
            Some(other) => Err(format!("unknown dev target: {other}")),
            None => Err("missing dev target (try: dev native)".to_string()),
        },
        other => Err(format!("unknown command: {other}")),
    }
}

fn help() -> Result<(), String> {
    println!(
        r#"fretboard dev tooling for the Fret workspace

Usage:
  fretboard help
  fretboard new [template] [--path <path>] [--name <name>] [--ui-assets]
  fretboard new             # interactive wizard
  fretboard new todo        # non-interactive (template shortcut)
  fretboard new hello       # non-interactive (template shortcut)
  fretboard init <template> [...]    # alias for `new` (compat)
  fretboard hotpatch poke [--path <path>]
  fretboard hotpatch path [--path <path>]
  fretboard hotpatch watch [--path <path>...] [--trigger-path <path>] [--poll-ms <ms>] [--debounce-ms <ms>]
  fretboard list native-demos
  fretboard list web-demos
  fretboard dev native [--bin <name> | --choose] [--hotpatch] [--hotpatch-trigger-path <path>] [--hotpatch-poll-ms <ms>] [-- <args...>]
  fretboard dev native [--bin <name> | --choose] --hotpatch-devserver <ws_endpoint> [--hotpatch-build-id <auto|none|u64>] [-- <args...>]
  fretboard dev native [--bin <name> | --choose] --hotpatch-dx [--hotpatch-dx-ws <ws_endpoint>] [--hotpatch-build-id <auto|none|u64>] [-- <args...>]
  fretboard dev web [--port <port>] [--demo <demo> | --choose]

Examples:
  fretboard new todo --name my-todo
  fretboard new hello --name hello-world
  fretboard dev native --bin components_gallery
  fretboard dev native --bin todo_demo
  fretboard dev native --bin assets_demo
  fretboard dev native --bin hotpatch_smoke_demo
  fretboard dev native --choose
  fretboard dev native --bin image_upload_demo -- --help
  fretboard dev native --hotpatch --choose   # file-triggered runner reload (default: `.fret/hotpatch.touch`)
  fretboard hotpatch poke                   # updates `.fret/hotpatch.touch` (triggers a reload)
  fretboard hotpatch watch                  # polls workspace sources and auto-pokes on change
  fretboard dev native --hotpatch-devserver ws://127.0.0.1:8080/_dioxus
  fretboard dev native --bin hotpatch_smoke_demo --hotpatch-dx
  fretboard dev web --demo plot_demo
"#
    );
    Ok(())
}

fn workspace_root() -> Result<PathBuf, String> {
    let cwd = std::env::current_dir().map_err(|e| e.to_string())?;
    for dir in cwd.ancestors() {
        if dir.join("Cargo.toml").is_file() {
            return Ok(dir.to_path_buf());
        }
    }
    Err("failed to locate workspace root (Cargo.toml not found in ancestors)".to_string())
}

fn init_cmd(args: Vec<String>) -> Result<(), String> {
    new_template_cmd("init", args)
}

fn new_cmd(args: Vec<String>) -> Result<(), String> {
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

    let ui_assets = match template {
        NewTemplate::Todo => prompt_yes_no("Enable UI assets cache? (--ui-assets)", false)?,
        _ => false,
    };

    println!();
    println!("Summary:");
    println!("  template: {:?}", template);
    println!("  name:     {package_name}");
    println!("  path:     {}", out_dir.display());
    if matches!(template, NewTemplate::Todo) {
        println!("  ui-assets: {ui_assets}");
    }
    println!();

    if !prompt_yes_no("Proceed?", true)? {
        return Err("aborted".to_string());
    }

    match template {
        NewTemplate::Empty => init_empty_at(&out_dir, &package_name),
        NewTemplate::Hello => init_hello_at(&out_dir, &package_name),
        NewTemplate::Todo => init_todo_at(&out_dir, &package_name, ui_assets),
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
            "--help" | "-h" => return help(),
            other => return Err(format!("unknown argument for init todo: {other}")),
        }
    }

    let package_name = sanitize_package_name(name.as_deref().unwrap_or("todo-app"))?;

    let out_dir = out_path.unwrap_or_else(|| root.join("local").join(&package_name));
    init_todo_at(&out_dir, &package_name, ui_assets)
}

fn init_todo_at(out_dir: &Path, package_name: &str, ui_assets: bool) -> Result<(), String> {
    ensure_dir_is_new_or_empty(out_dir)?;

    let cargo_toml = todo_template_cargo_toml(&package_name, ui_assets);
    write_new_file(&out_dir.join("Cargo.toml"), &cargo_toml)?;

    let src_dir = out_dir.join("src");
    std::fs::create_dir_all(&src_dir).map_err(|e| e.to_string())?;
    write_new_file(
        &src_dir.join("main.rs"),
        &todo_template_main_rs(&package_name, ui_assets),
    )?;
    write_new_file(
        &out_dir.join("README.md"),
        &todo_template_readme_md(&package_name, ui_assets),
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
            other => return Err(format!("unknown argument for init hello: {other}")),
        }
    }

    let package_name = sanitize_package_name(name.as_deref().unwrap_or("hello-world"))?;

    let out_dir = out_path.unwrap_or_else(|| root.join("local").join(&package_name));
    init_hello_at(&out_dir, &package_name)
}

fn init_hello_at(out_dir: &Path, package_name: &str) -> Result<(), String> {
    ensure_dir_is_new_or_empty(out_dir)?;

    let cargo_toml = hello_template_cargo_toml(package_name);
    write_new_file(&out_dir.join("Cargo.toml"), &cargo_toml)?;

    let src_dir = out_dir.join("src");
    std::fs::create_dir_all(&src_dir).map_err(|e| e.to_string())?;
    write_new_file(
        &src_dir.join("main.rs"),
        &hello_template_main_rs(package_name),
    )?;
    write_new_file(
        &out_dir.join("README.md"),
        &hello_template_readme_md(package_name),
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

fn todo_template_cargo_toml(package_name: &str, ui_assets: bool) -> String {
    let ui_assets_features = if ui_assets { ", \"ui-assets\"" } else { "" };

    format!(
        r#"[package]
name = "{package_name}"
version = "0.1.0"
edition = "2024"

[dependencies]
anyhow = "1"
fret-app = {{ path = "../../crates/fret-app" }}
fret-bootstrap = {{ path = "../../ecosystem/fret-bootstrap", features = ["ui-app-driver", "preload-icon-svgs", "icons-lucide", "diagnostics"{ui_assets_features}] }}
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

fn hello_template_cargo_toml(package_name: &str) -> String {
    format!(
        r#"[package]
name = "{package_name}"
version = "0.1.0"
edition = "2024"

[dependencies]
anyhow = "1"
fret-app = {{ path = "../../crates/fret-app" }}
fret-bootstrap = {{ path = "../../ecosystem/fret-bootstrap", features = ["ui-app-driver", "diagnostics"] }}
fret-ui-shadcn = {{ path = "../../ecosystem/fret-ui-shadcn", features = ["app-integration"] }}

[workspace]
"#
    )
}

fn todo_template_main_rs(_package_name: &str, ui_assets: bool) -> String {
    let ui_assets_builder = if ui_assets {
        "\n        .with_ui_assets_budgets(64 * 1024 * 1024, 4096, 16 * 1024 * 1024, 4096)"
    } else {
        ""
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
        })
        .with_lucide_icons()
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
    let add_btn = shadcn::Button::new("")
        .size(shadcn::ButtonSize::Icon)
        .disabled(!add_enabled)
        .on_click(CMD_ADD)
        .children(vec![icon::icon(cx, IconId::new("lucide.plus"))])
        .into_element(cx);

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
    let remove_btn = shadcn::Button::new("")
        .size(shadcn::ButtonSize::Icon)
        .variant(shadcn::ButtonVariant::Ghost)
        .on_click(remove_cmd)
        .children(vec![icon::icon(cx, IconId::new("lucide.trash-2"))])
        .into_element(cx);

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

    TEMPLATE.replace("__UI_ASSETS_BUILDER__", ui_assets_builder)
}

fn hello_template_main_rs(package_name: &str) -> String {
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
        }})
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
}

fn empty_template_main_rs() -> &'static str {
    r#"fn main() -> anyhow::Result<()> {
    println!("Hello from Fret!");
    Ok(())
}
"#
}

fn todo_template_readme_md(package_name: &str, ui_assets: bool) -> String {
    let ui_assets_line = if ui_assets {
        "- UI assets: enabled (`fret-bootstrap/ui-assets`)\n"
    } else {
        "- UI assets: disabled (use `fretboard init todo --ui-assets` if you need images/SVG caches)\n"
    };

    format!(
        r#"# {package_name}

Generated by `fretboard init todo`.

## Run

```bash
cargo run
```

## Notes

- Theme: shadcn new-york-v4 (Slate / Light)
- Icons: Lucide (`fret-bootstrap/icons-lucide`)
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

fn hello_template_readme_md(package_name: &str) -> String {
    format!(
        r#"# {package_name}

Generated by `fretboard init hello`.

## Run

```bash
cargo run
```

## Notes

- Theme: shadcn new-york-v4 (default via `fret-ui-shadcn/app-integration`)
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

fn list_native_demos() -> Result<(), String> {
    let root = workspace_root()?;
    let bin_dir = root.join("apps").join("fret-demo").join("src").join("bin");
    let mut demos = read_rs_stems(&bin_dir)?;
    demos.sort();
    for demo in demos {
        println!("{demo}");
    }
    Ok(())
}

fn list_web_demos() -> Result<(), String> {
    for demo in web_demos() {
        println!("{demo}");
    }
    Ok(())
}

fn dev_native(args: Vec<String>) -> Result<(), String> {
    let root = workspace_root()?;
    let demos = list_native_demos_from(&root)?;

    let mut bin: Option<String> = None;
    let mut choose = false;
    let mut hotpatch = false;
    let mut hotpatch_trigger_path: Option<String> = None;
    let mut hotpatch_poll_ms: Option<u64> = None;
    let mut hotpatch_devserver_ws: Option<String> = None;
    let mut hotpatch_build_id: Option<HotpatchBuildIdArg> = None;
    let mut hotpatch_dx = false;
    let mut hotpatch_dx_ws: Option<String> = None;
    let mut passthrough: Vec<String> = Vec::new();

    let mut it = args.into_iter();
    while let Some(a) = it.next() {
        match a.as_str() {
            "--bin" => {
                bin = Some(
                    it.next()
                        .ok_or_else(|| "--bin requires a value".to_string())?,
                );
            }
            "--choose" => choose = true,
            "--hotpatch" => hotpatch = true,
            "--hotpatch-trigger-path" => {
                hotpatch_trigger_path = Some(
                    it.next()
                        .ok_or_else(|| "--hotpatch-trigger-path requires a value".to_string())?,
                );
            }
            "--hotpatch-poll-ms" => {
                let raw = it
                    .next()
                    .ok_or_else(|| "--hotpatch-poll-ms requires a value".to_string())?;
                hotpatch_poll_ms = Some(raw.parse::<u64>().map_err(|e| e.to_string())?);
            }
            "--hotpatch-devserver" => {
                hotpatch_devserver_ws = Some(
                    it.next()
                        .ok_or_else(|| "--hotpatch-devserver requires a value".to_string())?,
                );
            }
            "--hotpatch-dx" => hotpatch_dx = true,
            "--hotpatch-dx-ws" => {
                hotpatch_dx_ws = Some(
                    it.next()
                        .ok_or_else(|| "--hotpatch-dx-ws requires a value".to_string())?,
                );
            }
            "--hotpatch-build-id" => {
                let raw = it
                    .next()
                    .ok_or_else(|| "--hotpatch-build-id requires a value".to_string())?;
                hotpatch_build_id = Some(parse_hotpatch_build_id(&raw)?);
            }
            "--" => {
                passthrough.extend(it);
                break;
            }
            "--help" | "-h" => return help(),
            other => return Err(format!("unknown argument for dev native: {other}")),
        }
    }

    if hotpatch && hotpatch_devserver_ws.is_some() {
        return Err("cannot combine --hotpatch and --hotpatch-devserver".to_string());
    }
    if hotpatch_dx && (hotpatch || hotpatch_devserver_ws.is_some()) {
        return Err(
            "cannot combine --hotpatch-dx with --hotpatch/--hotpatch-devserver".to_string(),
        );
    }

    let bin = match (bin.as_deref(), choose) {
        (Some(name), _) => {
            validate_native_demo(&demos, name)?;
            name.to_string()
        }
        (None, true) => prompt_choose_demo(&demos)?,
        (None, false) => "todo_demo".to_string(),
    };

    if (hotpatch || hotpatch_devserver_ws.is_some()) && !is_hotpatch_ready_native_demo(&bin) {
        eprintln!(
            "warning: `{bin}` is not a hotpatch-ready demo. Hotpatch will only trigger a safe runner reload boundary.\n  try: `--bin todo_demo` or `--bin assets_demo` for the FnDriver/UiAppDriver hotpatch path"
        );
    }

    if hotpatch_dx {
        return dev_native_hotpatch_dx(
            &root,
            &bin,
            hotpatch_dx_ws.as_deref(),
            hotpatch_build_id,
            passthrough,
        );
    }

    let mut cmd = Command::new("cargo");
    cmd.current_dir(&root).args(["run", "-p", "fret-demo"]);
    let mut cargo_features: Vec<&str> = Vec::new();
    if hotpatch || hotpatch_devserver_ws.is_some() {
        cargo_features.push("hotpatch");
        cmd.env("FRET_HOTPATCH", "1");
    }
    if matches!(bin.as_str(), "node_graph_demo" | "node_graph_domain_demo") {
        cargo_features.push("node-graph-demos");
    }
    let cargo_features = cargo_features.join(",");
    if !cargo_features.is_empty() {
        cmd.args(["--features", &cargo_features]);
    }
    if hotpatch {
        let trigger_path = hotpatch_trigger_path
            .as_deref()
            .unwrap_or(".fret/hotpatch.touch");
        let trigger_path = resolve_workspace_relative(&root, trigger_path);

        // Ensure the trigger file exists before the app starts so the runner can capture the
        // initial marker without forcing an immediate hot reload.
        ensure_hotpatch_trigger_file_initialized(&trigger_path)?;

        eprintln!(
            "Hotpatch(file-trigger): enabled (note: this only triggers a runner reload boundary; it does not rebuild/apply patches)"
        );
        eprintln!("  trigger: {}", trigger_path.display());
        eprintln!("  poke:    fretboard hotpatch poke");

        cmd.env("FRET_HOTPATCH_TRIGGER_PATH", &trigger_path);
        if let Some(ms) = hotpatch_poll_ms {
            cmd.env("FRET_HOTPATCH_POLL_MS", ms.to_string());
        }
    }
    if let Some(ws) = hotpatch_devserver_ws.as_deref() {
        eprintln!("Hotpatch(devserver): enabled");
        eprintln!("  ws: {}", ws);
        eprintln!(
            "  note: this expects an external devserver that delivers Subsecond JumpTables (e.g. dioxus-cli)"
        );
        cmd.env("FRET_HOTPATCH_DEVSERVER_WS", ws);

        let build_id = match hotpatch_build_id.unwrap_or(HotpatchBuildIdArg::Auto) {
            HotpatchBuildIdArg::None => None,
            HotpatchBuildIdArg::Auto => Some(generate_hotpatch_build_id()),
            HotpatchBuildIdArg::Value(v) => Some(v),
        };
        if let Some(build_id) = build_id {
            eprintln!("  build_id: {build_id}");
            cmd.env("FRET_HOTPATCH_BUILD_ID", build_id.to_string());
        }
    }
    cmd.args(["--bin", &bin]);
    if !passthrough.is_empty() {
        cmd.arg("--").args(passthrough);
    }

    let status = cmd.status().map_err(|e| e.to_string())?;
    if !status.success() {
        return Err(format!("cargo exited with status: {status}"));
    }
    Ok(())
}

fn is_hotpatch_ready_native_demo(name: &str) -> bool {
    matches!(name, "todo_demo" | "assets_demo" | "hotpatch_smoke_demo")
}

fn dev_web(args: Vec<String>) -> Result<(), String> {
    let mut port: Option<u16> = None;
    let mut demo: Option<String> = None;
    let mut choose = false;

    let mut it = args.into_iter();
    while let Some(a) = it.next() {
        match a.as_str() {
            "--port" => {
                let raw = it
                    .next()
                    .ok_or_else(|| "--port requires a value".to_string())?;
                port = Some(raw.parse::<u16>().map_err(|e| e.to_string())?);
            }
            "--demo" => {
                demo = Some(
                    it.next()
                        .ok_or_else(|| "--demo requires a value".to_string())?,
                );
            }
            "--choose" => choose = true,
            "--help" | "-h" => return help(),
            other => return Err(format!("unknown argument for dev web: {other}")),
        }
    }

    let root = workspace_root()?;
    let web_dir = root.join("apps").join("fret-demo-web");

    let effective_port = port.unwrap_or(8080);
    let mut url = format!("http://127.0.0.1:{effective_port}");
    let demo = match (demo.as_deref(), choose) {
        (Some(name), _) => {
            validate_web_demo(name)?;
            Some(name.to_string())
        }
        (None, true) => Some(prompt_choose_demo(&web_demos_as_vec())?),
        (None, false) => None,
    };
    if let Some(demo) = demo.as_deref() {
        url.push_str(&format!("/?demo={demo}"));
    }

    eprintln!("Starting Trunk dev server in `{}`", display_path(&web_dir));
    eprintln!("Open: {url}");

    let mut cmd = Command::new("trunk");
    cmd.current_dir(web_dir).args(["serve"]);
    if let Some(port) = port {
        cmd.args(["--port", &port.to_string()]);
    }

    let status = cmd.status().map_err(|e| {
        if e.kind() == std::io::ErrorKind::NotFound {
            "failed to run `trunk` (not found). Install it with: `cargo install trunk`".to_string()
        } else {
            e.to_string()
        }
    })?;
    if !status.success() {
        return Err(format!("trunk exited with status: {status}"));
    }
    Ok(())
}

fn dev_native_hotpatch_dx(
    workspace_root: &Path,
    bin: &str,
    hotpatch_dx_ws: Option<&str>,
    hotpatch_build_id: Option<HotpatchBuildIdArg>,
    passthrough: Vec<String>,
) -> Result<(), String> {
    let mut cmd = Command::new("dx");
    cmd.current_dir(workspace_root)
        .args(["serve", "--hotpatch", "--open", "false"]);

    if let Some(ws) = hotpatch_dx_ws {
        let (addr, port) = parse_ws_endpoint_addr(ws)?;
        cmd.args(["--addr", &addr, "--port", &port.to_string()]);
        cmd.env("FRET_HOTPATCH_DEVSERVER_WS", ws);
    }

    let resolved_build_id = match hotpatch_build_id.unwrap_or(HotpatchBuildIdArg::Auto) {
        HotpatchBuildIdArg::None => None,
        // In `dx serve` mode the devserver assigns its own build id (typically `0`).
        // Forcing a random build id breaks client matching ("no ASLR reference").
        HotpatchBuildIdArg::Auto => None,
        HotpatchBuildIdArg::Value(v) => Some(v),
    };

    cmd.env("FRET_HOTPATCH", "1");
    if let Some(build_id) = resolved_build_id {
        cmd.env("FRET_HOTPATCH_BUILD_ID", build_id.to_string());
    }

    let mut cargo_features: Vec<&str> = vec!["hotpatch"];
    if matches!(bin, "node_graph_demo" | "node_graph_domain_demo") {
        cargo_features.push("node-graph-demos");
    }
    let cargo_features = cargo_features.join(",");

    cmd.args([
        "--package",
        "fret-demo",
        "--features",
        &cargo_features,
        "--bin",
        bin,
    ]);

    if !passthrough.is_empty() {
        cmd.args(["--args", &passthrough.join(" ")]);
    }

    let status = cmd.status().map_err(|e| {
        if e.kind() == std::io::ErrorKind::NotFound {
            "failed to run `dx` (not found). Install it with: `cargo install dioxus-cli`"
                .to_string()
        } else {
            e.to_string()
        }
    })?;
    if !status.success() {
        return Err(format!("dx exited with status: {status}"));
    }
    Ok(())
}

fn parse_ws_endpoint_addr(ws: &str) -> Result<(String, u16), String> {
    let ws = ws.trim();
    let without_scheme = ws
        .strip_prefix("ws://")
        .or_else(|| ws.strip_prefix("wss://"))
        .ok_or_else(|| format!("invalid ws endpoint `{ws}` (expected ws://... or wss://...)"))?;

    let host_port = without_scheme.split('/').next().unwrap_or_default().trim();
    if host_port.is_empty() {
        return Err(format!(
            "invalid ws endpoint `{ws}` (expected ws://<host>:<port>/...)"
        ));
    }

    let (host, port_raw) = if let Some(rest) = host_port.strip_prefix('[') {
        let end = rest
            .find(']')
            .ok_or_else(|| format!("invalid ws endpoint `{ws}` (malformed IPv6 host)"))?;
        let host = &rest[..end];
        let port = rest[end + 1..]
            .strip_prefix(':')
            .ok_or_else(|| format!("invalid ws endpoint `{ws}` (missing port)"))?;
        (host, port)
    } else {
        host_port
            .rsplit_once(':')
            .ok_or_else(|| format!("invalid ws endpoint `{ws}` (missing port)"))?
    };

    let host = match host {
        "localhost" => "127.0.0.1",
        other => other,
    };
    let port = port_raw
        .parse::<u16>()
        .map_err(|e| format!("invalid ws endpoint `{ws}` (invalid port `{port_raw}`): {e}"))?;

    Ok((host.to_string(), port))
}

fn read_rs_stems(dir: &Path) -> Result<Vec<String>, String> {
    let mut out = Vec::new();
    let rd = std::fs::read_dir(dir)
        .map_err(|e| format!("read_dir failed for `{}`: {e}", display_path(dir)))?;
    for ent in rd {
        let ent = ent.map_err(|e| e.to_string())?;
        let path = ent.path();
        if path.extension().and_then(|s| s.to_str()) != Some("rs") {
            continue;
        }
        let Some(stem) = path.file_stem().and_then(|s| s.to_str()) else {
            continue;
        };
        out.push(stem.to_string());
    }
    Ok(out)
}

fn display_path(path: &Path) -> String {
    path.to_string_lossy().to_string()
}

fn hotpatch_cmd(args: Vec<String>) -> Result<(), String> {
    let mut it = args.into_iter();
    match it.next().as_deref() {
        Some("poke") => {
            let path = parse_hotpatch_path_arg(&mut it)?;
            hotpatch_poke(path.as_deref())
        }
        Some("path") => {
            let root = workspace_root()?;
            let path = parse_hotpatch_path_arg(&mut it)?;
            let path = path
                .as_deref()
                .map(|p| resolve_workspace_relative(&root, p))
                .unwrap_or_else(|| hotpatch_trigger_path(&root));
            println!("{}", path.display());
            Ok(())
        }
        Some("watch") => hotpatch_watch(it.collect()),
        Some("help") | Some("-h") | Some("--help") | None => {
            println!(
                r#"Usage:
  fretboard hotpatch poke [--path <path>]   # update the trigger file (causes runner reload when enabled)
  fretboard hotpatch path [--path <path>]   # print the trigger file path
  fretboard hotpatch watch [--path <path>...] [--trigger-path <path>] [--poll-ms <ms>] [--debounce-ms <ms>]

Notes:
  - Requires running the app with `--hotpatch` (sets `FRET_HOTPATCH=1`).
  - The runner watches `FRET_HOTPATCH_TRIGGER_PATH` (default: `.fret/hotpatch.touch`).
  - `watch` is polling-based and ignores `target/`, `.git/`, `.fret/`, and `repo-ref/`."#
            );
            Ok(())
        }
        Some(other) => Err(format!("unknown hotpatch subcommand: {other}")),
    }
}

fn hotpatch_trigger_path(workspace_root: &Path) -> PathBuf {
    workspace_root.join(".fret").join("hotpatch.touch")
}

fn hotpatch_poke(path: Option<&str>) -> Result<(), String> {
    let root = workspace_root()?;
    let path = match path {
        Some(path) => resolve_workspace_relative(&root, path),
        None => hotpatch_trigger_path(&root),
    };
    ensure_hotpatch_trigger_file_poked(&path)?;
    println!("{}", path.display());
    Ok(())
}

fn ensure_hotpatch_trigger_file_initialized(path: &Path) -> Result<(), String> {
    if path.is_file() {
        return Ok(());
    }
    ensure_hotpatch_trigger_file_poked(path)
}

fn ensure_hotpatch_trigger_file_poked(path: &Path) -> Result<(), String> {
    let dir = path
        .parent()
        .ok_or_else(|| "invalid hotpatch path".to_string())?;
    std::fs::create_dir_all(dir).map_err(|e| e.to_string())?;

    let now = std::time::SystemTime::now()
        .duration_since(std::time::UNIX_EPOCH)
        .map_err(|e| e.to_string())?;
    let marker = format!("{}", now.as_nanos());

    std::fs::write(&path, marker).map_err(|e| e.to_string())?;
    Ok(())
}

fn hotpatch_watch(args: Vec<String>) -> Result<(), String> {
    let root = workspace_root()?;

    let mut watch_paths: Vec<String> = Vec::new();
    let mut trigger_path: Option<String> = None;
    let mut poll_ms: u64 = 500;
    let mut debounce_ms: u64 = 200;

    let mut it = args.into_iter();
    while let Some(a) = it.next() {
        match a.as_str() {
            "--path" => {
                watch_paths.push(
                    it.next()
                        .ok_or_else(|| "--path requires a value".to_string())?,
                );
            }
            "--trigger-path" => {
                trigger_path = Some(
                    it.next()
                        .ok_or_else(|| "--trigger-path requires a value".to_string())?,
                );
            }
            "--poll-ms" => {
                let raw = it
                    .next()
                    .ok_or_else(|| "--poll-ms requires a value".to_string())?;
                poll_ms = raw.parse::<u64>().map_err(|e| e.to_string())?;
            }
            "--debounce-ms" => {
                let raw = it
                    .next()
                    .ok_or_else(|| "--debounce-ms requires a value".to_string())?;
                debounce_ms = raw.parse::<u64>().map_err(|e| e.to_string())?;
            }
            "--help" | "-h" => return Ok(()),
            other => return Err(format!("unknown argument for hotpatch watch: {other}")),
        }
    }

    let trigger_path = trigger_path.as_deref().unwrap_or(".fret/hotpatch.touch");
    let trigger_path = resolve_workspace_relative(&root, trigger_path);
    ensure_hotpatch_trigger_file_initialized(&trigger_path)?;

    let watch_roots: Vec<PathBuf> = if watch_paths.is_empty() {
        vec![
            root.join("crates"),
            root.join("ecosystem"),
            root.join("apps"),
        ]
    } else {
        watch_paths
            .iter()
            .map(|p| resolve_workspace_relative(&root, p))
            .collect()
    };

    eprintln!("Hotpatch watch: polling sources and poking trigger file on change");
    eprintln!("  trigger: {}", trigger_path.display());
    eprintln!("  poll_ms: {poll_ms}");
    eprintln!("  debounce_ms: {debounce_ms}");
    for p in &watch_roots {
        eprintln!("  watch: {}", p.display());
    }

    let mut last_sig = scan_watch_signature(&watch_roots)?;
    let mut last_poke_at: Option<std::time::Instant> = None;

    loop {
        std::thread::sleep(std::time::Duration::from_millis(poll_ms));
        let sig = scan_watch_signature(&watch_roots)?;
        if sig == last_sig {
            continue;
        }
        last_sig = sig;

        let now = std::time::Instant::now();
        if last_poke_at
            .is_some_and(|t| now.duration_since(t) < std::time::Duration::from_millis(debounce_ms))
        {
            continue;
        }

        ensure_hotpatch_trigger_file_poked(&trigger_path)?;
        last_poke_at = Some(now);
        eprintln!("poked: {}", trigger_path.display());
    }
}

fn scan_watch_signature(roots: &[PathBuf]) -> Result<u64, String> {
    let mut sig: u64 = 0;
    for root in roots {
        sig ^= scan_watch_root_signature(root)?;
    }
    Ok(sig)
}

fn scan_watch_root_signature(root: &Path) -> Result<u64, String> {
    let mut sig: u64 = 0;
    let mut stack: Vec<PathBuf> = vec![root.to_path_buf()];

    while let Some(path) = stack.pop() {
        let md = match std::fs::symlink_metadata(&path) {
            Ok(md) => md,
            Err(_) => continue,
        };

        let ft = md.file_type();
        if ft.is_symlink() {
            continue;
        }

        if ft.is_dir() {
            if should_skip_dir(&path) {
                continue;
            }
            let rd = match std::fs::read_dir(&path) {
                Ok(rd) => rd,
                Err(_) => continue,
            };
            for ent in rd {
                let ent = match ent {
                    Ok(ent) => ent,
                    Err(_) => continue,
                };
                stack.push(ent.path());
            }
            continue;
        }

        if !ft.is_file() {
            continue;
        }

        if !should_watch_file(&path) {
            continue;
        }

        let Some(mtime) = md.modified().ok() else {
            continue;
        };
        let ns = system_time_to_ns(mtime);
        let len = md.len();

        use std::hash::{Hash as _, Hasher as _};
        let mut h = std::collections::hash_map::DefaultHasher::new();
        path.hash(&mut h);
        ns.hash(&mut h);
        len.hash(&mut h);
        sig ^= h.finish();
    }

    Ok(sig)
}

fn should_skip_dir(path: &Path) -> bool {
    let Some(name) = path.file_name().and_then(|s| s.to_str()) else {
        return false;
    };
    matches!(name, "target" | ".git" | ".fret" | "repo-ref")
}

fn should_watch_file(path: &Path) -> bool {
    let Some(name) = path.file_name().and_then(|s| s.to_str()) else {
        return false;
    };
    if matches!(name, "Cargo.toml" | "Cargo.lock" | "rust-toolchain.toml") {
        return true;
    }

    let Some(ext) = path.extension().and_then(|s| s.to_str()) else {
        return false;
    };
    matches!(ext, "rs" | "toml" | "wgsl" | "md")
}

fn system_time_to_ns(t: std::time::SystemTime) -> u128 {
    t.duration_since(std::time::UNIX_EPOCH)
        .map(|d| d.as_nanos())
        .unwrap_or(0)
}

fn parse_hotpatch_path_arg(
    it: &mut impl Iterator<Item = String>,
) -> Result<Option<String>, String> {
    let mut path: Option<String> = None;
    while let Some(a) = it.next() {
        match a.as_str() {
            "--path" => {
                path = Some(
                    it.next()
                        .ok_or_else(|| "--path requires a value".to_string())?,
                );
            }
            "--help" | "-h" => return Ok(None),
            other => return Err(format!("unknown argument for hotpatch command: {other}")),
        }
    }
    Ok(path)
}

fn resolve_workspace_relative(workspace_root: &Path, raw: &str) -> PathBuf {
    let path = Path::new(raw);
    if path.is_absolute() {
        path.to_path_buf()
    } else {
        workspace_root.join(path)
    }
}

#[derive(Debug, Clone, Copy)]
enum HotpatchBuildIdArg {
    Auto,
    None,
    Value(u64),
}

fn parse_hotpatch_build_id(raw: &str) -> Result<HotpatchBuildIdArg, String> {
    match raw {
        "auto" => Ok(HotpatchBuildIdArg::Auto),
        "none" => Ok(HotpatchBuildIdArg::None),
        other => Ok(HotpatchBuildIdArg::Value(
            other.parse::<u64>().map_err(|e| e.to_string())?,
        )),
    }
}

fn generate_hotpatch_build_id() -> u64 {
    let now = std::time::SystemTime::now()
        .duration_since(std::time::UNIX_EPOCH)
        .unwrap_or_default();
    let nanos = now.as_nanos();
    let pid = std::process::id() as u64;
    (nanos as u64) ^ (pid.rotate_left(17)) ^ 0x6a09e667f3bcc909u64
}

fn web_demos() -> &'static [&'static str] {
    &[
        "components_gallery",
        "chart_demo",
        "plot_demo",
        "bars_demo",
        "grouped_bars_demo",
        "stacked_bars_demo",
        "area_demo",
        "candlestick_demo",
        "error_bars_demo",
        "heatmap_demo",
        "histogram_demo",
        "shaded_demo",
        "stairs_demo",
        "stems_demo",
        "linked_cursor_demo",
        "inf_lines_demo",
        "tags_demo",
        "drag_demo",
    ]
}

fn web_demos_as_vec() -> Vec<String> {
    web_demos().iter().copied().map(String::from).collect()
}

fn validate_web_demo(name: &str) -> Result<(), String> {
    if web_demos().iter().any(|d| *d == name) {
        return Ok(());
    }
    Err(format!(
        "unknown web demo `{name}`\n  try: fretboard list web-demos"
    ))
}

fn list_native_demos_from(workspace_root: &Path) -> Result<Vec<String>, String> {
    let bin_dir = workspace_root
        .join("apps")
        .join("fret-demo")
        .join("src")
        .join("bin");
    read_rs_stems(&bin_dir)
}

fn validate_native_demo(demos: &[String], name: &str) -> Result<(), String> {
    if demos.iter().any(|d| d == name) {
        return Ok(());
    }

    let mut hint = String::new();
    for d in demos {
        if d.contains(name) || name.contains(d) {
            hint = format!("\n  hint: did you mean `{d}`?");
            break;
        }
    }

    Err(format!(
        "unknown native demo `{name}`{hint}\n  try: fretboard list native-demos"
    ))
}

fn prompt_choose_demo(demos: &[String]) -> Result<String, String> {
    if demos.is_empty() {
        return Err("no native demos found".to_string());
    }

    eprintln!("Select a demo:");
    for (i, demo) in demos.iter().enumerate() {
        eprintln!("  {:>2}) {demo}", i + 1);
    }
    eprint!("Enter number or name: ");

    use std::io::Write as _;
    std::io::stdout().flush().map_err(|e| e.to_string())?;

    let mut input = String::new();
    std::io::stdin()
        .read_line(&mut input)
        .map_err(|e| e.to_string())?;
    let input = input.trim();
    if input.is_empty() {
        return Ok("components_gallery".to_string());
    }

    if let Ok(n) = input.parse::<usize>() {
        if n == 0 || n > demos.len() {
            return Err(format!("invalid selection: {n}"));
        }
        return Ok(demos[n - 1].clone());
    }

    validate_native_demo(demos, input)?;
    Ok(input.to_string())
}
