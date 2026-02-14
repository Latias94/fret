//! JSON Pointer helpers (RFC 6901-ish) for `serde_json::Value`.
//!
//! Notes:
//! - We treat `""` and `"/"` as the root pointer.
//! - We support `~0` and `~1` unescaping.

use serde_json::Value;

#[derive(Debug, thiserror::Error, Clone, PartialEq, Eq)]
pub enum JsonPointerError {
    #[error("json pointer must be empty or start with '/'")]
    InvalidRoot,
    #[error("json pointer segment expected array index: {segment}")]
    InvalidIndex { segment: String },
    #[error("json pointer path does not exist")]
    NotFound,
    #[error("json pointer cannot traverse non-container value")]
    NotContainer,
}

fn unescape_segment(seg: &str) -> String {
    // RFC 6901: "~1" -> "/", "~0" -> "~" (order matters).
    seg.replace("~1", "/").replace("~0", "~")
}

fn parse(pointer: &str) -> Result<Vec<String>, JsonPointerError> {
    if pointer.is_empty() || pointer == "/" {
        return Ok(Vec::new());
    }
    if !pointer.starts_with('/') {
        return Err(JsonPointerError::InvalidRoot);
    }
    Ok(pointer[1..].split('/').map(unescape_segment).collect())
}

pub fn get<'a>(root: &'a Value, pointer: &str) -> Result<&'a Value, JsonPointerError> {
    let segments = parse(pointer)?;
    let mut cur = root;
    for seg in segments {
        match cur {
            Value::Object(map) => {
                cur = map.get(&seg).ok_or(JsonPointerError::NotFound)?;
            }
            Value::Array(arr) => {
                let idx: usize = seg
                    .parse()
                    .map_err(|_| JsonPointerError::InvalidIndex { segment: seg })?;
                cur = arr.get(idx).ok_or(JsonPointerError::NotFound)?;
            }
            _ => return Err(JsonPointerError::NotContainer),
        }
    }
    Ok(cur)
}

pub fn get_opt<'a>(root: &'a Value, pointer: &str) -> Option<&'a Value> {
    get(root, pointer).ok()
}

pub fn set(root: &mut Value, pointer: &str, value: Value) -> Result<(), JsonPointerError> {
    let segments = parse(pointer)?;
    if segments.is_empty() {
        *root = value;
        return Ok(());
    }

    let mut cur = root;
    for i in 0..segments.len() - 1 {
        let seg = &segments[i];
        let next = &segments[i + 1];
        let next_is_index = next.parse::<usize>().is_ok() || next == "-";

        if cur.is_null() {
            let seg_is_index = seg.parse::<usize>().is_ok() || seg == "-";
            *cur = if seg_is_index {
                Value::Array(Vec::new())
            } else {
                Value::Object(serde_json::Map::new())
            };
        }

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
    if cur.is_null() {
        let last_is_index = last.parse::<usize>().is_ok() || last == "-";
        *cur = if last_is_index {
            Value::Array(Vec::new())
        } else {
            Value::Object(serde_json::Map::new())
        };
    }
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
                return Err(JsonPointerError::NotFound);
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

#[cfg(test)]
mod tests {
    use super::*;
    use serde_json::json;

    #[test]
    fn get_root() {
        let v = json!({"a": 1});
        assert_eq!(get(&v, "").unwrap(), &v);
        assert_eq!(get(&v, "/").unwrap(), &v);
    }

    #[test]
    fn get_nested_object() {
        let v = json!({"a": {"b": 2}});
        assert_eq!(get(&v, "/a/b").unwrap(), &json!(2));
    }

    #[test]
    fn set_creates_containers() {
        let mut v = Value::Null;
        set(&mut v, "/a/0/b", json!(3)).unwrap();
        assert_eq!(get(&v, "/a/0/b").unwrap(), &json!(3));
    }

    #[test]
    fn set_array_append_dash() {
        let mut v = json!({"a": []});
        set(&mut v, "/a/-", json!(1)).unwrap();
        set(&mut v, "/a/-", json!(2)).unwrap();
        assert_eq!(get(&v, "/a/0").unwrap(), &json!(1));
        assert_eq!(get(&v, "/a/1").unwrap(), &json!(2));
    }
}
