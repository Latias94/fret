# Perf handoff (when the “bug” is a hitch)

If the issue is “it feels janky” (resize/scroll/pointer-move) rather than a correctness regression:

1. Switch to `fret-perf-workflow` and run an appropriate gate/suite (`ui-gallery-steady`, `ui-resize-probes`, etc).
2. When a `diag perf` run fails, start with the thresholds evidence:
   - `<out-dir>/check.perf_thresholds.json` (or `attempt-N/check.perf_thresholds.json` for gate scripts)
   - Tip: `fret-perf-workflow` includes a compact gate triage helper:
     `.agents/skills/fret-perf-workflow/scripts/triage_gate.sh <out-dir>`
3. Use the worst bundle for root cause:
   - `cargo run -p fretboard -- diag stats <bundle.json> --sort time --top 30`
4. Turn the hitch class into a stable probe or a stricter gate once it is explainable:
   - Add a `tools/diag-scripts/*.json` script (stable `test_id` targets), then baseline/gate it.

## “Resize jank” fast path (copy/paste)

Run the P0 resize probes (numbers + thresholds):

```bash
tools/perf/diag_resize_probes_gate.sh --suite ui-resize-probes --attempts 3
tools/perf/diag_resize_probes_gate.sh --suite ui-code-editor-resize-probes --attempts 3
```

If a gate fails (or you want the worst bundles even on PASS):

```bash
.agents/skills/fret-perf-workflow/scripts/triage_gate.sh <out-dir> --all --app-snapshot
```

Then inspect the worst bundle:

```bash
cargo run -p fretboard -- diag stats <bundle.json> --sort time --top 30
```
