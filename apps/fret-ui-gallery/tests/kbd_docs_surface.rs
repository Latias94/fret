#[test]
fn kbd_page_keeps_docs_order_before_rtl_followup() {
    let source = include_str!("../src/ui/pages/kbd.rs");

    for needle in [
        "Preview mirrors the current shadcn Kbd docs path first: Demo, Usage, Group, Button, Tooltip, Input Group, and API Reference. RTL is a Fret-only follow-up.",
        "`Kbd::new(text)` is the default docs-aligned lane for a single key token such as `Ctrl`, `Esc`, or `⌘`.",
        "`KbdGroup::new([...])` groups adjacent keycaps or separators for shortcut chords while keeping spacing consistent.",
        "`Kbd::from_children([...])` / `.children([...])` remain explicit escape hatches for icon-only or mixed-content caps, so no broader generic `asChild` / `compose()` surface is warranted here.",
        "Composition into buttons, tooltips, and input-group addons stays caller-owned, matching the upstream docs layering.",
        "DocSection::build(cx, \"Demo\", demo)",
        "DocSection::build(cx, \"Usage\", usage)",
        "DocSection::build(cx, \"Group\", group)",
        "DocSection::build(cx, \"Button\", button)",
        "DocSection::build(cx, \"Tooltip\", tooltip)",
        "DocSection::build(cx, \"Input Group\", input_group)",
        "DocSection::build(cx, \"API Reference\", api_reference)",
        "DocSection::build(cx, \"RTL\", rtl)",
    ] {
        assert!(
            source.contains(needle),
            "kbd page should document the docs-path order and RTL follow-up; missing `{needle}`",
        );
    }

    let render_order = source
        .find("vec![demo, usage, group, button, tooltip, input_group, api_reference, rtl]")
        .expect("kbd page should render sections in docs order before the RTL follow-up");
    let docs_path_note = source
        .find("Preview mirrors the current shadcn Kbd docs path first")
        .expect("kbd page should label the docs path before the ordered sections");
    assert!(
        docs_path_note < render_order,
        "kbd page should explain the docs path before rendering the ordered sections",
    );
}

