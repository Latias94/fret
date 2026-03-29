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
        assert!(help.contains("fretboard diag perf ui-gallery"));
    }

    #[test]
    fn root_help_does_not_list_deleted_init_alias() {
        let help = render_root_help().expect("root help should render");
        assert!(!help.contains("fretboard init"));
        assert!(help.contains("fretboard new todo --name my-todo"));
    }
}
