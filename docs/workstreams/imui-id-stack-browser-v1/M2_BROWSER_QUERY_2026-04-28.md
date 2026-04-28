# M2 Browser Query Result - 2026-04-28

Status: landed

## Result

`diag query identity-warnings` now has an explicit browser query mode:

```text
fretboard-dev diag query identity-warnings <bundle-or-dir> --browser --json
```

Default `identity-warnings` output remains row-compatible with the existing query surface. The new
`--browser` flag adds:

- `browser: true`,
- `summary` with total, matching, deduped, returned row, and group counts,
- `groups` over warning kind, window, frame id, source file, list id, key hash, and element path.

The command still supports the existing filters:

- `--kind`
- `--window`
- `--element`
- `--list-id`
- `--element-path`
- `--file`
- `--timeline`
- `--top`

`--json` and `--out` continue to work for automation and future dashboard/HTML reuse.

## Compatibility

- Without `--browser`, the top-level query payload keeps the prior shape.
- With `--browser`, grouped diagnostics are opt-in and explicit.
- Clap contract and legacy cutover tests cover the new flag.

## Evidence

- `crates/fret-diag/src/identity_browser.rs`
- `crates/fret-diag/src/commands/query.rs`
- `crates/fret-diag/src/cli/contracts/commands/query.rs`
- `crates/fret-diag/src/cli/contracts/mod.rs`
- `crates/fret-diag/src/cli/cutover.rs`

## Gates

- `cargo nextest run -p fret-diag identity_browser --no-fail-fast`
- `cargo nextest run -p fret-diag query_identity_warnings --no-fail-fast`
- `cargo check -p fret-diag --jobs 1`
- `cargo fmt --package fret-diag --check`
