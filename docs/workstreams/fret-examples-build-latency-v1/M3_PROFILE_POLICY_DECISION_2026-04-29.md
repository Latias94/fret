# Fret Examples Build Latency v1 - M3 Profile Policy Decision - 2026-04-29

Status: complete

## Decision

Keep `profile.dev.package.fret-examples.incremental = false` in the default `dev` profile as the
conservative macOS incremental-link workaround.

Add an explicit `profile.dev-fast.package.fret-examples.incremental = true` override for local
speed iteration. On Windows, `fretboard-dev dev native ...` already defaults to `dev-fast`; for
standalone IMUI iteration, prefer direct `fret-examples-imui` bins and checks instead of routing
through the broad `fret-examples` compatibility crate.

## Rationale

- Cargo package profile overrides in the workspace manifest are profile-scoped. Encoding this as a
  hidden target-specific policy in `Cargo.toml` would be misleading, so the platform split belongs
  in documented profile usage.
- The default `dev` profile remains the safe path for repeated broad `fret-demo` and
  `fret-examples` compatibility runs on macOS.
- `dev-fast` is an explicit local iteration profile, and `fretboard-dev` already chooses it by
  default on Windows where PDB/link time dominates.
- The M2 split gives IMUI work a better fast path: `fret-examples-imui` does not depend on the
  monolithic `fret-examples` crate.

## Gates

```text
cargo check -p fret-examples --lib --jobs 1
cargo check -p fret-examples-imui --bins --profile dev-fast --jobs 1
cargo check -p fret-examples --lib --profile dev-fast --jobs 1
python tools/check_workstream_catalog.py
python -m json.tool docs/workstreams/fret-examples-build-latency-v1/WORKSTREAM.json
git diff --check
```

## Evidence

- Passed on Windows with Cargo 1.92.0: `cargo check -p fret-examples --lib --jobs 1`.
- Passed on Windows with Cargo 1.92.0: `cargo check -p fret-examples-imui --bins --profile
  dev-fast --jobs 1`.
- Passed on Windows with Cargo 1.92.0: `cargo check -p fret-examples --lib --profile dev-fast
  --jobs 1`.
- Existing docs already point Windows demo iteration at `fretboard-dev dev native ...`, which
  defaults to `--profile dev-fast`.
