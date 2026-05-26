# shadcn/ui v4 Audit - Slider

## Upstream references (non-normative)

This document references optional local checkouts under `repo-ref/` for convenience.
Upstream sources:

- shadcn/ui: https://github.com/shadcn-ui/ui

See `docs/repo-ref.md` for the optional local snapshot policy and pinned SHAs.
This audit compares Fret's shadcn-aligned `Slider` against the current upstream shadcn/ui v4 docs,
the `new-york-v4` registry implementation in `repo-ref/ui`, and the Base/Radix registry slider
examples used as secondary references for richer example coverage and compound-parts ownership.

## Upstream references (source of truth)

- Docs page:
  - `repo-ref/ui/apps/v4/content/docs/components/slider.mdx`
- Current shadcn example file:
  - `repo-ref/ui/apps/v4/registry/new-york-v4/examples/slider-demo.tsx`
- Secondary Base/Radix example files:
  - `repo-ref/ui/apps/v4/registry/bases/base/examples/slider-example.tsx`
  - `repo-ref/ui/apps/v4/registry/bases/radix/examples/slider-example.tsx`
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
- Pass: Vertical roots now keep the upstream `min-h-44` floor via `component.slider.vertical_min_height`, while caller-provided heights still merge as authored and only clamp upward when they fall below that floor.
- Pass: The vertical source axes are intentionally split: the Base/Radix example lane still shows caller-owned `h-40` (`repo-ref/ui/apps/v4/registry/bases/*/examples/slider-example.tsx`), while the default new-york-v4 recipe keeps `min-h-44` in the component source.
- Pass: A page-level gallery geometry gate now reads layout bounds for the two vertical docs examples and asserts both roots clamp to the same `176px` minimum without conflating thumb visual overflow with root layout height.
- Pass: Layout height follows the track; the thumb is allowed to overflow without being clipped
  (overflow-visible semantics), matching the DOM implementation.
- Pass: Thumb stays visually in-bounds at the edges (Radix `getThumbInBoundsOffset` outcome), so the
  center-aligned thumb does not underflow/overflow the track at `t=0` / `t=1`.

### Semantics

- Pass: Exposes slider semantics on each thumb (`SemanticsRole::Slider`) with numeric value, min/max, step, and focusability, matching the Radix/Base UI ownership split more closely than a root-level role would.
- Pass: The root keeps the overall bounds/test-id/value summary needed for diagnostics and `set_slider_value` automation, while thumb nodes carry the interactive slider role.
- Pass: The diagnostics `set_slider_value` driver now resolves from the root summary node down to the descendant thumb slider semantics before using numeric accessibility actions, so the existing slider automation gate passes again without changing the shadcn recipe surface.

### Gallery / docs parity

- Pass: The UI Gallery page now mirrors the current shadcn docs path first: `Demo` and `Usage`.
- Pass: `Demo` now mirrors the current upstream docs preview lane (`[50]` plus caller-owned `w-[60%]`), while `Usage` keeps the docs code-block lane (`[33]`) instead of flattening both sections to one shared default value.
- Pass: Base/Radix follow-up snippets keep the secondary example shapes visible: `Range` uses `[25, 50]` with `step(5)`, `Multiple Thumbs` uses `[10, 20, 70]` with `step(10)`, `Vertical` shows the two-slider `h-40` layout, and `Controlled` keeps the label/readout association via `ControlId` + `Label::for_control(...)`.
- Pass: `RTL`, `API Reference`, `Label Association`, `Extras`, and `Notes` stay after the current docs path because they are Fret-specific follow-ups rather than current upstream shadcn sections.
- Pass: Stable `ui-gallery-slider-*` root/test-id anchors are restored so the existing diag scripts target the real preview controls again.
- Pass: A docs-surface screenshot script now captures the docs-path sections so visual/page-order drift is reviewable without manual browsing.
- Pass: This work is docs/public-surface parity and diagnostics-surface repair, not a mechanism-layer rewrite.

## Validation

- `cargo test -p fret-ui-shadcn --lib slider`
- `cargo test -p fret-ui-shadcn --lib slider_vertical_layout_uses_upstream_floor_as_minimum_not_fixed_height`
- `cargo check -p fret-ui-gallery --message-format short`
- `cargo nextest run -p fret-ui-gallery --lib gallery_slider_vertical_examples_keep_upstream_recipe_min_height_floor`
- `cargo nextest run -p fret-ui-gallery --test slider_docs_surface`
- Web layout gate: `cargo nextest run -p fret-ui-shadcn --test web_vs_fret_layout`
  (`web_vs_fret_layout_slider_demo_geometry`).
- Web layout gate (thumb insets): `cargo nextest run -p fret-ui-shadcn -E "test(web_vs_fret_layout_field_slider_thumb_insets_match_web)"`
- Diagnostics scripts:
  - `tools/diag-scripts/ui-gallery/slider/ui-gallery-slider-set-value.json`
  - `tools/diag-scripts/ui-gallery/slider/ui-gallery-slider-range-drag-stability.json`
  - `tools/diag-scripts/ui-gallery/slider/ui-gallery-slider-label-click-focus.json`
  - `tools/diag-scripts/ui-gallery/slider/ui-gallery-slider-docs-screenshots.json`
  - The `ui-gallery-slider-set-value.json` gate is green again after the diagnostics driver started preferring descendant slider semantics for numeric `SetValue`.
- Matrix packet:
  - `docs/workstreams/shadcn-component-parity-matrix-v1/artifacts/slider_agent_packet_p0_v1.json`

## Follow-ups (recommended)

- Upstream limitation: the current shadcn Slider docs page only exposes `Demo` and `Usage`; keep Base/Radix richer examples covered as secondary source-axis tests plus local layout/unit gates unless upstream promotes those examples into the main docs path.
- If a Base UI-style compound slider API becomes necessary, land it as a headless/ui-kit surface first instead of widening the shadcn recipe lane.
- Add a Radix-web gate for keyboard step behavior (e.g. ArrowRight) once we have a stable event
  harness for non-overlay primitives.
