# Closeout Audit — 2026-04-09

Status: closed closeout record

Related:

- `docs/workstreams/iconify-presentation-defaults-suggestion-v1/DESIGN.md`
- `docs/workstreams/iconify-presentation-defaults-suggestion-v1/BASELINE_AUDIT_2026-04-09.md`
- `docs/workstreams/iconify-presentation-defaults-suggestion-v1/M1_CONTRACT_FREEZE_2026-04-09.md`
- `docs/workstreams/iconify-presentation-defaults-suggestion-v1/M2_PROOF_SURFACE_2026-04-09.md`
- `docs/workstreams/iconify-presentation-defaults-suggestion-v1/TODO.md`
- `docs/workstreams/iconify-presentation-defaults-suggestion-v1/MILESTONES.md`
- `docs/workstreams/iconify-presentation-defaults-suggestion-v1/EVIDENCE_AND_GATES.md`
- `docs/workstreams/generated-icon-presentation-defaults-v1/CLOSEOUT_AUDIT_2026-04-09.md`
- `docs/workstreams/iconify-acquisition-prestep-v1/CLOSEOUT_AUDIT_2026-04-09.md`
- `docs/adr/0065-icon-system-and-asset-packaging.md`
- `docs/adr/IMPLEMENTATION_ALIGNMENT.md`
- `docs/crate-usage-guide.md`
- `docs/examples/todo-app-golden-path.md`
- `crates/fretboard/src/icons/contracts.rs`
- `crates/fretboard/src/icons/suggest.rs`
- `crates/fretboard/src/icons/mod.rs`
- `crates/fretboard/src/icons/acquire.rs`
- `crates/fretboard/src/cli/help.rs`
- `ecosystem/fret/src/lib.rs`

## Verdict

This lane is now closed.

It successfully landed the narrow helper follow-on that the generated-defaults closeout explicitly
left open:

- a thin explicit suggestion command,
- provenance-driven pack-level default derivation only when evidence is strong,
- reuse of the existing `presentation-defaults.json` contract,
- end-to-end proof that the suggestion flows into import,
- and public docs that teach the helper as advisory rather than normative import policy.

## What shipped

### 1) Explicit helper stage

The public surface now distinguishes:

- `icons acquire ...`
- `icons suggest presentation-defaults ...`
- `icons import ...`

This keeps acquisition, suggestion, and import as separate explicit stages.

### 2) Suggestion output reuses the shipped config contract

The helper writes the same versioned `presentation-defaults.json` shape already consumed by the
generator/import path.

That means:

- no new config dialect,
- no generator contract change,
- and no special import-only shortcut path.

### 3) `palette` remains evidence, not hidden policy

The helper uses `palette` only when it is explicitly present in acquisition provenance.

When `palette` is absent, the helper fails rather than guessing. This preserves the correctness
posture of the closed generated-defaults lane.

### 4) Public teaching surface is aligned

Docs now teach the helper as:

- an explicit convenience step,
- useful when the source began as `icons acquire iconify-collection ...`,
- and something the user should review before passing into `icons import ...`.

## Gates that now define the shipped surface

- `cargo nextest run -p fretboard`
- `cargo nextest run -p fret usage_docs_prefer_explicit_app_submodules_for_optional_ecosystems todo_golden_path_keeps_icon_pack_setup_on_app_install_surface`

## Follow-on policy

Do not reopen this lane for:

- silent import defaults based on provenance,
- generic SVG heuristics,
- or mixed-pack classification logic.

If future work is needed, open a narrower follow-on such as:

1. richer suggestion reports that emit a human-review summary alongside the config file,
2. explicit SVG-analysis scaffolding that proposes per-icon overrides,
3. or a broader multi-source suggestion lane once there is real evidence beyond Iconify
   acquisition provenance.
