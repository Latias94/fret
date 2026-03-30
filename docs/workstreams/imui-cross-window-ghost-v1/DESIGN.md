# imui cross-window ghost v1 - design

Status: Historical reference (closed closeout record; successor lane is `docs/workstreams/imui-shell-ghost-choreography-v1/`)

Last updated: 2026-03-30

Status note (2026-03-30): this document remains useful for the shipped generic cross-window ghost
baseline, but the current landed guidance now lives in
`docs/workstreams/imui-cross-window-ghost-v1/CLOSEOUT_AUDIT_2026-03-30.md` and the current
follow-on planning now lives in `docs/workstreams/imui-shell-ghost-choreography-v1/DESIGN.md`.
References below to active cross-window planning should be read as closed for the generic
`current_window` transfer slice unless explicitly marked as deferred.

Related:

- `docs/workstreams/imui-stack-fearless-refactor-v1/DESIGN.md`
- `docs/workstreams/imui-sortable-recipe-v1/CLOSEOUT_AUDIT_2026-03-30.md`
- `docs/workstreams/imui-drag-preview-ghost-v1/CLOSEOUT_AUDIT_2026-03-30.md`
- `docs/workstreams/imui-cross-window-ghost-v1/M1_CONTRACT_FREEZE_2026-03-30.md`
- `docs/workstreams/imui-cross-window-ghost-v1/CLOSEOUT_AUDIT_2026-03-30.md`
- `docs/workstreams/imui-shell-ghost-choreography-v1/DESIGN.md`

## Purpose

This workstream is the direct follow-on after the same-window ghost lane closed.

That lane answered one narrow question:

- how Fret should ship a source-authored drag preview ghost inside one window.

It explicitly did **not** answer the next question:

> how should Fret keep a single coherent ghost contract once drag hover leaves the source window
> and enters another Fret window, viewport shell, or tear-out surface?

This lane exists to lock that next contract before implementation drift starts.

## Current assessment

Fret now has three important facts at the same time:

- the runtime already supports cross-window drag routing,
- the `imui` seam already exposes source activity plus current drag position,
- and the recipe layer already owns same-window source ghost policy.

What remains unresolved is not same-window painting.
What remains unresolved is **cross-window ownership and arbitration**.

The unresolved questions are concrete:

- which window paints the ghost once hover leaves the source window,
- when the source window should hide or keep its ghost,
- whether the hovered window renders the same source-authored visual or a transferred descriptor,
- how duplicate ghosts are avoided,
- how docking / workspace shells arbitrate z-order and ownership,
- and what fallback path exists when multi-window capability is absent.

## Why this lane should exist

Without an explicit lane, cross-window ghost behavior will likely drift into one of four wrong
outcomes:

- ad hoc shell-specific overlay glue in demo/app code,
- preview transfer logic leaking into `drag_source(...)`,
- shell choreography accidentally hardened inside `fret-ui-kit::recipes`,
- or an under-specified runtime widening pass before the actual owner split is clear.

This lane exists to prevent all four.

## Goals

### G1 - Freeze the owner split for cross-window choreography

This lane must answer which layer owns which part of the next problem:

- `fret-ui-kit::imui` should remain limited to drag observation seams,
- `fret-ui-kit::recipes` may own shell-agnostic cross-window preview policy only if that policy is
  truly generic,
- docking / workspace shell choreography belongs in shell-aware layers, not in `imui`,
- and runtime crates should change only if a real mechanism gap is proven.

### G2 - Define one coherent cross-window ghost ownership rule

The next stable contract must answer:

- where the ghost is rendered while crossing windows,
- whether ownership follows the pointer or remains source-owned,
- and how that rule behaves when hover temporarily resolves to no eligible Fret window.

### G3 - Preserve source-authored preview meaning

The lane should preserve the architectural win from the same-window slice:

- the source still owns payload meaning and preview intent,
- the next contract should not turn into a global preview skin registry,
- and any transfer surface should carry only the minimum descriptor needed to preserve that meaning.

### G4 - Keep fearless-refactor posture explicit

If the current same-window helper shape turns out to be the wrong base for cross-window transfer:

- rename it,
- split it,
- or move it,

instead of carrying compatibility aliases.

## Non-goals

- Native OS drag image / external application drag preview in v1.
- Multi-item aggregate preview in v1.
- General-purpose preview serialization across process boundaries.
- Reopening the same-window lane unless a hard contract bug is found.
- Putting shell-specific docking choreography directly into `fret-ui-kit::imui`.

