use gix::Repository;
use std::path::PathBuf;

#[derive(Debug, Clone)]
pub struct GitFileStatus {
    pub path: PathBuf,
    pub status: FileStatusType,
    pub file_size: Option<u64>,
    pub staged: bool, // Whether the file is staged for commit
}

#[derive(Debug, Clone)]
pub enum FileStatusType {
    Modified,
    Added,
    Deleted,
    Untracked,
    Renamed { from: String },
    TypeChange,
}

impl FileStatusType {
    pub fn as_symbol(&self) -> &'static str {
        match self {
            FileStatusType::Modified => "M",
            FileStatusType::Added => "A",
            FileStatusType::Deleted => "D",
            FileStatusType::Untracked => "?",
            FileStatusType::Renamed { .. } => "R",
            FileStatusType::TypeChange => "T",
        }
    }

    pub fn as_description(&self) -> &'static str {
        match self {
            FileStatusType::Modified => "Modified",
            FileStatusType::Added => "New file",
            FileStatusType::Deleted => "Deleted",
            FileStatusType::Untracked => "Untracked",
            FileStatusType::Renamed { .. } => "Renamed",
            FileStatusType::TypeChange => "Type changed",
        }
    }

    pub fn color(&self) -> ratatui::style::Color {
        match self {
            FileStatusType::Modified => ratatui::style::Color::Yellow,
            FileStatusType::Added => ratatui::style::Color::Green,
            FileStatusType::Deleted => ratatui::style::Color::Red,
            FileStatusType::Untracked => ratatui::style::Color::Cyan,
            FileStatusType::Renamed { .. } => ratatui::style::Color::Blue,
            FileStatusType::TypeChange => ratatui::style::Color::Magenta,
        }
    }
}

pub fn init_repo() -> Result<(), gix::init::Error> {
    gix::init(".")?;
    Ok(())
}

pub fn get_git_status() -> Result<Vec<GitFileStatus>, Box<dyn std::error::Error>> {
    let repo = gix::open(".")?;

    // Check if repository has a worktree
    let worktree_root = repo.work_dir().ok_or("Repository has no worktree")?;

    let mut files = Vec::new();

    // Get the index
    let index = repo.index()?;

    // Load gitignore patterns if available
    let gitignore_patterns = load_gitignore_patterns(worktree_root);

    // Recursively walk through all files in the worktree
    fn walk_dir(
        dir: &std::path::Path,
        worktree_root: &std::path::Path,
        index: &gix::index::File,
        files: &mut Vec<GitFileStatus>,
        gitignore_patterns: &[String],
    ) -> std::io::Result<()> {
        for entry in std::fs::read_dir(dir)? {
            let entry = entry?;
            let path = entry.path();

            // Skip .git directory and target directory entirely
            if let Some(name) = path.file_name().and_then(|n| n.to_str()) {
                if name == ".git" || name == "target" {
                    continue;
                }
            }

            if path.is_dir() {
                // Recursively walk subdirectories
                walk_dir(&path, worktree_root, index, files, gitignore_patterns)?;
            } else if path.is_file() {
                let relative_path = path.strip_prefix(worktree_root).unwrap_or(&path);
                let path_str = relative_path.to_string_lossy();

                // Convert to forward slashes for git compatibility
                let git_path = path_str.replace('\\', "/");

                // Skip if file matches gitignore patterns
                if is_ignored(&git_path, gitignore_patterns) {
                    continue;
                }

                // Skip common ignore patterns
                if git_path.ends_with(".tmp")
                    || git_path.ends_with("~")
                    || git_path.starts_with("target/")
                    || git_path.contains("/.git/")
                {
                    continue;
                }

                let path_bstr = gix::bstr::BStr::new(git_path.as_bytes());

                // Get file size
                let file_size = std::fs::metadata(&path).ok().map(|m| m.len());

                // Check if file is tracked in the index
                let is_tracked = index.entry_by_path(path_bstr).is_some();

                if !is_tracked {
                    // File is untracked
                    files.push(GitFileStatus {
                        path: std::path::PathBuf::from(relative_path),
                        status: FileStatusType::Untracked,
                        file_size,
                        staged: false,
                    });
                } else {
                    // For tracked files, do a modification check
                    if let Ok(metadata) = std::fs::metadata(&path) {
                        if let Ok(modified) = metadata.modified() {
                            if let Some(entry) = index.entry_by_path(path_bstr) {
                                let entry_mtime = entry.stat.mtime.secs;
                                let file_mtime = modified
                                    .duration_since(std::time::UNIX_EPOCH)
                                    .unwrap_or_default()
                                    .as_secs()
                                    as u32;

                                // Check if file is newer than index entry
                                if file_mtime > entry_mtime {
                                    files.push(GitFileStatus {
                                        path: std::path::PathBuf::from(relative_path),
                                        status: FileStatusType::Modified,
                                        file_size,
                                        staged: false,
                                    });
                                }
                            }
                        }
                    }
                }
            }
        }
        Ok(())
    }

    // Start the recursive walk from the worktree root
    walk_dir(
        worktree_root,
        worktree_root,
        &index,
        &mut files,
        &gitignore_patterns,
    )
    .ok();

    Ok(files)
}

