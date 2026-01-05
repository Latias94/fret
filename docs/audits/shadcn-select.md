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

- Pass: Trigger `Enter` / `Space` / `ArrowDown` / `ArrowUp` opens the popover on key down when
  closed (Radix `OPEN_KEYS` outcome; prevents the key-up activation from toggling it closed).
- Pass: Trigger typeahead while closed updates the selection without opening (Radix trigger
  typeahead behavior); `Space` is ignored while typing ahead.
- Pass: Listbox navigation uses roving focus + typeahead hooks (`cx.roving_nav_apg()` and
  `cx.roving_typeahead_prefix_arc_str(...)`).
- Pass: `loop_navigation(true)` defaults to looping behavior (Radix `loop` default).

### Selection & dismissal

- Pass: Selecting an item commits `model` and closes the overlay.
- Pass: Outside press dismissal is delegated to the shared dismissible popover infra (ADR 0069).

### Visual parity (shadcn)

- Pass: Selected option shows a trailing checkmark (`ids::ui::CHECK`) and selection background.
- Pass: Structural rows are supported via `SelectEntry` (`SelectLabel`, `SelectGroup`,
  `SelectSeparator`) rendered inside the listbox.
- Partial: Upstream includes scroll buttons (`SelectScrollUpButton` / `SelectScrollDownButton`) and
  richer content positioning modes (`item-aligned` vs `popper`).

## Validation

- `cargo test -p fret-ui-shadcn --lib select`

## Follow-ups (recommended)

- Add scroll buttons aligned with the upstream taxonomy.
- Align trigger-side typeahead behavior when closed (Radix trigger typeahead updates selection).
