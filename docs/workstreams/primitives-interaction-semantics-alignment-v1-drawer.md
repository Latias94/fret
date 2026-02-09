# Primitives Interaction Semantics Alignment v1 — Drawer (Audit Sheet)

Status: Active (workstream note; not a contract)

Baseline: shadcn/ui v4 `Drawer` outcomes (Vaul-style drag/snap semantics; Dialog-shaped overlay).

---

## Sources of truth (local pinned)

- Upstream shadcn recipe (v4 New York): `repo-ref/ui/apps/v4/registry/new-york-v4/ui/drawer.tsx`
- Upstream “semantic baseline”: Vaul-style drawer (drag + snap points), Dialog-shaped dismissal/focus.

Note: we do not currently pin Vaul sources under `repo-ref/`. When needed, capture the
high-value outcomes (drag thresholds + snap completion + dismissal) directly in this sheet and
gate them with diag scripts.

---

## Current Fret implementation anchors

- Underlay modal barrier + focus/dismiss substrate:
  - `ecosystem/fret-ui-kit/src/primitives/dialog.rs`
- shadcn recipe (drawer specifics: side defaults, drag thresholds, snap settle window):
  - `ecosystem/fret-ui-shadcn/src/drawer.rs`
- Sheet base (Drawer is currently modeled as a `Sheet` defaulting to `Bottom`):
  - `ecosystem/fret-ui-shadcn/src/sheet.rs`

Key implementation anchors (drag/snap):

- Drag thresholds and snap settle window:
  - `ecosystem/fret-ui-shadcn/src/drawer.rs` (`DRAWER_SNAP_SETTLE_TICKS`, swipe/edge thresholds)
- Pointer region and transform wiring:
  - `ecosystem/fret-ui-shadcn/src/drawer.rs` (`PointerRegionProps`, `RenderTransformProps`)

Related tests/gates:

- Scripted repros:
  - `tools/diag-scripts/ui-gallery-drawer-escape-focus-restore.json` (gate: escape close + focus restore)

---

## Outcome model (what we must preserve)

State:

- `open`
- `side` (Bottom/Left/Right/Top) and derived max-size clamp
- drag state (armed/dragging), current offset, snap point selection

Reasons:

- open: trigger press / programmatic
- close: escape / barrier outside press / drag dismissal / close button / programmatic

Invariants:

- Escape closes and restores focus to the trigger (unless configured otherwise).
- Drag gestures do not “leak” pointer events to the underlay while dragging.
- Snap points are normalized/deduped; settling window is deterministic.

---

## Audit checklist (dimension-driven)

- [ ] `M` Document drag/snap state machine (thresholds + completion semantics).
- [x] `G` Add a minimal diag gate: open → escape close → focus restore (baseline).
- [ ] `G` Add a drag gate: drag past threshold → dismiss OR snap to target point (one scenario).
