use delinea::VisualMapId;
use delinea::engine::model::VisualMapModel;
use delinea::engine::window::DataWindow;
use fret_core::{Point, Px, Rect, Size};

#[derive(Debug, Clone, Copy, PartialEq)]
pub(crate) struct VisualMapTrackLayout {
    pub id: VisualMapId,
    pub model: VisualMapModel,
    pub track: Rect,
}

pub(crate) fn visual_map_track_layouts(
    band: Option<Rect>,
    maps: &[(VisualMapId, VisualMapModel)],
    item_gap: Px,
    padding: Px,
) -> Vec<VisualMapTrackLayout> {
    let Some(band) = band else {
        return Vec::new();
    };
    if band.size.width.0 <= 0.0 || band.size.height.0 <= 0.0 || maps.is_empty() {
        return Vec::new();
    }

    let gap = item_gap.0.max(0.0);
    let pad = padding.0.max(0.0);
    let total_gap = gap * (maps.len().saturating_sub(1) as f32);
    let item_h = ((band.size.height.0 - total_gap) / (maps.len() as f32)).max(1.0);

    let mut y = band.origin.y.0;
    let mut out = Vec::with_capacity(maps.len());
    for &(id, model) in maps {
        let item = Rect::new(
            Point::new(band.origin.x, Px(y)),
            Size::new(band.size.width, Px(item_h)),
        );
        y += item_h + gap;

        let track = Rect::new(
            Point::new(Px(item.origin.x.0 + pad), Px(item.origin.y.0 + pad)),
            Size::new(
                Px((item.size.width.0 - 2.0 * pad).max(1.0)),
                Px((item.size.height.0 - 2.0 * pad).max(1.0)),
            ),
        );
        if track.size.width.0 > 0.0 && track.size.height.0 > 0.0 {
            out.push(VisualMapTrackLayout { id, model, track });
        }
    }
    out
}

pub(crate) fn visual_map_track_at(
    tracks: &[VisualMapTrackLayout],
    position: Point,
) -> Option<VisualMapTrackLayout> {
    tracks
        .iter()
        .copied()
        .find(|layout| layout.track.contains(position))
}

pub(crate) fn visual_map_domain_window(vm: VisualMapModel) -> DataWindow {
    DataWindow {
        min: vm.domain.min,
        max: vm.domain.max,
    }
}

pub(crate) fn visual_map_y_at_value(track: Rect, domain: DataWindow, value: f64) -> f32 {
    let mut domain = domain;
    domain.clamp_non_degenerate();
    let span = domain.span();
    if !span.is_finite() || span <= 0.0 {
        return track.origin.y.0 + track.size.height.0;
    }
    let t = ((value - domain.min) / span).clamp(0.0, 1.0) as f32;
    track.origin.y.0 + (1.0 - t) * track.size.height.0
}

#[cfg(test)]
mod tests {
    use super::*;

    fn visual_map_model(id: u64, domain: (f64, f64)) -> VisualMapModel {
        VisualMapModel {
            id: VisualMapId::new(id),
            mode: delinea::VisualMapMode::Continuous,
            field: delinea::FieldId::new(1),
            domain: delinea::engine::model::VisualMapDomain {
                min: domain.0,
                max: domain.1,
            },
            initial_range: None,
            initial_piece_mask: None,
            point_radius_mul_range: None,
            stroke_width_range: None,
            opacity_mul_range: None,
            buckets: 8,
            out_of_range_opacity: 0.25,
        }
    }

    #[test]
    fn visual_map_y_mapping_respects_domain_endpoints() {
        let track = Rect::new(
            Point::new(Px(10.0), Px(20.0)),
            Size::new(Px(8.0), Px(100.0)),
        );
        let domain = DataWindow {
            min: 0.0,
            max: 10.0,
        };

        let bottom = track.origin.y.0 + track.size.height.0;
        assert_eq!(visual_map_y_at_value(track, domain, 0.0), bottom);
        assert_eq!(visual_map_y_at_value(track, domain, 10.0), track.origin.y.0);
    }

    #[test]
    fn visual_map_track_layouts_apply_padding_and_gap() {
        let band = Rect::new(
            Point::new(Px(700.0), Px(40.0)),
            Size::new(Px(80.0), Px(220.0)),
        );
        let maps = [
            (VisualMapId::new(1), visual_map_model(1, (0.0, 10.0))),
            (VisualMapId::new(2), visual_map_model(2, (10.0, 20.0))),
        ];

        let tracks = visual_map_track_layouts(Some(band), &maps, Px(20.0), Px(10.0));

        assert_eq!(tracks.len(), 2);
        assert_eq!(tracks[0].id, VisualMapId::new(1));
        assert_eq!(tracks[0].track.origin.x.0, 710.0);
        assert_eq!(tracks[0].track.origin.y.0, 50.0);
        assert_eq!(tracks[0].track.size.width.0, 60.0);
        assert_eq!(tracks[0].track.size.height.0, 80.0);
        assert_eq!(tracks[1].id, VisualMapId::new(2));
        assert_eq!(tracks[1].track.origin.y.0, 170.0);
    }

    #[test]
    fn visual_map_track_at_selects_containing_track() {
        let band = Rect::new(
            Point::new(Px(100.0), Px(10.0)),
            Size::new(Px(20.0), Px(80.0)),
        );
        let maps = [
            (VisualMapId::new(1), visual_map_model(1, (0.0, 10.0))),
            (VisualMapId::new(2), visual_map_model(2, (0.0, 10.0))),
        ];
        let tracks = visual_map_track_layouts(Some(band), &maps, Px(0.0), Px(0.0));

        assert_eq!(
            visual_map_track_at(&tracks, Point::new(Px(110.0), Px(20.0))).map(|t| t.id),
            Some(VisualMapId::new(1))
        );
        assert_eq!(
            visual_map_track_at(&tracks, Point::new(Px(110.0), Px(70.0))).map(|t| t.id),
            Some(VisualMapId::new(2))
        );
        assert_eq!(
            visual_map_track_at(&tracks, Point::new(Px(90.0), Px(20.0))).map(|t| t.id),
            None
        );
    }
}
