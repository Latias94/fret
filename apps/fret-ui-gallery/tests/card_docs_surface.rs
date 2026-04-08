fn normalize_ws(source: &str) -> String {
    source.split_whitespace().collect()
}

#[test]
fn card_page_documents_source_axes_and_children_api_decision() {
    let source = include_str!("../src/ui/pages/card.rs");

    for needle in [
        "Reference baseline: shadcn/base + shadcn/radix Card docs.",
        "Visual/chrome baseline: `repo-ref/ui/apps/v4/registry/new-york-v4/ui/card.tsx` plus the upstream `Demo`, `Size`, `Image`, and `RTL` examples.",
        "Base UI / Radix headless references do not add extra Card-specific interaction machinery here; the remaining drift is recipe/docs-surface work rather than a `fret-ui` mechanism bug.",
        "`Card`, `CardHeader`, `CardAction`, `CardContent`, and `CardFooter` already accept composable children via `...::new([...])` or the helper-family builders, so no extra generic root-level `children(...)` API is needed for shadcn parity.",
        "`CardTitle` and `CardDescription` keep compact text lanes by default, while `card_title_children(...)` / `card_description_children(...)` stay as the focused composable-children follow-ups instead of widening the whole family to a generic root `children(...)` / `compose()` API.",
        "Gallery order now mirrors the upstream Card docs path through `API Reference` before appending Fret-only regression sections.",
        "The `Image` and `Meeting Notes` snippets now keep their demo media self-contained with inline RGBA sources, so the code tabs stay copyable without UI Gallery-only asset helpers.",
        "DocSection::build(cx, \"API Reference\", api_reference)",
        "DocSection::build(cx, \"Rich Title (Fret)\", rich_title)",
        "DocSection::build(cx, \"Rich Description (Fret)\", rich_description)",
    ] {
        assert!(
            source.contains(needle),
            "card page should document source axes and the children-api decision; missing `{needle}`",
        );
    }

    let mut last_index = 0usize;
    for needle in [
        "DocSection::build(cx, \"Demo\", login)",
        "DocSection::build(cx, \"Usage\", usage)",
        "DocSection::build(cx, \"Size\", size)",
        "DocSection::build(cx, \"Image\", image)",
        "DocSection::build(cx, \"RTL\", rtl)",
        "DocSection::build(cx, \"API Reference\", api_reference)",
        "DocSection::build(cx, \"Rich Title (Fret)\", rich_title)",
        "DocSection::build(cx, \"Rich Description (Fret)\", rich_description)",
        "DocSection::build(cx, \"Compositions\", compositions)",
        "DocSection::build(cx, \"CardContent\", card_content_inline_button)",
        "DocSection::build(cx, \"Meeting Notes\", meeting_notes)",
        "DocSection::build(cx, \"Notes\", notes)",
    ] {
        let index = source
            .find(needle)
            .unwrap_or_else(|| panic!("missing ordered Card section marker `{needle}`"));
        assert!(
            index >= last_index,
            "card page should keep the docs-path sections before the explicit Fret follow-ups",
        );
        last_index = index;
    }
}

