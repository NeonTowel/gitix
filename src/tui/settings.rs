use crate::app::{AppState, AuthorFocus, SettingsFocus, ThemeFocus};
use crate::tui::theme::{AccentColor, Theme, TitleColor};
use ratatui::layout::{Alignment, Constraint, Direction, Layout, Margin};
use ratatui::style::{Color, Modifier, Style};
use ratatui::text::{Line, Span};
use ratatui::widgets::{Block, Borders, Clear, List, ListItem, Paragraph, Wrap};
use ratatui::{layout::Rect, Frame};

pub fn render_settings_tab(f: &mut Frame, area: Rect, state: &AppState) {
    // Create theme with current settings for live preview
    let theme = Theme::with_accents_and_title(
        state.current_theme_accent,
        state.current_theme_accent2,
        state.current_theme_accent3,
        state.current_theme_title,
    );

    // Set panel background
    f.render_widget(
        Block::default().style(theme.secondary_background_style()),
        area,
    );

    if !state.git_enabled {
        render_no_git_message(f, area, &theme);
        return;
    }

    // Split into main content and status bar
    let main_chunks = Layout::default()
        .direction(Direction::Vertical)
        .constraints([Constraint::Min(1), Constraint::Length(3)])
        .split(area);

    // Split main area into two columns: Author and Theme
    let content_chunks = Layout::default()
        .direction(Direction::Horizontal)
        .constraints([Constraint::Percentage(50), Constraint::Percentage(50)])
        .margin(1)
        .split(main_chunks[0]);

    // Render Author panel
    render_author_panel(f, content_chunks[0], state, &theme);

    // Render Theme panel
    render_theme_panel(f, content_chunks[1], state, &theme);

    // Render status bar
    render_status_bar(f, main_chunks[1], state, &theme);
}

fn render_author_panel(f: &mut Frame, area: Rect, state: &AppState, theme: &Theme) {
    let is_focused = state.settings_focus == SettingsFocus::Author;

    let border_style = if is_focused {
        theme.focused_border_style()
    } else {
        theme.border_style()
    };

    let block = Block::default()
        .borders(Borders::ALL)
        .title("👤 Author Configuration")
        .title_style(theme.title_style())
        .border_style(border_style)
        .style(theme.secondary_background_style());

    let inner_area = block.inner(area);
    f.render_widget(block, area);

    // Split into name and email sections
    let author_chunks = Layout::default()
        .direction(Direction::Vertical)
        .constraints([
            Constraint::Length(3), // Name field
            Constraint::Length(1), // Spacing
            Constraint::Length(3), // Email field
            Constraint::Min(1),    // Help text
        ])
        .margin(1)
        .split(inner_area);

    // Name field
    let name_focused = is_focused && state.settings_author_focus == AuthorFocus::Name;
    let name_style = if name_focused {
        Style::default()
            .fg(theme.accent())
            .add_modifier(Modifier::BOLD)
    } else {
        theme.text_style()
    };

    let name_block = Block::default()
        .borders(Borders::ALL)
        .title("Name")
        .title_style(if name_focused {
            theme.accent_style()
        } else {
            theme.secondary_text_style()
        })
        .border_style(if name_focused {
            theme.focused_border_style()
        } else {
            theme.border_style()
        })
        .style(theme.secondary_background_style());

    f.render_widget(name_block, author_chunks[0]);

    // Render name input content
    let name_inner = author_chunks[0].inner(Margin {
        vertical: 1,
        horizontal: 1,
    });
    if name_focused {
        f.render_widget(state.user_name_input.widget(), name_inner);
    } else {
        // Always show the actual content, not placeholder when unfocused
        let name_text = if state.user_name_input.lines()[0].is_empty() {
            Span::styled("(not set)", theme.muted_text_style())
        } else {
            Span::styled(&state.user_name_input.lines()[0], name_style)
        };
        let name_paragraph = Paragraph::new(name_text);
        f.render_widget(name_paragraph, name_inner);
    }

    // Email field
    let email_focused = is_focused && state.settings_author_focus == AuthorFocus::Email;
    let email_style = if email_focused {
        Style::default()
            .fg(theme.accent())
            .add_modifier(Modifier::BOLD)
    } else {
        theme.text_style()
    };

    let email_block = Block::default()
        .borders(Borders::ALL)
        .title("Email")
        .title_style(if email_focused {
            theme.accent_style()
        } else {
            theme.secondary_text_style()
        })
        .border_style(if email_focused {
            theme.focused_border_style()
        } else {
            theme.border_style()
        })
        .style(theme.secondary_background_style());

    f.render_widget(email_block, author_chunks[2]);

    // Render email input content
    let email_inner = author_chunks[2].inner(Margin {
        vertical: 1,
        horizontal: 1,
    });
    if email_focused {
        f.render_widget(state.user_email_input.widget(), email_inner);
    } else {
        // Always show the actual content, not placeholder when unfocused
        let email_text = if state.user_email_input.lines()[0].is_empty() {
            Span::styled("(not set)", theme.muted_text_style())
        } else {
            Span::styled(&state.user_email_input.lines()[0], email_style)
        };
        let email_paragraph = Paragraph::new(email_text);
        f.render_widget(email_paragraph, email_inner);
    }

    // Help text
    let help_lines = vec![
        Line::from(vec![
            Span::styled("💡 ", Style::default().fg(theme.sky)),
            Span::styled(
                "These settings configure your Git identity",
                theme.secondary_text_style(),
            ),
        ]),
        Line::from(""),
        Line::from(vec![
            Span::styled("• ", theme.secondary_text_style()),
            Span::styled("Name: ", theme.stats_label_style()),
            Span::styled("Used for commit authorship", theme.secondary_text_style()),
        ]),
        Line::from(vec![
            Span::styled("• ", theme.secondary_text_style()),
            Span::styled("Email: ", theme.stats_label_style()),
            Span::styled("Associated with your commits", theme.secondary_text_style()),
        ]),
        Line::from(""),
        Line::from(vec![
            Span::styled("💾 ", Style::default().fg(theme.yellow)),
            Span::styled(
                "Press Ctrl+S to save changes to git config",
                theme.muted_text_style(),
            ),
        ]),
    ];

    let help_paragraph = Paragraph::new(help_lines).wrap(Wrap { trim: false });
    f.render_widget(help_paragraph, author_chunks[3]);
}

