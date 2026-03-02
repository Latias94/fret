use std::ffi::OsStr;
use std::fs;
use std::path::Path;

fn has_relative_snippet_include_str(src: &str) -> bool {
    // Keep this check lightweight and heuristic:
    // - pages should reference snippet sources via `snippets::*::SOURCE`
    // - avoid `include_str!("../snippets/...")` to prevent path drift during refactors/moves
    //
    // This intentionally does not attempt to parse Rust.
    src.contains("include_str!(\"../snippets/")
}

#[test]
fn ui_pages_do_not_include_snippets_by_relative_path() {
    let pages_dir = Path::new(env!("CARGO_MANIFEST_DIR")).join("src/ui/pages");
    let allowlist: [&str; 0] = [];

    let entries = fs::read_dir(&pages_dir)
        .unwrap_or_else(|e| panic!("read_dir failed for {}: {e}", pages_dir.display()));

    for entry in entries {
        let entry = entry.expect("read_dir entry");
        let path = entry.path();
        if path.extension() != Some(OsStr::new("rs")) {
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
            !has_relative_snippet_include_str(&src),
            "UI Gallery page uses a relative snippet include: {}\n\
             Prefer `DocSection::code_rust_from_file_region(snippets::...::SOURCE, \"example\")` \
             to keep snippet paths drift-free during refactors.",
            path.display()
        );
    }
}
