# Action-First Authoring + View Runtime (Fearless Refactor v1) — DataTable Authoring Audit

Status: draft, post-v1 audit
Last updated: 2026-03-09

Related:

- TODO: `docs/workstreams/action-first-authoring-fearless-refactor-v1/TODO.md`
- Milestones: `docs/workstreams/action-first-authoring-fearless-refactor-v1/MILESTONES.md`
- V2 golden path: `docs/workstreams/action-first-authoring-fearless-refactor-v1/V2_GOLDEN_PATH.md`
- DataTable golden path: `docs/workstreams/action-first-authoring-fearless-refactor-v1/DATA_TABLE_GOLDEN_PATH.md`
- Post-v1 proposal: `docs/workstreams/action-first-authoring-fearless-refactor-v1/POST_V1_AUTHORING_V2_PROPOSAL.md`

---

## Scope

This note audits `fret-ui-shadcn::DataTable` as a **separate post-v1 authoring surface**.

The narrow question is:

> Does `DataTable` still represent ordinary builder-first / early-landing noise, or is it now a
> higher-level business-table integration surface that needs productization guidance instead of more
> generic helper expansion?

Current conclusion:

- `DataTable` is **not** the same problem as primitive `Table` authoring.
- Primitive `Table::build(...)` cleanup can be considered largely closed for the current pass.
- The remaining `DataTable` pressure is about **state ownership, selection wiring, toolbar/output
  composition, and default recipe clarity**, not about one more `build(...)` helper.

---

## Audit matrix

| Surface | Current shape | What is already good | What is still noisy | Recommended stance |
| --- | --- | --- | --- | --- |
| Primitive `Table` stack | `Table::build(...)`, `TableHeader::build(...)`, `TableBody::build(...)`, `TableRow::build(...)`, `TableCell::build(...)` | Builder-first child composition now exists across gallery/reference surfaces | A few advanced retained/host boundaries still land explicitly | Treat as mostly aligned; do not reopen this track just because `DataTable` is denser |
| `DataTable` core widget | `DataTable::new()` + explicit `Model<TableState>` + data/columns/get_row_key + optional `output_model(...)` + retained/header-cell variants | Keeps headless table state, virtualization, column actions, and retained-host seams explicit | Author must compose multiple moving parts before even reaching toolbar/selection/pagination polish | Keep as an advanced integration surface for now; do not collapse it into generic default helpers yet |
| Gallery `basic_demo` | Explicit `TableState`, `TableViewOutput`, selection command plumbing, pagination controls | Honest about selection/pagination ownership; demonstrates real host integration | Repetitive command wiring and output bootstrapping are heavy for a “reference default” sample | Reposition as advanced/reference, not as the primitive table baseline |
| Gallery `guide_demo` | Adds facets, responsive toolbar state, column actions, row actions, and retained rendering variants | Good evidence of real editor-grade/business-table needs | Very dense for teaching the default path; mixes multiple concerns in one narrative | Split teaching intent: keep as advanced capability guide, add a smaller curated default recipe later if needed |

---

## Findings

### 1) `DataTable` is not a primitive-table builder problem

Representative evidence:

- `ecosystem/fret-ui-shadcn/src/table.rs`
- `ecosystem/fret-ui-shadcn/src/data_table.rs`
- `apps/fret-ui-gallery/src/ui/snippets/table/demo.rs`
- `apps/fret-ui-gallery/src/ui/snippets/data_table/basic_demo.rs`

What changed in the current refactor:

- primitive table composition already gained the late-landing builder path,
- gallery/reference table snippets now largely avoid decorate-only `into_element(cx)` cliffs,
- `DataTable` still looks dense because it owns a broader integration story than primitive table
  layout.

Practical implication:

- `DataTable` should not be used as evidence that primitive `Table` still needs more generic
  builder helpers.

### 2) The current `DataTable` surface is already honest about the right hard parts

Representative evidence:

- `ecosystem/fret-ui-shadcn/src/data_table.rs`

What the API already gets right:

- explicit `Model<TableState>` keeps headless table behavior inspectable,
- explicit `output_model(...)` makes pagination/filtered-row metadata shareable,
- retained/header-cell variants keep advanced composition possible without hiding cache/virtualized
  boundaries,
- row-key and column definitions remain explicit rather than being inferred through a fragile macro
  layer.

Assessment:

- this is the correct shape for an advanced business-table integration root,
- the current pressure is not that the API is “wrong”, but that the repo does not yet present a
  smaller **curated default recipe** on top of it.

### 3) The remaining noise is concentrated in host wiring, not in leaf builders

Representative evidence:

- `apps/fret-ui-gallery/src/ui/snippets/data_table/basic_demo.rs`
- `apps/fret-ui-gallery/src/ui/snippets/data_table/guide_demo.rs`

Main noisy areas:

- selection command generation/parsing (`CommandId` routes per row),
- output model allocation/bootstrapping,
- toolbar/facet helper state setup,
- pagination/status-row coordination,
- column-action menu wiring when the page wants a complete “app table” experience.

Assessment:

- this is closer to “business-table recipe assembly” than to simple authoring syntax noise,
- adding more generic builder helpers here would likely blur mechanism vs recipe ownership.

### 4) Some complexity should remain explicit

The repo should keep these parts visible:

- headless `TableState` ownership,
- explicit column definitions and row-key strategy,
- `TableViewOutput` / pagination / filtered-row coordination,
- retained/virtualized boundaries,
- command/action integration for table-specific menus and row actions.

Reason:

- these are exactly the parts that become important in editor-grade or data-heavy apps,
- hiding them too early would make the surface look simpler while actually weakening debuggability
  and composability.

### 5) The next step is productization, not another helper pass

Recommended post-v1 stance:

1. Keep `DataTable::new()` as the current low-level integration root.
2. Do **not** fold `DataTable` into the primitive `Table::build(...)` cleanup story.
3. Do **not** add macros or broad new helpers yet.
4. Add a smaller curated recipe or narrative later only if the repo wants a default “business
   table” teaching path distinct from primitive `Table`.

Suggested future deliverable:

- one narrow “DataTable golden path” example or note that deliberately scopes itself to:
  - one `TableState`,
  - one `output_model`,
  - snapshot-driven toolbar/footer wiring,
  - action/command hooks kept visible,
  - no extra facets/responsive variants unless the example is explicitly marked advanced.

---

## Decision

For the current workstream:

- treat primitive `Table` builder-first cleanup as a mostly closed adoption track,
- treat `DataTable` as a **separate post-v1 productization/reference-surface audit**,
- keep the next action on the documentation/product side before adding API surface.

That means the immediate follow-up is:

- document the distinction in TODO/milestones/golden-path notes,
- then keep the curated `DataTable` default recipe as a docs-first guidance note before widening
  any API surface.
