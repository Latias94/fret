# Workspace Shell TabStrip (Fearless Refactor v1) — Decisions (v1)

Status: **Accepted** (per maintainer direction).

This file records the explicit v1 decisions so implementation does not churn.

## D1: Pinned model — `pinned_tab_count` boundary vs per-tab pinned flag

**Decision:** start with **`pinned_tab_count`** (Zed-like boundary).

Why:

- Matches editor UX expectations (pinned is “left segment”).
- Simplifies reorder + drop targets (one boundary, easy invariants).
- Makes a “separate pinned row” optional without changing the underlying meaning.

Upgrade path:

- We can later migrate to per-tab pin flags if we need non-contiguous pinned groups.

## D2: Overflow dropdown list — “overflowed only” vs “all tabs”

**Decision:** **overflowed-only** list (dockview-like) for v1.

Why:

- Fast path for the common pain: selecting a tab you cannot see.
- Keeps the list short; typeahead/search can be added later if needed.

Optional later:

- “All tabs” list with grouping and typeahead (VS Code-like).

## D3: Click focus policy — should clicking a tab move keyboard focus?

**Decision:** clicking a tab **should not steal content focus** by default.

Why:

- Editor shells rely on stable focus for typing; tab activation is a view change, not a focus change.
- Aligns with “chrome interactions are focus-neutral” principle.

Escape hatches:

- Provide a command “Focus tab strip” for keyboard users.
- When the content is empty/unfocused, allow tab strip to become focus owner.

## D4: Keyboard nav semantics — roving focus vs “selection without focus”

**Decision:** **roving focus + activation on roving change** (APG-aligned).

Details:

- Arrow keys move the roving active item.
- Activation mode:
  - v1 default: automatic (activates as roving moves) for editor tab strips.
  - optional: manual (Enter/Space) if a use-case appears.

## D5: Preview tab semantics — do we implement it in v1?

**Decision:** **Yes**, modeled after Zed/VS Code.

Minimum contract:

- At most one preview tab per pane.
- Opening a previewable item reuses/replaces the existing preview tab slot.
- “Commit” turns preview → normal (e.g. after edit or explicit pin/keep).

Opt-out:

- Preview can be disabled globally; behavior becomes “always open normal tab”.

## D6: Drag-to-split — where does the split policy live?

**Decision:** split is **policy-owned** by workspace/docking; the kernel emits **intent only**.

Why:

- Avoids bleeding shell policy into shared logic.
- Different shells may disallow split (or vary thresholds/hysteresis).

## D7: Edge auto-scroll during drag reorder

**Decision:** implement in v1.

Why:

- Without it, reorder with many tabs is frustrating even if overflow list exists.
- Mechanically straightforward if we already have tab rects + viewport + scroll handle.

## D8: Drop targets — do we add explicit “end of strip” targets?

**Decision:** Yes, add explicit end targets.

Why:

- Zed and gpui-component both rely on “empty space” / “drop target” nodes for robust “after last tab”.
- It reduces reliance on fuzzy hit-testing near the last tab’s right edge.

## D9: Cross-pane drag — should we allow “drop into header space” (not on a tab)?

**Decision:** Yes.

Why:

- dockview uses “header_space” as a first-class drop surface.
- Enables “move to end” without requiring a specific tab as target.

## D10: Kernel placement — new crate vs module + shared primitives

**Decision:** keep the v1 kernel as a **module** inside `ecosystem/fret-workspace` (`tab_strip/kernel.rs`).

Why:

- The kernel currently encodes editor/workspace chrome policy (pinned boundary, preview semantics).
- A new crate would be premature surface area and would likely churn as we learn.

Reuse strategy:

- When docking needs shared pieces, extract **generic geometry/math** into existing `ecosystem/fret-dnd`
  (headless primitives), and keep workspace/docking policy wrappers in their own crates.

## D11: Reference selection — Zed vs dockview vs gpui-component

**Decision:** treat each upstream as a **reference for a specific slice**, not as a single “one true
port”.

- **Zed (`repo-ref/zed`) is the primary reference for editor semantics**:
  - pinned boundary + optional separate pinned row,
  - preview tabs + commit rules,
  - drop targets + drag-to-split outcomes,
  - focus neutrality rules for chrome.
- **dockview (`repo-ref/dockview`) is the primary reference for overflow pipeline + header-space UX**:
  - overflow membership computation and “more tabs” listing,
  - header-space drop surfaces as first-class targets.
- **gpui-component (`repo-ref/gpui-component`) is the reference for GPUI wiring patterns**, but it is
  not a complete editor semantics reference (no pinned/preview/MRU baseline).

Why:

- Editor-grade tab bars are *outcome-heavy*: copying one source wholesale tends to import unrelated
  constraints (DOM vs GPU scene, CSS layout vs measured rects).
- Using a per-capability reference keeps contracts stable and reviewable.

Practical rule:

- If a behavior affects editor workflows (pinned/preview/split/focus), use Zed outcomes first.
- If a behavior is about overflow UX and dropdown list patterns, use dockview outcomes first.
- If a behavior is about implementation shape in GPUI-style UI, consult gpui-component for patterns.