fn render_theme_panel(f: &mut Frame, area: Rect, state: &AppState, theme: &Theme) {
    let is_focused = state.settings_focus == SettingsFocus::Theme;

    let border_style = if is_focused {
        theme.focused_border_style()
    } else {
        theme.border_style()
    };

    let block = Block::default()
        .borders(Borders::ALL)
        .title("🎨 Theme Configuration")
        .title_style(theme.title_style())
        .border_style(border_style)
        .style(theme.secondary_background_style());

    let inner_area = block.inner(area);
    f.render_widget(block, area);

    // Create theme options list
    let theme_options = vec![
        create_theme_option(
            "Primary Accent",
            state.current_theme_accent,
            state.settings_theme_focus == ThemeFocus::Accent && is_focused,
            theme,
        ),
        create_theme_option(
            "Secondary Accent",
            state.current_theme_accent2,
            state.settings_theme_focus == ThemeFocus::Accent2 && is_focused,
            theme,
        ),
        create_theme_option(
            "Tertiary Accent",
            state.current_theme_accent3,
            state.settings_theme_focus == ThemeFocus::Accent3 && is_focused,
            theme,
        ),
        create_title_option(
            "Title Color",
            state.current_theme_title,
            state.settings_theme_focus == ThemeFocus::Title && is_focused,
            theme,
        ),
    ];

    // Split into options and preview
    let theme_chunks = Layout::default()
        .direction(Direction::Vertical)
        .constraints([
            Constraint::Length(theme_options.len() as u16 + 2), // Options list
            Constraint::Min(1),                                 // Preview and help
        ])
        .margin(1)
        .split(inner_area);

    // Render theme options
    let options_list = List::new(theme_options).block(
        Block::default()
            .borders(Borders::ALL)
            .title("Theme Options")
            .title_style(theme.secondary_text_style())
            .border_style(theme.border_style()),
    );
    f.render_widget(options_list, theme_chunks[0]);

    // Render preview and help
    render_theme_preview(f, theme_chunks[1], state, theme);
}

