# shadcn/ui v4 Audit — Sonner (Toast)


## Upstream references (non-normative)

This document references optional local checkouts under `repo-ref/` for convenience.
Upstream sources:

- shadcn/ui: https://github.com/shadcn-ui/ui

See `docs/repo-ref.md` for the optional local snapshot policy and pinned SHAs.
This audit compares Fret’s shadcn-aligned `Sonner` surface against the upstream shadcn/ui v4
integration of `sonner` (toast notifications) in `repo-ref/ui`.

## Upstream references (source of truth)

- shadcn wrapper: `repo-ref/ui/apps/v4/registry/new-york-v4/ui/sonner.tsx`
- Demo usage (action/cancel/promise): `repo-ref/ui/apps/v4/app/(internal)/sink/components/sonner-demo.tsx`

Notes:
- Upstream uses the `sonner` JS library. We do **not** aim for API compatibility; we port behavior
  outcomes and authoring ergonomics.

## Fret implementation

- Shadcn-facing facade: `ecosystem/fret-ui-shadcn/src/sonner.rs`
- Core policy + rendering: `ecosystem/fret-ui-kit/src/window_overlays/*` (`toast.rs` + `render.rs`)

## Audit checklist

### Authoring ergonomics

- Pass: Global entry point via `Sonner::global(app)`.
- Pass: Message-style API exists:
  - `Sonner::toast_message(...)`
  - `Sonner::toast_{success,error,info,warning,loading}_message(...)`
  - Options via `ToastMessageOptions` (description/action/cancel/duration/pinned/dismissible).
- Pass: Upsert-by-id for `loading -> success/error` flows.
- Pass: Manual promise handle via `Sonner::toast_promise(...) -> ToastPromise`.

### Interaction behavior

- Pass: Action and cancel buttons dispatch a command and close the toast.
- Pass: Close button is rendered for dismissible toasts.
- Pass: Hover pauses the auto-close timer and resumes from the remaining time.
- Pass: Swipe-to-dismiss is supported (dismissible-gated).

### Stacking & placement

- Pass: Positions include corners and center (`TopCenter` / `BottomCenter` supported).
- Pass: Newest-toasts ordering matches common UX (top stacks newest at top edge, bottom stacks newest at bottom edge).
- Pass: `max_toasts` is supported per-window; eviction prefers non-pinned toasts.

## Validation

- `cargo test -p fret-ui-kit window_overlays::toast`
- `cargo test -p fret-ui-shadcn --lib sonner`

## Follow-ups (recommended)

- Add async integration helpers in app code (runner-level tasks), if needed for true `promise` parity.
- Consider action/cancel styling parity with upstream examples (button variants, spacing, typography).
- A11y is intentionally deferred for now (see `docs/a11y-acceptance-checklist.md`).

