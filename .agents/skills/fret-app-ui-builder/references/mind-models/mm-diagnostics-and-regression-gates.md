# Mind model: Diagnostics and regression gates

Goal: treat UI bugs as “repro artifacts” (script + bundle + gate) instead of one-off investigations.

## Default approach

1. Add stable `test_id` on targets you need to click/assert.
2. Create a scripted repro in `tools/diag-scripts/<scenario>.json`.
3. Gate with the smallest reliable signal:
   - Prefer semantics/bounds invariants over pixel diffs.
   - Add `capture_screenshot` only when visuals are part of the contract.

## Common env knobs

- `FRET_DIAG=1` enables diagnostics bundles.
- `FRET_DIAG_SCREENSHOTS=1` is required for scripted screenshot capture.
- While authoring: `FRET_DIAG_REDACT_TEXT=0` can be useful for debugging.

## Tooling entrypoints

- Scripted repro runner: `cargo run -p fretboard -- diag run ...`
- Packaging for sharing: `cargo run -p fretboard -- diag pack ... --include-screenshots`
- Summarize regressions: `cargo run -p fretboard -- diag triage ... --json`

## See also

- `fret-diag-workflow` (scripted repro + packaging)
