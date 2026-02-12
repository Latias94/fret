# ADR 0265: Mobile Share Sheet and Open-In Intents (v1)

Status: Proposed

## Context

Mobile platforms have two “shell-level” integration surfaces that are easy to prototype poorly and
very expensive to fix once app ecosystems depend on them:

1. **Share sheet / export**: user-initiated “send this text/file to another app”.
2. **Open-in / import**: the OS asks the app to open content (files, URIs, or shared text), either
   at cold start or while the app is already running.

Both surfaces are strongly platform-shaped:

- Android uses intents (`ACTION_SEND`, `ACTION_VIEW`, `content://` URIs with scoped permissions).
- iOS uses `UIActivityViewController` (share), and open requests arrive via scene/app delegate APIs
  (often with security-scoped URLs).

If Fret exposes these as “just paths” or as platform-native objects, higher layers will accrete
non-portable branches and force cross-repo rewrites later.

This ADR defines a portable, effect-driven contract for:

- showing a share sheet (export), and
- receiving open-in requests (import),

while keeping `crates/fret-ui` mechanism-only (ADR 0066) and keeping privileged platform handles
runner-owned (ADR 0003).

## Goals

1. Provide a stable, portable share/export mechanism that works on desktop, web, and future mobile.
2. Provide a stable, portable import/open-in mechanism that does not leak paths/URIs.
3. Keep all byte access effect-driven and bounded (avoid unbounded memory copies / OOM hazards).
4. Make token lifetimes explicit (release required) to support sandboxed mobile permission models.

## Non-goals (v1)

- A complete “document provider” abstraction (Android SAF directory trees, iOS bookmark storage).
- Background “open in” handling while the app is fully suspended (platform-specific).
- Rich inter-app semantic metadata beyond name / media type / size (future).

## Decision

### D1 — Share/export is effect-driven and window-scoped

Define a new window-scoped effect for showing a native share sheet:

- `Effect::ShareSheetShow { window, token, items }`

Completion is delivered back into the window event stream:

- `Event::ShareSheetCompleted { token, outcome }`

Rules:

- The request MUST be **best-effort**: runners may ignore or reject when unsupported.
- The request SHOULD only be issued in direct response to a user action (mobile platforms may
  enforce “user activation” constraints).
- The request MUST NOT block the UI thread.

Outcome (v1):

- `Shared` (user completed the share flow),
- `Canceled` (user dismissed),
- `Unavailable` (platform rejects / not supported),
- `Failed { message }` (best-effort; message is safe for logs, not necessarily user-facing).

### D2 — Share payload (v1) is limited and portable

Define a minimal, portable `ShareItem` surface for v1:

- `ShareItem::Text(String)`
- `ShareItem::Url(String)` (must be a valid absolute URL; runners may reject others)
- `ShareItem::Bytes { name: String, mime: Option<String>, bytes: Vec<u8> }`

Notes:

- `Bytes` is intentionally “small payload friendly” and is expected to be used for things like
  exporting a snippet, a small image, or a generated file.
- Runners SHOULD apply size limits and return `Unavailable`/`Failed` rather than attempting to
  allocate unbounded temporary files on mobile.

Future (v2 candidate, non-normative):

- A handle-based share item that can reference app-owned durable storage without copying bytes into
  the effects queue (aligned with the handle strategy in ADR 0264).

### D3 — Open-in/import requests are token-based (no paths/URIs)

When the OS requests that the app open/import content, the runner MUST surface it as a token-based
event rather than as a path/URI:

- `Event::IncomingOpenRequest { window, token, items }`

`IncomingOpenItem` carries safe metadata only:

- `display_name: Option<String>`
- `media_type: Option<String>` (MIME where available)
- `estimated_size_bytes: Option<u64>`
- `kind: IncomingOpenKind` (v1: `FileLike` or `TextLike`)

Rules:

- The runner MUST keep any privileged platform references behind the token.
- The items MUST be safe to display in UI and safe to log.
- Apps MUST treat all incoming-open tokens as **ephemeral** and **revocable**.

### D4 — Reading incoming-open bytes is effect-driven and bounded

To obtain bytes for an incoming-open token, apps request reads via effects:

- `Effect::IncomingOpenReadAll { window, token }`
- `Effect::IncomingOpenReadAllWithLimits { window, token, limits }`

The runner responds via:

- `Event::IncomingOpenData(IncomingOpenDataEvent)` on success (partial success allowed),
- `Event::IncomingOpenUnavailable { token }` when access is denied/expired/unavailable.

Limits use the same shape as external drops and file dialogs:

- `ExternalDropReadLimits` (ADR 0053).

### D5 — Token lifetime is explicit; release is required

Incoming-open tokens are runner-owned resources. Callers MUST release tokens when done:

- `Effect::IncomingOpenRelease { token }`

After release:

- future reads for that token MUST be rejected/ignored (best-effort),
- runners SHOULD revoke temporary access where applicable (mobile sandbox permissions).

### D6 — Mapping guidance (non-normative)

Runners may map these contracts as follows:

- Desktop:
  - Share sheet: native “share” UI where available, else `Unavailable`.
  - Incoming open: OS file association open requests -> `IncomingOpenRequest`.
- Web:
  - Share sheet: `navigator.share` (capability-gated); bytes may require `files` support.
  - Incoming open: PWA share-target / file-handling APIs where available (best-effort).
- Android:
  - Share sheet: intent chooser for `ACTION_SEND` / `ACTION_SEND_MULTIPLE`.
  - Incoming open: `ACTION_VIEW` / `ACTION_SEND` mapped to token + `ContentResolver` reads.
- iOS:
  - Share sheet: `UIActivityViewController`.
  - Incoming open: scene/app delegate open callbacks mapped to token + security-scoped reads.

## Implementation status (non-normative)

As of 2026-02-12:

- Web/WASM:
  - Share sheet is implemented as a best-effort mapping to `navigator.share`, capability-gated via runtime detection.
  - `ShareItem::Bytes` is mapped via Web Share Level 2 (`navigator.share({ files: [...] })`) when supported; runners may still return `Unavailable` when the browser rejects the payload.
  - Incoming-open supports diag-only request injection plus bounded `ReadAll*` and explicit `Release`; there is no OS-produced request plumbing yet.
- Desktop:
  - Share sheet is currently a stub (completes as `Unavailable`).

## Consequences

- Higher-level apps/components never depend on non-portable path/URI strings.
- Sandboxed mobile permission behavior remains runner-owned and can evolve without rewriting UI code.
- Export/import UX becomes effect-driven and diagnostics-friendly (tokens, limits, explicit release).

## References

- Platform boundary and effect model: `docs/adr/0003-platform-boundary.md`, `docs/adr/0001-app-effects.md`
- Clipboard/DnD token model inspiration: `docs/adr/0041-drag-and-drop-clipboard-and-cross-window-drag-sessions.md`
- External payload portability + read limits: `docs/adr/0053-external-drag-payload-portability.md`
- Capabilities gating: `docs/adr/0054-platform-capabilities-and-portability-matrix.md`
- Mobile file picker + sandbox handles: `docs/adr/0264-mobile-file-picker-and-sandbox-handles-v1.md`
- Mobile shell ↔ runtime bridge: `docs/adr/0260-mobile-shell-runtime-bridge-v1.md`
