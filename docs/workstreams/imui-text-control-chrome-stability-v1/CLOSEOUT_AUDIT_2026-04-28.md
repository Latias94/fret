# ImUi Text Control Chrome Stability v1 - Closeout Audit - 2026-04-28

## Verdict

Close `imui-text-control-chrome-stability-v1` as a shipped narrow follow-on.

The lane removed the remaining shadcn input chrome dependency from IMUI text controls and proved the
intended compact field invariants without widening `crates/fret-ui`, shadcn recipes, or public
`fret-imui` contracts.

## Shipped Surface

- `input_text_model_with_options` now uses IMUI-specific compact text input chrome.
- `textarea_model_with_options` now derives textarea chrome from the same IMUI text-field policy.
- Focus does not configure an external `RingStyle` for IMUI text controls.
- Single-line IMUI input keeps the fixed `FIELD_MIN_HEIGHT` layout policy.

## Evidence

- `ecosystem/fret-ui-kit/src/imui/text_controls.rs`
- `ecosystem/fret-imui/src/tests/models_text.rs`
- `docs/workstreams/imui-text-control-chrome-stability-v1/M1_TEXT_CHROME_STABILITY_2026-04-28.md`
- `docs/workstreams/imui-text-control-chrome-stability-v1/EVIDENCE_AND_GATES.md`
- `docs/workstreams/imui-text-control-chrome-stability-v1/WORKSTREAM.json`

## Final Gate Set

```bash
cargo nextest run -p fret-ui-kit --features imui compact_imui_chrome_without_focus_ring --no-fail-fast
cargo nextest run -p fret-imui input_text_focus_keeps_control_bounds_stable --no-fail-fast
cargo check -p fret-ui-kit --features imui --jobs 2
cargo fmt --package fret-ui-kit --package fret-imui --check
python -m json.tool docs/workstreams/imui-text-control-chrome-stability-v1/WORKSTREAM.json
python tools/check_workstream_catalog.py
git diff --check
```

## Residual Gap Routing

- Future public style knobs for IMUI text controls need a separate API-proof lane.
- Future screenshot-only reports should first check whether the issue is compositor/pixel snapping,
  since the configured chrome no longer carries an outset focus ring.
- Do not reopen `imui-control-chrome-fearless-refactor-v1` or
  `imui-edit-lifecycle-hardening-v1` for this shipped slice.
