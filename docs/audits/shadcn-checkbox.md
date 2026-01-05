# shadcn/ui v4 Audit — Checkbox

This audit compares Fret’s shadcn-aligned `Checkbox` against the upstream shadcn/ui v4 docs and the
`new-york-v4` implementation in `repo-ref/ui`.

## Upstream references (source of truth)

- Docs page: `repo-ref/ui/apps/v4/content/docs/components/checkbox.mdx`
- Registry implementation (new-york): `repo-ref/ui/apps/v4/registry/new-york-v4/ui/checkbox.tsx`
- Underlying primitive: Radix `@radix-ui/react-checkbox`

## Fret implementation

- Component code: `ecosystem/fret-ui-shadcn/src/checkbox.rs`
- Shared primitives:
  - Radix checkbox outcomes: `ecosystem/fret-ui-kit/src/primitives/checkbox.rs`
  - Focus ring recipe: `ecosystem/fret-ui-kit/src/declarative/style.rs`
  - Control chrome composition: `ecosystem/fret-ui-kit/src/declarative/chrome.rs`

## Audit checklist

### Interaction

- Pass: Click toggles the bound `Model<bool>`.
- Pass: Disabled state blocks interaction and applies reduced opacity.

### Semantics

- Pass: Exposes `SemanticsRole::Checkbox` and `checked` state.

### Visual parity (new-york)

- Pass: Unchecked state uses `border-input` and transparent background.
- Pass: Checked state uses `primary` background, `primary-foreground` indicator color, and `primary`
  border.
- Pass: Uses `shadow_xs`, matching shadcn’s `shadow-xs` default.
- Pass: Focus-visible styling includes an outward focus ring and a `ring`-colored border (best-effort).

## Validation

- `cargo test -p fret-ui-shadcn --lib checkbox`

## Follow-ups (recommended)

- Pass: Supports Radix `checked="indeterminate"` (tri-state) via `Checkbox::new_tristate`.
  - Note: Semantics currently maps indeterminate to `checked: None`.
