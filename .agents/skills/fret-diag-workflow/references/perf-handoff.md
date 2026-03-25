# Perf handoff (when the “bug” is a hitch)

If the issue is “it feels janky” (resize/scroll/pointer-move) rather than a correctness regression:

1. Switch to `fret-perf-optimization` and run an appropriate gate/suite (`ui-gallery-steady`, `ui-resize-probes`, etc).
2. When a `diag perf` run fails, start with the thresholds evidence:
   - `<out-dir>/check.perf_thresholds.json` (or `attempt-N/check.perf_thresholds.json` for gate scripts)
   - Tip: use the cross-platform gate triage helper:
     `python3 .agents/skills/fret-diag-workflow/scripts/triage_perf_gate.py <out-dir>`
3. Use the worst bundle for root cause:
   - `cargo run -p fretboard -- diag stats <bundle.json> --sort cpu_cycles --top 30`
   - If CPU signal is near-zero but wall time is high, re-run with `--sort time` to separate scheduling noise from real UI-thread work.
4. Turn the hitch class into a stable probe or a stricter gate once it is explainable:
   - Add a `tools/diag-scripts/*.json` script (stable `test_id` targets), then baseline/gate it.
5. If the worst bundle still is not enough to explain the spike, escalate to `fret-perf-tracy-bridge`.

## “Resize jank” fast path (copy/paste)

Run the P0 resize probes (numbers + thresholds):

```bash
tools/perf/diag_resize_probes_gate.sh --suite ui-resize-probes --attempts 3
tools/perf/diag_resize_probes_gate.sh --suite ui-code-editor-resize-probes --attempts 3
```

If a gate fails (or you want the worst bundles even on PASS):

```bash
python3 .agents/skills/fret-diag-workflow/scripts/triage_perf_gate.py <out-dir> --all --app-snapshot
```

Then inspect the worst bundle:

```bash
cargo run -p fretboard -- diag stats <bundle.json> --sort cpu_cycles --top 30
```