## Non-negotiable boundaries

| Layer | Owns | Must not own |
| --- | --- | --- |
| `crates/fret-ui` / `fret-runtime` | drag session routing, window identity, overlay-root mechanisms | shell policy, recipe chrome defaults, workspace choreography |
| `ecosystem/fret-ui-kit::imui` | read-only drag observation seams | ghost transfer policy, shell ownership rules |
| `ecosystem/fret-ui-kit::recipes` | reusable shell-agnostic preview packaging if a generic cross-window policy exists | docking/workspace-specific ownership arbitration |
| `ecosystem/fret-docking` / shell layers | panel/viewport/dock-specific choreography and arbitration | generic runtime drag contracts |
| apps / demos | proof surfaces and product semantics | framework owner decisions disguised as local glue |

## Decision snapshot

### 1) The next problem is choreography, not chrome

The same-window lane already closed:

- pointer-follow layout,
- click-through overlay behavior,
- and source-authored preview content.

So the next lane should treat styling as solved and focus on ownership/arbitration semantics.

### 2) The next proof should start from first-party multi-window surfaces

The smallest meaningful proof is no longer a single-window interaction test.

The preferred proof surfaces are:

- the existing `imui_editor_proof_demo` main/aux window pair,
- and, if needed later, a docking/tear-out shell proof once the ownership rule is frozen.

### 3) Shell choreography should not be assumed generic too early

Cross-window drag preview looks generic from far away, but shell-specific cases can change the
actual owner rule:

- hovered auxiliary inspector window,
- tear-out dock panel,
- embedded viewport shell,
- transient no-hover gap while moving between windows.

This means the lane must first prove which parts are generic and which parts stay shell-owned.

### 4) Capability fallback must be part of the contract, not an afterthought

The repo already models environments where multi-window is unavailable.

So this lane must explicitly state how the cross-window ghost contract degrades when:

- multi-window is disabled,
- hover detection across windows is unreliable,
- or the platform cannot guarantee cross-window positioning fidelity.

### 5) Phase B is now frozen around `current_window` paint ownership

The accepted M1 contract freeze is:

- semantic ownership stays at the source,
- generic paint ownership follows `DragSession.current_window`,
- and `moving_window` / `window_under_moving_window` remain shell-specific choreography hints.

See:

- `docs/workstreams/imui-cross-window-ghost-v1/M1_CONTRACT_FREEZE_2026-03-30.md`

### 6) Cross-window rendering uses recipe-owned descriptor transfer

The accepted transfer rule is:

- the source authors preview meaning,
- the recipe layer may publish a minimal transferred descriptor keyed by drag session,
- and the hovered/current window renders from that descriptor without turning runtime into a
  preview-policy registry.

### 7) Capability fallback consumes runner truth and adds no recipe heuristics

The accepted fallback split is:

- `Reliable`: full generic transfer contract,
- `BestEffort`: follow runner-selected `current_window` on a best-effort basis only,
- `None` / single-window: stay on same-window ghost behavior.

## Target architecture

### `ecosystem/fret-ui-kit::imui`

Default assumption:

- no new preview-policy entry points,
- at most small read-only identity/position/window observation seams if the first proof proves they
  are needed,
- and no shell-specific arbitration hooks.

### `ecosystem/fret-ui-kit::recipes`

Accepted Phase B baseline:

- a shell-agnostic cross-window ghost helper may exist here,
- but it must be limited to generic transferred preview ownership keyed by `current_window`,
- and it must not absorb moving-window / docking / tear-out choreography.

### Shell-aware layers

Expected likely owners for non-generic behavior:

- `ecosystem/fret-docking`
- workspace shell layers
- first-party multi-window app surfaces during proof-first exploration

## Proof and regression requirements

Minimum package expected before implementation is considered reviewable:

- one accepted M1 contract freeze record for generic vs shell-specific choreography,
- one first-party multi-window proof surface,
- one interaction/regression gate that can observe ghost ownership moving across windows,
- and one explicit capability-fallback note for single-window / wasm-like environments.

## Exit criteria

This lane can close only when the repo can answer all of these clearly:

- who owns ghost rendering while the pointer is over another Fret window,
- how duplicate ghosts are prevented,
- which parts of the behavior are generic recipe policy,
- which parts remain shell choreography,
- and how the contract degrades when cross-window capability is not available.
