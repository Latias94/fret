## Gate checklist

Pick the smallest gate that has signal:

- **Invariant test** (fast, preferred):
  - Test location: `<path>`
  - Command: `<cargo nextest run ...>`
- **Scripted interaction** (for state machines):
  - Script: `tools/diag-scripts/<name>.json`
  - Command: `cargo run -p fretboard -- diag run tools/diag-scripts/<name>.json --env FRET_DIAG=1 --launch -- <cmd...>`
- **Parity/golden** (for layout/style outcomes):
  - Harness: `ecosystem/fret-ui-shadcn/tests/`
  - Golden: `goldens/shadcn-web/v4/new-york-v4/...`

**Selectors:**

- Prefer `test_id` or semantics-based targets.
- Avoid pixel coordinates unless absolutely necessary.

**Artifacts to attach in PRs/issues:**

- `bundle.json` (and screenshots if relevant)
- `script.result.json` / `triage.json` when available
