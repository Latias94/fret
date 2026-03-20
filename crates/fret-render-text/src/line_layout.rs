use crate::geometry::{TextLineCluster, TextLineDecorationGeometry, TextLineGeometry};
use fret_core::geometry::Px;
use std::sync::Arc;

#[derive(Debug, Clone)]
pub struct TextLineLayout {
    start: usize,
    end: usize,
    width: Px,
    y_top: Px,
    /// Baseline Y for this line (y=0 at the top of the text box).
    y_baseline: Px,
    height: Px,
    ascent: Px,
    descent: Px,
    ink_ascent: Px,
    ink_descent: Px,
    caret_stops: Vec<(usize, Px)>,
    clusters: Arc<[TextLineCluster]>,
}

impl TextLineLayout {
    #[allow(clippy::too_many_arguments)]
    pub(crate) fn new(
        start: usize,
        end: usize,
        width: Px,
        y_top: Px,
        y_baseline: Px,
        height: Px,
        ascent: Px,
        descent: Px,
        ink_ascent: Px,
        ink_descent: Px,
        caret_stops: Vec<(usize, Px)>,
        clusters: Arc<[TextLineCluster]>,
    ) -> Self {
        Self {
            start,
            end,
            width,
            y_top,
            y_baseline,
            height,
            ascent,
            descent,
            ink_ascent,
            ink_descent,
            caret_stops,
            clusters,
        }
    }

    pub fn start(&self) -> usize {
        self.start
    }

    pub fn end(&self) -> usize {
        self.end
    }

    pub fn width(&self) -> Px {
        self.width
    }

    pub fn y_top(&self) -> Px {
        self.y_top
    }

    pub fn y_baseline(&self) -> Px {
        self.y_baseline
    }

    pub fn height(&self) -> Px {
        self.height
    }

    pub fn ascent(&self) -> Px {
        self.ascent
    }

    pub fn descent(&self) -> Px {
        self.descent
    }

    pub fn ink_ascent(&self) -> Px {
        self.ink_ascent
    }

    pub fn ink_descent(&self) -> Px {
        self.ink_descent
    }

    pub fn caret_stops(&self) -> &[(usize, Px)] {
        self.caret_stops.as_ref()
    }

    pub fn caret_stops_capacity(&self) -> usize {
        self.caret_stops.capacity()
    }

    pub fn clusters(&self) -> &[TextLineCluster] {
        self.clusters.as_ref()
    }
}

impl TextLineGeometry for TextLineLayout {
    fn start(&self) -> usize {
        self.start()
    }

    fn end(&self) -> usize {
        self.end()
    }

    fn y_top(&self) -> Px {
        self.y_top()
    }

    fn height(&self) -> Px {
        self.height()
    }

    fn caret_stops(&self) -> &[(usize, Px)] {
        self.caret_stops()
    }

    fn clusters(&self) -> &[TextLineCluster] {
        self.clusters()
    }
}

impl TextLineDecorationGeometry for TextLineLayout {
    fn y_baseline(&self) -> Px {
        self.y_baseline()
    }
}
