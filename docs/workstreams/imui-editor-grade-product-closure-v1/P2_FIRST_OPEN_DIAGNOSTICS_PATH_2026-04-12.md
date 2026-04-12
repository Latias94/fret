# P2 First-Open Diagnostics Path - 2026-04-12

Status: focused P2 workflow decision / first-open developer loop freeze

Related:

- `DESIGN.md`
- `TODO.md`
- `MILESTONES.md`
- `EVIDENCE_AND_GATES.md`
- `docs/debugging-ui-with-inspector-and-scripts.md`
- `docs/ui-diagnostics-and-scripted-tests.md`
- `docs/workstreams/diag-fearless-refactor-v2/DEVTOOLS_GUI_DOGFOOD_WORKFLOW.md`
- `docs/workstreams/diag-devtools-gui-v1/diag-devtools-gui-v1.md`
- `docs/workstreams/diag-devtools-gui-v1/diag-devtools-gui-v1-ai-mcp.md`

## Purpose

P2 needs one narrow answer:

> when a maintainer or app author needs to debug an editor-grade Fret UI, what is the first-open
> diagnostics path from inspect to compare, and how do DevTools GUI and MCP fit without becoming a
> second diagnostics architecture?

This note freezes the default path before P2 grows into a larger implementation-heavy follow-on.

## Audited evidence

- `docs/debugging-ui-with-inspector-and-scripts.md`
- `docs/ui-diagnostics-and-scripted-tests.md`
- `docs/workstreams/diag-fearless-refactor-v2/START_HERE.md`
- `docs/workstreams/diag-fearless-refactor-v2/DEVTOOLS_GUI_DOGFOOD_WORKFLOW.md`
- `docs/workstreams/diag-devtools-gui-v1/diag-devtools-gui-v1.md`
- `docs/workstreams/diag-devtools-gui-v1/diag-devtools-gui-v1-ai-mcp.md`
- `crates/fret-diag/src/cli/mod.rs`
- `apps/fret-devtools/src/native.rs`
- `apps/fret-devtools-mcp/src/native.rs`

## Assumptions-first resume set

1. Confident: the canonical evidence producer is still the CLI-compatible diagnostics contract, not
   DevTools GUI state.
   Evidence:
   - `docs/ui-diagnostics-and-scripted-tests.md` names bundles, sidecars, scripts, compare, and
     pack/share as the portable unit,
   - `crates/fret-diag/src/cli/mod.rs` keeps `run`, `latest`, and `compare` visible as first-open
     CLI examples.
   Consequence if wrong:
   - P2 would drift into a GUI-first product story that leaves non-GUI users and CI behind.
2. Confident: DevTools GUI is already a thin consumer over shared artifacts rather than a second
   campaign model.
   Evidence:
   - `docs/workstreams/diag-fearless-refactor-v2/DEVTOOLS_GUI_DOGFOOD_WORKFLOW.md`,
   - `apps/fret-devtools/src/native.rs` reads `regression.summary.json` /
     `regression.index.json` and exposes `Summarize`.
   Consequence if wrong:
   - P2 would need a consumer-boundary correction before any first-open path could be trusted.
3. Confident: MCP is already a thin automation adapter over the same diagnostics operations.
   Evidence:
   - `docs/workstreams/diag-devtools-gui-v1/diag-devtools-gui-v1-ai-mcp.md`,
   - `apps/fret-devtools-mcp/src/native.rs` exposes `fret_diag_pick`,
     `fret_diag_run_script_file`, `fret_diag_regression_summarize`,
     `fret_diag_regression_dashboard`, and `fret_diag_compare`.
   Consequence if wrong:
   - P2 would silently teach an AI-only diagnostics loop with different semantics.
4. Likely: for P2, "compare" must mean an artifacts-layer verdict, not a GUI-only visual mode.
   Evidence:
   - `docs/ui-diagnostics-and-scripted-tests.md` already defines `diag compare` for bundle pairs,
   - regression summarize/dashboard artifacts already provide aggregate compare-oriented views.
   Consequence if wrong:
   - the first-open path would stop being portable across CLI, GUI, MCP, and offline review.

## Current gap

