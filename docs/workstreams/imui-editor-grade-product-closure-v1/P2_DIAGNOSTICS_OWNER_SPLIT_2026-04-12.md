# P2 Diagnostics Owner Split - 2026-04-12

Status: focused P2 owner decision / diagnostics consumer-boundary freeze

Related:

- `DESIGN.md`
- `TODO.md`
- `MILESTONES.md`
- `EVIDENCE_AND_GATES.md`
- `P2_FIRST_OPEN_DIAGNOSTICS_PATH_2026-04-12.md`
- `docs/workstreams/diag-fearless-refactor-v2/README.md`
- `docs/workstreams/diag-fearless-refactor-v2/IMPLEMENTATION_ROADMAP.md`
- `docs/workstreams/diag-devtools-gui-v1/diag-devtools-gui-v1.md`

## Purpose

P2 still needs one narrow owner answer:

> what must stay in `ecosystem/fret-bootstrap`, `crates/fret-diag`, `apps/fret-devtools`, and
> `apps/fret-devtools-mcp`, so the diagnostics/devtools loop feels integrated without reopening a
> second runtime, a second run model, or a GUI-only/MCP-only artifact contract?

This note freezes that owner split before the bounded devtools smoke package work starts.

## Audited evidence

- `ecosystem/fret-bootstrap/src/ui_diagnostics.rs`
- `ecosystem/fret-bootstrap/src/ui_diagnostics/service.rs`
- `crates/fret-diag/src/lib.rs`
- `crates/fret-diag/src/cli/mod.rs`
- `apps/fret-devtools/src/native.rs`
- `apps/fret-devtools-mcp/src/native.rs`
- `docs/ui-diagnostics-and-scripted-tests.md`
- `docs/workstreams/diag-devtools-gui-v1/diag-devtools-gui-v1.md`
- `docs/workstreams/diag-fearless-refactor-v2/MAINTAINER_AUTOMATION_FLOW.md`
- `docs/workstreams/diag-fearless-refactor-v2/IMPLEMENTATION_ROADMAP.md`

## Assumptions-first resume set

1. Confident: `ecosystem/fret-bootstrap` is the in-app diagnostics runtime/export seam, not the
   orchestration center.
   Evidence:
   - `UiDiagnosticsService`,
   - script execution, inspect/pick interception, bundle dump, sidecar writing,
   - runtime bridge registration such as debug extensions and WS integration.
   Consequence if wrong:
   - P2 would blur app-runtime capture with tooling-side orchestration and make ownership regress.
2. Confident: `crates/fret-diag` is the shared tooling engine and consumer-side artifact logic.
   Evidence:
   - `diag_run`, `diag_suite`, `diag_repeat`, `diag_perf`, `diag_matrix`, `diag_compare`,
     `diag_summarize`, `diag_dashboard`,
   - `transport` and `devtools::DevtoolsOps`,
   - crate docs already describe it as tooling-focused, not a runtime app dependency.
   Consequence if wrong:
   - GUI, MCP, and CLI would each need to fork orchestration and artifact logic.
3. Confident: `apps/fret-devtools` and `apps/fret-devtools-mcp` are consumer/adaptation surfaces,
   not owners of bundle/schema/orchestration semantics.
   Evidence:
   - DevTools GUI reads shared regression artifacts and triggers summarize/pack flows,
   - MCP tools bridge into shared summarize/dashboard/compare operations and resource projections.
   Consequence if wrong:
   - P2 would recreate the same semantics in multiple presentation surfaces.
4. Likely: the WebSocket hub/bootstrap convenience packaging may remain app-level, but the message
   contract and tooling semantics must stay below the apps.
   Evidence:
   - both `apps/fret-devtools` and `apps/fret-devtools-mcp` host convenience WS server wiring,
   - protocol types and tooling behavior already live outside those apps.
   Consequence if wrong:
   - P2 would overfit transport convenience code into the owner split and miss the actual contract
     boundaries.

