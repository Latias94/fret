# shadcn/ui v4 Audit — Switch

This audit compares Fret’s shadcn-aligned `Switch` against the upstream shadcn/ui v4 docs and the
`new-york-v4` implementation in `repo-ref/ui`.

## Upstream references (source of truth)

- Docs page: `repo-ref/ui/apps/v4/content/docs/components/switch.mdx`
- Registry implementation (new-york): `repo-ref/ui/apps/v4/registry/new-york-v4/ui/switch.tsx`
- Underlying primitive: Radix `@radix-ui/react-switch`

## Fret implementation

- Component code: `ecosystem/fret-ui-shadcn/src/switch.rs`
- Shared primitives:
  - Focus ring recipe: `ecosystem/fret-ui-kit/src/declarative/style.rs`
  - Control chrome composition: `ecosystem/fret-ui-kit/src/declarative/chrome.rs`

## Audit checklist

### Interaction

- Pass: Click toggles the bound `Model<bool>`.
- Pass: Supports optional state via `Switch::new_opt(Model<Option<bool>>)` where `None` renders as
  unchecked and click toggles to `Some(true)`.
- Pass: Disabled state blocks interaction and applies reduced opacity.

### Semantics

- Pass: Exposes `SemanticsRole::Switch` and `checked` state.

### Visual parity (new-york)

- Pass: Track uses `primary` when checked and `input` when unchecked (theme-key aligned).
- Pass: Thumb is rendered as a circular element with `background` color and is non-interactive.
- Pass: Thumb is vertically centered based on track/thumbnail sizes (aligns with `items-center`).
- Pass: Track uses `shadow_xs`, matching shadcn’s `shadow-xs` default.
- Pass: Focus-visible styling includes an outward focus ring and a `ring`-colored border (best-effort).

## Validation

- `cargo test -p fret-ui-shadcn --lib switch`

## Follow-ups (recommended)

- Consider exposing size variants (e.g. `sm` vs `default`) if parity needs it.
