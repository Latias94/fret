use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize, Default)]
#[serde(rename_all = "snake_case")]
pub enum ExecBackgroundWork {
    #[default]
    Threads,
    Cooperative,
    None,
}

impl ExecBackgroundWork {
    pub fn as_str(self) -> &'static str {
        match self {
            Self::None => "none",
            Self::Cooperative => "cooperative",
            Self::Threads => "threads",
        }
    }

    pub fn clamp_to_available(self, available: Self) -> Self {
        use ExecBackgroundWork::*;
        match (self, available) {
            (None, _) => None,
            (_, None) => None,
            (Cooperative, Cooperative | Threads) => Cooperative,
            (Threads, Threads) => Threads,
            (Threads, Cooperative) => Cooperative,
        }
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize, Default)]
#[serde(rename_all = "snake_case")]
pub enum ExecWake {
    #[default]
    Reliable,
    BestEffort,
    None,
}

impl ExecWake {
    pub fn as_str(self) -> &'static str {
        match self {
            Self::None => "none",
            Self::BestEffort => "best_effort",
            Self::Reliable => "reliable",
        }
    }

    pub fn clamp_to_available(self, available: Self) -> Self {
        use ExecWake::*;
        match (self, available) {
            (None, _) => None,
            (_, None) => None,
            (BestEffort, BestEffort | Reliable) => BestEffort,
            (Reliable, Reliable) => Reliable,
            (Reliable, BestEffort) => BestEffort,
        }
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize, Default)]
#[serde(rename_all = "snake_case")]
pub enum ExecTimers {
    #[default]
    Reliable,
    BestEffort,
    None,
}

impl ExecTimers {
    pub fn as_str(self) -> &'static str {
        match self {
            Self::None => "none",
            Self::BestEffort => "best_effort",
            Self::Reliable => "reliable",
        }
    }

    pub fn clamp_to_available(self, available: Self) -> Self {
        use ExecTimers::*;
        match (self, available) {
            (None, _) => None,
            (_, None) => None,
            (BestEffort, BestEffort | Reliable) => BestEffort,
            (Reliable, Reliable) => Reliable,
            (Reliable, BestEffort) => BestEffort,
        }
    }
}

#[derive(Debug, Clone, Copy, Default, PartialEq, Eq, Serialize, Deserialize)]
#[serde(default)]
pub struct ExecCapabilities {
    pub background_work: ExecBackgroundWork,
    pub wake: ExecWake,
    pub timers: ExecTimers,
}
