use fret_runtime::Model;
use fret_ui::{ElementContext, Invalidation, UiHost};

#[track_caller]
pub(super) fn linked_scale_model<H: UiHost>(
    cx: &mut ElementContext<'_, H>,
    default: bool,
) -> Model<bool> {
    cx.local_model(move || default)
}

#[track_caller]
pub(super) fn uniform_scale_sync_slot<H: UiHost>(
    cx: &mut ElementContext<'_, H>,
) -> fret_ui::GlobalElementId {
    cx.slot_id()
}

fn approx_eq(a: f64, b: f64) -> bool {
    (a - b).abs() <= 1e-9
}

fn next_uniform_scale(
    previous: Option<(f64, f64, f64)>,
    current: (f64, f64, f64),
) -> Option<(f64, f64, f64)> {
    let (sx, sy, sz) = current;
    match previous {
        None => Some((sx, sx, sx)),
        Some((lx, ly, lz)) => {
            let dx = !approx_eq(sx, lx);
            let dy = !approx_eq(sy, ly);
            let dz = !approx_eq(sz, lz);
            let diffs = dx as u8 + dy as u8 + dz as u8;
            if diffs == 1 {
                if dx {
                    Some((sx, sx, sx))
                } else if dy {
                    Some((sy, sy, sy))
                } else {
                    Some((sz, sz, sz))
                }
            } else {
                None
            }
        }
    }
}

pub(super) fn uniform_scale_sync<H: UiHost>(
    cx: &mut ElementContext<'_, H>,
    sync_slot: fret_ui::GlobalElementId,
    _linked: &Model<bool>,
    scale: (&Model<f64>, &Model<f64>, &Model<f64>),
) {
    let (sx, sy, sz) = (
        cx.get_model_copied(scale.0, Invalidation::Layout)
            .unwrap_or(1.0),
        cx.get_model_copied(scale.1, Invalidation::Layout)
            .unwrap_or(1.0),
        cx.get_model_copied(scale.2, Invalidation::Layout)
            .unwrap_or(1.0),
    );

    let next = cx.state_for(
        sync_slot,
        || None::<(f64, f64, f64)>,
        |last| {
            let last_v = *last;
            *last = Some((sx, sy, sz));
            next_uniform_scale(last_v, (sx, sy, sz))
        },
    );

    let Some((ux, uy, uz)) = next else { return };

    let mut did = false;
    if !approx_eq(sx, ux) {
        let _ = cx.app.models_mut().update(scale.0, |v| *v = ux);
        did = true;
    }
    if !approx_eq(sy, uy) {
        let _ = cx.app.models_mut().update(scale.1, |v| *v = uy);
        did = true;
    }
    if !approx_eq(sz, uz) {
        let _ = cx.app.models_mut().update(scale.2, |v| *v = uz);
        did = true;
    }

    if did {
        cx.state_for(
            sync_slot,
            || None::<(f64, f64, f64)>,
            |last| *last = Some((ux, uy, uz)),
        );
    }
}

#[cfg(test)]
mod tests {
    use super::next_uniform_scale;

    #[test]
    fn uniform_scale_sync_initializes_to_x_axis_value() {
        assert_eq!(
            next_uniform_scale(None, (2.0, 3.0, 4.0)),
            Some((2.0, 2.0, 2.0))
        );
    }

    #[test]
    fn uniform_scale_sync_projects_single_axis_edits_to_all_axes() {
        assert_eq!(
            next_uniform_scale(Some((1.0, 1.0, 1.0)), (1.0, 2.0, 1.0)),
            Some((2.0, 2.0, 2.0))
        );
        assert_eq!(
            next_uniform_scale(Some((1.0, 1.0, 1.0)), (1.0, 1.0, 3.0)),
            Some((3.0, 3.0, 3.0))
        );
    }

    #[test]
    fn uniform_scale_sync_ignores_multi_axis_or_near_equal_edits() {
        assert_eq!(
            next_uniform_scale(Some((1.0, 1.0, 1.0)), (2.0, 2.0, 1.0)),
            None
        );
        assert_eq!(
            next_uniform_scale(Some((1.0, 1.0, 1.0)), (1.0 + 1e-10, 1.0, 1.0)),
            None
        );
    }
}
