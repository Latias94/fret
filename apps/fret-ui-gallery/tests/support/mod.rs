#![allow(dead_code)]

use std::fs;
use std::path::{Path, PathBuf};

pub fn manifest_path(relative: &str) -> PathBuf {
    Path::new(env!("CARGO_MANIFEST_DIR")).join(relative)
}

pub fn read(relative: &str) -> String {
    read_path(&manifest_path(relative))
}

pub fn read_path(path: &Path) -> String {
    fs::read_to_string(path)
        .unwrap_or_else(|err| panic!("read_to_string failed for {}: {err}", path.display()))
}

pub fn collect_rust_sources(dir: &Path, out: &mut Vec<PathBuf>) {
    for entry in fs::read_dir(dir)
        .unwrap_or_else(|err| panic!("read_dir failed for {}: {err}", dir.display()))
    {
        let path = entry.expect("read_dir entry").path();
        if path.is_dir() {
            collect_rust_sources(&path, out);
            continue;
        }

        if path.extension().is_some_and(|ext| ext == "rs") {
            out.push(path);
        }
    }
}

pub fn rust_sources(relative: &str) -> Vec<PathBuf> {
    let root = manifest_path(relative);
    let mut paths = Vec::new();
    collect_rust_sources(&root, &mut paths);
    paths.sort();
    paths
}

pub fn gallery_rust_sources() -> Vec<PathBuf> {
    rust_sources("src")
}

pub fn assert_no_raw_app_surface(path: &Path, source: &str) {
    assert!(
        !source.contains("use fret_app::App;"),
        "{} should not teach the raw app runtime name",
        path.display()
    );
    assert!(
        !source.contains("ElementContext<'_, App>") && !source.contains("ElementContext<'a, App>"),
        "{} reintroduced the raw ElementContext app surface",
        path.display()
    );
}

pub fn assert_default_app_surface(
    path: &Path,
    source: &str,
    expected_patterns: &[&str],
    surface_label: &str,
) {
    let normalized = source.split_whitespace().collect::<String>();

    assert!(
        source.contains("use fret::AppComponentCx;")
            || normalized.contains("usefret::{UiChild,AppComponentCx};")
            || normalized.contains("usefret::{AppComponentCx,UiChild};"),
        "{} should use the default app helper context alias",
        path.display()
    );
    assert!(
        expected_patterns
            .iter()
            .any(|pattern| source_contains_equivalent_marker(source, &normalized, pattern)),
        "{} should expose AppComponentCx on the {}",
        path.display(),
        surface_label
    );
    assert_no_raw_app_surface(path, source);
}

pub fn source_contains_equivalent_marker(source: &str, normalized: &str, pattern: &str) -> bool {
    if source.contains(pattern) {
        return true;
    }

    let normalized_pattern = pattern.split_whitespace().collect::<String>();
    if normalized.contains(&normalized_pattern) {
        return true;
    }

    let is_default_app_component_import = matches!(
        normalized_pattern.as_str(),
        "usefret::{UiChild,AppComponentCx};" | "usefret::{AppComponentCx,UiChild};"
    );

    if !is_default_app_component_import {
        return false;
    }

    normalized.contains("usefret::{UiChild,AppComponentCx};")
        || normalized.contains("usefret::{AppComponentCx,UiChild};")
        || (normalized.contains("usefret::AppComponentCx;")
            && normalized.contains("usefret::UiChild;"))
}

pub fn assert_internal_preview_surface(
    path: &Path,
    source: &str,
    trigger_patterns: &[&str],
    expected_patterns: &[&str],
    extra_forbidden: &[&str],
    surface_label: &str,
) {
    assert_no_raw_app_surface(path, source);

    for forbidden in extra_forbidden {
        assert!(
            !source.contains(forbidden),
            "{} reintroduced forbidden context pattern `{}` on the {}",
            path.display(),
            forbidden,
            surface_label
        );
    }

    if trigger_patterns
        .iter()
        .any(|pattern| source.contains(pattern))
    {
        assert!(
            source.contains("use fret::AppComponentCx;"),
            "{} should use the shared helper context alias",
            path.display()
        );
        assert!(
            expected_patterns
                .iter()
                .any(|pattern| source.contains(pattern)),
            "{} should expose AppComponentCx on the {}",
            path.display(),
            surface_label
        );
    }
}
