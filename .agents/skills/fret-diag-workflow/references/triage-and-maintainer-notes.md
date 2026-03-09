# Triage and maintainer notes

Use this note when you already have a run directory or bundle and need to explain the failure, share bounded evidence, or keep diagnostics entrypoints maintainable.

## 1) Triage without grepping `bundle.json`

Prefer bounded queries:

- `fretboard diag meta <bundle_dir|bundle.json|bundle.schema2.json> --json`
- `fretboard diag windows <bundle_dir|bundle.json|bundle.schema2.json>`
- `fretboard diag dock-routing <bundle_dir|bundle.json|bundle.schema2.json>`
- `fretboard diag screenshots <out_dir|bundle_dir|bundle.json|bundle.schema2.json>`
- `fretboard diag resolve latest --dir <base_or_session_dir> [--within-session <id|latest>]`
- `fretboard diag query test-id <bundle_dir|bundle.json|bundle.schema2.json> <pattern> --top 50`
- `fretboard diag slice <bundle_dir|bundle.json|bundle.schema2.json> --test-id <test_id>`

Safe repo search templates live behind `tools/rg-safe.ps1`.

## 2) Fast query and slice helpers

Use these when you need a compact artifact for review:

- `fretboard diag meta <bundle_dir|bundle.json|bundle.schema2.json> [--warmup-frames <n>] [--json]`
- `fretboard diag query test-id [<bundle_dir|bundle.json|bundle.schema2.json>] <pattern> [--mode <contains|prefix|glob>] [--top <n>] [--case-sensitive] [--json]`
- `fretboard diag slice [<bundle_dir|bundle.json|bundle.schema2.json>] --test-id <test_id> [--frame-id <n>] [--window <id>] [--max-matches <n>] [--max-ancestors <n>] [--json]`

## 3) Troubleshooting patterns

Common signatures:

- `selector.not_found`
  - inspect `selector_resolution_trace`
- routing failures / click didn’t land
  - inspect `hit_test_trace`
- `focus.*` or `type_text_into` stalls
  - inspect `focus_trace` + `text_input_snapshot`
- overlay jumped / flipped / clipped
  - inspect `overlay_placement_trace`
- `timeout`
  - add an intermediate `capture_bundle` and shrink the script

Operational failures:

- missing script / subcommand
  - use promoted `script_id` or a valid path under `tools/diag-scripts/`
- no bundles produced
  - prefer `fretboard diag run ... --launch -- <cmd>`
- `tooling.launch.failed`
  - check writable `--dir`, then inspect `script.result.json`
- unexpectedly huge artifacts
  - run `fretboard diag config doctor --mode launch` and check whether raw bundle or pretty JSON was enabled
- screenshot capability missing
  - ensure the runner advertises `diag.screenshot_png`
- flaky selectors
  - add or repair `test_id` in the component/recipe layer

## 4) Component conformance playbooks

Use invariants-first, evidence-first gates; avoid snapshotting every internal state.

- Select: `select-conformance.md`
- Combobox: `combobox-conformance.md`
- Layout sweep: `layout-sweep.md`
- Web runner transport notes: `web-runner.md`

## 5) Maintainer guardrails

When adding or refactoring a diagnostics entrypoint that supports `--launch`:

- keep tool-launched execution funneled through the same launch helper path
- preserve per-run config writing and session isolation behavior
- keep reason-code-first error reporting stable
- prefer bounded sidecars over requiring raw bundle inspection

## 6) Root-cause hardening ideas

When docking or multi-window bugs keep finding new ways to hang:

- capture a smallest repro script first
- inspect `resource.footprint.json`, `script.result.json`, and routing traces before adding more logging
- keep fixed-delta or no-human-input runs available so intermittent failures remain reproducible
- leave a conformance or perf gate behind when the failure class is understood

## 7) Related references

- `evidence-triage.md`
- `perf-handoff.md`
- `select-conformance.md`
- `combobox-conformance.md`
- `layout-sweep.md`
