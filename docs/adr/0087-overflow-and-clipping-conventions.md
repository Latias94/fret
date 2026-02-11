---
title: "ADR 0087: Overflow and Clipping Conventions (Surfaces, Focus Rings, Portals)"
---

# ADR 0087: Overflow and Clipping Conventions (Surfaces, Focus Rings, Portals)

Status: Accepted

## Implementation status

This ADR codifies behavior that is already implemented in the repository:

- Runtime `Overflow::Clip` paint + hit-test semantics: `crates/fret-ui/src/declarative/host_widget/layout.rs` and `crates/fret-ui/src/declarative/host_widget/paint.rs`.
- Focus ring rendering that can extend outside the surface chrome: `crates/fret-ui/src/paint.rs`.
- Component-layer surface chrome composition helper:
  `ecosystem/fret-ui-kit/src/declarative/chrome.rs` (`Pressable (visible) -> SurfaceChrome (clip) -> content`).
- Overlay “portal” installation via window-scoped overlay roots:
  `ecosystem/fret-ui-kit/src/overlay_controller.rs` + `ecosystem/fret-ui-kit/src/window_overlays/`.
- Representative shadcn overlays already use overlay roots (not in-tree escape): `ecosystem/fret-ui-shadcn/src/popover.rs`.

## Context

Fret exposes two related but distinct concepts:

- **Layout overflow (`LayoutStyle.overflow`)**: a paint + hit-test contract for rectangular/rounded
  clipping at the element boundary (ADR 0057, ADR 0063).
- **Text overflow (`TextOverflow`)**: a text-layout constraint-level choice (`clip` vs `ellipsis`)
  for single-line truncation (ADR 0059).

In shadcn/Radix-style UI, many components rely on a default mental model:

- Interactive **surfaces** (rounded panels, inputs, popovers) behave like `overflow-hidden`.
- Overlays are rendered via **portals** so they are not accidentally clipped by ancestor surfaces.
- Focus rings are drawn outside the surface and must remain visible.

Without an explicit convention, component implementations drift:

- some controls accidentally bleed content through rounded corners,
- some controls clip focus rings and reduce keyboard usability,
- some overlays are positioned in-tree and get clipped by ancestor surfaces.

We want stable conventions that align with:

- GPUI/Zed outcomes (`with_content_mask` / overflow mask) — `repo-ref/zed`
- Radix Primitives outcomes (Portal + DismissableLayer + FocusScope) — <https://github.com/radix-ui/primitives> (pinned locally; see `docs/repo-ref.md`)
- shadcn composition expectations — `repo-ref/ui`

## Decision

### 1) `LayoutStyle.overflow` is a contract, not a styling knob

`Overflow::Clip` must imply:

- paint is clipped to the element bounds (rounded if corner radii are present; ADR 0063),
- hit testing respects the same clip shape (ADR 0063),
- child drawing order is preserved (ADR 0009).

`Overflow::Visible` must imply:

- paint may extend outside the element bounds (but is still subject to ancestor clips),
- hit testing is not constrained by a clip at this element boundary.

### 2) Default overflow remains `Visible` for low-level primitives

The default for primitives (`Container`, `Pressable`, `Flex`, etc.) remains `Overflow::Visible`
(ADR 0057). This keeps composition flexible and avoids surprising clipping of focus rings,
shadows, and badges.

### 3) Components introduce clipping at the **surface chrome** boundary

Component-layer surfaces (shadcn-style panels/inputs/popovers/menus) should introduce clipping at
the boundary that owns the visual chrome (background + border + corner radius).

**Rule of thumb:** if a node draws a rounded background/border, it should default to
`Overflow::Clip` unless it is explicitly intended to be overflow-visible.

This matches the DOM/shadcn mental model of `rounded-* + overflow-hidden`.

### 4) Focus rings must not be clipped by the component itself

To keep focus-visible rings readable:

- The outer interactive wrapper (e.g. `Pressable`) should generally remain overflow-visible.
- The inner chrome container clips its *content* (`Overflow::Clip`) and owns corner radius.
- The focus ring is drawn outside the chrome (ADR 0061) and should remain visible unless an
  ancestor surface intentionally clips it.

This mirrors common web composition: an outer focus ring wrapper + inner `overflow-hidden` surface.

### 5) Overlays must be installed via portal/overlay roots

Overlay content (popover/menu/tooltip/dialog) must not rely on `Overflow::Visible` to escape
ancestor clips. Instead it must be installed into an overlay root / portal layer (ADR 0011,
ADR 0067).

Radix reference: overlay primitives are composed from `Portal` + `DismissableLayer` + `FocusScope`.

### 6) Scrolling viewports own overflow explicitly

Scrollable containers should clamp overflow explicitly at the viewport boundary:

- Cross-axis overflow is typically clipped (e.g. Radix Select viewport uses `overflow: hidden auto`)
- Main-axis overflow is scroll/virtualized by an explicit mechanism (ADR 0042, ADR 0070)

Do not rely on incidental ancestor clipping for scroll behavior.

## Consequences

- Components converge on predictable `overflow-hidden` outcomes without baking policy into
  `crates/fret-ui`.
- Focus-visible behavior remains accessible and consistent (ADR 0061).
- Overlays are robust to arbitrary ancestor surfaces, matching Radix/GPUI patterns (ADR 0067).

## Implementation Notes (non-normative)

- Provide component-layer helpers/recipes that create a "surface chrome" container with
  `Overflow::Clip` + corner radii + border/background tokens.
- Prefer the structure: `Pressable (visible) -> SurfaceChrome (clip) -> content`.

## Conformance checklist (practical)

- A rounded surface chrome that clips content must also clip hit testing for its descendants.
- Focus-visible rings must remain visible for standard controls (unless an ancestor explicitly clips them).
- A popover/menu rendered inside an `Overflow::Clip` ancestor must remain visible and hit-testable
  outside that ancestor by installing content into an overlay root (“portal”).
