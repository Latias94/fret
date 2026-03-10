---
title: Diagnostics Retirement Criteria
status: draft
date: 2026-03-09
scope: diagnostics, retirement, compatibility, docs, enforcement
---

# Diagnostics Retirement Criteria

Status: Draft

Purpose:

- define when older diagnostics notes or compatibility shims are allowed to be retired,
- prevent cleanup from outrunning evidence,
- and give maintainers one explicit checklist before they remove an old path.

## 1) Core rule

Do not retire a diagnostics compatibility path, alias, or old planning note unless all of the
following are true:

1. a replacement seam is already the preferred path,
2. a protecting gate/test/script is already named,
3. active consumers/docs have moved to the replacement,
4. the retirement is recorded in the v2 workstream notes.

If any one of these is missing, the path is not ready to retire.

## 2) Retirement checklist

Before removing an old path, confirm:

- **Replacement seam**
  - the new path is already live and preferred,
  - the ownership boundary is clearer after removal, not blurrier.
- **Protecting gate**
  - there is at least one named test/script/lint/command that protects the replacement behavior.
- **Consumer convergence**
  - CLI/GUI/MCP/docs do not still depend on the old spelling/path/plan note as an active surface.
- **Documentation update**
  - the relevant v2 note is updated:
    - `DEBT_RETIREMENT_TRACKER.md`,
    - `SEAM_GATE_MATRIX.md`,
    - and any contract note affected by the removal.

## 3) Compatibility shim retirement

Examples:

- legacy manifest read compatibility,
- legacy artifact field aliases,
- transitional dual-write behavior.

A compatibility shim is ready to retire only when:

1. canonical write behavior has been stable for at least one full implementation slice,
2. all important in-tree readers/tests have canonical-first coverage,
3. the old path is no longer required by active maintainer workflows,
4. the removal can be described as a bounded compatibility-window change.

Required evidence:

- one named protecting test or command path,
- one note explaining why active consumers no longer need the old path.

## 4) Older note retirement or freeze

An older diagnostics workstream note does **not** need to be deleted to be considered retired.

Use these states instead:

- **Active**
  - still receives current roadmap/contract updates.
- **Frozen background**
  - still useful for unique technical depth or historical rationale,
  - no longer updated as an active roadmap.
- **Retirable**
  - no longer adds unique depth beyond the v2 umbrella and linked contract notes.

An older note should move from active to frozen background when:

1. the v2 umbrella already owns current roadmap and maintainer guidance,
2. the old note already has a forward link to v2,
3. keeping it active would create duplicate planning surfaces.

An older note is only a deletion candidate when:

1. it no longer adds unique technical detail,
2. its key content already lives in a v2 note or ADR,
3. removing it would not break useful repo navigation.

## 5) Major seam retirement

A seam migration can be called "done" only when:

1. the old inline/duplicated path is no longer the normal edit point,
2. the replacement helper/seam is the named maintenance surface,
3. at least one protecting gate is recorded in `SEAM_GATE_MATRIX.md`,
4. any compatibility bridges left behind are explicitly tracked in `DEBT_RETIREMENT_TRACKER.md`.

If the old path still exists for compatibility, the seam can be "landed" without being "retired".

## 6) Allowed outcomes

A cleanup decision can end in one of these outcomes:

- **Retire now**
  - all criteria are met; remove the old path.
- **Freeze**
  - keep the old path/document, but stop treating it as active.
- **Keep intentionally**
  - compatibility is still required; track it explicitly.
- **Defer**
  - replacement seam or protecting gate is still insufficient.

## 7) Short decision table

| Case | Minimum requirement before retirement |
| --- | --- |
| Legacy field alias | canonical-first readers + named test/gate + explicit compatibility-window note |
| Legacy manifest shape | stable manifest contract + named test/checklist + maintainer workflow no longer depends on legacy shape |
| Old workstream note | v2 umbrella owns roadmap + forward link exists + old note adds no unique depth |
| Duplicated helper path | replacement seam is default edit point + named protecting gate exists |

## 8) Short version

If one rule is enough:

- do not delete old diagnostics paths until the replacement seam, protecting gate, and consumer/doc
  convergence are all explicit.
