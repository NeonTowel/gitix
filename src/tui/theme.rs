use catppuccin::PALETTE;
use ratatui::style::{Color, Modifier, Style};

/// Available accent colors for the theme
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum AccentColor {
    Rosewater,
    Flamingo,
    Pink,
    Mauve,
    Red,
    Maroon,
    Peach,
    Yellow,
    Green,
    Teal,
    Sky,
    Sapphire,
    Blue,
    Lavender,
}

impl Default for AccentColor {
    fn default() -> Self {
        AccentColor::Blue
    }
}

/// Available title colors for panel headers
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum TitleColor {
    Overlay0,
    Overlay1,
    Overlay2,
    Text,
    Subtext0,
    Subtext1,
    // Allow accent colors for titles too
    Accent(AccentColor),
}

impl Default for TitleColor {
    fn default() -> Self {
        TitleColor::Overlay0
    }
}

impl TitleColor {
    /// Get the actual color from the theme
    pub fn get_color(&self, theme: &Theme) -> Color {
        match self {
            TitleColor::Overlay0 => theme.overlay0,
            TitleColor::Overlay1 => theme.overlay1,
            TitleColor::Overlay2 => theme.overlay2,
            TitleColor::Text => theme.text,
            TitleColor::Subtext0 => theme.subtext0,
            TitleColor::Subtext1 => theme.subtext1,
            TitleColor::Accent(accent) => match accent {
                AccentColor::Rosewater => theme.rosewater,
                AccentColor::Flamingo => theme.flamingo,
                AccentColor::Pink => theme.pink,
                AccentColor::Mauve => theme.mauve,
                AccentColor::Red => theme.red,
                AccentColor::Maroon => theme.maroon,
                AccentColor::Peach => theme.peach,
                AccentColor::Yellow => theme.yellow,
                AccentColor::Green => theme.green,
                AccentColor::Teal => theme.teal,
                AccentColor::Sky => theme.sky,
                AccentColor::Sapphire => theme.sapphire,
                AccentColor::Blue => theme.blue,
                AccentColor::Lavender => theme.lavender,
            },
        }
    }
}

/// Catppuccin Macchiato theme colors for the TUI
pub struct Theme {
    // Base colors (semantic usage)
    pub base: Color,   // Tab bar and status bar backgrounds
    pub mantle: Color, // Unified background for terminal and all content
    pub crust: Color,  // Deepest accent color (not used for backgrounds)

    // Surface colors (for layered UI elements)
    pub surface0: Color,
    pub surface1: Color,
    pub surface2: Color,

    // Overlay colors (for subtle elements)
    pub overlay0: Color,
    pub overlay1: Color,
    pub overlay2: Color,

    // Text colors (always use these for text)
    pub text: Color,     // Primary text
    pub subtext0: Color, // Muted text
    pub subtext1: Color, // Secondary text

    // All accent colors available
    pub rosewater: Color,
    pub flamingo: Color,
    pub pink: Color,
    pub mauve: Color,
    pub red: Color,
    pub maroon: Color,
    pub peach: Color,
    pub yellow: Color,
    pub green: Color,
    pub teal: Color,
    pub sky: Color,
    pub sapphire: Color,
    pub blue: Color,
    pub lavender: Color,

    // Configurable accent colors
    accent_color: AccentColor,  // Primary accent (titles, focus, etc.)
    accent2_color: AccentColor, // Secondary accent (labels, etc.)
    accent3_color: AccentColor, // Tertiary accent (timestamps, etc.)

    // Configurable title color for panel headers
    title_color: TitleColor, // Color for all panel headers/titles
}

impl Theme {
    /// Create a new Catppuccin Macchiato theme with default accent colors
    pub fn new() -> Self {
        Self::with_accents(
            AccentColor::Blue,      // Primary accent: blue
            AccentColor::Rosewater, // Secondary accent: rosewater
            AccentColor::Pink,      // Tertiary accent: pink (changed from mauve)
        )
    }

