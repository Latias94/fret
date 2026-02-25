// Streaming wheel-scroll gates.
//
// These checks intentionally avoid materializing the full bundle artifact in memory so they can
// run on huge `bundle.json` / `bundle.schema2.json` inputs.

mod legacy;
mod types;
mod wheel_frames_min;

pub(crate) use legacy::{
    check_bundle_for_wheel_scroll_hit_changes_streaming, check_bundle_for_wheel_scroll_streaming,
};
