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
- Pass: Supports optional state via `Checkbox::new_optional(Model<Option<bool>>)` where `None` maps
  to indeterminate (Radix outcome), and click toggles to `Some(true)`.
- Pass: Disabled state blocks interaction and applies reduced opacity.

### Semantics

- Pass: Exposes `SemanticsRole::Checkbox` and `checked` state.

### Visual parity (new-york)

- Pass: Unchecked state uses `border-input` and transparent background.
- Pass: Checked state uses `primary` background, `primary-foreground` indicator color, and `primary`
  border.
- Pass: Uses `shadow_xs`, matching shadcn’s `shadow-xs` default.
- Pass: Focus ring thickness (`ring-[3px]`) matches shadcn-web focus variant (`checkbox-demo.focus`).

## Validation

- `cargo test -p fret-ui-shadcn --lib checkbox`
- Web layout gate: `cargo nextest run -p fret-ui-shadcn --test web_vs_fret_layout`
  (`web_vs_fret_layout_checkbox_demo_control_size`).
- Focus ring gate: `cargo nextest run -p fret-ui-shadcn --test web_vs_fret_control_chrome`
  (`web_vs_fret_checkbox_demo_focus_ring_matches`).

## Follow-ups (recommended)

- Pass: Supports Radix `checked="indeterminate"` (tri-state) via `Checkbox::new_tristate`.
  - Note: Semantics currently maps indeterminate to `checked: None`.
