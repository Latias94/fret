# Workstream Catalog Integrity Gate v1

Status: Closed
Last updated: 2026-04-10

Related:

- `TODO.md`
- `MILESTONES.md`
- `EVIDENCE_AND_GATES.md`
- `BASELINE_AUDIT_2026-04-10.md`
- `M1_CONTRACT_FREEZE_2026-04-10.md`
- `M2_PROOF_SURFACE_2026-04-10.md`
- `CLOSEOUT_AUDIT_2026-04-10.md`
- `docs/workstreams/diag-skill-evidence-owner-drift-v1/CLOSEOUT_AUDIT_2026-04-10.md`
- `docs/workstreams/README.md`
- `docs/workstreams/standalone/README.md`
- `tools/check_workstream_catalog.py`
- `tools/gates_fast.py`
- `tools/pre_release.py`

Status note (2026-04-10): this lane is now closed on one narrow governance automation:

- `tools/check_workstream_catalog.py` validates the human-maintained workstream indexes with
  section-aware parsing,
- the checker now guards dedicated-directory coverage, standalone-file coverage, and the tracked
  count lines that drifted recently,
- and the gate is wired into both `tools/gates_fast.py` and `tools/pre_release.py`.

Read the landed proof in `M2_PROOF_SURFACE_2026-04-10.md` and the final verdict in
`CLOSEOUT_AUDIT_2026-04-10.md`.

This lane is a narrow governance follow-on to the closed
`diag-skill-evidence-owner-drift-v1` lane. That lane exposed that `docs/workstreams/README.md`
could drift silently even after the concrete skill-owner fix was correct. This follow-on does not
reopen broader docs automation, README generation, or workstream-state schema work.

It owns one narrower question:

> how should Fret keep the manually curated workstream indexes reviewable and human-authored while
> still catching missing entries and stale counts before they land?

## Why this lane exists

The repo already treats `docs/workstreams/README.md` and
`docs/workstreams/standalone/README.md` as first-open navigation surfaces, but their maintenance was
still fully manual.

That created three distinct drift modes:

- a dedicated workstream directory could exist without a corresponding `Directory Index` entry;
- the `Standalone markdown files:` and `Standalone Bucket` counts could silently lag behind the
  filesystem;
- and because both README files contain non-index links and cross-references, naive whole-file grep
  checks would confuse ordinary evidence links with catalog entries.

During the previous governance lane, the repo already showed this pressure in practice:

- several dedicated directories were missing from the top-level catalog,
- and the standalone bucket count in the top-level README lagged behind the real standalone file
  count.

That meant the repo needed a real structural gate, not another manual spot check.

## Assumptions-first baseline

### 1) This is a narrow follow-on, not a broad docs-generation lane

- Area: lane ownership
- Assumption: the right scope is a small integrity checker for the existing curated indexes, not a
  full generator that rewrites README files automatically.
- Evidence:
  - `docs/workstreams/diag-skill-evidence-owner-drift-v1/CLOSEOUT_AUDIT_2026-04-10.md`
  - `docs/workstreams/README.md`
  - `docs/workstreams/standalone/README.md`
- Confidence: Confident
- Consequence if wrong: this lane would under-scope a broader catalog-generation problem.

### 2) The README files remain the human-facing source of navigation

- Area: product posture
- Assumption: maintainers still want curated prose/status around the indexes, so the gate should
  validate them rather than replace them.
- Evidence:
  - `docs/workstreams/README.md`
  - `docs/workstreams/standalone/README.md`
  - `.agents/skills/fret-workstream-lifecycle/SKILL.md`
- Confidence: Confident
- Consequence if wrong: we would preserve a manual surface that should have been generated instead.

### 3) Section-aware parsing is required

- Area: checker design
- Assumption: the checker must parse only `## Directory Index` and `## File Index`, because both
  README files contain many non-index markdown links that should not count as catalog rows.
- Evidence:
  - `docs/workstreams/README.md`
  - `docs/workstreams/standalone/README.md`
- Confidence: Confident
- Consequence if wrong: the gate would either miss real drift or flag ordinary prose links as
  false positives.

### 4) The gate belongs in common maintainer entrypoints

- Area: adoption
- Assumption: a standalone checker is not enough; it should also run via `tools/gates_fast.py` and
  `tools/pre_release.py`.
- Evidence:
  - `tools/gates_fast.py`
  - `tools/pre_release.py`
  - `tools/check_layering.py`
- Confidence: Likely
- Consequence if wrong: the checker could exist but remain too easy to forget in ordinary work.

### 5) Fixing current count/coverage drift is part of landing the gate

- Area: immediate proof
- Assumption: the lane should not stop at writing the checker; it must also repair the currently
  detected count/coverage drift so the new guard lands green.
- Evidence:
  - `docs/workstreams/README.md`
  - `tools/check_workstream_catalog.py`
- Confidence: Confident
- Consequence if wrong: the repo would land a new gate in a permanently failing state.

## In scope

- A section-aware Python checker for the workstream catalog README files.
- Validation of dedicated-directory coverage and counts in `docs/workstreams/README.md`.
- Validation of standalone-file coverage and counts in `docs/workstreams/standalone/README.md`.
- Wiring the checker into common maintainer gate entrypoints.
- Fixing the current catalog drift so the new gate lands green.

## Out of scope

- Generating workstream README files automatically.
- Rewriting the workstream-state JSON schema.
- Reworking roadmap prioritization or broader docs navigation.
- Validating every prose sentence in the README files beyond the catalog sections and tracked count
  lines.

## Target shipped state

When this lane is done, the following must be true:

1. the top-level workstream README cannot silently miss a dedicated directory entry;
2. the tracked dedicated/standalone counts cannot silently drift from the filesystem;
3. the standalone README cannot silently miss a single-file workstream entry;
4. the checker ignores non-index markdown links and only validates the intended catalog sections;
5. common maintainer gate entrypoints exercise the checker automatically.
