# Clipboard Write Completion (Fearless Refactor v1)

Status: draft
Last updated: 2026-03-25

Related:

- `docs/adr/0041-drag-and-drop-clipboard-and-cross-window-drag-sessions.md`
- `docs/adr/0266-mobile-clipboard-portability-v1.md`
- `docs/ui-diagnostics-and-scripted-tests.md`
- `docs/workstreams/mobile-share-and-clipboard-v1/design.md`
- `docs/workstreams/clipboard-write-completion-fearless-refactor-v1/richer-clipboard-surfaces.md`

## Why this workstream exists

Clipboard write handling is one of the last places where the public-looking surface still reflects
prototype-era asymmetry instead of the real runtime model.

Today:

- reads are token-completed,
- writes are modeled as fire-and-forget,
- desktop and web runners already know whether a write succeeded or failed,
- but that result is only recorded in diagnostics, not surfaced back into the app event stream.

This creates three concrete problems:

1. copy-oriented components cannot expose honest success/failure callbacks,
2. `copied` UI state is currently optimistic instead of result-driven,
3. multi-window attribution is underspecified because `ClipboardSetText` carries no window.

Because Fret is still pre-release, this should be treated as a **fearless refactor** rather than a
compatibility exercise. The right answer is not to preserve a misleading API shape.

## Problem statement

### Contract drift

`Effect::ClipboardSetText { text }` looks synchronous and non-addressable, but real clipboard
writes are best-effort platform requests that may fail asynchronously.

The current write surface is weaker than the real system in exactly the wrong way:

- it hides completion from app/component code,
- it omits the originating window,
- and it forces higher layers to guess whether "copy" actually happened.

### Component semantic drift

AI copy buttons currently treat "copy requested" as "copy succeeded":

- `on_copy` fires immediately after issuing the effect,
- copied/checkmark state flips optimistically,
- `on_error` is not representable.

That diverges from upstream AI Elements semantics and from what app authors expect.

### Diagnostics / runtime split brain

The runner already captures write success/failure for diagnostics bundles, but runtime consumers
cannot observe the same fact through the normal completion path.

This is the wrong layering boundary:

- diagnostics should observe the runtime contract,
- not be the only place where the contract is effectively richer.

## Goals

1. Make explicit clipboard writes a first-class token/completion contract.
2. Restore honest component semantics for copy success, copy failure, and copied-state UI.
3. Make clipboard write attribution window-aware.
4. Lock an extension seam for non-text clipboard payloads so future rich clipboard work is additive.
5. Keep `crates/fret-ui` mechanism-only by exposing routing primitives, not component policy.
6. Use the pre-release window to delete misleading legacy names instead of preserving them.

## Non-goals

- Rich clipboard payloads (images, HTML, MIME bytes).
- Linux primary-selection redesign for copy-on-select flows.
- App-level policy such as toasts, retry UX, or analytics.
- A generic "listen to every window event" component API.

## Guiding rules

1. Explicit platform requests that can complete asynchronously must be token-addressable.
2. If completion matters to first-party components, it belongs in the runtime contract, not in a
   component-local heuristic.
3. Window-targeted requests should carry `window` explicitly; runners should not guess.
4. Prefer narrow routing hooks over a generic event-bus escape hatch.
5. Prefer deletion over deprecation while the repo is still private/pre-release.

## Current surface vs target surface

### Current surface

- `Effect::ClipboardSetText { text }`
- `Effect::ClipboardGetText { window, token }`
- `Event::ClipboardText { token, text }`
- `Event::ClipboardTextUnavailable { token, message }`

### Target surface

Clipboard should become symmetric for explicit app-facing text transfers while leaving a richer
payload lane available later.

#### P0 shipped lane: text-first

- `Effect::ClipboardWriteText { window, token, text }`
- `Effect::ClipboardReadText { window, token }`
- `Event::ClipboardWriteCompleted { token, outcome }`
- `Event::ClipboardReadText { token, text }`
- `Event::ClipboardReadFailed { token, error }`

Where:

- `ClipboardWriteOutcome`
  - `Succeeded`
  - `Failed { error: ClipboardAccessError }`

#### P1 additive lane: rich payloads

Do not implement this now, but reserve it as the future direction:

- `Effect::ClipboardWritePayload { window, token, payload }`
- `Effect::ClipboardReadPayload { window, token, formats, limits }`
- `Event::ClipboardWriteCompleted { token, outcome }`
- `Event::ClipboardPayload { token, payload }`
- `Event::ClipboardReadFailed { token, error }`

Where `error` is a structured portable type, for example:

- `ClipboardAccessError { kind, message }`
- `kind` initially supports at least:
  - `Unavailable`
  - `PermissionDenied`
  - `UserActivationRequired`
  - `Unsupported`
  - `BackendError`
  - `Unknown`

Where `payload` is a portable typed envelope, for example:

- `ClipboardPayload { items: Vec<ClipboardItem> }`
- `ClipboardItem`
  - `Text(String)`
  - `Html(String)`
  - `Bytes { media_type, bytes }`
  - future read-side file-like handles aligned with ADR 0264

This is intentionally more aggressive than a minimal additive patch. The goal is to leave behind a
surface that is easy to teach and does not encode legacy asymmetry into the first public release.

### Why `ClipboardWriteCompleted { outcome }` instead of separate success/failure events

For text-only writes, separate success/failure events would work.

For future rich payload writes, a single completion event is a better long-term fit because:

- some platforms may accept only a subset of requested representations,
- write completion may eventually need to report accepted formats,
- the shape matches existing completion patterns like `ShareSheetCompleted { outcome }`.

That lets the text lane stay simple now without forcing another event-shape rethink later.

## Naming decision

### Fearless-refactor recommendation

Rename the clipboard request surfaces now:

- `ClipboardSetText` -> `ClipboardWriteText`
- `ClipboardGetText` -> `ClipboardReadText`

And rename the read completion/failure surfaces to match:

- `ClipboardText` -> `ClipboardReadText`
- `ClipboardTextUnavailable` -> `ClipboardReadFailed`

Reasoning:

- `set/get` reads like synchronous state mutation/access.
- `read/write` better matches effect/completion semantics.
- the rename is expensive but one-time; keeping the old names would bake incorrect intuition into
  the public surface.

### Temporary migration rule

During implementation we may keep local adapters behind the branch to reduce churn, but the final
shipped surface should delete the old names rather than re-export them.

## Rich clipboard stance

Text-only clipboard support is sufficient for the immediate parity problem, but it is not the full
long-term shape for an editor-grade cross-platform framework.

The framework should explicitly prepare for at least these representation classes:

- `text/plain`
- `text/html` or attributed/rich text equivalents
- image bytes (for example PNG)
- bounded MIME-typed byte payloads
- read-side file-like handles/tokens rather than raw paths/URIs

What we should **not** do yet:

- lock a desktop-only file-path clipboard contract,
- expose DOM-native clipboard objects,
- force all current text callers onto a generic payload builder,
- promise that every platform can round-trip every representation.

The right split is:

- ship a first-class text lane now,
- reserve a typed payload lane now,
- implement richer formats later without renaming the contract again.

## Ownership by layer

### `crates/fret-core`

- owns the event payloads and structured clipboard error type,
- owns token identifiers,
- remains the source of truth for window-event vocabulary.

### `crates/fret-runtime`

- owns the effect vocabulary,
- owns capability definitions,
- owns the clipboard diagnostics store,
- should route diagnostics from the same completion facts the runtime uses.

### `crates/fret-launch`

- maps desktop/web clipboard APIs to the new write/read completion events,
- must stop treating diagnostics as the only observer of write failures.

### `crates/fret-ui`

- owns narrow action-routing hooks for clipboard completions,
- must not grow copy-policy or toast behavior,
- should provide the minimal mechanism needed for components to react to tokenized completions.

### `ecosystem/*`

- `fret-ui-ai`, `fret-code-view`, `fret-router-ui`, editors, and list/table surfaces migrate to
  the new write request semantics,
- component policies (`on_copy`, `on_error`, copied icon timing) stay here.

## Component routing design

The preferred mechanism is a **clipboard-specific action hook surface**, not a generic event bus.

Target shape:

- add clipboard completion action hooks in `crates/fret-ui/src/action.rs`,
- expose `ElementContext` helpers analogous to timer hooks,
- allow components to register token-aware handlers for:
  - write succeeded,
  - write failed,
  - read succeeded,
  - read failed.

Why not a generic window-event listener:

- it would widen the mechanism surface too far,
- it is harder to reason about consumption and ordering,
- clipboard is already a narrow tokenized platform service.

Why not diagnostics-store polling:

- it is frame-scoped and observational,
- it would keep runtime behavior coupled to diagnostics internals,
- it would still not solve honest callback timing cleanly.

## Copy-component target semantics

All copy-style components should converge on the same behavior:

1. allocate a clipboard token,
2. issue `ClipboardWriteText { window, token, text }`,
3. enter a pending state if desired, but do not show "Copied" yet,
4. on `ClipboardWriteSucceeded { token }`:
   - flip copied/check state,
   - fire `on_copy`,
   - arm the copied-state timeout,
5. on `ClipboardWriteFailed { token, error }`:
   - do not enter copied state,
   - fire `on_error`,
   - optionally expose app-owned retry/toast policy.