// Helper function to load .gitignore patterns
fn load_gitignore_patterns(worktree_root: &std::path::Path) -> Vec<String> {
    let mut patterns = Vec::new();

    // Add common patterns that should always be ignored
    patterns.push("target/".to_string());
    patterns.push("*.tmp".to_string());
    patterns.push("*~".to_string());

    // Try to read .gitignore file
    let gitignore_path = worktree_root.join(".gitignore");
    if let Ok(content) = std::fs::read_to_string(gitignore_path) {
        for line in content.lines() {
            let line = line.trim();
            if !line.is_empty() && !line.starts_with('#') {
                patterns.push(line.to_string());
            }
        }
    }

    patterns
}

// Simple pattern matching for gitignore
fn is_ignored(path: &str, patterns: &[String]) -> bool {
    for pattern in patterns {
        if pattern.ends_with('/') {
            // Directory pattern
            if path.starts_with(pattern) {
                return true;
            }
        } else if pattern.contains('*') {
            // Glob pattern (simple implementation)
            if pattern.starts_with("*.") {
                let ext = &pattern[2..];
                if path.ends_with(ext) {
                    return true;
                }
            }
        } else {
            // Exact match
            if path == pattern || path.ends_with(&format!("/{}", pattern)) {
                return true;
            }
        }
    }
    false
}

pub fn stage_file(file_path: &std::path::Path) -> Result<(), Box<dyn std::error::Error>> {
    // For now, we'll implement a simple version
    // TODO: Implement proper staging using gix
    //println!("Staging file: {:?}", file_path);
    Ok(())
}

pub fn unstage_file(file_path: &std::path::Path) -> Result<(), Box<dyn std::error::Error>> {
    // For now, we'll implement a simple version
    // TODO: Implement proper unstaging using gix
    //println!("Unstaging file: {:?}", file_path);
    Ok(())
}

pub fn commit(message: &str) -> Result<(), Box<dyn std::error::Error>> {
    // For now, we'll implement a simple version
    // TODO: Implement proper commit creation using gix
    //println!("Creating commit with message: {}", message);
    Ok(())
}

pub fn status() -> Result<Vec<GitFileStatus>, Box<dyn std::error::Error>> {
    get_git_status()
}

pub fn push() -> Result<(), Box<dyn std::error::Error>> {
    // TODO: Implement git push logic
    Ok(())
}

pub fn pull_rebase() -> Result<(), Box<dyn std::error::Error>> {
    // TODO: Implement git pull --rebase logic
    Ok(())
}

// Helper function to format file size
pub fn format_file_size(size: Option<u64>) -> String {
    match size {
        Some(bytes) => {
            if bytes < 1024 {
                format!("{} B", bytes)
            } else if bytes < 1024 * 1024 {
                format!("{:.1} KB", bytes as f64 / 1024.0)
            } else if bytes < 1024 * 1024 * 1024 {
                format!("{:.1} MB", bytes as f64 / (1024.0 * 1024.0))
            } else {
                format!("{:.1} GB", bytes as f64 / (1024.0 * 1024.0 * 1024.0))
            }
        }
        None => "-".to_string(),
    }
}
