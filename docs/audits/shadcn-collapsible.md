# shadcn/ui v4 Audit — Collapsible

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
- Pass: Uses a controlled open state (`Model<bool>`).
- Note: Upstream supports uncontrolled `defaultOpen`; Fret currently does not model it.

### A11y behavior

- Pass: Trigger exposes an expanded outcome (`expanded=true/false`).
- Note: Fret does not currently model `aria-controls` (content id wiring).

### Content mount/unmount

- Partial: Upstream uses `Presence` + measured content dimensions for height animations.
- Partial: Fret keeps content mounted while closing (presence-style) and approximates the height
  animation by caching the last measured content height and clipping with an eased 0..1 progress.
  - Shared helper: `ecosystem/fret-ui-kit/src/declarative/collapsible_motion.rs`
  - Implementation: `ecosystem/fret-ui-shadcn/src/collapsible.rs`
  - Backing helper: `ecosystem/fret-ui-kit/src/primitives/presence.rs`
  - Note: Fret does not expose Radix's `--radix-collapsible-content-height/width` variables; the
    cached height is stored in per-element state.

## Validation

- `cargo test -p fret-ui-shadcn --lib collapsible`
