# shadcn/ui v4 Audit — Select

This audit compares Fret’s shadcn-aligned `Select` against the upstream shadcn/ui v4 docs and
examples in `repo-ref/ui`.

## Upstream references (source of truth)

- Docs page: `repo-ref/ui/apps/v4/content/docs/components/select.mdx`
- Component wrapper (Radix Select skin): `repo-ref/ui/apps/v4/registry/new-york-v4/ui/select.tsx`
- Demo usage: `repo-ref/ui/apps/v4/registry/new-york-v4/examples/select-demo.tsx`

## Fret implementation

- Component code: `ecosystem/fret-ui-shadcn/src/select.rs`
- Key building blocks:
  - Overlay orchestration: `ecosystem/fret-ui-kit/src/overlay_controller.rs`
  - Anchored placement: `ecosystem/fret-ui-kit/src/primitives/popper.rs`
  - Dismissal policy: `ecosystem/fret-ui-kit/src/primitives/dismissable_layer.rs`
  - Focus policy: `ecosystem/fret-ui-kit/src/primitives/focus_scope.rs`

## Audit checklist

### Placement & alignment

- Pass: Exposes `side`/`align` and both `side_offset(...)` + `align_offset(...)`, mapping to
  `PopperContentPlacement`.
- Pass: Uses per-window overlay roots (portal-like) via `OverlayController`.
- Pass: Optional diamond arrow rendering (`Select::arrow(true)`).

### Keyboard & roving navigation

- Pass: Trigger `ArrowDown` / `ArrowUp` opens the popover when closed (Radix `OPEN_KEYS` outcome).
- Pass: Listbox navigation uses roving focus + typeahead hooks (`cx.roving_nav_apg()` and
  `cx.roving_typeahead_prefix_arc_str(...)`).
- Pass: `loop_navigation(true)` defaults to looping behavior (Radix `loop` default).

### Selection & dismissal

- Pass: Selecting an item commits `model` and closes the overlay.
- Pass: Outside press dismissal is delegated to the shared dismissible popover infra (ADR 0069).

### Visual parity (shadcn)

- Pass: Selected option shows a trailing checkmark (`ids::ui::CHECK`) and selection background.
- Partial: Upstream exposes a split taxonomy (`SelectContent`, `SelectGroup`, `SelectLabel`,
  `SelectSeparator`, scroll buttons). Fret currently ships a single `Select` with an `items` list.

## Validation

- `cargo test -p fret-ui-shadcn --lib select`

## Follow-ups (recommended)

- Add group/label/separator/scroll-button surfaces aligned with the upstream shadcn taxonomy (likely
  via a richer `entries` model similar to `DropdownMenuEntry`).
- Align trigger-side typeahead behavior when closed (Radix trigger typeahead updates selection).
