# Mechanism parity checklist (shadcn/Radix in a GPU-first renderer)

Use this checklist when a shadcn-aligned component “mostly matches” but still feels subtly wrong.
At this stage, mismatches are more likely to be **mechanism** (overlay routing, dismissal/focus,
hit-testing, responsive drivers) than CSS/tokens.

Goal: translate DOM/CSS assumptions into **explicit, deterministic contracts** suitable for a
GPU-first renderer.

## 1) Overlay family checklist (highest ROI)

Applies to: `Popover`, `DropdownMenu`, `ContextMenu`, `Select`, `Tooltip`, `HoverCard`, `Dialog`,
`Sheet`/`Drawer`, `NavigationMenu`.

### Dismissal and outside-press

- Outside-press is **observer-pass** and must not break normal hit-tested routing.
- Click-through defaults: confirm whether the overlay should consume pointer-down outside (Radix-like)
  or allow click-through (common for non-modal dismissables).
- Touch is not mouse:
  - outside-press often occurs on pointer-up with slop; ensure touch doesn’t perturb hover.

Start points:

- Contract + architecture: `docs/adr/0067-overlay-policy-architecture-dismissal-focus-portal.md`,
  `docs/adr/0069-outside-press-and-dismissable-non-modal-overlays.md`
- Mechanism entrypoints: `crates/fret-ui/src/tree/ui_tree_outside_press.rs`,
  `crates/fret-ui/src/tree/dispatch/window.rs`
- Policy wiring: `ecosystem/fret-ui-kit/src/window_overlays/`

### Focus trap / restore / auto-focus hooks

- Modal overlays must enforce an inert underlay (barrier semantics) and deterministic focus routing.
- Non-modal overlays must not steal focus unexpectedly; ensure close restores focus to the correct trigger.
- Nested portals: verify topmost overlay wins focus traversal, and close restores to the right owner.

Start points:

- Contract baseline: `docs/adr/0068-focus-traversal-and-focus-scopes.md`
- Policy primitives: `ecosystem/fret-ui-kit/src/primitives/focus_scope.rs`,
  `ecosystem/fret-ui-kit/src/primitives/dismissable_layer.rs`
- Runtime hooks substrate: `crates/fret-ui/src/action.rs`

### Placement / collision / constrained viewport sizing

- Confirm the reference stack:
  - placement vocabulary and outcomes follow Floating UI semantics (flip/shift/size/arrow),
  - not DOM layout side-effects.
- Constrained viewport outcomes are high-signal:
  - max-height clamping,
  - scroll buttons / scroll area behavior,
  - “available height” calculations for menus/listboxes.

Start points:

- Placement mechanism: `crates/fret-ui/src/overlay_placement/`
- Overlay chrome/placement parity tests: `ecosystem/fret-ui-shadcn/tests/web_vs_fret_overlay_placement.rs`,
  `ecosystem/fret-ui-shadcn/tests/web_vs_fret_overlay_chrome.rs`

### Safe-hover corridors (menus, nested submenus)

- Pointer grace areas prevent accidental dismissal while moving into submenus.
- Ensure this is policy-level (kit primitives), not runtime knobs.

Start points:

- Policy: `ecosystem/fret-ui-kit/src/primitives/safe_hover.rs`,
  `ecosystem/fret-ui-kit/src/primitives/pointer_grace_area.rs`

## 2) Hit-testing, transforms, clipping (GPU-first gotchas)

DOM assumes `transform` affects hit-testing the same way visuals move. In Fret, this is explicit.

- If a visual offset must move hit-testing and pointer coordinate mapping, use `RenderTransform`.
- If it is paint-only motion/offset, use `VisualTransform`, and gate pointer outcomes explicitly.
- Clipping stacks (`overflow: hidden` + radius) easily break:
  - outside-press detection,
  - hover suppression/occlusion,
  - focus rings (ring drawn but clipped).

Start points:

