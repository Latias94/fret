use super::super::super::super::*;

pub(in crate::ui) fn preview_select(cx: &mut ElementContext<'_, App>) -> Vec<AnyElement> {
    use crate::ui::doc_layout::{self, DocSection};

    let demo = {
        // A minimal shadcn-aligned demo (matches the upstream `select-demo.tsx` example).
        let shadcn_demo = crate::ui::snippets::select::demo::render(cx);

        stack::vstack(
            cx,
            stack::VStackProps::default()
                .gap(Space::N2)
                .items_start()
                .layout(LayoutRefinement::default().w_full().min_w_0()),
            |_cx| vec![shadcn_demo],
        )
        .test_id("ui-gallery-select-demo")
    };

    let diag_surface = crate::ui::snippets::select::diag_surface::render(cx);

    let align_item = crate::ui::snippets::select::align_item_with_trigger::render(cx);

    let groups = crate::ui::snippets::select::groups::render(cx);

    let scrollable = crate::ui::snippets::select::scrollable::render(cx);

    let disabled = crate::ui::snippets::select::disabled::render(cx);

    let invalid = crate::ui::snippets::select::invalid::render(cx);

    let rtl = crate::ui::snippets::select::rtl::render(cx);

    let body = doc_layout::render_doc_page(
        cx,
        Some(
            "Upstream shadcn Select docs cover Demo and Scrollable. This page also includes extra probes (positioning modes, groups/separators, disabled/invalid styling, and RTL) plus a long-list surface used by diag scripts.",
        ),
        vec![
            DocSection::new("Demo", demo)
                .description(
                    "Minimal shadcn-aligned demo (matches the upstream `select-demo.tsx` example).",
                )
                .test_id_prefix("ui-gallery-select-demo")
                .code_from_file("rust", include_str!("../../../snippets/select/demo.rs")),
            DocSection::new("Diag Surface", diag_surface)
                .description(
                    "Long-list surface with stable test_ids used by UI diagnostics scripts.",
                )
                .test_id_prefix("ui-gallery-select-diag-surface")
                .code_from_file(
                    "rust",
                    include_str!("../../../snippets/select/diag_surface.rs"),
                )
                .max_w(Px(540.0)),
            DocSection::new("Align Item With Trigger", align_item)
                .description(
                    "Toggle between item-aligned positioning and popper-style positioning.",
                )
                .test_id_prefix("ui-gallery-select-align-item")
                .code_from_file(
                    "rust",
                    include_str!("../../../snippets/select/align_item_with_trigger.rs"),
                )
                .max_w(Px(540.0)),
            DocSection::new("Groups", groups)
                .description("Group labels + separator patterns used by shadcn Select.")
                .test_id_prefix("ui-gallery-select-groups")
                .code_from_file("rust", include_str!("../../../snippets/select/groups.rs"))
                .max_w(Px(540.0)),
            DocSection::new("Scrollable", scrollable)
                .description("Long lists should clamp height and expose scroll affordances.")
                .test_id_prefix("ui-gallery-select-scrollable")
                .code_from_file(
                    "rust",
                    include_str!("../../../snippets/select/scrollable.rs"),
                )
                .max_w(Px(620.0)),
            DocSection::new("Disabled", disabled)
                .description("Disabled state should block open + selection and use muted styling.")
                .test_id_prefix("ui-gallery-select-disabled")
                .code_from_file("rust", include_str!("../../../snippets/select/disabled.rs"))
                .max_w(Px(540.0)),
            DocSection::new("Invalid", invalid)
                .test_id_prefix("ui-gallery-select-invalid")
                .description("Invalid styling is typically shown with a Field + error message.")
                .code_from_file("rust", include_str!("../../../snippets/select/invalid.rs"))
                .max_w(Px(620.0)),
            DocSection::new("RTL", rtl)
                .test_id_prefix("ui-gallery-select-rtl")
                .description("All shadcn components should work under an RTL direction provider.")
                .code_from_file("rust", include_str!("../../../snippets/select/rtl.rs"))
                .max_w(Px(620.0)),
        ],
    );

    vec![body]
}
