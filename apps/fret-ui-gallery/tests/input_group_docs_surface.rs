fn normalize_ws(source: &str) -> String {
    source.split_whitespace().collect()
}

#[test]
fn input_group_page_keeps_docs_order_and_fret_followups_explicit() {
    let source = include_str!("../src/ui/pages/input_group.rs");

    for needle in [
        "Preview follows shadcn Input Group docs order first: Demo, Usage, Parts Usage (Fret's typed translation of upstream Composition), Align, the example set through Custom Input, RTL, and API Reference. Tooltip, Label Association, and Button Group remain Fret follow-ups.",
        "The upstream `Composition` block maps to Fret's copyable `Parts Usage` section: it keeps `InputGroupInput`, `InputGroupTextarea`, `InputGroupAddon`, `InputGroupButton`, and `InputGroupText` visible without making that parts lane the default shorthand.",
        "Prefer the high-level `InputGroup::new(model)` shorthand for first-party app code, then reach for the explicit parts lane when you want direct shadcn docs parity at the call site.",
        "Both public surfaces stay intentional: the compact `InputGroup::new(model)` slot shorthand is the first-party ergonomic lane, while the part-based primitives remain the direct docs-parity lane.",
        "The `Dropdown` example intentionally stays on `DropdownMenu::compose()`; swapping the trigger to `InputGroupButton` does not by itself require falling back to `build_parts(...)`.",
        "DocSection::build(cx, \"Demo\", demo)",
        "DocSection::build(cx, \"Usage\", usage)",
        "DocSection::build(cx, \"Parts Usage\", parts_usage)",
        "DocSection::build(cx, \"Align\", align)",
        "DocSection::build(cx, \"Align / inline-start\", align_inline_start)",
        "DocSection::build(cx, \"Align / inline-end\", align_inline_end)",
        "DocSection::build(cx, \"Align / block-start\", align_block_start)",
        "DocSection::build(cx, \"Align / block-end\", align_block_end)",
        "DocSection::build(cx, \"Icon\", icon)",
        "DocSection::build(cx, \"Text\", text)",
        "DocSection::build(cx, \"Button\", button)",
        "DocSection::build(cx, \"Kbd\", kbd)",
        "DocSection::build(cx, \"Dropdown\", dropdown)",
        "DocSection::build(cx, \"Spinner\", spinner)",
        "DocSection::build(cx, \"Textarea\", textarea)",
        "DocSection::build(cx, \"Custom Input\", custom_input)",
        "DocSection::build(cx, \"RTL\", rtl)",
        "DocSection::build(cx, \"API Reference\", api_reference)",
        "DocSection::build(cx, \"Tooltip\", tooltip)",
        "DocSection::build(cx, \"Label Association\", label)",
        "DocSection::build(cx, \"Button Group\", button_group)",
        "DocSection::build(cx, \"Notes\", notes)",
    ] {
        assert!(
            source.contains(needle),
            "input-group page should document the docs path, composition bridge, and Fret follow-ups; missing `{needle}`",
        );
    }

    let ordered_sections = normalize_ws(
        r#"
        vec![
            demo,
            usage,
            parts_usage,
            align,
            align_inline_start,
            align_inline_end,
            align_block_start,
            align_block_end,
            icon,
            text,
            button,
            kbd,
            dropdown,
            spinner,
            textarea,
            custom_input,
            rtl,
            api_reference,
            tooltip,
            label,
            button_group,
            notes,
        ]
        "#,
    );
    assert!(
        normalize_ws(source).contains(&ordered_sections),
        "input-group page should render docs-path sections through API Reference before Fret follow-ups",
    );
}

