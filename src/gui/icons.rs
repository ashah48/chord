use eframe::egui::{self, Color32, Pos2, Rect, Stroke, Vec2};

/// Official marks for the chat services, filled with their brand colors.
fn brand_svg(source: usize) -> Option<&'static str> {
    Some(match source {
        1 => include_str!("../../assets/brands/slack.svg"),
        2 => include_str!("../../assets/brands/discord.svg"),
        3 => include_str!("../../assets/brands/whatsapp.svg"),
        4 => include_str!("../../assets/brands/imessage.svg"),
        _ => return None,
    })
}

/// Rasterize a brand mark at the exact pixel size it is shown, once per size.
fn brand_texture(ctx: &egui::Context, source: usize, px: u32) -> Option<egui::TextureHandle> {
    let id = egui::Id::new(("brand-mark", source, px));
    if let Some(texture) = ctx.data(|d| d.get_temp::<egui::TextureHandle>(id)) {
        return Some(texture);
    }
    let tree = resvg::usvg::Tree::from_str(brand_svg(source)?, &Default::default()).ok()?;
    let mut pixmap = resvg::tiny_skia::Pixmap::new(px, px)?;
    let scale = px as f32 / tree.size().width().max(tree.size().height());
    resvg::render(
        &tree,
        resvg::tiny_skia::Transform::from_scale(scale, scale),
        &mut pixmap.as_mut(),
    );
    let image = egui::ColorImage::from_rgba_premultiplied([px as usize; 2], pixmap.data());
    let texture = ctx.load_texture(
        format!("brand-mark-{source}-{px}"),
        image,
        egui::TextureOptions::LINEAR,
    );
    ctx.data_mut(|d| d.insert_temp(id, texture.clone()));
    Some(texture)
}

/// Draw a service's real mark. Returns false for services without one.
fn brand_mark(p: &egui::Painter, rect: Rect, source: usize) -> bool {
    let px = (rect.width().max(rect.height()) * p.ctx().pixels_per_point()).round() as u32;
    let Some(texture) = brand_texture(p.ctx(), source, px.clamp(8, 512)) else {
        return false;
    };
    let side = rect.width().min(rect.height());
    p.image(
        texture.id(),
        Rect::from_center_size(rect.center(), Vec2::splat(side)),
        Rect::from_min_max(Pos2::ZERO, Pos2::new(1., 1.)),
        Color32::WHITE,
    );
    true
}

// Brand marks for chat services; vector drawings for the rest, which need no assets.
pub fn service(p: &egui::Painter, rect: Rect, source: usize, ink: Color32) {
    if brand_mark(p, rect, source) {
        return;
    }
    let pt = |x: f32, y: f32| {
        Pos2::new(
            rect.left() + x * rect.width(),
            rect.top() + y * rect.height(),
        )
    };
    let line = |a: (f32, f32), b: (f32, f32), color: Color32, width: f32| {
        p.line_segment([pt(a.0, a.1), pt(b.0, b.1)], Stroke::new(width, color));
    };
    match source {
        5 | 6 => {
            let c = if source == 6 {
                Color32::from_rgb(224, 91, 85)
            } else {
                Color32::from_rgb(230, 174, 82)
            };
            p.rect_stroke(
                Rect::from_two_pos(pt(0.07, 0.2), pt(0.93, 0.82)),
                3.,
                Stroke::new(2.0_f32, c),
                egui::StrokeKind::Inside,
            );
            line((0.08, 0.23), (0.5, 0.55), c, 2.);
            line((0.5, 0.55), (0.92, 0.23), c, 2.);
        }
        _ => {
            p.rect_stroke(
                Rect::from_two_pos(pt(0.1, 0.22), pt(0.9, 0.85)),
                3.,
                Stroke::new(2.0_f32, ink),
                egui::StrokeKind::Inside,
            );
            line((0.1, 0.53), (0.35, 0.53), ink, 2.);
            line((0.35, 0.53), (0.43, 0.65), ink, 2.);
            line((0.43, 0.65), (0.57, 0.65), ink, 2.);
            line((0.57, 0.65), (0.65, 0.53), ink, 2.);
            line((0.65, 0.53), (0.9, 0.53), ink, 2.);
        }
    }
}

