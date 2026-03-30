# imui shell ghost choreography v1 - design

Status: closed (historical closeout evidence on 2026-03-30)

Last updated: 2026-03-30

Related:

- `docs/workstreams/imui-stack-fearless-refactor-v1/DESIGN.md`
- `docs/workstreams/imui-drag-preview-ghost-v1/CLOSEOUT_AUDIT_2026-03-30.md`
- `docs/workstreams/imui-cross-window-ghost-v1/CLOSEOUT_AUDIT_2026-03-30.md`
- `docs/workstreams/imui-shell-ghost-choreography-v1/CLOSEOUT_AUDIT_2026-03-30.md`
- `docs/workstreams/imui-shell-ghost-choreography-v1/M1_CONTRACT_FREEZE_2026-03-30.md`
- `docs/workstreams/docking-hovered-window-contract-v1/`
- `docs/workstreams/docking-multiviewport-arbitration-v1/`

## Purpose

This workstream is the direct follow-on after the generic cross-window ghost lane closed.

That lane answered one narrow question:

- how Fret should ship one shell-agnostic cross-window ghost baseline keyed by
  `DragSession.current_window`.

It explicitly did **not** answer the next question:

> how should docking/workspace/tear-out shells arbitrate ghost ownership, visibility, and
> continuity when generic `current_window` truth is necessary but not sufficient?

This lane exists to lock that shell-aware contract before shell-local glue starts to drift.

## Current assessment

Fret now has four important facts at the same time:

- the runtime already routes multi-window drag state,
- the recipe layer now ships a generic cross-window ghost transfer baseline,
- shell-aware surfaces can observe richer drag/window hints such as moving-window relationships,
- and editor-grade UX still needs choreography that is not captured by generic `current_window`
  paint ownership alone.

What remains unresolved is not generic transfer.
What remains unresolved is **shell-aware choreography**.

The unresolved questions are concrete:

- whether docking/workspace shells ever need to pin, delay, or suppress ghost ownership during
  tear-out handoff,
- how temporary no-hover gaps should behave when moving between viewport shells,
- whether shell layers need their own ghost host or only a wrapper around the generic recipe store,
- how shell z-order and viewport layering should arbitrate the ghost,
- and what diagnostics/gates best prove the shell-specific rules.

## Why this lane should exist

Without an explicit lane, shell-aware ghost behavior will likely drift into one of four wrong
outcomes:

- ad hoc docking/workspace glue in demos and apps,
- shell-specific flags pushed back into `fret-ui-kit::recipes`,
- premature runtime widening before a real shell owner split is proven,
- or hidden regressions around tear-out/no-hover transitions with no reproducible gate.

This lane exists to prevent all four.

## Goals

### G1 - Freeze the owner split for shell-aware choreography

This lane must answer which layer owns which part of the next problem:

- `fret-ui-kit::recipes` keeps the generic cross-window transfer baseline,
- shell-aware layers may add choreography around that baseline only where generic ownership is not
  sufficient,
- `fret-ui-kit::imui` remains observational,
- and runtime crates change only if a real mechanism gap is proven by shell evidence.

### G2 - Define one coherent shell choreography rule

The next stable contract must answer:

- when shell layers follow generic `current_window` directly,
- when they may intentionally delay or remap ghost visibility,
- and how that rule behaves during tear-out, viewport handoff, or transient no-hover gaps.

### G3 - Preserve source-authored meaning through shell adaptation

The lane should preserve the architectural win from the prior two ghost slices:

- the source still owns preview meaning,
- shell layers should adapt ownership/choreography, not replace meaning with a shell-local skin
  registry,
- and any shell transfer surface should carry only the minimum data needed to preserve that meaning.

### G4 - Keep fearless-refactor posture explicit

If the generic recipe API or current proof surface turns out to be the wrong base for shell-aware
handoff:

- split it,
- wrap it,
- or move it,

instead of carrying compatibility aliases.

## Non-goals