    /// Create a new Catppuccin Macchiato theme with specified accent color (legacy method)
    pub fn with_accent(accent_color: AccentColor) -> Self {
        Self::with_accents(accent_color, AccentColor::Rosewater, AccentColor::Mauve)
    }

    /// Create a new Catppuccin Macchiato theme with specified accent colors
    pub fn with_accents(
        accent_color: AccentColor,
        accent2_color: AccentColor,
        accent3_color: AccentColor,
    ) -> Self {
        Self::with_accents_and_title(
            accent_color,
            accent2_color,
            accent3_color,
            TitleColor::Overlay0,
        )
    }

    /// Create a new Catppuccin Macchiato theme with specified accent colors and title color
    pub fn with_accents_and_title(
        accent_color: AccentColor,
        accent2_color: AccentColor,
        accent3_color: AccentColor,
        title_color: TitleColor,
    ) -> Self {
        let macchiato = &PALETTE.macchiato.colors;

        Self {
            // Base colors (semantic usage per updated guidelines)
            base: macchiato.base.into(), // Tab bar and status bar backgrounds
            mantle: macchiato.mantle.into(), // Unified background for terminal and all content
            crust: macchiato.crust.into(), // Deepest accent color

            // Surface colors
            surface0: macchiato.surface0.into(),
            surface1: macchiato.surface1.into(),
            surface2: macchiato.surface2.into(),

            // Overlay colors
            overlay0: macchiato.overlay0.into(),
            overlay1: macchiato.overlay1.into(),
            overlay2: macchiato.overlay2.into(),

            // Text colors
            text: macchiato.text.into(),
            subtext0: macchiato.subtext0.into(),
            subtext1: macchiato.subtext1.into(),

            // All accent colors
            rosewater: macchiato.rosewater.into(),
            flamingo: macchiato.flamingo.into(),
            pink: macchiato.pink.into(),
            mauve: macchiato.mauve.into(),
            red: macchiato.red.into(),
            maroon: macchiato.maroon.into(),
            peach: macchiato.peach.into(),
            yellow: macchiato.yellow.into(),
            green: macchiato.green.into(),
            teal: macchiato.teal.into(),
            sky: macchiato.sky.into(),
            sapphire: macchiato.sapphire.into(),
            blue: macchiato.blue.into(),
            lavender: macchiato.lavender.into(),

            accent_color,
            accent2_color,
            accent3_color,

            title_color,
        }
    }

    /// Get the primary accent color
    pub fn accent(&self) -> Color {
        match self.accent_color {
            AccentColor::Rosewater => self.rosewater,
            AccentColor::Flamingo => self.flamingo,
            AccentColor::Pink => self.pink,
            AccentColor::Mauve => self.mauve,
            AccentColor::Red => self.red,
            AccentColor::Maroon => self.maroon,
            AccentColor::Peach => self.peach,
            AccentColor::Yellow => self.yellow,
            AccentColor::Green => self.green,
            AccentColor::Teal => self.teal,
            AccentColor::Sky => self.sky,
            AccentColor::Sapphire => self.sapphire,
            AccentColor::Blue => self.blue,
            AccentColor::Lavender => self.lavender,
        }
    }

    /// Get the secondary accent color
    pub fn accent2(&self) -> Color {
        match self.accent2_color {
            AccentColor::Rosewater => self.rosewater,
            AccentColor::Flamingo => self.flamingo,
            AccentColor::Pink => self.pink,
            AccentColor::Mauve => self.mauve,
            AccentColor::Red => self.red,
            AccentColor::Maroon => self.maroon,
            AccentColor::Peach => self.peach,
            AccentColor::Yellow => self.yellow,
            AccentColor::Green => self.green,
            AccentColor::Teal => self.teal,
            AccentColor::Sky => self.sky,
            AccentColor::Sapphire => self.sapphire,
            AccentColor::Blue => self.blue,
            AccentColor::Lavender => self.lavender,
        }
    }

