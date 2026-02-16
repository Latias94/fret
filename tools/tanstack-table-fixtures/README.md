# TanStack Table v8 fixture generator

This folder contains scripts for generating **TanStack Table v8 `table-core`** engine fixtures that
are committed and used by `fret-ui-headless` parity tests.

## Requirements

- Node.js (TypeScript stripping is used; this repo already runs `.mts` scripts via Node).
- A local checkout of TanStack Table (upstream: https://github.com/TanStack/table).
- The upstream `@tanstack/table-core` package built locally (this script imports the built CJS
  output from `packages/table-core/build/lib/index.js`).

If you're running from a git worktree (or any checkout) that does not have a local TanStack clone
under this repo, set either:

- `FRET_REPO_REF_TABLE` to the absolute path of your TanStack Table checkout root, or
- `FRET_REPO_REF_ROOT` to the absolute path of a local "repo-ref root" directory (the script will append `/table`).

## Generate fixtures

First ensure the upstream package is installed and built:

```bash
pnpm -C <tanstack-table-root> install --frozen-lockfile
pnpm -C <tanstack-table-root> -F @tanstack/table-core build
```

From the repo root (or any directory), run:

```bash
node tools/tanstack-table-fixtures/extract-fixtures.mts --out ecosystem/fret-ui-headless/tests/fixtures/tanstack/v8/demo_process.json
```

Notes:

- Fixtures are deterministic (no timestamps).
- The Rust tests should not require a local TanStack checkout; they only read committed JSON fixtures.
