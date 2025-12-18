# ADR 0026: Asset Database and Import Pipeline

Status: Deferred
Scope: Example editor application (out of scope for the Fret UI framework)

## Context

A game engine editor must manage assets at scale:

- source files (textures, meshes, scenes, shaders),
- imported/processed artifacts,
- metadata (import settings, GUIDs, dependencies),
- incremental rebuilds when sources change,
- background importing/indexing.

If asset identity, metadata, and import boundaries are not defined early, the editor tends to
accumulate inconsistent references and non-deterministic behavior (especially with hot reload).

This ADR is intentionally **not** a Fret framework commitment. Fret must stay UI-focused, but it
should not block asset tooling: the UI should support background work messages (ADR 0008) and
file-scoped settings (ADR 0014), while the asset database/pipeline remains app-owned.

References:

- Threading boundary and background work model:
  - `docs/adr/0008-threading-logging-errors.md`
- Settings as files and strong types (project/user scopes):
  - `docs/adr/0014-settings-and-configuration-files.md`

## Decision

### 1) Stable asset identity via GUIDs

Each asset has a stable GUID independent of its file path.

- file moves/renames do not change identity
- references in scenes/prefabs store GUIDs, not paths

### 2) Split “source” vs “imported artifacts”

The asset system distinguishes:

- **source assets**: user-authored files in the project (tracked by the editor)
- **imported artifacts**: derived data produced by importers (cached outputs)

Importers are deterministic functions of:

- source bytes,
- importer version,
- import settings.

### 3) Import is incremental and dependency-aware

The asset database tracks dependencies:

- changing a texture reimports dependent materials,
- changing an imported mesh triggers dependent scenes.

Imports run on background threads but publish results to the main thread via data-only messages.

### 4) Project-scoped configuration lives in files

Asset import settings are stored as files (project scope), enabling:

- VCS-friendly workflows,
- reproducible builds across machines.

## Consequences

- Editor references remain stable under refactors and file moves.
- Import behavior is reproducible and debuggable.
- Background processing remains compatible with main-thread UI and wasm constraints.

## Future Work

- Decide artifact cache location and eviction policy.
- Define how plugins register importers and contribute asset metadata UI.
- Define hot-reload semantics (which assets can be swapped live, and how to notify systems).
