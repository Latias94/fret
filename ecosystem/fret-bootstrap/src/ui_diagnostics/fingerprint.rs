fn semantics_fingerprint_v1(
    snapshot: &fret_core::SemanticsSnapshot,
    redact_text: bool,
    max_string_bytes: usize,
) -> u64 {
    let mut hasher = Fnv1a64::new();
    hasher.write_u64(snapshot.window.data().as_ffi());

    for root in &snapshot.roots {
        hasher.write_u64(key_to_u64(root.root));
        hasher.write_bool(root.visible);
        hasher.write_bool(root.blocks_underlay_input);
        hasher.write_bool(root.hit_testable);
        hasher.write_u32(root.z_index);
    }

    hasher.write_opt_u64(snapshot.barrier_root.map(key_to_u64));
    hasher.write_opt_u64(snapshot.focus_barrier_root.map(key_to_u64));
    hasher.write_opt_u64(snapshot.focus.map(key_to_u64));
    hasher.write_opt_u64(snapshot.captured.map(key_to_u64));

    for node in &snapshot.nodes {
        hasher.write_u64(key_to_u64(node.id));
        hasher.write_opt_u64(node.parent.map(key_to_u64));
        hasher.write_str_bytes(semantics_role_label(node.role).as_bytes());

        hasher.write_f32(node.bounds.origin.x.0);
        hasher.write_f32(node.bounds.origin.y.0);
        hasher.write_f32(node.bounds.size.width.0);
        hasher.write_f32(node.bounds.size.height.0);

        hasher.write_bool(node.flags.focused);
        hasher.write_bool(node.flags.captured);
        hasher.write_bool(node.flags.disabled);
        hasher.write_bool(node.flags.selected);
        hasher.write_bool(node.flags.expanded);
        hasher.write_opt_bool(node.flags.checked);

        hasher.write_opt_str(node.test_id.as_deref(), redact_text, max_string_bytes);
        hasher.write_opt_u64(node.active_descendant.map(key_to_u64));
        hasher.write_opt_u32(node.pos_in_set);
        hasher.write_opt_u32(node.set_size);
        hasher.write_opt_str(node.label.as_deref(), redact_text, max_string_bytes);
        hasher.write_opt_str(node.value.as_deref(), redact_text, max_string_bytes);
        hasher.write_opt_pair_u32(node.text_selection);
        hasher.write_opt_pair_u32(node.text_composition);

        hasher.write_bool(node.actions.focus);
        hasher.write_bool(node.actions.invoke);
        hasher.write_bool(node.actions.set_value);
        hasher.write_bool(node.actions.set_text_selection);

        hasher.write_u32(node.labelled_by.len() as u32);
        for id in &node.labelled_by {
            hasher.write_u64(key_to_u64(*id));
        }
        hasher.write_u32(node.described_by.len() as u32);
        for id in &node.described_by {
            hasher.write_u64(key_to_u64(*id));
        }
        hasher.write_u32(node.controls.len() as u32);
        for id in &node.controls {
            hasher.write_u64(key_to_u64(*id));
        }
    }

    hasher.finish()
}

struct Fnv1a64 {
    state: u64,
}

impl Fnv1a64 {
    const OFFSET_BASIS: u64 = 0xcbf29ce484222325;
    const PRIME: u64 = 0x100000001b3;

    fn new() -> Self {
        Self {
            state: Self::OFFSET_BASIS,
        }
    }

    fn write_u8(&mut self, v: u8) {
        self.state ^= v as u64;
        self.state = self.state.wrapping_mul(Self::PRIME);
    }

    fn write_bytes(&mut self, bytes: &[u8]) {
        for &b in bytes {
            self.write_u8(b);
        }
    }

    fn write_u32(&mut self, v: u32) {
        self.write_bytes(&v.to_le_bytes());
    }

    fn write_u64(&mut self, v: u64) {
        self.write_bytes(&v.to_le_bytes());
    }

    fn write_f32(&mut self, v: f32) {
        self.write_u32(v.to_bits());
    }

    fn write_bool(&mut self, v: bool) {
        self.write_u8(if v { 1 } else { 0 });
    }

    fn write_opt_u64(&mut self, v: Option<u64>) {
        match v {
            Some(v) => {
                self.write_u8(1);
                self.write_u64(v);
            }
            None => self.write_u8(0),
        }
    }

    fn write_opt_u32(&mut self, v: Option<u32>) {
        match v {
            Some(v) => {
                self.write_u8(1);
                self.write_u32(v);
            }
            None => self.write_u8(0),
        }
    }

    fn write_opt_bool(&mut self, v: Option<bool>) {
        match v {
            Some(v) => {
                self.write_u8(1);
                self.write_bool(v);
            }
            None => self.write_u8(0),
        }
    }

    fn write_opt_pair_u32(&mut self, v: Option<(u32, u32)>) {
        match v {
            Some((a, b)) => {
                self.write_u8(1);
                self.write_u32(a);
                self.write_u32(b);
            }
            None => self.write_u8(0),
        }
    }

    fn write_str_bytes(&mut self, bytes: &[u8]) {
        self.write_u32(bytes.len() as u32);
        self.write_bytes(bytes);
    }

    fn write_opt_str(&mut self, s: Option<&str>, redact_text: bool, max_string_bytes: usize) {
        match s {
            Some(s) => {
                self.write_u8(1);
                if redact_text {
                    self.write_u32(s.len().min(u32::MAX as usize) as u32);
                } else {
                    let bytes = s.as_bytes();
                    self.write_u32(bytes.len().min(max_string_bytes) as u32);
                    self.write_bytes(&bytes[..bytes.len().min(max_string_bytes)]);
                }
            }
            None => self.write_u8(0),
        }
    }

    fn finish(self) -> u64 {
        self.state
    }
}
