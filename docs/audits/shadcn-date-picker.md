# shadcn/ui v4 Audit - Date Picker

## Upstream references (non-normative)

This document references optional local checkouts under `repo-ref/` for convenience.
Upstream sources:

- shadcn/ui: https://github.com/shadcn-ui/ui
- Radix Primitives: https://github.com/radix-ui/primitives
- Base UI: https://github.com/mui/base-ui

See `docs/repo-ref.md` for the optional local snapshot policy and pinned SHAs.

## Upstream references (source of truth)

- shadcn docs: `repo-ref/ui/apps/v4/content/docs/components/base/date-picker.mdx`
- shadcn demo example: `repo-ref/ui/apps/v4/examples/base/date-picker-demo.tsx`
- shadcn range example: `repo-ref/ui/apps/v4/examples/base/date-picker-range.tsx`
- shadcn date-of-birth example: `repo-ref/ui/apps/v4/examples/base/date-picker-dob.tsx`
- shadcn input example: `repo-ref/ui/apps/v4/examples/base/date-picker-input.tsx`
- shadcn time example: `repo-ref/ui/apps/v4/examples/base/date-picker-time.tsx`

## Fret implementation anchors

- Component code: `ecosystem/fret-ui-shadcn/src/date_picker.rs`
- Related family recipes:
  - `ecosystem/fret-ui-shadcn/src/date_range_picker.rs`
  - `ecosystem/fret-ui-shadcn/src/date_picker_with_presets.rs`
- Building blocks:
  - `ecosystem/fret-ui-shadcn/src/button.rs`
  - `ecosystem/fret-ui-shadcn/src/popover.rs`
  - `ecosystem/fret-ui-shadcn/src/calendar.rs`
  - `ecosystem/fret-ui-shadcn/src/calendar_range.rs`
  - `ecosystem/fret-ui-shadcn/src/select.rs`
- Gallery page: `apps/fret-ui-gallery/src/ui/pages/date_picker.rs`
- Copyable snippets:
  - `apps/fret-ui-gallery/src/ui/snippets/date_picker/usage.rs`
  - `apps/fret-ui-gallery/src/ui/snippets/date_picker/label.rs`
- Gallery gate: `apps/fret-ui-gallery/src/driver/render_flow.rs`

## Audit checklist

### Authoring surface

- Pass: `DatePicker::new(open, month, selected)` covers the compact single-date recipe path.
- Pass: `DatePicker::new_controllable(...)` keeps the common controlled/uncontrolled authoring path lightweight.
- Pass: Compact wrappers now default to the same open/close semantics as upstream shadcn docs examples: single/range/presets selections do not dismiss automatically unless the caller opts into `.close_on_select(true)` or wires a controlled `open` model on the composed surface.
- Pass: `placeholder(...)`, `format_selected_by(...)`, `format_selected_iso()`, `week_start(...)`, `control_id(...)`, `test_id_prefix(...)`, and disabled/outside-day controls cover the current docs/examples surface.
- Pass: `DateRangePicker` and `DatePickerWithPresets` now expose the same `control_id(...)` / `test_id_prefix(...)` convenience surface for form-control wiring and diagnostics parity across the family.
- Pass: The `Input` gallery snippet now spells the upstream `InputGroupButton` chrome explicitly (`ghost` + `icon-xs`) instead of relying on broader local defaults, so the copyable authoring surface stays aligned even if input-group defaults evolve.
- Pass: The compact builder does not need a generic children API; upstream docs are still a recipe over `Popover + Calendar`, not an open-ended slot surface.

### Default-style ownership

- Pass: Trigger chrome belongs to the recipe (`outline`, calendar icon, `justify-start`, `font-normal`, placeholder muted foreground).
- Pass: Trigger width is caller-owned. Upstream examples put width decisions on the call site (`w-[212px]`, `w-32`, field/container classes), so Fret should not bake `w_full()` into `DatePicker` by default.
- Pass: The same caller-owned width rule also applies to `DateRangePicker` and `DatePickerWithPresets`; their default trigger builders should stay intrinsic unless the caller opts into `w_full()` or a fixed width.
- Pass: `PopoverContent` keeping `w-auto p-0` inside the compact builder is still recipe-owned, because upstream source places that on the date-picker composition itself rather than on page shells.
- Pass: Form/gallery call sites that want stretch behavior now opt in explicitly with `refine_layout(LayoutRefinement::default().w_full())`.

