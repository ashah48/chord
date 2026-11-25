//! Offline Twemoji graphics (CC BY 4.0), packed by scripts/build-emoji-atlas.py.
use eframe::egui::{self, Color32, FontFamily, FontId, Rect, TextureHandle, Vec2};
use std::{
    collections::{HashMap, HashSet},
    sync::OnceLock,
};
use unicode_segmentation::UnicodeSegmentation;

fn index() -> &'static HashMap<String, usize> {
    static INDEX: OnceLock<HashMap<String, usize>> = OnceLock::new();
    INDEX.get_or_init(|| {
        serde_json::from_str(include_str!("../../assets/emoji-atlas.json"))
            .expect("bundled emoji index")
    })
}
fn lookup(emoji: &str) -> Option<usize> {
    let first = emoji.chars().next()?;
    if first < '\u{2000}' && first != '©' && first != '®' && !emoji.contains('\u{20e3}') {
        return None;
    }
    let key = emoji
        .chars()
        .map(|c| format!("{:x}", c as u32))
        .collect::<Vec<_>>()
        .join("-");
    index()
        .get(&key)
        .or_else(|| {
            index().get(
                &key.split('-')
                    .filter(|part| *part != "fe0f")
                    .collect::<Vec<_>>()
                    .join("-"),
            )
        })
        .copied()
}
fn texture(ctx: &egui::Context) -> TextureHandle {
    let id = egui::Id::new("color-emoji-atlas");
    if let Some(texture) = ctx.data(|data| data.get_temp::<TextureHandle>(id)) {
        return texture;
    }
    let image = image::load_from_memory(include_bytes!("../../assets/emoji-atlas.png"))
        .expect("bundled emoji atlas")
        .to_rgba8();
    let texture = ctx.load_texture(
        "color-emoji-atlas",
        egui::ColorImage::from_rgba_unmultiplied(
            [image.width() as usize, image.height() as usize],
            image.as_raw(),
        ),
        egui::TextureOptions::LINEAR,
    );
    ctx.data_mut(|data| data.insert_temp(id, texture.clone()));
    texture
}
fn uv(index: usize, texture: &TextureHandle) -> Rect {
    let cell = Vec2::splat(32.) / texture.size_vec2();
    let min = egui::vec2((index % 64) as f32, (index / 64) as f32) * cell;
    Rect::from_min_size(egui::pos2(min.x, min.y), cell)
}

// Retain original Unicode for selection, copying, and link hit-testing. The
// leading glyph reserves a square; invisible continuation glyphs keep ZWJ,
// flags, and skin-tone sequences together without taking extra visible space.
pub fn append(job: &mut egui::text::LayoutJob, text: &str, format: egui::TextFormat) {
    append_ready(job, text, format, &HashSet::new());
}

// A ready Slack shortcode stays in the string, so copying still yields
// `:boomparrot:`. The first colon holds a square the image is painted into,
// and the rest of the shortcode takes no visible space.
pub fn append_ready(
    job: &mut egui::text::LayoutJob,
    text: &str,
    format: egui::TextFormat,
    ready: &HashSet<&str>,
) {
    let mut offset = 0;
    while offset < text.len() {
        if let Some((name, len)) = shortcode_at(&text[offset..])
            && ready.contains(name)
        {
            collapse(job, &text[offset..offset + len], &format);
            offset += len;
            continue;
        }
        let grapheme = text[offset..].graphemes(true).next().unwrap();
        if lookup(grapheme).is_some() {
            let first_len = grapheme.chars().next().unwrap().len_utf8();
            let mut head = format.clone();
            head.font_id = FontId::new(format.font_id.size, FontFamily::Name("emoji".into()));
            head.color = Color32::TRANSPARENT;
            head.underline = egui::Stroke::NONE;
            job.append(&grapheme[..first_len], 0., head.clone());
            if first_len < grapheme.len() {
                head.font_id.size = 0.01;
                job.append(&grapheme[first_len..], 0., head);
            }
        } else {
            job.append(grapheme, 0., format.clone());
        }
        offset += grapheme.len();
    }
}

