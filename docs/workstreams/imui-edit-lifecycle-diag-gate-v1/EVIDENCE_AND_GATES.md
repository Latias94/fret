# ImUi Edit Lifecycle Diag Gate v1 Evidence And Gates

Status: closed gate list
Last updated: 2026-04-24

## Smallest Repros

```bash
cargo run -p fretboard-dev -- diag suite imui-response-signals-edit-lifecycle --launch -- cargo run -p fret-demo --bin imui_response_signals_demo
cargo run -p fretboard-dev -- diag suite imui-editor-proof-edit-outcomes --launch -- cargo run -p fret-demo --bin imui_editor_proof_demo
```

The first suite proves the narrow response lifecycle counters. The second suite keeps the wider
editor-proof drag/text/numeric outcome surface from drifting while still launching the matching
demo.

## Required Gates

```bash
cargo fmt --package fret-examples
cargo check -p fret-examples
cargo test -p fret-examples imui_editor_proof_demo::tests
cargo build -p fret-demo --bin imui_response_signals_demo
cargo build -p fret-demo --bin imui_editor_proof_demo
python tools/check_diag_scripts_registry.py
cargo run -p fretboard-dev -- diag registry check
cargo run -p fretboard-dev -- diag suite imui-response-signals-edit-lifecycle --launch -- cargo run -p fret-demo --bin imui_response_signals_demo
cargo run -p fretboard-dev -- diag suite imui-editor-proof-edit-outcomes --launch -- cargo run -p fret-demo --bin imui_editor_proof_demo
python tools/check_workstream_catalog.py
python -m json.tool docs/workstreams/imui-edit-lifecycle-diag-gate-v1/WORKSTREAM.json
git diff --check
```

## Evidence Anchors

- `apps/fret-examples/src/imui_response_signals_demo.rs`
- `apps/fret-examples/src/imui_editor_proof_demo.rs`
- `tools/diag-scripts/ui-editor/imui/imui-response-signals-edit-lifecycle-gate.json`
- `tools/diag-scripts/ui-editor/imui/imui-editor-proof-drag-value-outcomes.json`
- `tools/diag-scripts/ui-editor/imui/imui-editor-proof-text-numeric-baseline-policy.json`
- `tools/diag-scripts/suites/imui-response-signals-edit-lifecycle/suite.json`
- `tools/diag-scripts/suites/imui-editor-proof-edit-outcomes/suite.json`
- `tools/diag-scripts/index.json`
- `docs/workstreams/imui-edit-lifecycle-diag-gate-v1/WORKSTREAM.json`

## Verified Gates

Passed on 2026-04-24:

```bash
cargo fmt --package fret-examples
cargo check -p fret-examples
cargo test -p fret-examples imui_editor_proof_demo::tests
cargo build -p fret-demo --bin imui_response_signals_demo
cargo build -p fret-demo --bin imui_editor_proof_demo
python tools/check_diag_scripts_registry.py
cargo run -p fretboard-dev -- diag registry check
cargo run -p fretboard-dev -- diag run tools/diag-scripts/ui-editor/imui/imui-editor-proof-text-numeric-baseline-policy.json --launch -- cargo run -p fret-demo --bin imui_editor_proof_demo
cargo run -p fretboard-dev -- diag suite imui-editor-proof-edit-outcomes --launch -- cargo run -p fret-demo --bin imui_editor_proof_demo
cargo run -p fretboard-dev -- diag suite imui-response-signals-edit-lifecycle --launch -- cargo run -p fret-demo --bin imui_response_signals_demo
python tools/check_workstream_catalog.py
python -m json.tool docs/workstreams/imui-edit-lifecycle-diag-gate-v1/WORKSTREAM.json
git diff --check
```
