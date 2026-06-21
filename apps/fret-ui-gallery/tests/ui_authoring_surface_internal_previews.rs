mod support;

use std::path::Path;

use support::{
    assert_internal_preview_surface, manifest_path, read_path, rust_sources,
    source_contains_equivalent_marker,
};

fn canonicalize_rust_fragment(fragment: &str) -> String {
    let mut canonical = fragment.split_whitespace().collect::<String>();
    loop {
        let next = canonical.replace(",)", ")");
        if next == canonical {
            return next;
        }
        canonical = next;
    }
}

fn assert_imports_ui_child_with_app_component(path: &Path, source: &str, normalized: &str) {
    assert!(
        source_contains_equivalent_marker(
            source,
            normalized,
            "use fret::{UiChild, AppComponentCx};"
        ),
        "{} should import UiChild with AppComponentCx",
        path.display(),
    );
}

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
    let normalized = canonicalize_rust_fragment(&source);

    for marker in required_markers {
        let marker = canonicalize_rust_fragment(marker);
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
        &["cx: &mut AppComponentCx<'_>"],
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
            "cx: &mut AppComponentCx<'_>",
            "cx: &mut AppComponentCx<'a>",
            "FnOnce(&mut AppComponentCx<'_>)",
            "FnOnce(&mut AppComponentCx<'a>)",
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
        normalized.contains(
            "pub(incrate::ui)fnpreview_magic_lens(cx:&mutAppComponentCx<'_>)->Vec<AnyElement>{"
        ),
        "{} should keep magic preview registry entries on the explicit `Vec<AnyElement>` seam",
        magic.display()
    );
    assert!(
        !normalized.contains(
            "pub(incrate::ui)fnpreview_magic_lens(cx:&mutAppComponentCx<'_>)->AnyElement{"
        ),
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
            "cx: &mut AppComponentCx<'_>",
            "cx: &mut AppComponentCx<'a>",
            "FnOnce(&mut AppComponentCx<'_>)",
            "FnOnce(&mut AppComponentCx<'a>)",
        ],
        &[],
        "internal harness preview surface",
    );
}

