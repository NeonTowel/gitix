use std::path::PathBuf;

pub struct AppState {
    pub git_enabled: bool,          // Is this a git repo?
    pub show_init_prompt: bool,     // Should we prompt to init?
    pub repo_root: Option<PathBuf>, // Path to repo root if found
    pub current_path: PathBuf,      // The directory the app is running in
                                    // Add other fields as needed (e.g., file list, user config)
}

impl Default for AppState {
    fn default() -> Self {
        let current_path = std::env::current_dir().unwrap_or_else(|_| PathBuf::from("."));
        let mut state = AppState {
            git_enabled: false,
            show_init_prompt: false,
            repo_root: None,
            current_path,
        };
        state.check_git_status();
        state
    }
}

impl AppState {
    pub fn check_git_status(&mut self) {
        match gix::discover(&self.current_path) {
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
        match gix::init(&self.current_path) {
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
    let current_path = std::env::current_dir().unwrap();

    // Initialize app state
    let mut state = AppState {
        git_enabled: false,
        show_init_prompt: false,
        repo_root: None,
        current_path,
    };
    state.check_git_status();

    // Pass state to TUI
    crate::tui::start_tui(&mut state);
}
