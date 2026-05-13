#[test]
fn checkbox_demo_snippet_keeps_upstream_composite_preview_surface() {
    let source = include_str!("../src/ui/snippets/checkbox/demo.rs");

    assert!(
        source.contains("shadcn::field_group(|cx| {"),
        "checkbox demo should keep the upstream FieldGroup preview shell"
    );
    assert!(
        source.contains("shadcn::Label::new(\"Accept terms and conditions\")"),
        "checkbox demo should keep the plain label row from the upstream preview"
    );
    assert!(
        source.contains("By clicking this checkbox, you agree to the terms."),
        "checkbox demo should keep the description row from the upstream preview"
    );
    assert!(
        source.contains(".disabled(true)")
            && source.contains("ui-gallery-checkbox-demo-disabled")
            && source.contains(".orientation(shadcn::FieldOrientation::Horizontal)"),
        "checkbox demo should keep the disabled field row from the upstream preview"
    );
    assert!(
        source.contains(".wrap([shadcn::Field::new(["),
        "checkbox demo should keep the wrapped title/content row from the upstream preview"
    );
    assert!(
        source.contains(".max_w(Px(384.0))"),
        "checkbox demo should keep the upstream max-w-sm width lane"
    );
    assert!(
        !source.contains(".action("),
        "checkbox demo should stay on the upstream docs-shaped preview lane instead of teaching action-first state here"
    );
    assert!(
        !source.contains("Checkbox::from_checked"),
        "checkbox demo should leave snapshot/action authoring to the dedicated Checked State section"
    );
}

#[test]
fn checkbox_rtl_diag_scripts_use_current_section_anchor_and_scroll_stability_gate() {
    let scroll_to_rtl = include_str!(
        "../../../tools/diag-scripts/ui-gallery/checkbox/ui-gallery-checkbox-scroll-to-rtl-field.json"
    );
    let rtl_and_checked = include_str!(
        "../../../tools/diag-scripts/ui-gallery/checkbox/ui-gallery-checkbox-rtl-and-checked-wrap.json"
    );

    for (path, source) in [
        (
            "tools/diag-scripts/ui-gallery/checkbox/ui-gallery-checkbox-scroll-to-rtl-field.json",
            scroll_to_rtl,
        ),
        (
            "tools/diag-scripts/ui-gallery/checkbox/ui-gallery-checkbox-rtl-and-checked-wrap.json",
            rtl_and_checked,
        ),
    ] {
        assert!(
            !source.contains("ui-gallery-checkbox-rtl-field"),
            "{path} should not target the retired single-field RTL id"
        );
        assert!(
            source.contains("\"id\": \"ui-gallery-checkbox-rtl\""),
            "{path} should target the current RTL section anchor"
        );
        assert!(
            source.contains("\"type\": \"wait_semantics_scroll_stable\"")
                && source.contains("\"id\": \"ui-gallery-content-viewport\"")
                && source.contains("\"field\": \"y\"")
                && source.contains("\"type\": \"wait_bounds_stable\""),
            "{path} should gate RTL scroll/bounds stability instead of relying on screenshots only"
        );
    }
}
