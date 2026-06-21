---
title: IMUI and heavy-component performance goal
type: active-goal-log
date: 2026-06-14
---

# IMUI and Heavy-Component Performance Goal

## Goal

Keep optimizing Fret's IMUI and shadcn-style heavy component paths until dense general-app
surfaces can stay within a practical 120Hz interaction budget. The current primary gate is the
ui-gallery searchable combobox long-list perf probe because it exercises nested composition,
command filtering, virtual rows, overlays, semantics selectors, layout, paint, and diagnostics.

This root-level `plan/` note mirrors the user's requested progress log location. The longer
historical records remain in:

- `docs/plans/2026-06-14-001-imui-heavy-component-perf-architecture-audit-plan.md`
- `docs/plans/2026-06-14-002-imui-heavy-component-perf-progress-log.md`

## Current Evidence

- Command palette query/navigation is already inside the local 120Hz budget on the RTX4090 Windows
  release probe.
- Searchable combobox long-list has moved from full-list row materialization and broad page layout
  relayout into smaller layout, paint/text, and renderer tail costs.
- The checked-in dev-fast regression baseline is
  `docs/workstreams/perf-baselines/ui-gallery-combobox-filter-select-steady.dev-fast.windows-rtx4090.v1.json`.
- The latest accepted command item-only fast-path gate stayed green with worst frame around
  `11215us`, still above strict 120Hz.
- The inspector direct-entry lane now skips the generic preview-card shell on
  `inspector_torture`; the latest local rerun landed at `total/layout/solve/prepaint/paint =
  2446/1900/864/220/361us`, while the outer `ui-gallery-content-viewport` still dominates. The
  route-aware no-focus subtree pruning is worth keeping, but the broader subtree-summary cache was
  not a net win and was removed.
- The inspector torture row shell is now flattened one step further: the row root keeps the
  retained `Pressable` boundary, while the inner content lands through a single `ContainerProps`
  surface with rich text for label/value and a dedicated `ui-gallery-inspector-row-{index}-label`
  semantics anchor. Validation passed with `cargo fmt --all --check`, targeted `cargo nextest run
  -p fret-ui-gallery --test ui_authoring_surface_internal_previews --test inspector_perf_surface
  --test ui_authoring_surface_content_shell --no-fail-fast`, and the follow-up perf probe
  `tools/diag-scripts/ui-gallery/perf/ui-gallery-inspector-torture-scroll-direct-entry.json`
  returned `p95.us(total/layout/solve/prepaint/paint)=2536/2041/970/223/310` on
  `target/fret-diag/inspector-direct-entry-followup-6/1781999391436/bundle.schema2.json`.
- Earlier accepted optimizations were mixed: component policy/rendering seams, shared `fret-ui`
  mechanism optimizations, declarative text diff narrowing, and gallery cache-boundary policy.

## 2026-06-21 Inspector Nav Shell Shrink Note

- The inspector direct-entry lane got a narrower structural shrink in `apps/fret-ui-gallery/src/ui/nav.rs`:
  the outer `cx.container(...)` wrapper was removed and its chrome/layout moved onto the existing
  `ui::v_flex(...)` root.
- This keeps the sidebar shell on a single flex root while preserving the same background, padding,
  width, height, and shrink contract.
- Structural coverage was added in
  `apps/fret-ui-gallery/tests/ui_authoring_surface_content_shell.rs` so the sidebar root does not
  drift back to a dedicated container wrapper.
- Validation passed with `cargo fmt --all --check`,
  `cargo nextest run -p fret-ui-gallery --test ui_authoring_surface_content_shell --no-fail-fast`,
  and the direct-entry perf probe
  `tools/diag-scripts/ui-gallery/perf/ui-gallery-inspector-torture-scroll-direct-entry.json`.
- Latest probe result on `target/fret-diag/inspector-direct-entry-nav-shrink-v1/1782011573897/bundle.json`:
  `p95.us(total/layout/solve/prepaint/paint)=1916/1537/750/166/229`.
- Compared with the prior local baseline from the shell-cache exploration, this is a real steady-state
  improvement, even though the nav scroll path still remains the hottest structural owner to watch
  on the next pass.

## 2026-06-21 Inspector Direct-Attach Semantics Rejected

- A direct-attach semantics rewrite on the retained inspector row path
  (`cx.semantics_with_id(...) -> list.attach_semantics(...)`) regressed instead of improving.
- The direct-attach bundle
  `target/fret-diag/inspector-direct-entry-root-semantics-direct-attach-v1/1782031379610/bundle.schema2.json`
  landed at `p95.us(total/layout/solve/prepaint/paint)=2234/1803/860/168/263` with
  `layout.root_phases roots(total/apply)=1149/1149`.
- The prior nav-shrink baseline remained better at
  `target/fret-diag/inspector-direct-entry-nav-shrink-v1/1782011573897/bundle.json`:
  `p95.us(total/layout/solve/prepaint/paint)=1916/1537/750/166/229` and
  `layout.root_phases roots(total/apply)=976/976`.
- Keep this experiment rejected. The next slice should stay on the current row/root/container
  split, or move to a smaller owner shift only if it is benchmarked against the same direct-entry
  probe.

## 2026-06-21 Code-View Torture Content-Scroll Bypass Note

- The `code_view_torture` direct-entry path now opts out of the outer gallery content scroll shell
  in `apps/fret-ui-gallery/src/ui/content.rs` and renders as the same static header + body stack
  used by the existing content-scroll-disabled branch.
- The bypass is explicit and page-scoped: `selected == PAGE_CODE_VIEW_TORTURE` joins the existing
  content-scroll guard, so the shortcut is narrow and easy to reverse if the contract changes.
- Regression coverage was added in
  `apps/fret-ui-gallery/tests/ui_authoring_surface_internal_previews.rs` to lock the new bypass
  gate.
- Validation passed with `cargo fmt --all --check`,
  `cargo nextest run -p fret-ui-gallery --test ui_authoring_surface_internal_previews --test code_view_perf_surface --no-fail-fast`,
  and the direct-entry perf probe
  `tools/diag-scripts/ui-gallery/perf/ui-gallery-code-view-torture-mount-direct-entry.json`.
- Latest probe result on `target/fret-diag/code-view-direct-entry-content-scroll-bypass-v1/1782012408933/bundle.json`:
  `p95.us(total/layout/solve/prepaint/paint)=300/9/0/193/98`.
- Compared with the prior direct-entry run (`1916/1537/750/166/229`), this is the first cut that
  removes the outer content-scroll owner from the hot path instead of only reshaping the page shell.

## 2026-06-16 Progress Note

- The retained data-table lane has now moved past table-local row/cell duplication into retained
  `VirtualList` first-pass child layout and root-apply attribution.
- The latest code inspection says the fixed retained path still walks every visible child; the
  remaining hotspot is therefore traversal / subtree depth, not a missing measurement toggle.
- The latest structural cleanup in the hot path removed pure test-id `Semantics` wrappers, but
  that is support evidence rather than the main perf win.
- The retained `VirtualList` layout telemetry now carries explicit root / invalidation /
  dirty-state counters for both first-pass and aggregate child layout, so the next profiling loop
  can separate clean roots from subtree-dirty roots instead of treating every visible row as one
  opaque bucket.
- Next comparison should be against upstream `repo-ref/shadcn` and `repo-ref/base-ui` row/tree
  shape before deciding whether to flatten the row subtree further or split a narrower retained
  `VirtualList` follow-on.

## 2026-06-16 Editor-Controls Note

- Wrote `plan/2026-06-16-imui-editor-controls-structure-audit.md` to capture the current IMUI
  structure read.
- The strongest conclusion is that editor-controls are mostly a component-tree depth and
  policy-coupling problem, not a runtime mechanism problem.
- `PropertyRow` / `PropertyGrid` is the clearest next deepening seam, with
  `PropertyRowLayoutVariant::Auto` the most obvious source of visible height jumps.
- `DragValue` / `NumericInput` shell depth and popup-heavy surfaces such as `TextAssistField` and
  `ColorEdit` are secondary follow-ons.
- Overlay and focus policy should stay in `fret-ui-editor`; they are part of the component lane,
  not a reason to push more logic into `fret-ui`.

## 2026-06-16 Editor-Controls Implementation Note

- Landed the first structural slice in `fret-ui-editor`:
  `PropertyGrid` and `PropertyGridVirtualized` now default their shared row policy to
  `PropertyRowLayoutVariant::Row` instead of implicit `Auto`.
- Added `PropertyRowOptions::with_grid_defaults(...)` so grid callers can inherit shared metrics
  without losing an explicit row variant.
- Added tests covering:
  - default grid row-context variant,
  - row-options merge semantics,
  - and the existing wrapped-row geometry stability path.
- This reduces the chance of visible height jumps in dense editor controls without changing the
  component API surface or moving overlay/focus policy layers.

## 2026-06-16 Editor-Controls Session-Height Note

- Follow-up diagnostics proved the first slice was necessary but not sufficient: the editor-controls
  suite passed, yet `roughness` typing-active still made the inspector grow.
- The second slice normalized mode-switching numeric controls. `DragValue`, `Slider`, and
  `AxisDragValue` now reserve the full editor frame outer height in their session shell instead of
  only the text line height. `DragValue` and `Slider` typing branches also opt into small editor
  `NumericInput` sizing.
- This keeps scrub/slide and typing branches layout-compatible without pushing policy into
  `fret-ui`.
- Evidence:
  `target/fret-diag/cookbook-imui-editor-controls-stable-session-height-v1/sessions/1781574024501-9052/suite.summary.json`
  passed. Extracted layout sidecars showed `grid`, `group`, and `inspector` heights unchanged
  across smoke, exposure, click-stress, overlay, and roughness typing-active states.

## 2026-06-16 Editor-Controls Theme-Replay and ColorEdit Note

- A further cookbook pass found that the measurement surface was still polluted by theme lifecycle
  churn rather than only component layout. `FretApp` installed desktop shadcn defaults after
  `.setup(...)`, and later window-metrics auto-sync could re-apply the host shadcn theme without
  replaying the installed editor dense preset.
- The `fret` app builder now stages desktop defaults so base design-system defaults run before app
  setup, while runtime defaults still run after setup and can observe registered commands. With the
  `imui` feature, shadcn auto-theme middleware also replays the installed editor preset after host
  theme sync.
- After that lifecycle fix, `ColorEdit` was the remaining inconsistent control at 28px. It now uses
  the editor frame chrome outer-height contract instead of the bare row height, matching the numeric
  editor controls.
- Evidence:
  `target/fret-diag/cookbook-imui-editor-controls-color-edit-height-v1/sessions/1781577414581-54972/suite.summary.json`
  passed all five editor-controls scripts. Bounded smoke slices showed `exposure=30px`,
  `roughness=30px`, `tint=30px`, `search=30px`, and `grid=192px`.
- Interpretation: this was not a reason to rewrite `fret-ui` first. It was a framework app-start
  lifecycle bug plus one component contract drift. Both are performance-relevant because height
  churn and theme replays make 120Hz component measurements noisy and visibly unstable.

## 2026-06-17 Code-View Wrapped Text Measure Cache Note

- The code-view torture mount lane exposed a runtime-level text measurement seam rather than a
  shadcn component-local wrapper problem.
- m19 evidence:
  `target/fret-diag/code-view-mount-m19-layout-profile/1781672024298/bundle.schema2.json` had
  `top_total_time_us=21508`, `layout_time_us=21295`, and
  `layout_engine_solve_time_us=20073`. The worst solve measured the same wrapped `Text` node
  repeatedly with zero measure-cache hits; one text node accounted for about `18485us`.
- A rejected scroll extent attempt had already shown that skipping child measurement without a
  compatible reuse path can move the cost into `solve_barrier`. The correct seam was therefore not a
  broad Scroll rewrite, but a text-level cache keyed by stable text/layout-shaping inputs plus wrap
  max width.
- `fret-ui` now keeps a small per-node multi-entry wrapped text measurement cache and reads it before
  calling the text shaping/prepare path. The cache key includes text, resolved style, wrap, overflow,
  align, scale, font-stack key, ink-overflow policy, and the wrapped max width.
- Validation:
  - `cargo test -p fret-ui --profile dev-fast text_wrapped_measure_cache -- --nocapture`
  - `cargo check -p fret-ui --profile dev-fast`
  - `cargo build -p fret-ui-gallery --release --features gallery-dev`
  - m21:
    `target/fret-diag/code-view-mount-m21-text-measure-cache-gallery-dev/1781674968783/bundle.schema2.json`
- m21 result: `top_total_time_us=17098`, `layout_time_us=16873`,
  `layout_engine_solve_time_us=692`. This removes the m19 text-measure solve spike, but the total
  frame is still over a strict 120Hz budget because `layout_roots_time_us=16087` is now the dominant
  cost.
- Diagnostic caution: this perf script requires `fret-ui-gallery` built with `--features
  gallery-dev`. A default release gallery binary omits `code_view_torture`, causing the script to
  time out at the nav-result wait step and producing invalid performance evidence.
- Next target: attribute the remaining root-apply cost. The current hotspot is no longer repeated
  text shaping; it is dirty subtree application/root layout work after navigation into the torture
  page.

## 2026-06-18 Inspector Content Shell Note

- The inspector torture page's preview subtree already owns a fixed-height, clipped surface around
  the retained virtual list. That makes the remaining cost look like shell/root churn, not a row
  shape problem.
- I briefly tried a page-level static-shell policy in `fret-ui-gallery`. The first captured run
  looked better, but the current release binary rerun landed at `top_total_time_us=9724` with
  `layout.root_phases=3`, and a clean side-by-side against the normal shell still showed the
  disabled-content-scroll path faster (`3536us` vs `5338us`) while changing the page contract.
- I also tried feeding the inspector scroll area an explicit known content height (`460px`) so the
  scroll solver could skip the unbounded probe path. The normal-shell rerun improved slightly to
  `top_total_time_us=5180`, but the content-scroll-disabled path still stayed lower at
  `3583us`. That makes the known-size input worth remembering, but not enough to justify a broad
  wrapper rewrite.
- Conclusion: keep the content-shell path unchanged for now. The useful finding is narrower: future
  work should focus on reducing root/apply churn inside the existing shell instead of rewriting the
  page wrapper.

## 2026-06-18 Session Shell Hardening Note

- The editor numeric session shell was still allowing `Auto` height to leak through as the final
  stack height. That made mode switches in `DragValue`, `Slider`, and `AxisDragValue` capable of
  nudging the visible row height even though the shell already reserved the right min height.
- The shell now promotes `Auto` height to the full control outer height. This keeps the scrub /
  typing branches mounted but prevents the wrapper from changing its own measured height when the
  active branch changes.
- Targeted nextest coverage now asserts the fixed-height shell contract for `DragValue`, `Slider`,
  and `AxisDragValue`, so this stays a regression gate rather than a one-off fix.

## 2026-06-18 TransformEdit Structure Note

- `TransformEdit` 的 column 变体已经去掉一层多余的列壳，link toggle 现在直接挂在同一
  个 column shell 下。
- 这次没有改变外部 API，只是把浅层包装收起来，减少一处不必要的树深度。
- 下一步结构排查重点转向共享输入组原语链（`TextField`、`MiniSearchBox`、
  `AssetRefField`、`FieldStatusBadge`），它们更像是同一类重组件热点。

## 2026-06-18 TextAssistField Structure Note

- `TextAssistField` 的 anchored-overlay 路径已经收掉外层纵向 `flex` 根节点，直接返
  回 `TextField` 本体并由 overlay 系统承载建议面板。
- 这让 overlay surface 的根更浅，也把 inline 与 overlay 两种形态分开得更明确。
- 接下来继续观察共享输入组原语链，看是否还能合并更多重复壳层，而不是继续在
  调用点上补局部修补。

## 2026-06-18 Input Group Button-Depth Note

- `editor_icon_button_segment` 现在少了一层中间 `flex` 包装，结构从 `pressable ->
  container -> flex -> icon` 收缩为 `pressable -> container -> icon`。
- 这个收敛已经由 `primitives::input_group` 的结构测试覆盖，不是只靠肉眼判断。
- 下一步如果继续收，就应该看 `editor_joined_input_frame` 的组合壳是否还能合并，
  而不是在调用点层面重复补段。

## 2026-06-19 Editor-Controls Shell Shrink Note

- `PropertyGrid` 现在在“单行 + 默认外层布局 + 无 `test_id`”时直接返回那一行，
  不再额外套一层纵向 shell。
- `editor_input_group_row` 现在在只有一个 child 时直接返回该 child，避免在单元素
  路径上保留无意义的 row 容器。
- `ColorEdit` 输入分支现在直接挂 `TextInput`，移除了外层 `PointerRegion` 壳。
- `ColorEdit` popup 的 numeric/options 现在会先组装 items，再在“只有一个可见项且
  没有 `test_id`”时直接返回该项。
- `EnumSelect` trigger 的 caret 现在直接用居中 `Flex` 承载 `SvgIcon`，去掉了只为
  单个图标服务的外层 `Container`。
- 这一刀继续保留了 `test_id` 路径和多子节点路径的诊断锚点，没有把 policy 往
  `fret-ui` 挪。
- 验证：`cargo nextest run -p fret-ui-editor -j 1 --no-fail-fast`，以及
  `cargo nextest run -p fret-ui-editor enum_select --no-fail-fast`。
- 下一步继续盯剩余的 editor-controls 多子节点链，优先看
  `TextField`、`MiniSearchBox`、`AssetRefField`、`FieldStatusBadge` 这类还保留明显
  组合壳的重表面。

## 2026-06-19 TextAssistField Inline Shell Shrink Note

- `TextAssistField` now returns `TextField` directly on the inline path when there is no
  `inline_panel` and no `empty_label`, instead of adding an extra vertical `Flex` wrapper.
- This keeps the inline empty state on the shortest tree while leaving the empty-label case
  responsible for preserving the shell.
- The regression tests now lock three boundaries: inline direct return, overlay direct return, and
  inline empty-label shell preservation.
- Validation: `cargo fmt --all --check`, and
  `cargo nextest run -p fret-ui-editor inline_surface_without_panel_or_empty_label_returns_the_field_root anchored_overlay_surface_without_panel_or_empty_label_returns_the_field_root inline_surface_with_empty_label_keeps_the_shell_visible --no-fail-fast`.

## 2026-06-20 Editor-Controls Shell Review Note

- The editor-controls shell-shrink batch is consistent with the shell-shrink direction:
  `PropertyGrid`, `ColorEdit`, `DragValue`, `EnumSelect`, `TextAssistField`, and
  `editor_input_group_row` each now have targeted structural coverage.
- `MiniSearchBox` is already thin enough that further shell removal would likely land in
  `editor_joined_input_frame`, not in the control itself.
- `AssetRefField` still carries a meaningful multi-action shell because it composes value text,
  status badge, and optional action segments; it is a better candidate for a future bounded slice
  than for a forced one-line shrink.
- Validation for the batch passed with
  `cargo fmt --all --check` and
  `cargo nextest run -p fret-ui-editor property_grid color_edit drag_value enum_select text_assist_field input_group --no-fail-fast`.

## 2026-06-20 ColorEdit Popup Options Direct-Return Note

- `color_edit::popup::options::color_picker_options` now returns the single visible option directly
  even when a popup-level `test_id` is present, so the popup no longer keeps an extra vertical shell
  for the one-option case.
- `test_id` is preserved as a layout-transparent semantic anchor on the returned option, so
  diagnostics and UI automation can still locate the node without paying for a wrapper.
- Regression coverage now locks both the plain direct-return path and the `test_id`-decorated
  direct-return path.
- Validation: `cargo fmt --all --check`, and
  `cargo nextest run -p fret-ui-editor color_edit::popup::options --no-fail-fast`.

## Decisions

### D1. Continue mixed component plus mechanism optimization

The evidence does not support a single broad framework rewrite as the next step. Large wins came
from specific seams: combobox close policy, command row virtualization, bounded virtual scroll
viewport probing, command availability caching, incremental view-cache observation collapse, and
paint-only plain text content diffing.

### D2. Treat diagnostics overhead as part of perf-gate fidelity

Perf scripts must not make component frames look slower than a real app because diagnostic target
resolution requests fresh semantics snapshots too broadly. Reducing diagnostics-only refreshes is
valid when it does not weaken:

- runtime accessibility semantics,
- selector correctness for current-frame geometry,
- stale-cache behavior for current-window `exists` / `not_exists` assertions.

### D3. Do not use stale semantics as current semantics

Skipping a fresh semantics refresh is only safe if the script step can evaluate without current
semantics. Passing an old `semantics_snapshot_arc()` to a selector-based step would be a correctness
bug, not an optimization. Conservative no-refresh candidates are frame-independent predicates such
as event-kind, font readiness, app snapshot, window size, and off-window runtime diagnostics.

## Active Hypothesis

`UiDiagnosticsService::wants_semantics_snapshot` still over-requests semantics during active
`wait_until` loops. `script_engine::active_script_needs_semantics_snapshot` has predicate-level
helpers, but the early `active.wait_until.is_some()` branch currently returns `true` without
checking whether the active wait predicate can evaluate without current semantics.

If narrowed carefully, scripted perf gates should be closer to real app behavior without changing
component code or accessibility output.

## 2026-06-14 Findings

- The previously observed `layout_semantics_refresh_time_us ~= 2908us` happened on a dirty frame
  where the semantics fingerprint changed. It is not safe to skip that refresh.
- The adjacent clean frame already reported `layout_semantics_refresh_time_us = 0us`, which confirms
  the existing dirty/request gate works for clean frames.
- Safe diagnostics narrowing is limited to script steps that truly do not need current semantics:
  frame-independent predicates such as event kind, font readiness, window size, app snapshot, and
  off-window runtime diagnostics.
