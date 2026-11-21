use crate::model::Emote;
use eframe::egui::{self};
use std::collections::HashSet;

fn ranges(text: &str) -> Vec<(std::ops::Range<usize>, String)> {
    let mut links = Vec::new();
    let mut offset = 0;
    while offset < text.len() {
        let Some(start) = text[offset..].char_indices().find_map(|(i, _)| {
            let rest = &text[offset + i..];
            ["https://", "http://", "www."]
                .iter()
                .any(|prefix| rest.starts_with(prefix))
                .then_some(offset + i)
        }) else {
            break;
        };
        let tail = &text[start..];
        let mut end = start
            + tail
                .find(|c: char| c.is_whitespace() || matches!(c, '<' | '>' | '"' | '|'))
                .unwrap_or(tail.len());
        while end > start {
            let candidate = &text[start..end];
            let last = candidate.chars().last().unwrap();
            let unmatched = [('(', ')'), ('[', ']'), ('{', '}')]
                .iter()
                .any(|(open, close)| {
                    last == *close
                        && candidate.matches(*close).count() > candidate.matches(*open).count()
                });
            if matches!(last, '.' | ',' | ';' | ':' | '!' | '?') || unmatched {
                end -= last.len_utf8();
            } else {
                break;
            }
        }
        let candidate = &text[start..end];
        let url = if candidate.starts_with("www.") {
            format!("https://{candidate}")
        } else {
            candidate.into()
        };
        if reqwest::Url::parse(&url).is_ok_and(|u| u.host_str().is_some()) {
            links.push((start..end, url));
        }
        offset = end.max(start + 1);
    }
    links
}

// Keep one selectable text galley, including newlines, and hit-test the link
// spans so a URL click does not also expand its containing message's thread.
pub fn text(ui: &mut egui::Ui, text: &str) -> bool {
    rich_text(ui, text, &[], |_| None)
}

// Ready Slack emotes collapse to a square and the image is painted on top.
// A shortcode whose picture has not loaded yet stays as readable text.
pub fn rich_text(
    ui: &mut egui::Ui,
    text: &str,
    emotes: &[Emote],
    mut texture: impl FnMut(&str) -> Option<egui::TextureHandle>,
) -> bool {
    let mut ready = HashSet::new();
    let mut images = Vec::new();
    for emote in emotes {
        if let Some(image) = texture(&emote.mxc) {
            ready.insert(emote.name.as_str());
            images.push((emote.name.as_str(), image));
        }
    }
    let links = ranges(text);
    let font = egui::TextStyle::Body.resolve(ui.style());
    let normal = egui::TextFormat {
        font_id: font.clone(),
        color: ui.visuals().text_color(),
        ..Default::default()
    };
    let mut job = egui::text::LayoutJob::default();
    job.wrap.max_width = ui.available_width();
    let mut offset = 0;
    for (range, _) in &links {
        super::emoji::append_ready(&mut job, &text[offset..range.start], normal.clone(), &ready);
        job.append(
            &text[range.clone()],
            0.,
            egui::TextFormat {
                font_id: font.clone(),
                color: ui.visuals().hyperlink_color,
                underline: egui::Stroke::new(1.0_f32, ui.visuals().hyperlink_color),
                ..Default::default()
            },
        );
        offset = range.end;
    }
    super::emoji::append_ready(&mut job, &text[offset..], normal, &ready);
    let galley = ui.fonts_mut(|fonts| fonts.layout_job(job.clone()));
    let response = ui.add(egui::Label::new(job).wrap().selectable(true));
    super::emoji::paint(ui, &galley, response.rect.min);
    let slots = super::emoji::custom_spans(text, &ready)
        .into_iter()
        .filter(|(range, _)| {
            !links
                .iter()
                .any(|(link, _)| link.start < range.end && range.start < link.end)
        })
        .filter_map(|(range, name)| {
            let image = images
                .iter()
                .find(|(candidate, _)| *candidate == name)?
                .1
                .clone();
            Some((text[..range.start].chars().count(), image))
        })
        .collect::<Vec<_>>();
    super::emoji::paint_custom(ui, &galley, response.rect.min, &slots);
    if let Some(pos) = response.hover_pos() {
        let local = pos - response.rect.min;
        if local.x <= galley.size().x && local.y <= galley.size().y {
            let index = galley.cursor_from_pos(local).index;
            let byte = text
                .char_indices()
                .nth(index)
                .map_or(text.len(), |(i, _)| i);
            if let Some((_, url)) = links.iter().find(|(range, _)| range.contains(&byte)) {
                if response.clicked() {
                    ui.ctx().open_url(egui::OpenUrl::new_tab(url));
                }
                response
                    .on_hover_cursor(egui::CursorIcon::PointingHand)
                    .on_hover_text(url);
                return false;
            }
        }
    }
    response.clicked()
}

pub fn open(url: impl AsRef<std::ffi::OsStr>) -> std::io::Result<()> {
    // macOS hands URLs and files to the user's default browser or viewer.
    let program = if cfg!(target_os = "macos") {
        "open"
    } else {
        "brave"
    };
    std::process::Command::new(program)
        .arg(url)
        .stdin(std::process::Stdio::null())
        .stdout(std::process::Stdio::null())
        .stderr(std::process::Stdio::null())
        .spawn()
        .map(|_| ())
}

#[cfg(test)]
mod tests {
    use super::*;
    #[test]
    fn clicking_inline_link_emits_browser_request_without_thread_click() {
        let ctx = egui::Context::default();
        let pos = egui::pos2(30., 15.);
        let mut time = 0.;
        let mut emitted = Vec::new();
        for pressed in [None, Some(true), Some(false)] {
            time += 0.1;
            let events = pressed
                .map(|pressed| {
                    vec![
                        egui::Event::PointerMoved(pos),
                        egui::Event::PointerButton {
                            pos,
                            button: egui::PointerButton::Primary,
                            pressed,
                            modifiers: Default::default(),
                        },
                    ]
                })
                .unwrap_or_default();
            let output = ctx.run(
                egui::RawInput {
                    time: Some(time),
                    events,
                    ..Default::default()
                },
                |ctx| {
                    egui::CentralPanel::default().show(ctx, |ui| {
                        assert!(!text(ui, "https://example.org/path"));
                    });
                },
            );
            emitted.extend(output.platform_output.commands);
        }
        assert!(emitted.iter().any(|command| matches!(command, egui::OutputCommand::OpenUrl(url) if url.url == "https://example.org/path")));
    }
    #[test]
    fn links_handle_wrappers_unicode_queries_and_balanced_parentheses() {
        let text = "Hi É <https://example.org/a?x=1&y=2|site> [Wiki](https://en.wikipedia.org/wiki/Foo_(bar)). www.example.org!";
        let result = ranges(text);
        assert_eq!(
            result
                .iter()
                .map(|(_, url)| url.as_str())
                .collect::<Vec<_>>(),
            vec![
                "https://example.org/a?x=1&y=2",
                "https://en.wikipedia.org/wiki/Foo_(bar)",
                "https://www.example.org"
            ]
        );
        assert!(ranges("javascript:alert(1) file:///tmp/test").is_empty());
    }
}