#[test]
fn input_group_snippets_keep_shorthand_parts_and_dropdown_lanes_separate() {
    let usage = include_str!("../src/ui/snippets/input_group/usage.rs");
    let parts_usage = include_str!("../src/ui/snippets/input_group/parts_usage.rs");
    let dropdown = include_str!("../src/ui/snippets/input_group/dropdown.rs");
    let custom_input = include_str!("../src/ui/snippets/input_group/custom_input.rs");
    let rtl = include_str!("../src/ui/snippets/input_group/rtl.rs");
    let label = include_str!("../src/ui/snippets/input_group/label.rs");
    let text = include_str!("../src/ui/snippets/input_group/text.rs");

    for needle in [
        "shadcn::InputGroup::new(query)",
        ".a11y_label(\"Search\")",
        ".placeholder(\"Search...\")",
        ".trailing([icon::icon(cx, IconId::new_static(\"lucide.search\"))])",
        ".test_id(\"ui-gallery-input-group-usage\")",
    ] {
        assert!(
            usage.contains(needle),
            "input-group Usage snippet should keep the compact shorthand lane copyable; missing `{needle}`",
        );
    }
    for forbidden in [".into_element_parts(", ".build_parts(", "InputGroupPart"] {
        assert!(
            !usage.contains(forbidden),
            "input-group Usage should stay on the shorthand lane and not expose `{forbidden}`",
        );
    }

    for needle in [
        "shadcn::InputGroup::new(query)",
        ".into_element_parts(cx, |cx|",
        "shadcn::InputGroupPart::input(",
        "shadcn::InputGroupInput::new()",
        "shadcn::InputGroupPart::addon(",
        "shadcn::InputGroupAddon::new([icon::icon(",
        ".align(shadcn::InputGroupAddonAlign::InlineStart)",
        "shadcn::InputGroupText::new(\"12 results\")",
        ".align(shadcn::InputGroupAddonAlign::InlineEnd)",
        ".test_id(\"ui-gallery-input-group-parts-usage\")",
    ] {
        assert!(
            parts_usage.contains(needle),
            "input-group Parts Usage snippet should keep the typed docs-parity composition lane; missing `{needle}`",
        );
    }

    for needle in [
        "shadcn::DropdownMenu::uncontrolled(cx)",
        ".compose()",
        ".trigger(more_trigger)",
        ".trigger(search_trigger)",
        "shadcn::InputGroupButton::new(\"\")",
        "shadcn::InputGroup::new(file_name)",
        "shadcn::InputGroup::new(query)",
        ".control_test_id(\"ui-gallery-input-group-dropdown-control\")",
        ".trailing([more_dropdown])",
        ".trailing([search_dropdown])",
        ".trailing_has_button(true)",
        ".test_id(\"ui-gallery-input-group-dropdown\")",
    ] {
        assert!(
            dropdown.contains(needle),
            "input-group Dropdown snippet should keep DropdownMenu compose separate from InputGroup parts adapters; missing `{needle}`",
        );
    }
    for forbidden in [".into_element_parts(", ".build_parts(", "InputGroupPart"] {
        assert!(
            !dropdown.contains(forbidden),
            "input-group Dropdown should not fall back to `{forbidden}`",
        );
    }

    for needle in [
        "fn custom_textarea_control(cx: &mut AppComponentCx<'_>, value: Model<String>) -> AnyElement",
        "shadcn::Textarea::new(value)",
        ".test_id(\"ui-gallery-input-group-custom-input-control\")",
        ".custom_textarea(control)",
        ".test_id(\"ui-gallery-input-group-custom-input\")",
    ] {
        assert!(
            custom_input.contains(needle),
            "input-group Custom Input snippet should keep the narrow caller-owned control seam; missing `{needle}`",
        );
    }

    for needle in [
        "with_direction_provider(cx, LayoutDirection::Rtl",
        ".control_test_id(\"ui-gallery-input-group-rtl-control\")",
        ".test_id(\"ui-gallery-input-group-rtl-leading\")",
        ".test_id(\"ui-gallery-input-group-rtl-trailing\")",
        ".test_id(\"ui-gallery-input-group-rtl\")",
    ] {
        assert!(
            rtl.contains(needle),
            "input-group RTL snippet should keep stable leading/control/trailing anchors; missing `{needle}`",
        );
    }

    for needle in [
        "shadcn::InputGroup::new(username)",
        ".control_id(username_id.clone())",
        ".for_control(username_id.clone())",
        ".control_test_id(\"ui-gallery-input-group-label-email-control\")",
        ".test_id(\"ui-gallery-input-group-label\")",
    ] {
        assert!(
            label.contains(needle),
            "input-group Label Association follow-up should keep control association copyable; missing `{needle}`",
        );
    }

    for needle in [
        "shadcn::InputGroup::new(amount)",
        "shadcn::InputGroup::new(website)",
        "ui-gallery-input-group-text-leading",
        "ui-gallery-input-group-text-trailing",
        ".test_id(\"ui-gallery-input-group-text\")",
    ] {
        assert!(
            text.contains(needle),
            "input-group Text snippet should keep stable non-overlap anchors; missing `{needle}`",
        );
    }

    let combined = [usage, parts_usage, dropdown, custom_input, rtl, label, text].join("\n");
    for forbidden in [
        "shadcn::raw::",
        "advanced::",
        "InputGroup::compose(",
        "InputGroup::children(",
        ".build_parts(",
        "asChild",
    ] {
        assert!(
            !combined.contains(forbidden),
            "input-group copyable docs snippets should not promote `{forbidden}` on the default docs surface",
        );
    }
}

