# ImUi Editor-Grade Product Closure Goal Completion Audit - 2026-05-13

Status: Not complete. Continue through narrow follow-ons.

This audit maps the active user goal to current repo evidence. It is intentionally stricter than a
green-test summary: a passing verifier only counts when it covers the explicit requirement being
claimed.

## Objective Restatement

The active goal asks Fret to reach Dear ImGui-class editor usability while keeping the Fret
architecture intact:

1. Work on `main`.
2. Keep `fret-imui` thin and policy-light.
3. Put generic IMUI policy in `fret-ui-kit::imui`.
4. Put editor controls/composites in `fret-ui-editor`.
5. Put docking and multi-window policy in `fret-docking` plus runner/backend owners.
6. Use fearless refactoring to remove unsuitable design, duplicate aliases, and oversized private
   owner files when evidence supports it.
7. Compare gaps against the local Dear ImGui reference in `repo-ref/imgui`.
8. Leave repro, gate, and evidence for every meaningful slice.
9. Do not mark the goal complete until editor-grade workflow, diagnostics, docking hand-feel, and
   proof-led helper growth rules are all covered by real evidence.

## Prompt-To-Artifact Checklist

| Requirement | Current evidence | Verdict |
| --- | --- | --- |
| Work on `main` | `git branch --show-current` returned `main`. | Met |
| Workstream tracking exists | `docs/workstreams/imui-imgui-gap-closure-v1/WORKSTREAM.json`, this maintenance umbrella, and `docs/workstreams/docking-multiwindow-imgui-parity/WORKSTREAM.json` are current machine-readable state anchors. | Met |
| `fret-imui` stays thin | `P3_EXECUTION_PRIORITY_REVIEW_2026-05-06.md` keeps `fret-imui` policy-light and routes policy-heavy widgets to `fret-ui-kit::imui`; `python tools/gate_imui_workstream_source.py` locks that read. | Met |
| Generic IMUI policy stays in kit | `ecosystem/fret-ui-kit/src/imui.rs` and the split `facade_writer/*` owners carry generic immediate widget policy and facade wrappers. | Met |
| Editor controls stay in editor crate | `ecosystem/fret-ui-editor/src/imui.rs` remains the editor-control adapter surface, with editor control depth tracked by closed follow-ons. | Met |
| Docking/multi-window stays out of runtime UI helpers | `docking-multiwindow-imgui-parity` keeps active execution in `fret-docking` and runner/backend owners; `crates/fret-ui` is not widened for these gaps. | Met |
| Fearless refactor is being used | The debug-draw owner split and the 2026-05-13 facade owner splits moved oversized private owners into narrower modules without public API widening. | Met for recent slices |
| Duplicate/stale helper deletion is evidence-driven | `P1_CLEANUP_AUDIT_2026-05-06.md` and `P1_CLOSEOUT_AUDIT_2026-05-06.md` record a no-delete verdict for current aliases instead of deleting canonical seams. | Met |
| Dear ImGui comparison is local and source-backed | `P0_CURRENT_SOURCE_AUDIT_2026-05-06.md`, the P3 catalog notes, and `repo-ref/imgui/imgui.h` / `imgui.cpp` / `imgui_demo.cpp` are evidence anchors. | Met |
| User-usable golden path exists | `imui_action_basics`, `imui_editor_controls_basics`, `imui_editor_proof_demo`, `editor_notes_demo`, `workspace_shell_demo`, `docking_arbitration_demo`, the docking perf entrypoint, and diagnostics docs form the current product chain; `imui_editor_controls_basics` and editor-notes surfaces now have launched evidence, and the default product-chain gate validates the docking campaign and perf suite manifests. | Partially met |
| Repro + gate + evidence discipline exists | `EVIDENCE_AND_GATES.md`, `tools/gate_imui_workstream_source.py`, `tools/gate_imui_facade_teaching_source.py`, and `tools/gate_imui_editor_collection_source.py` lock current source/doc proof surfaces. | Met |
| Docking local non-interactive gates are current | `M13_LOCAL_NONINTERACTIVE_GATE_REFRESH_2026-05-13.md` records green campaign manifest validation and non-GUI behavior gates. | Met for local non-interactive gates only |
| Launched bounded multi-window campaign is green | `M14_LAUNCHED_BOUNDED_CAMPAIGN_REPAIR_2026-05-13.md` records the repaired launched bounded P3 campaign at `target/fret-diag/campaigns/imui-p3-multiwindow-parity/1778655473217`, plus the post-documentation rerun at `target/fret-diag/campaigns/imui-p3-multiwindow-parity/1778656624160`, with `campaign: ok` and `passed: 4`. | Met for the generic bounded campaign only |
| Full multi-window hand-feel is closed | M14 still does not count as Linux Wayland compositor acceptance or full platform-specific real-host hand-feel closure. | Not met |
| DevTools/Demo/Metrics-style discoverability is closed | Diagnostics lanes define the first-open CLI/GUI/MCP split; `fretboard-dev --help` / `fretboard-dev list --help` expose the tool-app index; `fretboard-dev list tool-apps --json` exposes GUI/MCP entrypoints with a machine-readable form; and the default product-chain gate now validates those entrypoints. Broader DevTools/demo/metrics productization remains product work. | Partially met |
| Full Dear ImGui-class editor maturity is closed | Remaining gaps include real-host OS-window hand-feel, DevTools discoverability polish, perf attribution/smoothness, and proof-led helper candidates only after repeated first-party pressure. | Not met |

