# Primitives Interaction Semantics Alignment v1 — Toast (Sonner + Radix store) (Audit Sheet)

Status: Active (workstream note; not a contract)

Baseline:

- shadcn/ui v4 “Toast” docs are deprecated in favor of Sonner (visual + API baseline).
- Underlying reusable outcomes are Radix Toast-shaped (store, viewport layer, swipe dismissal).

---

## Sources of truth (local pinned)

- Upstream docs note (toast deprecated → sonner):
  - `repo-ref/ui/apps/v4/content/docs/components/toast.mdx`
- Upstream shadcn Sonner surface:
  - `repo-ref/ui/apps/v4/registry/new-york-v4/ui/sonner.tsx`
- Upstream Radix primitive:
  - `repo-ref/primitives/packages/react/toast/src/*`

---

## Current Fret implementation anchors

- Radix-aligned toast primitives (store + viewport layer + swipe config):
  - `ecosystem/fret-ui-kit/src/primitives/toast.rs`
- shadcn Sonner wrapper (shadcn defaults: position, margins, width, duration shaping):
  - `ecosystem/fret-ui-shadcn/src/sonner.rs`
- Deprecated `toast` module re-export (taxonomy compatibility):
  - `ecosystem/fret-ui-shadcn/src/toast.rs`

Related tests/gates:

- Scripted repros:
  - `tools/diag-scripts/ui-gallery-toast-visible.json` (smoke: toast is visible and within window)

---

## Outcome model (what we must preserve)

State:

- per-window toast store (upsert-by-id)
- viewport placement (corner/center) + size clamps
- swipe gesture state (direction, threshold, max drag)
- timers/durations (pinned vs auto-close)

Reasons:

- open: request dispatched (`ToastRequest`)
- close: timer elapsed, swipe dismissed, close button/action, programmatic dismissal

Invariants:

- Upsert by id updates an existing toast (loading → success/error) without reordering surprises.
- Max-toasts limiting is stable and per-window.
- Swipe thresholds are deterministic and configurable (policy surface uses `Px`/`Duration`).

---

## Audit checklist (dimension-driven)

- [ ] `M` Document Sonner defaults vs “Radix store” substrate (what belongs where).
- [ ] `M/I` Document dismiss + swipe invariants and ensure they are policy-configured (not recipe magic).
- [ ] `G` Add a diag gate for a dismiss path (timer or swipe) once the UI gallery exposes stable anchors.

