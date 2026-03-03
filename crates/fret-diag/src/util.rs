use std::path::Path;

pub(crate) fn read_file_bytes_shared(path: &Path) -> Option<Vec<u8>> {
    #[cfg(windows)]
    {
        use std::io::Read as _;
        use std::os::windows::fs::OpenOptionsExt as _;

        // Allow the UI process to update files while tooling polls them (Windows sharing semantics).
        const FILE_SHARE_READ: u32 = 0x0000_0001;
        const FILE_SHARE_WRITE: u32 = 0x0000_0002;
        const FILE_SHARE_DELETE: u32 = 0x0000_0004;

        let mut file = std::fs::OpenOptions::new()
            .read(true)
            .share_mode(FILE_SHARE_READ | FILE_SHARE_WRITE | FILE_SHARE_DELETE)
            .open(path)
            .ok()?;
        let mut bytes = Vec::new();
        file.read_to_end(&mut bytes).ok()?;
        Some(bytes)
    }

    #[cfg(not(windows))]
    {
        std::fs::read(path).ok()
    }
}

pub(crate) fn now_unix_ms() -> u64 {
    std::time::SystemTime::now()
        .duration_since(std::time::SystemTime::UNIX_EPOCH)
        .map(|d| d.as_millis() as u64)
        .unwrap_or(0)
}

pub(crate) fn read_json_value(path: &Path) -> Option<serde_json::Value> {
    let bytes = read_file_bytes_shared(path)?;
    serde_json::from_slice(&bytes).ok()
}

pub(crate) fn write_json_value(path: &Path, v: &serde_json::Value) -> Result<(), String> {
    if let Some(parent) = path.parent() {
        std::fs::create_dir_all(parent).map_err(|e| e.to_string())?;
    }
    let bytes = serde_json::to_vec_pretty(v).map_err(|e| e.to_string())?;

    // Write atomically (best-effort) to avoid readers observing partial JSON while the file is
    // being updated. This matters for the filesystem diagnostics transport, where the app reads
    // `script.json` concurrently with tooling that updates it.
    write_bytes_atomic(path, &bytes)
}

fn write_bytes_atomic(path: &Path, bytes: &[u8]) -> Result<(), String> {
    use std::io::Write as _;

    let Some(parent) = path.parent() else {
        return Err(format!("invalid path (missing parent): {}", path.display()));
    };
    let Some(file_name) = path.file_name().and_then(|v| v.to_str()) else {
        return Err(format!(
            "invalid path (missing UTF-8 file name): {}",
            path.display()
        ));
    };

    let pid = std::process::id();
    let mut attempt: u32 = 0;
    let tmp_path = loop {
        let candidate = parent.join(format!(".{file_name}.tmp.{pid}.{attempt}"));
        match std::fs::OpenOptions::new()
            .create_new(true)
            .write(true)
            .open(&candidate)
        {
            Ok(mut f) => {
                f.write_all(bytes).map_err(|e| e.to_string())?;
                let _ = f.flush();
                let _ = f.sync_all();
                drop(f);
                break candidate;
            }
            Err(err) if err.kind() == std::io::ErrorKind::AlreadyExists => {
                attempt = attempt.saturating_add(1);
                continue;
            }
            Err(err) => return Err(err.to_string()),
        }
    };

    #[cfg(windows)]
    {
        // `rename` fails if the destination exists on Windows. Prefer removing the destination
        // before renaming so readers never observe a partially-written file.
        if std::fs::rename(&tmp_path, path).is_err() {
            let _ = std::fs::remove_file(path);
            std::fs::rename(&tmp_path, path).map_err(|e| e.to_string())?;
        }
        Ok(())
    }

    #[cfg(not(windows))]
    {
        std::fs::rename(&tmp_path, path).map_err(|e| e.to_string())?;
        Ok(())
    }
}

