# M2 Identity Warnings Query - 2026-04-28

Status: landed

## Scope

This slice exposes the structured identity warnings added in M1 through the existing diagnostics
query surface:

- `fretboard diag query identity-warnings [SOURCE]`
- source resolution follows existing bundle lookup behavior for bundle dirs, run dirs,
  `bundle.json`, and `bundle.schema2.json`
- data is read from `debug.element_runtime.identity_warnings` in schema2 bundle snapshots

The command is a triage/query surface only. It does not add script capabilities, infer `test_id`
values from labels, expose render-pass or evaluation tokens, or change IMUI authoring identity APIs.

## Behavior

- Default output de-duplicates repeated observations of the same warning across later snapshots.
- `--timeline` keeps every matching snapshot observation for debugging capture history.
- Filters:
  - `--kind duplicate_keyed_list_item_key_hash|unkeyed_list_order_changed`
  - `--window <u64>`
  - `--element <u64>`
  - `--list-id <u64|0xHEX>`
  - `--element-path <TEXT>`
  - `--file <TEXT>`
- JSON output uses:
  - `kind: "query.identity_warnings"`
  - `filters`
  - `results[]` rows with snapshot anchors plus the warning fields

## Evidence

- `crates/fret-diag/src/commands/query.rs`
- `crates/fret-diag/src/cli/contracts/commands/query.rs`
- `crates/fret-diag/src/cli/cutover.rs`
- `crates/fret-diag/src/cli/contracts/mod.rs`
- `crates/fret-diag/src/diag_campaign.rs` (drive-by clippy cleanup discovered by the narrowed
  `fret-diag` gate)

## Gates

- `cargo nextest run -p fret-diag query_identity_warnings --no-fail-fast`
- `cargo check -p fret-diag --jobs 1`
- `cargo clippy -p fret-diag --all-targets -- -D warnings`

## Follow-Ons Still Deferred

- full interactive ID-stack browser,
- IMUI `for_each_keyed` duplicate-key authoring proof,
- label-to-`test_id` inference,
- table column identity.
