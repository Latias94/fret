# ImUi Text Control Chrome Stability v1 - Evidence and Gates

Status: closed
Last updated: 2026-04-28

## Repro

Smallest manual repro:

```bash
cargo run -p fret-demo --bin imui_response_signals_demo
```

Click the IMUI input fields. The intended behavior is that focus changes the compact border state
without making the field appear to grow from an external shadcn-style ring.

## Gates

```bash
cargo nextest run -p fret-ui-kit --features imui compact_imui_chrome_without_focus_ring --no-fail-fast
cargo nextest run -p fret-imui input_text_focus_keeps_control_bounds_stable --no-fail-fast
cargo check -p fret-ui-kit --features imui --jobs 2
cargo fmt --package fret-ui-kit --package fret-imui --check
python -m json.tool docs/workstreams/imui-text-control-chrome-stability-v1/WORKSTREAM.json
python tools/check_workstream_catalog.py
git diff --check
```

## Evidence Anchors

- `ecosystem/fret-ui-kit/src/imui/text_controls.rs`
- `ecosystem/fret-imui/src/tests/models_text.rs`
- `docs/workstreams/imui-control-chrome-fearless-refactor-v1/FINAL_STATUS.md`
- `docs/workstreams/imui-edit-lifecycle-hardening-v1/CLOSEOUT_AUDIT_2026-04-25.md`
- `repo-ref/ui/apps/v4/registry/new-york-v4/ui/input.tsx`
- `repo-ref/ui/apps/v4/registry/new-york-v4/ui/textarea.tsx`
- `repo-ref/imgui/imgui_widgets.cpp`
- `docs/workstreams/imui-text-control-chrome-stability-v1/CLOSEOUT_AUDIT_2026-04-28.md`

## Goal-Backward Verification

Truth:

- IMUI text controls do not configure a focus ring.
- IMUI single-line text input keeps a fixed `FIELD_MIN_HEIGHT`.
- shadcn input recipes remain unchanged and continue to own shadcn/new-york-v4 focus-ring parity.

Artifacts:

- IMUI compact chrome helpers in `text_controls.rs`.
- `fret-ui-kit --features imui` tests that inspect rendered element props.

Wiring:

- `input_text_model_with_options` and `textarea_model_with_options` consume the IMUI helpers.

Proof:

- Focused unit gate plus the existing `fret-imui` input bounds test.

Residual risk:

- A future report may need a screenshot or diagnostics script if the perceived size drift comes from
  a compositor/pixel-snap artifact rather than the configured chrome.