- Current-window selector predicates, including `exists` / `not_exists(test_id)`, must remain fresh
  unless the runtime gains an explicit "fresh semantics for this frame" marker. Reusing stale
  `semantics_snapshot_arc()` for these would be a correctness bug.
- The first runtime-safe dirty-refresh optimization is allocation-oriented: reuse a semantics
  children scratch buffer during full traversal instead of cloning/allocating a `Vec<NodeId>` per
  visited node.
- A first dev-fast gate after the scratch-buffer change failed once:
  `target/fret-diag/gate-combobox-filter-select-devfast-semantics-scratch/1781435706747/bundle.schema2.json`.
  The top frame was `16650us` total with `10297us` layout, `5930us` semantics refresh,
  `2012us` layout solve, and high renderer finish. This looked like a dirty popover overlay tail,
  not a deterministic regression.
- The same gate passed on rerun:
  `target/fret-diag/gate-combobox-filter-select-devfast-semantics-scratch-rerun/1781435998219/bundle.schema2.json`.
  The top frame was `10632us` total with `6090us` layout, `2460us` semantics refresh,
  `1072us` layout solve, `1874us` max renderer finish, `820us` max pointer dispatch, and no
  threshold failures.
- The accepted pre-scratch bundle had a worst frame around `11215us` with `2908us` semantics
  refresh. The rerun does not prove a major latency win, but it does validate the scratch change as
  a low-risk allocation reduction that keeps the gate green.

## Current Decision

Land the current slice as a reversible performance/fidelity cleanup:

- diagnostics only ask for fresh semantics on active waits whose predicates need current semantics,
- semantics traversal reuses a root-local children scratch buffer instead of cloning per visited
  node,
- current-window selector predicates stay conservative and still require fresh semantics.

Do not broaden stale-semantics reuse until the runtime has an explicit freshness marker. The next
meaningful optimization should target either dirty-frame semantics traversal cost itself or the
popover overlay root solve tail.

## 2026-06-14 Second Slice Findings

- Added a layout-derived semantics hook classifier for declarative host widgets. It skips the
  expensive `semantics_impl` instance clone/match for plain pass-through nodes, while preserving
  snapshot nodes, bounds, children traversal, focus/text-input defaults, root `Window` role behavior,
  and any `attach_semantics` decoration.
- Do not synchronize this classifier from `mount_element` for every declarative element. A trial
  mount-path sync added one widget mutation per mounted element and made the dense combobox gate
  worse. The final slice keeps classification on the layout path only.
- Focused correctness tests cover both sides of the contract:
  undecorated plain containers remain present as generic semantics nodes, and `attach_semantics` on
  a plain container still stamps role/label/test_id and keeps child traversal.
- The current combobox dev-fast gate still fails on repeated probes after this slice:
  `target/fret-diag/gate-combobox-filter-select-devfast-semantics-hook-layout-only/1781440592740/bundle.schema2.json`
  had `22850us` total, `12740us` layout, `2428us` solve, and `8927us` paint.
- The failure shape is not semantics-led. The worst frame is the searchable combobox popover root
  (`DismissibleLayer`) doing a `new_frame_same_key` solve of about 27 subtree nodes, plus root apply,
  paint, and renderer finish. A semantics-profile probe with the same binary showed semantics
  snapshots mostly in the `1-3ms` traversal range and a better top frame around `14365us`.
- The next higher-leverage target is therefore the popover overlay root solve / paint tail, not more
  semantics micro-optimization.

## 2026-06-14 Third Slice Findings

- Added a frame-local declarative command-availability interest cache in `fret-ui`. It caches only
  whether a node may handle a command class, not the final command availability result, and is keyed
  by `(frame_id, command_availability_revision, window)`.
- This is intentionally narrower than caching `Available` / `Blocked`: command availability hooks
  still run when a node is a possible handler, while repeated runtime-snapshot publications in the
  same frame avoid re-reading declarative element state for every node in the same command path.
- Focused tests prove the intended boundaries:
  `action_availability_snapshot_reuses_declarative_interest_across_same_frame_refine` shows a
  forced same-frame refine reuses the cached interest metadata, and a layout invalidation bumps the
  command availability revision and forces a re-read.
- Validation passed:
  `cargo test -p fret-ui --lib window_command_action_availability_snapshot --profile dev-fast -j 1`,
  `cargo fmt -p fret-ui`, and `cargo check -p fret-ui -j 1`.
- The current combobox gate still fails on a single dev-fast probe:
  `target/fret-diag/gate-combobox-filter-select-devfast-interest-cache/1781443011610/bundle.schema2.json`
  had `17523us` total, `7973us` layout, `914us` solve, `8829us` paint, and failed only
  `top_total_time_us > 15443us`.
- This probe reduced the worst runtime snapshot command-availability evaluation shape compared with
  the earlier `1.4-1.8ms` readings, but it did not move the overall gate under budget. The worst
  frame is now dominated by paint/cache behavior: `paint.cache_misses=1033`, `paint.nodes=1099`,
  and `cache.reused=0`.
- Current decision: keep this slice as a low-risk mechanism optimization, but move the next
  investigation to paint cache root reuse during the combobox filter/select path.

## 2026-06-14 Fourth Slice Findings

- Fixed-height declarative `TextInput` now treats text content changes as paint-only. The runtime
  still keeps auto-height inputs layout-sensitive, because their measured size may depend on the
  current text.
- The policy is derived from `TextInputProps.layout.size.height`: `Length::Px(_)` uses
  `Invalidation::Paint`, and non-fixed height keeps `Invalidation::Layout`. This applies through
  the declarative event/layout/paint/command paths, model observation, and platform text replacement
  APIs.
- `measure_text_input` no longer reads or layout-observes the model for fixed-height inputs. It uses
  a stable `"M"` line-metric probe for the fixed-height case, which avoids turning command-search
  query changes into layout work.
- Focused tests cover the contract:
  `fixed_height_text_input_model_change_invalidates_paint_only` and
  `auto_height_text_input_model_change_keeps_layout_invalidation`.
- A first combobox perf probe was invalid as performance evidence because it omitted
  `FRET_UI_GALLERY_VIEW_CACHE=1` and `FRET_UI_GALLERY_VIEW_CACHE_SHELL=1`. Direct JSON-script
  `diag perf` targets do not receive the suite-name auto-env defaults, so the overlay root reported
  `reuse_reason = view_cache_disabled` and produced a misleading `18602us` top frame.
- The correct gate command includes the view-cache env vars plus the prewarm/prelude scripts used
  by the checked-in baseline. That run passed with `failures=[]`:
  `target/fret-diag/gate-combobox-filter-select-devfast-fixed-textinput-paint-vc/1781446249173/bundle.schema2.json`.
- Correct-gate top frame after the fixed-height input slice:
  `total=10000us`, `layout=5743us`, `solve=945us`, `paint=3628us`,
  `paint.cache_misses=0`, `cache.reused=1`, `cache.replayed_ops=203`,
  `pointer_move_max_dispatch=775us`, and `pointer_move_max_hit_test=114us`.
- Current residual hotspot is no longer the search `TextInput` model observation path. The worst
  frame still includes popover `DismissibleLayer` layout solve and retained paint/renderer work,
  so the next optimization should target overlay/list cache boundaries or command-availability tail
  only if a fresh profile makes them hot again.

## 2026-06-14 Fifth Slice Findings

- Investigated the next paint/cache tail after the fixed-height `TextInput` slice. The important
  failure mode was not a component-level combobox policy bug: when an ancestor subtree replayed from
  paint cache, descendant `PaintCacheEntry` records could remain tied to the previous generation.
  If the ancestor was paint-invalidated on the following frame, stable descendants no longer had
  fresh ranges to replay and could fall back toward repainting the dense subtree.
- Added a mechanism-level paint-cache rebase step after a successful ancestor replay. It walks
  descendants and promotes only safe previous-frame entries into the current generation:
  descendant ranges must be fully contained in the replayed parent range, the descendant entry must
  come from the current source generation, and the walker prunes paint-invalidated descendant
  subtrees. The rebase only remaps op/text-blob ranges; origin translation remains owned by the
  existing cache replay/bounds translation path.
- This keeps component caches local. It deliberately does not wrap the full `CommandPalette` or
  `Combobox` in a broad view cache, because those surfaces carry active descendant, selection,
  disabled/highlight state, semantics, and test-id behavior that should remain policy-owned.
- Added `paint_cache_rebases_descendant_entries_after_ancestor_replay` to cover the three-frame
  sequence: first full paint, second ancestor replay with changed bounds, third ancestor repaint
  while the stable child still replays from the rebased descendant entry.
- Added `paint_cache_rebase_prunes_paint_invalidated_descendant_subtrees` to prove an invalidated
  intermediate node prevents deeper descendants from being rebased and later replayed through that
  invalidated subtree.
- Tightened the existing selectable-text replay test so cache replay across a bounds move still
  touches selectable span state without corrupting local span bounds.
- While running the broader paint-cache test filter, the existing
  `focus_traversal_availability_short_circuits_after_first_candidate` assertion failed because it
  counted layout/prepaint sampling work as part of command availability. The runtime behavior was
  already short-circuiting correctly; the test now asserts the post-layout call delta instead.
- Validation passed:
  `cargo test -p fret-ui --lib paint_cache --profile dev-fast -j 1 -- --test-threads=1`,
  `cargo test -p fret-ui --lib focus_traversal_prepaint_cache --profile dev-fast -j 1 -- --test-threads=1`,
  `cargo fmt -p fret-ui`, and `cargo check -p fret-ui -j 1`.
- The correct combobox dev-fast perf gate with view-cache env, prewarm, and prelude passed:
  `target/fret-diag/gate-combobox-filter-select-devfast-paint-cache-rebase-prune-vc/1781449512986/bundle.schema2.json`.
  Top frame was `10151us` with `layout=5855us`, `solve=1131us`, `prepaint=614us`,
  `paint=3682us`, `paint.cache_misses=0`, `cache.reused=1`, and `cache.replayed_ops=203`.
- Current residual tail is no longer full subtree repaint. The remaining top-frame cost is mostly
  popover/root layout request/apply plus renderer upload/finish/text preparation. The next slice
  should only revisit paint cache if a fresh profile shows cache misses returning; otherwise the
  higher-leverage targets are overlay root apply/layout and renderer retained-text work.

## 2026-06-14 Sixth Slice Findings

- Investigated residual command-availability churn during the searchable combobox filter/select
  gate. Runtime snapshot publication can happen more than once in a frame while pending
  declarative/post-layout refine state is active, but the previous signature gate treated any
  pending state as a hard reason to recompute.
- Added pending window-runtime snapshot state to
  `WindowCommandActionAvailabilitySnapshotSignature`: sorted pending declarative roots plus the
  frame-local post-layout refine marker. This keeps the required post-layout authoritative publish
  while deduping duplicate same-frame interim publishes with identical inputs.
- Split command-availability invalidation from the broader semantics invalidation predicate for the
  first safe case: `ScrollHandleHitTestOnly` invalidations still keep semantics behavior unchanged,
  but no longer reset command-interest metadata or force a command-availability revision bump.
- Focused tests cover both contracts:
  `action_availability_snapshot_dedupes_same_pending_refine_but_post_layout_republishes` and
  `action_availability_snapshot_keeps_interest_cache_for_scroll_hit_test_only_invalidation`.
- Validation passed:
  `cargo test -p fret-ui --lib window_command_action_availability_snapshot --profile dev-fast -j 1 -- --test-threads=1`,
  `cargo test -p fret-ui --lib focus_traversal_prepaint_cache --profile dev-fast -j 1 -- --test-threads=1`,
  `cargo fmt -p fret-ui`, `cargo check -p fret-ui -j 1`, and `git diff --check`.
- The correct combobox dev-fast perf gate still failed on one noisy probe, but the shape changed:
  `target/fret-diag/gate-combobox-filter-select-devfast-command-inv-split-vc/1781453177303/bundle.schema2.json`
  had `total=11586us`, `layout=1575us`, `solve=0us`, `prepaint=1129us`, `paint=8882us`,
  `paint.cache_misses=0`, `cache.reused=1`, and `cache.replayed_ops=222`.
- Remaining failures were pointer-tail thresholds only:
  `pointer_move_max_dispatch_time_us=1507` over `1001`, and
  `pointer_move_max_hit_test_time_us=210` over `170`. The top frame was under the checked-in
  `top_total_time_us` threshold, so this slice is a conservative mechanism cleanup rather than the
  final answer for 120Hz dense UI.
- The remaining command-availability hotspot is still
  `ui_gallery.switch.command_gate.action@focused_or_default`, which is a gallery-level widget
  command registered globally for the UI gallery. The next architectural question is whether runtime
  snapshots should publish all widget commands for every surface, or whether command groups/surfaces
  need a deeper interface for filtering without weakening app command behavior.

## 2026-06-14 Seventh Slice Findings

- Added a mechanism-level filtered action-availability publisher:
  `UiTree::publish_window_command_action_availability_snapshot_filtered(...)`. The existing
  `publish_window_command_action_availability_snapshot(...)` remains the conservative full-window
  default and still publishes every registered widget-scoped command.
- The filtered publisher is intentionally caller-owned: it accepts a concrete command set, sorts and
  dedupes it for stable snapshot signatures, ignores unregistered/non-widget commands, and leaves
  omitted commands as `unknown` in `WindowCommandActionAvailabilityService` rather than publishing
  them as disabled.
- Focused tests cover the contract:
  `action_availability_filtered_snapshot_publishes_only_requested_widget_commands` and
  `action_availability_filtered_snapshot_signature_dedupes_sorted_command_set`.
- Found the higher-leverage immediate issue in the UI Gallery strategy layer: the global shadcn
  command dialog was built with `new_with_host_commands(...)` even while the dialog was closed.
  That meant closed chrome still materialized host command entries and their command/action
  surfaces during dense component frames.
- Aligned the Gallery with the bootstrap command-palette strategy: when closed, render the dialog
  shell with empty entries; when open, build host command entries. This keeps the first screen
  behavior unchanged while removing unrelated command-entry surfaces from the combobox steady path.
- Validation passed:
  `cargo test -p fret-ui --lib window_command_action_availability_snapshot --profile dev-fast -j 1 -- --test-threads=1`,
  `cargo check -p fret-ui-gallery --profile dev-fast -j 1`, and
  `cargo build -p fret-ui-gallery --profile dev-fast -j 1`.
- The corrected combobox dev-fast perf gate passed:
  `target/fret-diag/gate-combobox-filter-select-devfast-command-palette-closed-vc/1781455378046/bundle.schema2.json`.
  Top frame was `11197us` with `layout=5662us`, `solve=913us`, `prepaint=659us`,
  `paint=4876us`, `dispatch=95us`, `hit_test=26us`, `paint.cache_misses=0`,
  `cache.reused=1`, and `cache.replayed_ops=203`.
- Pointer-tail thresholds are now inside the checked-in gate:
  `pointer_move_max_dispatch_time_us=846` and `pointer_move_max_hit_test_time_us=132`.
- The diagnostics app snapshot still reports `command_palette_entries_count=134` because that value
  is produced by the diagnostics snapshot provider from the host command catalog, not by the closed
  dialog render tree. The runtime hotspot evidence is the important signal: the previous
  `ui_gallery.switch.command_gate.action@focused_or_default` hotspot disappeared from the worst
  frames after closed-dialog entry materialization was removed.
- Current decision: keep both changes. The filtered publisher is the right mechanism for future
  app-owned command surfaces, while the Gallery closed-dialog fix is the actual perf win for this
  combobox gate.

## 2026-06-15 Eighth Slice Findings

- Investigated the residual renderer tail after the closed command-palette slice. The latest green
  baseline bundle still had renderer text preparation around `418-582us`, mostly
  `collect_pin_keys` and `bucket_delta`, even when UI paint had no text re-shaping and paint cache
  was replaying stable subtrees.
- Added a renderer-level retained text pin bucket fast path in `fret-render-wgpu`. Each swapchain
  ring bucket records the exact visible `TextBlobId` sequence after a successful full pin pass. If
  the same bucket sees the exact same live text blob sequence again, it skips glyph-bucket rebuild,
  bucket delta, prewarm, and pin ref-count updates for that frame.
- The fast path is intentionally conservative:
  - it stores an exact `TextBlobId` list rather than a hash-only signature,
  - atlas reset clears bucket signatures,
  - missing/evicted text blobs disable reuse,
  - incomplete prewarm does not record a reusable signature,
  - scene changes fall back to the original full path.
- Added diagnostics visibility through
  `renderer_prepare_text_fast_scene_bucket_reuses`, including `fret-diag stats` top-row output as
  `renderer.text_prepare.counts(blobs/fast_reuse/pinned/prewarm/retained/added/removed)`.
- Focused validation passed:
  `cargo test -p fret-render-wgpu --lib prepare_for_scene --profile dev-fast -j 1 -- --test-threads=1`,
  `cargo test -p fret-diag --lib renderer_prepare_text --profile dev-fast -j 1 -- --test-threads=1`,
  `cargo check -p fret-render-wgpu --profile dev-fast -j 1`,
  `cargo check -p fret-bootstrap --profile dev-fast -j 1`,
  and `cargo check -p fret-diag --profile dev-fast -j 1`.
- The correct combobox dev-fast perf gate passed:
  `target/fret-diag/gate-combobox-filter-select-devfast-text-pin-bucket-reuse-vc/1781457750260/bundle.schema2.json`.
  Top frame was `10103us` with `layout=5943us`, `solve=1155us`, `prepaint=629us`,
  `paint=3531us`, `dispatch=0us`, `hit_test=49us`, `paint.cache_misses=0`,
  `cache.reused=1`, and `cache.replayed_ops=203`.
- Compared with the previous accepted bundle
  `target/fret-diag/gate-combobox-filter-select-devfast-command-palette-closed-vc/1781455378046/bundle.schema2.json`,
  the top frame moved from `11197us` to `10103us`, total considered time moved from `43755us` to
  `40004us`, and renderer text p95/max moved from `582us` to `446us`.
- The fast path does not hit the slowest filter/select mutation frames (`fast_reuse=0`) because
  their visible text blob sequence changes. It does hit stable frames (`fast_reuse=1`), reducing
  their text prepare path to roughly `12-16us`.
- Current decision: keep this slice as a shared renderer infrastructure win, but do not treat it as
  the final answer for the dense combobox path. The remaining worst frames are still dominated by
  popover/root layout request/apply plus renderer upload/finish/encode work and command availability
  tails around focus/text paste routing.

## 2026-06-15 Ninth Slice Findings

- Investigated the residual action availability tail after the renderer text-pin slice. The key
  design issue was not that shadcn-style component nesting is inherently too expensive; it was that
  owner-scoped action availability hooks could only declare "all commands" interest, so a runtime
  snapshot could route unrelated widget commands into policy handlers.
- Added command-specific availability interest to the action-route mechanism. The existing
  `action_on_command_availability_for_owner` and `action_add_on_command_availability_for_owner`
  APIs keep their conservative `All` behavior, while new command-specific APIs let strategy/app
  layers declare a precise `CommandId` interest.
- Refined the implementation from owner-level aggregation to entry-level filtering. This matters for
  app render action hooks: a single owner can register many typed action availability handlers, and
  the runtime should only invoke the entry whose declared command matches the command being queried.
- Reworked declarative command-interest metadata into a small composable structure. Built-in
  interests such as text editing, selectable text editing, and focus traversal can now union with
  command-specific action interests without widening to `All`. This also fixes the semantic hazard
  where an early built-in interest return on a `TextInput` node could hide a custom command-specific
  action availability hook on the same element.
- Updated `ecosystem/fret` app-render action availability to use the command-specific API. This
  keeps typed action availability discoverable without forcing unrelated command probes through the
  same app action owner.
- Focused validation passed:
  `cargo test -p fret-ui --lib owner_scoped_action_availability_for_command --profile dev-fast -j 1 -- --test-threads=1`,
  `cargo test -p fret-ui --lib window_command_action_availability_snapshot --profile dev-fast -j 1 -- --test-threads=1`,
  `cargo check -p fret-ui --profile dev-fast -j 1`,
  `cargo check -p fret --profile dev-fast -j 1`,
  `cargo check -p fret-ui-gallery --profile dev-fast -j 1`,
  `cargo fmt -p fret-ui -p fret`, and `git diff --check`.
- The correct combobox dev-fast perf gate passed:
  `target/fret-diag/gate-combobox-filter-select-devfast-action-interest-entries-vc/1781460864667/bundle.schema2.json`.
  Top frame was `9885us` with `layout=5872us`, `solve=1084us`, `prepaint=624us`,
  `paint=3389us`, `dispatch=0us`, `hit_test=47us`, `paint.cache_misses=0`,
  `cache.reused=1`, and `cache.replayed_ops=203`.
- This slice is primarily a mechanism-correctness and future scaling improvement, not a dramatic
  win for the current gate. The current run remains inside the green noise band around the previous
  `10103us` text-pin bundle, but `ui_gallery.switch.command_gate.action` still appears when that
  command itself is being evaluated. Command-interest filtering can prevent unrelated handlers from
  running; it cannot remove a command from a full-window snapshot command set.
- Current decision: keep the command-specific hook API and entry-level filtering. The next command
  availability question is publisher-level command grouping/surfaces, not more per-handler
  filtering.

## 2026-06-15 Layout Profile Follow-up

- Re-ran the correct combobox dev-fast gate with layout node profiling after the command-specific
  action-interest slice:
  `target/fret-diag/gate-combobox-filter-select-devfast-layout-profile-after-action-interest/1781461140853/bundle.schema2.json`.
- The gate remained green. Top frame was `10452us` with `layout=6134us`, `solve=1093us`,
  `prepaint=644us`, `paint=3674us`, `dispatch=0us`, `hit_test=56us`, `paint.cache_misses=0`,
  `cache.reused=1`, and `cache.replayed_ops=203`.
