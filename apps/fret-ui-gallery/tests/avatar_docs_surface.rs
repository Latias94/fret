fn normalize_ws(source: &str) -> String {
    source.split_whitespace().collect()
}

#[test]
fn avatar_page_keeps_upstream_docs_order_before_fallback_followup() {
    let source = include_str!("../src/ui/pages/avatar.rs");

    for needle in [
        "Preview now mirrors the shadcn Avatar docs order first, and API Reference now tracks the upstream part breakdown more directly before appending a small Fret-specific fallback-only check.",
        "Gallery sections now mirror shadcn Avatar docs first: Demo, Usage, Basic, Badge, Badge with Icon, Avatar Group, Avatar Group Count, Avatar Group with Icon, Sizes, Dropdown, RTL, API Reference.",
        "`Fallback only` remains a Fret-specific follow-up section for compact regression coverage across sizes.",
        "DocSection::build(cx, \"Demo\", demo)",
        "DocSection::build(cx, \"Usage\", usage)",
        "DocSection::build(cx, \"Basic\", basic)",
        "DocSection::build(cx, \"Badge\", with_badge)",
        "DocSection::build(cx, \"Badge with Icon\", badge_icon)",
        "DocSection::build(cx, \"Avatar Group\", avatar_group)",
        "DocSection::build(cx, \"Avatar Group Count\", group_count)",
        "DocSection::build(cx, \"Avatar Group with Icon\", group_count_icon)",
        "DocSection::build(cx, \"Sizes\", sizes)",
        "DocSection::build(cx, \"Dropdown\", dropdown)",
        "DocSection::build(cx, \"RTL\", rtl)",
        "DocSection::build(cx, \"API Reference\", api_reference)",
        "DocSection::build(cx, \"Fallback only (Fret)\", fallback)",
        "DocSection::build(cx, \"Notes\", notes)",
    ] {
        assert!(
            source.contains(needle),
            "avatar page should document the docs-path order and Fret follow-ups; missing `{needle}`",
        );
    }

    let ordered_sections = normalize_ws(
        r#"
        vec![
            demo,
            usage,
            basic,
            with_badge,
            badge_icon,
            avatar_group,
            group_count,
            group_count_icon,
            sizes,
            dropdown,
            rtl,
            api_reference,
            fallback,
            notes,
        ]
        "#,
    );
    assert!(
        normalize_ws(source).contains(&ordered_sections),
        "avatar page should keep the upstream docs path through API Reference before the Fret-only fallback/notes follow-ups",
    );
}

#[test]
fn avatar_snippets_keep_copyable_docs_surface_and_trigger_ownership() {
    let usage = include_str!("../src/ui/snippets/avatar/usage.rs");
    let dropdown = include_str!("../src/ui/snippets/avatar/dropdown.rs");
    let rtl = include_str!("../src/ui/snippets/avatar/rtl.rs");

    for needle in [
        "use fret::component::ui_assets::ImageId;",
        "use fret::{AppComponentCx, UiChild};",
        "use fret_ui_shadcn::facade as shadcn;",
        "shadcn::Avatar::empty()",
        "shadcn::AvatarImage::maybe(image)",
        "shadcn::AvatarFallback::new(\"CN\")",
        ".when_image_missing(image)",
        ".delay_ms(120)",
        ".test_id(\"ui-gallery-avatar-usage\")",
    ] {
        assert!(
            usage.contains(needle),
            "avatar usage snippet should stay copyable on the curated facade lane; missing `{needle}`",
        );
    }

    for needle in [
        "use fret_ui_shadcn::{facade as shadcn, prelude::*};",
        "shadcn::Button::new(\"\")",
        ".variant(shadcn::ButtonVariant::Ghost)",
        ".a11y_label(\"Open user menu\")",
        ".test_id(\"ui-gallery-avatar-dropdown-trigger-avatar\")",
        ".test_id(\"ui-gallery-avatar-dropdown-trigger-avatar-leaf\")",
        "shadcn::DropdownMenuTrigger::new(trigger)",
        "The nested Avatar is presentational content inside that pressable child.",
        ".test_id(\"ui-gallery-avatar-dropdown-menu\")",
    ] {
        assert!(
            dropdown.contains(needle),
            "avatar dropdown snippet should keep the authored Button as the semantic trigger; missing `{needle}`",
        );
    }

    for needle in [
        "with_direction_provider(cx, LayoutDirection::Rtl",
        ".test_id(\"ui-gallery-avatar-rtl-basic\")",
        ".test_id(\"ui-gallery-avatar-rtl-badge\")",
        ".test_id(\"ui-gallery-avatar-rtl-group\")",
        ".test_id(\"ui-gallery-avatar-rtl-row\")",
    ] {
        assert!(
            rtl.contains(needle),
            "avatar RTL snippet should keep stable docs-path anchors; missing `{needle}`",
        );
    }
}

