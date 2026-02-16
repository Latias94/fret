//! Mixed stream utilities (json-render-inspired).
//!
//! In "chat" scenarios, an LLM may stream conversational text interleaved with JSONL patch lines.
//! This module provides a small, deterministic parser that:
//! - splits chunks into lines,
//! - supports ```spec fences (preferred),
//! - classifies lines as either text or JSON Patch operations,
//! - optionally applies patches into a `SpecStreamCompiler`.

use serde_json::Value;

use crate::spec_stream::{JsonPatch, SpecStreamCompiler, SpecStreamError};

const SPEC_FENCE_OPEN: &str = "```spec";
const SPEC_FENCE_CLOSE: &str = "```";

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum MixedStreamMode {
    /// Accept both text and patch lines. Outside fences, use heuristic classification.
    Mixed,
    /// Only accept patch lines (and fences). Any other non-empty line is an error.
    PatchOnly,
}

#[derive(Debug, Clone)]
pub struct MixedStreamOptions {
    pub mode: MixedStreamMode,
    /// Max buffered bytes (to prevent unbounded growth if the input never contains newlines).
    pub max_buffer_bytes: usize,
    /// Max single-line length (defense-in-depth for pathological inputs).
    pub max_line_bytes: usize,
}

impl Default for MixedStreamOptions {
    fn default() -> Self {
        Self {
            mode: MixedStreamMode::Mixed,
            max_buffer_bytes: 256 * 1024,
            max_line_bytes: 64 * 1024,
        }
    }
}

#[derive(Debug, thiserror::Error)]
pub enum MixedStreamError {
    #[error("mixed stream buffer exceeded limit ({limit} bytes)")]
    BufferLimit { limit: usize },
    #[error("mixed stream line exceeded limit ({limit} bytes)")]
    LineLimit { limit: usize },
    #[error("invalid patch-only stream: {message}")]
    PatchOnly { message: String },
    #[error(transparent)]
    PatchApply(#[from] SpecStreamError),
}

#[derive(Debug, Clone, PartialEq)]
pub enum MixedStreamEvent {
    Text(String),
    Patch(JsonPatch),
}

/// Parse a single json-render-style JSONL line into one or more patches.
///
/// Heuristics (mirrors upstream intent):
/// - returns `None` when the line doesn't look like JSON Patch
/// - returns `None` when JSON parsing fails (tolerant in mixed mode)
/// - accepts either a single patch object or an array of patch objects
pub fn parse_spec_stream_line(line: &str) -> Option<Vec<JsonPatch>> {
    let trimmed = line.trim();
    if trimmed.is_empty() {
        return None;
    }

    if trimmed.starts_with('{') {
        serde_json::from_str::<JsonPatch>(trimmed)
            .ok()
            .map(|p| vec![p])
    } else if trimmed.starts_with('[') {
        serde_json::from_str::<Vec<JsonPatch>>(trimmed).ok()
    } else {
        None
    }
}

fn strip_sse_prefix(line: &str) -> &str {
    let trimmed = line.trim_start();
    let Some(rest) = trimmed.strip_prefix("data:") else {
        return line;
    };
    rest.strip_prefix(' ').unwrap_or(rest)
}

#[derive(Debug, Clone)]
pub struct MixedStreamParser {
    opts: MixedStreamOptions,
    buffer: String,
    in_spec_fence: bool,
}

impl MixedStreamParser {
    pub fn new(opts: MixedStreamOptions) -> Self {
        Self {
            opts,
            buffer: String::new(),
            in_spec_fence: false,
        }
    }

    /// Push a chunk and return any parsed events for complete lines.
    pub fn push_chunk(&mut self, chunk: &str) -> Result<Vec<MixedStreamEvent>, MixedStreamError> {
        if self.buffer.len().saturating_add(chunk.len()) > self.opts.max_buffer_bytes {
            return Err(MixedStreamError::BufferLimit {
                limit: self.opts.max_buffer_bytes,
            });
        }
        self.buffer.push_str(chunk);

        let mut out: Vec<MixedStreamEvent> = Vec::new();
        loop {
            let Some(nl) = self.buffer.find('\n') else {
                if self.buffer.len() > self.opts.max_line_bytes {
                    return Err(MixedStreamError::LineLimit {
                        limit: self.opts.max_line_bytes,
                    });
                }
                break;
            };

            let mut line = self.buffer[..nl].to_string();
            self.buffer.drain(..=nl);
            if line.ends_with('\r') {
                line.pop();
            }

            self.process_line(&line, &mut out)?;
        }

        Ok(out)
    }