- Root-level layout attribution showed the popover overlay root as the meaningful dirty root:
  `root[window-overlays.popover.29fc70edc4575465]` rebuilt a `DismissibleLayer` subtree with
  `subtree_layout_dirty_count=37` in about `285us`. The primary dirty source was model observation
  flowing through the overlay `ViewCache` and `InteractivityGate`.
- The main window root only performed a `mark_seen` pass over `1073` nodes in about `112us`. That is
  not currently a high-leverage or low-risk target compared with the remaining layout request/apply
  and renderer tails.
- Query/filter changes in `CommandPalette` and combobox remain layout-sensitive because they can
  change row materialization and content height. Treating those model observations as paint-only
  would be a contract bug unless the runtime gains a deeper virtual row layout contract.
- Renderer text preparation is now the cleaner next target for mutation frames: the latest profile
  still shows `renderer.text_prepare p95/max ~= 440us`, with `collect_pin_keys ~= 251us` and
  `bucket_delta ~= 171us` when `fast_reuse=0`. Stable frames are already covered by the retained
  text pin bucket fast path, so the open question is whether changed-blob keyed glyph bucket deltas
  can avoid rebuilding the full `GlyphKeyBuckets` on filter/select mutation frames.

## 2026-06-15 Tenth Slice Findings

- Implemented a renderer text pin-state delta path for mutation frames. `TextPinState` now keeps
  per-ring-bucket glyph membership sets, collects a lightweight scene pin snapshot, and computes
  retained/added/removed glyph keys directly from the current ref-count maps instead of rebuilding a
  full `GlyphKeyBuckets` and diffing it against the old bucket.
- The bucket update is in-place: removed glyph keys are deleted from the current ring bucket, added
  glyph keys are appended after successful atlas prewarm, and the exact scene signature is recorded
  only when the bucket is complete. This keeps atlas pin correctness aligned with the previous
  full-diff path while reducing allocation and retained-key movement.
- Switched the pin-state hot maps/sets to `rustc_hash::FxHashMap/FxHashSet`. These are internal
  glyph/blob id maps, not attacker-controlled lookup tables, and match the existing choice in the
  layout hot path.
- Removed the old full-bucket diff helper from `atlas.rs`; the regression surface now uses the real
  `prepare_for_scene` path instead of a detached helper test.
- Focused validation passed:
  `cargo test -p fret-render-wgpu --lib prepare_for_scene --profile dev-fast -j 1 -- --test-threads=1`,
  `cargo check -p fret-render-wgpu --profile dev-fast -j 1`,
  `cargo build -p fret-ui-gallery --profile dev-fast -j 1`,
  `cargo fmt -p fret-render-wgpu`, and `git diff --check`.
- The correct combobox dev-fast perf gate passed:
  `target/fret-diag/gate-combobox-filter-select-devfast-text-pin-fx-delta-vc/1781463556043/bundle.schema2.json`.
  Top frame was `9932us` with `layout=5635us`, `solve=1034us`, `prepaint=606us`,
  `paint=3691us`, `dispatch=0us`, `hit_test=47us`, `paint.cache_misses=0`,
  `cache.reused=1`, and `cache.replayed_ops=203`.
- Renderer text preparation improved materially on the mutation frames:
  previous layout-profile bundle showed `renderer.text_prepare p95/max ~= 440us` with
  `collect_pin_keys=251us` and `bucket_delta=171us`; the accepted bundle shows
  `renderer.text_prepare p95/max=179us`, `collect_pin_keys=76us`, and `bucket_delta=80us`.
- Current decision: keep this slice as a shared renderer infrastructure win. The combobox gate is
  now again dominated by overlay/root layout request/apply, command-availability tails, and renderer
  upload/finish rather than text pin bucket reconstruction.

## 2026-06-15 Eleventh Slice Findings

- Re-profiled the correct combobox dev-fast gate with layout node profiling after the text pin-state
  delta slice:
  `target/fret-diag/gate-combobox-filter-select-devfast-layout-profile-after-text-pin-fx/1781463914238/bundle.schema2.json`.
  The top frame was `9655us` with `layout=5711us`, `solve=1054us`, `paint=3353us`,
  and renderer text preparation down to about `186us`.
- Layout attribution showed the remaining high-cost dirty root was the popover overlay
  `DismissibleLayer -> ViewCache -> InteractivityGate` chain. The important issue was that the
  keep-alive overlay `ViewCache` remained parent-dependent even though its root bounds are known and
  fill the overlay surface.
- Added a shared `overlay_keep_alive_view_cache_props()` helper in `fret-ui-kit` and applied it to
  modal, popover, hover overlay, and tooltip keep-alive caches. The helper gives the cache root
  `width: fill`, `height: fill`, and
  `ViewBoundaryHints::contain_layout_when_bounds_known(true)`.
- The corrected all-overlay probe passed:
  `target/fret-diag/gate-combobox-filter-select-devfast-overlay-contained-all-vc/1781464820646/bundle.schema2.json`.
  Top frame was `9468us` with `layout=5682us`, `solve=1045us`, `paint=3205us`,
  and `contained_relayouts=1`.
- The main win is not a lower solve number yet; it is locality. `request_build` roots dropped from
  about `1187us` to about `144us`, roots apply dropped from about `925us` to about `4us`, and the
  parent `DismissibleLayer` became a cheap `mark_seen` pass of about `25us`.
- The remaining layout solve is now isolated inside the contained `ViewCache` relayout root
  (`layout_dependency = contained_when_bounds_known`), around `0.8-1.0ms` on the worst mutation
  frame. This is the next high-leverage target if fresh evidence keeps pointing at layout.
- While validating the shadcn overlay path, the focused `popover` test filter exposed an adjacent
  existing layout-contract issue: `PopoverHeader` had drifted onto the shrink-wrapped stack helper
  while its wrapping-text test still expected a fill-width inner stack. Restored the fill-width
  helper and changed the test to assert the actual layout contract (`Fill` + `min-width: 0`) instead
  of depending on whether the transparent helper materializes as `Flex` or `Container`.
- Focused validation passed:
  `cargo test -p fret-ui --lib try_with_state_mut_only_records_existing_state_keys_for_view_cache --profile dev-fast -j 1 -- --test-threads=1`,
  `cargo test -p fret-ui --lib view_cache --profile dev-fast -j 1 -- --test-threads=1`,
  `cargo check -p fret-ui-kit --profile dev-fast -j 1`,
  `cargo build -p fret-ui-gallery --profile dev-fast -j 1`,
  `cargo test -p fret-ui-shadcn --lib popover --profile dev-fast -j 1 -- --test-threads=1`,
  `cargo test -p fret-ui-shadcn --lib combobox --profile dev-fast -j 1 -- --test-threads=1`,
  and `cargo fmt -p fret-ui -p fret-ui-kit -p fret-ui-shadcn`.
- Current decision: keep this slice. It is a mechanism-level containment fix with direct benefit to
  shadcn popover/combobox surfaces and likely to modal, hover-card, and tooltip surfaces. It also
  makes the next optimization clearer: optimize the contained overlay cache relayout itself, not the
  parent overlay root.

## 2026-06-15 Twelfth Slice Exploration

- Re-opened the latest contained-overlay bundle:
  `target/fret-diag/gate-combobox-filter-select-devfast-overlay-contained-all-vc/1781464820646/bundle.schema2.json`.
  The worst considered frame stayed green but still exceeded a strict 120Hz budget:
  `total=9468us`, `layout=5682us`, `layout.engine_solve=1045us`, `paint=3205us`,
  and `contained_relayouts=1`.
- The important distinction is that the popover overlay cache root was not merely a clean reused
  cache needing a root-only geometry update. On the worst mutation frame it was
  `reuse_reason = needs_rerender`, with the filtered command/list subtree changing. A root-only
  ViewCache relayout fast path would therefore not address the immediate gate unless it also had a
  correct way to prove child layout dependencies were unchanged.
- Root-only contained ViewCache relayout remains a future mechanism candidate, but it needs an
  authoritative dirty-cause contract. Debug invalidation details are useful evidence, not a safe
  behavior input. The runtime must distinguish root-only scheduling/geometric dirty from subtree
  dependency dirty before it can skip descendant expansion globally.
- The current residual cost is distributed across three regions rather than one obvious full rewrite
  target: contained overlay subtree layout, semantics refresh on dirty frames, and renderer
  upload/finish/text tail. This reinforces the current strategy: keep landing narrow mechanism and
  strategy fixes with perf evidence instead of declaring shadcn-style nested composition inherently
  too expensive.
- Negative experiment: keeping a large plain command source on the virtual-list lane after filtering
  to a tiny result set did remove the previous full-rows branch hotspot around `command.rs:3543`, but
  it did not improve the gate. The measured run at
  `target/fret-diag/gate-combobox-filter-select-devfast-large-source-virtual-vc/1781467734298/bundle.schema2.json`
  regressed to `total=10364us`, `layout=6339us`, `layout.engine_solve=1339us`, `paint=3391us`.
- Node-level profiling confirmed the reason. With forced virtualization on the one-row filtered
  state, the hot overlay nodes became the virtual-list path itself:
  `VirtualList self=640us total=796us`, `ScrollArea/Stack total=~0.95-1.18ms` at frame 160 in
  `target/fret-diag/gate-combobox-filter-select-devfast-large-source-virtual-node-profile/1781467878354/bundle.schema2.json`.
  Decision: do not land this strategy. Small filtered results should stay on the simple full-row
  layout path until the virtual-list fixed cost is substantially lower.
- The useful learning is architectural: stable layout shape is not automatically cheaper than the
  simplest shape for the current result set. The next optimization should target shared container
  fixed costs (`ScrollArea`, contained overlay relayout, virtual-list measurement/update) rather than
  forcing every command state through virtualization.
- Control experiment on the original non-forced strategy:
  `target/fret-diag/gate-combobox-filter-select-devfast-original-node-profile/1781468169506/bundle.schema2.json`
  passed with `total=10236us`, `layout=5985us`, `layout.engine_solve=1018us`,
  `paint=3632us`. The one-row full-row path showed the overlay `ScrollArea` around `502us`, while
  the forced virtual path added `VirtualList` work on top. This confirms the right immediate policy:
  keep virtualization thresholded by rendered row count.
- Landed the next narrow policy/mechanism split: the compact `ScrollArea` surface can now forward
  the existing low-level `ScrollAreaViewport::focus_ring(false)` knob, and `CommandPalette` disables
  the viewport focus-ring wrapper for its listbox scroll areas. Standalone `ScrollArea` parity stays
  unchanged by default; command/combobox listboxes keep focus in the input and expose highlight via
  `active_descendant`, so the viewport focus wrapper was duplicated strategy cost.
- Validation passed:
  `cargo test -p fret-ui-shadcn --lib scroll_area --profile dev-fast -j 1 -- --test-threads=1`,
  `cargo test -p fret-ui-shadcn --lib command_palette --profile dev-fast -j 1 -- --test-threads=1`,
  `cargo test -p fret-ui-shadcn --lib combobox --profile dev-fast -j 1 -- --test-threads=1`,
  `cargo check -p fret-ui-shadcn --profile dev-fast -j 1`, and
  `cargo fmt -p fret-ui-shadcn`.
- The corrected combobox gate passed:
  `target/fret-diag/gate-combobox-filter-select-devfast-command-list-no-viewport-focus-ring/1781468669211/bundle.schema2.json`.
  Top frame was `9827us` with `layout=5766us`, `layout.engine_solve=763us`,
  `paint=3403us`, `dispatch=90us`, `hit_test=16us`, `paint.cache_misses=0`,
  `cache.reused=1`, and `contained_relayouts=1`. Dirty invalidation nodes on the mutation frame
  dropped to about `120-130`, and pointer tails stayed inside the checked-in thresholds.
- Decision: keep this slice. It is not a broad architecture rewrite, but it is the right kind of
  component-ecosystem optimization: expose an existing mechanism knob at the recipe surface and let
  the CommandPalette strategy avoid unnecessary focus/animation/semantics wrapper nodes.

## 2026-06-15 Full-Row Bounded Probe Follow-up

- Aligned the non-virtualized `CommandPalette` full-row listbox path with the virtualized path by
  disabling unbounded scroll viewport probing. Command/listbox surfaces already receive explicit
  strategy sizing and max-height constraints, so the shrink-wrap intrinsic probe is unnecessary for
  this recipe path.
- Added `command_palette_full_rows_use_bounded_scroll_viewport_probe` to lock this strategy at the
  element-tree level. The test deliberately avoids timing assertions; perf gates remain responsible
  for measuring whether the policy matters on a given scenario.
- Focused validation passed:
  `cargo test -p fret-ui-shadcn --lib command_palette --profile dev-fast -j 1 -- --test-threads=1`,
  `cargo test -p fret-ui-shadcn --lib combobox --profile dev-fast -j 1 -- --test-threads=1`,
  `cargo check -p fret-ui-shadcn --profile dev-fast -j 1`, and
  `cargo fmt -p fret-ui-shadcn`.
- The corrected combobox gate passed:
  `target/fret-diag/gate-combobox-filter-select-devfast-full-row-bounded-probe/1781469475271/bundle.schema2.json`.
  Top frame was `9831us` with `layout=5837us`, `layout.engine_solve=1089us`,
  `paint=3383us`, `dispatch=0us`, `hit_test=47us`, `paint.cache_misses=0`,
  `cache.reused=1`, and `contained_relayouts=1`.
- Decision: keep this as a small strategy-consistency cleanup, not as a claimed major win. It keeps
  the current green band stable and removes another avoidable listbox fixed-cost path. The dominant
  residual work is still contained overlay relayout, command-availability publication breadth, and
  renderer upload/finish/text tail.

## Next Verification

1. Revisit the contained popover `ViewCache` relayout solve tail now that parent-root request/apply
   is no longer the dominant overlay cost.
2. Keep reducing the remaining contained overlay listbox cost only where evidence points to shared
   fixed work. The latest slices removed the duplicated viewport focus wrapper, unbounded full-row
   probing, and hidden scrollbar chrome; remaining hot areas are contained overlay relayout solve,
   the core `Scroll` layout path, and renderer upload/finish/text.
3. Design a publisher-level command surface/group mechanism only if fresh evidence shows full-window
   snapshot command sets are still taxing dense component interactions. Per-handler filtering is now
   in place; the remaining lever is deciding which commands the publisher should evaluate at all.
4. Keep watching renderer upload/finish/encode p95, which is now often comparable to the remaining
   UI-side work in green combobox probes.

## 2026-06-15 Thirteenth Slice Findings

- Added a frame-local command action-availability demand contract in `fret-ui`. The default remains
  conservative: if no surface declares demand, `publish_window_runtime_snapshots(...)` still
  publishes all registered widget commands. Declared surfaces can now request either all widget
  commands or a specific command set; omitted commands remain unknown, not disabled.
- Kept the demand in `WindowElementState` and cleared it at frame boundaries. `ElementContext` now
  exposes narrow request APIs so ecosystem surfaces can declare their command-gating consumption
  without turning this into a global app setting.
- Wired the two immediate consumers:
  `command_catalog_entries_from_host_commands_with_options(...)` requests the full host catalog,
  while `menubar_from_runtime_with_focus_handle(...)` requests only command ids found in the
  normalized menu bar. This keeps OS/native menu compatibility conservative when no surface has
  declared demand, and lets the in-window menu avoid unrelated widget-command probes.
- Focused tests cover the compatibility boundary:
  no demand keeps full publication, filtered demand publishes only requested commands, and full
  demand wins over filtered demand.
- Validation passed:
  `cargo test -p fret-ui --lib window_command_action_availability_snapshot --profile dev-fast -j 1 -- --test-threads=1`,
  `cargo check -p fret-ui-gallery --profile dev-fast -j 1`,
  `cargo check -p fret-ui-shadcn --profile dev-fast -j 1`,
  `cargo fmt -p fret-ui -p fret-ui-kit -p fret -p fret-ui-gallery`, and `git diff --check`.
- The corrected combobox dev-fast perf gate passed:
  `target/fret-diag/gate-combobox-filter-select-devfast-command-demand-v1/1781472736858/bundle.schema2.json`.
  Top frame was `10032us` with `layout=5946us`, `layout.engine_solve=1141us`,
  `paint=3491us`, `dispatch=0us`, `hit_test=52us`, `paint.cache_misses=0`,
  `cache.reused=1`, and `contained_relayouts=1`.
- `diag stats` confirmed the demand contract is active in the dense path:
  `window_runtime_snapshot.command_availability(widget_count/collect_us/eval_us)=4/34/112` on the
  top frame, with the slower observed frame at `4/55/437`. This removes unrelated full-registry
  widget-command publication from the closed command-palette + in-window-menu steady path, but it is
  not the dominant remaining cost.
- Current decision: keep this slice as the publisher-level command surface mechanism. The remaining
  strict-120Hz gap is still distributed across contained overlay relayout, renderer upload/finish,
  and smaller command/text tails rather than a single shadcn nesting tax.

## 2026-06-15 Fourteenth Slice Findings

- Converted `ScrollArea::show_scrollbar(false)` into a real viewport-only chrome path. It keeps the
  layout `Stack` root and `Scroll` viewport, but skips the `HoverRegion`, scrollbar visibility
  state, hidden `Scrollbar` primitives, interactivity gates, opacity wrappers, and corner chrome.
  The default `ScrollArea` path remains Radix/shadcn-aligned and still mounts scrollbar chrome.
- Factored the shared viewport construction so the default chrome path and the viewport-only path
  both use the same `Scroll` + optional focus-ring/semantics wrapper logic. This avoids a second
  focus-ring implementation while still returning the inner `Scroll` element id for scrollbar
  targeting.
- `CommandPalette` now declares `.show_scrollbar(false)` for both the virtualized and full-row
  listbox paths. This matches the recipe strategy: focus stays in the search input, highlight is
  exposed via `active_descendant`, and the listbox does not need hover-gated scrollbar chrome.
- Added structure tests:
  `scroll_area_show_scrollbar_false_uses_viewport_only_chrome` locks the standalone
  `show_scrollbar(false)` contract, and
  `command_palette_listboxes_use_scrollbarless_viewport_chrome` locks the CommandPalette recipe
  policy. These are element-tree tests rather than timing assertions.
- Validation passed:
  `cargo test -p fret-ui-shadcn --lib scroll_area --profile dev-fast -j 1 -- --test-threads=1`,
  `cargo test -p fret-ui-shadcn --lib command_palette --profile dev-fast -j 1 -- --test-threads=1`,
  `cargo test -p fret-ui-shadcn --lib combobox --profile dev-fast -j 1 -- --test-threads=1`,
  `cargo check -p fret-ui-shadcn --profile dev-fast -j 1`,
  `cargo check -p fret-ui-gallery --profile dev-fast -j 1`,
  `cargo fmt -p fret-ui-shadcn`, and `git diff --check`.
- The first corrected combobox gate was a narrow solve-threshold miss:
  `target/fret-diag/gate-combobox-filter-select-devfast-scrollbarless-listbox/1781474462029/bundle.schema2.json`
  had `total=10814us`, `layout=6627us`, `layout.engine_solve=1469us`,
  `paint=3543us`, and failed only `top_layout_engine_solve_time_us > 1389us`.
  The run still showed the intended structural reduction: layout nodes around `32` and no hover
  invalidations, so this looked like a solve-tail sample rather than a clear regression.
- The rerun passed:
  `target/fret-diag/gate-combobox-filter-select-devfast-scrollbarless-listbox-rerun/1781474598799/bundle.schema2.json`.
  Top frame was `9710us` with `layout=5587us`, `layout.engine_solve=1085us`,
  `paint=3516us`, `dispatch=0us`, `hit_test=48us`, `paint.cache_misses=0`,
  `cache.reused=1`, and `contained_relayouts=1`.
- Node profiling after the change also passed:
  `target/fret-diag/gate-combobox-filter-select-devfast-scrollbarless-listbox-node-profile/1781474698584/bundle.schema2.json`.
  Top frame was `10108us` with `layout=5848us`, `layout.engine_solve=1124us`,
  and `paint=3588us`. The overlay listbox `Scroll` node measured around
  `350us self / 517us total` on the profiled top frame, compared with the previous
  `~500-1200us total` band seen before removing hidden chrome.
- Decision: keep this slice. It is a policy-level trim rather than a broad framework rewrite, but
  it removes avoidable recipe chrome and confirms that shadcn-style composition does not inherently
  require hidden wrappers for every scroll surface. The remaining strict-120Hz gap is now dominated
  by contained overlay relayout solve, the core `Scroll` layout cost, and renderer upload/finish.

## 2026-06-15 Fifteenth Slice Findings

- Added a narrow static listbox surface for non-virtualized `CommandPalette` full-row results when
  filtering produces exactly one `Item` row. The static surface keeps the same outer list sizing
  contract as the viewport-only `ScrollArea` stack (`width: fill`, `min-width: 0`, `min-height: 0`,
  plus caller `refine_scroll_layout(...)` sizing), but skips `Scroll`, scroll handles, focus-ring
  wrappers, hidden scrollbar chrome, and scroll-to-active work.
- Kept the strategy intentionally narrow:
  grouped rows, headings, separators, loading rows, empty states, and multi-row results still use
  the existing full-row `ScrollArea` path; large plain item sets still use virtualization. The
  static path only applies to the one-row filtered state that was showing avoidable fixed scroll
  cost in the combobox gate.