    /// Get the tertiary accent color
    pub fn accent3(&self) -> Color {
        match self.accent3_color {
            AccentColor::Rosewater => self.rosewater,
            AccentColor::Flamingo => self.flamingo,
            AccentColor::Pink => self.pink,
            AccentColor::Mauve => self.mauve,
            AccentColor::Red => self.red,
            AccentColor::Maroon => self.maroon,
            AccentColor::Peach => self.peach,
            AccentColor::Yellow => self.yellow,
            AccentColor::Green => self.green,
            AccentColor::Teal => self.teal,
            AccentColor::Sky => self.sky,
            AccentColor::Sapphire => self.sapphire,
            AccentColor::Blue => self.blue,
            AccentColor::Lavender => self.lavender,
        }
    }

    /// Get the title color
    pub fn title_color(&self) -> Color {
        self.title_color.get_color(self)
    }

    /// Change the primary accent color
    pub fn set_accent(&mut self, accent_color: AccentColor) {
        self.accent_color = accent_color;
    }

    /// Change the secondary accent color
    pub fn set_accent2(&mut self, accent2_color: AccentColor) {
        self.accent2_color = accent2_color;
    }

    /// Change the tertiary accent color
    pub fn set_accent3(&mut self, accent3_color: AccentColor) {
        self.accent3_color = accent3_color;
    }

    /// Change the title color
    pub fn set_title_color(&mut self, title_color: TitleColor) {
        self.title_color = title_color;
    }

    /// Change all accent colors and title color at once
    pub fn set_accents_and_title(
        &mut self,
        accent_color: AccentColor,
        accent2_color: AccentColor,
        accent3_color: AccentColor,
        title_color: TitleColor,
    ) {
        self.accent_color = accent_color;
        self.accent2_color = accent2_color;
        self.accent3_color = accent3_color;
        self.title_color = title_color;
    }

    /// Change all accent colors at once
    pub fn set_accents(
        &mut self,
        accent_color: AccentColor,
        accent2_color: AccentColor,
        accent3_color: AccentColor,
    ) {
        self.accent_color = accent_color;
        self.accent2_color = accent2_color;
        self.accent3_color = accent3_color;
    }

    // === CONVENIENCE CONSTRUCTORS ===

    /// Create a theme with rosewater accent
    pub fn rosewater() -> Self {
        Self::with_accent(AccentColor::Rosewater)
    }

    /// Create a theme with flamingo accent
    pub fn flamingo() -> Self {
        Self::with_accent(AccentColor::Flamingo)
    }

    /// Create a theme with pink accent
    pub fn pink() -> Self {
        Self::with_accent(AccentColor::Pink)
    }

    /// Create a theme with mauve accent
    pub fn mauve() -> Self {
        Self::with_accent(AccentColor::Mauve)
    }

    /// Create a theme with red accent
    pub fn red() -> Self {
        Self::with_accent(AccentColor::Red)
    }

    /// Create a theme with maroon accent
    pub fn maroon() -> Self {
        Self::with_accent(AccentColor::Maroon)
    }

    /// Create a theme with peach accent
    pub fn peach() -> Self {
        Self::with_accent(AccentColor::Peach)
    }

    /// Create a theme with yellow accent
    pub fn yellow() -> Self {
        Self::with_accent(AccentColor::Yellow)
    }

    /// Create a theme with green accent
    pub fn green() -> Self {
        Self::with_accent(AccentColor::Green)
    }

    /// Create a theme with teal accent
    pub fn teal() -> Self {
        Self::with_accent(AccentColor::Teal)
    }

    /// Create a theme with sky accent
    pub fn sky() -> Self {
        Self::with_accent(AccentColor::Sky)
    }

    /// Create a theme with sapphire accent
    pub fn sapphire() -> Self {
        Self::with_accent(AccentColor::Sapphire)
    }

    /// Create a theme with blue accent (default)
    pub fn blue() -> Self {
        Self::with_accent(AccentColor::Blue)
    }

    /// Create a theme with lavender accent
    pub fn lavender() -> Self {
        Self::with_accent(AccentColor::Lavender)
    }

    // === TITLE COLOR CONVENIENCE METHODS ===

