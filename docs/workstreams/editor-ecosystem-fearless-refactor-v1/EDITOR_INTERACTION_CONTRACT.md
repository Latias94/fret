# Editor interaction and identity contract v1

Tracking doc:
`docs/workstreams/editor-ecosystem-fearless-refactor-v1/DESIGN.md`

Related docs:

- `docs/workstreams/editor-ecosystem-fearless-refactor-v1/EDITOR_COMPONENT_SYSTEM.md`
- `docs/workstreams/ui-editor-v1.md`
- `docs/workstreams/imui-ecosystem-facade-v3.md`

Status: Active design note (workstream contract, not an ADR)

Last updated: 2026-03-11

## Purpose

This note defines the minimum interaction and identity rules for reusable editor controls.

It exists to stop four common sources of drift:

- duplicated widget state across authoring frontends,
- accidental state sharing between repeated controls,
- unstable diagnostics selectors,
- and inconsistent edit-session behavior across the editor starter set.

This is an ecosystem-level contract.
It is intentionally narrower than an ADR and more specific than the high-level design note.

## Scope

This contract applies to:

- `ecosystem/fret-ui-editor`
- `ecosystem/fret-ui-editor::imui`
- editor-focused immediate facades built above `fret-imui`
- proof/demo surfaces that claim to exercise reusable editor behavior

It does not redefine low-level runtime mechanisms in `crates/fret-ui`.
If a new rule requires changing a hard runtime contract, that should escalate into an ADR.

## Core rules

### 1) One behavior model per reusable widget

Reusable editor widgets should have:

- one declarative implementation,
- one behavior model,
- optional multiple authoring syntaxes.

`imui` may change how a widget is called.
It must not change the widget's core interaction semantics.

### 2) `id_source` is identity, `test_id` is diagnostics

These two concepts must stay separate.

- `id_source` exists to give a widget stable identity across frames.
- `test_id` exists to give diagnostics and scripted repros stable selectors.

Rules:

- never use `test_id` as widget identity,
- never rely on callsite-only identity for loop-built widgets,
- prefer explicit `id_source` for any stateful or repeated control,
- if explicit identity is absent, derive from stable model identity rather than positional order.

Recommended fallback order for stateful editor controls:

1. explicit `id_source`
2. stable model identity
3. `(callsite, model identity)` when no stronger explicit id exists

Callsite-only identity is acceptable only for one-off controls that are not repeated and do not
carry cross-frame interaction state likely to collide.

### 3) Repeated controls must be keyed deliberately

Any control rendered in a loop or collection view must be keyed deliberately.

Examples:

- repeated `DragValue` rows
- dynamic property groups
- enum/list items
- repeated proof-surface parity columns

If the same control shape can appear multiple times, identity must come from data or explicit ids,
not from where the source line happens to sit today.

### 4) State helpers must follow the same discipline

This rule also applies to local harness/demo helpers.

If a demo or proof surface uses a helper backed by cross-frame state:

- the helper must be named or keyed,
- and the key must reflect semantic identity rather than screen position.

This prevents proof harnesses from hiding identity bugs that real editor surfaces will later hit.

## Response semantics

Promoted reusable editor controls should converge on a shared response vocabulary.

### Minimum interaction-state outcomes

Where a control exposes response state, the following meanings should stay consistent:

- `hovered`: pointer is meaningfully over the interactive affordance
- `focused`: keyboard focus is on the control or its primary editing surface
- `active` / `pressed`: the control is in an engaged pointer/drag/press state
- `open`: a popup/list/menu surface owned by the control is open
- `disabled`: interaction is intentionally unavailable

These are behavior meanings first and styling hooks second.

### Value-lifecycle outcomes

For editor controls that mutate values, the important lifecycle is:

- start
- live update
- commit
- cancel

Recommended meaning:

- `changed`: the value changed during the current interaction/update path
- `committed`: the interaction finished and the value should be treated as accepted
- `canceled`: the interaction was abandoned and the pre-edit value restored

Not every current response type must expose all of these as separate fields today.
But promoted editor controls should behave as if this lifecycle exists, and new APIs should not
contradict it.

## Numeric edit-session contract

Numeric editing is the highest-signal editor interaction and should remain stable.

### Start

Numeric edit starts when the user:

- begins scrubbing with pointer input, or
- enters a typed-edit path explicitly (for example by double-click or another explicit action).

### Live updates

During scrubbing or typing:

- value updates may be emitted continuously,
- visual state should make it clear that the control is actively editing,
- undo integration should treat the session as one logical edit when possible.

### Commit

Recommended commit points:

- pointer up after a scrub session,
- Enter in typing mode,
- blur in typing mode when the surface treats blur as accept.

### Cancel

Escape should restore the pre-edit value for promoted numeric editing flows.

This is one of the few behaviors that should be treated as sticky once the editor starter set
depends on it.

### Modifier defaults

Default editor expectations:

- `Shift` -> slower / precision mode
- `Alt` -> faster / coarse mode

These may remain configurable, but the default mapping should not drift casually across controls.

## Keyboard and focus rules

Reusable editor controls should follow explicit keyboard/focus defaults rather than per-widget
surprises.

Rules:

- keyboard focus must remain distinct from pointer hover,
- focus treatment must remain visible even on compact controls,
- Enter/Escape behavior should be documented for controls that enter edit sessions,
- popup-owning controls should define whether focus remains on trigger, moves into content, or
  restores on close,
- command/keybinding behavior should not fire through active text input or IME composition
  accidentally.

## `test_id` conventions

`test_id` is a diagnostics API.
Treat any id used by scripts as a stable contract unless intentionally renamed with script updates.

### Recommended naming pattern

Use semantic, namespaced ids:

- `editor.drag_value.<field>`
- `editor.numeric_input.<field>`
- `editor.property_row.<path>`
- `editor.property_group.<path>`
- `editor.color_edit.<field>`
- `editor.enum_select.<field>`

Rules:

- prefer field/path meaning over position indexes,
- prefer one stable root affordance per interactive surface,
- add secondary ids only when scripts genuinely need them,
- avoid geometry-based selectors when a semantic `test_id` is possible.

## Diagnostics and proof requirements

When promoting a new reusable editor control, leave behind:

1. one proof surface in a promoted demo or gallery,
2. one stable `test_id` strategy,
3. at least one focused regression gate for the most failure-prone behavior.

High-priority gate targets:

- edit-session commit/cancel
- repeated-control identity correctness
- popup open/close correctness
- keyboard focus traversal for composites

## Promotion checklist

Before a reusable editor control is considered stable enough for broad reuse, confirm:

1. identity is explicit and safe under repetition,
2. `test_id` is semantic and stable,
3. declarative and `imui` paths share the same core behavior,
4. edit-session semantics are documented if the control edits values,
5. at least one proof/gate artifact exists.

If any of these are missing, the control should remain experimental or app-local longer.
