# Iconify Presentation Defaults Suggestion v1 — Milestones

Status: Closed
Last updated: 2026-04-09

## M0 — Scope and evidence freeze

Exit criteria:

- The helper problem is explicitly separated from the closed generated-defaults and acquisition
  lanes.
- The available provenance evidence and missing helper gap are audited.
- The lane's non-goals are explicit.

Primary evidence:

- `docs/workstreams/iconify-presentation-defaults-suggestion-v1/DESIGN.md`
- `docs/workstreams/iconify-presentation-defaults-suggestion-v1/TODO.md`
- `docs/workstreams/iconify-presentation-defaults-suggestion-v1/BASELINE_AUDIT_2026-04-09.md`
- `docs/workstreams/generated-icon-presentation-defaults-v1/CLOSEOUT_AUDIT_2026-04-09.md`
- `docs/workstreams/iconify-acquisition-prestep-v1/CLOSEOUT_AUDIT_2026-04-09.md`

Current status:

- M0 evidence freeze closed on 2026-04-09.

## M1 — Suggestion contract freeze

Exit criteria:

- Helper ownership is explicit.
- Input/output contract is explicit.
- Missing-evidence behavior is explicit.

Primary evidence:

- `docs/workstreams/iconify-presentation-defaults-suggestion-v1/DESIGN.md`
- `docs/workstreams/iconify-presentation-defaults-suggestion-v1/BASELINE_AUDIT_2026-04-09.md`
- `docs/workstreams/iconify-presentation-defaults-suggestion-v1/M1_CONTRACT_FREEZE_2026-04-09.md`

Current status:

- M1 contract freeze closed on 2026-04-09.

## M2 — Proof surface

Exit criteria:

- Thin helper CLI surface lands.
- Emitted suggestion matches the existing config schema.
- Suggested config flows into the current import path.

Primary gates:

- `cargo nextest run -p fretboard`
- `cargo nextest run -p fret usage_docs_prefer_explicit_app_submodules_for_optional_ecosystems todo_golden_path_keeps_icon_pack_setup_on_app_install_surface`

Current status:

- M2 proof surface closed on 2026-04-09.
- See `docs/workstreams/iconify-presentation-defaults-suggestion-v1/M2_PROOF_SURFACE_2026-04-09.md`.

## M3 — Docs and closeout

Exit criteria:

- Deterministic gates define the shipped helper.
- Public docs teach the helper as advisory.
- The lane closes explicitly.

Primary gates:

- `cargo nextest run -p fretboard`
- `cargo nextest run -p fret usage_docs_prefer_explicit_app_submodules_for_optional_ecosystems todo_golden_path_keeps_icon_pack_setup_on_app_install_surface`

Current status:

- M3 docs and closeout closed on 2026-04-09.
- The lane is now closed on
  `docs/workstreams/iconify-presentation-defaults-suggestion-v1/CLOSEOUT_AUDIT_2026-04-09.md`.
