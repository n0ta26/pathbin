#![cfg(unix)]

use std::ffi::OsString;
use std::fs;
use std::os::unix::fs::PermissionsExt;
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
            "pathbin-{test_name}-{}-{sequence}",
            std::process::id()
        ));
        fs::create_dir(&path).expect("create test directory");
        Self { path }
    }

    fn child(&self, name: &str) -> PathBuf {
        let path = self.path.join(name);
        fs::create_dir(&path).expect("create PATH directory");
        path
    }
}

impl Drop for TempDir {
    fn drop(&mut self) {
        let _ = fs::remove_dir_all(&self.path);
    }
}

fn create_executable(directory: &Path, name: &str) -> PathBuf {
    let path = directory.join(name);
    fs::write(&path, "#!/bin/sh\n").expect("create executable");
    let mut permissions = fs::metadata(&path).expect("read permissions").permissions();
    permissions.set_mode(0o755);
    fs::set_permissions(&path, permissions).expect("mark executable");
    path
}

fn path_value(entries: &[&Path]) -> OsString {
    std::env::join_paths(entries).expect("join PATH entries")
}

fn run_pathbin(path: &OsString, arguments: &[&str]) -> Output {
    Command::new(env!("CARGO_BIN_EXE_pathbin"))
        .args(arguments)
        .env("PATH", path)
        .output()
        .expect("run pathbin")
}

fn assert_success(output: &Output, expected_stdout: &str) {
    assert_eq!(output.status.code(), Some(0));
    assert_eq!(String::from_utf8_lossy(&output.stdout), expected_stdout);
    assert!(output.stderr.is_empty());
}

#[test]
fn path_order_selects_active_and_classifies_duplicates() {
    let temp = TempDir::new("precedence");
    let first_dir = temp.child("first");
    let second_dir = temp.child("second");
    let first_tool = create_executable(&first_dir, "tool");
    let second_tool = create_executable(&second_dir, "tool");
    let path = path_value(&[&first_dir, &second_dir]);

    assert_success(
        &run_pathbin(&path, &["where", "tool"]),
        &format!("{}\n", first_tool.display()),
    );
    assert_success(
        &run_pathbin(&path, &["all", "tool"]),
        &format!(
            "[active] {}\n[shadowed] {}\n",
            first_tool.display(),
            second_tool.display()
        ),
    );
    assert_success(&run_pathbin(&path, &["duplicates"]), "tool\t2\n");
    assert_success(
        &run_pathbin(&path, &["shadowed"]),
        &format!("tool\n  {}\n", second_tool.display()),
    );

    let stats = run_pathbin(&path, &["stats"]);
    assert_eq!(stats.status.code(), Some(0));
    let stdout = String::from_utf8_lossy(&stats.stdout);
    assert!(stdout.contains("Duplicate command names: 1\n"));
    assert!(stdout.contains("Shadowed binaries: 1\n"));
    assert!(stats.stderr.is_empty());
}

#[test]
fn repeated_path_directory_reports_each_binary_once() {
    let temp = TempDir::new("repeated-directory");
    let bin_dir = temp.child("bin");
    let tool = create_executable(&bin_dir, "tool");
    let path = path_value(&[&bin_dir, &bin_dir]);

    assert_success(
        &run_pathbin(&path, &["where", "tool"]),
        &format!("{}\n", tool.display()),
    );
    assert_success(
        &run_pathbin(&path, &["list"]),
        &format!("tool\t{}\n", tool.display()),
    );
    assert_success(
        &run_pathbin(&path, &["all", "tool"]),
        &format!("[active] {}\n", tool.display()),
    );
    assert_success(
        &run_pathbin(&path, &["duplicates"]),
        "No duplicate command names found.\n",
    );
    assert_success(
        &run_pathbin(&path, &["shadowed"]),
        "No shadowed binaries found.\n",
    );
    assert_success(
        &run_pathbin(&path, &["stats"]),
        "PATH entries: 2\n\
         Existing directories: 2\n\
         Missing PATH entries: 0\n\
         Non-directory entries: 0\n\
         Unreadable PATH directories: 0\n\
         Empty PATH entries: 0\n\
         Executable binaries: 1\n\
         Unique command names: 1\n\
         Duplicate command names: 0\n\
         Shadowed binaries: 0\n\
         Broken symlinks: 0\n",
    );
    assert_success(
        &run_pathbin(&path, &["doctor"]),
        "No obvious PATH problems detected.\n",
    );
}
