fn normalize_ws(source: &str) -> String {
    source.split_whitespace().collect()
}

#[test]
fn toggle_group_page_documents_source_axes_and_children_api_decision() {
    let source = include_str!("../src/ui/pages/toggle_group.rs");

    for needle in [
        "Reference stack for this page: shadcn Toggle Group docs, the default registry recipe, Radix Primitives Toggle Group, and Base UI Toggle Group.",
        "The upstream docs-path examples come from the default shadcn demo/outline/sm/lg/spacing set plus the vertical, font-weight-selector, and RTL examples.",
        "toggle-group-demo.tsx",
        "toggle-group-outline.tsx",
        "toggle-group-sm.tsx",
        "toggle-group-lg.tsx",
        "toggle-group-spacing.tsx",
        "toggle-group-vertical.tsx",
        "toggle-group-font-weight-selector.tsx",
        "toggle-group-rtl.tsx",
        "`fret_ui_kit::primitives::toggle_group` already covers the mechanism lane",
        "No extra root `children([...])` or generic `compose()` API is warranted on the default lane because the helper family already covers composable item assembly without widening the recipe contract.",
        "Preview now mirrors the upstream Toggle Group docs path first: `Demo`, `Usage`, `Outline`, `Size`, `Spacing`, `Vertical`, `Disabled`, `Custom`, `RTL`, and `API Reference`.",
        "Focused Fret follow-ups stay afterward: `Children (Fret)`, `Single (Fret)`, `Small (Fret)`, `Large (Fret)`, `Label Association (Fret)`, `Full Width Items (Fret)`, `Flex-1 Items (Fret)`, and `Notes`.",
    ] {
        assert!(
            source.contains(needle),
            "toggle_group page should document the source axes and children-api decision; missing `{needle}`",
        );
    }

    let normalized = normalize_ws(source);
    let ordered_sections = normalize_ws(
        r#"
        vec![
            demo,
            usage,
            outline,
            size,
            spacing,
            vertical,
            disabled,
            custom,
            rtl,
            api_reference,
            children,
            single,
            small,
            large,
            label,
            full_width_items,
            stretch,
            notes,
        ]
        "#,
    );
    assert!(
        normalized.contains(&ordered_sections),
        "toggle_group page should keep the docs-path sections before the Fret follow-ups",
    );
}

