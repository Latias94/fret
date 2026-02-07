---
name: fret-boundary-checks
description: "Run and interpret boundary/portability guardrails for fearless refactors: layering checks, forbidden dependency spot checks for kernel crates, and module-size drift reports. Use before/after refactors that move code across crates or may accidentally pull backend deps into contracts."
---

# Fret boundary checks (guardrails)

This skill is for **quick guardrails** that keep bottom-up refactors safe and portable.

## Core guardrails (always-run)

- Layering (workspace crate boundaries):
  - `pwsh -NoProfile -File tools/check_layering.ps1`
- Module size drift (keep god files visible):
  - `pwsh -NoProfile -File tools/report_largest_files.ps1 -Top 30 -MinLines 800`

## Crate-focused checks (when auditing one crate)

- Quick audit snapshot:
  - `pwsh -NoProfile -File tools/audit_crate.ps1 -Crate <crate>`

## Interpreting failures (common cases)

- `check_layering.ps1` failures mean a **workspace->workspace** dependency edge violates ADR policy.
  - Fix by moving code to the correct layer, or by adding an explicit allowlist entry only when
    the crate is intentionally “wiring heavy”.
- Huge-file drift means you should split by responsibility before expanding behavior surface.

## Notes

- These scripts are intentionally “best-effort” and fast; they do not replace deeper audits.
- If a guardrail needs to become normative (CI gate), document it in the workstream and add a
  stable “Fast vs Full” command set.

