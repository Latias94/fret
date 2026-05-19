use super::super::*;
use crate::ui::doc_layout::{self, DocSection};
use crate::ui::snippets::resizable as snippets;
use fret::AppComponentCx;

const ENV_RESIZABLE_ADAPTIVE_PANEL: &str = "FRET_UI_GALLERY_RESIZABLE_ADAPTIVE_PANEL";
const ENV_RESIZABLE_MULTI_VIEWPORT_COMBOBOX: &str =
    "FRET_UI_GALLERY_RESIZABLE_MULTI_VIEWPORT_COMBOBOX";
const ENV_RESIZABLE_MULTI_VIEWPORT_SELECT: &str = "FRET_UI_GALLERY_RESIZABLE_MULTI_VIEWPORT_SELECT";
const ENV_RESIZABLE_MOVING_CACHED_COMBOBOX: &str =
    "FRET_UI_GALLERY_RESIZABLE_MOVING_CACHED_COMBOBOX";

fn should_build_resizable_section(
    focus_filters: Option<&[String]>,
    title: &'static str,
    test_id_prefix: &'static str,
) -> bool {
    focus_filters.is_none_or(|filters| {
        doc_layout::section_matches_focus_filter(title, None, None, Some(test_id_prefix), filters)
    })
}

pub(super) fn preview_resizable(cx: &mut AppComponentCx<'_>) -> Vec<AnyElement> {
    let focus_filters = doc_layout::section_focus_filters();
    let adaptive_panel_enabled =
        std::env::var_os(ENV_RESIZABLE_ADAPTIVE_PANEL).is_some_and(|value| !value.is_empty());
    let multi_viewport_combobox_enabled = std::env::var_os(ENV_RESIZABLE_MULTI_VIEWPORT_COMBOBOX)
        .is_some_and(|value| !value.is_empty());
    let multi_viewport_select_enabled = std::env::var_os(ENV_RESIZABLE_MULTI_VIEWPORT_SELECT)
        .is_some_and(|value| !value.is_empty());
    let moving_cached_combobox_enabled = std::env::var_os(ENV_RESIZABLE_MOVING_CACHED_COMBOBOX)
        .is_some_and(|value| !value.is_empty());
    let mut sections = Vec::new();

    if should_build_resizable_section(
        focus_filters.as_deref(),
        "Demo",
        "ui-gallery-resizable-demo",
    ) {
        let demo = snippets::demo::render(cx);
        sections.push(
            DocSection::build(cx, "Demo", demo)
                .description("Nested vertical panels inside a horizontal group.")
                .test_id_prefix("ui-gallery-resizable-demo")
                .code_rust_from_file_region(snippets::demo::SOURCE, "example"),
        );
    }
    if should_build_resizable_section(
        focus_filters.as_deref(),
        "About",
        "ui-gallery-resizable-about",
    ) {
        let about = doc_layout::notes_block([
            "Reference stack: shadcn Resizable docs on the Base UI and Radix lanes.",
            "The current visual/chrome baseline comes from the default shadcn registry recipe, with parallel headless baselines in the base and radix registry variants.",
            "Unlike `slider` or `progress`, there is no direct `Resizable` primitive in Radix Primitives or Base UI; those libraries still inform general headless/mechanism decisions, but the concrete source axis here is shadcn plus the runtime panel-group contract.",
            "This page is docs/public-surface parity work, not a mechanism-layer gap: drag routing, hit-testing, focusable splitter semantics, and min-size clamping already live in `fret-ui`.",
            "The Fret-only follow-ups below keep fixed-window splitter, viewport-root overlay ownership, and cached-source movement proofs diagnostics opt-in so `panel width`, `viewport width`, and cached overlay source ownership remain visibly distinct without bloating default docs smoke renders.",
        ]);
        sections.push(
            DocSection::build(cx, "About", about)
                .no_shell()
                .description(
                    "Source axes and why this component is already on the right runtime/mechanism split.",
                )
                .test_id_prefix("ui-gallery-resizable-about"),
        );
    }
    if should_build_resizable_section(
        focus_filters.as_deref(),
        "Usage",
        "ui-gallery-resizable-usage",
    ) {
        let usage = snippets::usage::render(cx);
        sections.push(
            DocSection::build(cx, "Usage", usage)
                .description(
                    "Copyable minimal usage for `resizable_panel_group(...)`, `ResizablePanel`, and `ResizableHandle`.",
                )
                .test_id_prefix("ui-gallery-resizable-usage")
                .code_rust_from_file_region(snippets::usage::SOURCE, "example"),
        );
    }
    if should_build_resizable_section(
        focus_filters.as_deref(),
        "Vertical",
        "ui-gallery-resizable-vertical",
    ) {
        let vertical = snippets::vertical::render(cx);
        sections.push(
            DocSection::build(cx, "Vertical", vertical)
                .description("Vertical orientation.")
                .test_id_prefix("ui-gallery-resizable-vertical")
                .code_rust_from_file_region(snippets::vertical::SOURCE, "example"),
        );
    }
    if should_build_resizable_section(
        focus_filters.as_deref(),
        "Handle",
        "ui-gallery-resizable-handle",
    ) {
        let handle = snippets::handle::render(cx);
        sections.push(
            DocSection::build(cx, "Handle", handle)
                .description("A handle with a visual grabber (`withHandle`).")
                .test_id_prefix("ui-gallery-resizable-handle")
                .code_rust_from_file_region(snippets::handle::SOURCE, "example"),
        );
    }
    if should_build_resizable_section(focus_filters.as_deref(), "RTL", "ui-gallery-resizable-rtl") {
        let rtl = snippets::rtl::render(cx);
        sections.push(
            DocSection::build(cx, "RTL", rtl)
                .description("Direction provider coverage for hit-testing and handle affordances.")
                .test_id_prefix("ui-gallery-resizable-rtl")
                .code_rust_from_file_region(snippets::rtl::SOURCE, "example"),
        );
    }
    if should_build_resizable_section(
        focus_filters.as_deref(),
        "API Reference",
        "ui-gallery-resizable-api-reference",
    ) {
        let api_reference = doc_layout::notes_block([
            "`ResizablePanelGroup::new(model).entries([...])` and `shadcn::resizable_panel_group(cx, model, |cx| ..)` cover the documented authoring surface.",
            "`resizable_panel_group(cx, model, |cx| ..)` is already the composable children-equivalent lane for Fret: it keeps `ResizablePanel` / `ResizableHandle` ordering explicit while preserving root-level `.axis(...)`, `.style(...)`, `.test_id_prefix(...)`, and layout refinements.",
            "A generic composable children / `compose()` API is not warranted here: the typed `ResizableEntry` stream already carries the source-aligned `Panel / Handle / Panel` contract without hiding handle order or widening the public surface.",
            "`ResizablePanelGroup` owns the upstream `w-full h-full` fill behavior plus handle chrome, while surrounding `rounded-lg border`, `max-w-*`, and fixed preview heights remain caller-owned like the shadcn docs/examples.",
            "`ResizableHandle::with_handle(true)` maps the documented visible-grabber lane, while keyboard splitter semantics and focus order remain runtime-owned.",
        ]);
        sections.push(
            DocSection::build(cx, "API Reference", api_reference)
                .no_shell()
                .description(
                    "Public surface summary, ownership notes, and the children-API decision.",
                )
                .test_id_prefix("ui-gallery-resizable-api-reference"),
        );
    }

    if adaptive_panel_enabled
        && should_build_resizable_section(
            focus_filters.as_deref(),
            "Adaptive Panel Proof",
            "ui-gallery-resizable-adaptive-panel-proof",
        )
    {
        let adaptive_panel = snippets::adaptive_panel::render(cx);
        let adaptive_panel = DocSection::build(cx, "Adaptive Panel Proof", adaptive_panel)
            .description(
                "Fixed-window splitter resize proving that the compact branch follows container width, not viewport width.",
            )
            .max_w(Px(1120.0))
            .test_id_prefix("ui-gallery-resizable-adaptive-panel-proof")
            .code_rust_from_file_region(snippets::adaptive_panel::SOURCE, "example");
        sections.push(adaptive_panel);
    }

    if multi_viewport_combobox_enabled
        && should_build_resizable_section(
            focus_filters.as_deref(),
            "Multi-Viewport Combobox",
            "ui-gallery-resizable-multi-viewport-combobox-docsec",
        )
    {
        let multi_viewport_combobox = snippets::multi_viewport_combobox::render(cx);
        let multi_viewport_combobox = DocSection::build(
            cx,
            "Multi-Viewport Combobox",
            multi_viewport_combobox,
        )
        .description(
            "Diagnostics fixture for anchored Combobox placement inside a Resizable panel viewport root.",
        )
        .max_w(Px(1120.0))
        .test_id_prefix("ui-gallery-resizable-multi-viewport-combobox-docsec")
        .no_shell()
        .code_rust_from_file_region(snippets::multi_viewport_combobox::SOURCE, "example");
        sections.push(multi_viewport_combobox);
    }

    if multi_viewport_select_enabled
        && should_build_resizable_section(
            focus_filters.as_deref(),
            "Multi-Viewport Select",
            "ui-gallery-resizable-multi-viewport-select-docsec",
        )
    {
        let multi_viewport_select = snippets::multi_viewport_select::render(cx);
        let multi_viewport_select =
            DocSection::build(cx, "Multi-Viewport Select", multi_viewport_select)
                .description(
                    "Diagnostics fixture for popper-positioned Select placement inside a Resizable panel viewport root.",
                )
                .max_w(Px(1120.0))
                .test_id_prefix("ui-gallery-resizable-multi-viewport-select-docsec")
                .no_shell()
                .code_rust_from_file_region(snippets::multi_viewport_select::SOURCE, "example");
        sections.push(multi_viewport_select);
    }

    if moving_cached_combobox_enabled
        && should_build_resizable_section(
            focus_filters.as_deref(),
            "Moving Cached Combobox",
            "ui-gallery-resizable-view-cache-moving-combobox-docsec",
        )
    {
        let moving_cached_combobox = snippets::moving_cached_combobox::render(cx);
        let moving_cached_combobox =
            DocSection::build(cx, "Moving Cached Combobox", moving_cached_combobox)
                .description(
                    "Diagnostics fixture for moving one cached Combobox source between Resizable panel viewport roots before reopening the overlay.",
                )
                .max_w(Px(1120.0))
                .test_id_prefix("ui-gallery-resizable-view-cache-moving-combobox-docsec")
                .no_shell();
        sections.push(moving_cached_combobox);
    }

    if should_build_resizable_section(
        focus_filters.as_deref(),
        "Notes",
        "ui-gallery-resizable-notes",
    ) {
        let notes = snippets::notes::render(cx);
        sections.push(
            DocSection::build(cx, "Notes", notes)
                .no_shell()
                .description("Remaining parity notes and diagnostics anchors.")
                .test_id_prefix("ui-gallery-resizable-notes"),
        );
    }

    let body = doc_layout::render_doc_page(
        cx,
        Some(
            "Preview mirrors the shadcn/Base UI Resizable docs path after collapsing the top `ComponentPreview` into `Demo` and skipping `Installation`: `Demo`, `About`, `Usage`, `Vertical`, `Handle`, `RTL`, and `API Reference`. `Adaptive Panel Proof`, `Multi-Viewport Combobox`, `Multi-Viewport Select`, and `Moving Cached Combobox` are diagnostics opt-in follow-ups for fixed-window container-query behavior, viewport-root overlay ownership, cross-family overlay ownership, and cached overlay-source movement before `Notes` closes on parity conclusions and diagnostics anchors.",
        ),
        sections,
    );

    let component = body.test_id("ui-gallery-resizable").into_element(cx);
    let page = ui::v_flex(move |_cx| vec![component])
        .layout(LayoutRefinement::default().w_full().min_w_0())
        .items_start();

    vec![page.into_element(cx)]
}
