# Editor Canvas Paint Replay Slice v1 Milestones

# Status

- M1: complete
- M2: complete
- M3: complete
- M4: complete

## M1: Owner Boundary Locked

Exit criteria:

- ECPR-010 source audit exists in this lane.
- The audit identifies the first implementation owner.
- The audit explicitly preserves row replay/cache and renderer payload guardrails.

## M2: Attribution Is Sufficient

Exit criteria:

- Existing summaries are proven sufficient, or new focused summary fields land with tests.
- The lane can distinguish Canvas-hosted replay/touch, row-surface callback assembly, paint-cache bookkeeping, and
  generic paint traversal well enough to justify one implementation change.

## M3: Reversible Optimization Landed

Exit criteria:

- One bounded implementation slice lands.
- Focused correctness tests pass.
- Three-probe editor-paint attribution rerun shows the intended owner moved without shifting cost into a guarded path.

Evidence:

- `target/fret-diag/editor-paint-contract-validate-20260523-r59/summary.json`
- `ecosystem/fret-code-editor/src/editor/paint/scene.rs`
- `ecosystem/fret-code-editor/src/editor/tests/row_text_cache.rs`

## M4: Target-Machine Closeout

Exit criteria:

- Target-machine validation and attribution pass.
- Artifact verifier and closeout pass.
- Baseline policy is explicitly documented.

Evidence:

- `target/fret-diag/editor-paint-contract-validate-20260523-r59/summary.json`
- `target/fret-diag/editor-paint-contract-validate-20260523-r59-attrib/summary.json`
- `target/fret-diag/editor-paint-contract-validate-20260523-r59/artifact-verification.summary.json`
- `target/fret-diag/editor-paint-contract-validate-20260523-r59/editor-paint-contract-closeout.summary.json`
