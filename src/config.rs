use crate::tui::theme::{AccentColor, TitleColor};
use git2::{Config, Repository};

#[derive(Debug)]
pub enum ConfigError {
    Git2(git2::Error),
    InvalidValue(String),
}

impl std::fmt::Display for ConfigError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            ConfigError::Git2(e) => write!(f, "Git config error: {}", e),
            ConfigError::InvalidValue(s) => write!(f, "Invalid config value: {}", s),
        }
    }
}

impl std::error::Error for ConfigError {}

impl From<git2::Error> for ConfigError {
    fn from(e: git2::Error) -> Self {
        ConfigError::Git2(e)
    }
}

/// Set git user name in local repository config
pub fn set_user_name(name: &str) -> Result<(), ConfigError> {
    let repo = Repository::open(".")?;
    let mut config = repo.config()?;
    config.set_str("user.name", name)?;
    Ok(())
}

/// Set git user email in local repository config
pub fn set_user_email(email: &str) -> Result<(), ConfigError> {
    let repo = Repository::open(".")?;
    let mut config = repo.config()?;
    config.set_str("user.email", email)?;
    Ok(())
}

/// Get git user name from repository config
pub fn get_user_name() -> Result<Option<String>, ConfigError> {
    let repo = Repository::open(".")?;
    let config = repo.config()?;
    match config.get_string("user.name") {
        Ok(name) => Ok(Some(name)),
        Err(e) if e.code() == git2::ErrorCode::NotFound => Ok(None),
        Err(e) => Err(ConfigError::Git2(e)),
    }
}

/// Get git user email from repository config
pub fn get_user_email() -> Result<Option<String>, ConfigError> {
    let repo = Repository::open(".")?;
    let config = repo.config()?;
    match config.get_string("user.email") {
        Ok(email) => Ok(Some(email)),
        Err(e) if e.code() == git2::ErrorCode::NotFound => Ok(None),
        Err(e) => Err(ConfigError::Git2(e)),
    }
}

/// Set gitix theme primary accent color in local repository config
pub fn set_theme_accent(accent: AccentColor) -> Result<(), ConfigError> {
    let repo = Repository::open(".")?;
    let mut config = repo.config()?;
    let accent_str = accent_color_to_string(accent);
    config.set_str("gitix.theme.accent", &accent_str)?;
    Ok(())
}

/// Set gitix theme secondary accent color in local repository config
pub fn set_theme_accent2(accent: AccentColor) -> Result<(), ConfigError> {
    let repo = Repository::open(".")?;
    let mut config = repo.config()?;
    let accent_str = accent_color_to_string(accent);
    config.set_str("gitix.theme.accent2", &accent_str)?;
    Ok(())
}

/// Set gitix theme tertiary accent color in local repository config
pub fn set_theme_accent3(accent: AccentColor) -> Result<(), ConfigError> {
    let repo = Repository::open(".")?;
    let mut config = repo.config()?;
    let accent_str = accent_color_to_string(accent);
    config.set_str("gitix.theme.accent3", &accent_str)?;
    Ok(())
}

/// Set gitix theme title color in local repository config
pub fn set_theme_title_color(title_color: TitleColor) -> Result<(), ConfigError> {
    let repo = Repository::open(".")?;
    let mut config = repo.config()?;
    let title_str = title_color_to_string(title_color);
    config.set_str("gitix.theme.title", &title_str)?;
    Ok(())
}

/// Get gitix theme primary accent color from repository config
pub fn get_theme_accent() -> Result<Option<AccentColor>, ConfigError> {
    let repo = Repository::open(".")?;
    let config = repo.config()?;
    match config.get_string("gitix.theme.accent") {
        Ok(accent_str) => Ok(Some(string_to_accent_color(&accent_str)?)),
        Err(e) if e.code() == git2::ErrorCode::NotFound => Ok(None),
        Err(e) => Err(ConfigError::Git2(e)),
    }
}

/// Get gitix theme secondary accent color from repository config
pub fn get_theme_accent2() -> Result<Option<AccentColor>, ConfigError> {
    let repo = Repository::open(".")?;
    let config = repo.config()?;
    match config.get_string("gitix.theme.accent2") {
        Ok(accent_str) => Ok(Some(string_to_accent_color(&accent_str)?)),
        Err(e) if e.code() == git2::ErrorCode::NotFound => Ok(None),
        Err(e) => Err(ConfigError::Git2(e)),
    }
}

/// Get gitix theme tertiary accent color from repository config
pub fn get_theme_accent3() -> Result<Option<AccentColor>, ConfigError> {
    let repo = Repository::open(".")?;
    let config = repo.config()?;
    match config.get_string("gitix.theme.accent3") {
        Ok(accent_str) => Ok(Some(string_to_accent_color(&accent_str)?)),
        Err(e) if e.code() == git2::ErrorCode::NotFound => Ok(None),
        Err(e) => Err(ConfigError::Git2(e)),
    }
}