## Current Strengths

- The public authoring story is no longer ambiguous by default: apps should use `fret::imui`
  instead of importing lower-level crates directly.
- The proof budget rule is explicit: new shared helpers need at least two real first-party proof
  surfaces unless they are thin adapters over existing declarative controls.
- The 2026-05-13 owner splits reduced private implementation pressure in `fret-ui-kit::imui`
  without moving policy into `fret-imui` or widening `crates/fret-ui`.
- The docking lane now has a current M13 local gate refresh, and it remains honest about what that
  does and does not prove.

## Missing Or Weakly Verified Requirements

- **Full OS-window multi-viewport hand-feel remains open.** M14 proves the generic launched bounded
  campaign on the local Windows host, but it still does not prove Linux Wayland acceptance or full
  platform-specific real-host cross-window feel.
- **DevTools discoverability is partially productized, with a stronger drift gate.** The
  diagnostics contract is strong, `fretboard-dev --help` and `fretboard-dev list --help` now stay
  covered as first CLI discovery points, `fretboard-dev list tool-apps --json` gives tools one
  machine-readable GUI/MCP map, and `python tools/diag_gate_imui_product_chain.py` now validates
  those entrypoints as part of the default IMUI product-chain gate. The current priority map still
  says Dear ImGui-class always-available demo/metrics/debug discoverability needs continued work.
- **Product workflow coherence is now source-backed, lightly consumer-audited, first-contact
  visually artifact-backed, and editor-notes suite-backed.** `P0_CONSUMER_WORKFLOW_AUDIT_2026-05-13.md`
  checks docs/tooling discovery, compile paths, and the launched `imui_editor_controls_basics`
  layout/screenshot/bundle smoke. The 2026-05-14 refresh promotes `editor_notes_demo` and
  `editor_notes_device_shell_demo` suite manifests into the product-chain gate; the follow-up
  source-gate refresh also validates `imui-p3-multiwindow-parity` as the docking campaign manifest
  and the DevTools GUI/MCP first-open help/JSON map in the default product-chain command. Broader
  product workflow closure still needs continued visual/interaction and hand-feel evidence before
  this goal is complete.
- **Performance discipline now has a product-chain entrypoint, but not closure.** The
  `perf-docking-arbitration-steady` suite is wired into the IMUI product-chain gate, and the
  explicit launched `perf-docking` slice verifies `regression.summary.json` records two passing
  `perf_case` rows. This is an entrypoint and tooling contract repair, not a broad smoothness
  closeout or platform-specific perf acceptance.
- **Public helper/API growth remains intentionally constrained.** This is correct architecture, but
  it means full Dear ImGui API breadth is not the completion criterion unless real proof surfaces
  demand it.

## Verification Snapshot

2026-05-13 local checks behind this status refresh:

```powershell
git branch --show-current
python -m json.tool docs/workstreams/imui-editor-grade-product-closure-v1/WORKSTREAM.json
python -m json.tool docs/workstreams/docking-multiwindow-imgui-parity/WORKSTREAM.json
python tools/gate_imui_workstream_source.py
python tools/gate_imui_facade_teaching_source.py
python tools/gate_imui_editor_collection_source.py
python tools/check_workstream_catalog.py
cargo fmt --package fret-ui-kit -- --check
cargo check -p fret-ui-kit --features imui
cargo run -p fretboard-dev -- diag script validate tools/diag-scripts/cookbook/imui-editor-controls-basics/cookbook-imui-editor-controls-basics-smoke.json --json
cargo run -p fretboard-dev -- diag run tools/diag-scripts/cookbook/imui-editor-controls-basics/cookbook-imui-editor-controls-basics-smoke.json --dir target/fret-diag/cookbook-imui-editor-controls-basics/2026-05-13-run6 --timeout-ms 180000 --exit-after-run --launch -- cargo run -p fret-cookbook --features cookbook-imui,cookbook-diag --example imui_editor_controls_basics
git diff --check
```

The docking M13 refresh also passed the four campaign manifest validation commands and the five
non-GUI behavior gates listed in
`docs/workstreams/docking-multiwindow-imgui-parity/M13_LOCAL_NONINTERACTIVE_GATE_REFRESH_2026-05-13.md`.
The launched M14 repair then turned the bounded campaign green and records the repaired session
evidence in
`docs/workstreams/docking-multiwindow-imgui-parity/M14_LAUNCHED_BOUNDED_CAMPAIGN_REPAIR_2026-05-13.md`.

## Next Concrete Follow-Ons

1. Continue `docking-multiwindow-imgui-parity` only with platform-specific real-host acceptance
   evidence or another narrow follow-on that proves a fresh gap.
2. Run a task-driven consumer audit across the current product chain and record the top friction
   points by owner layer before adding more helpers.
3. Continue DevTools discoverability on the diagnostics/DevTools lanes, not by widening `fret-imui`
   or `crates/fret-ui`.
4. Keep any future helper/API widening proof-led: two real first-party proof surfaces, one focused
   gate, and a clear owner layer.
