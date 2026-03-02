use std::env;
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

fn bool_cell(value: bool) -> &'static str {
    if value { "Yes" } else { "No" }
}

fn has_rust_code_literal(src: &str) -> bool {
    // Heuristic: this intentionally does not parse Rust.
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

fn has_relative_snippet_include(src: &str) -> bool {
    src.contains("include_str!(\"../snippets/")
}

fn snippet_uses_gallery_internals(src: &str) -> bool {
    src.contains("use crate::ui::")
        || src.contains("crate::ui::")
        || src.contains("use crate::spec::")
        || src.contains("crate::spec::")
}

fn snippet_missing_source_const(src: &str) -> bool {
    !src.contains("pub const SOURCE:")
}

fn rel_path_str(base: &Path, path: &Path) -> String {
    path.strip_prefix(base)
        .unwrap_or(path)
        .to_string_lossy()
        .to_string()
}

fn main() {
    let manifest_dir = PathBuf::from(env::var("CARGO_MANIFEST_DIR").expect("CARGO_MANIFEST_DIR"));
    let out_dir = PathBuf::from(env::var("OUT_DIR").expect("OUT_DIR"));

    let pages_dir = manifest_dir.join("src/ui/pages");
    let previews_dir = manifest_dir.join("src/ui/previews");
    let snippets_dir = manifest_dir.join("src/ui/snippets");

    // Rebuild the report when any UI source changes. This is an app-only convenience surface; it
    // should never gate builds/tests.
    println!("cargo:rerun-if-changed=build.rs");
    println!("cargo:rerun-if-changed={}", pages_dir.display());
    println!("cargo:rerun-if-changed={}", previews_dir.display());
    println!("cargo:rerun-if-changed={}", snippets_dir.display());

    let mut page_files = Vec::new();
    visit_rs_files(&pages_dir, &mut page_files);
    page_files.sort();

    let mut snippet_files = Vec::new();
    visit_rs_files(&snippets_dir, &mut snippet_files);
    snippet_files.sort();

    let mut preview_files = Vec::new();
    visit_rs_files(&previews_dir, &mut preview_files);
    preview_files.sort();

    let mut report = String::new();
    report.push_str("# UI Gallery drift audit (Preview ≡ Code)\n\n");
    report.push_str("Generated at build time.\n\n");
    report.push_str("This report is informational. Enforcement lives in tests:\n");
    report.push_str("- `ui_pages_deny_rust_code_literals`\n");
    report.push_str("- `ui_pages_deny_relative_snippet_includes`\n");
    report.push_str("- `ui_snippets_deny_gallery_internal_imports`\n");
    report.push_str("- `ui_snippets_require_source_const`\n\n");

    report.push_str("## Pages\n\n");
    report.push_str("| Path | `.code(\"rust\", ...)` | `include_str!(\"../snippets/\")` |\n");
    report.push_str("| --- | --- | --- |\n");
    for path in &page_files {
        let src = fs::read_to_string(path)
            .unwrap_or_else(|e| panic!("read_to_string failed for {}: {e}", path.display()));
        let has_literals = has_rust_code_literal(&src);
        let has_relative = has_relative_snippet_include(&src);
        report.push_str(&format!(
            "| `{}` | {} | {} |\n",
            rel_path_str(&manifest_dir, path),
            bool_cell(has_literals),
            bool_cell(has_relative)
        ));
    }

    report.push_str("\n## Previews\n\n");
    report.push_str("These sources back non-shadcn preview pages (harnesses, torture cases, etc).\n");
    report.push_str("If these ever start rendering copyable code tabs, they should follow the same\n");
    report.push_str("snippet-backed pattern to stay drift-free.\n\n");
    report.push_str("| Path | `.code(\"rust\", ...)` | `include_str!(\"../snippets/\")` |\n");
    report.push_str("| --- | --- | --- |\n");
    for path in &preview_files {
        let src = fs::read_to_string(path)
            .unwrap_or_else(|e| panic!("read_to_string failed for {}: {e}", path.display()));
        let has_literals = has_rust_code_literal(&src);
        let has_relative = has_relative_snippet_include(&src);
        report.push_str(&format!(
            "| `{}` | {} | {} |\n",
            rel_path_str(&manifest_dir, path),
            bool_cell(has_literals),
            bool_cell(has_relative)
        ));
    }

    report.push_str("\n## Snippets\n\n");
    report.push_str("| Path | Missing `SOURCE` | Uses `crate::ui/spec` |\n");
    report.push_str("| --- | --- | --- |\n");
    for path in &snippet_files {
        if path.file_name().and_then(|s| s.to_str()) == Some("mod.rs") {
            continue;
        }
        let src = fs::read_to_string(path)
            .unwrap_or_else(|e| panic!("read_to_string failed for {}: {e}", path.display()));
        let missing_source = snippet_missing_source_const(&src);
        let uses_internal = snippet_uses_gallery_internals(&src);
        report.push_str(&format!(
            "| `{}` | {} | {} |\n",
            rel_path_str(&manifest_dir, path),
            bool_cell(missing_source),
            bool_cell(uses_internal),
        ));
    }

    let out_path = out_dir.join("ui_gallery_drift_audit.md");
    fs::write(&out_path, report)
        .unwrap_or_else(|e| panic!("failed to write {}: {e}", out_path.display()));
}
