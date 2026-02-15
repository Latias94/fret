//! SpecStream (json-render-inspired): JSONL RFC 6902 patches for progressive spec building.
//!
//! This module is intentionally provider-agnostic: feed it text chunks from any LLM stream.
//! The common pattern is:
//! - Ask the model to output JSONL, one JSON Patch object per line.
//! - As chunks arrive, call `SpecStreamCompiler::push_chunk`.
//! - When patches apply, re-validate and re-render the current compiled spec.

use serde::{Deserialize, Serialize};
use serde_json::Value;

use crate::json_pointer::{JsonPointerError, get_opt};

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "lowercase")]
pub enum PatchOp {
    Add,
    Remove,
    Replace,
    Move,
    Copy,
    Test,
}

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct JsonPatch {
    pub op: PatchOp,
    pub path: String,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub value: Option<Value>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub from: Option<String>,
}

#[derive(Debug, thiserror::Error)]
pub enum SpecStreamError {
    #[error("invalid json patch line")]
    InvalidLine,
    #[error("invalid json patch: {0}")]
    InvalidPatch(#[from] serde_json::Error),
    #[error("json pointer error: {0}")]
    Pointer(#[from] JsonPointerError),
    #[error("test operation failed at {path}")]
    TestFailed { path: String },
}

/// Incrementally compiles a spec from a stream of JSONL patch lines.
#[derive(Debug, Clone)]
pub struct SpecStreamCompiler {
    pending: String,
    result: Value,
}

impl Default for SpecStreamCompiler {
    fn default() -> Self {
        Self {
            pending: String::new(),
            result: Value::Object(serde_json::Map::new()),
        }
    }
}

impl SpecStreamCompiler {
    pub fn new() -> Self {
        Self::default()
    }

    pub fn result(&self) -> &Value {
        &self.result
    }

    pub fn into_result(self) -> Value {
        self.result
    }

    /// Push a raw text chunk (typically from an LLM stream) and apply any complete patch lines.
    ///
    /// Returns the patches that were applied, in order.
    pub fn push_chunk(&mut self, chunk: &str) -> Result<Vec<JsonPatch>, SpecStreamError> {
        self.pending.push_str(chunk);

        let mut applied: Vec<JsonPatch> = Vec::new();

        loop {
            let Some(nl) = self.pending.find('\n') else {
                break;
            };
            let mut line = self.pending[..nl].to_string();
            self.pending.drain(..=nl);

            if line.ends_with('\r') {
                line.pop();
            }

            let trimmed = line.trim();
            if trimmed.is_empty() {
                continue;
            }

            // Tolerant parser: ignore non-JSON lines so callers can interleave prose.
            if !trimmed.starts_with('{') {
                continue;
            }

            let patch: JsonPatch = serde_json::from_str(trimmed)?;
            apply_patch(&mut self.result, &patch)?;
            applied.push(patch);
        }

        Ok(applied)
    }

    /// Convenience: parse and apply a full JSONL string at once.
    pub fn compile(mut self, jsonl: &str) -> Result<Value, SpecStreamError> {
        let _ = self.push_chunk(jsonl)?;
        // If the stream doesn't end with a newline, we intentionally do not try to parse
        // the final incomplete line.
        Ok(self.result)
    }
}

fn deep_equal(a: &Value, b: &Value) -> bool {
    a == b
}

fn unescape_segment(seg: &str) -> String {
    seg.replace("~1", "/").replace("~0", "~")
}

fn parse_pointer(pointer: &str) -> Result<Vec<String>, JsonPointerError> {
    if pointer.is_empty() || pointer == "/" {
        return Ok(Vec::new());
    }
    if !pointer.starts_with('/') {
        return Err(JsonPointerError::InvalidRoot);
    }
    Ok(pointer[1..].split('/').map(unescape_segment).collect())
}

fn ensure_container_for_segment(cur: &mut Value, seg: &str, next_is_index: bool) {
    if !cur.is_null() {
        return;
    }
    let seg_is_index = seg.parse::<usize>().is_ok() || seg == "-";
    *cur = if seg_is_index || next_is_index {
        Value::Array(Vec::new())
    } else {
        Value::Object(serde_json::Map::new())
    };
}

fn set_by_pointer(root: &mut Value, pointer: &str, value: Value) -> Result<(), JsonPointerError> {
    let segments = parse_pointer(pointer)?;
    if segments.is_empty() {
        *root = value;
        return Ok(());
    }

    let mut cur = root;
    for i in 0..segments.len() - 1 {
        let seg = &segments[i];
        let next = &segments[i + 1];
        let next_is_index = next.parse::<usize>().is_ok() || next == "-";

        ensure_container_for_segment(cur, seg, next_is_index);

        match cur {
            Value::Object(map) => {
                let entry = map.entry(seg).or_insert_with(|| {
                    if next_is_index {
                        Value::Array(Vec::new())
                    } else {
                        Value::Object(serde_json::Map::new())
                    }
                });
                cur = entry;
            }
            Value::Array(arr) => {
                let idx: usize = seg.parse().map_err(|_| JsonPointerError::InvalidIndex {
                    segment: seg.clone(),
                })?;
                if idx >= arr.len() {
                    arr.resize_with(idx + 1, || Value::Null);
                }
                if arr[idx].is_null() {
                    arr[idx] = if next_is_index {
                        Value::Array(Vec::new())
                    } else {
                        Value::Object(serde_json::Map::new())
                    };
                }
                cur = &mut arr[idx];
            }
            _ => return Err(JsonPointerError::NotContainer),
        }
    }

    let last = segments.last().expect("non-empty segments");
    ensure_container_for_segment(cur, last, false);
    match cur {
        Value::Object(map) => {
            map.insert(last.clone(), value);
            Ok(())
        }
        Value::Array(arr) => {
            if last == "-" {
                arr.push(value);
                return Ok(());
            }
            let idx: usize = last.parse().map_err(|_| JsonPointerError::InvalidIndex {
                segment: last.clone(),
            })?;
            if idx > arr.len() {
                // Streaming-tolerant: treat out-of-range as append.
                arr.push(value);
                return Ok(());
            }
            if idx == arr.len() {
                arr.push(value);
                return Ok(());
            }
            arr[idx] = value;
            Ok(())
        }
        _ => Err(JsonPointerError::NotContainer),
    }
}

fn add_by_pointer(root: &mut Value, pointer: &str, value: Value) -> Result<(), JsonPointerError> {
    let segments = parse_pointer(pointer)?;
    if segments.is_empty() {
        *root = value;
        return Ok(());
    }

    let mut cur = root;
    for i in 0..segments.len() - 1 {
        let seg = &segments[i];
        let next = &segments[i + 1];
        let next_is_index = next.parse::<usize>().is_ok() || next == "-";

        ensure_container_for_segment(cur, seg, next_is_index);

        match cur {
            Value::Object(map) => {
                let entry = map.entry(seg).or_insert_with(|| {
                    if next_is_index {
                        Value::Array(Vec::new())
                    } else {
                        Value::Object(serde_json::Map::new())
                    }
                });
                cur = entry;
            }
            Value::Array(arr) => {
                let idx: usize = seg.parse().map_err(|_| JsonPointerError::InvalidIndex {
                    segment: seg.clone(),
                })?;
                if idx >= arr.len() {
                    arr.resize_with(idx + 1, || Value::Null);
                }
                if arr[idx].is_null() {
                    arr[idx] = if next_is_index {
                        Value::Array(Vec::new())
                    } else {
                        Value::Object(serde_json::Map::new())
                    };
                }
                cur = &mut arr[idx];
            }
            _ => return Err(JsonPointerError::NotContainer),
        }
    }

    let last = segments.last().expect("non-empty segments");
    ensure_container_for_segment(cur, last, false);
    match cur {
        Value::Object(map) => {
            map.insert(last.clone(), value);
            Ok(())
        }
        Value::Array(arr) => {
            if last == "-" {
                arr.push(value);
                return Ok(());
            }
            let idx: usize = last.parse().map_err(|_| JsonPointerError::InvalidIndex {
                segment: last.clone(),
            })?;
            let idx = idx.min(arr.len());
            arr.insert(idx, value);
            Ok(())
        }
        _ => Err(JsonPointerError::NotContainer),
    }
}

fn remove_by_pointer(root: &mut Value, pointer: &str) -> Result<(), JsonPointerError> {
    let segments = parse_pointer(pointer)?;
    if segments.is_empty() {
        *root = Value::Object(serde_json::Map::new());
        return Ok(());
    }

    let mut cur = root;
    for i in 0..segments.len() - 1 {
        let seg = &segments[i];
        match cur {
            Value::Object(map) => match map.get_mut(seg) {
                Some(v) => cur = v,
                None => return Ok(()), // missing path: no-op
            },
            Value::Array(arr) => {
                let idx: usize = match seg.parse() {
                    Ok(v) => v,
                    Err(_) => {
                        return Err(JsonPointerError::InvalidIndex {
                            segment: seg.clone(),
                        });
                    }
                };
                let Some(v) = arr.get_mut(idx) else {
                    return Ok(());
                };
                cur = v;
            }
            _ => return Ok(()),
        }
    }

    let last = segments.last().expect("non-empty segments");
    match cur {
        Value::Object(map) => {
            let _ = map.remove(last);
            Ok(())
        }
        Value::Array(arr) => {
            let idx: usize = match last.parse() {
                Ok(v) => v,
                Err(_) => {
                    return Err(JsonPointerError::InvalidIndex {
                        segment: last.clone(),
                    });
                }
            };
            if idx < arr.len() {
                arr.remove(idx);
            }
            Ok(())
        }
        _ => Ok(()),
    }
}

pub fn apply_patch(root: &mut Value, patch: &JsonPatch) -> Result<(), SpecStreamError> {
    match patch.op {
        PatchOp::Add => {
            let value = patch.value.clone().ok_or(SpecStreamError::InvalidLine)?;
            add_by_pointer(root, &patch.path, value)?;
            Ok(())
        }
        PatchOp::Remove => {
            remove_by_pointer(root, &patch.path)?;
            Ok(())
        }
        PatchOp::Replace => {
            let value = patch.value.clone().ok_or(SpecStreamError::InvalidLine)?;
            set_by_pointer(root, &patch.path, value)?;
            Ok(())
        }
        PatchOp::Move => {
            let Some(from) = patch.from.as_deref() else {
                return Err(SpecStreamError::InvalidLine);
            };
            let Some(v) = get_opt(root, from).cloned() else {
                return Ok(());
            };
            remove_by_pointer(root, from)?;
            add_by_pointer(root, &patch.path, v)?;
            Ok(())
        }
        PatchOp::Copy => {
            let Some(from) = patch.from.as_deref() else {
                return Err(SpecStreamError::InvalidLine);
            };
            let Some(v) = get_opt(root, from).cloned() else {
                return Ok(());
            };
            add_by_pointer(root, &patch.path, v)?;
            Ok(())
        }
        PatchOp::Test => {
            let value = patch.value.as_ref().ok_or(SpecStreamError::InvalidLine)?;
            let cur = get_opt(root, &patch.path).unwrap_or(&Value::Null);
            if deep_equal(cur, value) {
                Ok(())
            } else {
                Err(SpecStreamError::TestFailed {
                    path: patch.path.clone(),
                })
            }
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use serde_json::json;

    #[test]
    fn compiler_ignores_non_json_lines_and_applies_patches() {
        let mut c = SpecStreamCompiler::new();
        let patches = c
            .push_chunk(
                "hello\n\
                 {\"op\":\"add\",\"path\":\"/a\",\"value\":1}\n\
                 {\"op\":\"replace\",\"path\":\"/a\",\"value\":2}\n",
            )
            .unwrap();
        assert_eq!(patches.len(), 2);
        assert_eq!(c.result(), &json!({ "a": 2 }));
    }

    #[test]
    fn add_inserts_into_arrays() {
        let mut v = json!({"a":[1,3]});
        apply_patch(
            &mut v,
            &JsonPatch {
                op: PatchOp::Add,
                path: "/a/1".to_string(),
                value: Some(json!(2)),
                from: None,
            },
        )
        .unwrap();
        assert_eq!(v, json!({"a":[1,2,3]}));
    }

    #[test]
    fn remove_deletes_from_objects_and_arrays() {
        let mut v = json!({"a":{"b":1},"x":[10,20,30]});
        apply_patch(
            &mut v,
            &JsonPatch {
                op: PatchOp::Remove,
                path: "/a/b".to_string(),
                value: None,
                from: None,
            },
        )
        .unwrap();
        apply_patch(
            &mut v,
            &JsonPatch {
                op: PatchOp::Remove,
                path: "/x/1".to_string(),
                value: None,
                from: None,
            },
        )
        .unwrap();
        assert_eq!(v, json!({"a":{},"x":[10,30]}));
    }

    #[test]
    fn move_copies_and_tests_work() {
        let mut v = json!({"a":{"b":1}});
        apply_patch(
            &mut v,
            &JsonPatch {
                op: PatchOp::Copy,
                path: "/a/c".to_string(),
                value: None,
                from: Some("/a/b".to_string()),
            },
        )
        .unwrap();
        apply_patch(
            &mut v,
            &JsonPatch {
                op: PatchOp::Move,
                path: "/a/d".to_string(),
                value: None,
                from: Some("/a/b".to_string()),
            },
        )
        .unwrap();
        apply_patch(
            &mut v,
            &JsonPatch {
                op: PatchOp::Test,
                path: "/a/c".to_string(),
                value: Some(json!(1)),
                from: None,
            },
        )
        .unwrap();
        assert_eq!(v, json!({"a":{"c":1,"d":1}}));
    }
}
