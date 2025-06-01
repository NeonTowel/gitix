use crate::tui::theme::{AccentColor, TitleColor};
use ratatui::widgets::ScrollbarState;
use ratatui::widgets::TableState;
use std::path::PathBuf;
use tui_textarea::TextArea;

pub struct AppState {
    pub git_enabled: bool,          // Is this a git repo?
    pub show_init_prompt: bool,     // Should we prompt to init?
    pub repo_root: Option<PathBuf>, // Path to repo root if found
    pub root_dir: PathBuf,          // The directory jail root
    pub current_dir: PathBuf,       // The directory currently being browsed
    pub files_selected_row: usize,  // Selected row in files tab

    // Save changes tab state
    pub save_changes_table_state: TableState, // Table state for save changes file list
    pub staged_files: Vec<PathBuf>,           // Files staged for commit
    pub commit_message: TextArea<'static>,    // Commit message input
    pub save_changes_focus: SaveChangesFocus, // Which part of the save changes UI has focus
    pub show_commit_help: bool,               // Whether to show commit message help popup
    pub help_popup_scroll: usize,             // Scroll position for help popup
    pub help_popup_scrollbar_state: ScrollbarState, // Scrollbar state for help popup
    pub show_template_popup: bool,            // Whether to show template selection popup
    pub template_popup_selection: TemplatePopupSelection, // Which button is selected in template popup

    // Settings tab state
    pub settings_focus: SettingsFocus, // Which settings section has focus
    pub settings_author_focus: AuthorFocus, // Which author field has focus
    pub settings_theme_focus: ThemeFocus, // Which theme setting has focus
    pub settings_git_focus: GitFocus,  // Which git setting has focus
    pub user_name_input: TextArea<'static>, // User name input field
    pub user_email_input: TextArea<'static>, // User email input field
    pub current_theme_accent: AccentColor, // Current primary accent color
    pub current_theme_accent2: AccentColor, // Current secondary accent color
    pub current_theme_accent3: AccentColor, // Current tertiary accent color
    pub current_theme_title: TitleColor, // Current title color
    pub settings_status_message: Option<String>, // Status message for settings operations

    // Git configuration
    pub pull_rebase: bool, // Whether to use rebase when pulling (gitix.pull.rebase)

    // Git status caching for save changes tab
    pub save_changes_git_status: Vec<crate::git::GitFileStatus>, // Cached git status for save changes tab
    pub save_changes_git_status_loaded: bool, // Whether git status has been loaded for save changes tab

    // Git status caching for files tab (reused from old status tab)
    pub status_git_status: Vec<crate::git::GitFileStatus>, // Cached git status for files tab
    pub status_git_status_loaded: bool, // Whether git status has been loaded for files tab

    // Update tab state
    pub update_remote_status: Option<crate::git::RemoteStatus>, // Cached remote status
    pub update_recent_operations: Vec<crate::git::SyncOperation>, // Recent sync operations

    // Error popup state
    pub show_error_popup: bool,      // Whether to show error popup
    pub error_popup_title: String,   // Title of the error popup
    pub error_popup_message: String, // Error message to display
}

#[derive(Debug, Clone, PartialEq)]
pub enum SaveChangesFocus {
    FileList,
    CommitMessage,
}

#[derive(Debug, Clone, PartialEq)]
pub enum TemplatePopupSelection {
    Yes,
    No,
}

#[derive(Debug, Clone, PartialEq)]
pub enum SettingsFocus {
    Author,
    Theme,
    Git,
}

#[derive(Debug, Clone, PartialEq)]
pub enum AuthorFocus {
    Name,
    Email,
}

#[derive(Debug, Clone, PartialEq)]
pub enum ThemeFocus {
    Accent,
    Accent2,
    Accent3,
    Title,
}

#[derive(Debug, Clone, PartialEq)]
pub enum GitFocus {
    PullRebase,
}

