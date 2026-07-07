# shadcn/ui v4 Audit - Tooltip

## Upstream references (non-normative)

This document references optional local checkouts under `repo-ref/` for convenience.
Upstream sources:

- MUI Base UI: https://github.com/mui/base-ui
- shadcn/ui: https://github.com/shadcn-ui/ui

See `docs/repo-ref.md` for the optional local snapshot policy and pinned SHAs.
This audit compares Fret's shadcn-aligned `Tooltip` against upstream shadcn/ui v4 recipes,
Radix semantics, and Base UI `Tooltip.Root` lifecycle behavior.

## Upstream references (source of truth)

- shadcn docs: `repo-ref/ui/apps/v4/content/docs/components/base/tooltip.mdx`
- shadcn registry (new-york-v4): `repo-ref/ui/apps/v4/registry/new-york-v4/ui/tooltip.tsx`
- Base UI root contract: `repo-ref/base-ui/packages/react/src/tooltip/root/TooltipRoot.tsx`

## Fret implementation anchors

- Component code: `ecosystem/fret-ui-shadcn/src/tooltip.rs`
- Declarative semantics invalidation: `crates/fret-ui/src/declarative/mount.rs`
- Hover intent and delay group: `ecosystem/fret-ui-kit/src/headless/hover_intent.rs`
- Overlay policy and placement helpers: `ecosystem/fret-ui-kit/src/overlay.rs`

## Audit checklist

### Composition & behavior surface

- Pass: Declarative `Tooltip`, `TooltipTrigger`, `TooltipContent`, and `TooltipProvider` are present.
- Pass: Supports open on hover and keyboard focus.
- Pass: Supports provider close delay aliases (`close_delay_duration_ms`, `close_delay_duration_frames`).
- Pass: Supports Base UI cursor tracking via `Tooltip::track_cursor_axis(...)`.
- Pass: `Tooltip::new(cx, trigger, content)` already acts as the recipe-level composition entry point,
  with `TooltipTrigger` / `TooltipContent` preserving shadcn-style part naming at call sites.
- Pass: The typed compound-parts lane is explicit on both sides via
  `TooltipTrigger::build(...)` and `TooltipContent::build(cx, ...)`, so first-party examples do
  not need to hand-land tooltip parts through `AnyElement`.
- Pass: UI Gallery now mirrors the shadcn/base docs path through `API Reference`, then keeps
  `Long Content` and `Keyboard Focus` as explicit Fret-only parity follow-ups.
- Pass: UI Gallery now locks that docs surface with a tooltip-specific docs-surface text test and a
  docs-smoke diagnostics script, so docs-path drift is treated separately from runtime parity.
- Pass: The first-party `Usage` snippet now wraps its local example in `TooltipProvider` so the
  code tab stays standalone and copyable while still teaching the default `Tooltip::new(...)`
  root lane.
- Pass: Tooltip dismissal clears the trigger's described-by relationship on Escape and outside
  press in the same post-dismiss frame. The underlying `Pressable` declarative diff now treats
  a11y relation changes as semantic invalidations, so dynamic `aria-describedby`-style relations
  do not reuse stale semantics snapshots.
- Note: Fret intentionally does not add a separate generic `children([...])` / `compose()` root
  builder for `Tooltip` today. Unlike modal overlays that need explicit portal/overlay/content
  slot assembly, tooltip authoring is already expressed by the root's required trigger/content
  slots plus root/provider-owned hover policy (`TooltipProvider`, delay settings,
  `track_cursor_axis`, `anchor_element`). An extra builder would mostly duplicate the current
  contract without improving semantics.

### Open lifecycle semantics (Base UI parity)

- Pass: `Tooltip::on_open_change(...)` emits when open state changes.
- Pass: `Tooltip::on_open_change_with_reason(...)` emits open state changes with reason metadata
  (`TriggerHover` / `TriggerFocus` / `TriggerPress` / `OutsidePress` / `FocusOutside` /
  `EscapeKey` / `Scroll` / `None`).
- Pass: `Tooltip::on_open_change_complete(...)` emits when transition settles
  (`present == open && !animating`).
- Pass: Callbacks are edge-triggered (no duplicate events when state is unchanged).

### Placement & visual defaults

- Pass: Content is rendered through overlay root (portal-like behavior).
- Pass: Supports side/align/offset and arrow defaults aligned with shadcn usage.
- Pass: Motion taxonomy aligns with shadcn expectations (fade/zoom/slide with transform origin).
- Pass: `TooltipContent` applies the upstream `text-xs` style to descendant text nodes by default
  (when no explicit text style is provided), matching the container-level `text-xs` class in shadcn.

## Known gaps

- Gap: Base UI `onOpenChange` includes cancellable event details (`isCanceled`, `preventUnmountOnClose`);
  Fret currently exposes non-cancelable callback metadata.

## Validation

- `cargo nextest run -p fret-ui --lib declarative::tests::semantics::declarative_pressable_a11y_relation_changes_refresh_semantics_snapshot`
- `cargo nextest run -p fret-ui-shadcn --lib tooltip::tests`
- `cargo nextest run -p fret-ui-shadcn --test tooltip_hover_and_escape`
- `cargo nextest run -p fret-ui-gallery --test tooltip_docs_surface`
- `cargo nextest run -p fret-ui-shadcn --features web-goldens --test web_vs_fret_overlay_chrome tooltip::fixtures`
- `cargo nextest run -p fret-ui-shadcn --features web-goldens --test web_vs_fret_overlay_placement misc_overlays::fixtures::web_vs_fret_misc_overlays_tooltip_cases_match_web_fixtures`
- `cargo run -p fretboard-dev -- diag run tools/diag-scripts/ui-gallery-tooltip-docs-smoke.json --dir target/fret-diag --session-auto --pack --ai-packet --launch -- cargo run -p fret-ui-gallery`