fn shortcode_at(text: &str) -> Option<(&str, usize)> {
    let rest = text.strip_prefix(':')?;
    let end = rest.find(':')?;
    let name = &rest[..end];
    if name.is_empty()
        || name.len() > 64
        || !name
            .bytes()
            .all(|b| b.is_ascii_alphanumeric() || matches!(b, b'_' | b'+' | b'-'))
    {
        return None;
    }
    Some((name, name.len() + 2))
}

pub fn custom_spans<'a>(
    text: &'a str,
    ready: &HashSet<&str>,
) -> Vec<(std::ops::Range<usize>, &'a str)> {
    let mut spans = Vec::new();
    let mut offset = 0;
    while offset < text.len() {
        if let Some((name, len)) = shortcode_at(&text[offset..])
            && ready.contains(name)
        {
            spans.push((offset..offset + len, name));
            offset += len;
            continue;
        }
        offset += text[offset..].chars().next().map_or(1, char::len_utf8);
    }
    spans
}

fn collapse(job: &mut egui::text::LayoutJob, token: &str, format: &egui::TextFormat) {
    let first = token.chars().next().unwrap().len_utf8();
    let mut head = format.clone();
    head.color = Color32::TRANSPARENT;
    head.underline = egui::Stroke::NONE;
    head.strikethrough = egui::Stroke::NONE;
    head.extra_letter_spacing = format.font_id.size;
    job.append(&token[..first], 0., head.clone());
    if first < token.len() {
        head.font_id.size = 0.01;
        head.extra_letter_spacing = 0.;
        job.append(&token[first..], 0., head);
    }
}
pub fn paint(ui: &egui::Ui, galley: &egui::Galley, origin: egui::Pos2) {
    paint_clipped(ui, galley, origin, ui.clip_rect());
}
pub fn paint_clipped(ui: &egui::Ui, galley: &egui::Galley, origin: egui::Pos2, clip: Rect) {
    let mut character = 0;
    for grapheme in galley.job.text.graphemes(true) {
        let visible_end = galley
            .end()
            .index
            .saturating_sub(usize::from(galley.elided));
        if character + grapheme.chars().count() > visible_end {
            break;
        }
        if let Some(index) = lookup(grapheme) {
            let cursor = galley.pos_from_cursor(egui::text::CCursor::new(character));
            let rect = cursor.translate(origin.to_vec2());
            let next = galley.pos_from_cursor(egui::text::CCursor::new(character + 1));
            let advance = next.left() - cursor.left();
            let size = if advance > 1. {
                rect.height().min(advance)
            } else {
                rect.height()
            }
            .max(1.);
            let target = Rect::from_min_size(
                egui::pos2(rect.left(), rect.center().y - size / 2.),
                Vec2::splat(size),
            );
            if ui.is_rect_visible(target) {
                let texture = texture(ui.ctx());
                ui.painter().with_clip_rect(clip).image(
                    texture.id(),
                    target,
                    uv(index, &texture),
                    Color32::WHITE,
                );
            }
        }
        character += grapheme.chars().count();
    }
}

pub fn paint_custom(
    ui: &egui::Ui,
    galley: &egui::Galley,
    origin: egui::Pos2,
    slots: &[(usize, TextureHandle)],
) {
    for (character, texture) in slots {
        let cursor = galley.pos_from_cursor(egui::text::CCursor::new(*character));
        let rect = cursor.translate(origin.to_vec2());
        let next = galley.pos_from_cursor(egui::text::CCursor::new(character + 1));
        let advance = next.left() - cursor.left();
        let size = if advance > 1. {
            rect.height().min(advance)
        } else {
            rect.height()
        }
        .max(1.);
        let target = Rect::from_min_size(
            egui::pos2(rect.left(), rect.center().y - size / 2.),
            Vec2::splat(size),
        );
        if ui.is_rect_visible(target) {
            ui.painter().image(
                texture.id(),
                target,
                Rect::from_min_max(egui::pos2(0., 0.), egui::pos2(1., 1.)),
                Color32::WHITE,
            );
        }
    }
}

