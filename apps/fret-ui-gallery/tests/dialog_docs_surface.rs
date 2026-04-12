fn normalize_ws(source: &str) -> String {
    source.split_whitespace().collect()
}

#[test]
fn dialog_page_documents_source_axes_and_children_api_decision() {
    let source = include_str!("../src/ui/pages/dialog.rs");

    for needle in [
        "Reference stack: shadcn/base Dialog docs and examples, the default registry recipe, Radix Primitives dialog semantics, and Base UI dialog ownership.",
        "`Dialog::children([...])` is the default copyable root path for part-based composition, and `DialogPart` is available on the curated `shadcn` facade so the default import lane stays copyable.",
        "`DialogPart::content_with(...)` plus `DialogContent::with_children(...)`, `DialogHeader::with_children(...)`, and `DialogFooter::with_children(...)` form the default copyable content lane when child parts need the current dialog scope.",
        "`Dialog::children([...])` is already the warranted composable root API here because the component owns Trigger/Portal/Overlay/Content parts and the scope-sensitive `DialogClose::from_scope()` buttons must stay inside `DialogContent`; no broader untyped JSX-style root children API is warranted beyond the typed `DialogPart` lane.",
        "Preview mirrors the shadcn/base Dialog docs path after `Installation`: `Demo`, `Usage`, `Custom Close Button`, `No Close Button`, `Sticky Footer`, `Scrollable Content`, `RTL`, and `API Reference`.",
        "`Usage` is the default copyable path; `Parts` stays as the advanced adapter section for explicit `DialogTrigger` / `DialogPortal` / `DialogOverlay` ownership.",
        "Radix Primitives and Base UI agree on the relevant semantics axis here: modal dialog, outside press dismisses by default, dismissal can be intercepted, and focus restores to the trigger on close. Those outcomes are already handled in `fret-ui-kit` / `fret-ui-shadcn`, so this page is now mainly a docs/public-surface alignment task rather than a `fret-ui` mechanism bug.",
        "`Detached Trigger` is now the focused Base UI-style follow-up for opener/content split ownership; it stays outside the default shadcn/base docs path and does not change the typed `DialogPart` default lane.",
        ".test_id_prefix(\"ui-gallery-dialog-demo-docsec\")",
        ".test_id_prefix(\"ui-gallery-dialog-custom-close-docsec\")",
        ".test_id_prefix(\"ui-gallery-dialog-no-close-docsec\")",
        ".test_id_prefix(\"ui-gallery-dialog-sticky-footer-docsec\")",
        ".test_id_prefix(\"ui-gallery-dialog-scrollable-docsec\")",
        ".test_id_prefix(\"ui-gallery-dialog-rtl-docsec\")",
        ".test_id_prefix(\"ui-gallery-dialog-parts-docsec\")",
        ".test_id_prefix(\"ui-gallery-dialog-detached-trigger-docsec\")",
    ] {
        assert!(
            source.contains(needle),
            "dialog page should document source axes and the children-api decision; missing `{needle}`",
        );
    }

    let normalized = normalize_ws(source);
    let ordered_sections = normalize_ws(
        r#"
        vec![
            demo,
            usage,
            custom_close,
            no_close,
            sticky_footer,
            scrollable_content,
            rtl,
            api_reference,
            extras,
            parts,
            detached_trigger,
            notes,
        ]
        "#,
    );
    assert!(
        normalized.contains(&ordered_sections),
        "dialog page should keep the docs-path sections before the explicit Fret follow-ups",
    );
}

