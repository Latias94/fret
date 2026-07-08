use std::fs;
use std::path::PathBuf;

fn manifest_path(relative: &str) -> PathBuf {
    PathBuf::from(env!("CARGO_MANIFEST_DIR")).join(relative)
}

fn read(relative: &str) -> String {
    fs::read_to_string(manifest_path(relative)).expect("read fixture source")
}

fn normalized(source: &str) -> String {
    source.split_whitespace().collect::<String>()
}

fn assert_ordered(source: &str, needles: &[&str], context: &str) {
    let mut cursor = 0;
    for needle in needles {
        let rest = &source[cursor..];
        let Some(offset) = rest.find(needle) else {
            panic!("{context}: missing `{needle}`");
        };
        cursor += offset + needle.len();
    }
}

#[test]
fn tabs_page_keeps_current_docs_path_and_followup_labels() {
    let source = read("src/ui/pages/tabs.rs");

    for needle in [
        "Reference stack: current shadcn Tabs docs and new-york-v4 source, with Base/Radix registry examples as secondary references.",
        "Current docs path stays `Demo` and `Usage`; richer line, vertical, disabled, icon, and basic-list examples stay labeled as Base/Radix registry follow-ups instead of being treated as current shadcn docs-path sections.",
        "Preview mirrors the current shadcn Tabs docs path first: `Demo` and `Usage`.",
        "DocSection::build(cx, \"Line (Base/Radix)\", line)",
        "DocSection::build(cx, \"Vertical (Base/Radix)\", vertical)",
        "DocSection::build(cx, \"Disabled (Base/Radix)\", disabled)",
        "DocSection::build(cx, \"Icons (Base/Radix)\", icons)",
        "DocSection::build(cx, \"List (Base/Radix)\", list)",
        "DocSection::build(cx, \"RTL (Fret)\", rtl)",
        "DocSection::build(cx, \"API Reference (Fret)\", api_reference)",
        "DocSection::build(cx, \"Composable Parts (Fret)\", parts)",
        "DocSection::build(cx, \"Vertical Line (Fret)\", vertical_line)",
        "DocSection::build(cx, \"Extras (Fret)\", extras)",
    ] {
        assert!(
            source.contains(needle),
            "tabs page should keep current source-axis wording and follow-up labels; missing `{needle}`",
        );
    }

    for stale in [
        "DocSection::build(cx, \"Line\", line)",
        "DocSection::build(cx, \"Vertical\", vertical)",
        "DocSection::build(cx, \"Disabled\", disabled)",
        "DocSection::build(cx, \"Icons\", icons)",
        "DocSection::build(cx, \"RTL\", rtl)",
        "DocSection::build(cx, \"API Reference\", api_reference)",
        "DocSection::build(cx, \"List\", list)",
        "DocSection::build(cx, \"Vertical (Line)\", vertical_line)",
        "DocSection::build(cx, \"Extras\", extras)",
    ] {
        assert!(
            !source.contains(stale),
            "tabs page should not treat registry/Fret follow-ups as current docs-path labels; found `{stale}`",
        );
    }

    let render_order_start = source
        .find("let body = doc_layout::render_doc_page")
        .expect("tabs page render_doc_page call");
    assert_ordered(
        &source[render_order_start..],
        &[
            "demo,",
            "usage,",
            "line,",
            "vertical,",
            "disabled,",
            "icons,",
            "rtl,",
            "api_reference,",
            "parts,",
            "list,",
            "vertical_line,",
            "extras,",
            "notes,",
        ],
        "tabs page section order",
    );
}