    /// Flush any remaining buffered content (call when the stream ends).
    pub fn flush(&mut self) -> Result<Vec<MixedStreamEvent>, MixedStreamError> {
        let mut out = Vec::new();
        if !self.buffer.is_empty() {
            if self.buffer.len() > self.opts.max_line_bytes {
                return Err(MixedStreamError::LineLimit {
                    limit: self.opts.max_line_bytes,
                });
            }
            let line = std::mem::take(&mut self.buffer);
            self.process_line(&line, &mut out)?;
        }
        Ok(out)
    }

    fn process_line(
        &mut self,
        line: &str,
        out: &mut Vec<MixedStreamEvent>,
    ) -> Result<(), MixedStreamError> {
        let line = strip_sse_prefix(line);
        let trimmed = line.trim();
        if trimmed.is_empty() {
            return Ok(());
        }

        if trimmed == "[DONE]" {
            return Ok(());
        }

        // Fence detection
        if !self.in_spec_fence && trimmed.starts_with(SPEC_FENCE_OPEN) {
            self.in_spec_fence = true;
            return Ok(());
        }
        if self.in_spec_fence && trimmed == SPEC_FENCE_CLOSE {
            self.in_spec_fence = false;
            return Ok(());
        }

        if self.in_spec_fence {
            if let Some(patches) = parse_spec_stream_line(trimmed) {
                out.extend(patches.into_iter().map(MixedStreamEvent::Patch));
                return Ok(());
            }
            return match self.opts.mode {
                MixedStreamMode::Mixed => Ok(()), // Drop non-patch lines inside fences.
                MixedStreamMode::PatchOnly => Err(MixedStreamError::PatchOnly {
                    message: "non-patch line inside ```spec fence".to_string(),
                }),
            };
        }

        // Outside fence: heuristic mode (or strict patch-only).
        if let Some(patches) = parse_spec_stream_line(trimmed) {
            out.extend(patches.into_iter().map(MixedStreamEvent::Patch));
            return Ok(());
        }

        match self.opts.mode {
            MixedStreamMode::Mixed => {
                out.push(MixedStreamEvent::Text(line.to_string()));
                Ok(())
            }
            MixedStreamMode::PatchOnly => Err(MixedStreamError::PatchOnly {
                message: "unexpected text line in patch-only mode".to_string(),
            }),
        }
    }
}

#[derive(Debug, Default, Clone)]
pub struct MixedStreamDelta {
    pub text_lines: Vec<String>,
    pub patches: Vec<JsonPatch>,
}

/// Convenience wrapper: parse a mixed stream and apply any recognized patches to a spec value.
#[derive(Debug, Clone)]
pub struct MixedSpecStreamCompiler {
    parser: MixedStreamParser,
    compiler: SpecStreamCompiler,
}

impl MixedSpecStreamCompiler {
    pub fn new(opts: MixedStreamOptions) -> Self {
        Self {
            parser: MixedStreamParser::new(opts),
            compiler: SpecStreamCompiler::new(),
        }
    }

    pub fn result(&self) -> &Value {
        self.compiler.result()
    }

    pub fn into_result(self) -> Value {
        self.compiler.into_result()
    }

    pub fn push_chunk(&mut self, chunk: &str) -> Result<MixedStreamDelta, MixedStreamError> {
        let events = self.parser.push_chunk(chunk)?;
        self.apply_events(events)
    }

    pub fn flush(&mut self) -> Result<MixedStreamDelta, MixedStreamError> {
        let events = self.parser.flush()?;
        self.apply_events(events)
    }

    fn apply_events(
        &mut self,
        events: Vec<MixedStreamEvent>,
    ) -> Result<MixedStreamDelta, MixedStreamError> {
        let mut delta = MixedStreamDelta::default();
        for ev in events {
            match ev {
                MixedStreamEvent::Text(t) => delta.text_lines.push(t),
                MixedStreamEvent::Patch(p) => {
                    self.compiler.apply_patch(&p)?;
                    delta.patches.push(p);
                }
            }
        }
        Ok(delta)
    }
}
