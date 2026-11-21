use eframe::egui::{self, Color32, FontId, TextStyle};
#[derive(Clone, Copy, Default, PartialEq, Eq)]
pub enum Theme {
    #[cfg_attr(not(target_os = "macos"), default)]
    Gruvbox,
    Dark,
    Light,
    #[cfg_attr(target_os = "macos", default)]
    MacOs,
}
#[derive(Clone, Copy)]
pub struct Palette {
    pub bg: Color32,
    pub panel: Color32,
    pub raised: Color32,
    pub text: Color32,
    pub muted: Color32,
    pub line: Color32,
    pub accent: Color32,
    pub blue: Color32,
    pub own: Color32,
    pub select: Color32,
}
impl Theme {
    pub const OPTIONS: [Self; 4] = [Self::Gruvbox, Self::Dark, Self::Light, Self::MacOs];

    pub fn palette(self, appearance: egui::Theme) -> Palette {
        let rgb = |r, g, b| Color32::from_rgb(r, g, b);
        match self {
            Self::Gruvbox => Palette {
                bg: rgb(29, 32, 33),
                panel: rgb(60, 56, 54),
                raised: rgb(102, 92, 84),
                text: rgb(251, 241, 199),
                muted: rgb(189, 174, 147),
                line: rgb(124, 111, 100),
                accent: rgb(254, 128, 25),
                blue: rgb(137, 180, 250),
                own: rgb(72, 52, 36),
                select: rgb(102, 92, 84),
            },
            Self::Dark => Palette {
                bg: rgb(22, 24, 28),
                panel: rgb(29, 32, 37),
                raised: rgb(44, 48, 56),
                text: rgb(233, 236, 242),
                muted: rgb(151, 160, 176),
                line: rgb(53, 59, 69),
                accent: rgb(137, 180, 250),
                blue: rgb(137, 180, 250),
                own: rgb(33, 46, 64),
                select: rgb(44, 48, 56),
            },
            Self::Light => Palette {
                bg: rgb(245, 246, 248),
                panel: Color32::WHITE,
                raised: rgb(232, 235, 241),
                text: rgb(35, 40, 49),
                muted: rgb(104, 113, 130),
                line: rgb(219, 224, 233),
                accent: rgb(69, 101, 166),
                blue: rgb(69, 101, 166),
                own: rgb(232, 239, 250),
                select: rgb(232, 235, 241),
            },
            Self::MacOs if appearance == egui::Theme::Light => Palette {
                bg: rgb(245, 245, 247),
                panel: Color32::WHITE,
                raised: rgb(232, 232, 237),
                text: Color32::BLACK,
                muted: rgb(110, 110, 115),
                line: rgb(222, 222, 226),
                accent: rgb(0, 122, 255),
                blue: rgb(0, 94, 204),
                own: rgb(229, 239, 255),
                select: rgb(214, 228, 255),
            },
            Self::MacOs => Palette {
                bg: rgb(30, 30, 30),
                panel: rgb(40, 40, 42),
                raised: rgb(58, 58, 60),
                text: rgb(245, 245, 247),
                muted: rgb(152, 152, 157),
                line: rgb(56, 56, 58),
                accent: rgb(10, 132, 255),
                blue: rgb(100, 170, 255),
                own: rgb(28, 52, 84),
                select: rgb(38, 64, 102),
            },
        }
    }
    pub fn apply(self, ctx: &egui::Context) {
        let preference = match self {
            Self::MacOs => egui::ThemePreference::System,
            Self::Light => egui::ThemePreference::Light,
            Self::Gruvbox | Self::Dark => egui::ThemePreference::Dark,
        };
        ctx.set_theme(preference);
        for system in [egui::Theme::Light, egui::Theme::Dark] {
            let appearance = match preference {
                egui::ThemePreference::Light => egui::Theme::Light,
                egui::ThemePreference::Dark => egui::Theme::Dark,
                egui::ThemePreference::System => system,
            };
            ctx.set_style_of(system, self.style(appearance));
        }
    }

