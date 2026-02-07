use fret_core::{ExternalDragFile, ExternalDragFiles, ExternalDropToken};

pub fn external_drag_files(
    token: ExternalDropToken,
    paths: &[std::path::PathBuf],
) -> ExternalDragFiles {
    let files = paths
        .iter()
        .map(|p| ExternalDragFile {
            name: p
                .file_name()
                .map(|n| n.to_string_lossy().to_string())
                .unwrap_or_else(|| p.to_string_lossy().to_string()),
        })
        .collect();
    ExternalDragFiles { token, files }
}
