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

fn assert_normalized_markers_present(relative_path: &str, required_markers: &[&str]) -> String {
    let path = manifest_path(relative_path);
    let source = read_path(&path);
    let normalized = source.split_whitespace().collect::<String>();

    for marker in required_markers {
        let marker = marker.split_whitespace().collect::<String>();
        assert!(
            normalized.contains(&marker),
            "{} is missing marker `{}`",
            path.display(),
            marker
        );
    }

    normalized
}

fn assert_preview_registry_entries_keep_vec_anyelement(relative_dir: &str) {
    for path in rust_sources(relative_dir) {
        let source = read_path(&path);
        if !source.contains("pub(in crate::ui) fn preview_") {
            continue;
        }

        let normalized = source.split_whitespace().collect::<String>();
        let mut remainder = normalized.as_str();
        let mut saw_preview_entry = false;

        while let Some(idx) = remainder.find("pub(incrate::ui)fnpreview_") {
            saw_preview_entry = true;
            let after_start = &remainder[idx..];
            let Some(open_brace_idx) = after_start.find('{') else {
                panic!(
                    "{} should expose an opening brace for its preview registry signature",
                    path.display()
                );
            };
            let signature = &after_start[..open_brace_idx];
            assert!(
                signature.contains("->Vec<AnyElement>"),
                "{} should keep preview registry entries on the explicit `Vec<AnyElement>` seam",
                path.display()
            );
            assert!(
                !signature.contains("->AnyElement"),
                "{} should not regress preview registry entries to a single landed `AnyElement` boundary",
                path.display()
            );
            remainder = &after_start[open_brace_idx + 1..];
        }

        assert!(
            saw_preview_entry,
            "{} should contain at least one preview registry entry once scanned",
            path.display()
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
fn internal_preview_registry_entries_remain_explicit_vec_anyelement_boundaries() {
    assert_preview_registry_entries_keep_vec_anyelement("src/ui/previews/pages");
    assert_preview_registry_entries_keep_vec_anyelement("src/ui/previews/gallery");

    let magic = manifest_path("src/ui/previews/magic.rs");
    let magic_source = read_path(&magic);
    assert!(
        magic_source.contains("pub(in crate::ui) fn preview_magic_"),
        "{} should keep preview registry entries visible to the test",
        magic.display()
    );
    let normalized = magic_source.split_whitespace().collect::<String>();
    assert!(
        normalized
            .contains("pub(incrate::ui)fnpreview_magic_lens(cx:&mutUiCx<'_>)->Vec<AnyElement>{"),
        "{} should keep magic preview registry entries on the explicit `Vec<AnyElement>` seam",
        magic.display()
    );
    assert!(
        !normalized.contains("pub(incrate::ui)fnpreview_magic_lens(cx:&mutUiCx<'_>)->AnyElement{"),
        "{} should not regress magic preview registry entries to `AnyElement`",
        magic.display()
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
fn wrap_preview_page_callers_land_the_typed_preview_shell_explicitly() {
    let normalized = assert_normalized_markers_present(
        "src/ui/doc_layout.rs",
        &[
            "pub(in crate::ui) fn wrap_preview_page(",
            ") -> impl UiChild + use<> {",
        ],
    );
    assert!(
        !normalized.contains(
            "pub(incrate::ui)fnwrap_preview_page(cx:&mutUiCx<'_>,intro:Option<&'staticstr>,section_title:&'staticstr,elements:Vec<AnyElement>,)->AnyElement{"
        ),
        "src/ui/doc_layout.rs should keep wrap_preview_page on the typed internal preview lane",
    );

    for relative_path in [
        "src/ui/previews/pages/editors/markdown.rs",
        "src/ui/previews/pages/editors/web_ime.rs",
        "src/ui/previews/pages/editors/code_view.rs",
        "src/ui/previews/pages/editors/text/measure_overlay.rs",
        "src/ui/previews/pages/editors/text/mixed_script_fallback.rs",
        "src/ui/previews/pages/editors/text/selection_perf.rs",
        "src/ui/previews/pages/editors/text/feature_toggles.rs",
        "src/ui/previews/pages/editors/text/bidi_rtl_conformance.rs",
        "src/ui/previews/pages/editors/text/outline_stroke.rs",
        "src/ui/previews/pages/editors/code_editor/mvp.rs",
        "src/ui/previews/pages/editors/code_editor/torture.rs",
    ] {
        let path = manifest_path(relative_path);
        let source = read_path(&path);
        let normalized = source.split_whitespace().collect::<String>();
        assert!(
            normalized.contains("wrap_preview_page("),
            "{} should keep using the shared typed preview-page wrapper",
            path.display()
        );
        assert!(
            normalized.contains("vec![page.into_element(cx)]"),
            "{} should keep the explicit landing seam at the preview-page call site",
            path.display()
        );
        assert!(
            !normalized.contains("vec![page]"),
            "{} should not fall back to the legacy raw wrap_preview_page result",
            path.display()
        );
    }
}

#[test]
fn render_doc_page_callers_land_the_typed_doc_page_explicitly() {
    for path in rust_sources("src/ui/previews") {
        let source = read_path(&path);
        if !source.contains("render_doc_page(") {
            continue;
        }

        let mut saw_final_return_line = false;
        for line in source.lines() {
            let trimmed = line.trim();
            assert_ne!(
                trimmed,
                "vec![body]",
                "{} should not keep the legacy raw render_doc_page landing",
                path.display()
            );
            assert_ne!(
                trimmed,
                "vec![page]",
                "{} should not keep the legacy raw render_doc_page landing",
                path.display()
            );
            if trimmed.starts_with("vec![body") || trimmed.starts_with("vec![page") {
                saw_final_return_line = true;
                assert!(
                    trimmed.contains(".into_element(cx)"),
                    "{} should keep the final render_doc_page landing explicit on the internal preview surface",
                    path.display()
                );
            }
        }
        assert!(
            saw_final_return_line,
            "{} should expose a final preview return line for render_doc_page output",
            path.display()
        );
    }
}

#[test]
fn internal_preview_scaffold_retains_only_the_audited_vec_anyelement_seams() {
    let path = manifest_path("src/ui/doc_layout.rs");
    let source = read_path(&path);
    let normalized = source.split_whitespace().collect::<String>();

    for marker in [
        "pub(incrate::ui)fnrender_doc_page(cx:&mutUiCx<'_>,intro:Option<&'staticstr>,sections:Vec<DocSection>,)->implUiChild+use<>",
        "letmutout:Vec<AnyElement>=Vec::with_capacity(sections.len()+1);",
        "pub(incrate::ui)fnwrap_preview_page(cx:&mutUiCx<'_>,intro:Option<&'staticstr>,section_title:&'staticstr,elements:Vec<AnyElement>,)->implUiChild+use<>",
        "FnOnce(&mutUiCx<'_>)->Vec<AnyElement>",
    ] {
        assert!(
            normalized.contains(marker),
            "{} is missing intentional scaffold seam marker `{marker}`",
            path.display()
        );
    }

    assert_eq!(
        normalized
            .matches("FnOnce(&mutUiCx<'_>)->Vec<AnyElement>")
            .count(),
        2,
        "{} should keep exactly the audited wrap-row closure seams on Vec<AnyElement>",
        path.display()
    );
    assert!(
        source.contains("Typed page scaffold:"),
        "{} should explain the typed page scaffold vs internal vector seam split",
        path.display()
    );
    assert!(
        source.contains("Typed preview-harness wrapper:"),
        "{} should explain the typed preview wrapper vs explicit preview vector seam split",
        path.display()
    );
    assert!(
        source.contains("Intentionally stored as a landed value because the doc scaffold still decorates preview"),
        "{} should keep the landed DocSection preview-field comment visible",
        path.display()
    );
    assert!(
        source.contains(
            "Intentional raw boundary: gap placeholders are assembled as concrete alert content"
        ),
        "{} should keep the gap-card raw-boundary rationale visible",
        path.display()
    );
    assert!(
        !normalized.contains(
            "pub(incrate::ui)fnrender_doc_page(cx:&mutUiCx<'_>,intro:Option<&'staticstr>,sections:Vec<DocSection>,)->AnyElement"
        ),
        "{} should not regress render_doc_page back to AnyElement",
        path.display()
    );
    assert!(
        !normalized.contains(
            "pub(incrate::ui)fnwrap_preview_page(cx:&mutUiCx<'_>,intro:Option<&'staticstr>,section_title:&'staticstr,elements:Vec<AnyElement>,)->AnyElement"
        ),
        "{} should not regress wrap_preview_page back to AnyElement",
        path.display()
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
fn gallery_overlay_preview_retains_intentional_raw_boundaries() {
    let overlay_normalized = assert_normalized_markers_present(
        "src/ui/previews/gallery/overlays/overlay.rs",
        &[
            "pub(in crate::ui) fn preview_overlay(",
            ") -> Vec<AnyElement> {",
            "let mut out: Vec<AnyElement> = vec![overlays, last_action_status];",
        ],
    );
    assert!(
        overlay_normalized
            .contains("vec![layout::compose_body(cx,models.clone()).into_element(cx)]"),
        "src/ui/previews/gallery/overlays/overlay.rs should keep the cached overlay body as a landed preview root",
    );
    assert!(
        read_path(&manifest_path(
            "src/ui/previews/gallery/overlays/overlay.rs"
        ))
        .contains("Intentional raw boundary:")
    );

    let layout_normalized = assert_normalized_markers_present(
        "src/ui/previews/gallery/overlays/overlay/layout.rs",
        &[
            "fn row(_cx: &mut UiCx<'_>, gap: Px, children: Vec<AnyElement>) -> impl UiChild + use<>",
            "fn row_end(_cx: &mut UiCx<'_>, gap: Px, children: Vec<AnyElement>) -> impl UiChild + use<>",
            "pub(super) fn compose_body(cx: &mut UiCx<'_>, models: OverlayModels) -> impl UiChild + use<>",
        ],
    );
    assert_eq!(
        layout_normalized.matches("->implUiChild+use<>").count(),
        3,
        "src/ui/previews/gallery/overlays/overlay/layout.rs should keep the typed row/body helper lane",
    );
    assert!(
        !layout_normalized.contains("->AnyElement"),
        "src/ui/previews/gallery/overlays/overlay/layout.rs should not regress row/body helpers back to AnyElement",
    );

    let widgets_normalized = assert_normalized_markers_present(
        "src/ui/previews/gallery/overlays/overlay/widgets.rs",
        &[
            "pub(super) fn overlay_reset(_cx: &mut UiCx<'_>, models: &OverlayModels) -> impl UiChild + use<>",
            "pub(super) fn dropdown(_cx: &mut UiCx<'_>, models: &OverlayModels) -> impl UiChild + use<>",
            "pub(super) fn context_menu(_cx: &mut UiCx<'_>, models: &OverlayModels) -> impl UiChild + use<>",
            "pub(super) fn context_menu_edge(_cx: &mut UiCx<'_>, models: &OverlayModels,) -> impl UiChild + use<>",
            "pub(super) fn underlay(_cx: &mut UiCx<'_>) -> impl UiChild + use<>",
            "pub(super) fn tooltip(cx: &mut UiCx<'_>) -> impl UiChild + use<>",
            "pub(super) fn hover_card(cx: &mut UiCx<'_>) -> impl UiChild + use<>",
            "pub(super) fn popover(cx: &mut UiCx<'_>, models: &OverlayModels) -> impl UiChild + use<>",
            "pub(super) fn dialog(_cx: &mut UiCx<'_>, models: &OverlayModels) -> impl UiChild + use<>",
            "pub(super) fn dialog_glass(_cx: &mut UiCx<'_>, models: &OverlayModels) -> impl UiChild + use<>",
            "pub(super) fn alert_dialog(_cx: &mut UiCx<'_>, models: &OverlayModels) -> impl UiChild + use<>",
            "pub(super) fn sheet(_cx: &mut UiCx<'_>, models: &OverlayModels) -> impl UiChild + use<>",
            "pub(super) fn portal_geometry(cx: &mut UiCx<'_>, models: &OverlayModels) -> impl UiChild + use<>",
        ],
    );
    assert_eq!(
        widgets_normalized.matches("->implUiChild+use<>").count(),
        13,
        "src/ui/previews/gallery/overlays/overlay/widgets.rs should keep the typed widget-helper inventory",
    );
    assert_eq!(
        widgets_normalized.matches("->AnyElement").count(),
        0,
        "src/ui/previews/gallery/overlays/overlay/widgets.rs should not regress widget helpers back to AnyElement",
    );
    let widgets_source = read_path(&manifest_path(
        "src/ui/previews/gallery/overlays/overlay/widgets.rs",
    ));
    assert!(widgets_source.contains(
        "Typed helper shells: these helpers may still lower to overlay/provider roots internally"
    ));

    let flags_normalized = assert_normalized_markers_present(
        "src/ui/previews/gallery/overlays/overlay/flags.rs",
        &[
            "pub(super) fn last_action_status(cx: &mut UiCx<'_>, models: &OverlayModels,) -> impl UiChild + use<>",
            "pub(super) fn status_flags(cx: &mut UiCx<'_>, models: &OverlayModels) -> Vec<AnyElement>",
        ],
    );
    assert_eq!(
        flags_normalized.matches("->AnyElement").count(),
        0,
        "src/ui/previews/gallery/overlays/overlay/flags.rs should keep the status-label helper on the typed lane",
    );
    assert!(
        read_path(&manifest_path(
            "src/ui/previews/gallery/overlays/overlay/flags.rs"
        ))
        .contains("Intentional raw boundary:")
    );
}

#[test]
fn code_editor_mvp_internal_helpers_prefer_ui_child_over_anyelement() {
    let header_source = read_path(&manifest_path(
        "src/ui/previews/pages/editors/code_editor/mvp/header.rs",
    ));
    let header_normalized = header_source.split_whitespace().collect::<String>();
    assert!(
        header_normalized.contains("usefret::UiChild;usefret::UiCx;"),
        "src/ui/previews/pages/editors/code_editor/mvp/header.rs should import UiChild with UiCx",
    );
    assert!(
        header_normalized.contains(
            "pub(super)fnbuild_header(cx:&mutUiCx<'_>,theme:&Theme,syntax_rust:Model<bool>,syntax_enabled:bool,boundary_identifier:Model<bool>,boundary_identifier_enabled:bool,soft_wrap:Model<bool>,soft_wrap_enabled:bool,set_identifier_mode:fret_ui::action::OnActivate,set_unicode_mode:fret_ui::action::OnActivate,handles:&CodeEditorMvpHandles,word_fixture_loaded:Rc<Cell<bool>>,word_idx:Rc<Cell<usize>>,word_debug:Rc<std::cell::RefCell<String>>,)->implUiChild+use<>"
        ),
        "src/ui/previews/pages/editors/code_editor/mvp/header.rs should keep build_header on the typed helper lane",
    );
    assert!(
        !header_normalized.contains(
            "pub(super)fnbuild_header(cx:&mutUiCx<'_>,theme:&Theme,syntax_rust:Model<bool>,syntax_enabled:bool,boundary_identifier:Model<bool>,boundary_identifier_enabled:bool,soft_wrap:Model<bool>,soft_wrap_enabled:bool,set_identifier_mode:fret_ui::action::OnActivate,set_unicode_mode:fret_ui::action::OnActivate,handles:&CodeEditorMvpHandles,word_fixture_loaded:Rc<Cell<bool>>,word_idx:Rc<Cell<usize>>,word_debug:Rc<std::cell::RefCell<String>>,)->AnyElement"
        ),
        "src/ui/previews/pages/editors/code_editor/mvp/header.rs should not regress build_header back to AnyElement",
    );

    let word_boundary_source = read_path(&manifest_path(
        "src/ui/previews/pages/editors/code_editor/mvp/word_boundary.rs",
    ));
    let word_boundary_normalized = word_boundary_source.split_whitespace().collect::<String>();
    assert!(
        word_boundary_normalized.contains("usefret::UiChild;usefret::UiCx;"),
        "src/ui/previews/pages/editors/code_editor/mvp/word_boundary.rs should import UiChild with UiCx",
    );
    for marker in [
        "pub(super)fnword_boundary_controls(cx:&mutUiCx<'_>,word_handle:code_editor::CodeEditorHandle,word_fixture_loaded:Rc<Cell<bool>>,word_idx:Rc<Cell<usize>>,word_debug:Rc<std::cell::RefCell<String>>,boundary_identifier:Model<bool>,)->implUiChild+use<>",
        "pub(super)fnword_boundary_debug_view(cx:&mutUiCx<'_>,theme:&Theme,word_handle:code_editor::CodeEditorHandle,word_debug:Rc<std::cell::RefCell<String>>,)->implUiChild+use<>",
    ] {
        assert!(
            word_boundary_normalized.contains(marker),
            "src/ui/previews/pages/editors/code_editor/mvp/word_boundary.rs is missing typed helper marker `{marker}`",
        );
    }
    for marker in [
        "pub(super)fnword_boundary_controls(cx:&mutUiCx<'_>,word_handle:code_editor::CodeEditorHandle,word_fixture_loaded:Rc<Cell<bool>>,word_idx:Rc<Cell<usize>>,word_debug:Rc<std::cell::RefCell<String>>,boundary_identifier:Model<bool>,)->AnyElement",
        "pub(super)fnword_boundary_debug_view(cx:&mutUiCx<'_>,theme:&Theme,word_handle:code_editor::CodeEditorHandle,word_debug:Rc<std::cell::RefCell<String>>,)->AnyElement",
    ] {
        assert!(
            !word_boundary_normalized.contains(marker),
            "src/ui/previews/pages/editors/code_editor/mvp/word_boundary.rs regressed helper marker `{marker}`",
        );
    }

    let gates_source = read_path(&manifest_path(
        "src/ui/previews/pages/editors/code_editor/mvp/gates.rs",
    ));
    let gates_normalized = gates_source.split_whitespace().collect::<String>();
    assert!(
        gates_normalized.contains("usefret::UiChild;usefret::UiCx;"),
        "src/ui/previews/pages/editors/code_editor/mvp/gates.rs should import UiChild with UiCx",
    );
    for marker in [
        "fngate_panel<B>(cx:&mutUiCx<'_>,theme:&Theme,child:B)->implUiChild+use<B>",
        "pub(super)fnword_boundary_gate(cx:&mutUiCx<'_>,theme:&Theme,handle:code_editor::CodeEditorHandle,)->implUiChild+use<>",
        "pub(super)fnword_boundary_soft_wrap_gate(cx:&mutUiCx<'_>,theme:&Theme,handle:code_editor::CodeEditorHandle,)->implUiChild+use<>",
        "pub(super)fna11y_selection_gate(cx:&mutUiCx<'_>,theme:&Theme,handle:code_editor::CodeEditorHandle,)->implUiChild+use<>",
        "pub(super)fna11y_composition_gate(cx:&mutUiCx<'_>,theme:&Theme,handle:code_editor::CodeEditorHandle,)->implUiChild+use<>",
        "pub(super)fna11y_selection_wrap_gate(cx:&mutUiCx<'_>,theme:&Theme,handle:code_editor::CodeEditorHandle,)->implUiChild+use<>",
        "pub(super)fna11y_composition_wrap_gate(cx:&mutUiCx<'_>,theme:&Theme,handle:code_editor::CodeEditorHandle,)->implUiChild+use<>",
        "pub(super)fna11y_composition_drag_gate(cx:&mutUiCx<'_>,theme:&Theme,handle:code_editor::CodeEditorHandle,)->implUiChild+use<>",
    ] {
        assert!(
            gates_normalized.contains(marker),
            "src/ui/previews/pages/editors/code_editor/mvp/gates.rs is missing typed helper marker `{marker}`",
        );
    }
    assert!(
        !gates_normalized.contains("->AnyElement"),
        "src/ui/previews/pages/editors/code_editor/mvp/gates.rs should not regress gate helpers back to AnyElement",
    );
}

#[test]
fn selected_internal_preview_helpers_prefer_typed_outputs() {
    let harness_intro = read_path(&manifest_path("src/ui/previews/pages/harness/intro.rs"));
    let harness_intro_normalized = harness_intro.split_whitespace().collect::<String>();
    assert!(
        harness_intro_normalized.contains("usefret::UiChild;usefret::UiCx;"),
        "src/ui/previews/pages/harness/intro.rs should import UiChild with UiCx",
    );
    assert!(
        harness_intro_normalized.contains(
            "fncard(cx:&mutUiCx<'_>,title:&'staticstr,desc:&'staticstr)->implUiChild+use<>"
        ),
        "src/ui/previews/pages/harness/intro.rs should keep card on the typed helper lane",
    );
    assert!(
        !harness_intro_normalized.contains("->AnyElement"),
        "src/ui/previews/pages/harness/intro.rs should not regress card back to AnyElement",
    );
    assert!(
        harness_intro_normalized.contains("DocSection::build(cx,\"Overview\",preview)"),
        "src/ui/previews/pages/harness/intro.rs should keep overview registration on DocSection::build(cx, ...)",
    );
    assert!(
        !harness_intro_normalized.contains("DocSection::new(\"Overview\",preview)"),
        "src/ui/previews/pages/harness/intro.rs should not regress overview registration to DocSection::new(...)",
    );

    let outline_stroke = read_path(&manifest_path(
        "src/ui/previews/pages/editors/text/outline_stroke.rs",
    ));
    let outline_stroke_normalized = outline_stroke.split_whitespace().collect::<String>();
    assert!(
        outline_stroke_normalized.contains("usefret::UiChild;usefret::UiCx;"),
        "src/ui/previews/pages/editors/text/outline_stroke.rs should import UiChild with UiCx",
    );
    assert!(
        outline_stroke_normalized.contains(
            "fntoggle_button(cx:&mutUiCx<'_>,label:&'staticstr,value:bool,test_id:&'staticstr,on_activate:fret_ui::action::OnActivate,)->implUiChild+use<>"
        ),
        "src/ui/previews/pages/editors/text/outline_stroke.rs should keep toggle_button on the typed helper lane",
    );
    assert!(
        !outline_stroke_normalized.contains(
            "fntoggle_button(cx:&mutUiCx<'_>,label:&'staticstr,value:bool,test_id:&'staticstr,on_activate:fret_ui::action::OnActivate,)->AnyElement"
        ),
        "src/ui/previews/pages/editors/text/outline_stroke.rs should not regress toggle_button back to AnyElement",
    );

    let mixed_script = read_path(&manifest_path(
        "src/ui/previews/pages/editors/text/mixed_script_fallback.rs",
    ));
    let mixed_script_normalized = mixed_script.split_whitespace().collect::<String>();
    assert!(
        mixed_script_normalized.contains("usefret::UiChild;usefret::UiCx;"),
        "src/ui/previews/pages/editors/text/mixed_script_fallback.rs should import UiChild with UiCx",
    );
    assert!(
        mixed_script_normalized.contains(
            "fnsample_row(cx:&mutUiCx<'_>,theme:&Theme,label:&'staticstr,sample:&'staticstr,test_id:&'staticstr,)->implUiChild+use<>"
        ),
        "src/ui/previews/pages/editors/text/mixed_script_fallback.rs should keep sample_row on the typed helper lane",
    );
    assert!(
        !mixed_script_normalized.contains(
            "fnsample_row(cx:&mutUiCx<'_>,theme:&Theme,label:&'staticstr,sample:&'staticstr,test_id:&'staticstr,)->AnyElement"
        ),
        "src/ui/previews/pages/editors/text/mixed_script_fallback.rs should not regress sample_row back to AnyElement",
    );

    let feature_toggles = read_path(&manifest_path(
        "src/ui/previews/pages/editors/text/feature_toggles.rs",
    ));
    let feature_toggles_normalized = feature_toggles.split_whitespace().collect::<String>();
    assert!(
        feature_toggles_normalized.contains("usefret::UiChild;usefret::UiCx;"),
        "src/ui/previews/pages/editors/text/feature_toggles.rs should import UiChild with UiCx",
    );
    for marker in [
        "fntoggle_button(cx:&mutUiCx<'_>,label:&'staticstr,value:bool,test_id:&'staticstr,on_activate:fret_ui::action::OnActivate,)->implUiChild+use<>",
        "fnsample_text(cx:&mutUiCx<'_>,theme:&Theme,label:&'staticstr,text:&'staticstr,features:Option<fret_core::TextShapingStyle>,test_id:&'staticstr,)->implUiChild+use<>",
    ] {
        assert!(
            feature_toggles_normalized.contains(marker),
            "src/ui/previews/pages/editors/text/feature_toggles.rs is missing typed helper marker `{marker}`",
        );
    }
    assert!(
        !feature_toggles_normalized.contains("fntoggle_button(cx:&mutUiCx<'_>,label:&'staticstr,value:bool,test_id:&'staticstr,on_activate:fret_ui::action::OnActivate,)->AnyElement"),
        "src/ui/previews/pages/editors/text/feature_toggles.rs should not regress toggle_button back to AnyElement",
    );
    assert!(
        !feature_toggles_normalized.contains("fnsample_text(cx:&mutUiCx<'_>,theme:&Theme,label:&'staticstr,text:&'staticstr,features:Option<fret_core::TextShapingStyle>,test_id:&'staticstr,)->AnyElement"),
        "src/ui/previews/pages/editors/text/feature_toggles.rs should not regress sample_text back to AnyElement",
    );
}

#[test]
fn selected_internal_preview_pages_use_typed_doc_sections() {
    for (relative_path, required_marker, forbidden_marker) in [
        (
            "src/ui/previews/pages/harness/layout.rs",
            "DocSection::build(cx, \"Demo\", row)",
            "DocSection::new(\"Demo\", row)",
        ),
        (
            "src/ui/previews/pages/harness/ui_kit_list_torture.rs",
            "DocSection::build(cx, \"Harness\", root)",
            "DocSection::new(\"Harness\", root)",
        ),
        (
            "src/ui/previews/pages/harness/hit_test_only_paint_cache_probe.rs",
            "DocSection::build(cx, \"Probe region\", panel)",
            "DocSection::new(\"Probe region\", panel)",
        ),
        (
            "src/ui/previews/pages/harness/view_cache.rs",
            "DocSection::build(cx, \"Harness\", root)",
            "DocSection::new(\"Harness\", root)",
        ),
        (
            "src/ui/previews/pages/harness/virtual_list_torture.rs",
            "DocSection::build(cx, \"Harness\", root)",
            "DocSection::new(\"Harness\", root)",
        ),
        (
            "src/ui/previews/pages/torture/chart_torture.rs",
            "DocSection::build(cx, \"Chart\", chart)",
            "DocSection::new(\"Chart\", chart)",
        ),
        (
            "src/ui/previews/pages/torture/canvas_cull_torture.rs",
            "DocSection::build(cx, \"Canvas\", canvas)",
            "DocSection::new(\"Canvas\", canvas)",
        ),
        (
            "src/ui/previews/pages/torture/chrome_torture.rs",
            "DocSection::build(cx, \"Harness\", content)",
            "DocSection::new(\"Harness\", content)",
        ),
        (
            "src/ui/previews/pages/torture/windowed_rows_surface_interactive_torture.rs",
            "DocSection::build(cx, \"Surface\", surface)",
            "DocSection::new(\"Surface\", surface)",
        ),
        (
            "src/ui/previews/pages/torture/windowed_rows_surface_torture.rs",
            "DocSection::build(cx, \"Surface\", surface)",
            "DocSection::new(\"Surface\", surface)",
        ),
        (
            "src/ui/previews/pages/torture/node_graph_cull_torture.rs",
            "DocSection::build(cx, \"Canvas\", surface)",
            "DocSection::new(\"Canvas\", surface)",
        ),
    ] {
        let normalized = assert_normalized_markers_present(relative_path, &[required_marker]);
        let forbidden_marker = forbidden_marker.split_whitespace().collect::<String>();
        assert!(
            !normalized.contains(&forbidden_marker),
            "{} should not regress to legacy `DocSection::new(...)` registration",
            manifest_path(relative_path).display()
        );
    }
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

#[test]
fn gallery_table_retained_torture_uses_structured_table_debug_ids() {
    let normalized = assert_normalized_markers_present(
        "src/ui/previews/gallery/torture/table_retained_torture.rs",
        &[
            "let table_debug_ids = fret_ui_kit::declarative::table::TableDebugIds {",
            "header_row_test_id: Some(Arc::<str>::from(\"ui-gallery-table-retained-header-row\")),",
            "header_cell_test_id_prefix: Some(Arc::<str>::from(\"ui-gallery-table-retained-header-\",)),",
            "row_test_id_prefix: Some(Arc::<str>::from(\"ui-gallery-table-retained-row-\")),",
            "Keep retained table diagnostics on table-owned layout wrappers.",
        ],
    );

    assert!(
        !normalized.contains("TableDebugIds::default()"),
        "table_retained_torture should not fall back to an empty default diagnostics contract"
    );
}

#[test]
fn gallery_data_table_torture_exposes_header_row_anchor() {
    let normalized = assert_normalized_markers_present(
        "src/ui/previews/gallery/data/table_torture.rs",
        &["header_row_test_id: Some(Arc::<str>::from(\"ui-gallery-data-table-header-row\",)),"],
    );

    assert!(
        normalized.contains("ui-gallery-data-table-header-")
            && normalized.contains("ui-gallery-data-table-row-"),
        "table_torture should keep the structured data-table header/body diagnostics prefixes alongside the header-row anchor"
    );
}
