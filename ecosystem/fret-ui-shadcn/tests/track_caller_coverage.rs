use std::fs;
use std::path::{Path, PathBuf};

fn collect_rs_files(root: &Path) -> Vec<PathBuf> {
    let mut out = Vec::new();
    let mut stack = vec![root.to_path_buf()];

    while let Some(dir) = stack.pop() {
        let entries = match fs::read_dir(&dir) {
            Ok(v) => v,
            Err(_) => continue,
        };
        for entry in entries.flatten() {
            let path = entry.path();
            if path.is_dir() {
                stack.push(path);
                continue;
            }
            if path.extension().is_some_and(|e| e == "rs") {
                out.push(path);
            }
        }
    }

    out.sort();
    out
}

fn has_track_caller_header(lines: &[&str], mut i: isize) -> bool {
    while i >= 0 {
        let line = lines[i as usize];
        let trimmed = line.trim_start();
        if trimmed.is_empty() {
            break;
        }
        if trimmed.starts_with("///") || trimmed.starts_with("#[") {
            if trimmed.replace(' ', "") == "#[track_caller]" {
                return true;
            }
            i -= 1;
            continue;
        }
        break;
    }
    false
}

#[test]
fn all_public_into_element_fns_are_track_caller() {
    // Guardrail: If a public ecosystem component entrypoint calls `cx.scope(...)` internally, it
    // should be `#[track_caller]` so element identity is anchored at the *application callsite*
    // rather than the component's source line. This avoids "state sticks to the wrong sibling"
    // bugs when siblings are inserted/removed.
    let src = Path::new(env!("CARGO_MANIFEST_DIR")).join("src");
    let files = collect_rs_files(&src);

    let mut missing = Vec::new();
    for file in files {
        let text = fs::read_to_string(&file).expect("read source file");
        let lines: Vec<&str> = text.lines().collect();

        for (idx, line) in lines.iter().enumerate() {
            let trimmed = line.trim_start();
            if !trimmed.starts_with("pub ") {
                continue;
            }
            // Intentionally narrow: we only gate public `into_element` entrypoints.
            // Variants like `pub(crate)` are allowed to evolve without this guardrail.
            if trimmed.starts_with("pub(crate)") {
                continue;
            }
            if !trimmed.contains("fn into_element") {
                continue;
            }

            if !has_track_caller_header(&lines, idx as isize - 1) {
                let rel = file.strip_prefix(&src).unwrap_or(&file);
                missing.push(format!("{}:{}", rel.display(), idx + 1));
            }
        }
    }

    if !missing.is_empty() {
        panic!(
            "Missing #[track_caller] for public into_element entrypoints:\n{}",
            missing.join("\n")
        );
    }
}