pub fn button(
    ui: &mut egui::Ui,
    source: usize,
    label: &str,
    compact: bool,
    selected: bool,
    p: super::Palette,
) -> egui::Response {
    let (rect, response) =
        ui.allocate_exact_size(Vec2::new(ui.available_width(), 38.), egui::Sense::click());
    if selected || response.hovered() {
        ui.painter().rect_filled(
            rect,
            9.,
            if selected {
                super::theme::tint(p.panel, super::theme::service_color(source), 0.18)
            } else {
                p.panel
            },
        );
    }
    let origin = if compact {
        rect.center() - Vec2::splat(12.)
    } else {
        egui::pos2(rect.left() + 10., rect.center().y - 12.)
    };
    service(
        ui.painter(),
        Rect::from_min_size(origin, Vec2::splat(24.)),
        source,
        if selected {
            super::theme::service_color(source)
        } else {
            p.text
        },
    );
    if !compact {
        ui.painter().text(
            egui::pos2(rect.left() + 44., rect.center().y),
            egui::Align2::LEFT_CENTER,
            label,
            egui::FontId::proportional(14.),
            if selected {
                super::theme::service_color(source)
            } else {
                p.text
            },
        );
    }
    response.widget_info(|| {
        egui::WidgetInfo::selected(egui::WidgetType::Button, ui.is_enabled(), selected, label)
    });
    response.on_hover_text(label)
}

/// The generic contact silhouette, for people with no photo and no letters in their name.
pub fn person(p: &egui::Painter, center: Pos2, radius: f32) {
    p.circle_filled(center, radius, Color32::from_rgb(160, 168, 204));
    let white = Color32::WHITE;
    p.circle_filled(center - Vec2::new(0., radius * 0.2), radius * 0.36, white);
    // Shoulders: a large circle below, cut off by the avatar's edge. Both shapes are
    // convex, so their overlap is too: keep each outline's points that fall inside the other.
    let shoulders = center + Vec2::new(0., radius * 0.98);
    let shoulder_radius = radius * 0.74;
    let ring = |c: Pos2, r: f32| {
        (0..96).map(move |i| {
            let a = i as f32 / 96. * std::f32::consts::TAU;
            c + Vec2::new(a.cos(), a.sin()) * r
        })
    };
    let mut points: Vec<Pos2> = ring(shoulders, shoulder_radius)
        .filter(|q| q.distance(center) <= radius)
        .chain(ring(center, radius).filter(|q| q.distance(shoulders) <= shoulder_radius))
        .collect();
    let middle = center + Vec2::new(0., radius * 0.6);
    points.sort_by(|a, b| {
        let angle = |q: &Pos2| (q.y - middle.y).atan2(q.x - middle.x);
        angle(a).total_cmp(&angle(b))
    });
    if points.len() >= 3 {
        p.add(egui::Shape::convex_polygon(points, white, Stroke::NONE));
    }
}

/// A pushpin tilted 45°, head up and right, like the pin in macOS lists.
pub fn pin(p: &egui::Painter, center: Pos2, size: f32, color: Color32) {
    // Drawn upright in a unit box (needle down), then rotated about its middle.
    let (sin, cos) = std::f32::consts::FRAC_PI_4.sin_cos();
    let at = |x: f32, y: f32| {
        let (x, y) = (x * size, (y - 0.5) * size);
        center + Vec2::new(x * cos - y * sin, x * sin + y * cos)
    };
    let fill = |points: &[(f32, f32)]| {
        p.add(egui::Shape::convex_polygon(
            points.iter().map(|&(x, y)| at(x, y)).collect(),
            color,
            Stroke::NONE,
        ));
    };
    fill(&[(-0.22, 0.0), (0.22, 0.0), (0.22, 0.13), (-0.22, 0.13)]);
    fill(&[(-0.13, 0.13), (0.13, 0.13), (0.13, 0.42), (-0.13, 0.42)]);
    fill(&[(-0.13, 0.42), (0.13, 0.42), (0.3, 0.6), (-0.3, 0.6)]);
    p.line_segment([at(0., 0.6), at(0., 1.)], Stroke::new(1.3_f32, color));
}