- Transform semantics: `crates/fret-ui/src/element.rs`
- Hit-testing: `crates/fret-ui/src/tree/hit_test.rs`
- Focus rings + pixel snapping: `docs/adr/0061-focus-rings-and-focus-visible.md`,
  `crates/fret-ui/src/pixel_snap.rs`

## 3) Responsive parity: pick the right driver (viewport vs container)

Most expensive refactors happen when responsiveness is driven by the wrong “truth”.

### Decide driver per decision point (non-negotiable)

- **Viewport / device shell** decision ⇒ environment query (ADR 0232)
  - Example: “drawer vs popover on small screens”
  - Use `ecosystem/fret-ui-kit/src/declarative/viewport_queries.rs`
- **Panel width** decision (docking splits, resizable panes) ⇒ container query (ADR 0231)
  - Use `ecosystem/fret-ui-kit/src/declarative/container_queries.rs`

Do not mix drivers inside the same subtree implicitly.
If both are needed, expose an explicit recipe-level knob and gate both modes.

### Hysteresis is required

- Use hysteresis around thresholds to prevent resize flicker.
- Expect 1-frame lag for container queries; design recipes so first-frame unknown is acceptable.

Start points:

- Container queries contract: `docs/adr/0231-container-queries-and-frame-lagged-layout-queries-v1.md`
- Environment/viewport snapshots: `docs/adr/0232-environment-queries-and-viewport-snapshots-v1.md`
- Helpers: `ecosystem/fret-ui-kit/src/declarative/container_queries.rs`,
  `ecosystem/fret-ui-kit/src/declarative/viewport_queries.rs`

### If upstream has no responsive rule

If shadcn doesn’t provide a responsive behavior, choose the most stable model for a custom renderer:

- Prefer **viewport-driven** rules for overlay shells and global interaction patterns.
- Prefer **container-driven** rules for editor panel content that lives inside docking/resizable panes.
- Always gate the decision with at least one:
  - invariant test, or
  - diag script that performs resize + asserts outcome.

## 4) Input modality and environment capability

If behavior differs between mouse vs touch vs pen, ensure the component reads capabilities from
environment snapshots rather than ad-hoc heuristics.

High-risk areas:

- hover-only affordances (tooltips/hover cards),
- touch slop for outside-press,
- reduced motion preferences.

Start points:

- Pointer queries helpers: `ecosystem/fret-ui-kit/src/declarative/pointer_queries.rs`
- Environment queries ADR: `docs/adr/0232-environment-queries-and-viewport-snapshots-v1.md`

## 5) Accessibility parity (outcomes, not DOM attributes)

Gate semantics snapshot outcomes for:

- roles/flags (disabled/selected/expanded/checked),
- relations (labelled-by / described-by / controls),
- composite widgets (active-descendant),
- collections (`pos_in_set` / `set_size`) for menu/listbox-like surfaces.

Start points:

- Schema: `crates/fret-core/src/semantics.rs`
- Helpers: `ecosystem/fret-ui-kit/src/primitives/trigger_a11y.rs`
- Acceptance checklist: `docs/a11y-acceptance-checklist.md`

## 6) Regression gates (prefer mechanism gates over visual goldens)

Default gate selection:

- State machine mismatch (dismiss/focus/keyboard/hover) ⇒ `tools/diag-scripts/*.json` + `fretboard diag run`
- Deterministic logic mismatch (placement math, breakpoints, invariants) ⇒ Rust unit/integration test
- Layout/style snapshot mismatch ⇒ web-vs-fret harness and existing goldens (only if it adds signal)

Existing high-signal in-tree harnesses (examples):

- Overlay placement/chrome: `ecosystem/fret-ui-shadcn/tests/web_vs_fret_overlay_placement.rs`,
  `ecosystem/fret-ui-shadcn/tests/web_vs_fret_overlay_chrome.rs`
- Radix timeline/state: `ecosystem/fret-ui-shadcn/tests/radix_web_primitives_state.rs`

