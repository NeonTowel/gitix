use ratatui::widgets::ScrollbarState;
use ratatui::widgets::TableState;
use std::path::PathBuf;
use tui_textarea::TextArea;

pub struct AppState {
    pub git_enabled: bool,              // Is this a git repo?
    pub show_init_prompt: bool,         // Should we prompt to init?
    pub repo_root: Option<PathBuf>,     // Path to repo root if found
    pub root_dir: PathBuf,              // The directory jail root
    pub current_dir: PathBuf,           // The directory currently being browsed
    pub files_selected_row: usize,      // Selected row in files tab
    pub status_table_state: TableState, // Table state for status tab scrolling

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
            status_table_state: TableState::default(),
            save_changes_table_state: TableState::default(),
            staged_files: Vec::new(),
            commit_message: TextArea::new(vec![String::new()]),
            save_changes_focus: SaveChangesFocus::CommitMessage,
            show_commit_help: false,
            help_popup_scroll: 0,
            help_popup_scrollbar_state: ScrollbarState::default(),
            show_template_popup: false,
            template_popup_selection: TemplatePopupSelection::No,
        };
        state.check_git_status();
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
}

pub fn run() {
    // Get the current directory
    let cwd = std::env::current_dir().unwrap();

    // Initialize app state
    let mut state = AppState {
        git_enabled: false,
        show_init_prompt: false,
        repo_root: None,
        root_dir: cwd.clone(),
        current_dir: cwd,
        files_selected_row: 0,
        status_table_state: TableState::default(),
        save_changes_table_state: TableState::default(),
        staged_files: Vec::new(),
        commit_message: TextArea::new(vec![String::new()]),
        save_changes_focus: SaveChangesFocus::CommitMessage,
        show_commit_help: false,
        help_popup_scroll: 0,
        help_popup_scrollbar_state: ScrollbarState::default(),
        show_template_popup: false,
        template_popup_selection: TemplatePopupSelection::No,
    };
    state.check_git_status();

    // Pass state to TUI
    crate::tui::start_tui(&mut state);
}
