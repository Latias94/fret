use fret_core::{Color, Px};
use fret_ui::{Theme, ThemeSnapshot};

/// Minimal theme token access surface used by style resolution helpers.
///
/// This is intentionally narrower than `Theme` so style code can accept a `ThemeSnapshot` for
/// borrow-checker-friendly, cheap token reads.
pub trait ThemeTokenRead {
    fn color_by_key(&self, key: &str) -> Option<Color>;
    fn color_required(&self, key: &str) -> Color;

    fn metric_by_key(&self, key: &str) -> Option<Px>;
    fn metric_required(&self, key: &str) -> Px;
}

impl ThemeTokenRead for Theme {
    fn color_by_key(&self, key: &str) -> Option<Color> {
        self.color_by_key(key)
    }

    fn color_required(&self, key: &str) -> Color {
        self.color_required(key)
    }

    fn metric_by_key(&self, key: &str) -> Option<Px> {
        self.metric_by_key(key)
    }

    fn metric_required(&self, key: &str) -> Px {
        self.metric_required(key)
    }
}

impl ThemeTokenRead for ThemeSnapshot {
    fn color_by_key(&self, key: &str) -> Option<Color> {
        self.color_by_key(key)
    }

    fn color_required(&self, key: &str) -> Color {
        self.color_required(key)
    }

    fn metric_by_key(&self, key: &str) -> Option<Px> {
        self.metric_by_key(key)
    }

    fn metric_required(&self, key: &str) -> Px {
        self.metric_required(key)
    }
}

impl<T: ThemeTokenRead + ?Sized> ThemeTokenRead for &T {
    fn color_by_key(&self, key: &str) -> Option<Color> {
        (*self).color_by_key(key)
    }

    fn color_required(&self, key: &str) -> Color {
        (*self).color_required(key)
    }

    fn metric_by_key(&self, key: &str) -> Option<Px> {
        (*self).metric_by_key(key)
    }

    fn metric_required(&self, key: &str) -> Px {
        (*self).metric_required(key)
    }
}
