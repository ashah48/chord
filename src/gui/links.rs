use eframe::egui::{self};

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

#[derive(Debug, PartialEq)]
struct Span {
    text: String,
    url: Option<String>,
    code: bool,
}

fn span(text: &str, url: Option<String>, code: bool) -> Span {
    Span {
        text: text.into(),
        url,
        code,
    }
}

fn plain(out: &mut Vec<Span>, text: &str) {
    let mut offset = 0;
    for (range, url) in ranges(text) {
        if range.start > offset {
            out.push(span(&text[offset..range.start], None, false));
        }
        out.push(span(&text[range.clone()], Some(url), false));
        offset = range.end;
    }
    if offset < text.len() {
        out.push(span(&text[offset..], None, false));
    }
}

fn web(url: &str) -> Option<String> {
    reqwest::Url::parse(url)
        .ok()
        .filter(|u| matches!(u.scheme(), "http" | "https") && u.host_str().is_some())
        .map(|_| url.to_owned())
}

// Slack's `<url|label>` or `<url>` at the start of `s`: (bytes consumed, label, url).
fn slack_link(s: &str) -> Option<(usize, &str, String)> {
    let rest = s.strip_prefix('<')?;
    let inner = &rest[..rest.find('>')?];
    if inner.contains(char::is_whitespace) {
        return None;
    }
    let (url, label) = inner.split_once('|').unwrap_or((inner, inner));
    Some((inner.len() + 2, label, web(url)?))
}

// Markdown's `[label](url)` at the start of `s`: (bytes consumed, label, url).
fn markdown_link(s: &str) -> Option<(usize, &str, String)> {
    let rest = s.strip_prefix('[')?;
    let close = rest.find("](")?;
    let label = &rest[..close];
    if label.is_empty() || label.contains(['[', '\n']) {
        return None;
    }
    let target = &rest[close + 2..];
    let mut depth = 0;
    for (i, c) in target.char_indices() {
        match c {
            '(' => depth += 1,
            ')' if depth == 0 => return Some((close + i + 4, label, web(&target[..i])?)),
            ')' => depth -= 1,
            c if c.is_whitespace() => return None,
            _ => {}
        }
    }
    None
}

// `code` spans (Slack-style links inside them stay clickable), Markdown and
// Slack links shown by their label, and bare URLs.
fn inline(text: &str) -> Vec<Span> {
    let mut out = Vec::new();
    let (mut start, mut i) = (0, 0);
    while i < text.len() {
        let rest = &text[i..];
        let token = if let Some(after) = rest.strip_prefix('`') {
            after
                .find('`')
                .filter(|&n| n > 0 && !after[..n].contains('\n'))
                .map(|n| {
                    let inner = &after[..n];
                    let token = match slack_link(inner) {
                        Some((len, label, url)) if len == inner.len() => {
                            span(label, Some(url), true)
                        }
                        _ => span(inner, None, true),
                    };
                    (n + 2, token)
                })
        } else {
            markdown_link(rest)
                .or_else(|| slack_link(rest))
                .map(|(len, label, url)| (len, span(label, Some(url), false)))
        };
        match token {
            Some((len, token)) => {
                plain(&mut out, &text[start..i]);
                out.push(token);
                i += len;
                start = i;
            }
            None => i += rest.chars().next().map_or(1, char::len_utf8),
        }
    }
    plain(&mut out, &text[start..]);
    out
}

// Consecutive "> " lines form one quote block; everything else is a paragraph.
fn blocks(text: &str) -> Vec<(bool, String)> {
    let mut out: Vec<(bool, String)> = Vec::new();
    for line in text.split('\n') {
        let quoted = line
            .strip_prefix("> ")
            .or_else(|| (line == ">").then_some(""));
        let (quote, line) = quoted.map_or((false, line), |rest| (true, rest));
        match out.last_mut() {
            Some((last, body)) if *last == quote => {
                body.push('\n');
                body.push_str(line);
            }
            _ => out.push((quote, line.to_owned())),
        }
    }
    out.into_iter()
        .map(|(quote, body)| (quote, body.trim_matches('\n').to_owned()))
        .filter(|(_, body)| !body.is_empty())
        .collect()
}

// A chat message body: rich inline text, with quote blocks drawn behind a bar like Slack.
pub fn message(ui: &mut egui::Ui, text: &str) -> bool {
    let blocks = blocks(text);
    if !blocks.iter().any(|(quote, _)| *quote) {
        return paragraph(ui, &inline(text));
    }
    ui.vertical(|ui| {
        ui.spacing_mut().item_spacing.y = 4.;
        let mut clicked = false;
        for (quote, body) in &blocks {
            if !quote {
                clicked |= paragraph(ui, &inline(body));
                continue;
            }
            let row = ui.horizontal(|ui| {
                ui.add_space(14.);
                ui.vertical(|ui| paragraph(ui, &inline(body))).inner
            });
            clicked |= row.inner;
            let rect = row.response.rect;
            ui.painter().rect_filled(
                egui::Rect::from_min_max(
                    rect.min + egui::vec2(2., 1.),
                    egui::pos2(rect.min.x + 6., rect.max.y - 1.),
                ),
                2.,
                ui.visuals().weak_text_color(),
            );
        }
        clicked
    })
    .inner
}

