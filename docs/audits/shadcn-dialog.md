# shadcn/ui v4 Audit - Dialog (new-york)

This audit compares Fret's shadcn-aligned `Dialog` surface against the upstream shadcn/ui v4 docs
and the `new-york-v4` registry implementation in `repo-ref/ui`.

## Upstream references (source of truth)

- Docs page: `repo-ref/ui/apps/v4/content/docs/components/dialog.mdx`
- Registry implementation (new-york): `repo-ref/ui/apps/v4/registry/new-york-v4/ui/dialog.tsx`
- Underlying primitive concept: Radix `@radix-ui/react-dialog` (portal + dismiss + focus management)

## Fret implementation

- Component code: `crates/fret-components-shadcn/src/dialog.rs`
- Depends on overlay policy/infra:
  - `crates/fret-components-ui/src/window_overlays/*` (modal overlays, focus restore/initial focus)
  - `crates/fret-components-ui/src/overlay_controller.rs` (overlay requests + presence)
  - `crates/fret-ui/src/tree/*` (modal barrier scoping + focus traversal contract, ADR 0068)

## What upstream exports (new-york)

Upstream shadcn/ui exports a thin wrapper around Radix:

- `Dialog`
- `DialogTrigger`
- `DialogContent`
- `DialogHeader`
- `DialogFooter`
- `DialogTitle`
- `DialogDescription`

## Audit checklist

### Composition surface

- Pass: Fret exposes `Dialog` as a recipe driven by a `Model<bool>` open state.
- Pass: Trigger/content composition matches the shadcn mental model.
- Pass: Content is rendered via a per-window overlay root (portal-like), so it is not clipped by
  underlay layout/overflow.
- Pass: `DialogClose` is available as an explicit close affordance recipe (close button parity).

### Dismissal behavior

- Pass: Escape dismiss is handled by the shared dismissible root (Radix-aligned outcome).
- Pass: Overlay click-to-dismiss is implemented by rendering a full-window barrier behind the
  content (default on).

### Focus behavior

- Pass: Modal barrier scoping prevents underlay focus traversal (ADR 0068).
- Pass: On open, initial focus falls back to the first focusable descendant within the modal root
  (via `window_overlays` focus helpers).
- Pass: On close, focus restoration is deterministic to the trigger (modal close unmount path).

### Visual parity (new-york)

- Partial: Motion/animations are simplified (Fret uses a small fade presence).
- Partial: Some sizing defaults differ from Tailwind classes; Fret favors theme-driven metrics.

## Validation

- `cargo check -p fret-components-shadcn`
- `cargo nextest run -p fret-components-shadcn dialog::tests`

## Follow-ups (recommended)

- Add side-based motion variants consistent with shadcn `data-[state=open]` animations.
