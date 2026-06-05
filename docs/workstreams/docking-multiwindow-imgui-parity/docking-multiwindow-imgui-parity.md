# Docking Tear-off (Multi-Window) — ImGui Parity Refactor Workstream (v1)


## Upstream references (non-normative)

This document references optional local checkouts under `repo-ref/` for convenience.
Upstream sources:

- Dear ImGui: https://github.com/ocornut/imgui

See `docs/repo-ref.md` for the optional local snapshot policy and pinned SHAs.
Status: Active execution lane (overview note; authoritative first-open index:
`docs/workstreams/docking-multiwindow-imgui-parity/WORKSTREAM.json`; normative contracts live in ADRs)

This workstream targets **editor-grade docking across multiple OS windows** (tear-off + re-dock),
aiming for Dear ImGui docking branch “multi-viewports” hand-feel parity.

Platform note:

- Lane state / first-open index:
  `docs/workstreams/docking-multiwindow-imgui-parity/WORKSTREAM.json`
- Current baseline audit:
  `docs/workstreams/docking-multiwindow-imgui-parity/M0_BASELINE_AUDIT_2026-04-13.md`
- Latest launched bounded-campaign repair:
  `docs/workstreams/docking-multiwindow-imgui-parity/M14_LAUNCHED_BOUNDED_CAMPAIGN_REPAIR_2026-05-13.md`
- Latest local Wayland-boundary refresh:
  `docs/workstreams/docking-multiwindow-imgui-parity/M15_LOCAL_WAYLAND_BOUNDARY_REFRESH_2026-05-14.md`
- Latest source-drift guard:
  `docs/workstreams/docking-multiwindow-imgui-parity/M16_SOURCE_DRIFT_GUARD_2026-05-14.md`
  (2026-05-15 follow-up also guards the Wayland campaign/script admission contract)
- Latest local Wayland policy-skip matrix:
  `docs/workstreams/docking-multiwindow-imgui-parity/M18_LOCAL_WAYLAND_POLICY_SKIP_MATRIX_2026-05-16.md`
  (proves every non-qualifying Wayland campaign admission predicate stops at `skipped_policy`
  before script execution)
- Latest local Wayland guard refresh:
  `docs/workstreams/docking-multiwindow-imgui-parity/M22_LOCAL_WAYLAND_GUARD_REFRESH_2026-05-31.md`
  (reruns local source, policy-skip, campaign validate, capability posture, and fallback gates
  while preserving the Wayland acceptance boundary)
- Latest docking runtime owner split:
  `docs/workstreams/docking-multiwindow-imgui-parity/M23_DOCKING_RUNTIME_TEAR_OFF_OWNER_SPLIT_2026-05-31.md`
  (moves tear-off registry and pending state into a private runtime child owner without changing
  fallback behavior or the Wayland acceptance boundary)
- Latest docking runtime fallback owner split:
  `docs/workstreams/docking-multiwindow-imgui-parity/M24_DOCKING_RUNTIME_IN_WINDOW_OWNER_SPLIT_2026-05-31.md`
  (moves in-window fallback and recovery geometry into a private runtime child owner without
  changing public recovery hook paths or the Wayland acceptance boundary)
- Latest docking runtime create-request owner split:
  `docs/workstreams/docking-multiwindow-imgui-parity/M25_DOCKING_RUNTIME_TEAR_OFF_CREATE_REQUEST_OWNER_SPLIT_2026-06-01.md`
  (moves DockFloating OS-window create request construction into the private tear-off owner without
  changing in-window fallback behavior or the Wayland acceptance boundary)
- Latest docking runtime cancellation owner split:
  `docs/workstreams/docking-multiwindow-imgui-parity/M26_DOCKING_RUNTIME_TEAR_OFF_CANCELLATION_OWNER_SPLIT_2026-06-01.md`
  (moves pending tear-off cancellation policy into the private tear-off owner without changing
  created-window completion behavior or the Wayland acceptance boundary)
- Latest docking runtime window-created owner split:
  `docs/workstreams/docking-multiwindow-imgui-parity/M27_DOCKING_RUNTIME_WINDOW_CREATED_OWNER_SPLIT_2026-06-02.md`
  (moves created-window completion, drag-source remapping, and registry registration into a
  private runtime child owner without changing create/cancel/fallback behavior or the Wayland
  acceptance boundary)
- Latest docking runtime before-close owner split:
  `docs/workstreams/docking-multiwindow-imgui-parity/M28_DOCKING_RUNTIME_BEFORE_CLOSE_OWNER_SPLIT_2026-06-02.md`
  (moves DockFloating OS close merge-back policy into a private runtime child owner without
  changing create/cancel/fallback/window-created behavior or the Wayland acceptance boundary)
- Latest docking runtime auto-close owner split:
  `docs/workstreams/docking-multiwindow-imgui-parity/M29_DOCKING_RUNTIME_AUTO_CLOSE_OWNER_SPLIT_2026-06-02.md`
  (moves empty DockFloating OS-window scanning and close effects into a private runtime child owner
  without changing graph mutation, invalidation, or the Wayland acceptance boundary)
- Latest docking runtime request owner split:
  `docs/workstreams/docking-multiwindow-imgui-parity/M30_DOCKING_RUNTIME_REQUEST_OWNER_SPLIT_2026-06-02.md`
  (moves DockFloating request-to-new-window capability fallback, pending correlation, and create
  request trigger policy into a private runtime child owner without changing public hook behavior)
- Latest docking runtime layout invalidation owner split:
  `docs/workstreams/docking-multiwindow-imgui-parity/M31_DOCKING_RUNTIME_LAYOUT_INVALIDATION_OWNER_SPLIT_2026-06-02.md`
  (moves DockOp post-mutation viewport cleanup and invalidation into a private runtime child owner
  without changing graph mutation or the Wayland acceptance boundary)
- Latest docking runtime apply owner split:
  `docs/workstreams/docking-multiwindow-imgui-parity/M44_DOCKING_RUNTIME_APPLY_OWNER_SPLIT_2026-06-02.md`
  (moves the ordinary DockOp mutation/logging/auto-close orchestration into a private runtime child
  owner without changing request handling or the Wayland acceptance boundary)
- Latest docking runtime test owner split:
  `docs/workstreams/docking-multiwindow-imgui-parity/M32_DOCKING_RUNTIME_TEST_OWNER_SPLIT_2026-06-02.md`
  (moves focused runtime regression coverage into a private runtime child owner without changing
  runtime behavior or the Wayland acceptance boundary)
- Latest docking declarative tab paint-state owner split:
  `docs/workstreams/docking-multiwindow-imgui-parity/M33_DOCKING_DECLARATIVE_TAB_PAINT_STATE_OWNER_SPLIT_2026-06-02.md`
  (moves tab hover/menu paint-state projection into a private declarative child owner without
  changing docking render behavior or the Wayland acceptance boundary)
- Latest docking declarative event owner split:
  `docs/workstreams/docking-multiwindow-imgui-parity/M45_DOCKING_DECLARATIVE_EVENT_OWNER_SPLIT_2026-06-02.md`
  (moves managed-surface event orchestration into a private declarative child owner without
  changing docking interaction behavior or the Wayland acceptance boundary)
- Latest docking declarative InternalDrag event owner split:
  `docs/workstreams/docking-multiwindow-imgui-parity/M46_DOCKING_DECLARATIVE_INTERNAL_DRAG_EVENT_OWNER_SPLIT_2026-06-02.md`
  (moves InternalDrag hover/drop/cancel routing into a private event child owner without changing
  docking drag behavior or the Wayland acceptance boundary)
- Latest docking declarative PointerDown event owner split:
  `docs/workstreams/docking-multiwindow-imgui-parity/M47_DOCKING_DECLARATIVE_POINTER_DOWN_EVENT_OWNER_SPLIT_2026-06-02.md`
  (moves PointerDown overflow/floating/split/viewport/tab-drag activation into a private event child
  owner without changing docking interaction behavior or the Wayland acceptance boundary)
- Latest docking declarative PointerUp event owner split:
  `docs/workstreams/docking-multiwindow-imgui-parity/M48_DOCKING_DECLARATIVE_POINTER_UP_EVENT_OWNER_SPLIT_2026-06-02.md`
  (moves PointerUp viewport/floating/split/tab release commits into a private event child owner
  without changing docking interaction behavior or the Wayland acceptance boundary)
