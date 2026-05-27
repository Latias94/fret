# shadcn/ui v4 Audit - Sonner (Toast)

Status note (2026-05-27): Sonner is now wired into the component parity matrix as
`sonner.docs-path.desktop-mobile` and promoted to `regression_locked`. This audit is anchored to
the main worktree `repo-ref` snapshot: the current shadcn docs path is `Demo`, `About`, `Usage`,
`Examples` / `Types`, and `Changelog` after skipping web-only installation prose. Fret-specific
`Mounting`, `Description`, `Position`, `API Reference`, and `Extras` remain follow-up sections in
the Gallery rather than current upstream docs-path sections.

## Upstream references (non-normative)

This document references optional local checkouts under `repo-ref/` for convenience.
Upstream sources:

- shadcn/ui: https://github.com/shadcn-ui/ui

See `docs/repo-ref.md` for the optional local snapshot policy and pinned SHAs.
This audit compares Fret's shadcn-aligned `Sonner` surface against the upstream shadcn/ui v4
integration of `sonner` (toast notifications) in `repo-ref/ui`.

## Upstream references (source of truth)

- Docs page: `repo-ref/ui/apps/v4/content/docs/components/sonner.mdx`
- shadcn wrapper: `repo-ref/ui/apps/v4/registry/new-york-v4/ui/sonner.tsx`
- Demo usage (action/promise/status variants): `repo-ref/ui/apps/v4/registry/new-york-v4/examples/sonner-demo.tsx`,
  `repo-ref/ui/apps/v4/registry/new-york-v4/examples/sonner-types.tsx`
- Headless/compound-toast references: `repo-ref/primitives/packages/react/toast/src/toast.tsx`,
  `repo-ref/base-ui/packages/react/src/toast/`

Notes:
- Upstream uses the `sonner` JS library. We do **not** aim for API compatibility; we port behavior
  outcomes and authoring ergonomics.

## Fret implementation

- Shadcn-facing facade: `ecosystem/fret-ui-shadcn/src/sonner.rs`
- Gallery page: `apps/fret-ui-gallery/src/ui/pages/sonner.rs`
- Copyable usage snippet: `apps/fret-ui-gallery/src/ui/snippets/sonner/usage.rs`
- Core policy + rendering: `ecosystem/fret-ui-kit/src/window_overlays/*` (`toast.rs` + `render.rs`)

## Audit checklist

### Authoring ergonomics

- Pass: Global entry point via `Sonner::global(app)`.
- Pass: Mounting surface via `Toaster::new()` covers the upstream "add `<Toaster />` once"
  outcome, including shadcn-aligned Lucide icon defaults.
- Pass: Message-style API exists:
  - `Sonner::toast_message(...)`
  - `Sonner::toast_{success,error,info,warning,loading}_message(...)`
  - Options via `ToastMessageOptions` (description/action/cancel/duration/pinned/dismissible).
- Pass: Upsert-by-id exists for `loading -> success/error` flows.
- Pass: Manual promise handle via `Sonner::toast_promise(...) -> ToastPromise`.
- Pass: The UI Gallery now mirrors the docs-facing structure after collapsing shadcn's top
  `ComponentPreview` into `Demo` and skipping npm-install prose:
  - `Demo`
  - `About`
  - `Usage`
  - `Examples`
  - `Types`
  - `Changelog`
- Pass: `Mounting (Fret)`, `Description (Fret)`, `Position (Fret)`, `API Reference (Fret)`, and
  `Extras (Fret)` stay visible as Fret follow-ups instead of being mislabeled as current upstream
  docs-path sections.
- Pass: Sonner code tabs now use standalone docs sources instead of page-local gallery helpers, so
  copied snippets include the required `Toaster` mount + message dispatch wiring.