#[test]
fn harness_hit_test_torture_uses_header_text_roles() {
    let normalized = assert_normalized_markers_present(
        "src/ui/previews/pages/harness/hit_test_torture.rs",
        &[
            "doc_layout::paragraph_text(cx,\"Goal:makehit-testameasurablehotspotsobounds-treevsfallbacktraversalA/Bismeaningful.\")",
            "doc_layout::control_readout_text(cx,format!(\"Shape:{stripes}stripes({}pxeach)plus{noise}1x1noiseregions.\",",
            "doc_layout::control_readout_text(cx,\"Env:FRET_UI_GALLERY_HIT_TEST_TORTURE_STRIPES/FRET_UI_GALLERY_HIT_TEST_TORTURE_NOISE\")",
        ],
    );

    for forbidden in [
        "cx.text(\"Goal:makehit-testameasurablehotspotsobounds-treevsfallbacktraversalA/Bismeaningful.\")",
        "cx.text(format!(\"Shape:{stripes}stripes({}pxeach)plus{noise}1x1noiseregions.\",",
        "cx.text(\"Env:FRET_UI_GALLERY_HIT_TEST_TORTURE_STRIPES/FRET_UI_GALLERY_HIT_TEST_TORTURE_NOISE\")",
    ] {
        assert!(
            !normalized.contains(forbidden),
            "hit_test_torture reintroduced bare header text: {forbidden}"
        );
    }
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
            "pub(incrate::ui)fnwrap_preview_page(cx:&mutAppComponentCx<'_>,intro:Option<&'staticstr>,section_title:&'staticstr,elements:Vec<AnyElement>,)->AnyElement{"
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
fn editor_code_view_header_uses_paragraph_roles() {
    let normalized = assert_normalized_markers_present(
        "src/ui/previews/pages/editors/code_view.rs",
        &[
            "doc_layout::paragraph_text(cx,\"Goal:stresslargescrollablecode/textsurfaces(candidateforprepaint-windowedlines).\")",
            "doc_layout::paragraph_text(cx,\"Usescriptedwheelsteps+stale-paintcheckstovalidatescrollstability.\")",
        ],
    );

    for forbidden in [
        "cx.text(\"Goal:stresslargescrollablecode/textsurfaces(candidateforprepaint-windowedlines).\")",
        "cx.text(\"Usescriptedwheelsteps+stale-paintcheckstovalidatescrollstability.\")",
    ] {
        assert!(
            !normalized.contains(forbidden),
            "code_view reintroduced bare explanatory header text: {forbidden}"
        );
    }
}

#[test]
fn editor_text_conformance_headers_use_text_roles() {
    for (relative_path, required, forbidden) in [
        (
            "src/ui/previews/pages/editors/text/feature_toggles.rs",
            vec![
                "doc_layout::paragraph_text(cx,\"Goal:validateOpenTypefeatureoverrides(`TextShapingStyle.features`)end-to-end.\")",
                "doc_layout::paragraph_text(cx,\"Thisisbest-effort:visibledifferencesdependonthechosenfont.Intertypicallyshows`liga`(fi/fl/ffi/ffl).\")",
                "doc_layout::paragraph_text(cx,\"Tip:setFRET_TEXT_SYSTEM_FONTS=0tovalidatethedeterministicno-system-fontspathonnative.\")",
            ],
            vec![
                "cx.text(\"Goal:validateOpenTypefeatureoverrides(`TextShapingStyle.features`)end-to-end.\")",
                "cx.text(\"Thisisbest-effort:visibledifferencesdependonthechosenfont.Intertypicallyshows`liga`(fi/fl/ffi/ffl).\")",
                "cx.text(\"Tip:setFRET_TEXT_SYSTEM_FONTS=0tovalidatethedeterministicno-system-fontspathonnative.\")",
            ],
        ),
        (
            "src/ui/previews/pages/editors/text/measure_overlay.rs",
            vec![
                "doc_layout::paragraph_text(cx,\"Goal:visualizemeasuredtextboundsvsallocatedcontainerbounds.\")",
                "doc_layout::paragraph_text(cx,\"Green=containerbounds;Yellow=measuredTextMetrics.size;Cyan=baseline.\")",
            ],
            vec![
                "cx.text(\"Goal:visualizemeasuredtextboundsvsallocatedcontainerbounds.\")",
                "cx.text(\"Green=containerbounds;Yellow=measuredTextMetrics.size;Cyan=baseline.\")",
            ],
        ),
        (
            "src/ui/previews/pages/editors/text/mixed_script_fallback.rs",
            vec![
                "doc_layout::paragraph_text(cx,\"Goal:ensuremixed-scriptfallbackstaystofu-freewithbundledfonts.\")",
                "doc_layout::paragraph_text(cx,\"Tip:setFRET_TEXT_SYSTEM_FONTS=0tovalidatethedeterministicno-system-fontspathonnative.\")",
                "doc_layout::paragraph_text(cx,\"Thispagere-injectsthebundleddefaultfontsetwheneverthelivecatalognolongerexposestheexpectedbundledfamilies.\")",
            ],
            vec![
                "cx.text(\"Goal:ensuremixed-scriptfallbackstaystofu-freewithbundledfonts.\")",
                "cx.text(\"Tip:setFRET_TEXT_SYSTEM_FONTS=0tovalidatethedeterministicno-system-fontspathonnative.\")",
                "cx.text(\"Thispagere-injectsthebundleddefaultfontsetwheneverthelivecatalognolongerexposestheexpectedbundledfamilies.\")",
            ],
        ),
        (
            "src/ui/previews/pages/editors/text/outline_stroke.rs",
            vec![
                "doc_layout::paragraph_text(cx,\"Goal:exercise`SceneOp::Text.outline:Option<TextOutlineV1>`end-to-end.\")",
                "doc_layout::paragraph_text(cx,\"Thispagedrawsthesametexttwice:fill-onlyvsfill+outline,onahigh-contrastbackdrop.\")",
                "doc_layout::paragraph_text(cx,\"Tip:setFRET_TEXT_SYSTEM_FONTS=0tovalidatedeterministicbundled-fontbehavior.\")",
            ],
            vec![
                "cx.text(\"Goal:exercise`SceneOp::Text.outline:Option<TextOutlineV1>`end-to-end.\")",
                "cx.text(\"Thispagedrawsthesametexttwice:fill-onlyvsfill+outline,onahigh-contrastbackdrop.\")",
                "cx.text(\"Tip:setFRET_TEXT_SYSTEM_FONTS=0tovalidatedeterministicbundled-fontbehavior.\")",
            ],
        ),
        (
            "src/ui/previews/pages/editors/text/selection_perf.rs",
            vec![
                "doc_layout::paragraph_text(cx,\"Goal:trackselectionrectcountforlargeselections.\")",
                "doc_layout::paragraph_text(cx,\"Expectation:rectgenerationscaleswithvisiblelineswhenclippedtotheviewport(notdocumentlength).\")",
                "doc_layout::paragraph_text(cx,\"Scrollwiththemousewheeloverthedemosurface.\")",
            ],
            vec![
                "cx.text(\"Goal:trackselectionrectcountforlargeselections.\")",
                "cx.text(\"Expectation:rectgenerationscaleswithvisiblelineswhenclippedtotheviewport(notdocumentlength).\")",
                "cx.text(\"Scrollwiththemousewheeloverthedemosurface.\")",
            ],
        ),
        (
            "src/ui/previews/pages/editors/text/bidi_rtl_conformance.rs",
            vec![
                "doc_layout::paragraph_text(cx,\"Goal:sanity-checkBiDi/RTLgeometryqueries(hit-test,caretrects,selectionrects).\")",
                "doc_layout::paragraph_text(cx,\"Usetheselectablesamplestovalidateeditor-likeselectionbehavior.\")",
                "doc_layout::paragraph_text(cx,\"Usethediagnosticpaneltoverify`hit_test_point`->caret/selectionrenderingundermixed-directionstrings.\")",
                "doc_layout::control_readout_text(cx,\"SelectableTextsamples:\")",
            ],
            vec![
                "cx.text(\"Goal:sanity-checkBiDi/RTLgeometryqueries(hit-test,caretrects,selectionrects).\")",
                "cx.text(\"Usetheselectablesamplestovalidateeditor-likeselectionbehavior.\")",
                "cx.text(\"Usethediagnosticpaneltoverify`hit_test_point`",
                "cx.text(\"SelectableTextsamples:\")",
            ],
        ),
    ] {
        let normalized = assert_normalized_markers_present(relative_path, &required);
        for forbidden in forbidden {
            assert!(
                !normalized.contains(forbidden),
                "{} reintroduced bare text for editor text/conformance header copy: {forbidden}",
                manifest_path(relative_path).display()
            );
        }
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
        "pub(incrate::ui)fnrender_doc_page(cx:&mutAppComponentCx<'_>,intro:Option<&'staticstr>,sections:Vec<DocSection>,)->implUiChild+use<>",
        "letmutout:Vec<AnyElement>=Vec::with_capacity(sections.len()+1);",
        "pub(incrate::ui)fnwrap_preview_page(cx:&mutAppComponentCx<'_>,intro:Option<&'staticstr>,section_title:&'staticstr,elements:Vec<AnyElement>,)->implUiChild+use<>",
        "FnOnce(&mutAppComponentCx<'_>)->Vec<AnyElement>",
    ] {
        assert!(
            normalized.contains(marker),
            "{} is missing intentional scaffold seam marker `{marker}`",
            path.display()
        );
    }

    assert_eq!(
        normalized
            .matches("FnOnce(&mutAppComponentCx<'_>)->Vec<AnyElement>")
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
            "pub(incrate::ui)fnrender_doc_page(cx:&mutAppComponentCx<'_>,intro:Option<&'staticstr>,sections:Vec<DocSection>,)->AnyElement"
        ),
        "{} should not regress render_doc_page back to AnyElement",
        path.display()
    );
    assert!(
        !normalized.contains(
            "pub(incrate::ui)fnwrap_preview_page(cx:&mutAppComponentCx<'_>,intro:Option<&'staticstr>,section_title:&'staticstr,elements:Vec<AnyElement>,)->AnyElement"
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
            "cx: &mut AppComponentCx<'_>",
            "cx: &mut AppComponentCx<'a>",
            "FnOnce(&mut AppComponentCx<'_>)",
            "FnOnce(&mut AppComponentCx<'a>)",
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
            "cx: &mut AppComponentCx<'_>",
            "cx: &mut AppComponentCx<'a>",
            "FnOnce(&mut AppComponentCx<'_>)",
            "FnOnce(&mut AppComponentCx<'a>)",
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
            "cx: &mut AppComponentCx<'_>",
            "cx: &mut AppComponentCx<'a>",
            "FnOnce(&mut AppComponentCx<'_>)",
            "FnOnce(&mut AppComponentCx<'a>)",
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
            "cx: &mut AppComponentCx<'_>",
            "cx: &mut AppComponentCx<'a>",
            "FnOnce(&mut AppComponentCx<'_>)",
            "FnOnce(&mut AppComponentCx<'a>)",
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
            "fn row(_cx: &mut AppComponentCx<'_>, gap: Px, children: Vec<AnyElement>) -> impl UiChild + use<>",
            "fn row_end(_cx: &mut AppComponentCx<'_>, gap: Px, children: Vec<AnyElement>) -> impl UiChild + use<>",
            "pub(super) fn compose_body(cx: &mut AppComponentCx<'_>, models: OverlayModels) -> impl UiChild + use<>",
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
            "pub(super) fn overlay_reset(_cx: &mut AppComponentCx<'_>, models: &OverlayModels) -> impl UiChild + use<>",
            "pub(super) fn dropdown(_cx: &mut AppComponentCx<'_>, models: &OverlayModels) -> impl UiChild + use<>",
            "pub(super) fn context_menu(_cx: &mut AppComponentCx<'_>, models: &OverlayModels) -> impl UiChild + use<>",
            "pub(super) fn context_menu_edge(_cx: &mut AppComponentCx<'_>, models: &OverlayModels,) -> impl UiChild + use<>",
            "pub(super) fn underlay(_cx: &mut AppComponentCx<'_>, models: &OverlayModels) -> impl UiChild + use<>",
            "pub(super) fn tooltip(cx: &mut AppComponentCx<'_>) -> impl UiChild + use<>",
            "pub(super) fn hover_card(cx: &mut AppComponentCx<'_>) -> impl UiChild + use<>",
            "pub(super) fn popover(cx: &mut AppComponentCx<'_>, models: &OverlayModels) -> impl UiChild + use<>",
            "pub(super) fn dialog(_cx: &mut AppComponentCx<'_>, models: &OverlayModels) -> impl UiChild + use<>",
            "pub(super) fn dialog_glass(_cx: &mut AppComponentCx<'_>, models: &OverlayModels) -> impl UiChild + use<>",
            "pub(super) fn alert_dialog(_cx: &mut AppComponentCx<'_>, models: &OverlayModels) -> impl UiChild + use<>",
            "pub(super) fn sheet(_cx: &mut AppComponentCx<'_>, models: &OverlayModels) -> impl UiChild + use<>",
            "pub(super) fn portal_geometry(cx: &mut AppComponentCx<'_>, models: &OverlayModels) -> impl UiChild + use<>",
        ],
    );
    assert_eq!(
        widgets_normalized.matches("->implUiChild+use<>").count(),
        13,
        "src/ui/previews/gallery/overlays/overlay/widgets.rs should keep the typed widget-helper inventory",
    );
    assert!(
        widgets_normalized.contains(
            "fnoverlay_scroll_row_text<T>(cx:&mutAppComponentCx<'_>,text:T)->implUiChild+use<T>"
        ),
        "src/ui/previews/gallery/overlays/overlay/widgets.rs should keep the typed scroll-row helper inventory"
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
            "pub(super) fn last_action_status(cx: &mut AppComponentCx<'_>, models: &OverlayModels,) -> impl UiChild + use<>",
            "pub(super) fn status_flags(cx: &mut AppComponentCx<'_>, models: &OverlayModels) -> Vec<AnyElement>",
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
    let header_path = manifest_path("src/ui/previews/pages/editors/code_editor/mvp/header.rs");
    let header_source = read_path(&header_path);
    let header_normalized = canonicalize_rust_fragment(&header_source);
    assert_imports_ui_child_with_app_component(&header_path, &header_source, &header_normalized);
    let build_header_marker = canonicalize_rust_fragment(
        "pub(super)fnbuild_header(cx:&mutAppComponentCx<'_>,theme:&Theme,syntax_rust:Model<bool>,syntax_enabled:bool,boundary_identifier:Model<bool>,boundary_identifier_enabled:bool,soft_wrap:Model<bool>,soft_wrap_enabled:bool,set_identifier_mode:fret_ui::action::OnActivate,set_unicode_mode:fret_ui::action::OnActivate,handles:&CodeEditorMvpHandles,word_fixture_loaded:Rc<Cell<bool>>,word_idx:Rc<Cell<usize>>,word_debug:Rc<std::cell::RefCell<String>>,)->implUiChild+use<>",
    );
    assert!(
        header_normalized.contains(&build_header_marker),
        "{} should keep build_header on the typed helper lane",
        header_path.display(),
    );
    let build_header_forbidden = canonicalize_rust_fragment(
        "pub(super)fnbuild_header(cx:&mutAppComponentCx<'_>,theme:&Theme,syntax_rust:Model<bool>,syntax_enabled:bool,boundary_identifier:Model<bool>,boundary_identifier_enabled:bool,soft_wrap:Model<bool>,soft_wrap_enabled:bool,set_identifier_mode:fret_ui::action::OnActivate,set_unicode_mode:fret_ui::action::OnActivate,handles:&CodeEditorMvpHandles,word_fixture_loaded:Rc<Cell<bool>>,word_idx:Rc<Cell<usize>>,word_debug:Rc<std::cell::RefCell<String>>,)->AnyElement",
    );
    assert!(
        !header_normalized.contains(&build_header_forbidden),
        "{} should not regress build_header back to AnyElement",
        header_path.display(),
    );

    let word_boundary_path =
        manifest_path("src/ui/previews/pages/editors/code_editor/mvp/word_boundary.rs");
    let word_boundary_source = read_path(&word_boundary_path);
    let word_boundary_normalized = canonicalize_rust_fragment(&word_boundary_source);
    assert_imports_ui_child_with_app_component(
        &word_boundary_path,
        &word_boundary_source,
        &word_boundary_normalized,
    );
    for marker in [
        "pub(super)fnword_boundary_controls(cx:&mutAppComponentCx<'_>,word_handle:code_editor::CodeEditorHandle,word_fixture_loaded:Rc<Cell<bool>>,word_idx:Rc<Cell<usize>>,word_debug:Rc<std::cell::RefCell<String>>,boundary_identifier:Model<bool>,)->implUiChild+use<>",
        "pub(super)fnword_boundary_debug_view(cx:&mutAppComponentCx<'_>,theme:&Theme,word_handle:code_editor::CodeEditorHandle,word_debug:Rc<std::cell::RefCell<String>>,)->implUiChild+use<>",
    ] {
        let marker = canonicalize_rust_fragment(marker);
        assert!(
            word_boundary_normalized.contains(&marker),
            "{} is missing typed helper marker `{marker}`",
            word_boundary_path.display(),
        );
    }
    for marker in [
        "pub(super)fnword_boundary_controls(cx:&mutAppComponentCx<'_>,word_handle:code_editor::CodeEditorHandle,word_fixture_loaded:Rc<Cell<bool>>,word_idx:Rc<Cell<usize>>,word_debug:Rc<std::cell::RefCell<String>>,boundary_identifier:Model<bool>,)->AnyElement",
        "pub(super)fnword_boundary_debug_view(cx:&mutAppComponentCx<'_>,theme:&Theme,word_handle:code_editor::CodeEditorHandle,word_debug:Rc<std::cell::RefCell<String>>,)->AnyElement",
    ] {
        let marker = canonicalize_rust_fragment(marker);
        assert!(
            !word_boundary_normalized.contains(&marker),
            "{} regressed helper marker `{marker}`",
            word_boundary_path.display(),
        );
    }

    let markdown_path = manifest_path("src/ui/previews/pages/editors/markdown.rs");
    let markdown_source = read_path(&markdown_path);
    let markdown_normalized = canonicalize_rust_fragment(&markdown_source);
    assert!(
        markdown_source.contains("use fret::AppComponentCx;"),
        "{} should use the shared helper context alias",
        markdown_path.display(),
    );
    for marker in [
        "doc_layout::paragraph_text(cx,\"Goal:validateaminimalMarkdownsourceeditormilestone.\")",
        "doc_layout::control_readout_text(cx,ifsoft_wrap_enabled{",
        "doc_layout::control_readout_text(cx,iffolds_enabled{",
        "doc_layout::control_readout_text(cx,ifinlays_enabled{",
        "doc_layout::button_label_text(cx,\"Preedit:inject\")",
        "doc_layout::button_label_text(cx,\"Preedit:clear\")",
        "doc_layout::control_readout_text(cx,format!(\"Interaction:{mode_label}\"))",
    ] {
        let marker = canonicalize_rust_fragment(marker);
        assert!(
            markdown_normalized.contains(&marker),
            "{} is missing marker `{marker}`",
            markdown_path.display(),
        );
    }
    for forbidden in [
        "cx.text(\"Goal: validate a minimal Markdown source editor milestone.\")",
        "cx.text(if soft_wrap_enabled {",
        "cx.text(if folds_enabled {",
        "cx.text(if inlays_enabled {",
        "cx.text(\"Preedit: inject\")",
        "cx.text(\"Preedit: clear\")",
        "cx.text(format!(\"Interaction: {mode_label}\"))",
    ] {
        assert!(
            !markdown_normalized.contains(forbidden),
            "{} reintroduced bare markdown editor text: {forbidden}",
            markdown_path.display(),
        );
    }

    let web_ime_path = manifest_path("src/ui/previews/pages/editors/web_ime.rs");
    let web_ime_source = read_path(&web_ime_path);
    let web_ime_normalized = canonicalize_rust_fragment(&web_ime_source);
    assert!(
        web_ime_source.contains("use fret::AppComponentCx;"),
        "{} should use the shared helper context alias",
        web_ime_path.display(),
    );
    for marker in [
        "fn debug_readout_text<T>(cx:&mutAppComponentCx<'_>,text:T)->AnyElement",
        "doc_layout::paragraph_text(cx,\"Goal:validatethewasmtextareaIMEbridge(ADR0180).\")",
        "doc_layout::paragraph_text(cx,\"Try:CJKIMEpreedit→commit;ensureno doubleinsertoncompositionend+input.\")",
        "doc_layout::button_label_text(cx,label)",
        "doc_layout::control_readout_text(cx,\"Editablewidgets(sanitycheck):\")",
        "debug_readout_text(cx,format!(\"harness_region_ime_enabled={harness_region_ime_enabled}\"))",
        "debug_readout_text(cx,\"window_text_input_snapshot:\")",
        "debug_readout_text(cx,\"bridge_debug_snapshot(wasmtextarea):\")",
    ] {
        let marker = canonicalize_rust_fragment(marker);
        assert!(
            web_ime_normalized.contains(&marker),
            "{} is missing marker `{marker}`",
            web_ime_path.display(),
        );
    }
    for forbidden in [
        "cx.text(\"Goal: validate the wasm textarea IME bridge (ADR 0180).\")",
        "cx.text(\"Click inside the region to focus it (IME should enable).\")",
        "cx.text(\"Editable widgets (sanity check):\")",
        "cx.text(label)",
        "cx.text(format!(\"harness_region_ime_enabled={harness_region_ime_enabled}\"))",
        "cx.text(\"window_text_input_snapshot:\")",
        "cx.text(\"bridge_debug_snapshot (wasm textarea):\")",
    ] {
        assert!(
            !web_ime_normalized.contains(forbidden),
            "{} reintroduced bare web-ime text: {forbidden}",
            web_ime_path.display(),
        );
    }

    let gates_path = manifest_path("src/ui/previews/pages/editors/code_editor/mvp/gates.rs");
    let gates_source = read_path(&gates_path);
    let gates_normalized = canonicalize_rust_fragment(&gates_source);
    assert_imports_ui_child_with_app_component(&gates_path, &gates_source, &gates_normalized);
    for marker in [
        "fngate_panel<B>(cx:&mutAppComponentCx<'_>,theme:&Theme,child:B)->implUiChild+use<B>",
        "usecrate::ui::doc_layout;",
        "pub(super)fnword_boundary_gate(cx:&mutAppComponentCx<'_>,theme:&Theme,handle:code_editor::CodeEditorHandle,)->implUiChild+use<>",
        "pub(super)fnword_boundary_soft_wrap_gate(cx:&mutAppComponentCx<'_>,theme:&Theme,handle:code_editor::CodeEditorHandle,)->implUiChild+use<>",
        "pub(super)fna11y_selection_gate(cx:&mutAppComponentCx<'_>,theme:&Theme,handle:code_editor::CodeEditorHandle,)->implUiChild+use<>",
        "pub(super)fna11y_composition_gate(cx:&mutAppComponentCx<'_>,theme:&Theme,handle:code_editor::CodeEditorHandle,)->implUiChild+use<>",
        "pub(super)fna11y_selection_wrap_gate(cx:&mutAppComponentCx<'_>,theme:&Theme,handle:code_editor::CodeEditorHandle,)->implUiChild+use<>",
        "pub(super)fna11y_composition_wrap_gate(cx:&mutAppComponentCx<'_>,theme:&Theme,handle:code_editor::CodeEditorHandle,)->implUiChild+use<>",
        "pub(super)fna11y_composition_drag_gate(cx:&mutAppComponentCx<'_>,theme:&Theme,handle:code_editor::CodeEditorHandle,)->implUiChild+use<>",
        "doc_layout::button_label_text(cx,\"Injectpreedit\")",
        "doc_layout::button_label_text(cx,\"Clearpreedit\")",
        "doc_layout::button_label_text(cx,\"IMEsetMarkedText(replaceselection)\")",
        "doc_layout::button_label_text(cx,\"IMEcancel(emptymarkedtext)\")",
        "doc_layout::button_label_text(cx,\"Injectpreedit(wrap)\")",
        "doc_layout::button_label_text(cx,\"Clearpreedit(wrap)\")",
        "doc_layout::button_label_text(cx,\"Injectpreedit(drag)\")",
        "doc_layout::button_label_text(cx,\"Clearpreedit(drag)\")",
    ] {
        let marker = canonicalize_rust_fragment(marker);
        assert!(
            gates_normalized.contains(&marker),
            "{} is missing typed helper marker `{marker}`",
            gates_path.display(),
        );
    }
    assert!(
        !gates_normalized.contains("->AnyElement"),
        "{} should not regress gate helpers back to AnyElement",
        gates_path.display(),
    );
    for forbidden in [
        "vec![cx.text(\"Inject preedit\")]",
        "vec![cx.text(\"Clear preedit\")]",
        "vec![cx.text(\"IME setMarkedText (replace selection)\")]",
        "vec![cx.text(\"IME cancel (empty marked text)\")]",
        "vec![cx.text(\"Inject preedit (wrap)\")]",
        "vec![cx.text(\"Clear preedit (wrap)\")]",
        "vec![cx.text(\"Inject preedit (drag)\")]",
        "vec![cx.text(\"Clear preedit (drag)\")]",
    ] {
        let forbidden = canonicalize_rust_fragment(forbidden);
        assert!(
            !gates_normalized.contains(&forbidden),
            "{} reintroduced bare IME gate button label text: {forbidden}",
            gates_path.display(),
        );
    }
}

#[test]
fn selected_internal_preview_helpers_prefer_typed_outputs() {
    let harness_intro_path = manifest_path("src/ui/previews/pages/harness/intro.rs");
    let harness_intro = read_path(&harness_intro_path);
    let harness_intro_normalized = canonicalize_rust_fragment(&harness_intro);
    assert_imports_ui_child_with_app_component(
        &harness_intro_path,
        &harness_intro,
        &harness_intro_normalized,
    );
    assert!(
        harness_intro_normalized.contains(
            "fncard(cx:&mutAppComponentCx<'_>,title:&'staticstr,desc:&'staticstr)->implUiChild+use<>"
        ),
        "{} should keep card on the typed helper lane",
        harness_intro_path.display(),
    );
    assert!(
        !harness_intro_normalized.contains("->AnyElement"),
        "{} should not regress card back to AnyElement",
        harness_intro_path.display(),
    );
    assert!(
        harness_intro_normalized.contains("DocSection::build(cx,\"Overview\",preview)"),
        "{} should keep overview registration on DocSection::build(cx, ...)",
        harness_intro_path.display(),
    );
    assert!(
        !harness_intro_normalized.contains("DocSection::new(\"Overview\",preview)"),
        "{} should not regress overview registration to DocSection::new(...)",
        harness_intro_path.display(),
    );

    let outline_stroke_path = manifest_path("src/ui/previews/pages/editors/text/outline_stroke.rs");
    let outline_stroke = read_path(&outline_stroke_path);
    let outline_stroke_normalized = canonicalize_rust_fragment(&outline_stroke);
    assert_imports_ui_child_with_app_component(
        &outline_stroke_path,
        &outline_stroke,
        &outline_stroke_normalized,
    );
    let outline_toggle_marker = canonicalize_rust_fragment(
        "fntoggle_button(cx:&mutAppComponentCx<'_>,label:&'staticstr,value:bool,test_id:&'staticstr,on_activate:fret_ui::action::OnActivate,)->implUiChild+use<>",
    );
    assert!(
        outline_stroke_normalized.contains(&outline_toggle_marker),
        "{} should keep toggle_button on the typed helper lane",
        outline_stroke_path.display(),
    );
    let outline_toggle_forbidden = canonicalize_rust_fragment(
        "fntoggle_button(cx:&mutAppComponentCx<'_>,label:&'staticstr,value:bool,test_id:&'staticstr,on_activate:fret_ui::action::OnActivate,)->AnyElement",
    );
    assert!(
        !outline_stroke_normalized.contains(&outline_toggle_forbidden),
        "{} should not regress toggle_button back to AnyElement",
        outline_stroke_path.display(),
    );

    let mixed_script_path =
        manifest_path("src/ui/previews/pages/editors/text/mixed_script_fallback.rs");
    let mixed_script = read_path(&mixed_script_path);
    let mixed_script_normalized = canonicalize_rust_fragment(&mixed_script);
    assert_imports_ui_child_with_app_component(
        &mixed_script_path,
        &mixed_script,
        &mixed_script_normalized,
    );
    let mixed_script_marker = canonicalize_rust_fragment(
        "fnsample_row(cx:&mutAppComponentCx<'_>,theme:&Theme,label:&'staticstr,sample:&'staticstr,test_id:&'staticstr,)->implUiChild+use<>",
    );
    assert!(
        mixed_script_normalized.contains(&mixed_script_marker),
        "{} should keep sample_row on the typed helper lane",
        mixed_script_path.display(),
    );
    let mixed_script_forbidden = canonicalize_rust_fragment(
        "fnsample_row(cx:&mutAppComponentCx<'_>,theme:&Theme,label:&'staticstr,sample:&'staticstr,test_id:&'staticstr,)->AnyElement",
    );
    assert!(
        !mixed_script_normalized.contains(&mixed_script_forbidden),
        "{} should not regress sample_row back to AnyElement",
        mixed_script_path.display(),
    );

    let feature_toggles_path =
        manifest_path("src/ui/previews/pages/editors/text/feature_toggles.rs");
    let feature_toggles = read_path(&feature_toggles_path);
    let feature_toggles_normalized = canonicalize_rust_fragment(&feature_toggles);
    assert_imports_ui_child_with_app_component(
        &feature_toggles_path,
        &feature_toggles,
        &feature_toggles_normalized,
    );
    for marker in [
        "fntoggle_button(cx:&mutAppComponentCx<'_>,label:&'staticstr,value:bool,test_id:&'staticstr,on_activate:fret_ui::action::OnActivate,)->implUiChild+use<>",
        "fnsample_text(cx:&mutAppComponentCx<'_>,theme:&Theme,label:&'staticstr,text:&'staticstr,features:Option<fret_core::TextShapingStyle>,test_id:&'staticstr,)->implUiChild+use<>",
    ] {
        let marker = canonicalize_rust_fragment(marker);
        assert!(
            feature_toggles_normalized.contains(&marker),
            "{} is missing typed helper marker `{marker}`",
            feature_toggles_path.display(),
        );
    }
    let feature_toggle_forbidden = canonicalize_rust_fragment(
        "fntoggle_button(cx:&mutAppComponentCx<'_>,label:&'staticstr,value:bool,test_id:&'staticstr,on_activate:fret_ui::action::OnActivate,)->AnyElement",
    );
    assert!(
        !feature_toggles_normalized.contains(&feature_toggle_forbidden),
        "{} should not regress toggle_button back to AnyElement",
        feature_toggles_path.display(),
    );
    let sample_text_forbidden = canonicalize_rust_fragment(
        "fnsample_text(cx:&mutAppComponentCx<'_>,theme:&Theme,label:&'staticstr,text:&'staticstr,features:Option<fret_core::TextShapingStyle>,test_id:&'staticstr,)->AnyElement",
    );
    assert!(
        !feature_toggles_normalized.contains(&sample_text_forbidden),
        "{} should not regress sample_text back to AnyElement",
        feature_toggles_path.display(),
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
            "cx: &mut AppComponentCx<'_>",
            "cx: &mut AppComponentCx<'a>",
            "FnOnce(&mut AppComponentCx<'_>)",
            "FnOnce(&mut AppComponentCx<'a>)",
            "Fn(&mut AppComponentCx<'_>)",
            "Fn(&mut AppComponentCx<'a>)",
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
            "cx: &mut AppComponentCx<'_>",
            "cx: &mut AppComponentCx<'a>",
            "FnOnce(&mut AppComponentCx<'_>)",
            "FnOnce(&mut AppComponentCx<'a>)",
            "Fn(&mut AppComponentCx<'_>)",
            "Fn(&mut AppComponentCx<'a>)",
        ],
        &[],
        "internal torture preview surface",
    );
}

