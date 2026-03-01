# Example Suite v1 — Gates and Diag Script Templates

This appendix defines minimal, repeatable gate patterns for examples.

The intended default gate is **a scripted diag run** using stable `test_id`s.

## Where scripts live

- Script library: `tools/diag-scripts/`
- Suite membership (redirect stubs): `tools/diag-scripts/suites/<suite-name>/`
- Suite design notes: `tools/diag-scripts/suites/README.md`

## Minimal example gate (script v2)

Use `schema_version: 2` for new scripts.

Recommended meta fields:

- `meta.name`: human-friendly name
- `meta.tags`: stable tags (use the example ID)
- `meta.required_capabilities`: keep explicit
- `meta.target_hints`: 1–3 bullets for what the script validates

Recommended step shape:

- wait for the root `test_id` to exist
- perform 1–3 actions
- capture a screenshot and a bundle

Skeleton:

```json
{
  "schema_version": 2,
  "meta": {
    "name": "Example: <id> minimal gate",
    "tags": ["example", "<id>"],
    "required_capabilities": ["diag.script_v2", "diag.screenshot_png"],
    "target_hints": ["Validates <one high-signal outcome>."]
  },
  "steps": [
    { "type": "reset_diagnostics" },
    {
      "type": "wait_until",
      "predicate": { "kind": "exists", "target": { "kind": "test_id", "id": "<id>.root" } },
      "timeout_frames": 600
    },
    { "type": "capture_screenshot", "label": "<id>.baseline" },
    { "type": "capture_bundle", "label": "<id>.baseline" }
  ]
}
```

Notes:

- Prefer `click_stable` when clicking controls with potential hover/press transitions.
- Prefer `scroll_into_view` + `require_fully_within_window` for screenshot stability.

## Text redaction: stable predicates

Diagnostics may run with text redaction enabled (for example via `FRET_DIAG_REDACT_TEXT=1`), where
`label` / `value` strings can be replaced with placeholders like `<redacted len=123>`.

Guidelines:

- Avoid `label_contains` / `value_contains` when redaction can be enabled.
- Prefer redaction-safe predicates:
  - `label_len_is` / `label_len_ge`
  - `value_len_is` / `value_len_ge`
- Prefer structured semantics over localized strings when possible:
  - `semantics_numeric_*` for range/progress-like controls
  - `checked_is`, `selected_is`, `role_is` for interaction state

## Suite membership (redirect stub)

Suites are curated directories of redirect stubs.

Add a file under `tools/diag-scripts/suites/<suite-name>/`:

```json
{
  "schema_version": 1,
  "kind": "script_redirect",
  "to": "tools/diag-scripts/<path-to-canonical-script>.json"
}
```

This keeps suite membership stable even if scripts move (fearless refactors).

## “Official example” gate expectations

For cookbook examples:

- each example has at least one script in `tools/diag-scripts/` tagged with the example ID
- each example is included in one suite (e.g. `tools/diag-scripts/suites/examples-app-track/`)

For reference apps (app-scale):

- define suites per workflow:
  - smoke (first frame)
  - overlay conformance
  - docking basics
  - perf steady (optional)
