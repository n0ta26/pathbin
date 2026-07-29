#![cfg(unix)]

use std::ffi::{OsStr, OsString};
use std::fs;
use std::os::unix::fs::{PermissionsExt, symlink};
use std::path::{Path, PathBuf};
use std::process::{Command, Output};
use std::sync::atomic::{AtomicU64, Ordering};

static NEXT_TEMP_DIR: AtomicU64 = AtomicU64::new(0);

struct TempDir {
    path: PathBuf,
}

impl TempDir {
    fn new(test_name: &str) -> Self {
        let sequence = NEXT_TEMP_DIR.fetch_add(1, Ordering::Relaxed);
        let path = std::env::temp_dir().join(format!(
            "pathbin-edge-{test_name}-{}-{sequence}",
            std::process::id()
        ));
        fs::create_dir(&path).expect("create test directory");
        Self { path }
    }

    fn child_dir(&self, name: &str) -> PathBuf {
        let path = self.path.join(name);
        fs::create_dir(&path).expect("create child directory");
        path
    }
}

impl Drop for TempDir {
    fn drop(&mut self) {
        let _ = fs::remove_dir_all(&self.path);
    }
}

fn run_pathbin(path: Option<&OsStr>, current_dir: &Path, arguments: &[&str]) -> Output {
    let mut command = Command::new(env!("CARGO_BIN_EXE_pathbin"));
    command.args(arguments).current_dir(current_dir);
    match path {
        Some(value) => {
            command.env("PATH", value);
        }
        None => {
            command.env_remove("PATH");
        }
    }
    command.output().expect("run pathbin")
}

fn assert_success(output: &Output) -> String {
    assert_eq!(output.status.code(), Some(0));
    assert!(output.stderr.is_empty());
    String::from_utf8(output.stdout.clone()).expect("UTF-8 stdout")
}

#[test]
fn unset_path_is_reported_as_empty() {
    let temp = TempDir::new("unset");
    let stdout = assert_success(&run_pathbin(None, &temp.path, &["stats"]));

    assert!(stdout.contains("PATH entries: 0\n"));
    assert!(stdout.contains("Existing directories: 0\n"));
    assert!(stdout.contains("Executable binaries: 0\n"));
}

#[test]
fn empty_path_entry_is_counted_and_scans_the_current_directory() {
    let temp = TempDir::new("empty-entry");
    let stdout = assert_success(&run_pathbin(Some(OsStr::new("")), &temp.path, &["stats"]));

    assert!(stdout.contains("PATH entries: 1\n"));
    assert!(stdout.contains("Existing directories: 1\n"));
    assert!(stdout.contains("Empty PATH entries: 1\n"));
}

#[test]
fn missing_and_non_directory_entries_are_reported() {
    let temp = TempDir::new("invalid-entries");
    let missing = temp.path.join("missing");
    let regular_file = temp.path.join("not-a-directory");
    fs::write(&regular_file, "not a directory").expect("create regular file");
    let path = std::env::join_paths([&missing, &regular_file]).expect("join PATH entries");

    let stdout = assert_success(&run_pathbin(Some(&path), &temp.path, &["broken"]));

    assert!(stdout.contains("Missing PATH entries:\n"));
    assert!(stdout.contains(&format!("  {}\n", missing.display())));
    assert!(stdout.contains("Non-directory PATH entries:\n"));
    assert!(stdout.contains(&format!("  {}\n", regular_file.display())));
}

#[test]
fn unreadable_directory_is_reported_when_the_platform_rejects_access() {
    let temp = TempDir::new("unreadable");
    let unreadable = temp.child_dir("unreadable");
    fs::set_permissions(&unreadable, fs::Permissions::from_mode(0o000))
        .expect("remove directory permissions");

    if fs::read_dir(&unreadable).is_ok() {
        fs::set_permissions(&unreadable, fs::Permissions::from_mode(0o700))
            .expect("restore directory permissions");
        return;
    }

    let path = OsString::from(unreadable.as_os_str());
    let stdout = assert_success(&run_pathbin(Some(&path), &temp.path, &["broken"]));
    fs::set_permissions(&unreadable, fs::Permissions::from_mode(0o700))
        .expect("restore directory permissions");

    assert!(stdout.contains("Unreadable PATH directories:\n"));
    assert!(stdout.contains(&format!("  {}\n", unreadable.display())));
}

#[test]
fn broken_symlink_is_reported() {
    let temp = TempDir::new("broken-symlink");
    let bin_dir = temp.child_dir("bin");
    let missing_target = temp.path.join("missing-target");
    let broken_link = bin_dir.join("broken-tool");
    symlink(&missing_target, &broken_link).expect("create broken symlink");
    let path = OsString::from(bin_dir.as_os_str());

    let stdout = assert_success(&run_pathbin(Some(&path), &temp.path, &["broken"]));

    assert!(stdout.contains("Broken symlinks:\n"));
    assert!(stdout.contains(&format!("  {}\n", broken_link.display())));
}
