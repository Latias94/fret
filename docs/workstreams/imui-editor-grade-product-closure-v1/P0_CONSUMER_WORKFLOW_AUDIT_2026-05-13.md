# P0 Consumer Workflow Audit - 2026-05-13

Status: consumer audit; no new API or implementation-heavy lane opened from this note.

This audit follows the current public/product IMUI chain as a framework consumer would:

1. find the IMUI lessons from the docs/tooling index,
2. build the first-contact cookbook lessons,
3. build the heavier editor/workbench/docking proof surfaces,
4. identify friction by owner layer before adding helpers.

## Consumer Story

Primary lane: complex app / ecosystem-fit, anchored in the editor notes / workspace shell family.

Task:

- Start from the public examples docs and cookbook index.
- Find the generic IMUI lesson, the editor-control lesson, the heavier editor proof, the workspace
  shell proof, and the docking proof.
- Verify the non-GUI compile path for those proof surfaces.
- Record only friction that affects a framework consumer; do not widen `fret-imui` or
  `fret-ui-kit::imui` from this audit alone.

## Commands Run

```powershell
rg -n "imui_action_basics|imui_editor_controls_basics|imui_editor_proof_demo|workspace_shell_demo|diagnostics-first-open|docking_arbitration_demo|fretboard" docs/examples/README.md apps/fret-cookbook/README.md apps/fret-cookbook/EXAMPLES.md docs/diagnostics-first-open.md docs/workstreams/imui-editor-grade-product-closure-v1/GOAL_COMPLETION_AUDIT_2026-05-13.md
rg --files apps/fret-cookbook/examples apps/fret-demo/src/bin apps/fret-examples/src | rg "imui_action_basics|imui_editor_controls_basics|imui_editor_proof_demo|workspace_shell_demo|docking_arbitration_demo"
cargo check -p fret-cookbook --features cookbook-imui --example imui_action_basics --example imui_editor_controls_basics
cargo check -p fret-demo --bin imui_editor_proof_demo --bin workspace_shell_demo --bin docking_arbitration_demo
cargo run -p fretboard-dev -- list cookbook-examples --all
cargo run -p fretboard-dev -- list --help
cargo run -p fretboard-dev -- list native-demos
cargo run -p fretboard-dev -- list native-demos --all
cargo run -p fretboard-dev -- diag script validate tools/diag-scripts/cookbook/imui-editor-controls-basics/cookbook-imui-editor-controls-basics-smoke.json --json
cargo run -p fretboard-dev -- diag run tools/diag-scripts/cookbook/imui-editor-controls-basics/cookbook-imui-editor-controls-basics-smoke.json --dir target/fret-diag/cookbook-imui-editor-controls-basics/2026-05-13-run6 --timeout-ms 180000 --exit-after-run --launch -- cargo run -p fret-cookbook --features cookbook-imui,cookbook-diag --example imui_editor_controls_basics
cargo run -p fretboard-dev -- diag suite cookbook-imui-editor-controls-basics --launch -- cargo run -p fret-cookbook --features cookbook-imui,cookbook-diag --example imui_editor_controls_basics
```

## Findings

| Severity | Lane | Finding | Evidence | Owner | Next move |
| --- | --- | --- | --- | --- | --- |
| P0 | Product chain | No blocking compile-path break was found for the current source-backed chain. | The two cookbook IMUI lessons and the three `fret-demo` proof binaries passed `cargo check`. | Existing owners | Keep current owner split. |
| P1 | Discoverability | A consumer using only `list cookbook-examples --all` sees the focused IMUI lessons but not the heavier product proof, because `imui_editor_proof_demo` correctly lives under `fret-demo`. | `list cookbook-examples --all` shows the three IMUI cookbook lessons; `list native-demos --all` shows `imui_editor_proof_demo`, `workspace_shell_demo`, and `docking_arbitration_demo`. | Docs / tooling docs | Patch docs to mention `list native-demos --all` as the product-proof discovery command. |
| P1 | Diagnostics evidence | `imui_editor_controls_basics` is the editor-control first-contact lesson and now has a focused diag suite for reviewable layout/screenshot/bundle evidence plus roughness typing-mode regression coverage. | `tools/diag-scripts/cookbook/imui-editor-controls-basics/cookbook-imui-editor-controls-basics-smoke.json`, `tools/diag-scripts/cookbook/imui-editor-controls-basics/cookbook-imui-editor-controls-roughness-typing.json`, `tools/diag-scripts/suites/cookbook-imui-editor-controls-basics/suite.json`, and `target/fret-diag/cookbook-imui-editor-controls-basics/2026-05-13-run6/1778653020746-cookbook-imui-editor-controls-basics-smoke`. | Cookbook / diagnostics | Keep this as the first-contact editor-control visual gate; add further interaction-specific scripts only when a real consumer task needs them. |
| P2 | Product workflow | The product chain is documented, but there is no one-command "walk the IMUI product chain" audit gate. | Current gates are intentionally split across source checks, compile checks, diagnostics, and docking manifests. | Tooling / workstream docs | Keep split gates for now; add a bundled audit command only after this repeats as real maintenance friction. |

