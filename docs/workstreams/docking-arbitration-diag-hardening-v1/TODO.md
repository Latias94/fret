# Docking arbitration diag hardening (v1) — TODO

Scope: stabilize the `docking-arbitration` diagnostics suite in `--launch` mode on desktop (native),
with special focus on multi-window tear-off + drag-back sequences.

## Immediate TODOs (next)

- Re-run `fretboard diag suite docking-arbitration` after the 2026-03-03 fixes below and record the new first failure
  (if any) in this workstream.
- Prioritize “timebase decoupling” so docking scripts cannot hang on occlusion/idle (root cause class):
  - Workstream: `docs/workstreams/ui-diagnostics-timebase-decoupling-v1/README.md`
  - Goal: scripted runs always progress or fail with `reason_code=timeout.no_frames` (never a tooling timeout).
- Turn correctness debugging into stage gates:
  - After each merge-back drop, gate `dock_drop_resolved_is_some` + `dock_drop_resolved_zone_is` and capture one bounded bundle.
  - Prefer inner-hint drops (`dock-arb-hint-inner-*`) over outer-hint drops for idempotence (outer-hint tends to produce `wrap_binary`).
- Decide the contract for “scripted cross-window drag release”:
  - which subsystem owns `Drop` routing (runner vs in-app diagnostics injection),
  - which coordinate space is the source of truth (screen vs window-client),
  - what the required evidence gates are (bundle fields + assertions).
- Document the repeat-mode contract for multi-window docking demos:
  - required env (e.g. `FRET_DOCK_ALLOW_MULTI_WINDOW_TEAR_OFF=1`),
  - recommended tooling flags (e.g. ignore window bounds / scene fingerprint drift),
  - whether `--reuse-launch` is required for stability.
- Convert any remaining schema v1 docking scripts to schema v2.
- Reduce coupling to layout presets (prefer fingerprints / structural assertions where possible).

## Hardening backlog (audit + future-proofing)

- Ensure bundle-level evidence is sufficient without logs:
  - `debug.docking_interaction.dock_graph_signature` / `dock_graph_stats` should be present and up-to-date for all frames
    that matter to gates (either by recording every frame, or by an explicit “latest snapshot” contract).
- Make shutdown failures unambiguous in artifacts:
  - require `resource.footprint.json` for `--launch` runs,
  - treat `killed=true` as a “not clean” run that should be investigated (exit trigger not observed / deadlock).
- “Diag resilience” policy: scripted repros should not be terminated by debug-only internal assertions (e.g. focus
  snapshot invariants) when a safe downgrade is possible and preserves evidence.
  - Prefer fixing the root cause, but allow temporary non-fatal behavior in diag/harness paths if it keeps the repro
    running and captures a bundle.
- Cached `test_id` predicate evaluation: keep auditing the trade-off (occlusion deadlock avoidance vs stale-cache false
  positives).
  - Maintain and revisit the freshness rule and evidence (event log) as we learn more about snapshot liveness.
  - Add a minimal “occluded window still progresses” repro script if we do not already have one that is stable.
- Script timeouts should be time-based, not frame-only:
  - Prefer `wait_until.timeout_ms` for no-frame/occlusion resilience.
  - Use `wait_ms` only as a last-resort stabilization yield when no semantic predicate exists (tooling lints long sleeps).

## Regression gates (candidate)

- A small “hardening smoke” suite for docking that includes:
  - tear-off creation,
  - hover routing across windows,
  - drag-back merge (tab restored into main),
  - no stuck drag sessions after release/cancel.

## Done (2026-03-02)

- Runner: isolate scripted cursor overrides from physical mouse movement.
- Window counting: `known_window_count_*` predicates use a runner-owned source-of-truth.
- Cached `test_id` predicate evaluation is freshness-bounded and emits evidence when used.
- `fretboard diag repeat --reuse-launch` clears `script.result.json` between runs to avoid stale `run_id` timeouts.
- Script migration: avoid `pointer_down` ping-pong for relative window targets (wake the active script window instead of
  migrating).
