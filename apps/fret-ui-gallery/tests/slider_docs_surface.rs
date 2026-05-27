fn normalize_ws(source: &str) -> String {
    source.split_whitespace().collect()
}

#[test]
fn slider_page_documents_source_axes_and_children_api_decision() {
    let source = include_str!("../src/ui/pages/slider.rs");

    for needle in [
        "Reference stack: current shadcn Slider docs and new-york-v4 source, with Base/Radix registry examples as secondary references.",
        "Example axis: current shadcn slider demo and usage first; Base/Radix examples supply range, multiple-thumbs, vertical, controlled, and disabled follow-ups.",
        "Recipe axis: the default shadcn registry slider plus the base and radix registry variants.",
        "The current upstream docs surface intentionally splits the top-of-page preview (`[50]`, `w-[60%]`) from the `Usage` code block (`[33]`), so this page mirrors those two lanes instead of normalizing them to one demo value.",
        "Default first-party teaching should prefer `slider(model)`, while `new_controllable(...)` stays as the builder-preserving bridge for the upstream `defaultValue` lane and element-owned state.",
        "generic composable children / `compose()` API",
        "Base UI's `Slider.Root/Label/Value/Control/Track/Indicator/Thumb` family is a useful headless reference, but it belongs to a future `fret-ui-kit`-level surface rather than the `fret-ui-shadcn::Slider` recipe.",
        "Vertical sliders keep the upstream `min-h-44` floor; examples can still pass an explicit height to bound the docs lane, but values below the floor clamp upward unless the caller asks for something taller.",
        "Preview mirrors the current shadcn Slider docs path first: `Demo` and `Usage`.",
    ] {
        assert!(
            source.contains(needle),
            "slider page should document source axes and the children-api decision; missing `{needle}`",
        );
    }

    let normalized = normalize_ws(source);
    let ordered_sections = normalize_ws(
        r#"
        vec![
            demo,
            usage,
            range,
            multiple,
            vertical,
            controlled,
            disabled,
            rtl,
            api_reference,
            label,
            extras,
            notes,
        ]
        "#,
    );
    assert!(
        normalized.contains(&ordered_sections),
        "slider page should keep the current docs-path sections before Base/Radix and Fret follow-ups",
    );
}

#[test]
fn slider_snippets_stay_copyable_and_upstream_example_aligned() {
    let demo = include_str!("../src/ui/snippets/slider/demo.rs");
    let usage = include_str!("../src/ui/snippets/slider/usage.rs");
    let range = include_str!("../src/ui/snippets/slider/range.rs");
    let multiple = include_str!("../src/ui/snippets/slider/multiple.rs");
    let vertical = include_str!("../src/ui/snippets/slider/vertical.rs");
    let controlled = include_str!("../src/ui/snippets/slider/controlled.rs");

    for needle in ["vec![50.0]", ".w_percent(60.0)", ".a11y_label(\"Slider\")"] {
        assert!(
            demo.contains(needle),
            "slider demo snippet should mirror the upstream preview lane; missing `{needle}`",
        );
    }
    assert!(
        !demo.contains("vec![75.0]"),
        "slider demo snippet should not keep the stale upstream preview value"
    );

    for needle in [
        "use fret::{AppComponentCx, UiChild};",
        "use fret_ui_shadcn::facade as shadcn;",
        "let values = cx.local_model_keyed(\"ui-gallery-slider-usage-values\", || vec![33.0]);",
        "shadcn::slider(values)",
        ".step(1.0)",
    ] {
        assert!(
            usage.contains(needle),
            "slider usage snippet should remain a complete copyable app-facing example; missing `{needle}`",
        );
    }

    for needle in ["vec![25.0, 50.0]", ".step(5.0)"] {
        assert!(
            range.contains(needle),
            "slider range snippet should keep the Base/Radix `[25, 50]` / `step(5)` example; missing `{needle}`",
        );
    }

    for needle in ["vec![10.0, 20.0, 70.0]", ".step(10.0)"] {
        assert!(
            multiple.contains(needle),
            "slider multiple-thumbs snippet should keep the Base/Radix `[10, 20, 70]` / `step(10)` example; missing `{needle}`",
        );
    }

    assert_eq!(
        vertical
            .matches("shadcn::Slider::new_controllable(")
            .count(),
        2,
        "slider vertical snippet should keep the upstream two-slider vertical example shape",
    );
    for needle in [
        "vec![50.0]",
        "vec![25.0]",
        ".h_px(Px(160.0))",
        ".gap(Space::N6)",
    ] {
        assert!(
            vertical.contains(needle),
            "slider vertical snippet should keep the upstream dual-slider layout and caller-owned height; missing `{needle}`",
        );
    }

    for needle in [
        "use fret_ui_kit::primitives::control_registry::ControlId;",
        "let control_id = ControlId::from(\"ui-gallery-slider-controlled-temperature\");",
        ".for_control(control_id.clone())",
        ".control_id(control_id.clone())",
    ] {
        assert!(
            controlled.contains(needle),
            "slider controlled snippet should keep the upstream label/readout association on the Fret surface; missing `{needle}`",
        );
    }
}