This restores honest semantics for:

- `StackTraceCopyButton`
- `CodeBlockCopyButton`
- `SnippetCopyButton`
- `CommitCopyButton`
- `TerminalCopyButton`
- `EnvironmentVariableCopyButton`

For future rich-copy surfaces such as Markdown/code viewers:

- plain text remains the required baseline,
- richer formats may be written in parallel when the caller has them,
- selection semantics from ADR 0108 do not need to change; richer payloads are additive
  representations, not a replacement for the plain-text extraction contract.

## Capability posture

Keep the existing capability keys:

- `clipboard.text`
- `clipboard.text_read`
- `clipboard.text_write`

Do not add a new capability merely for "write completion". Completion is part of the contract when
write support exists.

Future rich payload support may add narrower capability facets later, for example:

- `clipboard.html`
- `clipboard.image`
- `clipboard.payload_bytes`

But that should only happen once a concrete first-party use case exists.

## Diagnostics posture

Diagnostics should move from a side-channel observer to a normal consumer of the same contract.

Required updates:

- document that `set_clipboard_force_unavailable` affects both read and write failure paths,
- keep `last_write_*` fields in bundle snapshots,
- source them from the new write completion events or the same runner result path that emits those
  events,
- add a dedicated diag assertion for write outcomes instead of forcing scripts to infer from
  clipboard contents alone.

Suggested new script steps:

- `wait_clipboard_write_result`
- `assert_clipboard_write_result`

These should support:

- success,
- failure,
- optional expected error kind/message substring.

## Primary selection rule

Do **not** force Linux primary selection onto the same write-completion path in v1.

Reason:

- primary selection is often updated at much higher frequency,
- it is not the user-facing copy-button problem we are solving,
- forcing completion there would add churn and noise without product value.

Keep:

- `PrimarySelectionSetText` as best-effort for now,
- `PrimarySelectionGetText` as token-completed.

## Rich payload portability rules

When the richer payload lane lands, it should follow the same portability rules already used by
external drop and file dialogs:

1. no raw paths or platform URIs in `fret-core`,
2. token-based ownership for privileged/file-like references,
3. bounded byte payloads,
4. portable typed metadata instead of platform-native objects.

This keeps clipboard evolution aligned with:

- ADR 0053 (`ExternalDropToken` style portability),
- ADR 0264 (sandbox handle semantics),
- ADR 0266 D5 (rich clipboard direction).

## Migration plan (fearless but staged)

### Stage 0 — Contract docs first

- add this workstream,
- update ADR 0266 and ADR 0041 once final names are chosen,
- update `IMPLEMENTATION_ALIGNMENT.md` only when code lands.

### Stage 1 — Core contract reset

- add structured clipboard error payloads,
- add write success/failure events,
- add renamed read/write effect/event names,
- keep any compatibility shim internal and temporary only.

### Stage 2 — Runner + diagnostics

- desktop runner emits write success/failure events,
- web runner maps Promise resolution/rejection into the same events,
- diagnostics store records write results from the real completion flow,
- docs for diag clipboard failure simulation are corrected.

### Stage 3 — `fret-ui` routing primitive

- add clipboard completion action hooks,
- add focused unit tests for token routing and non-matching-token isolation,
- avoid introducing a generic event listener API.

### Stage 4 — Caller migration

- migrate all `ClipboardSetText` callers to tokenized writes,
- convert AI copy buttons to honest success/error semantics,
- convert `fret-code-view`, editors, router UI, diagnostics helpers, and devtools callers.

### Stage 5 — Delete transitional surface

- delete old `Set/Get` clipboard names,
- remove optimistic copy comments/docs,
- update UI Gallery notes and prop tables to the new semantics.

## Risks

### Risk: churn across many call sites

This is real, but acceptable. The repo is still private and unpublished; the cost of shipping the
wrong shape is higher than the cost of a one-time migration.

### Risk: tokenized writes add noise for callers that do not care about completion

That is acceptable for explicit copy commands. If some high-frequency path appears later, that path
should use a distinct best-effort surface instead of weakening the primary contract again.

### Risk: structured error kinds may vary by runner

That is expected. `kind` should be best-effort and portable, while `message` remains diagnostic.
The API should allow unknown/fallback mapping without panics.

## Definition of done

This workstream is done when all of the following are true:

1. The clipboard write surface is tokenized and window-aware.
2. Copy-oriented first-party components only report success after real completion.
3. Failure callbacks are representable without runner-specific hacks.
4. Diagnostics can script and assert write failure paths directly.
5. The old `ClipboardSetText` / `ClipboardGetText` naming is removed from the shipped surface.
