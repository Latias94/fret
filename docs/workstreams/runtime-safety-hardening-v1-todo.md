# Runtime Safety Hardening v1 — TODO Tracker

Status: Draft

This document tracks tasks for:

- `docs/workstreams/runtime-safety-hardening-v1.md`
- `docs/workstreams/runtime-safety-hardening-v1-milestones.md`

Tracking format:

- Status: `[ ]` open, `[~]` in progress, `[x]` done, `[!]` blocked
- ID: `RSH-{area}-{nnn}`

When completing an item, prefer leaving 1–3 evidence anchors:

- file paths + key functions/tests
- and/or a `fretboard diag` script/suite name

## M0 — Plan + gates

- [x] RSH-doc-001 Link this workstream from `docs/workstreams/README.md`.
  - Evidence: `docs/workstreams/README.md` (lists `runtime-safety-hardening-v1*` files)
- [x] RSH-gate-001 Define the minimal always-run gates for this workstream:
  - `cargo nextest run -p fret-runtime`
  - `cargo nextest run -p fret-ui`
  - `cargo nextest run -p fret-app`
  - `python3 tools/check_layering.py`
  - Last verified: 2026-02-13
  - Current status (as of this branch):
    - `cargo nextest run -p fret-runtime`: PASS
    - `cargo nextest run -p fret-ui`: PASS
    - `cargo nextest run -p fret-app`: PASS
    - `python3 tools/check_layering.py`: PASS

## M1 — `ModelStore v2` (remove public leasing; no panicking reads)

- [x] RSH-model-001 Write an ADR that locks the `ModelStore v2` public contract:
  - no public lease handles,
  - closure-based access only,
  - `AlreadyLeased/TypeMismatch` returned as errors (never panics by default),
  - unwind-safe invariant restoration.
  - Evidence: `docs/adr/0269-modelstore-v2-lease-and-unwind-safety-v1.md`
- [x] RSH-model-002 Implement `ModelStore v2` API surface in `crates/fret-runtime`:
  - remove/privatize `ModelLease` from the public surface,
  - add `try_get_copied/try_get_cloned -> Result<Option<T>, ModelUpdateError>` and make `get_copied/get_cloned` non-panicking by default.
  - Evidence: `crates/fret-runtime/src/model/mod.rs` (stop re-exporting `ModelLease`)
  - Evidence: `crates/fret-runtime/src/model/store.rs` (`ModelStore::{try_get_copied,try_get_cloned}`; `lease/end_lease` are `pub(super)`)
- [x] RSH-model-003 Migrate call sites in core/mechanism crates and first-party apps/ecosystem to the new APIs.
  - Evidence: `crates/fret-ui/src/resizable_split/widget.rs` (use `try_get_copied` with debug-only context logging)
  - Evidence: `crates/fret-ui/src/resizable_panel_group/widget.rs` (use `try_get_cloned` with debug-only context logging)
  - Evidence: `crates/fret-ui/src/text/area/bound.rs` (use `try_get_cloned` with debug-only context logging)
  - Evidence: `crates/fret-ui/src/text/input/bound.rs` (use `try_get_cloned` with debug-only context logging)
- [x] RSH-model-004 Add regression gates:
  - non-panicking `AlreadyLeased/TypeMismatch` behavior,
  - unwind does not poison the model store (when `panic=unwind`).
  - Evidence: `crates/fret-runtime/src/model/store.rs` (tests: `get_copied_returns_none_while_leased...`, `update_unwind_does_not_poison_store_state`)
- [x] RSH-model-005 Add optional “strict runtime” mode (feature or env) that can re-enable panics for development.
  - Evidence: `crates/fret-runtime/src/model/store.rs` (`FRET_STRICT_RUNTIME`, `strict_runtime_enabled`, strict-mode panic paths)

## M2 — Menu patch: delete avoidable `unsafe`

- [x] RSH-menu-001 Rewrite `crates/fret-runtime/src/menu/apply.rs` patch descent in safe Rust.
  - Evidence: `crates/fret-runtime/src/menu/apply.rs` (`resolve_menu_items_mut` no longer uses `unsafe`)