impl Default for AppState {
    fn default() -> Self {
        let cwd = std::env::current_dir().unwrap_or_else(|_| PathBuf::from("."));
        let mut state = AppState {
            git_enabled: false,
            show_init_prompt: false,
            repo_root: None,
            root_dir: cwd.clone(),
            current_dir: cwd,
            files_selected_row: 0,
            save_changes_table_state: TableState::default(),
            staged_files: Vec::new(),
            commit_message: TextArea::new(vec![String::new()]),
            save_changes_focus: SaveChangesFocus::CommitMessage,
            show_commit_help: false,
            help_popup_scroll: 0,
            help_popup_scrollbar_state: ScrollbarState::default(),
            show_template_popup: false,
            template_popup_selection: TemplatePopupSelection::No,

            // Settings state
            settings_focus: SettingsFocus::Author,
            settings_author_focus: AuthorFocus::Name,
            settings_theme_focus: ThemeFocus::Accent,
            settings_git_focus: GitFocus::PullRebase,
            user_name_input: TextArea::new(vec![String::new()]),
            user_email_input: TextArea::new(vec![String::new()]),
            current_theme_accent: AccentColor::Blue,
            current_theme_accent2: AccentColor::Rosewater,
            current_theme_accent3: AccentColor::Pink,
            current_theme_title: TitleColor::Overlay0,
            settings_status_message: None,

            // Git configuration
            pull_rebase: true, // Default to rebase

            save_changes_git_status: Vec::new(),
            save_changes_git_status_loaded: false,
            status_git_status: Vec::new(),
            status_git_status_loaded: false,

            // Update tab state
            update_remote_status: None,
            update_recent_operations: Vec::new(),

            // Error popup state
            show_error_popup: false,
            error_popup_title: String::new(),
            error_popup_message: String::new(),
        };
        state.check_git_status();
        state.load_settings();
        state
    }
}

impl AppState {
    pub fn check_git_status(&mut self) {
        match gix::discover(&self.current_dir) {
            Ok(repo) => {
                self.git_enabled = true;
                self.show_init_prompt = false;
                self.repo_root = Some(repo.path().to_path_buf());
            }
            Err(_) => {
                self.git_enabled = false;
                self.show_init_prompt = true;
                self.repo_root = None;
            }
        }
    }

    /// Load settings from git config
    pub fn load_settings(&mut self) {
        if !self.git_enabled {
            return;
        }

        // Load user name and email
        if let Ok(Some(name)) = crate::config::get_user_name() {
            self.user_name_input = TextArea::new(vec![name]);
        }
        if let Ok(Some(email)) = crate::config::get_user_email() {
            self.user_email_input = TextArea::new(vec![email]);
        }

        // Load theme settings
        if let Ok(Some(accent)) = crate::config::get_theme_accent() {
            self.current_theme_accent = accent;
        }
        if let Ok(Some(accent2)) = crate::config::get_theme_accent2() {
            self.current_theme_accent2 = accent2;
        }
        if let Ok(Some(accent3)) = crate::config::get_theme_accent3() {
            self.current_theme_accent3 = accent3;
        }
        if let Ok(Some(title)) = crate::config::get_theme_title_color() {
            self.current_theme_title = title;
        }

        // Load git configuration
        if let Ok(Some(pull_rebase)) = crate::config::get_pull_rebase() {
            self.pull_rebase = pull_rebase;
        }
    }

    /// Save current settings to git config
    pub fn save_settings(&mut self) -> Result<(), String> {
        if !self.git_enabled {
            return Err("Not in a git repository".to_string());
        }

        // Save user name and email
        let name = self.user_name_input.lines()[0].clone();
        let email = self.user_email_input.lines()[0].clone();

        if !name.is_empty() {
            if let Err(e) = crate::config::set_user_name(&name) {
                return Err(format!("Failed to save user name: {}", e));
            }
        }

        if !email.is_empty() {
            if let Err(e) = crate::config::set_user_email(&email) {
                return Err(format!("Failed to save user email: {}", e));
            }
        }

        // Save theme settings
        if let Err(e) = crate::config::set_theme_accent(self.current_theme_accent) {
            return Err(format!("Failed to save theme accent: {}", e));
        }
        if let Err(e) = crate::config::set_theme_accent2(self.current_theme_accent2) {
            return Err(format!("Failed to save theme accent2: {}", e));
        }
        if let Err(e) = crate::config::set_theme_accent3(self.current_theme_accent3) {
            return Err(format!("Failed to save theme accent3: {}", e));
        }
        if let Err(e) = crate::config::set_theme_title_color(self.current_theme_title) {
            return Err(format!("Failed to save theme title color: {}", e));
        }

        // Save git configuration
        if let Err(e) = crate::config::set_pull_rebase(self.pull_rebase) {
            return Err(format!("Failed to save pull rebase setting: {}", e));
        }

        Ok(())
    }