#[test]
fn toggle_group_snippets_stay_copyable_and_upstream_example_aligned() {
    let demo = include_str!("../src/ui/snippets/toggle_group/demo.rs");
    let usage = include_str!("../src/ui/snippets/toggle_group/usage.rs");
    let size = include_str!("../src/ui/snippets/toggle_group/size.rs");
    let spacing = include_str!("../src/ui/snippets/toggle_group/spacing.rs");
    let rtl = include_str!("../src/ui/snippets/toggle_group/rtl.rs");
    let children = include_str!("../src/ui/snippets/toggle_group/children.rs");
    let label = include_str!("../src/ui/snippets/toggle_group/label.rs");

    for needle in [
        "use fret::{UiChild, AppComponentCx};",
        "use fret_ui_shadcn::{facade as shadcn, prelude::*};",
        ".variant(shadcn::ToggleVariant::Outline)",
        "ToggleGroup::multiple_uncontrolled(std::iter::empty::<&'static str>())",
        "IconId::new_static(\"lucide.bold\")",
        "IconId::new_static(\"lucide.italic\")",
        "IconId::new_static(\"lucide.underline\")",
        ".test_id(\"ui-gallery-toggle-group-demo\")",
    ] {
        assert!(
            demo.contains(needle),
            "toggle_group demo snippet should stay aligned with the upstream top preview; missing `{needle}`",
        );
    }

    for needle in [
        "ToggleGroup::single_uncontrolled(Option::<&'static str>::None)",
        "ToggleGroupItem::new(\"a\", [cx.text(\"A\")])",
        "ToggleGroupItem::new(\"b\", [cx.text(\"B\")])",
        "ToggleGroupItem::new(\"c\", [cx.text(\"C\")])",
        ".test_id(\"ui-gallery-toggle-group-usage\")",
    ] {
        assert!(
            usage.contains(needle),
            "toggle_group usage snippet should remain the minimal copyable example; missing `{needle}`",
        );
    }

    for needle in [
        ".size(shadcn::ToggleSize::Sm)",
        ".size(shadcn::ToggleSize::Lg)",
        "ToggleGroup::single_uncontrolled(Option::<&'static str>::None)",
        "ToggleGroup::multiple_uncontrolled(std::iter::empty::<&'static str>())",
        ".test_id(\"ui-gallery-toggle-group-size\")",
    ] {
        assert!(
            size.contains(needle),
            "toggle_group size snippet should keep the upstream small/large icon-only lane; missing `{needle}`",
        );
    }

    for needle in [
        ".variant(shadcn::ToggleVariant::Outline)",
        ".size(shadcn::ToggleSize::Sm)",
        ".spacing(Space::N2)",
        "\"lucide.star\"",
        "\"lucide.heart\"",
        "\"lucide.bookmark\"",
        ".test_id(\"ui-gallery-toggle-group-spacing\")",
    ] {
        assert!(
            spacing.contains(needle),
            "toggle_group spacing snippet should keep the upstream icon-plus-label lane; missing `{needle}`",
        );
    }

    for needle in [
        "with_direction_provider(cx, LayoutDirection::Rtl, |cx| {",
        "\"قائمة\"",
        "\"شبكة\"",
        "\"بطاقات\"",
        ".test_id(\"ui-gallery-toggle-group-rtl\")",
    ] {
        assert!(
            rtl.contains(needle),
            "toggle_group RTL snippet should keep translated labels while staying copyable; missing `{needle}`",
        );
    }

    for needle in [
        "toggle_group_single_uncontrolled(cx, Some(\"list\"), |cx| {",
        "ToggleGroupItem::new(",
        ".test_id(\"ui-gallery-toggle-group-children\")",
    ] {
        assert!(
            children.contains(needle),
            "toggle_group children snippet should remain the builder-preserving composable lane; missing `{needle}`",
        );
    }

    for needle in [
        "ControlId::from(\"ui-gallery-toggle-group-label\")",
        ".test_id_prefix(\"ui-gallery-toggle-group-label\")",
        ".for_control(control_id.clone())",
        ".test_id(\"ui-gallery-toggle-group-label\")",
    ] {
        assert!(
            label.contains(needle),
            "toggle_group label snippet should keep the label/focus association lane; missing `{needle}`",
        );
    }
}

#[test]
fn toggle_group_docs_diag_script_covers_docs_path_and_follow_ups() {
    let script = include_str!(
        "../../../tools/diag-scripts/ui-gallery/toggle/ui-gallery-toggle-group-docs-smoke.json"
    );

    for needle in [
        "\"ui-gallery-toggle-group-demo-content\"",
        "\"ui-gallery-toggle-group-usage-content\"",
        "\"ui-gallery-toggle-group-outline-content\"",
        "\"ui-gallery-toggle-group-size-content\"",
        "\"ui-gallery-toggle-group-spacing-content\"",
        "\"ui-gallery-toggle-group-vertical-content\"",
        "\"ui-gallery-toggle-group-disabled-content\"",
        "\"ui-gallery-toggle-group-custom-content\"",
        "\"ui-gallery-toggle-group-rtl-content\"",
        "\"ui-gallery-toggle-group-api-reference-title\"",
        "\"ui-gallery-toggle-group-api-reference-content\"",
        "\"ui-gallery-toggle-group-children-content\"",
        "\"ui-gallery-toggle-group-single-content\"",
        "\"ui-gallery-toggle-group-small-content\"",
        "\"ui-gallery-toggle-group-large-content\"",
        "\"ui-gallery-toggle-group-label-content\"",
        "\"ui-gallery-toggle-group-full-width-items-content\"",
        "\"ui-gallery-toggle-group-stretch-content\"",
        "\"ui-gallery-toggle-group-notes-content\"",
        "\"ui-gallery-toggle-group-docs-smoke\"",
    ] {
        assert!(
            script.contains(needle),
            "toggle_group docs diag script should cover the docs path and focused Fret follow-ups; missing `{needle}`",
        );
    }
}
