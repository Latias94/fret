# Primitives Interaction Semantics Alignment v1 — NavigationMenu (Audit Sheet)

Status: Active (workstream note; not a contract)

Baseline: Radix Navigation Menu outcomes (hover intent, delayed close, indicator + content coordination).

---

## Sources of truth (local pinned)

- Upstream shadcn recipe (v4 New York): `repo-ref/ui/apps/v4/registry/new-york-v4/ui/navigation-menu.tsx`
- Upstream Radix primitive: `repo-ref/primitives/packages/react/navigation-menu/src/*`

---

## Current Fret implementation anchors

- Primitive/policy: `ecosystem/fret-ui-kit/src/primitives/navigation_menu.rs`
- shadcn recipe: `ecosystem/fret-ui-shadcn/src/navigation_menu.rs`

Related tests/gates:

- `ecosystem/fret-ui-shadcn/tests/radix_web_primitives_state.rs`
- `ecosystem/fret-ui-shadcn/tests/web_vs_fret_overlay_chrome.rs`
- `ecosystem/fret-ui-shadcn/tests/web_vs_fret_overlay_placement.rs`
- `ecosystem/fret-ui-shadcn/tests/radix_web_overlay_geometry.rs`

Relevant shadcn-web goldens (existing examples):

- `goldens/shadcn-web/v4/new-york-v4/navigation-menu-demo-indicator.open.json`

Evidence (Radix web timeline parity gates):

- `ecosystem/fret-ui-shadcn/tests/radix_web_primitives_state.rs`:
  - `navigation-menu-example.navigation-menu.open-close.light`
- `ecosystem/fret-ui-shadcn/tests/radix_web_overlay_geometry.rs`:
  - `navigation-menu-example.navigation-menu.open-close.light`

Scripted repros:

- `tools/diag-scripts/ui-gallery-navigation-menu-hover-switch-and-escape.json`
  - Scenario: hover-open → hover-switch across triggers → Escape close.
  - Requires `FRET_DIAG=1` (reserved var; set in the parent shell when using `--launch`).

Test-id note (needed for scripts):

- `ecosystem/fret-ui-shadcn/src/navigation_menu.rs` now supports `NavigationMenuItem::trigger_test_id(...)`
  to stamp stable trigger anchors.
- `apps/fret-ui-gallery/src/ui/pages/navigation_menu.rs` stamps stable `test_id` anchors for triggers
  and representative links (for scripts and manual inspection).

---

## Outcome model (what we must preserve)

State:

- open state per trigger
- active item + indicator geometry state
- hover intent / delayed close group state (provider-like)

Invariants:

- Hover intent is stable and does not flicker across small pointer movements.
- Delayed close is semantic (`Duration`) and reasoned.
- Indicator geometry follows the active trigger and is gated against web goldens when possible.

---

## Audit checklist (dimension-driven)

- [ ] `M` Document hover intent + delayed close state machine and reasons.
- [ ] `M/I` Ensure delays are `Duration` and centralized (provider or shared policy).
- [x] `G` Add a diag script for: hover open → move across triggers → Escape close.
