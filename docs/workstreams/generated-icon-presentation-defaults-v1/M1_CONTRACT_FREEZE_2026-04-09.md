# M1 Contract Freeze — 2026-04-09

Status: accepted v1 decision

Related:

- `docs/workstreams/generated-icon-presentation-defaults-v1/DESIGN.md`
- `docs/workstreams/generated-icon-presentation-defaults-v1/BASELINE_AUDIT_2026-04-09.md`
- `docs/workstreams/generated-icon-presentation-defaults-v1/TODO.md`
- `docs/workstreams/generated-icon-presentation-defaults-v1/MILESTONES.md`
- `docs/workstreams/generated-icon-presentation-defaults-v1/EVIDENCE_AND_GATES.md`
- `docs/workstreams/icon-system-extension-v1/CLOSEOUT_AUDIT_2026-04-09.md`
- `docs/workstreams/iconify-acquisition-prestep-v1/CLOSEOUT_AUDIT_2026-04-09.md`
- `crates/fret-icons-generator/src/contracts.rs`
- `crates/fret-icons-generator/src/svg_dir.rs`
- `crates/fret-icons-generator/src/iconify.rs`
- `crates/fret-icons-generator/src/templates.rs`
- `ecosystem/fret-icons/src/lib.rs`
- `ecosystem/fret-ui-kit/src/declarative/icon.rs`

## Purpose

Freeze the smallest correct v1 contract for generated-pack presentation defaults before code
starts drifting. This note decides:

- what the source of truth for default `IconPresentation` should be,
- where that policy should live,
- and what the first proof should target.

## Frozen decisions

### 1) Explicit versioned generator input is the authoritative source of presentation defaults

Decision:

- the v1 source of truth should be an explicit, versioned generator input contract for presentation
  defaults;
- do not make collection-level `palette` metadata or ad-hoc SVG analysis the silent normative
  default in v1;
- and do not infer presentation inside runtime rendering code.

Why:

- the runtime contract is already explicit and closed;
- SVG directory imports have no collection-level metadata at all;
- and correctness matters more here than “magic convenience” because a wrong default changes how
  icons render across an entire generated pack.

Operational consequence:

- future helpers may derive or suggest presentation config from upstream metadata or source
  analysis,
- but the shipped generator contract should stay explicit and reviewable.

### 2) The policy should live in the generator library + thin CLI, then compile into generated registration code

Decision:

- policy input belongs in `fret-icons-generator` contracts plus the thin `fretboard` CLI wrapper;
- generated pack code should emit explicit registry registration with the chosen presentation;
- and runtime crates should continue to consume `IconPresentation` rather than rediscovering policy.

Why:

- this keeps mechanism vs policy ownership aligned with the existing icon-system closeout;
- it avoids pushing import heuristics into app/runtime layers;
- and it keeps third-party generator users on the same stable contract as `fretboard`.

### 3) The v1 config should support both a pack-level default and per-icon overrides

Decision:

- the config should be able to name:
  - an optional pack-level default render mode,
  - plus explicit per-icon overrides keyed by generated icon name;
- this config should remain versioned, like semantic aliases.

Why:

- some imported packs may be consistently monochrome or consistently authored-color,
- while mixed packs or SVG-directory imports still need per-icon precision;
- and “default + override” avoids a future rewrite when pack authors need both cases.

### 4) The first proof should target explicit config, not heuristic inference

Decision:

- the first proof should show:
  - one authored-color imported icon reaching `OriginalColors`,
  - one monochrome imported icon staying on `Mask`,
  - and generated code registering both explicitly;
- do not spend the first proof slice on auto-detect heuristics.

Why:

- explicit config is the safest contract to freeze first,
- it works for both Iconify snapshots and SVG-directory sources,
- and it leaves future helper tooling as a narrow follow-on instead of blocking the core contract.

## Rejected alternatives

### Treat Iconify collection `palette` as the default source of truth

Rejected because:

- it is collection-level metadata, not a generated-pack policy contract,
- it does not cover local SVG directory imports,
- and it is too coarse for mixed packs.

### Use SVG-content heuristics as the normative v1 default

Rejected because:

- silent analysis is difficult to reason about and review,
- it can be wrong for icons that intentionally use `currentColor` plus extra authored details,
- and it would be harder to keep deterministic and well-explained than explicit config.

### Push presentation choice into runtime widgets

Rejected because:

- the runtime split is already closed,
- generated-pack policy belongs at import/generation time,
- and moving the decision later would blur layering.

## Immediate consequences

From this point forward:

1. generator/import work should target an explicit versioned presentation-defaults config;
2. the first implementation slice should carry presentation metadata through generator internals and
   emit explicit registry registration code;
3. `palette` metadata and SVG analysis may become helper inputs later, but not the v1 normative
   defaulting rule;
4. this lane should not reopen acquisition or runtime ownership.