    /// Create a theme with overlay0 title color (default)
    pub fn with_overlay0_titles() -> Self {
        Self::with_accents_and_title(
            AccentColor::Blue,      // Primary accent: blue
            AccentColor::Rosewater, // Secondary accent: rosewater
            AccentColor::Pink,      // Tertiary accent: pink (changed from mauve)
            TitleColor::Overlay0,
        )
    }

    /// Create a theme with overlay1 title color
    pub fn with_overlay1_titles() -> Self {
        Self::with_accents_and_title(
            AccentColor::Blue,      // Primary accent: blue
            AccentColor::Rosewater, // Secondary accent: rosewater
            AccentColor::Pink,      // Tertiary accent: pink (changed from mauve)
            TitleColor::Overlay1,
        )
    }

    /// Create a theme with overlay2 title color
    pub fn with_overlay2_titles() -> Self {
        Self::with_accents_and_title(
            AccentColor::Blue,      // Primary accent: blue
            AccentColor::Rosewater, // Secondary accent: rosewater
            AccentColor::Pink,      // Tertiary accent: pink (changed from mauve)
            TitleColor::Overlay2,
        )
    }

    /// Create a theme with text color titles
    pub fn with_text_titles() -> Self {
        Self::with_accents_and_title(
            AccentColor::Blue,      // Primary accent: blue
            AccentColor::Rosewater, // Secondary accent: rosewater
            AccentColor::Pink,      // Tertiary accent: pink (changed from mauve)
            TitleColor::Text,
        )
    }

    /// Create a theme with accent color titles
    pub fn with_accent_titles(accent: AccentColor) -> Self {
        Self::with_accents_and_title(
            AccentColor::Blue,      // Primary accent: blue
            AccentColor::Rosewater, // Secondary accent: rosewater
            AccentColor::Pink,      // Tertiary accent: pink (changed from mauve)
            TitleColor::Accent(accent),
        )
    }

    // === SEMANTIC STYLES ===

    /// Main background style (mantle color) - unified terminal background
    pub fn main_background_style(&self) -> Style {
        Style::default().bg(self.mantle)
    }

    /// Secondary background style (mantle color) - unified background for all content areas
    pub fn secondary_background_style(&self) -> Style {
        Style::default().bg(self.mantle)
    }

    /// Deep background style (mantle color) - unified background
    pub fn deep_background_style(&self) -> Style {
        Style::default().bg(self.mantle)
    }

    /// Default text style (always use text color)
    pub fn text_style(&self) -> Style {
        Style::default().fg(self.text)
    }

    /// Secondary text style
    pub fn secondary_text_style(&self) -> Style {
        Style::default().fg(self.subtext1)
    }

    /// Muted text style
    pub fn muted_text_style(&self) -> Style {
        Style::default().fg(self.subtext0)
    }

    /// Highlighted/active elements (uses accent color)
    pub fn highlight_style(&self) -> Style {
        Style::default()
            .fg(self.accent())
            .add_modifier(Modifier::BOLD)
    }

    /// Active selection style (uses accent color)
    pub fn active_style(&self) -> Style {
        Style::default().fg(self.accent())
    }

    /// Focus indicator style (uses accent color)
    pub fn focus_style(&self) -> Style {
        Style::default()
            .fg(self.accent())
            .add_modifier(Modifier::BOLD)
    }

    /// Regular borders (surface0 color for subtle distinction)
    pub fn border_style(&self) -> Style {
        Style::default().fg(self.surface0)
    }

    /// Focused/active borders (accent color)
    pub fn focused_border_style(&self) -> Style {
        Style::default().fg(self.accent())
    }

    /// Panel titles (configurable title color)
    pub fn title_style(&self) -> Style {
        Style::default()
            .fg(self.title_color())
            .add_modifier(Modifier::BOLD)
    }

    /// Status bar style (mantle background for unified look)
    pub fn status_bar_style(&self) -> Style {
        Style::default().bg(self.mantle).fg(self.surface0)
    }

