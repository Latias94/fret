mod driver;
mod harness;
mod spec;

mod ui;
pub use driver::{build_app, build_driver, build_runner_config, run};

#[cfg(not(target_arch = "wasm32"))]
pub use driver::run_with_event_loop;

#[cfg(test)]
mod authoring_surface_policy_tests {
    use std::path::{Path, PathBuf};

    const MENUBAR: &str = include_str!("driver/menubar.rs");
    const CHROME: &str = include_str!("driver/chrome.rs");
    const RUNTIME_DRIVER: &str = include_str!("driver/runtime_driver.rs");
    const SETTINGS_SHEET: &str = include_str!("driver/settings_sheet.rs");
    const THEME_RUNTIME: &str = include_str!("driver/theme_runtime.rs");
    const UI_MOD: &str = include_str!("ui/mod.rs");
    const PAGE_FIELD: &str = include_str!("ui/pages/field.rs");
    const PAGE_INPUT: &str = include_str!("ui/pages/input.rs");
    const PAGE_KBD: &str = include_str!("ui/pages/kbd.rs");
    const ACTION_FIRST_VIEW: &str = include_str!("ui/snippets/command/action_first_view.rs");

    fn collect_rust_sources(dir: &Path, out: &mut Vec<PathBuf>) {
        for entry in std::fs::read_dir(dir).unwrap() {
            let path = entry.unwrap().path();
            if path.is_dir() {
                collect_rust_sources(&path, out);
                continue;
            }

            if path.extension().is_some_and(|ext| ext == "rs") {
                out.push(path);
            }
        }
    }

    fn gallery_rust_sources() -> Vec<PathBuf> {
        let mut paths = Vec::new();
        collect_rust_sources(&Path::new(env!("CARGO_MANIFEST_DIR")).join("src"), &mut paths);
        paths.sort();
        paths
    }

    #[test]
    fn gallery_sources_do_not_depend_on_the_legacy_fret_prelude() {
        assert!(!MENUBAR.contains("fret::prelude"));
        assert!(MENUBAR.contains("use fret::workspace_menu::{"));

        assert!(!ACTION_FIRST_VIEW.contains("use fret::prelude::*;"));
        assert!(ACTION_FIRST_VIEW.contains("use fret::advanced::prelude::*;"));
        assert!(ACTION_FIRST_VIEW.contains("KernelApp"));
        assert!(!ACTION_FIRST_VIEW.contains("ViewCx<'_, '_, App>"));
        assert!(!ACTION_FIRST_VIEW.contains("ElementContext<'_, App>"));
    }

    #[test]
    fn gallery_curated_shadcn_surfaces_stay_explicit() {
        for source in [CHROME, RUNTIME_DRIVER, UI_MOD] {
            assert!(!source.contains("use fret_ui_shadcn as shadcn;"));
            assert!(!source.contains("use fret_ui_shadcn::{self as shadcn"));
        }

        assert!(CHROME.contains("use fret_ui_shadcn::facade as shadcn;"));
        assert!(RUNTIME_DRIVER.contains("use fret_ui_shadcn::facade as shadcn;"));
        assert!(UI_MOD.contains("use fret_ui_shadcn::{facade as shadcn, prelude::*};"));
        assert!(SETTINGS_SHEET.contains("use fret_ui_shadcn::{facade as shadcn, prelude::*};"));

        assert!(!THEME_RUNTIME.contains("fret_ui_shadcn::shadcn_themes::"));
        assert!(THEME_RUNTIME.contains("shadcn::themes::ShadcnBaseColor::"));
        assert!(THEME_RUNTIME.contains("shadcn::themes::apply_shadcn_new_york"));

        for page in [PAGE_FIELD, PAGE_INPUT, PAGE_KBD] {
            assert!(page.contains("use fret_ui_shadcn::{facade as shadcn, prelude::*};"));
            assert!(!page.contains("use fret_ui_shadcn::{self as shadcn, prelude::*};"));
        }
    }

    #[test]
    fn gallery_source_tree_rejects_legacy_shadcn_alias_patterns() {
        for path in gallery_rust_sources() {
            if path.ends_with("src/lib.rs") {
                continue;
            }

            let source = std::fs::read_to_string(&path).unwrap();
            assert!(
                !source.contains("fret_ui_fret_ui_shadcn::"),
                "{} duplicated an explicit fret_ui_shadcn path prefix",
                path.display()
            );
            assert!(
                !source.contains("use fret_ui_shadcn as shadcn;"),
                "{} reintroduced the legacy root alias import",
                path.display()
            );
            assert!(
                !source.contains("use fret_ui_shadcn::{self as shadcn"),
                "{} reintroduced the legacy self-as-shadcn import",
                path.display()
            );

            for (line_idx, line) in source.lines().enumerate() {
                for (offset, _) in line.match_indices("shadcn::") {
                    let previous = line[..offset].chars().next_back();
                    if previous.is_some_and(|ch| ch.is_ascii_alphanumeric() || ch == '_') {
                        continue;
                    }

                    let after = &line[offset + "shadcn::".len()..];
                    let segment_len = after
                        .chars()
                        .take_while(|ch| ch.is_ascii_alphanumeric() || *ch == '_')
                        .count();
                    if segment_len == 0 {
                        continue;
                    }

                    let segment = &after[..segment_len];
                    let is_module_path = after[segment_len..].starts_with("::");
                    let starts_with_lowercase =
                        segment.chars().next().is_some_and(|ch| ch.is_ascii_lowercase());
                    if is_module_path
                        && starts_with_lowercase
                        && !matches!(segment, "raw" | "themes" | "app")
                    {
                        panic!(
                            "{}:{} reintroduced non-curated shadcn module path `shadcn::{}::`",
                            path.display(),
                            line_idx + 1,
                            segment
                        );
                    }
                }
            }
        }
    }
}