    pub fn try_init_repo(&mut self) -> Result<(), gix::init::Error> {
        match gix::init(&self.current_dir) {
            Ok(repo) => {
                self.git_enabled = true;
                self.show_init_prompt = false;
                self.repo_root = Some(repo.path().to_path_buf());
                Ok(())
            }
            Err(e) => Err(e),
        }
    }

    pub fn decline_init_repo(&mut self) {
        self.git_enabled = false;
        self.show_init_prompt = false;
        self.repo_root = None;
    }

    pub fn toggle_commit_help(&mut self) {
        self.show_commit_help = !self.show_commit_help;
        // Reset scroll position when opening help
        if self.show_commit_help {
            self.help_popup_scroll = 0;
            self.help_popup_scrollbar_state = ScrollbarState::default();
        }
    }

    pub fn toggle_template_popup(&mut self) {
        self.show_template_popup = !self.show_template_popup;
        // Reset selection to Yes when opening (default to positive action)
        if self.show_template_popup {
            self.template_popup_selection = TemplatePopupSelection::Yes;
        }
    }

    pub fn template_popup_navigate_left(&mut self) {
        self.template_popup_selection = TemplatePopupSelection::Yes;
    }

    pub fn template_popup_navigate_right(&mut self) {
        self.template_popup_selection = TemplatePopupSelection::No;
    }

    pub fn apply_template_selection(&mut self) {
        if self.template_popup_selection == TemplatePopupSelection::Yes {
            // Apply conventional commits template
            let template = vec![
                "feat: ".to_string(),
                "".to_string(),
                "# Conventional Commits Format:".to_string(),
                "# <type>[optional scope]: <description>".to_string(),
                "#".to_string(),
                "# Types: feat, fix, docs, style, refactor, test, chore".to_string(),
                "# Example: feat(auth): add user login validation".to_string(),
            ];
            self.commit_message = TextArea::new(template);
            // Position cursor after "feat: "
            self.commit_message
                .move_cursor(tui_textarea::CursorMove::Jump(0, 5));
        }
        self.show_template_popup = false;
    }

    /// Load git status for save changes tab (called when tab becomes active)
    pub fn load_save_changes_git_status(&mut self) {
        if !self.save_changes_git_status_loaded {
            self.save_changes_git_status = crate::git::get_git_status().unwrap_or_default();
            self.save_changes_git_status_loaded = true;
        }
    }

    /// Refresh git status for save changes tab (called after staging/unstaging operations)
    pub fn refresh_save_changes_git_status(&mut self) {
        self.save_changes_git_status = crate::git::get_git_status().unwrap_or_default();
        self.save_changes_git_status_loaded = true;
    }

    /// Get cached git status for save changes tab
    pub fn get_save_changes_git_status(&self) -> &[crate::git::GitFileStatus] {
        &self.save_changes_git_status
    }

    /// Mark git status as needing refresh (called when leaving save changes tab)
    pub fn invalidate_save_changes_git_status(&mut self) {
        self.save_changes_git_status_loaded = false;
    }

    /// Load git status for files tab (called when tab becomes active)
    pub fn load_status_git_status(&mut self) {
        if !self.status_git_status_loaded {
            self.status_git_status = crate::git::get_git_status().unwrap_or_default();
            self.status_git_status_loaded = true;
        }
    }

    /// Get cached git status for files tab
    pub fn get_status_git_status(&self) -> &[crate::git::GitFileStatus] {
        &self.status_git_status
    }

    /// Mark git status as needing refresh (called when leaving files tab)
    pub fn invalidate_status_git_status(&mut self) {
        self.status_git_status_loaded = false;
    }

