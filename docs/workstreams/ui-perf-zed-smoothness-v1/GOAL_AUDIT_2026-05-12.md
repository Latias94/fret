# UI Performance + Architecture Goal Audit

Status: Incomplete
Date: 2026-05-12

## Objective Restatement

The long-term goal is to make Fret's editor-grade UI performance and architecture closure
measurable and reversible:

1. real editor-grade hot paths are covered by repeatable `p50/p95/max` contracts,
2. renderer payload is part of the contract surface where paint/text/render churn matters,
3. core contract and resource semantics are corrected before broad refactors,
4. redundant or obsolete code is removed when measurement proves it is safe,
5. the result remains explainable, regression-friendly, and comparable to Zed/GPUI/egui.

## Prompt-To-Artifact Checklist

| Requirement | Evidence | Status |
| --- | --- | --- |
| Representative editor-grade scripts are mapped to checked-in baselines and gates. | `docs/workstreams/ui-perf-zed-smoothness-v1/ui-perf-contract-matrix.md` maps resize, code-editor resize, autoscroll steady/typical, complex wheel, view-cache, hover/layout, menubar, tabs, overlay, and virtual-list probes to explicit baselines and gate commands. | Covered for the current Windows contract surface. |
| Baselines carry `p50/p95/max` evidence. | `docs/workstreams/perf-baselines/*.json` and the matrix/audit docs record checked-in baselines with `measured_p50`, `measured_p90`, `measured_p95`, and `measured_max` where required. | Covered for promoted baselines. |
| Renderer payload is a first-class contract where needed. | `docs/workstreams/ui-perf-zed-smoothness-v1/ui-perf-contract-audit.md` records `renderer_instance_bytes` and `renderer_encode_scene_text_ops` propagation through perf JSON, baseline JSON, threshold rows, and threshold failures. | Covered for editor paint contracts. |
| Resize contracts survive contract refreshes and semantic fixes. | `ui-code-editor-resize-probes` passed after the font-catalog and surface-recovery fixes, including the formal repeat=7/attempts=3 gate recorded in `docs/workstreams/ui-perf-zed-smoothness-v1/ui-perf-zed-smoothness-v1-log.md`. | Covered. |
| Core contract/resource semantics are fixed before broad refactors. | Commits `35b88ccb5f`, `80b718a7d5`, and `115ee9deca` fixed font-catalog publication, surface reconfigure recovery, and debug-only warning noise. | Covered for the current hot-path semantics slices. |
| Warning/debug noise is reduced instead of ignored. | `cargo check -p fret-ui -p fret-runtime` passed after the warning cleanup, and the resize gate still passed afterward. | Covered. |
| Fearless refactors remain reversible and logged. | The perf log records the font-catalog fix, surface recovery, warning cleanup, and the follow-up resize smoke/gate evidence. | Covered. |
| The architecture split stays explicit. | `docs/architecture.md`, `docs/code-editor.md`, and `docs/adr/0185-code-editor-ecosystem-v1.md` keep the framework/eco split and editor surface contracts explicit. | Covered. |
| Zed/GPUI/egui comparison remains explicit. | `docs/workstreams/standalone/ui-perf-gpui-gap-v1.md` and the contract matrix keep the comparison pressure visible. | Covered. |
| Linux editor-grade perf evidence exists as a formal contract. | `docs/workstreams/perf-baselines/README.md` and `docs/workstreams/ui-perf-zed-smoothness-v1/ui-perf-zed-smoothness-v1-todo.md` still mark Linux as blocked pending a real Linux runner/profile. WSL smoke passes were recorded, but they are not a checked-in Linux baseline. | Not covered. |

## Additional Smoke Evidence

- Windows `imui_hello_demo` screenshot recheck on 2026-05-13:
  `FRET_DIAG=1 FRET_DIAG_DIR=target/fret-diag/imui-hello-demo-screenshot-recheck FRET_DIAG_GPU_SCREENSHOTS=1 cargo run -p fretboard-dev -- diag run tools/diag-scripts/ui-editor/imui/local-debug/imui-hello-demo-screenshot.json --dir target/fret-diag/imui-hello-demo-screenshot-recheck --session-auto --timeout-ms 180000 --launch -- cargo run -p fret-demo --bin imui_hello_demo`
- Screenshot evidence:
  `target/fret-diag/imui-hello-demo-screenshot-recheck/sessions/1778605632348-99852/screenshots/1778606081730-imui-hello-demo/window-4294967297-tick-41-frame-40.png`
- Visible text is present again (`Count: 0`, `Increment`, `Enabled: false`, `Enabled`), so the earlier blank Windows smoke was pre-fix evidence rather than a WSL-specific symptom.
- Current machine-checkable IMUI hello smoke:
  `FRET_DIAG=1 FRET_DIAG_DIR=target/fret-diag/imui-hello-demo-semantic-smoke-r3 FRET_DIAG_GPU_SCREENSHOTS=1 cargo run -p fretboard-dev -- diag run tools/diag-scripts/ui-editor/imui/imui-hello-demo-semantic-smoke.json --dir target/fret-diag/imui-hello-demo-semantic-smoke-r3 --session-auto --timeout-ms 180000 --launch -- target/debug/imui_hello_demo.exe`
- Result:
  `target/fret-diag/imui-hello-demo-semantic-smoke-r3/sessions/1778617439258-104240/script.result.json` passed at
  `step_index=15` after asserting text/control semantics and the `Increment` / `Enabled` interactions.
- WSL code-editor resize smoke gate retry on current head:
  `CARGO_TARGET_DIR=/home/frankorz/fret-target python3 tools/perf/diag_code_editor_resize_jitter_smoke_gate.py --repeat 1 --warmup-frames 1 --timeout-ms 600000 --launch-bin /home/frankorz/fret-target/release/fret-ui-gallery --out-dir /home/frankorz/fret-diag-code-editor-resize-jitter-smoke-linux-recheck-current-20260513-t600`
  still times out with `Connection reset by peer` and `timeout waiting for script result`; it is not checked-in Linux contract evidence.

## Current Gaps

1. Real Linux runner/profile evidence is still missing.
2. A checked-in Linux baseline for the editor-grade probes is still missing.
3. The current WSL smoke runs are useful evidence, but they do not close the Linux contract.
4. The current WSL code-editor resize smoke gate still times out on the current head after rebuild, with `Connection reset by peer` in `stderr.log` and `stage=running` at `step_index=5`; do not infer a checked-in Linux editor-grade baseline from this run.
5. The code-editor public-API TODO still leaves the top-level `paint` owner boundary open, but the
   implementation is already split into `geom_cache`, `rich`, `scene`, and `text`. Treat this as a
   low-priority cleanup candidate, not as evidence for a broad renderer rewrite.

## Conclusion

The current Windows/macOS contract surface is materially stronger and now covers the hot-path
editor resize path, payload-aware code-editor baselines, and the resource semantics fixes that
were needed to keep the contracts honest.

The remaining uncovered requirement is formal Linux editor-grade evidence from a real Linux
runner/profile. Until that exists, the goal remains incomplete.

One additional follow-up remains in the architecture lane: the `paint` owner line in the
code-editor public-API TODO is not yet checked off, but the current implementation already has
submodule boundaries and no measured evidence yet calls for another broad split. Keep that item
separate from the Linux contract gap.