### Overlay composition (Popover + Calendar)

- Pass: Date picker remains a `Popover` + `Calendar` composition, matching shadcn docs structure.
- Pass: Calendar focus handoff on open is covered by a component unit test.
- Pass: Gallery docs order follows the upstream docs flow (`Demo -> Usage -> Basic -> Range -> Date of birth -> Input -> Time -> Natural language -> RTL`), then appends `With Presets` as the upstream registry follow-up and keeps `Label Association` / `With Dropdowns` as focused Fret-gallery follow-ups after the main path.

## Conclusion

- Result: The original family mismatch was not a mechanism-layer gap; it was a recipe-level ownership drift where trigger width had been baked into the default builder.
- Result: After removing the default `w_full()`, the compact `DatePicker` aligns better with shadcn source ownership: recipe-owned chrome stays in the component, while page/form width negotiation stays at the call site.
- Result: The same ownership fix now holds across the adjacent range/presets recipes, so the date-picker family no longer mixes intrinsic-width and fill-width defaults for equivalent trigger surfaces.
- Result: The remaining family-level consistency gap was public surface parity, not runtime mechanics; `DateRangePicker` and `DatePickerWithPresets` now match `DatePicker` on `control_id(...)` and `test_id_prefix(...)` support.
- Result: The compact wrapper family also no longer hard-codes "select then dismiss"; that behavior now matches the upstream docs defaults, while explicit dismiss-on-select remains available as a recipe-level opt-in.
- Result: The latest follow-up drift was on the gallery teaching surface, not in `fret-ui` or the shadcn recipes: the `Input` example had widened the trailing trigger beyond the upstream `ghost` + `icon-xs` shape, and the docs page had incorrectly described `With Presets` as a gallery-only extension even though it has an upstream registry source.
- Result: No generic children API is required for the current shadcn/Base UI parity target.

## Validation

- `cargo nextest run -p fret-ui-shadcn --lib date_picker_trigger_width_is_intrinsic_unless_caller_overrides_it --status-level fail`
- `cargo nextest run -p fret-ui-shadcn --lib date_range_picker_trigger_width_is_intrinsic_unless_caller_overrides_it date_picker_with_presets_trigger_width_is_intrinsic_unless_caller_overrides_it --status-level fail`
- `cargo nextest run -p fret-ui-shadcn --test date_picker_close_on_select --status-level fail`
- `cargo nextest run -p fret-ui-shadcn --lib date_range_picker_control_id_uses_registry_labelled_by_and_described_by_elements date_range_picker_test_id_prefix_stamps_trigger_content_and_calendar date_picker_with_presets_control_id_uses_registry_labelled_by_and_described_by_elements date_picker_with_presets_test_id_prefix_stamps_trigger_content_select_and_calendar --status-level fail`
- `cargo nextest run -p fret-ui-gallery --lib gallery_date_picker_core_examples_keep_upstream_aligned_targets_present --status-level fail`
- `cargo nextest run -p fret-ui-gallery --test ui_authoring_surface_default_app date_picker_page_uses_typed_doc_sections_for_app_facing_snippets date_picker_input_snippet_keeps_upstream_ghost_icon_xs_trigger_surface date_picker_time_snippet_explicitly_opts_into_close_on_select --status-level fail`
- `cargo run -p fretboard -- diag run --dir target/fret-diag --session-auto tools/diag-scripts/ui-gallery/date-picker/ui-gallery-date-picker-input-open-calendar.json --pack --ai-packet --launch -- cargo run -p fret-ui-gallery --release`
- `cargo run -p fretboard -- diag run --dir target/fret-diag --session-auto tools/diag-scripts/ui-gallery/date-picker/ui-gallery-date-picker-docs-screenshots.json --pack --ai-packet --launch -- cargo run -p fret-ui-gallery --release`
