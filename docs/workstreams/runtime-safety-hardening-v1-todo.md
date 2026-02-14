# Runtime Safety Hardening v1 — TODO Tracker

Status: Active (follow-ups)

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
  - Full gate set last verified: 2026-02-13
  - Partial re-verification notes:
    - 2026-02-14: `cargo clippy -p fret-ui --all-targets -- -D warnings`: PASS
    - 2026-02-14: `cargo nextest run -p fret-ui`: PASS
    - 2026-02-14: `python3 tools/check_layering.py`: PASS
  - Windows note: prefer `cargo fmt -p <crate>` for targeted formatting (workspace-wide `cargo fmt` may fail with `os error 206` on long paths).

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

## M5 — Clippy hygiene (warnings-as-errors, local gates)

- [x] RSH-clippy-001 Make `cargo clippy` pass with `-D warnings` for the workstream crates.
  - Evidence: `cargo clippy -p fret-ui --all-targets -- -D warnings`: PASS
  - Evidence: `cargo clippy -p fret-app --all-targets -- -D warnings`: PASS

## M6 — Local `unsafe` tightening (fret-ui follow-ups)

- [x] RSH-ui-001 Remove avoidable `unsafe` from `TestHost` globals leasing.
  - Evidence: `crates/fret-ui/src/test_host.rs` (`TestHost::with_globals` no longer uses pointer writes)
  - Evidence: `cargo nextest run -p fret-ui`: PASS
- [x] RSH-ui-002 Add regression gates for small inline list invariants (inline ↔ spill boundaries, stable ordering).
  - Evidence: `crates/fret-ui/src/tree/mod.rs` (tests: `small_list_tests::*`)
- [x] RSH-ui-003 Tighten `unsafe` in `SmallNodeList` / `SmallCopyList` slice views and spill conversion.
  - Evidence: `crates/fret-ui/src/tree/mod.rs` (`assume_init_slice_ref`, `Small{Node,Copy}List::as_slice`)
- [x] RSH-clippy-002 Fix clippy `items_after_test_module` in `fret-ui`.
  - Evidence: `crates/fret-ui/src/declarative/host_widget/paint.rs` (tests moved to file end)

## M7 — Defensive panic hardening (fret-app follow-ups)

- [x] RSH-global-003 Make `with_global_mut` resilient to corrupted globals state (non-strict mode recovers; strict mode panics).
  - Evidence: `crates/fret-app/src/app.rs` (`with_global_mut_impl`: downcast/marker validation no longer panics by default)
  - Evidence: `crates/fret-app/src/app.rs` (tests: `with_global_mut_recovers_from_corrupt_existing_global_value_in_non_strict_mode`, `with_global_mut_recovers_when_lease_marker_is_removed_in_non_strict_mode`)
  - Gates:
    - `cargo clippy -p fret-app --all-targets -- -D warnings`: PASS
    - `cargo nextest run -p fret-app`: PASS
    - `python3 tools/check_layering.py`: PASS

## M8 — Defensive panic hardening (fret-ui follow-ups)

- [x] RSH-ui-004 Make element state access resilient to corruption (type mismatches) and unwind-safe by default (strict mode panics).
  - Evidence: `crates/fret-ui/src/elements/runtime.rs` (`WindowElementState::{with_state_mut, try_with_state_mut}`)
  - Evidence: `crates/fret-ui/src/elements/access.rs` (tests: `with_element_state_recovers_from_type_mismatch_in_non_strict_mode`, `with_element_state_restores_state_on_panic`)
- [x] RSH-ui-005 Remove `expect("text input/text area")` from declarative host widgets (defensive debug assertions + stable fallbacks).
  - Evidence: `crates/fret-ui/src/declarative/host_widget.rs`
  - Evidence: `crates/fret-ui/src/declarative/host_widget/layout.rs`
  - Evidence: `crates/fret-ui/src/declarative/host_widget/paint.rs`
  - Evidence: `crates/fret-ui/src/declarative/host_widget/semantics.rs`
  - Evidence: `crates/fret-ui/src/declarative/host_widget/event/text.rs`
  - Gates:
    - `cargo clippy -p fret-ui --all-targets -- -D warnings`: PASS
    - `cargo nextest run -p fret-ui`: PASS
    - `python3 tools/check_layering.py`: PASS

## M9 — Panic surface audit (fret-ui follow-ups)

- [x] RSH-ui-006 Remove "checked above" `expect(...)` and redundant `Option` unwrapping in input/dispatch hot paths.
  - Evidence: `crates/fret-ui/src/tree/dispatch.rs`
  - Evidence: `crates/fret-ui/src/text/input/widget.rs`
  - Gates:
    - `cargo clippy -p fret-ui --all-targets -- -D warnings`: PASS
    - `cargo nextest run -p fret-ui`: PASS
    - `python3 tools/check_layering.py`: PASS
- [x] RSH-ui-007 Decide policy for `taffy` error handling (avoid `expect(...)` on layout engine operations).
  - Policy (v1):
    - Strict mode (`FRET_STRICT_RUNTIME`): panic on `taffy` errors.
    - Default (non-strict): warn once + fall back to naive measurement/layout (no process termination).
  - Evidence (implemented for host-widget measurement):
    - `crates/fret-ui/src/declarative/host_widget/measure.rs` (`warn_taffy_error_once`, `fallback_measure_{flex,grid}`)
  - Evidence (implemented for layout engine):
    - `crates/fret-ui/src/layout/engine.rs` (`TaffyLayoutEngine::warn_taffy_error_once`, `request_layout_node -> Option<LayoutId>`)
    - `crates/fret-ui/src/layout/engine.rs` (layout solve skips `mark_solved_subtree` when `compute_layout_with_measure` fails, enabling widget fallback)
  - Gates:
    - `cargo clippy -p fret-ui --all-targets -- -D warnings`: PASS
    - `cargo nextest run -p fret-ui`: PASS
    - `python3 tools/check_layering.py`: PASS

- [x] RSH-ui-008 Remove redundant text metrics `expect(...)` in declarative host-widget layout (defensive fallback for cache corruption).
  - Evidence: `crates/fret-ui/src/declarative/host_widget/layout.rs` (remove `expect("cached metrics")`; re-prepare text when metrics missing)
  - Gates:
    - `cargo clippy -p fret-ui --all-targets -- -D warnings`: PASS
    - `cargo nextest run -p fret-ui declarative::host_widget`: PASS