- [x] RSH-menu-002 Add targeted tests for patch path resolution (title/path targeting, nested submenus, error cases).
  - Evidence: `crates/fret-runtime/src/menu/tests.rs` (`patch_apply_errors_when_menu_path_is_missing`)

## M3 — Theme v2: validate + normalize; no required-token panics

- [x] RSH-theme-001 Write an ADR for Theme token contract tiers:
  - typed core keys for mechanism/runtime,
  - string extension tokens for ecosystem.
  - Evidence: `docs/adr/0270-theme-token-contract-tiers-and-missing-token-policy-v1.md`
- [x] RSH-theme-002 Implement theme normalization:
  - fill missing core keys from `default_theme()` by keeping baseline dotted keys and semantic aliases synchronized with the resolved typed baseline,
  - emit missing-token diagnostics at access-time (warn-once + stable fallback; strict mode can panic).
  - Evidence: `crates/fret-ui/src/theme/mod.rs` (`Theme::apply_config` baseline/semantic sync)
  - Evidence: `crates/fret-ui/src/theme/mod.rs` (tests: `semantic_keys_backfill_typed_baseline_colors_when_missing`, `baseline_dotted_keys_update_typed_theme_colors`)
- [x] RSH-theme-003 Reduce stringly `*_required` usage inside `crates/fret-ui` (prefer typed keys).
  - Evidence: `crates/fret-ui/src/text/area/mod.rs` (typed `ThemeColorKey`/`ThemeMetricKey` + `*_token` APIs)
  - Evidence: `crates/fret-ui/src/declarative/host_widget/paint.rs` (replace `*_required` usage)
- [x] RSH-theme-004 Migrate apps/ecosystem call sites off panicking token accessors.
  - Evidence: workspace-wide migration from `*_required` → `*_token` (e.g. `ecosystem/`, `apps/`)
  - Evidence: `ecosystem/fret-ui-kit/src/style/theme_read.rs` (`ThemeTokenRead::{color_token,metric_token}`; call sites compile against the narrow trait)
- [x] RSH-theme-005 Add regression gates:
  - missing tokens never panic by default,
  - diagnostics are emitted once with a stable summary.
  - Evidence: `crates/fret-ui/src/theme/mod.rs` (`*_required` uses fallback + warn-once; strict mode panics via `FRET_STRICT_RUNTIME`)
  - Evidence: `crates/fret-ui/src/theme/mod.rs` (tests: `required_accessors_do_not_panic_when_tokens_are_missing_by_default`, `required_accessors_panic_in_strict_runtime_mode`)
  - Evidence: `crates/fret-ui/src/theme/mod.rs` (test: `missing_theme_token_diagnostics_warn_once_per_key`)

## M4 — Globals + env flags hardening

- [x] RSH-global-001 Convert global lease violations to `Result` errors (panic only in strict/debug modes).
  - Evidence: `crates/fret-app/src/app.rs` (`GlobalAccessError`, `try_global`, `try_set_global`)
  - Evidence: `crates/fret-app/src/app.rs` (non-strict fallback: `global()` returns `None` when leased; `set_global()` defers via `pending_globals`)
  - Evidence: `crates/fret-app/src/app.rs` (tests: `global_access_returns_none_while_leased`, `set_global_defers_while_leased_and_applies_after`)
- [x] RSH-global-002 Remove re-entrant `with_global_mut` `unsafe` fallback (nested leases are an error).
  - Evidence: `crates/fret-app/src/app.rs` (`with_global_mut_impl`: nested leases run against a temporary value in non-strict mode; strict mode panics)
  - Evidence: `crates/fret-app/src/app.rs` (test: `nested_with_global_mut_is_an_error_and_never_requires_unsafe`)
- [x] RSH-env-001 Centralize `FRET_*` debug flags into a cached config struct and remove hot-path env reads.
  - Evidence: `crates/fret-ui/src/runtime_config.rs` (`UiRuntimeEnvConfig`, `ui_runtime_config`)
  - Evidence: `crates/fret-ui/src/tree/layout.rs` (layout profiling/taffy dump/fallback-solve gates read cached config)
  - Evidence: `crates/fret-ui/src/tree/mod.rs` (gate-sync, pointer-down-outside, semantics profiling read cached config)
