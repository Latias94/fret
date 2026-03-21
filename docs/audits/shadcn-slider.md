# shadcn/ui v4 Audit - Slider

## Upstream references (non-normative)

This document references optional local checkouts under `repo-ref/` for convenience.
Upstream sources:

- shadcn/ui: https://github.com/shadcn-ui/ui

See `docs/repo-ref.md` for the optional local snapshot policy and pinned SHAs.
This audit compares Fret's shadcn-aligned `Slider` against the upstream shadcn/ui v4 docs, the
`new-york-v4` registry implementation in `repo-ref/ui`, and the Base UI headless slider surface
used as an extra reference for compound-parts ownership.

## Upstream references (source of truth)

- Docs pages:
  - `repo-ref/ui/apps/v4/content/docs/components/radix/slider.mdx`
  - `repo-ref/ui/apps/v4/content/docs/components/base/slider.mdx`
- Registry implementations:
  - `repo-ref/ui/apps/v4/registry/new-york-v4/ui/slider.tsx`
  - `repo-ref/ui/apps/v4/registry/bases/radix/ui/slider.tsx`
  - `repo-ref/ui/apps/v4/registry/bases/base/ui/slider.tsx`
- Underlying primitives:
  - Radix `@radix-ui/react-slider`
  - Base UI `@base-ui/react/slider`

## Fret implementation

- Component code: `ecosystem/fret-ui-shadcn/src/slider.rs`
- Gallery page: `apps/fret-ui-gallery/src/ui/pages/slider.rs`
- Gallery snippets: `apps/fret-ui-gallery/src/ui/snippets/slider/`
- Shared primitives:
  - Radix-aligned slider semantics/value updates: `ecosystem/fret-ui-kit/src/primitives/slider.rs`
  - Pointer-to-value mapping helpers: `ecosystem/fret-ui-kit/src/declarative/slider.rs`

## Audit checklist

### Authoring surface

- Pass: `Slider::new(model)` covers the common controlled authoring path.
- Pass: `Slider::new_controllable(...)` covers the upstream `defaultValue`-style authoring path.
- Pass: `range(...)`, `step(...)`, `orientation(...)`, and `on_value_commit(...)` cover the important shadcn/Radix recipe surface.
- Pass: `slider(model)` remains the default first-party teaching helper for app-facing controlled usage, while `new_controllable(...)` covers self-owned default values.
- Pass: `Slider` already has the composition and interaction hooks it needs, so Fret intentionally does not add a generic `compose()` or arbitrary root `children(...)` API on the shadcn lane.
- Pass: Base UI's compound `Slider.Root/Label/Value/Control/Track/Indicator/Thumb` family is a useful headless reference, but it belongs to a future `fret-ui-kit`-level surface rather than the `fret-ui-shadcn::Slider` recipe.

### Layout & geometry (shadcn parity)

- Pass: Track height defaults to `h-1.5` (6px) via `component.slider.track_height`.
- Pass: Thumb defaults to `size-4` (16px) via `component.slider.thumb_size`.
- Pass: Layout height follows the track; the thumb is allowed to overflow without being clipped
  (overflow-visible semantics), matching the DOM implementation.
- Pass: Thumb stays visually in-bounds at the edges (Radix `getThumbInBoundsOffset` outcome), so the
  center-aligned thumb does not underflow/overflow the track at `t=0` / `t=1`.

### Semantics

- Pass: Exposes slider semantics on each thumb (`SemanticsRole::Slider`) with numeric value, min/max, step, and focusability, matching the Radix/Base UI ownership split more closely than a root-level role would.
- Pass: The root keeps the overall bounds/test-id/value summary needed for diagnostics and `set_slider_value` automation, while thumb nodes carry the interactive slider role.

### Gallery / docs parity

- Pass: The UI Gallery page now mirrors the upstream docs path first: `Demo`, `Usage`, `Range`, `Multiple Thumbs`, `Vertical`, `Controlled`, `Disabled`, `RTL`, and `API Reference`.
- Pass: `Label Association`, `Extras`, and `Notes` stay after the docs path because they are Fret-specific follow-ups rather than upstream shadcn sections.
- Pass: Stable `ui-gallery-slider-*` root/test-id anchors are restored so the existing diag scripts target the real preview controls again.
- Pass: This work is docs/public-surface parity and diagnostics-surface repair, not a mechanism-layer rewrite.

## Validation

- `cargo test -p fret-ui-shadcn --lib slider`
- `cargo check -p fret-ui-gallery --message-format short`
- Web layout gate: `cargo nextest run -p fret-ui-shadcn --test web_vs_fret_layout`
  (`web_vs_fret_layout_slider_demo_geometry`).
- Web layout gate (thumb insets): `cargo nextest run -p fret-ui-shadcn -E "test(web_vs_fret_layout_field_slider_thumb_insets_match_web)"`
- Diagnostics scripts:
  - `tools/diag-scripts/ui-gallery/slider/ui-gallery-slider-set-value.json`
  - `tools/diag-scripts/ui-gallery/slider/ui-gallery-slider-range-drag-stability.json`
  - `tools/diag-scripts/ui-gallery/slider/ui-gallery-slider-label-click-focus.json`

## Follow-ups (recommended)

- If a Base UI-style compound slider API becomes necessary, land it as a headless/ui-kit surface first instead of widening the shadcn recipe lane.
- Add a Radix-web gate for keyboard step behavior (e.g. ArrowRight) once we have a stable event
  harness for non-overlay primitives.
