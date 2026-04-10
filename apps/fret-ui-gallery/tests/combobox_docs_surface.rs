fn normalize_ws(source: &str) -> String {
    source.split_whitespace().collect()
}

#[test]
fn combobox_page_documents_source_axes_and_children_api_decision() {
    let source = include_str!("../src/ui/pages/combobox.rs");

    for needle in [
        "Reference stack: shadcn Combobox docs and examples plus the default registry recipe.",
        "Headless contract references: Base UI combobox parts and combobox root semantics.",
        "Radix Primitives does not ship a standalone `Combobox` primitive",
        "`Combobox::new(value, open)` plus the direct builder chain (`.trigger(...).input(...).clear(...).content(...)`) is the default recipe root lane, while `into_element_parts(...)` stays the focused upstream-shaped patch seam on that same lane rather than a separate `compose()` story.",
        "`Combobox::device_shell_responsive(true)` remains the explicit viewport/device-shell follow-up for the responsive example instead of widening the default docs path.",
        "docs/public-surface drift rather than a `fret-ui` mechanism bug",
        "No extra generic root `children(...)` / `compose()` / `asChild` API is warranted here",
        "Preview mirrors the shadcn/Base UI Combobox docs path after folding the top preview into `Basic` and skipping `Installation`: `Basic`, `Usage`, `Custom Items`, `Multiple Selection`, `Clear Button`, `Groups`, `Invalid`, `Disabled`, `Auto Highlight`, `Popup`, `Input Group`, `RTL`, and `API Reference`. `Conformance Demo`, `Groups + Separator`, `Label Association`, and `Long List` stay as explicit Fret follow-ups.",
        ".test_id_prefix(\"ui-gallery-combobox-usage\")",
        ".test_id_prefix(\"ui-gallery-combobox-api-reference\")",
        ".test_id_prefix(\"ui-gallery-combobox-label\")",
        ".test_id_prefix(\"ui-gallery-combobox-notes\")",
    ] {
        assert!(
            source.contains(needle),
            "combobox page should document the source axes and children-api decision; missing `{needle}`"
        );
    }

    let normalized = normalize_ws(source);
    let ordered_sections = normalize_ws(
        r#"
        vec![
            basic,
            usage,
            custom_items,
            multiple,
            clear,
            groups,
            invalid,
            disabled,
            auto_highlight,
            popup,
            input_group,
            rtl,
            api_reference,
            conformance_demo,
            groups_with_separator,
            label,
            long_list,
            notes,
        ]
        "#,
    );
    assert!(
        normalized.contains(&ordered_sections),
        "combobox page should keep the docs-path sections before the Fret follow-ups and notes"
    );
}

#[test]
fn combobox_docs_path_snippets_stay_copyable_and_docs_aligned() {
    let usage = include_str!("../src/ui/snippets/combobox/usage.rs");
    let basic = include_str!("../src/ui/snippets/combobox/basic.rs");
    let popup = include_str!("../src/ui/snippets/combobox/trigger_button.rs");
    let input_group = include_str!("../src/ui/snippets/combobox/input_group.rs");
    let rtl = include_str!("../src/ui/snippets/combobox/rtl.rs");
    let normalized_basic = normalize_ws(basic);

    for needle in [
        "use fret::{UiChild, UiCx};",
        "shadcn::Combobox::new(value, open)",
        ".query_model(query)",
        ".trigger(shadcn::ComboboxTrigger::new().width_px(Px(200.0)))",
        ".a11y_label(\"Framework combobox\")",
        ".test_id_prefix(\"ui-gallery-combobox-usage\")",
    ] {
        assert!(
            usage.contains(needle),
            "combobox usage snippet should remain a complete copyable direct-root example; missing `{needle}`"
        );
    }

    for needle in [
        ".content(shadcn::ComboboxContent::new([",
        "shadcn::ComboboxContentPart::input(",
        "shadcn::ComboboxEmpty::new(\"No framework found.\")",
        ".test_id_prefix(\"ui-gallery-combobox-basic\")",
    ] {
        assert!(
            basic.contains(needle),
            "combobox basic snippet should keep the docs-aligned content/input composition visible; missing `{needle}`"
        );
    }
    assert!(
        normalized_basic.contains(".w_full().max_w(Px(260.0)).min_w_0()"),
        "combobox basic snippet should keep width ownership on the caller/root lane while allowing whitespace-insensitive formatting",
    );
    assert!(
        !basic.contains(".width_px(Px(260.0))"),
        "combobox basic snippet should keep width ownership on the caller/root lane instead of forcing a fixed-width trigger",
    );

    for needle in [
        ".variant(shadcn::ComboboxTriggerVariant::Button)",
        ".input(shadcn::ComboboxInput::new().placeholder(\"Select a framework\"))",
        ".content(shadcn::ComboboxContent::new([",
        "placeholder(\"Change framework...\")",
        "shadcn::ComboboxEmpty::new(\"No results found.\")",
        ".test_id_prefix(\"ui-gallery-combobox-popup\")",
    ] {
        assert!(
            popup.contains(needle),
            "combobox popup snippet should keep the typed popup-content input lane explicit; missing `{needle}`"
        );
    }

    for needle in [
        "ComboboxInput::new()",
        ".children([shadcn::InputGroupAddon::new([icon::icon(",
        "InputGroupAddonAlign::InlineStart",
        "format!(\"{test_id_prefix}-selected\")",
        "format!(\"{test_id_prefix}-query\")",
    ] {
        assert!(
            input_group.contains(needle),
            "combobox input-group snippet should keep the typed addon composition and state rows; missing `{needle}`"
        );
    }

    for needle in [
        "with_direction_provider(cx, LayoutDirection::Rtl, |cx| {",
        "\"اختر إطار العمل\"",
        ".test_id_prefix(\"ui-gallery-combobox-rtl\")",
    ] {
        assert!(
            rtl.contains(needle),
            "combobox rtl snippet should keep the explicit RTL provider lane; missing `{needle}`"
        );
    }

    let combined = [usage, basic, popup, input_group, rtl].join("\n");
    assert!(
        !combined.contains("compose()"),
        "combobox docs-path snippets should keep the default teaching lane on the direct root builder instead of inventing a compose() surface",
    );
    assert!(
        !combined.contains("Combobox::children("),
        "combobox docs-path snippets should not widen the root surface into a generic children API",
    );
}

