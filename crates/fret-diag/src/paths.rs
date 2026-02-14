use std::collections::BTreeSet;
use std::path::{Path, PathBuf};

pub(crate) fn resolve_path(workspace_root: &Path, path: PathBuf) -> PathBuf {
    if path.is_absolute() {
        path
    } else {
        workspace_root.join(path)
    }
}

fn normalize_host_path_separators(path: PathBuf) -> PathBuf {
    #[cfg(windows)]
    {
        return PathBuf::from(path.to_string_lossy().replace('/', "\\"));
    }
    #[cfg(not(windows))]
    {
        path
    }
}

pub(crate) fn expand_script_inputs(
    workspace_root: &Path,
    inputs: &[String],
) -> Result<Vec<PathBuf>, String> {
    let mut set: BTreeSet<PathBuf> = BTreeSet::new();

    for input in inputs {
        let resolved = resolve_path(workspace_root, PathBuf::from(input));

        // Directory input: treat as recursive `**/*.json` to support suite-like workflows.
        if resolved.is_dir() {
            let mut pattern = resolved.to_string_lossy().to_string();
            pattern = pattern.replace('\\', "/");
            if !pattern.ends_with('/') {
                pattern.push('/');
            }
            pattern.push_str("**/*.json");

            let mut any = false;
            for entry in glob::glob(&pattern).map_err(|e| e.to_string())? {
                let path = entry.map_err(|e| e.to_string())?;
                set.insert(normalize_host_path_separators(path));
                any = true;
            }
            if !any {
                return Err(format!(
                    "script input matched no files: {input} ({pattern})"
                ));
            }
            continue;
        }

        // Wildcard input: expand via glob. (PowerShell doesn't always expand globs for child args.)
        if input.contains('*') || input.contains('?') || input.contains('[') {
            let mut pattern = resolved.to_string_lossy().to_string();
            pattern = pattern.replace('\\', "/");

            let mut any = false;
            for entry in glob::glob(&pattern).map_err(|e| e.to_string())? {
                let path = entry.map_err(|e| e.to_string())?;
                set.insert(normalize_host_path_separators(path));
                any = true;
            }
            if !any {
                return Err(format!(
                    "script input matched no files: {input} ({pattern})"
                ));
            }
            continue;
        }

        set.insert(resolved);
    }

    Ok(set.into_iter().collect())
}
