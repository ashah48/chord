use eframe::egui::{self, Color32, FontId, TextStyle};
use serde::{Deserialize, Serialize};

#[derive(Clone, Copy, Debug, PartialEq, Eq, Deserialize, Serialize)]
#[serde(rename_all = "lowercase")]
pub enum DisplayMode {
    Light,
    Dark,
}

impl DisplayMode {
    pub fn current(theme: egui::Theme) -> Self {
        match theme {
            egui::Theme::Light => Self::Light,
            egui::Theme::Dark => Self::Dark,
        }
    }
}

#[derive(Clone, Copy, Debug, Default, PartialEq, Eq, Deserialize, Serialize)]
#[serde(rename_all = "lowercase")]
pub enum ColorScheme {
    #[cfg_attr(target_os = "macos", default)]
    Standard,
    #[cfg_attr(not(target_os = "macos"), default)]
    Gruvbox,
    Solarized,
    Dracula,
    Nord,
}

impl ColorScheme {
    pub const OPTIONS: [Self; 5] = [
        Self::Standard,
        Self::Gruvbox,
        Self::Solarized,
        Self::Dracula,
        Self::Nord,
    ];

    pub fn label(self) -> &'static str {
        match self {
            Self::Standard => "Standard",
            Self::Gruvbox => "Gruvbox",
            Self::Solarized => "Solarized",
            Self::Dracula => "Dracula",
            Self::Nord => "Nord",
        }
    }
}

#[derive(Clone, Copy, Debug, Default, PartialEq, Eq, Deserialize, Serialize)]
#[serde(rename_all = "lowercase")]
pub enum FontStyle {
    Terminal,
    #[default]
    MacOS,
}

#[derive(Clone, Copy, Debug, Default, PartialEq, Eq, Deserialize, Serialize)]
#[serde(default)]
pub struct Theme {
    pub mode: Option<DisplayMode>,
    pub scheme: ColorScheme,
    pub font: FontStyle,
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
    pub fn from_settings(settings: &serde_json::Value) -> Self {
        if let Ok(theme) = serde_json::from_value(settings["appearance"].clone()) {
            return theme;
        }
        match settings["theme"].as_str() {
            Some("light") => Self {
                mode: Some(DisplayMode::Light),
                scheme: ColorScheme::Standard,
                ..Self::default()
            },
            Some("dark") => Self {
                mode: Some(DisplayMode::Dark),
                scheme: ColorScheme::Standard,
                ..Self::default()
            },
            Some("gruvbox") => Self {
                mode: Some(DisplayMode::Dark),
                scheme: ColorScheme::Gruvbox,
                ..Self::default()
            },
            Some("macos") => Self {
                scheme: ColorScheme::Standard,
                ..Self::default()
            },
            _ => Self::default(),
        }
    }

