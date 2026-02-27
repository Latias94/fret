use crate::geometry::{TextLineCluster, TextLineDecorationGeometry, TextLineGeometry};
use fret_core::geometry::Px;
use std::sync::Arc;

#[derive(Debug, Clone)]
pub struct TextLineLayout {
    pub start: usize,
    pub end: usize,
    pub width: Px,
    pub y_top: Px,
    /// Baseline Y for this line (y=0 at the top of the text box).
    pub y_baseline: Px,
    pub height: Px,
    pub ascent: Px,
    pub descent: Px,
    pub ink_ascent: Px,
    pub ink_descent: Px,
    pub caret_stops: Vec<(usize, Px)>,
    clusters: Arc<[TextLineCluster]>,
}

impl TextLineLayout {
    #[allow(clippy::too_many_arguments)]
    pub fn new(
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

    pub fn clusters(&self) -> &[TextLineCluster] {
        self.clusters.as_ref()
    }
}

impl TextLineGeometry for TextLineLayout {
    fn start(&self) -> usize {
        self.start
    }

    fn end(&self) -> usize {
        self.end
    }

    fn y_top(&self) -> Px {
        self.y_top
    }

    fn height(&self) -> Px {
        self.height
    }

    fn caret_stops(&self) -> &[(usize, Px)] {
        self.caret_stops.as_ref()
    }

    fn clusters(&self) -> &[TextLineCluster] {
        self.clusters.as_ref()
    }
}

impl TextLineDecorationGeometry for TextLineLayout {
    fn y_baseline(&self) -> Px {
        self.y_baseline
    }
}
