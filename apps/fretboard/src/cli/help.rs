const ROOT_USAGE: &str = r#"  fretboard help
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
  fretboard dev web --open [--no-open] [--port <port>] [--demo <demo> | --choose]"#;

const ROOT_EXAMPLES: &str = r#"  fretboard assets manifest write --dir assets --out assets.manifest.json --app-bundle my-todo
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
  fretboard diag suite ui-gallery --launch -- cargo run -p fret-ui-gallery --release
  fretboard diag repro ui-gallery --launch -- cargo run -p fret-ui-gallery --release
  fretboard diag perf ui-gallery --launch -- cargo run -p fret-ui-gallery --release
  fretboard diag campaign list --lane smoke --tag ui-gallery --platform native
  fretboard dev native --hotpatch-devserver ws://127.0.0.1:8080/_dioxus
  fretboard dev native --bin hotpatch_smoke_demo --hotpatch
  fretboard dev native --bin hotpatch_smoke_demo --hotpatch-dx
  fretboard dev web --demo plot_demo
  fretboard dev web --demo custom_effect_v2_web_demo"#;

pub(crate) fn print_root_help() {
    println!(
        "fretboard dev tooling for the Fret workspace\n\nUsage:\n{ROOT_USAGE}\n\nExamples:\n{ROOT_EXAMPLES}"
    );
}

#[cfg(test)]
mod tests {
    use super::{ROOT_EXAMPLES, ROOT_USAGE};

    #[test]
    fn root_help_keeps_diag_examples_visible() {
        assert!(ROOT_USAGE.contains("fretboard diag --help"));
        assert!(ROOT_EXAMPLES.contains("fretboard diag perf ui-gallery"));
    }
}
