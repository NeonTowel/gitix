use std::fs;
use std::path::PathBuf;
use std::time::{SystemTime, UNIX_EPOCH};

#[cfg(unix)]
fn get_permissions(metadata: &std::fs::Metadata) -> u32 {
    use std::os::unix::fs::PermissionsExt;
    metadata.permissions().mode()
}

#[cfg(windows)]
fn get_permissions(metadata: &std::fs::Metadata) -> u32 {
    use std::os::windows::fs::MetadataExt;
    metadata.file_attributes()
}

#[cfg(not(any(unix, windows)))]
fn get_permissions(_metadata: &std::fs::Metadata) -> u32 {
    0
}

pub struct FileEntry {
    pub name: String,
    pub size: u64,
    pub permissions: u32,
    pub modified: u64,
    pub is_dir: bool,
    pub git_status: Option<crate::git::FileStatusType>,
}

pub fn list_files(dir: &PathBuf, add_parent: bool) -> Vec<FileEntry> {
    let mut entries = Vec::new();
    if add_parent {
        entries.push(FileEntry {
            name: "..".to_string(),
            size: 0,
            permissions: 0,
            modified: 0,
            is_dir: true,
            git_status: None,
        });
    }
    if let Ok(read_dir) = fs::read_dir(dir) {
        for entry in read_dir.flatten() {
            if let Ok(metadata) = entry.metadata() {
                let name = entry.file_name().to_string_lossy().to_string();
                let size = metadata.len();
                let permissions = get_permissions(&metadata);
                let modified = metadata
                    .modified()
                    .ok()
                    .and_then(|m| m.duration_since(UNIX_EPOCH).ok())
                    .map(|d| d.as_secs())
                    .unwrap_or(0);
                let is_dir = metadata.is_dir();
                entries.push(FileEntry {
                    name,
                    size,
                    permissions,
                    modified,
                    is_dir,
                    git_status: None,
                });
            }
        }
    }
    entries
}

/// Enhanced version that includes git status information
pub fn list_files_with_git_status(
    dir: &PathBuf,
    add_parent: bool,
    git_status: &[crate::git::GitFileStatus],
) -> Vec<FileEntry> {
    let mut entries = list_files(dir, add_parent);

    // Create a map of git status by file path for quick lookup
    let mut git_status_map = std::collections::HashMap::new();

    // Get the repository root by finding the .git directory
    let repo_root = find_git_root(dir).unwrap_or_else(|| dir.clone());

    for git_file in git_status {
        // Git status paths are relative to the repository root
        // We need to check if the file would be visible in the current directory

        if git_file.path.is_relative() {
            // Convert git file path to absolute by joining with repo root
            let absolute_git_path = repo_root.join(&git_file.path);

            // Check if this file is directly in the current directory
            if let Some(git_parent) = absolute_git_path.parent() {
                if git_parent == dir {
                    if let Some(file_name) = absolute_git_path.file_name() {
                        git_status_map.insert(
                            file_name.to_string_lossy().to_string(),
                            git_file.status.clone(),
                        );
                    }
                }
            }
        } else {
            // Handle absolute paths (fallback)
            if let Some(git_parent) = git_file.path.parent() {
                if git_parent == dir {
                    if let Some(file_name) = git_file.path.file_name() {
                        git_status_map.insert(
                            file_name.to_string_lossy().to_string(),
                            git_file.status.clone(),
                        );
                    }
                }
            }
        }
    }

    // Update entries with git status information
    for entry in &mut entries {
        if !entry.is_dir && entry.name != ".." {
            entry.git_status = git_status_map.get(&entry.name).cloned();
        }
    }

    entries
}

/// Find the git repository root by looking for .git directory
fn find_git_root(start_dir: &PathBuf) -> Option<PathBuf> {
    let mut current = start_dir.clone();
    loop {
        let git_dir = current.join(".git");
        if git_dir.exists() {
            return Some(current);
        }

        match current.parent() {
            Some(parent) => current = parent.to_path_buf(),
            None => return None,
        }
    }
}
