# ImUi Editor-Grade Product Closure Goal Completion Audit - 2026-05-15

Status: Not complete. Continue through narrow follow-ons.

This audit refreshes the 2026-05-13 completion read after the latest product-chain discovery,
docking Wayland admission, local policy-skip, perf-threshold, DevTools first-class gate UI, and
P4 performance-alignment slices. The 2026-05-16 M18 local policy-skip matrix keeps this verdict
current without changing the completion status. The 2026-05-16 perf-docking trace attribution
refresh also keeps the verdict unchanged: the product-chain perf entrypoint now has release-gated
real-span trace artifacts, but broad editor smoothness remains a separate owner-lane requirement.
It is deliberately strict: a green source gate or manifest only counts when it covers the actual
requirement being claimed.

## Objective Restatement

The active goal asks Fret to reach Dear ImGui-class editor usability while keeping Fret's layered
architecture intact:

1. Work on `main`.
2. Keep `fret-imui` thin and policy-light.
3. Put generic IMUI policy in `fret-ui-kit::imui`.
4. Put editor controls/composites in `fret-ui-editor`.
5. Put docking and multi-window policy in `fret-docking` plus runner/backend owners.
6. Use fearless refactoring to remove unsuitable design, duplicate aliases, and oversized private
   owner files when evidence supports it.
7. Compare gaps against local Dear ImGui references in `repo-ref/imgui`.
8. Leave repro, gate, and evidence for every meaningful slice.
9. Do not mark the goal complete until editor-grade workflow, diagnostics discoverability, docking
   hand-feel, performance discipline, and proof-led helper growth rules are all covered by real
   evidence.

## Prompt-To-Artifact Checklist

| Requirement | Current evidence | Verdict |
| --- | --- | --- |
| Work on `main` | `git status --short --branch` reports `main...origin/main`. | Met |
| Workstream state is explicit | `imui-editor-grade-product-closure-v1/WORKSTREAM.json`, `imui-imgui-gap-closure-v1/WORKSTREAM.json`, and `docking-multiwindow-imgui-parity/WORKSTREAM.json` are the current machine-readable anchors. | Met |
| `fret-imui` stays thin | `python tools/gate_imui_workstream_source.py` validates `ecosystem/fret-imui/Cargo.toml` dependencies and rejects policy/runtime owner drift. | Met |
| Generic IMUI policy stays in kit | `fret-ui-kit::imui` owns generic widget policy, response signals, options, tables, tabs, drag/drop, child regions, virtual lists, debug draw, and product-chain gates. | Met |
| Editor controls stay in editor crate | `ecosystem/fret-ui-editor/src/imui.rs` remains the thin adapter over editor controls/composites, with accessor-first response/event records guarded by `gate_imui_workstream_source.py`. | Met |
| Docking/multi-window stays out of runtime UI helpers | `docking-multiwindow-imgui-parity` keeps execution in `fret-docking`, diagnostics, and runner/backend owners; no `crates/fret-ui` widening is claimed. | Met |
| Fearless refactor is evidence-driven | Debug-draw owner split, facade owner splits, opaque response/state records, and redundant wrapper deletion are recorded in the active gap lane and guarded by source checks. | Met for recent slices |
| Dear ImGui comparison remains source-backed | `imui-imgui-gap-closure-v1` references `repo-ref/imgui` and now registers `P4_PERFORMANCE_ALIGNMENT_REVIEW_2026-05-06.md` for Zed/egui performance interpretation. | Met |
| Product workflow is coherent | `tools/diag_gate_imui_product_chain.py` validates cookbook, editor controls, editor proof, editor notes, workspace shell, docking campaign manifest, DevTools/tool-app discovery, and the perf-docking entrypoint metadata. The latest canonical release run at `target/imui-product-chain-perf-docking-trace-gate-2026-05-16-release-after-fix/1778898757233` passes both promoted perf scripts, records `failures=[]`, and writes real-span `trace.chrome.json` artifacts. | Partially met |
| First-contact editor-control evidence exists | `P0_CONSUMER_WORKFLOW_AUDIT_2026-05-13.md` records the launched `imui_editor_controls_basics` smoke and roughness typing suite evidence. | Met for first-contact editor controls |
| Editor-notes/workbench product evidence exists | `editor_notes_demo` and `editor_notes_device_shell_demo` suites are promoted into the product-chain gate, including the selection-sync and device-shell a11y repair evidence. | Met for current promoted scripts |
| DevTools/Demo/Metrics entrypoint discoverability is gated | `fretboard-dev --help`, `fretboard-dev list --help`, `list tool-apps`, `list tool-apps --json`, `product_workflows.imui-product-chain`, and the GUI `demo-metrics-debug` route are gated. The DevTools GUI first-class gate UI is closed for stale paint/scene, pixels-changed, perf thresholds, resource footprint thresholds, and selected-summary follow-ups. The M6 DevTools live-inspect payload, UI-gallery dogfood workflow, and 50k semantics-tree scalability slices are now recorded as closed in `docs/workstreams/diag-devtools-gui-v1/diag-devtools-gui-v1-todo.md` and `EVIDENCE_AND_GATES.md`. The M0 secondary layout/element tree entrypoints are also closed as semantics-derived views, with an explicit caveat that they are not full native layout-engine or declarative runtime snapshots. Remaining DevTools risk is broader always-available product maturity, not those specific M0/M6 bullets. | Partially met |
| Docking bounded campaign is green | `M14_LAUNCHED_BOUNDED_CAMPAIGN_REPAIR_2026-05-13.md` records the launched bounded P3 campaign passing 4/4 scripts. | Met for generic bounded campaign |
| Wayland source/admission posture is current | `M15_LOCAL_WAYLAND_BOUNDARY_REFRESH_2026-05-14.md`, `M16_SOURCE_DRIFT_GUARD_2026-05-14.md`, `M17_LOCAL_WAYLAND_POLICY_SKIP_GATE_2026-05-15.md`, and `M18_LOCAL_WAYLAND_POLICY_SKIP_MATRIX_2026-05-16.md` cover local source policy, manifest/script drift, the first non-Wayland `skipped_policy` gate, and the Windows plus Linux/X11 sidecar matrix. | Met for local policy gates |
| Full platform-specific hand-feel remains open | `DW-P1-linux-003` still requires a real Linux Wayland compositor run from `M5_WAYLAND_COMPOSITOR_ACCEPTANCE_RUNBOOK_2026-04-21.md`. | Not met |
| Performance discipline is guarded, not closed | Product-chain `perf-docking` now has CPU/layout/pointer/renderer thresholds, real-span trace attribution artifacts, DevTools selected-summary drill-down surfaces perf evidence, and P4 is registered, but broad smoothness attribution remains in dedicated perf lanes. | Partially met |
| Helper/API widening remains proof-led | The gap lane keeps widget/API widening candidate-only unless repeated first-party proof surfaces justify the owner and gate. | Met as a rule; not a completion claim |
| Full Dear ImGui-class editor maturity remains open | Remaining gaps include real-host OS-window hand-feel, DevTools GUI productization, perf attribution/smoothness, and future proof-led helper candidates. | Not met |