#[test]
fn input_group_diag_scripts_cover_docs_dropdown_and_rtl_evidence() {
    let docs_smoke = include_str!(
        "../../../tools/diag-scripts/ui-gallery/input/ui-gallery-input-group-docs-smoke.json"
    );
    let dropdown = include_str!(
        "../../../tools/diag-scripts/ui-gallery/input-group/ui-gallery-input-group-dropdown-relation-action-state.json"
    );
    let rtl = include_str!(
        "../../../tools/diag-scripts/ui-gallery/input-group/ui-gallery-input-group-rtl-addon-order.json"
    );
    let text_non_overlap = include_str!(
        "../../../tools/diag-scripts/ui-gallery/input/ui-gallery-input-group-text-non-overlap.json"
    );
    let label_click = include_str!(
        "../../../tools/diag-scripts/ui-gallery/input/ui-gallery-input-group-label-click-focus.json"
    );
    let addon_tab = include_str!(
        "../../../tools/diag-scripts/ui-gallery/input/ui-gallery-input-group-addon-after-control-tab-focus.json"
    );
    let dropdown_suite = include_str!(
        "../../../tools/diag-scripts/suites/ui-gallery-input-group-dropdown-relation-action-state/suite.json"
    );
    let rtl_suite =
        include_str!("../../../tools/diag-scripts/suites/ui-gallery-button-group/suite.json");

    for needle in [
        "\"name\": \"ui-gallery-input-group-docs-smoke\"",
        "\"FRET_UI_GALLERY_START_PAGE\": \"input_group\"",
        "\"type_text\", \"text\": \"input_group\"",
        "\"ui-gallery-nav-input-group\"",
        "\"ui-gallery-input-group\"",
        "\"ui-gallery-input-group-demo\"",
        "\"ui-gallery-input-group-usage-tabs-trigger-preview\"",
        "\"ui-gallery-input-group-parts-usage-content\"",
        "\"ui-gallery-input-group-api-reference-content\"",
        "\"ui-gallery-input-group-docs-smoke\"",
    ] {
        assert!(
            docs_smoke.contains(needle),
            "input-group docs smoke script should keep docs-path anchors reachable; missing `{needle}`",
        );
    }
    let usage_index = docs_smoke
        .find("\"ui-gallery-input-group-usage-tabs-trigger-preview\"")
        .expect("docs smoke should wait for Usage");
    let parts_index = docs_smoke
        .find("\"ui-gallery-input-group-parts-usage-content\"")
        .expect("docs smoke should wait for Parts Usage");
    let api_index = docs_smoke
        .find("\"ui-gallery-input-group-api-reference-content\"")
        .expect("docs smoke should wait for API Reference");
    assert!(
        usage_index < parts_index && parts_index < api_index,
        "input-group docs smoke should follow Usage -> Parts Usage -> API Reference ordering",
    );

    for needle in [
        "\"FRET_UI_GALLERY_START_SECTION\": \"Dropdown\"",
        "\"ui-gallery-input-group-dropdown-control\"",
        "\"ui-gallery-input-group-dropdown-leading-button\"",
        "\"ui-gallery-input-group-dropdown-leading-menu\"",
        "\"role\": \"text_field\"",
        "\"role\": \"button\"",
        "\"expanded_is\"",
        "\"semantics_relation_includes\"",
        "\"ui-gallery-input-group-dropdown-relation-action-state\"",
    ] {
        assert!(
            dropdown.contains(needle),
            "input-group dropdown diag should gate text-field vs dropdown-trigger ownership; missing `{needle}`",
        );
    }

    for needle in [
        "\"FRET_UI_GALLERY_START_SECTION\": \"RTL\"",
        "\"ui-gallery-input-group-rtl\"",
        "\"ui-gallery-input-group-rtl-leading\"",
        "\"ui-gallery-input-group-rtl-control\"",
        "\"ui-gallery-input-group-rtl-trailing\"",
        "\"bounds_non_overlapping\"",
        "\"capture_layout_sidecar\"",
        "\"ui-gallery-input-group-rtl-addon-order\"",
    ] {
        assert!(
            rtl.contains(needle),
            "input-group RTL diag should gate addon order and non-overlap; missing `{needle}`",
        );
    }

    for (script, needle) in [
        (
            text_non_overlap,
            "\"ui-gallery-input-group-text-non-overlap\"",
        ),
        (label_click, "\"ui-gallery-input-group-label-click-focus\""),
        (
            addon_tab,
            "\"ui-gallery-input-group-addon-after-control-tab-focus\"",
        ),
    ] {
        assert!(
            script.contains(needle),
            "input-group supporting diag script should keep its canonical name `{needle}`",
        );
    }

    assert!(
        dropdown_suite.contains("ui-gallery-input-group-dropdown-relation-action-state.json"),
        "input-group dropdown relation suite should include the canonical relation script",
    );
    assert!(
        rtl_suite.contains(
            "tools/diag-scripts/ui-gallery/input-group/ui-gallery-input-group-rtl-addon-order.json"
        ),
        "button-group/RTL suite should include the input-group RTL addon-order script",
    );
}
