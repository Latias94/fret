use super::*;
use crate::ui::canvas::state::DrawOrderFingerprint;

pub(super) fn draw_order_fingerprint(ids: &[GraphNodeId]) -> DrawOrderFingerprint {
    fn mix64(mut x: u64) -> u64 {
        x ^= x >> 33;
        x = x.wrapping_mul(0xff51afd7ed558ccd);
        x ^= x >> 33;
        x = x.wrapping_mul(0xc4ceb9fe1a85ec53);
        x ^= x >> 33;
        x
    }

    let mut lo: u64 = 0x9e37_79b9_7f4a_7c15;
    let mut hi: u64 = 0xc2b2_ae3d_27d4_eb4f;
    let len = ids.len() as u64;
    lo ^= mix64(len);
    hi ^= mix64(len.wrapping_add(0x1656_67b1_9e37_79f9));

    for id in ids {
        let u = id.0.as_u128();
        let a = u as u64;
        let b = (u >> 64) as u64;

        lo = lo.wrapping_add(mix64(a ^ 0x243f_6a88_85a3_08d3));
        lo = lo.rotate_left(27) ^ hi;
        lo = lo.wrapping_mul(0x9e37_79b9_7f4a_7c15);

        hi = hi.wrapping_add(mix64(b ^ 0x1319_8a2e_0370_7344));
        hi = hi.rotate_left(31) ^ lo;
        hi = hi.wrapping_mul(0xc2b2_ae3d_27d4_eb4f);
    }

    DrawOrderFingerprint {
        lo: mix64(lo),
        hi: mix64(hi),
    }
}
