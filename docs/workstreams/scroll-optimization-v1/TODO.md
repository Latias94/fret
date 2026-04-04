# Scroll Optimization Workstream (v1) — TODO

Date: 2026-04-04  
Status: Active

## Current slice — Deferred probe seed vs authoritative extent

- [x] Make deferred probe policy read retained seed state before deciding to skip a deep probe.
- [x] Allow deferred invalidation frames to consume `intrinsic_measure_cache` as the seed extent
  when retained child measured sizes are absent.
- [x] Centralize authoritative extent commits so pending probe clearing only happens on explicit
  probe / authoritative observation paths.
- [x] Ensure unchanged authoritative post-layout observation still clears deferred invalidation
  pending state instead of forcing an extra at-edge probe on the next frame.
- [x] Record the dedicated verification results for the seed/authority regression gates in
  `EVIDENCE_AND_GATES.md`.

## Follow-on slice — Command and event focus targets resolve authoritative live attached nodes

- [x] Replace command dispatch source-element resolution so pending command metadata falls back from
  stale detached `node_entry` seeds to the live attached node.
- [x] Replace command-hook `request_focus(element)` resolution so the hook host no longer trusts
  `window_state.node_entry(target)` directly.
- [x] Add event-side `requested_focus_target` plumbing so key/pointer focus hooks defer element
  target resolution until dispatch regains access to `UiTree`.
- [x] Lock the command/event live-node resolution contract with focused stale-detached regression
  gates in `EVIDENCE_AND_GATES.md`.

## Follow-on slice — Declarative rebuild and invalidation element paths resolve authoritative live nodes

- [x] Replace element-runtime model/global invalidation target resolution so stale detached
  `node_entry` seeds fall back to the live attached node.
- [x] Replace rebuild-time `notify_for_animation_frame` invalidation target resolution so it no
  longer trusts `window_state.node_entry(element)` directly.
- [x] Replace declarative mount/root reuse node resolution so rebuild prefers the live attached
  node and only reuses a retained seed when no live attached node exists for the element.
- [x] Replace view-cache GC / retained virtual-list reconcile root resolution so detached stale
  `node_entry` seeds do not become authoritative keep-alive roots.
- [x] Lock the declarative rebuild/invalidation live-node contract with focused stale-detached
  regression gates in `EVIDENCE_AND_GATES.md`.

## Follow-on slice — Interaction targets resolve authoritative live attached nodes

- [x] Replace hover/pressed interaction target bookkeeping so runtime state stores element identity
  separately from the current authoritative live node instead of resolving `node_entry(element)`
  at mutation time.
- [x] Sync retained hover/pressed/hover-region target nodes against the live attached tree during
  final layout-frame commit so same-element rebuild/remount does not keep clearing a stale node.
- [x] Replace timer element-target dispatch so `Event::Timer` resolves the live attached node in
  `UiTree` / dispatch instead of trusting `window_state.node_entry(element)` directly.
- [x] Replace selectable-text active-selection dispatch so `Event::SetTextSelection` keeps routing
  through the live attached selectable-text node even if retained runtime state or `node_entry`
  was seeded with a stale detached node.
- [x] Lock the interaction-target live-node contract with focused stale-detached regression gates
  in `EVIDENCE_AND_GATES.md`.

## Follow-on slice — Final-layout / dispatch / anchored queries resolve authoritative live attached nodes

- [x] Replace render-time `focus-within` containment and focused-node-to-element sync so they read
  live window-frame nodes before falling back to retained `node_entry` / `element_for_node`
  mappings.
- [x] Replace final-layout focus repair so the canonical focus node resolves from the live attached
  tree instead of the last-known `elements::node_for_element(...)` mapping.
- [x] Replace touch-drag locked target and wheel scroll-dismiss element lookups so dispatch uses
  live attached nodes instead of retained stale `node_entry` mappings.
- [x] Replace anchored layout `anchor_element` resolution so layout uses the live attached anchor
  node rather than the last-known `elements::node_for_element(...)` mapping.
- [x] Add a dedicated wheel scroll-dismiss regression so this dispatch path is locked directly
  instead of only being covered by the broader stale-detached live-node suite.
- [x] Record the new regression gates and the non-test mechanism-path audit result in
  `EVIDENCE_AND_GATES.md`.