    pub fn palette(self, appearance: egui::Theme) -> Palette {
        let hex = |rgb: u32| Color32::from_rgb((rgb >> 16) as u8, (rgb >> 8) as u8, rgb as u8);
        match (self.scheme, appearance) {
            (ColorScheme::Standard, egui::Theme::Light) if cfg!(target_os = "macos") => Palette {
                bg: hex(0xf5f5f7),
                panel: hex(0xffffff),
                raised: hex(0xe8e8ed),
                text: hex(0x000000),
                muted: hex(0x6e6e73),
                line: hex(0xdedee2),
                accent: hex(0x007aff),
                blue: hex(0x005ecc),
                own: hex(0xe5efff),
                select: hex(0xd6e4ff),
            },
            (ColorScheme::Standard, egui::Theme::Dark) if cfg!(target_os = "macos") => Palette {
                bg: hex(0x1e1e1e),
                panel: hex(0x28282a),
                raised: hex(0x3a3a3c),
                text: hex(0xf5f5f7),
                muted: hex(0x98989d),
                line: hex(0x38383a),
                accent: hex(0x0a84ff),
                blue: hex(0x64aaff),
                own: hex(0x1c3454),
                select: hex(0x264066),
            },
            (ColorScheme::Standard, egui::Theme::Light) => Palette {
                bg: hex(0xf5f6f8),
                panel: hex(0xffffff),
                raised: hex(0xe8ebf1),
                text: hex(0x232831),
                muted: hex(0x687182),
                line: hex(0xdbe0e9),
                accent: hex(0x4565a6),
                blue: hex(0x4565a6),
                own: hex(0xe8effa),
                select: hex(0xe8ebf1),
            },
            (ColorScheme::Standard, egui::Theme::Dark) => Palette {
                bg: hex(0x16181c),
                panel: hex(0x1d2025),
                raised: hex(0x2c3038),
                text: hex(0xe9ecf2),
                muted: hex(0x97a0b0),
                line: hex(0x353b45),
                accent: hex(0x89b4fa),
                blue: hex(0x89b4fa),
                own: hex(0x212e40),
                select: hex(0x2c3038),
            },
            (ColorScheme::Gruvbox, egui::Theme::Light) => Palette {
                bg: hex(0xfbf1c7),
                panel: hex(0xf9f5d7),
                raised: hex(0xebdbb2),
                text: hex(0x3c3836),
                muted: hex(0x665c54),
                line: hex(0xd5c4a1),
                accent: hex(0xaf3a03),
                blue: hex(0x076678),
                own: hex(0xf2e5bc),
                select: hex(0xd5c4a1),
            },
            (ColorScheme::Gruvbox, egui::Theme::Dark) => Palette {
                bg: hex(0x1d2021),
                panel: hex(0x3c3836),
                raised: hex(0x665c54),
                text: hex(0xfbf1c7),
                muted: hex(0xbdae93),
                line: hex(0x7c6f64),
                accent: hex(0xfe8019),
                blue: hex(0x89b4fa),
                own: hex(0x483424),
                select: hex(0x665c54),
            },
            (ColorScheme::Solarized, egui::Theme::Light) => Palette {
                bg: hex(0xfdf6e3),
                panel: hex(0xf7efd9),
                raised: hex(0xeee8d5),
                text: hex(0x073642),
                muted: hex(0x586e75),
                line: hex(0xe0d6b6),
                accent: hex(0x9c3b0c),
                blue: hex(0x07618d),
                own: hex(0xe5efd9),
                select: hex(0xe5e2ca),
            },
            (ColorScheme::Solarized, egui::Theme::Dark) => Palette {
                bg: hex(0x002b36),
                panel: hex(0x073642),
                raised: hex(0x154653),
                text: hex(0xeee8d5),
                muted: hex(0x93a1a1),
                line: hex(0x34545b),
                accent: hex(0xb58900),
                blue: hex(0x60b7e8),
                own: hex(0x12444a),
                select: hex(0x215361),
            },
            (ColorScheme::Dracula, egui::Theme::Light) => Palette {
                bg: hex(0xfffbeb),
                panel: hex(0xefeddc),
                raised: hex(0xece9df),
                text: hex(0x1f1f1f),
                muted: hex(0x6c664b),
                line: hex(0xdedccf),
                accent: hex(0x644ac9),
                blue: hex(0x036a96),
                own: hex(0xe8e4fa),
                select: hex(0xcfcfde),
            },
            (ColorScheme::Dracula, egui::Theme::Dark) => Palette {
                bg: hex(0x282a36),
                panel: hex(0x21222c),
                raised: hex(0x343746),
                text: hex(0xf8f8f2),
                muted: hex(0xb3b7cc),
                line: hex(0x424450),
                accent: hex(0xbd93f9),
                blue: hex(0x8be9fd),
                own: hex(0x353747),
                select: hex(0x44475a),
            },
            (ColorScheme::Nord, egui::Theme::Light) => Palette {
                bg: hex(0xeceff4),
                panel: hex(0xf5f7fa),
                raised: hex(0xe5e9f0),
                text: hex(0x2e3440),
                muted: hex(0x4c566a),
                line: hex(0xd8dee9),
                accent: hex(0x3b608c),
                blue: hex(0x3b608c),
                own: hex(0xe1edf2),
                select: hex(0xd8e4f1),
            },
            (ColorScheme::Nord, egui::Theme::Dark) => Palette {
                bg: hex(0x2e3440),
                panel: hex(0x3b4252),
                raised: hex(0x434c5e),
                text: hex(0xeceff4),
                muted: hex(0xb3becf),
                line: hex(0x4c566a),
                accent: hex(0x88c0d0),
                blue: hex(0x8fbcbb),
                own: hex(0x344554),
                select: hex(0x48566d),
            },
        }
    }

    pub fn apply(self, ctx: &egui::Context) {
        ctx.set_theme(match self.mode {
            Some(DisplayMode::Light) => egui::ThemePreference::Light,
            Some(DisplayMode::Dark) => egui::ThemePreference::Dark,
            None if cfg!(target_os = "macos") => egui::ThemePreference::System,
            None => egui::ThemePreference::Dark,
        });
        for appearance in [egui::Theme::Light, egui::Theme::Dark] {
            ctx.set_style_of(appearance, self.style(appearance));
        }
    }

    fn style(self, appearance: egui::Theme) -> egui::Style {
        let p = self.palette(appearance);
        let native = cfg!(target_os = "macos");
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
    fn display_mode_switches_each_scheme_independently_of_font() {
        let ctx = egui::Context::default();
        for scheme in ColorScheme::OPTIONS {
            for (mode, expected) in [
                (DisplayMode::Light, egui::Theme::Light),
                (DisplayMode::Dark, egui::Theme::Dark),
            ] {
                let theme = Theme {
                    mode: Some(mode),
                    scheme,
                    font: FontStyle::Terminal,
                };
                theme.apply(&ctx);
                let _ = ctx.run(
                    egui::RawInput {
                        system_theme: Some(if expected == egui::Theme::Dark {
                            egui::Theme::Light
                        } else {
                            egui::Theme::Dark
                        }),
                        ..Default::default()
                    },
                    |ctx| {
                        assert_eq!(ctx.theme(), expected);
                        assert_eq!(
                            ctx.style().visuals.text_color(),
                            theme.palette(expected).text
                        );
                    },
                );
            }
            let theme = Theme {
                scheme,
                ..Theme::default()
            };
            assert_ne!(
                theme.palette(egui::Theme::Light).bg,
                theme.palette(egui::Theme::Dark).bg
            );
        }
    }

    #[test]
    fn legacy_settings_keep_their_scheme_and_mode() {
        let theme = |name| Theme::from_settings(&serde_json::json!({"theme":name}));
        assert_eq!(theme("gruvbox").scheme, ColorScheme::Gruvbox);
        assert_eq!(theme("gruvbox").mode, Some(DisplayMode::Dark));
        assert_eq!(theme("light").mode, Some(DisplayMode::Light));
        assert_eq!(theme("dark").mode, Some(DisplayMode::Dark));
        assert_eq!(theme("macos").scheme, ColorScheme::Standard);
        assert_eq!(theme("macos").mode, None);
    }

    #[test]
    fn gruvbox_keeps_surfaces_and_accents_apart() {
        let p = Theme {
            scheme: ColorScheme::Gruvbox,
            ..Theme::default()
        }
        .palette(egui::Theme::Dark);
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
