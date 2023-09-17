use macroquad::{
    color_u8,
    prelude::{Color, DARKGRAY, RED, WHITE},
};

pub const DARK_THEME: ColorTheme = ColorTheme {
    bg_color: color_u8!(27, 27, 27, 255),
    sp_color: WHITE,
    line_color: color_u8!(128, 128, 128, 255),
    shortcut_color: color_u8!(255, 20, 20, 125),
    node_color: WHITE,
    graph_up_color: color_u8!(0, 255, 255, 125),
    graph_down_color: color_u8!(255, 255, 0, 125),
};

pub const LIGHT_THEME: ColorTheme = ColorTheme {
    bg_color: WHITE,
    sp_color: RED,
    line_color: color_u8!(204, 204, 204, 255),
    shortcut_color: color_u8!(255, 20, 20, 125),
    node_color: DARKGRAY,
    graph_down_color: color_u8!(59, 60, 246, 255),
    graph_up_color: color_u8!(0, 125, 0, 255),
};

pub struct ColorTheme {
    pub bg_color: Color,
    pub sp_color: Color,
    pub line_color: Color,
    pub shortcut_color: Color,
    pub node_color: Color,
    pub graph_up_color: Color,
    pub graph_down_color: Color,
}

pub struct ActiveTheme {
    pub theme: ColorTheme,
    pub is_dark_theme: bool,
}

impl Default for ActiveTheme {
    fn default() -> Self {
        ActiveTheme {
            theme: DARK_THEME,
            is_dark_theme: true,
        }
    }
}

impl ActiveTheme {
    pub fn set_dark_theme(&mut self) {
        self.theme = DARK_THEME;
        self.is_dark_theme = true;
    }
    pub fn set_light_theme(&mut self) {
        self.theme = LIGHT_THEME;
        self.is_dark_theme = false;
    }

    pub fn sp_color(&self) -> Color {
        self.theme.sp_color
    }

    pub fn bg_color(&self) -> Color {
        self.theme.bg_color
    }
    pub fn line_color(&self) -> Color {
        self.theme.line_color
    }
    pub fn shortcut_color(&self) -> Color {
        self.theme.shortcut_color
    }
    pub fn node_color(&self) -> Color {
        self.theme.node_color
    }

    pub fn graph_up_color(&self) -> Color {
        self.theme.graph_up_color
    }

    pub fn graph_down_color(&self) -> Color {
        self.theme.graph_down_color
    }
}
