//! UTF-8 / UTF-16 index conversion helpers.
//!
//! These utilities are primarily intended for platform bridges (e.g. wasm DOM IME/text input),
//! where selection and composition ranges are typically expressed in UTF-16 code unit offsets.
//!
//! Conversions are deterministic and clamp to valid UTF-8 char boundaries.

/// Clamp strategy when converting an offset that may land inside a multi-unit character.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum UtfIndexClamp {
    /// Clamp down to the previous valid boundary (floor).
    Down,
    /// Clamp up to the next valid boundary (ceil).
    Up,
}

/// Convert a UTF-16 code unit offset into a UTF-8 byte offset.
///
/// Notes:
/// - Offsets are clamped to `[0, text.len()]`.
/// - If `utf16_offset` lands inside a scalar value that encodes to 2 UTF-16 code units (surrogate
///   pair), the result is clamped according to `clamp`.
pub fn utf16_offset_to_utf8_byte_offset(
    text: &str,
    utf16_offset: usize,
    clamp: UtfIndexClamp,
) -> usize {
    let target = utf16_offset;
    let mut utf16_units = 0usize;
    let mut last_byte = 0usize;

    for (byte, ch) in text.char_indices() {
        if utf16_units == target {
            return byte;
        }
        if utf16_units > target {
            return match clamp {
                UtfIndexClamp::Down => last_byte,
                UtfIndexClamp::Up => byte,
            };
        }

        last_byte = byte;
        utf16_units = utf16_units.saturating_add(ch.len_utf16());
        if utf16_units == target {
            return byte + ch.len_utf8();
        }
        if utf16_units > target {
            return match clamp {
                UtfIndexClamp::Down => byte,
                UtfIndexClamp::Up => byte + ch.len_utf8(),
            };
        }
    }

    // Target is at/after end.
    if utf16_units <= target {
        text.len()
    } else {
        // Should not happen, but keep deterministic.
        match clamp {
            UtfIndexClamp::Down => text.len(),
            UtfIndexClamp::Up => text.len(),
        }
    }
}

/// Convert a UTF-8 byte offset into a UTF-16 code unit offset.
///
/// Notes:
/// - `utf8_offset` is clamped to `[0, text.len()]`.
/// - If `utf8_offset` lands inside a UTF-8 code point, the result is clamped according to `clamp`.
pub fn utf8_byte_offset_to_utf16_offset(
    text: &str,
    utf8_offset: usize,
    clamp: UtfIndexClamp,
) -> usize {
    let target = utf8_offset.min(text.len());
    if target == 0 {
        return 0;
    }

    let mut utf16_units = 0usize;
    for (byte_start, ch) in text.char_indices() {
        let byte_end = byte_start + ch.len_utf8();
        let utf16_start = utf16_units;
        let utf16_end = utf16_start + ch.len_utf16();

        if target == byte_start {
            return utf16_start;
        }
        if target > byte_start && target < byte_end {
            return match clamp {
                UtfIndexClamp::Down => utf16_start,
                UtfIndexClamp::Up => utf16_end,
            };
        }

        utf16_units = utf16_end;
    }

    utf16_units
}

/// Convert a UTF-16 range to a UTF-8 byte range.
///
/// Start is clamped down, end is clamped up, so the resulting byte range is always valid.
pub fn utf16_range_to_utf8_byte_range(
    text: &str,
    start_utf16: usize,
    end_utf16: usize,
) -> (usize, usize) {
    let start = utf16_offset_to_utf8_byte_offset(text, start_utf16, UtfIndexClamp::Down);
    let end = utf16_offset_to_utf8_byte_offset(text, end_utf16, UtfIndexClamp::Up);
    (start.min(end), end.max(start))
}

/// Convert a UTF-8 byte range to a UTF-16 range.
///
/// Start is clamped down, end is clamped up, so the resulting UTF-16 range is always valid.
pub fn utf8_byte_range_to_utf16_range(
    text: &str,
    start_utf8: usize,
    end_utf8: usize,
) -> (usize, usize) {
    let start = utf8_byte_offset_to_utf16_offset(text, start_utf8, UtfIndexClamp::Down);
    let end = utf8_byte_offset_to_utf16_offset(text, end_utf8, UtfIndexClamp::Up);
    (start.min(end), end.max(start))
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn utf16_to_utf8_ascii_roundtrips() {
        let s = "hello";
        for i in 0..=5 {
            let b = utf16_offset_to_utf8_byte_offset(s, i, UtfIndexClamp::Down);
            assert_eq!(b, i);
            let u16 = utf8_byte_offset_to_utf16_offset(s, b, UtfIndexClamp::Down);
            assert_eq!(u16, i);
        }
    }

    #[test]
    fn utf16_to_utf8_surrogate_pair_clamps() {
        let s = "a😀b";
        // UTF-16 code units: a(1) 😀(2) b(1)
        assert_eq!(
            utf16_offset_to_utf8_byte_offset(s, 0, UtfIndexClamp::Down),
            0
        );
        assert_eq!(
            utf16_offset_to_utf8_byte_offset(s, 1, UtfIndexClamp::Down),
            1
        );
        // Inside the surrogate pair: down clamps to start of 😀, up clamps to end of 😀.
        assert_eq!(
            utf16_offset_to_utf8_byte_offset(s, 2, UtfIndexClamp::Down),
            1
        );
        assert_eq!(
            utf16_offset_to_utf8_byte_offset(s, 2, UtfIndexClamp::Up),
            1 + "😀".len()
        );
        assert_eq!(
            utf16_offset_to_utf8_byte_offset(s, 3, UtfIndexClamp::Down),
            1 + "😀".len()
        );
        assert_eq!(
            utf16_offset_to_utf8_byte_offset(s, 4, UtfIndexClamp::Down),
            s.len()
        );
    }

    #[test]
    fn utf16_range_converts_to_valid_utf8_range() {
        let s = "a😀b";
        // Select only the emoji in UTF-16: [1, 3)
        let (bs, be) = utf16_range_to_utf8_byte_range(s, 1, 3);
        assert_eq!(&s[bs..be], "😀");

        // If the DOM reports an invalid split [2, 2), it clamps to a valid (possibly empty) span.
        let (bs, be) = utf16_range_to_utf8_byte_range(s, 2, 2);
        assert!(bs <= be);
        assert!(s.is_char_boundary(bs));
        assert!(s.is_char_boundary(be));
    }

    #[test]
    fn utf8_to_utf16_clamps_inside_codepoint() {
        let s = "a😀b";
        // 😀 occupies bytes [1,5). Pick a byte inside the code point.
        let inside = 2;
        assert_eq!(
            utf8_byte_offset_to_utf16_offset(s, inside, UtfIndexClamp::Down),
            1
        );
        assert_eq!(
            utf8_byte_offset_to_utf16_offset(s, inside, UtfIndexClamp::Up),
            3
        );
    }
}