- Latest docking declarative PointerCancel event owner split:
  `docs/workstreams/docking-multiwindow-imgui-parity/M49_DOCKING_DECLARATIVE_POINTER_CANCEL_EVENT_OWNER_SPLIT_2026-06-02.md`
  (moves PointerCancel viewport/tab/floating cleanup into a private event child owner without
  changing docking interaction behavior or the Wayland acceptance boundary)
- Latest docking declarative PointerMove event owner split:
  `docs/workstreams/docking-multiwindow-imgui-parity/M50_DOCKING_DECLARATIVE_POINTER_MOVE_EVENT_OWNER_SPLIT_2026-06-02.md`
  (moves PointerMove viewport/divider/floating/pending-drag/hover/cursor behavior into a private
  event child owner without changing docking interaction behavior or the Wayland acceptance
  boundary)
- Latest docking declarative PointerWheel event owner split:
  `docs/workstreams/docking-multiwindow-imgui-parity/M51_DOCKING_DECLARATIVE_POINTER_WHEEL_EVENT_OWNER_SPLIT_2026-06-02.md`
  (moves PointerWheel overflow-menu and tab-strip scrolling into a private event child owner
  without changing docking interaction behavior or the Wayland acceptance boundary)
- Latest docking declarative PointerMove hover/cursor owner split:
  `docs/workstreams/docking-multiwindow-imgui-parity/M52_DOCKING_DECLARATIVE_POINTER_MOVE_HOVER_OWNER_SPLIT_2026-06-02.md`
  (moves PointerMove split-handle, floating, tab, overflow-menu, cursor, and redraw projection
  into a private hover owner without changing docking interaction behavior or the Wayland
  acceptance boundary)
- Latest docking declarative PointerMove viewport-capture owner split:
  `docs/workstreams/docking-multiwindow-imgui-parity/M53_DOCKING_DECLARATIVE_POINTER_MOVE_VIEWPORT_CAPTURE_OWNER_SPLIT_2026-06-02.md`
  (moves PointerMove viewport capture forwarding, right-button drag movement tracking, and
  same-window capture suppression into a private viewport-capture owner without changing docking
  interaction behavior or the Wayland acceptance boundary)
- Latest docking declarative PointerMove divider-drag owner split:
  `docs/workstreams/docking-multiwindow-imgui-parity/M54_DOCKING_DECLARATIVE_POINTER_MOVE_DIVIDER_DRAG_OWNER_SPLIT_2026-06-02.md`
  (moves PointerMove split-handle resize handling, fraction updates, layout invalidation, and
  propagation stop into a private divider-drag owner without changing docking interaction behavior
  or the Wayland acceptance boundary)
- Latest docking declarative PointerMove floating-drag owner split:
  `docs/workstreams/docking-multiwindow-imgui-parity/M55_DOCKING_DECLARATIVE_POINTER_MOVE_FLOATING_DRAG_OWNER_SPLIT_2026-06-02.md`
  (moves PointerMove floating title-bar movement, in-window rect updates, drag preview hover
  resolution, and drag-state persistence into a private floating-drag owner without changing
  docking interaction behavior or the Wayland acceptance boundary)
- Latest docking declarative PointerMove pending panel drag owner split:
  `docs/workstreams/docking-multiwindow-imgui-parity/M56_DOCKING_DECLARATIVE_POINTER_MOVE_PENDING_PANEL_DRAG_OWNER_SPLIT_2026-06-02.md`
  (moves PointerMove pending panel drag activation, hover clearing, capture release, and
  propagation stop into a private pending-panel-drag owner without changing docking interaction
  behavior or the Wayland acceptance boundary)
- Latest docking declarative PointerMove pending tabs-group drag owner split:
  `docs/workstreams/docking-multiwindow-imgui-parity/M57_DOCKING_DECLARATIVE_POINTER_MOVE_PENDING_TABS_GROUP_DRAG_OWNER_SPLIT_2026-06-02.md`
  (moves PointerMove pending tabs-group drag activation, hover clearing, capture release, and
  propagation stop into a private pending-tabs-group-drag owner without changing docking
  interaction behavior or the Wayland acceptance boundary)
- Latest docking declarative interaction type owner split:
  `docs/workstreams/docking-multiwindow-imgui-parity/M58_DOCKING_DECLARATIVE_INTERACTION_TYPE_OWNER_SPLIT_2026-06-03.md`
  (moves declarative docking interaction record types into a private `interaction/types.rs` owner
  without changing service methods, caller paths, field access, or the Wayland acceptance boundary)
- Latest docking declarative interaction drag-session owner split:
  `docs/workstreams/docking-multiwindow-imgui-parity/M66_DOCKING_DECLARATIVE_INTERACTION_DRAG_SESSION_OWNER_SPLIT_2026-06-03.md`
  (moves drag/capture session map helpers into `interaction/drag_sessions.rs` without changing
  sibling event call paths, session cleanup behavior, or the Wayland acceptance boundary)
- Latest runner DockFloating follow owner split:
  `docs/workstreams/docking-multiwindow-imgui-parity/M59_RUNNER_DOCKING_FOLLOW_OWNER_SPLIT_2026-06-03.md`
  (moves desktop runner DockFloating follow movement and transparent-payload style patching into
  a private `docking/follow.rs` owner without changing caller paths, follow behavior, or the
  Wayland acceptance boundary)
- Latest runner dock-drag pointer/poll-up owner split:
  `docs/workstreams/docking-multiwindow-imgui-parity/M60_RUNNER_DOCKING_POINTER_POLL_UP_OWNER_SPLIT_2026-06-03.md`
  (moves dock-drag pointer discovery/capture-cancel into `docking/pointer.rs` and platform
  release-outside poll-up fallbacks into `docking/poll_up.rs` without changing caller paths,
  drop routing, follow-stop cleanup, or the Wayland acceptance boundary)
- Latest runner platform capability owner split:
  `docs/workstreams/docking-multiwindow-imgui-parity/M61_RUNNER_PLATFORM_CAPABILITY_OWNER_SPLIT_2026-06-03.md`
  (moves desktop runner platform capability posture, including Linux Wayland degradation and
  effective-capability clamping, into `platform_capabilities.rs` without changing caller paths,
  capability values, or the Wayland acceptance boundary)
- Latest runner DockFloating create owner split:
  `docs/workstreams/docking-multiwindow-imgui-parity/M62_RUNNER_DOCKING_CREATE_OWNER_SPLIT_2026-06-03.md`
  (moves DockFloating/DockRestore post-create registration, placement refinement, follow
  initialization, and deferred front enqueueing into `docking/create.rs` without changing
  `WindowRequest::Create` ordering or the Wayland acceptance boundary)
- Latest runner window style owner split:
  `docs/workstreams/docking-multiwindow-imgui-parity/M63_RUNNER_WINDOW_STYLE_OWNER_SPLIT_2026-06-03.md`
  (moves `WindowRequest::SetStyle` platform application, style diagnostics, composited-alpha
  reconfiguration, and DockFloating transparent-payload follow state into `window_style.rs` without
  changing effect ordering or the Wayland acceptance boundary)
- Latest runner window close owner split:
  `docs/workstreams/docking-multiwindow-imgui-parity/M64_RUNNER_WINDOW_CLOSE_OWNER_SPLIT_2026-06-03.md`
  (moves `WindowRequest::Close` checked-close, main-window exit, force-close-all, empty-window
  shutdown, and event-loop exit policy into `window_close.rs` without changing close behavior or the
  Wayland acceptance boundary)
- Latest runner window close teardown owner split:
  `docs/workstreams/docking-multiwindow-imgui-parity/M113_RUNNER_WINDOW_CLOSE_TEARDOWN_OWNER_SPLIT_2026-06-04.md`
  (moves checked close and close-window teardown cleanup into `window_close.rs` without changing
  close behavior or the Wayland acceptance boundary)
