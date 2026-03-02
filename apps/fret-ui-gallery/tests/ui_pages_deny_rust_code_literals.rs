use std::ffi::OsStr;
use std::fs;
use std::path::Path;

fn has_docsection_rust_code_literal(src: &str) -> bool {
    // We intentionally keep this check heuristic and dependency-free:
    // - migrated pages should use `code_rust_from_file_region(include_str!(...), "example")`
    // - legacy pages still use `.code("rust", r#"...")`
    //
    // This test is meant to prevent regressions on migrated pages, not to fully parse Rust.
    let mut cursor = 0usize;
    while let Some(rel) = src[cursor..].find(".code(") {
        let abs = cursor.saturating_add(rel);
        let window_end = (abs + 240).min(src.len());
        let window = &src[abs..window_end];
        if window.contains("\"rust\"") {
            return true;
        }
        cursor = abs.saturating_add(".code(".len());
    }
    false
}

#[test]
fn ui_pages_do_not_embed_rust_code_literals() {
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
            !has_docsection_rust_code_literal(&src),
            "UI Gallery page embeds a Rust code literal via `DocSection::code(\"rust\", ...)`: {}\n\
             Migrate it to snippet-backed code (`code_rust_from_file_region(include_str!(...), \"example\")`) \
             to keep Preview ≡ Code.",
            path.display()
        );
    }
}
