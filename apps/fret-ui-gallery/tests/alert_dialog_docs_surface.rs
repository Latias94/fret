fn normalize_ws(source: &str) -> String {
    source.split_whitespace().collect()
}

#[test]
fn alert_dialog_page_documents_source_axes_and_children_api_decision() {
    let source = include_str!("../src/ui/pages/alert_dialog.rs");

    for needle in [
        "Reference stack: shadcn Alert Dialog docs and examples, the default registry recipe, Radix Primitives alert-dialog semantics, and Base UI alert-dialog ownership.",
        "`AlertDialog::children([...])` is the default copyable root path for part-based composition, and `AlertDialogPart` is available on the curated `shadcn` facade so the default import lane stays copyable.",
        "`AlertDialogPart::content_with(...)` plus `AlertDialogContent::with_children(...)`, `AlertDialogHeader::with_children(...)`, and `AlertDialogFooter::with_children(...)` form the default copyable content lane when child parts need the current alert-dialog scope.",
        "`AlertDialog::children([...])` is already the warranted composable root API here because the component owns Trigger/Portal/Overlay/Content parts and the scope-sensitive `from_scope(...)` buttons must stay inside `AlertDialogContent`; no broader untyped JSX-style root children API is warranted beyond the typed `AlertDialogPart` lane.",
        "Radix Primitives and Base UI agree on the relevant semantics axis here: modal alert dialog, `role=alertdialog`, outside press does not dismiss, and initial focus prefers the least-destructive `Cancel` action. Those outcomes are already handled in `fret-ui-kit` / `fret-ui-shadcn`, so this page is now mainly a docs/public-surface alignment task rather than a `fret-ui` mechanism bug.",
        "Preview mirrors the shadcn docs path after skipping `Installation`: `Demo`, `Usage`, `Basic`, `Small`, `Media`, `Small with Media`, `Destructive`, `RTL`, and `API Reference`.",
        "`Usage` is the default copyable path; `Parts` remains an advanced adapter lane for explicit root-part ownership.",
        ".test_id_prefix(\"ui-gallery-alert-dialog-demo-docsec\")",
        ".test_id_prefix(\"ui-gallery-alert-dialog-small-docsec\")",
        ".test_id_prefix(\"ui-gallery-alert-dialog-media-docsec\")",
        ".test_id_prefix(\"ui-gallery-alert-dialog-destructive-docsec\")",
        ".test_id_prefix(\"ui-gallery-alert-dialog-parts-docsec\")",
        ".test_id_prefix(\"ui-gallery-alert-dialog-detached-trigger-docsec\")",
        ".test_id_prefix(\"ui-gallery-alert-dialog-rich-content-docsec\")",
    ] {
        assert!(
            source.contains(needle),
            "alert dialog page should document source axes and the children-api decision; missing `{needle}`",
        );
    }

    let normalized = normalize_ws(source);
    let ordered_sections = normalize_ws(
        r#"
        vec![
            demo,
            usage,
            basic,
            small,
            media,
            small_with_media,
            destructive,
            rtl,
            api_reference,
            extras,
            parts,
            detached_trigger,
            rich_content,
            notes,
        ]
        "#,
    );
    assert!(
        normalized.contains(&ordered_sections),
        "alert dialog page should keep the docs-path sections before the explicit Fret follow-ups",
    );
}

