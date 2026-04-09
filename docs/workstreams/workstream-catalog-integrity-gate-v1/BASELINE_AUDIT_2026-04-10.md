# Baseline Audit — 2026-04-10

Status: closed audit record

Related:

- `docs/workstreams/workstream-catalog-integrity-gate-v1/DESIGN.md`
- `docs/workstreams/diag-skill-evidence-owner-drift-v1/CLOSEOUT_AUDIT_2026-04-10.md`
- `docs/workstreams/README.md`
- `docs/workstreams/standalone/README.md`

## Audit question

What concrete catalog drift exists today, and what is the smallest durable guard that can catch it
without misreading ordinary README links as index rows?

## Baseline findings

### 1) Top-level dedicated-directory coverage had already drifted recently

The previous governance lane uncovered that several dedicated workstream directories existed on disk
without corresponding `Directory Index` entries in `docs/workstreams/README.md`.

That established the need for a structural guard beyond manual review.

### 2) Historical ordering also drifted, but that is a different-sized cleanup

After the missing entries were patched, at least one inserted entry still landed out of alphabetic
order inside `## Directory Index`, and the broader file still contains older out-of-order blocks.

That is useful evidence, but normalizing the entire historical directory index would be a larger
docs-only follow-on than this lane needs.

### 3) Standalone counts had their own drift mode

`docs/workstreams/README.md` already tracked two standalone-oriented count surfaces:

- `Standalone markdown files: ...`
- and the `Standalone Bucket` entry for `docs/workstreams/standalone/README.md`.

At least one of those counts lagged behind the actual standalone file count.

### 4) Naive grep is not enough

Both README files contain many markdown links outside their true catalog sections:

- historical references,
- closeout links,
- and prose examples.

A naive whole-file `.md` grep would overcount and produce false positives, especially in
`docs/workstreams/standalone/README.md`.

## Audit conclusion

The repo needs one small section-aware checker that:

1. validates `docs/workstreams/README.md` dedicated-directory coverage and tracked counts,
2. validates `docs/workstreams/standalone/README.md` file-index coverage,
3. ignores non-index prose links,
4. and runs through common maintainer gate entrypoints.
