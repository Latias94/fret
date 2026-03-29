fn normalize_ws(source: &str) -> String {
    source.split_whitespace().collect()
}

#[test]
fn popover_page_documents_docs_path_follow_ups_and_children_api_decision() {
    let source = include_str!("../src/ui/pages/popover.rs");

    for needle in [
        "`Popover::trigger_element(...)` and `Popover::anchor_element(...)` cover the detached-trigger, anchor-aware follow-up without widening the default shadcn docs lane.",
        "`Popover::open_on_hover(...)`, `hover_open_delay_frames(...)`, and `hover_close_delay_frames(...)` cover the Base UI hover-open follow-up while staying outside the default docs path.",
        "A generic heterogeneous `children([...])` root API is not currently warranted here: unlike Dialog/Drawer, Popover root only needs trigger/content, while managed-open and anchor-aware cases already stay explicit on `from_open(...).into_element_with(...)` / `into_element_with_anchor(...)`.",
        "Preview mirrors the shadcn/base Popover docs path after `Installation`: `Demo`, `Usage`, `Basic`, `Align`, `With Form`, `RTL`, and `API Reference`. `Detached Trigger (Fret)`, `Open on Hover (Base UI/Fret)`, and `Inline Children (Fret)` stay as explicit follow-ups.",
        "`Detached Trigger (Fret)` documents the current advanced seam via `trigger_element(...)` / `anchor_element(...)`; Base UI's handle-driven multi-trigger/payload surface would be a public-surface follow-up rather than a mechanism fix.",
        "let detached_trigger = DocSection::build(cx, \"Detached Trigger (Fret)\", detached_trigger)",
        "let open_on_hover = DocSection::build(cx, \"Open on Hover (Base UI/Fret)\", open_on_hover)",
    ] {
        assert!(
            source.contains(needle),
            "popover page should document the docs-path order and advanced surface decision; missing `{needle}`"
        );
    }

    let normalized = normalize_ws(source);
    let ordered_sections = normalize_ws(
        r#"
        vec![
            demo,
            usage,
            basic,
            align,
            with_form,
            rtl,
            api_reference,
            detached_trigger,
            open_on_hover,
            inline_children,
            notes,
        ]
        "#,
    );
    assert!(
        normalized.contains(&ordered_sections),
        "popover page should keep advanced follow-ups after the docs-path sections and `API Reference`"
    );
}

#[test]
fn popover_rtl_snippet_covers_logical_sides_and_real_rtl_copy() {
    let source = include_str!("../src/ui/snippets/popover/rtl.rs");

    for needle in [
        "shadcn::PopoverSide::InlineStart",
        "shadcn::PopoverSide::InlineEnd",
        "\"بداية السطر\"",
        "\"نهاية السطر\"",
        "\"ui-gallery-popover-rtl-inline-start-trigger\"",
        "\"ui-gallery-popover-rtl-inline-end-trigger\"",
    ] {
        assert!(
            source.contains(needle),
            "popover RTL snippet should keep logical sides and readable RTL labels; missing `{needle}`"
        );
    }
}

#[test]
fn popover_follow_up_snippets_stay_copyable_and_explicit() {
    let detached = include_str!("../src/ui/snippets/popover/detached_trigger.rs");
    let open_on_hover = include_str!("../src/ui/snippets/popover/open_on_hover.rs");

    for needle in [
        "use fret::{UiChild, UiCx};",
        ".auto_toggle(false)",
        ".toggle_model(open.clone())",
        ".trigger_element(detached_id)",
        ".anchor_element(detached_id)",
        "\"ui-gallery-popover-detached-trigger-button\"",
        "\"ui-gallery-popover-detached-trigger-panel\"",
    ] {
        assert!(
            detached.contains(needle),
            "detached-trigger snippet should remain a complete copyable example; missing `{needle}`"
        );
    }

    for needle in [
        "use fret::{UiChild, UiCx};",
        ".open_on_hover(true)",
        ".hover_open_delay_frames(12)",
        ".hover_close_delay_frames(6)",
        "\"ui-gallery-popover-open-on-hover-trigger\"",
        "\"ui-gallery-popover-open-on-hover-panel\"",
    ] {
        assert!(
            open_on_hover.contains(needle),
            "open-on-hover snippet should remain a complete copyable example; missing `{needle}`"
        );
    }
}
