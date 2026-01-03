# shadcn/ui v4 Audit - Tooltip

This audit compares Fret's shadcn-aligned `Tooltip` against the upstream shadcn/ui v4 docs and
examples in `repo-ref/ui`.

## Upstream references (source of truth)

- Docs page: `repo-ref/ui/apps/v4/content/docs/components/tooltip.mdx`
- Reference implementation (Radix base): `repo-ref/ui/apps/v4/registry/bases/radix/ui/tooltip.tsx`
- Reference examples: `repo-ref/ui/apps/v4/registry/bases/radix/examples/tooltip-example.tsx`

Key upstream notes:

- Tooltip should open on **hover** and on **keyboard focus**.
- v4 updated tooltip colors to `bg-foreground text-background` (2025-09-22 changelog).
- Content includes an arrow and supports rich children (icons, `<Kbd />`, formatted blocks).

## Fret implementation

- Component code: `ecosystem/fret-ui-shadcn/src/tooltip.rs`
- Hover intent state machine: `ecosystem/fret-ui-kit/src/headless/hover_intent.rs`
- Overlay placement helpers: `ecosystem/fret-ui-kit/src/overlay.rs`
- Per-window tooltip overlay policy: `ecosystem/fret-ui-kit/src/window_overlays/mod.rs`

## Audit checklist

### Composition surface

- Pass: `Tooltip`, `TooltipTrigger`, `TooltipContent` exist and are declarative-only.
- Pass: `TooltipContent` now supports rich children (`Vec<AnyElement>`), matching upstream examples.
- Pass: `TooltipProvider` exists and provides a shared delay group for consistent open delays.
- Pass: Fret additionally supports an optional custom placement anchor via `TooltipAnchor` +
  `Tooltip::anchor_element(...)` (anchor can be separate from the trigger).

### Open/close behavior

- Pass: Hover open/close is implemented via `HoverRegion` + `HoverIntentState`.
- Pass: Open-on-keyboard-focus is implemented via `ElementContext::is_focused_element(trigger_id)`.

### Placement & portal behavior

- Pass: Renders into a per-window overlay root (portal-like), not clipped by ancestor overflow.
- Pass: Supports `side` and `align` placement options.
- Pass: Default `side_offset` aligns with upstream's default (`0`) and can be overridden.
- Pass: Placement anchors to **visual bounds** when available (render-transform aware) via
  `fret-ui-kit::overlay::anchor_bounds_for_element`.

### Visual defaults (shadcn parity)

- Pass: Default tooltip background aligns with upstream (`foreground`).
- Partial: Default “text color inheritance” is not a first-class concept; `TooltipContent::text(...)`
  sets `background` as the text color, but rich children must set colors explicitly for now.
- Pass: Arrow is implemented and enabled by default (can be disabled via `Tooltip::arrow(false)`).

## Follow-ups (recommended)

- Consider expanding `TooltipProvider` knobs (e.g. disable-hoverable-content) if parity needs it.
- Add nextest contract tests for tooltip hover timing + placement invariants.
