# ImUi Interaction Inspector Diag Gate v1 Evidence And Gates

Status: closed gate list
Last updated: 2026-04-24

## Smallest Repro

```bash
cargo run -p fretboard-dev -- diag suite imui-interaction-inspector-diag-gate --launch -- cargo run -p fret-demo --bin imui_interaction_showcase_demo
```

The script clicks the pulse control and waits for both the header status and the inspector summary /
flag detail to report the primary click edge.

## Required Gates

```bash
cargo fmt --package fret-examples
cargo check -p fret-examples
cargo nextest run -p fret-examples imui_interaction_showcase --no-fail-fast
cargo build -p fret-demo --bin imui_interaction_showcase_demo
python3 tools/check_diag_scripts_registry.py
cargo run -p fretboard-dev -- diag registry check
cargo run -p fretboard-dev -- diag suite imui-interaction-inspector-diag-gate --launch -- cargo run -p fret-demo --bin imui_interaction_showcase_demo
python3 tools/check_workstream_catalog.py
python3 -m json.tool docs/workstreams/imui-interaction-inspector-diag-gate-v1/WORKSTREAM.json
git diff --check
```

## Evidence Anchors

- `apps/fret-examples/src/imui_interaction_showcase_demo.rs`
- `apps/fret-examples/src/lib.rs`
- `tools/diag-scripts/ui-editor/imui/imui-interaction-showcase-inspector-response-gate.json`
- `tools/diag-scripts/suites/imui-interaction-inspector-diag-gate/suite.json`
- `tools/diag-scripts/index.json`
- `docs/workstreams/imui-interaction-inspector-diag-gate-v1/WORKSTREAM.json`
- `docs/workstreams/imui-interaction-inspector-v1/CLOSEOUT_AUDIT_2026-04-24.md`

## Verified Gates

Passed on 2026-04-24:

```bash
cargo fmt --package fret-examples
cargo check -p fret-examples
cargo nextest run -p fret-examples imui_interaction_showcase --no-fail-fast
cargo build -p fret-demo --bin imui_interaction_showcase_demo
python3 tools/check_diag_scripts_registry.py
cargo run -p fretboard-dev -- diag registry check
cargo run -p fretboard-dev -- diag suite imui-interaction-inspector-diag-gate --launch -- cargo run -p fret-demo --bin imui_interaction_showcase_demo
python3 tools/check_workstream_catalog.py
python3 -m json.tool docs/workstreams/imui-interaction-inspector-diag-gate-v1/WORKSTREAM.json
git diff --check
```