#[test]
fn card_docs_path_snippets_stay_copyable_and_docs_aligned() {
    let usage = include_str!("../src/ui/snippets/card/usage.rs");
    let demo = include_str!("../src/ui/snippets/card/demo.rs");
    let size = include_str!("../src/ui/snippets/card/size.rs");
    let image = include_str!("../src/ui/snippets/card/image.rs");
    let rtl = include_str!("../src/ui/snippets/card/rtl.rs");
    let rich_title = include_str!("../src/ui/snippets/card/title_children.rs");
    let rich_description = include_str!("../src/ui/snippets/card/description_children.rs");
    let meeting_notes = include_str!("../src/ui/snippets/card/meeting_notes.rs");

    for needle in [
        "use fret::{UiChild, UiCx};",
        "use fret_ui_shadcn::{facade as shadcn, prelude::*};",
        "shadcn::card(|cx| {",
        "shadcn::card_header(|cx| {",
        "shadcn::card_title(\"Card Title\")",
        "shadcn::card_action(",
        "shadcn::card_content(|cx|",
        "shadcn::card_footer(|cx|",
    ] {
        assert!(
            usage.contains(needle),
            "card usage snippet should stay on the default copyable docs-shaped lane; missing `{needle}`",
        );
    }

    for needle in [
        "shadcn::card_title(\"Login to your account\")",
        "shadcn::card_action(|cx| {",
        "LayoutRefinement::default()\n        .w_full()\n        .max_w(MetricRef::Px(Px(384.0)))\n        .min_w_0()",
        ".test_id(\"ui-gallery-card-demo-email-input\")",
        ".test_id(\"ui-gallery-card-demo-password-input\")",
        ".direction(shadcn::CardFooterDirection::Column)",
    ] {
        assert!(
            normalize_ws(demo).contains(&normalize_ws(needle)),
            "card demo snippet should keep the upstream-shaped demo shell and footer layout; missing `{needle}`",
        );
    }

    let normalized_demo = normalize_ws(demo);
    assert!(
        normalized_demo.contains(&normalize_ws(
            "shadcn::Button::new(\"Sign Up\") .variant(shadcn::ButtonVariant::Link) .ui() .test_id(\"ui-gallery-card-demo-sign-up\")"
        )),
        "card demo Sign Up action should stay on the upstream default link-button lane",
    );
    assert!(
        !normalized_demo.contains(&normalize_ws(
            "shadcn::Button::new(\"Sign Up\") .variant(shadcn::ButtonVariant::Link) .size(shadcn::ButtonSize::Sm)"
        )),
        "card demo Sign Up action should not downsize the upstream default link button",
    );
    assert!(
        !normalized_demo.contains(&normalize_ws(".placeholder(\"••••••••\")")),
        "card demo password input should stay aligned with the upstream example and omit a placeholder",
    );
    assert!(
        !normalize_ws(rtl).contains(&normalize_ws(".placeholder(\"••••••••\")")),
        "card rtl password input should stay aligned with the upstream translated example and omit a placeholder",
    );

    assert!(
        size.contains(".size(shadcn::CardSize::Sm)"),
        "card size snippet should keep the small-size variant visible on the copyable lane",
    );
    assert!(
        !size.contains(".text_sm()"),
        "card size snippet body should stay on the upstream default body-text lane",
    );

    for needle in [
        "use fret_ui_assets::ImageSource;",
        "use fret_ui_assets::ui::ImageSourceElementContextExt as _;",
        "ImageSource::rgba8(",
        "fn demo_cover_image(cx: &mut UiCx<'_>) -> Option<ImageId>",
        "A practical talk on component APIs, accessibility, and shipping faster.",
    ] {
        assert!(
            image.contains(needle),
            "card image snippet should stay self-contained and copyable; missing `{needle}`",
        );
    }
    assert!(
        !image.contains("self-contained RGBA source"),
        "card image snippet should keep copyability notes out of the upstream demo body text",
    );
    assert!(
        !image.contains("super::demo_cover_image"),
        "card image snippet should not depend on a sibling helper module anymore",
    );

    for needle in [
        "ImageSource::rgba8(",
        "fn demo_avatar_image(cx: &mut UiCx<'_>) -> Option<ImageId>",
        "AvatarImage::maybe(avatar_image)",
    ] {
        assert!(
            meeting_notes.contains(needle),
            "card meeting-notes snippet should keep its demo media self-contained; missing `{needle}`",
        );
    }
    assert!(
        !meeting_notes.contains("super::super::avatar::demo_image"),
        "card meeting-notes snippet should not depend on UI Gallery avatar helpers",
    );

    for needle in [
        "shadcn::card_title_children(|cx|",
        "shadcn::card_description_children(|cx|",
    ] {
        assert!(
            rich_title.contains("shadcn::card_title_children(|cx|")
                && rich_description.contains("shadcn::card_description_children(|cx|"),
            "card rich text follow-ups should keep the focused children helpers visible; missing `{needle}`",
        );
    }

    let combined = [
        usage,
        demo,
        size,
        image,
        rtl,
        rich_title,
        rich_description,
        meeting_notes,
    ]
    .join("\n");
    assert!(
        !combined.contains("compose()"),
        "card snippets should not introduce a generic compose() lane",
    );
}

#[test]
fn card_docs_diag_script_covers_docs_path_and_fret_followups() {
    let script =
        include_str!("../../../tools/diag-scripts/ui-gallery/card/ui-gallery-card-docs-smoke.json");
    let stub = include_str!("../../../tools/diag-scripts/ui-gallery-card-docs-smoke.json");
    let suite =
        include_str!("../../../tools/diag-scripts/suites/ui-gallery-shadcn-conformance/suite.json");

    for needle in [
        "\"ui-gallery-card-demo\"",
        "\"ui-gallery-card-demo-email-input\"",
        "\"ui-gallery-card-demo-password-input\"",
        "\"ui-gallery-card-usage\"",
        "\"ui-gallery-card-size\"",
        "\"ui-gallery-card-image\"",
        "\"ui-gallery-card-rtl\"",
        "\"ui-gallery-card-api-reference-content\"",
        "\"ui-gallery-card-title-children\"",
        "\"ui-gallery-card-description-children\"",
        "\"ui-gallery-card-compositions\"",
        "\"ui-gallery-card-content-inline-button-demo\"",
        "\"ui-gallery-card-meeting-notes\"",
        "\"ui-gallery-card-section-notes-content\"",
        "\"ui-gallery-card-docs-smoke\"",
    ] {
        assert!(
            script.contains(needle),
            "card docs diag script should cover the docs path and explicit Fret follow-ups; missing `{needle}`",
        );
    }

    assert!(
        stub.contains(
            "\"to\": \"tools/diag-scripts/ui-gallery/card/ui-gallery-card-docs-smoke.json\""
        ),
        "card docs smoke redirect stub should point at the canonical card script",
    );
    assert!(
        suite.contains("\"tools/diag-scripts/ui-gallery-card-docs-smoke.json\""),
        "card docs smoke script should stay promoted in the shadcn conformance suite",
    );
}