## Follow-on slice — Ecosystem runtime paths use explicit live-node query surfaces

- [x] Expose public `fret-ui` query surfaces that distinguish authoritative current/live node
  resolution from last-known retained mappings:
  - `UiTree::live_attached_node_for_element(...)`
  - `elements::live_node_for_element(...)`
  - `ElementContext::live_node_for_element(...)`
- [x] Keep `elements::node_for_element(...)` / `ElementContext::node_for_element(...)` as explicit
  last-known query surfaces instead of silently widening their semantics.
- [x] Tighten the public live query contract so current-frame liveness comes from
  `WindowElementState::node_entry(...).last_seen_frame`, not from
  `ElementFrame::window_frame.instances`, which may retain stale records until subtree GC.
- [x] Replace ecosystem authoritative runtime call sites with the new live query surfaces:
  - overlay focus request / restore paths in `window_overlays/render.rs`,
  - focus-scope initial-focus / restore helpers,
  - dismissable-layer branch resolution,
  - live active-descendant helpers in `fret-ui-kit`.
- [x] Keep render-time semantics authoring surfaces declarative when the parent relationship is
  known before the current frame's child nodes are mounted:
  - `fret-ui-kit/declarative/table.rs`,
  - `fret-ui-shadcn/select.rs`,
  - use `SemanticsDecoration::active_descendant_element(...)` instead of forcing a current-frame
    `NodeId` lookup.
- [x] Let semantics-time declarative relation resolution fall back from the local mounted element
  map to the authoritative current-frame live mapping so retained / virtualized child subtrees can
  still resolve `active_descendant`, `controls`, and related element relations.
- [x] Lock the public-surface contract with stale-last-known regressions in
  `fret-ui-kit` and record the gates in `EVIDENCE_AND_GATES.md`.

## Gates-first checklist

- [x] Confirm baseline scripts pass:
  - [x] `ui-gallery-scroll-area-wheel-scroll` (bundle: `target/fret-diag/1772468071457-scroll-area-wheel-scroll`, 2026-03-02)
  - [x] `ui-gallery-scrollbar-drag-baseline-content-growth` (bundle: `target/fret-diag/1772498133742-scrollbar-drag-baseline-content-growth`, 2026-03-03)
  - [x] `ui-gallery-scroll-area-wheel-torture` (bundle: `target/fret-diag/1772498149599-scroll-area-wheel-torture`, 2026-03-03)
  - [x] `ui-gallery-scroll-area-nested-scroll-routing` (bundle: `target/fret-diag-scroll-nested-debug6/sessions/1772508480737-75452/1772508483614-scroll-area-nested-scroll-routing`, 2026-03-03)
  - [x] `ui-gallery-wheel-burst-coalescing` (new gate: wheel events per frame ≤ 1; suite: `diag-hardening-smoke`)
    - `diag run` evidence: `target/fret-diag-runs/1772530803405-wheel-burst/check.wheel_events_max_per_frame.json` (2026-03-03)
  - [x] `ui-gallery-virtual-list-wheel-torture` (bundle: `target/fret-diag-vlist-wheel/sessions/1772508526189-62940/1772508528623-virtual-list-wheel-torture`, 2026-03-03)
  - [x] `ui-gallery-scroll-area-toggle-code-tabs` (bundle: `target/fret-diag-underflow-check/sessions/1772500876247-61448/1772500879851-scroll-area-toggle-code-tabs`, 2026-03-03)
  - [x] `ui-gallery-scroll-area-expand-at-bottom` (bundle: `target/fret-diag-scroll-expand-at-bottom-v4/sessions/1772539486117-27536/1772539488297`, 2026-03-03)
  - [x] `diag perf perf-ui-gallery-scroll-area` (bundle: `target/fret-perf-scroll-area/sessions/1772501734226-65632/1772501741770`, 2026-03-03)
  - [x] `diag perf perf-ui-gallery-virtual-list` (bundle: `target/fret-perf-vlist/1772508561962`, 2026-03-03)
- [x] Promote nested scroll routing into `diag-hardening-smoke`:
  - suite manifest: `tools/diag-scripts/suites/diag-hardening-smoke/suite.json`
  - script: `tools/diag-scripts/ui-gallery/scroll-area/ui-gallery-scroll-area-nested-scroll-routing.json`