#[test]
fn tabs_snippets_stay_copyable_and_source_aligned() {
    let demo = read("src/ui/snippets/tabs/demo.rs");
    let demo_normalized = normalized(&demo);
    let usage = read("src/ui/snippets/tabs/usage.rs");
    let usage_normalized = normalized(&usage);
    let line = read("src/ui/snippets/tabs/line.rs");
    let vertical = read("src/ui/snippets/tabs/vertical.rs");
    let disabled = read("src/ui/snippets/tabs/disabled.rs");
    let icons = read("src/ui/snippets/tabs/icons.rs");
    let list = read("src/ui/snippets/tabs/list.rs");

    for needle in [
        "shadcn::tabs_uncontrolled(cx, Some(\"account\"), |_cx|",
        "shadcn::TabsItem::new(\"account\", \"Account\", [account_card])",
        "shadcn::TabsItem::new(\"password\", \"Password\", [password_card])",
        "LayoutRefinement::default().w_full().max_w(Px(384.0)).min_w_0()",
        "shadcn::Button::new(\"Save changes\")",
        "shadcn::Button::new(\"Save password\")",
    ] {
        assert!(
            demo.contains(needle) || demo_normalized.contains(&normalized(needle)),
            "tabs demo snippet should mirror the current tabs-demo card shape; missing `{needle}`",
        );
    }

    for needle in [
        "shadcn::tabs_uncontrolled(cx, Some(\"account\"), |cx|",
        "decl_text::text_paragraph(cx, \"Make changes to your account here.\")",
        "decl_text::text_paragraph(cx, \"Change your password here.\")",
        "LayoutRefinement::default().w_px(Px(400.0)).min_w_0()",
    ] {
        assert!(
            usage.contains(needle) || usage_normalized.contains(&normalized(needle)),
            "tabs usage snippet should keep the current docs usage lane; missing `{needle}`",
        );
    }

    for needle in [
        "TabsItem::new(\"overview\", \"Overview\", Vec::<AnyElement>::new())",
        "TabsItem::new(\"analytics\", \"Analytics\", Vec::<AnyElement>::new())",
        "TabsItem::new(\"reports\", \"Reports\", Vec::<AnyElement>::new())",
        ".list_variant(shadcn::TabsListVariant::Line)",
    ] {
        assert!(
            line.contains(needle),
            "tabs line snippet should keep the Base/Radix line example lane; missing `{needle}`",
        );
    }

    for needle in [
        "TabsItem::new(\"account\", \"Account\", Vec::<AnyElement>::new())",
        "TabsItem::new(\"password\", \"Password\", Vec::<AnyElement>::new())",
        "TabsItem::new(\"notifications\", \"Notifications\", Vec::<AnyElement>::new())",
        ".orientation(shadcn::TabsOrientation::Vertical)",
    ] {
        assert!(
            vertical.contains(needle),
            "tabs vertical snippet should keep the Base/Radix vertical example lane; missing `{needle}`",
        );
    }

    for needle in [
        "TabsItem::new(\"home\", \"Home\", Vec::<AnyElement>::new())",
        "TabsItem::new(\"settings\", \"Disabled\", Vec::<AnyElement>::new()).disabled(true)",
    ] {
        assert!(
            disabled.contains(needle),
            "tabs disabled snippet should keep the Base/Radix disabled example lane; missing `{needle}`",
        );
    }

    for needle in [
        ".trigger_children([",
        "icon::icon(cx, IconId::new_static(\"lucide.app-window\"))",
        "decl_text::text_button_label(cx, \"Preview\")",
        "icon::icon(cx, IconId::new_static(\"lucide.code\"))",
        "decl_text::text_button_label(cx, \"Code\")",
    ] {
        assert!(
            icons.contains(needle),
            "tabs icons snippet should keep icon + label trigger children copyable; missing `{needle}`",
        );
    }

    for needle in [
        "TabsItem::new(\"home\", \"Home\", Vec::<AnyElement>::new())",
        "TabsItem::new(\"settings\", \"Settings\", Vec::<AnyElement>::new())",
    ] {
        assert!(
            list.contains(needle),
            "tabs list snippet should keep the Base/Radix Basic list lane; missing `{needle}`",
        );
    }
}

#[test]
fn tabs_diag_scripts_cover_current_state_depth_and_rtl_ids() {
    let docs_smoke =
        read("../../tools/diag-scripts/ui-gallery/tabs/ui-gallery-tabs-docs-smoke.json");
    let relation = read(
        "../../tools/diag-scripts/ui-gallery/tabs/ui-gallery-tabs-demo-relation-action-state.json",
    );
    let selected = read(
        "../../tools/diag-scripts/ui-gallery/tabs/ui-gallery-tabs-selected-state-mutation.json",
    );
    let rtl =
        read("../../tools/diag-scripts/ui-gallery/tabs/ui-gallery-tabs-rtl-keynav-screenshot.json");
    let vertical =
        read("../../tools/diag-scripts/ui-gallery/tabs/ui-gallery-tabs-vertical-list-grows.json");

    for needle in [
        "ui-gallery-tabs-demo-content",
        "ui-gallery-tabs-usage-tabs-trigger-code",
        "ui-gallery-tabs-api-reference-content",
        "ui-gallery-tabs-parts-content",
        "ui-gallery-tabs-extras-content",
    ] {
        assert!(
            docs_smoke.contains(needle),
            "tabs docs smoke script should cover docs and follow-up anchors; missing `{needle}`",
        );
    }

    for needle in [
        "ui-gallery-tabs-demo-list",
        "ui-gallery-tabs-demo-trigger-account",
        "ui-gallery-tabs-demo-trigger-password",
        "ui-gallery-tabs-demo-panel-account",
        "ui-gallery-tabs-demo-panel-password",
        "semantics_relation_includes",
    ] {
        assert!(
            relation.contains(needle) && selected.contains("selected_is"),
            "tabs relation/selection scripts should cover tab state and panel relations; missing `{needle}`",
        );
    }

    for needle in [
        "ui-gallery-tabs-rtl-trigger-overview",
        "ui-gallery-tabs-rtl-panel-overview",
        "ui-gallery-tabs-rtl-panel-analytics",
    ] {
        assert!(
            rtl.contains(needle),
            "tabs RTL script should follow the current four-tab RTL snippet ids; missing `{needle}`",
        );
    }

    for stale in [
        "ui-gallery-tabs-rtl-trigger-preview",
        "ui-gallery-tabs-rtl-panel-preview",
        "ui-gallery-tabs-rtl-panel-code",
    ] {
        assert!(
            !rtl.contains(stale),
            "tabs RTL script should not reference stale two-tab ids; found `{stale}`",
        );
    }

    for needle in ["ui-gallery-tabs-vertical", "ui-gallery-tabs-vertical-line"] {
        assert!(
            vertical.contains(needle),
            "tabs vertical diagnostics should cover vertical and vertical-line follow-ups; missing `{needle}`",
        );
    }
}
