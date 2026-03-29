fn normalize_ws(source: &str) -> String {
    source.split_whitespace().collect()
}

#[test]
fn slider_page_documents_source_axes_and_children_api_decision() {
    let source = include_str!("../src/ui/pages/slider.rs");

    for needle in [
        "repo-ref/ui/apps/v4/content/docs/components/radix/slider.mdx",
        "repo-ref/ui/apps/v4/content/docs/components/base/slider.mdx",
        "repo-ref/ui/apps/v4/examples/radix/slider-demo.tsx",
        "repo-ref/ui/apps/v4/examples/radix/slider-vertical.tsx",
        "repo-ref/ui/apps/v4/registry/new-york-v4/ui/slider.tsx",
        "repo-ref/ui/apps/v4/registry/bases/radix/ui/slider.tsx",
        "repo-ref/ui/apps/v4/registry/bases/base/ui/slider.tsx",
        "The upstream docs surface intentionally splits the top-of-page preview (`[75]`) from the `Usage` code block (`[33]`), so this page mirrors those two lanes instead of normalizing them to one demo value.",
        "Slider already exposes the important authoring surface (`new`, `new_controllable`, range/step/orientation/on_value_commit), so the main parity gap here is usage clarity rather than missing composition APIs.",
        "generic composable children / `compose()` API",
        "Base UI's `Slider.Root/Label/Value/Control/Track/Indicator/Thumb` family is a useful headless reference, but it belongs to a future `fret-ui-kit`-level surface rather than the `fret-ui-shadcn::Slider` recipe.",
        "Vertical sliders keep the upstream `min-h-44` floor; examples can still pass an explicit height to bound the docs lane, but values below the floor clamp upward unless the caller asks for something taller.",
        "Preview now mirrors the upstream shadcn/Base UI slider docs path first: `Demo`, `Usage`, `Range`, `Multiple Thumbs`, `Vertical`, `Controlled`, `Disabled`, `RTL`, and `API Reference`.",
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
        "slider page should keep the docs-path sections before the Fret-only follow-ups",
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

    for needle in ["vec![75.0]", ".a11y_label(\"Slider\")"] {
        assert!(
            demo.contains(needle),
            "slider demo snippet should mirror the upstream preview lane; missing `{needle}`",
        );
    }

    for needle in [
        "use fret::{UiChild, UiCx};",
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
            "slider range snippet should keep the upstream `[25, 50]` / `step(5)` example; missing `{needle}`",
        );
    }

    for needle in ["vec![10.0, 20.0, 70.0]", ".step(10.0)"] {
        assert!(
            multiple.contains(needle),
            "slider multiple-thumbs snippet should keep the upstream `[10, 20, 70]` / `step(10)` example; missing `{needle}`",
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
            "slider docs diag script should cover the docs-path sections; missing `{needle}`",
        );
    }
}

#[test]
fn slider_vertical_upstream_axes_keep_example_height_and_recipe_floor_split() {
    let example = include_str!("../../../repo-ref/ui/apps/v4/examples/radix/slider-vertical.tsx");
    let radix_nova = include_str!("../../../repo-ref/ui/apps/v4/styles/radix-nova/ui/slider.tsx");
    let new_york = include_str!("../../../repo-ref/ui/apps/v4/registry/new-york-v4/ui/slider.tsx");

    assert_eq!(
        example.matches("orientation=\"vertical\"").count(),
        2,
        "upstream radix slider vertical example should keep the two-slider authoring shape",
    );
    assert_eq!(
        example.matches("className=\"h-40\"").count(),
        2,
        "upstream radix slider vertical example should keep caller-owned `h-40` on both sliders",
    );
    assert!(
        radix_nova.contains("data-vertical:min-h-40"),
        "upstream radix-nova slider recipe should keep the docs/example lane vertical floor",
    );
    assert!(
        new_york.contains("data-[orientation=vertical]:min-h-44"),
        "upstream new-york-v4 slider recipe should keep the registry/default-lane vertical floor",
    );
}
