# shadcn/ui v4 Audit — Popover

This audit compares Fret’s shadcn-aligned `Popover` against the upstream shadcn/ui v4 docs and
examples in `repo-ref/ui`.

## Upstream references (source of truth)

- Docs page: `repo-ref/ui/apps/v4/content/docs/components/popover.mdx`
- Reference implementation (Radix base): `repo-ref/ui/apps/v4/registry/bases/radix/ui/popover.tsx`
- Reference examples: `repo-ref/ui/apps/v4/registry/bases/radix/examples/popover-example.tsx`

## Fret implementation

- Component code: `crates/fret-components-shadcn/src/popover.rs`
- Overlay policy substrate: `crates/fret-components-ui/src/overlay_controller.rs`
- Outside-press + dismissible layer mechanism: `crates/fret-ui/src/declarative/host_widget/event/dismissible.rs`

## Audit checklist

### Composition surface

- Pass: `Popover` + `PopoverTrigger` + `PopoverContent` exist and are declarative-only.
- Pass: `PopoverHeader` + `PopoverTitle` exist (used by upstream examples).
- Pass: `PopoverDescription` exists (used by upstream examples).
- TODO: `PopoverAnchor` is not implemented (upstream exports it in the Radix base wrapper).

### Placement & portal behavior

- Pass: Renders into a per-window overlay root (portal-like) via `OverlayController`.
- Pass: Supports `side` and `align` placement options.
- Pass: Supports `align_offset` and `side_offset` knobs.
- Pass: Placement anchors to **visual bounds** when available (render-transform aware) via
  `fret-components-ui::overlay::anchor_bounds_for_element`.

### Dismissal & focus behavior

- Pass: Outside press closes via click-through observer phase and does not override new focus
  (`popover_outside_press_closes_without_overriding_new_focus`).
- Pass: Escape requests dismissal via `DismissibleLayer` mechanism.
- Pass: Non-modal by default (no focus trap); modal behavior belongs to `Dialog`/`Sheet`.
- Note: Auto-focus behavior is currently explicit (`Popover::auto_focus(true)`), not automatic.

### Visual defaults (shadcn parity)

- Pass: Default content chrome uses theme popover colors + border + radius + shadow.
- Pass: Default content width targets upstream’s `w-72` (approx `288px`) and can be overridden via
  `PopoverContent::refine_layout(...)`.
- Pass: Default `side_offset` matches upstream `sideOffset=4`.

## Open questions / follow-ups

- Decide whether `PopoverContent` should use `SemanticsRole::Dialog` (Radix uses `role="dialog"`),
  or keep `Panel` (current).
- Consider adding a `PopoverAnchor` surface if shadcn blocks rely on it.

