fn normalize_ws(source: &str) -> String {
    source.split_whitespace().collect()
}

#[test]
fn separator_page_documents_source_axes_and_children_api_decision() {
    let source = include_str!("../src/ui/pages/separator.rs");

    for needle in [
        "Reference stack: shadcn base Separator docs, the base/radix registry recipe variants, Radix Primitives Separator, and Base UI Separator.",
        "`fret_ui_kit::primitives::separator::Separator` owns the mechanism layer",
        "`Separator::new()`, `Separator::orientation(...)`, and `Separator::decorative(...)` cover the public surface Fret needs",
        "Fret keeps the Radix-aligned `.decorative(...)` knob on the shadcn lane",
        "No generic composable children / `compose()` / `asChild` surface is warranted here because separator is a leaf primitive.",
        "Preview mirrors the current shadcn Base Separator docs path first: Demo, Usage, Vertical, Menu, List, RTL, and API Reference.",
    ] {
        assert!(
            source.contains(needle),
            "separator page should document the source axes and children-api decision; missing `{needle}`",
        );
    }

    let normalized = normalize_ws(source);
    let ordered_sections = normalize_ws(
        r#"
        vec![demo, usage, vertical, menu, list, rtl, api_reference]
        "#,
    );
    assert!(
        normalized.contains(&ordered_sections),
        "separator page should keep the docs-path sections in the upstream order",
    );
}

#[test]
fn separator_snippets_stay_docs_aligned_and_copyable() {
    let demo = include_str!("../src/ui/snippets/separator/demo.rs");
    let usage = include_str!("../src/ui/snippets/separator/usage.rs");
    let vertical = include_str!("../src/ui/snippets/separator/vertical.rs");
    let menu = include_str!("../src/ui/snippets/separator/menu.rs");
    let list = include_str!("../src/ui/snippets/separator/list.rs");
    let rtl = include_str!("../src/ui/snippets/separator/rtl.rs");

    for needle in [
        "use fret::{UiChild, AppComponentCx};",
        "use fret_ui_shadcn::{facade as shadcn, prelude::*};",
        "\"shadcn/ui\"",
        "\"The Foundation for your Design System\"",
        "\"A set of beautifully designed components that you can customize, extend, and build on.\"",
        ".max_w(Px(384.0))",
        ".test_id(\"ui-gallery-separator-demo\")",
    ] {
        assert!(
            demo.contains(needle),
            "separator demo snippet should match the upstream top preview and remain copyable; missing `{needle}`",
        );
    }
    assert!(
        !demo.contains("SeparatorOrientation::Vertical"),
        "separator demo snippet should stay on the horizontal top preview; vertical belongs in its own section",
    );

    for needle in [
        "\"Blog\"",
        "\"Docs\"",
        "\"Source\"",
        ".orientation(shadcn::SeparatorOrientation::Vertical)",
        ".h_px(Px(20.0))",
        ".test_id(\"ui-gallery-separator-vertical\")",
    ] {
        assert!(
            vertical.contains(needle),
            "separator vertical snippet should keep the docs-path nav row; missing `{needle}`",
        );
    }

    for needle in [
        "use fret_ui::Invalidation;",
        "use fret_ui_kit::declarative::viewport_queries;",
        "let is_md = viewport_queries::viewport_width_at_least(",
        "viewport_queries::tailwind::MD",
        "\"Settings\"",
        "\"Account\"",
        "\"Help\"",
        ".orientation(shadcn::SeparatorOrientation::Vertical)",
        ".test_id(\"ui-gallery-separator-menu\")",
    ] {
        assert!(
            menu.contains(needle),
            "separator menu snippet should keep the responsive docs composition; missing `{needle}`",
        );
    }
    assert!(
        !menu.contains(".wrap()"),
        "separator menu snippet should follow the upstream responsive hide/show split instead of wrapping the whole row",
    );
    assert!(
        !menu.contains(".h_px(Px(32.0))"),
        "separator menu snippet should rely on vertical self-stretch instead of hard-coded separator height",
    );

    for needle in [
        "shadcn::Separator::new()",
        ".test_id(\"ui-gallery-separator-usage\")",
    ] {
        assert!(
            usage.contains(needle),
            "separator usage snippet should remain the minimal copyable example; missing `{needle}`",
        );
    }

    for needle in [
        "\"Item 1\"",
        "\"Item 2\"",
        "\"Item 3\"",
        ".test_id(\"ui-gallery-separator-list\")",
    ] {
        assert!(
            list.contains(needle),
            "separator list snippet should keep the three-row docs example; missing `{needle}`",
        );
    }

    for needle in [
        "with_direction_provider(cx, LayoutDirection::Rtl, |cx| {",
        "\"الأساس لنظام التصميم الخاص بك\"",
        "\"ui-gallery-separator-rtl\"",
    ] {
        assert!(
            rtl.contains(needle),
            "separator RTL snippet should keep the translated docs example; missing `{needle}`",
        );
    }
}
