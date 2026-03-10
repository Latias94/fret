---
title: Diagnostics Debt Retirement Tracker
status: draft
date: 2026-03-09
scope: diagnostics, debt, retirement, enforcement, gates
---

# Diagnostics Debt Retirement Tracker

Status: Draft

Purpose:

- keep the remaining diagnostics debt visible,
- record which compatibility bridges are intentional,
- and define what evidence is required before removing them.

## How to use this tracker

For each debt item, record:

- what is still duplicated or compatibility-only,
- what currently protects the seam,
- what must be true before it can be retired.

Rule:

- do not remove compatibility or old paths just because a cleaner name exists,
- retire them only after the replacement seam is proven and consumer/document drift is closed.

## Debt list

| Area | Current debt / compatibility boundary | Current protection | Retirement trigger | Retirement action |
| --- | --- | --- | --- | --- |
| Aggregate dashboard presentation | CLI/GUI/MCP now share projection helpers, but future consumers could still reintroduce ad hoc dashboard parsing. | Shared tests in `fret-diag`, `fret-devtools`, and `fret-devtools-mcp`; shared wording/projection helpers already exist. | Another consumer lands without a parallel projection path for normalized status/reason/failing-summary data. | Keep future consumers on shared projection helpers; reject new parallel parsers. |
| Policy-skip evidence adoption | `skipped_policy` / `capability.missing` / `capabilities_check_path` are now consumer-visible, but maintainer interpretation still depends on following the new checklist/navigation flow. | `MAINTAINER_CHECKLIST.md`, `START_HERE.md`, DevTools drill-down coverage, MCP dashboard coverage. | Maintainer-facing docs and all active consumers consistently treat policy skip as non-executed evidence-bearing state. | Treat this contract slice as settled and avoid introducing alternate wording. |
| Legacy manifest entry aliases | Old top-level `suites` / `scripts` manifest shapes remain accepted for compatibility. | Explicit compatibility notes in campaign docs and roadmap/TODO tracking. | Manifest authoring and aggregate evidence contracts are stable enough that removing legacy shapes will not break active producers. | Remove legacy read compatibility in a dedicated compatibility window with explicit release/docs note. |
| Legacy artifact field aliases | Fields such as `bundle_json` and `script_result_json` still survive as compatibility aliases in some readers/payloads. | `RESIDUAL_NAMING_AUDIT.md`, `BUNDLE_ARTIFACT_ALIAS_AUDIT.md`, additive writer-first/read-compat policy. | Canonical names are the only actively written names and all important readers/tests have moved to canonical-first handling. | Retire aliases one family at a time with a bounded audit and tests. |
| Older diagnostics workstreams | Older v1/v1-architecture/GUI/MCP/simplification notes still exist and can attract duplicate planning edits. | Back-links to v2 umbrella, `DOCUMENT_MIGRATION_INTENT.md`, `START_HERE.md`. | Old notes stop receiving roadmap/priority updates and remain only as linked background or unique technical depth. | Freeze them as background and keep only thin forward links when touched. |
| Major seam migrations without explicit gate mapping | Some seam extractions are documented, but not every major seam has a named “this is the gate that protects it” note yet. | `IMPLEMENTATION_ROADMAP.md`, existing targeted tests, workstream TODO tracking. | Each major seam category has at least one named test/script/lint expectation recorded in this tracker or a linked note. | Add the missing gate mapping before declaring the seam category complete. |

## Immediate next candidates

The next debt items that deserve explicit closure criteria are:

1. any future major seam category that lands without a named protecting gate in
   `SEAM_GATE_MATRIX.md`.

## Practical maintainer rule

Before removing a compatibility path or duplicate helper:

1. identify the replacement seam,
2. identify the protecting gate/test/script,
3. update the related v2 note,
4. only then remove the old path.

## Short version

If one rule is enough:

- every retirement needs a replacement seam, a protecting gate, and one explicit note saying why it
  is now safe.
