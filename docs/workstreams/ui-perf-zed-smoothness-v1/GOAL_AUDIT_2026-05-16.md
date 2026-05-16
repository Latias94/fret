# Editor Paint Contract Closeout Goal Audit

Status: Incomplete
Date: 2026-05-16

## Objective Restatement

Complete the editor paint contract closeout and use verified attribution to decide the next true
refactor hot path:

1. produce a baseline validation artifact with
   `tools/perf/diag_editor_paint_contract_validate.py`,
2. produce an attribution artifact by rerunning validation with `--with-paint-perf`,
3. verify both artifact directories and run the closeout tool,
4. use the owner decision to either open a Canvas/paint replay slice, open a renderer text/glyph
   residency slice, or make no code change when both owners are below threshold.

## Prompt-To-Artifact Checklist

| Requirement | Evidence | Status |
| --- | --- | --- |
| Baseline validation artifact exists. | `target/fret-diag/editor-paint-contract-validate-goal-audit-local-cargo/summary.json` has `ok=true`, `with_paint_perf=false`, three checked probe steps, and zero threshold failures. | Covered for local macOS triage only. |
| Attribution validation artifact exists. | `target/fret-diag/editor-paint-contract-validate-goal-audit-local-cargo-attrib/summary.json` has `ok=true`, `with_paint_perf=true`, three checked probe steps, and paint/renderer/code-editor coverage for each probe. | Covered for local macOS triage only. |
| Artifact verifier accepts both directories. | `target/fret-diag/editor-paint-contract-validate-goal-audit-local-cargo/artifact-verification.summary.json` has `ok=true`, empty validation/attribution errors, and `allow_non_windows=true`. | Covered for local macOS triage only. |
| Closeout tool accepts the verified artifacts. | `target/fret-diag/editor-paint-contract-validate-goal-audit-local-cargo/editor-paint-contract-closeout.summary.json` has `ok=true` and `allow_non_windows=true`. | Covered for local macOS triage only. |
| Strict verifier rejects local artifacts when formal Windows rules are enforced. | `target/fret-diag/editor-paint-contract-validate-goal-audit-local-cargo/artifact-verification.strict.summary.json` has `ok=false`, `allow_non_windows=false`, and errors requiring `target/release/fretboard-dev.exe` / `target/release/fret-ui-gallery.exe`. | Covered as negative evidence. |
| Strict closeout refuses to choose an owner from invalid formal artifacts. | `target/fret-diag/editor-paint-contract-validate-goal-audit-local-cargo/editor-paint-contract-closeout.strict.summary.json` has `ok=false`, `allow_non_windows=false`, and `owner_decision.status=incomplete`. | Covered as negative evidence. |
| Current host is not the formal Windows target. | `sys.platform=darwin`, `platform.system=Darwin`, `platform.machine=arm64`; `uname -a` reports `Darwin ... RELEASE_ARM64_T8132 arm64`. | Covered as environment evidence. |
| Formal Windows RTX4090 closeout runs without `--allow-non-windows`. | Not present. The current formal TODO requires baseline validation, attribution validation, verifier, and closeout on the target Windows host without `--allow-non-windows`. | Missing; goal cannot be marked complete. |
| Owner decision identifies the next hot path. | Local closeout selected `owner=canvas-paint-replay`; complex-wheel `canvas_exclusive_p95_us=407`, `paint_widget_p95_us=509`, highest renderer text prepare `69us`. | Covered for local direction. |
| Canvas/paint replay implementation slice exists when Canvas dominates. | Commit `51af063328 perf(editor): recover row scene replay under preedit` keeps only the preedit row on the paint-time path and restores row-scene prepaint replay for other visible rows. | Covered for local slice. |
| Implementation slice has correctness and perf evidence. | Tests: `cargo fmt -p fret-code-editor -p fret-ui-kit --check`; `cargo nextest run -p fret-code-editor prepaint_row_scene_replay_plan_skips_only_inline_preedit_rows prepaint_row_scene_replay_plan_moves_row_text_work_out_of_paint planned_replay_rows_with_selection_still_paint_overlay --features syntax-rust --no-fail-fast`; `cargo nextest run -p fret-ui-kit windowed_rows_frame_row_rects_iterates_visible_rows --no-fail-fast`. Perf evidence: `target/fret-diag/local-next-editor-paint-20260516-preedit-row-plan-complex-wheel-r3/worst.stats.json`. | Covered locally. |
| Target-machine handoff is executable. | Dry-run plan: `target/fret-diag/editor-paint-contract-windows-handoff-goal-audit-handoff-dry-run/handoff-plan.json`. Every planned formal step omits `--allow-non-windows`. | Covered as handoff readiness, not as closeout. |
| Target-machine handoff guard is tested. | `python3 -m unittest test_diag_editor_paint_contract_windows_handoff.py` passes 6 tests; non-dry-run on this macOS host exits with rc `2` and `the editor paint contract handoff must run on the target Windows host`. | Covered as handoff readiness, not as closeout. |
| Existing local closeout reports were swept for a formal success. | A `target/fret-diag/**/editor-paint-contract-closeout*.summary.json` sweep found 7 reports: dry-run plans, failed strict/formal attempts, and one successful non-dry-run report with `allow_non_windows=true`. No successful formal report without `allow_non_windows` was found. | Covered as negative evidence. |

## Local Attribution Outcome

The verified local closeout chose `canvas-paint-replay`, not `renderer-text-prepare`.

The first Canvas/paint replay slice found a code-editor policy bug rather than a renderer bottleneck:
inline preedit caused `replay_row_scene_plan_candidates_for_frame(...)` to return before planning
any visible row. After the fix, the narrow complex-wheel run changed:

- `rows_scene_prepaint_planned`: `0 -> 288`
- `rows_scene_prepaint_skip_preedit`: `0 -> 1`
- code-editor `us_total` p95: `383 -> 111us`
- windowed-surface paint callback p95: `414 -> 151us`
- Canvas exclusive p95: `419 -> 152us`
- frame paint p95: `679 -> 427us`
- frame prepaint p95: `122 -> 268us`
- frame total p95: `830 -> 787us`

This moves the next local owner from paint-time Canvas replay to residual prepaint row-scene
planning cost and/or a coarser row-fragment replay contract. It does not justify a renderer text
or glyph residency slice from current evidence.

## Missing Requirement

The formal Windows RTX4090 artifact set is still missing. The current local closeout deliberately
uses `--allow-non-windows`, so it cannot satisfy the formal contract closeout. The active goal must
remain incomplete until the Windows target machine produces:

1. baseline validation without `--allow-non-windows`,
2. attribution validation with `--with-paint-perf` and without `--allow-non-windows`,
3. artifact verification without `--allow-non-windows`,
4. closeout without `--allow-non-windows`.

Preferred command:

```powershell
python tools/perf/diag_editor_paint_contract_windows_handoff.py --date-tag <date>
```

The strict verifier/closeout negative reports above are intentional: they prove the local cargo-run
artifacts cannot accidentally pass as the formal Windows artifact set.

## Conclusion

Local evidence is strong enough to continue baseline-neutral Canvas/paint replay work, and one
measured slice has landed. The editor paint contract closeout goal is still incomplete because the
formal Windows RTX4090 closeout has not been run.
