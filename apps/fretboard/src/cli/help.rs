const ROOT_EXAMPLES: &str = r#"  fretboard-dev assets manifest write --dir assets --out assets.manifest.json --app-bundle my-todo
  fretboard-dev assets rust write --dir assets --out src/generated_assets.rs --app-bundle my-todo
    # `--surface fret` modules expose both `register(app)` and `mount(builder)`
  fretboard-dev assets rust write --dir assets --out src/generated_assets.rs --app-bundle my-todo --surface framework
    # `--surface framework` modules expose `register(host)` for direct runtime mounting
  fretboard-dev new hello --name hello-world
  fretboard-dev new simple-todo --name my-simple-todo
  fretboard-dev new todo --name my-todo
    # onboarding ladder: hello -> simple-todo (recommended starter) -> todo (selector/query follow-up)
  fretboard-dev new hello --name hello-world --command-palette
  fretboard-dev new todo --name my-todo --icons none
  fretboard-dev new empty --name my-app
  fretboard-dev config menubar --path .
  fretboard-dev dev native --bin components_gallery
  fretboard-dev dev native --bin todo_demo
  fretboard-dev dev native --bin assets_demo
  fretboard-dev dev native --bin hotpatch_smoke_demo
    # dev launches default to strict runtime diagnostics; add `--no-strict-runtime` to inspect recovery paths
  fretboard-dev dev native --demo simple-todo
  fretboard-dev dev native --demo simple-todo --hotpatch
  fretboard-dev dev native --choose
  fretboard-dev dev native --bin image_upload_demo -- --help
  fretboard-dev dev native --hotpatch --choose   # hotpatch (prefers `dx serve --hotpatch` when `dx` is available; otherwise falls back to reload-boundary mode)
  fretboard-dev dev native --hotpatch-reload --choose   # file-triggered runner reload boundary (default: `.fret/hotpatch.touch`)
  fretboard-dev hotpatch poke                   # updates `.fret/hotpatch.touch` (triggers a reload)
  fretboard-dev hotpatch status                 # prints hotpatch-related log tails (runner/bootstrap)
  fretboard-dev hotpatch watch                  # polls workspace sources and auto-pokes on change
  fretboard-dev diag --help
  fretboard-dev diag poke
  fretboard-dev diag latest
  fretboard-dev diag run tools/diag-scripts/todo-baseline.json --dir target/fret-diag-todo-auto --launch -- cargo run -p fret-demo --bin todo_demo
  fretboard-dev diag suite ui-gallery --launch -- cargo run -p fret-ui-gallery --release
  fretboard-dev diag repro ui-gallery --launch -- cargo run -p fret-ui-gallery --release
  fretboard-dev diag perf ui-gallery --launch -- cargo run -p fret-ui-gallery --release
  fretboard-dev diag campaign list --lane smoke --tag ui-gallery --platform native
  fretboard-dev dev native --hotpatch-devserver ws://127.0.0.1:8080/_dioxus
  fretboard-dev dev native --bin hotpatch_smoke_demo --hotpatch
  fretboard-dev dev native --bin hotpatch_smoke_demo --hotpatch-dx
  fretboard-dev dev web --demo plot_demo
  fretboard-dev dev web --demo custom_effect_v2_web_demo"#;

fn render_root_help() -> Result<String, String> {
    let mut help = super::contracts::render_command_help_path(&[])?;
    if !help.ends_with('\n') {
        help.push('\n');
    }
    help.push_str("\nExamples:\n");
    help.push_str(ROOT_EXAMPLES);
    help.push('\n');
    Ok(help)
}

pub(crate) fn print_root_help() -> Result<(), String> {
    print!("{}", render_root_help()?);
    Ok(())
}

#[cfg(test)]
mod tests {
    use super::render_root_help;

    #[test]
    fn root_help_keeps_diag_examples_visible() {
        let help = render_root_help().expect("root help should render");
        assert!(help.contains("diag"));
        assert!(help.contains("fretboard-dev diag perf ui-gallery"));
    }

    #[test]
    fn root_help_does_not_list_deleted_init_alias() {
        let help = render_root_help().expect("root help should render");
        assert!(!help.contains("fretboard-dev init"));
        assert!(help.contains("fretboard-dev new todo --name my-todo"));
    }
}