#[test]
fn chart_torture_preview_uses_declarative_chart_panel() {
    let path = manifest_path("src/ui/previews/pages/torture/chart_torture.rs");
    let source = read_path(&path);
    let canonical = canonicalize_rust_fragment(&source);

    for marker in [
        "ChartEngine::new",
        "ChartCanvasPanelProps::new",
        "chart_canvas_panel_in(cx,props)",
        "UiGalleryChartTortureOutputHandle{output:output.clone(),engine:engine.clone(),}",
    ] {
        let marker = canonicalize_rust_fragment(marker);
        assert!(
            canonical.contains(&marker),
            "chart_torture.rs should keep declarative chart torture marker `{marker}`"
        );
    }

    for marker in [
        "RetainedSubtreeProps",
        "UiTreeRetainedExt",
        "cx.retained_subtree",
        "ChartCanvas::new_shared",
        "use fret_chart::ChartCanvas;",
        "shared_engine:Rc<RefCell<delinea::engine::ChartEngine>>",
    ] {
        let marker = canonicalize_rust_fragment(marker);
        assert!(
            !canonical.contains(&marker),
            "chart_torture.rs reintroduced retained chart torture marker `{marker}`"
        );
    }
}

#[test]
fn gallery_torture_preview_modules_prefer_ui_cx_on_the_internal_gallery_surface() {
    assert_internal_preview_dir(
        "src/ui/previews/gallery/torture",
        &["cx: &mut", "FnOnce(&mut", "Fn(&mut"],
        &[
            "cx: &mut AppComponentCx<'_>",
            "cx: &mut AppComponentCx<'a>",
            "FnOnce(&mut AppComponentCx<'_>)",
            "FnOnce(&mut AppComponentCx<'a>)",
            "Fn(&mut AppComponentCx<'_>)",
            "Fn(&mut AppComponentCx<'a>)",
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
            "fnretained_table_cell_text<T>(cx:&mutAppComponentCx<'_>,text:T)->AnyElement",
            "fret_ui_kit::declarative::text::text_table_cell(cx,text)",
            "doc_layout::paragraph_text(cx,\"Goal:baselineharnessfor`fret-ui-kit::declarative::table`runningonthevirt-003retainedhostpath.\")",
            "doc_layout::paragraph_text(cx,\"Usescriptedsort/selection+scrolltovalidatereconciledeltasunderview-cachereuse(nonotify-baseddirtyviews).\")",
            "doc_layout::control_readout_text(cx,\"Keeppinnedrows\")",
            "doc_layout::control_readout_text(cx,sorting_text.clone()).attach_semantics(",
            "retained_table_cell_text(cx,row.name.clone())",
            "retained_table_cell_text(cx,row.status.clone())",
            "retained_table_cell_text(cx,format!(\"{}%\",row.cpu))",
            "retained_table_cell_text(cx,format!(\"{}MB\",row.mem_mb))",
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
    for forbidden in [
        "cx.text(\"Goal:baselineharnessfor`fret-ui-kit::declarative::table`runningonthevirt-003retainedhostpath.\")",
        "cx.text(\"Usescriptedsort/selection+scrolltovalidatereconciledeltasunderview-cachereuse(nonotify-baseddirtyviews).\")",
        "cx.text(\"Keeppinnedrows\")",
        "cx.text(sorting_text.as_ref()).attach_semantics(",
        "cx.text(row_pinning_text.as_ref()).attach_semantics(",
        "cx.text(keep_pinned_rows_text.as_ref()).attach_semantics(",
        "cx.text(page_text.as_ref()).attach_semantics(",
        "cx.text(row.name.as_ref())",
        "cx.text(row.status.as_ref())",
        "cx.text(format!(\"{}%\",row.cpu))",
        "cx.text(format!(\"{}MB\",row.mem_mb))",
    ] {
        assert!(
            !normalized.contains(forbidden),
            "table_retained_torture reintroduced bare fixed table/readout text: {forbidden}"
        );
    }
}

#[test]
fn gallery_data_table_torture_exposes_header_row_anchor() {
    let normalized = assert_normalized_markers_present(
        "src/ui/previews/gallery/data/table_torture.rs",
        &[
            "fndata_table_torture_cell_text<T>(cx:&mutAppComponentCx<'_>,text:T)->AnyElement",
            "fret_ui_kit::declarative::text::text_table_cell(cx,text)",
            "doc_layout::paragraph_text(cx,\"Goal:baselineperfharnessforavirtualizedbusinesstable(TanStack-alignedheadlessengine+VirtualList).\")",
            "doc_layout::paragraph_text(cx,\"Usescriptedscroll+bundlestatstovalidatecache-rootreuseandprepaint-drivenwindowingrefactors.\")",
            "doc_layout::control_readout_text(cx,sorting_text.clone()).attach_semantics(",
            "doc_layout::control_readout_text(cx,pinning_text.clone()).attach_semantics(",
            "doc_layout::control_readout_text(cx,global_filter_text.clone()).attach_semantics(",
            "doc_layout::control_readout_text(cx,name_filter_text.clone()).attach_semantics(",
            "doc_layout::control_readout_text(cx,status_filter_text.clone()).attach_semantics(",
            "data_table_torture_cell_text(cx,row.name.clone())",
            "data_table_torture_cell_text(cx,row.status.clone())",
            "data_table_torture_cell_text(cx,format!(\"{}%\",row.cpu))",
            "data_table_torture_cell_text(cx,format!(\"{}MB\",row.mem_mb))",
            "header_row_test_id: Some(Arc::<str>::from(\"ui-gallery-data-table-header-row\",)),",
        ],
    );

    assert!(
        normalized.contains("ui-gallery-data-table-header-")
            && normalized.contains("ui-gallery-data-table-row-"),
        "table_torture should keep the structured data-table header/body diagnostics prefixes alongside the header-row anchor"
    );
    for forbidden in [
        "cx.text(\"Goal:baselineperfharnessforavirtualizedbusinesstable(TanStack-alignedheadlessengine+VirtualList).\")",
        "cx.text(\"Usescriptedscroll+bundlestatstovalidatecache-rootreuseandprepaint-drivenwindowingrefactors.\")",
        "cx.text(sorting_text.as_ref()).attach_semantics(",
        "cx.text(pinning_text.as_ref()).attach_semantics(",
        "cx.text(global_filter_text.as_ref()).attach_semantics(",
        "cx.text(name_filter_text.as_ref()).attach_semantics(",
        "cx.text(status_filter_text.as_ref()).attach_semantics(",
        "cx.text(row.name.as_ref())",
        "cx.text(row.status.as_ref())",
        "cx.text(format!(\"{}%\",row.cpu))",
        "cx.text(format!(\"{}MB\",row.mem_mb))",
    ] {
        assert!(
            !normalized.contains(forbidden),
            "table_torture reintroduced bare fixed table/readout text: {forbidden}"
        );
    }
}

#[test]
fn gallery_data_grid_uses_table_cell_text_roles() {
    let normalized = assert_normalized_markers_present(
        "src/ui/previews/gallery/data/data_grid.rs",
        &[
            "fndata_grid_cell_text<T>(cx:&mutAppComponentCx<'_>,text:T)->AnyElement",
            "fret_ui_kit::declarative::text::text_table_cell(cx,text)",
            "data_grid_cell_text(cx,pid.to_string())",
            "data_grid_cell_text(cx,format!(\"Process{row}\"))",
            "data_grid_cell_text(cx,ifrow%3==0{\"Running\"}else{\"Idle\"})",
            "data_grid_cell_text(cx,((row*7)%100).to_string())",
            "doc_layout::paragraph_text(cx,\"Virtualizedrows/colsviewport;clickarowtoselect(disabledevery17throw).\")",
            "doc_layout::control_readout_text(cx,format!(\"Selectedrow:{selected_text}\"))",
        ],
    );

    for forbidden in [
        "cx.text(pid.to_string())",
        "cx.text(format!(\"Process{row}\"))",
        "cx.text(ifrow%3==0{\"Running\"}else{\"Idle\"})",
        "cx.text(((row*7)%100).to_string())",
        "cx.text(\"Virtualizedrows/colsviewport;clickarowtoselect(disabledevery17throw).\")",
        "cx.text(format!(\"Selectedrow:{selected_text}\"))",
    ] {
        assert!(
            !normalized.contains(forbidden),
            "data_grid reintroduced bare fixed grid/readout text: {forbidden}"
        );
    }
}

#[test]
fn gallery_inspector_torture_uses_fixed_row_text_roles() {
    let normalized = assert_normalized_markers_present(
        "src/ui/previews/gallery/torture/inspector_torture.rs",
        &[
            "usefret_core::{AttributedText,Edges,TextSpan};",
            "usefret_ui_kit::ColorRef;",
            "usefret_ui_kit::typography::{UiTextSize,control_text_style,muted_foreground_color};",
            "fninspector_row_label_value_text(",
            "letmutvalue_span=TextSpan::new(value.len());",
            "value_span.paint.fg=Some(value_color);",
            "AttributedText::new(text,Arc::<[TextSpan]>::from([TextSpan::new(label.len()),TextSpan::new(1),value_span]))",
            "letlabel_color=theme.color_token(\"foreground\");",
            "letvalue_color=muted_foreground_color(theme);",
            "lettext_style=control_text_style(theme,UiTextSize::Sm);",
            "label_color,",
            "value_color,",
            "ColorRef::Color(label_color)",
            ".test_id(inspector_row_label_test_id(index))",
        ],
    );

    for forbidden in [
        "fninspector_row_label_text<T>(cx:&mutAppComponentCx<'_>,text:T)->AnyElement",
        "fninspector_row_value_text<T>(cx:&mutAppComponentCx<'_>,text:T)->AnyElement",
        "fret_ui_kit::declarative::text::text_list_row_label(cx,text)",
        "doc_layout::control_readout_text(cx,text)",
        "ui::h_flex(|_cx|[name,value])",
        "inspector_row_label_text(cx,format!(\"prop_{index}\"))",
        "inspector_row_value_text(cx,format!(\"value{index}\"))",
        ".test_id(inspector_row_value_test_id(index))",
    ] {
        assert!(
            !normalized.contains(forbidden),
            "inspector_torture reintroduced bare fixed inspector-row text: {forbidden}"
        );
    }
}

#[test]
fn gallery_inspector_torture_stamps_row_root_semantics_and_action_state() {
    let normalized = assert_normalized_markers_present(
        "src/ui/previews/gallery/torture/inspector_torture.rs",
        &[
            "fninspector_row_test_id(index:usize)->Arc<str>",
            "fninspector_row_label_test_id(index:usize)->Arc<str>",
            "fninspector_row_semantics(index:usize,len:usize,selected:bool)->PressableA11y",
            "PressableA11y{role:Some(fret_core::SemanticsRole::ListItem),",
            "test_id:Some(inspector_row_test_id(index)),",
            "selected_row_value==Some(index)",
            "cx.pressable_add_on_activate(on_select_row.clone())",
            "cx.pressable_add_on_activate_focus(Arc::new(",
            "host.request_focus(root_id);",
            "row.test_id(inspector_row_test_id(index))",
            "SemanticsDecoration::default().test_id(inspector_row_label_test_id(index))",
        ],
    );

    for forbidden in [
        "row.test_id(format!(\"ui-gallery-inspector-row-{index}-label\"))",
        "row.test_id(format!(\"ui-gallery-inspector-row-{index}\"))",
        "fninspector_row_value_test_id(index:usize)->Arc<str>",
    ] {
        assert!(
            !normalized.contains(forbidden),
            "inspector_torture row-root semantics guard should stay on helper-based ids: {forbidden}"
        );
    }
}

#[test]
fn gallery_inspector_torture_keeps_selected_row_model_on_paint_invalidation() {
    let normalized = assert_normalized_markers_present(
        "src/ui/previews/gallery/torture/inspector_torture.rs",
        &[
            "letselected_row_value=cx.get_model_copied(&selected_row,Invalidation::Paint).flatten();",
        ],
    );

    let selected_row_read = normalized
        .find("letselected_row_value=cx.get_model_copied(&selected_row,Invalidation::Paint).flatten();")
        .expect("inspector_torture selected-row model read");
    let row_closure = normalized
        .find("letrow=move|cx:&mutAppComponentCx<'_>,index:usize|{")
        .expect("inspector_torture row closure");
    assert!(
        selected_row_read < row_closure,
        "inspector_torture selected-row model should be read before the row closure so the retained list does not pay per-row model reads"
    );

    assert!(
        !normalized.contains("get_model_copied(&selected_row,Invalidation::Layout)"),
        "inspector_torture selected-row model should stay on paint invalidation because it only affects selection chrome and semantics"
    );
}

#[test]
fn gallery_inspector_torture_keeps_row_shell_shrunk() {
    let normalized = assert_normalized_markers_present(
        "src/ui/previews/gallery/torture/inspector_torture.rs",
        &[
            "letrow_gap_px=MetricRef::space(Space::N2).resolve(theme)",
            "letaccent_color=theme.color_token(\"accent\")",
            "letmuted_color=theme.color_token(\"muted\")",
            "letbackground_color=theme.color_token(\"background\")",
            "letrow_padding_left=Px(indent_px.0+row_gap_px.0*2.0)",
            "letrow_content=inspector_row_label_value_text(",
            "SemanticsDecoration::default().test_id(inspector_row_label_test_id(index))",
            "chrome.background=Some(ifst.pressed{accent_color}else{row_background})",
            "chrome.layout.size.width=Length::Fill;",
            "chrome.layout.size.height=Length::Fill;",
            "chrome.padding=Edges{top:row_gap_px,right:row_gap_px,bottom:row_gap_px,left:row_padding_left,}.into();",
            "ui::container_props(chrome,move|_cx|[row_content]).into_element(cx)",
            "[row_content]",
        ],
    );

    for forbidden in [
        "letspacer=cx.spacer(",
        "vec![cx.container(row_props,|cx|{",
        "ui::h_flex(|_cx|vec![spacer,name,value])",
    ] {
        assert!(
            !normalized.contains(forbidden),
            "inspector_torture row shell should stay shrunk without the older spacer/container wrappers: {forbidden}"
        );
    }
}

#[test]
fn gallery_code_view_torture_keeps_page_preview_fixed_height_shell() {
    let source = read_path(&manifest_path("src/ui/content.rs"));

    assert!(
        source.contains("selected == PAGE_CODE_VIEW_TORTURE"),
        "code_view_torture should keep the fixed preview shell",
    );
    assert!(
        source.contains("CODE_VIEW_TORTURE_PREVIEW_HEIGHT"),
        "code_view_torture should keep the existing fixed preview height constant",
    );
    assert!(
        source.contains("preview_semantics_layout.overflow = fret_ui::element::Overflow::Clip;"),
        "code_view_torture should keep the clipped fixed preview shell",
    );
    assert!(
        source.contains("scroll = scroll.viewport_focus_ring(false);"),
        "code_view_torture should keep the viewport focus ring wrapper disabled on the torture surface",
    );
}

#[test]
fn gallery_code_view_torture_can_disable_the_outer_content_scroll_shell() {
    let source = read_path(&manifest_path("src/ui/content.rs"));

    assert!(
        source.contains("disable_content_scroll_for_code_view"),
        "code_view_torture should have an explicit content-scroll bypass gate",
    );
    assert!(
        source.contains("selected == PAGE_CODE_VIEW_TORTURE"),
        "code_view_torture should key the bypass gate off the torture page selection",
    );
    assert!(
        source.contains("|| disable_content_scroll_for_code_view"),
        "code_view_torture should join the bypass with the existing content-scroll guard",
    );
}

#[test]
fn gallery_inspector_torture_keeps_its_own_fixed_preview_shell() {
    let source = read_path(&manifest_path("src/ui/content.rs"));

    assert!(
        source.contains("selected == PAGE_INSPECTOR_TORTURE"),
        "inspector_torture should keep its own fixed preview shell",
    );
    assert!(
        source.contains("INSPECTOR_TORTURE_PREVIEW_HEIGHT"),
        "inspector_torture should keep its own fixed preview height constant",
    );
    assert!(
        source.contains("preview_semantics_layout.overflow = fret_ui::element::Overflow::Clip;"),
        "inspector_torture should keep the clipped fixed preview shell",
    );
}

#[test]
fn gallery_inspector_torture_keeps_tight_virtual_list_overscan() {
    let normalized = assert_normalized_markers_present(
        "src/ui/previews/gallery/torture/inspector_torture.rs",
        &[
            "letoverscan=8",
            "VirtualListOptions::fixed(row_height,overscan)",
            "options.key_cache=fret_ui::element::VirtualListKeyCacheMode::VisibleOnly",
            ".keep_alive(keep_alive)",
        ],
    );

    assert!(
        !normalized.contains("letoverscan=4") && !normalized.contains("VirtualListOptions::known("),
        "inspector_torture should stay on the tighter overscan budget so the retained window does not grow wider than needed"
    );
}

#[test]
fn gallery_inspector_torture_wraps_the_retained_list_in_a_stable_root_semantics_host() {
    let normalized = assert_normalized_markers_present(
        "src/ui/previews/gallery/torture/inspector_torture.rs",
        &[
            "letroot=cx.semantics_with_id(",
            "SemanticsProps{role:fret_core::SemanticsRole::List,",
            "test_id:Some(Arc::from(\"ui-gallery-inspector-root\")),",
            "vec![list]",
            "vec![root]",
        ],
    );

    assert!(
        !normalized.contains("cx.cached_subtree_with("),
        "inspector_torture should not reintroduce an extra cached subtree wrapper around the retained list",
    );
    assert!(
        !normalized
            .contains("CachedSubtreeProps::default().contain_layout_when_bounds_known(true)"),
        "inspector_torture should keep the retained list boundary direct even with the stable root semantics host",
    );
}

#[test]
fn gallery_content_shell_keeps_page_semantics_on_the_landed_content_root() {
    let normalized = assert_normalized_markers_present(
        "src/ui/content.rs",
        &[
            "let content = content.attach_semantics(",
            "SemanticsDecoration::default()",
            ".role(fret_core::SemanticsRole::Group)",
            ".test_id(page_test_id)",
            "cx.container(",
            ".attach_semantics(",
            ".test_id(Arc::from(\"ui-gallery-content-shell\"))",
        ],
    );

    assert!(
        !normalized.contains("cx.named(\"ui_gallery.content_view_root\""),
        "gallery content shell should not regress to a dedicated named content-view root",
    );
}

#[test]
fn harness_virtual_list_torture_uses_fixed_row_text_roles() {
    let normalized = assert_normalized_markers_present(
        "src/ui/previews/pages/harness/virtual_list_torture.rs",
        &[
            "fnvirtual_list_row_label_text<T>(cx:&mutAppComponentCx<'_>,text:T)->AnyElement",
            "fret_ui_kit::declarative::text::text_list_row_label(cx,text)",
            "fnvirtual_list_row_detail_text<T>(cx:&mutAppComponentCx<'_>,text:T)->AnyElement",
            "doc_layout::control_readout_text(cx,text)",
            "doc_layout::paragraph_text(cx,\"Goal:deterministicvirtualizationtorturesurface(10krows+scroll-to-item+inlineedit).\")",
            "doc_layout::control_readout_text(cx,ifretained_host{",
            "doc_layout::control_readout_text(cx,ifknown_heights{",
            "doc_layout::control_readout_text(cx,ifkeep_alive>0{",
            "doc_layout::paragraph_text(cx,\"Harness:minimal(nofocusablecontrols;reducesRAF/notifynoiseinperfbundles).\")",
            "virtual_list_row_detail_text(cx,format!(\"Editingrow:{row}\"))",
            "virtual_list_row_detail_text(cx,\"Editingrow:<none>\")",
            "virtual_list_row_label_text(cx,format!(\"Row{index}\"))",
            "virtual_list_row_detail_text(cx,format!(\"Details:index={index}seed={}repeat={}\"",
        ],
    );

    for forbidden in [
        "cx.text(format!(\"Editingrow:{row}\"))",
        "cx.text(\"Editingrow:<none>\")",
        "cx.text(format!(\"Row{index}\"))",
        "cx.text(format!(\"Details:index={index}seed={}repeat={}\"",
        "cx.text(\"Goal:deterministicvirtualizationtorturesurface(10krows+scroll-to-item+inlineedit).\")",
        "cx.text(ifretained_host{",
        "cx.text(ifknown_heights{",
        "cx.text(ifkeep_alive>0{",
        "cx.text(\"Harness:minimal(nofocusablecontrols;reducesRAF/notifynoiseinperfbundles).\")",
    ] {
        assert!(
            !normalized.contains(forbidden),
            "virtual_list_torture reintroduced bare fixed virtual-row text: {forbidden}"
        );
    }
}

#[test]
fn harness_ui_kit_list_torture_uses_fixed_row_text_roles() {
    let normalized = assert_normalized_markers_present(
        "src/ui/previews/pages/harness/ui_kit_list_torture.rs",
        &[
            "fnui_kit_list_row_label_text<T>(cx:&mutAppComponentCx<'_>,text:T)->AnyElement",
            "fret_ui_kit::declarative::text::text_list_row_label(cx,text)",
            "doc_layout::paragraph_text(cx,\"Goal:validatefret-ui-kitlistvirtualizationunderview-cache+shellreuse(ADR0177).\")",
            "doc_layout::paragraph_text(cx,\"Expect:scrollboundaryshiftsreconcilewithoutscroll-windowdirtyviews.\")",
            "ui_kit_list_row_label_text(cx,format!(\"Item{i}\"))",
        ],
    );

    for forbidden in [
        "cx.text(format!(\"Item{i}\"))",
        "cx.text(\"Goal:validatefret-ui-kitlistvirtualizationunderview-cache+shellreuse(ADR0177).\")",
        "cx.text(\"Expect:scrollboundaryshiftsreconcilewithoutscroll-windowdirtyviews.\")",
    ] {
        assert!(
            !normalized.contains(forbidden),
            "ui_kit_list_torture reintroduced bare header/list text: {forbidden}"
        );
    }
}

#[test]
fn harness_view_cache_uses_fixed_row_text_roles() {
    let normalized = assert_normalized_markers_present(
        "src/ui/previews/pages/harness/view_cache.rs",
        &[
            "fnview_cache_list_row_label_text<T>(cx:&mutAppComponentCx<'_>,text:T)->AnyElement",
            "fret_ui_kit::declarative::text::text_list_row_label(cx,text)",
            "view_cache_list_row_label_text(cx,format!(\"Row{i}\"))",
            "doc_layout::paragraph_text(cx,\"Goal:validatecached-subtreecorrectnessunderrealinteraction.\")",
            "doc_layout::control_readout_text(cx,format!(\"Currentsettings:view_cache={}shell_cache={}content_cache={}inner_cache={}continuous={}\"",
            "doc_layout::control_label_text(cx,\"Enableview-cachemode(globalUiTreeflag)\")",
            "doc_layout::control_label_text(cx,\"Cacheshell(sidebar/contentwrappers)\")",
            "doc_layout::control_label_text(cx,\"Cachecontentroot(requires'Cacheshell')\")",
            "doc_layout::control_label_text(cx,\"EnableinnerViewCacheboundary(torturesubtree)\")",
            "doc_layout::control_label_text(cx,\"Continuousframes(cache-hitshouldstillkeepstatealive)\")",
            "doc_layout::paragraph_text(cx,\"Popovercontent\")",
        ],
    );

    for forbidden in [
        "vec![cx.text(format!(\"Row{i}\"))]",
        "cx.text(\"Goal:validatecached-subtreecorrectnessunderrealinteraction.\")",
        "cx.text(format!(\"Currentsettings:view_cache={}shell_cache={}content_cache={}inner_cache={}continuous={}\"",
        "cx.text(\"Enableview-cachemode(globalUiTreeflag)\")",
        "cx.text(\"Cacheshell(sidebar/contentwrappers)\")",
        "cx.text(\"Cachecontentroot(requires'Cacheshell')\")",
        "cx.text(\"EnableinnerViewCacheboundary(torturesubtree)\")",
        "cx.text(\"Continuousframes(cache-hitshouldstillkeepstatealive)\")",
        "cx.text(\"Popovercontent\")",
    ] {
        assert!(
            !normalized.contains(forbidden),
            "view_cache reintroduced bare fixed control/list text: {forbidden}"
        );
    }
}

#[test]
fn gallery_tree_torture_uses_control_readout_for_status_text() {
    let normalized = assert_normalized_markers_present(
        "src/ui/previews/gallery/data/tree_torture.rs",
        &[
            "doc_layout::paragraph_text(cx,\"Goal:baselineperfharnessforavirtualizedtree(expand/collapse+selection+scroll).\")",
            "doc_layout::paragraph_text(cx,\"Usescriptedscroll+bundlestatstovalidatecache-rootreuseandprepaint-drivenwindowingrefactors.\")",
            "doc_layout::control_readout_text(cx,status.clone())",
            ".test_id(\"ui-gallery-tree-target-disabled-status\")",
        ],
    );

    for forbidden in [
        "cx.text(\"Goal:baselineperfharnessforavirtualizedtree(expand/collapse+selection+scroll).\")",
        "cx.text(\"Usescriptedscroll+bundlestatstovalidatecache-rootreuseandprepaint-drivenwindowingrefactors.\")",
        "ui::text(status.clone()).text_sm().text_color(",
        "cx.text(status.clone())",
    ] {
        assert!(
            !normalized.contains(forbidden),
            "tree_torture reintroduced local fixed status text policy: {forbidden}"
        );
    }
}

#[test]
fn gallery_overlay_status_text_uses_control_readout_roles() {
    let normalized = assert_normalized_markers_present(
        "src/ui/previews/gallery/overlays/overlay/flags.rs",
        &[
            "fnoverlay_status_text<T>(cx:&mutAppComponentCx<'_>,text:T)->implUiChild+use<T>",
            "doc_layout::control_readout_text(cx,text)",
            "overlay_status_text(cx,text).test_id(\"ui-gallery-overlay-last-action\")",
            "overlay_status_text(cx,\"Popoverdismissed\").test_id(\"ui-gallery-popover-dismissed\")",
            "overlay_status_text(cx,\"Dialogopen\").test_id(\"ui-gallery-dialog-open\")",
            "overlay_status_text(cx,\"Dialog(Glass)open\").test_id(\"ui-gallery-dialog-glass-open\")",
            "overlay_status_text(cx,\"Underlayactivated\").test_id(\"ui-gallery-overlay-underlay-activated\")",
            "overlay_status_text(cx,\"AlertDialogopen\").test_id(\"ui-gallery-alert-dialog-open\")",
        ],
    );

    for forbidden in [
        "cx.text(text).test_id(\"ui-gallery-overlay-last-action\")",
        "cx.text(\"Popoverdismissed\")",
        "cx.text(\"Dialogopen\")",
        "cx.text(\"Dialog(Glass)open\")",
        "cx.text(\"Underlayactivated\")",
        "cx.text(\"AlertDialogopen\")",
    ] {
        assert!(
            !normalized.contains(forbidden),
            "overlay flags reintroduced bare status text: {forbidden}"
        );
    }
}

#[test]
fn gallery_menus_last_action_uses_control_readout_role() {
    let normalized = assert_normalized_markers_present(
        "src/ui/previews/gallery/overlays/menus.rs",
        &["doc_layout::control_readout_text(cx,format!(\"lastaction:{last}\"))"],
    );

    assert!(
        !normalized.contains("cx.text(format!(\"lastaction:{last}\"))"),
        "menus preview reintroduced bare last-action status text"
    );
}

#[test]
fn gallery_overlay_scroll_rows_use_list_row_roles() {
    let normalized = assert_normalized_markers_present(
        "src/ui/previews/gallery/overlays/overlay/widgets.rs",
        &[
            "fnoverlay_scroll_row_text<T>(cx:&mutAppComponentCx<'_>,text:T)->implUiChild+use<T>",
            "fret_ui_kit::declarative::text::text_list_row_label(cx,text)",
            "overlay_scroll_row_text(cx,format!(\"Scrollablecontentline{}\",i+1))",
            "overlay_scroll_row_text(cx,format!(\"Sheetbodyline{}\",i+1))",
            "overlay_scroll_row_text(cx,format!(\"Scrollitem{i:02}\"))",
        ],
    );

    for forbidden in [
        "cx.text(format!(\"Scrollablecontentline{}\",i+1))",
        "cx.text(format!(\"Sheetbodyline{}\",i+1))",
        "cx.text(format!(\"Scrollitem{i:02}\"))",
    ] {
        assert!(
            !normalized.contains(forbidden),
            "overlay widgets reintroduced bare scroll-row text: {forbidden}"
        );
    }
}

#[test]
fn gallery_overlay_body_copy_uses_paragraph_roles() {
    let normalized = assert_normalized_markers_present(
        "src/ui/previews/gallery/overlays/overlay/widgets.rs",
        &[
            "doc_layout::paragraph_text(cx,\"HoverCardcontent(overlay-root)\")",
            "doc_layout::paragraph_text(cx,\"Movepointerfromtriggertocontent.\")",
            "doc_layout::paragraph_text(cx,\"Popovercontent(placement+clamp)\")",
            "doc_layout::paragraph_text(cx,\"Wheel-scrolltheviewportwhileopen.\")",
        ],
    );

    for forbidden in [
        "cx.text(\"HoverCardcontent(overlay-root)\")",
        "cx.text(\"Movepointerfromtriggertocontent.\")",
        "cx.text(\"Popovercontent(placement+clamp)\")",
        "cx.text(\"Wheel-scrolltheviewportwhileopen.\")",
    ] {
        assert!(
            !normalized.contains(forbidden),
            "overlay body copy reintroduced bare paragraph text: {forbidden}"
        );
    }
}

#[test]
fn page_chrome_torture_uses_control_label_roles() {
    let doc_layout = assert_normalized_markers_present(
        "src/ui/doc_layout.rs",
        &[
            "pub(incrate::ui)fncontrol_label_text<T>(cx:&mutAppComponentCx<'_>,text:T)->AnyElement",
            "decl_text::text_control_label(cx,text)",
        ],
    );
    assert!(
        !doc_layout.contains("fncontrol_label_text_props"),
        "doc_layout should delegate control labels to the shared text role"
    );

    let normalized = assert_normalized_markers_present(
        "src/ui/previews/pages/torture/chrome_torture.rs",
        &[
            "doc_layout::control_label_text(cx,\"Textinput\")",
            "doc_layout::control_label_text(cx,\"Textarea\")",
        ],
    );

    for forbidden in ["cx.text(\"Textinput\")", "cx.text(\"Textarea\")"] {
        assert!(
            !normalized.contains(forbidden),
            "chrome_torture reintroduced bare fixed control label text: {forbidden}"
        );
    }
}
