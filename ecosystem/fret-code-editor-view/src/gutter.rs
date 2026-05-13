use std::sync::Arc;

use fret_code_editor_buffer::TextBuffer;

use crate::DisplayMap;

#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Hash)]
pub enum GutterMarkerAnchor {
    LogicalLine(usize),
    DisplayRow(usize),
}

#[derive(Debug, Clone, PartialEq, Eq, PartialOrd, Ord, Hash)]
pub enum GutterMarkerKind {
    Diagnostic,
    Breakpoint,
    Bookmark,
    Runnable,
    Diff,
    Custom(Arc<str>),
}

#[derive(Debug, Clone, PartialEq, Eq, PartialOrd, Ord, Hash, Default)]
pub enum GutterMarkerVisual {
    #[default]
    None,
    Icon(Arc<str>),
    Text(Arc<str>),
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Hash, Default)]
pub enum GutterMarkerHitTarget {
    #[default]
    None,
    Marker,
    Line,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct GutterMarker {
    pub anchor: GutterMarkerAnchor,
    pub kind: GutterMarkerKind,
    pub visual: GutterMarkerVisual,
    pub tooltip: Option<Arc<str>>,
    pub action_id: Option<Arc<str>>,
    pub priority: i16,
    pub hit_target: GutterMarkerHitTarget,
}

impl GutterMarker {
    pub fn new(anchor: GutterMarkerAnchor, kind: GutterMarkerKind) -> Self {
        Self {
            anchor,
            kind,
            visual: GutterMarkerVisual::None,
            tooltip: None,
            action_id: None,
            priority: 0,
            hit_target: GutterMarkerHitTarget::None,
        }
    }

    pub fn logical_line(line: usize, kind: GutterMarkerKind) -> Self {
        Self::new(GutterMarkerAnchor::LogicalLine(line), kind)
    }

    pub fn display_row(row: usize, kind: GutterMarkerKind) -> Self {
        Self::new(GutterMarkerAnchor::DisplayRow(row), kind)
    }
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub enum GutterMarkerError {
    LogicalLineOutOfBounds { line: usize, line_count: usize },
    DisplayRowRequiresDisplayMap { row: usize },
    DisplayRowOutOfBounds { row: usize, row_count: usize },
}

pub fn validate_gutter_markers(
    buf: &TextBuffer,
    display_map: Option<&DisplayMap>,
    markers: &[GutterMarker],
) -> Result<(), GutterMarkerError> {
    let line_count = buf.line_count().max(1);
    let row_count = display_map.map(DisplayMap::row_count);

    for marker in markers {
        match marker.anchor {
            GutterMarkerAnchor::LogicalLine(line) => {
                if line >= line_count {
                    return Err(GutterMarkerError::LogicalLineOutOfBounds { line, line_count });
                }
            }
            GutterMarkerAnchor::DisplayRow(row) => {
                let Some(row_count) = row_count else {
                    return Err(GutterMarkerError::DisplayRowRequiresDisplayMap { row });
                };
                if row >= row_count {
                    return Err(GutterMarkerError::DisplayRowOutOfBounds { row, row_count });
                }
            }
        }
    }

    Ok(())
}

pub fn normalized_gutter_markers(markers: &[GutterMarker]) -> Vec<GutterMarker> {
    let mut out = markers.to_vec();
    out.sort_by(|a, b| {
        a.anchor
            .cmp(&b.anchor)
            .then(b.priority.cmp(&a.priority))
            .then(a.kind.cmp(&b.kind))
            .then(a.visual.cmp(&b.visual))
            .then_with(|| a.action_id.as_deref().cmp(&b.action_id.as_deref()))
            .then_with(|| a.tooltip.as_deref().cmp(&b.tooltip.as_deref()))
            .then(a.hit_target.cmp(&b.hit_target))
    });
    out
}

#[cfg(test)]
mod tests {
    use super::*;
    use fret_code_editor_buffer::DocId;

    fn buffer(text: &str) -> TextBuffer {
        TextBuffer::new(DocId::new(), text.to_string()).unwrap()
    }

    #[test]
    fn gutter_markers_validate_logical_line_bounds() {
        let buf = buffer("one\ntwo");
        let markers = vec![GutterMarker::logical_line(1, GutterMarkerKind::Diagnostic)];

        assert_eq!(validate_gutter_markers(&buf, None, &markers), Ok(()));
        assert_eq!(
            validate_gutter_markers(
                &buf,
                None,
                &[GutterMarker::logical_line(2, GutterMarkerKind::Diagnostic)]
            ),
            Err(GutterMarkerError::LogicalLineOutOfBounds {
                line: 2,
                line_count: 2
            })
        );
    }

    #[test]
    fn gutter_markers_validate_display_row_bounds_with_display_map() {
        let buf = buffer("abcdef");
        let map = DisplayMap::new(&buf, Some(2));
        let markers = vec![GutterMarker::display_row(2, GutterMarkerKind::Diagnostic)];

        assert_eq!(validate_gutter_markers(&buf, Some(&map), &markers), Ok(()));
        assert_eq!(
            validate_gutter_markers(
                &buf,
                Some(&map),
                &[GutterMarker::display_row(3, GutterMarkerKind::Diagnostic)]
            ),
            Err(GutterMarkerError::DisplayRowOutOfBounds {
                row: 3,
                row_count: 3
            })
        );
    }

    #[test]
    fn display_row_gutter_markers_require_display_map_for_validation() {
        let buf = buffer("abcdef");

        assert_eq!(
            validate_gutter_markers(
                &buf,
                None,
                &[GutterMarker::display_row(0, GutterMarkerKind::Diagnostic)]
            ),
            Err(GutterMarkerError::DisplayRowRequiresDisplayMap { row: 0 })
        );
    }

    #[test]
    fn normalized_gutter_markers_sort_by_anchor_priority_and_payload() {
        let mut low = GutterMarker::logical_line(1, GutterMarkerKind::Bookmark);
        low.priority = 0;
        low.visual = GutterMarkerVisual::Icon("bookmark".into());

        let mut high = GutterMarker::logical_line(1, GutterMarkerKind::Diagnostic);
        high.priority = 10;
        high.visual = GutterMarkerVisual::Icon("error".into());
        high.tooltip = Some("error".into());

        let row = GutterMarker::display_row(0, GutterMarkerKind::Runnable);

        let normalized = normalized_gutter_markers(&[low.clone(), row.clone(), high.clone()]);

        assert_eq!(normalized, vec![high, low, row]);
    }

    #[test]
    fn gutter_marker_payload_keeps_action_as_data() {
        let mut marker = GutterMarker::logical_line(0, GutterMarkerKind::Custom("test".into()));
        marker.visual = GutterMarkerVisual::Text("Run".into());
        marker.tooltip = Some("Run test".into());
        marker.action_id = Some("editor.run_test_at_line".into());
        marker.hit_target = GutterMarkerHitTarget::Marker;

        assert_eq!(marker.action_id.as_deref(), Some("editor.run_test_at_line"));
        assert_eq!(marker.hit_target, GutterMarkerHitTarget::Marker);
    }
}