## Patch From This Audit

The docs now point readers from the cookbook-focused IMUI lessons to the native demo discovery
surface:

```powershell
cargo run -p fretboard-dev -- list native-demos --all
```

This keeps the product proof in `fret-demo` while making the transition from cookbook lesson to
heavier editor proof easier to discover.

The audit also promoted a focused diag suite for `imui_editor_controls_basics` so the first-contact
editor-control lesson has layout, screenshot, and bundle evidence instead of compile evidence only.
The launched local proof passed on 2026-05-13 as `PASS (run_id=1778653020152)` with:

- layout sidecar:
  `target/fret-diag/cookbook-imui-editor-controls-basics/2026-05-13-run6/1778653020648-cookbook-imui-editor-controls-basics-smoke.layout/layout.taffy.v1.json`
- screenshot:
  `target/fret-diag/cookbook-imui-editor-controls-basics/2026-05-13-run6/screenshots/1778653020668-cookbook-imui-editor-controls-basics-smoke/window-4294967297-tick-34-frame-33.png`
- final bundle:
  `target/fret-diag/cookbook-imui-editor-controls-basics/2026-05-13-run6/1778653020746-cookbook-imui-editor-controls-basics-smoke/bundle.schema2.json`

The documented suite command also passed on 2026-05-13 with both scripts:

- smoke: `PASS ... (run_id=1778653340628)`
- roughness typing: `PASS ... (run_id=1778653344599)`

Its `suite.summary.json` reported `scripts_with_evidence: 2` and `warning_issues: 0` for both
bundles after hidden typing inputs stopped publishing inactive `.typing.input` test ids.

## Verdict

- Do not add a new workstream from this audit alone.
- Do not widen `fret-imui`, `fret-ui-kit::imui`, or `crates/fret-ui`.
- The next implementation-heavy work should still be one of:
  - real-host docking hand-feel evidence,
  - DevTools discoverability productization,
  - a narrower interaction-specific editor-control script only after a concrete consumer task needs
    it.

## 2026-05-14 Product-Chain Evidence Refresh

The heavier editor-notes/workbench chain already had focused scripts, but they were not promoted
as product-chain suites. This refresh adds two suite manifests and wires them into
`tools/diag_gate_imui_product_chain.py`:

- `tools/diag-scripts/suites/editor-notes-demo/suite.json` validates the app-local
  `editor_notes_demo` preserved multiline draft script, draft-controller commit/discard script, and
  asset selection -> inspector sync script.
- `tools/diag-scripts/suites/editor-notes-device-shell-demo/suite.json` validates the adaptive
  `editor_notes_device_shell_demo` responsive desktop-rail/mobile-drawer proof.

The suites stay separate because they launch different binaries. This is a gate/productization
improvement, not API widening and not a reason to widen `fret-imui`,
`fret-ui-kit::imui`, or `crates/fret-ui`.

The product-chain gate also supports `--reuse-built` for launched `fret-demo` surfaces so heavy
editor/workbench diagnostics can run against existing binaries without turning build-lock timing
into product-chain signal.

Follow-up source-gate refresh on 2026-05-14: `tools/diag_gate_imui_product_chain.py` now treats
`docking_arbitration_demo` as a campaign-backed product surface and runs
`diag campaign validate tools/diag-campaigns/imui-p3-multiwindow-parity.json --json` in the default
lightweight maintainer gate. This keeps the product chain honest about the discovered docking
surface without running the launched multi-window campaign by default.

Follow-up DevTools discoverability refresh on 2026-05-14: the same product-chain gate now validates
`fretboard-dev list tool-apps --json` as a stable machine-readable first-open map. The check covers
the `fretboard_tool_apps` kind, schema version, canonical diagnostics first-open doc, GUI branch
doc, repo preflight command/JSON command/purpose, and the DevTools GUI/MCP command/docs/gate/best-for
fields. This is still a gate/productization improvement, not API widening and not a reason to widen
`fret-imui`, `fret-ui-kit::imui`, or `crates/fret-ui`.

Follow-up CLI entrypoint refresh on 2026-05-14: the product-chain discovery gate also validates
`fretboard-dev --help` and `fretboard-dev list --help`, so the `tool-apps` index remains
discoverable before a maintainer already knows the exact `list tool-apps` subcommand.

Follow-up product workflow discovery refresh on 2026-05-15: `fretboard-dev list tool-apps` now
prints a `workflow: imui-product-chain` row, and `fretboard-dev list tool-apps --json` exposes that
same route as `product_workflows`. The default product-chain discovery gate validates the default
gate command, the focused discovery command, the launched `perf-docking` command, the promoted
`perf-docking-arbitration-steady` suite, and the expected `perf-docking/regression.summary.json` /
`perf-docking/check.perf_thresholds.json` artifacts. This is still a tooling/discoverability
refresh, not a reason to widen `fret-imui`, `fret-ui-kit::imui`, or `crates/fret-ui`.

