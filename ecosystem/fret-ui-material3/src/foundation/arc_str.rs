use std::sync::Arc;
use std::sync::OnceLock;

pub fn empty_arc_str() -> Arc<str> {
    static EMPTY: OnceLock<Arc<str>> = OnceLock::new();
    EMPTY.get_or_init(|| Arc::<str>::from("")).clone()
}