#[test]
fn alert_dialog_docs_path_snippets_stay_copyable_and_docs_aligned() {
    let usage = include_str!("../src/ui/snippets/alert_dialog/usage.rs");
    let basic = include_str!("../src/ui/snippets/alert_dialog/basic.rs");
    let small = include_str!("../src/ui/snippets/alert_dialog/small.rs");
    let media = include_str!("../src/ui/snippets/alert_dialog/media.rs");
    let small_with_media = include_str!("../src/ui/snippets/alert_dialog/small_with_media.rs");
    let destructive = include_str!("../src/ui/snippets/alert_dialog/destructive.rs");
    let rtl = include_str!("../src/ui/snippets/alert_dialog/rtl.rs");

    for needle in [
        "use fret::{UiChild, UiCx};",
        "use fret_ui_shadcn::facade as shadcn;",
        "shadcn::AlertDialog::new_controllable(cx, None, false)",
        "shadcn::AlertDialogPart::trigger(",
        "shadcn::AlertDialogPart::content_with(|cx| {",
        "shadcn::AlertDialogContent::new([]).with_children(cx, |cx| {",
        "shadcn::Button::new(\"Show Dialog\").variant(shadcn::ButtonVariant::Outline)",
    ] {
        assert!(
            usage.contains(needle),
            "alert dialog usage snippet should remain a complete copyable docs-path example; missing `{needle}`",
        );
    }

    for needle in [
        "\"ui-gallery-alert-dialog-basic-trigger\"",
        "\"ui-gallery-alert-dialog-basic-content\"",
        "\"Are you absolutely sure?\"",
        "\"Continue\"",
    ] {
        assert!(
            basic.contains(needle),
            "alert dialog basic snippet should keep the upstream-shaped starter example stable; missing `{needle}`",
        );
    }

    for needle in [
        ".size(shadcn::AlertDialogContentSize::Sm)",
        "\"Allow accessory to connect?\"",
        "\"ui-gallery-alert-dialog-small-content\"",
    ] {
        assert!(
            small.contains(needle),
            "alert dialog small snippet should keep the compact-size docs example stable; missing `{needle}`",
        );
    }

    for needle in [
        "shadcn::AlertDialogMedia::new(icon).into_element(cx)",
        "\"Share Project\"",
        "\"ui-gallery-alert-dialog-media-content\"",
    ] {
        assert!(
            media.contains(needle),
            "alert dialog media snippet should keep the media docs example stable; missing `{needle}`",
        );
    }

    for needle in [
        "\"ui-gallery-alert-dialog-small-media-content\"",
        "\"ui-gallery-alert-dialog-small-media-title\"",
        "\"ui-gallery-alert-dialog-small-media-description\"",
        "\"Don't allow\"",
    ] {
        assert!(
            small_with_media.contains(needle),
            "alert dialog small-with-media snippet should keep the narrow-layout anchors stable; missing `{needle}`",
        );
    }

    for needle in [
        "shadcn::AlertDialogDescription::new_selectable_with(",
        "\"Delete Chat\"",
        "\"ui-gallery-alert-dialog-destructive-description-link\"",
        "shadcn::AlertDialogAction::from_scope(\"Delete\")",
    ] {
        assert!(
            destructive.contains(needle),
            "alert dialog destructive snippet should keep the selectable-description docs example stable; missing `{needle}`",
        );
    }

    for needle in [
        "with_direction_provider(cx, LayoutDirection::Rtl, move |cx| {",
        "\"إظهار الحوار\"",
        "\"ui-gallery-alert-dialog-rtl-small-content\"",
        "\"ui-gallery-alert-dialog-rtl-small-title\"",
        "\"ui-gallery-alert-dialog-rtl-small-description\"",
    ] {
        assert!(
            rtl.contains(needle),
            "alert dialog rtl snippet should keep the explicit RTL provider lane and stable selectors; missing `{needle}`",
        );
    }

    let combined = [
        usage,
        basic,
        small,
        media,
        small_with_media,
        destructive,
        rtl,
    ]
    .join("\n");
    assert!(
        combined.contains(".children(["),
        "alert dialog docs-path snippets should keep the default root lane on `children([...])`",
    );
    assert!(
        !combined.contains("compose()"),
        "alert dialog docs-path snippets should keep `compose()` out of the default docs lane",
    );
}

#[test]
fn alert_dialog_follow_up_snippets_stay_explicit_and_copyable() {
    let parts = include_str!("../src/ui/snippets/alert_dialog/parts.rs");
    let detached = include_str!("../src/ui/snippets/alert_dialog/detached_trigger.rs");
    let rich = include_str!("../src/ui/snippets/alert_dialog/rich_content.rs");

    for needle in [
        ".compose()",
        ".portal(shadcn::AlertDialogPortal::new())",
        ".overlay(shadcn::AlertDialogOverlay::new())",
        "\"ui-gallery-alert-dialog-parts-content\"",
    ] {
        assert!(
            parts.contains(needle),
            "alert dialog parts follow-up should keep the explicit adapter surface stable; missing `{needle}`",
        );
    }

    for needle in [
        "shadcn::AlertDialogHandle::new_controllable(cx, None, false);",
        ".handle(handle.clone())",
        "shadcn::AlertDialog::from_handle(handle)",
        "\"ui-gallery-alert-dialog-detached-trigger-content\"",
    ] {
        assert!(
            detached.contains(needle),
            "alert dialog detached-trigger follow-up should keep the handle-based advanced seam stable; missing `{needle}`",
        );
    }

    for needle in [
        "AlertDialogTitle::new_children([cx",
        "AlertDialogDescription::new_children([description_body])",
        ".children([cancel_visual])",
        ".children([action_visual])",
        "\"ui-gallery-alert-dialog-rich-content\"",
    ] {
        assert!(
            rich.contains(needle),
            "alert dialog rich-content follow-up should keep the composable-content seam stable; missing `{needle}`",
        );
    }

    let combined = [parts, detached, rich].join("\n");
    assert!(
        combined.contains("compose()"),
        "alert dialog follow-up snippets should keep `compose()` as the explicit advanced lane",
    );
}