- Latest runner window geometry owner split:
  `docs/workstreams/docking-multiwindow-imgui-parity/M65_RUNNER_WINDOW_GEOMETRY_OWNER_SPLIT_2026-06-03.md`
  (moves `WindowRequest` visible, size, outer-position, raise, native drag, and native resize
  application into `window_geometry.rs` without changing geometry/chrome behavior or the Wayland
  acceptance boundary)
- Latest runner window request dispatch owner split:
  `docs/workstreams/docking-multiwindow-imgui-parity/M67_RUNNER_WINDOW_REQUEST_DISPATCH_OWNER_SPLIT_2026-06-03.md`
  (moves full `Effect::Window` request dispatch into `window_requests.rs` without changing close,
  create, geometry, style, driver callback, or exit behavior)
- Latest runner window metrics owner split:
  `docs/workstreams/docking-multiwindow-imgui-parity/M68_RUNNER_WINDOW_METRICS_OWNER_SPLIT_2026-06-04.md`
  (moves `Effect::WindowMetricsSetInsets` and `Effect::WindowMetricsSetPreferences` service
  updates into `window_metrics.rs` without changing diagnostic override, known-state, redraw, or
  RAF behavior)
- Latest runner clipboard effects owner split:
  `docs/workstreams/docking-multiwindow-imgui-parity/M69_RUNNER_CLIPBOARD_EFFECTS_OWNER_SPLIT_2026-06-04.md`
  (moves clipboard diagnostics, clipboard read/write, and primary-selection effect handling into
  `clipboard_effects.rs` without changing unavailable, completion-event, capability-gating, or
  platform-error behavior)
- Latest runner incoming-open effects owner split:
  `docs/workstreams/docking-multiwindow-imgui-parity/M70_RUNNER_INCOMING_OPEN_EFFECTS_OWNER_SPLIT_2026-06-04.md`
  (moves diagnostic incoming-open injection, read limit capping, path payload reads, unavailable
  events, and release cleanup into `incoming_open_effects.rs` without changing runtime behavior)
- Latest runner file-transfer effects owner split:
  `docs/workstreams/docking-multiwindow-imgui-parity/M71_RUNNER_FILE_TRANSFER_EFFECTS_OWNER_SPLIT_2026-06-04.md`
  (moves external-drop and file-dialog completion handling into `file_transfer_effects.rs` without
  changing read, selection/cancel, capability-gating, or release behavior)
- Latest runner shell effects owner split:
  `docs/workstreams/docking-multiwindow-imgui-parity/M72_RUNNER_SHELL_EFFECTS_OWNER_SPLIT_2026-06-04.md`
  (moves macOS app-shell actions, open-url capability gating, and share-sheet completion handling
  into `shell_effects.rs` without changing runtime behavior)
- Latest runner image effects owner split:
  `docs/workstreams/docking-multiwindow-imgui-parity/M73_RUNNER_IMAGE_EFFECTS_OWNER_SPLIT_2026-06-04.md`
  (moves image registration, streaming updates, and unregister handling into `image_effects.rs`
  without changing runtime behavior)
- Latest runner text effects owner split:
  `docs/workstreams/docking-multiwindow-imgui-parity/M74_RUNNER_TEXT_EFFECTS_OWNER_SPLIT_2026-06-04.md`
  (moves font asset injection and system-font rescan handling into `text_effects.rs` without
  changing runtime behavior)
- Latest runner system-font rescan owner split:
  `docs/workstreams/docking-multiwindow-imgui-parity/M75_RUNNER_SYSTEM_FONT_RESCAN_OWNER_SPLIT_2026-06-04.md`
  (moves async rescan gating, state publication, result application, resize deferral, and restart
  handling into `text_effects.rs` without changing runtime behavior)
- Latest runner IME effects owner split:
  `docs/workstreams/docking-multiwindow-imgui-parity/M76_RUNNER_IME_EFFECTS_OWNER_SPLIT_2026-06-04.md`
  (moves IME allow, virtual-keyboard request, and cursor-area handling into `ime_effects.rs`
  without changing runtime behavior)
- Latest runner frame effects owner split:
  `docs/workstreams/docking-multiwindow-imgui-parity/M77_RUNNER_FRAME_EFFECTS_OWNER_SPLIT_2026-06-04.md`
  (moves redraw, request-animation-frame, and diagnostic event injection handling into
  `frame_effects.rs` without changing runtime behavior)
- Latest runner timer effects owner split:
  `docs/workstreams/docking-multiwindow-imgui-parity/M78_RUNNER_TIMER_EFFECTS_OWNER_SPLIT_2026-06-04.md`
  (moves set/cancel timer effect handling into the existing `timers.rs` owner without changing
  runtime behavior)
- Latest runner cursor effects owner split:
  `docs/workstreams/docking-multiwindow-imgui-parity/M79_RUNNER_CURSOR_EFFECTS_OWNER_SPLIT_2026-06-04.md`
  (moves cursor icon application into `cursor_effects.rs` without changing runtime behavior)
- Latest runner quit-app effects owner split:
  `docs/workstreams/docking-multiwindow-imgui-parity/M80_RUNNER_QUIT_APP_EFFECTS_OWNER_SPLIT_2026-06-04.md`
  (moves quit-app prompt, dev-state flush, force-close, dispatcher shutdown, and event-loop exit
  handling into `quit_effects.rs` without changing runtime behavior)
- Latest runner command effects owner split:
  `docs/workstreams/docking-multiwindow-imgui-parity/M81_RUNNER_COMMAND_EFFECTS_OWNER_SPLIT_2026-06-04.md`
  (moves window/global command context assembly and driver routing into `command_effects.rs`
  without changing runtime behavior)
- Latest runner menu effects owner split:
  `docs/workstreams/docking-multiwindow-imgui-parity/M82_RUNNER_MENU_EFFECTS_OWNER_SPLIT_2026-06-04.md`
  (moves menu-bar caching and platform menu installation into `menu_effects.rs` without changing
  runtime behavior)
- Latest runner change propagation owner split:
  `docs/workstreams/docking-multiwindow-imgui-parity/M83_RUNNER_CHANGE_PROPAGATION_OWNER_SPLIT_2026-06-04.md`
  (moves model/global change propagation, menu sync, and renderer font/locale sync into
  `change_propagation.rs` without changing runtime behavior)
- Latest runner driver effects owner split:
  `docs/workstreams/docking-multiwindow-imgui-parity/M84_RUNNER_DRIVER_EFFECTS_OWNER_SPLIT_2026-06-04.md`
  (moves viewport-input and Dock effect driver dispatch into `driver_effects.rs` without changing
  runtime behavior)
- Latest runner streaming effects owner split:
  `docs/workstreams/docking-multiwindow-imgui-parity/M85_RUNNER_STREAMING_EFFECTS_OWNER_SPLIT_2026-06-04.md`
  (moves streaming upload preprocessing, ack delivery, diagnostics, and pending redraw wakeups into
  `streaming_effects.rs` without changing runtime behavior)
- Latest runner effect queue owner split:
  `docs/workstreams/docking-multiwindow-imgui-parity/M86_RUNNER_EFFECT_QUEUE_OWNER_SPLIT_2026-06-04.md`
  (moves queued effect dispatch into `effect_queue.rs` without changing runtime behavior)
- Latest runner wheel coalescing owner split:
  `docs/workstreams/docking-multiwindow-imgui-parity/M87_RUNNER_WHEEL_COALESCING_OWNER_SPLIT_2026-06-04.md`
  (moves wheel coalescing math/configuration into `wheel_coalescing.rs` without changing runtime
  behavior)
- Latest runner redraw hitch owner split:
  `docs/workstreams/docking-multiwindow-imgui-parity/M88_RUNNER_REDRAW_HITCH_OWNER_SPLIT_2026-06-04.md`
  (moves redraw hitch diagnostics into `redraw_hitch.rs` without changing runtime behavior)
- Latest runner monitor topology owner split:
  `docs/workstreams/docking-multiwindow-imgui-parity/M89_RUNNER_MONITOR_TOPOLOGY_OWNER_SPLIT_2026-06-04.md`
  (moves monitor topology diagnostics into `monitor_topology.rs` without changing runtime behavior)
- Latest runner surface lifecycle owner split:
  `docs/workstreams/docking-multiwindow-imgui-parity/M90_RUNNER_SURFACE_LIFECYCLE_OWNER_SPLIT_2026-06-04.md`
  (moves deferred surface lifecycle helpers into `surface_lifecycle.rs` without changing runtime
  behavior)
