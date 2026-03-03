# `crates/fret-diag` assessment (capabilities, gaps, and refactor targets)

Status: Draft (workstream note)

This note answers: “How good is `crates/fret-diag` today?”, “What is missing compared to other UI
projects?”, and “What should we refactor if we can be fearless?”.

This is intentionally **tooling-focused** (not a runtime contract). Hard contracts should live in
ADRs and protocol types.

## What `fret-diag` is good at (today)

1. **Artifact-first debugging**
   - A bundle is the portable unit: you can share it, lint it, triage it, and diff it.
   - This is a strong alignment with “editor-grade” needs: bugs are often non-local and need
     evidence, not screenshots alone.

2. **Scripted interaction + deterministic gates**
   - Scripts can drive UI state and assert invariants, which enables CI and reproducible repros.
   - Post-run checks (pixels/triage/hotspots) scale better than manual inspection once stabilized.

3. **Transport parity direction (FS + WS)**
   - The same concepts work across filesystem transport (native) and WS transport (web/remote).
   - Suite runs already force bounded bundle dumps to make downstream tooling deterministic.

4. **Practical ergonomics for authoring**
   - `fretboard diag` provides a single surface for run/suite/lint/pack/perf workflows.
   - “Pack/AI packet” flows reduce friction when iterating with external tools.

## Current architectural weaknesses (why it churns)

1. **Orchestration logic is still boolean-heavy**
   - Many decisions used to be “wired by convention” (OR-chains) instead of metadata-driven
     planning.
   - Post-run planning is now registry-driven; remaining churn tends to come from launch/runtime
     capability wiring and ad-hoc demo policies.

2. **Engine policy vs demo policy is interleaved**
   - UI gallery / demo-specific rules frequently leak into “engine” code paths.
   - This increases churn and makes it hard to treat `fret-diag` as a reusable subsystem.

3. **Missing a coherent layout debugging workflow**
   - There is a Taffy dump escape hatch, but it is not a first-class artifact tied to a bundle.
   - There is no “inspect → pick selector → assert geometry → explain with layout sidecar” path.

4. **Extensibility is still mostly “edit the monolith”**
   - Ecosystem crates cannot easily contribute:
     - new runtime debug payloads,
     - new gates/checks,
     - new viewers/panels,
     without touching central wiring in `crates/fret-diag` and `fret-bootstrap`.

## Comparison: strengths vs common open-source UI stacks

This comparison is outcome-based (not implementation-based).

### Where Fret’s diagnostics approach is ahead

- **Portable repro artifacts** (bundles) are stronger than “just live inspector” workflows.
- **Script-driven gates** make it feasible to prevent regressions in complex editor interactions.

### Where Fret is behind (today)

- **Live inspector UX** (Flutter DevTools / React DevTools / Compose Layout Inspector style):
  - tree browsing, selection, “jump to source”, and property views are still early.
- **Layout explainability**
  - many stacks provide “show bounds”, “show constraints”, “show hit test”, and “why is this
    re-laid out?” tooling as a first-class workflow.
- **Timeline correlation**
  - mature stacks often provide a timeline (input → state → layout → paint) and correlate it with
    snapshots and traces.

## What “layout debugging support” should mean for Fret

We should treat layout debugging as two complementary products:

1. **Layout correctness (semantics-first)**
   - Gates should primarily assert semantics bounds keyed by stable selectors (`test_id`), not raw
     pixels.
   - Failure artifacts should include enough evidence to explain *which subtree* drifted.

2. **Layout explainability (sidecars)**
   - When a gate fails, it should be possible to request a sidecar dump (best-effort) that explains
     layout results for a selected subtree (e.g. Taffy node tree + constraints + computed sizes).

This workstream tracks a staged plan for this under `TODO.md` (M3/M4).

## Fearless refactor targets (prioritized)

1. **Make planning metadata-driven**
   - Keep moving “should we run X?” / “do we need screenshots?” / “do we need a bundle?” decisions
     behind registries and explicit metadata.

2. **Separate demo policy from engine policy**
   - Move UI gallery and other demo-specific rules behind:
     - a demo policy module, or
     - a registry entry that is keyed by suite/script metadata.

3. **Define an ecosystem diagnostics extension path**
   - Add a bounded `debug.extensions` slot to runtime snapshots (keyed JSON, capability-gated).
   - Let ecosystem crates add:
     - extension writers (runtime),
     - gates/checks (tooling),
     - viewers/panels (DevTools GUI),
     without central churn.

4. **Promote layout sidecars to first-class artifacts**
   - Script-level request → bundle-scoped sidecar file(s) → tooling viewer → optional gate hooks.

## Suggested next steps (landable)

- Continue migrating ad-hoc post-run checks into `CheckRegistry` until `diag_run.rs` no longer needs
  large OR-chains.
- Add a first “layout correctness” gate script (semantics bounds) and lock it as a regression suite.
- Design the minimal `debug.extensions` contract (keys, bounds, capability gating) before adding
  runtime payloads.
