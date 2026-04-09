# M1 Contract Freeze — 2026-04-10

Status: closed decision record

Related:

- `docs/workstreams/workstream-catalog-integrity-gate-v1/DESIGN.md`
- `docs/workstreams/workstream-catalog-integrity-gate-v1/BASELINE_AUDIT_2026-04-10.md`
- `docs/workstreams/README.md`
- `docs/workstreams/standalone/README.md`
- `tools/check_workstream_catalog.py`
- `tools/gates_fast.py`
- `tools/pre_release.py`

## Frozen decision

This lane freezes the following gate contract:

1. `tools/check_workstream_catalog.py` owns structural validation of:
   - `docs/workstreams/README.md`
     - `Dedicated directories`
     - `Standalone markdown files`
     - `Standalone Bucket` README count
     - `## Directory Index` dedicated-directory entries
   - `docs/workstreams/standalone/README.md`
     - `## File Index` standalone markdown entries
2. the checker must parse only the intended catalog sections and tracked count lines;
3. the checker must fail on missing entries, extra entries, duplicate entries, and stale tracked
   counts;
4. the checker must run from both `tools/gates_fast.py` and `tools/pre_release.py`.

## Consequences

- The curated README indexes stay manual but become structurally guarded.
- Count drift can no longer linger silently once common gates are run.
- Full directory-order normalization stays available as a separate docs follow-on if the repo wants
  to pay that migration cost later.

## Explicit non-goals

- Do not auto-generate README prose or entry descriptions.
- Do not validate every markdown link in either README file.
- Do not turn full historical alphabetical normalization into part of this lane's hard gate.
- Do not widen this lane into roadmap or docs-site generation work.
