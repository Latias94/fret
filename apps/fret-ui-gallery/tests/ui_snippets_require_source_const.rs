use std::ffi::OsStr;
use std::fs;
use std::path::{Path, PathBuf};

fn visit_rs_files(dir: &Path, files: &mut Vec<PathBuf>) {
    let entries =
        fs::read_dir(dir).unwrap_or_else(|e| panic!("read_dir failed for {}: {e}", dir.display()));

    for entry in entries {
        let entry = entry.expect("read_dir entry");
        let path = entry.path();
        if path.is_dir() {
            visit_rs_files(&path, files);
            continue;
        }

        if path.extension() == Some(OsStr::new("rs")) {
            files.push(path);
        }
    }
}

#[test]
fn ui_snippets_export_source_constant() {
    let snippets_dir = Path::new(env!("CARGO_MANIFEST_DIR")).join("src/ui/snippets");
    let allowlist: [&str; 0] = [];

    let mut files = Vec::new();
    visit_rs_files(&snippets_dir, &mut files);

    for path in files {
        if path.file_name() == Some(OsStr::new("mod.rs")) {
            continue;
        }

        let file_name = path
            .file_name()
            .and_then(|s| s.to_str())
            .unwrap_or_default();
        if allowlist.contains(&file_name) {
            continue;
        }

        let src = fs::read_to_string(&path)
            .unwrap_or_else(|e| panic!("read_to_string failed for {}: {e}", path.display()));

        assert!(
            src.contains("pub const SOURCE:"),
            "UI Gallery snippet does not export a `SOURCE` const: {}\n\
             Exporting `SOURCE` avoids refactor-prone relative includes and keeps Preview ≡ Code \
             wiring uniform across pages.",
            path.display()
        );
    }
}
