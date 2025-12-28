---
title: Declarative-Only Migration Plan (No Retained Widgets)
---

# Declarative-Only Migration Plan (No Retained Widgets)

Goal: make the **declarative element tree** (ADR 0028 / ADR 0039) the only supported UI authoring
model for the component ecosystem, so Tailwind-like primitives and shadcn-like recipes have a
single, stable semantic substrate.

Non-goal: preserve legacy retained widgets. This plan assumes we are willing to break APIs and
delete code.

## Target crate responsibilities

### `crates/fret-ui` (runtime substrate)

Owns:

- event routing (pointer/keyboard), focus + capture, focus-visible
- deterministic layers/overlays composition and hit testing
- renderer-facing scene ops (including rounded clipping)
- declarative layout vocabulary (`LayoutStyle`) + Flex/Grid implementation (Taffy-backed)
- scroll + virtualization primitives as **declarative elements** (handles + pure algorithms)
- overlay placement solver (ADR 0064)

Does not own:

- shadcn/Radix policy surfaces (Popover/Dialog/Menu/Tooltip/HoverCard/Toast/Command palette)
- per-component default values (row heights, paddings, delays) beyond minimal deterministic runtime defaults

### `crates/fret-components-ui` (infrastructure)

Owns:

- typed Tailwind-like primitives (`Space`, `Radius`, `LayoutRefinement`, `ChromeRefinement`)
- style-patch → runtime declarative bridging
- headless state machines (roving focus, typeahead, dismissal) as reusable helpers
- overlay policy managers that compose runtime layers/placement/focus/capture

### `crates/fret-components-shadcn` (taxonomy + recipes)

Owns:

- shadcn/ui v4 naming surface and recipe-level composition
- behavior outcomes aligned with Radix/APG/Floating UI, implemented on top of `fret-components-ui`

## Migration strategy (delete-first, then re-add)

### Phase 0 — stop new drift

- Declarative-only is the default authoring model for the component ecosystem (ADR 0028 / ADR 0039).
- Do not add any new exported retained widget primitives ("Column/Scroll/Stack" style helpers).
- Keep `UiTree` + `Widget` as runtime-internal hosting glue; demos and components should render via
  `declarative::render_root`.

### Phase 1 — extract “engine” types out of retained widgets

Problem: some core element props may depend on types that historically lived in demo/widgets scaffolding
(previously `crates/fret-ui-widgets`).

Actions:

- Move non-widget “engine” types to stable runtime modules, e.g.:
  - `fret_ui::text_input` (editing state + selection/caret + style structs)
  - `fret_ui::text_area` (multiline editing engine)
- Update `crates/fret-ui/src/element.rs` and `crates/fret-ui/src/declarative.rs` to depend only on
  these stable modules, not on `primitives`.

Acceptance:

- `crates/fret-ui` builds with retained widgets removed entirely.

### Phase 2 — declarative ScrollHandle and VirtualList v2

Actions:

- Introduce a runtime `ScrollHandle` contract:
  - offset, set_offset, content_size, viewport_size
  - scroll_to (pixel + item index strategies)
- Implement VirtualList v2 as a declarative element:
  - supports variable row heights (measured + cached)
  - exposes a `VirtualListHandle` (scroll-to-item + query visible range)
  - keeps visible-range computation in runtime (not as authoring-layer props)

Acceptance:

- `fret-components-ui` list/command/table/tree surfaces can share one virtualization contract.

### Phase 3 — move overlay policy out of runtime

Actions:

- Keep runtime-only:
  - layers (portal roots), focus/capture semantics, placement solver, deterministic hit testing
- Move to `fret-components-ui`:
  - open/close state machines
  - dismissal rules (escape/click-outside)
  - focus trap + restore policy
  - hover delays (Tooltip/HoverCard) and pointer intent heuristics

Acceptance:

- `fret-ui` has no `Dialog*/Popover*/Tooltip*/HoverCard*` public types; only substrate contracts.

### Phase 4 — delete retained widgets

Actions:

- Convert remaining component widgets into declarative element helpers (preferred).
- If a retained widget is still needed temporarily, keep it private to a single component module
  (no shared "primitives" module), and schedule deletion.

Acceptance:

- No `impl Widget for ...` remains outside runtime-internal glue.
- UI kit and shadcn recipes are fully declarative.

## Known “big rocks” (expected churn)

- `WindowOverlays` installation will move from “create retained nodes” to “render declarative roots”
  per overlay layer, using stable root IDs and layer roots.
- `TextField/TextArea/Combobox` will migrate from retained `BoundTextInput/BoundTextArea` widgets to
  declarative `TextInput`/`TextArea` elements with shared engine state.
- `ResizablePanelGroup` should become a declarative composition over a runtime split contract.
