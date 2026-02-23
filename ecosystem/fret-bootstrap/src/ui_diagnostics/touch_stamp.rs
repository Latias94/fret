fn write_latest_pointer(out_dir: &Path, export_dir: &Path) -> Result<(), std::io::Error> {
    let path = out_dir.join("latest.txt");
    let Some(parent) = path.parent() else {
        return Ok(());
    };
    std::fs::create_dir_all(parent)?;
    let rel = export_dir.strip_prefix(out_dir).unwrap_or(export_dir);
    std::fs::write(path, rel.to_string_lossy().as_bytes())
}

fn touch_file(path: &Path) -> Result<(), std::io::Error> {
    if let Some(parent) = path.parent() {
        std::fs::create_dir_all(parent)?;
    }
    // `fret-diag`'s filesystem transport uses a monotonically increasing stamp written into
    // `*.touch` files. `SystemTime` millisecond resolution is not sufficient on all platforms
    // (multiple writes can occur within the same millisecond), so ensure the stamp is strictly
    // increasing within the current process.
    //
    // The stamp is used only for edge detection (not for wall-clock semantics), so it's safe to
    // synthesize values above `unix_ms_now()` when needed.
    use std::sync::atomic::{AtomicU64, Ordering};
    static LAST_TOUCH_STAMP: AtomicU64 = AtomicU64::new(0);
    let mut stamp = unix_ms_now();
    loop {
        let prev = LAST_TOUCH_STAMP.load(Ordering::Relaxed);
        let next = stamp.max(prev.saturating_add(1));
        match LAST_TOUCH_STAMP.compare_exchange_weak(
            prev,
            next,
            Ordering::Relaxed,
            Ordering::Relaxed,
        ) {
            Ok(_) => {
                stamp = next;
                break;
            }
            Err(_) => continue,
        }
    }
    use std::io::Write as _;
    let mut f = std::fs::OpenOptions::new()
        .create(true)
        .write(true)
        .truncate(true)
        .open(path)?;
    writeln!(f, "{stamp}")?;
    let _ = f.flush();
    Ok(())
}
