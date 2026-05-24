use delinea::VisualMapId;
use delinea::engine::model::VisualMapModel;
use delinea::engine::window::DataWindow;
use fret_core::{Point, Px, Rect, Size};

use crate::slider_logic::{SliderDragKind, slider_window_after_delta};

#[derive(Debug, Clone, Copy, PartialEq)]
pub(crate) struct VisualMapTrackLayout {
    pub id: VisualMapId,
    pub model: VisualMapModel,
    pub track: Rect,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub(crate) struct VisualMapPieceMaskUpdate {
    pub mask: Option<u64>,
    pub anchor: Option<(VisualMapId, u32)>,
}

#[derive(Debug, Clone, Copy, PartialEq)]
pub(crate) struct VisualMapContinuousDragStart {
    pub kind: SliderDragKind,
    pub start_window: DataWindow,
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

pub(crate) fn visual_map_full_piece_mask(vm: VisualMapModel) -> u64 {
    let buckets = vm.buckets.clamp(1, 64) as u32;
    if buckets >= 64 {
        u64::MAX
    } else {
        (1u64 << buckets) - 1
    }
}

pub(crate) fn visual_map_current_piece_mask(vm: VisualMapModel, state_mask: Option<u64>) -> u64 {
    let full_mask = visual_map_full_piece_mask(vm);
    state_mask.or(vm.initial_piece_mask).unwrap_or(full_mask) & full_mask
}

pub(crate) fn visual_map_piece_mask_after_click(
    id: VisualMapId,
    vm: VisualMapModel,
    click_value: f64,
    current: u64,
    anchor: Option<(VisualMapId, u32)>,
    shift: bool,
    reset: bool,
) -> VisualMapPieceMaskUpdate {
    if reset {
        return VisualMapPieceMaskUpdate {
            mask: None,
            anchor: None,
        };
    }

    let buckets = vm.buckets.clamp(1, 64) as u32;
    let full_mask = visual_map_full_piece_mask(vm);
    let bucket = delinea::visual_map::bucket_index_for_value(&vm, click_value) as u32;
    let bit = 1u64 << bucket.min(63);
    let is_selected = ((current >> bucket) & 1) == 1;

    let mut next = current;
    if shift {
        if let Some((anchor_vm, anchor_bucket)) = anchor
            && anchor_vm == id
        {
            let lo = anchor_bucket.min(bucket);
            let hi = anchor_bucket.max(bucket).min(buckets.saturating_sub(1));
            let width = hi.saturating_sub(lo).saturating_add(1);
            let range_mask = if width >= 64 {
                u64::MAX
            } else {
                ((1u64 << width) - 1) << lo
            } & full_mask;

            if is_selected {
                next &= !range_mask;
            } else {
                next |= range_mask;
            }
        } else {
            next ^= bit;
        }
    } else {
        next ^= bit;
    }
    next &= full_mask;

    VisualMapPieceMaskUpdate {
        mask: (next != full_mask).then_some(next),
        anchor: Some((id, bucket)),
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

pub(crate) fn visual_map_continuous_drag_start(
    track: Rect,
    domain: DataWindow,
    current_window: DataWindow,
    click_value: f64,
    position_y: f32,
    handle_hit_px: f32,
) -> VisualMapContinuousDragStart {
    let y_min = visual_map_y_at_value(track, domain, current_window.min);
    let y_max = visual_map_y_at_value(track, domain, current_window.max);
    let (top, bottom) = (y_max.min(y_min), y_max.max(y_min));
    let handle_hit_px = handle_hit_px.max(0.0);

    if (position_y - y_min).abs() <= handle_hit_px {
        return VisualMapContinuousDragStart {
            kind: SliderDragKind::HandleMin,
            start_window: current_window,
        };
    }
    if (position_y - y_max).abs() <= handle_hit_px {
        return VisualMapContinuousDragStart {
            kind: SliderDragKind::HandleMax,
            start_window: current_window,
        };
    }
    if position_y >= top && position_y <= bottom {
        return VisualMapContinuousDragStart {
            kind: SliderDragKind::Pan,
            start_window: current_window,
        };
    }

    let center = (current_window.min + current_window.max) * 0.5;
    let delta = click_value - center;
    VisualMapContinuousDragStart {
        kind: SliderDragKind::Pan,
        start_window: slider_window_after_delta(domain, current_window, delta, SliderDragKind::Pan),
    }
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

    #[test]
    fn visual_map_piece_mask_update_toggles_ranges_and_resets() {
        let mut vm = visual_map_model(1, (0.0, 100.0));
        vm.mode = delinea::VisualMapMode::Piecewise;
        vm.buckets = 8;

        let full = visual_map_full_piece_mask(vm);
        assert_eq!(full, 0xff);
        assert_eq!(visual_map_current_piece_mask(vm, None), full);

        let first = visual_map_piece_mask_after_click(
            VisualMapId::new(1),
            vm,
            25.0,
            full,
            None,
            false,
            false,
        );
        assert_eq!(first.mask, Some(0xfb));
        assert_eq!(first.anchor, Some((VisualMapId::new(1), 2)));

        let range = visual_map_piece_mask_after_click(
            VisualMapId::new(1),
            vm,
            62.5,
            first.mask.expect("partial mask"),
            first.anchor,
            true,
            false,
        );
        assert_eq!(range.mask, Some(0xc3));
        assert_eq!(range.anchor, Some((VisualMapId::new(1), 5)));

        let reset = visual_map_piece_mask_after_click(
            VisualMapId::new(1),
            vm,
            62.5,
            range.mask.expect("partial mask"),
            range.anchor,
            false,
            true,
        );
        assert_eq!(
            reset,
            VisualMapPieceMaskUpdate {
                mask: None,
                anchor: None
            }
        );
    }

    #[test]
    fn visual_map_continuous_drag_start_selects_handles_pan_and_jump() {
        let track = Rect::new(
            Point::new(Px(10.0), Px(20.0)),
            Size::new(Px(8.0), Px(100.0)),
        );
        let domain = DataWindow {
            min: 0.0,
            max: 100.0,
        };
        let current = DataWindow {
            min: 25.0,
            max: 75.0,
        };

        let min_handle = visual_map_continuous_drag_start(track, domain, current, 25.0, 95.0, 8.0);
        assert_eq!(min_handle.kind, SliderDragKind::HandleMin);
        assert_eq!(min_handle.start_window, current);

        let max_handle = visual_map_continuous_drag_start(track, domain, current, 75.0, 45.0, 8.0);
        assert_eq!(max_handle.kind, SliderDragKind::HandleMax);
        assert_eq!(max_handle.start_window, current);

        let pan = visual_map_continuous_drag_start(track, domain, current, 50.0, 70.0, 8.0);
        assert_eq!(pan.kind, SliderDragKind::Pan);
        assert_eq!(pan.start_window, current);

        let jumped = visual_map_continuous_drag_start(track, domain, current, 90.0, 30.0, 1.0);
        assert_eq!(jumped.kind, SliderDragKind::Pan);
        assert_eq!(
            jumped.start_window,
            DataWindow {
                min: 50.0,
                max: 100.0
            }
        );
    }
}
