use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum WindowResizeDirection {
    N,
    Ne,
    E,
    Se,
    S,
    Sw,
    W,
    Nw,
}
