---
title: IMUI editor-controls structure audit
date: 2026-06-16
type: working-note
---

# IMUI Editor-Controls Structure Audit

## Purpose

Record the current structural read on the IMUI/editor-controls lane before any fearless refactor.
This is not an implementation plan. It is a checkpoint for deciding which seam deserves the next
deepening pass.

## What the evidence says

- The current editor-controls pain is mostly composition depth and layout-policy coupling.
- The strongest source of visible height change is `PropertyRowLayoutVariant::Auto` in
  `ecosystem/fret-ui-editor/src/composites/property_row/layout.rs`, because the row can flip
  between horizontal and vertical composition based on last-frame bounds.
- `DragValue` and `NumericInput` both build a dual-branch shell spine through
  `ecosystem/fret-ui-editor/src/controls/session_shell.rs`, which is correct but heavy for dense
  inspector surfaces.
- `TextAssistField` and `ColorEdit` keep popup/overlay policy inside the editor ecosystem, which is
  the right layer, but those surfaces add more wrapper depth around already dense rows.
- The hot path is therefore not a runtime rewrite candidate first. It is a component-structure and
  lane-policy candidate.

## 2026-06-16 Implementation Note

- The default `PropertyGrid` / `PropertyGridVirtualized` row option path now uses
  `PropertyRowLayoutVariant::Row` instead of implicit `Auto`.
- `PropertyRowOptions::with_grid_defaults(...)` preserves explicit caller variants and only fills
  missing shared metrics.
- Coverage now includes:
  - a default-row context test for `PropertyGrid`,
  - a merge test for `PropertyRowOptions`,
  - and the existing row-separation geometry test.
- The visible-height jump remains a valid concern for explicit `Auto` callers, but it is now an
  opt-in policy rather than the grid default.

## 2026-06-16 DragValue Height-Stability Note

- Post-grid diagnostics showed the suite passed but `roughness` typing still increased
  `grid/group/inspector` height by about 10px.
- The root cause was not the property grid anymore. `DragValue` scrub mode reserved only the dense
  editor line height, while the typing branch reused `NumericInput` chrome and measured as a taller
  joined text field.
- `ResolvedEditorFrameChrome::control_outer_height(...)` now gives session-switch controls a shared
  outer-height contract: line height plus vertical padding plus border.
- `DragValue`, `Slider`, and `AxisDragValue` session shells now reserve that outer chrome height.
  `DragValue` and `Slider` typing branches also use `Size::Small` when they delegate to
  `NumericInput`, matching the editor inspector lane instead of the generic input default.
- Evidence after the change:
  `target/fret-diag/cookbook-imui-editor-controls-stable-session-height-v1/sessions/1781574024501-9052/suite.summary.json`
  passed all five editor-controls scripts. Layout sidecars showed stable `grid=201.33`,
  `group=240.00`, and `inspector=286.00` heights across smoke, exposure, click-stress, overlay,
  and roughness typing-active stages.

## 2026-06-16 FretApp Theme Lifecycle and ColorEdit Height Note

- A later cookbook run exposed a deeper startup-order issue: `FretApp::setup(...)` executed before
  the desktop shadcn default install, so app setup could not reliably override the base
  design-system theme. The fix stages desktop defaults into:
  - base defaults before app setup (`fret_ui_shadcn::app::install`),
  - runtime defaults after app setup (i18n, diagnostics, config/keymap defaults, assets, icons).
- Window-metrics theme sync was a second lifecycle issue. On color-scheme or metrics changes, the
  default shadcn middleware re-synced the host theme and wiped the installed editor dense preset.
  With the `imui` feature, the middleware now replays the installed editor preset after the host
  shadcn sync.
- After the theme lifecycle fix, the main editor controls converged to 30px, but `ColorEdit`
  remained 28px because it used the bare editor row height rather than the editor text-field frame
  chrome outer height.
- `ColorEdit` now derives its root/input minimum height from
  `EditorStyle::frame_chrome_small().control_outer_height(...)`, matching `DragValue`, `Slider`, and
  `AxisDragValue`.
