use super::super::*;

pub(super) fn resolve_repro_scripts(
    rest: &[String],
    workspace_root: &Path,
) -> Result<(Vec<PathBuf>, Option<String>), String> {
    if rest.len() == 1 && rest[0] == "ui-gallery" {
        let inputs = diag_suite_scripts::ui_gallery_suite_scripts();
        let scripts = expand_script_inputs(workspace_root, &inputs)?;
        Ok((scripts, Some("ui-gallery".to_string())))
    } else if rest.len() == 1 && rest[0] == "ui-gallery-code-editor" {
        let inputs = diag_suite_scripts::ui_gallery_code_editor_suite_scripts();
        let scripts = expand_script_inputs(workspace_root, &inputs)?;
        Ok((scripts, Some("ui-gallery-code-editor".to_string())))
    } else if rest.len() == 1 && rest[0] == "docking-arbitration" {
        let inputs = diag_suite_scripts::docking_arbitration_suite_scripts();
        let scripts = expand_script_inputs(workspace_root, &inputs)?;
        Ok((scripts, Some("docking-arbitration".to_string())))
    } else {
        Ok((resolve_explicit_repro_paths(rest, workspace_root)?, None))
    }
}

pub(super) fn compute_required_caps(scripts: &[PathBuf]) -> Vec<String> {
    let mut required_caps: Vec<String> = Vec::new();
    for src in scripts.iter() {
        required_caps.extend(script_required_capabilities(src));
    }
    required_caps.sort();
    required_caps.dedup();
    required_caps
}

pub(super) fn merged_script_env_defaults(
    scripts: &[PathBuf],
) -> Result<Vec<(String, String)>, String> {
    use std::collections::BTreeMap;

    let mut defaults: BTreeMap<String, String> = BTreeMap::new();
    let mut conflicts: Vec<String> = Vec::new();
    for script in scripts {
        for (key, value) in script_env_defaults(script) {
            if let Some(prev) = defaults.insert(key.clone(), value.clone())
                && prev != value
            {
                conflicts.push(format!(
                    "meta.env_defaults conflict for {key}: {prev} vs {value} (script={})",
                    script.display()
                ));
            }
        }
    }

    if !conflicts.is_empty() {
        conflicts.sort();
        return Err(format!(
            "conflicting script meta.env_defaults in repro:\n- {}",
            conflicts.join("\n- ")
        ));
    }

    Ok(defaults.into_iter().collect())
}

fn resolve_explicit_repro_paths(
    rest: &[String],
    workspace_root: &Path,
) -> Result<Vec<PathBuf>, String> {
    let scripts: Vec<PathBuf> = rest
        .iter()
        .map(|p| resolve_path(workspace_root, PathBuf::from(p)))
        .collect();

    if rest.len() == 1 {
        let name = rest[0].as_str();
        let resolved = &scripts[0];
        if !resolved.exists() {
            let looks_like_suite_name =
                !name.contains(['/', '\\', ':']) && !name.ends_with(".json");
            if looks_like_suite_name {
                return Err(format!(
                    "unknown suite or script path: {name:?}\n\
hint: list suites via `fretboard-dev diag list suites --contains {name}`\n\
hint: list promoted scripts via `fretboard-dev diag list scripts --contains {name}`"
                ));
            }
            return Err(format!(
                "script path does not exist: {}",
                resolved.display()
            ));
        }
        return Ok(scripts);
    }

    if let Some(missing) = scripts.iter().find(|path| !path.exists()) {
        return Err(format!("script path does not exist: {}", missing.display()));
    }

    Ok(scripts)
}
