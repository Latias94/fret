# Goal-backward verification

Use this note when a task needs real proof, not just a completed checklist.

The core rule:

- do not start from "what files changed?",
- start from "what must be true if this work actually succeeded?"

Then verify downward from outcome to implementation.

## The three layers

### 1) Truths

Write 3-5 observable truths.

These should describe outcomes a reviewer or maintainer can reason about, for example:

- a menu closes on outside press and restores focus to the trigger,
- a snippet teaches the intended public authoring surface,
- a perf fix actually reduces worst-frame hitching under the named probe.

Good truths are observable and bounded. Bad truths just restate tasks like "added dialog refactor".

### 2) Artifacts

For each truth, name the concrete artifacts that must exist.

Examples:

- a component implementation,
- a policy layer helper,
- a UI Gallery snippet,
- a diag script,
- a unit/integration test,
- an ADR/alignment update.

If a truth has no supporting artifact, it is not yet delivered.

### 3) Wiring

For each truth, ask what must be connected for the artifact to matter.

Examples:

- the snippet must actually use the intended facade,
- the diag script must hit the right `test_id`,
- the overlay policy must be called by the surface that claims to implement it,
- the perf gate must point at the probe or bundle that captures the regression.

This is where many false positives hide: files exist, but the outcome is still not real.

## How to use it in Fret

### Reviews

When reviewing, do not only list code smells or missing gates.

Instead:

1. state the broken or unproven truth,
2. point to the missing/wrong artifact or wiring,
3. recommend the smallest gate that would prove the truth next time.

### Framework changes

When landing framework work:

1. define the must-be-true outcomes,
2. align repro/gate/evidence to those outcomes,
3. update ADR/alignment/docs only when they are part of the truth set.

ADR text is not the truth by itself. It is one supporting artifact.

### Workstreams

For workstreams, each landable slice should still be explainable through:

- truth,
- artifact,
- wiring,
- gate/evidence.

If a slice cannot be described that way, it is probably too vague.

## Common failure modes

- Treating code existence as success even though the behavior is unwired.
- Treating a new doc as success even though examples/tests still teach the old path.
- Treating a passing narrow test as success even though the user-facing invariant is not covered.
- Treating “I implemented the planned tasks” as equivalent to “the intended outcome now exists”.

## Minimal output shape

When you need a compact verification note, use:

- `Truth`
- `Artifacts`
- `Wiring`
- `Proof`
- `Residual risk`
