# ImUi Collection Helper Readiness v1 - Closeout Audit

Status: closed closeout record
Date: 2026-04-24

## Verdict

Treat `imui-collection-helper-readiness-v1` as a closed no-helper-widening verdict.

The lane found no policy-light shared collection helper that both current proof surfaces need. The
right next move is not `fret-ui-kit::imui` API widening; it is to keep collection behavior app-owned
and carry forward only documentation/recipe guidance for stable collection evidence.

## What Shipped

1. A narrow helper-readiness lane was created instead of reopening
   `imui-collection-second-proof-surface-v1`.
2. M1 compared the collection-first asset-browser grid against the shell-mounted `Scene collection`
   outline.
3. M1 rejected generic `collection(...)`, `collection_list(...)`, `collection_rows(...)`,
   `collection_commands(...)`, and `selection_summary(...)` helper candidates for this cycle.
4. The source-policy gate keeps helper readiness separate from helper implementation.
5. No public `fret-imui`, `fret-ui-kit::imui`, or `crates/fret-ui` API changed.

## Why Shared Helper Growth Stays Closed

The two proof surfaces still do not need the same reusable helper shape:

1. The asset-browser proof is a dense grid with multi-select, keyboard owner, marquee selection,
   zoom/layout, context-menu, and command-package policy.
2. The `Scene collection` proof is a compact shell-mounted outline with single selection and editor
   action routing.
3. The overlap is stable evidence vocabulary and test-id discipline, not runtime/helper behavior.
4. Extracting a helper now would either be empty wrapper sugar or would pull app/product policy into
   generic IMUI.

## Reopen Policy

Do not reopen this folder for generic collection helper implementation.

Start a different narrow implementation follow-on only if fresh first-party evidence names:

- one exact helper shape,
- both proof surfaces that need it,
- the app policy that remains outside the helper,
- and a focused gate package that proves the helper is not just documentation/test-id convention.

Until then, keep collection depth app-owned.