- Coverage now includes:
  - `FretApp` setup-order tests for theme override and runtime keybinding defaults,
  - shadcn auto-theme middleware replay coverage for the installed editor preset,
  - and a `ColorEdit` element-tree height regression test.
- Evidence after the change:
  `target/fret-diag/cookbook-imui-editor-controls-color-edit-height-v1/sessions/1781577414581-54972/suite.summary.json`
  passed all five editor-controls scripts. Smoke slices showed `exposure=30px`,
  `roughness=30px`, `tint=30px`, `search=30px`, and `grid=192px`.

## Upstream shape comparison

- `repo-ref/base-ui` keeps its composite list machinery shallow: `CompositeList` is a registry and
  ordering layer, `ComboboxList` is a thin list container, `ComboboxCollection` is a fragment
  mapper, and `ComboboxItem` owns the item-specific behavior.
- `repo-ref/imgui` shows the opposite extreme: immediate-mode code stays flat and lets the demo
  surface own the composition locally.
- Fret's editor controls sit between those two. The current issue is that the inspector lane has
  accumulated too many policy shells around each row, not that the runtime substrate is missing a
  primitive.

## Fearless refactor candidates

1. Stabilize the property row shell and make the row/column split explicit for dense inspector
   surfaces. Keep `Auto` as an opt-in policy, not the default shape everywhere.
2. Simplify the numeric edit spine so `DragValue` / `NumericInput` share a flatter stable shell and
   stop changing the row's effective height when the mode changes.
3. Keep overlay policy in the editor ecosystem, but factor repeated popup surface chrome into a
   shared helper instead of repeating the same wrapper pattern in each control.

## 2026-06-18 Numeric Shell Note

- The earlier `DragValue` height jump was not a `PropertyGrid` default-variant issue, and not an
  overlay bug.
- The shared session shell for mode-switching numeric controls was leaving `height = Auto`, which
  made the visible wrapper track branch-specific content height more closely than it should.
- That shell is now a fixed-height wrapper for the editor numeric controls. The refactor candidate
  remains valid as a deeper simplification target, but the immediate geometry jump is no longer
  produced by the shell contract.

## 2026-06-18 ColorEdit Error-Path Note

- `ColorEdit` already keeps popup, tooltip, copy-menu, and eyedropper behavior in overlay/popup
  paths, which is the right layering shape for dense inspector surfaces.
- The remaining layout-risk path is the inline error text: when the color parse fails, the root
  layout appends a text node below the swatch/input row. That means invalid input can still grow
  the control vertically.
- This is not yet the same class of bug as the numeric session shell jump, but it is a real
  follow-on candidate if the goal is strict height stability even in invalid state.

## 2026-06-18 Numeric Input Default Error Policy Note

- The generic `NumericInput` default error display was still `InlineTextAndIcon`, which meant dense
  caller surfaces could expand vertically in invalid state unless they remembered to override it.
- The default now prefers `TrailingIcon`, which keeps the common invalid-state presentation inside
  the existing control height.
- Dense editor callers that want explicit inline error copy can still opt in directly, but the
  safer default is now the stable one.

## 2026-06-18 TransformEdit Structure Note

- `TransformEdit` 的 `section_col_with_link` 之前是“先生成一层列，再外包一层列”才能
  放下 link toggle，这属于不必要的树深度。
- 现在 link 变体和普通变体都复用同一个 column shell，只有内容组合不同，不再额外
  叠一层容器。
- 这不是当前示例里最重的热点，但它验证了一个更一般的方向：组件 lane 里还有
  许多“为了插一个小附属控件而多包一层”的浅壳，值得持续收敛。

## 2026-06-18 Next Candidate Note

- 继续优先观察 `TextField` / `MiniSearchBox` / `AssetRefField` / `FieldStatusBadge` 这条
  共享输入组原语链。
- 它们目前共享 `editor_joined_input_frame(...)`，结构上比单个控件更像真正的共性层。
- 下一步若再出现高度跳变，优先检查这条共享原语，而不是每个调用点单独补丁。

## 2026-06-18 TextAssistField Structure Note

- `TextAssistField` 的 anchored-overlay 路径现在不再额外套一层外部 `flex` 根节点，直
  接返回 `TextField` 根节点，同时仍保留 overlay 请求和输入-owned 键处理。
