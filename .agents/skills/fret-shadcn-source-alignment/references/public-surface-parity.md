# Public surface parity (prop-driven upstream vs model-centered Fret ports)

Use this note when a shadcn/Radix-aligned component technically works, but app authors are pushed
into avoidable `Model<T>` boilerplate that does not exist in upstream composition.

Typical symptom set:

- small local collections fall back to per-row `Model<bool>` / `Model<Vec<_>>`
- a widget can only be authored via model handles even when upstream is prop-driven (`checked`,
  `value`, `open`, `default*`, callback)
- app code replaces the intended widget with a surrogate `Button` just to stay action-first
- the component is visually correct, but its authoring surface still feels unlike shadcn

## First question: is this a runtime contract problem or a public surface drift?

Before changing runtime mechanisms, ask:

1. What is the upstream public API shape?
   - prop-driven snapshot (`checked`, `value`, `open`)
   - controlled/uncontrolled pair (`checked` + `defaultChecked` + callback)
   - purely imperative or async bridge
2. What is the current Fret public API shape?
   - model-only
   - model + uncontrolled helper
   - snapshot + action/event path
3. Which part actually hurts authoring density?
   - internal implementation may stay model-backed
   - the public surface can still expose a narrower snapshot/action path

Rule of thumb:

- Do **not** generalize immediately to a broad `IntoModel<T>` conversion story.
- Prefer a **narrow, source-aligned surface** that matches the upstream component contract while
  preserving Fret internals.

If curated docs/examples still have to spell `UiIntoElement`, `UiChildIntoElement`,
`UiHostBoundIntoElement`, or `UiBuilderHostBoundIntoElementExt`, classify that as
**conversion-surface drift** rather than a component-parity success. The owning follow-up tracker
is:

- `docs/workstreams/into-element-surface-fearless-refactor-v1/DESIGN.md`
- `docs/workstreams/into-element-surface-fearless-refactor-v1/TARGET_INTERFACE_STATE.md`

Target guidance:

- app-facing helpers teach `Ui` / `UiChild`,
- reusable generic helpers converge on the unified component conversion trait,
- advanced/manual-assembly reusable helpers converge on `IntoUiElement<H>` instead of
  `UiChildIntoElement<H>`,
- raw `AnyElement` remains explicit for justified raw seams only.

## Preferred adaptation patterns

### 1) Text / IME widgets

Examples: `Input`, `Textarea`

- These often need model-backed internals for IME, selection, and editing semantics.
- Good adaptation: a narrow bridge like `IntoTextValueModel`.
- Avoid turning the entire ecosystem into generic `IntoModel<T>`.

### 2) Discrete toggle / choice widgets

Examples: `Checkbox`, sometimes `Switch`, `Toggle`

- Upstream is often prop-driven for the rendered value and event-driven for mutations.
- Prefer adding a snapshot constructor plus action/event hooks.
- Example shape:
  - `from_checked(...)`
  - `action(...)`
  - `action_payload(...)`

This keeps small view-owned collections in plain local state while preserving typed actions.

### 3) Disclosure / overlay roots

Examples: `Popover`, `Dialog`, `Collapsible`

- First check whether an uncontrolled path already exists.
- If yes, do not allocate local models in app code just to satisfy a controlled constructor.
- Promote controlled models only when external synchronization is truly required.

### 4) Selection widgets with outward synchronization

Examples: data-heavy selects, complex form fields, widgets with shared app state

- Explicit `Model<T>` may still be the correct public boundary.
- The goal is not to remove all model-backed APIs.
- The goal is to stop forcing them onto simple local-authoring scenarios.

## A11y / label parity checklist

When you add a snapshot/action path, verify that you did not regress the non-visual contract:

- role + checked/selected/expanded semantics still match upstream outcomes
- `control_id` / label forwarding still works, or the remaining gap is documented explicitly
- disabled/availability behavior remains consistent with command gating
- keyboard activation still matches the same pressable policy

## Minimum regression pack

For a public-surface parity fix, leave behind:

1. one focused unit test for semantics/state
2. one migrated example proving the authoring win in real code
3. one note in the workstream/docs describing what remains unresolved

## Escalation rule

If the component still needs per-row models after this checklist, decide which of these is true:

- the component genuinely needs a shared/runtime-visible state boundary
- the component is missing a narrow snapshot/action path
- the label/control contract is incomplete for the action-only path
- the issue is app-builder guidance, not component parity
