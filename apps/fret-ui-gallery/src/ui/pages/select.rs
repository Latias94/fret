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

    let notes = doc_layout::notes(
        cx,
        [
            "Select is a Popover + Listbox recipe. Use it for rich overlays and custom interactions.",
            "This page keeps stable `test_id`s for `tools/diag-scripts/ui-gallery/select/*`.",
            "Use `SelectTriggerSize::Sm` to match compact shadcn control heights.",
        ],
    );

    let body = doc_layout::render_doc_page(
        cx,
        Some(
            "Preview follows upstream shadcn Select docs and includes extra probes for parity work (positioning, groups/separators, long-list scrolling, disabled/invalid, RTL).",
        ),
        vec![
            DocSection::new("Demo", demo)
                .description("Minimal shadcn-aligned demo (matches upstream `select-demo.tsx`).")
                .test_id_prefix("ui-gallery-select-demo")
                .code_rust_from_file_region(snippets::demo::SOURCE, "example"),
            DocSection::new("Label Association", label)
                .description("Use `FieldLabel::for_control` + `Select::control_id` so label clicks route to the trigger and open the popup.")
                .test_id_prefix("ui-gallery-select-label")
                .code_rust_from_file_region(snippets::label::SOURCE, "example"),
            DocSection::new("Diag Surface", diag_surface)
                .description("Long-list surface with stable test_ids used by diagnostics scripts.")
                .test_id_prefix("ui-gallery-select-diag-surface")
                .code_rust_from_file_region(snippets::diag_surface::SOURCE, "example"),
            DocSection::new("Align Item With Trigger", align_item)
                .description("Toggle between item-aligned and popper-style positioning.")
                .test_id_prefix("ui-gallery-select-align-item")
                .code_rust_from_file_region(snippets::align_item_with_trigger::SOURCE, "example"),
            DocSection::new("Groups", groups)
                .description("Group labels + separator patterns used by shadcn Select.")
                .test_id_prefix("ui-gallery-select-groups")
                .code_rust_from_file_region(snippets::groups::SOURCE, "example"),
            DocSection::new("Scrollable", scrollable)
                .description("Long lists should clamp height and expose scroll affordances.")
                .test_id_prefix("ui-gallery-select-scrollable")
                .code_rust_from_file_region(snippets::scrollable::SOURCE, "example"),
            DocSection::new("Disabled", disabled)
                .description("Disabled state should block open/selection and use muted styling.")
                .test_id_prefix("ui-gallery-select-disabled")
                .code_rust_from_file_region(snippets::disabled::SOURCE, "example"),
            DocSection::new("Invalid", invalid)
                .description("Invalid styling is typically shown with a Field + error message.")
                .test_id_prefix("ui-gallery-select-invalid")
                .code_rust_from_file_region(snippets::invalid::SOURCE, "example"),
            DocSection::new("RTL", rtl)
                .description("All shadcn components should work under an RTL direction provider.")
                .test_id_prefix("ui-gallery-select-rtl")
                .code_rust_from_file_region(snippets::rtl::SOURCE, "example"),
            DocSection::new("Notes", notes).description("Parity notes and usage guidance."),
        ],
    );

    vec![body.test_id("ui-gallery-select")]
}
