use read_fonts::tables::name::NameId;
use read_fonts::{FontRef, TableProvider as _};

pub fn best_family_name_from_font_bytes(bytes: &[u8], face_index: u32) -> Option<String> {
    let face = FontRef::from_index(bytes, face_index).ok()?;
    let name_table = face.name().ok()?;
    let string_data = name_table.string_data();

    let mut best: Option<(i32, String)> = None;
    for record in name_table.name_record() {
        let name_id = record.name_id();
        let is_typographic_family = name_id == NameId::new(16);
        let is_family = name_id == NameId::new(1);
        if !is_typographic_family && !is_family {
            continue;
        }

        let Ok(value) = record.string(string_data).map(|s| s.to_string()) else {
            continue;
        };
        let value = value.trim().to_string();
        if value.is_empty() {
            continue;
        }

        let mut score: i32 = 0;
        score += if is_typographic_family { 200 } else { 180 };
        if record.is_unicode() {
            score += 10;
        }
        // Prefer Windows + en-US when available.
        if record.platform_id() == 3 && record.language_id() == 0x0409 {
            score += 5;
        }
        // Prefer shorter strings if otherwise tied.
        score -= (value.len() as i32).min(128);

        match &best {
            Some((best_score, _)) if *best_score >= score => {}
            _ => best = Some((score, value)),
        }
    }

    Some(best?.1)
}
