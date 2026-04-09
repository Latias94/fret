# Fretboard Public App-Author Surface v1

Status: Closed
Last updated: 2026-04-09

Goal: define the smallest public `fretboard` surface that external Fret app authors should learn,
while keeping repo-owned maintainer flows on `fretboard-dev`.

Status note (2026-04-09): this lane is closed. The public/private/package taxonomy is now frozen
here and in `FINAL_STATUS.md`. Future work should open narrow implementation follow-ons instead of
reopening the product-taxonomy question.

This lane is a narrow follow-on to:

- `docs/workstreams/release-surface-fearless-refactor-v1/README.md`
- `docs/workstreams/fretboard-cli-fearless-refactor-v1/README.md`

It does **not** reopen parser-shell structure work. The typed split between `fretboard` and
`fretboard-dev` already landed. This lane owns the **product contract** question instead:
which commands belong to the public app-author story, and which commands remain repo-local
maintainer tooling.

Primary lane docs:

- `docs/workstreams/fretboard-public-app-author-surface-v1/README.md`
- `docs/workstreams/fretboard-public-app-author-surface-v1/DESIGN.md`
- `docs/workstreams/fretboard-public-app-author-surface-v1/TARGET_INTERFACE_STATE.md`
- `docs/workstreams/fretboard-public-app-author-surface-v1/DIAG_TARGET_INTERFACE_STATE.md`
- `docs/workstreams/fretboard-public-app-author-surface-v1/HOTPATCH_TARGET_INTERFACE_STATE.md`
- `docs/workstreams/fretboard-public-app-author-surface-v1/THEME_TARGET_INTERFACE_STATE.md`
- `docs/workstreams/fretboard-public-app-author-surface-v1/FINAL_STATUS.md`
- `docs/workstreams/fretboard-public-app-author-surface-v1/TODO.md`
- `docs/workstreams/fretboard-public-app-author-surface-v1/MILESTONES.md`
- `docs/workstreams/fretboard-public-app-author-surface-v1/EVIDENCE_AND_GATES.md`

## Current state snapshot (2026-04-09)

- Public `fretboard` currently exposes:
  - `assets`
  - `config`
  - `new`
- Repo-only `fretboard-dev` currently exposes:
  - `assets`
  - `config`
  - `dev`
  - `diag`
  - `hotpatch`
  - `list`
  - `new`
  - `theme`
- Shared user-facing command families already reuse the public implementation:
  - `apps/fretboard/src/assets.rs`
  - `apps/fretboard/src/config.rs`
  - `apps/fretboard/src/scaffold.rs`
- Future public `diag` target state is now frozen separately from the repo-only tree:
  - `docs/workstreams/fretboard-public-app-author-surface-v1/DIAG_TARGET_INTERFACE_STATE.md`
- Public hotpatch posture is now frozen separately from both public `dev` v1 and repo-only helper
  commands:
  - `docs/workstreams/fretboard-public-app-author-surface-v1/HOTPATCH_TARGET_INTERFACE_STATE.md`
- Theme import posture is now frozen as a non-`fretboard` public follow-on:
  - `docs/workstreams/fretboard-public-app-author-surface-v1/THEME_TARGET_INTERFACE_STATE.md`

## Problem statement

The current split is mechanically valid, but the long-term public product surface is still only
partially defined.

Today we have three kinds of commands mixed together:

1. commands that are already stable for any external app repo (`new`, `assets`, `config`);
2. commands that external app authors will eventually need, but whose current implementation still
   depends on this mono-repo (`dev`, large parts of `diag`);
3. commands that are clearly repo-owned maintainer utilities (`list`, repo demo launchers,
   workspace hotpatch helpers).

If we do not make that taxonomy explicit, docs drift accumulates:

- onboarding docs keep teaching repo-local spellings where the installed public CLI should be the
  product surface,
- ADRs keep mixing future goals with current shipped behavior,
- and future CLI growth risks re-exporting repo assumptions as a public contract.

## Assumptions-first snapshot

