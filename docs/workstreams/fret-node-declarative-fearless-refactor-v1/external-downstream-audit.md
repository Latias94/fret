# `fret-node` External Downstream Audit for UI Transport Alias Deprecation

## Purpose

This note defines the minimum audit required before we add deprecated markers to the temporary root
`fret_node::ui::*` queue/helper aliases.

The repository is already in the desired in-tree posture:

- retained-facing samples use `fret_node::ui::advanced::*`,
- crate-internal retained/test callers use explicit module paths,
- root `fret_node::ui::*` queue/helper exports exist only as temporary compatibility aliases.

The remaining question is external compatibility, not in-tree cleanup.

## Decision to unlock

This audit exists to answer one governance question:

> Is it safe to mark the root `fret_node::ui::*` queue/helper aliases deprecated in the next
> migration-oriented release?

Default answer until the audit is done: **no**.

## Alias surface under review

Audit every downstream use of these root exports:

- `fret_node::ui::NodeGraphEditQueue`
- `fret_node::ui::NodeGraphViewQueue`
- `fret_node::ui::NodeGraphViewRequest`
- `fret_node::ui::NodeGraphViewportHelper`
- `fret_node::ui::NodeGraphSetViewportOptions`
- `fret_node::ui::NodeGraphFitViewOptions`

Equivalent `crate::ui::*` uses matter only for this mono-repo and are already cleared.

## Current in-tree status (2026-03-06)

Completed:

- explicit advanced namespace exists: `fret_node::ui::advanced::*`
- retained-backed examples import queue surfaces from `advanced::*`
- crate-internal retained/test code no longer depends on root queue/helper aliases

Still intentionally open:

- no deprecated markers on root aliases yet
- no release-note migration note yet
- no populated external downstream inventory yet (`external-downstream-inventory.md` is now the skeleton)

## Audit scope

Include any external repo, app template, internal product repo, cookbook mirror, or shared snippet set
that imports `fret-node` and could plausibly consume retained transport seams.

Minimum buckets:

1. first-party downstream apps outside this mono-repo
2. internal/private repos that pin published `fret-node`
3. public examples, blog snippets, or copied starter templates
4. diagnostic harnesses or experiments not versioned in this mono-repo

## Audit procedure

### Step 1 - enumerate candidate repos

For each downstream repo, record:

- repo name / owner
- branch or tag audited
- whether it builds against workspace path or published crates
- expected maintenance owner
- whether the repo is actively maintained

### Step 2 - grep for root alias usage

Recommended search patterns:

```powershell
rg -n "fret_node::ui::(NodeGraphEditQueue|NodeGraphViewQueue|NodeGraphViewRequest|NodeGraphViewportHelper|NodeGraphSetViewportOptions|NodeGraphFitViewOptions)" <repo>
rg -n "use\s+fret_node::ui::\{[^\n]*(NodeGraphEditQueue|NodeGraphViewQueue|NodeGraphViewRequest|NodeGraphViewportHelper|NodeGraphSetViewportOptions|NodeGraphFitViewOptions)" <repo>
```

If a repo aliases imports or re-exports them, also search for:

```powershell
rg -n "NodeGraph(EditQueue|ViewQueue|ViewRequest|ViewportHelper|SetViewportOptions|FitViewOptions)" <repo>
```

### Step 3 - classify each hit

Classify every hit into one of these buckets:

- **Must migrate now**
  - app-facing code
  - documentation snippets
  - starter templates
  - reusable helpers that teach integration posture
- **Can migrate with the next touch**
  - local experiments
  - one-off prototypes
  - private compatibility harnesses with no active consumers
- **Needs explicit exception**
  - a downstream that cannot migrate yet for release or staffing reasons

### Step 4 - apply the migration recipe

Use these replacements:

- `fret_node::ui::NodeGraphEditQueue`
  -> `fret_node::ui::advanced::NodeGraphEditQueue`
- `fret_node::ui::NodeGraphViewQueue`
  -> `fret_node::ui::advanced::NodeGraphViewQueue`
- `fret_node::ui::NodeGraphViewRequest`
  -> `fret_node::ui::advanced::NodeGraphViewRequest`
- `fret_node::ui::NodeGraphSetViewportOptions`
  -> `fret_node::ui::advanced::NodeGraphSetViewportOptions`
- `fret_node::ui::NodeGraphFitViewOptions`
  -> `fret_node::ui::advanced::NodeGraphFitViewOptions`
- `fret_node::ui::NodeGraphViewportHelper`
  -> `fret_node::ui::advanced::NodeGraphViewportHelper`

Behavior guidance:

- If downstream code is app-facing, prefer `NodeGraphController` first.
- If downstream code truly owns raw transport seams, import from `ui::advanced::*` explicitly.
- If downstream only uses viewport helper ergonomics, prefer
  `NodeGraphViewportHelper::from_controller(...)` over transport-first construction when possible.

### Step 5 - record the disposition

For every downstream repo, capture:

- exact hit count before migration
- exact files migrated
- remaining exceptions, if any
- whether a release note or direct owner follow-up is required

### Step 6 - decide deprecation readiness

The root aliases are ready for deprecated markers only if all of the following are true:

- in-tree usage remains at zero
- all first-party downstream apps are migrated or have an explicit written exception
- all public-facing snippets/templates are migrated
- any private exceptions have a named owner and sunset target
- a migration note is ready for release communication

## Recommended deliverables

Leave behind these artifacts when the audit is executed:

- a filled downstream inventory file (`external-downstream-inventory.md`)
- links to migration PRs / commits when applicable
- a short exception list with owners and dates
- the exact release note wording to use when deprecated markers are added

## Execution ledger

Use `external-downstream-inventory.md` as the canonical execution ledger for this audit.

## Suggested inventory table

| Repo | Ref | Uses found | Status | Owner | Notes |
| --- | --- | --- | --- | --- | --- |
| example-repo | `main@<sha>` | 3 | Migrated | `@owner` | switched to `ui::advanced::*` |

## Recommended release-note wording

> `fret_node::ui::*` queue/helper aliases are now deprecated. Use
> `fret_node::ui::advanced::*` for explicit retained transport seams, or prefer
> `NodeGraphController` / `NodeGraphViewportHelper::from_controller(...)` for app-facing code.

## Not in scope

This audit does not decide:

- whether the root aliases are removed entirely,
- whether `ui::advanced::*` should later be renamed again,
- whether the retained compatibility path itself is deleted.

Those are follow-up governance decisions.

## Evidence anchors

- `ecosystem/fret-node/src/ui/advanced.rs`
- `ecosystem/fret-node/src/ui/mod.rs`
- `apps/fret-examples/src/node_graph_domain_demo.rs`
- `apps/fret-ui-gallery/src/ui/snippets/ai/workflow_node_graph_demo.rs`
- `docs/workstreams/fret-node-declarative-fearless-refactor-v1/external-downstream-inventory.md`
- `docs/workstreams/fret-node-declarative-fearless-refactor-v1/README.md`
- `docs/workstreams/fret-node-declarative-fearless-refactor-v1/todo.md`
- `docs/workstreams/fret-node-declarative-fearless-refactor-v1/milestones.md`
