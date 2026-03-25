# ADR 0053: External Drag-and-Drop Payload Portability (Desktop + wasm)

Status: Accepted
Scope: `fret-core` input event contracts + platform boundary (`fret-platform-*`)

## Context

Fret supports external OS drag-and-drop for “files dropped onto the editor” workflows (asset import, open scene,
etc.). A desktop-only contract based on `PathBuf` is not portable:

- **wasm/web** does not provide real filesystem paths for dropped items.
- Sandboxed environments may provide only “file handles” or “blob tokens”.
- Exposing real paths broadly can be undesirable from a security and portability perspective.

If we keep `PathBuf` in `fret-core` events, we will face a breaking change when we add a web runner/backend.

## Goals

- Keep `fret-core` portable across platforms (including wasm).
- Preserve the effect-driven platform boundary (ADR 0003 / ADR 0001).
- Allow desktop backends to provide high-quality UX (file names, multiple file drops, best-effort hover).
- Avoid forcing the UI layer to read files directly from OS paths.

## Decision

Replace “external file drag payload = `PathBuf`” with a portable representation based on **opaque handles**.

### 1) `fret-core` drag event payload becomes a token + metadata

Represent external file drags as:

- a stable `ExternalDropToken` (opaque handle),
- plus safe metadata for UI/UX (`name`, future MIME/size hints).

Concretely:

```rust
pub enum ExternalDragKind {
    EnterFiles(ExternalDragFiles),
    OverFiles(ExternalDragFiles),
    DropFiles(ExternalDragFiles),
    Leave,
}

pub struct ExternalDragFiles {
    pub token: ExternalDropToken,
    pub files: Vec<ExternalDragFile>,
}

pub struct ExternalDragFile {
    pub name: String,
}
```

### 2) File access is effect-driven via the platform boundary

To read data or resolve a concrete source, the app requests it via effects, e.g.:

- `Effect::ExternalDropReadAll { window, token }` (P0 desktop),
- future: range/stream reads or “persist to project” effects.

The runner/platform backend is responsible for producing the data and delivering it back as an event or callback
message (similar to `ClipboardReadText -> Event::ClipboardReadText`).

### 3) Desktop backends may still expose `PathBuf` *internally*

The winit desktop backend may keep a `PathBuf` mapping behind the token, but this stays within:

- `fret-runner-winit` / runner state,
- or an app-owned “import service”.

The core contract remains portable.

## Consequences

- We can add a web runner without breaking `fret-core` input enums.
- Security posture improves (paths are not broadcast through UI events by default).
- Demo/editor code will need to be updated to request file import through effects rather than reading paths directly.

## Migration Plan (Suggested)

1. Switch `fret-core` external drag payloads to tokens (done).
2. Update demo import pipeline to use the effect-driven API (done).
3. Add stricter portability gating via `PlatformCapabilities.dnd.external_payload` (ADR 0054).

## Open Questions

- Should `ExternalFileRef` support non-file payloads (text/URIs/images) for future web use?
- Do we want a unified “external drag payload registry” similar to internal drag sessions (ADR 0041)?
- How to model progressive read (streams) vs eager read (bytes) for large assets?

## References

- ADR 0001: `docs/adr/0001-app-effects.md`
- ADR 0003: `docs/adr/0003-platform-boundary.md`
- ADR 0041: `docs/adr/0041-drag-and-drop-clipboard-and-cross-window-drag-sessions.md`
