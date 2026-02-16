# shadcn/ui v4 Audit — Toggle Group


## Upstream references (non-normative)

This document references optional local checkouts under `repo-ref/` for convenience.
Upstream sources:

- shadcn/ui: https://github.com/shadcn-ui/ui

See `docs/repo-ref.md` for the optional local snapshot policy and pinned SHAs.
This audit compares Fret’s shadcn-aligned `ToggleGroup` against the upstream shadcn/ui v4 docs and
the `new-york-v4` implementation in `repo-ref/ui`.

## Upstream references (source of truth)

- Docs page: `repo-ref/ui/apps/v4/content/docs/components/toggle-group.mdx`
- Registry implementation (new-york): `repo-ref/ui/apps/v4/registry/new-york-v4/ui/toggle-group.tsx`
- Underlying primitive: Radix `@radix-ui/react-toggle-group`

## Fret implementation

- Component code: `ecosystem/fret-ui-shadcn/src/toggle_group.rs`
- Related surfaces:
  - Toggle tokens: `ecosystem/fret-ui-shadcn/src/toggle.rs`
  - Roving focus policy: `ecosystem/fret-ui-kit/src/primitives/roving_focus_group.rs`

## Audit checklist

### Composition surface

- Pass: Supports `single` (`Model<Option<Arc<str>>>`) and `multiple` (`Model<Vec<Arc<str>>>`) modes.
- Pass: Supports uncontrolled `defaultValue` (internal selection model).
- Pass: Supports `orientation` and `loop_navigation` (Radix `orientation` / `loop` outcomes).
- Pass: Supports `variant` (`default` / `outline`) and size (`sm` / `default` / `lg`).
- Pass: Supports `spacing(...)` (gap between items).

### Selection behavior

- Pass: Single mode deactivates when clicking the selected item (Radix single-toggle outcome).
- Pass: Multiple mode toggles membership per item value.

### Visual parity (new-york)

- Pass: When `spacing == 0`, items form a segmented control:
  - Items have no inner rounding, first/last get rounded corners.
  - Outline variant collapses inner borders (`border-left: 0` / `border-top: 0`) for non-first items.
- Pass: Focus-visible styling includes an outward focus ring and a `ring`-colored outline border (best-effort).
- Pass: When `spacing != 0` and `variant == outline`, the group container uses `shadow_xs`, matching shadcn’s `shadow-xs`.

## Validation

- `cargo test -p fret-ui-shadcn --lib toggle_group`

## Follow-ups (recommended)

- Consider matching shadcn’s exact size scale (`h-8 / h-9 / h-10`) via theme tokens.
