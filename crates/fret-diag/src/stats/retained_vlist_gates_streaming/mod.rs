// Streaming retained virtual-list gates.
//
// These checks intentionally avoid materializing the full bundle artifact in memory so they can
// run on huge `bundle.json` / `bundle.schema2.json` inputs.

mod attach_detach_max;
mod keep_alive_reuse_min;
mod reconcile_no_notify_min;

pub(crate) use attach_detach_max::check_bundle_for_retained_vlist_attach_detach_max_streaming;
pub(crate) use keep_alive_reuse_min::check_bundle_for_retained_vlist_keep_alive_reuse_min_streaming;
pub(crate) use reconcile_no_notify_min::check_bundle_for_retained_vlist_reconcile_no_notify_min_streaming;
