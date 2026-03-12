mod support;

use support::{assert_internal_preview_surface, manifest_path, read_path, rust_sources};

fn assert_internal_preview_dir(
    relative_dir: &str,
    trigger_patterns: &[&str],
    expected_patterns: &[&str],
    extra_forbidden: &[&str],
    surface_label: &str,
) {
    for path in rust_sources(relative_dir) {
        let source = read_path(&path);
        assert_internal_preview_surface(
            &path,
            &source,
            trigger_patterns,
            expected_patterns,
            extra_forbidden,
            surface_label,
        );
    }
}

fn assert_curated_internal_preview_paths(
    relative_paths: &[&str],
    trigger_patterns: &[&str],
    expected_patterns: &[&str],
    extra_forbidden: &[&str],
    surface_label: &str,
) {
    for relative_path in relative_paths {
        let path = manifest_path(relative_path);
        let source = read_path(&path);
        assert_internal_preview_surface(
            &path,
            &source,
            trigger_patterns,
            expected_patterns,
            extra_forbidden,
            surface_label,
        );
    }
}

#[test]
fn magic_preview_prefers_ui_cx_on_the_internal_gallery_surface() {
    let path = manifest_path("src/ui/previews/magic.rs");
    let source = read_path(&path);

    assert_internal_preview_surface(
        &path,
        &source,
        &["cx: &mut"],
        &["cx: &mut UiCx<'_>"],
        &[],
        "internal gallery preview surface",
    );
}

#[test]
fn component_preview_modules_prefer_ui_cx_on_the_internal_gallery_surface() {
    assert_internal_preview_dir(
        "src/ui/previews/pages/components",
        &["cx: &mut", "FnOnce(&mut"],
        &[
            "cx: &mut UiCx<'_>",
            "cx: &mut UiCx<'a>",
            "FnOnce(&mut UiCx<'_>)",
            "FnOnce(&mut UiCx<'a>)",
        ],
        &[],
        "internal component preview surface",
    );
}

#[test]
fn harness_preview_shells_prefer_ui_cx_on_the_internal_gallery_surface() {
    assert_curated_internal_preview_paths(
        &[
            "src/ui/previews/pages/harness/intro.rs",
            "src/ui/previews/pages/harness/layout.rs",
            "src/ui/previews/pages/harness/view_cache.rs",
            "src/ui/previews/pages/harness/hit_test_only_paint_cache_probe.rs",
            "src/ui/previews/pages/harness/ui_kit_list_torture.rs",
            "src/ui/previews/pages/harness/virtual_list_torture.rs",
        ],
        &["cx: &mut", "FnOnce(&mut"],
        &[
            "cx: &mut UiCx<'_>",
            "cx: &mut UiCx<'a>",
            "FnOnce(&mut UiCx<'_>)",
            "FnOnce(&mut UiCx<'a>)",
        ],
        &[],
        "internal harness preview surface",
    );
}

#[test]
fn gallery_atom_preview_modules_prefer_ui_cx_on_the_internal_gallery_surface() {
    assert_internal_preview_dir(
        "src/ui/previews/gallery/atoms",
        &["cx: &mut", "FnOnce(&mut"],
        &[
            "cx: &mut UiCx<'_>",
            "cx: &mut UiCx<'a>",
            "FnOnce(&mut UiCx<'_>)",
            "FnOnce(&mut UiCx<'a>)",
        ],
        &[],
        "internal atom preview surface",
    );
}

#[test]
fn gallery_form_preview_modules_prefer_ui_cx_on_the_internal_gallery_surface() {
    assert_internal_preview_dir(
        "src/ui/previews/gallery/forms",
        &["cx: &mut", "FnOnce(&mut"],
        &[
            "cx: &mut UiCx<'_>",
            "cx: &mut UiCx<'a>",
            "FnOnce(&mut UiCx<'_>)",
            "FnOnce(&mut UiCx<'a>)",
        ],
        &[],
        "internal form preview surface",
    );
}

#[test]
fn gallery_data_preview_modules_prefer_ui_cx_on_the_internal_gallery_surface() {
    assert_internal_preview_dir(
        "src/ui/previews/gallery/data",
        &["cx: &mut", "FnOnce(&mut"],
        &[
            "cx: &mut UiCx<'_>",
            "cx: &mut UiCx<'a>",
            "FnOnce(&mut UiCx<'_>)",
            "FnOnce(&mut UiCx<'a>)",
        ],
        &["ElementContext<'_, H>", "ElementContext<'a, H>"],
        "internal data preview surface",
    );
}

#[test]
fn gallery_overlay_preview_modules_prefer_ui_cx_on_the_internal_gallery_surface() {
    assert_internal_preview_dir(
        "src/ui/previews/gallery/overlays",
        &["cx: &mut", "FnOnce(&mut"],
        &[
            "cx: &mut UiCx<'_>",
            "cx: &mut UiCx<'a>",
            "FnOnce(&mut UiCx<'_>)",
            "FnOnce(&mut UiCx<'a>)",
        ],
        &[],
        "internal overlay preview surface",
    );
}

#[test]
fn editor_preview_modules_prefer_ui_cx_on_the_internal_gallery_surface() {
    assert_internal_preview_dir(
        "src/ui/previews/pages/editors",
        &["cx: &mut", "FnOnce(&mut", "Fn(&mut"],
        &[
            "cx: &mut UiCx<'_>",
            "cx: &mut UiCx<'a>",
            "FnOnce(&mut UiCx<'_>)",
            "FnOnce(&mut UiCx<'a>)",
            "Fn(&mut UiCx<'_>)",
            "Fn(&mut UiCx<'a>)",
        ],
        &["ElementContext<'_, H>", "ElementContext<'a, H>"],
        "internal editor preview surface",
    );
}

#[test]
fn page_torture_preview_modules_prefer_ui_cx_on_the_internal_gallery_surface() {
    assert_internal_preview_dir(
        "src/ui/previews/pages/torture",
        &["cx: &mut", "FnOnce(&mut", "Fn(&mut"],
        &[
            "cx: &mut UiCx<'_>",
            "cx: &mut UiCx<'a>",
            "FnOnce(&mut UiCx<'_>)",
            "FnOnce(&mut UiCx<'a>)",
            "Fn(&mut UiCx<'_>)",
            "Fn(&mut UiCx<'a>)",
        ],
        &[],
        "internal torture preview surface",
    );
}

#[test]
fn gallery_torture_preview_modules_prefer_ui_cx_on_the_internal_gallery_surface() {
    assert_internal_preview_dir(
        "src/ui/previews/gallery/torture",
        &["cx: &mut", "FnOnce(&mut", "Fn(&mut"],
        &[
            "cx: &mut UiCx<'_>",
            "cx: &mut UiCx<'a>",
            "FnOnce(&mut UiCx<'_>)",
            "FnOnce(&mut UiCx<'a>)",
            "Fn(&mut UiCx<'_>)",
            "Fn(&mut UiCx<'a>)",
        ],
        &[],
        "internal gallery torture preview surface",
    );
}
