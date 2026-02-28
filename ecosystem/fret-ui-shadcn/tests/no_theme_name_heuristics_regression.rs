use std::ffi::OsStr;
use std::fs;
use std::path::{Path, PathBuf};

fn visit_rs_files(dir: &Path, out: &mut Vec<PathBuf>) {
    let Ok(entries) = fs::read_dir(dir) else {
        return;
    };
    for entry in entries.flatten() {
        let path = entry.path();
        if path.is_dir() {
            visit_rs_files(&path, out);
            continue;
        }
        if path.extension() == Some(OsStr::new("rs")) {
            out.push(path);
        }
    }
}

#[test]
fn shadcn_src_does_not_reintroduce_theme_name_heuristics() {
    let crate_root = Path::new(env!("CARGO_MANIFEST_DIR"));
    let src_dir = crate_root.join("src");

    let mut files = Vec::new();
    visit_rs_files(&src_dir, &mut files);
    files.sort();

    let patterns = &[
        "theme.name.contains(",
        "theme.name.starts_with(",
        "theme.name.ends_with(",
    ];

    let mut hits: Vec<String> = Vec::new();
    for file in files {
        let Ok(text) = fs::read_to_string(&file) else {
            continue;
        };
        for pat in patterns {
            if text.contains(pat) {
                let rel = file
                    .strip_prefix(crate_root)
                    .unwrap_or(&file)
                    .display()
                    .to_string();
                hits.push(format!("{rel}: contains `{pat}`"));
            }
        }
    }

    assert!(
        hits.is_empty(),
        "Theme name heuristics reintroduced in src/ (prefer Theme metadata + component-owned variant keys):\n{}",
        hits.join("\n")
    );
}
