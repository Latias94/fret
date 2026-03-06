# shadcn/ui v4 Audit - Date Picker

## Upstream references (non-normative)

This document references optional local checkouts under `repo-ref/` for convenience.
Upstream sources:

- shadcn/ui: https://github.com/shadcn-ui/ui
- Radix Primitives: https://github.com/radix-ui/primitives

See `docs/repo-ref.md` for the optional local snapshot policy and pinned SHAs.

## Upstream references (source of truth)

- shadcn docs: `repo-ref/ui/apps/v4/content/docs/components/date-picker.mdx`
- shadcn demo recipe: `repo-ref/ui/apps/v4/registry/new-york-v4/examples/date-picker-demo.tsx`

## Fret implementation anchors

- Component code: `ecosystem/fret-ui-shadcn/src/date_picker.rs`
- Building blocks:
  - `ecosystem/fret-ui-shadcn/src/button.rs`
  - `ecosystem/fret-ui-shadcn/src/popover.rs`
  - `ecosystem/fret-ui-shadcn/src/drawer.rs`
  - `ecosystem/fret-ui-shadcn/src/calendar.rs`
- Gallery page: `apps/fret-ui-gallery/src/ui/pages/date_picker.rs`
- Copyable usage snippet: `apps/fret-ui-gallery/src/ui/snippets/date_picker/usage.rs`

## Audit checklist

### Authoring surface

- Pass: `DatePicker::new(open, month, selected)` already covers the common compact recipe path.
- Pass: `DatePicker::new_controllable(...)` covers Radix-style controlled/uncontrolled authoring without
  forcing callers to manually allocate models.
- Pass: `placeholder(...)`, `format_selected_by(...)`, `format_selected_iso()`, `week_start(...)`,
  `control_id(...)`, `test_id_prefix(...)`, and disable/show-outside-days toggles cover the important
  shadcn recipe outcomes.
- Note: Because `DatePicker` is intentionally a compact recipe over `Popover + Calendar`, it does not need
  a composable children API or a generic `compose()` builder.

### Trigger composition (Button)

- Pass: Trigger uses outline button styling and left-justified content (`justify-start` parity).
- Pass: Trigger includes a leading calendar icon (Lucide ID: `lucide.calendar`).
- Pass: Placeholder state uses muted foreground color when no date is selected.
- Pass: Trigger uses normal font weight (shadcn demo uses `font-normal`).

### Overlay composition (Popover + Calendar)

- Pass: Date picker is authored as a composition of `Popover` and `Calendar` (shadcn docs contract).
- Pass: Popover content uses `w-auto p-0` style intent (no default padding).

### Responsive recipe (gallery-only)

The upstream docs describe a Popover-based recipe. In the Fret UI gallery, the "With dropdowns"
section is rendered as explicit desktop/mobile branches for deterministic validation:

- Pass: Desktop (`>= md`) uses `Popover`.
- Pass: Narrow viewports use `Drawer` (mobile-friendly recipe).

## Conclusion

- Result: This component does not currently point to a missing mechanism-layer gap in the shadcn-facing surface.
- Result: The main missing piece for docs parity was a concise gallery `Usage` example for the compact builder.
- Result: Follow-up work should focus on concrete parsing/calendar behavior regressions only if they appear.

## Validation

- `cargo check -p fret-ui-gallery --message-format short`
- `cargo check -p fret-ui-shadcn -p fret-ui-gallery -p fret-diag`
- `cargo run -p fretboard -- diag run tools/diag-scripts/ui-gallery-date-picker-dropdowns-mobile-drawer.json --launch -- cargo run -p fret-ui-gallery --release`
- `cargo run -p fretboard -- diag run tools/diag-scripts/ui-gallery-date-picker-nested-caption-select-scroll-clamp.json --launch -- cargo run -p fret-ui-gallery --release`
