# Diagnostics First-Open Workflow

This is the **default first-open diagnostics entry** for Fret.

Use this page when you need one quick answer to:

- where to start when a Fret UI feels wrong,
- which command sequence is the default loop,
- and which deeper note to open next without hopping across diagnostics workstreams first.

Scope note:

- This page is the canonical **first-open workflow**.
- For the interactive inspect/pick branch, continue with `docs/debugging-ui-with-inspector-and-scripts.md`.
- For bundle/script schema details and bounded triage commands, continue with
  `docs/ui-diagnostics-and-scripted-tests.md`.
- For DevTools GUI dogfood over the same artifact contracts, continue with
  `docs/workstreams/diag-fearless-refactor-v2/DEVTOOLS_GUI_DOGFOOD_WORKFLOW.md`.
- For diagnostics workstream planning/navigation, use
  `docs/workstreams/diag-fearless-refactor-v2/START_HERE.md` only after this page.

## Default loop

When in doubt, use this order:

1. Inspect and pick one stable selector.
2. Patch or choose one JSON script.
3. Run the script into one explicit diagnostics artifacts root.
4. Read bounded evidence first.
5. Compare either one bundle pair or one aggregate root.

The important rule is simple:

- CLI-compatible artifact production comes first,
- DevTools GUI and MCP come after the same artifacts root exists,
- and portable artifacts stay the handoff unit.

## 1) Inspect and pick

Start with one target app or demo, then arm inspect:

- repo maintainer path:
  `cargo run -p fretboard-dev -- diag inspect on`
- public CLI shape:
  `cargo run -p fretboard -- diag inspect on`

Pick selectors that survive refactors:

- prefer `test_id`,
- prefer semantics-oriented selectors over geometry assumptions,
- treat copied selector JSON as the thing you will reuse in scripts.

If you need the inspect shortcuts and picker UX details, stop here and continue with:

- `docs/debugging-ui-with-inspector-and-scripts.md`

## 2) Patch or choose one script

Keep committed repros in JSON so CLI, GUI, MCP, and review all share the same script surface.

Typical edit path:

- `cargo run -p fretboard-dev -- diag pick-apply <script> --ptr <json-pointer>`

Use an existing promoted script or patch a nearby one before creating a new workflow note.

If you need script schema details, sidecars, or authoring guidance, continue with:

- `docs/ui-diagnostics-and-scripted-tests.md`

## 3) Run into one explicit artifacts root

The default launched shape is:

```bash
cargo run -p fretboard-dev -- diag run <script> --dir <session-dir> --session-auto --launch -- <target cmd>
```

Use one explicit out dir per task so you do not mix concurrent runs.

This is the first real closure point:

- the run leaves normal diagnostics artifacts behind,
- the result is not tied to live GUI state,
- and later consumers should reuse that same root instead of inventing a second store.

## 4) Read bounded evidence first

Start with bounded reads instead of opening `bundle.json` directly:

- `cargo run -p fretboard-dev -- diag latest`
- `cargo run -p fretboard-dev -- diag resolve latest --dir <base-or-session-dir> --json`
- `cargo run -p fretboard-dev -- diag meta <bundle_dir|bundle.json|bundle.schema2.json> --json`
- `cargo run -p fretboard-dev -- diag query test-id <source> <pattern> --top 50`
- `cargo run -p fretboard-dev -- diag query identity-warnings <bundle_dir|bundle.json|bundle.schema2.json> --browser --json`
- `cargo run -p fretboard-dev -- diag slice <bundle_dir|bundle.json|bundle.schema2.json> --test-id <test_id>`

Use these commands to answer:

- what bundle was produced,
- which `test_id` exists,
- whether immediate/runtime identity warnings were captured,
- and what one bounded slice says about the failing area.

For identity-warning triage without running a demo first, use the committed sample bundle:

```bash
cargo run -p fretboard-dev -- diag query identity-warnings crates/fret-diag/tests/fixtures/identity_warnings/bundle.schema2.json --browser --json
cargo run -p fretboard-dev -- diag query identity-warnings crates/fret-diag/tests/fixtures/identity_warnings/bundle.schema2.json --html-out target/identity.html --html-check-out target/check.identity_browser_html.json
```

The HTML artifact is a self-contained offline sidecar. The check sidecar is
`check.identity_browser_html` and is intended for quick structural smoke validation before a
browser-driven review path exists.

## 5) Compare at the shared artifacts layer

There are two default compare shapes.

### Direct bundle/session compare

Use this when you already know which two bundle dirs you want:

```bash
cargo run -p fretboard-dev -- diag compare <a> <b> --json
```

The success condition is a machine-readable compare verdict.

- If the UI states differ, `ok=false` is still a successful compare run.
- Do not treat compare as a GUI-only visual mode.

### Aggregate root compare/read path

Use this when the interesting handoff unit is a run root rather than one direct bundle pair:

```bash
cargo run -p fretboard-dev -- diag summarize <root> --dir <root> --json
cargo run -p fretboard-dev -- diag dashboard <root> --json
```

This is the shared aggregate contract used by CLI, DevTools GUI, and MCP:

- `regression.summary.json`
- `regression.index.json`

## Consumer branches after the default loop

Once the artifacts root exists, branch by consumer surface only as needed.

### Inspect branch

Open:

- `docs/debugging-ui-with-inspector-and-scripts.md`

Use it for:

- picker shortcuts,
- overlay/focus/outside-press debugging,
- inspect-first selector capture.

### Bundle/script branch

Open:

- `docs/ui-diagnostics-and-scripted-tests.md`

Use it for:

- script schema details,
- sidecars and artifact resolution,
- bounded triage commands,
- env knobs and launch/session hygiene.

### DevTools GUI branch

Open:

- `docs/workstreams/diag-fearless-refactor-v2/DEVTOOLS_GUI_DOGFOOD_WORKFLOW.md`

Use it for:

- GUI inspect/pick over the same selector contract,
- GUI summarize/regression reading over the same aggregate artifacts,
- GUI evidence packing over the same artifact root.

### Maintainer/workstream branch

Open:

- `docs/workstreams/diag-fearless-refactor-v2/START_HERE.md`

Use it for:

- diagnostics workstream planning,
- artifact vocabulary/background notes,
- migration/history navigation.

Do not start there if your real need is just "how do I debug this UI right now?"

## First-open rule

If you only remember one rule, remember this:

- start with `docs/diagnostics-first-open.md`,
- create or reuse one artifacts root with CLI-compatible commands,
- then branch into inspect, bundles/scripts, or GUI notes only after the root exists.
