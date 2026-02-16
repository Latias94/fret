# Contributing to Fret

Thanks for your interest in contributing!

Fret is an experimental, documentation-driven UI framework. The fastest way to contribute is to
start from the docs and align changes with the existing layering/contract philosophy.

## Quick links

- Docs index: `docs/README.md`
- Architecture: `docs/architecture.md`
- ADR index: `docs/adr/README.md`
- Dependency/layering policy: `docs/dependency-policy.md`
- `repo-ref/` policy (optional upstream checkouts): `docs/repo-ref.md`

## Development setup

- Rust toolchain is pinned via `rust-toolchain.toml` (workspace MSRV is `1.92`).
- Use `cargo nextest` if available (recommended).

## Common commands

Format:

```bash
cargo fmt
```

Lint:

```bash
cargo clippy --workspace --all-targets -- -D warnings
```

Tests:

```bash
cargo nextest run
```

If you don’t have nextest:

```bash
cargo test --workspace
```

Build:

```bash
cargo build
```

## Running demos

Fret ships a dev tool called `fretboard` for consistent native/web demo runs:

```bash
cargo run -p fretboard -- dev native --bin todo_demo
cargo run -p fretboard -- dev web --demo ui_gallery
```

## Repo structure & layering rules (important)

At a high level:

- `crates/`: mechanism/contract surfaces and backend glue (stable boundaries)
- `ecosystem/`: policy-heavy components and fast-iterating surfaces
- `apps/`: runnable shells and end-to-end harnesses

When in doubt about “where should this go?”, prefer:

1) keep `crates/*` mechanism-only and portable,
2) move interaction defaults/policy into `ecosystem/*`,
3) lock hard-to-change behavior via ADRs before scaling surface area.

## Documentation/ADR workflow

This repo is docs/ADR-driven:

- If you change a contract (input/focus/docking/text/render boundaries), update or add an ADR under
  `docs/adr/`.
- If you implement/refactor behavior covered by an ADR, update
  `docs/adr/IMPLEMENTATION_ALIGNMENT.md` with evidence anchors (file paths/tests).

## Optional upstream references (`repo-ref/`)

Some audits and alignment docs reference upstream source paths under `repo-ref/` for convenience.
These directories are optional local state (ignored by git). See `docs/repo-ref.md` for how to
fetch them.

## Submitting changes

- Keep PRs focused and small when possible.
- Include screenshots/GIFs when behavior changes in the UI gallery or demos.
- Run `cargo fmt`, `cargo clippy ... -D warnings`, and tests before submitting.