- Multi-window overlap z-order gate unblocked:
  - Docking no longer cancels the global dock drag on `PointerCancel` (runner-synthesized stale-state cleanup).
  - Diagnostics allow global predicates to evaluate off-window (avoid occlusion stalls).
  - Runner cancels the routed drag using the internal routing pointer id on mouse-up.
- Unblocked Windows/MSVC rebuild of docking demos (`taffy`-related LNK2019) by compiling `taffy` with a single
  codegen unit in dev profiles.
- Chained tear-off (two tabs) now returns to the pre-tearoff fingerprint after two merge-backs (script-level targeting
  fix: avoid hint retargeting to the wrong leaf by explicitly hovering a stable viewport in the destination window).
- Added the chained tear-off script to `diag-hardening-smoke-docking`.

## Done (2026-03-03)

- Script termination hardening: avoid trailing `wait_frames` after a final `capture_bundle` in multi-window docking
  scripts, because the last remaining window can be occluded/idle and stop producing redraw callbacks (tooling timeout).
- `drag_pointer_until` hardening: allow out-of-window cursor motion when waiting on window-count predicates
  (`known_window_count_*`) so tear-off creation scripts can drive the cursor beyond the active window bounds.
- Diagnostics protocol: allow `drag_pointer` steps to opt out of clamping the end position to the active window bounds
  (`clamp_to_window_bounds=false`) so tear-off drags can deterministically leave the window.
- Script runner: `set_cursor_in_window_logical` updates the in-memory pointer session position when targeting the active
  window (prevents stale `dock_drop_resolve` position during docking gates).
- Script runner: when a cross-window dock drag is active, `move_pointer` injects `InternalDrag::Over` (instead of
  `PointerMove`) to refresh docking hover/resolve without relying on per-window pointer broadcasting.
- Docking UI: `dock_hint_pick_zone` keeps `Center` tightly coupled to the visual center rect (avoids “sticky center”
  picks just outside the center pad; unblocks `dock_drop_preview_kind_is(insert_into_split)` gates).
- Script runner: pending “cross-window drag cancel” escape hatch clears immediately when the pointer has no active drag,
  preventing accidental cancellation of later drags started within the retry window.
- Docking policy: `panel_min_content_size=None` now falls back to the default viewport min size (240×160 logical px) even
  when a `DockingPolicy` is installed for other hooks (e.g. drop-zone masking); unblocks splitter-drag min-size gates.
- Runtime hardening: while a diagnostics script is active, arm a keepalive timer that can advance a conservative subset
  of script steps (and fail with `timeout.no_frames`) even if redraw callbacks stop (occlusion/idle).
- Runtime hardening: allow a small “burst” of frame-independent tail steps so scripts do not require an additional frame
  to run a final `capture_bundle` after the last semantic assertion (reduces tooling timeouts at tight `--timeout-ms`).
- Docs: clarify window-count and docking drop resolve predicate semantics in the main diagnostics reference:
  `docs/ui-diagnostics-and-scripted-tests.md`.
- Runtime hardening: include `serde_json` parse error details in `script.result.json.reason` for
  `reason_code=script.parse_failed` (schema v1/v2), so suite failures are actionable without opening logs.
- Script hardening: make schema v2 `wait_until` tolerate missing `timeout_frames` and avoid treating tab order as a
  contract in `docking-arbitration-demo-tab-bar-drop-end-insert-index-two-tabs` (assert both panels exist instead of a
  specific ordering).
- Diagnostics input synthesis: allow scripted drags to move outside window bounds (tear-off requires OOB routing)
  without clamping the end position back inside the window client rect.
  - implementation: `ecosystem/fret-bootstrap/src/ui_diagnostics/script_steps_drag.rs`
- Docking UI: include `TextFontStackKey` in the retained tab-title cache key so tab labels do not disappear after a
  system font rescan / font stack stabilization.
  - implementation: `ecosystem/fret-docking/src/dock/space.rs`