- Latest runner wgpu adapter diagnostics owner split:
  `docs/workstreams/docking-multiwindow-imgui-parity/M91_RUNNER_WGPU_ADAPTER_DIAGNOSTICS_OWNER_SPLIT_2026-06-04.md`
  (moves adapter selection diagnostics into `wgpu_adapter_diagnostics.rs` without changing runtime
  behavior)
- Latest runner renderer bootstrap owner split:
  `docs/workstreams/docking-multiwindow-imgui-parity/M92_RUNNER_RENDERER_BOOTSTRAP_OWNER_SPLIT_2026-06-04.md`
  (moves renderer/caps/font startup installation into `renderer_bootstrap.rs` without changing
  runtime behavior)
- Latest runner factory surface attach owner split:
  `docs/workstreams/docking-multiwindow-imgui-parity/M93_RUNNER_FACTORY_SURFACE_ATTACH_OWNER_SPLIT_2026-06-04.md`
  (moves mobile factory-provided main surface attachment into `surface_lifecycle.rs` without
  changing runtime behavior)
- Latest runner device event owner split:
  `docs/workstreams/docking-multiwindow-imgui-parity/M94_RUNNER_DEVICE_EVENT_OWNER_SPLIT_2026-06-04.md`
  (moves cross-window device-event routing into `device_events.rs` without changing runtime
  behavior)
- Latest runner proxy wake owner split:
  `docs/workstreams/docking-multiwindow-imgui-parity/M95_RUNNER_PROXY_WAKE_OWNER_SPLIT_2026-06-04.md`
  (moves queued proxy event dispatch into `event_loop.rs` without changing runtime behavior)
- Latest runner surface lifecycle hook owner split:
  `docs/workstreams/docking-multiwindow-imgui-parity/M96_RUNNER_SURFACE_LIFECYCLE_HOOK_OWNER_SPLIT_2026-06-04.md`
  (moves destroy/resume/suspend lifecycle hook bodies into `surface_lifecycle.rs` without changing
  runtime behavior)
- Latest runner about-to-wait control-flow owner split:
  `docs/workstreams/docking-multiwindow-imgui-parity/M97_RUNNER_ABOUT_TO_WAIT_CONTROL_FLOW_OWNER_SPLIT_2026-06-04.md`
  (moves pending-front, timer, dispatcher, RAF, and final control-flow scheduling into
  `event_loop.rs` without changing runtime behavior)
- Latest runner about-to-wait internal drag poll owner split:
  `docs/workstreams/docking-multiwindow-imgui-parity/M98_RUNNER_ABOUT_TO_WAIT_INTERNAL_DRAG_POLL_OWNER_SPLIT_2026-06-04.md`
  (moves pre-turn internal drag polling into `device_events.rs` without changing runtime behavior)
- Latest runner about-to-wait dock follow stop owner split:
  `docs/workstreams/docking-multiwindow-imgui-parity/M99_RUNNER_ABOUT_TO_WAIT_DOCK_FOLLOW_STOP_OWNER_SPLIT_2026-06-04.md`
  (moves idle DockFloating follow-stop checks into `docking/follow.rs` without changing runtime
  behavior)
- Latest runner about-to-wait dock released-outside fallback owner split:
  `docs/workstreams/docking-multiwindow-imgui-parity/M100_RUNNER_ABOUT_TO_WAIT_DOCK_RELEASED_OUTSIDE_FALLBACK_OWNER_SPLIT_2026-06-04.md`
  (moves platform released-outside fallback scheduling into `docking/poll_up.rs` without changing
  runtime behavior)
- Latest runner about-to-wait turn bookkeeping owner split:
  `docs/workstreams/docking-multiwindow-imgui-parity/M101_RUNNER_ABOUT_TO_WAIT_TURN_BOOKKEEPING_OWNER_SPLIT_2026-06-04.md`
  (moves tick-id, turn timestamp, release reset, and environment polling into `event_loop.rs`
  without changing runtime behavior)
- Latest runner about-to-wait window turn accessibility owner split:
  `docs/workstreams/docking-multiwindow-imgui-parity/M102_RUNNER_ABOUT_TO_WAIT_WINDOW_TURN_ACCESSIBILITY_OWNER_SPLIT_2026-06-04.md`
  (moves per-window platform inset projection and accessibility action draining into
  `window_turn.rs` without changing runtime behavior)
- Latest runner about-to-wait mobile surface recreation owner split:
  `docs/workstreams/docking-multiwindow-imgui-parity/M103_RUNNER_ABOUT_TO_WAIT_MOBILE_SURFACE_RECREATION_OWNER_SPLIT_2026-06-04.md`
  (moves Android/iOS missing-surface recreation gating into `surface_lifecycle.rs` without changing
  runtime behavior)
- Latest runner about-to-wait diag screenshot poll owner split:
  `docs/workstreams/docking-multiwindow-imgui-parity/M104_RUNNER_ABOUT_TO_WAIT_DIAG_SCREENSHOT_POLL_OWNER_SPLIT_2026-06-04.md`
  (moves feature-gated screenshot request polling and pending-window redraw/RAF requests into
  `diag_screenshots.rs` without changing runtime behavior)
- Latest runner about-to-wait dev-state observation owner split:
  `docs/workstreams/docking-multiwindow-imgui-parity/M105_RUNNER_ABOUT_TO_WAIT_DEV_STATE_OBSERVATION_OWNER_SPLIT_2026-06-04.md`
  (moves feature-gated desktop dev-state window observation into `dev_state.rs` without changing
  runtime behavior)
- Latest runner about-to-wait preamble owner split:
  `docs/workstreams/docking-multiwindow-imgui-parity/M106_RUNNER_ABOUT_TO_WAIT_PREAMBLE_OWNER_SPLIT_2026-06-04.md`
  (moves the pre-render drain, suspended wait fast path, and monitor topology refresh into
  `event_loop.rs` without changing runtime behavior)
- Latest runner monitor geometry owner split:
  `docs/workstreams/docking-multiwindow-imgui-parity/M107_RUNNER_MONITOR_GEOMETRY_OWNER_SPLIT_2026-06-04.md`
  (moves monitor geometry helpers and outer-position settling into `monitor_topology.rs` without
  changing runtime behavior)
- Latest runner window position owner split:
  `docs/workstreams/docking-multiwindow-imgui-parity/M108_RUNNER_WINDOW_POSITION_OWNER_SPLIT_2026-06-04.md`
  (moves client/screen coordinate helpers and cursor-grab placement into `window_position.rs`
  without changing runtime behavior)
- Latest runner window under-cursor owner split:
  `docs/workstreams/docking-multiwindow-imgui-parity/M109_RUNNER_WINDOW_UNDER_CURSOR_OWNER_SPLIT_2026-06-04.md`
  (moves platform under-cursor lookup and z-order fallback into `window_under_cursor.rs` without
  changing runtime behavior)
- Latest runner window platform owner split:
  `docs/workstreams/docking-multiwindow-imgui-parity/M110_RUNNER_WINDOW_PLATFORM_OWNER_SPLIT_2026-06-04.md`
  (moves platform raise/focus, opacity, hit-test, and background-material helpers into
  `window_platform.rs` without changing runtime behavior)
- Latest runner window front owner split:
  `docs/workstreams/docking-multiwindow-imgui-parity/M111_RUNNER_WINDOW_FRONT_OWNER_SPLIT_2026-06-04.md`
  (moves the pending-front retry queue and deadline processing into `window_front.rs` without
  changing runtime behavior)
- Latest runner surface alpha owner split:
  `docs/workstreams/docking-multiwindow-imgui-parity/M112_RUNNER_SURFACE_ALPHA_OWNER_SPLIT_2026-06-04.md`
  (moves composited-alpha surface configuration into `surface_lifecycle.rs` without changing
  runtime behavior)
- Latest runner window close teardown owner split:
  `docs/workstreams/docking-multiwindow-imgui-parity/M113_RUNNER_WINDOW_CLOSE_TEARDOWN_OWNER_SPLIT_2026-06-04.md`
  (moves close-window teardown, drag cleanup, diagnostics cleanup, and service cleanup into
  `window_close.rs` without changing runtime behavior)
