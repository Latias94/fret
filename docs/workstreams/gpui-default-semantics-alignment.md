---
title: "GPUI/Zed default semantics alignment"
status: "active"
updated: "2026-02-18"
---

This note tracks *default semantics* differences between Fret and upstream GPUI/Zed-style UI authoring.
The goal is to reduce “mysterious” layout/style mismatches when porting shadcn/Radix-inspired recipes.

Scope (this document)

- Defaults implied by common helpers (`h_flex` / `v_flex`, theme token reads, truncation).
- Cross-layer ownership notes (mechanism vs policy vs recipe), so fixes land in the right crate.

Non-goals

- Full component parity (that belongs in `fret-ui-shadcn` + tests/diag scripts).
- Design decisions / visual restyling.

## Alignment table

| Area | Upstream (Zed/GPUI) default | Fret default (prior) | Impact | Fix / owner | Status | Gate | Evidence anchors |
|---|---|---|---|---|---|---|---|
| Primitive length defaults | `Length::default()` is a definite `0px` (not `auto`), while `Style::default().size` is explicitly `auto`. | `Length::default()` is `Auto` and is used throughout `LayoutStyle::default()`. | Mostly a low-level API sharp edge: callers who rely on `Default::default()` for `Length` (outside `LayoutStyle`) may get different behavior vs upstream helpers. | **Mechanism**: likely keep as-is (Fret uses `Length::Auto` heavily), but document this difference to avoid subtle port bugs when translating helper code. | Open | `crates/fret-ui/src/element.rs` | `repo-ref/zed/crates/gpui/src/geometry.rs:3654`<br>`crates/fret-ui/src/element.rs:437` |
| Low-level flex container width | `Style::default()` is `display: block`, and `h_flex()/v_flex()` builds on that; flex containers usually end up with block-like stretch width without extra calls. | `FlexProps::default()` leaves `layout.size.width = Auto`; some measure paths can propagate “near-zero width” constraints unless the caller opts into fill/stretch via parent align or explicit `Fill`. | Same symptom class as the tabs/date truncation: unexpected wrapping/ellipsis due to an underestimated available width. | **Mechanism vs policy decision**: changing `FlexProps::default()` has a high blast radius (many callers use it directly). Prefer authoring via `fret-ui-kit` helpers which stamp the expected width semantics (already done for `ui::h_flex/v_flex`). If we still see repeats outside kit, consider an opt-in helper or a narrowly-scoped mechanism change. | Open | `crates/fret-ui/src/element.rs`<br>`tools/diag-scripts/todo-layout-debug.json` | `repo-ref/zed/crates/gpui/src/style.rs:750`<br>`repo-ref/zed/crates/ui/src/traits/styled_ext.rs:23`<br>`crates/fret-ui/src/element.rs:1784`<br>`ecosystem/fret-ui-kit/src/ui.rs:154`<br>`crates/fret-ui/src/declarative/tests/layout/basics.rs` |
| Helper `h_flex` cross-axis alignment | `h_flex()` stamps `items_center()` (Zed helper convenience). | `FlexProps::default()` uses `align: Stretch`; but `ui::h_flex` stamps horizontal helpers with `Items::Center` by default. | Ports that assume “horizontal stacks center items by default” will look vertically misaligned when using low-level `FlexProps::default()` directly. | **Policy/infra**: encourage using `fret-ui-kit` helpers for Zed-style ergonomics; keep low-level defaults conservative. | Aligned (policy) | None | `repo-ref/zed/crates/ui/src/traits/styled_ext.rs:30`<br>`crates/fret-ui/src/element.rs:1784`<br>`ecosystem/fret-ui-kit/src/ui.rs:72` |
| Flex container width | `h_flex()/v_flex()` relies on the engine’s default “block-like stretch” width semantics. No explicit `width: 100%` is required. | `ui::h_flex/v_flex` left inner flex width as `Auto` in some measure paths, which could propagate “near-zero width” constraints. | Premature ellipsis/wrapping: tabs, dates, and row text truncate even when there is space. | **Policy/infra**: set inner `FlexProps.layout.size.width = Fill` in `ecosystem/fret-ui-kit`. | Aligned | `tools/diag-scripts/todo-layout-debug.json` | `ecosystem/fret-ui-kit/src/ui.rs:154` |
| Position + inset defaults | Style defaults to `position: relative` (inset offsets can “nudge” without an explicit `relative()` call). | Layout defaults to `PositionStyle::Static`; inset offsets are ignored unless author opts into `.relative()` or `.absolute()`. | Ports that assume “inset works by default” can silently no-op; hard to spot in layout-heavy recipes. | **Policy/infra**: in `fret-ui-kit`, applying inset refinements now implies `position: relative` unless the author already chose `absolute()`. | Aligned (policy) | `ecosystem/fret-ui-kit/src/style/tests.rs` | `repo-ref/zed/crates/gpui/src/style.rs:750`<br>`crates/fret-ui/src/element.rs:362`<br>`ecosystem/fret-ui-kit/src/style/layout.rs:90`<br>`ecosystem/fret-ui-kit/src/style/tests.rs` |
| Overflow defaults | `overflow: visible` (both axes). | `Overflow::Visible` by default. | Usually neutral; matters when author expects clipping or when hit-testing should be constrained. | No change (authoring sets clip/hidden explicitly). | Aligned | None | `repo-ref/zed/crates/gpui/src/style.rs:750`<br>`crates/fret-ui/src/element.rs:341` |
| Flex shrink defaults | `flex_shrink: 1` (CSS/Tailwind default). | `FlexItemStyle.shrink = 1.0` by default. | Enables shrink + ellipsis patterns; recipes opt out via `shrink-0`/equivalent. | No change. | Aligned | None | `repo-ref/zed/crates/gpui/src/style.rs:750`<br>`crates/fret-ui/src/element.rs:417` |
| Text whitespace + overflow defaults | `TextStyle::default()` uses `white_space: Normal` and no `text_overflow` (wrapping is allowed; overflow behavior is opt-in). | `ui::text` defaults to `wrap: Word` but `overflow: Clip`; `ui::label` defaults to `wrap: None` + fixed line box height. | Rare “mysterious clipping” when porting DOM patterns that rely on overflow-visible text (or on parent-driven clipping only). | **Policy/infra**: likely keep Fret defaults (renderer-friendly, deterministic) but document where to opt into `TextOverflow::Ellipsis` vs `Clip` vs `overflow_visible` on the parent. Add a tiny invariant test if we discover a concrete mismatch. | Open | `ecosystem/fret-ui-kit/src/lib.rs` | `repo-ref/zed/crates/gpui/src/style.rs:410`<br>`ecosystem/fret-ui-kit/src/ui.rs:878`<br>`ecosystem/fret-ui-kit/src/lib.rs` |
| Scroll intrinsic probing | (Taffy/CSS-like) scroll sizing behavior typically probes content but is sensitive to available-space modes; “wrap vs scroll long tokens” is often decided by the container constraints. | `ScrollProps::default()` uses `probe_unbounded: true` (measure full extent along the scroll axis) and `Overflow::Clip`. | Can change whether text wraps vs becomes a long single line requiring scrolling; affects porting list/code/markdown surfaces. | **Mechanism**: keep the default but add an explicit authoring knob (already exists) + document recommended settings per surface (`probe_unbounded: false` for wrap-friendly scroll containers). Consider a small diag gate if this causes regressions. | Open | `crates/fret-ui/src/declarative/tests/layout/scroll.rs` | `crates/fret-ui/src/element.rs:1989`<br>`crates/fret-ui/src/declarative/tests/layout/scroll.rs` |
| Theme snapshot token reads | Theme reads go through the full theme object (no “snapshot silently drops token tables” path). | `ThemeSnapshot` only held typed baseline fields; reading shadcn semantic tokens (e.g. `muted`) could degrade to baseline fallbacks. | “White-on-white” style bugs: track/background tokens collapse so active states are not visible. | **Mechanism**: make `ThemeSnapshot` carry the configured token maps (Arc) so `ThemeSnapshot.color_token(k)` matches `Theme.color_token(k)`. | Aligned | `crates/fret-ui/src/theme/mod.rs` | `crates/fret-ui/src/theme/mod.rs:637`<br>`crates/fret-ui/src/theme/mod.rs:2210` |
| Flex + truncation authoring | `.min_w_0()` is applied at flex boundaries to allow ellipsis/shrink. | Same requirement in Fret; missing `min_w_0()` causes text to refuse shrinking and forces layout to overflow/clip unexpectedly. | Hard-to-debug truncation/overflow issues when porting Tailwind `truncate` patterns. | **Authoring rule**: require `min_w_0()` on the flex item that hosts ellipsized text. Consider adding higher-level helpers in `fret-ui-kit` if this repeats. | Known pitfall | `tools/diag-scripts/todo-layout-debug.json` | `apps/fret-examples/src/todo_demo.rs:872`<br>`repo-ref/zed/crates/editor/src/element.rs:8236` |

## Candidate defaults to audit next

These are the next “default semantics” candidates to compare and, if needed, align:

- **Overflow defaults**: when a parent clips vs allows overflow, and how that affects ellipsis + hit-testing.
- **Scroll containers**: default min/max sizing rules for scroll areas and how they interact with flex children.
- **Focus-visible + ring**: default focus ring bounds and ring-offset semantics (especially inside clipped parents).
- **Overlay clipping roots**: whether transforms/layout affect hit-testing the same way as upstream.

For each candidate, prefer leaving behind:

- A small invariant test (unit or layout-level), and/or
- A deterministic `tools/diag-scripts/*.json` repro with stable `test_id`.
