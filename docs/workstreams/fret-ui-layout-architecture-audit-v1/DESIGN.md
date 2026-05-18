# fret-ui Layout Architecture Audit v1

Status: Closed audit lane
Start: 2026-05-18

## Why this workstream exists

`fret-ui` layout performance work has recently added several narrow clean-geometry proofs. That
work was evidence-driven and useful, but it also made the layout node path large enough that we
should pause and classify the architecture before widening it again.

This lane is now closed. The clean-geometry proof helpers were extracted in a behavior-preserving
split, and the next runtime performance owner has been split to
`docs/workstreams/retained-layout-orchestration-v1/`.

This lane answers one question:

> Is the current `fret-ui` layout / node classification model the right long-term shape, or should
> we split or redesign part of it before the next performance push?

The answer must be based on code evidence, diagnostics, and gates, not on file size alone.

## Current assumptions

- The current clean-geometry code is not random growth. It encodes proven safety subsets for
  width-only interactive resize: pure geometry wrappers, side-effect boundaries, stable
  auto-height wrappers, selected flex/grid/container shapes, explicit zero driver leaves, and
  cached nowrap text.
- `scroll-optimization-v1` has explicitly closed its local clean-geometry resize-jitter phase.
  Future clean-geometry expansion should start as a narrower follow-on rather than reopening that
  phase.
- The largest architectural risk is not the presence of many guard clauses. The risk is mixing
  classification, proof logic, geometry derivation, diagnostics attribution, and layout execution in
  a shape that becomes hard to audit.
- Wrapped text, root `Scroll`, retained/cache roots, and `Canvas` remain separate owner lanes unless
  fresh evidence proves that they are the next bottleneck.

## Scope

In scope:

- `crates/fret-ui/src/tree/layout/node.rs`
- `crates/fret-ui/src/tree/layout/solve.rs`
- `crates/fret-ui/src/tree/layout/entrypoints.rs`
- `crates/fret-ui/src/layout/engine.rs`
- related diagnostics surfaces that explain layout solve cost and clean-geometry rejection reasons
- the active text and scroll workstream handoff notes that define stop conditions

Out of scope for this audit:

- broad renderer redesign
- shadcn recipe policy changes unless they are the proven cause of a layout mechanism issue
- expanding clean-geometry to wrapped text or `Scroll` before a dedicated proof exists
- RTX4090-specific closeout as a blocker for local architecture judgment

## Target outcome

This audit should produce one of three decisions:

1. **Keep current model, continue with narrow proofs.**
   The code is large but the axes are correct; next work should be targeted tests or small
   extraction only.
2. **Refactor organization only.**
   Preserve behavior but split classification/proof helpers into clearer modules so future reviews
   are safer.
3. **Change the model.**
   Introduce a stronger internal representation for node layout roles, side-effect boundaries, and
   clean geometry proofs before any more performance expansion.

## Initial architecture read

The current model already contains the right conceptual axes:

- layout effect: pure geometry vs side-effect boundary,
- child-bounds strategy: pass-through, container, flex, grid, leaf,
- size stability: propagated size vs cached nowrap text metrics,
- rejection attribution: reason, node, and element kind.

That is a good sign. It means a full redesign is not automatically justified.

The main concern is placement: most of the proof model and derivation helpers live in
`tree/layout/node.rs`, next to the ordinary widget layout execution path. If future proofs continue
to land in the same file, reviewability will degrade even if runtime behavior remains correct.

## Evidence anchors

- `docs/workstreams/scroll-optimization-v1/HANDOFF.md`
- `docs/workstreams/scroll-optimization-v1/WORKSTREAM.json`
- `docs/workstreams/text-intrinsic-sizing-and-wrap-v1/text-intrinsic-sizing-and-wrap-v1.md`
- `crates/fret-ui/src/tree/layout/node.rs`
- `crates/fret-ui/src/tree/layout/solve.rs`
- `crates/fret-ui/src/tree/layout/entrypoints.rs`
- `crates/fret-ui/src/layout/engine.rs`
- `crates/fret-ui/src/declarative/tests/layout/layout_engine.rs`

## Non-negotiable constraints

- Do not weaken layout correctness to win a microbenchmark.
- Do not broaden `fret-ui` policy; component policy stays in ecosystem crates.
- Do not treat `Size::default()` ambiguity as a broad data-model migration unless new evidence
  appears outside explicit zero-driver leaves.
- Any model change must be behavior-preserving first and gated before optimization behavior changes.