- Latest runner window insert owner split:
  `docs/workstreams/docking-multiwindow-imgui-parity/M114_RUNNER_WINDOW_INSERT_OWNER_SPLIT_2026-06-04.md`
  (moves window insertion bootstrap, metrics/bootstrap diagnostics, registry/menu registration, and
  first redraw bootstrap into `window_insert.rs` without changing runtime behavior)
- Latest runner OS window create owner split:
  `docs/workstreams/docking-multiwindow-imgui-parity/M115_RUNNER_OS_WINDOW_CREATE_OWNER_SPLIT_2026-06-04.md`
  (moves winit OS window attributes, create-time style application, and accessibility bootstrap
  into `window_os_create.rs` without changing runtime behavior)
- Latest runner window create-request owner split:
  `docs/workstreams/docking-multiwindow-imgui-parity/M116_RUNNER_WINDOW_CREATE_REQUEST_OWNER_SPLIT_2026-06-04.md`
  (moves final create-request orchestration into `window_create_request.rs` and removes the
  current `window_lifecycle.rs` source owner without changing runtime behavior; no lifecycle owner)
- Latest runner window external drag owner split:
  `docs/workstreams/docking-multiwindow-imgui-parity/M117_RUNNER_WINDOW_EXTERNAL_DRAG_OWNER_SPLIT_2026-06-04.md`
  (moves the external file drag state machine for enter/move/drop/leave into
  `window_external_drag.rs` while leaving app-handler pointer-event merge orchestration unchanged)
- Latest runner window surface resize owner split:
  `docs/workstreams/docking-multiwindow-imgui-parity/M118_RUNNER_WINDOW_SURFACE_RESIZE_OWNER_SPLIT_2026-06-04.md`
  (moves immediate `WindowEvent::SurfaceResized` handling into `surface_lifecycle.rs` while leaving
  redraw-time pending resize fallback in the application handler)
- Latest runner window pointer move owner split:
  `docs/workstreams/docking-multiwindow-imgui-parity/M119_RUNNER_WINDOW_POINTER_MOVE_OWNER_SPLIT_2026-06-04.md`
  (moves `WindowEvent::PointerMoved` mapping, external drag over delivery, and dock drag move
  reroute handling into `window_pointer_move.rs`)
- Latest runner window pointer button owner split:
  `docs/workstreams/docking-multiwindow-imgui-parity/M120_RUNNER_WINDOW_POINTER_BUTTON_OWNER_SPLIT_2026-06-04.md`
  (moves `WindowEvent::PointerButton` left-release drag cleanup and dock-source Up/Down rerouting
  into `window_pointer_button.rs`)
- Latest runner window state events owner split:
  `docs/workstreams/docking-multiwindow-imgui-parity/M121_RUNNER_WINDOW_STATE_EVENTS_OWNER_SPLIT_2026-06-04.md`
  (moves `WindowEvent::ModifiersChanged`, `WindowEvent::ThemeChanged`, and
  `WindowEvent::Focused` handling into `window_state_events.rs`)
- Latest runner window mapped events owner split:
  `docs/workstreams/docking-multiwindow-imgui-parity/M122_RUNNER_WINDOW_MAPPED_EVENTS_OWNER_SPLIT_2026-06-04.md`
  (moves catchall mapped-event delivery, wheel coalescing, RenderDoc F12 handling, and Escape
  dock-drag cancellation into `window_mapped_events.rs` while leaving redraw-time wheel drain in
  `app_handler.rs`)
- Latest runner window moved events owner split:
  `docs/workstreams/docking-multiwindow-imgui-parity/M123_RUNNER_WINDOW_MOVED_EVENTS_OWNER_SPLIT_2026-06-04.md`
  (moves the macOS `WindowEvent::Moved(..)` hit-test region refresh into
  `window_moved_events.rs` while leaving only cfg-gated dispatch in `app_handler.rs`)
- Latest runner window pre-dispatch events owner split:
  `docs/workstreams/docking-multiwindow-imgui-parity/M124_RUNNER_WINDOW_PRE_DISPATCH_EVENTS_OWNER_SPLIT_2026-06-04.md`
  (moves raw winit event accessibility feed and `FRET_IME_DEBUG` IME logging into
  `window_pre_dispatch_events.rs` while leaving only the pre-dispatch call in `app_handler.rs`)
- Latest runner window redraw accessibility owner split:
  `docs/workstreams/docking-multiwindow-imgui-parity/M125_RUNNER_WINDOW_REDRAW_ACCESSIBILITY_OWNER_SPLIT_2026-06-04.md`
  (moves redraw-time semantics snapshot and accessibility tree update/cache maintenance into
  `window_redraw_accessibility.rs` while leaving only dispatch in `app_handler.rs`)
- Latest runner window redraw text-input owner split:
  `docs/workstreams/docking-multiwindow-imgui-parity/M126_RUNNER_WINDOW_REDRAW_TEXT_INPUT_OWNER_SPLIT_2026-06-04.md`
  (moves redraw-time text-input snapshot/IME synchronization into
  `window_redraw_text_input.rs` while leaving only cfg-gated dispatch in `app_handler.rs`)
- Latest runner window redraw renderer perf owner split:
  `docs/workstreams/docking-multiwindow-imgui-parity/M127_RUNNER_WINDOW_REDRAW_RENDERER_PERF_OWNER_SPLIT_2026-06-04.md`
  (moves redraw-time renderer perf sample publication into `window_redraw_renderer_perf.rs` while
  leaving only dispatch in `app_handler.rs`)
- Latest runner window redraw WGPU hub report owner split:
  `docs/workstreams/docking-multiwindow-imgui-parity/M128_RUNNER_WINDOW_REDRAW_WGPU_HUB_REPORT_OWNER_SPLIT_2026-06-04.md`
  (moves redraw-time WGPU hub report publication into `window_redraw_wgpu_report.rs`)
- Latest runner window redraw WGPU allocator report owner split:
  `docs/workstreams/docking-multiwindow-imgui-parity/M129_RUNNER_WINDOW_REDRAW_WGPU_ALLOCATOR_REPORT_OWNER_SPLIT_2026-06-04.md`
  (moves redraw-time WGPU allocator report publication into
  `window_redraw_wgpu_allocator_report.rs` while leaving only dispatch in `app_handler.rs`)
- Latest runner window redraw text diagnostics owner split:
  `docs/workstreams/docking-multiwindow-imgui-parity/M130_RUNNER_WINDOW_REDRAW_TEXT_DIAGNOSTICS_OWNER_SPLIT_2026-06-04.md`
  (moves redraw-time renderer text diagnostics publication into
  `window_redraw_text_diagnostics.rs` while leaving only dispatch in `app_handler.rs`)
- Latest runner window redraw diag screenshots owner split:
  `docs/workstreams/docking-multiwindow-imgui-parity/M131_RUNNER_WINDOW_REDRAW_DIAG_SCREENSHOTS_OWNER_SPLIT_2026-06-04.md`
  (moves redraw-time diagnostic screenshot capture/readback lifecycle into
  `window_redraw_diag_screenshots.rs` while leaving submit/present orchestration in
  `app_handler.rs`)
- Latest runner window redraw pending-wheel owner split:
  `docs/workstreams/docking-multiwindow-imgui-parity/M132_RUNNER_WINDOW_REDRAW_PENDING_WHEEL_OWNER_SPLIT_2026-06-04.md`
  (moves redraw-time pending wheel drain into `window_redraw_pending_wheel.rs` while leaving only
  dispatch in `app_handler.rs`)
- Latest runner window redraw surface-resize owner split:
  `docs/workstreams/docking-multiwindow-imgui-parity/M133_RUNNER_WINDOW_REDRAW_SURFACE_RESIZE_OWNER_SPLIT_2026-06-04.md`
  (moves redraw-time pending surface resize fallback into `window_redraw_surface_resize.rs` while
  leaving only dispatch in `app_handler.rs`)