- Note: A composable children/custom-content API is still not exposed on the shadcn-facing
  surface. That remains the right call for now: upstream Sonner is message-template-oriented, while
  a richer custom-content lane would belong on a lower-level toast primitive rather than widening
  the default shadcn recipe surface.

### Interaction behavior

- Pass: Action and cancel buttons dispatch a command and close the toast.
- Pass: Close button is rendered for dismissible toasts.
- Pass: Hover pauses the auto-close timer and resumes from the remaining time.
- Pass: Swipe-to-dismiss is supported (dismissible-gated).

### Stacking & placement

- Pass: Positions include corners and center (`TopCenter` / `BottomCenter` supported).
- Pass: Newest-toasts ordering matches common UX (top stacks newest at top edge, bottom stacks newest at bottom edge).
- Pass: `max_toasts` is supported per-window; eviction prefers non-pinned toasts.

### Chrome

- Pass: The shared toast fallback shadow now has dedicated light/dark footprint gates against the
  checked-in `sonner-demo.open` web baseline.
- Pass: The retained generic toast fallback in
  `ecosystem/fret-ui-kit/src/window_overlays/render.rs` intentionally matches the current Sonner
  baseline `rgba(0, 0, 0, 0.1) 0px 4px 12px 0px` instead of acting as an unreviewed placeholder.

### Harness matrix

- Pass: Matrix packet `docs/workstreams/shadcn-component-parity-matrix-v1/artifacts/sonner_agent_packet_p0_v1.json`
  records the current source refs, upstream open snapshots, Fret layout/placement/chrome gates,
  Gallery docs-surface tests, diagnostics scripts, and zero repair/hardening/gate queues.
- Pass: Required state depth for this component is scoped to Sonner-specific evidence:
  hover/pause, swipe drag dismiss, open toast, constrained viewport, text metrics, and paint tokens.
  Keyboard, RTL, and disabled are not currently required because the current shadcn Sonner docs path
  and Fret Sonner recipe do not expose those as component-specific obligations.

## Conclusion

- Result: This component does not currently indicate a missing mechanism-layer gap in the shadcn-facing surface.
- Result: The main drift was in the shadcn recipe/documentation surface, not in the toast mechanism.
- Result: `Toaster::new()` behaves like the upstream wrapper defaults, and the gallery page now
  mirrors the current docs structure more directly instead of only mirroring the example subset.
- Result: The shared toast fallback shadow is now evidence-backed, source-aligned, and intentionally
  retained as the generic toast chrome baseline.
- Result: Follow-up work should focus on richer async/app integration helpers or a lower-level
  custom-content toast primitive only if a concrete product need appears.

## Validation

- `cargo test -p fret-ui-kit window_overlays::toast`
- `cargo test -p fret-ui-shadcn --lib sonner`
- `cargo nextest run -p fret-ui-shadcn --features web-goldens --test web_vs_fret_layout --status-level fail sonner`
- `cargo nextest run -p fret-ui-shadcn --features web-goldens --test web_vs_fret_overlay_placement --status-level fail sonner`
- `cargo nextest run -p fret-ui-shadcn --features web-goldens --test web_vs_fret_overlay_chrome web_vs_fret_sonner_demo_toast_shadow_matches_web_light web_vs_fret_sonner_demo_toast_shadow_matches_web_dark --status-level fail`
- `cargo test -p fret-ui-gallery --test ui_authoring_surface_default_app sonner_ -- --nocapture`
- `cargo test -p fret-ui-gallery --test sonner_docs_surface -- --nocapture`
- `cargo check -p fret-ui-gallery --message-format short`
- `tools/diag-scripts/ui-gallery/sonner/ui-gallery-sonner-docs-screenshots.json`

## Follow-ups (recommended)

- Add async integration helpers in app code (runner-level tasks), if needed for true `promise` parity.
- Consider action/cancel styling parity with upstream examples (button variants, spacing, typography).
- A11y is intentionally deferred for now (see `docs/a11y-acceptance-checklist.md`).
