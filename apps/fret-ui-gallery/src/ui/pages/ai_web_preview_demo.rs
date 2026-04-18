use super::super::*;

use crate::ui::doc_layout::{self, DocSection};
use crate::ui::snippets::ai as snippets;
use fret::{AppComponentCx, UiChild};

fn parts_table(cx: &mut AppComponentCx<'_>) -> impl UiChild + use<> {
    doc_layout::text_table(
        cx,
        ["Part", "Method", "Description"],
        [
            [
                "WebPreview",
                "navigation / body / console / children / default_url / url_model / on_url_change / backend / into_element_with_children",
                "Root provider for shared URL + console state. `navigation(...)`, `body(...)`, and `console(...)` are the common docs-shaped lane; `children([...])` and `into_element_with_children(...)` remain the lower-level eager/lazy escape hatches; `backend(...)` stays optional and app-owned.",
            ],
            [
                "WebPreviewChild / WebPreviewNavigationChild",
                "typed child enums",
                "Advanced eager composition seam. Keeps context-dependent parts such as `WebPreviewUrl` unlanded until the root provider scope exists when the builder shortcuts are not enough.",
            ],
            [
                "WebPreviewNavigation",
                "default / button / url / children / test_id",
                "Navigation row wrapper for compound parts such as buttons and the URL field. Builder shortcuts cover the common docs-shaped lane without explicit child enums.",
            ],
            [
                "WebPreviewNavigationButton",
                "go_back / go_forward / reload / tooltip / action / on_activate / backend_action / test_id",
                "Ghost button chrome aligned with AI Elements. The semantic helpers keep caller-owned visual children while prewiring default tooltip text and, when available, the matching backend action.",
            ],
            [
                "WebPreviewUrl",
                "placeholder / test_id / refine_layout",
                "Context-backed URL input that commits on submit while keeping draft text local.",
            ],
            [
                "WebPreviewBody",
                "child / loading / test_id",
                "Main preview area. Without a backend it can host caller-owned placeholder content; with `webview` it becomes the registered native surface slot.",
            ],
            [
                "WebPreviewConsole",
                "logs / children / backend_logs / test_id_*",
                "Collapsible console disclosure with caller-owned footer content and optional backend log bridging.",
            ],
        ],
        true,
    )
}

pub(super) fn preview_ai_web_preview_demo(
    cx: &mut AppComponentCx<'_>,
    _theme: &Theme,
) -> Vec<AnyElement> {
    let usage = snippets::web_preview_demo::render(cx);
    let composable = snippets::web_preview_demo::render_composable_children(cx);
    let features = doc_layout::notes_block([
        "The compound parts surface now stays closer to the official AI Elements docs: `WebPreview` owns shared URL + console state, while navigation, URL input, body, and console remain composable child parts.",
        "The main usage snippet now uses `navigation(...)`, `body(...)`, and `console(...)`, so common docs-shaped compositions no longer need explicit child enums or the lower-level `into_element_with_children(cx, ...)` root closure.",
        "Navigation buttons now also expose `go_back(...)`, `go_forward(...)`, and `reload(...)`, so common browser-style controls can keep custom icon/glyph children without restating the semantic label wiring.",
        "UI Gallery still keeps the lightweight history markers and stable `test_id` selectors required by the promoted diagnostics scripts.",
        "Live page embedding remains app-owned in Fret: chrome is always available, while native preview + console bridging live behind the optional `webview` backend.",
    ]);
    let notes = doc_layout::notes_block([
        "Layering conclusion: this is still a policy-level `fret-ui-ai` surface, not a new `crates/fret-ui` mechanism contract.",
        "This page mirrors the official AI Elements Web Preview docs after skipping Installation and the v0 SDK backend route example. `Composable Children (Fret)` and backend notes stay as explicit Fret follow-ups.",
        "This detail page is feature-gated behind `gallery-dev`, which also enables the `fret-ui-ai` surfaces in UI Gallery.",
        "Because Fret's element tree is move-only, the builder shortcuts land on typed child enums internally instead of raw DOM-style children passthrough.",
        "Keep `ui-ai-web-preview-demo-*` selectors stable; the existing commit/navigation/console diagnostics scripts depend on them.",
    ]);
    let parts = parts_table(cx);

    let body = crate::ui::doc_layout::render_doc_page_after(
        Some(
            "Composable Web Preview chrome aligned with AI Elements: docs-shaped usage, eager compound children, and the current Fret backend ownership notes for native builds.",
        ),
        vec![
            DocSection::build(cx, "Usage", usage)
                .test_id_prefix("ui-gallery-ai-web-preview-demo")
                .description(
                    "Docs-aligned preview surface with URL commit, simple history markers, and console disclosure selectors for diagnostics.",
                )
                .code_rust_from_file_region(snippets::web_preview_demo::SOURCE, "example"),
            DocSection::build(cx, "Features", features)
                .test_id_prefix("ui-gallery-ai-web-preview-features")
                .description("High-signal parity notes against the official AI Elements docs surface.")
                .no_shell(),
            DocSection::build(cx, "Composable Children (Fret)", composable)
                .test_id_prefix("ui-gallery-ai-web-preview-composable")
                .description(
                    "Default builder shortcuts plus caller-owned body/footer content, without dropping to explicit child enums or a lazy root closure.",
                )
                .code_rust_from_file_region(
                    snippets::web_preview_demo::SOURCE,
                    "composable_children",
                ),
            DocSection::build(cx, "Parts & Props", parts)
                .test_id_prefix("ui-gallery-ai-web-preview-parts")
                .description(
                    "Current Fret builder surface that maps the upstream JSX compound parts to move-only element composition.",
                )
                .no_shell(),
            DocSection::build(cx, "Notes", notes)
                .test_id_prefix("ui-gallery-ai-web-preview-notes")
                .description("Layering, docs-surface parity, feature-gate context, and diagnostics anchors.")
                .no_shell(),
        ],
        cx,
    );

    vec![body.into_element(cx)]
}