/// Get gitix theme title color from repository config
pub fn get_theme_title_color() -> Result<Option<TitleColor>, ConfigError> {
    let repo = Repository::open(".")?;
    let config = repo.config()?;
    match config.get_string("gitix.theme.title") {
        Ok(title_str) => Ok(Some(string_to_title_color(&title_str)?)),
        Err(e) if e.code() == git2::ErrorCode::NotFound => Ok(None),
        Err(e) => Err(ConfigError::Git2(e)),
    }
}

/// Set gitix pull rebase setting in local repository config
pub fn set_pull_rebase(rebase: bool) -> Result<(), ConfigError> {
    let repo = Repository::open(".")?;
    let mut config = repo.config()?;
    config.set_bool("gitix.pull.rebase", rebase)?;
    Ok(())
}

/// Get gitix pull rebase setting from repository config
pub fn get_pull_rebase() -> Result<Option<bool>, ConfigError> {
    let repo = Repository::open(".")?;
    let config = repo.config()?;
    match config.get_bool("gitix.pull.rebase") {
        Ok(rebase) => Ok(Some(rebase)),
        Err(e) if e.code() == git2::ErrorCode::NotFound => Ok(None),
        Err(e) => Err(ConfigError::Git2(e)),
    }
}

/// Convert AccentColor to string for storage
fn accent_color_to_string(accent: AccentColor) -> String {
    match accent {
        AccentColor::Rosewater => "rosewater".to_string(),
        AccentColor::Flamingo => "flamingo".to_string(),
        AccentColor::Pink => "pink".to_string(),
        AccentColor::Mauve => "mauve".to_string(),
        AccentColor::Red => "red".to_string(),
        AccentColor::Maroon => "maroon".to_string(),
        AccentColor::Peach => "peach".to_string(),
        AccentColor::Yellow => "yellow".to_string(),
        AccentColor::Green => "green".to_string(),
        AccentColor::Teal => "teal".to_string(),
        AccentColor::Sky => "sky".to_string(),
        AccentColor::Sapphire => "sapphire".to_string(),
        AccentColor::Blue => "blue".to_string(),
        AccentColor::Lavender => "lavender".to_string(),
    }
}

/// Convert string to AccentColor
fn string_to_accent_color(s: &str) -> Result<AccentColor, ConfigError> {
    match s.to_lowercase().as_str() {
        "rosewater" => Ok(AccentColor::Rosewater),
        "flamingo" => Ok(AccentColor::Flamingo),
        "pink" => Ok(AccentColor::Pink),
        "mauve" => Ok(AccentColor::Mauve),
        "red" => Ok(AccentColor::Red),
        "maroon" => Ok(AccentColor::Maroon),
        "peach" => Ok(AccentColor::Peach),
        "yellow" => Ok(AccentColor::Yellow),
        "green" => Ok(AccentColor::Green),
        "teal" => Ok(AccentColor::Teal),
        "sky" => Ok(AccentColor::Sky),
        "sapphire" => Ok(AccentColor::Sapphire),
        "blue" => Ok(AccentColor::Blue),
        "lavender" => Ok(AccentColor::Lavender),
        _ => Err(ConfigError::InvalidValue(format!(
            "Unknown accent color: {}",
            s
        ))),
    }
}

/// Convert TitleColor to string for storage
fn title_color_to_string(title_color: TitleColor) -> String {
    match title_color {
        TitleColor::Overlay0 => "overlay0".to_string(),
        TitleColor::Overlay1 => "overlay1".to_string(),
        TitleColor::Overlay2 => "overlay2".to_string(),
        TitleColor::Text => "text".to_string(),
        TitleColor::Subtext0 => "subtext0".to_string(),
        TitleColor::Subtext1 => "subtext1".to_string(),
        TitleColor::Accent(accent) => format!("accent:{}", accent_color_to_string(accent)),
    }
}

/// Convert string to TitleColor
fn string_to_title_color(s: &str) -> Result<TitleColor, ConfigError> {
    if s.starts_with("accent:") {
        let accent_str = &s[7..]; // Remove "accent:" prefix
        let accent = string_to_accent_color(accent_str)?;
        Ok(TitleColor::Accent(accent))
    } else {
        match s.to_lowercase().as_str() {
            "overlay0" => Ok(TitleColor::Overlay0),
            "overlay1" => Ok(TitleColor::Overlay1),
            "overlay2" => Ok(TitleColor::Overlay2),
            "text" => Ok(TitleColor::Text),
            "subtext0" => Ok(TitleColor::Subtext0),
            "subtext1" => Ok(TitleColor::Subtext1),
            _ => Err(ConfigError::InvalidValue(format!(
                "Unknown title color: {}",
                s
            ))),
        }
    }
}
