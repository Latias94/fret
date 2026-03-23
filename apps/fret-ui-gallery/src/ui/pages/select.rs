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
    let usage = snippets::usage::render(cx);
    let label = snippets::label::render(cx);
    let field_association = snippets::field_association::render(cx);
    let diag_surface = snippets::diag_surface::render(cx);
    let parts = snippets::parts::render(cx);
    let rich_items = snippets::rich_items::render(cx);
    let align_item = snippets::align_item_with_trigger::render(cx);
    let groups = snippets::groups::render(cx);
    let scrollable = snippets::scrollable::render(cx);
    let disabled = snippets::disabled::render(cx);
    let invalid = snippets::invalid::render(cx);
    let rtl = snippets::rtl::render(cx);

    let api_reference = doc_layout::notes_block([
        "`Select::new(...)` / `new_controllable(...)` plus the direct builder chain (`.trigger(...).value(...).content(...).entries(...)`) stay the default copyable root story.",
        "`Select::into_element_parts(...)` plus `SelectContent::with_entries(...)` is the current typed equivalent of the upstream nested `SelectTrigger` / `SelectValue` / `SelectContent` children lane.",
        "That composable seam should stay typed and narrow around `SelectEntry` (`SelectGroup` / `SelectItem` / `SelectLabel` / `SelectSeparator`) instead of widening `Select` to arbitrary generic children.",
        "`Select` remains a single-select text-keyed surface today; Base UI-style object values and multi-select remain separate public-surface work rather than a recipe/mechanism bug.",
        "Width negotiation remains caller-owned at the trigger/root call site; overlay placement, dismissal, scroll buttons, and listbox semantics stay recipe/mechanism-owned in `fret-ui-kit` + `fret-ui-shadcn`.",
        "No new mechanism bug was identified in this pass; the remaining drift was first-party docs discoverability around the already-landed parts surface.",
    ]);
    let notes = doc_layout::notes_block([
        "Select is a Popover + Listbox recipe. Use it for rich overlays and custom interactions.",
        "Gallery order now mirrors the upstream shadcn/Base UI Select docs path first: `Demo`, `Usage`, `Align Item With Trigger`, `Groups`, `Scrollable`, `Disabled`, `Invalid`, `RTL`, and `API Reference`.",
        "`Select::new(...)` / `new_controllable(...)` plus the direct builder chain (`.trigger(...).value(...).content(...).entries(...)`) are now the default copyable root story; `into_element_parts(...)` remains the focused upstream-shaped adapter on the same lane rather than a separate `compose()` story.",
        "`Composable Parts (Fret)` now makes that nested lane explicit with a full copyable example, so app authors do not need a broader generic children API just to match upstream docs ergonomics.",
        "`Rich Items (Fret)` documents the current typed answer to richer `SelectItemText` content: use styled text runs for value + secondary label style instead of widening items to arbitrary child trees.",
        "Base UI's `multiple` and object-value examples are not blocked on Select overlay mechanics; they would require a separate public-surface expansion beyond the current `Model<Option<Arc<str>>>` contract, so prefer `Combobox` for multi-select today.",
        "Standalone label wiring still uses `FieldLabel::for_control(...)` + `Select::control_id(...)`; inside `Field::build(...)`, Select can inherit the field-local label/description association automatically.",
        "`Composable Parts (Fret)`, `Rich Items (Fret)`, `Label Association`, `Field Builder Association`, and `Diag Surface` stay after `API Reference` as explicit Fret follow-ups so the docs path stays clean without sacrificing existing scripted coverage.",
        "This page keeps stable `test_id`s for `tools/diag-scripts/ui-gallery/select/*`.",
        "Use `SelectTriggerSize::Sm` to match compact shadcn control heights.",
    ]);
    let api_reference = DocSection::build(cx, "API Reference", api_reference)
        .no_shell()
        .test_id_prefix("ui-gallery-select-api-reference")
        .description(
            "Public surface summary, ownership notes, and the current parts/children guidance.",
        );
    let parts = DocSection::build(cx, "Composable Parts (Fret)", parts)
        .test_id_prefix("ui-gallery-select-composable-parts")
        .description(
            "Typed equivalent of the upstream nested `SelectTrigger` / `SelectValue` / `SelectContent` lane.",
        )
        .code_rust_from_file_region(snippets::parts::SOURCE, "example");
    let rich_items = DocSection::build(cx, "Rich Items (Fret)", rich_items)
        .test_id_prefix("ui-gallery-select-rich-items")
        .description(
            "Use `SelectItemText` + `SelectTextRun` for richer item rows without widening Select to arbitrary generic children.",
        )
        .code_rust_from_file_region(snippets::rich_items::SOURCE, "example");
    let notes =
        DocSection::build(cx, "Notes", notes).description("Parity notes and usage guidance.");
    let demo = DocSection::build(cx, "Demo", demo)
        .description("Minimal shadcn-aligned demo (matches upstream `select-demo.tsx`).")
        .test_id_prefix("ui-gallery-select-demo")
        .code_rust_from_file_region(snippets::demo::SOURCE, "example");
    let usage = DocSection::build(cx, "Usage", usage)
        .description("Copyable minimal usage for the default Select root lane.")
        .test_id_prefix("ui-gallery-select-usage")
        .code_rust_from_file_region(snippets::usage::SOURCE, "example");
    let label = DocSection::build(cx, "Label Association", label)
        .description("Use `FieldLabel::for_control` + `Select::control_id` so label clicks route to the trigger and open the popup.")
        .test_id_prefix("ui-gallery-select-label")
        .code_rust_from_file_region(snippets::label::SOURCE, "example");
    let field_association = DocSection::build(cx, "Field Builder Association", field_association)
        .description("Inside `Field::build(...)`, Select can inherit the field-local label + description association without explicit ids.")
        .test_id_prefix("ui-gallery-select-field-association")
        .code_rust_from_file_region(snippets::field_association::SOURCE, "example");
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
            "Preview mirrors the upstream shadcn/Base UI Select docs path first, then keeps typed parts composition, label wiring, and diagnostics-specific surfaces as explicit Fret follow-ups.",
        ),
        vec![
            demo,
            usage,
            align_item,
            groups,
            scrollable,
            disabled,
            invalid,
            rtl,
            api_reference,
            parts,
            rich_items,
            label,
            field_association,
            diag_surface,
            notes,
        ],
    );

    vec![body.test_id("ui-gallery-select").into_element(cx)]
}
