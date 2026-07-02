---
type: Work Progress
title: ADR numbering and pre-release smoke restored
tags: fret,adr,pre-release,execution-surface
timestamp: 2026-07-02
related_plan: docs/plans/2026-06-30-001-refactor-ui-framework-architecture-plan.md
git_branch: feat/ui-framework-convergence
---

# Summary

The duplicate ADR ID `0324` blocker is resolved. The older window input hit-testing ADR keeps
`0324`; the later a11y state-description ADR moved to `0332`.

The follow-on also fixed execution-surface policy allowlist drift exposed after the ADR gate was
unblocked: devtools polling surfaces are allowed to use raw thread sleeps, and the mutation toast
cookbook Tokio example is classified with the existing async/Tokio lab examples.

# Evidence

- ADR moved to `docs/adr/0332-a11y-state-description-semantics-v1.md`.
- Alignment updated in `docs/adr/IMPLEMENTATION_ALIGNMENT.md`.
- Material3 workstream references updated under
  `docs/workstreams/material3-visual-behavior-layout-parity-v2/`.
- Execution policy updated in `tools/check_execution_surface.py`.

# Verification

- `python3 tools/check_adr_numbers.py`
- `python3 tools/check_execution_surface.py`
- `python3 -m py_compile tools/check_execution_surface.py`
- `python3 tools/pre_release.py --skip-fmt --skip-clippy --skip-nextest --skip-icons --skip-release-closure --skip-portable-time --skip-diff-check`
- `python3 tools/check_workstream_catalog.py`
- `python /Users/frankorz/.codex/skills/engineering-wiki-memory/scripts/wiki_memory.py validate --root docs/knowledge/engineering`
- `git diff --check`

# Next Action

No ADR numbering blocker remains. Run the full aggregate pre-release chain when a release-sized
verification is needed; otherwise continue from the UI convergence closeout retained/deferred table.
