use std::path::PathBuf;

pub(crate) mod contracts;

use self::contracts::{ConfigCommandArgs, ConfigTargetContract};

pub(crate) fn run_config_contract(args: ConfigCommandArgs) -> Result<(), String> {
    match args.target {
        ConfigTargetContract::Menubar(args) => menubar_cmd(args.path, args.force),
    }
}

fn menubar_cmd(project_root: Option<PathBuf>, force: bool) -> Result<(), String> {
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

#[cfg(test)]
mod tests {
    use super::contracts::{ConfigCommandArgs, ConfigMenubarCommandArgs, ConfigTargetContract};
    use super::run_config_contract;

    fn make_temp_dir(prefix: &str) -> std::path::PathBuf {
        let nonce = std::time::SystemTime::now()
            .duration_since(std::time::UNIX_EPOCH)
            .expect("system clock should be after unix epoch")
            .as_nanos();
        let dir = std::env::temp_dir().join(format!("{prefix}-{nonce}"));
        std::fs::create_dir_all(&dir).expect("create temp dir");
        dir
    }

    #[test]
    fn config_menubar_contract_writes_default_template() {
        let project_root = make_temp_dir("fretboard-config-menubar");
        run_config_contract(ConfigCommandArgs {
            target: ConfigTargetContract::Menubar(ConfigMenubarCommandArgs {
                path: Some(project_root.clone()),
                force: false,
            }),
        })
        .expect("config menubar should write the template");

        let path = project_root.join(".fret").join("menubar.json");
        let contents = std::fs::read_to_string(path).expect("menubar config should exist");
        assert!(contents.contains("\"menu_bar_version\": 2"));
        assert!(contents.contains("\"app.command_palette\""));
    }
}