## Current Strengths

- The public IMUI lane is explicit: app code should teach `fret::imui`, not lower-level crate
  imports.
- The policy split remains intact: `fret-imui` is thin, generic policy stays in
  `fret-ui-kit::imui`, editor controls stay in `fret-ui-editor`, and docking stays with
  `fret-docking` plus runner/backend owners.
- The product-chain gate now covers the maintainer first-open route, promoted script/suite/campaign
  inputs, DevTools/tool-app discovery, and perf-docking threshold plus trace artifact expectations.
- The DevTools first-class gate UI is now a closed productization slice: shared `fret-diag`
  projections own gate templates and structured runnable args, while the GUI owns form UX,
  launch buttons, and bounded result histories.
- The Wayland local boundary is more honest than before: source/admission and non-Wayland
  policy-skip behavior are gated, including the M18 Windows plus Linux/X11 sidecar matrix, without
  pretending to close real-host Wayland acceptance.
- Performance pressure is now explicitly registered as a smoothness discipline problem, not a
  reason to copy Dear ImGui or egui runtime/API shape. The current product-chain perf-docking slice
  is release-gated for conservative thresholds and real-span trace attribution.

## Missing Or Weakly Verified Requirements

- **Real-host multi-window hand-feel remains open.** Local campaign validation, source drift guards,
  and policy-skip probes do not replace a Linux Wayland compositor acceptance run.
- **GUI productization is still not complete.** The current DevTools GUI slices close specific
  first-open, gate-builder, follow-up, live-inspect, and semantics-tree surfaces, but that is not
  yet the same as Dear ImGui-style always-available editor tooling across real workflows.
- **DevTools GUI productization remains partial.** The CLI/tool-app/product-workflow discovery path,
  GUI `demo-metrics-debug` route, first-class gate UI, live inspect overlay details, UI-gallery
  dogfood workflow, 50k semantics-tree scalability proof, and semantics-derived Layout/Elements
  secondary tree entrypoints are now gated, including a `--discovery-only --reuse-built` drift check
  for the first-open entrypoints. Dear ImGui-style always-available tooling still needs broader
  product maturity beyond those first-open, M0, and M6 closure slices.
- **Performance is guarded, not closed.** The `perf-docking` product-chain entrypoint now enforces
  conservative thresholds, emits real-span trace artifacts, and DevTools can surface selected perf
  evidence, but that is not the same as full smoothness attribution or broad editor workload
  acceptance.
- **Future helper/API growth remains intentionally constrained.** This is correct architecture, but
  it means Dear ImGui API breadth is not the completion criterion until repeated proof surfaces pay
  the same tax.

## Verification Snapshot

Latest focused checks for this audit:

```powershell
python -m json.tool docs/workstreams/imui-editor-grade-product-closure-v1/WORKSTREAM.json
python -m json.tool docs/workstreams/imui-imgui-gap-closure-v1/WORKSTREAM.json
python -m json.tool docs/workstreams/docking-multiwindow-imgui-parity/WORKSTREAM.json
python tools/gate_imui_workstream_source.py
python tools/diag_gate_imui_p2_devtools_first_open.py --discovery-only --reuse-built
python tools/diag_gate_imui_product_chain.py --only discovery
python tools/diag_gate_imui_product_chain.py --reuse-built --launched --only perf-docking --release --out-dir target/imui-product-chain-perf-docking-trace-gate-2026-05-16-release-after-fix
python tools/diag_gate_docking_wayland_policy_skip.py
python tools/check_workstream_catalog.py
git diff --check
```

## Next Concrete Follow-Ons

1. Run the real Linux Wayland compositor acceptance path for `DW-P1-linux-003` when an appropriate
   host is available.
2. Continue DevTools GUI productization on diagnostics/DevTools lanes, using the existing
   product-workflow discovery map instead of widening `fret-imui`.
3. Continue perf attribution/smoothness in `diag-perf-attribution-v1` and
   `ui-perf-zed-smoothness-v1`, with `perf-docking` as the current product-chain entrypoint.
4. Keep helper/API widening proof-led: two real first-party proof surfaces, one focused gate, and a
   clear owner layer before adding shared surface area.
