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
}

#[derive(Debug, Clone, PartialEq)]
pub enum SaveChangesFocus {
    FileList,
    CommitMessage,
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
            save_changes_focus: SaveChangesFocus::FileList,
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
        save_changes_focus: SaveChangesFocus::FileList,
    };
    state.check_git_status();

    // Pass state to TUI
    crate::tui::start_tui(&mut state);
}
