fn normalize_ws(source: &str) -> String {
    source.split_whitespace().collect()
}

#[test]
fn toggle_group_page_documents_source_axes_and_children_api_decision() {
    let source = include_str!("../src/ui/pages/toggle_group.rs");

    for needle in [
        "Reference stack for this page: current shadcn Toggle Group docs, the `new-york-v4` registry recipe, Radix Primitives Toggle Group, and Base UI Toggle Group.",
        "Current upstream docs path: top `ComponentPreview` uses `toggle-group-spacing`, then `Usage`, `Outline`, `Single`, `Small`, `Large`, `Disabled`, `Spacing`, and `API Reference`; this gallery keeps one `Spacing` section for that repeated preview/example.",
        "toggle-group.tsx",
        "toggle-group-spacing.tsx",
        "toggle-group-outline.tsx",
        "toggle-group-single.tsx",
        "toggle-group-sm.tsx",
        "toggle-group-lg.tsx",
        "toggle-group-disabled.tsx",
        "`Demo`, `Vertical`, `Custom`, and `RTL` are explicit Fret/base-radix follow-ups rather than current new-york-v4 docs-path sections.",
        "`fret_ui_kit::primitives::toggle_group` already covers the mechanism lane",
        "No extra root `children([...])` or generic `compose()` API is warranted on the default lane because the helper family already covers composable item assembly without widening the recipe contract.",
        "Preview now mirrors the current Toggle Group docs path first: `Spacing`, `Usage`, `Outline`, `Single`, `Small`, `Large`, `Disabled`, and `API Reference`.",
        "Focused follow-ups stay afterward: `Demo (Fret)`, `Vertical (Base/Radix)`, `Custom (Fret)`, `RTL (Fret)`, `Children (Fret)`, `Label Association (Fret)`, `Disabled Item Action-State (Fret)`, `Full Width Items (Fret)`, `Flex-1 Items (Fret)`, and `Notes`.",
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
            spacing,
            usage,
            outline,
            single,
            small,
            large,
            disabled,
            api_reference,
            demo,
            vertical,
            custom,
            rtl,
            children,
            label,
            disabled_item_action_state,
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

    assert!(
        !source.contains("DocSection::build(cx, \"Size\", size)"),
        "toggle_group page should not keep the old aggregate Size docs section after splitting current upstream Single/Small/Large examples",
    );
}

#[test]
fn toggle_group_snippets_stay_copyable_and_upstream_example_aligned() {
    let demo = include_str!("../src/ui/snippets/toggle_group/demo.rs");
    let usage = include_str!("../src/ui/snippets/toggle_group/usage.rs");
    let outline = include_str!("../src/ui/snippets/toggle_group/outline.rs");
    let spacing = include_str!("../src/ui/snippets/toggle_group/spacing.rs");
    let single = include_str!("../src/ui/snippets/toggle_group/single.rs");
    let small = include_str!("../src/ui/snippets/toggle_group/small.rs");
    let large = include_str!("../src/ui/snippets/toggle_group/large.rs");
    let disabled = include_str!("../src/ui/snippets/toggle_group/disabled.rs");
    let rtl = include_str!("../src/ui/snippets/toggle_group/rtl.rs");
    let children = include_str!("../src/ui/snippets/toggle_group/children.rs");
    let label = include_str!("../src/ui/snippets/toggle_group/label.rs");

    for needle in [
        "use fret::{AppComponentCx, UiChild};",
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
        "ToggleGroupItem::new(\"a\", [decl_text::text_button_label(cx, \"A\")])",
        "ToggleGroupItem::new(\"b\", [decl_text::text_button_label(cx, \"B\")])",
        "ToggleGroupItem::new(\"c\", [decl_text::text_button_label(cx, \"C\")])",
        ".test_id(\"ui-gallery-toggle-group-usage\")",
    ] {
        assert!(
            usage.contains(needle),
            "toggle_group usage snippet should remain the minimal copyable example; missing `{needle}`",
        );
    }

    for needle in [
        ".variant(shadcn::ToggleVariant::Outline)",
        "ToggleGroup::multiple_uncontrolled(std::iter::empty::<&'static str>())",
        "IconId::new_static(\"lucide.bold\")",
        "IconId::new_static(\"lucide.italic\")",
        "IconId::new_static(\"lucide.underline\")",
        ".test_id(\"ui-gallery-toggle-group-outline\")",
    ] {
        assert!(
            outline.contains(needle),
            "toggle_group outline snippet should mirror the current upstream multiple icon-only outline example; missing `{needle}`",
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

    for (source, size_marker, test_id, label) in [
        (
            single,
            "ToggleGroup::single_uncontrolled(Option::<&'static str>::None)",
            ".test_id(\"ui-gallery-toggle-group-single\")",
            "Single",
        ),
        (
            small,
            ".size(shadcn::ToggleSize::Sm)",
            ".test_id(\"ui-gallery-toggle-group-small\")",
            "Small",
        ),
        (
            large,
            ".size(shadcn::ToggleSize::Lg)",
            ".test_id(\"ui-gallery-toggle-group-large\")",
            "Large",
        ),
    ] {
        for needle in [
            size_marker,
            "IconId::new_static(\"lucide.bold\")",
            "IconId::new_static(\"lucide.italic\")",
            "IconId::new_static(\"lucide.underline\")",
            test_id,
        ] {
            assert!(
                source.contains(needle),
                "toggle_group {label} snippet should mirror the current upstream example; missing `{needle}`",
            );
        }
    }

    for needle in [
        "ToggleGroup::multiple_uncontrolled(std::iter::empty::<&'static str>())",
        ".disabled(true)",
        ".test_id(\"ui-gallery-toggle-group-disabled\")",
    ] {
        assert!(
            disabled.contains(needle),
            "toggle_group disabled snippet should match the current upstream multiple disabled example; missing `{needle}`",
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
        "\"ui-gallery-toggle-group-spacing-content\"",
        "\"ui-gallery-toggle-group-single-content\"",
        "\"ui-gallery-toggle-group-small-content\"",
        "\"ui-gallery-toggle-group-large-content\"",
        "\"ui-gallery-toggle-group-vertical-content\"",
        "\"ui-gallery-toggle-group-disabled-content\"",
        "\"ui-gallery-toggle-group-custom-content\"",
        "\"ui-gallery-toggle-group-rtl-content\"",
        "\"ui-gallery-toggle-group-api-reference-title\"",
        "\"ui-gallery-toggle-group-api-reference-content\"",
        "\"ui-gallery-toggle-group-children-content\"",
        "\"ui-gallery-toggle-group-label-content\"",
        "\"ui-gallery-toggle-group-disabled-item-action-state-content\"",
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

    assert!(
        !script.contains("\"ui-gallery-toggle-group-size-content\""),
        "toggle_group docs diag script should target current upstream Single/Small/Large sections instead of the old aggregate Size section",
    );
}
