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
        overlay_normalized.contains("vec![layout::compose_body(cx,widgets).into_element(cx)]"),
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
            "pub(super) fn compose_body(cx: &mut UiCx<'_>, widgets: OverlayWidgets) -> impl UiChild + use<>",
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
            "pub(super) struct OverlayWidgets {",
            "fn overlay_reset(cx: &mut UiCx<'_>, models: &OverlayModels) -> AnyElement",
            "fn popover(cx: &mut UiCx<'_>, models: &OverlayModels) -> AnyElement",
            "fn dialog(cx: &mut UiCx<'_>, models: &OverlayModels) -> AnyElement",
            "fn sheet(cx: &mut UiCx<'_>, models: &OverlayModels) -> AnyElement",
            "fn portal_geometry(cx: &mut UiCx<'_>, models: &OverlayModels) -> AnyElement",
        ],
    );
    assert_eq!(
        widgets_normalized.matches(":AnyElement,").count(),
        13,
        "src/ui/previews/gallery/overlays/overlay/widgets.rs should keep the audited landed widget inventory",
    );
    assert_eq!(
        widgets_normalized.matches("->AnyElement").count(),
        13,
        "src/ui/previews/gallery/overlays/overlay/widgets.rs should keep the audited raw widget-root inventory",
    );
    let widgets_source = read_path(&manifest_path(
        "src/ui/previews/gallery/overlays/overlay/widgets.rs",
    ));
    assert!(widgets_source.contains(
        "Intentionally stored as landed values because the overlay preview arranges already-built roots"
    ));
    assert!(widgets_source.contains("Intentional raw boundary:"));

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