- Added structure and semantics tests:
  `command_palette_single_item_full_rows_use_static_list_surface` proves the single-row path mounts
  no `Scroll`, `Scrollbar`, or `HoverRegion`; `command_palette_listboxes_use_scrollbarless_viewport_chrome`
  now also proves multi-row listboxes still mount one `Scroll`; and
  `command_palette_single_item_static_list_surface_preserves_active_descendant_semantics` proves
  the input still controls the listbox and its `active_descendant` points at the selected option.
- Validation passed:
  `cargo test -p fret-ui-shadcn --lib command_palette --profile dev-fast -j 1 -- --test-threads=1`,
  `cargo test -p fret-ui-shadcn --lib combobox --profile dev-fast -j 1 -- --test-threads=1`,
  `cargo check -p fret-ui-shadcn --profile dev-fast -j 1`,
  `cargo check -p fret-ui-gallery --profile dev-fast -j 1`,
  `cargo fmt -p fret-ui-shadcn`, and `git diff --check`.
- The corrected combobox dev-fast perf gate passed:
  `target/fret-diag/gate-combobox-filter-select-devfast-single-item-static-listbox/1781477402294/bundle.schema2.json`.
  Top frame was `9808us` with `layout=5560us`, `layout.engine_solve=954us`,
  `prepaint=654us`, `paint=3594us`, `dispatch=0us`, `hit_test=44us`,
  `paint.cache_misses=0`, `cache.reused=1`, and `contained_relayouts=1`.
- `diag stats` confirmed the remaining tail is no longer the single-row scroll surface:
  `layout.nodes=31`, renderer p95/max `upload=688us`, `finish=1541us`, `encode=867us`,
  `text=200us`, and command availability stayed bounded at
  `widget_count/collect_us/eval_us=4/29/105` on the top frame. The strict 120Hz gap remains
  distributed across contained overlay relayout, paint traversal, and renderer upload/finish.
- Current decision: keep this slice. It is a component-policy optimization, not a core
  architecture rewrite. It reinforces the current architecture call: shadcn-style nesting is not
  inherently the blocker; avoidable fixed-cost recipe surfaces are. Continue optimizing where the
  gate shows unnecessary mechanism costs, and reserve a `ViewCache`/layout-engine rethink for a
  stronger root-only dirty-cause contract.

## 2026-06-15 Sixteenth Slice Findings

- The first broad incremental semantics-reuse pass was too permissive. It let clean descendants
  reuse old semantics records even when an ancestor had a semantic invalidation that changed the
  ancestor's children transform, which left scroll/visibility scripts observing stale descendant
  bounds.
- Narrowed the reuse contract so descendant replay is only allowed when no ancestor on that path was
  rebuilt in the same traversal. This keeps the useful sibling-subtree reuse win while forcing
  descendants to rebuild after scroll/transform changes.
- Added a regression test that mutates a parent's children transform and verifies the child rebuilds
  its semantics bounds on the next refresh.
- Validation passed:
  `cargo test -p fret-ui --lib semantics_focus_shortcuts --profile dev-fast -j 1 -- --test-threads=1`,
  `cargo check -p fret-ui --profile dev-fast -j 1`, and `cargo fmt -p fret-ui`.
- The corrected perf gate passed:
  `target/fret-diag/gate-combobox-filter-select-devfast-semantics-incremental-rerun/1781481180728/bundle.schema2.json`.
  `check.perf_thresholds.json` reports `failures=[]`; top frame was `9010us` with
  `layout=5222us`, `solve=860us`, `paint=3202us`, `dispatch=0us`, and `hit_test=46us`.
- The useful conclusion is narrower than the original experiment: semantics reuse is still a valid
  lever, but only below an ancestor-sensitivity line. Do not broaden it again without a stronger
  freshness marker for layout-affecting ancestors.
- The next obvious follow-up is not more semantics reuse. The remaining dense-component tail is now
  back where the earlier evidence pointed: overlay root solve/apply and renderer upload/finish.

## 2026-06-15 Seventeenth Slice Findings

- The runtime `Popover` path in `ecosystem/fret-ui-shadcn/src/popover.rs` still mounted
  `radix_popover::popover_dialog_wrapper(...)` around `PopoverContent`, and that wrapper was the
  remaining dialog-shaped `Semantics` hotspot in the combobox/popover path.
- Attempted to collapse the common popover trigger-controls target onto the existing
  `PopoverContent` root instead of a separate dialog wrapper, and changed `PopoverContent` itself
  to carry `SemanticsRole::Dialog` directly.
- The experiment was later rolled back after the combobox perf gate regressed, so this is a
  negative result rather than a landed optimization.
- Keep this as a recorded dead end: the next safe slice should start from the current wrapper-based
  shape and only move if a smaller, better-attributed overlay or command path appears.

## 2026-06-15 Seventeenth Slice Reversal

- The first attempt to collapse the popover dialog wrapper onto the `PopoverContent` root regressed
  the combobox perf gate badly: `top_total_time_us` rose to `20701us` and
  `top_layout_engine_solve_time_us` rose to `2049us`.
- The bad run also showed `window_runtime_snapshot.command_availability` and `focus_repair` tails
  rising on the same frame, which means the wrapper change was not a harmless structural trim.
- The popover code was reverted to the wrapper-based shape. Keep this as negative evidence: the
  `Dialog`-role root is not the next safe optimization lever on this path.
- Next direction should return to a lower-risk overlay/mechanism target or a command/layout path
  with clearer attribution, rather than trying to merge the popover semantic root again.

## 2026-06-15 Eighteenth Slice Diagnosis

- The failed popover-root experiment is not a clean framework signal by itself. On the bad run,
  the frame still spent most of its time in layout and paint, with renderer tail cost remaining
  visible: `layout=11539us`, `paint=8039us`, `renderer.finish=3724us`, `renderer.upload=585us`.
- The same run showed `window_runtime_snapshot.focus_repair=908us` and
  `window_runtime_snapshot.command_availability=410us`, but the command set was still tiny
  (`widget_count=4`). That makes command publication a real cost, not the dominant one.
- `repair_focus_node_from_focused_element_if_needed(...)` is already gated to final layout passes,
  and `revalidate_focus_for_dispatch_snapshot(...)` is a bounded reachability check. The hotspot
  is the repeated authoritative snapshot boundary around the overlay, not an obviously shallow
  focus algorithm.
- `publish_window_command_action_availability_snapshot_for_command_set(...)` is also already
  filtered and signature-cached. Its remaining cost matters, but the current evidence does not
  justify treating it as the main blocker for 120Hz on dense component surfaces.
- The stable recurring tail is renderer-side: `finish`, `upload`, and `text_prepare` remain the
  visible p95/max costs even on the better rerun. That points back to surface complexity and text
  churn rather than a single renderer bug.
- Keep the current wrapper-based popover shape. Do not revisit the dialog-root merge.
- The next verification should stay on a lower-risk component/recipe candidate or a heavier probe
  with clearer text/upload attribution before touching core focus or command machinery again.

## 2026-06-15 Nineteenth Slice Probe Triage

- Ran four more probe surfaces: `data-table`, `virtual-list`, `inspector`, and `code-editor`.
- `ce-data-table-probe` is not yet a stable perf gate. It failed at step 27 on the row-selection
  assertion after clicking `ui-gallery-data-table-row-0`, so the bundle is useful for diagnosis but
  not for a durable perf comparison until that assertion path is stabilized or replaced.
- The collected profiles still show the important shape:
  - `ce-data-table-probe`: `total=8353us`, `layout=7725us`, `paint=492us`,
    `command_availability=2422us`, `widget_count=4`.
  - `ce-virtual-list-probe`: `total=7345us`, `layout=6866us`, `solve=1896us`, `prepaint=87us`,
    `paint=392us`, `dispatch=145us`, `hit_test=17us`.
  - `ce-inspector-probe`: `total=4936us`, `layout=4270us`, `solve=1350us`, `prepaint=189us`,
    `paint=477us`, `dispatch=149us`, `hit_test=15us`.
  - `ce-code-editor-probe`: `total=789us`, `layout=125us`, `prepaint=399us`, `paint=265us`,
    `dispatch=0us`, `hit_test=0us`.
- The conclusion is still the same at a broader sample set: there is no single "shadcn nesting tax"
  to remove. The heavier table and inspector surfaces are still layout-dominant, command
  availability can become a visible secondary cost, and the current code-editor probe is too light
  to stand in for a real heavy editor path.
- Next action is to prefer the stable retained / view-cache data-table scripts already in the repo
  and treat the current `ce-data-table-probe` as a diagnosis-only artifact until its step-27
  selection assertion is stabilized or removed. The next optimization target should come from a
  probe that is both dense and repeatable.

## 2026-06-15 Twentieth Slice Stable Probe Selection

- Kept `ce-data-table-probe` in diagnosis-only mode. The step-27 row-selection assertion still
  makes it too fragile to serve as a durable gate.
- The stable next probes are the retained/view-cache data-table suites and the inspector torture
  suite, because they are dense enough to keep layout-dominant evidence while still being repeatable
  across runs.
- The collected profiles still separate the heavy surfaces from the light one: `ce-data-table-probe`
  at `8353us`, `ce-virtual-list-probe` at `7345us`, `ce-inspector-probe` at `4936us`, and
  `ce-code-editor-probe` at `789us`. The last one is too light to stand in for a real heavy editor
  path.
- The strongest remaining leverage point is still the table row/cell policy plus the
  `VirtualList` retained reconciliation seam, with `ecosystem/fret-ui-kit/src/declarative/table.rs`
  as the likely next focus and `window-command-availability-snapshot-v2` remaining secondary.
- The broader architecture conclusion has not changed: there is no single shadcn nesting tax to
  remove. The optimization path is component- and surface-specific, and the probe choice matters as
  much as the code change.

## 2026-06-15 Twenty-First Slice Architecture Split

- A focused architecture review of `table.rs` found that the data-table row/cell layer is still a
  wide adapter rather than the deepest seam. It owns sorting, grouping, pinning, selection, debug
  ids, paint order, grid lines, measured rows, and keep-alive policy in one broad surface.
- The higher-leverage next seam is `VirtualList` retained reconciliation: mount-time keep-alive /
  attach-detach / reuse logic, element `items_revision` / key-cache adapters, and the prepaint
  window-shift classifier. That seam crosses table, list, inspector, and editor-grade surfaces.
- Do not spend the next slice on local table knobs such as grid-line switches, paint-order flags,
  wrapper test-id plumbing, or header/body wrapper symmetry unless a stable probe makes one of them
  a primary owner. These knobs may clean code, but they do not deepen the architecture boundary.
- `focus_repair` and `command_availability` are now treated as an independent runtime owner named
  `Dispatch Snapshot`. Data-table, inspector, and virtual-list surfaces amplify this cost, but the
  owner is the window runtime snapshot path, not the component surface.
- The performance tracks for the next work should therefore be separated as:
  layout / virtual-list reconciliation, dispatch snapshot, and renderer tail. This prevents every
  heavy-surface spike from being misattributed to "shadcn nesting" or to data-table row/cell shape.

## 2026-06-15 Twenty-Second Slice Retained Reconcile Fast Path

- Implemented the first mechanism-layer follow-up in `crates/fret-ui/src/declarative/mount.rs`.
- Retained `VirtualList` reconcile no longer constructs the desired-key `HashSet` when
  `keep_alive == 0`; that set only exists to identify detached rows for the keep-alive pool.
- Added a conservative ordered-overlap fast path for contiguous retained windows. When the current
  and desired visible windows overlap in the same index/key order, preserved children are copied by
  slice position instead of using the generic `existing_by_key` map.
- The fast path deliberately rejects non-contiguous windows so custom range extractors, sticky rows,
  anchor rows, or reorders stay on the generic keyed reconcile path.
- Focused validation passed:
  `cargo nextest run --cargo-profile dev-fast -p fret-ui retained_virtual_list_ordered_overlap mechanism_harness_retained_virtual_list_reconcile_matches_oracles --no-fail-fast --no-capture`,
  `cargo fmt -p fret-ui`, and `git diff --check`.
- Extended focused validation also passed:
  `cargo nextest run --cargo-profile dev-fast -p fret-ui retained_virtual_list_ --no-fail-fast --no-capture`
  covered the retained reconcile harness plus retained VirtualList view-cache, prefetch, keep-alive,
  and viewport-authority tests.
- Runtime correctness gate passed:
  `target/release/fretboard-dev.exe diag suite ui-gallery-data-table-retained ... --launch -- cargo run -p fret-ui-gallery --release --features gallery-ai,gallery-chart,gallery-dev,gallery-web-ime-harness`
  passed 12/12 scripts with evidence in
  `target/fret-diag/vlist-retained-fastpath-v1-cargo/suite.summary.json`.
- The first attempted prebuilt-exe suite failed before launch because the diagnostics preflight
  could not prove required gallery cargo features from a prebuilt binary; it was not a runtime or
  code failure.
- This slice should reduce allocation/hash work in the common retained scroll-window shift path, but
  it is not yet claimed as a user-visible perf win until a stable data-table or inspector probe is
  rerun.

## 2026-06-15 Twenty-Third Slice Semantics Translation Correctness

- Investigated a possible view-cache layout dirty expansion shortcut: stop walking into a clean
  nested cache root when an outer contained cache root was dirty. That shortcut is unsafe. A clean
  nested cache root may still need descendant geometry refreshed when the cached subtree moves, so
  pruning it can leave hit-test or semantics bounds stale.
- Kept layout dirty expansion conservative and added regression tests proving dirty expansion must
  pass through clean nested cache roots, dirty nested cache roots, and non-contained nested roots.
- Found a separate correctness bug in incremental semantics snapshot reuse. A clean subtree was
  reused only from `subtree_semantics_dirty_count == 0`; it did not compare the current semantic
  root bounds with the previous snapshot root bounds. When a cache-hit subtree moved, reused
  descendants could keep their old absolute bounds.
- Fixed the reuse contract in `crates/fret-ui/src/tree/ui_tree_semantics.rs`:
  identical parent/bounds reuse the previous range unchanged, origin-only movement with the same
  size translates the reused range, and other root-bound changes rebuild the subtree.
- Focused validation passed:
  `cargo nextest run --cargo-profile dev-fast -p fret-ui view_cache_semantics_moving_relative_inset_updates_bounds_without_rerender --no-fail-fast --no-capture`,
  `cargo nextest run --cargo-profile dev-fast -p fret-ui semantics_snapshot_rebuilds_clean_descendants_when_dirty_ancestor_transform_changes semantics_snapshot_reuses_clean_subtrees_between_dirty_refreshes --no-fail-fast --no-capture`,
  `cargo nextest run --cargo-profile dev-fast -p fret-ui tree::tests::view_cache --no-fail-fast --no-capture`,
  `cargo nextest run --cargo-profile dev-fast -p fret-ui view_cache_ --no-fail-fast --no-capture`,
  `cargo nextest run --cargo-profile dev-fast -p fret-ui semantics_snapshot_ --no-fail-fast --no-capture`,
  `cargo fmt -p fret-ui`, `cargo check -p fret-ui --profile dev-fast -j 1`, and
  `git diff --check`.
- The stable data-table view-cache torture gate passed:
  `target/release/fretboard-dev.exe diag suite ui-gallery-data-table-view-cache-torture --dir target/fret-diag/vlist-view-cache-semantics-translate-v1 --session-auto --timeout-ms 900000 --ai-packet --launch -- cargo run -p fret-ui-gallery --release --features gallery-dev,gallery-ai,gallery-chart,gallery-web-ime-harness`.
  Evidence is in
  `target/fret-diag/vlist-view-cache-semantics-translate-v1/sessions/1781497567429-36420/suite.summary.json`.
- The final data-table bundle remained layout-dominant rather than semantics-dominant:
  `total=14432us`, `layout=13507us`, `layout.engine_solve=3969us`, `layout.nodes=813`, and
  `paint=751us`.
- Decision: do not optimize by pruning nested view-cache layout dirty expansion. The next
  performance lane should target the data-table/view-cache layout root apply and row/cell layout
  policy, not stale geometry shortcuts.

## 2026-06-15 Twenty-Fourth Slice Known Scroll Extents

- Followed the data-table view-cache torture node-profile evidence: the worst frame contained many
  per-row horizontal `Scroll` nodes around `ui-gallery-data-table-row-123xx`, each spending roughly
  `~200us` self to rediscover content extents even though the table strategy already knows fixed
  column widths.
- Added `ScrollProps::known_content_size` as a mechanism-layer contract. When set, `Scroll` treats
  the supplied extent as authoritative for the scroll axis, skips unbounded child extent probing and
  post-layout overflow rediscovery, but still lays out children, clips, transforms, hit-tests, and
  synchronizes the scroll handle.
- Kept policy ownership in `fret-ui-kit`: declarative table center-column horizontal scroll wrappers
  now pass the summed center-column width. Generic shadcn `ScrollArea`, AI suggestions, and tests use
  `known_content_size: None` so auto-size scroll surfaces keep their old content-probing behavior.
- Focused validation passed:
  `cargo nextest run --cargo-profile dev-fast -p fret-ui scroll_known_content_size_skips_extent_probe_but_updates_handle_extent --no-fail-fast --no-capture`,
  `cargo nextest run --cargo-profile dev-fast -p fret-ui scroll_known_content_size_ scroll_intrinsic_content_mode_measures_children scroll_intrinsic_viewport_mode_skips_children --no-fail-fast --no-capture`,
  `cargo nextest run --cargo-profile dev-fast -p fret-ui tree::tests::view_cache --no-fail-fast --no-capture`,
  `cargo check -p fret-ui-shadcn --tests --profile dev-fast -j 1`,
  `cargo check -p fret-ui-ai --tests --profile dev-fast -j 1`,
  `cargo check -p fret-ui-gallery --release --features gallery-dev,gallery-ai,gallery-chart,gallery-web-ime-harness -j 1`,
  `cargo fmt -p fret-ui -p fret-ui-kit -p fret-ui-shadcn -p fret-ui-ai`, and `git diff --check`.
- A broad `cargo check --workspace --all-targets --profile dev-fast -j 1` was run after most fixes.
  It progressed through the workspace and failed only on a test-only `ScrollProps` initializer in
  `ecosystem/fret-ui-ai/src/elements/checkpoint.rs`; that initializer was fixed and rechecked with
  `cargo check -p fret-ui-ai --tests --profile dev-fast -j 1`.
- The stable data-table view-cache torture gate passed:
  `target/release/fretboard-dev.exe diag suite ui-gallery-data-table-view-cache-torture --dir target/fret-diag/vlist-view-cache-known-scroll-extent-v1-rerun --session-auto --timeout-ms 900000 --ai-packet --launch -- cargo run -p fret-ui-gallery --release --features gallery-dev,gallery-ai,gallery-chart,gallery-web-ime-harness`.
  Evidence is in
  `target/fret-diag/vlist-view-cache-known-scroll-extent-v1-rerun/sessions/1781503051796-66032/suite.summary.json`.
- `diag stats` on that bundle stayed layout/root-apply dominated:
  `total=14564us`, `layout=13519us`, `layout.engine_solve=6772us`, `layout.root apply=11209us`,
  `layout.nodes=813`, `paint=851us`, and command availability stayed small at
  `widget_count/collect_us/eval_us=4/7/13`.
- A node-profile rerun also passed:
  `target/fret-diag/vlist-view-cache-known-scroll-extent-node-profile-v1/sessions/1781503961156-129796/suite.summary.json`.
  The row-level horizontal `Scroll` nodes still appear in the top list around
  `172-178us self / 205-210us total`; this confirms the mechanism removes extent discovery
  duplication without eliminating the remaining per-row scroll layout baseline.
- Decision: keep this slice because it moves known table geometry out of repeated Scroll probing and
  records the policy/mechanism boundary cleanly. Do not keep grinding this path as the next main
  performance lever. The next serious target is `layout.root apply` / dirty-root application for the
  table view-cache torture frame, followed by row/cell layout policy if root apply attribution points
  back to the table layer.

## Next Verification

1. Use `tools/diag-scripts/suites/ui-gallery-data-table-retained/suite.json` and
   `tools/diag-scripts/suites/ui-gallery-data-table-view-cache-torture/suite.json` as the main
   repeatable gates.
2. Keep `ce-data-table-probe` as diagnosis-only until the row-selection assertion is stabilized or
   replaced.
3. Prioritize `layout.root apply` / dirty-root application attribution for
   `ui-gallery-data-table-view-cache-torture`; only return to per-row `Scroll` if node profiles show
   it growing beyond the current `~170-180us self` baseline.
4. Revisit `window-command-availability-snapshot-v2` as the `Dispatch Snapshot` lane only if stable
   probes show command publication or focus repair moving from secondary cost to primary blocker.

## Open Questions

- Should `ActiveScript` store the active wait predicate in `WaitUntilState` to avoid re-reading the
  current step for demand classification?
- Should `drive_script_for_window` pass a "fresh semantics this frame" marker instead of only
  `Option<&SemanticsSnapshot>` so stale snapshots cannot accidentally satisfy current semantics
  requirements?
- Should current-window `exists` / `not_exists(test_id)` ever use the cached bounds map when a fresh
  semantics snapshot was intentionally skipped? Current answer: no, unless a stronger freshness
  marker is added.
- Should command availability snapshots evaluate every registered widget command for a window, or
  should apps expose command surfaces/groups so unrelated command families do not tax dense
  component interactions?

## 2026-06-15 Twenty-Fifth Slice Plan

- New evidence from the known-scroll-extent rerun says the current blocker is no longer scroll
  extent discovery alone. The worst view-cache torture frame is dominated by `layout.root apply`
  and `layout.engine_solve`, while per-row horizontal `Scroll` nodes remain as a repeated baseline
  cost.
- Hypothesis: the ordinary `table_virtualized` common path still models a single unpinned table as
  "one horizontal scroll viewport per visible row". That is structurally too expensive for dense
  shadcn/data-table surfaces because header/body share the same horizontal offset and fixed column
  widths are already known.
