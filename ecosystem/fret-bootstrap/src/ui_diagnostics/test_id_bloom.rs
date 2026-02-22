pub(super) const TEST_ID_BLOOM_V1_SCHEMA_VERSION: u32 = 1;
pub(super) const TEST_ID_BLOOM_V1_M_BITS: usize = 1024;
pub(super) const TEST_ID_BLOOM_V1_K: usize = 4;

const TEST_ID_BLOOM_V1_BYTES: usize = TEST_ID_BLOOM_V1_M_BITS / 8;

#[derive(Clone, Copy)]
pub(super) struct TestIdBloomV1 {
    bits: [u8; TEST_ID_BLOOM_V1_BYTES],
}

impl Default for TestIdBloomV1 {
    fn default() -> Self {
        Self::new()
    }
}

impl TestIdBloomV1 {
    pub(super) fn new() -> Self {
        Self {
            bits: [0u8; TEST_ID_BLOOM_V1_BYTES],
        }
    }

    pub(super) fn add(&mut self, s: &str) {
        let s = s.trim();
        if s.is_empty() {
            return;
        }

        let (h1, h2) = bloom_hashes(s);
        for i in 0..TEST_ID_BLOOM_V1_K {
            let p = bloom_pos(h1, h2, i, TEST_ID_BLOOM_V1_M_BITS);
            set_bit(&mut self.bits, p);
        }
    }

    pub(super) fn to_hex(self) -> String {
        bytes_to_hex(&self.bits)
    }
}

fn bloom_hashes(s: &str) -> (u64, u64) {
    let h = blake3::hash(s.as_bytes());
    let b = h.as_bytes();
    let h1 = u64::from_le_bytes(b[0..8].try_into().unwrap_or([0u8; 8]));
    let mut h2 = u64::from_le_bytes(b[8..16].try_into().unwrap_or([0u8; 8]));
    if h2 == 0 {
        h2 = 0x9e3779b97f4a7c15;
    }
    (h1, h2)
}

fn bloom_pos(h1: u64, h2: u64, i: usize, m_bits: usize) -> usize {
    let m = m_bits as u64;
    let v = h1.wrapping_add((i as u64).wrapping_mul(h2));
    (v % m) as usize
}

fn set_bit(bits: &mut [u8], pos: usize) {
    let idx = pos / 8;
    let bit = pos % 8;
    if let Some(b) = bits.get_mut(idx) {
        *b |= 1u8 << bit;
    }
}

fn bytes_to_hex(bytes: &[u8]) -> String {
    const LUT: &[u8; 16] = b"0123456789abcdef";
    let mut out = vec![0u8; bytes.len() * 2];
    for (i, b) in bytes.iter().copied().enumerate() {
        out[i * 2] = LUT[(b >> 4) as usize];
        out[i * 2 + 1] = LUT[(b & 0x0f) as usize];
    }
    String::from_utf8(out).unwrap_or_default()
}