pub(crate) fn touch(path: &Path) -> Result<(), String> {
    if let Some(parent) = path.parent() {
        std::fs::create_dir_all(parent).map_err(|e| e.to_string())?;
    }
    let mut f = std::fs::OpenOptions::new()
        .create(true)
        .truncate(true)
        .write(true)
        .open(path)
        .map_err(|e| e.to_string())?;
    use std::io::Write as _;
    let ts = std::time::SystemTime::now()
        .duration_since(std::time::SystemTime::UNIX_EPOCH)
        .map(|d| d.as_millis())
        .unwrap_or_default();
    writeln!(f, "{ts}").map_err(|e| e.to_string())?;
    f.flush().map_err(|e| e.to_string())
}

pub(crate) fn sanitize_for_filename(raw: &str, max_len: usize, fallback: &str) -> String {
    let mut out = String::with_capacity(raw.len().min(max_len.max(1)));
    for ch in raw.chars().take(max_len.max(1)) {
        if ch.is_ascii_alphanumeric() || ch == '-' || ch == '_' || ch == '.' {
            out.push(ch);
        } else {
            out.push('_');
        }
    }
    if out.trim().is_empty() {
        fallback.to_string()
    } else {
        out
    }
}

pub(crate) fn read_script_result(path: &Path) -> Option<serde_json::Value> {
    read_json_value(path)
}

pub(crate) fn read_script_result_run_id(path: &Path) -> Option<u64> {
    read_script_result(path)?.get("run_id")?.as_u64()
}

pub(crate) fn read_pick_result(path: &Path) -> Option<serde_json::Value> {
    read_json_value(path)
}

pub(crate) fn read_pick_result_run_id(path: &Path) -> Option<u64> {
    read_pick_result(path)?.get("run_id")?.as_u64()
}

pub(crate) fn json_pointer_set(
    root: &mut serde_json::Value,
    pointer: &str,
    value: serde_json::Value,
) -> Result<(), String> {
    if pointer.is_empty() {
        *root = value;
        return Ok(());
    }
    if !pointer.starts_with('/') {
        return Err(format!(
            "invalid JSON pointer (must start with '/'): {pointer}"
        ));
    }

    let mut tokens: Vec<String> = pointer[1..]
        .split('/')
        .map(unescape_json_pointer_token)
        .collect();
    if tokens.is_empty() {
        *root = value;
        return Ok(());
    }

    let last = tokens
        .pop()
        .ok_or_else(|| "invalid JSON pointer".to_string())?;

    let mut cur: &mut serde_json::Value = root;
    for t in tokens {
        match cur {
            serde_json::Value::Object(map) => {
                let Some(next) = map.get_mut(&t) else {
                    return Err(format!("JSON pointer path does not exist: {pointer}"));
                };
                cur = next;
            }
            serde_json::Value::Array(arr) => {
                let idx = t
                    .parse::<usize>()
                    .map_err(|_| format!("JSON pointer expected array index, got: {t}"))?;
                let Some(next) = arr.get_mut(idx) else {
                    return Err(format!("JSON pointer array index out of bounds: {pointer}"));
                };
                cur = next;
            }
            _ => {
                return Err(format!(
                    "JSON pointer path does not resolve to a container: {pointer}"
                ));
            }
        }
    }

    match cur {
        serde_json::Value::Object(map) => {
            map.insert(last, value);
            Ok(())
        }
        serde_json::Value::Array(arr) => {
            if last == "-" {
                arr.push(value);
                return Ok(());
            }

            let idx = last
                .parse::<usize>()
                .map_err(|_| format!("JSON pointer expected array index, got: {last}"))?;
            if idx < arr.len() {
                arr[idx] = value;
                return Ok(());
            }
            if idx == arr.len() {
                arr.push(value);
                return Ok(());
            }
            Err(format!("JSON pointer array index out of bounds: {pointer}"))
        }
        _ => Err(format!(
            "JSON pointer path does not resolve to a container: {pointer}"
        )),
    }
}

fn unescape_json_pointer_token(raw: &str) -> String {
    let mut out = String::with_capacity(raw.len());
    let mut it = raw.chars();
    while let Some(c) = it.next() {
        if c == '~' {
            match it.next() {
                Some('0') => out.push('~'),
                Some('1') => out.push('/'),
                Some(other) => {
                    out.push('~');
                    out.push(other);
                }
                None => out.push('~'),
            }
        } else {
            out.push(c);
        }
    }
    out
}
