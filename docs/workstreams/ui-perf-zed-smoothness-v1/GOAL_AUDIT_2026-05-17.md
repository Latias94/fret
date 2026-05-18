# Editor Paint Contract Closeout Goal Audit

Status: Incomplete
Date: 2026-05-17

## Objective Restatement

Complete the editor paint contract closeout and use verified attribution to decide the next true
refactor hot path:

1. produce a baseline validation artifact with
   `tools/perf/diag_editor_paint_contract_validate.py`,
2. produce an attribution artifact by rerunning validation with `--with-paint-perf`,
3. verify both artifact directories and run the closeout tool,
4. use the owner decision to either open a Canvas/paint replay slice, open a renderer text/glyph
   residency slice, or make no code change when both owners are below threshold.

## Completion Checklist

| Requirement | Evidence | Status |
| --- | --- | --- |
| Baseline validation artifact exists for the formal target. | Strict 2026-05-17 verifier expects `target/fret-diag/editor-paint-contract-validate-20260517-goal-audit/summary.json` and reports it missing. | Missing. |
| Attribution validation artifact exists for the formal target. | Strict 2026-05-17 verifier expects `target/fret-diag/editor-paint-contract-validate-20260517-goal-audit-attrib/summary.json` and reports it missing. | Missing. |
| Baseline validation was attempted on this host. | `python3 tools/perf/diag_editor_paint_contract_validate.py --date-tag 20260517-goal-audit` exits with `windows-rtx4090 validation must run on the target Windows host`. | Blocked by host guard. |
| Attribution validation was attempted on this host. | `python3 tools/perf/diag_editor_paint_contract_validate.py --date-tag 20260517-goal-audit-attrib-host-guard --with-paint-perf` exits with the same Windows-host guard. | Blocked by host guard. |
| Artifact verifier accepts formal artifacts. | `target/fret-diag/editor-paint-contract-windows-handoff-20260517-goal-audit/verify/artifact-verification.summary.json` has `ok=false`, `allow_non_windows=false`, and missing validation/attribution summaries. | Missing. |
| Closeout accepts formal artifacts and chooses owner. | `target/fret-diag/editor-paint-contract-windows-handoff-20260517-goal-audit/closeout/editor-paint-contract-closeout.summary.json` has `ok=false`, `allow_non_windows=false`, and `owner_decision.status=incomplete`. | Missing. |
| Formal handoff command plan exists and omits local-triage escape hatches. | `target/fret-diag/editor-paint-contract-windows-handoff-20260517-goal-audit/handoff-plan.json` has build, preflight, baseline validation, attribution validation, verifier, and closeout steps; no planned command contains `--allow-non-windows`. | Covered as handoff readiness. |
| Local triage baseline/attribution exists. | `target/fret-diag/editor-paint-contract-validate-goal-audit-local-cargo/summary.json` and `target/fret-diag/editor-paint-contract-validate-goal-audit-local-cargo-attrib/summary.json` both have `ok=true` on this macOS host. | Covered locally only. |
| Local closeout chose a provisional owner. | `target/fret-diag/editor-paint-contract-validate-goal-audit-local-cargo/editor-paint-contract-closeout.summary.json` has `allow_non_windows=true` and chose `owner=canvas-paint-replay`. | Covered locally only. |
| Current local owner after follow-up slices is recorded. | `docs/workstreams/scroll-optimization-v1/EVIDENCE_AND_GATES.md` records the 2026-05-17 root-solve attribution refresh: view-cache rerender is gone, row replay stays `289/0`, renderer text stays `64us`, and the next local owner is resize root solve / geometry propagation. | Covered for local direction. |

## Findings

The formal editor paint contract closeout is still incomplete. The current host is not the Windows
RTX4090 target, and both formal validation commands correctly refuse to run here unless the
local-triage `--allow-non-windows` escape hatch is used.

The strict verifier and closeout outputs are the decisive completion evidence for this audit:

- `artifact-verification.summary.json`: `ok=false`, missing both formal summaries.
- `editor-paint-contract-closeout.summary.json`: `ok=false`, `owner_decision.status=incomplete`,
  `owner=null`, `action=wait-for-valid-artifacts`.

The local macOS evidence remains useful for deciding baseline-neutral implementation work, but it
does not satisfy the formal closeout gate. Local attribution originally selected
`canvas-paint-replay`; subsequent measured local slices recovered row-scene replay under inline
preedit, reduced row-fragment planning cost, fixed retained/windowed scroll paint invalidation, and
now identify resize changing-bounds root solve / geometry propagation as the next local owner.

## Required Formal Closeout

Run this on the target Windows RTX4090 host:

```powershell
python tools/perf/diag_editor_paint_contract_windows_handoff.py --date-tag <date>
```

The generated 2026-05-17 dry-run plan confirms the formal sequence:

1. `cargo build -p fretboard-dev --release`
2. `cargo build -p fret-ui-gallery --release --features gallery-ai,gallery-chart,gallery-dev,gallery-web-ime-harness`
3. `python tools/perf/diag_editor_paint_contract_preflight.py --out-summary ...`
4. `python tools/perf/diag_editor_paint_contract_validate.py --date-tag <date> --skip-preflight`
5. `python tools/perf/diag_editor_paint_contract_validate.py --date-tag <date>-attrib --skip-preflight --with-paint-perf`
6. `python tools/perf/diag_editor_paint_contract_verify_artifacts.py ...`
7. `python tools/perf/diag_editor_paint_contract_closeout.py ...`

None of the formal steps should include `--allow-non-windows`.

## Conclusion

Do not mark the active goal complete. The prompt-to-artifact checklist has two missing formal
validation artifacts, a failing strict verifier, and a failing strict closeout. Continue local
optimization only as baseline-neutral work, with the next local implementation discussion focused
on resize root solve / geometry propagation rather than renderer text, shadcn `ScrollArea` policy,
or row replay.