The repo already has all the ingredients for a strong diagnostics story:

- live inspect and pick,
- selector-driven scripts,
- bounded bundle and sidecar artifacts,
- DevTools GUI regression readers,
- and MCP automation tools.

What is still missing is one first-open path that teaches these as one workflow instead of as
separate notes and entrypoints.

## Frozen first-open path

From this point forward, the default P2 developer loop is:

1. Inspect and pick a stable selector
   - default entry:
     `cargo run -p fretboard-dev -- diag inspect on`
   - prefer semantics-oriented selectors, especially `test_id`
   - if DevTools GUI is already open, it may drive inspect/pick, but the output must still be the
     same selector JSON rather than a GUI-only identifier
2. Patch or choose a CLI-compatible script
   - use `cargo run -p fretboard-dev -- diag pick-apply <script> --ptr <json-pointer>` when
     updating JSON scripts directly
   - DevTools Script Studio may help author or fork scripts, but committed scripts remain JSON and
     CLI-compatible
3. Run into one explicit diagnostics artifacts root
   - canonical launched shape:
     `cargo run -p fretboard-dev -- diag run <script> --dir <session-dir> --session-auto --launch -- <target cmd>`
   - suites are acceptable when the scenario is already curated, but the path still writes the same
     shared bundle/sidecar artifacts
4. Read bounded evidence first, not raw bundle payloads
   - first-open commands:
     - `cargo run -p fretboard-dev -- diag latest`
     - `cargo run -p fretboard-dev -- diag meta <bundle_dir|bundle.json|bundle.schema2.json> --json`
     - `cargo run -p fretboard-dev -- diag query test-id <source> <pattern> --top 50`
     - `cargo run -p fretboard-dev -- diag slice <bundle_dir|bundle.json|bundle.schema2.json> --test-id <test_id>`
     - `cargo run -p fretboard-dev -- diag ai-packet <bundle_dir|bundle.json|bundle.schema2.json> --packet-out <dir>`
5. Compare at the shared artifacts layer
   - direct bundle-vs-bundle compare:
     `cargo run -p fretboard-dev -- diag compare <a> <b> --json`
   - aggregate run-set compare:
     `diag summarize` writes `regression.summary.json` + `regression.index.json`, then DevTools GUI
     `Regression` and MCP `fret_diag_regression_dashboard` read those same artifacts as thin
     consumers

## Consumer branches after the default path

Once the shared artifacts root exists, consumer-specific branches are allowed:

1. DevTools GUI branch
   - start with `cargo run -p fret-devtools`
   - treat the GUI as a reader/launcher over the same script library and artifacts root
   - use `Summarize` and `Regression` as readers of `regression.summary.json` /
     `regression.index.json`, not as a GUI-only campaign store
2. MCP automation branch
   - start with `cargo run -p fret-devtools-mcp`
   - use `fret_diag_pick`, `fret_diag_run_script_file`, `fret_diag_regression_summarize`,
     `fret_diag_regression_dashboard`, and `fret_diag_compare`
   - keep MCP as an adapter over the same selector/script/artifact contract rather than a separate
     automation model

## Decision

From this point forward:

1. The P2 first-open path is CLI-first for evidence production.
2. DevTools GUI and MCP are thin consumers of the same selector, script, bundle, and regression
   artifact contracts.
3. "Compare" belongs to the shared artifacts layer:
   - direct `diag compare` for bundle/session pairs,
   - summarize/dashboard artifacts for aggregate run sets.
4. Portable artifacts remain the handoff unit; live GUI or MCP session state is not the canonical
   evidence package.
5. This slice freezes the first-open path only.
   - final owner split across `apps/fret-devtools`, `crates/fret-diag`, `ecosystem/fret-bootstrap`,
     and `apps/fret-devtools-mcp` remains a separate open P2 task
   - the bounded devtools smoke package also remains a separate open P2 task

## Immediate execution consequence

For this lane:

- treat this note as the first-open P2 answer before digging into older diagnostics workstreams,
- keep new GUI or MCP features aligned with the shared artifacts root and CLI-compatible script
  model,
- and reject any P2 change that only works by inventing GUI-only or MCP-only diagnostics state.