/// Paint navigation arrows directly so they do not depend on font fallback.
pub fn back_button(ui: &mut egui::Ui, label: &str) -> egui::Response {
    let response = ui.button(format!("    {label}"));
    if ui.is_rect_visible(response.rect) {
        let color = ui.style().interact(&response).text_color();
        let tip = egui::pos2(response.rect.left() + 8., response.rect.center().y);
        let stroke = Stroke::new(1.5_f32, color);
        ui.painter()
            .line_segment([tip, tip + Vec2::new(9., 0.)], stroke);
        ui.painter().add(egui::Shape::line(
            vec![tip + Vec2::new(4., -4.), tip, tip + Vec2::new(4., 4.)],
            stroke,
        ));
    }
    response
}

pub enum GalleryIcon {
    Download,
    External,
    Close,
    Previous,
    Next,
}

pub fn gallery_button(ui: &mut egui::Ui, icon: GalleryIcon, label: &str) -> egui::Response {
    let (rect, response) = ui.allocate_exact_size(Vec2::splat(36.), egui::Sense::click());
    let ink = if ui.is_enabled() {
        Color32::from_gray(225)
    } else {
        Color32::from_gray(90)
    };
    if response.hovered() {
        ui.painter()
            .rect_filled(rect, 6., Color32::from_white_alpha(22));
    }
    let r = Rect::from_center_size(rect.center(), Vec2::splat(20.));
    let pt = |x: f32, y: f32| r.min + Vec2::new(x * r.width(), y * r.height());
    let line = |a: (f32, f32), b: (f32, f32)| {
        ui.painter()
            .line_segment([pt(a.0, a.1), pt(b.0, b.1)], Stroke::new(1.8_f32, ink));
    };
    match icon {
        GalleryIcon::Close => {
            line((0.2, 0.2), (0.8, 0.8));
            line((0.8, 0.2), (0.2, 0.8));
        }
        GalleryIcon::Download => {
            line((0.5, 0.05), (0.5, 0.65));
            line((0.25, 0.4), (0.5, 0.65));
            line((0.75, 0.4), (0.5, 0.65));
            line((0.1, 0.65), (0.1, 0.9));
            line((0.1, 0.9), (0.9, 0.9));
            line((0.9, 0.9), (0.9, 0.65));
        }
        GalleryIcon::External => {
            line((0.5, 0.1), (0.9, 0.1));
            line((0.9, 0.1), (0.9, 0.5));
            line((0.9, 0.1), (0.4, 0.6));
            line((0.35, 0.2), (0.1, 0.2));
            line((0.1, 0.2), (0.1, 0.9));
            line((0.1, 0.9), (0.8, 0.9));
            line((0.8, 0.9), (0.8, 0.65));
        }
        GalleryIcon::Previous => {
            line((0.65, 0.15), (0.3, 0.5));
            line((0.3, 0.5), (0.65, 0.85));
        }
        GalleryIcon::Next => {
            line((0.35, 0.15), (0.7, 0.5));
            line((0.7, 0.5), (0.35, 0.85));
        }
    }
    response
        .on_hover_text(label)
        .on_hover_cursor(egui::CursorIcon::PointingHand)
}

#[cfg(test)]
mod tests {
    #[test]
    fn brand_marks_render_in_color() {
        let ctx = eframe::egui::Context::default();
        for source in 1..=4 {
            let texture = super::brand_texture(&ctx, source, 48).expect("brand mark renders");
            assert_eq!(texture.size(), [48, 48]);
        }
        assert!(super::brand_texture(&ctx, 5, 48).is_none());
    }
}