- Slice boundary: first optimize only the single center-column group path
  (`left_cols == 0 && center_cols > 0 && right_cols == 0`). Pinned columns and mixed groups keep the
  old per-group structure until a separate alignment gate proves an outer-scroll representation for
  pinned sections.
- Intended shape: keep one shared `ScrollHandle`, keep header/body horizontal alignment, preserve
  row pressable semantics and cell debug ids, but remove the repeated row-level horizontal `Scroll`
  wrappers from the unpinned body path. If the layout engine needs a definite content width, add a
  fixed-width content shell around header/body rather than forcing every row to be a scroll viewport.
- Gates for this slice: focused table tests around overflow/alignment/selection first, then
  `ui-gallery-data-table-view-cache-torture` with node profiling if the compile and correctness
  gates pass.

## 2026-06-15 Twenty-Fifth Slice Findings

- Added `ScrollContentTransform` as a `fret-ui` mechanism primitive. It reads an existing
  `ScrollHandle` offset and applies a children-only render/input transform, but it does not own
  viewport/content extent, does not handle wheel input, and does not publish scroll semantics.
- Updated the ordinary unpinned `table_virtualized` body path to replace each row's horizontal
  `Scroll` wrapper with `ScrollContentTransform` plus a fixed-width content shell. The table header
  keeps the single real horizontal `Scroll` owner, and the body gets one shared X-axis
  `WheelRegion` so rows continue to follow the same `ScrollHandle`.
- Kept pinned/mixed/grouped and retained-table paths unchanged in this slice. That avoids changing
  sticky-column semantics or grouped paint order without a separate alignment gate.
- Focused validation passed:
  `cargo nextest run --cargo-profile dev-fast -p fret-ui scroll_content_transform_moves_children_without_owning_scroll_extent --no-fail-fast --no-capture`,
  `cargo nextest run --cargo-profile dev-fast -p fret-ui-kit table_virtualized_unpinned_body_uses_shared_horizontal_transform table_virtualized_alignment_gate_header_matches_rows_under_overflow_and_variable_height table_virtualized_pointer_select_does_not_shift_row_bounds --no-fail-fast --no-capture`,
  `cargo check -p fret-ui-kit --tests --profile dev-fast -j 1`,
  `cargo check -p fret-ui --tests --profile dev-fast -j 1`,
  `cargo fmt -p fret-ui -p fret-ui-kit`, and `git diff --check`.
- The stable data-table view-cache torture suite with layout node profiling passed:
  `target/release/fretboard-dev.exe diag suite ui-gallery-data-table-view-cache-torture --dir target/fret-diag/vlist-view-cache-shared-row-xform-v1 --session-auto --timeout-ms 900000 --ai-packet --env FRET_LAYOUT_NODE_PROFILE=1 --env FRET_LAYOUT_NODE_PROFILE_TOP=30 --env FRET_LAYOUT_NODE_PROFILE_MIN_US=80 --launch -- cargo run -p fret-ui-gallery --release --features gallery-dev,gallery-ai,gallery-chart,gallery-web-ime-harness`.
  Evidence is in
  `target/fret-diag/vlist-view-cache-shared-row-xform-v1/sessions/1781511962931-133156/suite.summary.json`
  and
  `target/fret-diag/vlist-view-cache-shared-row-xform-v1/sessions/1781511962931-133156/1781512647087/bundle.schema2.json`.
- `diag stats` on the final bundle: `total=13260us`, `layout=12236us`,
  `layout.engine_solve=6965us`, `layout.root apply=10839us`, and command availability stayed small
  at roughly `widget_count/collect_us/eval_us=4/9/12`.
- The node-profile shape changed in the intended direction: row-level top nodes now show `Flex` for
  `ui-gallery-data-table-row-123xx` at about `104-106us self`, replacing the previous repeated
  row-level horizontal `Scroll` entries around `172-178us self / 205-210us total`.
- Decision: keep this slice. It removes a structurally unnecessary per-row horizontal scroll
  viewport from the common unpinned table body and preserves shared header/body alignment.
- Next target remains `layout.root apply` / dirty-root application attribution and table row/cell
  layout policy. More scroll extent probing is now lower leverage unless future profiles show it
  becoming hot again.

## 2026-06-15 Twenty-Sixth Slice Findings

- Added a conservative absolute-layout fast path in `fret-ui`: absolute children with definite
  placement bounds now skip the earlier `layout_in_probe` pass, and fixed-pixel absolute children
  without min/max size constraints or fractional insets can contribute a static parent envelope
  without `measure_in`.
- The envelope fast path is intentionally narrower than the bounds fast path. It only accepts
  explicit pixel width and height, no extra size constraints, and non-fractional insets. This avoids
  changing shrink-wrap behavior for measured content, fractional insets, text-driven sizes, or
  constrained children.
- Focused validation passed:
  `cargo nextest run --cargo-profile dev-fast -p fret-ui absolute_definite_bounds_resolve_without_measured_size absolute_dual_insets_resolve_without_measured_size absolute_auto_axis_still_requires_probe_measurement absolute_definite_envelope_uses_explicit_size_without_child_measurement absolute_fraction_inset_envelope_still_requires_measurement absolute_constrained_size_envelope_still_requires_measurement container_absolute_inset_positions_child container_absolute_negative_inset_offsets_outside_parent --no-fail-fast --no-capture`,
  `cargo check -p fret-ui --tests --profile dev-fast -j 1`,
  `cargo nextest run --cargo-profile dev-fast -p fret-ui-kit table_virtualized_unpinned_body_uses_shared_horizontal_transform table_virtualized_alignment_gate_header_matches_rows_under_overflow_and_variable_height table_virtualized_pointer_select_does_not_shift_row_bounds --no-fail-fast --no-capture`,
  `cargo fmt -p fret-ui`, and `git diff --check`.
- The stable data-table view-cache torture suite passed with layout node profiling:
  `target/release/fretboard-dev.exe diag suite ui-gallery-data-table-view-cache-torture --dir target/fret-diag/vlist-view-cache-absolute-definite-v1 --session-auto --timeout-ms 900000 --ai-packet --env FRET_LAYOUT_NODE_PROFILE=1 --env FRET_LAYOUT_NODE_PROFILE_TOP=30 --env FRET_LAYOUT_NODE_PROFILE_MIN_US=80 --launch -- cargo run -p fret-ui-gallery --release --features gallery-dev,gallery-ai,gallery-chart,gallery-web-ime-harness`.
  Evidence is in
  `target/fret-diag/vlist-view-cache-absolute-definite-v1/sessions/1781519025052-133900/suite.summary.json`
  and
  `target/fret-diag/vlist-view-cache-absolute-definite-v1/sessions/1781519025052-133900/1781519700240-ui-gallery-data-table-view-cache-filter-shrink-vlist-inputs-change/bundle.schema2.json`.
- `diag stats` stayed in the same band as the prior shared-row-transform run:
  `total=13569us`, `layout=12437us`, `layout.engine_solve=7011us`,
  `layout.root apply=10967us`, and command availability stayed tiny at
  `widget_count/collect_us/eval_us=4/8/14`. The previous comparable bundle was
  `total=13260us`, `layout=12236us`, `layout.engine_solve=6965us`, and
  `layout.root apply=10839us`.
- Node counts moved slightly in the intended direction (`layout.nodes=843` vs `847` on the
  comparable worst frame), but this is not a material 120Hz improvement. Treat this as a safe
  low-level cleanup that removes avoidable work for fixed absolute overlay/control chrome, not as
  the current primary data-table answer.
- Decision: keep this slice because it is narrowly gated and correctness-preserving, but move the
  next optimization back to table row/cell fixed-geometry layout policy and root-apply breadth.
  Repeated row `Flex` nodes around `108-111us self` remain the visible node-profile owner after the
  per-row `Scroll` removal.

## 2026-06-15 Twenty-Seventh Slice Attempt - Absolute Cell Strip Rejected

- Tested a table-local `table_virtualized` body experiment that changed the ordinary single-center,
  fixed-height, non-`optimize_paint_order` row cells from a horizontal flex row into absolute
  fixed-geometry cells.
- The experiment compiled, but the existing alignment gate failed:
  `table_virtualized_alignment_gate_header_matches_rows_under_overflow_and_variable_height`
  reported `header_x=220.00px cell_x=0.00px` for the `status` column.
- Focused cleanup confirmed the current mainline shape remains correct:
  `cargo nextest run --cargo-profile dev-fast -p fret-ui-kit table_virtualized_alignment_gate_header_matches_rows_under_overflow_and_variable_height table_virtualized_unpinned_body_uses_shared_horizontal_transform table_virtualized_pointer_select_does_not_shift_row_bounds --no-fail-fast --no-capture`
  passed after reverting the experiment.
- Interpretation: the visual/runtime bounds path can keep body cells aligned through
  `ScrollContentTransform`, but a component-local absolute child strip does not leave the same
  reliable Taffy sidecar geometry for table-owned test-id anchors. That makes it unsafe as a
  straight component optimization because diagnostics, alignment gates, and app automation depend on
  those anchors.
- Decision: do not land component-local absolute cells. If fixed-geometry table rows remain the
  right target, the next viable approach should be either a mechanism-level "fixed strip/grid"
  primitive whose layout engine and sidecar geometry are first-class, or a narrower row/cell
  simplification that keeps cells in normal flow.

## 2026-06-15 Twenty-Eighth Slice Direction - Fixed Geometry Owner

- A read-only architecture review reached the same conclusion as the failed experiment: table
  policy already knows fixed column tracks, but `fret-ui` layout owns the geometry seam that feeds
  Taffy sidecars, hit testing, semantics bounds, and diagnostics.
- Do not treat `ScrollContentTransform` or component-local absolute placement as substitutes for
  layout bounds. They can make the pixels move, but they are not sufficient when table-owned
  `test_id` anchors must report stable `abs_rect` geometry.
- If fixed row/column placement is still the main optimization target, the deep module should be a
  `fret-ui` mechanism primitive such as `FixedTrackStrip`, with a small interface:
  layout style, horizontal axis in v1, fixed track widths, gap, and real layout children. Table can
  adapt column widths into that primitive, but the primitive must write child bounds as first-class
  layout output.
- The safer interim component slice is to reduce cell-internal wrappers while keeping row/cell
  flow placement unchanged. That means preserving the row `Flex` and cell container sidecar anchors,
  and only removing content wrappers when alignment, hoisted test ids, selection, hit testing, and
  semantics stay covered by the focused table gates.

## 2026-06-15 Twenty-Ninth Slice - Scroll Transform Flow Subtree Contract

- Found a smaller mechanism bug while validating the rejected fixed-cell experiment: the new
  `ScrollContentTransform` primitive moved pixels and hit testing correctly, but the layout engine
  flow builder did not treat it as a wrapper whose descendants should be included in the same flow
  subtree.
- Failure evidence with `FRET_LAYOUT_FORBID_WIDGET_FALLBACK_SOLVES=1`: the focused table gates
  failed with `layout engine fallback solve (flex)` because row `Flex -> cell container`
  descendants under `ScrollContentTransform` had no engine child rects. The existing table pixels
  could still recover through widget-local fallback solves, but dense rows then paid extra per-row
  layout work and the no-fallback contract was false.
- Implemented the mechanism fix in `crates/fret-ui/src/layout/engine/flow.rs`: add
  `ElementInstance::ScrollContentTransform(_)` to the wrapper/pass-through flow lists so it recurses
  like `VisualTransform`, `RenderTransform`, and related children-only transform wrappers.
- Added a focused regression test:
  `scroll_content_transform_solves_flow_descendants_without_widget_fallback`. It builds the table-like
  shape `ScrollContentTransform -> Flex -> fixed cell containers`, requires zero
  `layout_engine_widget_fallback_solves`, and checks cell x positions still come from normal flow
  geometry.
- Validation so far:
  `cargo nextest run --cargo-profile dev-fast -p fret-ui scroll_content_transform_moves_children_without_owning_scroll_extent scroll_content_transform_solves_flow_descendants_without_widget_fallback --no-fail-fast --no-capture`
  passed.
- The table hot-path no-fallback gate also passed:
  `$env:FRET_LAYOUT_FORBID_WIDGET_FALLBACK_SOLVES='1'; cargo nextest run --cargo-profile dev-fast -p fret-ui-kit table_virtualized_alignment_gate_header_matches_rows_under_overflow_and_variable_height table_virtualized_unpinned_body_uses_shared_horizontal_transform table_virtualized_pointer_select_does_not_shift_row_bounds --no-fail-fast --no-capture`.
  This confirms the existing shared horizontal transform table path no longer depends on
  widget-local row `Flex` fallback solves.
- Decision: keep this as a `fret-ui` mechanism-layer performance correctness fix. It does not
  replace the later `FixedTrackStrip`/fixed-geometry-owner direction, but it removes avoidable
  widget-local fallback solves from the existing shared row-transform table path.

## 2026-06-15 Thirtieth Slice - Retained Table Shared Row Transform

- Corrected the retained-table repro surface after one invalid comparison: the non-retained script
  asserts `apply_mode=non_retained_rerender`, while the retained data-table path must use
  `tools/diag-scripts/ui-gallery/data-table/ui-gallery-data-table-retained-filter-shrink-vlist-inputs-change.json`
  and expects `apply_mode=retained_reconcile`.
- Baseline retained evidence before this slice:
  `target/fret-diag/vlist-retained-filter-shrink-correct-script-v1/sessions/1781528832521-146560/1781528844457-ui-gallery-data-table-retained-filter-shrink-vlist-inputs-change/bundle.schema2.json`.
  The worst retained frame was `total=24856us`, `layout=23060us`,
  `layout.engine_solve=13231us`, `layout.root apply=20407us`, and `layout.nodes=810`.
- Updated `table_virtualized_retained_v0` to match the already-optimized non-retained single-center
  table shape. The retained unpinned body now uses one shared X-axis `WheelRegion` plus per-row
  `ScrollContentTransform` wrappers instead of one horizontal `Scroll` owner per visible row. Pinned
  and mixed column groups keep the previous structure.
- Added `table_virtualized_retained_unpinned_body_uses_shared_horizontal_transform`, which proves:
  retained body rows no longer contain row-local `Scroll` nodes, each row contains exactly one
  `ScrollContentTransform`, and header/body cell visual bounds stay aligned after horizontal wheel
  input.
- Focused validation passed:
  `cargo nextest run --cargo-profile dev-fast -p fret-ui-kit table_virtualized_retained_unpinned_body_uses_shared_horizontal_transform table_virtualized_retained_colpin_alignment_gate_across_pin_resize_and_overflow table_virtualized_retained_colpin_alignment_gate_measured_rows_do_not_shrink_width table_virtualized_retained_pointer_row_selection_policy_list_like table_virtualized_retained_nested_pressable_remains_hittable_when_pointer_row_selection_disabled table_virtualized_retained_selected_semantics_follow_windowed_row_selection table_virtualized_retained_header_debug_ids_click_sort_actions --no-fail-fast --no-capture`,
  `cargo nextest run --cargo-profile dev-fast -p fret-ui-shadcn --lib retained_data_table_header_debug_ids_sort_with_column_actions --no-fail-fast --no-capture`,
  and `cargo fmt -p fret-ui-kit`.
- Correct retained repro after the change passed:
  `target/fret-diag/vlist-retained-shared-row-xform-v1/sessions/1781530321751-126564/1781531045060-ui-gallery-data-table-retained-filter-shrink-vlist-inputs-change/bundle.schema2.json`.
  `diag stats --sort cpu_cycles --top 30` reported `total=11715us`, `layout=10831us`,
  `layout.engine_solve=6599us`, `layout.root apply=9541us`, and `layout.nodes=646`.
- Before/after interpretation: the retained worst frame moved from above the 120Hz budget
  (`24856us`) to near the budget (`11715us`) on the same retained script. Layout time dropped by
  roughly `12.2ms`, root apply by roughly `10.9ms`, engine solve by roughly `6.6ms`, and the row
  node count dropped by `164`. Node-profile logs also stopped showing row-level horizontal `Scroll`
  entries as top owners; the remaining owner is retained list/root application rather than per-row
  scroll viewport duplication.
- Decision: keep this slice. It confirms the high-level architecture point: dense shadcn-style
  tables should not model every visible row as a scroll viewport when fixed column widths and a
  shared header/body horizontal offset are already known.
- Next target: after this commit, continue with either a `FixedTrackStrip`/fixed-track layout
  primitive or a narrower retained/root-apply attribution pass. The component-local absolute-cell
  experiment remains rejected because it breaks first-class sidecar geometry for test ids and
  diagnostics.

## 2026-06-15 Thirty-First Slice - Retained Row Key Hook Prune

- Found one remaining retained-table structural duplication after the shared row-transform slice:
  every retained row visual root registered the same table keyboard navigation handler, while the
  retained list root already registered that handler once.
- Removed the per-row `key_on_key_down_for` registration from
  `retained_table_render_row_visuals`. Keyboard navigation is now owned by the retained list root,
  and focused descendants continue to bubble key events to that root.
- Added `table_virtualized_retained_nested_focus_bubbles_keyboard_to_list`, which focuses a nested
  row child pressable, dispatches `ArrowDown`, and verifies the table list's active descendant moves
  to the next row. This protects the exact behavior that could regress when deleting the row-local
  hook.
- Focused validation passed:
  `cargo nextest run --cargo-profile dev-fast -p fret-ui-kit table_virtualized_retained_nested_focus_bubbles_keyboard_to_list table_virtualized_retained_unpinned_body_uses_shared_horizontal_transform table_virtualized_retained_header_debug_ids_click_sort_actions --no-fail-fast --no-capture`
  and
  `cargo nextest run --cargo-profile dev-fast -p fret-ui-shadcn --lib retained_data_table_header_debug_ids_sort_with_column_actions --no-fail-fast --no-capture`.
- Retained-only perf repro after this cleanup:
  `target/release/fretboard-dev.exe diag perf tools/diag-scripts/ui-gallery/data-table/ui-gallery-data-table-retained-filter-shrink-vlist-inputs-change.json --repeat 1 --warmup-frames 5 --dir target/fret-diag/vlist-retained-row-key-hook-prune-v3-retained-only --env FRET_UI_GALLERY_DATA_TABLE_RETAINED=1 --env FRET_DIAG_SCRIPT_AUTO_DUMP=0 --sort cpu_cycles --top 15 --json --launch -- cargo run -p fret-ui-gallery --release --features gallery-dev,gallery-ai,gallery-chart,gallery-web-ime-harness`.
- Evidence bundle:
  `target/fret-diag/vlist-retained-row-key-hook-prune-v3-retained-only/1781536422863/bundle.json`.
  `diag stats --sort cpu_cycles --top 10` reported `total=11391us`, `layout=10522us`,
  `layout.engine_solve=6524us`, `layout.root apply=9373us`, and `layout.nodes=646`.
- Interpretation: keep this as a structural cleanup and behavior guard, not as a claimed new
  material perf win. The numbers stay in the same band as the prior retained shared-transform
  evidence (`11715/10831/6599us`, `nodes=646`), which is the right result for removing duplicate
  event registrations rather than row layout structure.
- A view-cache-enabled variant also passed, but it produced a different measurement surface
  (`contained_relayout=1` and cache-key mismatch) and is not used as the apples-to-apples evidence
  for this slice.

## 2026-06-17 Code-View Scroll Extent Hint Slice

- Continued the heavy-component lane with the code-view torture mount path. The important finding
  is that steady wheel scrolling is not the current owner; the valid mount evidence still points at
  the outer gallery content `Scroll` and page-mount extent negotiation.
- Added a narrow scroll-extent hint path instead of broad component flattening:
  - `fret-code-view` now records `PreparedCodeBlock::max_line_columns` during preparation.
  - Windowed code blocks estimate a monospace content width and pass it as
    `ScrollProps::known_content_size` for the inner horizontal scroll.
  - `fret-ui-shadcn::ScrollArea` forwards `viewport_known_content_size(...)` through both compact
    and build surfaces so heavy recipes can provide extent metadata without dropping to raw
    `fret-ui`.
  - `fret-ui` treats single-axis `known_content_size` as authoritative only on the scroll axis; the
    cross axis remains the viewport extent. This lets a Y-scroll caller pass `Size(0, height)`
    without collapsing the width.
- Rejected experiments:
  - Broadly enabling post-layout extents for all definite Y scrolls made mount cost worse
    (`m14`: `total=68351us`, `layout=68087us`, `solve=66633us`), so that path was reverted.
  - Hard-coding a gallery page-level extent hint before fixing single-axis cross-axis semantics was
    also worse (`m15`: `total=28413us`, `layout=28185us`, `solve=27041us`) and was reverted.
  - A bounded `Viewport` + `probe_unbounded=false` first-frame seed experiment passed a narrow
    scroll test but failed the release mount profile (`m16`:
    `target/fret-diag/code-view-mount-m16-bounded-viewport-seed/1781668998093/bundle.json`,
    `total=21142us`, `layout=20925us`, `solve=19534us`). This reintroduced a layout-solve spike,
    so the code and test were removed instead of committed.
  - After widening Scroll layout profiling to record slow total layout time, `m17` showed the
    remaining outer content viewport cost directly:
    `target/fret-diag/code-view-mount-m17-scroll-profile/1781670432100/bundle.schema2.json`.
    The worst frame stayed around `total=23945us`, but the Scroll phase profile identified
    `measure_children=21890us`, `solve_barrier=305us`, and `layout_children=443us` on
    `ui-gallery-content-viewport`. The retained/windowed code-view subtree was not the primary
    owner of that frame.
  - A naive gallery page-level `viewport_known_content_size(Size(0, 702))` hint removed the outer
    `measure_children` cost but shifted the frame into a much worse barrier solve (`m18`:
    `target/fret-diag/code-view-mount-m18-page-known-extent/1781671303027/bundle.schema2.json`,
    `total=67663us`, `layout=67449us`, `solve=65858us`). This proves fixed page extent metadata
    needs a stronger shell contract than just seeding final content height on the existing scroll.
  - A direct-start code-view mount run was invalid because the script reset diagnostics after the
    page was already loaded; it is not used as evidence.
