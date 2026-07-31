#![cfg(unix)]

use std::ffi::OsStr;
use std::fs;
use std::os::unix::ffi::OsStrExt;
use std::os::unix::fs::PermissionsExt;
use std::path::{Path, PathBuf};
use std::process::{Command, Output};
use std::sync::atomic::{AtomicU64, Ordering};

const USAGE: &str = "\
Usage: pathbin [OPTIONS] <COMMAND>

Options:
  --homebrew  Scan only Homebrew's common bin and sbin directories

Commands:
  list        List executable binaries in PATH
  where       Show where a command is located
  all         Show all matching binaries with the same name
  shadowed    Show binaries hidden by PATH priority
  duplicates  Show duplicate binary names
  broken      Show broken symlinks and missing PATH entries
  stats       Show PATH binary statistics
  doctor      Diagnose PATH-related problems
";

static NEXT_TEMP_DIR: AtomicU64 = AtomicU64::new(0);

struct Fixture {
    root: PathBuf,
    bin: PathBuf,
    tool: PathBuf,
}

impl Fixture {
    fn new() -> Self {
        let sequence = NEXT_TEMP_DIR.fetch_add(1, Ordering::Relaxed);
        let root = std::env::temp_dir().join(format!(
            "pathbin-cli-contract-{}-{sequence}",
            std::process::id()
        ));
        let bin = root.join("bin");
        fs::create_dir_all(&bin).expect("create PATH directory");
        let tool = bin.join("tool");
        create_executable(&tool);
        Self { root, bin, tool }
    }

    fn run(&self, arguments: &[&str]) -> Output {
        run_pathbin(Some(self.bin.as_os_str()), &self.root, arguments)
    }
}

impl Drop for Fixture {
    fn drop(&mut self) {
        let _ = fs::remove_dir_all(&self.root);
    }
}