- Reopening the generic `current_window` transfer baseline unless a hard contract bug is found.
- Native OS drag image / external application preview in v1.
- Multi-item aggregate preview in v1 unless shell proof makes it unavoidable.
- Pushing shell choreography directly into `fret-ui-kit::imui`.
- Treating docking/workspace policy as generic recipe defaults before proof exists.

## Non-negotiable boundaries

| Layer | Owns | Must not own |
| --- | --- | --- |
| `crates/fret-ui` / `fret-runtime` | drag session routing, window identity, overlay-root mechanisms, generic multi-window truth | docking/workspace ghost policy |
| `ecosystem/fret-ui-kit::imui` | read-only drag observation seams | shell ghost arbitration or shell policy |
| `ecosystem/fret-ui-kit::recipes` | generic cross-window ghost transfer baseline | docking/workspace/tear-out choreography |
| `ecosystem/fret-docking` / workspace shell layers | docking/viewports/tear-out ghost ownership and choreography | generic runtime drag contracts |
| apps / demos | proof surfaces and product semantics | framework owner decisions disguised as app glue |

## Decision snapshot

### 1) The generic baseline is now closed

This lane starts from a closed assumption:

- `publish_cross_window_drag_preview_ghost(...)` is the shipped generic transfer helper,
- `render_cross_window_drag_preview_ghosts(...)` is the shipped per-window-root render hook,
- and the remaining work is shell-aware behavior above that baseline.

### 2) The next proof must use a shell-aware surface

The smallest meaningful proof is no longer the generic main/aux pair by itself.

The preferred proof surfaces are:

- a docking tear-out or viewport-shell path,
- or another first-party shell surface where generic `current_window` ownership is visibly
  insufficient.

### 3) Moving-window hints are shell inputs, not generic API commitments

Existing runtime/window hints may be enough for shell choreography, but they are not automatically
generic recipe contracts.

This lane must prove:

- which hints remain shell-local consumption details,
- which ones need durable shell-facing documentation,
- and whether any new runtime surface is actually justified.

### 4) Shell gaps should be solved with proof and gates, not intuition

This lane should prefer:

- one smallest shell-aware proof,
- one explicit gate or scripted repro,
- and one short defer list,

instead of broadening APIs preemptively.

### 5) Phase B is now frozen around docking-owned choreography over runner-owned hover truth

The accepted M1 contract freeze is:

- the first landed shell-aware slice belongs to docking-aware layers, not to `fret-ui-kit::recipes`
  and not to workspace-first surfaces,
- runner-owned `current_window` remains the primary hover/drop truth,
- `moving_window` and `window_under_moving_window` remain shell inputs for choreography, not new
  generic recipe owner rules,
- and the first proof/gate package starts from the docking arbitration demo + diag scripts.

See:

- `docs/workstreams/imui-shell-ghost-choreography-v1/M1_CONTRACT_FREEZE_2026-03-30.md`

## Target architecture

### `ecosystem/fret-ui-kit::recipes`

Default assumption:

- keep the generic cross-window helper as-is unless shell evidence proves a necessary reshape,
- and do not absorb docking/workspace policy into the generic recipe path.

### Shell-aware layers

Expected likely owners for the next landed slice:

- `ecosystem/fret-docking`
- workspace shell layers
- first-party shell proof surfaces during exploration

### `fret-ui-kit::imui`

Default assumption:

- no new ghost-policy entry points,
- at most small read-only shell-relevant observation seams if a proof demonstrates they are needed,
- and no shell-specific arbitration hooks.

## Proof and regression requirements

Minimum package expected before implementation is considered reviewable:

- one first-party shell-aware proof surface,
- one explicit owner decision for shell choreography vs generic transfer,
- one regression artifact that can observe handoff/no-duplicate behavior across shell transitions,
- and one short defer list for native/external preview or aggregate-preview questions that stay out
  of scope.

## Exit criteria

This lane can close only when the repo can answer all of these clearly:

- which shell layer owns docking/tear-out ghost choreography,
- when shell-aware behavior follows generic `current_window` directly and when it does not,
- how duplicate or missing ghosts are prevented during shell transitions,
- what shell-specific gates prove the chosen rule,
- and which wider preview questions remain deferred.
