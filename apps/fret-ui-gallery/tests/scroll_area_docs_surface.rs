fn normalize_ws(source: &str) -> String {
    source.split_whitespace().collect()
}

fn canonicalize_rust_fragment(fragment: &str) -> String {
    let mut canonical = fragment.split_whitespace().collect::<String>();
    loop {
        let next = canonical.replace(",)", ")");
        if next == canonical {
            return canonical;
        }
        canonical = next;
    }
}

#[test]
fn scroll_area_page_documents_wrapper_mapping_and_children_api_decision() {
    let source = include_str!("../src/ui/pages/scroll_area.rs");

    for needle in [
        "Reference stack: shadcn Scroll Area docs, Base UI docs, and the default registry chrome.",
        "`ScrollArea::new([...])` is the default copyable wrapper lane for the docs surface.",
        "Because the Fret recipe stays layout-only, the upstream root chrome (`rounded-md border` and fixed size) maps to a caller-owned wrapper container around the scroll surface.",
        "`ScrollAreaRoot::new(ScrollAreaViewport::new([...])).scrollbar(ScrollBar::new().orientation(...))` already covers the shadcn/Radix mixed `ScrollArea` + `ScrollBar` examples without widening this family into an untyped arbitrary-children API.",
        "Base UI's extra `Content` / `Thumb` parts are useful headless references",
        "No generic `children([...])` / `compose()` root API is warranted here because `ScrollArea::new([...])` already covers the upstream children-owned wrapper lane while `ScrollAreaRoot` / `ScrollAreaViewport` / `ScrollBar` keep the explicit parts path typed.",
        "Preview now mirrors the upstream shadcn/Base UI docs path first: `Demo`, `Usage`, `Horizontal`, `RTL`, and `API Reference`.",
    ] {
        assert!(
            source.contains(needle),
            "scroll area page should document wrapper ownership and the children-api decision; missing `{needle}`",
        );
    }

    let normalized = normalize_ws(source);
    let ordered_sections = normalize_ws(
        r#"
        vec![
            demo,
            usage,
            horizontal,
            rtl,
            api_reference,
            compact_helper,
            nested_scroll_routing,
            drag_baseline,
            expand_at_bottom,
            notes,
        ]
        "#,
    );
    assert!(
        normalized.contains(&ordered_sections),
        "scroll area page should keep the docs-path sections before the Fret-only follow-ups",
    );
}

#[test]
fn scroll_area_usage_snippet_keeps_wrapper_chrome_and_copyable_root_lane() {
    let source = include_str!("../src/ui/snippets/scroll_area/usage.rs");

    for needle in [
        "use fret::{UiChild, AppComponentCx};",
        "use fret_ui_shadcn::{facade as shadcn, prelude::*};",
        "let area = shadcn::ScrollArea::new([content])",
        ".refine_layout(LayoutRefinement::default().w_full().h_full())",
        "ChromeRefinement::default().border_1().rounded(Radius::Md)",
        "LayoutRefinement::default()",
        ".w_px(Px(350.0))",
        ".h_px(Px(200.0))",
        ".overflow_hidden()",
        "cx.container(props, move |_cx| [area])",
        ".test_id(\"ui-gallery-scroll-area-usage\")",
    ] {
        assert!(
            source.contains(needle),
            "scroll area usage snippet should stay copyable and teach the caller-owned wrapper mapping; missing `{needle}`",
        );
    }
}

#[test]
fn scroll_area_snippets_route_visible_text_through_shared_roles() {
    let cases = [
        (
            "demo",
            include_str!("../src/ui/snippets/scroll_area/demo.rs"),
            &[
                "use fret_ui_kit::declarative::text as decl_text;",
                "decl_text::text_section_chrome_label(cx, \"Tags\")",
                "decl_text::text_list_row_label(cx, tag.clone())",
            ][..],
            &[
                "ui::text(\"Tags\")",
                "ui::text(tag.clone())",
                ".text_size_px(Px(14.0))",
            ][..],
        ),
        (
            "usage",
            include_str!("../src/ui/snippets/scroll_area/usage.rs"),
            &[
                "use fret_ui_kit::declarative::text as decl_text;",
                "decl_text::text_paragraph(cx, story)",
            ][..],
            &[
                "use fret_core::TextWrap;",
                "ui::text(story)",
                ".wrap(TextWrap::Word)",
            ][..],
        ),
        (
            "compact_helper",
            include_str!("../src/ui/snippets/scroll_area/compact_helper.rs"),
            &[
                "use fret_ui_kit::declarative::text as decl_text;",
                "decl_text::text_paragraph(",
            ][..],
            &[
                "use fret_core::TextWrap;",
                "ui::text(",
                ".wrap(TextWrap::Word)",
            ][..],
        ),
        (
            "rtl",
            include_str!("../src/ui/snippets/scroll_area/rtl.rs"),
            &[
                "use fret_ui_kit::declarative::text as decl_text;",
                "decl_text::text_section_chrome_label(cx,",
                "decl_text::text_list_row_label(cx, (41 - idx).to_string())",
            ][..],
            &["ui::text((41 - idx).to_string())", ".text_sm()"][..],
        ),
        (
            "horizontal",
            include_str!("../src/ui/snippets/scroll_area/horizontal.rs"),
            &[
                "use fret_ui_kit::declarative::text as decl_text;",
                "decl_text::text_control_readout(cx, format!(\"Photo by {artist}\"))",
            ][..],
            &["shadcn::raw::typography::muted(format!(\"Photo by {artist}\"))"][..],
        ),
        (
            "nested_scroll_routing",
            include_str!("../src/ui/snippets/scroll_area/nested_scroll_routing.rs"),
            &[
                "use fret_ui_kit::declarative::text as decl_text;",
                "decl_text::text_control_readout(cx, format!(\"Item {i}\"))",
            ][..],
            &["shadcn::raw::typography::muted(format!(\"Item {i}\"))"][..],
        ),
    ];

    for (name, source, required, forbidden) in cases {
        let canonical = canonicalize_rust_fragment(source);
        for marker in required {
            let marker = canonicalize_rust_fragment(marker);
            assert!(
                canonical.contains(&marker),
                "{name} should route scroll-area visible text through shared text roles; missing `{marker}`"
            );
        }
        for marker in forbidden {
            let marker = canonicalize_rust_fragment(marker);
            assert!(
                !canonical.contains(&marker),
                "{name} reintroduced bare or locally styled scroll-area text: `{marker}`"
            );
        }
    }
}
