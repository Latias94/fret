# fret-bootstrap-diag-ws

Thin `fret-bootstrap` bridge for diagnostics WebSocket transport.

This crate keeps the transport-specific `fret-diag-ws` client integration out of
`fret-bootstrap` while preserving the existing `fret-bootstrap/diagnostics-ws` feature story.

Current scope:

- derive devtools WS config from the host environment or web query string
- provide the diagnostics transport bridge used by `fret-bootstrap::ui_diagnostics`
