# Iconify Import-Pack Generator v1 — Milestones

Status: Active
Last updated: 2026-04-09

## M0 — Scope and evidence freeze

Exit criteria:

- The current icon generation toolchain is audited.
- The shipped predecessor contract is named explicitly.
- The lane clearly states what belongs here versus what stays closed in
  `icon-system-extension-v1`.

Primary evidence:

- `docs/workstreams/iconify-import-pack-generator-v1/DESIGN.md`
- `docs/workstreams/iconify-import-pack-generator-v1/TODO.md`
- `docs/workstreams/iconify-import-pack-generator-v1/EVIDENCE_AND_GATES.md`
- `docs/workstreams/icon-system-extension-v1/CLOSEOUT_AUDIT_2026-04-09.md`

Current status:

- Opened on 2026-04-09 as a narrow follow-on.
- The lane is indexed and its initial evidence/gate set is recorded.
- `BASELINE_AUDIT_2026-04-09.md` now freezes the exact current toolchain and generated-pack
  baseline.
- M0 exit criteria are satisfied.
- The next active work is M1 generator contract freeze.

## M1 — Generator contract freeze

Exit criteria:

- The v1 input boundary is explicit.
- The generated output boundary is explicit.
- The stable reusable/public surface is explicit.
- Provenance semantics for generated packs are explicit.

Primary evidence:

- `docs/workstreams/iconify-import-pack-generator-v1/M1_CONTRACT_FREEZE_2026-04-09.md`
- `tools/gen_icons.py`
- `tools/icon_codegen.py`
- `ecosystem/fret-icons/src/lib.rs`
- `ecosystem/fret-icons-lucide/src/lib.rs`
- `ecosystem/fret-icons-radix/src/lib.rs`

Current status:

- `M1_CONTRACT_FREEZE_2026-04-09.md` now freezes:
  - the supported local input boundary,
  - full-crate output expectations,
  - explicit alias-policy handling,
  - generated-pack provenance classification,
  - and the CLI/library direction for the future reusable surface.
- M1 exit criteria are satisfied.
- The next active work is M2 proof surface.

## M2 — Proof surface

Exit criteria:

- One smallest proof surface exercises the chosen generator boundary.
- The generated output fits the shipped icon-pack contract without ad-hoc manual fixes.
- The external-developer story is explicit rather than monorepo-only.

Primary gates:

- `python3 tools/check_icons_generation.py --pack all`
- `cargo nextest run -p fret-icons -p fret-icons-lucide -p fret-icons-radix`
- `cargo check -p fret-bootstrap --features "icons-lucide,icons-radix,icons-ui-semantic-lucide,icons-ui-semantic-radix"`

## M3 — Docs and regression closure

Exit criteria:

- The generator leaves a deterministic gate.
- The user-facing guidance is explicit.
- Future work can continue as narrower pack-specific/tooling follow-ons rather than reopening this
  contract lane.

Primary gates:

- `python3 tools/check_icons_generation.py --pack all`
- `cargo nextest run -p fret usage_docs_prefer_explicit_app_submodules_for_optional_ecosystems todo_golden_path_keeps_icon_pack_setup_on_app_install_surface`