#[test]
fn dialog_docs_path_snippets_stay_copyable_and_docs_aligned() {
    let usage = include_str!("../src/ui/snippets/dialog/usage.rs");
    let demo = include_str!("../src/ui/snippets/dialog/demo.rs");
    let custom_close = include_str!("../src/ui/snippets/dialog/custom_close_button.rs");
    let no_close = include_str!("../src/ui/snippets/dialog/no_close_button.rs");
    let sticky = include_str!("../src/ui/snippets/dialog/sticky_footer.rs");
    let scrollable = include_str!("../src/ui/snippets/dialog/scrollable_content.rs");
    let rtl = include_str!("../src/ui/snippets/dialog/rtl.rs");

    for needle in [
        "use fret::{UiChild, UiCx};",
        "use fret_ui_shadcn::facade as shadcn;",
        "shadcn::Dialog::new_controllable(cx, None, false)",
        ".children([",
        "shadcn::DialogPart::trigger(shadcn::DialogTrigger::build(",
        "shadcn::DialogPart::content_with(|cx| {",
        "shadcn::DialogContent::new([]).with_children(cx, |cx| {",
    ] {
        assert!(
            usage.contains(needle),
            "dialog usage snippet should remain a complete copyable docs-path example; missing `{needle}`",
        );
    }

    for needle in [
        "LayoutRefinement::default().w_full().min_w_0()",
        ".layout(LayoutRefinement::default().w_full().max_w(Px(220.0)).min_w_0())",
        "ui::v_flex(move |cx| {",
        "shadcn::DialogTitle::new(\"Edit profile\")",
        "shadcn::DialogClose::from_scope()",
        "shadcn::Button::new(\"Save changes\").into_element(cx)",
        "LayoutRefinement::default().max_w(Px(425.0))",
        "\"ui-gallery-dialog-demo-content\"",
    ] {
        assert!(
            demo.contains(needle),
            "dialog demo snippet should stay aligned with the upstream example shape; missing `{needle}`",
        );
    }
    assert!(
        !demo.contains("DialogClose::from_scope()\n                                    .build(cx, shadcn::Button::new(\"Save changes\"))"),
        "dialog demo snippet should keep the primary save action as a normal action button",
    );

    for needle in [
        "https://ui.shadcn.com/docs/installation",
        "LayoutRefinement::default().max_w(Px(448.0))",
        "\"ui-gallery-dialog-custom-close-content\"",
    ] {
        assert!(
            custom_close.contains(needle),
            "dialog custom-close snippet should stay aligned with the upstream docs example; missing `{needle}`",
        );
    }

    assert!(
        no_close.contains("This dialog doesn't have a close button in the top-right corner."),
        "dialog no-close snippet should keep the upstream docs copy",
    );

    assert!(
        sticky.contains(
            "This dialog has a sticky footer that stays visible while the content scrolls."
        ),
        "dialog sticky-footer snippet should keep the upstream docs copy",
    );
    assert!(
        sticky.contains("\"ui-gallery-dialog-sticky-footer-description\""),
        "dialog sticky-footer snippet should expose a stable description test id for wrap diagnostics",
    );
    assert!(
        !sticky.contains("Save changes"),
        "dialog sticky-footer snippet should not reintroduce the extra action that drifts from upstream",
    );

    assert!(
        scrollable.contains("This is a dialog with scrollable content."),
        "dialog scrollable-content snippet should keep the upstream docs copy",
    );

    for needle in [
        "\"فتح الحوار\"",
        "\"تعديل الملف الشخصي\"",
        "\"قم بإجراء تغييرات على ملفك الشخصي هنا. انقر فوق حفظ عند الانتهاء.\"",
        "\"الاسم\"",
        "\"اسم المستخدم\"",
        "\"إلغاء\"",
        "\"حفظ التغييرات\"",
        "LayoutRefinement::default().max_w(Px(384.0))",
    ] {
        assert!(
            rtl.contains(needle),
            "dialog RTL snippet should keep readable RTL copy and upstream-shaped sizing; missing `{needle}`",
        );
    }

    let combined = [usage, demo, custom_close, no_close, sticky, scrollable, rtl].join("\n");
    assert!(
        combined.contains(".children(["),
        "dialog docs-path snippets should keep the default root lane on `children([...])`",
    );
    assert!(
        !combined.contains("compose()"),
        "dialog docs-path snippets should keep `compose()` out of the default docs lane",
    );
}

#[test]
fn dialog_follow_up_parts_snippet_stays_explicit() {
    let parts = include_str!("../src/ui/snippets/dialog/parts.rs");
    let detached = include_str!("../src/ui/snippets/dialog/detached_trigger.rs");

    for needle in [
        ".compose()",
        ".portal(shadcn::DialogPortal::new())",
        ".overlay(shadcn::DialogOverlay::new())",
        "\"ui-gallery-dialog-parts-content\"",
    ] {
        assert!(
            parts.contains(needle),
            "dialog parts follow-up should keep the explicit adapter surface stable; missing `{needle}`",
        );
    }

    for needle in [
        "shadcn::DialogHandle::new_controllable(cx, None, false);",
        ".handle(handle.clone())",
        "shadcn::Dialog::from_handle(handle)",
        "\"ui-gallery-dialog-detached-trigger-content\"",
    ] {
        assert!(
            detached.contains(needle),
            "dialog detached-trigger follow-up should keep the handle-based advanced seam stable; missing `{needle}`",
        );
    }
}