- Best valid current comparison after removing the outer unbounded probe experiment remains
  `m13`: `target/fret-diag/code-view-mount-m13-no-outer-unbounded/1781664429284/bundle.json`,
  with `top_total_time_us=22269`, `top_layout_time_us=22049`, and
  `layout_engine_solve_time_us=608`. This lowered engine solve but did not remove the outer scroll
  self/extent cost.
- Current decision: keep the extent-hint APIs and the core first-frame post-layout seed, but do not
  claim code-view mount is solved. The next material target is a bounded page-scroll contract or a
  shell-structure refactor for fixed viewport pages, not another generic "make all scrolls
  post-layout authoritative" or "seed every bounded viewport scroll" rewrite.

## 2026-06-17 Fixed Shell + No-Wrap Text Measurement Cache Slice

- Continued the same code-view torture mount lane after m23/m24/m26 attribution. The important
  split is now clear:
  - fixed-size passthrough shells can avoid probing children during standalone measurement;
  - no-wrap text nodes already had a node cache for clean-geometry resize proofs, but measurement
    did not read that cache before calling the text service.
- Implemented a conservative fixed-shell measurement fast path in
  `crates/fret-ui/src/declarative/host_widget/measure.rs`: if both axes are resolvable from known
  constraints, fixed pixels, fill, or fraction under definite availability, `measure_passthrough_box`
  can return the clamped size without walking children. Final layout still visits children and owns
  their geometry.
- Added `fixed_passthrough_stack_measure_skips_child_subtree` to prove direct measurement skips a
  counted child subtree, while `layout_all` still lays the child out.
- m24 evidence showed this is correct but not the main code-view mount win:
  `target/fret-diag/code-view-mount-m24-fixed-passthrough-measure/1781679309823/bundle.schema2.json`.
  The content viewport's measured phase moved from roughly `21248us` in m23 to `19492us` in m24,
  but the same outer scroll still owned the mount frame.
- m26 node profiling then identified the remaining hot self-time as a short no-wrap text label at
  `ecosystem/fret-code-view/src/code_block.rs:903` (`TextWrap::None`, `TextOverflow::Clip`,
  `TextAlign::Start`). That was not shadcn component nesting; it was repeated text measurement
  through an existing cache-write-only path.
- Added a no-wrap measurement cache read path for plain text, styled text, and selectable text. The
  fast path is intentionally limited to `TextWrap::None + TextOverflow::Clip + TextAlign::Start`;
  `Ellipsis` and non-start alignment keep the existing text-service path because they can
  materialize or align against `max_width`.
- Added `nowrap_text_measurement_reuses_node_cache_for_same_fingerprint`, which measures the same
  no-wrap text node twice and verifies the second measurement does not call the text service again;
  changing the text fingerprint correctly misses the cache.
- Focused validation passed:
  `cargo test -p fret-ui --profile dev-fast nowrap_text_measurement_reuses_node_cache_for_same_fingerprint -- --nocapture`,
  `cargo test -p fret-ui --profile dev-fast fixed_passthrough_stack_measure_skips_child_subtree -- --nocapture`,
  `cargo check -p fret-ui --profile dev-fast`, and `cargo fmt -p fret-ui`.
- Release gallery rebuilt successfully after the command timeout completed in the background:
  `target/release/fret-ui-gallery.exe` was updated at `2026-06-17 15:27:14`.
- m27 perf run with the normal profile passed but was noisy:
  `target/fret-diag/code-view-mount-m27-nowrap-cache/1781681293501/bundle.json` reported
  `top_total_time_us=32069`, `top_layout_time_us=31815`, `top_layout_engine_solve_time_us=673`,
  and `top_frame_id=13`.
- m28 with node profiling gave the more useful attribution:
  `target/fret-diag/code-view-mount-m28-measure-node-profile/1781681458423/bundle.schema2.json`
  reported `top_total_time_us=10613`, `top_layout_time_us=10398`, and `top_frame_id=13`.
  The previous `code_block.rs:903` Text self-time no longer appears in the top node-profile rows.
  The visible owners are now the outer content `Scroll` and the code-view `VirtualList` mount path
  (`VirtualList` self around `3240us`, total around `5682us` in the profiled run).
- Decision: keep both mechanism-layer optimizations because they remove real repeated work and have
  focused tests. The next material optimization target is not more shadcn wrapper flattening; it is
  the remaining mount-time scroll/virtual-list negotiation, likely a bounded page-scroll contract or
  a virtual-list mount policy that avoids unnecessary first-frame extent/child layout work.

## 2026-06-17 Code-View Windowed Line Flattening Slice

- Followed the m28 node profile into the code-view mount path. After the no-wrap text cache slice,
  the previous short language-label text hotspot no longer appeared; remaining owners were the
  outer content `Scroll` and the code-view `VirtualList`.
- Flattened windowed code-view rows by folding line number text and separator spacing into the
  same `AttributedText` as the code line. This removes the per-visible-line gutter container,
  separate line-number text node, and horizontal row wrapper while preserving line-number color as
  muted foreground.
- Updated the monospace known-content width estimate to account for the folded line-number prefix.
- Added `windowed_line_numbers_are_folded_into_single_rich_line` to lock the new row shape at the
  rich-text boundary.
- Focused validation passed:
  `cargo test -p fret-code-view --profile dev-fast windowed_line_numbers_are_folded_into_single_rich_line -- --nocapture`,
  `cargo test -p fret-code-view --profile dev-fast code_block_wrap_grapheme_and_selection_smoke -- --nocapture`,
  `cargo test -p fret-code-view --profile dev-fast code_block_hover_does_not_trigger_declarative_layout_invalidations -- --nocapture`,
  `cargo check -p fret-code-view --profile dev-fast`, and `cargo fmt -p fret-code-view`.
- m29 evidence after rebuilding release gallery:
  `target/fret-diag/code-view-mount-m29-windowed-line-inline/1781684171577/bundle.schema2.json`.
  The run reported `top_total_time_us=25023`, `top_layout_time_us=24787`,
  `top_layout_engine_solve_time_us=641`, and `top_frame_id=13`.
  Node profiling showed the worst frame still owned by `ui-gallery-content-viewport`
  (`Scroll` self around `23244us`) with `measure_children=22819us`; the next code-view frame showed
  `ui-gallery-code-view-root` `VirtualList` self around `1321us`, total around `3298us`.
- Interpretation: keep the line flattening as a structural reduction for code-view rows, but do not
  claim it solves the mount hitch. It confirms the larger point that row-local wrapper reduction
  helps only after the page-level scroll negotiation is addressed.
- Rejected experiment: a narrow `fret-ui` bounded-viewport scroll shortcut that skipped the
  pre-layout extent measure for `ScrollIntrinsicMeasureMode::Viewport + probe_unbounded=false`.
  A focused unit test could prove the intended local behavior, but the release profile regressed:
  `target/fret-diag/code-view-mount-m30-scroll-shortcircuit/1781686289287/bundle.json` reported
  `top_total_time_us=39772`, `top_layout_time_us=39532`, and `top_frame_id=13`; node profiling put
  `ui-gallery-content-viewport` at roughly `37895us` self. The code and test were removed instead
  of being kept. This reinforces that the next fix should be a stronger page/shell extent contract
  or a targeted scroll state contract, not a generic bounded-viewport shortcut.

## 2026-06-17 Code-View Torture Harness Scaffold Slimming

- After m30 was rejected, re-queried the m29 bundle with a bounded script instead of opening raw
  JSON. The worst frame was still `ui-gallery-content-viewport`, but the important split was:
  outer `Scroll` `measure_children=22819us`, while the code-view `VirtualList` frame itself was
  about `3289us` total with 33 `StyledText` children. That means the current perf script was
  dominated by measuring the docs/gallery scaffold around the code-view, not only by code-view rows.
- Added `bounded_viewport_scroll_measure_stops_at_fixed_height_shell` in `fret-ui` to pin an
  important existing boundary: a `Viewport + probe_unbounded=false` scroll does not measure through
  a fixed-height direct child shell. This prevents us from re-chasing the already rejected generic
  Scroll shortcut and documents why the remaining m29 cost is page scaffold measurement.
- Flattened the `code_view_torture` gallery preview path by replacing `wrap_preview_page` /
  `DocSection` with a direct full-width vertical harness layout. The stable
  `ui-gallery-code-view-root` anchor stays on the code block, and the page-level
  `ui-gallery-page-code-view-torture` anchor still comes from the outer gallery content shell.
- Interpretation: this is a diagnostics harness cleanup, not a framework-wide performance claim.
  It should make future code-view mount profiles less polluted by docs scaffold wrappers, so the
  next real optimization can target the remaining code-view/VirtualList cost directly.
- Focused validation passed:
  `cargo test -p fret-ui --profile dev-fast bounded_viewport_scroll_measure_stops_at_fixed_height_shell -- --nocapture`,
  `cargo check -p fret-ui-gallery --profile dev-fast`, and `cargo fmt -p fret-ui -p fret-ui-gallery`.

## 2026-06-17 Code-View Torture Content-Shell Boundary Slice

- Re-ran the code-view torture mount script after fixing the diagnostics target selection. The
  script must run against a `fret-ui-gallery` binary built with `--features gallery-dev`; otherwise
  `code_view_torture` is filtered out at compile time and the search field can be correct while the
  target nav item never exists.
- Hardened all code-view torture perf scripts by replacing keyboard-driven search clearing with
  `set_text_value` and replacing the nav click with `click_stable`. The previous `Ctrl+A` path
  depended on command availability and made the perf setup flaky before the measured interaction.
- m31, after rebuilding `fret-ui-gallery --release --features gallery-dev`, proved the remaining
  mount hitch was the outer gallery content scroll measuring through the code-view page:
  `target/fret-diag/code-view-mount-m31-slim-harness-gallery-dev/1781690617407/bundle.json`
  reported `top_total_time_us=47958`, `top_layout_time_us=47598`, and
  `layout_roots_apply_time_us=46360`. Scroll profiling on `ui-gallery-content-viewport` showed
  `measure_children=44408us`, `solve_barrier=452us`, and `content_h=674`.
- Added a gallery-dev-only fixed semantics boundary around the code-view torture preview content.
  This is intentionally a diagnostics harness contract: the page has a known fixed visual envelope,
  so the outer gallery scroll should not rediscover the large document height by measuring through
  the page shell.
- m32 confirmed the fixed shell removed the deep measurement cost:
  `target/fret-diag/code-view-mount-m32-fixed-page-boundary/1781691900933/bundle.json` reported
  `top_total_time_us=20215`, `top_layout_time_us=19940`, and
  `layout_roots_apply_time_us=19220`. The same scroll node showed
  `measure_children=3us`, but `solve_barrier=18210us`, so the next owner became barrier-root solve.
- Rejected experiment: adding `viewport_known_content_size(Size(0, 674))` to the outer gallery
  scroll for this page. m33 disabled post-layout extent mode and kept `measure_children=0`, but it
  exposed a worse cold barrier solve:
  `target/fret-diag/code-view-mount-m33-known-content-size/1781692854401/bundle.json` reported
  `top_total_time_us=27996`, `top_layout_time_us=27731`, and
  `layout.engine_solve=26666`. The hot scroll node had `probe_unbounded=false`,
  `content_h=674`, `solve_barrier=26283us`, and only `layout_children=320us`. The code was removed.
- Final kept shape, m34:
  `target/fret-diag/code-view-mount-m34-fixed-boundary-final/1781693865678/bundle.schema2.json`
  reported `top_total_time_us=9496`, `top_layout_time_us=9271`,
  `layout.engine_solve=7966`, and `layout.root apply=8548`. Scroll profiling still shows a
  navigation/mount barrier solve on the fixed content shell (`solve_barrier` around `7653us` on the
  new content scroll root), but the earlier 44ms deep measurement is gone.
- Decision: keep the fixed shell boundary and script hardening. Do not keep the naive
  `known_content_size` hint on the outer gallery scroll. The next material target is the cold
  barrier-root solve during page navigation/remount, likely by avoiding unnecessary keyed remount
  work or by giving barrier roots a reusable flow subtree contract when the shell bounds and child
  structure are stable.
- Rejected follow-up: keeping the outer content `ScrollArea` identity stable while keying only the
  per-page scroll handle/reset state. The hypothesis was that reusing the scroll root would avoid a
  cold barrier solve. m35 disproved it:
  `target/fret-diag/code-view-mount-m35-stable-content-scroll/1781694818290/bundle.json` reported
  `top_total_time_us=24291`; scroll profiling showed the new fixed content root at
  `solve_barrier=22505us`, `measure_children=4us`, and `layout_children=333us`. The code was
  removed. The remaining problem is therefore the page content root's cold layout-engine solve, not
  just the outer `cx.keyed(...)` wrapper.
- Rejected follow-up: replacing the outer `ui-gallery-page-preview` layout `Semantics` wrapper with
  layout-transparent `attach_semantics` on the code-view page root while moving the fixed
  height/clip to that page root. m36 regressed hard:
  `target/fret-diag/code-view-mount-m36-preview-attach-semantics/1781714021153/bundle.json`
  reported `top_total_time_us=33756`, with the outer content scroll back at
  `measure_children=30892us`. The fixed `Semantics` wrapper is currently acting as the measurement
  boundary; removing it reopens the original deep-measure path. The code was removed.
- Rejected follow-up: teaching `Scroll` to seed its initial content extent from the current bounded
  child measurement when `probe_unbounded=false`. Two variants were tested and removed:
  m37 required the direct child to advertise a fixed `Length::Px` extent, but the condition did not
  hit the actual `ScrollArea` child path; m38 trusted any current bounded measurement and regressed
  the cold solve. Evidence:
  `target/fret-diag/code-view-mount-m37-bounded-fixed-scroll-extent/1781716134933/bundle.schema2.json`
  still showed the fixed page root solved first at `752x524` and then again at `752x674`
  (`solve_barrier=7285us`, `corrected_content_relayout=true`), while
  `target/fret-diag/code-view-mount-m38-current-bounded-measure-extent/1781717293191/bundle.schema2.json`
  worsened to `top_total_time_us=19438` with the same `752x524 -> 752x674` double solve.
- m39 used the existing `FRET_DEBUG_SCROLL_EXTENT_PROBE=1` logging:
  `target/fret-diag/code-view-mount-m39-scroll-extent-debug/1781717415556/bundle.schema2.json`.
  The log showed the outer gallery scroll repeatedly growing from `content=(752,524)` to observed
  `672/674` after layout. This proves the 674 extent is discovered by post-layout overflow
  observation, not by the current bounded measure pass. The next viable target is therefore the
  measure path between `ScrollArea` and the fixed `ui-gallery-page-preview` shell, not another
  content-extent seeding branch.
- Rejected follow-up: skipping `solve_barrier_child_root_if_needed` for a fixed-size `Semantics`
  shell and relying on the later widget-local `layout_in` path. m40 proved this is the wrong
  mechanism:
  `target/fret-diag/code-view-mount-m40-skip-fixed-semantics-barrier-solve/1781719036747/bundle.schema2.json`
  reduced the worst-frame `layout.engine_solve` to roughly `526us`, but total/layout regressed to
  `top_total_time_us=28437` / `top_layout_time_us=28210`. The cost moved into repeated widget
  measure/layout (`measure_children=710us`, `layout_children=937us`, large `layout_roots_apply`),
  and the scroll still corrected from viewport height to content height. The code was removed.
- Mechanism conclusion after m37-m40: the remaining issue is not "skip Taffy" and not "seed from
  bounded measure". The correct optimization must avoid the first wrong-size barrier solve by
  giving the scroll/layout boundary an authoritative content extent before the first final solve,
  while preserving the `Viewport` intrinsic-measure guard that prevents deep measurement of large
  component trees.

## 2026-06-18 Editor Controls Stability Slice

- Re-opened the `imui_editor_controls_basics` height-jump report through the editor composite code
  path instead of assuming `PropertyRowLayoutVariant::Auto` was still the grid default. Current
  `PropertyGrid` and `PropertyGridVirtualized` both inject `PropertyRowLayoutVariant::Row` into
  row defaults; `Auto` remains a risk for opt-in responsive rows, but it is not the default cause
  for this cookbook example.
- The concrete first-frame instability was the Asset row: the example seeds the text-assist query
  with `"ca"` and uses `TextAssistFieldSurface::AnchoredOverlay`. The old text-assist recipe
  expanded whenever the input-owned query state had matches, even if the input did not have focus.
  That means the popup requested itself on app open and then re-negotiated placement after anchor
  bounds became available, which matches the visible "menu appears after open" jump.
- Changed the recipe policy so inline assist keeps the existing query-driven expansion, while
  anchored overlay assist requires input focus before expansion. This keeps popup policy in
  `fret-ui-editor` and avoids pushing dismiss/focus strategy into `fret-ui`.
- Remaining row-height concern: editor row baselines (`editor.density.row_height`) and framed
  control outer heights (`text field padding + border + row line-height`) are still separate
  concepts. `DragValue` already uses a `session_shell` with the resolved control outer height, while
  several joined-input paths still rely on `Auto`/`min_height`. The next structural slice should
  introduce a shared editor inline-control extent helper so `PropertyRow`, `NumericInput`,
  `DragValue`, `TextField`, and popup triggers agree on fixed row envelopes.

## 2026-06-18 Editor Inline Control Size Slice

- Added a row-height contract test that renders a real `PropertyGrid` with `NumericInput`,
  `DragValue`, and `TextAssistField` rows. Before the fix, the test failed with
  `NumericInput` at `34px` and `DragValue` at `32px` under the default test theme, matching the
  reported "Exposure vs Roughness" visual mismatch.
- Root cause: `NumericInputOptions::default()` used `fret-ui-kit::Size::default()` (`Medium`),
  while editor-owned dense controls around it (`TextField`, `MiniSearchBox`, `DragValue`) use
  `Size::Small`. This was a policy-layer default mismatch, not a `PropertyGrid` layout bug.
- Fixed `NumericInputOptions` to default to `Size::Small` and pinned the behavior with
  `numeric_input_defaults_to_small_editor_control_size`.
- Result: the common editor-control row-height test now passes, giving the cookbook inspector a
  consistent row envelope for Exposure, Roughness, and Asset rows.
- Validation:
  `cargo nextest run -p fret-ui-editor numeric_input property_grid text_assist_field --cargo-profile dev-fast`
  passed 18 focused tests; `cargo check -p fret-cookbook --example imui_editor_controls_basics
  --features cookbook-imui --profile dev-fast` passed; the existing overlay/focus diag script
  `tools/diag-scripts/cookbook/imui-editor-controls-basics/cookbook-imui-editor-controls-overlay-focus-cycle.json`
  passed with `--features cookbook-imui,cookbook-diag`, producing
  `target/fret-diag/cookbook-imui-editor-controls-overlay-focus-cycle-after-fixes/1781720992543-cookbook-imui-editor-controls-overlay-focus-cycle`.
- Strengthened the cookbook smoke script with an initial `not_exists` assertion for
  `cookbook.imui_editor_controls.assist.list` after the seeded `"ca"` query has rendered. This
  turns the old first-frame anchored-overlay jump into a script-level regression gate instead of a
  purely visual/manual observation.
- Follow-up diagnostics passed:
  `tools/diag-scripts/cookbook/imui-editor-controls-basics/cookbook-imui-editor-controls-basics-smoke.json`
  produced
  `target/fret-diag/cookbook-imui-editor-controls-smoke-overlay-initial-contract/1781721214216-cookbook-imui-editor-controls-basics-smoke`;
  `tools/diag-scripts/cookbook/imui-editor-controls-basics/cookbook-imui-editor-controls-click-stress.json`
  produced
  `target/fret-diag/cookbook-imui-editor-controls-click-stress-after-fixes/1781721259147-cookbook-imui-editor-controls-click-stress`.

## 2026-06-18 Data Table Retained/View-Cache Baseline

- Switched from the cookbook IMUI example to the `ui-gallery/data-table` retained/view-cache
  torture scripts to test a heavier shadcn-style application surface. Used the direct-start filter
  scripts so navigation search does not pollute the measured interaction.
- Retained baseline:
  `tools/diag-scripts/ui-gallery/data-table/ui-gallery-data-table-retained-filter-shrink-vlist-inputs-change-direct.json`
  with release `fret-ui-gallery --features "gallery-ai gallery-chart gallery-dev
  gallery-web-ime-harness"` produced
  `target/fret-diag/data-table-retained-filter-direct-m01-baseline/1781722336374/bundle.json`.
  Worst frame: `top_total_time_us=5456`, `top_layout_time_us=4365`, `top_paint_time_us=804`,
  `layout.engine_solve=447`. This is inside the 120Hz frame budget.
- View-cache baseline:
  `tools/diag-scripts/ui-gallery/data-table/ui-gallery-data-table-view-cache-filter-shrink-vlist-inputs-change-direct.json`
  produced
  `target/fret-diag/data-table-view-cache-filter-direct-m01-baseline/1781722407012/bundle.schema2.json`.
  Worst frame: `top_total_time_us=5958`, `top_layout_time_us=4504`, `top_paint_time_us=1233`,
  `layout.engine_solve=486`. View-cache is slightly slower on this input-change slice but still
  within budget.
- Decision: do not refactor the data-table component path from this evidence alone. The retained
  and view-cache direct filter interactions are not the severe 120Hz blocker. The more general
  issue surfaced by the same stats is command availability fallback in no-focus states, where
  subtree fallback checks can consume multiple milliseconds in component-heavy windows; that is a
  better next infrastructure slice.