    // === SEMANTIC COLORS FOR SPECIFIC PURPOSES ===

    /// Success indicators (always green)
    pub fn success_style(&self) -> Style {
        Style::default().fg(self.green)
    }

    /// Warning indicators (always yellow)
    pub fn warning_style(&self) -> Style {
        Style::default().fg(self.yellow)
    }

    /// Error indicators (always red)
    pub fn error_style(&self) -> Style {
        Style::default().fg(self.red)
    }

    /// Info indicators (always sky)
    pub fn info_style(&self) -> Style {
        Style::default().fg(self.sky)
    }

    // === GIT-SPECIFIC STYLES ===

    /// Commit authors (accent2 for consistency)
    pub fn author_style(&self) -> Style {
        Style::default().fg(self.accent2())
    }

    /// Timestamps (accent3 for distinction)
    pub fn timestamp_style(&self) -> Style {
        Style::default().fg(self.accent3())
    }

    /// Commit messages (main text)
    pub fn commit_message_style(&self) -> Style {
        Style::default().fg(self.text)
    }

    /// Statistics labels (accent2 for distinction)
    pub fn stats_label_style(&self) -> Style {
        Style::default()
            .fg(self.accent2())
            .add_modifier(Modifier::BOLD)
    }

    /// Statistics values (green for positive data)
    pub fn stats_value_style(&self) -> Style {
        Style::default().fg(self.green)
    }

    // === ACCENT-SPECIFIC STYLES ===

    /// Primary accent style (for titles, focus, active elements)
    pub fn accent_style(&self) -> Style {
        Style::default().fg(self.accent())
    }

    /// Secondary accent style (for labels, secondary highlights)
    pub fn accent2_style(&self) -> Style {
        Style::default().fg(self.accent2())
    }

    /// Tertiary accent style (for timestamps, metadata)
    pub fn accent3_style(&self) -> Style {
        Style::default().fg(self.accent3())
    }

    /// Secondary accent with bold (for emphasized labels)
    pub fn accent2_bold_style(&self) -> Style {
        Style::default()
            .fg(self.accent2())
            .add_modifier(Modifier::BOLD)
    }

    /// Tertiary accent with bold (for emphasized metadata)
    pub fn accent3_bold_style(&self) -> Style {
        Style::default()
            .fg(self.accent3())
            .add_modifier(Modifier::BOLD)
    }

    // === PANEL-SPECIFIC STYLES ===

    /// Tab panel background (base for distinction from content)
    pub fn tab_panel_style(&self) -> Style {
        Style::default().bg(self.base)
    }

    /// Active tab style (accent color with bold)
    pub fn active_tab_style(&self) -> Style {
        Style::default()
            .fg(self.accent())
            .add_modifier(Modifier::BOLD)
    }

    /// Inactive tab style (accent color without bold)
    pub fn inactive_tab_style(&self) -> Style {
        Style::default().fg(self.accent())
    }

    /// Disabled tab style (muted)
    pub fn disabled_tab_style(&self) -> Style {
        Style::default().fg(self.subtext0)
    }

    // === POPUP-SPECIFIC STYLES ===

    /// Popup background style (surface0 for depth)
    pub fn popup_background_style(&self) -> Style {
        Style::default().bg(self.surface0)
    }

    /// Popup border style (mauve for importance)
    pub fn popup_border_style(&self) -> Style {
        Style::default().fg(self.mauve)
    }

    /// Popup title style (lavender with bold)
    pub fn popup_title_style(&self) -> Style {
        Style::default()
            .fg(self.lavender)
            .add_modifier(Modifier::BOLD)
    }

    /// Popup button style (blue background with base text)
    pub fn popup_button_style(&self) -> Style {
        Style::default()
            .fg(self.base)
            .bg(self.blue)
            .add_modifier(Modifier::BOLD)
    }

    /// Popup button border style (blue to match button)
    pub fn popup_button_border_style(&self) -> Style {
        Style::default().fg(self.blue)
    }
}

impl Default for Theme {
    fn default() -> Self {
        Self::new()
    }
}
