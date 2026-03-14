use super::super::*;
use fret::UiCx;

use crate::ui::doc_layout::{self, DocSection};
use crate::ui::snippets::select as snippets;

pub(super) fn preview_select(cx: &mut UiCx<'_>) -> Vec<AnyElement> {
    let demo = {
        // A minimal shadcn-aligned demo (matches upstream `select-demo.tsx` example).
        let shadcn_demo = snippets::demo::render(cx);

        ui::v_flex(|_cx| vec![shadcn_demo])
            .gap(Space::N2)
            .items_start()
            .layout(LayoutRefinement::default().w_full().min_w_0())
            .into_element(cx)
            .test_id("ui-gallery-select-demo")
    };
    let label = snippets::label::render(cx);
    let diag_surface = snippets::diag_surface::render(cx);
    let align_item = snippets::align_item_with_trigger::render(cx);
    let groups = snippets::groups::render(cx);
    let scrollable = snippets::scrollable::render(cx);
    let disabled = snippets::disabled::render(cx);
    let invalid = snippets::invalid::render(cx);
    let rtl = snippets::rtl::render(cx);

    let notes = doc_layout::notes_block([
        "Select is a Popover + Listbox recipe. Use it for rich overlays and custom interactions.",
        "`Select::new(...)` / `new_controllable(...)` plus the direct builder chain (`.trigger(...).value(...).content(...).entries(...)`) are now the default copyable root story; `into_element_parts(...)` remains the focused upstream-shaped adapter on the same lane rather than a separate `compose()` story.",
        "This page keeps stable `test_id`s for `tools/diag-scripts/ui-gallery/select/*`.",
        "Use `SelectTriggerSize::Sm` to match compact shadcn control heights.",
    ]);
    let notes =
        DocSection::build(cx, "Notes", notes).description("Parity notes and usage guidance.");
    let demo = DocSection::build(cx, "Demo", demo)
        .description("Minimal shadcn-aligned demo (matches upstream `select-demo.tsx`).")
        .test_id_prefix("ui-gallery-select-demo")
        .code_rust_from_file_region(snippets::demo::SOURCE, "example");
    let label = DocSection::build(cx, "Label Association", label)
        .description("Use `FieldLabel::for_control` + `Select::control_id` so label clicks route to the trigger and open the popup.")
        .test_id_prefix("ui-gallery-select-label")
        .code_rust_from_file_region(snippets::label::SOURCE, "example");
    let diag_surface = DocSection::build(cx, "Diag Surface", diag_surface)
        .description("Long-list surface with stable test_ids used by diagnostics scripts.")
        .test_id_prefix("ui-gallery-select-diag-surface")
        .code_rust_from_file_region(snippets::diag_surface::SOURCE, "example");
    let align_item = DocSection::build(cx, "Align Item With Trigger", align_item)
        .description("Toggle between item-aligned and popper-style positioning.")
        .test_id_prefix("ui-gallery-select-align-item")
        .code_rust_from_file_region(snippets::align_item_with_trigger::SOURCE, "example");
    let groups = DocSection::build(cx, "Groups", groups)
        .description("Group labels + separator patterns used by shadcn Select.")
        .test_id_prefix("ui-gallery-select-groups")
        .code_rust_from_file_region(snippets::groups::SOURCE, "example");
    let scrollable = DocSection::build(cx, "Scrollable", scrollable)
        .description("Long lists should clamp height and expose scroll affordances.")
        .test_id_prefix("ui-gallery-select-scrollable")
        .code_rust_from_file_region(snippets::scrollable::SOURCE, "example");
    let disabled = DocSection::build(cx, "Disabled", disabled)
        .description("Disabled state should block open/selection and use muted styling.")
        .test_id_prefix("ui-gallery-select-disabled")
        .code_rust_from_file_region(snippets::disabled::SOURCE, "example");
    let invalid = DocSection::build(cx, "Invalid", invalid)
        .description("Invalid styling is typically shown with a Field + error message.")
        .test_id_prefix("ui-gallery-select-invalid")
        .code_rust_from_file_region(snippets::invalid::SOURCE, "example");
    let rtl = DocSection::build(cx, "RTL", rtl)
        .description("All shadcn components should work under an RTL direction provider.")
        .test_id_prefix("ui-gallery-select-rtl")
        .code_rust_from_file_region(snippets::rtl::SOURCE, "example");

    let body = doc_layout::render_doc_page(
        cx,
        Some(
            "Preview follows upstream shadcn Select docs and includes extra probes for parity work (positioning, groups/separators, long-list scrolling, disabled/invalid, RTL).",
        ),
        vec![
            demo,
            label,
            diag_surface,
            align_item,
            groups,
            scrollable,
            disabled,
            invalid,
            rtl,
            notes,
        ],
    );

    vec![body.test_id("ui-gallery-select").into_element(cx)]
}
