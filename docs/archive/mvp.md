> Archived: this plan is kept for history; prefer `docs/roadmap.md` + `docs/shadcn-declarative-progress.md` for active work.

# MVP Plan (Historical Overview Snapshot)

This file is a historical snapshot and is not maintained as an active execution queue.

- Historical expanded plan (details, definitions, status notes): `docs/archive/mvp/active-plan.md`
- Completed stage definitions: `docs/archive/mvp-archive.md`
- Long-horizon priorities and refactors: `docs/roadmap.md`
- Runtime contract gap list (ADR 0066 conformance): `docs/archive/backlog/runtime-contract-gap.md`

## Historical “What’s Next” Snapshot

- Contract note: the `fret-ui` runtime contract surface is now locked (ADR 0066). New runtime public APIs must name an authoritative reference and land with tests before broadening usage.
- Execution note: we follow a “foundation-first, component-validated” loop (Plan C). See `docs/foundation-first-workflow.md`.

- MVP 49 (in progress): Make the declarative component authoring model (ADR 0028 + ADR 0039) the primary, end-to-end usable path (not just a state store): `IntoElement` + `Render`/`RenderOnce` + composition ergonomics, plus a clear `render_root(...)` contract (when it must be called, and what it guarantees).
- MVP 50 (in progress): Consolidate virtualization around composable, declarative row content (GPUI-style). Runtime contract is now TanStack-aligned (ADR 0070); remaining work is migrating surfaces off fixed-schema runtime rows (`VirtualListRow { text/secondary/trailing... }`) and retiring the legacy path where feasible.
- MVP 51 (in progress): Tighten the framework/components boundary by moving “standard surfaces” (popover/dialog/menu/tooltip/toast/command palette/menubar) fully into the components layer, keeping `fret-ui` as runtime substrate + performance primitives. `fret-components-ui` remains the reusable infrastructure, while `fret-components-shadcn` becomes the shadcn/ui (v4) aligned naming/taxonomy surface. Compatibility retained widgets stay behind `fret-ui`’s `retained-widgets` feature (`crates/fret-ui/src/primitives/*`) until removal.
- MVP 68 (done): Eliminate interaction policy leaks from `fret-ui` before scaling the component surface.
  - Removed runtime “shortcut model writes” (pressable toggle/set variants, dismiss-by-model, roving/typeahead coupling) per ADR 0074.
  - Components must express policy via action hooks + component-owned headless helpers (`fret-components-ui` / `fret-components-shadcn`).
- MVP 69 (done): Docking layering cleanup (B route): move docking UI/policy out of `fret-ui`.
  - Keep dock graph/ops/persistence in `fret-core` (stable contract).
  - Add a generic internal-drag routing hook in `fret-ui` so docking can preserve tear-off/cross-window drags.
  - Add a feature-gated retained bridge (`fret-ui/unstable-retained-bridge`) so the docking UI can move without a rewrite.
  - Move viewport overlay drawing/policy (gizmos, marquee, selection rect) to `fret-editor` / app-layer code (ADR 0027).
    Docking exposes `DockViewportOverlayHooks` to let the app paint overlays without docking owning overlay shapes.
- MVP 66 (in progress): Make model observation and invalidation propagation (ADR 0051) an "Accepted + tested" contract.
  - Ensure changed-model propagation invalidates all observing nodes across all windows deterministically.
  - Remove remaining "manual invalidate all panels" glue from demo/components once conformance is locked.
- MVP 60 (done): Rounded clipping / `overflow-hidden` semantics (shadcn-critical).
  - Landed: `SceneOp::PushClipRRect` (ADR 0063) + renderer soft clipping (AA) + UI hit-test parity.
- MVP 61 (done): Declarative layout performance hardening (Taffy integration).
  - Landed: per-solve measure memoization + persistent container-owned Taffy trees (ADR 0076).
- MVP 62 (next): Overlay behavior + placement contract (APG/Radix/Floating UI alignment).
  - Lock dismissal/focus/portal rules for popover/menu/tooltip/dialog/sheet (Radix-like outcomes; ADR 0067).
  - Lock modal-aware Tab traversal baseline (`focus.next`/`focus.previous`) to keep overlay focus policies consistent (ADR 0068).
  - Implement stable anchored positioning with flip/shift/size/offset (P0) and deterministic results across windows (Floating-like algorithms). Arrow is deferred to P1 (ADR 0066).
  - See: `docs/reference-stack-ui-behavior.md`.
- MVP 63 (next): Unify scroll ergonomics around a single handle model (GPUI-like).
  - Introduce a consistent `ScrollHandle`-style substrate: offset, scroll-to, scrollbar, and scroll-into-view primitives.
  - Ensure `ScrollArea` + `VirtualList` composition shares one contract surface (no divergent wheel/drag behavior).
- MVP 65 (done): Lock a GPUI-style frame request contract to avoid "mode toggle" sprawl.
  - One-shot `request_frame` + `request_animation_frame`, plus refcounted RAII `begin_continuous_frames` leases (ADR 0034).
- MVP 67 (next): Fix initial render / invalidation ordering regressions ("doesn't draw until hover").
  - Add a small regression harness in `fret-demo` that asserts the first frame draws without requiring pointer motion.
  - Audit the render lifecycle contract (ADR 0015 / ADR 0028) and make initial invalidation/redraw deterministic.
- MVP 64 (next): APG-aligned keyboard/focus patterns as reusable component-layer recipes.
  - Roving focus, typeahead, Home/End/PageUp/PageDown patterns for menus/listbox/combobox/tree.
  - Expand semantics roles/flags where needed to keep future accessibility bridge viable (ADR 0033).
  - Lock cmdk-style command palette semantics for accessibility:
    - keep focus in the text input while navigating results,
    - expose the active result via an `active_descendant` semantics association (ADR 0073; Phase A done, policy wiring next),
    - avoid virtualization until an AT-facing virtualization strategy is defined.
- MVP 59 (next): Eliminate Tailwind-like “layout no-ops” and harden composition semantics by splitting style patches into `ChromeRefinement` vs `LayoutRefinement`, standardizing token/key vocabulary, and making `merge` semantics match Tailwind-style edge accumulation (e.g. `mt-*` + `ml-*` should compose without dropping edges). Layout refinements must apply only in the declarative path (or explicit layout wrappers), never silently in retained widgets.
- MVP 55 (next): Expand style patch → layout bridging so Tailwind-like recipes can drive declarative layout without widget-local magic numbers: map additional sizing/flex/overflow knobs into declarative `LayoutStyle` (beyond the current minimal subset).
- MVP 58 (next): Implement Tailwind layout primitives at the runtime vocabulary level (margin, position/inset, grid, aspect-ratio) per ADR 0062, so shadcn-style layouts (badges, input icons, simple grids) are expressible without bespoke components.
- MVP 56 (in progress): Add missing shadcn “polish primitives” as reusable contracts (not per-widget hacks): shadow/elevation baseline is implemented (ADR 0060); focus ring baseline is implemented (ADR 0061, including a minimal focus-visible heuristic); richer scroll ergonomics is partially implemented (vertical scrollbar thumb drag + track paging); remaining: horizontal/bidirectional scroll and scroll-to-child.
- MVP 57 (done): Declarative icon helper (glyph-based) so shadcn-style list/menu rows can compose leading/trailing icons without falling back to retained widgets.

## Current Status (Snapshot)

See `docs/roadmap.md` for current priorities and milestones.