fn create_theme_option<'a>(
    label: &'a str,
    accent: AccentColor,
    is_selected: bool,
    theme: &'a Theme,
) -> ListItem<'a> {
    let color_name = format!("{:?}", accent);
    let color = get_accent_color(accent, theme);

    let style = if is_selected {
        Style::default()
            .fg(theme.accent())
            .add_modifier(Modifier::BOLD)
    } else {
        theme.text_style()
    };

    let line = Line::from(vec![
        Span::styled(if is_selected { "▶ " } else { "  " }, style),
        Span::styled(format!("{}: ", label), theme.stats_label_style()),
        Span::styled("●", Style::default().fg(color).add_modifier(Modifier::BOLD)),
        Span::styled(format!(" {}", color_name), style),
    ]);

    ListItem::new(line)
}

fn create_title_option<'a>(
    label: &'a str,
    title_color: TitleColor,
    is_selected: bool,
    theme: &'a Theme,
) -> ListItem<'a> {
    let color_name = match title_color {
        TitleColor::Overlay0 => "Overlay0".to_string(),
        TitleColor::Overlay1 => "Overlay1".to_string(),
        TitleColor::Overlay2 => "Overlay2".to_string(),
        TitleColor::Text => "Text".to_string(),
        TitleColor::Subtext0 => "Subtext0".to_string(),
        TitleColor::Subtext1 => "Subtext1".to_string(),
        TitleColor::Accent(accent) => format!("Accent({})", format!("{:?}", accent)),
    };

    let color = title_color.get_color(theme);

    let style = if is_selected {
        Style::default()
            .fg(theme.accent())
            .add_modifier(Modifier::BOLD)
    } else {
        theme.text_style()
    };

    let line = Line::from(vec![
        Span::styled(if is_selected { "▶ " } else { "  " }, style),
        Span::styled(format!("{}: ", label), theme.stats_label_style()),
        Span::styled("●", Style::default().fg(color).add_modifier(Modifier::BOLD)),
        Span::styled(format!(" {}", color_name), style),
    ]);

    ListItem::new(line)
}

fn get_accent_color(accent: AccentColor, theme: &Theme) -> Color {
    match accent {
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
    }
}

fn render_theme_preview(f: &mut Frame, area: Rect, state: &AppState, theme: &Theme) {
    let preview_chunks = Layout::default()
        .direction(Direction::Vertical)
        .constraints([
            Constraint::Length(8), // Preview
            Constraint::Min(1),    // Help
        ])
        .split(area);

    // Theme preview
    let preview_block = Block::default()
        .borders(Borders::ALL)
        .title("Live Preview")
        .title_style(theme.title_style())
        .border_style(theme.border_style())
        .style(theme.secondary_background_style());

    let preview_inner = preview_block.inner(preview_chunks[0]);
    f.render_widget(preview_block, preview_chunks[0]);

    let preview_lines = vec![
        Line::from(vec![
            Span::styled("Primary: ", theme.stats_label_style()),
            Span::styled(
                "●",
                Style::default()
                    .fg(theme.accent())
                    .add_modifier(Modifier::BOLD),
            ),
            Span::styled("  Secondary: ", theme.stats_label_style()),
            Span::styled(
                "●",
                Style::default()
                    .fg(theme.accent2())
                    .add_modifier(Modifier::BOLD),
            ),
            Span::styled("  Tertiary: ", theme.stats_label_style()),
            Span::styled(
                "●",
                Style::default()
                    .fg(theme.accent3())
                    .add_modifier(Modifier::BOLD),
            ),
        ]),
        Line::from(""),
        Line::from(vec![
            Span::styled("Title Color: ", theme.stats_label_style()),
            Span::styled(
                "Sample Title",
                Style::default()
                    .fg(theme.title_color())
                    .add_modifier(Modifier::BOLD),
            ),
        ]),
        Line::from(""),
        Line::from(vec![
            Span::styled("✓ ", theme.success_style()),
            Span::styled("Success  ", theme.success_style()),
            Span::styled("⚠ ", theme.warning_style()),
            Span::styled("Warning  ", theme.warning_style()),
            Span::styled("✗ ", theme.error_style()),
            Span::styled("Error", theme.error_style()),
        ]),
    ];

    let preview_paragraph = Paragraph::new(preview_lines);
    f.render_widget(preview_paragraph, preview_inner);

    // Help text
    let help_lines = vec![
        Line::from(vec![
            Span::styled("🎨 ", Style::default().fg(theme.pink)),
            Span::styled("Catppuccin Macchiato Theme", theme.secondary_text_style()),
        ]),
        Line::from(""),
        Line::from(vec![
            Span::styled("• ", theme.secondary_text_style()),
            Span::styled("Primary: ", theme.stats_label_style()),
            Span::styled(
                "Active elements, focus indicators",
                theme.secondary_text_style(),
            ),
        ]),
        Line::from(vec![
            Span::styled("• ", theme.secondary_text_style()),
            Span::styled("Secondary: ", theme.stats_label_style()),
            Span::styled("Labels, authors, metadata", theme.secondary_text_style()),
        ]),
        Line::from(vec![
            Span::styled("• ", theme.secondary_text_style()),
            Span::styled("Tertiary: ", theme.stats_label_style()),
            Span::styled("Timestamps, subtle accents", theme.secondary_text_style()),
        ]),
        Line::from(""),
        Line::from(vec![
            Span::styled("💾 ", Style::default().fg(theme.green)),
            Span::styled(
                "Saved to git config as gitix.theme.*",
                theme.muted_text_style(),
            ),
        ]),
    ];

    let help_paragraph = Paragraph::new(help_lines).wrap(Wrap { trim: false });
    f.render_widget(help_paragraph, preview_chunks[1]);
}