#[test]
fn alert_dialog_docs_diag_scripts_cover_docs_path_and_existing_regression_gates() {
    let docs_script = include_str!(
        "../../../tools/diag-scripts/ui-gallery/overlay/ui-gallery-alert-dialog-docs-smoke.json"
    );
    let docs_examples = include_str!(
        "../../../tools/diag-scripts/ui-gallery/overlay/ui-gallery-alert-dialog-docs-example-open-screenshots.json"
    );
    let narrow_examples = include_str!(
        "../../../tools/diag-scripts/ui-gallery/overlay/ui-gallery-alert-dialog-narrow-docs-example-open-screenshots.json"
    );
    let docs_stub =
        include_str!("../../../tools/diag-scripts/ui-gallery-alert-dialog-docs-smoke.json");
    let narrow_stub = include_str!(
        "../../../tools/diag-scripts/ui-gallery-alert-dialog-narrow-docs-example-open-screenshots.json"
    );
    let docs_suite =
        include_str!("../../../tools/diag-scripts/suites/ui-gallery-shadcn-conformance/suite.json");
    let detached_focus = include_str!(
        "../../../tools/diag-scripts/ui-gallery/overlay/ui-gallery-alert-dialog-detached-trigger-focus-restore.json"
    );
    let least_destructive = include_str!(
        "../../../tools/diag-scripts/ui-gallery/overlay/ui-gallery-alert-dialog-least-destructive-initial-focus.json"
    );
    let destructive_link = include_str!(
        "../../../tools/diag-scripts/ui-gallery/overlay/ui-gallery-alert-dialog-destructive-inline-link-activate.json"
    );

    for needle in [
        "\"ui-gallery-page-alert-dialog\"",
        "\"ui-gallery-section-usage-title\"",
        "\"ui-gallery-section-notes-title\"",
        "\"ui-gallery-alert-dialog-demo-content\"",
        "\"ui-gallery-alert-dialog-api-reference-content\"",
        "\"ui-gallery-alert-dialog-extras-content\"",
        "\"ui-gallery-alert-dialog-docs-smoke\"",
    ] {
        assert!(
            docs_script.contains(needle),
            "alert dialog docs smoke script should cover the docs path anchors; missing `{needle}`",
        );
    }

    for needle in [
        "\"ui-gallery-alert-dialog-small-content\"",
        "\"ui-gallery-alert-dialog-media-content\"",
        "\"ui-gallery-alert-dialog-small-media-content\"",
        "\"ui-gallery-alert-dialog-destructive-content\"",
    ] {
        assert!(
            docs_examples.contains(needle),
            "alert dialog docs example screenshots script should cover the major docs examples; missing `{needle}`",
        );
    }

    for needle in [
        "\"ui-gallery-alert-dialog-small-media-content\"",
        "\"ui-gallery-alert-dialog-small-media-title\"",
        "\"ui-gallery-alert-dialog-small-media-description\"",
        "\"ui-gallery-alert-dialog-rtl-small-content\"",
        "\"ui-gallery-alert-dialog-rtl-small-title\"",
        "\"ui-gallery-alert-dialog-rtl-small-description\"",
    ] {
        assert!(
            narrow_examples.contains(needle),
            "alert dialog narrow docs examples script should cover the narrow-risk title/description anchors; missing `{needle}`",
        );
    }

    for needle in [
        "\"ui-gallery-alert-dialog-detached-trigger-inline\"",
        "\"ui-gallery-alert-dialog-detached-trigger-toolbar\"",
        "\"ui-gallery-alert-dialog-detached-trigger-content\"",
        "\"ui-gallery-alert-dialog-detached-trigger-focus-restore\"",
    ] {
        assert!(
            detached_focus.contains(needle),
            "alert dialog detached-trigger gate should keep the focus-restore selectors stable; missing `{needle}`",
        );
    }

    for needle in [
        "\"ui-gallery-alert-dialog-trigger\"",
        "\"ui-gallery-alert-dialog-content\"",
        "\"ui-gallery-alert-dialog-cancel\"",
        "\"ui-gallery-alert-dialog-least-destructive-initial-focus\"",
    ] {
        assert!(
            least_destructive.contains(needle),
            "alert dialog least-destructive focus gate should keep the overlay selectors stable; missing `{needle}`",
        );
    }

    for needle in [
        "\"ui-gallery-alert-dialog-destructive-trigger\"",
        "\"ui-gallery-alert-dialog-destructive-description-link\"",
        "\"ui-gallery-alert-dialog-destructive-inline-link-activate\"",
    ] {
        assert!(
            destructive_link.contains(needle),
            "alert dialog destructive inline-link gate should keep the docs example selectors stable; missing `{needle}`",
        );
    }

    assert!(
        docs_stub.contains(
            "\"to\": \"tools/diag-scripts/ui-gallery/overlay/ui-gallery-alert-dialog-docs-smoke.json\""
        ),
        "alert dialog docs smoke top-level redirect stub should point at the canonical overlay docs script",
    );
    assert!(
        narrow_stub.contains(
            "\"to\": \"tools/diag-scripts/ui-gallery/overlay/ui-gallery-alert-dialog-narrow-docs-example-open-screenshots.json\""
        ),
        "alert dialog narrow docs screenshots redirect stub should point at the canonical overlay script",
    );
    assert!(
        docs_suite.contains("\"tools/diag-scripts/ui-gallery-alert-dialog-docs-smoke.json\""),
        "shadcn conformance suite should include the alert dialog docs smoke gate",
    );
}

