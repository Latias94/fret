use std::fs;
use std::path::{Path, PathBuf};

fn rust_sources(root: &Path, out: &mut Vec<PathBuf>) {
    let entries = fs::read_dir(root).expect("read_dir should succeed");
    for entry in entries {
        let entry = entry.expect("directory entry should be readable");
        let path = entry.path();
        if path.is_dir() {
            rust_sources(&path, out);
            continue;
        }
        if path.extension().is_some_and(|ext| ext == "rs") {
            out.push(path);
        }
    }
}

#[test]
fn ai_elements_avoid_flat_shadcn_root_imports() {
    let mut sources = Vec::new();
    rust_sources(
        &PathBuf::from(env!("CARGO_MANIFEST_DIR")).join("src/elements"),
        &mut sources,
    );

    for path in sources {
        let source = fs::read_to_string(&path).expect("source should be readable");
        for (line_idx, line) in source.lines().enumerate() {
            let trimmed = line.trim();
            if !trimmed.starts_with("use fret_ui_shadcn::") {
                continue;
            }

            let allowed = trimmed.starts_with("use fret_ui_shadcn::facade::")
                || trimmed.starts_with("use fret_ui_shadcn::raw::")
                || trimmed == "use fret_ui_shadcn::prelude::*;";
            assert!(
                allowed,
                "{}:{} reintroduced a flat fret_ui_shadcn import lane: {}",
                path.display(),
                line_idx + 1,
                trimmed
            );
        }
    }
}
