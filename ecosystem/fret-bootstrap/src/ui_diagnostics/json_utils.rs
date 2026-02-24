fn key_to_u64(key: NodeId) -> u64 {
    key.data().as_ffi()
}

fn write_json<T: Serialize>(path: PathBuf, value: &T) -> Result<(), std::io::Error> {
    let Some(parent) = path.parent() else {
        return Ok(());
    };
    std::fs::create_dir_all(parent)?;
    let bytes = serde_json::to_vec_pretty(value).unwrap_or_default();
    std::fs::write(path, bytes)
}

fn write_json_compact<T: Serialize>(path: PathBuf, value: &T) -> Result<(), std::io::Error> {
    let Some(parent) = path.parent() else {
        return Ok(());
    };
    std::fs::create_dir_all(parent)?;
    let bytes = serde_json::to_vec(value).unwrap_or_default();
    std::fs::write(path, bytes)
}

fn take_last_vecdeque<T: Clone>(items: &VecDeque<T>, max: usize) -> Vec<T> {
    if max == 0 {
        return Vec::new();
    }
    let len = items.len();
    let start = len.saturating_sub(max);
    items.iter().skip(start).cloned().collect()
}
