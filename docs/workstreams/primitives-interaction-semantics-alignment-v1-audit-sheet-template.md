# Primitives Interaction Semantics Alignment v1 — Audit Sheet Template

Status: Active (workstream note; not a contract)

This template is used by the per-component audit sheets under:

- `docs/workstreams/primitives-interaction-semantics-alignment-v1-*.md`

The intent is to standardize how we audit upstream interaction semantics (Radix/Base UI) against
Fret’s non-DOM architecture (overlay layers, semantics tree, runner-owned input dispatch).

Workstream:

- Overview: `docs/workstreams/primitives-interaction-semantics-alignment-v1.md`
- Progress matrix: `docs/workstreams/primitives-interaction-semantics-alignment-v1-matrix.md`

---

## Sources of truth (local pinned)

- Upstream shadcn recipe (v4 New York): `repo-ref/ui/apps/v4/registry/new-york-v4/ui/<component>.tsx`
- Upstream primitive source:
  - Radix: `repo-ref/primitives/packages/react/<primitive>/src/*`
  - Base UI: `repo-ref/base-ui/packages/*` (module depends on the pinned revision)

---

## Current Fret implementation anchors

- Primitive/policy: `ecosystem/fret-ui-kit/src/primitives/<component>.rs`
- shadcn recipe: `ecosystem/fret-ui-shadcn/src/<component>.rs`
- Related tests:
  - `ecosystem/fret-ui-shadcn/tests/radix_web_primitives_state.rs`
  - `ecosystem/fret-ui-shadcn/tests/radix_web_overlay_geometry.rs`
  - `ecosystem/fret-ui-shadcn/tests/web_vs_fret_overlay_placement.rs`
  - `ecosystem/fret-ui-shadcn/tests/web_vs_fret_overlay_chrome.rs`
- Scripted repros (when present): `tools/diag-scripts/*.json`

---

## Outcome model (what we must preserve)

Describe the primitive in terms of outcomes:

- **State**: `open`, `value`, highlight/active, “armed guards”, provider state, etc.
- **Events**: pointer down/up/move, key down, focus in/out, outside press, timer fired.
- **Reasons**: open/close/commit reasons (`OutsidePress`, `EscapeKey`, `FocusOut`, …).
- **Invariants**: things we will gate with tests/scripts.

Avoid translating DOM event wiring directly. Prefer “state machine + invariants”.

---

## Audit checklist (dimension-driven)

Mark each item with one of:

- `-` not audited
- `M` modeled (explicit outcomes + transitions written down)
- `I` implemented (policy exists in code)
- `G` gated (tests/scripts/goldens prevent regressions)

### Model

- [ ] `M` State machine + reasons + invariants written down.

### Policy (Trigger / Listbox / Commit) (if applicable)

- [ ] `M/I` TriggerPolicy outcomes and knobs.
- [ ] `M/I` ListboxPolicy outcomes (highlight/nav/typeahead/scroll) and knobs.
- [ ] `M/I` SelectionCommitPolicy outcomes and knobs (commit once, close-on-commit, misclick guards).

### Focus

- [ ] `M/I` Focus trap / restore / tab order outcomes; reason-aware restore policy.

### Dismiss

- [ ] `M/I` Escape/outside press/focus outside/scroll dismissal semantics.

### Pointer

- [ ] `M/I` Misclick guards, click-through vs barrier, capture/hover intent where relevant.

### Keys

- [ ] `M/I` Keyboard navigation and typeahead semantics.

### A11y (semantics)

- [ ] `M/I` Roles, expanded/controls relationships, active-descendant vs roving mapping (AccessKit).

### Placement / size

- [ ] `M/I` Anchored placement, collision, size clamping outcomes.

### Time

- [ ] `M/I` Any delays are `Duration` and represent a named semantic (not “magic ms”).

### Tests / gates

- [ ] `G` Unit tests for invariants where possible.
- [ ] `G` Diag scripts for multi-step timelines where needed.
- [ ] `G` Web goldens parity gates for layout/style outcomes where appropriate.

