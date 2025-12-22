# ADR 0053: External Drag-and-Drop Payload Portability (Desktop + wasm)

Status: Proposed
Scope: `fret-core` input event contracts + platform boundary (`fret-platform-*`)

## Context

Fret supports external OS drag-and-drop for “files dropped onto the editor” workflows (assets import, open scene,
etc.). Today, `fret-core` represents external file drags as:

- `ExternalDragKind::{EnterFiles, OverFiles, DropFiles}(Vec<PathBuf>)`

This works on desktop, but it is not a stable cross-platform contract:

- **wasm/web** does not provide real filesystem paths for dropped items.
- Sandboxed environments may provide only “file handles” or “blob tokens”.
- Exposing real paths broadly can be undesirable from a security and portability perspective.

If we keep `PathBuf` in `fret-core` events, we will face a breaking change when we add a web runner/backend.

## Goals

- Keep `fret-core` portable across platforms (including wasm).
- Preserve the effect-driven platform boundary (ADR 0003 / ADR 0001).
- Allow desktop backends to provide high-quality UX (file names, multiple file drops, best-effort hover).
- Avoid forcing the UI layer to read files directly from OS paths.

## Decision (Proposed)

Replace “external file drag payload = `PathBuf`” with a portable representation based on **opaque handles**.

### 1) `fret-core` event payload becomes a handle type

Introduce an `ExternalFileRef` (name TBD) that is:

- cloneable, comparable, and printable for diagnostics,
- does **not** expose a concrete OS path in the core contract,
- may carry safe metadata useful for UX (e.g. display name, MIME type hint, size hint).

Example sketch:

```rust
pub struct ExternalFileRef {
    pub token: ExternalFileToken,
    pub display_name: Arc<str>,
}
```

### 2) File access is effect-driven via the platform boundary

To read data or resolve a concrete source, the app requests it via effects, e.g.:

- `Effect::ExternalFileReadBytes { token, range_hint }`
- `Effect::ExternalFilePersistToProject { token, dest_dir }`
- or a simpler initial step: `Effect::ExternalFileOpen { token }`

The runner/platform backend is responsible for producing the data and delivering it back as an event or callback
message (similar to `ClipboardGetText -> Event::ClipboardText`).

### 3) Desktop backends may still expose `PathBuf` *internally*

The winit desktop backend may keep a `PathBuf` mapping behind the token, but this stays within:

- `fret-platform-winit` / runner state,
- or an app-owned “import service”.

The core contract remains portable.

## Consequences

- We can add a web runner without breaking `fret-core` input enums.
- Security posture improves (paths are not broadcast through UI events by default).
- Demo/editor code will need to be updated to request file import through effects rather than reading paths directly.

## Migration Plan (Suggested)

1. Add the new token-based types alongside the current `PathBuf` events (temporary compatibility).
2. Update demo import pipeline to use the effect-driven API.
3. Deprecate the `PathBuf` variants and remove them in a versioned breaking change window.

## Open Questions

- Should `ExternalFileRef` support non-file payloads (text/URIs/images) for future web use?
- Do we want a unified “external drag payload registry” similar to internal drag sessions (ADR 0041)?
- How to model progressive read (streams) vs eager read (bytes) for large assets?

## References

- ADR 0001: `docs/adr/0001-app-effects.md`
- ADR 0003: `docs/adr/0003-platform-boundary.md`
- ADR 0041: `docs/adr/0041-drag-and-drop-clipboard-and-cross-window-drag-sessions.md`