#[test]
fn combobox_follow_up_long_list_keeps_width_on_the_caller_lane() {
    let long_list = include_str!("../src/ui/snippets/combobox/long_list.rs");
    let normalized_long_list = normalize_ws(long_list);

    for needle in [".test_id_prefix(\"ui-gallery-combobox-long-list\")"] {
        assert!(
            long_list.contains(needle),
            "combobox long-list follow-up should clamp to the docs column while keeping caller-owned width; missing `{needle}`",
        );
    }
    assert!(
        normalized_long_list.contains(".w_full().max_w(Px(320.0)).min_w_0()"),
        "combobox long-list follow-up should keep the combobox root on the caller-owned width lane",
    );
    assert!(
        normalized_long_list.contains(".w_full().max_w(Px(360.0)).min_w_0()"),
        "combobox long-list follow-up should keep the outer preview shell on the caller-owned width lane",
    );

    assert!(
        !long_list.contains(".width_px(Px(320.0))"),
        "combobox long-list follow-up should not force a fixed-width trigger that can overflow a narrow docs column",
    );
}

#[test]
fn combobox_docs_diag_scripts_cover_docs_smoke_and_existing_follow_ups() {
    let docs_script = include_str!(
        "../../../tools/diag-scripts/ui-gallery/combobox/ui-gallery-combobox-docs-smoke.json"
    );
    let docs_stub = include_str!("../../../tools/diag-scripts/ui-gallery-combobox-docs-smoke.json");
    let docs_suite =
        include_str!("../../../tools/diag-scripts/suites/ui-gallery-shadcn-conformance/suite.json");
    let popup_gate = include_str!(
        "../../../tools/diag-scripts/ui-gallery/combobox/ui-gallery-combobox-popup-trigger.json"
    );
    let label_gate = include_str!(
        "../../../tools/diag-scripts/ui-gallery/combobox/ui-gallery-combobox-label-click-focus.json"
    );

    for needle in [
        "\"ui-gallery-page-combobox\"",
        "\"docsec-basic-content\"",
        "\"ui-gallery-combobox-usage-content\"",
        "\"docsec-custom-items-content\"",
        "\"docsec-multiple-selection-content\"",
        "\"docsec-clear-button-content\"",
        "\"docsec-groups-content\"",
        "\"docsec-invalid-content\"",
        "\"docsec-disabled-content\"",
        "\"docsec-auto-highlight-content\"",
        "\"docsec-popup-content\"",
        "\"docsec-input-group-content\"",
        "\"docsec-rtl-content\"",
        "\"ui-gallery-combobox-api-reference-content\"",
        "\"docsec-conformance-demo-content\"",
        "\"docsec-groups-separator-content\"",
        "\"ui-gallery-combobox-label-content\"",
        "\"docsec-long-list-content\"",
        "\"ui-gallery-combobox-notes-content\"",
        "\"ui-gallery-combobox-docs-smoke\"",
    ] {
        assert!(
            docs_script.contains(needle),
            "combobox docs smoke script should cover the docs path and the Fret follow-ups; missing `{needle}`"
        );
    }

    assert!(
        docs_stub.contains(
            "\"to\": \"tools/diag-scripts/ui-gallery/combobox/ui-gallery-combobox-docs-smoke.json\""
        ),
        "combobox docs smoke stub should redirect to the docs smoke script"
    );
    assert!(
        docs_suite.contains("\"tools/diag-scripts/ui-gallery-combobox-docs-smoke.json\""),
        "shadcn conformance suite should include the combobox docs smoke gate"
    );

    for needle in [
        "\"ui-gallery-combobox-popup-trigger\"",
        "\"ui-gallery-combobox-popup-listbox\"",
        "\"ui-gallery-combobox-popup-item-next\"",
    ] {
        assert!(
            popup_gate.contains(needle),
            "combobox popup gate should keep the popup follow-up selectors stable; missing `{needle}`"
        );
    }

    for needle in [
        "\"ui-gallery-combobox-label-label\"",
        "\"ui-gallery-combobox-label-trigger\"",
        "\"ui-gallery-combobox-label-click-focus\"",
    ] {
        assert!(
            label_gate.contains(needle),
            "combobox label gate should keep the label-association follow-up selectors stable; missing `{needle}`"
        );
    }
}