pub fn text(ui: &mut egui::Ui, text: &str) -> bool {
    let mut spans = Vec::new();
    plain(&mut spans, text);
    paragraph(ui, &spans)
}

// Keep one selectable text galley, including newlines, and hit-test the link
// spans so a URL click does not also expand its containing message's thread.
fn paragraph(ui: &mut egui::Ui, spans: &[Span]) -> bool {
    let font = egui::TextStyle::Body.resolve(ui.style());
    let link = ui.visuals().hyperlink_color;
    let normal = egui::TextFormat {
        font_id: font.clone(),
        color: ui.visuals().text_color(),
        ..Default::default()
    };
    let code = egui::TextFormat {
        font_id: egui::FontId::monospace(font.size * 0.9),
        color: if ui.visuals().dark_mode {
            egui::Color32::from_rgb(0xfb, 0x83, 0x6f)
        } else {
            egui::Color32::from_rgb(0xc0, 0x1e, 0x5a)
        },
        background: ui.visuals().code_bg_color,
        valign: egui::Align::Center,
        ..Default::default()
    };
    let mut job = egui::text::LayoutJob::default();
    job.wrap.max_width = ui.available_width();
    let mut links = Vec::new();
    let mut chars = 0;
    for span in spans {
        let start = chars;
        if span.code {
            let pad = egui::TextFormat {
                font_id: egui::FontId::monospace(font.size * 0.45),
                ..code.clone()
            };
            job.append(" ", 0., pad.clone());
            job.append(&span.text, 0., code.clone());
            job.append(" ", 0., pad);
            chars += span.text.chars().count() + 2;
        } else if span.url.is_some() {
            let labeled = !span.text.contains("://") && !span.text.starts_with("www.");
            super::emoji::append(
                &mut job,
                &span.text,
                egui::TextFormat {
                    color: link,
                    underline: if labeled {
                        egui::Stroke::NONE
                    } else {
                        egui::Stroke::new(1.0_f32, link)
                    },
                    ..normal.clone()
                },
            );
            chars += span.text.chars().count();
        } else {
            super::emoji::append(&mut job, &span.text, normal.clone());
            chars += span.text.chars().count();
        }
        if let Some(url) = &span.url {
            links.push((start..chars, url));
        }
    }
    let galley = ui.fonts_mut(|fonts| fonts.layout_job(job.clone()));
    let response = ui.add(egui::Label::new(job).wrap().selectable(true));
    super::emoji::paint(ui, &galley, response.rect.min);
    if let Some(pos) = response.hover_pos() {
        let local = pos - response.rect.min;
        if local.x <= galley.size().x && local.y <= galley.size().y {
            let index = galley.cursor_from_pos(local).index;
            if let Some((_, url)) = links.iter().find(|(range, _)| range.contains(&index)) {
                let url = url.as_str();
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

pub fn open(url: &str) -> std::io::Result<()> {
    std::process::Command::new("brave")
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
    #[test]
    fn github_push_notifications_render_like_slack() {
        let text = "[2 new commits](https://github.com/o/r/compare/a...b) pushed to `<https://github.com/o/r/tree/feat/x|feat/x>` by [ashah48](https://github.com/ashah48)\n\n> `<https://github.com/o/r/commit/76d38b50aa|76d38b50>` - Revert \"bottom k\"\n> [o/r](https://github.com/o/r)";
        let blocks = blocks(text);
        assert_eq!(
            blocks.iter().map(|(q, _)| *q).collect::<Vec<_>>(),
            vec![false, true]
        );
        let head = inline(&blocks[0].1);
        assert_eq!(
            head[0],
            span(
                "2 new commits",
                Some("https://github.com/o/r/compare/a...b".into()),
                false
            )
        );
        assert_eq!(head[1], span(" pushed to ", None, false));
        assert_eq!(
            head[2],
            span(
                "feat/x",
                Some("https://github.com/o/r/tree/feat/x".into()),
                true
            )
        );
        assert_eq!(
            head[4],
            span("ashah48", Some("https://github.com/ashah48".into()), false)
        );
        let quote = inline(&blocks[1].1);
        assert_eq!(
            quote[0],
            span(
                "76d38b50",
                Some("https://github.com/o/r/commit/76d38b50aa".into()),
                true
            )
        );
        assert_eq!(quote[1].text, " - Revert \"bottom k\"\n");
        assert_eq!(quote[2].text, "o/r");
    }
    #[test]
    fn inline_markup_falls_back_to_text_when_malformed() {
        assert_eq!(
            inline("`code` and <not a link>"),
            vec![
                span("code", None, true),
                span(" and <not a link>", None, false),
            ]
        );
        assert_eq!(
            inline("[x](javascript:alert(1)) a > b `"),
            vec![span("[x](javascript:alert(1)) a > b `", None, false)]
        );
        assert_eq!(
            inline("<https://a.org>")[0],
            span("https://a.org", Some("https://a.org".into()), false)
        );
        assert_eq!(
            blocks("a\n>b\n> c"),
            vec![(false, "a\n>b".into()), (true, "c".into())]
        );
    }
}
