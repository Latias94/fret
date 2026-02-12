---
name: fret-boundary-checks
description: "Run and interpret boundary/portability guardrails for fearless refactors: layering checks, forbidden dependency spot checks for kernel crates, and module-size drift reports. Use before/after refactors that move code across crates or may accidentally pull backend deps into contracts."
---

# Fret boundary checks (guardrails)

This skill is for **quick guardrails** that keep bottom-up refactors safe and portable.

## When to use

- Before/after refactors that move code across crates.
- When you suspect an accidental dependency edge (e.g. backend deps leaking into contract crates).
- When a file/module is drifting into a “god file” and you want an early warning.

## Quick start

Run the always-on guardrails:

- Layering (workspace crate boundaries):
  - `python3 tools/check_layering.py`
- Module size drift (keep god files visible):
  - `python3 tools/report_largest_files.py --top 30 --min-lines 800`

## Workflow

### Core guardrails (always-run)

- Layering (workspace crate boundaries):
  - `python3 tools/check_layering.py`
- Module size drift (keep god files visible):
  - `python3 tools/report_largest_files.py --top 30 --min-lines 800`

### Crate-focused checks (when auditing one crate)

- Quick audit snapshot:
  - `python3 tools/audit_crate.py --crate <crate>`

### Interpreting failures (common cases)

- `check_layering.py` failures mean a **workspace->workspace** dependency edge violates ADR policy.
  - Fix by moving code to the correct layer, or by adding an explicit allowlist entry only when
    the crate is intentionally “wiring heavy”.
- Huge-file drift means you should split by responsibility before expanding behavior surface.

## Notes

- These scripts are intentionally “best-effort” and fast; they do not replace deeper audits.
- If a guardrail needs to become normative (CI gate), document it in the workstream and add a
  stable “Fast vs Full” command set.

## Evidence anchors

- Layering checks: `tools/check_layering.py`, `docs/dependency-policy.md`
- Crate audit snapshot: `tools/audit_crate.py`
- Module-size drift: `tools/report_largest_files.py`

## Common pitfalls

- Treating allowlists as a first-choice fix (prefer moving code to the correct layer).
- Ignoring a “small” layering violation during a refactor (it compounds quickly).
- Measuring file-size drift after the refactor lands (run guardrails early).

## Related skills

- `fret-crate-audits` (deeper crate-by-crate review)
- `fret-fixture-driven-harnesses` (when large test matrices become unreviewable)