pub fn icon(ui: &egui::Ui, key: &str, size: f32) -> Option<egui::Image<'static>> {
    let index = lookup(key)?;
    let texture = texture(ui.ctx());
    Some(egui::Image::new((texture.id(), Vec2::splat(size))).uv(uv(index, &texture)))
}

pub const REACTION_SECTIONS: &[(&str, &[(&str, &str)])] = &[
    (
        "Common emoji",
        &[
            ("👍", "thumbs up +1 like"),
            ("❤️", "red heart love"),
            ("🔥", "fire lit"),
            ("😂", "tears of joy laugh"),
            ("🎉", "party popper celebration"),
            ("👀", "eyes"),
            ("🙏", "folded hands thanks pray"),
            ("✅", "check mark yes"),
            ("💯", "hundred points 100"),
            ("💙", "blue heart"),
            ("😮", "surprised open mouth"),
            ("😢", "crying face sad"),
        ],
    ),
    (
        "Faces",
        &[
            ("😀", "grinning face"),
            ("😄", "happy smile"),
            ("😊", "smiling eyes blush"),
            ("🤣", "rolling on the floor laughing rofl"),
            ("😭", "loudly crying sob"),
            ("😍", "heart eyes"),
            ("🥰", "smiling face with hearts"),
            ("🤩", "star struck"),
            ("😎", "sunglasses cool"),
            ("😉", "wink"),
            ("🙂", "slight smile"),
            ("🙃", "upside down face"),
            ("🤔", "thinking face"),
            ("🫡", "saluting face"),
            ("😬", "grimacing face"),
            ("😅", "smile sweat"),
            ("😳", "flushed face"),
            ("🥺", "pleading face"),
            ("😐", "neutral face"),
            ("🙄", "rolling eyes"),
            ("😡", "angry face"),
            ("😱", "screaming face"),
            ("🤯", "exploding head"),
            ("💀", "skull"),
            ("🤡", "clown face"),
            ("🫠", "melting face"),
        ],
    ),
    (
        "Hands",
        &[
            ("👎", "thumbs down -1 dislike"),
            ("👏", "clapping hands applause"),
            ("🙌", "raising hands hooray"),
            ("👋", "waving hand hello"),
            ("👌", "OK hand"),
            ("✌️", "victory peace"),
            ("🤝", "handshake"),
            ("💪", "flexed biceps muscle"),
            ("🫶", "heart hands"),
            ("🤞", "crossed fingers"),
            ("👆", "point up"),
            ("👇", "point down"),
            ("✍️", "writing hand"),
        ],
    ),
    (
        "Symbols and objects",
        &[
            ("✨", "sparkles"),
            ("⭐", "star"),
            ("💖", "sparkling heart"),
            ("💔", "broken heart"),
            ("💕", "two hearts"),
            ("‼️", "double exclamation"),
            ("❓", "question mark"),
            ("❌", "cross mark no"),
            ("⚠️", "warning"),
            ("🚀", "rocket"),
            ("💡", "light bulb idea"),
            ("💎", "gem diamond"),
            ("🏆", "trophy"),
            ("🎶", "music notes"),
            ("🏁", "checkered flag"),
            ("🎂", "birthday cake"),
            ("🍕", "pizza"),
            ("☕", "coffee"),
            ("🐈", "cat"),
            ("🐕", "dog"),
        ],
    ),
];

fn common_name(key: &str) -> Option<&'static str> {
    REACTION_SECTIONS
        .iter()
        .flat_map(|(_, entries)| entries.iter())
        .find_map(|(emoji, name)| (*emoji == key).then_some(*name))
}

pub fn reaction_name(key: &str) -> Option<&'static str> {
    common_name(key).or_else(|| {
        catalog()
            .iter()
            .flat_map(|(_, entries)| entries)
            .find(|entry| entry.glyph == key)
            .map(|entry| entry.name)
    })
}

pub struct CatalogEmoji {
    pub glyph: &'static str,
    pub name: &'static str,
    pub shortcode: String,
}