## Frozen owner split

From this point forward, use this owner map by default.

### `ecosystem/fret-bootstrap`

`ecosystem/fret-bootstrap` owns the in-app diagnostics runtime/export seam.

This includes:

- `UiAppDriver`-hosted diagnostics enablement,
- inspect/pick/script event interception inside the target app process,
- snapshot capture, bundle/schema2 writing, and bounded sidecars,
- runtime-local script execution and result writing,
- runtime debug extension registration and app-side transport bridge hooks.

This does **not** include:

- campaign/suite/repeat/matrix/perf orchestration policy,
- aggregate summarize/dashboard projections,
- DevTools GUI state or MCP resource shaping,
- a second consumer-specific artifact schema.

### `crates/fret-diag`

`crates/fret-diag` owns orchestration, artifact tooling, compare/summarize/dashboard projections,
and transport client helpers.

This includes:

- shared command/engine behavior for `run`, `suite`, `repeat`, `repro`, `perf`, `matrix`,
  `compare`, `summarize`, `dashboard`, `pack`, and triage-style artifact reads,
- shared artifact resolution, evidence indexing, regression summary/index projections,
- compare/report semantics over bundles or session roots,
- transport client helpers and DevTools-facing operation helpers reused by CLI, GUI, and MCP.

This does **not** include:

- in-app event interception or snapshot capture inside the target app,
- GUI-specific panel state, docking chrome, or script-studio UX,
- MCP-specific resource subscription UX,
- policy that only makes sense for one consumer surface.

### `apps/fret-devtools`

`apps/fret-devtools` owns editor-grade diagnostics UX over the shared contracts.

This includes:

- session selection, script studio, semantics browser, and regression workspace UX,
- user-facing triggers such as inspect/pick arm, summarize, pack selected evidence, and viewer
  handoff,
- app-level convenience hosting for the loopback WS hub used by the GUI workflow.

This does **not** include:

- a GUI-only campaign store,
- a GUI-only summary/index schema,
- a second compare implementation,
- or any attempt to replace `crates/fret-diag` as the orchestration engine.

### `apps/fret-devtools-mcp`

`apps/fret-devtools-mcp` owns the headless MCP adapter and resource/tool projection over the same
contracts.

This includes:

- mapping shared diagnostics operations into rmcp tools,
- exposing shared bundle/regression artifacts as MCP resources,
- session selection and subscription/update glue for automation clients,
- app-level convenience hosting for the loopback WS hub used by automation workflows.

This does **not** include:

- a new automation-only run model,
- MCP-only bundle or regression schemas,
- or a second implementation of summarize/dashboard/compare semantics.

## Guardrails

From this point forward:

1. Do not move orchestration policy into `ecosystem/fret-bootstrap`.
2. Do not let `apps/fret-devtools` or `apps/fret-devtools-mcp` invent a second run model or
   artifact schema.
3. When GUI or MCP needs a new capability that CLI would also need, land it in `crates/fret-diag`
   or the underlying protocol/runtime seam first, then project it upward.
4. When the target app needs new inspect/pick/script/runtime evidence, land it in
   `ecosystem/fret-bootstrap` first, then reuse it from tooling/GUI/MCP.

## Decision

From this point forward:

1. `ecosystem/fret-bootstrap` owns runtime capture/export.
2. `crates/fret-diag` owns shared tooling/orchestration and artifact projections.
3. `apps/fret-devtools` owns GUI UX over those shared contracts.
4. `apps/fret-devtools-mcp` owns headless automation/resource projection over those same
   contracts.
5. P2 should treat GUI and MCP as consumer lanes, not alternate centers of diagnostics truth.

## Immediate execution consequence

For this lane:

- evaluate future P2 work against the owner map above before widening any contract,
- keep the remaining P2 bounded smoke package focused on the shared first-open loop rather than on
  consumer-specific implementation detail,
- and start a narrower devtools follow-on once work stops being primarily about owner/boundary
  closure.