#[test]
fn slider_docs_diag_script_covers_docs_path_sections() {
    let script = include_str!(
        "../../../tools/diag-scripts/ui-gallery/slider/ui-gallery-slider-docs-screenshots.json"
    );

    for needle in [
        "ui-gallery-slider-demo-content",
        "ui-gallery-slider-usage-section-content",
        "ui-gallery-slider-range-section-content",
        "ui-gallery-slider-multiple-section-content",
        "ui-gallery-slider-vertical-section-content",
        "ui-gallery-slider-controlled-section-content",
        "ui-gallery-slider-disabled-section-content",
        "ui-gallery-slider-rtl-section-content",
        "ui-gallery-slider-api-reference-content",
    ] {
        assert!(
            script.contains(needle),
            "slider docs diag script should cover current docs-path and follow-up sections; missing `{needle}`",
        );
    }
}

#[test]
fn slider_docs_diag_script_is_promoted_in_registry_and_suite() {
    let registry = include_str!("../../../tools/diag-scripts/index.json");
    let suite =
        include_str!("../../../tools/diag-scripts/suites/ui-gallery-slider-docs/suite.json");

    for needle in [
        "\"id\": \"ui-gallery-slider-docs-screenshots\"",
        "\"path\": \"tools/diag-scripts/ui-gallery/slider/ui-gallery-slider-docs-screenshots.json\"",
        "\"ui-gallery-slider-docs\"",
        "\"slider\"",
        "\"screenshots\"",
    ] {
        assert!(
            registry.contains(needle),
            "slider docs diag script should stay promoted in the registry; missing `{needle}`",
        );
    }

    for needle in [
        "\"kind\": \"diag_script_suite_manifest\"",
        "\"tools/diag-scripts/ui-gallery/slider/ui-gallery-slider-docs-screenshots.json\"",
    ] {
        assert!(
            suite.contains(needle),
            "slider docs diag suite should keep the promoted docs screenshot script; missing `{needle}`",
        );
    }
}

#[test]
fn slider_vertical_snippet_and_page_keep_example_height_and_recipe_floor_split() {
    let page = include_str!("../src/ui/pages/slider.rs");
    let vertical = include_str!("../src/ui/snippets/slider/vertical.rs");

    assert_eq!(
        vertical
            .matches("orientation(SliderOrientation::Vertical)")
            .count(),
        2,
        "slider vertical snippet should keep the two-slider authoring shape",
    );
    assert_eq!(
        vertical.matches(".h_px(Px(160.0))").count(),
        2,
        "slider vertical snippet should keep caller-owned `h-40`-equivalent height on both sliders",
    );
    assert!(
        page.contains("Vertical sliders keep the upstream `min-h-44` floor"),
        "slider page should keep the recipe-owned vertical floor note",
    );
    assert!(
        page.contains("Base/Radix two-slider vertical example"),
        "slider page should keep the Base/Radix example-height vs recipe-floor split",
    );
}
