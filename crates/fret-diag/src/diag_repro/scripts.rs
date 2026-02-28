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
        Ok((
            rest.iter()
                .map(|p| resolve_path(workspace_root, PathBuf::from(p)))
                .collect(),
            None,
        ))
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