    fn style(self, appearance: egui::Theme) -> egui::Style {
        let p = self.palette(appearance);
        let native = self == Self::MacOs;
        let mut style = egui::Style {
            visuals: match appearance {
                egui::Theme::Light => egui::Visuals::light(),
                egui::Theme::Dark => egui::Visuals::dark(),
            },
            ..Default::default()
        };
        let visuals = &mut style.visuals;
        visuals.override_text_color = Some(p.text);
        visuals.panel_fill = p.bg;
        visuals.window_fill = p.panel;
        visuals.extreme_bg_color = p.bg;
        visuals.faint_bg_color = p.panel;
        visuals.selection.bg_fill = p.raised;
        visuals.selection.stroke = egui::Stroke::new(1.0_f32, p.accent);
        visuals.hyperlink_color = p.blue;
        visuals.widgets.noninteractive.bg_fill = p.panel;
        visuals.widgets.noninteractive.bg_stroke = egui::Stroke::new(1.0_f32, p.line);
        visuals.widgets.inactive.bg_fill = p.raised;
        visuals.widgets.inactive.weak_bg_fill = p.raised;
        visuals.widgets.inactive.bg_stroke = egui::Stroke::NONE;
        visuals.widgets.hovered.bg_fill = p.raised;
        visuals.widgets.hovered.weak_bg_fill = p.raised;
        visuals.widgets.hovered.bg_stroke = egui::Stroke::new(1.0_f32, p.accent);
        visuals.window_corner_radius = 12.into();
        if native {
            visuals.extreme_bg_color = p.panel;
            let border = egui::Stroke::new(0.5_f32, p.line);
            let hover = tint(p.raised, p.line, 0.5);
            visuals.widgets.hovered.bg_fill = hover;
            visuals.widgets.hovered.weak_bg_fill = hover;
            visuals.widgets.inactive.bg_stroke = border;
            visuals.widgets.hovered.bg_stroke = border;
            visuals.widgets.active.bg_stroke = border;
            for widget in [
                &mut visuals.widgets.noninteractive,
                &mut visuals.widgets.inactive,
                &mut visuals.widgets.hovered,
                &mut visuals.widgets.active,
                &mut visuals.widgets.open,
            ] {
                widget.corner_radius = 6.into();
            }
        }
        style.spacing.item_spacing = egui::vec2(8., if native { 7. } else { 8. });
        style.spacing.button_padding = if native {
            egui::vec2(10., 5.)
        } else {
            egui::vec2(11., 7.)
        };
        style.spacing.window_margin = egui::Margin::same(20);
        for (kind, size) in [
            (TextStyle::Body, if native { 14. } else { 15. }),
            (TextStyle::Button, if native { 13. } else { 14. }),
            (TextStyle::Small, if native { 11.5 } else { 12. }),
            (TextStyle::Heading, if native { 21. } else { 23. }),
        ] {
            style.text_styles.insert(kind, FontId::proportional(size));
        }
        style
    }

    pub fn from_setting(value: Option<&str>) -> Self {
        Self::OPTIONS
            .into_iter()
            .find(|theme| Some(theme.setting()) == value)
            .unwrap_or_default()
    }

    pub fn setting(self) -> &'static str {
        match self {
            Self::Gruvbox => "gruvbox",
            Self::Dark => "dark",
            Self::Light => "light",
            Self::MacOs => "macos",
        }
    }

    pub fn name(self) -> &'static str {
        match self {
            Self::Gruvbox => "Gruvbox dark",
            Self::Dark => "Dark",
            Self::Light => "Light",
            Self::MacOs => "macOS",
        }
    }
}

pub fn service_color(source: usize) -> Color32 {
    let (r, g, b) = match source {
        1 => (192, 139, 229),
        2 => (130, 148, 255),
        3 => (85, 199, 131),
        4 => (85, 195, 216),
        5 => (239, 180, 96),
        6 => (240, 133, 143),
        _ => (91, 202, 177),
    };
    Color32::from_rgb(r, g, b)
}

/// Sidebar color for one connected account. Slack workspaces are separate
/// accounts: `slack:T…` picks a stable color that stays off the other networks.
pub fn account_color(account_key: &str) -> Color32 {
    if let Some(team) = account_key.strip_prefix("slack:")
        && !team.is_empty()
    {
        return slack_workspace_color(team);
    }
    let index = crate::model::SOURCES
        .iter()
        .position(|(id, _, _)| *id == account_key)
        .unwrap_or(0);
    service_color(index)
}

