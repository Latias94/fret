const ROOT_EXAMPLES: &str = r#"  fretboard assets manifest write --dir assets --out assets.manifest.json --app-bundle my-app
  fretboard assets rust write --dir assets --out src/generated_assets.rs --app-bundle my-app
  fretboard assets rust write --dir assets --out src/generated_assets.rs --app-bundle my-app --surface framework
  fretboard config menubar --path .
  fretboard diag latest
  fretboard diag run ./diag/dialog-escape.json --launch -- cargo run --manifest-path ./Cargo.toml
  fretboard dev native --manifest-path ./Cargo.toml
  fretboard dev web --manifest-path ./Cargo.toml --no-open
  fretboard icons acquire iconify-collection --collection mdi --icon home --out ./iconify/mdi-home.json
  fretboard icons suggest presentation-defaults --provenance ./iconify/mdi-home.provenance.json --out ./iconify/presentation-defaults.json
  fretboard icons import svg-dir --source ./icons --crate-name my-icons --vendor-namespace app --presentation-defaults ./presentation-defaults.json
  fretboard icons import svg-dir --source ./icons --crate-name my-icons --vendor-namespace app
  fretboard icons import svg-dir --source ./icons --crate-name my-icons --vendor-namespace app --semantic-aliases ./semantic-aliases.json
  fretboard icons import iconify-collection --source ./iconify/lucide.json --crate-name lucide-icons --vendor-namespace lucide
  fretboard new hello --name hello-world
  fretboard new simple-todo --name my-simple-todo
  fretboard new todo --name my-todo"#;

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

pub fn print_root_help() -> Result<(), String> {
    print!("{}", render_root_help()?);
    Ok(())
}

#[cfg(test)]
mod tests {
    use super::render_root_help;

    #[test]
    fn root_help_mentions_public_commands() {
        let help = render_root_help().expect("root help should render");
        assert!(help.contains("fretboard assets manifest write"));
        assert!(help.contains("fretboard config menubar --path ."));
        assert!(help.contains("fretboard diag latest"));
        assert!(help.contains("fretboard dev native --manifest-path ./Cargo.toml"));
        assert!(help.contains("fretboard icons acquire iconify-collection"));
        assert!(help.contains("fretboard icons suggest presentation-defaults"));
        assert!(help.contains("fretboard icons import svg-dir"));
        assert!(help.contains("--semantic-aliases ./semantic-aliases.json"));
        assert!(help.contains("--presentation-defaults ./presentation-defaults.json"));
        assert!(help.contains("fretboard icons import iconify-collection"));
        assert!(help.contains("fretboard new todo --name my-todo"));
    }
}
