# shadcn/ui v4 Audit — Collapsible


## Upstream references (non-normative)

This document references optional local checkouts under `repo-ref/` for convenience.
Upstream sources:

- shadcn/ui: https://github.com/shadcn-ui/ui

See `docs/repo-ref.md` for the optional local snapshot policy and pinned SHAs.
This audit compares Fret's shadcn-aligned `Collapsible` surface against the upstream shadcn/ui v4
docs and the `new-york-v4` registry implementation in `repo-ref/ui`.

## Upstream references (source of truth)

- Docs page: `repo-ref/ui/apps/v4/content/docs/components/collapsible.mdx`
- Registry implementation (new-york): `repo-ref/ui/apps/v4/registry/new-york-v4/ui/collapsible.tsx`
- Registry demo: `repo-ref/ui/apps/v4/registry/new-york-v4/examples/collapsible-demo.tsx`
- Underlying primitive: Radix `@radix-ui/react-collapsible`

## Fret implementation

- Component code: `ecosystem/fret-ui-shadcn/src/collapsible.rs`
- Radix-aligned primitive helpers: `ecosystem/fret-ui-kit/src/primitives/collapsible.rs`

## Audit checklist

### Composition surface

- Pass: Provides `Collapsible`, `CollapsibleTrigger`, and `CollapsibleContent` wrappers.
- Pass: A composable Radix/shadcn-shaped children surface is available via
  `fret_ui_shadcn::collapsible::primitives`.
- Pass: The legacy flat module path `fret_ui_shadcn::collapsible_primitives` remains available for
  compatibility.
- Note: Fret intentionally does not add a generic `compose()` builder on the top-level shadcn
  wrapper because the composable children API already exists in the primitives surface, while the
  wrapper keeps a compact closure-based ergonomic API.
- Pass: Supports a controlled open state (`Model<bool>`).
- Pass: Supports uncontrolled `defaultOpen` (internal open model).

### A11y behavior

- Pass: Trigger exposes an expanded outcome (`expanded=true/false`).
- Pass: Trigger can model Radix `aria-controls` via the `controls_element` relationship when the
  content wrapper is mounted.

### Content mount/unmount

- Pass: Upstream uses `Presence` + measured content dimensions for height animations; Fret models
  the same outcome by caching the measured content height in per-element state and driving a clipped
  wrapper height using an eased 0..1 progress value.
  - Shared helper: `ecosystem/fret-ui-kit/src/primitives/collapsible.rs` (delegates to `declarative/collapsible_motion.rs`)
  - Implementation: `ecosystem/fret-ui-shadcn/src/collapsible.rs`
  - Note: Fret does not expose Radix's `--radix-collapsible-content-height/width` CSS variables; the
    cached size is stored in element state.

## Validation

- `cargo test -p fret-ui-shadcn --lib collapsible`