fn slack_workspace_color(team: &str) -> Color32 {
    // Kept away from Discord blue, WhatsApp green, iMessage cyan, and both mails.
    const PALETTE: &[(u8, u8, u8)] = &[
        (176, 98, 214),
        (214, 112, 72),
        (214, 86, 140),
        (120, 112, 196),
        (196, 92, 72),
        (156, 108, 188),
    ];
    let hash = team.bytes().fold(0_u32, |hash, byte| {
        hash.wrapping_mul(31).wrapping_add(u32::from(byte))
    });
    let (r, g, b) = PALETTE[(hash as usize) % PALETTE.len()];
    Color32::from_rgb(r, g, b)
}
pub fn avatar_color(name: &str) -> Color32 {
    let palette = [
        (90, 155, 208),
        (157, 117, 211),
        (211, 116, 137),
        (199, 149, 73),
        (72, 167, 145),
        (90, 160, 170),
        (190, 118, 84),
        (111, 148, 207),
    ];
    let hash = name.bytes().fold(0_usize, |hash, byte| {
        hash.wrapping_mul(31).wrapping_add(byte as usize)
    });
    let (r, g, b) = palette[hash % palette.len()];
    Color32::from_rgb(r, g, b)
}
pub fn tint(base: Color32, accent: Color32, amount: f32) -> Color32 {
    let mix = |a: u8, b: u8| (a as f32 * (1. - amount) + b as f32 * amount) as u8;
    Color32::from_rgb(
        mix(base.r(), accent.r()),
        mix(base.g(), accent.g()),
        mix(base.b(), accent.b()),
    )
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn macos_follows_system_appearance_and_explicit_themes_keep_their_style() {
        let ctx = egui::Context::default();
        let expected_default = if cfg!(target_os = "macos") {
            Theme::MacOs
        } else {
            Theme::Gruvbox
        };
        assert!(Theme::from_setting(None) == expected_default);
        for theme in Theme::OPTIONS {
            assert!(Theme::from_setting(Some(theme.setting())) == theme);
            theme.apply(&ctx);
            for system in [egui::Theme::Light, egui::Theme::Dark] {
                let _ = ctx.run(
                    egui::RawInput {
                        system_theme: Some(system),
                        ..Default::default()
                    },
                    |ctx| {
                        let expected = match theme {
                            Theme::MacOs => system,
                            Theme::Light => egui::Theme::Light,
                            Theme::Gruvbox | Theme::Dark => egui::Theme::Dark,
                        };
                        assert_eq!(ctx.theme(), expected);
                        assert_eq!(
                            ctx.style().visuals.text_color(),
                            theme.palette(expected).text
                        );
                        assert_eq!(
                            TextStyle::Body.resolve(&ctx.style()).size,
                            if theme == Theme::MacOs { 14. } else { 15. }
                        );
                    },
                );
            }
        }
    }

    #[test]
    fn gruvbox_keeps_surfaces_and_accents_apart() {
        let p = Theme::Gruvbox.palette(egui::Theme::Dark);
        let dist = |a: Color32, b: Color32| {
            u16::from(a.r().abs_diff(b.r()))
                + u16::from(a.g().abs_diff(b.g()))
                + u16::from(a.b().abs_diff(b.b()))
        };
        assert!(dist(p.bg, p.panel) >= 60, "background and panels");
        assert!(dist(p.panel, p.raised) >= 40, "panels and raised controls");
        assert!(dist(p.bg, p.own) >= 40, "thread and your messages");
        assert!(dist(p.text, p.muted) >= 80, "primary and secondary text");
        assert!(dist(p.panel, p.line) >= 80, "panels and separators");
        assert!(dist(p.accent, p.blue) >= 150, "accent and links");
    }
    #[test]
    fn each_account_has_its_own_color_and_slack_workspaces_differ() {
        let lab = account_color("slack:TEMMBGZU3");
        let course = account_color("slack:TSAU3JLKZ");
        assert_ne!(lab, course);
        for key in [
            "slack",
            "discord",
            "whatsapp",
            "iphone",
            "mail:home",
            "mail:cmu",
        ] {
            let color = account_color(key);
            assert_ne!(lab, color, "{key}");
            assert_ne!(course, color, "{key}");
        }
        assert_eq!(account_color("slack"), service_color(1));
        assert_eq!(account_color("discord"), service_color(2));
        assert_eq!(account_color("mail:home"), service_color(5));
        assert_eq!(account_color("mail:cmu"), service_color(6));
        assert_ne!(account_color("mail:home"), account_color("mail:cmu"));
    }
}
