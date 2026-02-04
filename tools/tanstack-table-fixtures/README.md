# TanStack Table v8 fixture generator

This folder contains scripts for generating **TanStack Table v8 `table-core`** engine fixtures that
are committed and used by `fret-ui-headless` parity tests.

## Requirements

- Node.js (TypeScript stripping is used; this repo already runs `.mts` scripts via Node).
- A local checkout of TanStack Table under `repo-ref/table` in the main repo checkout.
- The upstream `@tanstack/table-core` package built locally (this script imports the built CJS
  output from `packages/table-core/build/lib/index.js`).

If you're running from a git worktree that does not contain `repo-ref/`, set:

- `FRET_REPO_REF_TABLE` to the absolute path of `repo-ref/table`, or
- `FRET_REPO_REF_ROOT` to the absolute path of `repo-ref/` (the script will append `/table`).

## Generate fixtures

First ensure the upstream package is installed and built:

```bash
pnpm -C repo-ref/table install --frozen-lockfile
pnpm -C repo-ref/table -F @tanstack/table-core build
```

From the repo root (or any directory), run:

```bash
node tools/tanstack-table-fixtures/extract-fixtures.mts --out ecosystem/fret-ui-headless/tests/fixtures/tanstack/v8/demo_process.json
```

Notes:

- Fixtures are deterministic (no timestamps).
- The Rust tests should not require `repo-ref/table`; they only read committed JSON fixtures.
