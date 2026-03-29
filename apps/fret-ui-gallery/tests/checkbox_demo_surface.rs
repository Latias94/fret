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
