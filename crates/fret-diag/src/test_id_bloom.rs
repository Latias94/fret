pub(crate) const TEST_ID_BLOOM_V1_SCHEMA_VERSION: u64 = 1;
pub(crate) const TEST_ID_BLOOM_V1_M_BITS: usize = 1024;
pub(crate) const TEST_ID_BLOOM_V1_K: usize = 4;

const TEST_ID_BLOOM_V1_BYTES: usize = TEST_ID_BLOOM_V1_M_BITS / 8;

#[derive(Clone, Copy)]
pub(crate) struct TestIdBloomV1 {
    bits: [u8; TEST_ID_BLOOM_V1_BYTES],
}

impl std::fmt::Debug for TestIdBloomV1 {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.debug_struct("TestIdBloomV1")
            .field("m_bits", &TEST_ID_BLOOM_V1_M_BITS)
            .field("k", &TEST_ID_BLOOM_V1_K)
            .finish_non_exhaustive()
    }
}

impl Default for TestIdBloomV1 {
    fn default() -> Self {
        Self::new()
    }
}

impl TestIdBloomV1 {
    pub(crate) fn new() -> Self {
        Self {
            bits: [0u8; TEST_ID_BLOOM_V1_BYTES],
        }
    }

    pub(crate) fn add(&mut self, s: &str) {
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

    pub(crate) fn might_contain(&self, s: &str) -> bool {
        let s = s.trim();
        if s.is_empty() {
            return false;
        }
        let (h1, h2) = bloom_hashes(s);
        for i in 0..TEST_ID_BLOOM_V1_K {
            let p = bloom_pos(h1, h2, i, TEST_ID_BLOOM_V1_M_BITS);
            if !get_bit(&self.bits, p) {
                return false;
            }
        }
        true
    }

    pub(crate) fn to_hex(self) -> String {
        bytes_to_hex(&self.bits)
    }

    pub(crate) fn from_hex(hex: &str) -> Option<Self> {
        let bytes = hex_to_bytes(hex)?;
        if bytes.len() != TEST_ID_BLOOM_V1_BYTES {
            return None;
        }
        let mut out = [0u8; TEST_ID_BLOOM_V1_BYTES];
        out.copy_from_slice(&bytes);
        Some(Self { bits: out })
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

fn get_bit(bits: &[u8], pos: usize) -> bool {
    let idx = pos / 8;
    let bit = pos % 8;
    bits.get(idx).is_some_and(|b| (b & (1u8 << bit)) != 0)
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

fn hex_to_bytes(s: &str) -> Option<Vec<u8>> {
    let s = s.trim();
    if s.len() % 2 != 0 {
        return None;
    }
    let mut out = Vec::with_capacity(s.len() / 2);
    let bytes = s.as_bytes();
    let mut i = 0;
    while i < bytes.len() {
        let hi = from_hex_nibble(bytes[i])?;
        let lo = from_hex_nibble(bytes[i + 1])?;
        out.push((hi << 4) | lo);
        i += 2;
    }
    Some(out)
}

fn from_hex_nibble(b: u8) -> Option<u8> {
    match b {
        b'0'..=b'9' => Some(b - b'0'),
        b'a'..=b'f' => Some(10 + (b - b'a')),
        b'A'..=b'F' => Some(10 + (b - b'A')),
        _ => None,
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn bloom_round_trips_hex() {
        let mut b = TestIdBloomV1::new();
        b.add("hello");
        b.add("world");
        let hex = b.to_hex();
        let b2 = TestIdBloomV1::from_hex(&hex).expect("decode");
        assert!(b2.might_contain("hello"));
        assert!(b2.might_contain("world"));
    }

    #[test]
    fn bloom_has_no_false_negatives_for_added() {
        let mut b = TestIdBloomV1::new();
        b.add("foo");
        b.add("bar");
        assert!(b.might_contain("foo"));
        assert!(b.might_contain("bar"));
        assert!(!b.might_contain(""));
        assert!(!b.might_contain("   "));
    }
}