impl CatalogEmoji {
    pub fn matches(&self, query: &str) -> bool {
        query.is_empty()
            || self.shortcode.contains(query)
            || matches_search(self.glyph, self.name, query)
            || common_name(self.glyph).is_some_and(|name| matches_search(self.glyph, name, query))
    }
}

pub fn catalog() -> &'static [(&'static str, Vec<CatalogEmoji>)] {
    static CATALOG: OnceLock<Vec<(&str, Vec<CatalogEmoji>)>> = OnceLock::new();
    CATALOG.get_or_init(|| {
        let mut groups = Vec::<(&str, Vec<CatalogEmoji>)>::new();
        for line in include_str!("../../assets/emoji-catalog.tsv").lines() {
            if let Some(group) = line
                .strip_prefix('[')
                .and_then(|line| line.strip_suffix(']'))
            {
                groups.push((group, Vec::new()));
            } else if let Some((glyph, name)) = line.split_once('\t') {
                let shortcode = standard_shortcode(glyph, name);
                groups
                    .last_mut()
                    .expect("emoji group")
                    .1
                    .push(CatalogEmoji {
                        glyph,
                        name,
                        shortcode,
                    });
            }
        }
        groups
    })
}

pub fn standard_shortcode(key: &str, name: &str) -> String {
    let mut regional = key.chars();
    if let (Some(first), Some(second), None) = (regional.next(), regional.next(), regional.next())
        && [first, second].iter().all(|c| ('🇦'..='🇿').contains(c))
    {
        return format!(
            "flag_{}{}",
            (b'a' + (first as u32 - '🇦' as u32) as u8) as char,
            (b'a' + (second as u32 - '🇦' as u32) as u8) as char
        );
    }
    match key {
        "👍" => "thumbsup",
        "👎" => "thumbsdown",
        "❤️" => "heart",
        "😂" => "joy",
        "🤣" => "rofl",
        "🎉" => "tada",
        "🙏" => "pray",
        "✅" => "white_check_mark",
        "😭" => "sob",
        "😢" => "cry",
        "100" | "💯" => "100",
        "😀" => "grinning",
        "😄" => "smile",
        "😊" => "blush",
        _ => {
            return name
                .split(|c: char| !c.is_alphanumeric())
                .filter(|word| !word.is_empty())
                .collect::<Vec<_>>()
                .join("_")
                .to_lowercase();
        }
    }
    .to_owned()
}

pub fn completion_token(text: &str, cursor: usize) -> Option<(std::ops::Range<usize>, &str)> {
    completion_token_for(text, cursor, ':')
}

pub fn completion_token_for(
    text: &str,
    cursor: usize,
    trigger: char,
) -> Option<(std::ops::Range<usize>, &str)> {
    let end = text
        .char_indices()
        .nth(cursor)
        .map_or(text.len(), |(offset, _)| offset);
    let start = text[..end].rfind(trigger)?;
    if text[..start]
        .chars()
        .next_back()
        .is_some_and(|c| !c.is_whitespace() && !"([{".contains(c))
    {
        return None;
    }
    let query = &text[start + 1..end];
    let valid = |c: char| {
        if trigger == '@' {
            c.is_alphanumeric() || "_.-".contains(c)
        } else {
            c.is_ascii_alphanumeric() || "_+-~".contains(c)
        }
    };
    if query.len() > 40 || !query.chars().all(valid) {
        return None;
    }
    let suffix = &text[end..];
    let token_end = end + suffix.find(|c: char| !valid(c)).unwrap_or(suffix.len());
    let token_end = token_end + usize::from(trigger == ':' && text[token_end..].starts_with(':'));
    Some((start..token_end, query))
}

pub fn matches_search(key: &str, name: &str, query: &str) -> bool {
    let name = name.to_lowercase().replace(['_', '-'], " ");
    query
        .replace('_', " ")
        .split_whitespace()
        .all(|term| name.contains(term) || key.contains(term))
}

