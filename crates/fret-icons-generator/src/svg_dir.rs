use std::collections::BTreeMap;
use std::path::{Path, PathBuf};

use crate::GeneratePackError;
use crate::contracts::{PresentationRenderMode, SvgDirectorySource};
use crate::fs::path_label;
use crate::naming::normalize_svg_icon_name;

#[derive(Debug, Clone, PartialEq, Eq, serde::Serialize)]
pub(crate) struct CollectedSvg {
    pub icon_name: String,
    pub source_relative_path: String,
    pub render_mode: PresentationRenderMode,
    #[serde(skip_serializing)]
    pub svg_bytes: Vec<u8>,
}

pub(crate) fn collect_svg_directory(
    source: &SvgDirectorySource,
) -> Result<Vec<CollectedSvg>, GeneratePackError> {
    if !source.dir.exists() {
        return Err(GeneratePackError::MissingSourceDirectory(
            source.dir.display().to_string(),
        ));
    }
    if !source.dir.is_dir() {
        return Err(GeneratePackError::SourcePathNotDirectory(
            source.dir.display().to_string(),
        ));
    }

    let mut svg_files = Vec::new();
    collect_svg_paths_recursive(&source.dir, &source.dir, &mut svg_files)?;
    svg_files.sort();

    if svg_files.is_empty() {
        return Err(GeneratePackError::NoSvgIconsFound(
            source.dir.display().to_string(),
        ));
    }

    let mut seen = BTreeMap::<String, String>::new();
    let mut collected = Vec::with_capacity(svg_files.len());
    for relative_path in svg_files {
        let icon_name = normalize_svg_icon_name(&relative_path)?;
        let relative_label = path_label(&relative_path);
        if let Some(previous) = seen.insert(icon_name.clone(), relative_label.clone()) {
            return Err(GeneratePackError::IconNameCollision {
                icon_name,
                first: previous,
                second: relative_label,
            });
        }

        let absolute_path = source.dir.join(&relative_path);
        let svg_bytes = std::fs::read(&absolute_path)?;
        collected.push(CollectedSvg {
            icon_name,
            source_relative_path: relative_label,
            render_mode: PresentationRenderMode::Mask,
            svg_bytes,
        });
    }

    collected.sort_by(|left, right| left.icon_name.cmp(&right.icon_name));
    Ok(collected)
}

fn collect_svg_paths_recursive(
    root: &Path,
    current: &Path,
    out: &mut Vec<PathBuf>,
) -> Result<(), GeneratePackError> {
    let mut entries = std::fs::read_dir(current)?
        .collect::<Result<Vec<_>, _>>()?
        .into_iter()
        .map(|entry| entry.path())
        .collect::<Vec<_>>();
    entries.sort();

    for path in entries {
        if path.is_dir() {
            collect_svg_paths_recursive(root, &path, out)?;
            continue;
        }
        let is_svg = path
            .extension()
            .and_then(|ext| ext.to_str())
            .is_some_and(|ext| ext.eq_ignore_ascii_case("svg"));
        if !is_svg {
            continue;
        }
        let relative = path
            .strip_prefix(root)
            .expect("source path should stay under root");
        out.push(relative.to_path_buf());
    }

    Ok(())
}
