mod support;

use support::{assert_default_app_surface, manifest_path, read_path};

fn assert_normalized_markers_present(relative_path: &str, required_markers: &[&str]) -> String {
    let path = manifest_path(relative_path);
    let source = read_path(&path);
    let normalized = source.split_whitespace().collect::<String>();

    for marker in required_markers {
        let marker = marker.split_whitespace().collect::<String>();
        assert!(
            normalized.contains(&marker),
            "{} is missing marker `{}`",
            path.display(),
            marker
        );
    }

    normalized
}

#[test]
fn card_rich_description_snippet_prefers_copyable_card_description_children_helper() {
    let path = manifest_path("src/ui/snippets/card/description_children.rs");
    let source = read_path(&path);
    assert_default_app_surface(
        &path,
        &source,
        &[
            "use fret::{UiChild, UiCx};",
            "pub fn render(cx: &mut UiCx<'_>) -> impl UiChild + use<>",
        ],
        "app-facing snippet surface",
    );

    let normalized = source.split_whitespace().collect::<String>();
    for marker in [
        "shadcn::card_description_children(|cx|",
        "cx.styled_text(rich_description_text())",
        "icon::icon(cx, IconId::new_static(\"lucide.info\"))",
    ] {
        let marker = marker.split_whitespace().collect::<String>();
        assert!(
            normalized.contains(&marker),
            "{} is missing marker `{}`",
            path.display(),
            marker
        );
    }

    assert!(
        !normalized.contains("CardDescription::build("),
        "{} reintroduced the lower-level `CardDescription::build(...)` teaching surface",
        path.display()
    );
    assert!(
        !normalized.contains("CardDescription::new_children("),
        "{} should prefer the app-facing `shadcn::card_description_children(...)` helper",
        path.display()
    );
}

#[test]
fn card_page_exposes_rich_description_doc_section() {
    let normalized = assert_normalized_markers_present(
        "src/ui/pages/card.rs",
        &[
            "title: \"Rich Description (Fret)\",",
            "code_source: Some(snippets::description_children::SOURCE),",
            "DocSection::build(cx, \"Rich Description (Fret)\", rich_description)",
            "ui-gallery-card-section-rich-description",
            "snippets::description_children::SOURCE",
        ],
    );

    assert!(
        normalized.contains(
            "`RichTitle(Fret)`and`RichDescription(Fret)`keepthe`card_title_children(...)`/`card_description_children(...)`lanescopyable"
        ),
        "src/ui/pages/card.rs should explain why both rich text lanes stay on the copyable default app-facing surface"
    );
}