    /// Refresh remote status for update tab
    pub fn refresh_update_remote_status(&mut self) {
        match crate::git::refresh_remote_status() {
            Ok((remote_status, sync_operation)) => {
                self.update_remote_status = Some(remote_status);
                self.add_sync_operation(sync_operation);
            }
            Err(e) => {
                // Show user-friendly error popup
                self.show_error(
                    "Refresh Failed",
                    &format!("Failed to refresh repository status:\n\n{}", e),
                );

                // Also add to sync operations log for debugging
                let error_operation = crate::git::SyncOperation {
                    operation_type: crate::git::SyncOperationType::Refresh,
                    status: crate::git::OperationStatus::Error,
                    message: format!("Failed to refresh: {}", e),
                    timestamp: std::time::SystemTime::now(),
                };
                self.add_sync_operation(error_operation);
            }
        }
    }

    /// Perform pull operation
    pub fn perform_pull(&mut self) {
        match crate::git::pull_origin(self.pull_rebase) {
            Ok(sync_operation) => {
                self.add_sync_operation(sync_operation);
                // Refresh remote status after pull
                if let Ok(remote_status) = crate::git::get_remote_status() {
                    self.update_remote_status = Some(remote_status);
                }
            }
            Err(e) => {
                // Show user-friendly error popup
                self.show_error(
                    "Pull Failed",
                    &format!("Failed to pull changes from remote:\n\n{}", e),
                );

                // Also add to sync operations log for debugging
                let error_operation = crate::git::SyncOperation {
                    operation_type: crate::git::SyncOperationType::Pull,
                    status: crate::git::OperationStatus::Error,
                    message: format!("Pull failed: {}", e),
                    timestamp: std::time::SystemTime::now(),
                };
                self.add_sync_operation(error_operation);
            }
        }
    }

    /// Perform push operation
    pub fn perform_push(&mut self) {
        match crate::git::push_origin() {
            Ok(sync_operation) => {
                self.add_sync_operation(sync_operation);
                // Refresh remote status after push
                if let Ok(remote_status) = crate::git::get_remote_status() {
                    self.update_remote_status = Some(remote_status);
                }
            }
            Err(e) => {
                // Show user-friendly error popup
                self.show_error(
                    "Push Failed",
                    &format!("Failed to push changes to remote:\n\n{}", e),
                );

                // Also add to sync operations log for debugging
                let error_operation = crate::git::SyncOperation {
                    operation_type: crate::git::SyncOperationType::Push,
                    status: crate::git::OperationStatus::Error,
                    message: format!("Push failed: {}", e),
                    timestamp: std::time::SystemTime::now(),
                };
                self.add_sync_operation(error_operation);
            }
        }
    }

    /// Add a sync operation to the recent operations list
    fn add_sync_operation(&mut self, operation: crate::git::SyncOperation) {
        self.update_recent_operations.insert(0, operation);
        // Keep only the last 10 operations
        if self.update_recent_operations.len() > 10 {
            self.update_recent_operations.truncate(10);
        }
    }

    /// Load initial remote status for update tab
    pub fn load_update_remote_status(&mut self) {
        if self.update_remote_status.is_none() {
            if let Ok(remote_status) = crate::git::get_remote_status() {
                self.update_remote_status = Some(remote_status);
            }
        }
    }

    /// Load/refresh update tab data when tab becomes active
    /// This ensures timestamps are current and remote status is loaded
    pub fn load_update_tab(&mut self) {
        // Load remote status if not already loaded
        self.load_update_remote_status();
        // Note: Timestamps are refreshed automatically when rendering since they're calculated
        // relative to the current time each time the UI is drawn
    }

    /// Show an error popup with title and message
    pub fn show_error(&mut self, title: &str, message: &str) {
        self.show_error_popup = true;
        self.error_popup_title = title.to_string();
        self.error_popup_message = message.to_string();
    }

    /// Hide the error popup
    pub fn hide_error(&mut self) {
        self.show_error_popup = false;
        self.error_popup_title.clear();
        self.error_popup_message.clear();
    }
}
