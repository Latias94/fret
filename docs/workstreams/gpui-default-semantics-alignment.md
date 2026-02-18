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

| Area | Upstream (Zed/GPUI) default | Fret default (prior) | Impact | Fix / owner | Status | Evidence anchors |
|---|---|---|---|---|---|---|
| Flex container width | `h_flex()/v_flex()` relies on the engine’s default “block-like stretch” width semantics. No explicit `width: 100%` is required. | `ui::h_flex/v_flex` left inner flex width as `Auto` in some measure paths, which could propagate “near-zero width” constraints. | Premature ellipsis/wrapping: tabs, dates, and row text truncate even when there is space. | **Policy/infra**: set inner `FlexProps.layout.size.width = Fill` in `ecosystem/fret-ui-kit`. | Aligned | `ecosystem/fret-ui-kit/src/ui.rs:154` |
| Position + inset defaults | Style defaults to `position: relative` (inset offsets can “nudge” without an explicit `relative()` call). | Layout defaults to `PositionStyle::Static`; inset offsets are ignored unless author opts into `.relative()` or `.absolute()`. | Ports that assume “inset works by default” can silently no-op; hard to spot in layout-heavy recipes. | **TBD (policy/infra)**: keep CSS/Tailwind semantics but document the pitfall; consider adding an opt-in helper (e.g. `inset_relative_*`) in `fret-ui-kit` if this repeats. | Open | `repo-ref/zed/crates/gpui/src/style.rs`<br>`crates/fret-ui/src/element.rs`<br>`ecosystem/fret-ui-kit/src/style/layout.rs` |
| Overflow defaults | `overflow: visible` (both axes). | `Overflow::Visible` by default. | Usually neutral; matters when author expects clipping or when hit-testing should be constrained. | No change (authoring sets clip/hidden explicitly). | Aligned | `repo-ref/zed/crates/gpui/src/style.rs`<br>`crates/fret-ui/src/element.rs` |
| Flex shrink defaults | `flex_shrink: 1` (CSS/Tailwind default). | `FlexItemStyle.shrink = 1.0` by default. | Enables shrink + ellipsis patterns; recipes opt out via `shrink-0`/equivalent. | No change. | Aligned | `repo-ref/zed/crates/gpui/src/style.rs`<br>`crates/fret-ui/src/element.rs` |
| Theme snapshot token reads | Theme reads go through the full theme object (no “snapshot silently drops token tables” path). | `ThemeSnapshot` only held typed baseline fields; reading shadcn semantic tokens (e.g. `muted`) could degrade to baseline fallbacks. | “White-on-white” style bugs: track/background tokens collapse so active states are not visible. | **Mechanism**: make `ThemeSnapshot` carry the configured token maps (Arc) so `ThemeSnapshot.color_token(k)` matches `Theme.color_token(k)`. | Aligned | `crates/fret-ui/src/theme/mod.rs:637` |
| Flex + truncation authoring | `.min_w_0()` is applied at flex boundaries to allow ellipsis/shrink. | Same requirement in Fret; missing `min_w_0()` causes text to refuse shrinking and forces layout to overflow/clip unexpectedly. | Hard-to-debug truncation/overflow issues when porting Tailwind `truncate` patterns. | **Authoring rule**: require `min_w_0()` on the flex item that hosts ellipsized text. Consider adding higher-level helpers in `fret-ui-kit` if this repeats. | Known pitfall | `apps/fret-examples/src/todo_demo.rs:872`<br>`repo-ref/zed/crates/editor/src/element.rs:8236` |

## Candidate defaults to audit next

These are the next “default semantics” candidates to compare and, if needed, align:

- **Overflow defaults**: when a parent clips vs allows overflow, and how that affects ellipsis + hit-testing.
- **Scroll containers**: default min/max sizing rules for scroll areas and how they interact with flex children.
- **Focus-visible + ring**: default focus ring bounds and ring-offset semantics (especially inside clipped parents).
- **Overlay clipping roots**: whether transforms/layout affect hit-testing the same way as upstream.

For each candidate, prefer leaving behind:

- A small invariant test (unit or layout-level), and/or
- A deterministic `tools/diag-scripts/*.json` repro with stable `test_id`.