#[test]
fn kbd_snippets_keep_textual_docs_lane_and_narrow_children_escape_hatch() {
    let demo = include_str!("../src/ui/snippets/kbd/demo.rs");
    let usage = include_str!("../src/ui/snippets/kbd/usage.rs");
    let group = include_str!("../src/ui/snippets/kbd/group.rs");
    let button = include_str!("../src/ui/snippets/kbd/button.rs");
    let tooltip = include_str!("../src/ui/snippets/kbd/tooltip.rs");
    let input_group = include_str!("../src/ui/snippets/kbd/input_group.rs");
    let rtl = include_str!("../src/ui/snippets/kbd/rtl.rs");

    for needle in [
        "use fret_ui_kit::declarative::text as decl_text;",
        "shadcn::KbdGroup::new([",
        "shadcn::Kbd::new(\"⌘\")",
        "shadcn::Kbd::new(\"⇧\")",
        "shadcn::Kbd::new(\"⌥\")",
        "shadcn::Kbd::new(\"⌃\")",
        "shadcn::Kbd::new(\"Ctrl\")",
        "decl_text::text_chrome_glyph(cx, \"+\")",
        "shadcn::Kbd::new(\"B\")",
        ".test_id(\"ui-gallery-kbd-demo\")",
    ] {
        assert!(
            demo.contains(needle),
            "kbd Demo snippet should keep the textual/glyph docs lane; missing `{needle}`",
        );
    }

    for needle in [
        "use fret_ui_shadcn::facade as shadcn;",
        "shadcn::Kbd::new(\"Ctrl\")",
        ".test_id(\"ui-gallery-kbd-usage\")",
    ] {
        assert!(
            usage.contains(needle),
            "kbd Usage snippet should stay on the single-token facade lane; missing `{needle}`",
        );
    }

    for needle in [
        "decl_text::text_control_readout(cx, \"Use\")",
        "shadcn::KbdGroup::new([",
        "shadcn::Kbd::new(\"Ctrl + B\")",
        "shadcn::Kbd::new(\"Ctrl + K\")",
        "decl_text::text_control_readout(cx, \"to open the command palette\")",
        ".test_id(\"ui-gallery-kbd-group\")",
    ] {
        assert!(
            group.contains(needle),
            "kbd Group snippet should keep grouped shortcuts on the primary lane; missing `{needle}`",
        );
    }

    for needle in [
        "shadcn::Button::new(\"Accept\")",
        "shadcn::Kbd::new(\"⏎\")",
        "shadcn::Button::new(\"Cancel\")",
        "shadcn::Kbd::new(\"Esc\")",
        ".test_id(\"ui-gallery-kbd-button\")",
    ] {
        assert!(
            button.contains(needle),
            "kbd Button snippet should keep caller-owned button composition; missing `{needle}`",
        );
    }

    for needle in [
        "shadcn::TooltipProvider::new()",
        "decl_text::text_control_readout(cx, \"Save Changes\")",
        "shadcn::Kbd::new(\"S\")",
        "decl_text::text_control_readout(cx, \"Print Document\")",
        "shadcn::Kbd::new(\"Ctrl\")",
        "shadcn::Kbd::new(\"P\")",
        ".test_id(\"ui-gallery-kbd-tooltip\")",
    ] {
        assert!(
            tooltip.contains(needle),
            "kbd Tooltip snippet should keep caller-owned tooltip composition; missing `{needle}`",
        );
    }

    for needle in [
        "shadcn::InputGroup::new(value)",
        ".trailing([",
        "shadcn::Kbd::new(\"⌘\")",
        "shadcn::Kbd::new(\"K\")",
        ".trailing_has_kbd(true)",
        ".test_id(\"ui-gallery-kbd-input-group\")",
    ] {
        assert!(
            input_group.contains(needle),
            "kbd Input Group snippet should keep trailing kbd composition on InputGroup; missing `{needle}`",
        );
    }

    for needle in [
        "with_direction_provider(cx, LayoutDirection::Rtl",
        "shadcn::KbdGroup::new([",
        "shadcn::Kbd::new(\"Ctrl\")",
        "decl_text::text_chrome_glyph(cx, \"+\")",
        ".test_id(\"ui-gallery-kbd-rtl\")",
    ] {
        assert!(
            rtl.contains(needle),
            "kbd RTL follow-up should keep the same textual/glyph lane under direction context; missing `{needle}`",
        );
    }

    let combined = [demo, usage, group, button, tooltip, input_group, rtl].join("\n");
    for forbidden in ["shadcn::raw::", "advanced::", "compose()", "asChild"] {
        assert!(
            !combined.contains(forbidden),
            "kbd copyable snippets should not promote `{forbidden}` on the docs lane",
        );
    }
    assert!(
        !combined.contains("Kbd::from_children"),
        "kbd snippets should keep from_children as an API Reference escape hatch, not the docs-path default",
    );
}

#[test]
fn kbd_diag_script_matches_docs_order_and_runtime_anchors() {
    let script =
        include_str!("../../../tools/diag-scripts/ui-gallery/kbd/ui-gallery-kbd-docs-smoke.json");

    for needle in [
        "\"ui-gallery-kbd-docs-smoke\"",
        "\"type_text\", \"text\": \"kbd\"",
        "\"ui-gallery-nav-kbd\"",
        "\"ui-gallery-page-kbd\"",
        "\"ui-gallery-kbd-demo-content\"",
        "\"ui-gallery-kbd-usage-content\"",
        "\"ui-gallery-kbd-group-content\"",
        "\"ui-gallery-kbd-button-content\"",
        "\"ui-gallery-kbd-tooltip-content\"",
        "\"ui-gallery-kbd-input-group-content\"",
        "\"ui-gallery-kbd-api-reference-content\"",
        "\"ui-gallery-kbd-rtl-content\"",
    ] {
        assert!(
            script.contains(needle),
            "kbd docs smoke script should cover the docs-path and RTL follow-up anchors; missing `{needle}`",
        );
    }

    let api_reference_index = script
        .find("\"ui-gallery-kbd-api-reference-content\"")
        .expect("kbd docs smoke script should wait for API Reference");
    let rtl_index = script
        .find("\"ui-gallery-kbd-rtl-content\"")
        .expect("kbd docs smoke script should wait for RTL");
    assert!(
        api_reference_index < rtl_index,
        "kbd docs smoke script should follow the page order: API Reference before the Fret-only RTL follow-up",
    );
}