1. `Confident`: a public `fretboard` command must work in an external app repo without this
   mono-repo checkout.
   - Evidence:
     - `crates/fretboard/src/cli/contracts.rs`
     - `apps/fretboard/src/cli/mod.rs`
     - `.agents/skills/fret-external-app-mode/SKILL.md`
   - Consequence if wrong:
     - we would leak repo layout assumptions (`apps/`, `tools/`, workspace root discovery) into the
       public contract.

2. `Confident`: `new`, `assets`, and `config` belong on the public CLI.
   - Evidence:
     - `crates/fretboard/src/cli/contracts.rs`
     - `crates/fretboard/src/scaffold/mod.rs`
     - `docs/README.md`
   - Consequence if wrong:
     - external users lose the simplest installable onboarding lane and must fall back to
       repo-specific `cargo run -p ...` commands.

3. `Likely`: app authors eventually need a public `dev` command, but the current `dev`
   implementation is too repo-bound to publish as-is.
   - Evidence:
     - `apps/fretboard/src/dev/native.rs`
     - `apps/fretboard/src/dev/web.rs`
     - `apps/fretboard/src/demos.rs`
     - `docs/adr/0109-user-facing-crate-surfaces-and-golden-path.md`
   - Consequence if wrong:
     - we may over-invest in a public CLI lane that should remain plain Cargo + Trunk guidance.

4. `Likely`: diagnostics need a future public core, but not the entire current `fretboard-dev diag`
   tree.
   - Evidence:
     - `docs/ui-diagnostics-and-scripted-tests.md`
     - `apps/fretboard/src/diag.rs`
     - `crates/fret-diag/Cargo.toml`
   - Consequence if wrong:
     - we either under-serve app authors, or prematurely freeze a very large maintainer-oriented
       diagnostics surface.

5. `Likely`: hotpatch should stay subordinate to `dev` if it ever becomes public.
   - Evidence:
     - `docs/adr/0105-dev-hotpatch-subsecond-and-hot-reload-safety.md`
     - `apps/fretboard/src/hotpatch/contracts.rs`
     - `apps/fretboard/src/dev/native.rs`
   - Consequence if wrong:
     - we may publish top-level hotpatch helpers before the base run-loop contract is stable.

6. `Confident`: `theme import-vscode` is project-agnostic, but it still does not belong on public
   `fretboard` v1; if it becomes public later, it should do so as a dedicated package around
   `fret-vscode-theme`.
   - Evidence:
     - `apps/fretboard/src/theme.rs`
     - `apps/fretboard/src/theme/contracts.rs`
     - `ecosystem/fret-vscode-theme/Cargo.toml`
     - `docs/vscode-theme-import.md`
     - `docs/workstreams/fretboard-public-app-author-surface-v1/THEME_TARGET_INTERFACE_STATE.md`
   - Consequence if wrong:
     - we either keep a useful utility on the repo-only lane too long, or we bloat the main public
       CLI with a niche conversion workflow that should have been a focused package.

## Current direction

- Keep `new`, `assets`, and `config` on `fretboard`.
- Design a future public `dev` lane around project-local inputs (`--manifest-path`, package/bin/example
  selection), not repo demo IDs.
- Freeze a future public diagnostics **core** around user-supplied app commands and artifacts, not
  repo script catalogs or campaign presets.
- Keep `list` repo-only.
- Keep top-level `hotpatch` repo-only and reserve any future public hotpatch story for
  `dev native --hotpatch` only.
- Keep `theme import-vscode` off public `fretboard`; if it becomes public later, it should do so as
  a dedicated package around `fret-vscode-theme`.

Public `dev` target-state is frozen in:

- `docs/workstreams/fretboard-public-app-author-surface-v1/TARGET_INTERFACE_STATE.md`

Public `diag` target-state is frozen in:

- `docs/workstreams/fretboard-public-app-author-surface-v1/DIAG_TARGET_INTERFACE_STATE.md`

Public hotpatch posture is frozen in:

- `docs/workstreams/fretboard-public-app-author-surface-v1/HOTPATCH_TARGET_INTERFACE_STATE.md`

Theme import posture is frozen in:

- `docs/workstreams/fretboard-public-app-author-surface-v1/THEME_TARGET_INTERFACE_STATE.md`
