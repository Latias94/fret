#[test]
fn button_link_render_keeps_runtime_action_state_anchors() {
    let link_render = include_str!("../src/ui/snippets/button/link_render.rs");

    for needle in [
        ".render(shadcn::ButtonRender::Link",
        ".action(\"ui_gallery.app.open\")",
        ".test_id(\"ui-gallery-button-render-link\")",
        ".test_id(\"ui-gallery-button-render-link-row\")",
    ] {
        assert!(
            link_render.contains(needle),
            "button link-render snippet should keep runtime-observable semantic-link anchors; missing `{needle}`",
        );
    }
}

#[test]
fn button_link_diag_script_gates_action_state() {
    let script = include_str!(
        "../../../tools/diag-scripts/ui-gallery/button/ui-gallery-button-link-render.json"
    );
    let stub = include_str!("../../../tools/diag-scripts/ui-gallery-button-link-render.json");
    let suite = include_str!(
        "../../../tools/diag-scripts/suites/ui-gallery-button-link-action-state/suite.json"
    );

    for needle in [
        "\"ui-gallery-button-link-render\"",
        "\"FRET_UI_GALLERY_START_PAGE\": \"button\"",
        "\"FRET_UI_GALLERY_START_SECTION\": \"As Link / As Child (Semantic)\"",
        "\"ui-gallery-button-link-semantic-content\"",
        "\"ui-gallery-button-render-link-row\"",
        "\"ui-gallery-button-render-link\"",
        "\"role_is\"",
        "\"label_contains\"",
        "\"semantics_action_is\"",
        "\"focus_is\"",
        "\"wait_command_dispatch_trace\"",
        "\"app_snapshot_field_equals\"",
        "\"ui_gallery.app.open\"",
        "\"cmd.open\"",
        "\"started_from_focus\": true",
        "\"role\": \"link\"",
        "\"text\": \"Login\"",
    ] {
        assert!(
            script.contains(needle),
            "button link-render script should gate the expected runtime path; missing `{needle}`",
        );
    }

    assert!(
        stub.contains(
            "\"to\": \"tools/diag-scripts/ui-gallery/button/ui-gallery-button-link-render.json\""
        ),
        "button link-render redirect stub should point at the canonical button script",
    );
    assert!(
        suite.contains("tools/diag-scripts/ui-gallery/button/ui-gallery-button-link-render.json"),
        "button link action-state suite should reference the promoted script",
    );
}

fn normalize_ws(source: &str) -> String {
    source.split_whitespace().collect()
}

#[test]
fn button_page_keeps_upstream_docs_order_before_fret_followups() {
    let source = include_str!("../src/ui/pages/button.rs");

    for needle in [
        "Preview mirrors the shadcn Button docs order first while keeping the current `new-york-v4` button chrome baseline and a shared semantic-link lane for the Base `As Link` / Radix `As Child` follow-up.",
        "Gallery sections now mirror shadcn Button docs first: Demo, Usage, Cursor, Size, Default, Outline, Secondary, Ghost, Destructive, Link, Icon, With Icon, Rounded, Spinner, Button Group, As Link / As Child (Semantic), RTL, API Reference.",
        "`Children (Fret)` stays after the upstream path to document the landed-element equivalent of JSX child composition without widening `Button` into a generic root `asChild` surface.",
        "`Variants Overview (Fret)` stays after the upstream path so existing variant chrome diagnostics remain easy to compare without displacing the docs order.",
        "DocSection::build(cx, \"Demo\", demo)",
        "DocSection::build(cx, \"Usage\", usage)",
        "DocSection::build(cx, \"Cursor\", cursor)",
        "DocSection::build(cx, \"Size\", size)",
        "DocSection::build(cx, \"Default\", default)",
        "DocSection::build(cx, \"Outline\", outline)",
        "DocSection::build(cx, \"Secondary\", secondary)",
        "DocSection::build(cx, \"Ghost\", ghost)",
        "DocSection::build(cx, \"Destructive\", destructive)",
        "DocSection::build(cx, \"Link\", link)",
        "DocSection::build(cx, \"Icon\", icon_only)",
        "DocSection::build(cx, \"With Icon\", with_icon)",
        "DocSection::build(cx, \"Rounded\", rounded)",
        "DocSection::build(cx, \"Spinner\", spinner)",
        "DocSection::build(cx, \"Button Group\", button_group)",
        "DocSection::build(cx, \"As Link / As Child (Semantic)\", link_render)",
        "DocSection::build(cx, \"RTL\", rtl)",
        "DocSection::build(cx, \"API Reference\", api_reference)",
        "DocSection::build(cx, \"Children (Fret)\", children)",
        "DocSection::build(cx, \"Variants Overview (Fret)\", variants)",
    ] {
        assert!(
            source.contains(needle),
            "button page should document the docs-path order and Fret follow-ups; missing `{needle}`",
        );
    }

    let normalized = normalize_ws(source);
    let ordered_sections = normalize_ws(
        r#"
        vec![
            demo,
            usage,
            cursor,
            size,
            default,
            outline,
            secondary,
            ghost,
            destructive,
            link,
            icon_only,
            with_icon,
            rounded,
            spinner,
            button_group,
            link_render,
            rtl,
            api_reference,
            children,
            variants,
            notes,
        ]
        "#,
    );
    assert!(
        normalized.contains(&ordered_sections),
        "button page should keep the upstream docs path through API Reference before Fret-only follow-ups",
    );
}
