use std::path::PathBuf;
use std::process::ExitCode;

pub(crate) fn main() -> ExitCode {
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
        "assets" => crate::assets::assets_cmd(args.collect()),
        "init" => crate::scaffold::init_cmd(args.collect()),
        "new" => crate::scaffold::new_cmd(args.collect()),
        "config" => crate::config::config_cmd(args.collect()),
        "theme" => crate::theme::theme_cmd(args.collect()),
        "hotpatch" => crate::hotpatch::hotpatch_cmd(args.collect()),
        "diag" => crate::diag::diag_cmd(args.collect()),
        "list" => match args.next().as_deref() {
            Some("native-demos") => crate::demos::list_native_demos(args.collect()),
            Some("web-demos") => crate::demos::list_web_demos(args.collect()),
            Some("cookbook-examples") => crate::demos::list_cookbook_examples(args.collect()),
            Some(other) => Err(format!("unknown list target: {other}")),
            None => Err("missing list target (try: list native-demos)".to_string()),
        },
        "dev" => match args.next().as_deref() {
            Some("native") => crate::dev::dev_native(args.collect()),
            Some("web") => crate::dev::dev_web(args.collect()),
            Some(other) => Err(format!("unknown dev target: {other}")),
            None => Err("missing dev target (try: dev native)".to_string()),
        },
        other => Err(format!("unknown command: {other}")),
    }
}

