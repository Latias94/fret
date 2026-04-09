# Fretboard Public Dev Implementation v1 Final Status

Status: Closed
Closed on: 2026-04-09

## Outcome

This lane shipped the first public project-facing `fretboard dev` surface in `crates/fretboard`.

Landed surface:

- `fretboard dev native`
- `fretboard dev web`
- Cargo metadata-based package/target resolution
- public help/docs/ADR wording aligned to the shipped surface

The implementation stayed within the frozen taxonomy from
`docs/workstreams/fretboard-public-app-author-surface-v1/TARGET_INTERFACE_STATE.md`:

- public `fretboard` owns project-facing `dev`
- repo-only `fretboard-dev` retains demo ids, hotpatch orchestration, and maintainer shortcuts
- public `diag` / theme-import remain follow-ons

## Verification closed in this lane

- `cargo run -p fretboard -- --help`
- `cargo run -p fretboard -- dev --help`
- `cargo run -p fretboard -- dev native --help`
- `cargo run -p fretboard -- dev web --help`
- `cargo run -p fretboard -- dev native --manifest-path crates/fretboard/Cargo.toml --bin fretboard -- --help`
- `cargo nextest run -p fretboard`
- `cargo check -p fret-demo-web --target wasm32-unknown-unknown`
- `cargo fmt --check`
- `git diff --check`

Additional runtime evidence:

- `cargo run -p fretboard -- dev web --manifest-path apps/fret-demo-web/Cargo.toml --no-open --port 8091`
- observed `Fret web target ready: http://127.0.0.1:8091`
- `curl -I http://127.0.0.1:8091` returned HTTP 200
- `curl -s http://127.0.0.1:8091 | sed -n '1,20p'` returned the selected package-root HTML

## Notable closeout detail

The final web smoke initially exposed an unrelated repo compile break in
`crates/fret-launch/src/runner/web/effects.rs`: the `DiagInjectEvent` path referenced
`WinitEventContext` without importing it. Fixing that missing import was necessary to complete the
real `dev web` proof for this lane.

## Follow-on guidance

Do not reopen this lane for adjacent surface expansion.

Start narrower follow-ons for:

- public `diag`
- public hotpatch
- theme-import packaging or other publish-boundary work
