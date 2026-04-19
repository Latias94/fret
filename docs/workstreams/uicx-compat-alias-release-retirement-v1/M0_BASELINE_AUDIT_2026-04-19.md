# M0 Baseline Audit - 2026-04-19

Status: active baseline note
Last updated: 2026-04-19

Related:

- `DESIGN.md`
- `TODO.md`
- `MILESTONES.md`
- `EVIDENCE_AND_GATES.md`
- `WORKSTREAM.json`
- `docs/workstreams/public-authoring-state-lanes-and-identity-fearless-refactor-v1/CLOSEOUT_AUDIT_2026-04-19.md`
- `docs/workstreams/public-authoring-state-lanes-and-identity-fearless-refactor-v1/UICX_COMPAT_ALIAS_DEPRECATION_AUDIT_2026-04-19.md`
- `docs/crate-usage-guide.md`
- `docs/authoring-golden-path-v2.md`
- `docs/examples/todo-app-golden-path.md`
- `docs/first-hour.md`
- `docs/shadcn-declarative-progress.md`
- `ecosystem/fret/src/lib.rs`
- `ecosystem/fret/src/view.rs`
- `release-plz.toml`

## Why this note exists

The broad public-authoring lane closed on 2026-04-19 and should stay closed.
This note records the assumptions-first baseline for the narrower release-facing follow-on so the
repo does not misread leftover deprecated aliases as evidence that the broad authoring refactor is
still unresolved.

## Assumptions-first read

### 1) The previous broad lane is closed and should not be reopened.

- Evidence:
  - `docs/workstreams/public-authoring-state-lanes-and-identity-fearless-refactor-v1/CLOSEOUT_AUDIT_2026-04-19.md`
  - `docs/roadmap.md`
- Confidence:
  - Confident
- Consequence if wrong:
  - work would drift back into a broad historical lane and blur release policy with architecture.

### 2) First-party canonical teaching no longer depends on `UiCx<'a>`.

- Evidence:
  - `docs/crate-usage-guide.md`
  - `docs/authoring-golden-path-v2.md`
  - `docs/examples/todo-app-golden-path.md`
  - `docs/first-hour.md`
  - `docs/shadcn-declarative-progress.md`
  - `tools/gate_no_raw_app_context_in_default_teaching_snippets.py`
- Confidence:
  - Confident
- Consequence if wrong:
  - alias retention/removal would still be coupled to unresolved first-party onboarding.

### 3) The `UiCx*` family is still part of a publish-facing surface.

- Evidence:
  - `ecosystem/fret/src/lib.rs`
  - `ecosystem/fret/src/view.rs`
  - `ecosystem/fret/Cargo.toml`
- Confidence:
  - Confident
- Consequence if wrong:
  - the repo could treat the remaining aliases as purely internal debt and delete them cheaply.

### 4) Alias removal is now a release-policy decision, not a framework-architecture decision.

- Evidence:
  - `release-plz.toml`
  - `docs/workstreams/public-authoring-state-lanes-and-identity-fearless-refactor-v1/CLOSEOUT_AUDIT_2026-04-19.md`
  - `docs/adr/0319-public-authoring-state-lanes-and-identity-contract-v1.md`
- Confidence:
  - Confident
- Consequence if wrong:
  - the repo could over-index on internal cleanliness and under-specify downstream migration impact.

### 5) The hidden carrier aliases may still need the same release-facing scrutiny as the explicit ones.

- Evidence:
  - `ecosystem/fret/src/view.rs`
  - `ecosystem/fret/tests/uicx_actions_surface.rs`
  - `ecosystem/fret/tests/uicx_data_surface.rs`
- Confidence:
  - Likely
- Consequence if wrong:
  - the repo might either delete semver-visible names too early or keep purely internal debris for
    longer than necessary.

## Findings

### 1) Category A: explicit compatibility exports are still visible on the `fret` facade

The current explicit compatibility surface is:

- root alias in `ecosystem/fret/src/lib.rs`:
  - `pub type UiCx<'a> = AppComponentCx<'a>;`
- explicit app-lane compatibility exports in `ecosystem/fret/src/lib.rs`:
  - `UiCxActionsExt`
  - `UiCxDataExt`
- explicit advanced-view compatibility export in `ecosystem/fret/src/lib.rs`:
  - `UiCxDataExt`

These are no longer the canonical authoring answer, but they are still importable names in a
published crate.

### 2) Category B: deprecated hidden carrier aliases still exist in `view.rs`

The current hidden deprecated carrier family is:

- `UiCxDataExt`
- `UiCxData`
- `UiCxActionsExt`
- `UiCxActions`
- `UiCxActionLocal`
- `UiCxLocalsWith`

Because these aliases are still emitted from Rust source, the repo should decide on purpose whether
they retire with the visible public aliases or on a separate proof-backed path.

### 3) Category C: many remaining `UiCx` references are historical evidence, not live API

The old broad lane contains valid audit history for:

- why `UiCx` stopped being the default teaching surface,
- how the grouped helper names moved to `AppRenderActionsExt` / `AppRenderDataExt`,
- and why deprecation was more correct than blind deletion on 2026-04-19.

Those documents are still useful as evidence, but they do not mean the broad lane remains active.

### 4) The default teaching and gate posture is already frozen on canonical names

The repo now teaches:

- `AppComponentCx<'a>` for app-hosted snippets/components,
- `AppRenderCx<'a>` for concrete render helpers,
- `AppRenderContext<'a>` for named helper signatures.

The targeted source-policy tests and the default-snippet gate already protect that posture.
This follow-on should not renegotiate that teaching surface.

### 5) The unresolved question is release window and delete criteria

The remaining work is now narrow and explicit:

- decide the release window for deprecated compatibility aliases,
- decide whether hidden carriers move on the same window,
- and record what downstream or release evidence is sufficient before removal.

## Execution consequence

Use `uicx-compat-alias-release-retirement-v1` as the only active lane for future `UiCx` alias
retention/removal decisions.

From this note forward:

1. keep the broad public-authoring lane closed,
2. keep canonical teaching on `AppComponentCx<'a>` / `AppRenderCx<'a>` / `AppRenderContext<'a>`,
3. treat any `UiCx*` deletion as release-facing work,
4. and freeze the delete criteria before touching code again.
