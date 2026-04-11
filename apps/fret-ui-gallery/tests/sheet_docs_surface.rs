fn normalize_ws(source: &str) -> String {
    source.split_whitespace().collect()
}

#[test]
fn sheet_page_documents_source_axes_and_children_api_decision() {
    let source = include_str!("../src/ui/pages/sheet.rs");

    for needle in [
        "Reference stack: shadcn Sheet docs and default registry recipe, with Radix dialog semantics and Base UI dialog ownership as the headless baseline.",
        "`SheetContent::new([]).with_children(cx, ...)` plus `SheetHeader::new([]).with_children(cx, ...)` / `SheetFooter::new([]).with_children(cx, ...)` is the default copyable content lane for upstream-like nested composition.",
        "Radix/Base UI semantics are already largely covered by the existing overlay, dismissal, focus-restore, and sizing tests in `ecosystem/fret-ui-shadcn/src/sheet.rs`; the remaining drift addressed here is recipe/public-surface parity rather than a `fret-ui` mechanism bug.",
        "A broader generic heterogeneous root children API is not warranted beyond `Sheet::children([...])`",
        "Preview mirrors the shadcn Sheet docs path after `Installation`: `Demo`, `Usage`, `Side`, `No Close Button`, `RTL`, and `API Reference`.",
        "DocSection::build(cx, \"API Reference\", api_reference)",
        "DocSection::build(cx, \"Parts\", parts)",
    ] {
        assert!(
            source.contains(needle),
            "sheet page should document source axes and the children-api decision; missing `{needle}`",
        );
    }

    let normalized = normalize_ws(source);
    let ordered_sections = normalize_ws(
        r#"
        vec![
            demo,
            usage,
            side,
            no_close_button,
            rtl,
            api_reference,
            parts,
            notes,
        ]
        "#,
    );
    assert!(
        normalized.contains(&ordered_sections),
        "sheet page should keep the docs-path sections before the Parts and Notes follow-ups",
    );
}

#[test]
fn sheet_snippets_keep_the_default_root_lane_and_docs_aligned_followups() {
    let usage = include_str!("../src/ui/snippets/sheet/usage.rs");
    let no_close = include_str!("../src/ui/snippets/sheet/no_close_button.rs");
    let rtl = include_str!("../src/ui/snippets/sheet/rtl.rs");
    let side = include_str!("../src/ui/snippets/sheet/side.rs");

    for needle in [
        "use fret::{UiChild, UiCx};",
        "shadcn::Sheet::new_controllable(cx, None, false)",
        ".children([",
        "shadcn::SheetPart::trigger(shadcn::SheetTrigger::build(",
        "shadcn::SheetPart::content_with(",
        "shadcn::SheetContent::new([]).with_children(",
        "shadcn::SheetHeader::new([]).with_children(",
    ] {
        assert!(
            usage.contains(needle),
            "sheet usage snippet should remain a complete copyable docs-path example; missing `{needle}`",
        );
    }

    assert!(
        !usage.contains("shadcn::SheetContent::build("),
        "sheet usage snippet should keep builder-first content assembly off the default docs lane",
    );

    for needle in [
        ".show_close_button(false)",
        "shadcn::SheetTitle::new(\"No Close Button\")",
        "Click outside to close.",
    ] {
        assert!(
            no_close.contains(needle),
            "no-close snippet should stay aligned with the upstream docs teaching surface; missing `{needle}`",
        );
    }
    assert!(
        !no_close.contains("SheetFooter::new"),
        "no-close snippet should not add a footer-only close workaround that the upstream docs page does not teach",
    );

    for needle in [
        "\"فتح\"",
        "\"تعديل الملف الشخصي\"",
        "\"قم بإجراء تغييرات على ملفك الشخصي هنا. انقر حفظ عند الانتهاء.\"",
        "\"حفظ التغييرات\"",
        "\"إغلاق\"",
        "\"الاسم\"",
        "\"اسم المستخدم\"",
    ] {
        assert!(
            rtl.contains(needle),
            "sheet RTL snippet should keep readable RTL copy instead of an English-only placeholder; missing `{needle}`",
        );
    }

    assert!(
        side.contains("shadcn::SheetClose::from_scope().build("),
        "sheet side snippet should keep the single footer action on the upstream-aligned close-as-child lane",
    );
    assert!(
        !side.contains("\"Cancel\""),
        "sheet side snippet should not reintroduce the extra Cancel action that drifts from the upstream docs example",
    );
}

#[test]
fn sheet_docs_diag_script_covers_docs_path_and_parts_followup() {
    let script = include_str!(
        "../../../tools/diag-scripts/ui-gallery/overlay/ui-gallery-sheet-docs-smoke.json"
    );

    for needle in [
        "\"ui-gallery-sheet-demo-content\"",
        "\"ui-gallery-sheet-side-content\"",
        "\"ui-gallery-sheet-no-close-content\"",
        "\"ui-gallery-sheet-rtl-content\"",
        "\"ui-gallery-sheet-api-reference-content\"",
        "\"ui-gallery-sheet-parts-content\"",
        "\"ui-gallery-sheet-docs-smoke\"",
    ] {
        assert!(
            script.contains(needle),
            "sheet docs diag script should cover the docs path and parts follow-up; missing `{needle}`",
        );
    }
}

#[test]
fn sheet_demo_snippet_uses_a_unique_overlay_panel_test_id() {
    let demo = include_str!("../src/ui/snippets/sheet/demo.rs");

    assert!(
        demo.contains("\"ui-gallery-sheet-demo-panel\""),
        "sheet demo snippet should expose a unique overlay panel test id for diag scripts",
    );
    assert!(
        !demo.contains("\"ui-gallery-sheet-demo-content\""),
        "sheet demo snippet should not reuse the DocSection content test id for the open sheet panel",
    );
}

#[test]
fn sheet_docs_demo_diag_script_waits_for_stable_overlay_bounds() {
    let script = include_str!(
        "../../../tools/diag-scripts/ui-gallery/overlay/ui-gallery-sheet-docs-demo-open-screenshot.json"
    );

    for needle in [
        "\"ui-gallery-sheet-demo-panel\"",
        "\"type\": \"wait_bounds_stable\"",
        "\"stable_frames\": 6",
        "\"max_move_px\": 1.0",
        "\"ui-gallery-sheet-docs-demo-open-desktop\"",
    ] {
        assert!(
            script.contains(needle),
            "sheet docs demo diag script should wait for stable overlay bounds before screenshots; missing `{needle}`",
        );
    }
}
