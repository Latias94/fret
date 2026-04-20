# UiCx Compat Alias Release Retirement v1 — Milestones

Status: Closed

## M0: Baseline freeze and ownership lock

Exit criteria:

- The repo explicitly records that `public-authoring-state-lanes-and-identity-fearless-refactor-v1`
  is closed and should not be reopened for alias retirement.
- The live `UiCx*` inventory is classified:
  - explicit public compatibility exports,
  - hidden deprecated carriers,
  - historical evidence only.
- The lane explicitly states that canonical teaching stays on
  `AppComponentCx<'a>` / `AppRenderCx<'a>` / `AppRenderContext<'a>`.

Primary evidence:

- `docs/workstreams/uicx-compat-alias-release-retirement-v1/DESIGN.md`
- `docs/workstreams/uicx-compat-alias-release-retirement-v1/M0_BASELINE_AUDIT_2026-04-19.md`
- `docs/workstreams/public-authoring-state-lanes-and-identity-fearless-refactor-v1/CLOSEOUT_AUDIT_2026-04-19.md`

Status:

- Completed on 2026-04-19.

## M1: Release policy freeze

Exit criteria:

- The lane names the release window for the explicit compatibility aliases.
- The lane decides whether hidden carrier aliases move on the same window or a separate one.
- The lane writes explicit delete criteria and migration wording expectations.

Primary evidence:

- `docs/workstreams/uicx-compat-alias-release-retirement-v1/DESIGN.md`
- `docs/workstreams/uicx-compat-alias-release-retirement-v1/TODO.md`
- `docs/workstreams/uicx-compat-alias-release-retirement-v1/EVIDENCE_AND_GATES.md`
- `release-plz.toml`

Status:

- Completed on 2026-04-20.

## M2: Release-facing verdict and closeout

Exit criteria:

- The repo has one explicit published verdict:
  - retain the aliases for a named window, or
  - remove the aliases with release-facing evidence.
- The relevant docs, source-policy tests, and release preflight are updated to match the verdict.
- The lane closes with a clear next-action rule instead of an implied future cleanup.

Primary gates:

- `python3 tools/gate_no_raw_app_context_in_default_teaching_snippets.py`
- `cargo nextest run -p fret --test render_authoring_capability_surface --test raw_state_advanced_surface_docs --test app_render_actions_surface --test app_render_data_surface --test crate_usage_grouped_query_surface --no-fail-fast`
- `python3 tools/check_workstream_catalog.py`
- `python3 .agents/skills/fret_skills.py validate --strict --check-anchors --check-symbols`

Status:

- Completed on 2026-04-20.