pub fn button(ui: &egui::Ui, key: &str, label: &str) -> egui::Button<'static> {
    if let Some(image) = icon(ui, key, 22.) {
        egui::Button::image_and_text(image, label.to_owned())
    } else {
        egui::Button::new(format!("{key} {label}"))
    }
}
#[cfg(test)]
mod tests {
    use super::*;
    #[test]
    fn unicode_catalog_covers_the_standard_and_has_artwork() {
        let entries: Vec<_> = catalog().iter().flat_map(|(_, entries)| entries).collect();
        assert_eq!(entries.len(), 3953);
        assert_eq!(
            entries
                .iter()
                .map(|entry| entry.glyph)
                .collect::<HashSet<_>>()
                .len(),
            entries.len()
        );
        assert!(entries.iter().all(|entry| lookup(entry.glyph).is_some()));
        assert!(
            entries
                .iter()
                .any(|entry| entry.glyph == "🇲🇲" && entry.matches("flag_mm"))
        );
    }

    #[test]
    fn completion_token_uses_character_cursor_and_replaces_the_whole_shortcode() {
        assert_eq!(completion_token("😀 :thum", 7), Some((5..10, "thum")));
        assert_eq!(
            completion_token(":thumbsup: later", 4),
            Some((0..10, "thu"))
        );
        assert_eq!(completion_token(":", 1), Some((0..1, "")));
        assert_eq!(completion_token("https://example.com", 6), None);
        assert_eq!(completion_token("12:20", 5), None);
        assert_eq!(completion_token(":thumbsup:", 10), None);
    }
    #[test]
    fn reaction_search_matches_shortcodes_and_bundled_names() {
        assert!(matches_search(
            "123456789012345678",
            "SDC_Buggy",
            "sdc_buggy"
        ));
        assert!(matches_search("👍", "thumbs up +1 like", "thumb +1"));
        assert!(!matches_search("👎", "thumbs down", "thumbs up"));
        for (_, entries) in REACTION_SECTIONS {
            for (key, name) in *entries {
                assert!(lookup(key).is_some(), "missing picker icon for {name}");
            }
        }
    }
    #[test]
    fn complete_sequences_and_new_emoji_are_bundled() {
        for emoji in ["👍🏽", "❤️", "🇺🇸", "👩‍💻", "👨‍👩‍👧‍👦", "🫠", "🩷", "✅", "🎉"]
        {
            assert!(lookup(emoji).is_some(), "{emoji}");
        }
        assert!(lookup("A").is_none());
        let mut job = egui::text::LayoutJob::default();
        let text = "Hi 👍🏽 🇺🇸 👩‍💻!";
        append(&mut job, text, egui::TextFormat::default());
        assert_eq!(job.text, text);
    }
    #[test]
    fn slack_shortcode_stays_copyable_and_collapses_only_when_its_image_is_ready() {
        let text = "Hi :boomparrot: and 7:30-4:00";
        let ready = HashSet::from(["boomparrot"]);
        assert_eq!(
            custom_spans(text, &ready)
                .into_iter()
                .map(|(range, name)| (text[range].to_owned(), name))
                .collect::<Vec<_>>(),
            vec![(":boomparrot:".into(), "boomparrot")]
        );
        assert!(custom_spans("7:30-4:00 :not-ready:", &ready).is_empty());
        let mut job = egui::text::LayoutJob::default();
        append_ready(&mut job, text, egui::TextFormat::default(), &ready);
        assert_eq!(job.text, text);
        let colon = job
            .sections
            .iter()
            .find(|section| job.text[section.byte_range.clone()].starts_with(':'))
            .unwrap();
        assert_eq!(colon.format.color, Color32::TRANSPARENT);
        assert!(colon.format.extra_letter_spacing > 0.);
    }
    #[test]
    fn atlas_contains_actual_color_pixels() {
        let image = image::load_from_memory(include_bytes!("../../assets/emoji-atlas.png"))
            .unwrap()
            .to_rgba8();
        let index = lookup("❤️").unwrap();
        let x = (index % 64) as u32 * 32;
        let y = (index / 64) as u32 * 32;
        assert!((x..x + 32).any(|px| (y..y + 32).any(|py| {
            let p = image.get_pixel(px, py);
            p[3] > 100 && p[0] > p[1].saturating_add(50)
        })));
    }
}
