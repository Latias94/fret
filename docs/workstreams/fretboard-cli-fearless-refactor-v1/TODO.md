# Fretboard CLI Fearless Refactor v1 — TODO

Status: Active
Last updated: 2026-03-26

Related:

- Main note: `docs/workstreams/fretboard-cli-fearless-refactor-v1/README.md`
- Milestones: `docs/workstreams/fretboard-cli-fearless-refactor-v1/MILESTONES.md`
- Top-level shell: `apps/fretboard/src/cli/mod.rs`
- Top-level contract: `apps/fretboard/src/cli/contracts.rs`
- Top-level cutover: `apps/fretboard/src/cli/cutover.rs`
- Remaining manual lane: `apps/fretboard/src/scaffold/mod.rs`

Tracking legend:

- `[ ]` open
- `[~]` in progress
- `[x]` done
- `[>]` moved elsewhere
- `[!]` blocked

ID format:

- `FCR-{area}-{nnn}`

## M0 — Scope and break policy

- [x] FCR-docs-001 Create a dedicated workstream folder for the top-level CLI reset.
- [x] FCR-docs-002 Lock the no-compatibility direction for repo-owned top-level commands.
- [x] FCR-docs-003 Record the target modular structure (`cli/contracts.rs` + `cli/cutover.rs` +
  family-local `contracts.rs`).

## M1 — Land the modular shell shape

- [x] FCR-shell-010 Introduce a typed top-level `clap` tree in `apps/fretboard/src/cli/contracts.rs`.
- [x] FCR-shell-011 Introduce a dedicated cutover dispatcher in `apps/fretboard/src/cli/cutover.rs`.
- [x] FCR-shell-012 Keep diagnostics delegated to `crates/fret-diag` instead of duplicating its
  parser tree in `apps/fretboard`.

## M2 — Migrate repo-owned command families

- [x] FCR-family-020 Migrate `assets` to a typed family-local contract.
- [x] FCR-family-021 Migrate `dev` to a typed family-local contract and split execution modules.
- [x] FCR-family-022 Migrate `hotpatch` to a typed family-local contract.
- [x] FCR-family-023 Migrate `config` to a typed family-local contract.
- [x] FCR-family-024 Migrate `theme` to a typed family-local contract.
- [ ] FCR-family-025 Migrate `new` to a typed family-local contract.
- [ ] FCR-family-026 Delete `init` as a compatibility alias instead of forwarding it.

## M3 — Help and gate closure

- [ ] FCR-help-030 Replace the remaining hand-maintained root help command surface with a
  contract-driven or narrowly curated source that cannot drift from subcommand syntax.
- [x] FCR-help-031 Add parser/help coverage for the migrated `config` and `theme` families.
- [ ] FCR-help-032 Add parser/help coverage for the final `new` surface, including the wizard entry
  path and invalid template rejection.
- [ ] FCR-help-033 Grep and update repo-owned command snippets if `new` / `init` syntax changes.

## M4 — Closeout

- [ ] FCR-close-040 Confirm no repo-owned top-level command family still depends on manual argv
  loops.
- [ ] FCR-close-041 Write a short closeout note once `scaffold` and root help are on the final
  model.
