# `fret-node` External Downstream Inventory for UI Transport Alias Migration

## Purpose

This file is the execution ledger for `external-downstream-audit.md`.

Use it to track which downstream repos were audited, what root alias usage was found, whether the
repo was migrated, and whether any exception blocks deprecation of the temporary root
`fret_node::ui::*` queue/helper aliases.

## Status summary

- Audit status: Not started
- Last updated: 2026-03-06
- In-tree root alias usage: Cleared
- External downstream audit complete: No
- Root alias deprecated-ready: No

## How to use this file

1. Add one row per downstream repo.
2. Link migration PRs / commits in the notes column.
3. Keep exceptions explicit and owned.
4. Do not mark deprecated-ready until every required row is either migrated or has a written exception.

## Candidate repo inventory

| Repo | Ref | Distribution mode | Uses found | Classification | Status | Owner | Exception sunset | Notes |
| --- | --- | --- | --- | --- | --- | --- | --- | --- |
| `<repo-name>` | `<branch-or-tag>` | `workspace` / `published` | `0` | `Must migrate now` / `Can migrate with next touch` / `Needs explicit exception` | `Not started` | `<owner>` | `<date-or-n/a>` | `<search result, PR link, or reason>` |

## First-party downstream apps

| Repo | Ref | Uses found | Status | Owner | Notes |
| --- | --- | --- | --- | --- | --- |
| `<app-repo>` | `<ref>` | `<n>` | `Not started` | `<owner>` | `<notes>` |

## Internal / private published-crate consumers

| Repo | Ref | Uses found | Status | Owner | Notes |
| --- | --- | --- | --- | --- | --- |
| `<private-repo>` | `<ref>` | `<n>` | `Not started` | `<owner>` | `<notes>` |

## Public snippets / templates / mirrors

| Repo | Ref | Uses found | Status | Owner | Notes |
| --- | --- | --- | --- | --- | --- |
| `<public-snippet-source>` | `<ref>` | `<n>` | `Not started` | `<owner>` | `<notes>` |

## Exceptions

Only list repos here if they block immediate migration.

| Repo | Reason | Owner | Sunset target | Mitigation | Notes |
| --- | --- | --- | --- | --- | --- |
| `<repo>` | `<reason>` | `<owner>` | `<date>` | `<temporary plan>` | `<notes>` |

## Migration log

| Date | Repo | Change | Evidence |
| --- | --- | --- | --- |
| `2026-03-06` | `<repo>` | `Initial inventory row created` | `<PR / commit / note>` |

## Release-note readiness checklist

- [ ] All first-party downstream apps audited
- [ ] All public snippets/templates audited
- [ ] Remaining exceptions have named owners
- [ ] Remaining exceptions have sunset targets
- [ ] Migration note wording reviewed
- [ ] Root alias deprecated-ready explicitly approved

## Decision log

- 2026-03-06: In-tree root alias usage is already cleared; external downstream audit remains the
  only blocker before deprecated markers can be considered.

## Related documents

- `docs/workstreams/fret-node-declarative-fearless-refactor-v1/external-downstream-audit.md`
- `docs/workstreams/fret-node-declarative-fearless-refactor-v1/README.md`
- `docs/workstreams/fret-node-declarative-fearless-refactor-v1/todo.md`
- `docs/workstreams/fret-node-declarative-fearless-refactor-v1/milestones.md`
