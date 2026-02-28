## Diagnostics config examples (schema v1)

This directory contains a canonical example for `FRET_DIAG_CONFIG_PATH`:

- `tools/diag-configs/diag.config.example.json`

The config file schema is defined in `crates/fret-diag-protocol/src/lib.rs` (`UiDiagnosticsConfigFileV1`).
The runtime loader lives in `ecosystem/fret-bootstrap/src/ui_diagnostics/config.rs`.

Evidence anchors:

- Schema: `crates/fret-diag-protocol/src/lib.rs` (`UiDiagnosticsConfigFileV1`, `UiDiagnosticsConfigPathsV1`).
- Runtime resolution: `ecosystem/fret-bootstrap/src/ui_diagnostics/config.rs` (`UiDiagnosticsConfig::default`).

### Drift audit: `diag.config.example.json` vs runtime

Goal: ensure every example field is either implemented by the runtime or explicitly documented as planned/ignored.
As of `2026-02-26`, **all fields present in the example config are implemented** by the runtime loader.

Notes:

- Resolution order is usually **env overrides config file** (manual escape hatch), then runtime defaults.
- A few fields are intentionally **config-only** (no env override) to keep tool-launched runs deterministic and to avoid
  adding more “switches” during the v2 hardening period:
  - `write_bundle_json`
  - `write_bundle_schema2`
- Tool-launched escape hatch: if you explicitly need raw `bundle.json` for a `--launch` run, use
  `fretboard diag ... --launch-write-bundle-json --launch -- <cmd>` (tooling writes a per-run config with
  `write_bundle_json=true`).
- Most `paths.*` values are resolved relative to the effective `out_dir` when the config value is not absolute.
- Some numeric fields are clamped by the runtime for safety/bundle size control.

#### Top-level fields

- `schema_version`:
  - Used by `serde_json` parsing (tooling expects `1` for `UiDiagnosticsConfigFileV1`).
- `enabled`:
  - Runtime: `config_enabled = c.enabled.unwrap_or(true)` in `ecosystem/fret-bootstrap/src/ui_diagnostics/config.rs`.
  - Effective enablement is also implied by `FRET_DIAG`, `FRET_DIAG_DIR`, `--fret-diag*` args, and DevTools WS config.
- `out_dir`:
  - Runtime: `c.out_dir` (trim + non-empty) influences `FRET_DIAG_DIR` fallback.
  - iOS note: relative paths may be re-rooted to `$HOME`/`tmp` (see `resolve_ios_diag_out_dir`).
- `paths`:
  - Runtime: `c.paths.*` is used for all supported trigger/result/script/inspect/screenshot files.
- `write_bundle_json`:
  - Runtime: `c.write_bundle_json` controls whether the large raw `bundle.json` is written.
  - Default: `true` (manual dumps keep writing it unless explicitly configured).
- `write_bundle_schema2`:
  - Runtime: `c.write_bundle_schema2` controls whether a compact `bundle.schema2.json` companion artifact is written.
  - Default: `false`.
- `allow_script_schema_v1`:
  - Runtime: `c.allow_script_schema_v1` controls whether script schema v1 inputs are accepted.
  - Default: `true` (manual/backwards-compat).
  - Tool-launched runs: tooling writes `false` into its generated `diag.config.json` so the runtime stays on the v2-only path.
- `max_events`:
  - Runtime: `c.max_events` (fallback default `2000`).
  - Env override: `FRET_DIAG_MAX_EVENTS`.
- `max_snapshots`:
  - Runtime: `c.max_snapshots` (fallback default `300`).
  - Env override: `FRET_DIAG_MAX_SNAPSHOTS`.
- `script_dump_max_snapshots`:
  - Runtime: `c.script_dump_max_snapshots` (fallback default `30`), clamped to `[1, max_snapshots]` when `max_snapshots > 0`.
  - Env override: `FRET_DIAG_SCRIPT_DUMP_MAX_SNAPSHOTS`.
- `capture_semantics`:
  - Runtime: `c.capture_semantics` (fallback default `true`).
  - Env override: `FRET_DIAG_SEMANTICS`.
- `max_semantics_nodes`:
  - Runtime: `c.max_semantics_nodes` (fallback default `50_000`), clamped to `[0, 500_000]`.
  - Env override: `FRET_DIAG_MAX_SEMANTICS_NODES`.
- `semantics_test_ids_only`:
  - Runtime: `c.semantics_test_ids_only` (fallback default `false`).
  - Env override: `FRET_DIAG_SEMANTICS_TEST_IDS_ONLY`.
