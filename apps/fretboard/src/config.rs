use std::path::PathBuf;

pub(crate) fn config_cmd(args: Vec<String>) -> Result<(), String> {
    let mut it = args.into_iter();
    let Some(target) = it.next() else {
        return Err("missing config target (try: fretboard config menubar)".to_string());
    };

    match target.as_str() {
        "--help" | "-h" => crate::cli::help(),
        "menubar" => menubar_cmd(it.collect()),
        other => Err(format!("unknown config target: {other}")),
    }
}

fn menubar_cmd(args: Vec<String>) -> Result<(), String> {
    let mut project_root: Option<PathBuf> = None;
    let mut force = false;

    let mut it = args.into_iter();
    while let Some(a) = it.next() {
        match a.as_str() {
            "--path" => {
                let raw = it
                    .next()
                    .ok_or_else(|| "--path requires a value".to_string())?;
                project_root = Some(PathBuf::from(raw));
            }
            "--force" => force = true,
            "--help" | "-h" => return crate::cli::help(),
            other => return Err(format!("unknown argument for config menubar: {other}")),
        }
    }

    let project_root = match project_root {
        Some(p) => p,
        None => std::env::current_dir().map_err(|e| e.to_string())?,
    };

    let fret_dir = project_root.join(".fret");
    std::fs::create_dir_all(&fret_dir).map_err(|e| e.to_string())?;

    let path = fret_dir.join("menubar.json");
    if path.exists() && !force {
        return Err(format!(
            "refusing to overwrite existing file: {} (use --force)",
            path.display()
        ));
    }

    let template = serde_json::json!({
        "menu_bar_version": 2,
        "ops": [
            {
                "type": "append_menu",
                "title": "Custom",
                "items": [
                    { "type": "command", "command": "app.command_palette" },
                    { "type": "separator" },
                    { "type": "command", "command": "edit.copy" },
                    { "type": "command", "command": "edit.paste" },
                    { "type": "command", "command": "edit.select_all" }
                ]
            }
        ]
    });

    let contents = serde_json::to_string_pretty(&template).map_err(|e| e.to_string())?;
    std::fs::write(&path, contents).map_err(|e| e.to_string())?;
    println!("wrote {}", path.display());
    Ok(())
}
