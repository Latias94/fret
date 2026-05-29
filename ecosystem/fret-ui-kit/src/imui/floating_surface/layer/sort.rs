use fret_ui::element::AnyElement;

use super::z_order::FloatWindowLayerZOrderSnapshot;

pub(super) fn sort_floating_layer_windows(
    windows: Vec<AnyElement>,
    z_order: &FloatWindowLayerZOrderSnapshot,
) -> Vec<AnyElement> {
    let mut indexed: Vec<(usize, usize, AnyElement)> = windows
        .into_iter()
        .enumerate()
        .map(|(original, w)| {
            let idx = z_order.rank.get(&w.id).copied().unwrap_or(usize::MAX);
            (idx, original, w)
        })
        .collect();

    indexed.sort_by_key(|(idx, original, _)| (*idx, *original));
    indexed.into_iter().map(|(_, _, w)| w).collect()
}
