# Fret Node Graph Add-ons API Stabilization (M2)

This workstream stabilizes the built-in “editor add-ons” surface (background, minimap, controls,
toolbars) so B-layer integrations can compose them without forking `NodeGraphCanvas`.

Unlike M0 (internals refactor harness), M2 is user-facing API work. It must remain policy-light and
portable across platforms.

## Why this workstream exists

The node graph editor ships useful built-in add-ons (MiniMap, controls, overlays), but consumers
still need bespoke glue to compose them, theme them, or wire them into their own store/actions.
Without a stable API, downstream code tends to reach into widget internals, causing drift and
forcing “big rewrites” later.

M2 locks a minimal, refactor-friendly contract for:

- where add-ons plug in (layout/paint/input/focus),
- what derived geometry they are allowed to consume,
- how theming/config flows in,
- and what determinism / hit-testing / focus rules are guaranteed.

## Scope

In scope (P1):

- A stable “add-ons host” surface for built-in overlays:
  - MiniMap (read-only derived internals + navigation hooks),
  - Controls (zoom/fit/reset + lock toggles),
  - Background (grid variants + theming tokens),
  - Toolbars/panels composition point (policy-light).
- A B-layer integration story: bind add-ons to external store/actions without forking the widget.

Out of scope (P1):

- New UX features (unless needed to validate the contract).
- New component policies (e.g. tooltip behavior, focus-trap policy) — those belong in ecosystem kits.
- Touch gesture parity and Web input layering (tracked separately in parity docs).

## Hard contracts (locked outcomes)

### 1) Add-ons are “policy-light”

- `fret-node` provides mechanisms (layout, hit-testing math helpers, focus routing hooks).
- Default policies (padding, row height, hover intent, dismiss behavior) must not live in `fret-node`.

### 2) Add-ons consume derived internals only

- Add-ons must not read mutable widget state to compute geometry.
- Add-ons must not trigger additional geometry rebuilds as part of their own logic.
- Add-ons should prefer `NodeGraphInternalsStore` snapshots and explicit view transforms.

Evidence (baseline):

- Internals store: `ecosystem/fret-node/src/ui/internals.rs`
- Derived geometry caching: `ecosystem/fret-node/src/ui/canvas/widget/derived_geometry/`

### 3) Deterministic hit-testing and focus routing

- Overlay hit-testing is deterministic (no hash-map iteration leaks).
- Focus traversal must reach overlays in a predictable order.
- Escape returns focus to the canvas root.

Evidence (existing):

- `ecosystem/fret-node/src/ui/canvas/widget/tests/overlay_minimap_controls_conformance.rs`
- `ecosystem/fret-node/src/ui/canvas/widget/tests/overlay_toolbars_conformance.rs`

### 4) Theming is explicit and bounded

- Background theming/config must be passed via explicit structs/fields (no global implicit lookups).
- Token surfaces are additive; backwards compatibility is maintained across minor releases.

## Exit criteria (Definition of Done)

This workstream is “done” when:

1) There is a stable public API surface for add-ons composition and configuration,
2) Conformance tests lock the hard contracts,
3) At least one demo uses the API without bespoke glue.

## Conformance suite (fast gates)

Recommended gates while iterating:

- `cargo nextest run -p fret-node overlay_`
- `cargo nextest run -p fret-node internals invalidation hit_testing`

Planned additions (M2):

- [x] Background variants + token surface conformance (lines/dots/cross + cache guardrails).
- [x] Controls B-layer wiring conformance (command binding overrides; no widget peeking).
- [x] MiniMap navigation conformance (click/drag-to-pan, deterministic mapping + view queue binding).
- [x] Toolbars composition conformance (pointer passthrough + focus release when hidden).
- [ ] Follow-up: per-editor theme token plumbing guidance (bridge `Theme` → `NodeGraphStyle`/`NodeGraphBackgroundStyle`).

## Code map (likely touch points)

- Add-ons: `ecosystem/fret-node/src/ui/overlays/`
- Panel host: `ecosystem/fret-node/src/ui/panel.rs`
- Overlay layout/hit helpers: `ecosystem/fret-node/src/ui/canvas/widget/overlay_layout.rs`,
  `ecosystem/fret-node/src/ui/canvas/widget/overlay_hit.rs`
- Internals/derived: `ecosystem/fret-node/src/ui/internals.rs`,
  `ecosystem/fret-node/src/ui/canvas/widget/stores/`

## Evidence (M2)

- Background style updates do not rebuild derived geometry:
  `ecosystem/fret-node/src/ui/canvas/widget/tests/background_style_conformance.rs`
- Background variants (dots/cross) emit stable scene ops in unit conformance:
  `ecosystem/fret-node/src/ui/canvas/widget/paint_grid.rs`
- Controls overlay supports B-layer command injection:
  `ecosystem/fret-node/src/ui/canvas/widget/tests/overlay_minimap_controls_conformance.rs`
- MiniMap supports B-layer navigation wiring via `NodeGraphViewQueue`:
  `ecosystem/fret-node/src/ui/canvas/widget/tests/overlay_minimap_controls_conformance.rs`
- Toolbars participate in overlay hit-testing (passthrough) and focus routing:
  `ecosystem/fret-node/src/ui/canvas/widget/tests/overlay_toolbars_conformance.rs`
