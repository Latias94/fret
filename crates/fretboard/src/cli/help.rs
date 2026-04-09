const ROOT_EXAMPLES: &str = r#"  fretboard assets manifest write --dir assets --out assets.manifest.json --app-bundle my-app
  fretboard assets rust write --dir assets --out src/generated_assets.rs --app-bundle my-app
  fretboard assets rust write --dir assets --out src/generated_assets.rs --app-bundle my-app --surface framework
  fretboard config menubar --path ."#;

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
    }
}