## Mechanism hardening

- [x] Fix view-cache contained relayout bookkeeping (layout invalidation clears must keep subtree aggregation in sync):
  - `crates/fret-ui/src/tree/layout/entrypoints.rs`
- [x] Keep layout-only contained relayout from forcing next-frame cache-root rerender / dirty-view carry-over:
  - `crates/fret-ui/src/tree/layout/entrypoints.rs`
  - `crates/fret-ui/src/tree/ui_tree_view_cache.rs`
  - `crates/fret-ui/src/tree/tests/view_cache.rs`
- [x] Prune detached roots from layout follow-up scheduling before final layout:
  - `crates/fret-ui/src/tree/layout/entrypoints.rs`
  - `crates/fret-ui/src/tree/tests/view_cache.rs`
  - `crates/fret-ui/src/tree/tests/barrier_subtree_layout_dirty_aggregation.rs`
- [x] Audit all barrier-related paths that can affect scroll surfaces:
  - [x] `set_children_barrier(...same_children...)` schedules authoritative follow-up relayout when
    descendant layout work is still pending,
  - [x] `set_children(...same_children...)` and `set_children_in_mount(...same_children...)`
    reconnect authoritative layout when they repaired stale parent pointers under pending
    descendant layout work,
  - [x] `set_root(...)` / `set_base_root(...)` now prune detached focus/capture immediately when a
    layer-root replacement rebases the active layer roots, while preserving still-active overlay
    interaction state,
  - [x] pending multi-stroke shortcut continuation now revalidates the authoritative key-context
    stack before matching the next chord, so root replacement cannot keep stale shortcut contexts
    alive,
  - [x] publishing command/action availability snapshots now refreshes the authoritative
    window-level key-context stack, so cross-surface gating cannot keep stale `keyctx.*` state
    alive after rebuild,
  - [x] declarative rebuild commit points (`render_root(...)` / `render_dismissible_root_with_hooks(...)`)
    now republish authoritative window input/key-context/action-availability snapshots after tree
    GC/root reuse, so later same-frame surfaces cannot keep consuming stale window services,
  - [x] raw imperative `UiTree` mutations now have an explicit authoritative window-snapshot
    commit surface, so same-frame consumers can republish input/key-context/action-availability
    after retained-state changes without waiting for rebuild/dispatch/paint,
  - [x] best-effort `WindowInputContextService` consumers now overlay the authoritative
    `WindowCommandAvailabilityService` before using `edit.can_*` / `router.can_*`, so stale
    published input snapshots cannot suppress cross-surface gating or shortcut lookup,
  - [x] add a source-policy guardrail so future first-party code cannot reintroduce raw
    `WindowInputContextService` command-availability drift outside the runtime owner files,
  - [x] audit remaining raw `WindowInputContextService` readers and confirm they are limited to
    runtime ownership, diagnostics/debug surfaces, or text-boundary/IME reads,
  - [x] remaining child-list mutation helpers now route through the same authoritative structural
    contract; `add_child(...)` reparents by severing old parent edges, avoiding duplicate child
    edges, and delegating the structural write to `set_children(...)`,
  - [x] reparent cleanup now respects the old parent's structural child-write policy
    (`Standard` vs `Barrier`) instead of guessing from the new write path, so stale child edges
    from cached/runtime wrapper transitions cannot force the wrong detach semantics,
  - [x] contained cache-root dirty markers now align with main-pass layout consumption and
    descendant-truncated contained relayout scheduling,
  - [x] subtree dirty aggregation bookkeeping.
