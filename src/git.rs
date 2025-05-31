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
    // Use git status --porcelain to get both staged and unstaged changes
    let output = std::process::Command::new("git")
        .arg("status")
        .arg("--porcelain")
        .output()?;

    if !output.status.success() {
        return Err(format!(
            "Failed to get git status: {}",
            String::from_utf8_lossy(&output.stderr)
        )
        .into());
    }

    let mut files = Vec::new();
    let status_output = String::from_utf8_lossy(&output.stdout);

    for line in status_output.lines() {
        if line.len() < 3 {
            continue;
        }

        let index_status = line.chars().nth(0).unwrap_or(' ');
        let worktree_status = line.chars().nth(1).unwrap_or(' ');
        let file_path = &line[3..]; // Skip the two status characters and space

        let path = std::path::PathBuf::from(file_path);

        // Get file size if the file exists
        let file_size = std::fs::metadata(&path).ok().map(|m| m.len());

        // Determine if file is staged (has changes in index)
        let staged = index_status != ' ' && index_status != '?';

        // Determine the primary status to show
        let (status, is_staged) = if staged && worktree_status == ' ' {
            // File is staged and clean in worktree - show as staged
            match index_status {
                'A' => (FileStatusType::Added, true),
                'M' => (FileStatusType::Modified, true),
                'D' => (FileStatusType::Deleted, true),
                'R' => (
                    FileStatusType::Renamed {
                        from: "".to_string(),
                    },
                    true,
                ),
                'T' => (FileStatusType::TypeChange, true),
                _ => (FileStatusType::Modified, true),
            }
        } else if worktree_status != ' ' {
            // File has changes in worktree - show worktree status
            match worktree_status {
                'M' => (FileStatusType::Modified, false),
                'D' => (FileStatusType::Deleted, false),
                '?' => (FileStatusType::Untracked, false),
                _ => (FileStatusType::Modified, false),
            }
        } else if staged {
            // File is staged but we prefer to show it as staged
            match index_status {
                'A' => (FileStatusType::Added, true),
                'M' => (FileStatusType::Modified, true),
                'D' => (FileStatusType::Deleted, true),
                'R' => (
                    FileStatusType::Renamed {
                        from: "".to_string(),
                    },
                    true,
                ),
                'T' => (FileStatusType::TypeChange, true),
                _ => (FileStatusType::Modified, true),
            }
        } else {
            continue; // Skip files with no changes
        };

        files.push(GitFileStatus {
            path,
            status,
            file_size,
            staged: is_staged,
        });
    }

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
    // For now, use git command directly until we can figure out the correct gix API
    let output = std::process::Command::new("git")
        .arg("add")
        .arg(file_path)
        .output()?;

    if !output.status.success() {
        return Err(format!(
            "Failed to stage file: {}",
            String::from_utf8_lossy(&output.stderr)
        )
        .into());
    }

    Ok(())
}

pub fn unstage_file(file_path: &std::path::Path) -> Result<(), Box<dyn std::error::Error>> {
    // For now, use git command directly until we can figure out the correct gix API
    let output = std::process::Command::new("git")
        .arg("reset")
        .arg("HEAD")
        .arg("--")
        .arg(file_path)
        .output()?;

    if !output.status.success() {
        return Err(format!(
            "Failed to unstage file: {}",
            String::from_utf8_lossy(&output.stderr)
        )
        .into());
    }

    Ok(())
}

pub fn commit(message: &str) -> Result<(), Box<dyn std::error::Error>> {
    // For now, use git command directly until we can figure out the correct gix API
    let output = std::process::Command::new("git")
        .arg("commit")
        .arg("-m")
        .arg(message)
        .output()?;

    if !output.status.success() {
        return Err(format!(
            "Failed to create commit: {}",
            String::from_utf8_lossy(&output.stderr)
        )
        .into());
    }

    println!("Created commit successfully");

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
