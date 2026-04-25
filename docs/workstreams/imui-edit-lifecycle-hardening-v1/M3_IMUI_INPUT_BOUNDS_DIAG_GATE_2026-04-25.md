# ImUi Edit Lifecycle Hardening v1 - M3 ImUi Input Bounds Diag Gate - 2026-04-25

## Decision

The M2 IMUI input sizing fix needs a rendered diagnostics proof because the original user-visible
failure was observed by clicking an input in an app, not only through retained-tree unit tests. The
diagnostics gate should stay in the policy/demo layer: it proves the public IMUI single-line helper
keeps fixed field height through click focus and typing without changing `crates/fret-ui` text-input
mechanism contracts.

## Shipped Invariant

- `imui_response_signals_demo` now has a focused script for the lifecycle text input bounds.
- The script waits for pre-focus bounds stability, clicks the input through `click_stable`, asserts
  focus, types text, and verifies the input remains at the fixed 24px IMUI field height in each
  phase.
- Layout sidecars are captured before focus, after focus, and after edit so future failures can be
  triaged from layout evidence instead of screenshots or manual observation.
- Exact before/after origin and size equality remains covered by
  `input_text_focus_keeps_control_bounds_stable`; the rendered gate covers the user-visible app path.

## Evidence

- `tools/diag-scripts/ui-editor/imui/imui-response-signals-input-bounds-stability.json`
- `tools/diag-scripts/suites/imui-response-signals-edit-lifecycle/suite.json`
- `tools/diag-scripts/index.json`
- `ecosystem/fret-imui/src/tests/models_text.rs`
- `ecosystem/fret-ui-kit/src/imui/text_controls.rs`

## Verification

Verified on 2026-04-25:

```bash
cargo run -p fretboard-dev -- diag run tools/diag-scripts/ui-editor/imui/imui-response-signals-input-bounds-stability.json --launch -- cargo run -p fret-demo --bin imui_response_signals_demo
cargo run -p fretboard-dev -- diag registry check
cargo run -p fretboard-dev -- diag suite imui-response-signals-edit-lifecycle --launch -- cargo run -p fret-demo --bin imui_response_signals_demo
python -m json.tool tools/diag-scripts/ui-editor/imui/imui-response-signals-input-bounds-stability.json
python -m json.tool tools/diag-scripts/suites/imui-response-signals-edit-lifecycle/suite.json
python tools/check_workstream_catalog.py
python -m json.tool docs/workstreams/imui-edit-lifecycle-hardening-v1/WORKSTREAM.json
git diff --check
```

The named suite resolves through the promoted registry, so `tools/diag-scripts/index.json` is part
of the shipped gate surface rather than an optional generated artifact.

## Residual Risk

This gate intentionally targets public IMUI single-line input. Multiline textarea and retained node
portal editor proof remain separate surfaces because their sizing policies are different.