- [x] Add/extend unit tests to cover:
  - [x] barrier relayout sets `subtree_layout_dirty_count` consistently,
  - [x] barrier same-children clean remount stays no-op,
  - [x] barrier same-children dirty descendant converges via contained relayout,
  - [x] descendant layout invalidation under a contained cache root stays layout-only but still
    schedules contained relayout,
  - [x] same-children parent repair reconnects detached descendant layout for normal and mount-time
    child-list mutation helpers,
  - [x] layer-root replacement clears detached interaction state without clearing still-active
    overlay interaction state,
  - [x] pending shortcut continuation drops stale key-contexts after root replacement,
  - [x] cross-surface command gating refreshes stale key-context snapshots when action
    availability is republished,
  - [x] declarative rebuild republished window input snapshots before the next paint,
  - [x] declarative rebuild republished window key-context snapshots before later same-frame
    consumers read them,
  - [x] declarative rebuild republished widget command availability before the next explicit
    publish/dispatch boundary,
  - [x] imperative raw tree mutation only refreshes window input/key-context/action-availability
    after an explicit window snapshot commit,
  - [x] best-effort input-context readers inherit authoritative command availability over stale or
    fallback published snapshots,
  - [x] `add_child(...)` reparents without stale child edges and no-ops when already attached once,
  - [x] barrier-parent reparent cleanup keeps the old parent on contained-relayout semantics,
  - [x] barrier-to-barrier reparent cleanup removes stale edges without bubbling ancestor relayout,
  - [x] scroll handle revision-only bumps stay classified correctly,
  - [x] scroll handle invalidation ignores detached same-frame stale bindings,
  - [x] scroll handle registry dedupes same-frame duplicate element bindings,
  - [x] event-time scroll handle invalidation resolves authoritative live bindings across layers,
  - [x] explicit scroll-target invalidation resolves authoritative live target nodes.

## Wheel/trackpad delta coalescing

- [ ] Decide coalescing layer:
  - [ ] runner/platform (preferred),
  - [ ] UI core (fallback).
- [x] Implement behind a runtime knob (opt-in) with a clear default.
  - [x] Native (winit): `FRET_WINIT_COALESCE_WHEEL=1` (coalesce consecutive wheel events).
- [x] Add a max-abs delta guardrail for a single coalesced wheel event (still needs perf validation on VirtualList):
  - `FRET_WINIT_COALESCE_WHEEL_MAX_ABS_PX` (default: `120`)
- [x] Implement frame-boundary buffering in the desktop runner (deliver ≤ 1 wheel per frame when enabled):
  - `crates/fret-launch/src/runner/desktop/runner/app_handler.rs`
  - `crates/fret-launch/src/runner/desktop/runner/window.rs`
- [x] Add a runner-level “same-frame wheel burst” regression gate:
  - Script: `tools/diag-scripts/ui-gallery/scroll-area/ui-gallery-wheel-burst-coalescing.json`
  - Suite redirect: `tools/diag-scripts/suites/diag-hardening-smoke/ui-gallery-wheel-burst-coalescing.json`
  - Post-run check: `check.wheel_events_max_per_frame.json`
- [x] Collect repeatable perf evidence (repeat=11, warmup=10):
  - `perf-ui-gallery-scroll-area` (script: `ui-gallery-scroll-area-wheel-torture`)
    - OFF (`FRET_WINIT_COALESCE_WHEEL=0`):
      - p50/p95 `total/layout/solve` us: `30777/46060` / `29402/43910` / `3072/4510`
      - worst bundle: `target/fret-perf-scroll-area-coalesce-off-r11/1772509265134/bundle.json`
      - log: `target/perf-logs/scroll-area-coalesce-off-r11.log`
    - ON (`FRET_WINIT_COALESCE_WHEEL=1`):
      - p50/p95 `total/layout/solve` us: `28134/29352` / `26956/28203` / `2859/3036`
      - worst bundle: `target/fret-perf-scroll-area-coalesce-on-r11/1772509316761/bundle.json`
      - log: `target/perf-logs/scroll-area-coalesce-on-r11.log`
  - `perf-ui-gallery-virtual-list` (script: `ui-gallery-virtual-list-wheel-torture`)
    - OFF (`FRET_WINIT_COALESCE_WHEEL=0`):
      - p50/p95 `total/layout/solve` us: `10910/11393` / `10180/10595` / `2996/3363`
      - worst bundle: `target/fret-perf-vlist-coalesce-off-r11/1772509365874/bundle.json`
      - log: `target/perf-logs/virtual-list-coalesce-off-r11.log`
    - ON (`FRET_WINIT_COALESCE_WHEEL=1`):
      - p50/p95 `total/layout/solve` us: `11870/18175` / `11185/17468` / `3437/5012`
      - worst bundle: `target/fret-perf-vlist-coalesce-on-r11/1772509420507/bundle.json`
      - log: `target/perf-logs/virtual-list-coalesce-on-r11.log`
