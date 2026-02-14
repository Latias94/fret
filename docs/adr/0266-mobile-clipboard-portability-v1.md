# ADR 0266: Mobile Clipboard Portability (v1)

Status: Proposed

## Context

Fret already treats clipboard access as a platform service accessed via effects (ADR 0041), with a
portable “read text” completion delivered back into the window event stream.

Mobile platforms add two constraints that become extremely painful to retrofit later:

1. **Privacy / user activation restrictions**: reading clipboard may be denied, rate-limited, or
   trigger user-visible privacy prompts unless initiated by an explicit user gesture.
2. **Sandbox and representation**: “clipboard files” are often exposed as URIs/handles, not paths.

This ADR locks a mobile-friendly baseline for clipboard behavior without forcing policy into
`crates/fret-ui` (ADR 0066).

## Goals

1. Keep clipboard access **effect-driven** and portable across desktop, web, and future mobile.
2. Make clipboard reads explicitly **best-effort** and safe under mobile privacy constraints.
3. Preserve a clear extension seam for rich clipboard formats and file-like payloads (future).
4. Keep high-level policy (“when to request paste”, “paste button enablement”) in ecosystem/app code.

## Non-goals (v1)

- Implement rich clipboard formats (images, HTML, MIME bytes).
- Implement clipboard file handles.
- Define selection-handle UI/gesture policy (ecosystem responsibility).

## Decision

### D1 — Clipboard text remains the portable baseline

The portable v1 surface is unchanged and remains normative:

- `Effect::ClipboardSetText { text }`
- `Effect::ClipboardGetText { window, token }`
- Completion:
  - `Event::ClipboardText { token, text }`
  - `Event::ClipboardTextUnavailable { token }`

This surface is intentionally minimal and keeps `crates/fret-ui` free of platform branching.

### D2 — Clipboard reads are best-effort and may be denied on mobile

Contract rules:

- Runners MAY deny clipboard reads when:
  - the app is not in the foreground,
  - the request is not directly user-initiated (user activation),
  - the platform restricts clipboard reads for privacy reasons,
  - or the backend is unavailable.
- When denied/unavailable, runners MUST complete the request with:
  - `Event::ClipboardTextUnavailable { token }`
  rather than blocking or panicking.

Implication for ecosystem/app code:

- Treat “paste” as a user action that can fail.
- Do not poll the clipboard as a reactive data source.

### D3 — Clipboard writes are best-effort (no read-back guarantee)

Contract rules:

- `ClipboardSetText` is a best-effort request.
- Runners MAY ignore or reject it in constrained environments.
- The contract does not require read-after-write to succeed across platforms.

### D4 — Primary selection is capability-gated and is expected to be false on mobile

Linux primary selection is modeled separately (existing contract):

- `Effect::PrimarySelectionSetText { text }`
- `Effect::PrimarySelectionGetText { window, token }`

Mobile runners SHOULD report `clipboard.primary_text = false` and ignore these effects.

### D5 — Future extension seam: rich clipboard and file-like payloads (non-normative)

When adding richer clipboard support, preserve the same principles:

- token-based ownership,
- effect-driven reads with explicit limits,
- portable typed payloads (no raw paths/URIs).

A likely future direction is:

- `Effect::ClipboardRead { window, token, formats, limits }`
- Completion:
  - `Event::ClipboardPayload { token, payload }`
  - `Event::ClipboardPayloadUnavailable { token }`

Where `payload` can include:

- text,
- MIME-typed byte blobs (bounded),
- and file-like handles aligned with ADR 0264 (sandboxed mobile).

## Consequences

- Mobile privacy constraints do not force breaking changes to the UI ecosystem.
- Clipboard becomes a reliable “command-like” interaction surface (paste/copy) rather than a reactive data feed.
- Future rich clipboard support can be layered without introducing paths/URIs into contract crates.

## References

- Clipboard as effects + tokens: `docs/adr/0041-drag-and-drop-clipboard-and-cross-window-drag-sessions.md`
- Platform boundary: `docs/adr/0003-platform-boundary.md`
- Text editing commands and paste behavior: `docs/adr/0044-text-editing-state-and-commands.md`
- Rich content selection and copy semantics: `docs/adr/0108-rich-content-selection-and-clipboard.md`
- Read-only selection + clipboard commands: `docs/adr/0137-readonly-text-selection-and-clipboard.md`
- Mobile file picker + sandbox handles: `docs/adr/0264-mobile-file-picker-and-sandbox-handles-v1.md`
- Mobile shell ↔ runtime bridge: `docs/adr/0260-mobile-shell-runtime-bridge-v1.md`

