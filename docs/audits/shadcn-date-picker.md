# shadcn/ui v4 Audit - Date Picker

## Upstream references (non-normative)

This document references optional local checkouts under `repo-ref/` for convenience.
Upstream sources:

- shadcn/ui: https://github.com/shadcn-ui/ui
- Radix Primitives: https://github.com/radix-ui/primitives

See `docs/repo-ref.md` for the optional local snapshot policy and pinned SHAs.

## Upstream references (source of truth)

- shadcn docs: `repo-ref/ui/apps/v4/content/docs/components/radix/date-picker.mdx`
- shadcn demo recipe: `repo-ref/ui/apps/v4/registry/new-york-v4/examples/date-picker-demo.tsx`

## Fret implementation anchors

- Component code: `ecosystem/fret-ui-shadcn/src/date_picker.rs`
- Building blocks:
  - `ecosystem/fret-ui-shadcn/src/button.rs`
  - `ecosystem/fret-ui-shadcn/src/popover.rs`
  - `ecosystem/fret-ui-shadcn/src/drawer.rs`
  - `ecosystem/fret-ui-shadcn/src/calendar.rs`
- Gallery demo: `apps/fret-ui-gallery/src/ui/pages/date_picker.rs`

## Audit checklist

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

## Validation

- `cargo check -p fret-ui-shadcn -p fret-ui-gallery -p fret-diag`
- `cargo run -p fretboard -- diag run tools/diag-scripts/ui-gallery-date-picker-dropdowns-mobile-drawer.json --launch -- cargo run -p fret-ui-gallery --release`
- `cargo run -p fretboard -- diag run tools/diag-scripts/ui-gallery-date-picker-nested-caption-select-scroll-clamp.json --launch -- cargo run -p fret-ui-gallery --release`