- [x] Re-run repeat=11 perf after adding the max-abs cap (2026-03-03):
  - Goal: keep `perf-ui-gallery-scroll-area` improved, remove `perf-ui-gallery-virtual-list` p95 regression.
  - Result (pre frame-boundary buffering): `cap=120` still shows high variance for VirtualList; see the “Full rerun (pre frame-boundary buffering)” section below.
  - Result (after frame-boundary buffering): `cap=120` is stable for both scripts; see the “Frame-boundary buffering rerun” section below.

### Rerun (2026-03-03) — max-abs cap default (`120`)

Short rerun (repeat=5, warmup=10) to sanity-check the new default cap behavior:

- `perf-ui-gallery-virtual-list`
  - OFF (`FRET_WINIT_COALESCE_WHEEL=0`):
    - p50/p95 `total` us: `10963/11109`
    - worst bundle: `target/fret-diag/1772514040891/bundle.json`
    - log: `target/perf-logs/virtual-list-coalesce-off-current-r5.log`
  - ON (`FRET_WINIT_COALESCE_WHEEL=1`, default cap `120`):
    - p50/p95 `total` us: `10424/11102`
    - worst bundle: `target/fret-diag/1772513954382/bundle.json`
    - log: `target/perf-logs/virtual-list-coalesce-on-cap120-r5.log`

- `perf-ui-gallery-scroll-area`
  - OFF (`FRET_WINIT_COALESCE_WHEEL=0`):
    - p50/p95 `total` us: `30567/49376`
    - worst bundle: `target/fret-diag/1772513830516/bundle.json`
    - log: `target/perf-logs/scroll-area-coalesce-off-current-r5.log`
  - ON (`FRET_WINIT_COALESCE_WHEEL=1`, default cap `120`):
    - p50/p95 `total` us: `28054/29181`
    - worst bundle: `target/fret-diag/1772513987367/bundle.json`
    - log: `target/perf-logs/scroll-area-coalesce-on-cap120-r5.log`
- [ ] Add diag evidence:
  - [x] stress wheel in a scroll area (`ui-gallery-scroll-area-wheel-torture`),
  - [x] stress wheel in a virtual list (`ui-gallery-virtual-list-wheel-torture`),
  - [x] nested scrollable case (inner X should not consume Y wheel: `ui-gallery-scroll-area-nested-scroll-routing`).

Full rerun (repeat=11, warmup=10) with explicit env overrides (2026-03-03, pre frame-boundary buffering):

- `perf-ui-gallery-virtual-list` (script: `ui-gallery-virtual-list-wheel-torture`)
  - OFF (`FRET_WINIT_COALESCE_WHEEL=0`):
    - p50/p95 `total/layout/solve` us: `10619/12213` / `9945/11573` / `2987/3319`
    - worst bundle: `target/fret-diag/1772517019308/bundle.json`
    - log: `target/perf-logs/virtual-list-coalesce-off-r11-20260303c.log`
  - ON (`FRET_WINIT_COALESCE_WHEEL=1`, cap `120`):
    - p50/p95 `total/layout/solve` us: `11611/24223` / `10978/22757` / `3258/5983`
    - worst bundle: `target/fret-diag/1772517054654/bundle.json`
    - log: `target/perf-logs/virtual-list-coalesce-on-cap120-r11-20260303c.log`
  - ON (`FRET_WINIT_COALESCE_WHEEL=1`, cap `60`):
    - p50/p95 `total/layout/solve` us: `10872/12343` / `10231/11625` / `3042/3281`
    - worst bundle: `target/fret-diag/1772517987201/bundle.json`
    - log: `target/perf-logs/virtual-list-coalesce-on-cap60-r11-20260303c.log`

