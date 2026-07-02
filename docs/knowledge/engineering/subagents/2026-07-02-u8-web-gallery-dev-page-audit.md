---
type: Subagent Finding
title: U8 web gallery-dev page availability audit
tags: fret,u8,wasm,ui-gallery,subagent
timestamp: 2026-07-02
subagent_id: 019f2076-d489-7150-90fd-7b9a9c56dbac
related_plan: docs/plans/2026-06-30-001-refactor-ui-framework-architecture-plan.md
git_branch: feat/ui-framework-convergence
---

# Finding

The failed web diagnostics run waiting for `ui-gallery-nav-code-editor-torture` was not caused by a
bad `test_id`. The selector is correct, but the running web bundle did not include the
`gallery-dev` feature, so the `code_editor_torture` page and nav item were absent. The nav search
chain also made the script more fragile than necessary for U8 evidence.

# Evidence

- `apps/fret-ui-gallery/src/ui/nav.rs` derives nav item test IDs by replacing `_` with `-`, so
  `code_editor_torture` becomes `ui-gallery-nav-code-editor-torture`.
- `apps/fret-ui-gallery/src/spec.rs` gates `PAGE_CODE_EDITOR_TORTURE` and its page group behind
  `gallery-dev`.
- `apps/fret-ui-gallery-web/Cargo.toml` defaults to `cjk-lite-fonts` and `devtools-ws`; it only
  forwards `gallery-dev` when launched with `--features gallery-dev`.
- `tools/diag-scripts/ui-gallery/code-editor/ui-gallery-code-editor-torture-perf-steady-web.json`
  already declares `required_launch_features: ["gallery-dev"]` and directly waits for
  `ui-gallery-code-editor-torture-root`.

# Recommendation

For U8 web evidence, launch `apps/fret-ui-gallery-web` with `trunk serve --features gallery-dev`
and open `/gallery?page=code_editor_torture` directly. Prefer direct page-root waits over the nav
search path for perf/runtime evidence.

# Disposition

Accepted. The web evidence run used the existing direct `code_editor_torture` web steady script and
a `gallery-dev` trunk build. The budget gate passed after the web runner also published text
resource cache snapshots when renderer perf diagnostics are enabled.
