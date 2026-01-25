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

    let fret_kit_path = join_workspace_path(workspace_prefix, "ecosystem/fret-kit");

    format!(
        r#"[package]
name = "{package_name}"
version = "0.1.0"
edition = "2024"

[dependencies]
anyhow = "1"
fret-kit = {{ path = "{fret_kit_path}", default-features = false, features = [{kit_features}] }}
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

    let fret_kit_path = join_workspace_path(workspace_prefix, "ecosystem/fret-kit");

    format!(
        r#"[package]
name = "{package_name}"
version = "0.1.0"
edition = "2024"

[dependencies]
anyhow = "1"
fret-kit = {{ path = "{fret_kit_path}", default-features = false, features = [{kit_features}] }}

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
        .on_click(CMD_ADD)
        .children([icon::icon(cx, IconId::new("lucide.plus"))])
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

use fret_kit::prelude::*;

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
    fret_kit::app_with_hooks("todo", init_window, view, |d| d.on_command(on_command))?
        .with_main_window("todo", (560.0, 520.0))
        .run()?;
    Ok(())
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
    let todos = cx
        .watch_model(&st.todos)
        .layout()
        .cloned()
        .unwrap_or_default();
    let draft_value = cx
        .watch_model(&st.draft)
        .layout()
        .cloned()
        .unwrap_or_default();

    let theme = Theme::global(&*cx.app).clone();

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
        |_cx| [input, add_btn],
    );

    let rows = stack::vstack(
        cx,
        stack::VStackProps::default()
            .layout(LayoutRefinement::default().w_full())
            .gap(Space::N3),
        |cx| todos.iter().map(|t| cx.keyed(t.id, |cx| todo_row(cx, &theme, t))),
    );

    let chrome = ChromeRefinement::default()
        .bg(ColorRef::Color(theme.color_required("background")))
        .rounded(Radius::Lg)
        .border_1()
        .border_color(ColorRef::Color(theme.color_required("border")));

    let card = shadcn::Card::new([
        shadcn::CardHeader::new([
            shadcn::CardTitle::new("Todo").into_element(cx),
            shadcn::CardDescription::new("A minimal Fret + shadcn template.").into_element(cx),
        ])
        .into_element(cx),
        shadcn::CardContent::new([
            stack::vstack(
                cx,
                stack::VStackProps::default()
                    .layout(LayoutRefinement::default().w_full())
                    .gap(Space::N4),
                |_cx| [input_row, rows],
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
        |cx| [stack::vstack(
                cx,
                stack::VStackProps::default()
                    .layout(LayoutRefinement::default().w_full().h_full())
                    .justify_center()
                    .items_center(),
                |_cx| [card],
            )],
    );

    vec![page]
}

fn todo_row(cx: &mut ElementContext<'_, App>, theme: &Theme, item: &TodoItem) -> AnyElement {
    let done = cx
        .watch_model(&item.done)
        .layout()
        .copied()
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

    cx.container(props, |cx| [stack::hstack(
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

            [
                stack::hstack(
                    cx,
                    stack::HStackProps::default()
                        .layout(LayoutRefinement::default().flex_1().min_w_0())
                        .gap(Space::N3)
                        .items_center(),
                    |_cx| [checkbox.clone(), label],
                ),
                remove_btn.clone(),
            ]
        },
    )])
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
        .replace("__ADD_BTN_DEF__", add_btn_def)
        .replace("__REMOVE_BTN_DEF__", remove_btn_def)
}

pub(super) fn todo_mvu_template_main_rs(_package_name: &str, opts: ScaffoldOptions) -> String {
    // Radix doesn't currently ship plus/trash icons in our curated set; keep the todo template
    // functional by falling back to text buttons when Lucide isn't selected.
    let has_action_icons = matches!(opts.icon_pack, IconPack::Lucide);

    let add_btn_def = if has_action_icons {
        r#"    let add_cmd = msg.cmd(Msg::Add);
    let add_btn = shadcn::Button::new("")
        .size(shadcn::ButtonSize::Icon)
        .disabled(!add_enabled)
        .on_click(add_cmd.clone())
        .children([icon::icon(cx, IconId::new("lucide.plus"))])
        .into_element(cx);
"#
    } else {
        r#"    let add_cmd = msg.cmd(Msg::Add);
    let add_btn = shadcn::Button::new("Add")
        .disabled(!add_enabled)
        .on_click(add_cmd.clone())
        .into_element(cx);
"#
    };

    let remove_btn_def = if has_action_icons {
        r#"    let remove_btn = shadcn::Button::new("")
        .size(shadcn::ButtonSize::Icon)
        .variant(shadcn::ButtonVariant::Ghost)
        .on_click(remove_cmd.clone())
        .children([icon::icon(cx, IconId::new("lucide.trash"))])
        .into_element(cx);
"#
    } else {
        r#"    let remove_btn = shadcn::Button::new("Remove")
        .variant(shadcn::ButtonVariant::Ghost)
        .on_click(remove_cmd.clone())
        .into_element(cx);
"#
    };

    const TEMPLATE: &str = r#"use std::sync::Arc;

use fret_kit::prelude::*;

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
    Remove(u64),
    ClearDone,
}

fn main() -> anyhow::Result<()> {
    fret_kit::mvu::app::<TodoProgram>("todo-mvu")?
        .with_main_window("todo-mvu", (560.0, 520.0))
        .run()?;
    Ok(())
}

struct TodoProgram;

impl MvuProgram for TodoProgram {
    type State = TodoState;
    type Message = Msg;

    fn init(app: &mut App, _window: AppWindowId) -> Self::State {
        let done_1 = app.models_mut().insert(false);
        let done_2 = app.models_mut().insert(true);
        let todos = app.models_mut().insert(vec![
            TodoItem {
                id: 1,
                done: done_1,
                text: Arc::from("Try the MVU authoring surface"),
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
                let _ = app.models_mut().update(&st.todos, |v| v.insert(0, item));
                let _ = app.models_mut().update(&st.draft, |s| s.clear());
            }
            Msg::Remove(id) => {
                let _ = app.models_mut().update(&st.todos, |todos| {
                    todos.retain(|t| t.id != id);
                });
            }
            Msg::ClearDone => {
                let snapshot = app.models().read(&st.todos, |v| v.clone()).unwrap_or_default();
                let mut keep: Vec<TodoItem> = Vec::new();
                for t in snapshot {
                    let done = app.models().read(&t.done, |v| *v).ok().unwrap_or(false);
                    if !done {
                        keep.push(t);
                    }
                }
                let _ = app.models_mut().update(&st.todos, |todos| *todos = keep);
            }
        }
    }

    fn view(
        cx: &mut ElementContext<'_, App>,
        st: &mut Self::State,
        msg: &mut MessageRouter<Self::Message>,
    ) -> Vec<AnyElement> {
        let todos = cx
            .watch_model(&st.todos)
            .layout()
            .cloned()
            .unwrap_or_default();
        for t in &todos {
            cx.watch_model(&t.done).layout().observe();
        }

        let theme = Theme::global(&*cx.app).clone();

        let draft_value = cx
            .watch_model(&st.draft)
            .layout()
            .cloned()
            .unwrap_or_default();
        let add_enabled = !draft_value.trim().is_empty();
__ADD_BTN_DEF__

        let input = shadcn::Input::new(st.draft.clone())
            .placeholder("Add a task…")
            .submit_command(add_cmd)
            .into_element(cx);

        let input_row = stack::hstack(
            cx,
            stack::HStackProps::default()
                .layout(LayoutRefinement::default().w_full())
                .gap(Space::N2)
                .items_center(),
            |_cx| [input, add_btn],
        );

        let rows = stack::vstack(
            cx,
            stack::VStackProps::default()
                .layout(LayoutRefinement::default().w_full())
                .gap(Space::N3),
            |cx| todos.iter().map(|t| cx.keyed(t.id, |cx| todo_row(cx, &theme, msg, t))),
        );

        let clear_done = shadcn::Button::new("Clear done")
            .variant(shadcn::ButtonVariant::Secondary)
            .on_click(msg.cmd(Msg::ClearDone))
            .into_element(cx);

        let chrome = ChromeRefinement::default()
            .bg(ColorRef::Color(theme.color_required("background")))
            .rounded(Radius::Lg)
            .border_1()
            .border_color(ColorRef::Color(theme.color_required("border")));

        let card = shadcn::Card::new([
            shadcn::CardHeader::new([
                shadcn::CardTitle::new("Todo (MVU)").into_element(cx),
                shadcn::CardDescription::new("Typed messages via MessageRouter.")
                    .into_element(cx),
            ])
            .into_element(cx),
            shadcn::CardContent::new([
                stack::vstack(
                    cx,
                    stack::VStackProps::default()
                        .layout(LayoutRefinement::default().w_full())
                        .gap(Space::N4),
                    |_cx| [input_row, rows, clear_done],
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
            |cx| [stack::vstack(
                    cx,
                    stack::VStackProps::default()
                        .layout(LayoutRefinement::default().w_full().h_full())
                        .justify_center()
                        .items_center(),
                    |_cx| [card],
                )],
        );

        vec![page]
    }
}

fn todo_row(
    cx: &mut ElementContext<'_, App>,
    theme: &Theme,
    msg: &mut MessageRouter<Msg>,
    item: &TodoItem,
) -> AnyElement {
    let done = cx
        .watch_model(&item.done)
        .layout()
        .copied()
        .unwrap_or(false);

    let checkbox = shadcn::Checkbox::new(item.done.clone()).into_element(cx);
    let remove_cmd = msg.cmd(Msg::Remove(item.id));
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

    cx.container(props, |cx| [stack::hstack(
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

                [
                    stack::hstack(
                        cx,
                        stack::HStackProps::default()
                            .layout(LayoutRefinement::default().flex_1().min_w_0())
                            .gap(Space::N3)
                            .items_center(),
                        |_cx| [checkbox.clone(), label],
                    ),
                    remove_btn.clone(),
                ]
            },
        )])
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
        r#"use fret_kit::prelude::*;

const CMD_CLICK: &str = "hello.click";

fn main() -> anyhow::Result<()> {{
    fret_kit::app_with_hooks("{package_name}", init_window, view, |d| d.on_command(on_command))?
        .with_main_window("{package_name}", (560.0, 360.0))
        .run()?;
    Ok(())
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
            [
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
    .replace("__PALETTE_BUTTON__", palette_button)
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
        "- UI assets: enabled (`fret-kit/ui-assets`)\n"
    } else {
        "- UI assets: disabled (use `fretboard new todo --ui-assets` if you need images/SVG caches)\n"
    };

    let icons_line = match opts.icon_pack {
        IconPack::Lucide => "- Icons: Lucide (`fret-kit/icons-lucide`)\n",
        IconPack::Radix => "- Icons: Radix (`fret-kit/icons-radix`)\n",
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
- If you want hotpatch later, keep commands/IDs stable and prefer the `fret_kit::app_with_hooks` golden path (ADR 0107 / 0112).
"#
    )
}

pub(super) fn todo_mvu_template_readme_md(package_name: &str, opts: ScaffoldOptions) -> String {
    let ui_assets_line = if opts.ui_assets {
        "- UI assets: enabled (`fret-kit/ui-assets`)\n"
    } else {
        "- UI assets: disabled (use `fretboard new todo-mvu --ui-assets` if you need images/SVG caches)\n"
    };

    let icons_line = match opts.icon_pack {
        IconPack::Lucide => "- Icons: Lucide (`fret-kit/icons-lucide`)\n",
        IconPack::Radix => "- Icons: Radix (`fret-kit/icons-radix`)\n",
        IconPack::None => "- Icons: disabled\n",
    };

    let palette_line = if opts.command_palette {
        "- Command palette: enabled (Cmd/Ctrl+Shift+P)\n"
    } else {
        "- Command palette: disabled\n"
    };

    format!(
        r#"# {package_name}

Generated by `fretboard new todo-mvu`.

This template uses `fret_kit::mvu` (typed `State` + `Message`) to avoid stringly `CommandId`
parsing in app code.

## Run

```bash
cargo run
```

## Notes

- Theme: shadcn app integration via `fret-kit` (use `shadcn::shadcn_themes::apply_shadcn_new_york_v4` for an explicit theme)
{icons_line}{palette_line}
{ui_assets_line}
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

pub(super) fn hello_template_readme_md(package_name: &str, opts: ScaffoldOptions) -> String {
    let icons_line = match opts.icon_pack {
        IconPack::Lucide => "- Icons: Lucide (`fret-kit/icons-lucide`)\n",
        IconPack::Radix => "- Icons: Radix (`fret-kit/icons-radix`)\n",
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
