# UI Performance Contract Audit

Status: Active audit; goal not complete.
Date: 2026-05-09

## Objective

Establish and maintain an editor-grade performance contract comparable to Zed/GPUI and egui:

- representative scripts are mapped to checked-in baselines and gates,
- baseline evidence tracks `p50`, `p95`, and `max`,
- hot-path churn is reduced only when measured evidence points at a real bottleneck,
- fearless refactors remain reversible through commands, logs, and gates.

## Prompt-To-Artifact Checklist

| Requirement | Evidence | Status |
| --- | --- | --- |
| Representative editor-grade scripts are listed. | `docs/workstreams/ui-perf-zed-smoothness-v1/ui-perf-contract-matrix.md` maps steady gallery, resize, code editor resize, view-cache resize torture, pointer move/hit-test, and renderer/effects churn. | Partially covered: the matrix exists, but view-cache post-virtualization is not yet promoted to a dedicated baseline. |
| Baseline rows can record `p50/p95/max`. | `crates/fret-diag/src/diag_perf/baseline_rows.rs` writes `measured_p50`; smoke output `target/fret-diag/codex-p50-baseline-smoke/baseline.json` included `measured_p50`. | Tooling covered for new baselines. |
| Checked-in baselines actually contain `p50`. | Scan on 2026-05-09: 58 perf baseline files, 296 rows, 0 rows with `measured_p50`. | Not covered. Existing baselines are compatible but need intentional re-seeding if p50 must be checked in. |
| Gates use the correct machine baseline. | `tools/perf/diag_resize_probes_gate.py` and `.sh` choose Windows RTX 4090 or macOS baseline by host platform. | Covered for resize helpers; other gate helpers remain explicit-baseline by design. |
| Gates use normalization hooks. | Resize gate helpers and baseline selectors now apply `tooling-suite-prewarm-fonts.json` and `tooling-suite-prelude-reset-diagnostics.json` by default. | Covered for the updated helpers/selectors. |
| A short real gate proves the helper path works. | `target/fret-diag/codex-resize-gate-default-hooks-smoke/summary.json`: `ui-resize-probes`, attempts=1, repeat=1, PASS, Windows baseline selected, default hooks recorded. | Smoke covered, not a full formal gate. |
| Full formal gates are green after the helper changes. | No attempts=3 repeat=7 run was performed after the helper default-hook change. | Not covered. |
| Zed/GPUI and egui comparison remains explicit. | `docs/workstreams/standalone/ui-perf-gpui-gap-v1.md` plus the contract matrix reference pressure column. | Covered as a design map; still needs updates when new gaps close. |
| Churn reduction is evidence-led. | Perf log entries show measured view-cache harness virtualization, code editor resize attribution, and decisions not to start broad root-solve quantization or `WindowedRowsSurface` rewrites without evidence. | Covered for recent work. |
| Baseline maintenance rules are documented. | `docs/workstreams/perf-baselines/README.md` defines machine tags, re-seed criteria, required hooks, selector workflow, validation workflow, and review checklist. | Covered. |
| Completion criteria are unambiguous. | This audit maps requirements to evidence and gaps. | Covered, with open gaps below. |

## Current Evidence Snapshot

- Recent commits:
  - `5592215523 feat(diag): record p50 in perf baselines`
  - `7506e02351 fix(perf): select resize baselines by host platform`
  - `fd45b0d1cf fix(perf): normalize resize gate suite hooks`
  - `5998e1df82 docs(perf): document baseline maintenance policy`
- Short resize gate smoke:
  - Summary: `target/fret-diag/codex-resize-gate-default-hooks-smoke/summary.json`
  - Result: PASS, `failures=0`
  - `drag-jitter`: `top_total/layout/solve=1728/1103/661us`
  - `resize-stress`: `top_total/layout/solve=4021/1664/671us`
- Baseline p50 coverage scan:
  - `TOTAL_FILES=58`
  - `TOTAL_ROWS=296`
  - `TOTAL_ROWS_WITH_P50=0`

## Open Gaps

1. Re-seed primary checked-in baselines with `measured_p50` only through the documented selector workflow.
   - First candidates: `ui-resize-probes.windows-rtx4090.v2.json`,
     `ui-code-editor-resize-probes.windows-rtx4090.v2.json`, and
     `ui-gallery-steady.windows-rtx4090.v2.json`.
2. Run full formal gates after the helper normalization change.
   - `python tools/perf/diag_resize_probes_gate.py --suite ui-resize-probes --attempts 3 --repeat 7`
   - `python tools/perf/diag_resize_probes_gate.py --suite ui-code-editor-resize-probes --attempts 3 --repeat 7`
3. Decide whether the post-virtualization view-cache resize torture scripts should remain evidence-only or become a
   dedicated baseline suite.
4. Add a stricter editor paint stressor before considering a `WindowedRowsSurface` display-list rewrite.
5. Keep non-Windows/macOS machine profiles explicit until a checked-in baseline and owner profile exist.

## Audit Conclusion

The goal is not complete. The contract foundation is now in place, but checked-in baseline artifacts do not yet carry
`measured_p50`, and the full formal gates have not been re-run after normalizing the helper defaults.
