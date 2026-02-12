# ADR 0264: Mobile File Picker and Sandbox Handle Semantics (v1)

Status: Proposed

## Context

Mobile platforms are commonly sandboxed:

- Android file picking often returns SAF `content://` URIs (permission-scoped, not real paths).
- iOS file picking often requires security-scoped access and does not guarantee stable file paths.

Even on desktop and wasm, “real filesystem paths” are not a portable abstraction (ADR 0053).

Fret already models external drops and file dialogs as **token + safe metadata**, with reading done
via effects. This ADR locks the mobile-facing semantics so ecosystem code never depends on
desktop-only path assumptions.

## Goals

1. Keep file picking portable across desktop + wasm + future mobile.
2. Ensure selection results are safe to display in UI but do not expose privileged paths.
3. Ensure file bytes are obtained via effect-driven platform reads, not direct filesystem access.
4. Provide an explicit seam for future “persistent access” behavior on sandboxed platforms.

## Non-goals (v1)

- A “save file” dialog contract (export) and directory picking (future).
- A full virtual filesystem abstraction.
- A contract that guarantees stable access across app restarts without an explicit persistence step.

## Decision

### D1 — File picker selection is token-based (no paths)

File picker selection returns:

- an opaque `FileDialogToken`, and
- a list of safe `ExternalDragFile` metadata (name, optional size, optional media type).

This is the portable contract surface:

- `crates/fret-core/src/file_dialog.rs` (`FileDialogSelection`, `FileDialogOptions`)
- `crates/fret-runtime/src/effect.rs` (`Effect::FileDialogOpen`)

Rules:

- The selection MUST NOT expose `PathBuf` in `fret-core` or `fret-runtime`.
- Runners MAY keep platform-specific references (paths/URIs/bookmarks) behind the token.

### D2 — Reading bytes is effect-driven and bounded

To obtain bytes for the selected files, apps request reads via effects:

- `Effect::FileDialogReadAll { window, token }`
- `Effect::FileDialogReadAllWithLimits { window, token, limits }`

The runner/platform backend responds via:

- `Event::FileDialogData(FileDialogDataEvent)` on success (partial success allowed),
- `Event::FileDialogCanceled` when no selection occurred,
- and records per-file errors in the returned data event.

Read limits exist to prevent OOM and to keep mobile behavior predictable.

### D3 — Token lifetime is explicit; release is required

File dialog selections are runner-owned resources. Callers MUST release tokens when done:

- `Effect::FileDialogRelease { token }`

After release:

- future reads for that token MUST be rejected/ignored (best-effort),
- runners SHOULD drop platform references and revoke any temporary access where applicable.

### D4 — Persistence is an explicit, app-owned step (future seam)

On sandboxed platforms, selection access may be temporary.

Contract rule (v1):

- Apps MUST treat file picker tokens as **ephemeral** and MUST NOT assume they survive:
  - app restart,
  - process death,
  - or platform revocation.

Recommended app pattern:

- “Import” selected bytes into an app-owned asset database or workspace storage (ADR 0026),
  then refer to that app-owned representation moving forward.

Future seam (non-normative, v2 candidate):

- introduce an explicit “persist selection handle” effect that:
  - requests durable access when the platform supports it,
  - returns an app-owned durable token/handle,
  - and remains portable (no raw paths).

### D5 — Platform mapping guidance (non-normative)

Runners may implement the token mapping as follows:

- Desktop: token -> `PathBuf` list (internal only), read via filesystem APIs.
- Web: token -> `web_sys::File` list, read via `File.arrayBuffer()`.
- Android: token -> SAF `Uri` list + granted permissions, read via `ContentResolver`.
- iOS: token -> security-scoped URL/bookmark reference, read via `NSFileCoordinator` / file APIs.

All of these remain hidden behind the token-based contract.

## Consequences

- Component ecosystems remain mobile-ready by construction (no path leakage).
- Platform-specific permission quirks are localized to runners/platform crates.
- File import UX can be made consistent across platforms by reusing the same effect-driven read
  pipeline as external drops (ADR 0053).

## References

- Platform boundary + effects: `docs/adr/0003-platform-boundary.md`
- External payload portability (token model): `docs/adr/0053-external-drag-payload-portability.md`
- Platform capabilities gating (file dialogs availability): `docs/adr/0054-platform-capabilities-and-portability-matrix.md`
- Asset database/import pipeline (import into app-owned storage): `docs/adr/0026-asset-database-and-import-pipeline.md`
- Mobile shell ↔ runtime bridge: `docs/adr/0260-mobile-shell-runtime-bridge-v1.md`