fn render_status_bar(f: &mut Frame, area: Rect, state: &AppState, theme: &Theme) {
    let status_text = if let Some(ref message) = state.settings_status_message {
        message.clone()
    } else {
        match state.settings_focus {
            SettingsFocus::Author => match state.settings_author_focus {
                AuthorFocus::Name => {
                    "Type to edit name • ↑/↓: Switch field • Ctrl+←/→: Switch panel • Ctrl+S: Save"
                        .to_string()
                }
                AuthorFocus::Email => {
                    "Type to edit email • ↑/↓: Switch field • Ctrl+←/→: Switch panel • Ctrl+S: Save"
                        .to_string()
                }
            },
            SettingsFocus::Theme => match state.settings_theme_focus {
                ThemeFocus::Accent => {
                    "←/→: Change primary accent • ↑/↓: Switch option • Ctrl+←/→: Switch panel • Ctrl+S: Save"
                        .to_string()
                }
                ThemeFocus::Accent2 => {
                    "←/→: Change secondary accent • ↑/↓: Switch option • Ctrl+←/→: Switch panel • Ctrl+S: Save"
                        .to_string()
                }
                ThemeFocus::Accent3 => {
                    "←/→: Change tertiary accent • ↑/↓: Switch option • Ctrl+←/→: Switch panel • Ctrl+S: Save"
                        .to_string()
                }
                ThemeFocus::Title => {
                    "←/→: Change title color • ↑/↓: Switch option • Ctrl+←/→: Switch panel • Ctrl+S: Save".to_string()
                }
            },
        }
    };

    let status_paragraph = Paragraph::new(status_text)
        .style(theme.secondary_text_style())
        .alignment(Alignment::Center)
        .block(
            Block::default()
                .borders(Borders::ALL)
                .title("Controls")
                .title_style(theme.title_style())
                .border_style(theme.border_style())
                .style(theme.secondary_background_style()),
        );
    f.render_widget(status_paragraph, area);
}

fn render_no_git_message(f: &mut Frame, area: Rect, theme: &Theme) {
    let message = Paragraph::new(vec![
        Line::from(""),
        Line::from(Span::styled(
            "⚠️  Not a Git Repository",
            Style::default()
                .fg(theme.yellow)
                .add_modifier(Modifier::BOLD),
        )),
        Line::from(""),
        Line::from("Settings require a Git repository to store configuration."),
        Line::from("Initialize a repository first to access settings."),
        Line::from(""),
        Line::from(Span::styled(
            "💡 Tip:",
            Style::default().fg(theme.sky).add_modifier(Modifier::BOLD),
        )),
        Line::from("Use the Overview tab to initialize a new repository."),
    ])
    .alignment(Alignment::Center)
    .style(theme.text_style())
    .block(
        Block::default()
            .borders(Borders::ALL)
            .title("Settings")
            .title_style(theme.title_style())
            .border_style(theme.border_style())
            .style(theme.secondary_background_style()),
    );
    f.render_widget(message, area);
}