Local verification on 2026-05-14 passed with run root
`target/imui-product-chain-editor-notes-launched-2026-05-14-reuse/1778729721045`: the
`editor-notes-demo` suite passed 2/2 scripts, and the `editor-notes-device-shell-demo` suite passed
1/1 script.

Follow-up local verification on 2026-05-14 passed with run root
`target/imui-product-chain-editor-notes-selection-sync-2026-05-14-r3/1778735909022`: the expanded
`editor-notes-demo` suite passed 3/3 scripts with `scripts_with_evidence: 3` and `warning_issues:
0` for all three script lint outputs. The new `editor-notes-demo-selection-sync` script covers
asset selection, inspector field binding, collection summary, and summary-command status as one
product workflow. The first launched run exposed a mechanism-layer selector bug: model-backed
selector dependency signatures used only model revision, so switching to a different same-revision
asset could replay stale derived UI text. The fix stays in `fret-selector` by including `ModelId`
before revision in model-backed dependency signatures.

The follow-up accessibility repair stays in the shared headless overlay policy layer rather than in
the demo: `fret-ui-kit` now hides modal backdrop/barrier pressables from the accessibility tree
while keeping pointer dismissal working. Local verification on 2026-05-14 passed with run root
`target/imui-product-chain-editor-notes-device-shell-a11y-2026-05-14/1778731960670`; the
`editor-notes-device-shell-demo` suite passed 1/1 script and its lint output reported
`warning_issues: 0` and `findings: []`.

Follow-up perf entrypoint refresh on 2026-05-14: the product-chain gate now treats
`tools/diag-scripts/suites/perf-docking-arbitration-steady/suite.json` as the current docking perf
entrypoint and adds a launched `perf-docking` slice. The first local run exposed a diagnostics
tooling bug rather than an app bug: `diag perf` printed human `PERF ...` rows but wrote a failed
`regression.summary.json` unless `--json` was used. The repair keeps row evidence internal for both
stdout modes in `crates/fret-diag/src/diag_perf.rs`, with the focused test
`perf_regression_summary_uses_rows_when_stdout_is_human`. The follow-up artifact projection test
`perf_row_to_regression_item_uses_single_run_bundle_artifact` keeps single-run bundle paths visible
as `bundle_artifact` evidence, and
`perf_row_to_regression_item_projects_single_run_metrics` /
`perf_row_to_regression_item_projects_repeat_stats_metrics` keep curated perf metrics visible in
`evidence.extra.metrics`. The repaired local run at
`target/imui-product-chain-perf-docking-metrics-gate-2026-05-14/1778775354481/perf-docking/regression.summary.json`
reports two passing `perf_case` rows and `failed_tooling=0`.

Follow-up docking perf threshold refresh on 2026-05-15: the product-chain `perf-docking` slice now
passes conservative `diag perf` thresholds before accepting the IMUI docking smoothness entrypoint.
The gate launches with `--max-top-total-us 20000`, `--max-top-layout-us 10000`,
`--max-top-solve-us 10000`, `--max-pointer-move-dispatch-us 5000`,
`--max-pointer-move-hit-test-us 5000`, and `--max-pointer-move-global-changes 0`; it then verifies
that each `perf_case` item exposes readable `compare_json` evidence and empty
`threshold_failures`. The local threshold artifact at
`target/imui-product-chain-perf-docking-threshold-gate-2026-05-15/1778776635280/perf-docking/check.perf_thresholds.json`
reports `kind=perf_thresholds`, `observed_aggregate=max`, and `failures=[]`, while
`target/imui-product-chain-perf-docking-threshold-gate-2026-05-15/1778776635280/perf-docking/regression.summary.json`
reports two passing rows with `wants_perf_thresholds=true`.

Follow-up renderer threshold refresh on 2026-05-15: the previous diagnostics model already measured
renderer encode/upload/record/finish/text/SVG/payload fields, but `diag perf` did not expose CLI
thresholds for them. The repair adds renderer threshold flags such as
`--max-renderer-encode-scene-us`, `--max-renderer-upload-us`, and
`--max-renderer-instance-bytes`, projects them into `check.perf_thresholds.json`, and wires them
into the same launched `perf-docking` product-chain slice. The local run at
`target/imui-product-chain-perf-docking-renderer-threshold-gate-2026-05-15/1778778141759/perf-docking/regression.summary.json`
reports two passing `perf_case` rows; the paired
`target/imui-product-chain-perf-docking-renderer-threshold-gate-2026-05-15/1778778141759/perf-docking/check.perf_thresholds.json`
reports `failures=[]` and CLI-sourced renderer threshold rows.