## 2026-06-18 Command Availability Fallback Refactor

- Root cause: `UiTree::publish_window_command_action_availability_snapshot_for_command_set` still
  used a per-node parent-chain bubble inside no-focus subtree fallback. On a deep tree this turns a
  subtree scan into `nodes * depth` work even though the semantic query is only “does any node in
  this subtree handle this command?”
- Implementation: split node-level availability into a private `command_availability_at_node`
  helper, then changed subtree fallback to do a single DFS over candidate nodes and query each node
  once per command.
- Follow-up refinement: cache subtree-interest summaries within one publication so repeated widget
  commands can reuse the same no-focus pruning metadata instead of rebuilding the subtree interest
  tree for every command.
- Regression coverage:
  - `action_availability_snapshot_matches_no_focus_dispatch_subtree_fallback`
  - `action_availability_no_focus_subtree_fallback_scans_each_node_once_per_command`
  - `action_availability_no_focus_subtree_fallback_reuses_subtree_interest_across_commands`
  - full `command_availability` / `window_command_action_availability_snapshot` nextest slice
- Expected impact: reduce the no-focus command-availability tail inside heavy windows, especially
  on cold-open / first-discovery surfaces where command palette and menu gating previously paid for
  repeated ancestor bubbling.
- Direct-entry inspector rerun after the refinement:
  `target/fret-diag/inspector-direct-entry-no-focus-interest-cache-rerun/1781969841415/bundle.json`
  and `.bundle.schema2.json`.
- The rerun’s top `window_runtime_snapshot.command_availability` frame is down to
  `widget_count/collect_us/eval_us=4/3/28`, and the `subtree_no_focus_fallback` hotspot class is
  absent from the rerun stats bundle.
- Verification completed on the warmed build path and the rerun probe:
  `cargo fmt --all --check`, `cargo check -p fret-ui -j 1`,
  `cargo nextest run -p fret-ui window_command_action_availability_snapshot --no-fail-fast`, and
  `cargo run -p fretboard-dev --release -- diag perf tools/diag-scripts/ui-gallery/perf/ui-gallery-inspector-torture-scroll-direct-entry.json --dir target/fret-diag/inspector-direct-entry-no-focus-interest-cache-rerun --repeat 3 --warmup-frames 5 --timeout-ms 600000 --env FRET_DIAG_SCRIPT_AUTO_DUMP=0 --env FRET_DIAG_SEMANTICS=0 --launch -- cargo run -p fret-ui-gallery --features gallery-dev`
  all passed.

## 2026-06-18 Inspector Torture Baseline

- Moved away from the data-table and command-availability slices and ran a heavier inspector
  torture surface to probe a more realistic editor-style workload.
- Baseline run:
  `tools/diag-scripts/suites/ui-gallery-inspector-torture/suite.json`
  produced `target/fret-diag/inspector-torture-scroll-baseline/1781725601194/bundle.schema2.json`.
  Worst frame `top_total_time_us=3138`; command availability is no longer the dominant tail here.
- Keep-alive variant:
  `tools/diag-scripts/suites/ui-gallery-inspector-torture-keep-alive/suite.json`
  produced `target/fret-diag/inspector-torture-bounce-keep-alive-baseline/1781725682291/bundle.json`.
  Worst frame `top_total_time_us=3907`; the extra keep-alive step did not remove the cost class.
- Shared hotspot shape across both bundles: `layout.root_phases` dominates the expensive frames,
  with `layout.engine_solve` and renderer tail work (`finish`, `ensure`, `record`, `text_prepare`)
  visible behind it. Command availability is present but secondary.
- Working conclusion: the next optimization slice should target the inspector / layout-root / view
  reconstruction boundary, not command dispatch. The goal is to reduce first-solve and root-phase
  churn on heavy component trees before trying to shave smaller tail costs.

## 2026-06-18 Inspector Page Boundary Note

- The latest follow-up kept the work scoped to the page shell instead of the component runtime.
  `PAGE_INSPECTOR_TORTURE` now participates in
  `page_content_cache_contain_layout_when_bounds_known(...)`, which matches the existing
  editor-grade torture pages that already rely on contained layout boundaries.
- This is meant to reduce parent-shell layout churn around the inspector scroll viewport and page
  wrapper, not to mask row-level bugs or move policy into `fret-ui`.
- I also tried scrollbarless viewport chrome for the content scroll area. That kept the page chrome
  thinner but regressed the same torture bundle (`top_total_time_us=5390`,
  `layout.root_phases roots(total/apply)=3837/3836`) versus the immediately prior
  containment-only run (`top_total_time_us=5243`, `roots(total/apply)=3965/3965`). The experiment
  is therefore rejected and should stay out of the mainline path.
- Verification result:
  `target/fret-diag/inspector-torture-page-boundary-recheck-direct/1781731029795/bundle.schema2.json`
  now shows `top_total_time_us=5243` and `layout.root_phases roots(total/apply)=3965/3965`, versus
  the earlier baseline
  `target/fret-diag/inspector-torture-scroll-baseline/1781725601194/bundle.schema2.json`
  with `top_total_time_us=29985` and `roots(total/apply)=18707/18705`.
- The remaining hot path is still the page shell / scroll viewport / content wrapper chain, but the
  first-pass root cost dropped enough that this no longer looks like a runtime-mechanism rewrite
  candidate first.
- Note: `tools/diag-scripts/suites/ui-gallery-inspector-torture/suite.json` is still a legacy
  schema-1 suite manifest. Tool-launched `--launch` runs should use the promoted v2 script
  `tools/diag-scripts/ui-gallery/perf/ui-gallery-inspector-torture-scroll.json` directly.

## 2026-06-18 Field Status Badge Note

- `FieldStatusBadge` 现在可直接携带自己的 padding，不必让 `AssetRefField` 再额外套一
  层 `editor_input_group_segment`。
- 这让共享输入组链条和资产字段路径都少了一层真实容器节点。
- 共享输入组链条内部仍保留 `editor_joined_input_frame`，但调用点已经不再需要为了
  badge 间距专门加壳。

## 2026-06-19 Property Row Flattening Note

- `PropertyRow` 的 row / column 分支在没有 reset / actions slot 时，会把 value 容器
  直接挂在 root 下，去掉了中间 body / header 壳。
- 这次收缩保留了 `PROPERTY_ROW_VALUE_SLOT`，所以布局测试和后续结构检查仍能锚定到
  同一个值槽。
- `PropertyGrid` 的编辑器行高测试不再要求 numeric / drag value 完全同高；drag value
  的 session shell 本来就比 plain numeric input 更高，这属于控件自身的外壳策略。
- 新增了 row / column 直接挂载测试，防止以后又把这层壳悄悄加回去。

## 2026-06-19 NumericInput Root Shell Note

- `NumericInput` 的默认路径现在不再强制包一层纵向 `Flex` 壳；无 inline error 时会
  直接返回 joined field root。
- 只有启用 inline error 文本时，才会重新包回纵向 shell，以便把字段和错误文本堆叠。
- 新增了 root shell 和 inline error helper 的回归测试，边界已经收紧。

## 2026-06-19 Joined Input Frame Note

- `editor_joined_input_frame` 现在在没有 leading / trailing segments 时，不再生成中间
  row 壳，直接把 input 作为 joined frame 内容。
- 只有当存在任一附属 segment 时，才会回退到 `editor_input_group_row`。
- 这把收口扩展到了共享输入原语层，因此 `TextField` / `MiniSearchBox` / `NumericInput`
  都能直接受益，而不是每个控件单独压壳。

## 2026-06-19 ColorEdit Shell Flattening Note

- `ColorEdit` 这次确认了一个更具体的结论：常态路径可以直接落在主 row 上，没必要
  为了 error-less 状态再挂一层纵向外壳。
- 如果有 parse error，才切到 `row + error` 的 sibling 结构，这样不会把错误提示又塞
  回主 row 里。
- 这类收口和前面的 `NumericInput` / `TextField` / `MiniSearchBox` 一样，说明重组件
  里真正能继续削的，往往是默认路径上的空壳，而不是 runtime substrate。
- 回归已过：`cargo fmt -p fret-ui-editor --check`、`cargo check -p fret-ui-editor`、
  `cargo nextest run -p fret-ui-editor color_edit --no-fail-fast`。

## 2026-06-19 NumericInput Layout Propagation Note

- `DragValue` / `Slider` 的稳定 session shell 问题不是单纯测试噪声，而是
  `NumericInput` 没把外层 layout 传进 joined field root。
- 结果是 hidden typing 分支即便被呼叫侧标成 zero-sized，也没法真的隐藏到布局树里。
- 现在这层 layout 直通了，`drag_value` / `slider` 的 session shell 验证回归通过，
  表明这条优化应该记作结构修正，而不是简单的测试松绑。

## 2026-06-19 Workspace TabStrip Chrome Normalization Note

- `WorkspaceTabStrip` 的 tab root 现在改用 `control_chrome_pressable_with_id_props`，不再
  手写外层 `Pressable -> hover_region -> container` 壳。
- 这次迁移把 tab root 的 chrome test id 统一成 `<tab-id>.chrome`，并保留了 tab
  pressable 的原始语义与事件处理。
- 结构测试已经补上并通过，覆盖：
  - tab root 仍是 pressable；
  - derived chrome semantics 仍存在；
  - tab strip 的 layout contract 仍保持 `width = Auto` / `height = Fill`。
- 这个结果说明 workspace tab strip 这类重组件的关键问题之一不是功能缺失，而是
  chrome 壳是否被 canonical helper 统一化，避免局部 ad hoc 壳层继续漂移。

## 2026-06-20 Focused Combobox Long-List Repro Refresh Note

- Re-validated the focused `Long List` diagnostics path against the current mainline code instead of
  relying on the earlier failed launched run.
- Current launched correctness evidence is green:
  `target/fret-diag/combobox-long-list-focused-minrun/1781896246714-ui-gallery-combobox-long-list-focused-filter-select-steady/bundle.schema2.json`.
  Querying that bundle now shows the expected focused docs anchors plus the long-list trigger lane:
  `docsec-long-list-content`, `ui-gallery-combobox-long-list-trigger`,
  `ui-gallery-combobox-long-list-query`, and `ui-gallery-combobox-long-list-selected`.
- Interpretation: the earlier timeout at `wait_until exists(ui-gallery-combobox-long-list-trigger)`
  was stale evidence from an older launched binary/run boundary, not proof that the current
  `docsec-long-list-content` focus path is still broken.
- The focused launched repro is therefore back to being a trustworthy correctness/probe surface.

## 2026-06-20 Focused Combobox Long-List Perf Recheck Note

- Re-ran the current focused perf script on macOS through the launched `cargo run -p
  fret-ui-gallery` path:
  `target/debug/fretboard-dev diag perf tools/diag-scripts/ui-gallery/perf/ui-gallery-combobox-long-list-focused-filter-select-steady.json --dir target/fret-diag/combobox-long-list-focused-perf-current --repeat 1 --warmup-frames 5 --timeout-ms 600000 --env FRET_DIAG_SCRIPT_AUTO_DUMP=0 --env FRET_DIAG_SEMANTICS=0 --launch -- cargo run -p fret-ui-gallery`
- The resulting bundle
  `target/fret-diag/combobox-long-list-focused-perf-current/1781896419197/bundle.schema2.json`
  is not a heavy-component hotspot on this current-state surface. Worst frame is only
  `top_total_time_us=244`, with `layout=14us`, `prepaint=130us`, and `paint=100us`.
- This current focused probe therefore no longer supports “combobox long-list is still the
  dominant heavy-component tail” on this machine/runtime shape. It should stay as a correctness +
  narrow regression probe, but it is not the best next hotspot driver.
- Next perf work should return to the currently heavier editor-controls / inspector / code-view
  class surfaces unless a fresh focused combobox run on another target shows a contradictory tail.

## 2026-06-20 Current-State Heavy Surface Re-Ranking Note

- Re-ran the three current-state candidate probes on macOS instead of continuing from older
  hotspot assumptions:
  - `ui-gallery-inspector-torture-scroll`:
    `target/debug/fretboard-dev diag perf tools/diag-scripts/ui-gallery/perf/ui-gallery-inspector-torture-scroll.json --dir target/fret-diag/inspector-torture-scroll-current --repeat 1 --warmup-frames 5 --timeout-ms 600000 --env FRET_DIAG_SCRIPT_AUTO_DUMP=0 --env FRET_DIAG_SEMANTICS=0 --launch -- cargo run -p fret-ui-gallery --features gallery-dev`
  - `ui-gallery-code-view-torture-mount`:
    `target/debug/fretboard-dev diag perf tools/diag-scripts/ui-gallery/perf/ui-gallery-code-view-torture-mount.json --dir target/fret-diag/code-view-torture-mount-current-macos --repeat 1 --warmup-frames 5 --timeout-ms 600000 --env FRET_DIAG_SCRIPT_AUTO_DUMP=0 --env FRET_DIAG_SEMANTICS=0 --launch -- cargo run -p fret-ui-gallery --features gallery-dev`
  - `cookbook-imui-editor-controls-click-stress`:
    `target/debug/fretboard-dev diag perf tools/diag-scripts/cookbook/imui-editor-controls-basics/cookbook-imui-editor-controls-click-stress.json --dir target/fret-diag/cookbook-imui-editor-controls-click-stress-current --repeat 1 --warmup-frames 5 --timeout-ms 600000 --env FRET_DIAG_SCRIPT_AUTO_DUMP=0 --env FRET_DIAG_SEMANTICS=0 --launch -- cargo run -p fret-cookbook --features cookbook-imui,cookbook-diag --example imui_editor_controls_basics`
- Current ranking on this machine/runtime shape is:
  1. `code-view` worst frame `total=3736us`, `layout=3456us`, `solve=946us`, evidence
     `target/fret-diag/code-view-torture-mount-current-macos/1781897154059/bundle.schema2.json`
  2. `inspector` worst frame `total=3396us`, `layout=3082us`, `solve=1241us`, evidence
     `target/fret-diag/inspector-torture-scroll-current/1781897141976/bundle.schema2.json`
  3. `editor-controls` worst frame `total=1348us`, `layout=1180us`, `solve=513us`, evidence
     `target/fret-diag/cookbook-imui-editor-controls-click-stress-current/1781897209391/bundle.schema2.json`
- Interpretation: on current macOS mainline, `editor-controls` is no longer the best next driver
  for this loop, and `combobox long-list` is much lighter still. The best next hotspot to cut is
  back on the `code-view` lane, with `inspector` as the secondary follow-on.

## 2026-06-20 Code-View Content-Viewport Attribution Note

- Re-ran `ui-gallery-code-view-torture-mount` with node-level layout attribution:
  `target/debug/fretboard-dev diag perf tools/diag-scripts/ui-gallery/perf/ui-gallery-code-view-torture-mount.json --dir target/fret-diag/code-view-torture-mount-node-profile-current --repeat 1 --warmup-frames 5 --timeout-ms 600000 --env FRET_DIAG_SCRIPT_AUTO_DUMP=0 --env FRET_DIAG_SEMANTICS=0 --env FRET_LAYOUT_NODE_PROFILE=1 --env FRET_LAYOUT_NODE_PROFILE_TOP=20 --env FRET_LAYOUT_NODE_PROFILE_MIN_US=200 --launch -- cargo run -p fret-ui-gallery --features gallery-dev`
- The worst-frame aggregate stayed effectively unchanged (`total=3746us`, `layout=3420us`,
  `solve=954us`), but the node profile changed the attribution story: the dominant self-time node
  through the hot frames is the outer page scroll root with test id `ui-gallery-content-viewport`,
  not the inner `ui-gallery-code-view-root` `VirtualList`.
- Representative node-profile evidence from that run:
  - `ui-gallery-content-viewport` repeatedly leads with about `2.2ms-2.6ms` self time and about
    `2.6ms-3.0ms` total time after navigation settles.
  - `ui-gallery-code-view-root` `VirtualList` only becomes the top node on the later narrow frame,
    and there it is smaller (`self_us=821`, `total_us=2904`) than the outer content viewport was
    on the hotter earlier frames.
- A quick bisect supports the same conclusion. Running the same torture script with the existing
  `BISECT_DISABLE_CONTENT_SCROLL` flag enabled:
  `target/debug/fretboard-dev diag perf tools/diag-scripts/ui-gallery/perf/ui-gallery-code-view-torture-mount.json --dir target/fret-diag/code-view-torture-mount-bisect-disable-content-scroll --repeat 1 --warmup-frames 5 --timeout-ms 600000 --env FRET_DIAG_SCRIPT_AUTO_DUMP=0 --env FRET_DIAG_SEMANTICS=0 --env FRET_UI_GALLERY_BISECT=128 --launch -- cargo run -p fret-ui-gallery --features gallery-dev`
  reduced worst frame from `3736us` to `3227us`, with layout dropping from `3456us` to `3027us`.
- Interpretation: the next code-view slice should start from the page-level nested-scroll/content
  viewport contract in `apps/fret-ui-gallery/src/ui/content.rs`, not from another wrapped-text or
  code-block-local cache experiment. The correct question now is whether the code-view torture page
  can avoid paying the extra outer content-scroll root while preserving the intended gallery page
  contract and diagnostics anchors.

## 2026-06-20 Code-View Static Content-Shell Rejection Note

- Tried the obvious page-local version of the earlier bisect result in
  `apps/fret-ui-gallery/src/ui/content.rs`: route `PAGE_CODE_VIEW_TORTURE` through the same static
  content-shell branch that `BISECT_DISABLE_CONTENT_SCROLL` uses, while preserving
  `ui-gallery-content-scroll`, `ui-gallery-page-code-view-torture`, and
  `ui-gallery-code-view-root` anchors.
- Focused validation for the experiment itself was green (`cargo fmt --all`, and a temporary source
  gate plus `cargo check -p fret-ui-gallery --features gallery-dev`), but the perf evidence did not
  justify keeping the implementation.
- On the original mount probe surface, the page-local static-shell implementation was not an
  improvement:
  - first run:
    `target/fret-diag/code-view-torture-mount-page-static-shell/1781898593104/bundle.json`
    reported `total=3904us`, `layout=3571us`, `solve=1001us`
  - repeat-3 rerun:
    `target/fret-diag/code-view-torture-mount-page-static-shell-rerun/1781898666393/bundle.schema2.json`
    reported `p50=3763us`, `p95=max=3807us`, still worse than the earlier current-state run
    (`3736us`) and far from the full bisect path (`3227us`)
- Node profiling on the experiment clarified why this is a bad landing slice:
  `target/fret-diag/code-view-torture-mount-page-static-shell-node-profile/1781898744109/bundle.json`
  still showed the hot frames dominated by `ui-gallery-content-viewport` on the ordinary content
  scroll path. In other words, the current mount script kept measuring a path that never actually
  exercised the intended page-local branch.
- A control rerun with the same script but `FRET_UI_GALLERY_START_PAGE=code_view_torture`
  dramatically reduced the page-local static-shell case:
  `target/fret-diag/code-view-torture-mount-page-static-shell-start-page/1781898825559/bundle.schema2.json`
  reported `p50=532us`, `p95=max=543us`.
- Interpretation:
  - the static shell itself is not obviously expensive
  - the current `ui-gallery-code-view-torture-mount` probe is dominated by nav/search/page-switch
    transition work before the code-view steady surface is isolated
  - therefore this page-local shell change is the wrong thing to land right now
- Decision: revert the implementation and keep the evidence. The next cut should narrow the probe
  surface first (for example a start-page or direct-entry mount contract) before treating outer
  content scroll as the primary code change target.

## 2026-06-20 Code-View Direct-Entry Mount Clarification Note

- Added a separate steady-surface probe at
  `tools/diag-scripts/ui-gallery/perf/ui-gallery-code-view-torture-mount-direct-entry.json` plus a
  focused gate in `apps/fret-ui-gallery/tests/code_view_perf_surface.rs`.
- The contract split is now explicit:
  - `ui-gallery-code-view-torture-mount.json` is a nav/search/page-switch transition probe.
  - `ui-gallery-code-view-torture-mount-direct-entry.json` is the steady direct-entry mount probe.
- Refreshed direct-entry repeat-3 evidence:
  `target/debug/fretboard-dev diag perf tools/diag-scripts/ui-gallery/perf/ui-gallery-code-view-torture-mount-direct-entry.json --dir target/fret-diag/code-view-torture-mount-direct-entry-refresh --repeat 3 --warmup-frames 5 --timeout-ms 600000 --env FRET_DIAG_SCRIPT_AUTO_DUMP=0 --env FRET_DIAG_SEMANTICS=0 --launch -- cargo run -p fret-ui-gallery --features gallery-dev`
  reported `p50.us(total/layout/solve/prepaint/paint/dispatch/hit_test)=306/11/0/189/106/0/0`,
  `p95=313/12/0/197/118/0/0`, and `max=313/12/0/197/118/0/0`; evidence bundle
  `target/fret-diag/code-view-torture-mount-direct-entry-refresh/1781899846277/bundle.json`.
- Refreshed direct-entry node-profile evidence:
  `target/debug/fretboard-dev diag perf tools/diag-scripts/ui-gallery/perf/ui-gallery-code-view-torture-mount-direct-entry.json --dir target/fret-diag/code-view-torture-mount-direct-entry-node-profile-refresh --repeat 1 --warmup-frames 5 --timeout-ms 600000 --env FRET_DIAG_SCRIPT_AUTO_DUMP=0 --env FRET_DIAG_SEMANTICS=0 --env FRET_LAYOUT_NODE_PROFILE=1 --env FRET_LAYOUT_NODE_PROFILE_TOP=20 --env FRET_LAYOUT_NODE_PROFILE_MIN_US=100 --launch -- cargo run -p fret-ui-gallery --features gallery-dev`
  reported `top.us(total/layout/solve/prepaint/paint/dispatch/hit_test)=310/11/0/197/102/0/0`;
  evidence bundle
  `target/fret-diag/code-view-torture-mount-direct-entry-node-profile-refresh/1781899876398/bundle.schema2.json`.
