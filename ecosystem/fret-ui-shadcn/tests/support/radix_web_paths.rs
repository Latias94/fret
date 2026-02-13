use std::path::PathBuf;

#[path = "repo_root.rs"]
mod repo_root;

use repo_root::repo_root;

pub(crate) fn radix_web_dir() -> PathBuf {
    repo_root()
        .join("goldens")
        .join("radix-web")
        .join("v4")
        .join("radix-vega")
}

#[allow(dead_code)]
pub(crate) fn radix_web_path(file_stem: &str) -> PathBuf {
    radix_web_dir().join(format!("{file_stem}.json"))
}