pub(crate) fn help() -> Result<(), String> {
    println!(
        r#"fretboard dev tooling for the Fret workspace

Usage:
  fretboard help
  fretboard assets manifest write --dir <dir> --out <path> (--app-bundle <name> | --package-bundle <name> | --bundle <id>) [--force]
  fretboard assets rust write --dir <dir> --out <path> (--app-bundle <name> | --package-bundle <name> | --bundle <id>) [--surface <fret|framework>] [--crate-root <dir>] [--force]
  fretboard new [template] [--path <path>] [--name <name>] [--ui-assets] [--icons <lucide|radix|none>] [--command-palette] [--no-check]
  fretboard new             # interactive wizard
  fretboard new hello       # rung 1: smallest runnable UI
  fretboard new simple-todo # rung 2: recommended starter
  fretboard new todo        # rung 3: selector/query follow-up
  fretboard new empty       # minimal Cargo-like project
  fretboard init <template> [...]    # alias for `new` (compat)
  fretboard config menubar [--path <path>] [--force]
  fretboard theme import-vscode <theme.json> [--out <path>] [--base <path>] [--all-tags] [--map <path>] [--set <key=value>...] [--report <path>] [--force]
  fretboard hotpatch poke [--path <path>]        # dev-only (experimental)
  fretboard hotpatch path [--path <path>]        # dev-only (experimental)
  fretboard hotpatch status [--tail <n>]         # dev-only (experimental)
  fretboard hotpatch watch [--path <path>...] [--trigger-path <path>] [--poll-ms <ms>] [--debounce-ms <ms>]  # dev-only (experimental)
  fretboard diag --help
  fretboard diag <subcommand> --help   # diagnostics help is generated from the executable contract
  fretboard list native-demos [--all]
  fretboard list web-demos
  fretboard list cookbook-examples
  fretboard dev native [--bin <name> | --choose [--all]] [--profile <cargo_profile>] [--hotpatch] [--hotpatch-reload] [--hotpatch-trigger-path <path>] [--hotpatch-poll-ms <ms>] [-- <args...>]
  fretboard dev native --demo <demo> [--profile <cargo_profile>] [--dev-state-reset] [--hotpatch|--watch] [-- <args...>]
  fretboard dev native --example <name> [--profile <cargo_profile>] [-- <args...>]
  fretboard dev native [--bin <name> | --choose [--all]] [--profile <cargo_profile>] [--hotpatch] [--no-supervise] [-- <args...>]
  fretboard dev native [--bin <name> | --choose [--all]] [--profile <cargo_profile>] [--hotpatch] [--watch] [--watch-poll-ms <ms>] [--no-watch] [--dev-state-reset] [-- <args...>]
  fretboard dev native [--bin <name> | --choose [--all]] [--profile <cargo_profile>] --hotpatch-devserver <ws_endpoint> [--hotpatch-build-id <auto|none|u64>] [-- <args...>]
  fretboard dev native [--bin <name> | --choose [--all]] [--profile <cargo_profile>] --hotpatch-dx [--hotpatch-dx-ws <ws_endpoint>] [--hotpatch-build-id <auto|none|u64>] [-- <args...>]
  fretboard dev web [--no-open] [--port <port>] [--demo <demo> | --choose] [--devtools-ws-url <ws://.../> --devtools-token <token>]
  fretboard dev web --open [--no-open] [--port <port>] [--demo <demo> | --choose]

Examples:
  fretboard assets manifest write --dir assets --out assets.manifest.json --app-bundle my-todo
  fretboard assets rust write --dir assets --out src/generated_assets.rs --app-bundle my-todo
    # `--surface fret` modules expose both `register(app)` and `mount(builder)`
  fretboard assets rust write --dir assets --out src/generated_assets.rs --app-bundle my-todo --surface framework
    # `--surface framework` modules expose `register(host)` for direct runtime mounting
  fretboard new hello --name hello-world
  fretboard new simple-todo --name my-simple-todo
  fretboard new todo --name my-todo
    # onboarding ladder: hello -> simple-todo (recommended starter) -> todo (selector/query follow-up)
  fretboard new hello --name hello-world --command-palette
  fretboard new todo --name my-todo --icons none
  fretboard new empty --name my-app
  fretboard config menubar --path .
  fretboard dev native --bin components_gallery
  fretboard dev native --bin todo_demo
  fretboard dev native --bin assets_demo
  fretboard dev native --bin hotpatch_smoke_demo
  fretboard dev native --demo simple-todo
  fretboard dev native --demo simple-todo --hotpatch
  fretboard dev native --choose
  fretboard dev native --bin image_upload_demo -- --help
  fretboard dev native --hotpatch --choose   # hotpatch (prefers `dx serve --hotpatch` when `dx` is available; otherwise falls back to reload-boundary mode)
  fretboard dev native --hotpatch-reload --choose   # file-triggered runner reload boundary (default: `.fret/hotpatch.touch`)
  fretboard hotpatch poke                   # updates `.fret/hotpatch.touch` (triggers a reload)
  fretboard hotpatch status                 # prints hotpatch-related log tails (runner/bootstrap)
  fretboard hotpatch watch                  # polls workspace sources and auto-pokes on change
  fretboard diag --help
  fretboard diag poke
  fretboard diag latest
  fretboard diag run tools/diag-scripts/todo-baseline.json --dir target/fret-diag-todo-auto --launch -- cargo run -p fret-demo --bin todo_demo
  fretboard diag perf ui-gallery --launch -- cargo run -p fret-ui-gallery --release
  fretboard diag campaign list --lane smoke --tag ui-gallery --platform native
  fretboard dev native --hotpatch-devserver ws://127.0.0.1:8080/_dioxus
  fretboard dev native --bin hotpatch_smoke_demo --hotpatch
  fretboard dev native --bin hotpatch_smoke_demo --hotpatch-dx
  fretboard dev web --demo plot_demo
  fretboard dev web --demo custom_effect_v2_web_demo
"#
    );
    Ok(())
}

pub(crate) fn workspace_root() -> Result<PathBuf, String> {
    let cwd = std::env::current_dir().map_err(|e| e.to_string())?;
    for dir in cwd.ancestors() {
        if dir.join("Cargo.toml").is_file() {
            return Ok(dir.to_path_buf());
        }
    }
    Err("failed to locate workspace root (Cargo.toml not found in ancestors)".to_string())
}