- Latest runner window redraw frame-prepare owner split:
  `docs/workstreams/docking-multiwindow-imgui-parity/M134_RUNNER_WINDOW_REDRAW_FRAME_PREPARE_OWNER_SPLIT_2026-06-04.md`
  (moves redraw-time platform frame preparation, bounds projection, and driver
  `gpu_frame_prepare` dispatch into `window_redraw_frame_prepare.rs` while leaving only prepare
  dispatch in `app_handler.rs`)
- Latest runner window redraw render owner split:
  `docs/workstreams/docking-multiwindow-imgui-parity/M135_RUNNER_WINDOW_REDRAW_RENDER_OWNER_SPLIT_2026-06-04.md`
  (moves redraw-time app render dispatch, including `RedrawPhase::Render`, text diagnostics frame
  begin, and app `driver.render(...)` dispatch into `window_redraw_render.rs` while leaving only
  render owner dispatch in `app_handler.rs`)
- Latest runner window redraw record owner split:
  `docs/workstreams/docking-multiwindow-imgui-parity/M136_RUNNER_WINDOW_REDRAW_RECORD_OWNER_SPLIT_2026-06-04.md`
  (moves redraw-time engine frame recording, including `RedrawPhase::Record`,
  `scene_ops`, and `driver.record_engine_frame(...)`, into `window_redraw_record.rs` while leaving
  only record owner dispatch in `app_handler.rs`)
- Latest runner window redraw target-updates owner split:
  `docs/workstreams/docking-multiwindow-imgui-parity/M137_RUNNER_WINDOW_REDRAW_TARGET_UPDATES_OWNER_SPLIT_2026-06-04.md`
  (moves redraw-time render-target update application into `window_redraw_target_updates.rs` while
  leaving only target-updates owner dispatch in `app_handler.rs`)
- Latest runner window redraw present-target owner split:
  `docs/workstreams/docking-multiwindow-imgui-parity/M138_RUNNER_WINDOW_REDRAW_PRESENT_TARGET_OWNER_SPLIT_2026-06-04.md`
  (moves redraw-time present surface target preparation into `window_redraw_present_target.rs`
  while leaving render-scene/submit/present orchestration in `app_handler.rs`)
- Latest runner window redraw render-scene owner split:
  `docs/workstreams/docking-multiwindow-imgui-parity/M139_RUNNER_WINDOW_REDRAW_RENDER_SCENE_OWNER_SPLIT_2026-06-04.md`
  (moves redraw-time render-scene command recording into `window_redraw_render_scene.rs` while
  leaving diagnostics/submit/present orchestration in `app_handler.rs`)
- Latest runner window redraw present-submit owner split:
  `docs/workstreams/docking-multiwindow-imgui-parity/M140_RUNNER_WINDOW_REDRAW_PRESENT_SUBMIT_OWNER_SPLIT_2026-06-04.md`
  (moves redraw-time command submission and surface frame presentation into
  `window_redraw_present_submit.rs` while leaving diagnostics/recovery orchestration in
  `app_handler.rs`)
- Latest runner window redraw present-finish owner split:
  `docs/workstreams/docking-multiwindow-imgui-parity/M141_RUNNER_WINDOW_REDRAW_PRESENT_FINISH_OWNER_SPLIT_2026-06-04.md`
  (moves redraw-time successful present finish into `window_redraw_present_finish.rs` while leaving
  recovery/hitch orchestration in `app_handler.rs`)
- Latest runner window redraw present-error owner split:
  `docs/workstreams/docking-multiwindow-imgui-parity/M142_RUNNER_WINDOW_REDRAW_PRESENT_ERROR_OWNER_SPLIT_2026-06-04.md`
  (moves redraw-time present error recovery into `window_redraw_present_error.rs` while leaving
  hitch reporting in `app_handler.rs`)
- Latest runner window redraw hitch-summary owner split:
  `docs/workstreams/docking-multiwindow-imgui-parity/M143_RUNNER_WINDOW_REDRAW_HITCH_SUMMARY_OWNER_SPLIT_2026-06-04.md`
  (moves redraw hitch summary formatting into `window_redraw_hitch_summary.rs` while leaving phase
  timing in existing redraw owners)
- Latest runner window redraw RenderDoc capture owner split:
  `docs/workstreams/docking-multiwindow-imgui-parity/M144_RUNNER_WINDOW_REDRAW_RENDERDOC_CAPTURE_OWNER_SPLIT_2026-06-04.md`
  (moves redraw-time RenderDoc capture begin/end into `window_redraw_renderdoc_capture.rs` while
  leaving initialization and request hotkeys in their existing owners)
- Latest runner window redraw clear-color owner split:
  `docs/workstreams/docking-multiwindow-imgui-parity/M145_RUNNER_WINDOW_REDRAW_CLEAR_COLOR_OWNER_SPLIT_2026-06-04.md`
  (moves transparent-window clear-color selection into `window_redraw_clear_color.rs` while leaving
  render-scene recording in its existing owner)
- Latest runner window redraw webviews owner split:
  `docs/workstreams/docking-multiwindow-imgui-parity/M146_RUNNER_WINDOW_REDRAW_WEBVIEWS_OWNER_SPLIT_2026-06-04.md`
  (moves redraw-time webview snapshot selection and sync dispatch into
  `window_redraw_webviews.rs` while leaving request/event bridging in `webview.rs`)
- Latest runner window redraw post-render diagnostics owner split:
  `docs/workstreams/docking-multiwindow-imgui-parity/M147_RUNNER_WINDOW_REDRAW_POST_RENDER_DIAGNOSTICS_OWNER_SPLIT_2026-06-04.md`
  (moves text diagnostics, renderer perf, and WGPU report dispatch into
  `window_redraw_post_render_diagnostics.rs` while leaving diagnostics internals in existing owners)
- Latest runner window redraw present-capture command owner split:
  `docs/workstreams/docking-multiwindow-imgui-parity/M148_RUNNER_WINDOW_REDRAW_PRESENT_CAPTURE_COMMANDS_OWNER_SPLIT_2026-06-05.md`
  (moves command-buffer assembly plus screenshot capture/readback begin into
  `window_redraw_present_capture_commands.rs` while leaving submit/finish in existing owners)
- Latest runner window redraw present owner split:
  `docs/workstreams/docking-multiwindow-imgui-parity/M149_RUNNER_WINDOW_REDRAW_PRESENT_OWNER_SPLIT_2026-06-05.md`
  (moves redraw-time present-phase orchestration into `window_redraw_present.rs` while leaving
  winit scheduling, present error recovery, and hitch summary orchestration in `app_handler.rs`)
- Latest runner window redraw requested owner split:
  `docs/workstreams/docking-multiwindow-imgui-parity/M150_RUNNER_WINDOW_REDRAW_REQUESTED_OWNER_SPLIT_2026-06-05.md`
  (moves redraw-time frame-drive orchestration into `window_redraw.rs` while leaving
  `WindowEvent::RedrawRequested` dispatch in `app_handler.rs`)
- Latest runner surface bootstrap owner split:
  `docs/workstreams/docking-multiwindow-imgui-parity/M151_RUNNER_SURFACE_BOOTSTRAP_OWNER_SPLIT_2026-06-05.md`
  (moves `ApplicationHandler::can_create_surfaces` lifecycle bootstrap orchestration into
  `surface_bootstrap.rs` while leaving only hook dispatch in `app_handler.rs`; projection marker:
  surface creation lifecycle bootstrap)
- macOS-specific plan: `docs/workstreams/standalone/macos-docking-multiwindow-imgui-parity.md`
- Hovered window contract (reduce heuristics): `docs/workstreams/docking-hovered-window-contract-v1/docking-hovered-window-contract-v1.md`
- Executable TODO tracker: `docs/workstreams/docking-multiwindow-imgui-parity/docking-multiwindow-imgui-parity-todo.md`
- Detailed parity matrix (mechanics + hand feel): `docs/docking-imgui-parity-matrix.md`

## Upstream reference anchors (Dear ImGui)

These non-normative anchors are useful when matching “multi-viewports” hand feel and backend responsibilities:

- Backend responsibilities and the “hovered viewport” problem:
  - `repo-ref/imgui/docs/BACKENDS.md:162` (multi-viewports overview)
  - `repo-ref/imgui/docs/BACKENDS.md:184` (`ImGuiBackendFlags_PlatformHasViewports`)
  - `repo-ref/imgui/docs/BACKENDS.md:185` (`ImGuiBackendFlags_HasMouseHoveredViewport` + ignore `ImGuiViewportFlags_NoInputs`)
  - `repo-ref/imgui/docs/BACKENDS.md:198` (use `io.AddMouseViewportEvent()`; “not as simple as it seems”)