#[test]
fn avatar_diag_scripts_gate_docs_and_trigger_relation_evidence() {
    let docs_screenshots = include_str!(
        "../../../tools/diag-scripts/ui-gallery/avatar/ui-gallery-avatar-docs-screenshots.json"
    );
    let badge_and_group = include_str!(
        "../../../tools/diag-scripts/ui-gallery/avatar/ui-gallery-avatar-badge-and-group-count.json"
    );
    let relation = include_str!(
        "../../../tools/diag-scripts/ui-gallery/avatar/ui-gallery-avatar-dropdown-relation-action-state.json"
    );
    let attribution_suite = include_str!(
        "../../../tools/diag-scripts/suites/ui-gallery-avatar-dropdown-attribution/suite.json"
    );
    let relation_suite = include_str!(
        "../../../tools/diag-scripts/suites/ui-gallery-avatar-dropdown-relation-action-state/suite.json"
    );

    for needle in [
        "\"ui-gallery-avatar-docs-screenshots\"",
        "\"ui-gallery-avatar-badge-icon\"",
        "\"ui-gallery-avatar-dropdown-trigger-avatar\"",
        "\"ui-gallery-avatar-dropdown-item-profile\"",
        "\"ui-gallery-avatar-dropdown-open\"",
    ] {
        assert!(
            docs_screenshots.contains(needle),
            "avatar docs screenshot script should cover docs-path badge and dropdown evidence; missing `{needle}`",
        );
    }

    for needle in [
        "\"ui-gallery-avatar-badge-and-group-count\"",
        "\"ui-gallery-avatar-badge-icon\"",
        "\"ui-gallery-avatar-group-count-default\"",
    ] {
        assert!(
            badge_and_group.contains(needle),
            "avatar badge/group-count script should keep docs-path anchors observable; missing `{needle}`",
        );
    }

    for needle in [
        "\"FRET_UI_GALLERY_START_PAGE\": \"avatar\"",
        "\"FRET_UI_GALLERY_START_SECTION\": \"Dropdown\"",
        "\"ui-gallery-avatar-dropdown-trigger-avatar\"",
        "\"ui-gallery-avatar-dropdown-trigger-avatar-leaf\"",
        "\"role\": \"button\"",
        "\"role\": \"generic\"",
        "\"action\": \"invoke\"",
        "\"relation\": \"controls\"",
        "\"ui-gallery-avatar-dropdown-menu\"",
    ] {
        assert!(
            relation.contains(needle),
            "avatar dropdown relation/action-state script should gate trigger ownership; missing `{needle}`",
        );
    }

    for script in [
        "ui-gallery-avatar-dropdown-activate-open-trigger.json",
        "ui-gallery-avatar-dropdown-activate-open.json",
        "ui-gallery-avatar-dropdown-click-stable-open.json",
        "ui-gallery-avatar-dropdown-escape-focus-restore.json",
        "ui-gallery-avatar-dropdown-focus-trigger.json",
    ] {
        assert!(
            attribution_suite.contains(script),
            "avatar dropdown attribution suite should include `{script}`",
        );
    }
    assert!(
        relation_suite.contains("ui-gallery-avatar-dropdown-relation-action-state.json"),
        "avatar dropdown relation/action-state suite should include the canonical relation script",
    );
}
