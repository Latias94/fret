use super::super::*;

use crate::ui::doc_layout::{self, DocSection};
use crate::ui::snippets::ai as snippets;
use fret::{UiChild, UiCx};

pub(super) fn preview_ai_package_info_demo(cx: &mut UiCx<'_>, _theme: &Theme) -> Vec<AnyElement> {
    let usage = snippets::package_info_demo::render(cx);
    let custom_children = snippets::package_info_demo::render_custom_children(cx);

    let features = doc_layout::notes_block([
        "Version change display (`current_version -> new_version`) follows the official AI Elements `PackageInfoVersion` example.",
        "Change type badges keep the same major/minor/patch/added/removed mapping, with theme-token escape hatches on the background and foreground colors.",
        "Dependencies and description stay policy-level composition in `fret-ui-ai`; no `crates/fret-ui` mechanism change is required for this surface.",
        "The root and key parts now support docs-shaped compound children authoring, so Gallery snippets no longer need a closure-only `into_element_with_children(cx, ...)` path for the common case.",
    ]);

    let notes = doc_layout::notes_block([
        "Layering conclusion: this component was already visually close to upstream; the main drift was public authoring surface, not runtime semantics or default sizing policy.",
        "Keep `ui-ai-package-info-demo-badge-major` and `ui-ai-package-info-demo-version-major` stable; the promoted diagnostics scripts depend on them.",
        "This detail page is feature-gated behind `gallery-dev`, which also enables the `fret-ui-ai` demo surfaces in UI Gallery.",
        "Because Fret is GPU-first and move-only, React-style `children` becomes typed builder methods and stored child lists instead of raw DOM prop passthrough.",
    ]);
    let change_types = change_types_table(cx);
    let builder_surface = parts_table(cx);

    let body = crate::ui::doc_layout::render_doc_page_after(
        Some(
            "The `PackageInfo` component displays package dependency information including version changes, change type badges, and an optional dependencies block.",
        ),
        vec![
            DocSection::build(cx, "Usage", usage)
                .test_id_prefix("ui-gallery-ai-package-info-demo")
                .description(
                    "Docs-aligned Fret translation of the official AI Elements example, including description and dependencies.",
                )
                .code_rust_from_file_region(snippets::package_info_demo::SOURCE, "example"),
            DocSection::build(cx, "Composable Children", custom_children)
                .test_id_prefix("ui-gallery-ai-package-info-custom")
                .description(
                    "Override the inner label/content slots while keeping the default PackageInfo chrome and context wiring.",
                )
                .code_rust_from_file_region(
                    snippets::package_info_demo::SOURCE,
                    "custom_children",
                ),
            DocSection::build(cx, "Features", features)
                .description("High-signal parity notes against the official AI Elements docs surface.")
                .no_shell(),
            DocSection::build(cx, "Change Types", change_types)
                .description("Color intent and use cases carried over from the upstream docs.")
                .no_shell(),
            DocSection::build(cx, "Builder Surface", builder_surface)
                .description("Current Fret authoring surface for the `PackageInfo*` family.")
                .no_shell(),
            DocSection::build(cx, "Notes", notes)
                .description("Where the alignment work landed: policy layer, diagnostics hooks, and authoring-shape parity.")
                .no_shell()
                .test_id_prefix("ui-gallery-ai-package-info-notes"),
        ],
        cx,
    );

    vec![body.into_element(cx)]
}

fn change_types_table(cx: &mut UiCx<'_>) -> impl UiChild + use<> {
    doc_layout::text_table(
        cx,
        ["Type", "Color", "Use Case"],
        [
            ["major", "Red", "Breaking changes"],
            ["minor", "Yellow", "New features"],
            ["patch", "Green", "Bug fixes"],
            ["added", "Blue", "New dependency"],
            ["removed", "Gray", "Removed dependency"],
        ],
        false,
    )
}

fn parts_table(cx: &mut UiCx<'_>) -> impl UiChild + use<> {
    doc_layout::text_table(
        cx,
        ["Part", "Method", "Description"],
        [
            [
                "PackageInfo",
                "children / header / version / description / content",
                "Docs-shaped root composition surface aligned with upstream `<PackageInfo>...</PackageInfo>` authoring.",
            ],
            [
                "PackageInfo",
                "into_element_with_children",
                "Low-level escape hatch when move-only trees are easier to assemble lazily inside a live element scope.",
            ],
            [
                "PackageInfoHeader",
                "children",
                "Accepts typed `PackageInfoName` / `PackageInfoChangeType` parts or arbitrary landed elements.",
            ],
            [
                "PackageInfoName",
                "label / icon / children",
                "Uses context `name` by default, while `children([...])` overrides only the inner label slot.",
            ],
            [
                "PackageInfoChangeType",
                "children / test_id",
                "Uses context `change_type` by default and keeps the badge chrome/icons when overriding the visible label content.",
            ],
            [
                "PackageInfoVersion",
                "children / test_id",
                "Defaults to the official version transition row and supports custom inline content when needed.",
            ],
            [
                "PackageInfoDescription",
                "new(text) / children",
                "Paragraph-style description block; `children([...])` replaces the inner copy while keeping spacing and typography scope.",
            ],
            [
                "PackageInfoContent / Dependencies",
                "new(children) / children",
                "Container parts for the bordered content block and dependencies list.",
            ],
            [
                "PackageInfoDependency",
                "version / children",
                "Defaults to `name + version` row, or accepts custom row content for app-specific formatting.",
            ],
        ],
        true,
    )
}
