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

fn assert_output(output: &Output, code: i32, stdout: &str, stderr: &str) {
    assert_eq!(output.status.code(), Some(code));
    assert_eq!(String::from_utf8_lossy(&output.stdout), stdout);
    assert_eq!(String::from_utf8_lossy(&output.stderr), stderr);
}

#[test]
fn ineligible_entries_are_excluded_from_command_results() {
    let temp = TempDir::new("ineligible-entries");
    let bin_dir = temp.child_dir("bin");
    let non_executable = bin_dir.join("not-executable");
    fs::write(&non_executable, "not executable\n").expect("create non-executable file");
    let nested_dir = bin_dir.join("nested");
    fs::create_dir(&nested_dir).expect("create nested directory");
    let nested_tool = nested_dir.join("nested-tool");
    fs::write(&nested_tool, "#!/bin/sh\n").expect("create nested executable");
    fs::set_permissions(&nested_tool, fs::Permissions::from_mode(0o755))
        .expect("mark nested file executable");
    let path = OsString::from(bin_dir.as_os_str());

    assert_output(
        &run_pathbin(Some(&path), &temp.path, &["list"]),
        0,
        "No executable binaries found in PATH.\n",
        "",
    );

    for command_name in ["not-executable", "nested-tool"] {
        for command in ["where", "all"] {
            assert_output(
                &run_pathbin(Some(&path), &temp.path, &[command, command_name]),
                1,
                "",
                &format!("Command '{command_name}' was not found in PATH.\n"),
            );
        }
    }

    assert_output(
        &run_pathbin(Some(&path), &temp.path, &["shadowed"]),
        0,
        "No shadowed binaries found.\n",
        "",
    );
    assert_output(
        &run_pathbin(Some(&path), &temp.path, &["duplicates"]),
        0,
        "No duplicate command names found.\n",
        "",
    );
    assert_output(
        &run_pathbin(Some(&path), &temp.path, &["stats"]),
        0,
        "PATH entries: 1\n\
         Existing directories: 1\n\
         Missing PATH entries: 0\n\
         Non-directory entries: 0\n\
         Unreadable PATH directories: 0\n\
         Empty PATH entries: 0\n\
         Executable binaries: 0\n\
         Unique command names: 0\n\
         Duplicate command names: 0\n\
         Shadowed binaries: 0\n\
         Broken symlinks: 0\n",
        "",
    );
    assert_output(
        &run_pathbin(Some(&path), &temp.path, &["doctor"]),
        0,
        "No obvious PATH problems detected.\n",
        "",
    );
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

#[test]
fn invalid_path_categories_are_consistent_across_diagnostic_commands() {
    let temp = TempDir::new("combined-invalid-categories");
    let missing = temp.path.join("missing");
    let non_directory = temp.path.join("not-a-directory");
    fs::write(&non_directory, "not a directory\n").expect("create non-directory entry");

    let bin_dir = temp.child_dir("bin");
    let missing_target = temp.path.join("missing-target");
    let broken_link = bin_dir.join("broken-tool");
    symlink(&missing_target, &broken_link).expect("create broken symlink");

    let unreadable = temp.child_dir("unreadable");
    fs::set_permissions(&unreadable, fs::Permissions::from_mode(0o000))
        .expect("remove directory permissions");
    let include_unreadable = fs::read_dir(&unreadable).is_err();

    let mut entries: Vec<&Path> = vec![
        Path::new(""),
        missing.as_path(),
        non_directory.as_path(),
        bin_dir.as_path(),
    ];
    if include_unreadable {
        entries.push(unreadable.as_path());
    }
    let path = std::env::join_paths(entries).expect("join invalid PATH entries");

    let mut broken_stdout = format!(
        "Missing PATH entries:\n  {}\n\
         Non-directory PATH entries:\n  {}\n",
        missing.display(),
        non_directory.display()
    );
    if include_unreadable {
        broken_stdout.push_str(&format!(
            "Unreadable PATH directories:\n  {}\n",
            unreadable.display()
        ));
    }
    broken_stdout.push_str(&format!("Broken symlinks:\n  {}\n", broken_link.display()));
    assert_output(
        &run_pathbin(Some(&path), &temp.path, &["broken"]),
        0,
        &broken_stdout,
        "",
    );

    let path_entries = if include_unreadable { 5 } else { 4 };
    let existing_directories = if include_unreadable { 3 } else { 2 };
    let unreadable_directories = usize::from(include_unreadable);
    let stats_stdout = format!(
        "PATH entries: {path_entries}\n\
         Existing directories: {existing_directories}\n\
         Missing PATH entries: 1\n\
         Non-directory entries: 1\n\
         Unreadable PATH directories: {unreadable_directories}\n\
         Empty PATH entries: 1\n\
         Executable binaries: 0\n\
         Unique command names: 0\n\
         Duplicate command names: 0\n\
         Shadowed binaries: 0\n\
         Broken symlinks: 1\n"
    );
    assert_output(
        &run_pathbin(Some(&path), &temp.path, &["stats"]),
        0,
        &stats_stdout,
        "",
    );

    let mut doctor_stdout =
        "[WARN] PATH contains 1 empty entry/entries (current directory lookup).\n\
                             [WARN] PATH contains 1 missing directory/directories.\n\
                             [WARN] PATH contains 1 non-directory entry/entries.\n"
            .to_string();
    if include_unreadable {
        doctor_stdout.push_str("[WARN] PATH contains 1 unreadable directory/directories.\n");
    }
    doctor_stdout.push_str("[WARN] Found 1 broken symlink(s) in PATH directories.\n");
    let findings = if include_unreadable { 5 } else { 4 };
    doctor_stdout.push_str(&format!(
        "Doctor summary: {findings} issue category/categories detected.\n"
    ));
    assert_output(
        &run_pathbin(Some(&path), &temp.path, &["doctor"]),
        1,
        &doctor_stdout,
        "",
    );

    fs::set_permissions(&unreadable, fs::Permissions::from_mode(0o700))
        .expect("restore directory permissions");
}
