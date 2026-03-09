# Action-First Authoring + View Runtime (Fearless Refactor v1) — Command-First Retained Seams Decision Draft

Status: draft, retained-seam decision
Last updated: 2026-03-09

Related:

- `docs/workstreams/action-first-authoring-fearless-refactor-v1/COMMAND_FIRST_WIDGET_CONTRACT_AUDIT.md`
- `docs/workstreams/action-first-authoring-fearless-refactor-v1/COMMAND_FIRST_INTENTIONAL_SURFACES.md`
- `docs/workstreams/action-first-authoring-fearless-refactor-v1/HARD_DELETE_ENDGAME_INDEX.md`
- `docs/workstreams/action-first-authoring-fearless-refactor-v1/HARD_DELETE_STATUS_MATRIX.md`
- `docs/workstreams/action-first-authoring-fearless-refactor-v1/HARD_DELETE_EXECUTION_CHECKLIST.md`

---

## Decision summary

Recommended decision:

- the repo should treat the remaining command-first surfaces as **split retained seams**, not as one
  generic leftover migration bucket,
- **mechanism/catalog-facing command seams** should remain command-centric by default,
- **advanced/internal app-facing residue** should remain allowed for now, but only as explicitly
  retained non-default surfaces,
- future cleanup should reopen only when a surface either:
  1. leaks back into the default authoring path, or
  2. becomes part of an explicit deprecation/removal plan.

In short:

- keep command-centric mechanism/catalog surfaces intentionally,
- keep advanced/internal residue explicit,
- do not schedule another broad command-first migration pass by default.

---

## Why this is the recommended choice

### 1) The broad public alias pass is already done

The main default-facing pressure has already been reduced:

- button-like app-facing widgets have action-first aliases,
- menu-family builders now expose `action(...)`,
- curated internal/app-facing menu residue has also been aligned where it matters.

That means the remaining command-shaped surface is no longer the same problem as before.

### 2) Some command-centric surfaces are actually the correct contract

Examples:

- command palette / command catalog rows,
- metadata-driven shortcut display and availability,
- conformance/compat tests that must assert legacy spellings directly.

These are not authoring mistakes.
They are the right place for command identity to stay explicit.

### 3) The remaining non-default app-facing residue is now a product-boundary question

Surfaces like:

- `DataTable` business-table action wiring,
- selected internal helpers,
- explicit advanced examples

are no longer evidence that the default public authoring story is broken.

They are evidence that some advanced domains still intentionally expose lower-level wiring.

### 4) The failure mode now is policy drift, not missing aliases

The real risk is no longer:

- “users cannot write action-first UI at all”.

It is:

- a new default-facing surface may accidentally reintroduce command-shaped naming,
- or reviewers may misread retained command seams as unfinished migration debt.

That is a docs/policy problem first.

---

## Surface classification

### Class A — Permanent mechanism/catalog seams

These should stay command-centric unless a future deeper architecture change happens:

- command palette / command catalog rows,
- metadata/shortcut/gating-driven command presentation,
- direct contract/conformance coverage for command routing.

Rule:

- do not force action-first renaming here just for symmetry.

### Class B — Intentionally retained advanced/internal seams

These may still look command-shaped today, but they are acceptable as long as they stay out of the
default path:

- `DataTable` business-table wiring,
- curated internal helper surfaces,
- advanced/reference demos where explicit command routing is part of the point.

Rule:

- keep them explicit,
- do not treat them as proof that the public default path regressed.

### Class C — Reopen only on leak or deprecation

These are not active migration targets today, but they should be reopened if either:

1. a new default-facing example/widget/doc starts teaching command-first naming again,
2. or the repo chooses to actually deprecate/remove a retained command-centric public API.

Rule:

- no broad sweep unless one of those triggers appears.

---

## What this means for the hard-delete track

The command-first lane should currently be read as:

- **maintenance mode with explicit retained seams**,
- not “the next delete candidate”,
- and not “one more generic alias pass”.

Practical consequence:

- `App::ui*`, compat runner, and `use_state` each needed dedicated execution notes because they have
  real future surface-reduction decisions.
- command-first retained seams do **not** need a delete playbook right now.
- they need a stable decision note that explains why they are intentionally kept and what would
  cause the repo to revisit them.

---

## Reopen triggers

Only reopen this track when one of the following is true:

1. a default-facing builder/doc/example leaks command-first naming back into the teaching path,
2. a retained public command-centric API is selected for actual deprecation/removal,
3. a new component family appears and lacks an action-first public spelling where one is clearly
   expected,
4. the repo changes the command catalog / keymap / metadata architecture enough that Class A
   surfaces are no longer naturally command-centric.

If none of these are true, the repo should leave this track in maintenance mode.

---

## Practical verdict

If the repo wants one short rule:

> Command-first cleanup is no longer a broad migration track.
> The remaining seams are either permanent mechanism/catalog contracts or explicitly retained
> advanced/internal surfaces, and should only be revisited on leak or deprecation.
