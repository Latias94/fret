use super::*;

pub(super) fn build_paste_transaction<H: UiHost, M: NodeGraphCanvasMiddleware>(
    canvas: &mut NodeGraphCanvasWith<M>,
    host: &mut H,
    fragment: &GraphFragment,
    tuning: PasteTuning,
    label: Option<String>,
) -> GraphTransaction {
    let remapper = IdRemapper::new(IdRemapSeed::new_random());
    let mut tx = fragment.to_paste_transaction(&remapper, tuning);
    retain_non_duplicate_import_ops(canvas, host, fragment, &mut tx);
    tx.label = label;
    tx
}

fn retain_non_duplicate_import_ops<H: UiHost, M: NodeGraphCanvasMiddleware>(
    canvas: &mut NodeGraphCanvasWith<M>,
    host: &mut H,
    fragment: &GraphFragment,
    tx: &mut GraphTransaction,
) {
    if fragment.imports.is_empty() {
        return;
    }

    canvas
        .graph
        .read_ref(host, |graph| {
            tx.ops.retain(
                |op| !matches!(op, GraphOp::AddImport { id, .. } if graph.imports.contains_key(id)),
            );
        })
        .ok();
}
