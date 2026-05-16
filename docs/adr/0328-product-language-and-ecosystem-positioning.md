# ADR 0328: Product Language and Ecosystem Positioning

Status: Accepted

## Context

Fret documentation has historically used overlapping descriptions: a GPU-first Rust UI framework,
a game-editor-grade UI framework, a general-purpose app framework, and a shadcn-like component
ecosystem. Each phrase is useful, but without a shared product vocabulary future readers can
mistake capability targets for product ownership, or mistake the default component path for the
whole ecosystem.

## Decision

Fret is positioned as a **GPU-first Rust application UI framework** with editor-grade scalability,
a desktop-first platform strategy, a WebGPU/wasm path, a shadcn-backed golden path for App Authors,
and a broader official Component Ecosystem.

This means:

- "Editor-grade" describes UI capability level, not ownership of editor or engine product domains.
- The Golden Path is the recommended App Author route, not the only supported route.
- shadcn is the Default Component Surface, not the whole Component Ecosystem.
- Incubating surfaces such as Material 3 remain official ecosystem surfaces even when they are not
  golden-path defaults.
- `fret-ui` remains the Runtime Substrate; component behavior and visual recipes live in Policy
  Layer and ecosystem crates.
- Fret is GPU-first, not GPU-only; headless, diagnostic, semantic, and test surfaces remain valid.

## Consequences

README, onboarding, crate documentation, templates, and diagnostics docs should use the shared
terms from `CONTEXT.md` when describing Fret's product surface. ADRs remain the source of truth for
hard contracts; `CONTEXT.md` is the glossary for the language used to discuss those contracts.