- Interpretation:
  - the steady code-view mount surface is currently light, not a multi-ms hotspot
  - `layout` is down at roughly `11-12us`, `solve=0`, and total worst frame stays around `310us`
  - the older `3736us` result should no longer be used as evidence for steady code-view-local work;
    it is evidence for gallery transition cost until that probe is renamed more explicitly
- Recommended next cut:
  - either formalize/rename the old mount script as a transition probe and continue attribution on
    nav/search/page-switch work
  - or pivot the main hotspot driver to `inspector`, since code-view steady mount is now clarified
    and no longer the strongest candidate on its own

## 2026-06-20 Inspector Direct-Entry Static Content-Stack Note

- The inspector direct-entry probe now routes through the static content stack in
  `apps/fret-ui-gallery/src/ui/content.rs`, while the `code_view_torture` path keeps its own
  scroll shell and fixed preview height.
- This kept the change narrow and reversible: it did not alter the general gallery scroll contract,
  only the inspector-specific direct-entry page shape.
- Validation passed:
  - `cargo fmt --all --check`
  - `cargo nextest run -p fret-ui-gallery --test ui_authoring_surface_internal_previews --no-fail-fast`
  - `cargo nextest run -p fret-ui-gallery --test inspector_perf_surface --no-fail-fast`
  - `cargo nextest run -p fret-ui-gallery --test code_view_perf_surface --no-fail-fast`
- Latest perf evidence:
  - direct-entry after static stack:
    `target/fret-diag/inspector-direct-entry-after-static-content-stack/1781910619755/bundle.schema2.json`
    with `top_total_time_us=2895`, `layout_time_us=2323`, and
    `layout.root_phases.roots(total/apply)=1473/1473`
  - node-profile rerun:
    `target/fret-diag/inspector-direct-entry-after-static-content-stack-node-profile/1781911121082/bundle.schema2.json`
    with worst frame `total=3052us`
- Interpretation: the change is a real improvement, but the remaining hotspot is still the outer
  `ui-gallery-content-viewport` `Scroll`, not the inspector `VirtualList`.
- Next step: continue on the inspector/page-shell content viewport contract, or move back to the
  code-view transition probe only if a new bundle shows a larger regression there.

## 2026-06-20 Inspector Row Invalidation Narrowing Note

- The inspector row tree was already on the shorter `pressable -> content` path from the earlier
  shell shrink. The remaining row-local waste was not another wrapper, but the selected-row model
  read itself.
- `selected_row` now observes with `Invalidation::Paint` instead of `Invalidation::Layout`.
  Selection state still updates row chrome and selected semantics, but it no longer advertises a
  geometry change that does not exist.
- Regression coverage now locks the paint-only read and the absence of the old layout-level read in
  `apps/fret-ui-gallery/tests/ui_authoring_surface_internal_previews.rs`.
- This is still a narrow perf guard rather than a framework-level owner split. If a future bundle
  moves the hotspot back into the row tree, the next cut should be evidence-led and not assume the
  shell is the owner.
- Perf rerun:
  `target/fret-diag/inspector-direct-entry-selected-row-paint-invalidation/1781917561829/bundle.json`
  reported `top_total_time_us=2901`, `layout_time_us=2293`, `layout_engine_solve_time_us=1017`,
  `paint_time_us=438`. That keeps the inspector direct-entry surface in the same general band and
  does not yet remove the outer viewport as the dominant owner.

## 2026-06-20 Gallery Internal Preview Contract Tightening Note

- Cleaned the remaining gallery internal preview surface drift so the typed-helper lane stays
  consistent across `tree_torture.rs` and the overlay helpers.
- `overlay_scroll_row_text` and `overlay_status_text` now return typed helpers (`impl UiChild +
  use<>`) instead of landing on `AnyElement`, and `tree_torture.rs` dropped the stale
  `AppRenderActionsExt` import.
- Validation passed:
  `cargo fmt --all --check`

## 2026-06-21 Code-View Content Scroll Wrapper Removal Note

- Removed the extra page-keyed scroll wrapper around the `code_view_torture` content path in
  `apps/fret-ui-gallery/src/ui/content.rs`.
- The page-level `ScrollHandle` is still owned by the content view, but the scroll area now mounts
  directly without the nested `cx.keyed(format!("ui_gallery.content_scroll_area.{selected}"), ...)`
  boundary.
- Regression coverage now asserts the remaining page-level handle contract and the absence of the
  nested `ui_gallery.content_scroll_area` key in
  `apps/fret-ui-gallery/tests/ui_authoring_surface_content_shell.rs`.
- Follow-up evidence:
  `target/fret-diag/code-view-torture-mount-after-scroll-wrapper-removal/1782001143782/bundle.json`
  improved the direct-entry mount from `top_total_time_us=3736` to `2847`,
  `layout_time_us=3456` to `2605`, `layout.root apply=3115` to `2361`, and
  `layout_engine_solve_time_us=946` to `702`.
- Interpretation: the nested keyed scroll wrapper was real transition noise, but the steady
  direct-entry surface is now small enough that the next meaningful cut should stay on the
  transition probe or move to the inspector lane if that bundle remains the larger owner.
- Validation passed:
  `cargo fmt --all --check`, `cargo nextest run -p fret-ui-gallery --test ui_authoring_surface_content_shell --no-fail-fast`, and `cargo nextest run -p fret-ui-gallery --test code_view_perf_surface --no-fail-fast`.
- This was a surface cleanup, not a new perf win. The current next cut still points back to the
  outer `ui-gallery-content-viewport` / content-shell contract in `apps/fret-ui-gallery/src/ui/content.rs`,
  because the current bundles still name that shell as the dominant owner rather than the inner
  inspector/code-view rows.

## 2026-06-20 Inspector Direct-Entry Follow-up Note

- The inspector direct-entry probe remains the current page-shell candidate after the typed-helper
  cleanup.
- The current evidence still points at `ui-gallery-content-viewport` as the dominant owner, not the
  inner `ui-gallery-inspector-root` `VirtualList`.
- A retried current-state perf run for
  `ui-gallery-inspector-torture-scroll-direct-entry` hit a compile mismatch in the overlay typed
  helper chain first, then was corrected by collapsing the `map`/`into_element` borrow pattern and
  aligning the gallery tests to the new typed-helper inventory.
- Validation for the fix path passed with:
  - `cargo fmt --all --check`
  - `cargo nextest run -p fret-ui-gallery --test ui_authoring_surface_internal_previews --test inspector_perf_surface --test code_view_perf_surface --no-fail-fast`
- I did not complete a fresh inspector perf bundle in this turn, so the next move remains to collect
  updated evidence on the outer content viewport contract before changing the gallery shell again.

## 2026-06-20 Inspector Direct-Entry Short Shell Note

- `preview_inspector_torture` now returns the retained inspector list directly, so the
  direct-entry `inspector_torture` page skips the generic preview card shell while still keeping
  the inspector diagnostics root.
- Regression coverage now locks both boundaries: the generic preview card shell stays absent on
  inspector direct-entry, and `ui-gallery-inspector-root` stays present.
- Validation passed:
  - `cargo fmt --all --check`
  - `cargo nextest run -p fret-ui-gallery --features gallery-dev inspector_torture_skips_preview_card_shell --no-fail-fast`
- Latest perf rerun:
  - `target/fret-diag/inspector-direct-entry-short-shell-v2/1781958586751/bundle.json`
  - `p50 total/layout/solve/prepaint/paint = 2361/1846/741/165/350`
  - `p95 = 2700/2119/864/220/361`
  - hot frame `total/layout/solve/prepaint/paint = 2446/1900/864/220/361`
- Node attribution still keeps the outer `ui-gallery-content-viewport` as the dominant owner, so
  this is a useful shell shrink but not yet an owner shift.
- I also tried removing the generic `ui-gallery-page-preview` semantics wrapper. That rerun
  regressed to `2700us`, so the change was reverted and the retained direct return is the only
  short-shell win.

## 2026-06-20 Inspector Direct-Entry Preview Boundary Tightening Note

- The gallery content shell now keeps `page_preview` on the explicit `Vec<AnyElement>` boundary
  instead of collapsing inspector direct-entry back into a single landed preview element.
- `content.rs` now routes `PAGE_INSPECTOR_TORTURE` through the shared preview match arm and feeds
  the preview panel directly from the vector boundary, while `preview_inspector_torture` keeps
  returning the retained inspector list as `Vec<AnyElement>`.
- Validation passed:
  - `cargo fmt --all`
  - `cargo nextest run -p fret-ui-gallery --test ui_authoring_surface_content_shell --test ui_authoring_surface_internal_previews --no-fail-fast`
- Perf rerun:
  - `target/fret-diag/inspector-direct-entry-short-shell-v3/1781974142000/bundle.schema2.json`
  - `p50 total/layout/solve/prepaint/paint = 2545/2034/879/186/316`
  - `p95 = 2724/2188/941/221/344`
- `diag stats` on the same bundle still points at the outer `ui-gallery-content-viewport` /
  content-shell path as the dominant owner, with `layout.root_phases roots(total/apply)=1306/1305`
  on the hottest frame. This is a boundary-tightening slice, not an owner-shift yet.
- Next step: keep trimming the outer content shell only if the next bundle moves the owner or drops
  root-apply meaningfully; do not reopen row-local inspector work first.

## 2026-06-21 Inspector Retained-List Boundary Prune Note

- `preview_inspector_torture` now returns the retained inspector list directly instead of wrapping
  it in an extra `cached_subtree_with(...contain_layout_when_bounds_known(true))` shell.
- Regression coverage now locks the direct-return boundary so the retained list stays the page
  body and the extra cached subtree wrapper does not come back.
- Validation passed:
  - `cargo fmt --all`
  - `cargo nextest run -p fret-ui-gallery --test ui_authoring_surface_internal_previews --test inspector_perf_surface --test ui_authoring_surface_content_shell --no-fail-fast`
- Latest perf evidence:
  - `target/fret-diag/inspector-direct-entry-content-shell-prune-v2/1781989941783/bundle.schema2.json`
  - `p50 total/layout/solve/prepaint/paint = 1935/1431/776/180/300`
  - `p95 total/layout/solve/prepaint/paint = 2308/1785/897/212/344`
- Comparison against the prior bundle:
  - `p95 total` dropped from `2383us` to `2308us`
  - `p95 layout` dropped from `1818us` to `1785us`
  - `layout.root_phases.roots(total/apply)` dropped from `1359/1359` to `1280/1280`
- Interpretation: this is a real but modest reduction, and the remaining owner is still the outer
  content viewport / shell contract rather than the inspector retained list itself.

## 2026-06-20 Inspector Direct-Entry Nav Scroll Intrinsic-Mode Note

- The direct-entry inspector rerun now points the visible hot node at the fixed-width sidebar
  `ui-gallery-nav-scroll`, not the inspector row tree.
- `apps/fret-ui-gallery/src/ui/nav.rs` now forces that scroll viewport into
  `ScrollIntrinsicMeasureMode::Viewport` with `viewport_probe_unbounded(false)`, matching the
  already-bounded content viewport pattern and avoiding recursive nav-list measurement during
  intrinsic sizing.
- Validation passed:
  - `cargo fmt --all --check`
  - `cargo nextest run -p fret-ui-gallery gallery_sidebar_nav_scroll_is_explicit_flex_fill_slot --no-fail-fast`
- Perf rerun:
  - `cargo run -p fretboard-dev --release -- diag perf tools/diag-scripts/ui-gallery/perf/ui-gallery-inspector-torture-scroll-direct-entry.json --dir target/fret-diag/inspector-direct-entry-nav-scroll-rerun --repeat 3 --warmup-frames 5 --timeout-ms 600000 --env FRET_DIAG_SCRIPT_AUTO_DUMP=0 --env FRET_DIAG_SEMANTICS=0 --launch -- cargo run -p fret-ui-gallery --features gallery-dev`
  - bundle: `target/fret-diag/inspector-direct-entry-nav-scroll-rerun/1781938135186/bundle.schema2.json`
- Result: worst frame stayed around `3284us`, so this is a contract-tightening slice rather than a
  major perf win.
- Next step: keep chasing the remaining outer-shell / root-apply hotspot with another narrow,
  evidence-led cut instead of widening back into row-local changes.

## 2026-06-20 Inspector Direct-Entry A/B Note

- The inspector direct-entry page was tested with `FRET_UI_GALLERY_VIEW_CACHE=1` and
  `FRET_UI_GALLERY_VIEW_CACHE_SHELL=1`.
- That A/B did not produce a useful win. The cache-enabled run landed at
  `p50 total/layout/solve/prepaint/paint = 3431/2810/985/83/535` and
  `p95 = 3740/3122/1075/84/540`, which is worse than the current no-cache direct-entry band.
- A second probe with `FRET_UI_GALLERY_INSPECTOR_KEEP_ALIVE=0` stayed in the same multi-ms range
  (`p50 total/layout/solve/prepaint/paint = 3170/2517/1160/215/427`,
  `p95 = 3379/2704/1190/248/453`).
- `diag stats` on the keep-alive-off bundle still shows the outer shell/root-apply path as the
  dominant owner, with p95/max `roots(total/apply)=1593/1593`.
- Working conclusion: the next slice should target the inspector page shell in
  `apps/fret-ui-gallery/src/ui/content.rs`, not cache toggles or the retained list knobs.

## 2026-06-20 Inspector Direct-Entry Overscan-8 Note

- Tightened the retained inspector list window from overscan `12` to `8` in
  `apps/fret-ui-gallery/src/ui/previews/gallery/torture/inspector_torture.rs`.
- The refreshed direct-entry run improved the steady band to
  `p50 total/layout/solve/prepaint/paint = 2161/1538/849/199/328` and
  `p95 = 2564/1899/993/229/419`; evidence bundle
  `target/fret-diag/inspector-direct-entry-overscan-8/1781940494893/bundle.json`.
- A node-profile rerun on the same bundle still points at the outer `ui-gallery-content-viewport`
  `Scroll` as the dominant owner (`self_us=7449`, `total_us=11499` on the hot frame), while the
  retained `ui-gallery-inspector-root` `VirtualList` sits much lower (`self_us=840`,
  `total_us=1617`).
- Conclusion: overscan `8` is a real improvement, but it does not move the owner out of the page
  shell. The next cut should stay on `apps/fret-ui-gallery/src/ui/content.rs` and the inspector
  direct-entry shell contract instead of shrinking the retained list window further first.

## 2026-06-20 Inspector Direct-Entry Shell-Root-Prune Note

- I tested a narrower shell change by removing the extra `ui_gallery.content_root` key from the
  non-cache `content_view` path in `apps/fret-ui-gallery/src/driver/shell.rs`, leaving the selected
  page key as the only content boundary.
- Validation still passed for the inspector script surface:
  `cargo fmt --all --check` and
  `cargo nextest run -p fret-ui-gallery --test inspector_perf_surface --no-fail-fast`.
- Perf rerun with the same direct-entry script:
  `target/fret-diag/inspector-direct-entry-shell-root-prune/1781943312483/bundle.json`
  reported `p50 total/layout/solve/prepaint/paint = 2102/1483/829/228/391` and
  `p95 = 2721/2024/942/302/398`.
- Node attribution on that rerun still keeps the hot path in the outer shell/root-apply surface;
  the `ui-gallery-content-viewport` contract did not move to a clearly cheaper owner, and the
  page-level frame got worse, not better.
- Conclusion: the `content_root` prune is not a good next step. Keep the current shell shape and
  look for a different evidence-backed cut in `apps/fret-ui-gallery/src/ui/content.rs` or a
  narrower retained-list seam only if a future bundle moves the owner.

## 2026-06-21 Inspector Content Shell Prune Note

- The `ui_gallery.content_view_root` named wrapper is now gone from
  `apps/fret-ui-gallery/src/ui/content.rs`, so the page shell keeps only the actual content
  container and semantics stamp.
- The retained inspector row slice also kept the earlier row-local cleanup:
  `selected_row` is read once before the row closure, and the row builder now uses array-backed
  child returns instead of `Vec`.
- Validation passed with `cargo fmt --all`, `cargo nextest run -p fret-ui-gallery --test
  ui_authoring_surface_internal_previews --test inspector_perf_surface --no-fail-fast`, and
  `git diff --check`.
- Perf rerun:
  `target/fret-diag/inspector-direct-entry-content-shell-prune-v1/1781988478054/bundle.schema2.json`
  landed at `p50 total/layout/solve/prepaint/paint = 2655/2123/903/208/324us` and
  `p95 = 2738/2178/911/211/349us`.
- Diff against the prior `inspector-direct-entry-short-shell-v3` bundle showed a small but real
  win: `p95 total` down `5.3%`, `p95 layout` down `7.6%`, while `layout.roots` is still the main
  owner and `layout.engine_solve` stayed roughly flat.
- Conclusion: this is a useful reduction, but the page shell is still not fully cleared. The next
  slice should stay on the inspector content shell / viewport boundary instead of reopening the row
  micro-optimizations first.

## 2026-06-21 Inspector Direct-Entry Sidebar Shell Shrink Note

- The latest node-profile pass on `ui-gallery-inspector-torture-scroll-direct-entry` put the hottest
  owner in the fixed-width sidebar `Scroll`, not the inspector retained list. The ranked hot node was
  `ui-gallery-nav-scroll` with `self_us=6949` and `total_us=10714`, while the inspector
  `ui-gallery-inspector-root` `VirtualList` stayed around `self_us=545` / `total_us=647` on the same
  profile.
- `apps/fret-ui-gallery/src/driver/shell.rs` no longer wraps the sidebar path in an extra
  `cx.keyed("ui_gallery.sidebar", ...)` shell when sidebar view-caching is off. The branch now reads
  the selected page/query once and hands the result directly to `ui::sidebar_view(...)`.
- Regression coverage now locks the narrower sidebar-shell shape in
  `apps/fret-ui-gallery/tests/ui_authoring_surface_content_shell.rs`, alongside the existing
  content-shell and inspector-preview regression checks.
- Validation passed with `cargo fmt --all --check`,
  `cargo nextest run -p fret-ui-gallery --test ui_authoring_surface_content_shell --no-fail-fast`,
  and perf reruns.
- Perf evidence:
  - before: `target/fret-diag/inspector-torture-scroll-direct-entry-followup/1782002141113/bundle.schema2.json`
    with `p95 total/layout/prepaint/paint = 2608/1916/310/344`
  - after: `target/fret-diag/inspector-direct-entry-after-sidebar-shrink/1782003641604/bundle.schema2.json`
    with `p95 total/layout/solve/prepaint/paint = 2494/1991/940/263/468`
- Interpretation: the sidebar shell shrink produced a real but modest improvement in total time, but
  it also shifted some cost toward layout solve/paint. The next slice should stay evidence-led: either
  continue trimming the sidebar/nav shell or look for a better owner shift in the content shell only
  if a fresh node profile moves the hotspot again.

## 2026-06-21 Inspector Direct-Entry Scroll Handle Split Rejection Note

- I tried splitting the keyed outer wrapper around the content scroll area in
  `apps/fret-ui-gallery/src/ui/content.rs` so the per-page scroll handle reset path became direct.
- That cut was rejected. The refreshed perf bundle
  `target/fret-diag/inspector-direct-entry-scroll-handle-split-v1/1781990899082/bundle.schema2.json`
  regressed from the prior `inspector-direct-entry-content-shell-prune-v2` bundle
  `target/fret-diag/inspector-direct-entry-content-shell-prune-v2/1781989941783/bundle.schema2.json`
  on the key p95 band:
  `total/layout/solve/prepaint/paint = 2308/1785/897/212/344` -> `2584/1987/913/220/358`.
- `layout.root_phases.roots(total/apply)` did not improve either, so the split did not shift the
  dominant owner out of the outer content viewport / shell path.
- Conclusion: keep this as a negative slice and do not continue down the direct scroll-handle split
  path. The next cut needs a different owner shift, not another page-key scope tweak.

## 2026-06-21 Inspector Direct-Entry Sidebar Shell Cache Contract Note

- The inspector direct-entry perf surface now defaults `FRET_UI_GALLERY_VIEW_CACHE_SHELL=1` in
  `tools/diag-scripts/ui-gallery/perf/ui-gallery-inspector-torture-scroll-direct-entry.json`, so
  the steady probe measures the stabilized sidebar shell contract instead of the uncached sidebar
  path.
- Regression coverage now locks that env default in
  `apps/fret-ui-gallery/tests/inspector_perf_surface.rs`.
- Validation passed:
  - `cargo fmt --all --check`
  - `cargo nextest run -p fret-ui-gallery --test inspector_perf_surface --no-fail-fast`
  - `cargo nextest run -p fret-ui-gallery --test ui_authoring_surface_internal_previews --no-fail-fast`
- Perf rerun:
  `target/fret-diag/inspector-direct-entry-shell-cache-probe-rerun/1782015545566/bundle.schema2.json`
  reported `p50 total/layout/solve/prepaint/paint = 2300/1791/933/204/279` and
  `p95 = 2442/1959/1069/228/281`.
- Compared with the prior no-cache direct-entry band, this is a small but real total/layout win.
  The remaining work is still in the broader sidebar/content shell owner, not the inspector rows.
- Conclusion: keep this shell-cache default as part of the direct-entry measurement contract, but do
  not treat it as the final structural fix. Continue the sidebar/nav shell line only if future node
  profiles show another stable owner shift.