- `perf-ui-gallery-scroll-area` (script: `ui-gallery-scroll-area-wheel-torture`)
  - OFF (`FRET_WINIT_COALESCE_WHEEL=0`):
    - p50/p95 `total/layout/solve` us: `27674/28643` / `26613/27521` / `2844/3019`
    - worst bundle: `target/fret-diag/1772517184852/bundle.json`
    - log: `target/perf-logs/scroll-area-coalesce-off-r11-20260303c.log`
  - ON (`FRET_WINIT_COALESCE_WHEEL=1`, cap `120`):
    - p50/p95 `total/layout/solve` us: `27873/28904` / `26766/27801` / `2859/3254`
    - worst bundle: `target/fret-diag/1772517215826/bundle.json`
    - log: `target/perf-logs/scroll-area-coalesce-on-cap120-r11-20260303c.log`
  - ON (`FRET_WINIT_COALESCE_WHEEL=1`, cap `60`):
    - p50/p95 `total/layout/solve` us: `29862/32033` / `28487/30812` / `2965/3549`
    - worst bundle: `target/fret-diag/1772518038237/bundle.json`
    - log: `target/perf-logs/scroll-area-coalesce-on-cap60-r11-20260303c.log`

Notes:

- Current evidence suggests the cap is workload-sensitive:
  - `cap=120` is acceptable for `scroll-area` but shows high variance/regression in `virtual-list`.
  - `cap=60` removes the `virtual-list` spikes but regresses `scroll-area` in this torture script.
- Follow-up (partially resolved): frame-boundary buffering makes `cap=120` stable in repeat=11 for both scripts; next step is deciding if/when this becomes default-on across platforms.

Frame-boundary buffering rerun (repeat=11, warmup=10) (2026-03-03):

- `perf-ui-gallery-virtual-list` (script: `ui-gallery-virtual-list-wheel-torture`)
  - OFF (`FRET_WINIT_COALESCE_WHEEL=0`):
    - p50/p95 `total/layout/solve` us: `10927/12140` / `10263/11451` / `2986/3307`
    - worst bundle: `target/fret-diag/1772519046872/bundle.json`
    - log: `target/perf-logs/virtual-list-coalesce-off-frame-r11-20260303.log`
  - ON (`FRET_WINIT_COALESCE_WHEEL=1`, cap `120`):
    - p50/p95 `total/layout/solve` us: `10729/11614` / `10099/10922` / `2985/3187`
    - worst bundle: `target/fret-diag/1772519094741/bundle.json`
    - log: `target/perf-logs/virtual-list-coalesce-on-frame-cap120-r11-20260303.log`

- `perf-ui-gallery-scroll-area` (script: `ui-gallery-scroll-area-wheel-torture`)
  - OFF (`FRET_WINIT_COALESCE_WHEEL=0`):
    - p50/p95 `total/layout/solve` us: `28544/52680` / `27404/50496` / `2855/5707`
    - worst bundle: `target/fret-diag/1772519164488/bundle.json`
    - log: `target/perf-logs/scroll-area-coalesce-off-frame-r11-20260303.log`
  - ON (`FRET_WINIT_COALESCE_WHEEL=1`, cap `120`):
    - p50/p95 `total/layout/solve` us: `29282/31195` / `28203/30055` / `2957/3434`
    - worst bundle: `target/fret-diag/1772519183814/bundle.json`
    - log: `target/perf-logs/scroll-area-coalesce-on-frame-cap120-r11-20260303.log`

## Perf harness plumbing

- [x] Allow `fretboard diag perf perf-ui-gallery-scroll-area` to resolve via the promoted scripts registry:
  - `crates/fret-diag/src/perf_seed_policy.rs`

## Scrollbar drag stability

- [x] Add “drag baseline” to `ScrollbarState` (mechanism-only).
- [x] Update thumb math while dragging to use baseline.
- [x] Add diag script + semantics assertions (`ui-gallery-scrollbar-drag-baseline-content-growth`).

## Extents probing / observation

- [x] Add diag script for “expand at bottom” (pinned extents regression):
  - script: `tools/diag-scripts/ui-gallery/scroll-area/ui-gallery-scroll-area-expand-at-bottom.json`
  - suite redirect: `tools/diag-scripts/suites/diag-hardening-smoke/ui-gallery-scroll-area-expand-at-bottom.json`
  - bundle: `target/fret-diag-scroll-expand-at-bottom-v4/sessions/1772539486117-27536/1772539488297` (2026-03-03)
- [x] Validate post-layout observation budgets:
  - [x] wrapper peel budget hit triggers a probe next frame,
  - [x] deep scan budget hit triggers a probe next frame.
  - test: `crates/fret-ui/src/declarative/host_widget/layout/scrolling.rs` (`scroll_post_layout_observation_budget_hit_schedules_probe_next_frame`)
