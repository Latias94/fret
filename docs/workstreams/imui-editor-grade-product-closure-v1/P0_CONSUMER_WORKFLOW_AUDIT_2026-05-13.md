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