fn create_executable(path: &Path) {
    fs::write(path, "#!/bin/sh\n").expect("create executable");
    let mut permissions = fs::metadata(path).expect("read permissions").permissions();
    permissions.set_mode(0o755);
    fs::set_permissions(path, permissions).expect("mark executable");
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

#[cfg(target_os = "linux")]
fn run_pathbin_os(path: &OsStr, current_dir: &Path, arguments: &[&OsStr]) -> Output {
    Command::new(env!("CARGO_BIN_EXE_pathbin"))
        .args(arguments)
        .current_dir(current_dir)
        .env("PATH", path)
        .output()
        .expect("run pathbin with OS-native arguments")
}

fn assert_output(output: &Output, code: i32, stdout: &str, stderr: &str) {
    assert_eq!(output.status.code(), Some(code));
    assert_eq!(String::from_utf8_lossy(&output.stdout), stdout);
    assert_eq!(String::from_utf8_lossy(&output.stderr), stderr);
}

#[test]
fn every_public_subcommand_has_a_success_contract() {
    let fixture = Fixture::new();
    let tool = fixture.tool.display();

    assert_output(&fixture.run(&["list"]), 0, &format!("tool\t{tool}\n"), "");
    assert_output(
        &fixture.run(&["where", "tool"]),
        0,
        &format!("{tool}\n"),
        "",
    );
    assert_output(
        &fixture.run(&["all", "tool"]),
        0,
        &format!("[active] {tool}\n"),
        "",
    );
    assert_output(
        &fixture.run(&["shadowed"]),
        0,
        "No shadowed binaries found.\n",
        "",
    );
    assert_output(
        &fixture.run(&["duplicates"]),
        0,
        "No duplicate command names found.\n",
        "",
    );
    assert_output(
        &fixture.run(&["broken"]),
        0,
        "No broken PATH entries or symlinks found.\n",
        "",
    );

    let stats = fixture.run(&["stats"]);
    assert_eq!(stats.status.code(), Some(0));
    assert!(stats.stderr.is_empty());
    let stats_stdout = String::from_utf8_lossy(&stats.stdout);
    assert!(stats_stdout.contains("PATH entries: 1\n"));
    assert!(stats_stdout.contains("Executable binaries: 1\n"));
    assert!(stats_stdout.contains("Unique command names: 1\n"));

    assert_output(
        &fixture.run(&["doctor"]),
        0,
        "No obvious PATH problems detected.\n",
        "",
    );
}

#[test]
fn help_and_invalid_invocations_have_explicit_streams_and_exit_codes() {
    let fixture = Fixture::new();

    assert_output(&fixture.run(&["--help"]), 0, "", USAGE);
    assert_output(&fixture.run(&["--homebrew", "--help"]), 0, "", USAGE);
    assert_output(&fixture.run(&["--help", "--homebrew"]), 0, "", USAGE);
    assert_output(&fixture.run(&[]), 1, "", USAGE);
    assert_output(
        &fixture.run(&["unknown"]),
        1,
        "",
        &format!("Unknown command: unknown\n{USAGE}"),
    );
    assert_output(
        &fixture.run(&["--homebrew", "--homebrew", "list"]),
        1,
        "",
        &format!("Invalid arguments.\n{USAGE}"),
    );
    assert_output(
        &fixture.run(&["list", "--homebrew", "--homebrew"]),
        1,
        "",
        &format!("Invalid arguments.\n{USAGE}"),
    );
    assert_output(
        &fixture.run(&["where", "--homebrew"]),
        1,
        "",
        &format!("Invalid arguments.\n{USAGE}"),
    );
    assert_output(
        &fixture.run(&["--homebrew", "all"]),
        1,
        "",
        &format!("Invalid arguments.\n{USAGE}"),
    );
}

#[test]
fn every_public_subcommand_rejects_invalid_arity() {
    let fixture = Fixture::new();
    let invalid_invocations: &[&[&str]] = &[
        &["list", "extra"],
        &["where"],
        &["where", "tool", "extra"],
        &["all"],
        &["all", "tool", "extra"],
        &["shadowed", "extra"],
        &["duplicates", "extra"],
        &["broken", "extra"],
        &["stats", "extra"],
        &["doctor", "extra"],
    ];

    for arguments in invalid_invocations {
        assert_output(
            &fixture.run(arguments),
            1,
            "",
            &format!("Invalid arguments.\n{USAGE}"),
        );
    }
}

#[test]
fn lookup_failures_use_stderr_and_exit_one() {
    let fixture = Fixture::new();

    assert_output(
        &fixture.run(&["where", "missing"]),
        1,
        "",
        "Command 'missing' was not found in PATH.\n",
    );
    assert_output(
        &fixture.run(&["all", "missing"]),
        1,
        "",
        "Command 'missing' was not found in PATH.\n",
    );
}

#[test]
fn doctor_exits_one_for_warnings_and_errors() {
    let fixture = Fixture::new();

    assert_output(
        &run_pathbin(None, &fixture.root, &["doctor"]),
        1,
        "[ERROR] PATH is empty or not set.\n\
Doctor summary: 1 issue category/categories detected.\n",
        "",
    );

    let missing = fixture.root.join("missing");
    assert_output(
        &run_pathbin(Some(missing.as_os_str()), &fixture.root, &["doctor"]),
        1,
        "[WARN] PATH contains 1 missing directory/directories.\n\
Doctor summary: 1 issue category/categories detected.\n",
        "",
    );
}

#[test]
fn doctor_exits_zero_for_informational_duplicate_only() {
    let fixture = Fixture::new();
    let second_bin = fixture.root.join("second-bin");
    fs::create_dir(&second_bin).expect("create second PATH directory");
    let second_tool = second_bin.join("tool");
    fs::write(&second_tool, "#!/bin/sh\n").expect("create duplicate executable");
    let mut permissions = fs::metadata(&second_tool)
        .expect("read duplicate permissions")
        .permissions();
    permissions.set_mode(0o755);
    fs::set_permissions(&second_tool, permissions).expect("mark duplicate executable");
    let path =
        std::env::join_paths([fixture.bin.as_path(), second_bin.as_path()]).expect("join PATH");

    assert_output(
        &run_pathbin(Some(path.as_os_str()), &fixture.root, &["doctor"]),
        0,
        "[INFO] Found 1 duplicate command name(s).\n\
Doctor summary: 1 issue category/categories detected.\n",
        "",
    );
}

#[test]
fn escapes_control_characters_from_filesystem_and_arguments() {
    let fixture = Fixture::new();
    let untrusted_name = OsStr::from_bytes(b"bad\n\x1b[31m");
    create_executable(&fixture.bin.join(untrusted_name));

    let list = fixture.run(&["list"]);
    assert_eq!(list.status.code(), Some(0));
    assert!(list.stderr.is_empty());
    assert!(!list.stdout.contains(&0x1b));
    assert_eq!(list.stdout.iter().filter(|byte| **byte == b'\n').count(), 2);
    let list_stdout = String::from_utf8(list.stdout).expect("escaped list output is UTF-8");
    assert!(list_stdout.contains("bad\\n\\u{1b}[31m\t"));

    let lookup = fixture.run(&["where", "missing\n\x1b[31m"]);
    assert_eq!(lookup.status.code(), Some(1));
    assert!(lookup.stdout.is_empty());
    assert!(!lookup.stderr.contains(&0x1b));
    assert_eq!(
        String::from_utf8(lookup.stderr).expect("escaped lookup error is UTF-8"),
        "Command 'missing\\n\\u{1b}[31m' was not found in PATH.\n"
    );

    let unknown = fixture.run(&["unknown\n\x1b[31m"]);
    assert_eq!(unknown.status.code(), Some(1));
    assert!(!unknown.stderr.contains(&0x1b));
    assert!(
        String::from_utf8(unknown.stderr)
            .expect("escaped unknown-command error is UTF-8")
            .starts_with("Unknown command: unknown\\n\\u{1b}[31m\n")
    );
}

#[test]
#[cfg(target_os = "linux")]
fn preserves_and_looks_up_distinct_non_utf8_names() {
    let fixture = Fixture::new();
    let first_name = OsStr::from_bytes(b"tool-\xfe");
    let second_name = OsStr::from_bytes(b"tool-\xff");
    let first_path = fixture.bin.join(first_name);
    let second_path = fixture.bin.join(second_name);
    create_executable(&first_path);
    create_executable(&second_path);

    let list = fixture.run(&["list"]);
    assert_eq!(list.status.code(), Some(0));
    let list_stdout = String::from_utf8(list.stdout).expect("escaped list output is UTF-8");
    assert!(list_stdout.contains("tool-\\xFE\t"));
    assert!(list_stdout.contains("tool-\\xFF\t"));
    assert!(!list_stdout.contains('\u{fffd}'));
    let rendered_path_prefix = format!("{}/tool-", fixture.bin.display());

    let first_lookup = run_pathbin_os(
        fixture.bin.as_os_str(),
        &fixture.root,
        &[OsStr::new("where"), first_name],
    );
    assert_output(
        &first_lookup,
        0,
        &format!("{rendered_path_prefix}\\xFE\n"),
        "",
    );

    let second_lookup = run_pathbin_os(
        fixture.bin.as_os_str(),
        &fixture.root,
        &[OsStr::new("where"), second_name],
    );
    assert_output(
        &second_lookup,
        0,
        &format!("{rendered_path_prefix}\\xFF\n"),
        "",
    );

    assert_output(
        &fixture.run(&["duplicates"]),
        0,
        "No duplicate command names found.\n",
        "",
    );
    let stats = fixture.run(&["stats"]);
    assert_eq!(stats.status.code(), Some(0));
    assert!(
        String::from_utf8(stats.stdout)
            .expect("stats output is UTF-8")
            .contains("Unique command names: 3\n")
    );
}
