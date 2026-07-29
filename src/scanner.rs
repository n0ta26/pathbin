use crate::model::{BinaryEntry, ScanResult};
use std::env;
use std::fs;
use std::path::{Path, PathBuf};

pub fn scan_path() -> ScanResult {
    let mut scan = ScanResult::default();

    let separator = if cfg!(windows) { ';' } else { ':' };
    let raw_path = env::var_os("PATH");

    if let Some(raw_path_text) = raw_path.as_ref().map(|value| value.to_string_lossy()) {
        scan.empty_path_entries = raw_path_text
            .split(separator)
            .filter(|part| part.is_empty())
            .count();
    }

    let path_entries: Vec<PathBuf> = raw_path
        .as_ref()
        .map(|value| env::split_paths(value).collect())
        .unwrap_or_default();
    scan.path_entries_total = path_entries.len();

    for (path_index, raw_entry) in path_entries.into_iter().enumerate() {
        let entry = normalize_path_entry(&raw_entry);

        if !entry.exists() {
            scan.missing_entries.push(entry);
            continue;
        }
        if !entry.is_dir() {
            scan.non_dir_entries.push(entry);
            continue;
        }

        scan.existing_dirs += 1;
        let directory_iter = match fs::read_dir(&entry) {
            Ok(iter) => iter,
            Err(_) => {
                scan.unreadable_entries.push(entry);
                continue;
            }
        };

        for dir_entry in directory_iter.flatten() {
            let candidate = dir_entry.path();
            let symlink_meta = match fs::symlink_metadata(&candidate) {
                Ok(meta) => meta,
                Err(_) => continue,
            };

            if symlink_meta.file_type().is_symlink() && fs::metadata(&candidate).is_err() {
                scan.broken_symlinks.push(candidate);
                continue;
            }

            let metadata = match fs::metadata(&candidate) {
                Ok(meta) => meta,
                Err(_) => continue,
            };

            if !metadata.is_file() || !is_executable(&metadata, &candidate) {
                continue;
            }

            let Some(file_name) = candidate.file_name() else {
                continue;
            };

            scan.binaries.push(BinaryEntry::new(
                file_name.to_string_lossy().to_string(),
                candidate,
                path_index,
            ));
        }
    }

    scan.build_name_index();
    scan
}

fn normalize_path_entry(entry: &Path) -> PathBuf {
    if entry.as_os_str().is_empty() {
        env::current_dir().unwrap_or_else(|_| PathBuf::from("."))
    } else {
        entry.to_path_buf()
    }
}

#[cfg(unix)]
fn is_executable(metadata: &fs::Metadata, _path: &Path) -> bool {
    use std::os::unix::fs::PermissionsExt;
    metadata.permissions().mode() & 0o111 != 0
}

#[cfg(windows)]
fn is_executable(_metadata: &fs::Metadata, path: &Path) -> bool {
    has_windows_executable_extension(path)
}

#[cfg(any(windows, test))]
fn has_windows_executable_extension(path: &Path) -> bool {
    let Some(ext) = path.extension().and_then(|ext| ext.to_str()) else {
        return false;
    };

    matches!(
        ext.to_ascii_lowercase().as_str(),
        "exe" | "cmd" | "bat" | "com" | "ps1"
    )
}

#[cfg(not(any(unix, windows)))]
fn is_executable(metadata: &fs::Metadata, _path: &Path) -> bool {
    metadata.is_file()
}

#[cfg(test)]
mod tests {
    use super::has_windows_executable_extension;
    use std::path::Path;

    #[test]
    fn windows_executable_extensions_are_case_insensitive() {
        for path in ["tool.exe", "tool.CMD", "tool.Bat", "tool.cOm", "tool.PS1"] {
            assert!(has_windows_executable_extension(Path::new(path)), "{path}");
        }
    }

    #[test]
    fn windows_executable_detection_rejects_unknown_or_missing_extensions() {
        for path in ["tool", "tool.sh", "tool.txt", "tool.exe.backup"] {
            assert!(!has_windows_executable_extension(Path::new(path)), "{path}");
        }
    }
}