- `screenshots_enabled`:
  - Runtime: `c.screenshots_enabled` (fallback default `false`).
  - Env override: `FRET_DIAG_GPU_SCREENSHOTS`.
- `screenshot_on_dump`:
  - Runtime: `c.screenshot_on_dump` (fallback default `false`).
  - Env override: `FRET_DIAG_BUNDLE_SCREENSHOT`.
- `redact_text`:
  - Runtime: `c.redact_text` (fallback default `true`).
  - Env override: `FRET_DIAG_REDACT_TEXT`.
- `max_debug_string_bytes`:
  - Runtime: `c.max_debug_string_bytes` (fallback default `4096`), clamped to `[0, 256KiB]`.
  - Env override: `FRET_DIAG_MAX_DEBUG_STRING_BYTES`.
- `max_gating_trace_entries`:
  - Runtime: `c.max_gating_trace_entries` (fallback default `200`), clamped to `[0, 2000]`.
  - Env override: `FRET_DIAG_MAX_GATING_TRACE_ENTRIES`.
- `script_keepalive`:
  - Runtime: `c.script_keepalive` (fallback default `true`).
  - Env override: `FRET_DIAG_SCRIPT_KEEPALIVE`.
- `script_auto_dump`:
  - Runtime: `c.script_auto_dump` (fallback default `true`).
  - Env override: `FRET_DIAG_SCRIPT_AUTO_DUMP`.
- `pick_auto_dump`:
  - Runtime: `c.pick_auto_dump` (fallback default `true`).
  - Env override: `FRET_DIAG_PICK_AUTO_DUMP`.
- `frame_clock_fixed_delta_ms`:
  - Runtime: `c.frame_clock_fixed_delta_ms` is used as a fallback when the window frame clock env override is not set.
  - Env override: `fret_core::WindowFrameClockService::fixed_delta_from_env()` (see `ecosystem/fret-bootstrap/src/ui_diagnostics/config.rs`).
- `devtools_embed_bundle`:
  - Runtime: `c.devtools_embed_bundle` controls whether `bundle.dumped` embeds a `bundle.json` payload over DevTools WS.
  - Fallback default: `true` on `wasm32`, otherwise `false`.

#### `paths.*` fields

All `paths` fields in the example config are used by the runtime as a fallback when the matching env var is not set:

- dump trigger/readiness/exit:
  - `trigger_path` (`FRET_DIAG_TRIGGER_PATH`)
  - `ready_path` (`FRET_DIAG_READY_PATH`)
  - `exit_path` (`FRET_DIAG_EXIT_PATH`)
- scripts:
  - `script_path` (`FRET_DIAG_SCRIPT_PATH`)
  - `script_trigger_path` (`FRET_DIAG_SCRIPT_TRIGGER_PATH`)
  - `script_result_path` (`FRET_DIAG_SCRIPT_RESULT_PATH`)
  - `script_result_trigger_path` (`FRET_DIAG_SCRIPT_RESULT_TRIGGER_PATH`)
- picking:
  - `pick_trigger_path` (`FRET_DIAG_PICK_TRIGGER_PATH`)
  - `pick_result_path` (`FRET_DIAG_PICK_RESULT_PATH`)
  - `pick_result_trigger_path` (`FRET_DIAG_PICK_RESULT_TRIGGER_PATH`)
- inspect:
  - `inspect_path` (`FRET_DIAG_INSPECT_PATH`)
  - `inspect_trigger_path` (`FRET_DIAG_INSPECT_TRIGGER_PATH`)
- on-demand PNG screenshots:
  - `screenshot_request_path` (`FRET_DIAG_SCREENSHOT_REQUEST_PATH`)
  - `screenshot_trigger_path` (`FRET_DIAG_SCREENSHOT_TRIGGER_PATH`)
  - `screenshot_result_path` (`FRET_DIAG_SCREENSHOT_RESULT_PATH`)
  - `screenshot_result_trigger_path` (`FRET_DIAG_SCREENSHOT_RESULT_TRIGGER_PATH`)

### Known gaps (runtime knobs not expressible via config file)

The runtime currently has a few relevant toggles that are **not** part of `UiDiagnosticsConfigFileV1`, so they cannot be
expressed in `FRET_DIAG_CONFIG_PATH` (they are env-only or transport-only today):

- DevTools WS connection (`devtools_ws_url` / `devtools_token`): provided by URL query (wasm32) or env vars (native).
