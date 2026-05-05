# ImUi Text Input Filter Policy v1 Closeout Audit - 2026-05-04

Status: Closed

## Verdict

This slice closes the Dear ImGui named character filter gap for single-line IMUI input text.

## Shipped

- Runtime mechanism: `TextInputProps::insert_filter` and `TextInputInsertFilter`.
- Runtime application: text input, single-line clipboard, primary selection, and platform text
  replacement insertion paths.
- IMUI policy: `InputTextFilters` with decimal, hexadecimal, scientific, uppercase, and no-blank
  options.
- Regression coverage for both runtime insertion filtering and public IMUI model-backed inputs.

## Boundaries Preserved

- `crates/fret-ui` owns only the generic insertion mechanism.
- `fret-ui-kit::imui` owns the Dear ImGui-shaped named-filter policy.
- `fret-imui` remains thin and only verifies the public authoring path.

## Follow-Ons

- `CallbackCharFilter`-style app policy hooks are covered by
  `docs/workstreams/imui-text-input-custom-filter-policy-v1/`.
- Undo/redo command routing is covered by `docs/workstreams/imui-text-input-undo-command-policy-v1/`.
- The first visible completion/history picker recipe is covered by
  `docs/workstreams/imui-text-input-picker-recipe-v1/`.
- Deeper multiline policy remains open.
