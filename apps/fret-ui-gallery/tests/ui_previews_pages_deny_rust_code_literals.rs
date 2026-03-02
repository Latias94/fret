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

        if path.extension().and_then(|s| s.to_str()) == Some("rs") {
            files.push(path);
        }
    }
}

fn has_docsection_rust_code_literal(src: &str) -> bool {
    // Keep this check heuristic and dependency-free; it is meant to prevent re-introducing drift
    // surfaces in preview-backed doc pages.
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
fn ui_previews_pages_do_not_embed_rust_code_literals() {
    let previews_pages_dir = Path::new(env!("CARGO_MANIFEST_DIR")).join("src/ui/previews/pages");
    let mut files = Vec::new();
    visit_rs_files(&previews_pages_dir, &mut files);
    files.sort();

    for path in files {
        let src = fs::read_to_string(&path)
            .unwrap_or_else(|e| panic!("read_to_string failed for {}: {e}", path.display()));

        assert!(
            !has_docsection_rust_code_literal(&src),
            "UI Gallery previews/pages embeds a Rust code literal via `DocSection::code(\"rust\", ...)`: {}\n\
             Prefer snippet-backed code (`code_rust_from_file_region(snippets::...::SOURCE, \"example\")`) \
             so Preview ≡ Code stays true across refactors.",
            path.display()
        );
    }
}