#[test]
fn alert_dialog_docs_demo_diag_script_waits_for_stable_overlay_bounds() {
    let script = include_str!(
        "../../../tools/diag-scripts/ui-gallery/overlay/ui-gallery-alert-dialog-docs-demo-open-screenshot.json"
    );

    for needle in [
        "\"ui-gallery-alert-dialog-demo-content\"",
        "\"type\": \"wait_bounds_stable\"",
        "\"stable_frames\": 6",
        "\"max_move_px\": 1.0",
        "\"ui-gallery-alert-dialog-docs-demo-open-desktop\"",
    ] {
        assert!(
            script.contains(needle),
            "alert dialog docs demo diag script should wait for stable overlay bounds before screenshots; missing `{needle}`",
        );
    }
}

#[test]
fn alert_dialog_docs_examples_diag_script_waits_for_stable_overlay_bounds() {
    let script = include_str!(
        "../../../tools/diag-scripts/ui-gallery/overlay/ui-gallery-alert-dialog-docs-example-open-screenshots.json"
    );

    for needle in [
        "\"ui-gallery-alert-dialog-small-content\"",
        "\"ui-gallery-alert-dialog-media-content\"",
        "\"ui-gallery-alert-dialog-small-media-content\"",
        "\"ui-gallery-alert-dialog-destructive-content\"",
        "\"type\": \"wait_bounds_stable\"",
        "\"stable_frames\": 6",
        "\"max_move_px\": 1.0",
        "\"ui-gallery-alert-dialog-small-open-desktop\"",
        "\"ui-gallery-alert-dialog-media-open-desktop\"",
        "\"ui-gallery-alert-dialog-small-media-open-desktop\"",
        "\"ui-gallery-alert-dialog-destructive-open-desktop\"",
    ] {
        assert!(
            script.contains(needle),
            "alert dialog docs examples diag script should wait for stable overlay bounds before screenshots; missing `{needle}`",
        );
    }
}

#[test]
fn alert_dialog_narrow_docs_examples_diag_script_waits_for_stable_overlay_bounds() {
    let script = include_str!(
        "../../../tools/diag-scripts/ui-gallery/overlay/ui-gallery-alert-dialog-narrow-docs-example-open-screenshots.json"
    );

    for needle in [
        "\"ui-gallery-alert-dialog-small-media-content\"",
        "\"ui-gallery-alert-dialog-small-media-title\"",
        "\"ui-gallery-alert-dialog-small-media-description\"",
        "\"ui-gallery-alert-dialog-rtl-small-content\"",
        "\"ui-gallery-alert-dialog-rtl-small-title\"",
        "\"ui-gallery-alert-dialog-rtl-small-description\"",
        "\"type\": \"wait_bounds_stable\"",
        "\"stable_frames\": 6",
        "\"max_move_px\": 1.0",
        "\"bounds_within_window\"",
        "\"ui-gallery-alert-dialog-small-media-open-narrow\"",
        "\"ui-gallery-alert-dialog-rtl-small-open-narrow\"",
    ] {
        assert!(
            script.contains(needle),
            "alert dialog narrow docs examples diag script should keep the narrow-window overlay evidence explicit; missing `{needle}`",
        );
    }
}