#[test]
fn dialog_docs_diag_scripts_cover_docs_path_and_existing_regression_gates() {
    let docs_smoke = include_str!(
        "../../../tools/diag-scripts/ui-gallery/overlay/ui-gallery-dialog-docs-order-smoke.json"
    );
    let docs_smoke_stub =
        include_str!("../../../tools/diag-scripts/ui-gallery-dialog-docs-order-smoke.json");
    let demo_screenshot = include_str!(
        "../../../tools/diag-scripts/ui-gallery/overlay/ui-gallery-dialog-docs-demo-open-screenshot.json"
    );
    let narrow_demo = include_str!(
        "../../../tools/diag-scripts/ui-gallery/overlay/ui-gallery-dialog-demo-narrow-sweep.json"
    );
    let default_close = include_str!(
        "../../../tools/diag-scripts/ui-gallery/dialog/ui-gallery-dialog-default-close-click.json"
    );
    let wrap_smoke = include_str!(
        "../../../tools/diag-scripts/ui-gallery/dialog/ui-gallery-dialog-wrap-smoke.json"
    );
    let detached_focus = include_str!(
        "../../../tools/diag-scripts/ui-gallery/overlay/ui-gallery-dialog-detached-trigger-focus-restore.json"
    );
    let suite =
        include_str!("../../../tools/diag-scripts/suites/ui-gallery-shadcn-conformance/suite.json");

    for needle in [
        "\"ui-gallery-dialog-usage-content\"",
        "\"ui-gallery-dialog-custom-close-content\"",
        "\"ui-gallery-dialog-no-close-content\"",
        "\"ui-gallery-dialog-rtl-content\"",
        "\"ui-gallery-dialog-docs-order-smoke\"",
    ] {
        assert!(
            docs_smoke.contains(needle),
            "dialog docs smoke script should cover the docs-path anchors; missing `{needle}`",
        );
    }

    assert!(
        docs_smoke_stub.contains(
            "\"to\": \"tools/diag-scripts/ui-gallery/overlay/ui-gallery-dialog-docs-order-smoke.json\""
        ),
        "dialog docs smoke top-level redirect stub should point at the canonical overlay docs script",
    );

    for needle in [
        "\"ui-gallery-dialog-demo-trigger\"",
        "\"ui-gallery-dialog-demo-content\"",
        "\"ui-gallery-dialog-docs-demo-open-desktop\"",
    ] {
        assert!(
            demo_screenshot.contains(needle),
            "dialog docs demo screenshot script should keep the upstream-shaped demo evidence path stable; missing `{needle}`",
        );
    }

    for needle in [
        "\"ui-gallery-dialog-demo-trigger\"",
        "\"ui-gallery-dialog-demo-content\"",
        "\"bounds_within_window\"",
        "\"ui-gallery-dialog-demo-open-narrow\"",
    ] {
        assert!(
            narrow_demo.contains(needle),
            "dialog narrow sweep should keep the narrow-window width-hygiene selectors stable; missing `{needle}`",
        );
    }

    for needle in [
        "\"ui-gallery-dialog-demo-trigger\"",
        "\"ui-gallery-dialog-demo-content\"",
        "\"name\": \"Close\"",
    ] {
        assert!(
            default_close.contains(needle),
            "dialog default-close gate should keep the default close affordance covered; missing `{needle}`",
        );
    }

    for needle in [
        "\"ui-gallery-dialog-scrollable-content\"",
        "\"ui-gallery-dialog-sticky-footer-content\"",
        "\"ui-gallery-dialog-sticky-footer-description\"",
        "\"ui-gallery-dialog-scrollable-row-01\"",
        "\"ui-gallery-dialog-sticky-footer-row-01\"",
    ] {
        assert!(
            wrap_smoke.contains(needle),
            "dialog wrap smoke should keep the scrollable/sticky docs examples covered; missing `{needle}`",
        );
    }

    for needle in [
        "\"ui-gallery-dialog-detached-trigger-inline\"",
        "\"ui-gallery-dialog-detached-trigger-toolbar\"",
        "\"ui-gallery-dialog-detached-trigger-content\"",
        "\"ui-gallery-dialog-detached-trigger-focus-restore\"",
    ] {
        assert!(
            detached_focus.contains(needle),
            "dialog detached-trigger focus-restore script should keep the handle follow-up covered; missing `{needle}`",
        );
    }

    assert!(
        suite.contains("tools/diag-scripts/ui-gallery-dialog-docs-order-smoke.json"),
        "shadcn conformance suite should include the dialog docs smoke gate",
    );
    assert!(
        suite.contains("tools/diag-scripts/ui-gallery-dialog-detached-trigger-focus-restore.json"),
        "shadcn conformance suite should include the dialog detached-trigger focus-restore gate",
    );
}
