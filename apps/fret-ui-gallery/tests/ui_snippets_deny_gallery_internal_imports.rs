use std::ffi::OsStr;
use std::fs;
use std::path::{Path, PathBuf};

fn find_first_internal_gallery_ref(src: &str) -> Option<&'static str> {
    // Snippet files are the copy/paste surface, so they should avoid importing UI Gallery
    // internals or repo-relative demo assets. Keep this check lightweight and heuristic; it
    // intentionally does not parse Rust.
    let patterns = [
        "use crate::driver::",
        "crate::driver::",
        "use crate::ui::",
        "crate::ui::",
        "use crate::spec::",
        "crate::spec::",
        "env!(\"CARGO_MANIFEST_DIR\")",
        "../../assets/",
        "Arc::<str>::from(\"textures/",
    ];
    patterns.into_iter().find(|p| src.contains(p))
}

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
fn ui_snippets_do_not_import_ui_gallery_internals() {
    let snippets_dir = Path::new(env!("CARGO_MANIFEST_DIR")).join("src/ui/snippets");
    let allowlist: [&str; 0] = [];

    let mut files = Vec::new();
    visit_rs_files(&snippets_dir, &mut files);

    for path in files {
        let file_name = path
            .file_name()
            .and_then(|s| s.to_str())
            .unwrap_or_default();
        if allowlist.contains(&file_name) {
            continue;
        }

        let src = fs::read_to_string(&path)
            .unwrap_or_else(|e| panic!("read_to_string failed for {}: {e}", path.display()));

        if let Some(pattern) = find_first_internal_gallery_ref(&src) {
            panic!(
                "UI Gallery snippet references gallery-only glue ({pattern}): {}\n\
                 Snippets are the copy/paste surface. Prefer user-facing imports like:\n\
                 - `use fret::AppComponentCx;`\n\
                 - `use fret_ui_shadcn::{{facade as shadcn, prelude::*}};`\n\
                 - self-contained demo assets via `fret_ui_assets::ImageSource::rgba8(...)`\n\
                 and inline any stable command ids as `const CMD_*: &str = ...`.\n\
                 If a demo depends on gallery-only glue, keep it in `apps/fret-ui-gallery/src/ui/previews/` \
                 and avoid showing it as a snippet-backed code tab.",
                path.display()
            );
        }
    }
}