- Canonical flag semantics and API surface:
  - `repo-ref/imgui/imgui.h:1811` (`ImGuiBackendFlags_PlatformHasViewports`)
  - `repo-ref/imgui/imgui.h:1812` (`ImGuiBackendFlags_HasMouseHoveredViewport`)
  - `repo-ref/imgui/imgui.h:2626` (`ImGuiIO::AddMouseViewportEvent`)
  - `repo-ref/imgui/imgui.h:2672` (`MouseHoveredViewport` docs; ignore `NoInputs` improves correctness)
  - `repo-ref/imgui/imgui.h:4060` (`ImGuiViewportFlags_NoInputs`: “mouse pass through so we can drag this window while peeking behind it”)
- Core fallback heuristics when backends can’t provide hovered-viewport reliably:
  - `repo-ref/imgui/imgui.cpp:16621` (backend doesn’t set hovered viewport or doesn’t honor `NoInputs` → search)
  - `repo-ref/imgui/imgui.cpp:16840` (skip `NoInputs` for hovered viewport selection)
- Windows backend example of “peek behind moving window”:
  - `repo-ref/imgui/backends/imgui_impl_win32.cpp:1422` (`NoInputs` set while dragging to detect window behind)
  - `repo-ref/imgui/backends/imgui_impl_win32.cpp:1127` (viewport flags → Win32 window styles: taskbar, top-most, decorations)
- Transparent payload option:
  - `repo-ref/imgui/imgui.h:2515` (`ImGuiIO::ConfigDockingTransparentPayload`)

## Key ImGui mechanics to match (multi-viewports + docking)

This section records the non-normative upstream behavior that most directly informs Fret’s
runner/backend responsibilities. It exists to avoid re-inventing heuristics.

### Hovered viewport selection: backend-first, heuristic fallback

In Dear ImGui, the *preferred* path is backend-provided hovered viewport:

- Backend sets `io.BackendFlags |= ImGuiBackendFlags_HasMouseHoveredViewport` and calls
  `io.AddMouseViewportEvent(viewport_id)`.
- Backend should ideally ignore viewports with `ImGuiViewportFlags_NoInputs` when reporting hovered
  viewport, so a moving “payload” viewport can be skipped (peek-behind).

If the backend cannot provide hovered viewport reliably, core falls back to a heuristic search:

- `ImGui::FindHoveredViewportFromPlatformWindowStack()` scans `g.Viewports`, excludes viewports with
  `ImGuiViewportFlags_NoInputs` and `ImGuiViewportFlags_IsMinimized`, and picks the candidate with
  the highest `LastFocusedStampCount` (a proxy for platform z-order).
  - Anchor: `repo-ref/imgui/imgui.cpp:16642`
- `UpdateViewportsNewFrame()` maintains focus stamps using `Platform_GetWindowFocus` when platform
  windows exist.
  - Anchor: `repo-ref/imgui/imgui.cpp:16678`

Implication for Fret:

- Prefer a platform-backed “window under cursor” provider when `ui.window_hover_detection=Reliable`.
- If the platform cannot supply hovered window, explicitly treat the result as `BestEffort` and use
  a bounded fallback (e.g. “focus stamp + window rect contains point”), mirroring ImGui’s intent.

### “Peek behind moving window”: NoInputs + transparent payload

ImGui’s moving-window paths explicitly mention toggling `NoInputs` after moving has started to
detect what is behind the moving window (useful for docking):

- Anchor: `repo-ref/imgui/imgui.cpp:5538`

Additionally, `ImGuiViewportFlags_NoInputs` is documented as “mouse pass through so we can drag
this window while peeking behind it”:

- Anchor: `repo-ref/imgui/imgui.h:4060`

Implication for Fret:

- The runner should be able to mark the moving DockFloating window as click-through (mouse
  passthrough) during follow, and hover selection should naturally skip it.
- The runner should separately control z-order (e.g. temporary always-on-top) so the moving payload
  stays visible without preventing “peek behind”.

## Scope

In scope:

- multiple **OS windows** (`AppWindowId`) created for docking tear-off,
- cross-window drag hover/drop routing and window-under-cursor selection,
- window ordering / focus behavior during tracked interactions,
- closing/merging semantics (close floating window, close on empty, etc.),
- deterministic arbitration with overlays during dock drags (window-scoped).

Out of scope:

- engine render-target viewports (`RenderTargetId`) and their forwarded input (tracked separately):
  `docs/workstreams/docking-multiviewport-arbitration-v1/docking-multiviewport-arbitration-v1.md`
- external OS file drag-and-drop hover quality (macOS winit limitations; see `docs/known-issues.md`).

## Contract gates (hard boundaries)

- Docking ops + persistence: `docs/adr/0013-docking-ops-and-persistence.md`
- Multi-root overlays: `docs/adr/0011-overlays-and-multi-root.md`
- Cross-window drag sessions: `docs/adr/0041-drag-and-drop-clipboard-and-cross-window-drag-sessions.md`
- Multi-window + DPI semantics: `docs/adr/0017-multi-window-display-and-dpi.md`
- Docking arbitration matrix: `docs/adr/0072-docking-interaction-arbitration-matrix.md`
- Multi-window degradation policy: `docs/adr/0083-multi-window-degradation-policy.md`
- Platform capabilities (runtime): `docs/adr/0054-platform-capabilities-and-portability-matrix.md`
- Window styles / utility windows (future): `docs/adr/0139-window-styles-and-utility-windows.md` (Proposed)

## Parity checklist (platform-agnostic outcomes)

1) **Tear-off creates a new OS window predictably**
   - No flash; reasonable initial placement near cursor/anchor.
2) **New window orders above the source when required**
   - Especially during tracked interactions (menus, drags).
3) **Cross-window hover is stable**
   - Drop hints track the cursor without flicker when windows overlap.
   - When the tear-off window follows the cursor, hover selection can still target the window behind it.
4) **Mouse-up outside any window still completes the drop**
   - Cross-window docking must not “stick” due to missing platform mouse-up delivery.
5) **Re-docking closes empty dock-floating OS windows (P0)**
   - If a dock-floating OS window loses its last panel via re-dock, it should auto-close.
6) **Closing a floating OS window merges content back (P0)**
   - Close should merge panels into a stable target window (usually main) instead of discarding.
7) **Escape cancels dock drag safely (P0)**
   - Cancels drag session, stops tear-off follow, clears internal-drag hover, and does not fight overlays.

## Interaction model (what “ImGui-style” means in practice)

This section exists to avoid a common confusion: in Dear ImGui, “multi-viewports” means **multiple OS
platform windows**, but the gesture people remember as “drag the window title” is actually dragging the
ImGui window/tab title **inside the client area**, not the OS window decoration title bar.

For Fret, the intended UX contract is:

- Re-docking a torn-off panel/tabs is performed by **dragging the tab** (or docking chrome title band)
  inside the dock host widget.
- Dragging the **OS window title bar** of a `DockFloating` window is treated as an OS-managed “move
  the platform window” gesture and is not a docking interaction.

Rationale:

- It keeps the interaction portable and consistent across backends (winit + platform window managers).
- It avoids coupling docking to platform-specific tracked window-move loops.
- It aligns with ADR 0041: docking is an internal-drag/session problem, not a platform window-move problem.

## Baseline architecture (current shape)

Non-normative summary of the current layering:

- Docking UI emits `DockOp` transactions (including `RequestFloatPanelToNewWindow`).
- Docking runtime routes tear-off ops; the private tear-off owner translates supported requests into
  `WindowRequest::Create(CreateWindowKind::DockFloating { .. })`.
- Runner owns OS window lifecycle and cross-window internal-drag routing via screen-space cursor tracking.
- UI runtime enforces overlay/docking arbitration (Escape cancel, overlay suppression, etc.).

Evidence anchors:

- Dock ops vocabulary: `crates/fret-core/src/dock_op.rs`
- Dock graph model: `crates/fret-core/src/dock.rs`
- Docking runtime wiring: `ecosystem/fret-docking/src/runtime.rs`
- Cross-window routing and tear-off follow: `crates/fret-launch/src/runner/desktop/mod.rs`,
  `crates/fret-launch/src/runner/desktop/app_handler.rs`
- Arbitration rules: `docs/adr/0072-docking-interaction-arbitration-matrix.md`
- Optional “transparent payload” (ImGui-style):
  - `FRET_DOCK_TEAROFF_TRANSPARENT_PAYLOAD=1`
  - Runner implementation: `crates/fret-launch/src/runner/desktop/runner/docking/follow.rs` (emits `WindowRequest::SetStyle`),
    `crates/fret-launch/src/runner/desktop/runner/window_requests.rs` (dispatches the request),
    `crates/fret-launch/src/runner/desktop/runner/window_style.rs` (applies style), and
    `crates/fret-launch/src/runner/desktop/runner/window.rs` (`set_window_opacity`, `set_window_hit_test_passthrough_all`)
  - Programmatic switch: `DockingInteractionSettings::transparent_payload_during_follow`
  - Note: the follow loop also requests a temporary `WindowZLevel::AlwaysOnTop` (capability-gated) so the moving window stays
    visible above other app windows. This is applied via `WindowRequest::SetStyle` and patched back to `Normal` when follow stops.

## Cross-platform gaps (common failure modes)

### Gap A: Empty dock-floating OS windows persist after re-dock

The data model is correct (panels moved), but user experience is degraded by empty shells.

Target policy:

- Auto-close a dock-floating OS window when it becomes empty due to docking ops (unless app opts out).

### Gap B: Hovered window selection quality is not capability-modeled

Window-under-cursor selection may be:

- “continuous” and accurate (e.g. absolute cursor APIs),
- “best-effort” (event gaps; lacking z-order; compositor constraints),
- “none” (single-window backends; wasm; sandboxed contexts).

Target:

- model this as a capability quality signal, not as an implicit assumption.

### Gap C: Window ordering/focus behavior differs across platforms

Ordering above the source window during tracked interactions is:

- easy on some platforms,
- difficult or restricted on others,
- requires explicit window-style requests for tool windows in the long run (ADR 0139).

## Platform notes (risk hotspots)

### macOS

See: `docs/workstreams/standalone/macos-docking-multiwindow-imgui-parity.md`

### Windows

Typical hotspots:

- non-client area offsets and initial placement (cursor vs client vs outer bounds),
- top-most ordering interactions (temporary AlwaysOnTop while following),
- mouse capture and raw input differences when leaving windows,
- per-monitor DPI transitions while a drag is active (ADR 0017).

### Linux (X11 / Wayland)

Typical hotspots:

- Wayland limitations on `set_outer_position`, z-order hints, and window-under-cursor semantics,
- lack of a reliable global cursor position under some compositors,
- decoration offsets differ across WMs; initial placement may drift without a stable “outer rect” contract.

Current policy (Wayland):

- Disable docking OS tear-off and prefer in-window floating fallback to keep docking predictable (see
  `docs/workstreams/docking-multiwindow-imgui-parity/docking-multiwindow-imgui-parity-todo.md` `DW-P1-linux-003`).
- Source-policy freeze for that posture now lives in
  `docs/workstreams/docking-multiwindow-imgui-parity/M4_WAYLAND_DEGRADATION_POLICY_2026-04-21.md`.
- Real-host compositor acceptance for that posture now uses
  `docs/workstreams/docking-multiwindow-imgui-parity/M5_WAYLAND_COMPOSITOR_ACCEPTANCE_RUNBOOK_2026-04-21.md`.
- Latest acceptance-open source guard:
  `docs/workstreams/docking-multiwindow-imgui-parity/M19_WAYLAND_ACCEPTANCE_OPEN_GUARD_2026-05-17.md`.
- Latest local Wayland guard refresh:
  `docs/workstreams/docking-multiwindow-imgui-parity/M22_LOCAL_WAYLAND_GUARD_REFRESH_2026-05-31.md`.
- Latest docking runtime window-created owner split:
  `docs/workstreams/docking-multiwindow-imgui-parity/M27_DOCKING_RUNTIME_WINDOW_CREATED_OWNER_SPLIT_2026-06-02.md`.
- Latest docking runtime before-close owner split:
  `docs/workstreams/docking-multiwindow-imgui-parity/M28_DOCKING_RUNTIME_BEFORE_CLOSE_OWNER_SPLIT_2026-06-02.md`.
- Latest docking runtime auto-close owner split:
  `docs/workstreams/docking-multiwindow-imgui-parity/M29_DOCKING_RUNTIME_AUTO_CLOSE_OWNER_SPLIT_2026-06-02.md`.
- Latest docking runtime request owner split:
  `docs/workstreams/docking-multiwindow-imgui-parity/M30_DOCKING_RUNTIME_REQUEST_OWNER_SPLIT_2026-06-02.md`.
- Latest docking runtime layout invalidation owner split:
  `docs/workstreams/docking-multiwindow-imgui-parity/M31_DOCKING_RUNTIME_LAYOUT_INVALIDATION_OWNER_SPLIT_2026-06-02.md`.
- Latest docking runtime test owner split:
  `docs/workstreams/docking-multiwindow-imgui-parity/M32_DOCKING_RUNTIME_TEST_OWNER_SPLIT_2026-06-02.md`.
- Latest docking declarative tab paint-state owner split:
  `docs/workstreams/docking-multiwindow-imgui-parity/M33_DOCKING_DECLARATIVE_TAB_PAINT_STATE_OWNER_SPLIT_2026-06-02.md`.
- Latest docking declarative event owner split:
  `docs/workstreams/docking-multiwindow-imgui-parity/M45_DOCKING_DECLARATIVE_EVENT_OWNER_SPLIT_2026-06-02.md`.
- Latest docking declarative InternalDrag event owner split:
  `docs/workstreams/docking-multiwindow-imgui-parity/M46_DOCKING_DECLARATIVE_INTERNAL_DRAG_EVENT_OWNER_SPLIT_2026-06-02.md`.
- Latest docking declarative PointerDown event owner split:
  `docs/workstreams/docking-multiwindow-imgui-parity/M47_DOCKING_DECLARATIVE_POINTER_DOWN_EVENT_OWNER_SPLIT_2026-06-02.md`.
- Latest docking declarative PointerUp event owner split:
  `docs/workstreams/docking-multiwindow-imgui-parity/M48_DOCKING_DECLARATIVE_POINTER_UP_EVENT_OWNER_SPLIT_2026-06-02.md`.
- Latest docking declarative PointerCancel event owner split:
  `docs/workstreams/docking-multiwindow-imgui-parity/M49_DOCKING_DECLARATIVE_POINTER_CANCEL_EVENT_OWNER_SPLIT_2026-06-02.md`.

## Capabilities (contract) — windowing quality signals (v1)

To avoid platform forks inside widgets, use the windowing quality signals in ADR 0054:

- `ui.window_hover_detection`: `None | BestEffort | Reliable`
- `ui.window_set_outer_position`: `None | BestEffort | Reliable`
- `ui.window_z_level`: `None | BestEffort | Reliable`
- `ui.window.opacity`: `bool` for the DockFloating transparent moving payload posture

Note: capability enum values are spelled `none|best_effort|reliable` in the contract; this workstream uses TitleCase for readability.

These should gate policies such as:

- enabling tear-off follow (manual window movement),
- selecting the “hovered window” under overlap,
- applying AlwaysOnTop during drags,
- applying temporary moving-window opacity during transparent payload drags,
- auto-raising target windows on drop.

Contract source of truth:

- `docs/adr/0054-platform-capabilities-and-portability-matrix.md`

## Diagnostics and regressions

Preferred demos:

- `cargo run -p fret-demo --bin docking_demo`
- `cargo run -p fret-demo --bin docking_arbitration_demo`

Recommended regression suite shape:

- scripted “tear off → hover another window → re-dock” scenarios,
- “release outside any window” scenarios,
- “re-dock last tab closes window” scenarios,
- “OS close merges content back” scenarios.

Platform-specific logging hooks should be documented per platform (macOS already has dedicated logs).
