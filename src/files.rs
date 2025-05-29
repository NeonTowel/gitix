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
                });
            }
        }
    }
    entries
}
