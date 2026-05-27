fn normalize_ws(source: &str) -> String {
    source.split_whitespace().collect()
}

#[test]
fn switch_page_keeps_current_docs_path_and_followup_labels() {
    let source = include_str!("../src/ui/pages/switch.rs");

    for needle in [
        "Reference stack: current shadcn Switch docs and new-york-v4 source, with current new-york-v4 Field/Form examples plus Base/Radix registry examples as secondary references.",
        "Current docs path stays `Demo` and `Usage`; richer field/form/base examples stay labeled as registry or Fret follow-ups instead of being treated as current shadcn docs-path sections.",
        "Switch remains a leaf control surface",
        "generic children API",
        "DocSection::build(cx, \"Demo\", demo)",
        "DocSection::build(cx, \"Usage\", usage)",
        "DocSection::build(cx, \"Description (Registry)\", description)",
        "DocSection::build(cx, \"Invalid (Registry)\", invalid)",
        "DocSection::build(cx, \"Disabled (Base/Radix)\", disabled)",
        "DocSection::build(cx, \"Size (Base/Radix)\", sizes)",
        "DocSection::build(cx, \"Choice Card (Fret)\", choice_card)",
        "DocSection::build(cx, \"Read Only (Fret)\", read_only)",
        "DocSection::build(cx, \"Command Gate (Fret)\", command_gate)",
        "DocSection::build(cx, \"RTL (Fret)\", rtl)",
        "DocSection::build(cx, \"Label Association (Fret)\", label)",
        "DocSection::build(cx, \"Style Override (Fret)\", style_override)",
        "DocSection::build(cx, \"API Reference (Fret)\", api_reference)",
    ] {
        assert!(
            source.contains(needle),
            "switch page should keep current source-axis wording and follow-up labels; missing `{needle}`",
        );
    }

    let normalized = normalize_ws(source);
    let ordered_sections = normalize_ws(
        r#"
        vec![
            demo,
            usage,
            description,
            choice_card,
            disabled,
            read_only,
            command_gate,
            invalid,
            sizes,
            rtl,
            label,
            style_override,
            api_reference,
        ]
        "#,
    );
    assert!(
        normalized.contains(&ordered_sections),
        "switch page should keep current docs-path sections before registry/base-radix and Fret follow-ups",
    );
}

#[test]
fn switch_snippets_stay_copyable_and_source_aligned() {
    let demo = include_str!("../src/ui/snippets/switch/airplane_mode.rs");
    let usage = include_str!("../src/ui/snippets/switch/usage.rs");
    let description = include_str!("../src/ui/snippets/switch/description.rs");
    let disabled = include_str!("../src/ui/snippets/switch/disabled.rs");
    let sizes = include_str!("../src/ui/snippets/switch/sizes.rs");
    let invalid = include_str!("../src/ui/snippets/switch/invalid.rs");

    for needle in [
        "use fret::{AppComponentCx, UiChild};",
        "let control_id = ControlId::from(\"ui-gallery-switch-airplane\");",
        ".control_id(control_id.clone())",
        "shadcn::Label::new(\"Airplane Mode\")",
        ".for_control(control_id)",
        ".test_id(\"ui-gallery-switch-airplane-toggle\")",
        ".test_id(\"ui-gallery-switch-airplane-label\")",
    ] {
        assert!(
            demo.contains(needle),
            "switch demo snippet should mirror the current switch-demo label/control lane; missing `{needle}`",
        );
    }

    for needle in [
        "use fret::{AppComponentCx, UiChild};",
        "use fret_ui_shadcn::facade as shadcn;",
        "shadcn::Switch::new(cx.local_model(|| false))",
        ".a11y_label(\"Airplane mode\")",
    ] {
        assert!(
            usage.contains(needle),
            "switch usage snippet should remain a complete copyable app-facing example; missing `{needle}`",
        );
    }

    for needle in [
        "FieldContent::new([",
        "FieldLabel::new(\"Share across devices\")",
        "FieldDescription::new(",
        ".orientation(shadcn::FieldOrientation::Horizontal)",
        ".max_w(Px(320.0))",
    ] {
        assert!(
            description.contains(needle),
            "switch description snippet should keep the current new-york-v4 field-switch shape; missing `{needle}`",
        );
    }

    for needle in [
        "Switch::from_checked(false)",
        ".disabled(true)",
        "FieldLabel::new(\"Disabled\")",
    ] {
        assert!(
            disabled.contains(needle),
            "switch disabled snippet should keep the Base/Radix disabled example lane; missing `{needle}`",
        );
    }

    for needle in [
        "let small_id = ControlId::from(\"ui-gallery-switch-size-sm\");",
        "let default_id = ControlId::from(\"ui-gallery-switch-size-default\");",
        ".size(shadcn::SwitchSize::Sm)",
        ".test_id(\"ui-gallery-switch-size-small-label\")",
        ".test_id(\"ui-gallery-switch-size-default-label\")",
    ] {
        assert!(
            sizes.contains(needle),
            "switch sizes snippet should keep the Base/Radix size example lane; missing `{needle}`",
        );
    }

    for needle in [
        ".aria_invalid(true)",
        ".test_id(\"ui-gallery-switch-invalid-control\")",
        ".invalid(true)",
    ] {
        assert!(
            invalid.contains(needle),
            "switch invalid snippet should keep control-level invalid ownership visible; missing `{needle}`",
        );
    }
}