- 这把 overlay surface 的结构再收薄了一层，也让 inline 与 overlay 的根形态更加
  分离：inline 继续用外层纵向布局，overlay 只保留字段本体。
- 这类收敛是对的方向，但它还不是终点。下一批应继续看共享输入组原语是否还有
  可以合并的壳层，而不是在每个调用点重复叠层。

## 2026-06-18 Input Group Button-Depth Note

- `editor_icon_button_segment` 原来是 `pressable -> container -> flex -> icon`，对一个简
  单图标按钮来说层数偏厚。
- 现在中间那层 `flex` 已经去掉，按钮段变成 `pressable -> container -> icon`。
- 这不是大规模重构，但它把共享输入组原语再压薄了一层，并且有明确的测试门守
  住，后面如果继续收 `editor_joined_input_frame`，可以把它当作已知基线。

## 2026-06-18 Field Status Badge Note

- `FieldStatusBadge` 现在自己携带 `padding`，不再需要 `AssetRefField` 外面再包一层
  `editor_input_group_segment` 来补间距。
- 这次删掉的是一个真实容器节点，而不是只改了调用写法。
- `editor_input_group_inset` 仍然保留给其他显式需要 inset 的调用点；当前收口只把
  资产状态这条路径收薄。

## Not the next target

- Do not start with `fret-ui` runtime rewrites.
- Do not move overlay/focus policy out of `fret-ui-editor` just because these controls are dense.
- Do not treat `repo-ref/base-ui` as evidence that the editor lane should become equally shallow;
  the problem here is the shape of the current inspector lane, not the existence of wrappers by
  itself.

## Current recommendation

Start with `PropertyRow` / `PropertyGrid` first. That seam has the clearest link to the observed
height jump and the biggest leverage over the rest of the dense editor surface.

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

## 2026-06-19 ColorEdit Row Shell Note

- `ColorEdit` 的默认路径现在直接返回主横向 row，不再额外包一层纵向 shell。
- 行高约束没有丢：`min_height` 现在落在主 row 上，所以无 error 时仍然保留编辑器
  chrome 高度。
- 错误态也被收敛成 sibling 结构：`row + error`，而不是把 error 再塞回 row 里面。
- 回归测试已经补上并通过，覆盖了稳定 chrome 高度和错误态 row 形状两条边界。

## 2026-06-19 NumericInput Layout Propagation Note

- `NumericInput` 之前把 `options.layout` 只用在 inline error 外壳，joined field 根本没有吃
  到这层布局参数。
- 这会让 `DragValue` / `Slider` 传入的 hidden session layout 失效，非 typing 状态的分支
  依然保持可布局形状。
- 现在 joined field root 直接消费 `options.layout`，这样 session shell 的 hidden branch
  才能真正成为零尺寸/绝对定位的隐藏分支。
- 相关回归测试已经改成穿透事件壳检查稳定 shell，并验证 `drag_value` / `slider`
  的 focused nextest slice 通过。

## 2026-06-22 ColorEdit Error Text Direct Sibling Note

- `ColorEdit` used to wrap the invalid-state error text in
  `editor_input_group_segment` just to get 4px of leading inset, producing a
  `row + Container(Text)` sibling shape.
- The inset now lives on the text element's own `layout.margin.left`, so the
  error sibling is direct `Text` and the invalid-only path avoids one
  behaviorless container layer.
- This does not change the normal-state structure or the contract that
  invalid-state copy appears below the main row; it only thins that error-text
  shell and locks the shape with a structure test.

## 2026-06-22 Slider Track Segment Direct Flex Note

- `Slider` used to build its visual track as `Container(Flex(track))` because the
  input-group segment carried frame padding and flex sizing.
- The track flex now owns that padding and outer flex sizing directly, so the
  slider frame row mounts the track as a direct `Flex` child.
- The value readout segment stays wrapped for now because its fixed width and
  text padding are a separate visual contract.
- The structure gate verifies that the track is no longer hidden behind a
  segment container while the broader `slider` nextest filter keeps pointer,
  typing, value-math, and chrome coverage green.