#[test]
fn switch_docs_diag_scripts_cover_state_depth() {
    let docs_smoke = include_str!(
        "../../../tools/diag-scripts/ui-gallery/switch/ui-gallery-switch-docs-smoke.json"
    );
    let label_clicks = include_str!(
        "../../../tools/diag-scripts/ui-gallery/switch/ui-gallery-switch-docs-label-clicks.json"
    );
    let choice_card = include_str!(
        "../../../tools/diag-scripts/ui-gallery/switch/ui-gallery-switch-choice-card-checked-state-mutation.json"
    );
    let command_gate = include_str!(
        "../../../tools/diag-scripts/ui-gallery/switch/ui-gallery-switch-command-gated-action-state.json"
    );
    let read_only = include_str!(
        "../../../tools/diag-scripts/ui-gallery/switch/ui-gallery-switch-read-only-action-state.json"
    );

    for needle in [
        "ui-gallery-switch-sizes",
        "ui-gallery-switch-docs-zinc-light",
        "ui-gallery-switch-docs-zinc-dark",
    ] {
        assert!(
            docs_smoke.contains(needle),
            "switch docs smoke script should cover size/chrome screenshots; missing `{needle}`",
        );
    }

    for needle in [
        "ui-gallery-switch-airplane-label",
        "ui-gallery-switch-size-small-label",
        "ui-gallery-switch-size-default-label",
        "ui-gallery-switch-rtl-label",
        "checked_is",
    ] {
        assert!(
            label_clicks.contains(needle),
            "switch label-click script should cover docs label/control associations; missing `{needle}`",
        );
    }

    for needle in [
        "ui-gallery-switch-choice-card-share-label",
        "ui-gallery-switch-choice-card-notifications-label",
        "semantics_action_is",
        "checked_is",
    ] {
        assert!(
            choice_card.contains(needle),
            "switch choice-card script should cover checked state and invoke semantics; missing `{needle}`",
        );
    }

    for needle in [
        "ui-gallery-switch-command-gate-control",
        "ui-gallery-switch-command-gate-enabled-toggle",
        "semantics_action_is",
    ] {
        assert!(
            command_gate.contains(needle),
            "switch command-gate script should cover disabled action-state changes; missing `{needle}`",
        );
    }

    for needle in [
        "ui-gallery-switch-read-only-control",
        "ui-gallery-switch-read-only-label",
        "semantics_action_is",
    ] {
        assert!(
            read_only.contains(needle),
            "switch read-only script should keep non-invokable semantics observable; missing `{needle}`",
        );
    }
}
